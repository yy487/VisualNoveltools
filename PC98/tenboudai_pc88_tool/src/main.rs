use std::path::PathBuf;

use tenboudai_pc88_tool::disk;
use tenboudai_pc88_tool::operations::{ExportText, ExtractMain, ImportText, RebuildMain};
use vn_cli::{
    Field, FieldKind, Operation, OperationSpec, Panel, Parameters, PreparedOperation, Preview,
    Progress, RunReport,
};
use vn_font::panel::FontOperation;

struct Inspect;

impl Operation for Inspect {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "inspect",
            "检查展望台 D88",
            vec![Field::new("source", "原始软碟 D88", FieldKind::Path).required()],
        );
        spec.description = "只读检查所有串联子盘和物理轨道、扇区编号；不假定标准文件系统。".into();
        spec.writes = false;
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(path) = paths.first() {
            parameters.set("source", vn_cli::Value::Path(path.clone()));
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let source = parameters.path("source")?.to_path_buf();
        let bytes = std::fs::read(&source)
            .map_err(|error| format!("无法读取 {}: {error}", source.display()))?;
        let report = disk::inspect(&bytes).map_err(vn_cli::Error::from)?;
        let pretty = serde_json::to_string_pretty(&report)
            .map_err(|error| vn_cli::Error::from(error.to_string()))?;
        Ok(Box::new(InspectJob {
            report: pretty,
            preview: Preview {
                inputs: vec![source],
                outputs: vec![],
                steps: vec!["解析串联 D88 并保留物理扇区信息".into()],
                details: vec![
                    "输出包含来源摘要、所有非空物理轨槽、实际扇区 ID 顺序和解析诊断。".into(),
                ],
            },
        }))
    }
}

struct InspectJob {
    report: String,
    preview: Preview,
}

impl PreparedOperation for InspectJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        progress.report("输入结构已在预检中解析；此操作为只读")?;
        progress.report(&self.report)?;
        Ok(RunReport {
            summary: "D88 物理结构检查完成。".into(),
            ..RunReport::default()
        })
    }
}

fn run() -> vn_cli::Result<()> {
    Panel::new(
        std::env::current_exe()?.into_os_string(),
        "展望台 PC-8801 工具",
        env!("CARGO_PKG_VERSION"),
        vec![
            Box::new(Inspect),
            Box::new(ExtractMain),
            Box::new(RebuildMain),
            Box::new(ExportText),
            Box::new(ImportText),
            Box::new(FontOperation::pc88()),
        ],
    )?
    .run_env()
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
