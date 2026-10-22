use std::path::PathBuf;
use std::process::ExitCode;
use vn_cli::*;
use yoshinaga_sayuri_pc98::Job;

struct Action {
    import: bool,
}
impl Operation for Action {
    fn spec(&self) -> OperationSpec {
        let mut fields =
            vec![Field::new("source", "原始 HDI 或解包后的盘内目录", FieldKind::Path).required()];
        if self.import {
            fields.push(
                Field::new("translations", "导出的翻译工作区根目录", FieldKind::Path).required(),
            );
            fields.push(Field::new(
                "font-source",
                "原始 NP2 BMP 字库（可选，默认内置 font98 基线）",
                FieldKind::Path,
            ));
            fields.push(
                Field::new("font-face", "补画字形字体", FieldKind::Text)
                    .default(Value::Text(vn_font::font_98::FONT_FACE.into())),
            );
            fields.push(Field::new(
                "base-hdi",
                "原始 HDI 基准（目录来源时用于重封；HDI 来源自动使用原盘）",
                FieldKind::Path,
            ));
        }
        fields.push(
            Field::new(
                "output",
                "新的独立输出目录（父目录须存在）",
                FieldKind::Path,
            )
            .required(),
        );
        let mut spec = OperationSpec::new(
            if self.import { "import" } else { "export" },
            if self.import {
                "导入译文与变长重建"
            } else {
                "导出翻译文本"
            },
            fields,
        );
        spec.description = if self.import {
            "回注文本、调整 EXE 提示与 font98 字库；使用 HDI 来源时自动重封 game.hdi。"
        } else {
            "提取 TDH 文本及 EXE 硬编码文字，生成一次记录和 vn-text 翻译 JSON。"
        }
        .into();
        spec
    }
    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            parameters.set("source", Value::Path(path.clone()));
        }
        Ok(())
    }
    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        if parameters.flag("overwrite") {
            return Err("本工具只提交到新输出目录，请保留旧输出或另选路径。".into());
        }
        let job = if self.import {
            Job::import_with_options(
                parameters.path("source")?,
                parameters.path("translations")?,
                parameters.path("output")?,
                match parameters.get("font-source") {
                    Some(Value::Path(path)) => Some(path.as_path()),
                    _ => None,
                },
                parameters.text("font-face")?,
                match parameters.get("base-hdi") {
                    Some(Value::Path(path)) => Some(path.as_path()),
                    _ => None,
                },
            )
        } else {
            Job::export(parameters.path("source")?, parameters.path("output")?)
        }
        .map_err(Error)?;
        let preview = Preview {
            inputs: job.protected_inputs.clone(),
            outputs: vec![job.output.clone()],
            details: job.details.clone(),
            steps: vec!["校验来源与译文".into(), "写入暂存并校验后提交".into()],
        };
        Ok(Box::new(Prepared { job, preview }))
    }
}
struct Prepared {
    job: Job,
    preview: Preview,
}
impl PreparedOperation for Prepared {
    fn preview(&self) -> &Preview {
        &self.preview
    }
    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("写入暂存目录并核对输入快照")?;
        let output = self.job.output.clone();
        let count = self.job.execute().map_err(Error)?;
        Ok(RunReport {
            summary: "完成；原始来源与译后 JSON 保持不动。".into(),
            outputs: vec![output],
            totals: vec![("输出文件数".into(), count as u64)],
            ..RunReport::default()
        })
    }
}
fn main() -> ExitCode {
    let result = (|| -> Result<()> {
        Panel::new(
            std::env::current_exe()?.into_os_string(),
            "吉永サユリ PC-98 文本工具",
            env!("CARGO_PKG_VERSION"),
            vec![
                Box::new(Action { import: false }),
                Box::new(Action { import: true }),
            ],
        )?
        .run_env()
    })();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("错误: {error}");
            ExitCode::FAILURE
        }
    }
}
