use crate::{sha256, Result};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Default)]
pub struct Inputs {
    snapshots: BTreeMap<PathBuf, String>,
    pub roots: Vec<PathBuf>,
}
impl Inputs {
    pub fn read(&mut self, path: &Path) -> Result<Vec<u8>> {
        let path = fs::canonicalize(path)?;
        let bytes = fs::read(&path)?;
        self.snapshots.insert(path.clone(), sha256(&bytes));
        self.roots.push(path);
        Ok(bytes)
    }
    pub fn protect(&mut self, path: &Path) -> Result<()> {
        self.roots.push(fs::canonicalize(path)?);
        Ok(())
    }
    pub fn verify(&self) -> Result<()> {
        for (path, expected) in &self.snapshots {
            if sha256(&fs::read(path)?) != *expected {
                return Err(format!("输入在预检后变化: {}", path.display()).into());
            }
        }
        Ok(())
    }
    pub fn output(&self, path: &Path) -> Result<PathBuf> {
        let path = std::path::absolute(path)?;
        let parent = fs::canonicalize(path.parent().ok_or("输出缺少父目录")?)?;
        let output = parent.join(path.file_name().ok_or("输出缺少目录名")?);
        if output.try_exists()? {
            return Err("输出已存在，工具始终拒绝覆盖".into());
        }
        for input in &self.roots {
            if overlap(&output, input) {
                return Err("输出与受保护输入重叠".into());
            }
        }
        Ok(output)
    }
}
fn overlap(a: &Path, b: &Path) -> bool {
    #[cfg(windows)]
    {
        let a = a.to_string_lossy().to_lowercase();
        let b = b.to_string_lossy().to_lowercase();
        a == b || a.starts_with(&(b.clone() + "\\")) || b.starts_with(&(a + "\\"))
    }
    #[cfg(not(windows))]
    {
        a.starts_with(b) || b.starts_with(a)
    }
}
struct Stage(PathBuf);
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
pub fn publish(inputs: &Inputs, output: &Path, files: &BTreeMap<String, Vec<u8>>) -> Result<()> {
    let output = inputs.output(output)?;
    inputs.verify()?;
    for name in files.keys() {
        if Path::new(name)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err("输出相对路径含非法组件".into());
        }
    }
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = output.parent().ok_or("输出父目录缺失")?;
    let stage = loop {
        let path = parent.join(format!(
            ".suishou-stage-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&path) {
            Ok(()) => break Stage(path),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.into()),
        }
    };
    for (name, bytes) in files {
        let path = stage.0.join(name);
        fs::create_dir_all(path.parent().ok_or("文件父目录缺失")?)?;
        let mut f = OpenOptions::new().write(true).create_new(true).open(path)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    inputs.verify()?;
    inputs.output(&output)?;
    #[cfg(windows)]
    {
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
        }
        use std::os::windows::ffi::OsStrExt;
        let from = stage
            .0
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        let to = output
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), 0) } == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
    }
    #[cfg(not(windows))]
    fs::rename(&stage.0, &output)?;
    Ok(())
}
