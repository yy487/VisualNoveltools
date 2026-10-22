use lipstick_pc98::{
    Game,
    workflow::{self, Job},
};
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
            .execute(|s| progress.report(s).map_err(|e| e.to_string()))
            .map_err(Error)
    }
}

impl Operation for Export {
    fn spec(&self) -> OperationSpec {
        let mut s = OperationSpec::new(
            "export",
            "导出平铺 name/message JSON",
            vec![
                Field::new(
                    "game",
                    "游戏代数",
                    FieldKind::Choice(vec![
                        "lipstick1".into(),
                        "lipstick2".into(),
                        "lipstick3".into(),
                    ]),
                )
                .required(),
                Field::new("source", "这一部游戏的原始 FDI 目录", FieldKind::Path).required(),
                Field::new("output", "独立提取工作区", FieldKind::Path).required(),
            ],
        );
        s.description = "复用 FDI/FAT12 解析；三代分别解析脚本，平铺导出翻译 JSON；原始字节、控制和宏来源留在清单。无哈希校验。".into();
        s
    }
    fn prefill(&self, paths: &[PathBuf], p: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            p.set("source", Value::Path(path.clone()));
        }
        Ok(())
    }
    fn prepare(&self, p: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let game = Game::parse(p.text("game")?).map_err(Error)?;
        Ok(Box::new(Ready(
            workflow::prepare_export(
                game,
                p.path("source")?,
                p.path("output")?,
                p.flag("overwrite"),
            )
            .map_err(Error)?,
        )))
    }
}

impl Operation for Import {
    fn spec(&self) -> OperationSpec {
        let mut s = OperationSpec::new(
            "import",
            "回注脚本、内嵌语句、字库与 FDI",
            vec![
                Field::new("source", "本工具的原始提取工作区", FieldKind::Path).required(),
                Field::new(
                    "translation",
                    "独立译文目录或选定 JSON；留空用工作区 translations",
                    FieldKind::Paths,
                ),
                Field::new("output", "独立成品目录", FieldKind::Path).required(),
                Field::new(
                    "font",
                    "原始 NP2 字库；留空用公共模块参考字库",
                    FieldKind::Path,
                ),
                Field::new("face", "汉字字体", FieldKind::Text)
                    .default(Value::Text("新宋体".into())),
            ],
        );
        s.description = "保留自定义码位与 NAME.DEF 绑定；接入 vn_font::font_98；检查固定槽、缓存、软盘容量后输出完整新盘。不做哈希校验。".into();
        s
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
            workflow::prepare_import(
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
            "Lipstick ADV 1・2・3 PC-98",
            env!("CARGO_PKG_VERSION"),
            vec![Box::new(Export), Box::new(Import)],
        )?
        .run_env()
    })();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("[失败] {e}");
            ExitCode::FAILURE
        }
    }
}
