//! Transactional directory output shared by export and import operations.
use crate::Result;
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Clone, Debug)]
pub struct ProtectedInput {
    pub path: PathBuf,
}

pub struct Prepared {
    sources: Vec<ProtectedInput>,
    output: PathBuf,
    payloads: BTreeMap<String, Vec<u8>>,
    pub disks: usize,
    pub files: usize,
    pub text_units: usize,
    pub translation_files: usize,
    pub changed_entries: usize,
    pub font_glyphs: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct OutputMetrics {
    pub disks: usize,
    pub files: usize,
    pub text_units: usize,
    pub translation_files: usize,
    pub changed_entries: usize,
    pub font_glyphs: usize,
}

fn output_absent(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("检查输出路径失败: {error}")),
        Ok(_) => Err(format!("输出已存在，拒绝覆盖: {}", path.display())),
    }
}

fn safe_relative(raw: &str) -> Result<PathBuf> {
    let normalized = raw.replace('\\', "/");
    let path = Path::new(&normalized);
    if normalized.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("不安全的相对输出路径: {raw}"));
    }
    Ok(path.to_path_buf())
}

impl Prepared {
    pub fn new(
        sources: Vec<ProtectedInput>,
        output: &Path,
        files: BTreeMap<String, Vec<u8>>,
        metrics: OutputMetrics,
    ) -> Result<Self> {
        let absolute = std::path::absolute(output).map_err(|error| error.to_string())?;
        output_absent(&absolute)?;
        let name = absolute.file_name().ok_or("输出路径缺少目录名")?;
        let parent = fs::canonicalize(absolute.parent().ok_or("输出路径缺少父目录")?)
            .map_err(|error| format!("输出父目录不可用: {error}"))?;
        if !parent.is_dir() {
            return Err(format!("输出父路径不是目录: {}", parent.display()));
        }
        let output = parent.join(name);
        output_absent(&output)?;
        for source in &sources {
            let source_path = fs::canonicalize(&source.path)
                .map_err(|error| format!("来源路径不可用 {}: {error}", source.path.display()))?;
            if output == source_path
                || output.starts_with(&source_path)
                || source_path.starts_with(&output)
            {
                return Err(format!("输出与受保护来源重叠: {}", source_path.display()));
            }
        }
        for path in files.keys() {
            safe_relative(path)?;
        }
        Ok(Self {
            sources,
            output,
            payloads: files,
            disks: metrics.disks,
            files: metrics.files,
            text_units: metrics.text_units,
            translation_files: metrics.translation_files,
            changed_entries: metrics.changed_entries,
            font_glyphs: metrics.font_glyphs,
        })
    }

    pub fn inputs(&self) -> Vec<PathBuf> {
        self.sources
            .iter()
            .map(|source| source.path.clone())
            .collect()
    }

    pub fn output(&self) -> &Path {
        &self.output
    }

    pub fn execute(self) -> Result<PathBuf> {
        output_absent(&self.output)?;
        let parent = self.output.parent().ok_or("输出路径缺少父目录")?;
        let stage = Stage::new(parent)?;
        for (relative, bytes) in &self.payloads {
            let path = stage.path.join(safe_relative(relative)?);
            let parent = path.parent().ok_or("暂存文件缺少父目录")?;
            fs::create_dir_all(parent)
                .map_err(|error| format!("建立暂存目录失败 {}: {error}", parent.display()))?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|error| format!("创建暂存文件失败 {}: {error}", path.display()))?;
            file.write_all(bytes)
                .map_err(|error| format!("写入暂存文件失败 {}: {error}", path.display()))?;
            file.sync_all()
                .map_err(|error| format!("同步暂存文件失败 {}: {error}", path.display()))?;
            drop(file);
            if fs::read(&path).map_err(|error| error.to_string())? != *bytes {
                return Err(format!("暂存文件读回不匹配: {relative}"));
            }
        }
        output_absent(&self.output)?;
        fs::rename(&stage.path, &self.output)
            .map_err(|error| format!("提交输出目录失败: {error}"))?;
        Ok(self.output)
    }
}

pub fn directory_input(path: &Path) -> Result<ProtectedInput> {
    let resolved = fs::canonicalize(path)
        .map_err(|error| format!("解析目录来源失败 {}: {error}", path.display()))?;
    if !resolved.is_dir() {
        return Err(format!("来源不是目录: {}", resolved.display()));
    }
    Ok(ProtectedInput { path: resolved })
}

struct Stage {
    path: PathBuf,
    parent: PathBuf,
    committed: bool,
}

impl Stage {
    fn new(parent: &Path) -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let path = parent.join(format!(
                ".jesus2-stage-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => {
                    let path = fs::canonicalize(&path).map_err(|error| error.to_string())?;
                    if path.parent() != Some(parent) {
                        return Err("暂存目录不在指定输出父目录下".into());
                    }
                    return Ok(Self {
                        path,
                        parent: parent.to_path_buf(),
                        committed: false,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(format!("创建暂存目录失败: {error}")),
            }
        }
        Err("无法分配唯一暂存目录".into())
    }
}

impl Drop for Stage {
    fn drop(&mut self) {
        if !self.committed {
            if let Ok(path) = fs::canonicalize(&self.path) {
                if path.parent() == Some(self.parent.as_path()) {
                    let _ = fs::remove_dir_all(path);
                }
            }
        }
    }
}
