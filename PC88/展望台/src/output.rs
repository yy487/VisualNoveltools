//! Snapshot-checked directory staging for game-adapter operations.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use crate::Result;

static NEXT_STAGE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectorySnapshot(Option<String>);

impl DirectorySnapshot {
    /// Captures a directory's relative paths, entry types, lengths, and bytes.
    /// Symlinks and special files are rejected instead of being followed.
    pub fn capture(path: &Path) -> Result<Self> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Self(None)),
            Err(error) => return Err(format!("无法检查输出 {}：{error}", path.display())),
        };
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(format!(
                "输出目标必须是普通目录，且不能是符号链接：{}",
                path.display()
            ));
        }
        let mut hasher = Sha256::new();
        hash_directory(path, Path::new(""), &mut hasher)?;
        Ok(Self(Some(format!("{:X}", hasher.finalize()))))
    }

    pub fn exists(&self) -> bool {
        self.0.is_some()
    }
}

/// Writes a complete staged directory and then swaps it into place.
///
/// `expected_output` must be captured during operation preparation. Existing
/// output is replaced only when `overwrite` is true and the captured tree has
/// not changed since preparation.
pub fn write_directory_transaction(
    output: &Path,
    files: &BTreeMap<PathBuf, Vec<u8>>,
    overwrite: bool,
    expected_output: &DirectorySnapshot,
) -> Result<()> {
    validate_relative_files(files)?;
    if expected_output.exists() && !overwrite {
        return Err(format!(
            "输出已存在：{}；如需替换请显式启用 overwrite",
            output.display()
        ));
    }
    if DirectorySnapshot::capture(output)? != *expected_output {
        return Err(format!(
            "输出在预检后发生变化，拒绝提交：{}",
            output.display()
        ));
    }

    let parent = output_parent(output);
    if !parent.is_dir() {
        return Err(format!("输出父目录不存在或不是目录：{}", parent.display()));
    }
    let stage = create_unique_directory(&parent, output, "stage")?;
    let mut stage_guard = StageGuard(Some(stage.clone()));
    for (relative, bytes) in files {
        let destination = stage.join(relative);
        if let Some(directory) = destination.parent() {
            fs::create_dir_all(directory)
                .map_err(|error| format!("无法创建暂存子目录 {}：{error}", directory.display()))?;
        }
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
            .map_err(|error| format!("无法创建暂存文件 {}：{error}", destination.display()))?;
        file.write_all(bytes)
            .map_err(|error| format!("写入暂存文件 {} 失败：{error}", destination.display()))?;
        file.sync_all()
            .map_err(|error| format!("刷新暂存文件 {} 失败：{error}", destination.display()))?;
    }

    if DirectorySnapshot::capture(output)? != *expected_output {
        return Err(format!(
            "暂存期间输出目标发生变化，拒绝提交：{}",
            output.display()
        ));
    }

    if !expected_output.exists() {
        fs::rename(&stage, output)
            .map_err(|error| format!("无法将暂存目录提交到 {}：{error}", output.display()))?;
        stage_guard.0 = None;
        return Ok(());
    }

    let backup = create_unique_name(&parent, output, "backup")?;
    fs::rename(output, &backup)
        .map_err(|error| format!("无法暂存原输出 {}：{error}", output.display()))?;
    if let Err(error) = fs::rename(&stage, output) {
        let rollback = fs::rename(&backup, output);
        return match rollback {
            Ok(()) => Err(format!("提交新输出失败，已恢复旧目录：{error}")),
            Err(rollback_error) => Err(format!(
                "提交新输出失败：{error}；恢复旧目录也失败：{rollback_error}；旧目录保留在 {}",
                backup.display()
            )),
        };
    }
    stage_guard.0 = None;
    fs::remove_dir_all(&backup).map_err(|error| {
        format!(
            "新输出已提交，但清理旧目录 {} 失败：{error}",
            backup.display()
        )
    })?;
    Ok(())
}

fn validate_relative_files(files: &BTreeMap<PathBuf, Vec<u8>>) -> Result<()> {
    if files.is_empty() {
        return Err("拒绝提交空输出目录".into());
    }
    for path in files.keys() {
        if path.as_os_str().is_empty()
            || path
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(format!(
                "输出文件名必须是安全的相对路径：{}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn hash_directory(path: &Path, relative: &Path, hasher: &mut Sha256) -> Result<()> {
    let mut entries = fs::read_dir(path)
        .map_err(|error| format!("无法读取输出目录 {}：{error}", path.display()))?
        .map(|entry| entry.map_err(|error| error.to_string()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| format!("枚举输出目录 {} 失败：{error}", path.display()))?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let child = entry.path();
        let name = entry.file_name();
        let child_relative = relative.join(&name);
        let metadata = fs::symlink_metadata(&child)
            .map_err(|error| format!("无法检查输出项目 {}：{error}", child.display()))?;
        let relative_text = child_relative.to_string_lossy().replace('\\', "/");
        hasher.update((relative_text.len() as u64).to_le_bytes());
        hasher.update(relative_text.as_bytes());
        if metadata.file_type().is_symlink() {
            return Err(format!("输出目录不能包含符号链接：{}", child.display()));
        } else if metadata.is_dir() {
            hasher.update([b'D']);
            hash_directory(&child, &child_relative, hasher)?;
        } else if metadata.is_file() {
            hasher.update([b'F']);
            let bytes = fs::read(&child)
                .map_err(|error| format!("无法读取输出文件 {}：{error}", child.display()))?;
            hasher.update((bytes.len() as u64).to_le_bytes());
            hasher.update(&bytes);
        } else {
            return Err(format!("输出目录包含不支持的文件类型：{}", child.display()));
        }
    }
    Ok(())
}

fn output_parent(output: &Path) -> PathBuf {
    output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn create_unique_directory(parent: &Path, output: &Path, role: &str) -> Result<PathBuf> {
    loop {
        let candidate = create_unique_name(parent, output, role)?;
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!("无法创建暂存目录 {}：{error}", candidate.display()));
            }
        }
    }
}

fn create_unique_name(parent: &Path, output: &Path, role: &str) -> Result<PathBuf> {
    let name = output
        .file_name()
        .ok_or_else(|| format!("输出路径必须包含目录名：{}", output.display()))?;
    let sequence = NEXT_STAGE_ID.fetch_add(1, Ordering::Relaxed);
    Ok(parent.join(format!(
        ".{}.tenboudai-{role}-{}-{sequence}",
        name.to_string_lossy(),
        std::process::id()
    )))
}

struct StageGuard(Option<PathBuf>);

impl Drop for StageGuard {
    fn drop(&mut self) {
        if let Some(path) = &self.0 {
            let _ = fs::remove_dir_all(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "tenboudai-output-test-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create test directory");
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn directory_write_commits_and_detects_post_prepare_changes() {
        let temp = TestDirectory::new();
        let output = temp.0.join("result");
        let absent = DirectorySnapshot::capture(&output).expect("capture absent target");
        let first_files = BTreeMap::from([(PathBuf::from("nested/data.bin"), b"first".to_vec())]);
        write_directory_transaction(&output, &first_files, false, &absent)
            .expect("commit first output");
        assert_eq!(
            fs::read(output.join("nested/data.bin")).expect("read output"),
            b"first"
        );

        let prepared = DirectorySnapshot::capture(&output).expect("capture existing output");
        fs::write(output.join("nested/data.bin"), b"changed").expect("modify output after prepare");
        let replacement = BTreeMap::from([(PathBuf::from("result.bin"), b"replacement".to_vec())]);
        let stale = write_directory_transaction(&output, &replacement, true, &prepared)
            .expect_err("stale output snapshot must fail");
        assert!(stale.contains("预检后发生变化"));

        let current = DirectorySnapshot::capture(&output).expect("capture changed output");
        write_directory_transaction(&output, &replacement, true, &current)
            .expect("replace output using current snapshot");
        assert!(!output.join("nested/data.bin").exists());
        assert_eq!(
            fs::read(output.join("result.bin")).expect("read replacement"),
            b"replacement"
        );
    }

    #[test]
    fn directory_write_rejects_unsafe_relative_paths() {
        let temp = TestDirectory::new();
        let output = temp.0.join("result");
        let snapshot = DirectorySnapshot::capture(&output).expect("capture absent target");
        let files = BTreeMap::from([(PathBuf::from("../escape.bin"), b"no".to_vec())]);
        let error = write_directory_transaction(&output, &files, false, &snapshot)
            .expect_err("parent traversal must fail");
        assert!(error.contains("安全的相对路径"));
        assert!(!output.exists());
    }
}
