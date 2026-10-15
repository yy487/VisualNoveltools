use fivec_new::{Archive, PreparedExport};
use std::{path::PathBuf, process::ExitCode};
use vn_cli::*;

#[derive(Clone, Copy)]
enum Action {
    Inspect,
    List,
    Extract,
}

impl Operation for Action {
    fn spec(&self) -> OperationSpec {
        let (id, title) = match self {
            Self::Inspect => ("inspect", "检查镜像与卷"),
            Self::List => ("list", "列出文件"),
            Self::Extract => ("extract", "提取文件与来源清单"),
        };
        let mut fields = vec![Field::new("source", "原始磁盘镜像", FieldKind::Path).required()];
        if !matches!(self, Self::Inspect) {
            fields.push(
                Field::new("volume", "卷 ID 或 all", FieldKind::Text)
                    .default(Value::Text("all".into())),
            );
        }
        if matches!(self, Self::Extract) {
            fields.push(
                Field::new("output", "独立输出目录（父目录须存在）", FieldKind::Path).required(),
            );
        }
        let mut spec = OperationSpec::new(id, title, fields);
        spec.writes = matches!(self, Self::Extract);
        spec.description = "D88 / Anex86 FDI·HDI；FAT12 整盘卷及已验证 PC98 DOS 分区。".into();
        spec
    }
    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            parameters.set("source", Value::Path(path.clone()));
        }
        Ok(())
    }
    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let archive = Archive::open(parameters.path("source")?).map_err(Error)?;
        let mut preview = Preview {
            inputs: vec![archive.source_path().unwrap().to_path_buf()],
            ..Preview::default()
        };
        let mut lines = Vec::new();
        let mut count = 0;
        let export = match self {
            Self::Inspect => {
                lines.push(
                    serde_json::to_string_pretty(archive.inspection())
                        .map_err(|e| Error(e.to_string()))?,
                );
                preview
                    .steps
                    .push("检查容器、逻辑视图、分区及文件系统；显示 JSON 检查报告".into());
                None
            }
            Self::List => {
                for volume in archive
                    .selected_volumes(parameters.text("volume")?)
                    .map_err(Error)?
                {
                    let filesystem = volume.filesystem.as_ref().unwrap();
                    for file in &filesystem.files {
                        lines.push(format!(
                            "{}\t{}\t{}\t{}\t{}",
                            volume.id, file.id, file.size, file.sha256, file.display_path
                        ));
                        count += 1;
                    }
                }
                preview
                    .steps
                    .push("列出所选卷的完整文件树、条目 ID、大小和 SHA-256".into());
                None
            }
            Self::Extract => {
                let export = archive
                    .prepare_export(
                        parameters.text("volume")?,
                        parameters.path("output")?,
                        parameters.flag("overwrite"),
                    )
                    .map_err(Error)?;
                preview.outputs.push(export.output().to_path_buf());
                preview.details.push(format!(
                    "{} 个文件；{} 个卷",
                    export.manifest().files.len(),
                    export.manifest().selected_volumes.len()
                ));
                preview.steps = vec![
                    "核对源镜像与输出预检状态".into(),
                    "暂存文件和来源清单，校验哈希后提交整个目录".into(),
                ];
                Some(export)
            }
        };
        Ok(Box::new(Job {
            preview,
            lines,
            count,
            export,
        }))
    }
}

struct Job {
    preview: Preview,
    lines: Vec<String>,
    count: u64,
    export: Option<PreparedExport>,
}
impl PreparedOperation for Job {
    fn preview(&self) -> &Preview {
        &self.preview
    }
    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        if let Some(export) = self.export {
            progress.report("写入暂存目录并校验来源映射")?;
            let report = export.execute().map_err(Error)?;
            Ok(RunReport {
                summary: "提取完成".into(),
                totals: vec![
                    ("文件数".into(), report.files as u64),
                    ("字节数".into(), report.bytes as u64),
                ],
                outputs: vec![report.output],
                warnings: report.warnings,
            })
        } else {
            for line in self.lines {
                progress.report(&line)?;
            }
            Ok(RunReport {
                summary: "读取完成".into(),
                totals: vec![("文件数".into(), self.count)],
                ..RunReport::default()
            })
        }
    }
}

fn main() -> ExitCode {
    let result = (|| {
        Panel::new(
            std::env::current_exe()?.into_os_string(),
            "fivec_new 磁盘资源读取",
            env!("CARGO_PKG_VERSION"),
            vec![
                Box::new(Action::Inspect),
                Box::new(Action::List),
                Box::new(Action::Extract),
            ],
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
