use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vn_cli::{
    Field, FieldKind, Operation, OperationSpec, Parameters, PreparedOperation, Preview, Progress,
    RunReport, Value,
};

use super::mjo::{MjoDocument, MjoVersion};
use super::mjo_bytecode::{parse_payload, BytecodeProgram};
use super::mjo_rebuild_clean::rebuild_payload;
use super::mjo_text_clean::{self as mjo_text, ScriptTextRecord};

const MANIFEST_NAME: &str = "_katahane_text_manifest.json";
const MANIFEST_FORMAT: &str = "katahane-mjo-text-v9";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TextManifest {
    format: String,
    version: u32,
    files: BTreeMap<String, FileBinding>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileBinding {
    source_sha256: String,
    translation_file: String,
    records: Vec<RecordBinding>,
    choice_group_count: usize,
    choice_option_count: usize,
    dynamic_instruction_count: usize,
    dynamic_append_instruction_count: usize,
    skipped_dynamic_offsets: Vec<usize>,
    skipped_unreachable_offsets: Vec<usize>,
    skipped_blank_offsets: Vec<usize>,
    removed_instruction_offsets: BTreeSet<usize>,
    removed_linebreak_count: usize,
    removed_ruby_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RecordBinding {
    kind: String,
    offset: usize,
    fragment_offsets: Vec<usize>,
    native_events: Vec<NativeEventBinding>,
    opcode: u16,
    has_name: bool,
    source_name: Option<String>,
    source_text: String,
    source_fragments: Vec<NativeFragmentBinding>,
    choice_quote_wrapped: bool,
    choice_control: Option<ChoiceControlBinding>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct NativeFragmentBinding {
    offset: usize,
    opcode: u16,
    source_text: String,
    display_text: String,
    source_bytes_hex: String,
    physical_newline_positions: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct NativeEventBinding {
    before_fragment_index: usize,
    offset: usize,
    opcode: u16,
    kind: String,
    source_text: Option<String>,
    source_bytes_hex: Option<String>,
    related_offset: Option<usize>,
    visible_token: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ChoiceControlBinding {
    call_offset: usize,
    callee_hash: u32,
    argument_count: u16,
    option_index: u16,
    source_option_offsets: Vec<usize>,
    flow_edges: Vec<ChoiceFlowEdgeBinding>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ChoiceFlowEdgeBinding {
    instruction_offset: usize,
    opcode: u16,
    target_offset: usize,
}

struct ExportFile {
    relative_path: PathBuf,
    contents: Vec<u8>,
}

pub struct ExportText;

impl Operation for ExportText {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "extract-text",
            "提取 MJO 文本",
            vec![
                Field::new("source", "解包后的原始 MJO 目录", FieldKind::Path).required(),
                Field::new("output", "翻译 JSON 输出目录", FieldKind::Path).required(),
            ],
        );
        spec.primary = false;
        spec.description =
            "按 VM 显示关系合并对白并单列选项；换行和注音保留在主记录中，注回时删除对应指令。"
                .into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(source) = paths.first() {
            parameters.set("source", Value::Path(source.clone()));
            if parameters.get("output").is_none() {
                if let Some(parent) = source.parent() {
                    parameters.set("output", Value::Path(parent.join("translations")));
                }
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let source = fs::canonicalize(parameters.path("source")?)?;
        if !source.is_dir() {
            return Err("MJO 文本输入必须是解包后的目录".into());
        }
        let mut source_files = BTreeMap::new();
        super::collect_member_files(&source, &source, &mut source_files)?;

        let mut outputs = Vec::new();
        let mut bindings = BTreeMap::new();
        let mut script_count = 0usize;
        let mut text_count = 0usize;
        let mut dynamic_count = 0usize;
        let mut dynamic_append_count = 0usize;
        let mut skipped_count = 0usize;
        let mut unreachable_count = 0usize;
        let mut protected_marker_count = 0usize;
        let mut blank_count = 0usize;
        let mut choice_group_count = 0usize;
        let mut choice_option_count = 0usize;

        for (relative_path, original) in &source_files {
            if !is_mjo_path(relative_path) {
                continue;
            }
            script_count += 1;
            let source_key = path_key(relative_path);
            let translation_file = format!("{source_key}.json");
            let document = parse_mjo(&source_key, original)?;
            let program = parse_program(&source_key, &document)?;
            let report = mjo_text::extract_records_with_report(document.payload(), &program)
                .map_err(|error| format!("无法提取 {source_key}: {error}"))?;
            let entries = mjo_text::translation_entries(&report.records);
            let json = vn_text::write_template(&entries)
                .map_err(|error| format!("无法序列化 {source_key} 的翻译 JSON: {error}"))?;
            dynamic_count += report.dynamic_instruction_count;
            dynamic_append_count += report.dynamic_append_instruction_count;
            skipped_count += report.skipped_dynamic_offsets.len();
            unreachable_count += report.skipped_unreachable_offsets.len();
            protected_marker_count += report.native_event_count;
            blank_count += report.skipped_blank_offsets.len();
            text_count += report
                .records
                .iter()
                .filter(|record| record.kind == mjo_text::RecordKind::Display)
                .count();
            choice_group_count += report.choice_group_count;
            choice_option_count += report.choice_option_count;

            let display_bindings = record_bindings(&report.records);
            bindings.insert(
                source_key.clone(),
                FileBinding {
                    source_sha256: sha256_hex(original),
                    translation_file: translation_file.clone(),
                    records: display_bindings,
                    choice_group_count: report.choice_group_count,
                    choice_option_count: report.choice_option_count,
                    dynamic_instruction_count: report.dynamic_instruction_count,
                    dynamic_append_instruction_count: report.dynamic_append_instruction_count,
                    skipped_dynamic_offsets: report.skipped_dynamic_offsets,
                    skipped_unreachable_offsets: report.skipped_unreachable_offsets,
                    skipped_blank_offsets: report.skipped_blank_offsets,
                    removed_instruction_offsets: report.removed_instruction_offsets,
                    removed_linebreak_count: report.removed_linebreak_count,
                    removed_ruby_count: report.removed_ruby_count,
                },
            );
            outputs.push(ExportFile {
                relative_path: PathBuf::from(translation_file),
                contents: json,
            });
        }
        if script_count == 0 {
            return Err("输入目录中没有 .mjo 文件".into());
        }

        let manifest = TextManifest {
            format: MANIFEST_FORMAT.into(),
            version: 9,
            files: bindings,
        };
        outputs.push(ExportFile {
            relative_path: PathBuf::from(MANIFEST_NAME),
            contents: serde_json::to_vec_pretty(&manifest)
                .map_err(|error| format!("无法序列化翻译源清单: {error}"))?,
        });

        let output = super::resolve_output(parameters.path("output")?)?;
        ensure_no_overlap(
            &source,
            &output,
            "文本输出目录不能位于 MJO 输入目录内，也不能覆盖输入目录",
        )?;
        let original_output = super::snapshot_output(&output)?;
        let overwrite = parameters.flag("overwrite");
        if !matches!(original_output, super::OutputSnapshot::Missing) && !overwrite {
            return Err("输出目录已存在；请选择新目录或显式启用 --overwrite".into());
        }

        let mut warnings = Vec::new();
        if dynamic_count > 0 {
            warnings.push(format!(
                "发现 {dynamic_count} 个 2114 格式/控制指令（其中 {dynamic_append_count} 个 d 指令会追加运行时值）；指令及栈关系原样保留。"
            ));
        }
        if skipped_count > 0 {
            warnings.push(format!(
                "仍有 {skipped_count} 个无法安全绑定动态占位符的 2112 文本保持原样。"
            ));
        }
        if unreachable_count > 0 {
            warnings.push(format!(
                "过滤 {unreachable_count} 个从任何符号入口均不可达的 2112 字符串；源指令保持原样。"
            ));
        }
        if blank_count > 0 {
            warnings.push(format!(
                "忽略 {blank_count} 个仅含空白的静态字符串记录；对应指令注入时保持原样。"
            ));
        }
        if choice_group_count > 0 {
            warnings.push(format!(
                "提取了 {choice_group_count} 个菜单选择组，共 {choice_option_count} 条独立选项；选项数据来自 2049 字符串，控制流及其绑定保存在完整主记录中。"
            ));
        }
        let preview = Preview {
            inputs: vec![source.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "解析并验证每个 MJO 的头部、符号表和完整指令边界".into(),
                "保存包含原始片段、保护控制、注音和菜单分支的完整主记录".into(),
                "把对白与独立选项投影为只含 name/message 的 vn-text JSON".into(),
                "写入 SHA-256 源绑定与指令定位后暂存完整翻译目录".into(),
                "提交完整翻译目录".into(),
            ],
            details: vec![
                format!("MJO 文件: {script_count}"),
                format!("2112 显示记录: {text_count}"),
                format!("菜单选择组: {choice_group_count}"),
                format!("独立选项: {choice_option_count}"),
                format!("2114 格式/动态指令: {dynamic_count}"),
                format!("2114 d 动态追加指令: {dynamic_append_count}"),
                format!("受保护控制/动态占位符: {protected_marker_count}"),
                format!("无法安全并入的动态区文本: {skipped_count}"),
                format!("不可达且未导出的 2112 字符串: {unreachable_count}"),
                format!("仅空白且未导出的记录: {blank_count}"),
            ],
        };
        Ok(Box::new(ExportTextJob {
            preview,
            outputs,
            output,
            original_output,
            overwrite,
            warnings,
            text_count,
        }))
    }
}

struct ExportTextJob {
    preview: Preview,
    outputs: Vec<ExportFile>,
    output: PathBuf,
    original_output: super::OutputSnapshot,
    overwrite: bool,
    warnings: Vec<String>,
    text_count: usize,
}

impl PreparedOperation for ExportTextJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        let parent = self.output.parent().ok_or("翻译输出目录缺少父目录")?;
        let stage = super::StageDirectory::create(
            parent,
            self.output.file_name().ok_or("翻译输出目录名为空")?,
        )?;
        for (index, file) in self.outputs.iter().enumerate() {
            progress.report(&format!(
                "写入翻译产物 {}/{}: {}",
                index + 1,
                self.outputs.len(),
                file.relative_path.display()
            ))?;
            write_staged_file(&stage.path, &file.relative_path, &file.contents)?;
        }
        let warning =
            super::commit_stage(&stage, &self.output, &self.original_output, self.overwrite)?;
        let mut warnings = self.warnings;
        warnings.extend(warning);
        Ok(RunReport {
            summary: format!(
                "MJO 文本提取完成，共 {} 条 2112 显示记录。",
                self.text_count
            ),
            totals: vec![
                ("2112 显示记录".into(), self.text_count as u64),
                ("生成文件".into(), self.outputs.len() as u64),
            ],
            outputs: vec![self.output.clone()],
            warnings,
        })
    }
}

pub struct ImportText;

impl Operation for ImportText {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "inject-text",
            "注入 MJO 文本",
            vec![
                Field::new("source", "原始 MJO 解包目录", FieldKind::Path).required(),
                Field::new(
                    "translations",
                    "extract-text 生成的翻译目录",
                    FieldKind::Path,
                )
                .required(),
                Field::new("output", "完整重建成员目录", FieldKind::Path).required(),
            ],
        );
        spec.primary = false;
        spec.description =
            "校验源 SHA-256 与 vn-text 结构，使用 font98.rs 内嵌映射编码并重定位 MJO。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(source) = paths.first() {
            parameters.set("source", Value::Path(source.clone()));
            if parameters.get("output").is_none() {
                if let Some(parent) = source.parent() {
                    parameters.set("output", Value::Path(parent.join("injected")));
                }
            }
        }
        if let Some(translations) = paths.get(1) {
            parameters.set("translations", Value::Path(translations.clone()));
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let source = fs::canonicalize(parameters.path("source")?)?;
        if !source.is_dir() {
            return Err("MJO 注入源必须是原始解包目录".into());
        }
        let translations = fs::canonicalize(parameters.path("translations")?)?;
        if !translations.is_dir() {
            return Err("翻译输入必须是 extract-text 生成的目录".into());
        }
        ensure_no_overlap(&source, &translations, "原始 MJO 目录和翻译目录必须分开")?;

        let manifest_bytes =
            match read_optional_relative_file(&translations, Path::new(MANIFEST_NAME))? {
                Some(bytes) => bytes,
                None => {
                    let workspace_root = translations.parent().ok_or("翻译目录缺少工作区父目录")?;
                    read_relative_file(
                        workspace_root,
                        &PathBuf::from(".katahane").join("manifest.json"),
                    )?
                }
            };
        let manifest: TextManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|error| format!("v9 翻译源清单无效: {error}"))?;
        if manifest.format != MANIFEST_FORMAT || manifest.version != 9 {
            return Err("翻译源清单格式或版本不受支持".into());
        }

        let mut source_files = BTreeMap::new();
        super::collect_member_files(&source, &source, &mut source_files)?;
        let mjo_paths = source_files
            .keys()
            .filter(|path| is_mjo_path(path))
            .cloned()
            .collect::<Vec<_>>();
        if mjo_paths.is_empty() {
            return Err("输入目录中没有 .mjo 文件".into());
        }
        let source_keys = mjo_paths
            .iter()
            .map(|path| path_key(path))
            .collect::<BTreeSet<_>>();
        let manifest_keys = manifest.files.keys().cloned().collect::<BTreeSet<_>>();
        if source_keys != manifest_keys {
            let missing = source_keys
                .difference(&manifest_keys)
                .cloned()
                .collect::<Vec<_>>();
            let extra = manifest_keys
                .difference(&source_keys)
                .cloned()
                .collect::<Vec<_>>();
            return Err(format!(
                "翻译清单与当前 MJO 文件集合不一致；缺少绑定: [{}]；多余绑定: [{}]",
                missing.join(", "),
                extra.join(", ")
            )
            .into());
        }

        let mut pending_scripts = Vec::new();
        let mut translated_texts = Vec::new();
        let mut total_records = 0usize;
        let mut dynamic_count = 0usize;
        let mut dynamic_append_count = 0usize;
        let mut skipped_count = 0usize;
        let mut unreachable_count = 0usize;
        let mut protected_marker_count = 0usize;
        let mut blank_count = 0usize;
        let mut choice_group_count = 0usize;
        let mut choice_option_count = 0usize;
        let mut v1_count = 0usize;
        let mut x1_count = 0usize;

        for relative_path in &mjo_paths {
            let key = path_key(relative_path);
            let original = source_files
                .get(relative_path)
                .ok_or_else(|| vn_cli::Error(format!("缺少源文件: {key}")))?;
            let binding = manifest
                .files
                .get(&key)
                .ok_or_else(|| vn_cli::Error(format!("翻译清单缺少绑定: {key}")))?;
            if binding.source_sha256 != sha256_hex(original) {
                return Err(
                    format!("源文件哈希与提取时不一致: {key}；请从当前 MJO 重新提取文本").into(),
                );
            }
            let expected_translation_file = format!("{key}.json");
            if binding.translation_file != expected_translation_file {
                return Err(format!("翻译清单中的路径映射被修改: {key}").into());
            }

            let document = parse_mjo(&key, original)?;
            let program = parse_program(&key, &document)?;
            let report = mjo_text::extract_records_with_report(document.payload(), &program)
                .map_err(|error| format!("无法重新解析 {key}: {error}"))?;
            let template_path = PathBuf::from(&binding.translation_file);
            let translation_bytes = read_relative_file(&translations, &template_path)?;
            let supplied = vn_text::read_template(&translation_bytes)
                .map_err(|error| format!("翻译 JSON 无效 ({key}): {error}"))?;
            let expected_entries = mjo_text::translation_entries(&report.records);
            let current_bindings = record_bindings(&report.records);
            if binding.records != current_bindings
                || binding.choice_group_count != report.choice_group_count
                || binding.choice_option_count != report.choice_option_count
                || binding.dynamic_instruction_count != report.dynamic_instruction_count
                || binding.dynamic_append_instruction_count
                    != report.dynamic_append_instruction_count
                || binding.skipped_dynamic_offsets != report.skipped_dynamic_offsets
                || binding.skipped_unreachable_offsets != report.skipped_unreachable_offsets
                || binding.skipped_blank_offsets != report.skipped_blank_offsets
                || binding.removed_instruction_offsets != report.removed_instruction_offsets
                || binding.removed_linebreak_count != report.removed_linebreak_count
                || binding.removed_ruby_count != report.removed_ruby_count
            {
                return Err(format!("源完整主记录与提取清单不一致: {key}").into());
            }
            vn_text::validate_shape(&expected_entries, &supplied)
                .map_err(|error| format!("翻译 JSON 结构不匹配 ({key}): {error}"))?;
            let entries = supplied;
            for (index, (record, entry)) in report.records.iter().zip(&entries).enumerate() {
                let translated = mjo_text::translated_text(record, entry).map_err(|error| {
                    format!("翻译记录无效 ({key}, 第 {} 条): {error}", index + 1)
                })?;
                translated_texts.push(mjo_text::encoding_projection(record, &translated));
            }

            total_records += report.records.len();
            choice_group_count += report.choice_group_count;
            choice_option_count += report.choice_option_count;
            dynamic_count += report.dynamic_instruction_count;
            dynamic_append_count += report.dynamic_append_instruction_count;
            skipped_count += report.skipped_dynamic_offsets.len();
            unreachable_count += report.skipped_unreachable_offsets.len();
            protected_marker_count += report.native_event_count;
            blank_count += report.skipped_blank_offsets.len();
            match document.version {
                MjoVersion::V1 => v1_count += 1,
                MjoVersion::X1 => x1_count += 1,
            }
            pending_scripts.push(PendingScript {
                key,
                relative_path: relative_path.clone(),
                original: original.clone(),
                document,
                program,
                report,
                entries,
            });
        }

        let encoder = CarrierEncoder::build(&translated_texts)?;
        for script in &pending_scripts {
            let edits = mjo_text::build_edits(
                script.document.payload(),
                &script.program,
                &script.report,
                &script.entries,
                |text| encoder.encode(text),
            )
            .map_err(|error| format!("无法编码 {}: {error}", script.key))?;
            let rebuilt = rebuild_payload(
                script.document.payload(),
                &script.program,
                &edits.replacement_operands,
                &edits.removed_instruction_offsets,
                &script.document.symbols,
                script.document.entrypoint,
            )
            .map_err(|error| format!("无法重建 {}: {error}", script.key))?;
            let rebuilt_file = script
                .document
                .rebuild_with_payload(
                    &script.original,
                    &rebuilt.payload,
                    &rebuilt.symbols,
                    rebuilt.entrypoint,
                )
                .map_err(|error| format!("无法封装重建后的 {}: {error}", script.key))?;
            source_files.insert(script.relative_path.clone(), rebuilt_file);
            // Consume the map size as a useful prepared-operation detail and keep
            // its checked relocations visibly tied to this script.
            let _relocated_boundaries = rebuilt.offset_map.len();
        }

        let output = super::resolve_output(parameters.path("output")?)?;
        ensure_no_overlap(
            &source,
            &output,
            "MJO 输出目录不能覆盖或嵌套在原始输入目录中",
        )?;
        ensure_no_overlap(
            &translations,
            &output,
            "MJO 输出目录不能覆盖或嵌套在翻译输入目录中",
        )?;
        let original_output = super::snapshot_output(&output)?;
        let overwrite = parameters.flag("overwrite");
        if !matches!(original_output, super::OutputSnapshot::Missing) && !overwrite {
            return Err("输出目录已存在；请选择新目录或显式启用 --overwrite".into());
        }

        let output_bytes = source_files.values().map(Vec::len).sum::<usize>();
        let preview = Preview {
            inputs: vec![source.clone(), translations.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "验证 manifest、源 SHA-256、文件映射和 vn-text 条目形状".into(),
                "汇总所有译文的 CP932 映射与字体槽位冲突".into(),
                "更新长度前缀、符号/入口偏移和分支/跳转表 rel32".into(),
                "保留未编辑 MJO 指令与非 MJO 成员，并暂存完整目录".into(),
            ],
            details: vec![
                format!(
                    "MJO 文件: {}（X1 {x1_count} / V1 {v1_count}）",
                    mjo_paths.len()
                ),
                format!("可编辑记录: {total_records}"),
                format!("菜单选择组: {choice_group_count}"),
                format!("独立选项: {choice_option_count}"),
                format!("保留的 2114 格式/动态指令: {dynamic_count}"),
                format!("其中追加运行时值的 d 指令: {dynamic_append_count}"),
                format!("受保护控制/动态占位符: {protected_marker_count}"),
                format!("无法安全并入的动态区文本: {skipped_count}"),
                format!("不可达且未导出的 2112 字符串: {unreachable_count}"),
                format!("空白记录保持原样: {blank_count}"),
                format!("重建成员总字节: {output_bytes}"),
            ],
        };
        let mut warnings = Vec::new();
        if dynamic_count > 0 {
            warnings.push(format!(
                "{dynamic_count} 个 2114 指令保持原样，其中 {dynamic_append_count} 个 d 指令会追加运行时值。"
            ));
        }
        if skipped_count > 0 {
            warnings.push(format!(
                "{skipped_count} 个无法安全绑定动态占位符的 2112 片段保持原样。"
            ));
        }
        if blank_count > 0 {
            warnings.push(format!(
                "{blank_count} 个仅含空白的字符串记录没有导出，源指令保持原样。"
            ));
        }
        Ok(Box::new(ImportTextJob {
            preview,
            files: source_files,
            output,
            original_output,
            overwrite,
            warnings,
            total_records,
        }))
    }
}

struct PendingScript {
    key: String,
    relative_path: PathBuf,
    original: Vec<u8>,
    document: MjoDocument,
    program: BytecodeProgram,
    report: mjo_text::ExtractionReport,
    entries: Vec<vn_text::Entry>,
}

struct ImportTextJob {
    preview: Preview,
    files: BTreeMap<PathBuf, Vec<u8>>,
    output: PathBuf,
    original_output: super::OutputSnapshot,
    overwrite: bool,
    warnings: Vec<String>,
    total_records: usize,
}

impl PreparedOperation for ImportTextJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        let parent = self.output.parent().ok_or("重建成员目录缺少父目录")?;
        let stage = super::StageDirectory::create(
            parent,
            self.output.file_name().ok_or("重建成员目录名为空")?,
        )?;
        for (index, (relative_path, contents)) in self.files.iter().enumerate() {
            progress.report(&format!(
                "写入重建成员 {}/{}: {}",
                index + 1,
                self.files.len(),
                relative_path.display()
            ))?;
            write_staged_file(&stage.path, relative_path, contents)?;
        }
        let warning =
            super::commit_stage(&stage, &self.output, &self.original_output, self.overwrite)?;
        let mut warnings = self.warnings;
        warnings.extend(warning);
        Ok(RunReport {
            summary: format!("MJO 文本注入完成，共重建 {} 条记录。", self.total_records),
            totals: vec![
                ("成员文件".into(), self.files.len() as u64),
                ("注入记录".into(), self.total_records as u64),
            ],
            outputs: vec![self.output.clone()],
            warnings,
        })
    }
}

#[derive(Clone, Debug)]
struct CarrierEncoder {
    bytes_by_character: HashMap<char, Vec<u8>>,
}

impl CarrierEncoder {
    fn build(texts: &[String]) -> vn_cli::Result<Self> {
        let raw: BTreeMap<String, String> =
            serde_json::from_str(vn_font::font_98::EMBEDDED_SUBSTITUTIONS)
                .map_err(|error| format!("font98.rs 内嵌 subs_cn_jp.json 无法解析: {error}"))?;
        let mut substitutions = HashMap::with_capacity(raw.len());
        for (source, carrier) in raw {
            let source_chars = source.chars().collect::<Vec<_>>();
            let carrier_chars = carrier.chars().collect::<Vec<_>>();
            if source_chars.len() != 1 || carrier_chars.len() != 1 {
                return Err("font98.rs 内嵌 subs_cn_jp.json 必须每侧恰好一个字符".into());
            }
            substitutions.insert(source_chars[0], carrier_chars[0]);
        }

        let characters = texts
            .iter()
            .flat_map(|text| text.chars())
            .collect::<BTreeSet<_>>();
        let mut bytes_by_character = HashMap::with_capacity(characters.len());
        let mut slots = BTreeMap::<[u8; 2], BTreeSet<char>>::new();
        let mut unmapped = BTreeSet::new();

        for character in characters {
            if character == '\0'
                || (character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
            {
                unmapped.insert(character);
                continue;
            }
            if matches!(character, '\n' | '\r' | '\t') {
                bytes_by_character.insert(character, vec![character as u8]);
                continue;
            }
            if character.is_ascii() {
                bytes_by_character.insert(character, vec![character as u8]);
                continue;
            }

            // Preserve native CP932 first. The embedded table also contains
            // common kanji that can appear in Japanese source text; applying a
            // Chinese substitution to those would change an untranslated line.
            if let Some(bytes) = encode_native_character(character) {
                if bytes.len() == 2 {
                    slots
                        .entry([bytes[0], bytes[1]])
                        .or_default()
                        .insert(character);
                }
                bytes_by_character.insert(character, bytes);
                continue;
            }

            let Some(carrier) = substitutions.get(&character).copied() else {
                unmapped.insert(character);
                continue;
            };
            let bytes = match vn_font::font_98::cp932_for_carrier(carrier) {
                Ok(bytes) if vn_font::font_98::has_loaded_np2_slot(carrier) => bytes,
                _ => {
                    unmapped.insert(character);
                    continue;
                }
            };
            slots.entry(bytes).or_default().insert(character);
            bytes_by_character.insert(character, bytes.to_vec());
        }

        let duplicates = slots
            .values()
            .filter(|owners| owners.len() > 1)
            .flat_map(|owners| owners.iter().copied())
            .collect::<BTreeSet<_>>();
        if !unmapped.is_empty() || !duplicates.is_empty() {
            let mut details = Vec::new();
            if !unmapped.is_empty() {
                details.push(format!(
                    "未映射或字体槽不可用字符: {}",
                    format_characters(&unmapped)
                ));
            }
            if !duplicates.is_empty() {
                details.push(format!(
                    "映射槽位重复字符: {}",
                    format_characters(&duplicates)
                ));
            }
            return Err(format!("{}。请修改或删除这些字符后再注入。", details.join("；")).into());
        }

        Ok(Self { bytes_by_character })
    }

    fn encode(&self, text: &str) -> Result<Vec<u8>, String> {
        let capacity = text.len().saturating_mul(2);
        let mut bytes = Vec::with_capacity(capacity);
        for character in text.chars() {
            let mapped = self.bytes_by_character.get(&character).ok_or_else(|| {
                format!(
                    "字符 {character} (U+{:04X}) 不在已预检的映射集合中",
                    character as u32
                )
            })?;
            bytes.extend_from_slice(mapped);
        }
        Ok(bytes)
    }
}

fn encode_native_character(character: char) -> Option<Vec<u8>> {
    let text = character.to_string();
    let (encoded, _, had_errors) = encoding_rs::SHIFT_JIS.encode(&text);
    if had_errors {
        return None;
    }
    match encoded.len() {
        1 if (0xa1..=0xdf).contains(&encoded[0]) => Some(encoded.into_owned()),
        2 if vn_font::font_98::has_loaded_np2_slot(character) => Some(encoded.into_owned()),
        _ => None,
    }
}

fn format_characters(characters: &BTreeSet<char>) -> String {
    characters
        .iter()
        .map(|character| format!("{} (U+{:04X})", character.escape_debug(), *character as u32))
        .collect::<Vec<_>>()
        .join("、")
}

fn record_bindings(records: &[ScriptTextRecord]) -> Vec<RecordBinding> {
    records
        .iter()
        .map(|record| RecordBinding {
            kind: match record.kind {
                mjo_text::RecordKind::Display => "display".into(),
                mjo_text::RecordKind::ChoiceOption => "choice_option".into(),
            },
            offset: record.offset,
            fragment_offsets: record
                .fragments
                .iter()
                .map(|fragment| fragment.offset)
                .collect(),
            native_events: record
                .events
                .iter()
                .map(|event| NativeEventBinding {
                    before_fragment_index: event.before_fragment_index,
                    offset: event.offset,
                    opcode: event.opcode,
                    kind: match event.kind {
                        mjo_text::NativeEventKind::LineBreak => "line_break".into(),
                        mjo_text::NativeEventKind::Ruby => "ruby".into(),
                        mjo_text::NativeEventKind::Audio => "audio".into(),
                        mjo_text::NativeEventKind::Dynamic => "dynamic".into(),
                        mjo_text::NativeEventKind::OpaqueParameter => "opaque_parameter".into(),
                    },
                    source_text: event.source_text.clone(),
                    source_bytes_hex: event.source_bytes_hex.clone(),
                    related_offset: event.related_offset,
                    visible_token: event.visible_token.clone(),
                })
                .collect(),
            opcode: record.opcode,
            has_name: record.name.is_some(),
            source_name: record.name.clone(),
            source_text: mjo_text::primary_source_text(record),
            source_fragments: record
                .fragments
                .iter()
                .map(|fragment| NativeFragmentBinding {
                    offset: fragment.offset,
                    opcode: fragment.opcode,
                    source_text: fragment.source_text.clone(),
                    display_text: fragment.display_text.clone(),
                    source_bytes_hex: fragment.source_bytes_hex.clone(),
                    physical_newline_positions: fragment.physical_newline_positions.clone(),
                })
                .collect(),
            choice_quote_wrapped: record.choice_quote_wrapped,
            choice_control: record
                .choice_control
                .as_ref()
                .map(|control| ChoiceControlBinding {
                    call_offset: control.call_offset,
                    callee_hash: control.callee_hash,
                    argument_count: control.argument_count,
                    option_index: control.option_index,
                    source_option_offsets: control.source_option_offsets.clone(),
                    flow_edges: control
                        .flow_edges
                        .iter()
                        .map(|edge| ChoiceFlowEdgeBinding {
                            instruction_offset: edge.instruction_offset,
                            opcode: edge.opcode,
                            target_offset: edge.target_offset,
                        })
                        .collect(),
                }),
        })
        .collect()
}

fn parse_mjo(source_key: &str, bytes: &[u8]) -> vn_cli::Result<MjoDocument> {
    MjoDocument::parse(bytes).map_err(|error| format!("无法解析 {source_key}: {error}").into())
}

fn parse_program(source_key: &str, document: &MjoDocument) -> vn_cli::Result<BytecodeProgram> {
    parse_payload(document.payload(), &document.symbols, document.entrypoint)
        .map_err(|error| format!("无法解码 {source_key} 指令边界: {error}").into())
}

fn read_relative_file(root: &Path, relative: &Path) -> vn_cli::Result<Vec<u8>> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("拒绝翻译目录中的不安全相对路径: {}", relative.display()).into());
    }
    let mut current = root.to_path_buf();
    let components = relative.components().collect::<Vec<_>>();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(name) = component else {
            return Err("翻译路径包含非普通组件".into());
        };
        current.push(name);
        let metadata = fs::symlink_metadata(&current)?;
        if metadata.file_type().is_symlink() {
            return Err(format!("翻译目录含符号链接，拒绝读取: {}", current.display()).into());
        }
        if index + 1 == components.len() {
            if !metadata.is_file() {
                return Err(format!("翻译路径不是普通文件: {}", relative.display()).into());
            }
        } else if !metadata.is_dir() {
            return Err(format!("翻译路径的中间组件不是目录: {}", relative.display()).into());
        }
    }
    Ok(fs::read(current)?)
}

fn read_optional_relative_file(root: &Path, relative: &Path) -> vn_cli::Result<Option<Vec<u8>>> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("拒绝翻译目录中的不安全相对路径: {}", relative.display()).into());
    }
    let mut current = root.to_path_buf();
    let components = relative.components().collect::<Vec<_>>();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(name) = component else {
            return Err("翻译路径包含非普通组件".into());
        };
        current.push(name);
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        if metadata.file_type().is_symlink() {
            return Err(format!("翻译目录含符号链接，拒绝读取: {}", current.display()).into());
        }
        if index + 1 == components.len() {
            if !metadata.is_file() {
                return Err(format!("翻译路径不是普通文件: {}", relative.display()).into());
            }
        } else if !metadata.is_dir() {
            return Err(format!("翻译路径的中间组件不是目录: {}", relative.display()).into());
        }
    }
    Ok(Some(fs::read(current)?))
}

fn write_staged_file(root: &Path, relative: &Path, contents: &[u8]) -> vn_cli::Result<()> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("拒绝写入不安全的输出相对路径: {}", relative.display()).into());
    }
    let destination = root.join(relative);
    let parent = destination.parent().ok_or("输出文件缺少父目录")?;
    fs::create_dir_all(parent)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    file.write_all(contents)?;
    file.sync_all()?;
    Ok(())
}

fn ensure_no_overlap(first: &Path, second: &Path, message: &str) -> vn_cli::Result<()> {
    if first == second || first.starts_with(second) || second.starts_with(first) {
        return Err(message.into());
    }
    Ok(())
}

fn is_mjo_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("mjo"))
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
