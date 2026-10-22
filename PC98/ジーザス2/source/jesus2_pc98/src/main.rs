use jesus2_pc98::{workflow, Prepared};
use std::{path::PathBuf, process::ExitCode};
use vn_cli::{
    Error, Field, FieldKind, Operation, OperationSpec, Panel, Parameters, PreparedOperation,
    Preview, Progress, Result, RunReport, Value,
};

fn prefill_disk_directory(paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
    if let Some(path) = paths.first() {
        let directory = if path.is_dir() {
            path.clone()
        } else {
            path.parent().ok_or("FDI 路径没有父目录")?.to_path_buf()
        };
        parameters.set("input-dir", Value::Path(directory));
    }
    Ok(())
}

struct Export;
impl Operation for Export {
    fn spec(&self) -> OperationSpec {
        let mut fields =
            vec![Field::new("input-dir", "含五张 FDI 的目录", FieldKind::Path).required()];
        fields.push(Field::new("output", "新建文本工作区目录", FieldKind::Path).required());
        let mut spec = OperationSpec::new("export", "导出完整资源与翻译 JSON", fields);
        spec.description =
            "从输入目录自动识别并排序五张 FDI，保存完整盘内文件、结构记录和翻译 JSON。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        prefill_disk_directory(paths, parameters)
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        reject_overwrite(parameters)?;
        let job =
            workflow::prepare_export(parameters.path("input-dir")?, parameters.path("output")?)
                .map_err(Error)?;
        let preview = Preview {
            inputs: job.inputs(),
            outputs: vec![job.output().to_path_buf()],
            details: vec![format!(
                "{} 张软碟；{} 个盘内文件；{} 条可翻译文本；{} 份 JSON",
                job.disks, job.files, job.text_units, job.translation_files
            )],
            steps: vec![
                "验证五张 FDI 与游戏文件集".into(),
                "解析 PRG/MES 与可编辑菜单文本".into(),
                "写入完整源文件、一次提取记录与翻译 JSON".into(),
            ],
        };
        Ok(Box::new(ExportJob { job, preview }))
    }
}

struct Import;
impl Operation for Import {
    fn spec(&self) -> OperationSpec {
        let mut fields =
            vec![Field::new("input-dir", "含五张 FDI 的目录", FieldKind::Path).required()];
        fields.extend([
            Field::new("workspace", "原始导出工作区", FieldKind::Path).required(),
            Field::new(
                "translation",
                "译后 JSON 文件或目录（可多选）",
                FieldKind::Paths,
            )
            .required(),
            Field::new("font", "NP2 font.tmp（空白使用公共模板）", FieldKind::Path),
            Field::new("face", "绘字字体", FieldKind::Text)
                .default(Value::Text(vn_font::font_98::FONT_FACE.into())),
            Field::new("output", "新建成品目录", FieldKind::Path).required(),
        ]);
        let mut spec = OperationSpec::new("import", "回注译文并重建五张 FDI", fields);
        spec.description =
            "按盘内标记自动识别五张原盘，绑定译文并生成配套 font.tmp 与新 FDI。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        reject_overwrite(parameters)?;
        let disk_directory = parameters.path("input-dir")?;
        let translations = parameters.paths("translation")?.to_vec();
        let font = match parameters.get("font") {
            Some(Value::Path(path)) => Some(path.as_path()),
            _ => None,
        };
        let face = match parameters.get("face") {
            Some(Value::Text(face)) if !face.is_empty() => face.as_str(),
            _ => vn_font::font_98::FONT_FACE,
        };
        let job = workflow::prepare_import(
            disk_directory,
            parameters.path("workspace")?,
            &translations,
            font,
            face,
            parameters.path("output")?,
        )
        .map_err(Error)?;
        let preview = Preview {
            inputs: job.inputs(),
            outputs: vec![job.output().to_path_buf()],
            details: vec![format!(
                "{} 张 FDI；{} 个选定 JSON；{} 条译文变更；{} 个字形载体",
                job.disks, job.translation_files, job.changed_entries, job.font_glyphs
            )],
            steps: vec![
                "重新解析五张 FDI 并核对工作区结构与文本定位".into(),
                "验证 JSON 形状、控制标记与编码容量".into(),
                "重建 MES、FAT12 与 FDI，并读回校验".into(),
                "生成配套 font.tmp 和映射清单".into(),
            ],
        };
        Ok(Box::new(ImportJob { job, preview }))
    }
}

struct ExportJob {
    job: Prepared,
    preview: Preview,
}

struct ImportJob {
    job: Prepared,
    preview: Preview,
}

impl PreparedOperation for ExportJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("提交完整文本工作区")?;
        let disks = self.job.disks as u64;
        let files = self.job.files as u64;
        let text_units = self.job.text_units as u64;
        let output = self.job.execute().map_err(Error)?;
        Ok(RunReport {
            summary: "完整工作区已导出".into(),
            totals: vec![
                ("FDI 磁盘".into(), disks),
                ("盘内文件".into(), files),
                ("文本条目".into(), text_units),
            ],
            outputs: vec![output],
            ..RunReport::default()
        })
    }
}

impl PreparedOperation for ImportJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("提交五张 FDI 与配套字库")?;
        let disks = self.job.disks as u64;
        let translation_files = self.job.translation_files as u64;
        let changed_entries = self.job.changed_entries as u64;
        let font_glyphs = self.job.font_glyphs as u64;
        let output = self.job.execute().map_err(Error)?;
        Ok(RunReport {
            summary: "译文已回注，五张 FDI 与 font.tmp 已生成".into(),
            totals: vec![
                ("FDI 磁盘".into(), disks),
                ("译文文件".into(), translation_files),
                ("译文变更".into(), changed_entries),
                ("重绘字形".into(), font_glyphs),
            ],
            outputs: vec![output],
            ..RunReport::default()
        })
    }
}

fn reject_overwrite(parameters: &Parameters) -> Result<()> {
    if parameters.flag("overwrite") {
        Err("请选择新的输出目录；本工具不覆盖原盘或已有工作区".into())
    } else {
        Ok(())
    }
}

fn run() -> Result<()> {
    Panel::new(
        std::env::current_exe()?.into_os_string(),
        "《ジーザス2》PC-98 文本工具",
        env!("CARGO_PKG_VERSION"),
        vec![Box::new(Export), Box::new(Import)],
    )?
    .run_env()
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[失败] {error}");
            ExitCode::FAILURE
        }
    }
}
