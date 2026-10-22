use std::fs;
use std::path::{Path, PathBuf};

use vn_cli::{
    Field, FieldKind, Operation, OperationSpec, Parameters, PreparedOperation, Preview, Progress,
    RunReport, Value,
};

use super::{ExportText, ImportText, PackArchive, Unpack};

const UNPACKED_DIR: &str = "scenario_unpacked";
const TRANSLATIONS_DIR: &str = "translations";
const INTERNAL_DIR: &str = ".katahane";
const MANIFEST_NAME: &str = "_katahane_text_manifest.json";

pub struct UnpackAndExtract;

impl Operation for UnpackAndExtract {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "unpack-extract",
            "解包 scenario.arc 并提取文本",
            vec![
                Field::new("archive", "scenario.arc", FieldKind::Path).required(),
                Field::new("workspace", "项目工作目录", FieldKind::Path).required(),
            ],
        );
        spec.description = "解包原始 MJO，生成折叠换行/注音后的对白和独立选择肢翻译 JSON。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(archive) = paths.first() {
            parameters.set("archive", Value::Path(archive.clone()));
            if parameters.get("workspace").is_none() {
                if let Some(parent) = archive.parent() {
                    parameters.set(
                        "workspace",
                        Value::Path(default_workspace(parent).join("workspace")),
                    );
                }
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let archive = fs::canonicalize(parameters.path("archive")?)?;
        if !archive.is_file() {
            return Err("scenario.arc 输入必须是文件".into());
        }
        let workspace = super::resolve_output(parameters.path("workspace")?)?;
        ensure_separate(
            &archive,
            &workspace,
            "工作目录不能包含原始 scenario.arc，也不能覆盖它",
        )?;
        let original_output = super::snapshot_output(&workspace)?;
        let overwrite = parameters.flag("overwrite");
        if !matches!(original_output, super::OutputSnapshot::Missing) && !overwrite {
            return Err("工作目录已存在；请选择新目录或显式启用 --overwrite".into());
        }
        let archive_bytes = fs::metadata(&archive)?.len();
        let preview = Preview {
            inputs: vec![archive.clone()],
            outputs: vec![workspace.clone()],
            steps: vec![
                "按 ARC 结构与对应版本哈希索引解包原始成员".into(),
                "按 MJO 指令边界提取 2112 显示文本".into(),
                "将翻译清单移入隐藏的 .katahane 元数据目录".into(),
                "只提交翻译 JSON 和必要工作文件".into(),
            ],
            details: vec![
                format!("scenario.arc: {} 字节", archive_bytes),
                format!("工作区内目录: {UNPACKED_DIR}/、{TRANSLATIONS_DIR}/、{INTERNAL_DIR}/"),
            ],
        };
        Ok(Box::new(UnpackAndExtractJob {
            preview,
            archive,
            workspace,
            original_output,
            overwrite,
        }))
    }
}

struct UnpackAndExtractJob {
    preview: Preview,
    archive: PathBuf,
    workspace: PathBuf,
    original_output: super::OutputSnapshot,
    overwrite: bool,
}

impl PreparedOperation for UnpackAndExtractJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        let parent = self.workspace.parent().ok_or("工作目录缺少父目录")?;
        let stage = super::StageDirectory::create(
            parent,
            self.workspace.file_name().ok_or("工作目录名为空")?,
        )?;
        let unpacked = stage.path.join(UNPACKED_DIR);
        let translations = stage.path.join(TRANSLATIONS_DIR);

        progress.report("阶段 1/2：解包 scenario.arc")?;
        let mut unpack_parameters = Parameters::default();
        unpack_parameters.set("source", Value::Path(self.archive.clone()));
        unpack_parameters.set("output", Value::Path(unpacked.clone()));
        let unpack_job = Unpack.prepare(&unpack_parameters)?;
        let unpack_report = unpack_job.execute(progress)?;

        progress.report("阶段 2/2：提取翻译文本")?;
        let mut export_parameters = Parameters::default();
        export_parameters.set("source", Value::Path(unpacked));
        export_parameters.set("output", Value::Path(translations.clone()));
        let export_job = ExportText.prepare(&export_parameters)?;
        let export_report = export_job.execute(progress)?;

        let metadata_dir = stage.path.join(INTERNAL_DIR);
        fs::create_dir(&metadata_dir)?;
        let manifest = translations.join(MANIFEST_NAME);
        let saved_manifest = metadata_dir.join("manifest.json");
        fs::rename(&manifest, &saved_manifest)?;

        let warning = super::commit_stage(
            &stage,
            &self.workspace,
            &self.original_output,
            self.overwrite,
        )?;
        let mut warnings = unpack_report.warnings;
        warnings.extend(export_report.warnings);
        warnings.extend(warning);
        Ok(RunReport {
            summary: "scenario.arc 解包和文本提取完成。".into(),
            totals: export_report.totals,
            outputs: vec![self.workspace.clone()],
            warnings,
        })
    }
}

pub struct InjectAndPack;

impl Operation for InjectAndPack {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "inject-pack",
            "注回文本并重新封包",
            vec![
                Field::new("template", "原始 scenario.arc", FieldKind::Path).required(),
                Field::new("workspace", "解包及翻译工作目录", FieldKind::Path).required(),
                Field::new(
                    "translations",
                    "翻译 JSON 目录（默认使用工作区内 translations）",
                    FieldKind::Path,
                ),
                Field::new("output", "重新封包后的 scenario.arc", FieldKind::Path).required(),
            ],
        );
        spec.description = "读取工作目录中的翻译 JSON，重建 MJO 后封包。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(template) = paths.first() {
            parameters.set("template", Value::Path(template.clone()));
            if let Some(parent) = template.parent() {
                if parameters.get("workspace").is_none() {
                    parameters.set(
                        "workspace",
                        Value::Path(default_workspace(parent).join("workspace")),
                    );
                }
                if parameters.get("output").is_none() {
                    parameters.set(
                        "output",
                        Value::Path(default_workspace(parent).join("scenario_repacked.arc")),
                    );
                }
            }
        }
        if let Some(workspace) = paths.get(1) {
            parameters.set("workspace", Value::Path(workspace.clone()));
        }
        if let Some(translations) = paths.get(2) {
            parameters.set("translations", Value::Path(translations.clone()));
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let template = fs::canonicalize(parameters.path("template")?)?;
        if !template.is_file() {
            return Err("scenario.arc 模板必须是文件".into());
        }
        let workspace = fs::canonicalize(parameters.path("workspace")?)?;
        if !workspace.is_dir() {
            return Err("工作目录必须来自“解包 scenario.arc 并提取文本”".into());
        }
        let unpacked = workspace.join(UNPACKED_DIR);
        let workspace_translations = workspace.join(TRANSLATIONS_DIR);
        if !unpacked.is_dir() || !workspace_translations.is_dir() {
            return Err(
                format!("工作目录必须同时包含 {UNPACKED_DIR}/ 和 {TRANSLATIONS_DIR}/").into(),
            );
        }
        let translations = match parameters.get("translations") {
            Some(Value::Path(path)) => {
                let path = fs::canonicalize(path)?;
                if !path.is_dir() {
                    return Err("外部翻译输入必须是目录".into());
                }
                path
            }
            Some(_) => return Err("--translations 必须是目录路径".into()),
            None => workspace_translations,
        };
        let output = super::resolve_output(parameters.path("output")?)?;
        ensure_separate(
            &workspace,
            &output,
            "封包输出不能位于输入工作目录内，也不能覆盖它",
        )?;
        ensure_separate(&template, &output, "封包输出不能覆盖原始 scenario.arc")?;
        let original_output = super::snapshot_file_output(&output)?;
        let overwrite = parameters.flag("overwrite");
        if original_output.is_some() && !overwrite {
            return Err("scenario.arc 输出已存在；请选择新路径或显式启用 --overwrite".into());
        }
        let preview = Preview {
            inputs: vec![template.clone(), workspace.clone(), translations.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "校验隐藏翻译清单、原始 MJO 源哈希和模板结构".into(),
                "按 CP932 字符与嵌入映射回注对白及独立选项".into(),
                "更新 MJO 长度、符号/入口偏移、分支及跳转表".into(),
                "用原 scenario.arc 文件表重建并暂存封包".into(),
            ],
            details: vec![format!("最终输出: {}", output.display())],
        };
        Ok(Box::new(InjectAndPackJob {
            preview,
            template,
            workspace,
            translations,
            output,
            original_output,
            overwrite,
        }))
    }
}

struct InjectAndPackJob {
    preview: Preview,
    template: PathBuf,
    workspace: PathBuf,
    translations: PathBuf,
    output: PathBuf,
    original_output: Option<Vec<u8>>,
    overwrite: bool,
}

impl PreparedOperation for InjectAndPackJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        let parent = self.output.parent().ok_or("封包输出缺少父目录")?;
        let stage = super::StageDirectory::create(
            parent,
            self.output.file_name().ok_or("封包输出文件名为空")?,
        )?;
        let rebuilt = stage.path.join("rebuilt_members");
        let source = self.workspace.join(UNPACKED_DIR);

        progress.report("阶段 1/2：注入 MJO 文本")?;
        let mut import_parameters = Parameters::default();
        import_parameters.set("source", Value::Path(source));
        import_parameters.set("translations", Value::Path(self.translations));
        import_parameters.set("output", Value::Path(rebuilt.clone()));
        let import_job = ImportText.prepare(&import_parameters)?;
        let import_report = import_job.execute(progress)?;

        progress.report("阶段 2/2：重建 scenario.arc")?;
        let staged_archive = stage.path.join("scenario.arc");
        let mut pack_parameters = Parameters::default();
        pack_parameters.set("template", Value::Path(self.template));
        pack_parameters.set("source", Value::Path(rebuilt));
        pack_parameters.set("output", Value::Path(staged_archive.clone()));
        let pack_job = PackArchive.prepare(&pack_parameters)?;
        let pack_report = pack_job.execute(progress)?;

        let warning = super::commit_file(
            &staged_archive,
            &self.output,
            &self.original_output,
            self.overwrite,
        )?;
        let mut warnings = import_report.warnings;
        warnings.extend(pack_report.warnings);
        warnings.extend(warning);
        Ok(RunReport {
            summary: "MJO 文本注入和 scenario.arc 封包完成。".into(),
            totals: pack_report.totals,
            outputs: vec![self.output.clone()],
            warnings,
        })
    }
}

fn default_workspace(project_root: &Path) -> PathBuf {
    project_root.join("work").join("katahane_windows")
}

fn ensure_separate(first: &Path, second: &Path, message: &str) -> vn_cli::Result<()> {
    if first == second || first.starts_with(second) || second.starts_with(first) {
        return Err(message.into());
    }
    Ok(())
}
