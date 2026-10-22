//! Real filesystem operations demonstrating the shared panel, without game logic.
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use vn_cli::*;

struct Inspect;
impl Operation for Inspect {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "inspect",
            "检查输入文件",
            vec![
                Field::new("source", "原始文件", FieldKind::Path).required(),
                Field::new(
                    "translation",
                    "独立译后 JSON 文件（可多选）",
                    FieldKind::Paths,
                ),
            ],
        );
        spec.writes = false;
        spec.description = "只读取文件大小，演示独立输入选择；不解析游戏或 JSON 内容。".into();
        spec
    }
    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            parameters.set("source", Value::Path(path.clone()));
        }
        if paths.len() > 1 {
            parameters.set("translation", Value::Paths(paths[1..].to_vec()));
        }
        Ok(())
    }
    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let mut inputs = vec![parameters.path("source")?.to_path_buf()];
        if let Some(Value::Paths(paths)) = parameters.get("translation") {
            inputs.extend(paths.clone());
        }
        let mut details = Vec::new();
        for path in &inputs {
            let metadata = fs::metadata(path)
                .map_err(|e| Error(format!("无法读取 {}: {e}", path.display())))?;
            if !metadata.is_file() {
                return Err(format!("需要文件: {}", path.display()).into());
            }
            details.push(format!("{}: {} 字节", path.display(), metadata.len()));
        }
        let count = inputs.len() as u64;
        Ok(Box::new(Inspection {
            preview: Preview {
                inputs,
                details,
                steps: vec!["读取所选文件的元数据".into()],
                ..Preview::default()
            },
            count,
        }))
    }
}
struct Inspection {
    preview: Preview,
    count: u64,
}
impl PreparedOperation for Inspection {
    fn preview(&self) -> &Preview {
        &self.preview
    }
    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        for detail in &self.preview.details {
            progress.report(detail)?;
        }
        Ok(RunReport {
            summary: "输入检查完成，文件内容未改动。".into(),
            totals: vec![("文件数".into(), self.count)],
            ..RunReport::default()
        })
    }
}

struct CopyFile;
impl Operation for CopyFile {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "copy",
            "复制文件（接入示例）",
            vec![
                Field::new("source", "原始文件", FieldKind::Path).required(),
                Field::new("output", "输出文件", FieldKind::Path).required(),
            ],
        );
        spec.primary = false;
        spec.description = "实际复制到独立路径，演示预检、确认及安全提交。".into();
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
        if !source.is_file() {
            return Err("原始路径必须是文件".into());
        }
        let output = std::path::absolute(parameters.path("output")?)?;
        // This example requires an existing parent, avoiding hidden directory creation.
        let parent = fs::canonicalize(output.parent().ok_or("输出缺少父目录")?)?;
        if !parent.is_dir() {
            return Err("输出父路径不是目录".into());
        }
        let output = parent.join(output.file_name().ok_or("输出必须有文件名")?);
        if output.try_exists()? && !output.is_file() {
            return Err("输出路径已经是目录".into());
        }
        let bytes = fs::read(&source)?;
        let original_output = if output.try_exists()? {
            Some(fs::read(&output)?)
        } else {
            None
        };
        let preview = Preview {
            inputs: vec![source],
            outputs: vec![output.clone()],
            steps: vec![
                "读取源文件快照".into(),
                "写入同目录临时文件并提交到输出".into(),
            ],
            details: vec![format!("待复制: {} 字节", bytes.len())],
        };
        Ok(Box::new(CopyJob {
            preview,
            output,
            bytes,
            original_output,
            overwrite: parameters.flag("overwrite"),
        }))
    }
}
struct CopyJob {
    preview: Preview,
    output: PathBuf,
    bytes: Vec<u8>,
    original_output: Option<Vec<u8>>,
    overwrite: bool,
}

// Exclusive temporary files are owned by this guard; never remove a path we did not create.
struct Temporary {
    path: PathBuf,
}
impl Temporary {
    fn create(parent: &Path) -> Result<(Self, File)> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let path = parent.join(format!(
                ".panel-demo-{}-{}.tmp",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => return Ok((Self { path }, file)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Err("无法分配临时文件".into())
    }
}
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

impl PreparedOperation for CopyJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }
    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("写入临时文件，原始文件保持不动")?;
        let (stage, mut file) = Temporary::create(self.output.parent().ok_or("输出缺少父目录")?)?;
        file.write_all(&self.bytes)?;
        file.sync_all()?;
        drop(file);
        let current = if self.output.try_exists()? {
            Some(fs::read(&self.output)?)
        } else {
            None
        };
        if current != self.original_output {
            return Err("输出在预检后发生变化，请重新执行预检".into());
        }
        if current.is_some() && !self.overwrite {
            return Err("输出已存在，默认不覆盖".into());
        }
        commit_file(&stage.path, &self.output, current.is_some())?;
        Ok(RunReport {
            summary: "复制完成。".into(),
            totals: vec![("字节数".into(), self.bytes.len() as u64)],
            outputs: vec![self.output.clone()],
            ..RunReport::default()
        })
    }
}

// Windows replacement is one filesystem operation; no delete-then-rename gap.
#[cfg(windows)]
fn commit_file(stage: &Path, output: &Path, replace: bool) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    extern "system" {
        fn MoveFileExW(existing: *const u16, new: *const u16, flags: u32) -> i32;
    }
    let stage: Vec<u16> = stage.as_os_str().encode_wide().chain(Some(0)).collect();
    let output: Vec<u16> = output.as_os_str().encode_wide().chain(Some(0)).collect();
    // MOVEFILE_REPLACE_EXISTING, only after explicit overwrite authorization.
    let success = unsafe { MoveFileExW(stage.as_ptr(), output.as_ptr(), u32::from(replace)) };
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

fn main() -> ExitCode {
    let result = (|| {
        let executable = std::env::current_exe()?.into_os_string();
        Panel::new(
            executable,
            "共享 CLI 面板示例",
            env!("CARGO_PKG_VERSION"),
            vec![Box::new(Inspect), Box::new(CopyFile)],
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
