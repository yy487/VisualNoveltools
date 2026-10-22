use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use vn_cli::{
    Error, Field, FieldKind, Operation, OperationSpec, Parameters, PreparedOperation, Preview,
    Progress, Result, RunReport, Value,
};

const SCRIPT_XOR_KEY: u8 = 0x84;

/// NScripter 2.82 applies this bytewise transform to every on-disk nscript.dat byte.
/// It is its own inverse, so the same operation also reconstructs the packed bytes.
pub fn unpack_bytes(packed: &[u8]) -> Vec<u8> {
    packed.iter().map(|byte| byte ^ SCRIPT_XOR_KEY).collect()
}

pub struct Unpack;

impl Operation for Unpack {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "unpack",
            "解封包 nscript.dat",
            vec![
                Field::new("source", "原始 nscript.dat", FieldKind::Path).required(),
                Field::new("output", "明文脚本输出文件", FieldKind::Path).required(),
            ],
        );
        spec.description = "按游戏运行时的 XOR 0x84 规则解码 nscript.dat，写入独立文件。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            parameters.set("source", Value::Path(path.clone()));
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let source = fs::canonicalize(parameters.path("source")?)?;
        if !fs::metadata(&source)?.is_file() {
            return Err("源路径必须是文件".into());
        }

        let output_arg = std::path::absolute(parameters.path("output")?)?;
        let parent_arg = output_arg.parent().ok_or("输出路径缺少父目录")?;
        let parent = fs::canonicalize(parent_arg)?;
        if !parent.is_dir() {
            return Err("输出父路径必须是目录".into());
        }
        let name = output_arg.file_name().ok_or("输出路径必须包含文件名")?;
        let output = parent.join(name);
        if source == output {
            return Err("输出不能覆盖原始 nscript.dat".into());
        }

        let packed = fs::read(&source)?;
        let plaintext = unpack_bytes(&packed);
        let original_output = read_existing_output(&output)?;
        let overwrite = parameters.flag("overwrite");
        if original_output.is_some() && !overwrite {
            return Err(format!("输出已存在，默认不覆盖: {}", output.display()).into());
        }

        let preview = Preview {
            inputs: vec![source.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "读取并保留源文件快照".into(),
                "逐字节 XOR 0x84 解码".into(),
                "写入同目录临时文件并安全提交".into(),
            ],
            details: vec![
                format!("源文件: {} ({} 字节)", source.display(), packed.len()),
                format!("明文长度: {} 字节；不追加运行时生成的换行", plaintext.len()),
            ],
        };
        Ok(Box::new(UnpackJob {
            preview,
            output,
            plaintext,
            original_output,
            overwrite,
        }))
    }
}

struct UnpackJob {
    preview: Preview,
    output: PathBuf,
    plaintext: Vec<u8>,
    original_output: Option<Vec<u8>>,
    overwrite: bool,
}

impl PreparedOperation for UnpackJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("将解码结果写入临时文件")?;
        let parent = self.output.parent().ok_or("输出路径缺少父目录")?;
        let (stage, mut file) = Temporary::create(parent)?;
        file.write_all(&self.plaintext)?;
        file.sync_all()?;
        drop(file);

        let current_output = read_existing_output(&self.output)?;
        if current_output != self.original_output {
            return Err("输出在预检后发生变化，请重新执行".into());
        }
        if current_output.is_some() && !self.overwrite {
            return Err("输出已存在，默认不覆盖".into());
        }

        commit_file(&stage.path, &self.output, current_output.is_some())?;
        Ok(RunReport {
            summary: "nscript.dat 解封包完成。".into(),
            totals: vec![("字节数".into(), self.plaintext.len() as u64)],
            outputs: vec![self.output.clone()],
            ..RunReport::default()
        })
    }
}

fn read_existing_output(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err(format!("拒绝把符号链接作为输出: {}", path.display()).into());
            }
            if !metadata.is_file() {
                return Err(format!("输出路径不是普通文件: {}", path.display()).into());
            }
            Ok(Some(fs::read(path)?))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Error(error.to_string())),
    }
}

struct Temporary {
    path: PathBuf,
}

impl Temporary {
    fn create(parent: &Path) -> Result<(Self, File)> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let path = parent.join(format!(
                ".watashi-saku-hana-{}-{}.tmp",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => return Ok((Self { path }, file)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
        Err("无法创建唯一的临时输出文件".into())
    }
}

impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(windows)]
fn commit_file(stage: &Path, output: &Path, replace: bool) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    extern "system" {
        fn MoveFileExW(existing: *const u16, new: *const u16, flags: u32) -> i32;
    }
    let stage: Vec<u16> = stage.as_os_str().encode_wide().chain(Some(0)).collect();
    let output: Vec<u16> = output.as_os_str().encode_wide().chain(Some(0)).collect();
    // MOVEFILE_REPLACE_EXISTING is used only after explicit overwrite authorization.
    let flags = if replace { 1 } else { 0 };
    let success = unsafe { MoveFileExW(stage.as_ptr(), output.as_ptr(), flags) };
    if success == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}

#[cfg(not(windows))]
fn commit_file(stage: &Path, output: &Path, replace: bool) -> Result<()> {
    if replace {
        fs::rename(stage, output)?;
    } else {
        fs::hard_link(stage, output)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::unpack_bytes;

    #[test]
    fn xor_decode_is_its_own_inverse_for_every_byte() {
        let bytes: Vec<u8> = (0u8..=u8::MAX).collect();
        assert_eq!(unpack_bytes(&unpack_bytes(&bytes)), bytes);
    }
}
