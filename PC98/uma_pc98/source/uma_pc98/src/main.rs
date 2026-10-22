use std::{path::PathBuf, process::ExitCode};
use uma_pc98::{workflow, Job};
use vn_cli::*;

#[derive(Clone, Copy)]
enum Action {
    Export,
    Import,
    MemoryFix,
}
impl Operation for Action {
    fn spec(&self) -> OperationSpec {
        let mut fields =
            vec![Field::new("source", "四张原始 D88 所在目录", FieldKind::Path).required()];
        let (id, title) = match self {
            Self::Export => ("export", "导出 MES、EXE 与马名词典翻译文本"),
            Self::Import => {
                fields.extend([
                    Field::new("workspace", "导出工作区（含原生来源记录）", FieldKind::Path)
                        .required(),
                    Field::new(
                        "translations",
                        "独立译文目录（可仅含部分 JSON）",
                        FieldKind::Path,
                    ),
                    Field::new(
                        "font",
                        "原始 NP2 font.tmp（留空用公共模块内置字库）",
                        FieldKind::Path,
                    ),
                    Field::new(
                        "hdi-base",
                        "可选：已安装游戏且可启动的 HDI 基础镜像",
                        FieldKind::Path,
                    ),
                    Field::new("face", "绘字字体", FieldKind::Text)
                        .default(Value::Text("新宋体".into())),
                ]);
                ("import", "注入文本、生成字库并输出磁盘")
            }
            Self::MemoryFix => {
                fields.push(
                    Field::new("hdi-base", "保留游戏存档的现有 HDI", FieldKind::Path).required(),
                );
                ("memory-fix", "修复现有 HDI 的 DOS 主内存启动配置")
            }
        };
        fields
            .push(Field::new("output", "独立输出目录（父目录须存在）", FieldKind::Path).required());
        let mut spec = OperationSpec::new(id, title, fields);
        spec.allow_empty_output_dir = true;
        spec.description = "馬なり１ハロン劇場；D88 为安装盘，import 生成 installed 游戏目录；可基于现有 HDI 输出新 game.hdi；不校验哈希。".into();
        spec
    }
    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            let path = if path.is_dir() {
                path.clone()
            } else {
                path.parent().ok_or("输入缺少目录")?.to_path_buf()
            };
            parameters.set("source", Value::Path(path));
        }
        Ok(())
    }
    fn prepare(&self, p: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let job = match self {
            Self::Export => {
                workflow::prepare_export(p.path("source")?, p.path("output")?, p.flag("overwrite"))
            }
            Self::Import => workflow::prepare_import(workflow::ImportOptions {
                source: p.path("source")?,
                workspace: p.path("workspace")?,
                translations: optional_path(p, "translations"),
                font: optional_path(p, "font"),
                hdi_base: optional_path(p, "hdi-base"),
                face: p.text("face")?,
                output: p.path("output")?,
                overwrite: p.flag("overwrite"),
            }),
            Self::MemoryFix => workflow::prepare_memory_fix(
                p.path("source")?,
                p.path("hdi-base")?,
                p.path("output")?,
                p.flag("overwrite"),
            ),
        }
        .map_err(Error)?;
        Ok(Box::new(Prepared(job)))
    }
}
fn optional_path<'a>(p: &'a Parameters, field: &str) -> Option<&'a std::path::Path> {
    match p.get(field) {
        Some(Value::Path(path)) => Some(path),
        _ => None,
    }
}
struct Prepared(Job);
impl PreparedOperation for Prepared {
    fn preview(&self) -> &Preview {
        &self.0.preview
    }
    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        self.0.execute(progress).map_err(Error)
    }
}
fn main() -> ExitCode {
    let result = (|| {
        Panel::new(
            std::env::current_exe()?.into_os_string(),
            "馬なり１ハロン劇場 PC98 工具",
            env!("CARGO_PKG_VERSION"),
            vec![
                Box::new(Action::Export),
                Box::new(Action::Import),
                Box::new(Action::MemoryFix),
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
