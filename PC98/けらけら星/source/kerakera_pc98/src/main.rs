use kerakera_pc98::{Job, prepare_export, prepare_import};
use std::path::PathBuf;
use std::process::ExitCode;
use vn_cli::*;

struct Export;
struct Import;
struct Ready(Job);

impl PreparedOperation for Ready {
    fn preview(&self) -> &Preview {
        &self.0.preview
    }
    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        self.0
            .execute(|message| progress.report(message).map_err(|e| e.to_string()))
            .map_err(Error)
    }
}

impl Operation for Export {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "export",
            "提取 name/message 翻译 JSON",
            vec![
                Field::new("source", "原始 FDI 目录或单个镜像", FieldKind::Path).required(),
                Field::new("output", "独立提取工作区", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "按本作解释器解析 MES，保留原盘和一次提取记录，导出仅含 name/message 的翻译视图。"
                .into();
        spec
    }
    fn prefill(&self, paths: &[PathBuf], p: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            p.set("source", Value::Path(path.clone()));
        }
        Ok(())
    }
    fn prepare(&self, p: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        Ok(Box::new(Ready(
            prepare_export(p.path("source")?, p.path("output")?, p.flag("overwrite"))
                .map_err(Error)?,
        )))
    }
}

impl Operation for Import {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "import",
            "注回脚本、生成字库和 FDI",
            vec![
                Field::new("source", "本工具的原始提取工作区", FieldKind::Path).required(),
                Field::new(
                    "translation",
                    "独立译文目录或选定 JSON；留空使用工作区 translations",
                    FieldKind::Paths,
                ),
                Field::new("output", "独立成品目录", FieldKind::Path).required(),
                Field::new(
                    "font",
                    "可选原始 NP2 字库；留空用公共模块参考字库",
                    FieldKind::Path,
                ),
                Field::new("face", "中文字体", FieldKind::Text)
                    .default(Value::Text("新宋体".into())),
            ],
        );
        spec.description = "姓名自动生成［name］和 A5；正文 LF 转 A5；用同一 font_98 映射生成脚本和 font.bmp，重分配 FAT12 文件簇。".into();
        spec
    }
    fn prefill(&self, paths: &[PathBuf], p: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            p.set("source", Value::Path(path.clone()));
        }
        Ok(())
    }
    fn prepare(&self, p: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let translations = match p.get("translation") {
            Some(Value::Paths(paths)) => paths.as_slice(),
            _ => &[],
        };
        let font = match p.get("font") {
            Some(Value::Path(path)) => Some(path.as_path()),
            _ => None,
        };
        Ok(Box::new(Ready(
            prepare_import(
                p.path("source")?,
                translations,
                p.path("output")?,
                p.flag("overwrite"),
                font,
                p.text("face")?,
            )
            .map_err(Error)?,
        )))
    }
}

fn main() -> ExitCode {
    let result = (|| {
        Panel::new(
            std::env::current_exe()?.into_os_string(),
            "けらけら星 PC-98 提取注回工具",
            env!("CARGO_PKG_VERSION"),
            vec![Box::new(Export), Box::new(Import)],
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
