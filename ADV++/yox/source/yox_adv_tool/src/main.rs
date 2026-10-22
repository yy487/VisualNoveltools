use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vn_cli::*;
use yox_adv_tool::{
    workflow::{Export, Import},
    Archive,
};

struct Unpack;

impl Operation for Unpack {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "unpack",
            "解包 YOX 档案",
            vec![
                Field::new("source", "原始 DAT", FieldKind::Path).required(),
                Field::new("output", "新的输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "按原编号导出原始条目，并保留文件表、填充与尾部；只接受新输出目录。".into();
        spec.primary = false;
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
            return Err("本工具只解包到新目录，不支持 --overwrite".into());
        }
        let source = fs::canonicalize(parameters.path("source")?)?;
        let requested = std::path::absolute(parameters.path("output")?)?;
        let output = fs::canonicalize(requested.parent().ok_or("输出缺少父目录")?)?
            .join(requested.file_name().ok_or("输出缺少目录名")?);
        if output.try_exists()? {
            return Err("输出目录已存在，请选择新目录".into());
        }
        let name = source
            .file_name()
            .ok_or("输入缺少文件名")?
            .to_string_lossy()
            .into_owned();
        let archive = Archive::parse(name, fs::read(&source)?)?;
        let preview = Preview {
            inputs: vec![source],
            outputs: vec![output],
            steps: vec!["读取并验证文件表".into(), "导出原始条目和非条目数据".into()],
            details: vec![format!(
                "{} 个条目；不计算哈希、不执行回包。",
                archive.entries.len()
            )],
        };
        Ok(Box::new(UnpackJob { preview, archive }))
    }
}

struct UnpackJob {
    preview: Preview,
    archive: Archive,
}

impl PreparedOperation for UnpackJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在导出 YOX 原始条目")?;
        let output = self.archive.unpack(
            Path::new(&self.preview.inputs[0]),
            Path::new(&self.preview.outputs[0]),
        )?;
        Ok(RunReport {
            summary: "解包完成。".into(),
            totals: vec![("条目数".into(), self.archive.entries.len() as u64)],
            outputs: vec![output],
            ..RunReport::default()
        })
    }
}

fn main() -> ExitCode {
    let result = (|| {
        Panel::new(
            std::env::current_exe()?.into_os_string(),
            "ADV++ YOX 解包工具",
            env!("CARGO_PKG_VERSION"),
            vec![Box::new(Export), Box::new(Import), Box::new(Unpack)],
        )?
        .run_env()
    })();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
