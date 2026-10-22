use kohaku_pc98::{Mode, Prepared};
use std::{path::PathBuf, process::ExitCode};
use vn_cli::*;

struct Action {
    mode: Mode,
    main_export: bool,
}
impl Operation for Action {
    fn spec(&self) -> OperationSpec {
        let (id, label) = match self.mode {
            Mode::Unpack => ("unpack", "解包全部 DAT 资源"),
            Mode::Extract if self.main_export => ("export", "导出翻译 JSON 与参考字库"),
            Mode::Extract => ("extract", "解包并导出翻译 JSON（兼容命令）"),
        };
        let mut spec = OperationSpec::new(
            id,
            label,
            vec![
                Field::new("disk1", "第一张原始镜像（含 KOHAKU.COM）", FieldKind::Path).required(),
                Field::new("disk2", "第二张原始镜像（含 DISK2.DAT）", FieldKind::Path).required(),
                Field::new("output", "新建输出目录（父目录须已存在）", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "读取已验证版本的 PC-98 原盘；保留资源索引、来源及原始字节。只接受新输出目录。".into();
        spec.primary = self.main_export;
        spec
    }
    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        for (field, path) in ["disk1", "disk2"].iter().zip(paths) {
            parameters.set(*field, Value::Path(path.clone()));
        }
        Ok(())
    }
    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        if parameters.flag("overwrite") {
            return Err("本版本不覆盖已有目录，请选择新的输出目录".into());
        }
        let job = kohaku_pc98::prepare(
            parameters.path("disk1")?,
            parameters.path("disk2")?,
            parameters.path("output")?,
            self.mode,
        )
        .map_err(Error)?;
        let preview = Preview {
            inputs: job.inputs(),
            outputs: vec![job.output().to_path_buf()],
            details: vec![format!(
                "{} 个独立资源；{} 份 JSON；{} 条可翻译文本",
                job.resources, job.translation_files, job.translation_entries
            )],
            steps: vec![
                "验证原盘与索引快照".into(),
                "写入暂存目录并核对所有字节后提交".into(),
            ],
        };
        Ok(Box::new(Job { job, preview }))
    }
}
struct Import;
impl Operation for Import {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "import",
            "回注译文、生成字库并重建 FDI",
            vec![
                Field::new("disk1", "第一张原始镜像", FieldKind::Path).required(),
                Field::new("disk2", "第二张原始镜像", FieldKind::Path).required(),
                Field::new(
                    "translation",
                    "译后 JSON 文件或目录（可多选）",
                    FieldKind::Paths,
                )
                .required(),
                Field::new(
                    "font",
                    "原始 NP2 font.tmp（空白使用公共模板）",
                    FieldKind::Path,
                ),
                Field::new("face", "绘字字体", FieldKind::Text)
                    .required()
                    .default(Value::Text(vn_font::font_98::FONT_FACE.into())),
                Field::new("output", "新建成品目录", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "从完整原盘合入所选译文；生成配套 font.tmp、完整 DAT/COM 和两张 FDI。图片原样保留。"
                .into();
        spec
    }
    fn prepare(&self, p: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        if p.flag("overwrite") {
            return Err("请使用新建输出目录；不覆盖原盘或已有成品".into());
        }
        let translations = match p.get("translation") {
            Some(Value::Paths(paths)) => paths.clone(),
            _ => return Err("需要选择译文 JSON".into()),
        };
        let font = match p.get("font") {
            Some(Value::Path(path)) => Some(path.as_path()),
            _ => None,
        };
        let face = match p.get("face") {
            Some(Value::Text(face)) => face.as_str(),
            _ => vn_font::font_98::FONT_FACE,
        };
        let job = kohaku_pc98::workflow::prepare_import(kohaku_pc98::workflow::ImportOptions {
            disk1: p.path("disk1")?,
            disk2: p.path("disk2")?,
            translations: &translations,
            font,
            face,
            output: p.path("output")?,
        })
        .map_err(Error)?;
        let preview = Preview {
            inputs: job.inputs(),
            outputs: vec![job.output().to_path_buf()],
            details: vec![format!(
                "{} 份选定 JSON；{} 条修改；公共 PC-98 字库与两张 FDI",
                job.translation_files, job.translation_entries
            )],
            steps: vec![
                "校验译文并规划字槽".into(),
                "重建文本、DAT、COM 与 FAT12，读回验证后提交".into(),
            ],
        };
        Ok(Box::new(Job { job, preview }))
    }
}
struct Job {
    job: Prepared,
    preview: Preview,
}
impl PreparedOperation for Job {
    fn preview(&self) -> &Preview {
        &self.preview
    }
    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("校验来源并提交本次构建结果")?;
        let resources = self.job.resources;
        let translations = self.job.translation_entries;
        let output = self.job.execute().map_err(Error)?;
        Ok(RunReport {
            summary: "操作完成".into(),
            totals: vec![
                ("资源数".into(), resources as u64),
                ("翻译条目".into(), translations as u64),
            ],
            outputs: vec![output],
            ..RunReport::default()
        })
    }
}
fn main() -> ExitCode {
    let result = (|| {
        Panel::new(
            std::env::current_exe()?.into_os_string(),
            "琥珀色の遺言 PC-98",
            env!("CARGO_PKG_VERSION"),
            vec![
                Box::new(Action {
                    mode: Mode::Extract,
                    main_export: true,
                }),
                Box::new(Import),
                Box::new(Action {
                    mode: Mode::Unpack,
                    main_export: false,
                }),
                Box::new(Action {
                    mode: Mode::Extract,
                    main_export: false,
                }),
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
