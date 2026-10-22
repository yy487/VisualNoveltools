use std::fs;
use std::path::PathBuf;

use encoding_rs::SHIFT_JIS;
use vn_cli::{
    Field, FieldKind, Operation, OperationSpec, Parameters, PreparedOperation, Preview, Progress,
    Result, RunReport, Value,
};

use crate::transaction;

pub fn pack_bytes(plain: &[u8]) -> Vec<u8> {
    crate::unpack_bytes(plain)
}

pub struct Pack;

impl Operation for Pack {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "pack",
            "重封包 nscript.dat",
            vec![
                Field::new("source", "明文脚本文件", FieldKind::Path).required(),
                Field::new("output", "重封包输出文件", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "将 Shift_JIS / CP932 明文脚本逐字节 XOR 0x84，写入独立 nscript.dat。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(source) = paths.first() {
            parameters.set("source", Value::Path(source.clone()));
            if parameters.get("output").is_none() {
                parameters.set(
                    "output",
                    Value::Path(default_work_output("nscript-repacked.dat")?),
                );
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let source = fs::canonicalize(parameters.path("source")?)?;
        if !fs::metadata(&source)?.is_file() {
            return Err("明文脚本源路径必须是文件".into());
        }
        let plain = fs::read(&source)
            .map_err(|error| vn_cli::Error(format!("读取明文脚本失败：{error}")))?;
        validate_plain_script(&plain)?;

        let output = transaction::resolve_file_output(parameters.path("output")?)?;
        if source == output {
            return Err("输出不能覆盖明文脚本源文件".into());
        }
        let overwrite = parameters.flag("overwrite");
        let (output, original_output) = transaction::prepare_file_output(&output, overwrite)?;
        let packed = pack_bytes(&plain);
        let preview = Preview {
            inputs: vec![source.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "读取并校验 CP932 明文脚本".into(),
                "逐字节 XOR 0x84 重封包".into(),
                "写入同目录临时文件并安全提交".into(),
            ],
            details: vec![format!(
                "源脚本 {} 字节；输出 {} 字节；不添加或删除任何换行字节",
                plain.len(),
                packed.len()
            )],
        };
        Ok(Box::new(PreparedPack {
            source,
            plain,
            output,
            packed,
            original_output,
            overwrite,
            preview,
        }))
    }
}

struct PreparedPack {
    source: PathBuf,
    plain: Vec<u8>,
    output: PathBuf,
    packed: Vec<u8>,
    original_output: Option<Vec<u8>>,
    overwrite: bool,
    preview: Preview,
}

impl PreparedOperation for PreparedPack {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("复核明文脚本源文件")?;
        let current = fs::read(&self.source)
            .map_err(|error| vn_cli::Error(format!("重新读取明文脚本失败：{error}")))?;
        if current != self.plain {
            return Err("明文脚本在预检后发生变化；未写出封包".into());
        }
        progress.report("暂存并提交 nscript.dat")?;
        transaction::write_file_atomic(
            &self.output,
            &self.packed,
            &self.original_output,
            self.overwrite,
        )?;
        Ok(RunReport {
            summary: "nscript.dat 重封包完成。".into(),
            totals: vec![("字节数".into(), self.packed.len() as u64)],
            outputs: vec![self.output.clone()],
            ..RunReport::default()
        })
    }
}

fn validate_plain_script(plain: &[u8]) -> Result<()> {
    if !plain.starts_with(b"*define") {
        return Err("明文脚本未以 *define 开始；拒绝封包".into());
    }
    if plain.contains(&0) {
        return Err("明文脚本包含 NUL 字节".into());
    }
    let (_, had_errors) = SHIFT_JIS.decode_without_bom_handling(plain);
    if had_errors {
        return Err("明文脚本包含无法按 Shift_JIS / CP932 解码的字节".into());
    }
    Ok(())
}

fn default_work_output(name: &str) -> Result<PathBuf> {
    let executable = std::env::current_exe()?;
    let parent = executable.parent().ok_or("无法确定工具程序所在目录")?;
    Ok(parent.join(name))
}
