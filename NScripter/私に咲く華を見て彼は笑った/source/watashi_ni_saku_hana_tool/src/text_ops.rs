use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vn_cli::{
    Field, FieldKind, Operation, OperationSpec, Parameters, PreparedOperation, Preview, Progress,
    Result, RunReport, Value,
};

use crate::script::ScriptCatalog;
use crate::transaction;
use crate::{pack_bytes, unpack_bytes};

const WORKSPACE_SCHEMA: &str = "watashi-saku-hana-text-workspace-v1";
const PARSER_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkspaceManifest {
    schema: String,
    parser_version: u32,
    source_file: String,
    source_length: usize,
    source_sha256: String,
    plaintext_sha256: String,
    record_count: usize,
}

pub struct ExportText;

impl Operation for ExportText {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "export",
            "提取脚本文本",
            vec![
                Field::new("source", "原始 nscript.dat", FieldKind::Path).required(),
                Field::new("workspace", "文本工作目录输出", FieldKind::Path).required(),
            ],
        );
        spec.description = "解码 nscript.dat，提取对白、选择项、字符串变量和文字精灵文本；把固定前缀/句末控制符留在原脚本中。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(source) = paths.first() {
            if parameters.get("source").is_none() {
                parameters.set("source", Value::Path(source.clone()));
            }
            if parameters.get("workspace").is_none() {
                parameters.set(
                    "workspace",
                    Value::Path(default_work_path("text_workspace")?),
                );
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let source = fs::canonicalize(parameters.path("source")?)?;
        validate_source_name(&source)?;
        let packed = fs::read(&source)
            .map_err(|error| vn_cli::Error(format!("读取原始 nscript.dat 失败：{error}")))?;
        let plain = unpack_bytes(&packed);
        let catalog = ScriptCatalog::parse(&plain).map_err(vn_cli::Error::from)?;

        let overwrite = parameters.flag("overwrite");
        let (workspace, original_workspace) =
            transaction::prepare_directory_output(parameters.path("workspace")?, overwrite)?;
        transaction::ensure_inputs_disjoint_from_output(&[&source], &workspace)?;

        let manifest = WorkspaceManifest {
            schema: WORKSPACE_SCHEMA.into(),
            parser_version: PARSER_VERSION,
            source_file: source
                .file_name()
                .ok_or("源文件缺少文件名")?
                .to_string_lossy()
                .into_owned(),
            source_length: packed.len(),
            source_sha256: sha256_hex(&packed),
            plaintext_sha256: sha256_hex(&plain),
            record_count: catalog.entries().len(),
        };
        let manifest_bytes = json_bytes(&manifest)?;
        let translation_bytes = vn_text::write_template(catalog.entries())
            .map_err(|error| vn_cli::Error(format!("生成 translation.json 失败：{error}")))?;
        let files = vec![
            (PathBuf::from("manifest.json"), manifest_bytes),
            (PathBuf::from("translation.json"), translation_bytes),
        ];
        let preview = Preview {
            inputs: vec![source.clone()],
            outputs: vec![workspace.clone()],
            steps: vec![
                "校验来源 nscript.dat 与 CP932 明文脚本".into(),
                "识别脚本文本和命令内可见文字".into(),
                "写出 translation.json 与来源身份 manifest.json".into(),
                "将固定位置的控制符保留在原始脚本记录中".into(),
            ],
            details: vec![
                format!("来源 SHA-256：{}", manifest.source_sha256),
                format!("文本记录：{}", manifest.record_count),
                "JSON 顺序绑定到来源顺序；不要重排条目".into(),
            ],
        };
        Ok(Box::new(PreparedExport {
            source,
            packed,
            workspace,
            original_workspace,
            files,
            overwrite,
            preview,
            record_count: manifest.record_count,
        }))
    }
}

struct PreparedExport {
    source: PathBuf,
    packed: Vec<u8>,
    workspace: PathBuf,
    original_workspace: Option<String>,
    files: Vec<(PathBuf, Vec<u8>)>,
    overwrite: bool,
    preview: Preview,
    record_count: usize,
}

impl PreparedOperation for PreparedExport {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("复核 nscript.dat 来源快照")?;
        verify_file_snapshot(&self.source, &self.packed, "nscript.dat")?;
        progress.report("事务式写出翻译工作目录")?;
        let warning = transaction::write_directory_atomic(
            &self.workspace,
            &self.files,
            &self.original_workspace,
            self.overwrite,
        )?;
        Ok(RunReport {
            summary: format!("文本导出完成：{} 条记录。", self.record_count),
            totals: vec![
                ("翻译记录".into(), self.record_count as u64),
                ("工作目录文件".into(), self.files.len() as u64),
            ],
            outputs: vec![self.workspace],
            warnings: warning.into_iter().collect(),
        })
    }
}

pub struct ImportText;

impl Operation for ImportText {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "import",
            "注入译文并重封包",
            vec![
                Field::new("source", "导出时使用的原始 nscript.dat", FieldKind::Path).required(),
                Field::new(
                    "workspace",
                    "包含 manifest.json / translation.json 的工作目录",
                    FieldKind::Path,
                )
                .required(),
                Field::new("output", "译后 nscript.dat 输出文件", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "重解析原始脚本并校验来源哈希和 JSON 形状，按 CP932 安全回注，再执行 XOR 0x84 重封包。"
                .into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(source) = paths.first() {
            if parameters.get("source").is_none() {
                parameters.set("source", Value::Path(source.clone()));
            }
            if parameters.get("workspace").is_none() {
                parameters.set(
                    "workspace",
                    Value::Path(default_work_path("text_workspace")?),
                );
            }
            if parameters.get("output").is_none() {
                parameters.set(
                    "output",
                    Value::Path(default_work_path("nscript-translated.dat")?),
                );
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let source = fs::canonicalize(parameters.path("source")?)?;
        validate_source_name(&source)?;
        let packed = fs::read(&source)
            .map_err(|error| vn_cli::Error(format!("读取原始 nscript.dat 失败：{error}")))?;
        let plain = unpack_bytes(&packed);
        let catalog = ScriptCatalog::parse(&plain).map_err(vn_cli::Error::from)?;

        let workspace = fs::canonicalize(parameters.path("workspace")?)
            .map_err(|error| vn_cli::Error(format!("读取文本工作目录失败：{error}")))?;
        if !fs::metadata(&workspace)?.is_dir() {
            return Err("文本工作区必须是目录".into());
        }
        let manifest_path = workspace.join("manifest.json");
        let translation_path = workspace.join("translation.json");
        let manifest_bytes = read_regular_file(&manifest_path)?;
        let translation_bytes = read_regular_file(&translation_path)?;
        let manifest: WorkspaceManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|error| vn_cli::Error(format!("manifest.json 格式无效：{error}")))?;
        validate_manifest(&manifest, &source, &packed, &plain, catalog.entries().len())?;

        let translations = vn_text::read_template(&translation_bytes)
            .map_err(|error| vn_cli::Error(format!("translation.json 格式无效：{error}")))?;
        vn_text::validate_shape(catalog.entries(), &translations)
            .map_err(|error| vn_cli::Error(format!("译文条目校验失败：{error}")))?;
        let injection = catalog.inject(&translations).map_err(vn_cli::Error::from)?;
        let rebuilt_plain = injection.plain;
        let expected_entries = injection.expected_entries;
        let normalized_literals = injection.normalized_literals;
        let normalized_literal_entries = injection.normalized_literal_entries;
        let wrapped_entries = injection.wrapped_entries;
        let rebuilt_catalog = ScriptCatalog::parse(&rebuilt_plain).map_err(vn_cli::Error::from)?;
        if rebuilt_catalog.entries().len() != expected_entries.len() {
            let first_mismatch = rebuilt_catalog
                .entries()
                .iter()
                .zip(&expected_entries)
                .enumerate()
                .find(|(_, (actual, expected))| actual != expected);
            if let Some((index, (actual, expected))) = first_mismatch {
                return Err(format!(
                    "回注后得到 {} 条文本记录，预期 {} 条；第 {} 条解析为 {:?}，预期 {:?}；未写出结果",
                    rebuilt_catalog.entries().len(),
                    expected_entries.len(),
                    index + 1,
                    actual.message,
                    expected.message
                )
                .into());
            }
            return Err(format!(
                "回注后重新解析得到 {} 条文本记录，预期 {} 条；未写出结果",
                rebuilt_catalog.entries().len(),
                expected_entries.len()
            )
            .into());
        }
        if let Some((index, (actual, expected))) = rebuilt_catalog
            .entries()
            .iter()
            .zip(&expected_entries)
            .enumerate()
            .find(|(_, (actual, expected))| actual != expected)
        {
            return Err(format!(
                "回注后第 {} 条文本记录与译文不一致；解析得到 {:?}，预期 {:?}；未写出结果",
                index + 1,
                actual.message,
                expected.message
            )
            .into());
        }
        let repacked = pack_bytes(&rebuilt_plain);

        let overwrite = parameters.flag("overwrite");
        let (output, original_output) =
            transaction::prepare_file_output(parameters.path("output")?, overwrite)?;
        transaction::ensure_inputs_disjoint_from_output(&[&source, &workspace], &output)?;
        let changed = catalog
            .entries()
            .iter()
            .zip(&translations)
            .filter(|(original, translated)| original.message != translated.message)
            .count();
        let preview = Preview {
            inputs: vec![source.clone(), workspace.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "复核来源哈希、记录数与解析器版本".into(),
                "按原序读取 translation.json 并校验形状".into(),
                "严格编码为 Shift_JIS / CP932 并恢复隐藏的固定控制符".into(),
                "重新解析回注脚本，然后 XOR 0x84 重封包".into(),
                "写入同目录临时文件并安全提交".into(),
            ],
            details: vec![
                format!("译文记录：{}；发生文字改动：{changed}", translations.len()),
                format!("为避免控制语法冲突，全角化普通标点：{normalized_literals} 个字符"),
                format!("自动插入 br 换行的译文条目：{}", wrapped_entries.len()),
                format!(
                    "原始 / 回注明文长度：{} / {} 字节",
                    plain.len(),
                    rebuilt_plain.len()
                ),
                format!("目标封包长度：{} 字节", repacked.len()),
            ],
        };
        Ok(Box::new(PreparedImport {
            source,
            packed,
            manifest_path,
            manifest_bytes,
            translation_path,
            translation_bytes,
            output,
            original_output,
            repacked,
            overwrite,
            preview,
            record_count: translations.len(),
            changed_count: changed,
            normalized_literals,
            normalized_literal_entries,
            wrapped_entries,
        }))
    }
}

struct PreparedImport {
    source: PathBuf,
    packed: Vec<u8>,
    manifest_path: PathBuf,
    manifest_bytes: Vec<u8>,
    translation_path: PathBuf,
    translation_bytes: Vec<u8>,
    output: PathBuf,
    original_output: Option<Vec<u8>>,
    repacked: Vec<u8>,
    overwrite: bool,
    preview: Preview,
    record_count: usize,
    changed_count: usize,
    normalized_literals: usize,
    normalized_literal_entries: Vec<usize>,
    wrapped_entries: Vec<usize>,
}

impl PreparedOperation for PreparedImport {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("复核来源与译文工作区快照")?;
        verify_file_snapshot(&self.source, &self.packed, "nscript.dat")?;
        verify_file_snapshot(&self.manifest_path, &self.manifest_bytes, "manifest.json")?;
        verify_file_snapshot(
            &self.translation_path,
            &self.translation_bytes,
            "translation.json",
        )?;
        progress.report("提交译后 nscript.dat")?;
        transaction::write_file_atomic(
            &self.output,
            &self.repacked,
            &self.original_output,
            self.overwrite,
        )?;
        let mut warnings = Vec::new();
        if self.normalized_literals > 0 {
            warnings.push(format!(
                "{} 个 ASCII 保留标点已在回注结果中转换为全角，涉及译文条目：{}。",
                self.normalized_literals,
                self.normalized_literal_entries
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !self.wrapped_entries.is_empty() {
            warnings.push(format!(
                "已为超出文本窗口宽度的普通对白插入 br 换行，涉及译文条目：{}。",
                self.wrapped_entries
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        Ok(RunReport {
            summary: "文本注入与 nscript.dat 重封包完成。".into(),
            totals: vec![
                ("翻译记录".into(), self.record_count as u64),
                ("修改记录".into(), self.changed_count as u64),
                ("封包字节".into(), self.repacked.len() as u64),
            ],
            outputs: vec![self.output.clone()],
            warnings,
        })
    }
}

fn validate_source_name(source: &Path) -> Result<()> {
    let name = source.file_name().ok_or("源文件缺少文件名")?;
    if !name.to_string_lossy().eq_ignore_ascii_case("nscript.dat") {
        return Err("此操作要求输入本作的 nscript.dat".into());
    }
    Ok(())
}

fn validate_manifest(
    manifest: &WorkspaceManifest,
    source: &Path,
    packed: &[u8],
    plain: &[u8],
    record_count: usize,
) -> Result<()> {
    let source_name = source
        .file_name()
        .ok_or("源文件缺少文件名")?
        .to_string_lossy();
    if manifest.schema != WORKSPACE_SCHEMA
        || manifest.parser_version != PARSER_VERSION
        || !manifest.source_file.eq_ignore_ascii_case(&source_name)
        || manifest.source_length != packed.len()
        || manifest.source_sha256 != sha256_hex(packed)
        || manifest.plaintext_sha256 != sha256_hex(plain)
        || manifest.record_count != record_count
    {
        return Err("工作区 manifest 与当前 nscript.dat 或解析结果不匹配；请重新导出".into());
    }
    Ok(())
}

fn read_regular_file(path: &Path) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| vn_cli::Error(format!("读取 {} 失败：{error}", path.display())))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("工作区输入不是普通文件：{}", path.display()).into());
    }
    fs::read(path).map_err(|error| vn_cli::Error(format!("读取 {} 失败：{error}", path.display())))
}

fn verify_file_snapshot(path: &Path, expected: &[u8], label: &str) -> Result<()> {
    let actual =
        fs::read(path).map_err(|error| vn_cli::Error(format!("重新读取 {label} 失败：{error}")))?;
    if actual != expected {
        return Err(format!("{label} 在预检后发生变化；未提交输出").into());
    }
    Ok(())
}

fn json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| vn_cli::Error(format!("JSON 序列化失败：{error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        output.push(HEX[usize::from(byte >> 4)] as char);
        output.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    output
}

fn default_work_path(name: &str) -> Result<PathBuf> {
    let executable = std::env::current_exe()?;
    let parent = executable.parent().ok_or("无法确定工具程序所在目录")?;
    Ok(parent.join(name))
}
