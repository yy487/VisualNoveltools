use mank_void_pc98::workflow;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vn_cli::{
    Error, Field, FieldKind, Operation, OperationSpec, Panel, Parameters, PreparedOperation,
    Preview, Progress, Result, RunReport,
};

struct Extract;

impl Operation for Extract {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "extract",
            "提取文本工作区",
            vec![
                Field::new("disk1", "第一张原始 FDI", FieldKind::Path).required(),
                Field::new("disk2", "第二张原始 FDI", FieldKind::Path).required(),
                Field::new("output", "新建文本工作区目录", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "读取两张原盘，导出原始文件、资源目录和可编辑的 UTF-8 文本 JSON。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        reject_overwrite(parameters)?;
        let disk1 = source_file(parameters.path("disk1")?, "第一张原盘")?;
        let disk2 = source_file(parameters.path("disk2")?, "第二张原盘")?;
        let output = new_output(parameters.path("output")?)?;
        let preview = Preview {
            inputs: vec![disk1.clone(), disk2.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "校验并解包两张 FDI".into(),
                "解析文本资源，生成原始文件与翻译 JSON".into(),
                "提交完整的新工作区".into(),
            ],
            details: Vec::new(),
        };
        Ok(Box::new(PreparedJob {
            mode: Mode::Extract {
                disk1,
                disk2,
                output,
            },
            preview,
        }))
    }
}

struct Inject;

impl Operation for Inject {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "inject",
            "注入译文并重建 FDI",
            vec![
                Field::new("disk1", "第一张原始 FDI", FieldKind::Path).required(),
                Field::new("disk2", "第二张原始 FDI", FieldKind::Path).required(),
                Field::new("workspace", "原始文本工作区", FieldKind::Path).required(),
                Field::new(
                    "translations",
                    "独立译文目录（可只含部分 JSON）",
                    FieldKind::Path,
                )
                .required(),
                Field::new("output", "新建成品目录", FieldKind::Path).required(),
            ],
        );
        spec.description = "校验来源及原文，回注可编辑文本，重建两张 FAT12 FDI。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        reject_overwrite(parameters)?;
        let disk1 = source_file(parameters.path("disk1")?, "第一张原盘")?;
        let disk2 = source_file(parameters.path("disk2")?, "第二张原盘")?;
        let workspace = source_dir(parameters.path("workspace")?, "文本工作区")?;
        let translations = source_dir(parameters.path("translations")?, "独立译文目录")?;
        let output = new_output(parameters.path("output")?)?;
        let preview = Preview {
            inputs: vec![
                disk1.clone(),
                disk2.clone(),
                workspace.clone(),
                translations.clone(),
            ],
            outputs: vec![output.clone()],
            steps: vec![
                "校验原盘和工作区来源".into(),
                "编码译文并重建资源与 FAT12 文件".into(),
                "提交两张新 FDI 和构建报告".into(),
            ],
            details: Vec::new(),
        };
        Ok(Box::new(PreparedJob {
            mode: Mode::Inject {
                disk1,
                disk2,
                workspace,
                translations,
                output,
            },
            preview,
        }))
    }
}

enum Mode {
    Extract {
        disk1: PathBuf,
        disk2: PathBuf,
        output: PathBuf,
    },
    Inject {
        disk1: PathBuf,
        disk2: PathBuf,
        workspace: PathBuf,
        translations: PathBuf,
        output: PathBuf,
    },
}

struct PreparedJob {
    mode: Mode,
    preview: Preview,
}

impl PreparedOperation for PreparedJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        let report = match &self.mode {
            Mode::Extract {
                disk1,
                disk2,
                output,
            } => {
                progress.report("正在提取文本工作区")?;
                workflow::extract_workflow(disk1, disk2, output).map_err(Error)?
            }
            Mode::Inject {
                disk1,
                disk2,
                workspace,
                translations,
                output,
            } => {
                progress.report("正在注入译文并重建 FDI")?;
                workflow::inject_workflow(disk1, disk2, workspace, translations, output)
                    .map_err(Error)?
            }
        };
        Ok(RunReport {
            summary: report.summary,
            totals: vec![
                ("文件".into(), report.files as u64),
                ("文本".into(), report.messages as u64),
            ],
            outputs: report.outputs,
            warnings: Vec::new(),
        })
    }
}

fn reject_overwrite(parameters: &Parameters) -> Result<()> {
    if parameters.flag("overwrite") {
        Err("请指定新的输出目录；本工具不覆盖已有目录或文件".into())
    } else {
        Ok(())
    }
}

fn source_file(path: &Path, label: &str) -> Result<PathBuf> {
    let resolved = fs::canonicalize(path).map_err(|e| Error(format!("{label} 无法读取: {e}")))?;
    if !resolved.is_file() {
        return Err(format!("{label} 不是文件: {}", resolved.display()).into());
    }
    Ok(resolved)
}

fn source_dir(path: &Path, label: &str) -> Result<PathBuf> {
    let resolved = fs::canonicalize(path).map_err(|e| Error(format!("{label} 无法读取: {e}")))?;
    if !resolved.is_dir() {
        return Err(format!("{label} 不是目录: {}", resolved.display()).into());
    }
    Ok(resolved)
}

fn new_output(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return Err(format!("输出已存在: {}", path.display()).into());
    }
    let name = path
        .file_name()
        .ok_or_else(|| Error(format!("无效的输出目录: {}", path.display())))?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent =
        fs::canonicalize(parent).map_err(|e| Error(format!("输出目录的父目录无法读取: {e}")))?;
    if !parent.is_dir() {
        return Err(format!("输出目录的父路径不是目录: {}", parent.display()).into());
    }
    Ok(parent.join(name))
}

fn main() -> ExitCode {
    let result = (|| {
        Panel::new(
            std::env::current_exe()?.into_os_string(),
            "38万公里的虚空 PC-98",
            env!("CARGO_PKG_VERSION"),
            vec![Box::new(Extract), Box::new(Inject)],
        )?
        .run_env()
    })();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[失败] {error}");
            ExitCode::FAILURE
        }
    }
}
