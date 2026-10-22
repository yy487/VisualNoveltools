//! Source-bound export/import for the installed DOS game directory.

use crate::{
    analysis::{
        self, AnalysisDiagnostic, MenuTableCandidate, ParserCallSite, PushExpr,
        StaticPointerSource, StaticTextTarget, StaticTextTargetPath,
    },
    json_bytes, lzss, output, sha256_hex, text, Result,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};
use vn_font::font_98::{self as font98, EncodingPlan, EncodingPlanEntry, SubstitutionMap};
use vn_text::Entry;

const WORKSPACE_FORMAT: &str = "cyber-illusion-pc98-workspace-v4";
const PRIMARY_FORMAT: &str = "cyber-illusion-pc98-primary-v5";
const WORKSPACE_FILE: &str = "cyber_illusion_workspace.json";
const PRIMARY_FILE: &str = "primary/extraction.json";
const FONT_FILE: &str = "font.tmp";
const FONT_MANIFEST_FILE: &str = "font98_mapping.json";
const MODULE_LOAD_BIAS: usize = 0x0100;
const MAX_MODULE_IMAGE_SIZE: usize = 0xE000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct WorkspaceManifest {
    format: String,
    tool_version: String,
    source_tree_sha256: String,
    primary_sha256: String,
    source_files: Vec<SourceFileBinding>,
    modules: Vec<ModuleBinding>,
}

#[derive(Debug, Clone, Serialize)]
struct Pc98FontManifest {
    schema: String,
    face: String,
    source_font_sha256: String,
    output_font_sha256: String,
    patched_glyphs: usize,
    reserved_cp932: Vec<String>,
    mapping: Vec<EncodingPlanEntry>,
}

struct Pc98FontArtifacts {
    encoding: EncodingPlan,
    font_bytes: Vec<u8>,
    manifest_bytes: Vec<u8>,
    patched_glyphs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SourceFileBinding {
    path: String,
    size: usize,
    sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ModuleBinding {
    path: String,
    translation_path: String,
    source_sha256: String,
    expanded_sha256: String,
    expanded_size: usize,
    records: Vec<RecordBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RecordBinding {
    source_offset: usize,
    source_end: usize,
    raw_sha256: String,
    parser_calls: Vec<ParserUse>,
    translation_segments: Vec<SegmentBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SegmentBinding {
    segment_index: usize,
    source_offset: usize,
    source_end: usize,
    name_sha256: Option<String>,
    message_sha256: String,
    read_only_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ParserUse {
    function_entry_output_offset: usize,
    code_callsite_output_offset: usize,
    window: Option<u16>,
    window_expr: Option<PushExpr>,
    segment: String,
    runtime_pointer: u16,
    output_offset: usize,
    pointer_expr: PushExpr,
    pointer_source: StaticPointerSource,
    push_output_offsets: [usize; 3],
    definition_offsets: Vec<usize>,
    path_offsets: Vec<usize>,
    target_raw_sha256: String,
    evidence_paths: Vec<StaticTextTargetPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrimaryExtraction {
    format: String,
    tool_version: String,
    source_tree_sha256: String,
    source_files: Vec<SourceFileBinding>,
    modules: Vec<PrimaryModule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrimaryModule {
    path: String,
    compressed_size: usize,
    source_sha256: String,
    expanded_size: usize,
    expanded_sha256: String,
    instruction_count: usize,
    edge_count: usize,
    resolved_indirect_jump_tables: Vec<analysis::ResolvedIndirectJumpTable>,
    parser_calls: Vec<ParserCallSite>,
    diagnostics: Vec<AnalysisDiagnostic>,
    direct_menu_tables: Vec<MenuTableCandidate>,
    records: Vec<PrimaryTextRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrimaryTextRecord {
    source_offset: usize,
    source_end: usize,
    terminator_offset: usize,
    terminator: u8,
    raw_hex: String,
    message: String,
    /// Concatenated four-digit JIS codes for mapped glyphs, in parser order.
    glyph_jis_hex: String,
    /// Controls, skipped bytes, and RAW spans; ordinary glyphs are omitted.
    non_glyph_tokens: Vec<PrimaryTextToken>,
    parser_calls: Vec<ParserUse>,
    translation_segments: Vec<PrimaryProjectionSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrimaryProjectionSegment {
    source_offset: usize,
    source_end: usize,
    name: Option<String>,
    message: String,
    read_only_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrimaryTextToken {
    source_offset: usize,
    raw_hex: String,
    kind: text::TextTokenKind,
}

#[derive(Debug, Clone)]
struct ParsedTextRecord {
    record: text::TextRecord,
    parser_calls: Vec<ParserUse>,
    segments: Vec<ProjectedSegment>,
}

#[derive(Debug, Clone)]
struct ProjectedSegment {
    entry: Entry,
    source_offset: usize,
    source_end: usize,
    original_bytes: Vec<u8>,
    plan: SegmentPlan,
}

#[derive(Debug, Clone)]
struct SegmentPlan {
    prefix_controls: Vec<Vec<u8>>,
    name: Option<NamePlan>,
    after_name_controls: Vec<Vec<u8>>,
    lines: Vec<MessageLinePlan>,
    suffix_controls: Vec<Vec<u8>>,
    read_only_reason: Option<String>,
}

#[derive(Debug, Clone)]
enum NamePlan {
    StaticBracketed {
        left_bytes: Vec<u8>,
        original_bytes: Vec<u8>,
        original_text: String,
        right_bytes: Vec<u8>,
        allowed_raw: Vec<Vec<u8>>,
    },
    DynamicSubstitution {
        slot: u8,
        raw_bytes: Vec<u8>,
    },
}

#[derive(Debug, Clone)]
struct MessageLinePlan {
    original_text: String,
    original_atoms: Vec<Vec<u8>>,
    all_original_atoms: Vec<Vec<u8>>,
    original_controls: Vec<ProjectionControlAnchor>,
    leading_padding: Vec<Vec<u8>>,
    trailing_padding: Vec<Vec<u8>>,
    controls_before_padding: Vec<Vec<u8>>,
    controls_after_padding: Vec<Vec<u8>>,
    controls: Vec<ProjectionControlAnchor>,
    controls_before_trailing: Vec<Vec<u8>>,
    controls_after_trailing: Vec<Vec<u8>>,
    layout_only: bool,
    newline_after: Option<Vec<u8>>,
    allowed_raw: Vec<Vec<u8>>,
    allowed_substitutions: Vec<u8>,
}

#[derive(Debug, Clone)]
struct ProjectionControlAnchor {
    atom_index: usize,
    raw_bytes: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
struct ProjectionLineBuilder {
    bytes: Vec<u8>,
    atoms: Vec<Vec<u8>>,
    atom_texts: Vec<String>,
    atom_count: usize,
    controls: Vec<ProjectionControlAnchor>,
    allowed_raw: Vec<Vec<u8>>,
    substitutions: Vec<u8>,
}

struct RuntimeSource {
    root: PathBuf,
    files: BTreeMap<String, Vec<u8>>,
    bindings: Vec<SourceFileBinding>,
    tree_sha256: String,
}

struct ParsedModule {
    path: String,
    compressed: Vec<u8>,
    expanded: Vec<u8>,
    analysis: analysis::AnalysisReport,
    menu_tables: Vec<MenuTableCandidate>,
    records: Vec<ParsedTextRecord>,
}

pub fn prepare_export(input_directory: &Path, output_path: &Path) -> Result<output::Prepared> {
    let source = read_runtime_source(input_directory)?;
    let modules = parse_modules(&source)?;
    let module_bindings = build_module_bindings(&modules);

    let primary = PrimaryExtraction {
        format: PRIMARY_FORMAT.into(),
        tool_version: env!("CARGO_PKG_VERSION").into(),
        source_tree_sha256: source.tree_sha256.clone(),
        source_files: source.bindings.clone(),
        modules: modules.iter().map(primary_module).collect(),
    };
    let primary_bytes = json_bytes(&primary)?;
    let manifest = WorkspaceManifest {
        format: WORKSPACE_FORMAT.into(),
        tool_version: env!("CARGO_PKG_VERSION").into(),
        source_tree_sha256: source.tree_sha256.clone(),
        primary_sha256: sha256_hex(&primary_bytes),
        source_files: source.bindings.clone(),
        modules: module_bindings.clone(),
    };

    let mut payloads = BTreeMap::new();
    for (relative, bytes) in &source.files {
        payloads.insert(format!("source/{relative}"), bytes.clone());
    }
    payloads.insert(PRIMARY_FILE.into(), primary_bytes);
    for module in &modules {
        let entries = translation_entries(module);
        let translation_path = translation_path(&module.path)?;
        payloads.insert(
            translation_path,
            vn_text::write_template(&entries).map_err(|error| error.to_string())?,
        );
    }
    payloads.insert(WORKSPACE_FILE.into(), json_bytes(&manifest)?);

    let mut warnings = analysis_warnings(&modules);
    if modules.iter().any(|module| !module.menu_tables.is_empty()) {
        warnings.push("已识别直接菜单渲染器的指针表线索，但这些字符串尚未纳入翻译/回注".into());
    }
    let text_entries = modules
        .iter()
        .map(|module| translation_entries(module).len())
        .sum::<usize>();
    let output_file_count = payloads.len();
    let details = vec![
        format!("来源文件：{}", source.bindings.len()),
        format!("来源目录 SHA-256：{}", source.tree_sha256),
        format!("场景模块：{}", modules.len()),
        format!("翻译条目：{text_entries}"),
        format!("主提取：{}", PRIMARY_FILE),
        format!("工作区绑定：{WORKSPACE_FILE}"),
    ];
    output::Prepared::new(
        vec![source.root],
        output_path,
        payloads,
        format!(
            "已提取 {} 个场景模块中的 {text_entries} 条 slot-54 文本记录",
            modules.len()
        ),
        vec![
            "校验并读取完整 DOS 游戏目录".into(),
            "解开 eLZ0 并按 slot-54 调用及分派路径定位记录".into(),
            "将台词名与正文分开；控制指令保留在主提取映射".into(),
            "生成完整主提取、逐模块翻译 JSON 与来源快照".into(),
        ],
        details,
        output::Metrics {
            files: output_file_count,
            modules: modules.len(),
            text_entries,
            changed_entries: 0,
            warnings,
        },
    )
}

pub fn prepare_import(
    input_directory: &Path,
    workspace_path: &Path,
    translation_paths: &[PathBuf],
    output_path: &Path,
) -> Result<output::Prepared> {
    if translation_paths.is_empty() {
        return Err("至少选择一份翻译 JSON；可只回注部分 TEXT 模块".into());
    }

    let source = read_runtime_source(input_directory)?;
    let modules = parse_modules(&source)?;
    let current_bindings = build_module_bindings(&modules);

    let workspace_root = canonical_directory(workspace_path, "文本工作区")?;
    let manifest_path = workspace_root.join(WORKSPACE_FILE);
    let manifest_bytes = read_regular_file(&manifest_path, "工作区绑定清单")?;
    let manifest: WorkspaceManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("工作区绑定清单 JSON 无效：{error}"))?;
    if manifest.format != WORKSPACE_FORMAT {
        return Err(format!("不支持的工作区格式：{}", manifest.format));
    }
    if manifest.source_tree_sha256 != source.tree_sha256 || manifest.source_files != source.bindings
    {
        return Err("当前游戏目录与导出工作区的来源文件或哈希不一致".into());
    }
    if manifest.modules != current_bindings {
        return Err("重新解析后的模块、slot-54 调用或文本边界与工作区绑定不一致".into());
    }

    let primary_path = workspace_root.join(safe_relative(PRIMARY_FILE)?);
    let primary_bytes = read_regular_file(&primary_path, "主提取 JSON")?;
    if sha256_hex(&primary_bytes) != manifest.primary_sha256 {
        return Err("主提取 JSON 已变化或与工作区清单不匹配".into());
    }
    verify_workspace_sources(&workspace_root, &manifest.source_files)?;

    let modules_by_path = modules
        .iter()
        .map(|module| (module.path.as_str(), module))
        .collect::<BTreeMap<_, _>>();
    let translation_root =
        canonical_directory(&workspace_root.join("translations"), "工作区翻译目录")?;
    let bindings_by_filename = manifest
        .modules
        .iter()
        .map(|binding| {
            let filename = Path::new(&binding.translation_path)
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| format!("工作区绑定含无效译文路径：{}", binding.translation_path))?;
            Ok((filename.to_owned(), binding))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    if bindings_by_filename.len() != manifest.modules.len() {
        return Err("工作区绑定中有重复的译文文件名".into());
    }
    let mut selected = BTreeMap::<String, Vec<Entry>>::new();
    let mut selected_inputs = Vec::new();
    let mut selected_translation_paths = BTreeSet::new();

    for selection in translation_paths {
        let input_files = collect_translation_inputs(selection, &translation_root)?;
        if input_files.is_empty() {
            return Err(format!(
                "所选翻译目录中没有 JSON 文件：{}",
                selection.display()
            ));
        }
        for input_path in input_files {
            let filename = input_path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| format!("翻译 JSON 文件名无效：{}", input_path.display()))?;
            let binding = bindings_by_filename.get(filename).ok_or_else(|| {
                format!(
                    "翻译 JSON 文件名未绑定到当前来源模块：{}",
                    input_path.display()
                )
            })?;
            if !selected_translation_paths.insert(binding.translation_path.clone()) {
                return Err(format!(
                    "重复选择场景模块译文：{}",
                    binding.translation_path
                ));
            }
            let module = modules_by_path
                .get(binding.path.as_str())
                .ok_or_else(|| format!("工作区绑定引用了未知模块：{}", binding.path))?;
            let expected = translation_entries(module);
            let bytes = read_regular_file(&input_path, "翻译 JSON")?;
            let supplied = vn_text::read_template(&bytes)
                .map_err(|error| format!("翻译 JSON {} 无效：{error}", input_path.display()))?;
            vn_text::validate_shape(&expected, &supplied).map_err(|error| {
                format!(
                    "翻译 JSON {} 与来源条目形状不符：{error}",
                    input_path.display()
                )
            })?;
            selected.insert(binding.path.clone(), supplied);
            selected_inputs.push(input_path);
        }
    }

    let font_artifacts = build_pc98_font_artifacts(&modules, &selected)?;
    let mut payloads = source.files.clone();
    if payloads.contains_key(FONT_FILE) {
        return Err(format!(
            "来源游戏目录已有 {FONT_FILE}，不能安全替换它作为 NP2 字库输出"
        ));
    }
    payloads.insert(FONT_FILE.into(), font_artifacts.font_bytes.clone());
    payloads.insert(
        FONT_MANIFEST_FILE.into(),
        font_artifacts.manifest_bytes.clone(),
    );
    let mut changed_entries = 0usize;
    for module in &modules {
        let Some(supplied) = selected.get(&module.path) else {
            continue;
        };
        let expected_count = translation_entries(module).len();
        if supplied.len() != expected_count {
            return Err(format!(
                "内部错误：{} 的翻译条目数未通过绑定校验",
                module.path
            ));
        }

        let mut expanded = module.expanded.clone();
        let mut module_changed = false;
        let mut supplied_index = 0usize;
        let mut protected_ranges = menu_target_ranges(module);
        protected_ranges.extend(parser_pointer_ranges(module));
        let mut free_spans = Vec::<(usize, usize)>::new();
        let mut pending_relocations = Vec::<(usize, Vec<u8>)>::new();
        for (record_index, projected_record) in module.records.iter().enumerate() {
            let original = &projected_record.record;
            let mut rebuilt_record = Vec::with_capacity(original.raw_bytes.len());
            if projected_record.segments.is_empty() {
                rebuilt_record.extend_from_slice(&original.raw_bytes);
            }
            for segment in &projected_record.segments {
                let edited = &supplied[supplied_index];
                if edited != &segment.entry {
                    changed_entries += 1;
                }
                let rebuilt = rebuild_projected_segment(segment, edited, &font_artifacts.encoding)
                    .map_err(|error| {
                        format!(
                            "{} 翻译条目 {}（源偏移 0x{:X}）：{error}",
                            module.path, supplied_index, segment.source_offset
                        )
                    })?;
                if rebuilt != segment.original_bytes {
                    module_changed = true;
                }
                rebuilt_record.extend_from_slice(&rebuilt);
                supplied_index += 1;
            }

            if rebuilt_record != original.raw_bytes {
                if original.terminator != 0
                    || original.terminator_offset.checked_add(1) != Some(original.source_end)
                {
                    return Err(format!(
                        "{} 源记录 0x{:X} 的外层 NUL 位置不符合已确认边界",
                        module.path, original.source_offset
                    ));
                }
                if rebuilt_record.len() <= original.raw_bytes.len() {
                    let terminator_offset = original
                        .source_offset
                        .checked_add(rebuilt_record.len())
                        .ok_or_else(|| format!("{} 的文本范围溢出", module.path))?;
                    let target = expanded
                        .get_mut(original.source_offset..terminator_offset)
                        .ok_or_else(|| format!("{} 的回注范围越界", module.path))?;
                    target.copy_from_slice(&rebuilt_record);
                    *expanded
                        .get_mut(terminator_offset)
                        .ok_or_else(|| format!("{} 的 NUL 终止符范围越界", module.path))? =
                        original.terminator;
                    if rebuilt_record.len() < original.raw_bytes.len() {
                        let free_start = terminator_offset + 1;
                        let free_end = original.source_end;
                        let overlaps_protected = protected_ranges
                            .iter()
                            .any(|(start, end)| *start < free_end && free_start < *end);
                        if free_start < free_end && !overlaps_protected {
                            free_spans.push((free_start, free_end - free_start));
                        }
                    }
                } else {
                    pending_relocations.push((record_index, rebuilt_record));
                }
                module_changed = true;
            }
        }
        if supplied_index != supplied.len() {
            return Err(format!("内部错误：{} 有未应用的翻译条目", module.path));
        }

        for (record_index, _) in &pending_relocations {
            let record = &module.records[*record_index].record;
            let overlaps_protected = protected_ranges
                .iter()
                .any(|(start, end)| *start < record.source_end && record.source_offset < *end);
            if !overlaps_protected {
                free_spans.push((
                    record.source_offset,
                    record.source_end - record.source_offset,
                ));
            }
        }
        pending_relocations.sort_by(|left, right| right.1.len().cmp(&left.1.len()));
        let mut relocation_plan = Vec::<(usize, Vec<u8>, usize)>::new();
        let mut append_cursor = expanded.len();
        for (record_index, rebuilt_record) in pending_relocations {
            let required = rebuilt_record
                .len()
                .checked_add(1)
                .ok_or_else(|| format!("{} 的扩展文本长度溢出", module.path))?;
            let free_index = free_spans
                .iter()
                .enumerate()
                .filter(|(_, (_, capacity))| *capacity >= required)
                .min_by_key(|(_, (_, capacity))| *capacity)
                .map(|(index, _)| index);
            let relocated_offset = if let Some(index) = free_index {
                let (start, capacity) = free_spans.swap_remove(index);
                if capacity > required {
                    free_spans.push((start + required, capacity - required));
                }
                start
            } else {
                let start = append_cursor;
                append_cursor = start
                    .checked_add(required)
                    .ok_or_else(|| format!("{} 的扩展文本长度溢出", module.path))?;
                if append_cursor > MAX_MODULE_IMAGE_SIZE {
                    return Err(format!(
                        "{} 添加超长译文后需要 {} 字节，超过 PS1SYS 模块缓冲上限 {}",
                        module.path, append_cursor, MAX_MODULE_IMAGE_SIZE
                    ));
                }
                start
            };
            relocation_plan.push((record_index, rebuilt_record, relocated_offset));
        }
        for (record_index, _, relocated_offset) in &relocation_plan {
            let projected_record = &module.records[*record_index];
            let runtime_pointer = relocated_offset
                .checked_add(MODULE_LOAD_BIAS)
                .filter(|pointer| *pointer <= u16::MAX as usize)
                .ok_or_else(|| format!("{} 的文本目标超出 16 位段地址", module.path))?;
            patch_relocated_record_pointers(
                &mut expanded,
                projected_record,
                runtime_pointer as u16,
            )?;
        }
        relocation_plan.sort_by_key(|(_, _, offset)| *offset);
        for (record_index, rebuilt_record, relocated_offset) in relocation_plan {
            let record = &module.records[record_index].record;
            let required = rebuilt_record.len() + 1;
            let end = relocated_offset + required;
            if end <= expanded.len() {
                let target = expanded
                    .get_mut(relocated_offset..end)
                    .ok_or_else(|| format!("{} 的可回收文本区越界", module.path))?;
                target[..rebuilt_record.len()].copy_from_slice(&rebuilt_record);
                target[rebuilt_record.len()] = record.terminator;
            } else if relocated_offset == expanded.len() {
                expanded.extend_from_slice(&rebuilt_record);
                expanded.push(record.terminator);
            } else {
                return Err(format!(
                    "{} 的文本目标区间 0x{relocated_offset:X}..0x{end:X} 不是连续可用空间",
                    module.path
                ));
            }
        }

        if module_changed {
            patch_ps1sys_module_bounds(&module.path, &module.expanded, &mut expanded)?;
            let rebuilt = lzss::encode_literal(&expanded)?;
            let verified = lzss::decode_file(&rebuilt)?;
            if verified.bytes != expanded {
                return Err(format!("{} 重建后解压结果不一致", module.path));
            }
            payloads.insert(module.path.clone(), rebuilt);
        }
    }

    let mut inputs = vec![source.root.clone(), workspace_root.clone()];
    inputs.extend(selected_inputs);
    let warnings = analysis_warnings(&modules);
    let output_file_count = payloads.len();
    let details = vec![
        format!("来源目录 SHA-256：{}", source.tree_sha256),
        format!("所选译文文件：{}", selected_translation_paths.len()),
        format!("有变化的译文条目：{changed_entries}"),
        format!("NP2 自定义字形：{}", font_artifacts.patched_glyphs),
        format!("配套字库与映射：{FONT_FILE}、{FONT_MANIFEST_FILE}"),
        "未选择的模块与其他游戏文件按当前来源原样复制".into(),
    ];
    output::Prepared::new(
        inputs,
        output_path,
        payloads,
        format!("已回注 {changed_entries} 条译文，重建完整 DOS 游戏目录和 NP2 字库"),
        vec![
            "重新解析当前游戏目录与全部 TEXT 模块".into(),
            "校验工作区来源、记录边界、JSON 顺序与控制语法".into(),
            "规划 CP932 字槽、生成配套 NP2 font.tmp 并编码译文".into(),
            "只改写已选择且有变化的模块并重新编码 eLZ0".into(),
            "暂存完整游戏目录、读回核对后提交".into(),
        ],
        details,
        output::Metrics {
            files: output_file_count,
            modules: modules.len(),
            text_entries: modules.iter().map(|module| module.records.len()).sum(),
            changed_entries,
            warnings,
        },
    )
}

fn build_pc98_font_artifacts(
    modules: &[ParsedModule],
    selected: &BTreeMap<String, Vec<Entry>>,
) -> Result<Pc98FontArtifacts> {
    let mut reserved_cp932 = BTreeSet::<u16>::new();
    let mut display_texts = Vec::<String>::new();

    for module in modules {
        for record in &module.records {
            collect_reserved_cp932_from_tokens(&record.record.tokens, &mut reserved_cp932);
        }
        for table in &module.menu_tables {
            for entry in &table.entries {
                if !entry.nul_terminated {
                    continue;
                }
                if let Some(offset) = entry.output_offset {
                    if let Ok(record) = text::parse_message(&module.expanded, offset) {
                        collect_reserved_cp932_from_tokens(&record.tokens, &mut reserved_cp932);
                    }
                }
            }
        }

        let supplied = selected.get(&module.path);
        let mut entry_index = 0usize;
        for record in &module.records {
            for segment in &record.segments {
                let entry = match supplied {
                    Some(entries) => entries.get(entry_index).ok_or_else(|| {
                        format!("内部错误：{} 的字体计划缺少译文条目", module.path)
                    })?,
                    None => &segment.entry,
                };
                if let Some(name) = entry.name.as_deref() {
                    let visible = font_plan_display_text(name)?;
                    add_native_font_slots(&visible, &mut reserved_cp932);
                    if !visible.is_empty() {
                        display_texts.push(visible);
                    }
                }
                let visible = font_plan_display_text(&entry.message)?;
                add_native_font_slots(&visible, &mut reserved_cp932);
                if !visible.is_empty() {
                    display_texts.push(visible);
                }
                entry_index += 1;
            }
        }
        if supplied.is_some_and(|entries| entries.len() != entry_index) {
            return Err(format!(
                "内部错误：{} 的字体计划条目数与工作区绑定不一致",
                module.path
            ));
        }
    }

    let substitutions = SubstitutionMap::embedded()
        .map_err(|error| format!("读取内置 NP2 简中字体映射失败：{error}"))?;
    let encoding = EncodingPlan::build(
        &substitutions,
        reserved_cp932.iter().copied(),
        display_texts.iter().map(String::as_str),
    )
    .map_err(|error| format!("规划 NP2 字形载体失败：{error}"))?;
    let mapping = encoding
        .manifest_entries()
        .map_err(|error| format!("生成 NP2 字形映射清单失败：{error}"))?;
    let build = font98::prepare_font(
        font98::EMBEDDED_FONT,
        &encoding.requests(),
        &reserved_cp932,
        font98::FONT_FACE,
    )
    .map_err(|error| format!("重绘 NP2 font.tmp 失败：{error}"))?;
    let manifest = Pc98FontManifest {
        schema: "cyber-illusion-pc98-font-v1".into(),
        face: font98::FONT_FACE.into(),
        source_font_sha256: font98::embedded_font_sha256(),
        output_font_sha256: sha256_hex(&build.bytes),
        patched_glyphs: build.patched_glyphs,
        reserved_cp932: reserved_cp932
            .iter()
            .map(|code| format!("{code:04X}"))
            .collect(),
        mapping,
    };
    Ok(Pc98FontArtifacts {
        encoding,
        font_bytes: build.bytes,
        manifest_bytes: json_bytes(&manifest)?,
        patched_glyphs: build.patched_glyphs,
    })
}

fn patch_relocated_record_pointers(
    expanded: &mut [u8],
    record: &ParsedTextRecord,
    new_runtime_pointer: u16,
) -> Result<()> {
    let mut patches = BTreeMap::<usize, u16>::new();
    for parser_call in &record.parser_calls {
        let patch_offset = match &parser_call.pointer_source {
            StaticPointerSource::ImmediatePush => {
                let push_offset = parser_call.push_output_offsets[2];
                if expanded.get(push_offset) != Some(&0x68) {
                    return Err(format!(
                        "回收文本记录 0x{:X} 的指针推入位点 0x{push_offset:X} 不是 PUSH imm16",
                        record.record.source_offset
                    ));
                }
                push_offset + 1
            }
            StaticPointerSource::DsAbsoluteTable {
                table_entry_output_offset,
                ..
            } => *table_entry_output_offset,
        };
        let existing = expanded
            .get(patch_offset..patch_offset + 2)
            .ok_or_else(|| {
                format!(
                    "文本记录 0x{:X} 的指针位点 0x{patch_offset:X} 越界",
                    record.record.source_offset
                )
            })?;
        let existing = u16::from_le_bytes([existing[0], existing[1]]);
        if existing != parser_call.runtime_pointer {
            return Err(format!(
                "文本记录 0x{:X} 的原指针 0x{existing:04X} 与反汇编记录 0x{:04X} 不符",
                record.record.source_offset, parser_call.runtime_pointer
            ));
        }
        if let Some(previous) = patches.insert(patch_offset, new_runtime_pointer) {
            if previous != new_runtime_pointer {
                return Err(format!(
                    "文本记录 0x{:X} 的指针位点 0x{patch_offset:X} 有冲突目标",
                    record.record.source_offset
                ));
            }
        }
    }
    if patches.is_empty() {
        return Err(format!(
            "超长文本记录 0x{:X} 没有可回写的静态指针证据",
            record.record.source_offset
        ));
    }
    for (offset, pointer) in patches {
        let bytes = expanded
            .get_mut(offset..offset + 2)
            .ok_or_else(|| format!("回写文本指针 0x{offset:X} 越界"))?;
        bytes.copy_from_slice(&pointer.to_le_bytes());
    }
    Ok(())
}

/// PS1SYS loads a module at offset 0x100 and uses its words at file offsets
/// 0x0A/0x0C as the initialized-data and BSS boundaries. It zeroes the range
/// between them, then fills the remaining segment with the 0x5453 "TS" marker.
/// If text is appended without moving both boundaries, the loader erases that
/// text or exposes the marker as dialogue.
fn patch_ps1sys_module_bounds(path: &str, original: &[u8], rebuilt: &mut [u8]) -> Result<()> {
    if rebuilt.len() <= original.len() {
        return Ok(());
    }
    const FILE_END_OFFSET: usize = 0x0A;
    const BSS_END_OFFSET: usize = 0x0C;

    let read_word = |bytes: &[u8], offset: usize| -> Result<u16> {
        let word = bytes
            .get(offset..offset + 2)
            .ok_or_else(|| format!("{path} 的 PS1SYS 边界字段 0x{offset:X} 越界"))?;
        Ok(u16::from_le_bytes([word[0], word[1]]))
    };
    let old_file_end = usize::from(read_word(original, FILE_END_OFFSET)?);
    let old_bss_end = usize::from(read_word(original, BSS_END_OFFSET)?);
    let expected_file_end = original
        .len()
        .checked_add(MODULE_LOAD_BIAS)
        .ok_or_else(|| format!("{path} 的 PS1SYS 文件边界溢出"))?;
    if old_file_end != expected_file_end || old_bss_end < old_file_end {
        return Err(format!(
            "{path} 扩展后不能安全更新 PS1SYS 内存边界：模块长度 0x{:X}，文件边界 0x{old_file_end:X}，BSS 边界 0x{old_bss_end:X}",
            original.len()
        ));
    }

    let growth = rebuilt.len() - original.len();
    let new_file_end = rebuilt
        .len()
        .checked_add(MODULE_LOAD_BIAS)
        .ok_or_else(|| format!("{path} 的 PS1SYS 文件边界溢出"))?;
    let new_bss_end = old_bss_end
        .checked_add(growth)
        .ok_or_else(|| format!("{path} 的 PS1SYS BSS 边界溢出"))?;
    if new_file_end > new_bss_end || new_bss_end > MODULE_LOAD_BIAS + MAX_MODULE_IMAGE_SIZE {
        return Err(format!(
            "{path} 扩展后 PS1SYS 数据区 0x{new_file_end:X}..0x{new_bss_end:X} 超出 0x{:X} 字节模块缓冲区",
            MAX_MODULE_IMAGE_SIZE
        ));
    }

    let file_end = u16::try_from(new_file_end)
        .map_err(|_| format!("{path} 的 PS1SYS 文件边界超过 16 位地址"))?;
    let bss_end = u16::try_from(new_bss_end)
        .map_err(|_| format!("{path} 的 PS1SYS BSS 边界超过 16 位地址"))?;
    rebuilt[FILE_END_OFFSET..FILE_END_OFFSET + 2].copy_from_slice(&file_end.to_le_bytes());
    rebuilt[BSS_END_OFFSET..BSS_END_OFFSET + 2].copy_from_slice(&bss_end.to_le_bytes());
    Ok(())
}

fn collect_reserved_cp932_from_tokens(tokens: &[text::TextToken], reserved: &mut BTreeSet<u16>) {
    for token in tokens {
        if let Ok(pair) = <[u8; 2]>::try_from(token.raw_bytes.as_slice()) {
            if font98::cp932_to_jis(pair).is_ok() {
                reserved.insert(u16::from_be_bytes(pair));
            }
        }
    }
}

fn menu_target_ranges(module: &ParsedModule) -> Vec<(usize, usize)> {
    let mut ranges = BTreeSet::new();
    for table in &module.menu_tables {
        for entry in &table.entries {
            if !entry.nul_terminated {
                continue;
            }
            if let Some(offset) = entry.output_offset {
                if let Ok(record) = text::parse_message(&module.expanded, offset) {
                    ranges.insert((record.source_offset, record.source_end));
                }
            }
        }
    }
    ranges.into_iter().collect()
}

fn parser_pointer_ranges(module: &ParsedModule) -> Vec<(usize, usize)> {
    let mut ranges = BTreeSet::new();
    for record in &module.records {
        for parser_call in &record.parser_calls {
            let offset = match &parser_call.pointer_source {
                StaticPointerSource::ImmediatePush => {
                    parser_call.push_output_offsets[2].saturating_add(1)
                }
                StaticPointerSource::DsAbsoluteTable {
                    table_entry_output_offset,
                    ..
                } => *table_entry_output_offset,
            };
            ranges.insert((offset, offset.saturating_add(2)));
        }
    }
    ranges.into_iter().collect()
}

fn add_native_font_slots(display_text: &str, reserved: &mut BTreeSet<u16>) {
    for character in display_text
        .chars()
        .filter(|character| !character.is_ascii())
    {
        let character_text = character.to_string();
        let (bytes, _, had_errors) = encoding_rs::SHIFT_JIS.encode(&character_text);
        if had_errors || bytes.len() != 2 {
            continue;
        }
        let pair = [bytes[0], bytes[1]];
        if font98::cp932_to_jis(pair).is_ok() {
            reserved.insert(u16::from_be_bytes(pair));
        }
    }
}

fn font_plan_display_text(text_value: &str) -> Result<String> {
    let mut output = String::new();
    let mut cursor = 0usize;
    while cursor < text_value.len() {
        let rest = &text_value[cursor..];
        if let Some(after_prefix) = rest.strip_prefix("{{SUB:") {
            let close = after_prefix
                .find("}}")
                .ok_or_else(|| "SUB 占位符缺少结束标记".to_owned())?;
            let value = &after_prefix[..close];
            if value.len() != 1 || !value.as_bytes()[0].is_ascii_digit() {
                return Err(format!("无效的 SUB 占位符：{{{{SUB:{value}}}}}"));
            }
            cursor += "{{SUB:".len() + close + "}}".len();
            continue;
        }
        if let Some(after_prefix) = rest.strip_prefix("[[RAW:") {
            let close = after_prefix
                .find("]]")
                .ok_or_else(|| "RAW 标记缺少结束标记".to_owned())?;
            cursor += "[[RAW:".len() + close + "]]".len();
            continue;
        }
        if rest.starts_with("[[CYBER:") {
            return Err("presentation control markers do not belong in translation text".into());
        }

        let character = rest
            .chars()
            .next()
            .ok_or_else(|| "无效的 UTF-8 边界".to_owned())?;
        match character {
            '\n' => {}
            '\r' => return Err("message 中不支持 CR 换行".into()),
            '!' => output.push('！'),
            '#' => output.push('＃'),
            '@' => output.push('＠'),
            '$' => output.push('＄'),
            value if value.is_control() => {
                return Err(format!("不支持文本控制字符 U+{:04X}", value as u32));
            }
            value if !value.is_ascii() => output.push(value),
            _ => {}
        }
        cursor += character.len_utf8();
    }
    Ok(output)
}

fn read_runtime_source(input_directory: &Path) -> Result<RuntimeSource> {
    let metadata = fs::symlink_metadata(input_directory)
        .map_err(|error| format!("读取游戏目录失败 {}：{error}", input_directory.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "游戏来源必须是实际目录：{}",
            input_directory.display()
        ));
    }
    let root = fs::canonicalize(input_directory)
        .map_err(|error| format!("解析游戏目录失败 {}：{error}", input_directory.display()))?;
    let mut files = BTreeMap::new();
    collect_runtime_files(&root, &root, &mut files)?;
    if files.is_empty() {
        return Err("游戏目录中没有可读取的运行时文件".into());
    }

    let bindings = files
        .iter()
        .map(|(path, bytes)| SourceFileBinding {
            path: path.clone(),
            size: bytes.len(),
            sha256: sha256_hex(bytes),
        })
        .collect::<Vec<_>>();
    let tree_sha256 = hash_source_tree(&bindings);
    Ok(RuntimeSource {
        root,
        files,
        bindings,
        tree_sha256,
    })
}

fn collect_runtime_files(
    root: &Path,
    directory: &Path,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("遍历游戏目录失败 {}：{error}", directory.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| format!("读取游戏目录项失败 {}：{error}", directory.display()))?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("读取文件元数据失败 {}：{error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("游戏目录中不支持符号链接：{}", path.display()));
        }
        if metadata.is_dir() {
            collect_runtime_files(root, &path, files)?;
            continue;
        }
        if !metadata.is_file() {
            return Err(format!("游戏目录中不支持特殊文件：{}", path.display()));
        }
        if is_ida_sidecar(&path) {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|_| format!("路径越出游戏根目录：{}", path.display()))?;
        let relative = normalized_relative(relative)?;
        let bytes = fs::read(&path)
            .map_err(|error| format!("读取游戏文件失败 {}：{error}", path.display()))?;
        if files.insert(relative.clone(), bytes).is_some() {
            return Err(format!("游戏目录中有重复的规范化路径：{relative}"));
        }
    }
    Ok(())
}

fn is_ida_sidecar(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    let belongs_to_named_input = path
        .file_stem()
        .and_then(|stem| Path::new(stem).extension())
        .is_some();
    belongs_to_named_input
        && extension.as_deref().is_some_and(|extension| {
            matches!(extension, "i64" | "id0" | "id1" | "id2" | "nam" | "til")
        })
}

fn parse_modules(source: &RuntimeSource) -> Result<Vec<ParsedModule>> {
    let mut paths = BTreeMap::<u8, String>::new();
    for path in source.files.keys() {
        let relative = Path::new(path);
        if relative.components().count() != 1 {
            continue;
        }
        let Some(name) = relative.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let upper = name.to_ascii_uppercase();
        let Some(stem) = upper.strip_suffix(".COM") else {
            continue;
        };
        let Some(digits) = stem.strip_prefix("TEXT") else {
            continue;
        };
        if digits.len() == 2 && digits.bytes().all(|byte| byte.is_ascii_digit()) {
            let index = digits
                .parse::<u8>()
                .map_err(|_| format!("无效的场景模块名：{name}"))?;
            if index > 29 {
                return Err(format!(
                    "发现未支持的场景模块 {name}；当前版本要求 TEXT00.COM 至 TEXT29.COM"
                ));
            }
            if paths.insert(index, path.clone()).is_some() {
                return Err(format!("场景模块编号 {index:02} 有多个文件"));
            }
        }
    }

    let expected = (0u8..=29).collect::<BTreeSet<_>>();
    let actual = paths.keys().copied().collect::<BTreeSet<_>>();
    if actual != expected {
        let missing = expected
            .difference(&actual)
            .map(|value| format!("{value:02}"))
            .collect::<Vec<_>>();
        return Err(format!(
            "场景模块集合不完整；缺少 TEXT{}.COM",
            missing.join(".COM、TEXT")
        ));
    }

    let mut modules = Vec::with_capacity(paths.len());
    for path in paths.into_values() {
        let compressed = source
            .files
            .get(&path)
            .ok_or_else(|| format!("内部错误：缺少场景模块 {path}"))?
            .clone();
        let decoded =
            lzss::decode_file(&compressed).map_err(|error| format!("解开 {path} 失败：{error}"))?;
        let report = analysis::analyze_module(&decoded.bytes)
            .map_err(|error| format!("反汇编 {path} 失败：{error}"))?;
        if report.parser_calls.is_empty() {
            return Err(format!("{path} 中没有确认到可达的 PS1SYS slot-54 文本调用"));
        }
        let mut by_start = BTreeMap::<usize, ParsedTextRecord>::new();
        for call in &report.parser_calls {
            let targets = analysis::resolve_slot54_targets(&decoded.bytes, &report, call).map_err(
                |error| {
                    format!(
                        "{path} slot-54 调用 0x{:X} 的静态文本目标无法闭合：{error}",
                        call.code_output_offset
                    )
                },
            )?;
            if targets.is_empty() {
                return Err(format!(
                    "{path} slot-54 调用 0x{:X} 没有确认到静态文本目标",
                    call.code_output_offset
                ));
            }
            for target in targets {
                let start = target.output_offset;
                let record = text::parse_message(&decoded.bytes, start).map_err(|error| {
                    format!(
                        "{path} slot-54 调用 0x{:X} 的目标 0x{start:X} 无法解析：{error}",
                        call.code_output_offset
                    )
                })?;
                let segments = project_text_record(&record).map_err(|error| {
                    format!(
                        "{path} slot-54 调用 0x{:X} 的译文投影失败：{error}",
                        call.code_output_offset
                    )
                })?;
                let use_record = parser_use(&target);
                match by_start.get_mut(&start) {
                    Some(existing) if existing.record == record => {
                        if !existing.parser_calls.contains(&use_record) {
                            existing.parser_calls.push(use_record);
                        }
                    }
                    Some(_) => {
                        return Err(format!(
                            "{path} 的同一文本偏移 0x{start:X} 解析出不一致的记录"
                        ));
                    }
                    None => {
                        by_start.insert(
                            start,
                            ParsedTextRecord {
                                record,
                                parser_calls: vec![use_record],
                                segments,
                            },
                        );
                    }
                }
            }
        }

        let records = by_start.into_values().collect::<Vec<_>>();
        for pair in records.windows(2) {
            if pair[0].record.source_end > pair[1].record.source_offset {
                return Err(format!(
                    "{path} 的 slot-54 记录在 0x{:X} 和 0x{:X} 处重叠，无法安全独立回注",
                    pair[0].record.source_offset, pair[1].record.source_offset
                ));
            }
        }

        let menu_tables = analysis::find_local_menu_tables(&decoded.bytes, &report);
        modules.push(ParsedModule {
            path,
            compressed,
            expanded: decoded.bytes,
            analysis: report,
            menu_tables,
            records,
        });
    }
    Ok(modules)
}

fn project_text_record(record: &text::TextRecord) -> Result<Vec<ProjectedSegment>> {
    let mut projected = Vec::<ProjectedSegment>::new();
    let mut pending_prefix = Vec::<Vec<u8>>::new();
    let mut pending_prefix_offset = None;
    for page in split_text_pages(&record.tokens) {
        if let Some(mut segment) = project_text_page(&page)? {
            if segment.entry.name.is_none() && segment.entry.message.is_empty() {
                if let Some(previous) = projected.last_mut() {
                    previous
                        .plan
                        .suffix_controls
                        .push(segment.original_bytes.clone());
                    previous
                        .original_bytes
                        .extend_from_slice(&segment.original_bytes);
                    previous.source_end = segment.source_end;
                } else {
                    pending_prefix_offset.get_or_insert(segment.source_offset);
                    pending_prefix.push(segment.original_bytes);
                }
                continue;
            }

            if !pending_prefix.is_empty() {
                let mut prefix = Vec::new();
                for bytes in pending_prefix.drain(..) {
                    prefix.extend_from_slice(&bytes);
                }
                segment.plan.prefix_controls.insert(0, prefix.clone());
                let mut original_bytes = prefix;
                original_bytes.extend_from_slice(&segment.original_bytes);
                segment.original_bytes = original_bytes;
                segment.source_offset = pending_prefix_offset
                    .take()
                    .unwrap_or(segment.source_offset);
            }
            projected.push(segment);
        }
    }
    if !pending_prefix.is_empty() {
        if let Some(previous) = projected.last_mut() {
            for bytes in pending_prefix {
                previous.plan.suffix_controls.push(bytes.clone());
                previous.original_bytes.extend_from_slice(&bytes);
            }
            previous.source_end = record.terminator_offset;
        }
    }
    Ok(projected)
}

fn split_text_pages(tokens: &[text::TextToken]) -> Vec<Vec<text::TextToken>> {
    let mut pages = Vec::new();
    let mut current = Vec::new();
    let mut has_text = false;
    for token in tokens {
        current.push(token.clone());
        has_text |= is_projection_text_token(&token.kind);
        if has_text && is_page_boundary(&token.kind) {
            pages.push(std::mem::take(&mut current));
            has_text = false;
        }
    }
    if !current.is_empty() {
        if has_text {
            pages.push(current);
        } else if let Some(previous) = pages.last_mut() {
            previous.extend(current);
        }
    }
    pages
}

fn is_projection_text_token(kind: &text::TextTokenKind) -> bool {
    matches!(
        kind,
        text::TextTokenKind::Glyph { .. }
            | text::TextTokenKind::Substitution { .. }
            | text::TextTokenKind::Raw {
                reason: text::RawReason::UnmappedGlyph | text::RawReason::MalformedControlPrefix
            }
    )
}

fn is_page_boundary(kind: &text::TextTokenKind) -> bool {
    matches!(
        kind,
        text::TextTokenKind::WaitInput | text::TextTokenKind::ClearWindow
    )
}

fn is_hidden_projection_control(kind: &text::TextTokenKind) -> bool {
    matches!(
        kind,
        text::TextTokenKind::CursorAlign
            | text::TextTokenKind::ClearWindow
            | text::TextTokenKind::WaitInput
            | text::TextTokenKind::WaitParameter { .. }
            | text::TextTokenKind::GlyphWaitMode
            | text::TextTokenKind::AlternateWaitMode
            | text::TextTokenKind::Raw {
                reason: text::RawReason::SkippedControl
            }
    )
}

fn project_text_page(tokens: &[text::TextToken]) -> Result<Option<ProjectedSegment>> {
    if !tokens
        .iter()
        .any(|token| is_projection_text_token(&token.kind))
    {
        return Ok(None);
    }
    let source_offset = tokens
        .first()
        .map(|token| token.source_offset)
        .ok_or_else(|| "empty text page".to_owned())?;
    let source_end = tokens
        .last()
        .map(|token| token.source_offset + token.raw_bytes.len())
        .ok_or_else(|| "empty text page".to_owned())?;
    let original_bytes = tokens
        .iter()
        .flat_map(|token| token.raw_bytes.iter().copied())
        .collect::<Vec<_>>();

    let mut cursor = 0usize;
    let mut prefix_controls = Vec::new();
    while cursor < tokens.len() && is_hidden_projection_control(&tokens[cursor].kind) {
        prefix_controls.push(tokens[cursor].raw_bytes.clone());
        cursor += 1;
    }

    let mut name = None;
    let mut after_name_controls = Vec::new();
    if let Some((name_plan, next)) = find_bracketed_name(tokens, cursor) {
        name = Some(name_plan);
        cursor = next;
        while cursor < tokens.len() && is_hidden_projection_control(&tokens[cursor].kind) {
            after_name_controls.push(tokens[cursor].raw_bytes.clone());
            cursor += 1;
        }
    } else if let Some((name_plan, next)) = find_substitution_name(tokens, cursor) {
        name = Some(name_plan);
        cursor = next;
        while cursor < tokens.len() && is_hidden_projection_control(&tokens[cursor].kind) {
            after_name_controls.push(tokens[cursor].raw_bytes.clone());
            cursor += 1;
        }
    }

    let mut lines = Vec::<MessageLinePlan>::new();
    let mut line = ProjectionLineBuilder::default();
    let mut suffix_controls = Vec::new();
    let mut read_only_reason = None;

    for token in &tokens[cursor..] {
        match &token.kind {
            text::TextTokenKind::Glyph { text: glyph, .. } => {
                line.bytes.extend_from_slice(&token.raw_bytes);
                line.atoms.push(token.raw_bytes.clone());
                line.atom_texts.push(glyph.clone());
                line.atom_count += 1;
            }
            text::TextTokenKind::Substitution { slot } => {
                let placeholder = format!("{{{{SUB:{slot}}}}}");
                line.bytes.extend_from_slice(&token.raw_bytes);
                line.atoms.push(token.raw_bytes.clone());
                line.atom_texts.push(placeholder);
                line.substitutions.push(*slot);
                line.atom_count += 1;
            }
            text::TextTokenKind::Newline => {
                let (projected_line, line_read_only) = finish_projection_line(
                    std::mem::take(&mut line),
                    Some(token.raw_bytes.clone()),
                );
                if let Some(reason) = line_read_only {
                    read_only_reason.get_or_insert(reason);
                }
                lines.push(projected_line);
            }
            text::TextTokenKind::Raw {
                reason: text::RawReason::UnmappedGlyph,
            } => {
                let marker = format!("[[RAW:{}]]", hex_upper(&token.raw_bytes));
                line.bytes.extend_from_slice(&token.raw_bytes);
                line.atoms.push(token.raw_bytes.clone());
                line.atom_texts.push(marker);
                line.allowed_raw.push(token.raw_bytes.clone());
                line.atom_count += 1;
            }
            text::TextTokenKind::Raw {
                reason: text::RawReason::MalformedControlPrefix,
            } => {
                let marker = format!("[[RAW:{}]]", hex_upper(&token.raw_bytes));
                line.bytes.extend_from_slice(&token.raw_bytes);
                line.atoms.push(token.raw_bytes.clone());
                line.atom_texts.push(marker);
                line.allowed_raw.push(token.raw_bytes.clone());
                line.atom_count += 1;
                read_only_reason
                    .get_or_insert_with(|| "畸形控制前缀所在页面保留原文，不能编辑".to_owned());
            }
            text::TextTokenKind::Raw {
                reason: text::RawReason::SkippedControl,
            }
            | text::TextTokenKind::CursorAlign
            | text::TextTokenKind::WaitParameter { .. }
            | text::TextTokenKind::GlyphWaitMode
            | text::TextTokenKind::AlternateWaitMode => {
                line.controls.push(ProjectionControlAnchor {
                    atom_index: line.atom_count,
                    raw_bytes: token.raw_bytes.clone(),
                });
            }
            text::TextTokenKind::WaitInput | text::TextTokenKind::ClearWindow => {
                suffix_controls.push(token.raw_bytes.clone());
            }
        }
    }
    if line.atom_count > 0
        || !line.controls.is_empty()
        || lines.is_empty()
        || lines
            .last()
            .is_some_and(|line| line.newline_after.is_some())
    {
        let (projected_line, line_read_only) = finish_projection_line(line, None);
        if let Some(reason) = line_read_only {
            read_only_reason.get_or_insert(reason);
        }
        lines.push(projected_line);
    }

    let mut name_entry = None;
    let name_bytes = match &name {
        None => Vec::new(),
        Some(NamePlan::StaticBracketed {
            left_bytes,
            original_bytes,
            original_text,
            right_bytes,
            ..
        }) => {
            let mut bytes = left_bytes.clone();
            bytes.extend_from_slice(original_bytes);
            bytes.extend_from_slice(right_bytes);
            name_entry = Some(original_text.clone());
            bytes
        }
        Some(NamePlan::DynamicSubstitution { slot, raw_bytes }) => {
            name_entry = Some(format!("{{{{SUB:{slot}}}}}"));
            raw_bytes.clone()
        }
    };
    let message = projection_lines_text(&lines);
    let mut reconstructed = Vec::new();
    reconstructed.extend(prefix_controls.iter().flatten().copied());
    reconstructed.extend_from_slice(&name_bytes);
    reconstructed.extend(after_name_controls.iter().flatten().copied());
    append_projection_lines(&mut reconstructed, &lines)?;
    reconstructed.extend(suffix_controls.iter().flatten().copied());
    if reconstructed != original_bytes {
        return Err(format!(
            "投影重组与源字节不一致，记录从 0x{source_offset:X} 到 0x{source_end:X}"
        ));
    }

    Ok(Some(ProjectedSegment {
        entry: Entry {
            name: name_entry,
            message,
        },
        source_offset,
        source_end,
        original_bytes,
        plan: SegmentPlan {
            prefix_controls,
            name,
            after_name_controls,
            lines,
            suffix_controls,
            read_only_reason,
        },
    }))
}

fn find_bracketed_name(tokens: &[text::TextToken], start: usize) -> Option<(NamePlan, usize)> {
    if !matches!(
        &tokens.get(start)?.kind,
        text::TextTokenKind::Glyph { text, .. } if text == "【"
    ) {
        return None;
    }
    let left_bytes = tokens[start].raw_bytes.clone();
    let mut name_text = String::new();
    let mut name_bytes = Vec::new();
    let mut cursor = start + 1;
    while let Some(token) = tokens.get(cursor) {
        match &token.kind {
            text::TextTokenKind::Glyph { text, .. } if text == "】" => {
                let right_bytes = token.raw_bytes.clone();
                let next = skip_hidden_controls(tokens, cursor + 1);
                if !tokens.get(next).is_some_and(is_quote_opening) {
                    return None;
                }
                if name_text.is_empty() {
                    return None;
                }
                return Some((
                    NamePlan::StaticBracketed {
                        left_bytes,
                        original_bytes: name_bytes,
                        original_text: name_text,
                        right_bytes,
                        allowed_raw: Vec::new(),
                    },
                    cursor + 1,
                ));
            }
            text::TextTokenKind::Glyph { text, .. } => {
                name_text.push_str(text);
                name_bytes.extend_from_slice(&token.raw_bytes);
            }
            _ => return None,
        }
        cursor += 1;
    }
    None
}

fn find_substitution_name(tokens: &[text::TextToken], start: usize) -> Option<(NamePlan, usize)> {
    let token = tokens.get(start)?;
    let slot = match &token.kind {
        text::TextTokenKind::Substitution { slot } => *slot,
        _ => return None,
    };
    let next = skip_hidden_controls(tokens, start + 1);
    if !tokens.get(next).is_some_and(is_quote_opening) {
        return None;
    }
    Some((
        NamePlan::DynamicSubstitution {
            slot,
            raw_bytes: token.raw_bytes.clone(),
        },
        start + 1,
    ))
}

fn is_quote_opening(token: &text::TextToken) -> bool {
    matches!(
        &token.kind,
        text::TextTokenKind::Glyph { text, .. }
            if matches!(text.as_str(), "「" | "『" | "“" | "‘")
    )
}

fn skip_hidden_controls(tokens: &[text::TextToken], mut index: usize) -> usize {
    while tokens
        .get(index)
        .is_some_and(|token| is_hidden_projection_control(&token.kind))
    {
        index += 1;
    }
    index
}

fn finish_projection_line(
    line: ProjectionLineBuilder,
    newline_after: Option<Vec<u8>>,
) -> (MessageLinePlan, Option<String>) {
    let all_bytes = line.atoms.iter().flatten().copied().collect::<Vec<_>>();
    if line.atom_texts.len() != line.atoms.len()
        || line.atom_count != line.atoms.len()
        || line.bytes != all_bytes
    {
        return (
            MessageLinePlan {
                original_text: String::new(),
                original_atoms: Vec::new(),
                all_original_atoms: line.atoms,
                original_controls: line.controls.clone(),
                leading_padding: Vec::new(),
                trailing_padding: Vec::new(),
                controls_before_padding: Vec::new(),
                controls_after_padding: Vec::new(),
                controls: Vec::new(),
                controls_before_trailing: Vec::new(),
                controls_after_trailing: Vec::new(),
                layout_only: true,
                newline_after,
                allowed_raw: line.allowed_raw,
                allowed_substitutions: line.substitutions,
            },
            Some("字形原子与来源字节数量不一致".into()),
        );
    }

    let total = line.atoms.len();
    let mut leading_end = 0usize;
    while leading_end < total && is_whitespace_atom(&line.atom_texts[leading_end]) {
        leading_end += 1;
    }
    let mut visible_end = total;
    while visible_end > leading_end && is_whitespace_atom(&line.atom_texts[visible_end - 1]) {
        visible_end -= 1;
    }
    let layout_only = leading_end == visible_end;
    let leading_padding = line.atoms[..leading_end].to_vec();
    let trailing_padding = line.atoms[visible_end..].to_vec();
    let original_atoms = line.atoms[leading_end..visible_end].to_vec();
    let original_text = line.atom_texts[leading_end..visible_end].concat();

    let mut controls_before_padding = Vec::new();
    let mut controls_after_padding = Vec::new();
    let mut controls = Vec::new();
    let mut controls_before_trailing = Vec::new();
    let mut controls_after_trailing = Vec::new();
    let mut read_only_reason = None;
    for control in &line.controls {
        let position = control.atom_index;
        if position == 0 {
            controls_before_padding.push(control.raw_bytes.clone());
        } else if position < leading_end {
            read_only_reason
                .get_or_insert_with(|| "状态控制位于隐藏缩进内部，所在文本行只读".to_owned());
        } else if position == leading_end && leading_end > 0 {
            controls_after_padding.push(control.raw_bytes.clone());
        } else if position > leading_end && position < visible_end {
            controls.push(ProjectionControlAnchor {
                atom_index: position - leading_end,
                raw_bytes: control.raw_bytes.clone(),
            });
        } else if position == visible_end && trailing_padding.len() > 0 {
            controls_before_trailing.push(control.raw_bytes.clone());
        } else if position > visible_end && position < total {
            read_only_reason
                .get_or_insert_with(|| "状态控制位于隐藏结尾空格内部，所在文本行只读".to_owned());
        } else if position == total && trailing_padding.len() > 0 {
            controls_after_trailing.push(control.raw_bytes.clone());
        } else {
            controls.push(ProjectionControlAnchor {
                atom_index: position
                    .saturating_sub(leading_end)
                    .min(original_atoms.len()),
                raw_bytes: control.raw_bytes.clone(),
            });
        }
    }

    let plan = MessageLinePlan {
        original_text,
        original_atoms,
        all_original_atoms: line.atoms,
        original_controls: line.controls,
        leading_padding,
        trailing_padding,
        controls_before_padding,
        controls_after_padding,
        controls,
        controls_before_trailing,
        controls_after_trailing,
        layout_only,
        newline_after,
        allowed_raw: line.allowed_raw,
        allowed_substitutions: line.substitutions,
    };
    (plan, read_only_reason)
}

fn is_whitespace_atom(atom: &str) -> bool {
    !atom.is_empty() && atom.chars().all(char::is_whitespace)
}

fn projection_lines_text(lines: &[MessageLinePlan]) -> String {
    let mut message = String::new();
    let visible = lines
        .iter()
        .filter(|line| !line.layout_only)
        .collect::<Vec<_>>();
    for (index, line) in visible.iter().enumerate() {
        message.push_str(&line.original_text);
        if index + 1 < visible.len() {
            message.push('\n');
        }
    }
    message
}

fn append_projection_lines(output: &mut Vec<u8>, lines: &[MessageLinePlan]) -> Result<()> {
    for line in lines {
        if line
            .original_controls
            .iter()
            .any(|control| control.atom_index > line.all_original_atoms.len())
        {
            return Err("内部错误：原文投影的可见字形数发生变化".into());
        }
        append_original_projection_line(output, line)?;
        if let Some(newline) = &line.newline_after {
            output.extend_from_slice(newline);
        }
    }
    Ok(())
}

fn append_original_projection_line(output: &mut Vec<u8>, line: &MessageLinePlan) -> Result<()> {
    let mut controls = line.original_controls.iter().collect::<Vec<_>>();
    controls.sort_by_key(|control| control.atom_index);
    let mut control_index = 0usize;
    for atom_index in 0..=line.all_original_atoms.len() {
        while controls
            .get(control_index)
            .is_some_and(|control| control.atom_index == atom_index)
        {
            output.extend_from_slice(&controls[control_index].raw_bytes);
            control_index += 1;
        }
        if let Some(atom) = line.all_original_atoms.get(atom_index) {
            output.extend_from_slice(atom);
        }
    }
    if control_index != controls.len() {
        return Err("源控制锚点超出文本行长度".into());
    }
    Ok(())
}

fn rebuild_projected_segment(
    segment: &ProjectedSegment,
    edited: &Entry,
    encoding: &EncodingPlan,
) -> Result<Vec<u8>> {
    if edited == &segment.entry {
        return Ok(segment.original_bytes.clone());
    }
    if segment.plan.read_only_reason.is_some() && edited != &segment.entry {
        return Err(segment.plan.read_only_reason.clone().unwrap_or_default());
    }
    if edited.name.is_some() != segment.entry.name.is_some() {
        return Err("不能改变 name 字段是否存在".into());
    }

    let mut output = Vec::with_capacity(segment.original_bytes.len());
    output.extend(segment.plan.prefix_controls.iter().flatten().copied());
    match &segment.plan.name {
        None => {}
        Some(NamePlan::StaticBracketed {
            left_bytes,
            original_bytes,
            original_text,
            right_bytes,
            allowed_raw,
        }) => {
            output.extend_from_slice(left_bytes);
            let edited_name = edited.name.as_deref().ok_or("name 字段缺失")?;
            let encoded_name_atoms = if edited_name == original_text {
                vec![original_bytes.clone()]
            } else {
                encode_projection_field(edited_name, allowed_raw, "name", encoding)?
            };
            append_projection_atoms(&mut output, &encoded_name_atoms, &[])?;
            output.extend_from_slice(right_bytes);
        }
        Some(NamePlan::DynamicSubstitution { slot, raw_bytes }) => {
            let expected = format!("{{{{SUB:{slot}}}}}");
            if edited.name.as_deref() != Some(expected.as_str()) {
                return Err(format!(
                    "动态 name 槽位 {slot} 只读；译文中请保留 {expected}"
                ));
            }
            output.extend_from_slice(raw_bytes);
        }
    }
    output.extend(segment.plan.after_name_controls.iter().flatten().copied());

    let expected_lines = segment
        .plan
        .lines
        .iter()
        .filter(|line| !line.layout_only)
        .count();
    let edited_lines = if expected_lines == 0 && edited.message.is_empty() {
        Vec::new()
    } else {
        edited.message.split('\n').collect::<Vec<_>>()
    };
    if edited_lines.len() != expected_lines {
        return Err(format!(
            "译文有 {} 行；源记录有 {} 个可见文本行",
            edited_lines.len(),
            expected_lines
        ));
    }
    let mut edited_line_index = 0usize;
    for plan in &segment.plan.lines {
        if plan.layout_only {
            append_original_projection_line(&mut output, plan)?;
            if let Some(newline) = &plan.newline_after {
                output.extend_from_slice(newline);
            }
            continue;
        }
        let edited_line = edited_lines[edited_line_index];
        edited_line_index += 1;
        let atoms = if *edited_line == plan.original_text {
            plan.original_atoms.clone()
        } else {
            validate_substitution_slots(edited_line, &plan.allowed_substitutions)?;
            encode_projection_field(edited_line, &plan.allowed_raw, "message", encoding)?
        };
        output.extend(plan.controls_before_padding.iter().flatten().copied());
        output.extend(plan.leading_padding.iter().flatten().copied());
        output.extend(plan.controls_after_padding.iter().flatten().copied());
        append_projection_atoms(&mut output, &atoms, &plan.controls)?;
        output.extend(plan.controls_before_trailing.iter().flatten().copied());
        output.extend(plan.trailing_padding.iter().flatten().copied());
        output.extend(plan.controls_after_trailing.iter().flatten().copied());
        if let Some(newline) = &plan.newline_after {
            output.extend_from_slice(newline);
        }
    }
    output.extend(segment.plan.suffix_controls.iter().flatten().copied());
    Ok(output)
}

fn encode_projection_field(
    field: &str,
    allowed_raw: &[Vec<u8>],
    label: &str,
    encoding: &EncodingPlan,
) -> Result<Vec<Vec<u8>>> {
    text::encode_projection_atoms_with(field, allowed_raw, |character, offset| {
        encode_pc98_glyph(character, offset, encoding)
    })
    .map_err(|error| format!("{label}：{error}"))
}

fn encode_pc98_glyph(
    character: char,
    message_offset: usize,
    encoding: &EncodingPlan,
) -> Result<Vec<u8>> {
    if !character.is_ascii() {
        return encoding
            .encode_cp932(&character.to_string())
            .map_err(|error| format!("{error} (message byte 0x{message_offset:X})"));
    }

    if matches!(character, '!' | '#' | '@' | '$') {
        let fullwidth = char::from_u32(character as u32 + 0xFEE0)
            .ok_or_else(|| format!("无法转换 ASCII 控制字符 {character:?} 的全角字形"))?;
        return encoding
            .encode_cp932(&fullwidth.to_string())
            .map_err(|error| format!("{error} (message byte 0x{message_offset:X})"));
    }

    text::encode_native_projection_character(character, message_offset)
}

fn validate_substitution_slots(message: &str, allowed: &[u8]) -> Result<()> {
    let mut found = Vec::new();
    let mut cursor = 0usize;
    while cursor < message.len() {
        let rest = &message[cursor..];
        if let Some(after_prefix) = rest.strip_prefix("{{SUB:") {
            let close = after_prefix
                .find("}}")
                .ok_or_else(|| "SUB 占位符缺少结束标记".to_owned())?;
            let value = &after_prefix[..close];
            if value.len() != 1 || !value.as_bytes()[0].is_ascii_digit() {
                return Err(format!("无效的 SUB 占位符：{{{{SUB:{value}}}}}"));
            }
            found.push(value.as_bytes()[0] - b'0');
            cursor += "{{SUB:".len() + close + "}}".len();
            continue;
        }
        let character = rest.chars().next().ok_or("无效的 UTF-8 边界")?;
        cursor += character.len_utf8();
    }

    let mut expected = allowed.to_vec();
    found.sort_unstable();
    expected.sort_unstable();
    if found != expected {
        return Err("不能新增、删除或替换源记录中的运行时 SUB 槽位".into());
    }
    Ok(())
}

fn append_projection_atoms(
    output: &mut Vec<u8>,
    atoms: &[Vec<u8>],
    controls: &[ProjectionControlAnchor],
) -> Result<()> {
    let mut controls = controls.iter().collect::<Vec<_>>();
    controls.sort_by_key(|control| control.atom_index.min(atoms.len()));
    let mut control_index = 0usize;
    for atom_index in 0..=atoms.len() {
        while controls
            .get(control_index)
            .is_some_and(|control| control.atom_index.min(atoms.len()) == atom_index)
        {
            output.extend_from_slice(&controls[control_index].raw_bytes);
            control_index += 1;
        }
        if let Some(atom) = atoms.get(atom_index) {
            output.extend_from_slice(atom);
        }
    }
    Ok(())
}

fn primary_module(module: &ParsedModule) -> PrimaryModule {
    PrimaryModule {
        path: module.path.clone(),
        compressed_size: module.compressed.len(),
        source_sha256: sha256_hex(&module.compressed),
        expanded_size: module.expanded.len(),
        expanded_sha256: sha256_hex(&module.expanded),
        instruction_count: module.analysis.instructions.len(),
        edge_count: module.analysis.edges.len(),
        resolved_indirect_jump_tables: module.analysis.resolved_indirect_jump_tables.clone(),
        parser_calls: module.analysis.parser_calls.clone(),
        diagnostics: module.analysis.diagnostics.clone(),
        direct_menu_tables: module.menu_tables.clone(),
        records: module.records.iter().map(primary_text_record).collect(),
    }
}

fn build_module_bindings(modules: &[ParsedModule]) -> Vec<ModuleBinding> {
    modules
        .iter()
        .map(|module| ModuleBinding {
            path: module.path.clone(),
            translation_path: translation_path(&module.path)
                .expect("module file name was validated before binding"),
            source_sha256: sha256_hex(&module.compressed),
            expanded_sha256: sha256_hex(&module.expanded),
            expanded_size: module.expanded.len(),
            records: module
                .records
                .iter()
                .map(|record| RecordBinding {
                    source_offset: record.record.source_offset,
                    source_end: record.record.source_end,
                    raw_sha256: sha256_hex(&record.record.raw_bytes),
                    parser_calls: record.parser_calls.clone(),
                    translation_segments: record
                        .segments
                        .iter()
                        .enumerate()
                        .map(|(segment_index, segment)| SegmentBinding {
                            segment_index,
                            source_offset: segment.source_offset,
                            source_end: segment.source_end,
                            name_sha256: segment
                                .entry
                                .name
                                .as_ref()
                                .map(|name| sha256_hex(name.as_bytes())),
                            message_sha256: sha256_hex(segment.entry.message.as_bytes()),
                            read_only_reason: segment.plan.read_only_reason.clone(),
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect()
}

fn parser_use(target: &StaticTextTarget) -> ParserUse {
    ParserUse {
        function_entry_output_offset: target.function_entry_output_offset,
        code_callsite_output_offset: target.code_callsite_output_offset,
        window: target.window,
        window_expr: target.window_expr.clone(),
        segment: target.segment.clone(),
        runtime_pointer: target.runtime_pointer,
        output_offset: target.output_offset,
        pointer_expr: target.pointer_expr.clone(),
        pointer_source: target.pointer_source.clone(),
        push_output_offsets: target.push_output_offsets,
        definition_offsets: target.definition_offsets.clone(),
        path_offsets: target.path_offsets.clone(),
        target_raw_sha256: sha256_hex(&target.raw_text),
        evidence_paths: target.evidence_paths.clone(),
    }
}

fn primary_text_record(parsed: &ParsedTextRecord) -> PrimaryTextRecord {
    let record = &parsed.record;
    let mut glyph_jis_hex = String::new();
    let mut non_glyph_tokens = Vec::new();
    for token in &record.tokens {
        match &token.kind {
            text::TextTokenKind::Glyph { jis_code, .. } => {
                use std::fmt::Write as _;
                let _ = write!(glyph_jis_hex, "{jis_code:04X}");
            }
            kind => non_glyph_tokens.push(PrimaryTextToken {
                source_offset: token.source_offset,
                raw_hex: hex_upper(&token.raw_bytes),
                kind: kind.clone(),
            }),
        }
    }
    PrimaryTextRecord {
        source_offset: record.source_offset,
        source_end: record.source_end,
        terminator_offset: record.terminator_offset,
        terminator: record.terminator,
        raw_hex: hex_upper(&record.raw_bytes),
        message: record.message.clone(),
        glyph_jis_hex,
        non_glyph_tokens,
        parser_calls: parsed.parser_calls.clone(),
        translation_segments: parsed
            .segments
            .iter()
            .map(|segment| PrimaryProjectionSegment {
                source_offset: segment.source_offset,
                source_end: segment.source_end,
                name: segment.entry.name.clone(),
                message: segment.entry.message.clone(),
                read_only_reason: segment.plan.read_only_reason.clone(),
            })
            .collect(),
    }
}

fn translation_entries(module: &ParsedModule) -> Vec<Entry> {
    module
        .records
        .iter()
        .flat_map(|record| record.segments.iter().map(|segment| segment.entry.clone()))
        .collect()
}

fn translation_path(module_path: &str) -> Result<String> {
    let path = Path::new(module_path);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("场景模块路径缺少 Unicode 文件名：{module_path}"))?;
    Ok(format!("translations/{name}.json"))
}

fn hash_source_tree(bindings: &[SourceFileBinding]) -> String {
    let mut canonical = Vec::new();
    for file in bindings {
        let path = file.path.as_bytes();
        canonical.extend_from_slice(&(path.len() as u64).to_le_bytes());
        canonical.extend_from_slice(path);
        canonical.extend_from_slice(&(file.size as u64).to_le_bytes());
        canonical.extend_from_slice(file.sha256.as_bytes());
    }
    sha256_hex(&canonical)
}

fn hex_upper(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(result, "{byte:02X}");
    }
    result
}

fn analysis_warnings(modules: &[ParsedModule]) -> Vec<String> {
    let affected_modules = modules
        .iter()
        .filter(|module| !module.analysis.diagnostics.is_empty())
        .count();
    let diagnostic_count = modules
        .iter()
        .map(|module| module.analysis.diagnostics.len())
        .sum::<usize>();
    if diagnostic_count == 0 {
        Vec::new()
    } else {
        vec![format!(
            "保留了 {diagnostic_count} 条静态控制流/调用参数诊断（涉及 {affected_modules} 个模块）；未确认的间接代码路径可能仍含未提取文本，详见 primary/extraction.json"
        )]
    }
}

fn verify_workspace_sources(root: &Path, files: &[SourceFileBinding]) -> Result<()> {
    for binding in files {
        let path = root.join("source").join(safe_relative(&binding.path)?);
        let bytes = read_regular_file(&path, "工作区来源快照")?;
        if bytes.len() != binding.size || sha256_hex(&bytes) != binding.sha256 {
            return Err(format!("工作区来源快照与清单不符：{}", binding.path));
        }
    }
    Ok(())
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("读取{label}失败 {}：{error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("{label}必须是实际目录：{}", path.display()));
    }
    fs::canonicalize(path).map_err(|error| format!("解析{label}失败 {}：{error}", path.display()))
}

fn canonical_regular_path(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("读取{label}失败 {}：{error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("{label}必须是普通文件：{}", path.display()));
    }
    fs::canonicalize(path).map_err(|error| format!("解析{label}失败 {}：{error}", path.display()))
}

fn collect_translation_inputs(selection: &Path, translation_root: &Path) -> Result<Vec<PathBuf>> {
    let metadata = fs::symlink_metadata(selection)
        .map_err(|error| format!("读取翻译输入失败 {}：{error}", selection.display()))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("翻译输入不能是符号链接：{}", selection.display()));
    }
    let canonical = fs::canonicalize(selection)
        .map_err(|error| format!("解析翻译输入失败 {}：{error}", selection.display()))?;
    if !canonical.starts_with(translation_root) {
        return Err(format!(
            "翻译文件或目录必须位于工作区 translations 目录内：{}",
            canonical.display()
        ));
    }

    if metadata.is_file() {
        if canonical
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("json")
        {
            return Err(format!("所选文件不是 JSON：{}", canonical.display()));
        }
        return Ok(vec![canonical]);
    }
    if !metadata.is_dir() {
        return Err(format!(
            "翻译输入必须是普通 JSON 文件或目录：{}",
            selection.display()
        ));
    }

    let mut files = Vec::new();
    collect_translation_directory(&canonical, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_translation_directory(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("遍历翻译目录失败 {}：{error}", directory.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| format!("读取翻译目录项失败 {}：{error}", directory.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("读取翻译文件元数据失败 {}：{error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("翻译目录中不支持符号链接：{}", path.display()));
        }
        if metadata.is_dir() {
            collect_translation_directory(&path, files)?;
        } else if metadata.is_file() {
            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                return Err(format!("翻译目录中包含非 JSON 文件：{}", path.display()));
            }
            files.push(
                fs::canonicalize(&path)
                    .map_err(|error| format!("解析翻译 JSON 失败 {}：{error}", path.display()))?,
            );
        } else {
            return Err(format!("翻译目录中不支持特殊文件：{}", path.display()));
        }
    }
    Ok(())
}

fn read_regular_file(path: &Path, label: &str) -> Result<Vec<u8>> {
    let path = canonical_regular_path(path, label)?;
    fs::read(&path).map_err(|error| format!("读取{label}失败 {}：{error}", path.display()))
}

fn normalized_relative(path: &Path) -> Result<String> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("不安全的相对路径：{}", path.display()));
    }
    let components = path
        .components()
        .map(|component| {
            component
                .as_os_str()
                .to_str()
                .ok_or_else(|| format!("路径包含非 Unicode 文件名：{}", path.display()))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(components.join("/"))
}

fn safe_relative(raw: &str) -> Result<PathBuf> {
    let normalized = raw.replace('\\', "/");
    let path = Path::new(&normalized);
    if normalized.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("不安全的工作区相对路径：{raw}"));
    }
    Ok(path.to_path_buf())
}
