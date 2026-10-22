use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};
use vn_cli::{Error, Result};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

pub fn resolve_file_output(path: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    let parent = absolute.parent().ok_or("输出文件缺少父目录")?;
    let parent =
        fs::canonicalize(parent).map_err(|error| Error(format!("输出父目录不可用：{error}")))?;
    let name = absolute.file_name().ok_or("输出路径必须包含文件名")?;
    if !parent.is_dir() {
        return Err("输出父路径必须是目录".into());
    }
    Ok(parent.join(name))
}

pub fn resolve_directory_output(path: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    let parent = absolute.parent().ok_or("输出目录缺少父目录")?;
    let parent =
        fs::canonicalize(parent).map_err(|error| Error(format!("输出父目录不可用：{error}")))?;
    let name = absolute.file_name().ok_or("输出目录路径必须包含目录名")?;
    if !parent.is_dir() {
        return Err("输出父路径必须是目录".into());
    }
    Ok(parent.join(name))
}

pub fn ensure_inputs_disjoint_from_output(inputs: &[&Path], output: &Path) -> Result<()> {
    for input in inputs {
        let input = fs::canonicalize(input)?;
        if input == output || input.starts_with(output) || output.starts_with(&input) {
            return Err(format!(
                "输入与输出路径重叠：{} / {}",
                input.display(),
                output.display()
            )
            .into());
        }
    }
    Ok(())
}

pub fn read_existing_file(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(format!("拒绝把非普通文件作为输出：{}", path.display()).into());
            }
            Ok(Some(fs::read(path)?))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub fn prepare_file_output(path: &Path, overwrite: bool) -> Result<(PathBuf, Option<Vec<u8>>)> {
    let output = resolve_file_output(path)?;
    let original = read_existing_file(&output)?;
    if original.is_some() && !overwrite {
        return Err(format!("输出已存在，默认不覆盖：{}", output.display()).into());
    }
    Ok((output, original))
}

pub fn write_file_atomic(
    output: &Path,
    bytes: &[u8],
    expected_output: &Option<Vec<u8>>,
    overwrite: bool,
) -> Result<()> {
    write_file_atomic_with(output, expected_output, overwrite, |file| {
        file.write_all(bytes)?;
        Ok(())
    })
}

pub fn write_file_atomic_with(
    output: &Path,
    expected_output: &Option<Vec<u8>>,
    overwrite: bool,
    write_contents: impl FnOnce(&mut File) -> Result<()>,
) -> Result<()> {
    let parent = output.parent().ok_or("输出文件缺少父目录")?;
    let (stage, mut file) = TemporaryFile::create(parent, output)?;
    write_contents(&mut file)?;
    file.sync_all()?;
    drop(file);

    let current = read_existing_file(output)?;
    if current != *expected_output {
        return Err("输出在预检后发生变化；未提交写入".into());
    }
    if current.is_some() && !overwrite {
        return Err("输出已存在，默认不覆盖".into());
    }
    commit_file(&stage.path, output, current.is_some())
}

pub fn directory_snapshot(path: &Path) -> Result<Option<String>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(format!("输出路径不是普通目录：{}", path.display()).into());
            }
            let mut entries = Vec::new();
            collect_directory_entries(path, Path::new(""), &mut entries)?;
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let mut hasher = Sha256::new();
            for (relative, is_dir, entry_path) in entries {
                hasher.update([if is_dir { b'D' } else { b'F' }]);
                hasher.update((relative.len() as u64).to_le_bytes());
                hasher.update(relative.as_bytes());
                if !is_dir {
                    let contents = fs::read(entry_path)?;
                    hasher.update((contents.len() as u64).to_le_bytes());
                    hasher.update(contents);
                }
            }
            Ok(Some(hex(&hasher.finalize())))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn collect_directory_entries(
    root: &Path,
    relative: &Path,
    entries: &mut Vec<(String, bool, PathBuf)>,
) -> Result<()> {
    let current = root.join(relative);
    for item in fs::read_dir(&current)? {
        let item = item?;
        let name = item.file_name();
        let child_relative = relative.join(&name);
        let child = root.join(&child_relative);
        let metadata = fs::symlink_metadata(&child)?;
        if metadata.file_type().is_symlink() {
            return Err(format!("输出目录包含符号链接：{}", child.display()).into());
        }
        let rel = child_relative.to_string_lossy().replace('\\', "/");
        if metadata.is_dir() {
            entries.push((rel, true, child.clone()));
            collect_directory_entries(root, &child_relative, entries)?;
        } else if metadata.is_file() {
            entries.push((rel, false, child));
        } else {
            return Err(format!("输出目录包含不支持的文件类型：{}", child.display()).into());
        }
    }
    Ok(())
}

pub fn prepare_directory_output(path: &Path, overwrite: bool) -> Result<(PathBuf, Option<String>)> {
    let output = resolve_directory_output(path)?;
    let original = directory_snapshot(&output)?;
    if original.is_some() && !overwrite {
        return Err(format!("输出目录已存在，默认不覆盖：{}", output.display()).into());
    }
    Ok((output, original))
}

pub fn write_directory_atomic(
    output: &Path,
    files: &[(PathBuf, Vec<u8>)],
    expected_snapshot: &Option<String>,
    overwrite: bool,
) -> Result<Option<String>> {
    let parent = output.parent().ok_or("输出目录缺少父目录")?;
    let stage = TemporaryDirectory::create(parent, output)?;
    for (relative, bytes) in files {
        validate_relative_output_path(relative)?;
        let staged_file = stage.path.join(relative);
        if let Some(parent) = staged_file.parent() {
            fs::create_dir_all(parent)?;
        }
        write_new_file(&staged_file, bytes)?;
    }

    let current = directory_snapshot(output)?;
    if current != *expected_snapshot {
        return Err("输出目录在预检后发生变化；未提交导出".into());
    }
    if current.is_some() && !overwrite {
        return Err("输出目录已存在，默认不覆盖".into());
    }
    commit_directory(&stage.path, output, expected_snapshot, overwrite)
}

pub fn write_directory_atomic_with(
    output: &Path,
    expected_snapshot: &Option<String>,
    overwrite: bool,
    build_stage: impl FnOnce(&Path) -> Result<()>,
) -> Result<Option<String>> {
    let parent = output.parent().ok_or("输出目录缺少父目录")?;
    let stage = TemporaryDirectory::create(parent, output)?;
    build_stage(&stage.path)?;

    let current = directory_snapshot(output)?;
    if current != *expected_snapshot {
        return Err("输出目录在预检后发生变化；未提交导出".into());
    }
    if current.is_some() && !overwrite {
        return Err("输出目录已存在，默认不覆盖".into());
    }
    commit_directory(&stage.path, output, expected_snapshot, overwrite)
}

pub fn write_staged_file(stage: &Path, relative: &Path, bytes: &[u8]) -> Result<()> {
    validate_relative_output_path(relative)?;
    let staged_file = stage.join(relative);
    if let Some(parent) = staged_file.parent() {
        fs::create_dir_all(parent)?;
    }
    write_new_file(&staged_file, bytes)
}

fn validate_relative_output_path(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!("输出文件必须使用安全的相对路径：{}", path.display()).into());
    }
    Ok(())
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn commit_directory(
    stage: &Path,
    output: &Path,
    expected_snapshot: &Option<String>,
    overwrite: bool,
) -> Result<Option<String>> {
    let current = directory_snapshot(output)?;
    if current != *expected_snapshot {
        return Err("输出目录在提交前发生变化；未提交导出".into());
    }
    let Some(expected) = expected_snapshot else {
        return fs::rename(stage, output)
            .map(|()| None)
            .map_err(|error| Error(format!("提交输出目录失败：{error}")));
    };
    if !overwrite {
        return Err("输出目录已存在，默认不覆盖".into());
    }

    let parent = output.parent().ok_or("输出目录缺少父目录")?;
    let backup = unused_sibling(parent, output, "backup")?;
    fs::rename(output, &backup).map_err(|error| Error(format!("暂存旧输出目录失败：{error}")))?;
    let moved_snapshot = match directory_snapshot(&backup) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let rollback = fs::rename(&backup, output);
            return Err(match rollback {
                Ok(()) => Error(format!("无法复核暂存目录；已恢复旧目录：{error}")),
                Err(rollback_error) => Error(format!(
                    "无法复核暂存目录：{error}；旧目录仍在 {}，恢复失败：{rollback_error}",
                    backup.display()
                )),
            });
        }
    };
    if moved_snapshot.as_deref() != Some(expected.as_str()) {
        let rollback = fs::rename(&backup, output);
        return Err(match rollback {
            Ok(()) => Error("输出目录在暂存时发生变化；已恢复旧目录".into()),
            Err(error) => Error(format!(
                "输出目录在暂存时发生变化；旧目录仍在 {}，恢复失败：{error}",
                backup.display()
            )),
        });
    }
    if let Err(error) = fs::rename(stage, output) {
        let rollback = fs::rename(&backup, output);
        return Err(match rollback {
            Ok(()) => Error(format!("提交新输出目录失败，已恢复旧目录：{error}")),
            Err(rollback_error) => Error(format!(
                "提交新输出目录失败：{error}；旧目录仍在 {}，恢复失败：{rollback_error}",
                backup.display()
            )),
        });
    }
    match fs::remove_dir_all(&backup) {
        Ok(()) => Ok(None),
        Err(error) => Ok(Some(format!(
            "新输出已提交；旧目录 {} 未清理：{error}",
            backup.display()
        ))),
    }
}

fn unused_sibling(parent: &Path, output: &Path, suffix: &str) -> Result<PathBuf> {
    let base = output
        .file_name()
        .ok_or("输出目录缺少名称")?
        .to_string_lossy();
    for _ in 0..100 {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(".{base}.{suffix}.{}.{}", std::process::id(), id));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("无法分配唯一备份目录".into())
}

fn commit_file(stage: &Path, output: &Path, replace: bool) -> Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "kernel32")]
        extern "system" {
            fn MoveFileExW(existing: *const u16, new: *const u16, flags: u32) -> i32;
        }
        let stage: Vec<u16> = stage.as_os_str().encode_wide().chain(Some(0)).collect();
        let output: Vec<u16> = output.as_os_str().encode_wide().chain(Some(0)).collect();
        let flags = if replace { 1 } else { 0 };
        let success = unsafe { MoveFileExW(stage.as_ptr(), output.as_ptr(), flags) };
        if success == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        if replace {
            fs::rename(stage, output)?;
        } else {
            fs::hard_link(stage, output)?;
        }
        Ok(())
    }
}

struct TemporaryFile {
    path: PathBuf,
}

impl TemporaryFile {
    fn create(parent: &Path, output: &Path) -> Result<(Self, File)> {
        let base = output
            .file_name()
            .ok_or("输出文件缺少名称")?
            .to_string_lossy();
        for _ in 0..100 {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!(".{base}.{}.{}.tmp", std::process::id(), id));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => return Ok((Self { path }, file)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
        Err("无法创建唯一的临时输出文件".into())
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn create(parent: &Path, output: &Path) -> Result<Self> {
        let base = output
            .file_name()
            .ok_or("输出目录缺少名称")?
            .to_string_lossy();
        for _ in 0..100 {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!(".{base}.stage.{}.{}", std::process::id(), id));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
        Err("无法创建唯一的暂存目录".into())
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[usize::from(byte >> 4)] as char);
        output.push(DIGITS[usize::from(byte & 0x0f)] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{
        directory_snapshot, prepare_directory_output, prepare_file_output, write_directory_atomic,
        write_file_atomic,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn create() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            for _ in 0..100 {
                let id = NEXT.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!(
                    "watashi-saku-hana-test-{}-{id}",
                    std::process::id()
                ));
                match fs::create_dir(&path) {
                    Ok(()) => return Self(path),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("create test root: {error}"),
                }
            }
            panic!("could not allocate test root");
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn atomic_file_write_refuses_preflight_changes() {
        let root = TestRoot::create();
        let output = root.0.join("packed.dat");
        let (output, original) = prepare_file_output(&output, false).unwrap();
        write_file_atomic(&output, b"initial", &original, false).unwrap();
        assert_eq!(fs::read(&output).unwrap(), b"initial");

        let (output, original) = prepare_file_output(&output, true).unwrap();
        fs::write(&output, b"concurrent change").unwrap();
        assert!(write_file_atomic(&output, b"replacement", &original, true).is_err());
        assert_eq!(fs::read(&output).unwrap(), b"concurrent change");
    }

    #[test]
    fn directory_export_commits_as_a_unit_and_rejects_changed_workspace() {
        let root = TestRoot::create();
        let output = root.0.join("workspace");
        let (output, original) = prepare_directory_output(&output, false).unwrap();
        let initial = vec![(PathBuf::from("manifest.json"), b"source".to_vec())];
        write_directory_atomic(&output, &initial, &original, false).unwrap();
        assert_eq!(fs::read(output.join("manifest.json")).unwrap(), b"source");

        let snapshot = directory_snapshot(&output).unwrap();
        fs::write(output.join("manifest.json"), b"edited").unwrap();
        let replacement = vec![(PathBuf::from("translation.json"), b"template".to_vec())];
        assert!(write_directory_atomic(&output, &replacement, &snapshot, true).is_err());
        assert_eq!(fs::read(output.join("manifest.json")).unwrap(), b"edited");
        assert!(!output.join("translation.json").exists());

        let (output, current) = prepare_directory_output(&output, true).unwrap();
        write_directory_atomic(&output, &replacement, &current, true).unwrap();
        assert_eq!(
            fs::read(output.join("translation.json")).unwrap(),
            b"template"
        );
        assert!(!output.join("manifest.json").exists());
    }
}
