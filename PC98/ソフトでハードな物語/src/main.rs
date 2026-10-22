use soft_hard_pc98::workflow::{self, Prepared};
use std::process::ExitCode;
use vn_cli::*;

struct Export;

impl Operation for Export {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "export",
            "导出可编辑脚本译文",
            vec![
                Field::new("disk-a", "SOFT_A 原始 FDI", FieldKind::Path).required(),
                Field::new("disk-b", "SOFT_B 原始 FDI", FieldKind::Path).required(),
                Field::new("output", "新建译文工作区目录", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "校验两张原盘，解码 TXT/CHR/ITEM/HELP/SEND 脚本并生成带来源校验的 UTF-8 翻译模板。"
                .into();
        spec.primary = true;
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        if parameters.flag("overwrite") {
            return Err("请指定一个新的输出目录；不会覆盖既有文件".into());
        }
        let job = workflow::prepare_export(
            parameters.path("disk-a")?,
            parameters.path("disk-b")?,
            parameters.path("output")?,
        )
        .map_err(Error)?;
        Ok(Box::new(PreparedJob::new(job, "导出脚本与翻译模板")))
    }
}

struct Build;

impl Operation for Build {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "build",
            "回注译文、生成字库并重建两张 FDI",
            vec![
                Field::new("disk-a", "SOFT_A 原始 FDI", FieldKind::Path).required(),
                Field::new("disk-b", "SOFT_B 原始 FDI", FieldKind::Path).required(),
                Field::new("translation", "已编辑的译文工作区", FieldKind::Path).required(),
                Field::new("face", "PC-98 字形绘制字体", FieldKind::Text)
                    .required()
                    .default(Value::Text(vn_font::font_98::FONT_FACE.into())),
                Field::new("output", "新建成品目录", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "按脚本结构回注译文、更新变长文本的相对分支、调用共享 PC-98 字库并重建 FAT12 FDI。"
                .into();
        spec.primary = true;
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        if parameters.flag("overwrite") {
            return Err("请指定一个新的成品目录；不会覆盖原盘或既有文件".into());
        }
        let face = parameters.text("face")?;
        let job = workflow::prepare_build(
            parameters.path("disk-a")?,
            parameters.path("disk-b")?,
            parameters.path("translation")?,
            parameters.path("output")?,
            face,
        )
        .map_err(Error)?;
        Ok(Box::new(PreparedJob::new(job, "回注译文并重建镜像")))
    }
}

struct PreparedJob {
    job: Prepared,
    preview: Preview,
    label: &'static str,
}

impl PreparedJob {
    fn new(job: Prepared, label: &'static str) -> Self {
        let preview = Preview {
            inputs: job.inputs(),
            outputs: vec![job.output().to_path_buf()],
            details: Vec::new(),
            steps: vec![
                "校验原盘与翻译模板 SHA-256".into(),
                "在暂存目录生成所有产物并核对输入快照".into(),
                "一次性提交新输出目录".into(),
            ],
        };
        Self {
            job,
            preview,
            label,
        }
    }
}

impl PreparedOperation for PreparedJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report(self.label)?;
        self.job.execute(progress)
    }
}

fn main() -> ExitCode {
    let result = (|| {
        Panel::new(
            std::env::current_exe()?.into_os_string(),
            "ソフトでハードな物語 PC-98",
            env!("CARGO_PKG_VERSION"),
            vec![Box::new(Export), Box::new(Build)],
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
