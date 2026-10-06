use galaxy_railway_pc98::workflow::{self, DiskInput, TranslationInputs, WorkspaceManifest};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vn_cli::{
    Error, Field, FieldKind, Operation, OperationSpec, Panel, Parameters, PreparedOperation,
    Preview, Progress, Result, RunReport, Value,
};

const TITLE: &str = "银河铁道之旅 PC-98 工具";

struct Extract;

impl Operation for Extract {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new("extract", "提取翻译工作区", disk_fields(true));
        spec.description = "校验五张 D88，按 CRS 指令流提取可达显示文本，并输出 MSG、CRS、启动提示的扁平译文 JSON 与资源目录。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        reject_overwrite(parameters)?;
        let disks = workflow::snapshot_disks(disk_paths(parameters)?)?;
        let output = new_output(parameters.path("output")?)?;
        let preview = Preview {
            inputs: disks.iter().map(|disk| disk.path.clone()).collect(),
            outputs: vec![output.clone()],
            steps: vec![
                "验证五张单盘 D88 和 Disk 1 FAT12 文件系统".into(),
                "提取原始文件、MSG 与 CRS 可达显示文本、启动提示和 PAL/PRS 目录".into(),
                "写入一个扁平 translations/ 目录和 workspace.json".into(),
            ],
            details: Vec::new(),
        };
        Ok(Box::new(PreparedJob {
            mode: Mode::Extract { disks, output },
            preview,
        }))
    }
}

struct Rebuild;

impl Operation for Rebuild {
    fn spec(&self) -> OperationSpec {
        let mut fields = disk_fields(false);
        fields.extend([
            Field::new("workspace", "原始提取工作区", FieldKind::Path).required(),
            Field::new("translations", "扁平翻译 JSON 目录", FieldKind::Path).required(),
            Field::new("output", "新建成品目录", FieldKind::Path).required(),
            Field::new("face", "重绘字库字体", FieldKind::Text)
                .default(Value::Text("新宋体".into())),
        ]);
        let mut spec = OperationSpec::new("rebuild", "注回译文并重建五张软碟与字库", fields);
        spec.description = "按 MSG 文本池指针重排，并对 CRS 显示字符串追加变长译文、重定向指针；重建 Disk 1 FAT12、更新数据盘启动行并输出 font.bmp。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        reject_overwrite(parameters)?;
        let disks = workflow::snapshot_disks(disk_paths(parameters)?)?;
        let workspace_path = source_dir(parameters.path("workspace")?, "原始提取工作区")?;
        let translation_path = source_dir(parameters.path("translations")?, "扁平翻译 JSON 目录")?;
        let workspace = workflow::read_workspace(&workspace_path).map_err(Error)?;
        let translations = workflow::read_translations(&translation_path).map_err(Error)?;
        let output = new_output(parameters.path("output")?)?;
        let face = match parameters.get("face") {
            Some(Value::Text(face)) => face.clone(),
            _ => "新宋体".into(),
        };
        if face.trim().is_empty() || face.contains('\0') {
            return Err("字库字体名称不能为空或包含 NUL".into());
        }
        let mut inputs = disks
            .iter()
            .map(|disk| disk.path.clone())
            .collect::<Vec<_>>();
        inputs.push(workspace_path);
        inputs.push(translation_path);
        let preview = Preview {
            inputs,
            outputs: vec![output.clone()],
            steps: vec![
                "校验五张原盘、workspace.json 与翻译来源".into(),
                "建立一份覆盖所有译文的 NP2 字槽计划".into(),
                "重排 MSG 文本池和指针，追加 CRS 译文并重定向显示指针，按 FAT12 簇链重建 Disk 1"
                    .into(),
                "回写数据盘启动提示，生成 font.bmp 和映射报告".into(),
                "暂存并提交完整成品目录".into(),
            ],
            details: vec![format!("字库字体：{face}")],
        };
        Ok(Box::new(PreparedJob {
            mode: Mode::Rebuild {
                disks,
                workspace,
                translations,
                output,
                face,
            },
            preview,
        }))
    }
}

fn disk_fields(include_output: bool) -> Vec<Field> {
    let mut fields = (1..=5)
        .map(|number| {
            Field::new(
                &format!("disk{number}"),
                &format!("第 {number} 张原始 D88"),
                FieldKind::Path,
            )
            .required()
        })
        .collect::<Vec<_>>();
    if include_output {
        fields.push(Field::new("output", "新建翻译工作区目录", FieldKind::Path).required());
    }
    fields
}

fn disk_paths(parameters: &Parameters) -> Result<[PathBuf; 5]> {
    let mut paths = Vec::with_capacity(5);
    for number in 1..=5 {
        let path = fs::canonicalize(parameters.path(&format!("disk{number}"))?)
            .map_err(|error| Error(format!("解析第 {number} 张 D88 失败: {error}")))?;
        if !path.is_file() {
            return Err(format!("第 {number} 张 D88 不是文件: {}", path.display()).into());
        }
        paths.push(path);
    }
    paths
        .try_into()
        .map_err(|_| Error("必须提供五张 D88".into()))
}

fn reject_overwrite(parameters: &Parameters) -> Result<()> {
    if parameters.flag("overwrite") {
        Err("请指定新的输出目录；此工具拒绝覆盖已有目录或文件".into())
    } else {
        Ok(())
    }
}

fn source_dir(path: &Path, label: &str) -> Result<PathBuf> {
    let resolved =
        fs::canonicalize(path).map_err(|error| Error(format!("{label}无法读取: {error}")))?;
    if !resolved.is_dir() {
        return Err(format!("{label}不是目录: {}", resolved.display()).into());
    }
    Ok(resolved)
}

fn new_output(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return Err(format!("输出已存在: {}", path.display()).into());
    }
    let name = path
        .file_name()
        .ok_or_else(|| Error(format!("无效的输出路径: {}", path.display())))?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent =
        fs::canonicalize(parent).map_err(|error| Error(format!("输出父目录无法读取: {error}")))?;
    if !parent.is_dir() {
        return Err(format!("输出父路径不是目录: {}", parent.display()).into());
    }
    Ok(parent.join(name))
}

enum Mode {
    Extract {
        disks: [DiskInput; 5],
        output: PathBuf,
    },
    Rebuild {
        disks: [DiskInput; 5],
        workspace: WorkspaceManifest,
        translations: TranslationInputs,
        output: PathBuf,
        face: String,
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
        let report = match self.mode {
            Mode::Extract { disks, output } => {
                progress.report("正在提取五张软碟的翻译工作区")?;
                workflow::extract_workflow(&disks, &output).map_err(Error)?
            }
            Mode::Rebuild {
                disks,
                workspace,
                translations,
                output,
                face,
            } => {
                progress.report("正在回注翻译并重建软碟与字库")?;
                workflow::rebuild_workflow(&disks, &workspace, &translations, &output, &face)
                    .map_err(Error)?
            }
        };
        Ok(RunReport {
            summary: report.summary,
            totals: vec![
                ("软碟文件".into(), 5),
                ("源文件".into(), report.files as u64),
                ("文本条目".into(), report.messages as u64),
                ("改写文件".into(), report.changed_files as u64),
                ("重绘字形".into(), report.patched_glyphs as u64),
            ],
            outputs: report.outputs,
            warnings: Vec::new(),
        })
    }
}

fn main() -> ExitCode {
    let result = (|| {
        Panel::new(
            std::env::current_exe()?.into_os_string(),
            TITLE,
            env!("CARGO_PKG_VERSION"),
            vec![Box::new(Extract), Box::new(Rebuild)],
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
