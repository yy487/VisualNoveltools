//! Staged output for a complete DOS directory or text workspace.

use crate::{sha256_hex, Result};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use vn_cli::{PreparedOperation, Preview, Progress, Result as CliResult, RunReport};

#[derive(Clone, Debug, Default)]
pub struct Metrics {
    pub files: usize,
    pub modules: usize,
    pub text_entries: usize,
    pub changed_entries: usize,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug)]
struct SourceGuard {
    path: PathBuf,
    digest: String,
}

pub struct Prepared {
    guards: Vec<SourceGuard>,
    output: PathBuf,
    payloads: BTreeMap<String, Vec<u8>>,
    preview: Preview,
    summary: String,
    metrics: Metrics,
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
        return Err(format!("输出文件相对路径不安全: {raw}"));
    }
    Ok(path.to_path_buf())
}

fn fingerprint(path: &Path) -> Result<String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("读取受保护来源失败 {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("受保护来源不能是符号链接: {}", path.display()));
    }
    if metadata.is_file() {
        let bytes = fs::read(path)
            .map_err(|error| format!("读取受保护文件失败 {}: {error}", path.display()))?;
        return Ok(sha256_hex(&bytes));
    }
    if !metadata.is_dir() {
        return Err(format!("受保护来源不是普通文件或目录: {}", path.display()));
    }

    let mut entries = Vec::new();
    collect_files(path, path, &mut entries)?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    let mut canonical = Vec::new();
    for (relative, bytes) in entries {
        let path_bytes = relative.as_bytes();
        canonical.extend_from_slice(&(path_bytes.len() as u64).to_le_bytes());
        canonical.extend_from_slice(path_bytes);
        canonical.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        canonical.extend_from_slice(&Sha256Bytes::digest(&bytes));
    }
    Ok(sha256_hex(&canonical))
}

fn collect_files(root: &Path, directory: &Path, output: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("遍历目录失败 {}: {error}", directory.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| format!("读取目录项失败 {}: {error}", directory.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("读取文件元数据失败 {}: {error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("输入树中不支持符号链接: {}", path.display()));
        }
        if metadata.is_dir() {
            collect_files(root, &path, output)?;
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| "目录遍历越出来源根目录")?;
            let relative = relative
                .components()
                .map(|component| {
                    component
                        .as_os_str()
                        .to_str()
                        .ok_or_else(|| format!("路径包含非 Unicode 文件名: {}", path.display()))
                })
                .collect::<Result<Vec<_>>>()?
                .join("/");
            output.push((
                relative,
                fs::read(&path)
                    .map_err(|error| format!("读取来源文件失败 {}: {error}", path.display()))?,
            ));
        } else {
            return Err(format!("输入树中不支持特殊文件: {}", path.display()));
        }
    }
    Ok(())
}

struct Sha256Bytes;
impl Sha256Bytes {
    fn digest(bytes: &[u8]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        Sha256::digest(bytes).into()
    }
}

impl Prepared {
    pub fn new(
        inputs: Vec<PathBuf>,
        output: &Path,
        payloads: BTreeMap<String, Vec<u8>>,
        summary: impl Into<String>,
        steps: Vec<String>,
        details: Vec<String>,
        metrics: Metrics,
    ) -> Result<Self> {
        let output = new_output(output)?;
        let mut guards = Vec::with_capacity(inputs.len());
        for input in inputs {
            let path = fs::canonicalize(&input)
                .map_err(|error| format!("来源路径无法解析 {}: {error}", input.display()))?;
            let digest = fingerprint(&path)?;
            if output == path || output.starts_with(&path) || path.starts_with(&output) {
                return Err(format!("输出与受保护来源重叠: {}", path.display()));
            }
            guards.push(SourceGuard { path, digest });
        }
        for path in payloads.keys() {
            safe_relative(path)?;
        }
        let preview = Preview {
            inputs: guards.iter().map(|guard| guard.path.clone()).collect(),
            outputs: vec![output.clone()],
            steps,
            details,
        };
        Ok(Self {
            guards,
            output,
            payloads,
            preview,
            summary: summary.into(),
            metrics,
        })
    }

    fn revalidate_sources(&self) -> Result<()> {
        for guard in &self.guards {
            let current = fingerprint(&guard.path)?;
            if current != guard.digest {
                return Err(format!(
                    "预检后来源已变化，停止写入: {}",
                    guard.path.display()
                ));
            }
        }
        Ok(())
    }
}

impl PreparedOperation for Prepared {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> CliResult<RunReport> {
        self.revalidate_sources().map_err(vn_cli::Error)?;
        ensure_absent(&self.output).map_err(vn_cli::Error)?;
        progress.report("写入并读回暂存文件")?;
        let mut stage =
            Stage::new(self.output.parent().ok_or("输出目录缺少父目录")?).map_err(vn_cli::Error)?;
        for (relative, bytes) in &self.payloads {
            let target = stage
                .path
                .join(safe_relative(relative).map_err(vn_cli::Error)?);
            let parent = target.parent().ok_or("暂存文件缺少父目录")?;
            fs::create_dir_all(parent).map_err(|error| {
                vn_cli::Error(format!("建立暂存目录失败 {}: {error}", parent.display()))
            })?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target)
                .map_err(|error| {
                    vn_cli::Error(format!("创建暂存文件失败 {}: {error}", target.display()))
                })?;
            file.write_all(bytes).map_err(|error| {
                vn_cli::Error(format!("写入暂存文件失败 {}: {error}", target.display()))
            })?;
            file.sync_all().map_err(|error| {
                vn_cli::Error(format!("同步暂存文件失败 {}: {error}", target.display()))
            })?;
            drop(file);
            let readback = fs::read(&target).map_err(|error| {
                vn_cli::Error(format!("读回暂存文件失败 {}: {error}", target.display()))
            })?;
            if &readback != bytes {
                return Err(vn_cli::Error(format!("暂存文件读回不匹配: {relative}")));
            }
        }
        self.revalidate_sources().map_err(vn_cli::Error)?;
        ensure_absent(&self.output).map_err(vn_cli::Error)?;
        fs::rename(&stage.path, &self.output)
            .map_err(|error| vn_cli::Error(format!("提交输出失败: {error}")))?;
        stage.committed = true;

        let mut totals = vec![
            ("文件".into(), self.metrics.files as u64),
            ("场景模块".into(), self.metrics.modules as u64),
            ("文本条目".into(), self.metrics.text_entries as u64),
            ("译文变更".into(), self.metrics.changed_entries as u64),
        ];
        totals.retain(|(_, count)| *count != 0);
        Ok(RunReport {
            summary: self.summary,
            totals,
            outputs: vec![self.output],
            warnings: self.metrics.warnings,
        })
    }
}

fn new_output(path: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(path)
        .map_err(|error| format!("解析输出路径失败 {}: {error}", path.display()))?;
    ensure_absent(&absolute)?;
    let name = absolute
        .file_name()
        .ok_or_else(|| format!("输出路径缺少目录名: {}", absolute.display()))?;
    let parent = fs::canonicalize(absolute.parent().ok_or("输出路径缺少父目录")?)
        .map_err(|error| format!("输出父目录不可用: {error}"))?;
    if !parent.is_dir() {
        return Err(format!("输出父路径不是目录: {}", parent.display()));
    }
    let resolved = parent.join(name);
    ensure_absent(&resolved)?;
    Ok(resolved)
}

fn ensure_absent(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("检查输出路径失败 {}: {error}", path.display())),
        Ok(_) => Err(format!("输出已存在，拒绝覆盖: {}", path.display())),
    }
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
                ".cyber-illusion-stage-{}-{}",
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
        Err("无法创建唯一的暂存目录".into())
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
