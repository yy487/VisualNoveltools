use crate::{json, read, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const MARKER: &str = ".uma-output.json";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Ownership {
    schema: String,
    files: Vec<PathBuf>,
}
#[derive(PartialEq, Eq)]
struct Tree {
    files: BTreeMap<PathBuf, Vec<u8>>,
    dirs: BTreeSet<PathBuf>,
}
impl Tree {
    fn is_empty(&self) -> bool {
        self.files.is_empty() && self.dirs.is_empty()
    }
}

pub struct Job {
    pub preview: vn_cli::Preview,
    output: PathBuf,
    previous: Option<Tree>,
    files: BTreeMap<PathBuf, Vec<u8>>,
    pub report: vn_cli::RunReport,
}

fn safe(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!("不是安全相对路径: {}", path.display()));
    }
    Ok(())
}
fn exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}
fn resolve(path: &Path) -> Result<PathBuf> {
    let path = std::path::absolute(path).map_err(|e| e.to_string())?;
    if exists(&path)? {
        return fs::canonicalize(path).map_err(|e| e.to_string());
    }
    let parent = fs::canonicalize(path.parent().ok_or("输出缺少父目录")?)
        .map_err(|e| format!("输出父目录须已存在: {e}"))?;
    Ok(parent.join(path.file_name().ok_or("输出缺少名称")?))
}
fn overlaps(a: &Path, b: &Path) -> bool {
    let a = a.to_string_lossy().replace('/', "\\").to_lowercase();
    let b = b.to_string_lossy().replace('/', "\\").to_lowercase();
    a == b || a.starts_with(&(b.clone() + "\\")) || b.starts_with(&(a + "\\"))
}
fn reject_link(path: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if meta.file_type().is_symlink() {
        return Err("不接受链接作为输出树内容".into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err("不接受目录联接或重解析点".into());
        }
    }
    Ok(())
}
pub fn read_tree(root: &Path) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
    Ok(snapshot(root)?.files)
}
fn snapshot(root: &Path) -> Result<Tree> {
    reject_link(root)?;
    if !root.is_dir() {
        return Err("需要目录".into());
    }
    let mut tree = Tree {
        files: BTreeMap::new(),
        dirs: BTreeSet::new(),
    };
    let mut pending = vec![PathBuf::new()];
    while let Some(relative) = pending.pop() {
        for entry in fs::read_dir(root.join(&relative)).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            reject_link(&entry.path())?;
            let path = relative.join(entry.file_name());
            safe(&path)?;
            if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                tree.dirs.insert(path.clone());
                pending.push(path);
            } else if entry.file_type().map_err(|e| e.to_string())?.is_file() {
                tree.files.insert(path, read(&entry.path())?);
            } else {
                return Err("输出树包含特殊文件".into());
            }
        }
    }
    Ok(tree)
}
fn owned(root: &Path) -> Result<Tree> {
    reject_link(root)?;
    let owner: Ownership = serde_json::from_slice(&read(&root.join(MARKER))?)
        .map_err(|e| format!("输出不属于 uma 工具: {e}"))?;
    if owner.schema != "uma-output-v1" {
        return Err("输出归属版本不同".into());
    }
    let mut expected = BTreeSet::from([PathBuf::from(MARKER)]);
    let mut dirs = BTreeSet::new();
    for path in owner.files {
        safe(&path)?;
        if !expected.insert(path.clone()) {
            return Err("重复输出路径".into());
        }
        for parent in path
            .ancestors()
            .skip(1)
            .filter(|p| !p.as_os_str().is_empty())
        {
            dirs.insert(parent.to_path_buf());
        }
    }
    let current = snapshot(root)?;
    if current.files.keys().cloned().collect::<BTreeSet<_>>() != expected || current.dirs != dirs {
        return Err("已有输出出现外来、缺失文件或目录；拒绝覆盖".into());
    }
    Ok(current)
}

fn is_empty_directory(path: &Path) -> Result<bool> {
    reject_link(path)?;
    Ok(path.is_dir()
        && fs::read_dir(path)
            .map_err(|e| e.to_string())?
            .next()
            .is_none())
}

fn existing_output(path: &Path) -> Result<Tree> {
    if is_empty_directory(path)? {
        snapshot(path)
    } else {
        owned(path)
    }
}

struct Stage(PathBuf);
impl Stage {
    fn create(parent: &Path, role: &str) -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let path = parent.join(format!(
                ".uma-{role}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.to_string()),
            }
        }
        Err("不能创建独立暂存目录".into())
    }
    fn preserve(self) -> PathBuf {
        let p = self.0.clone();
        std::mem::forget(self);
        p
    }
}
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl Job {
    pub fn new(
        inputs: Vec<PathBuf>,
        output: &Path,
        overwrite: bool,
        mut files: BTreeMap<PathBuf, Vec<u8>>,
        report: vn_cli::RunReport,
    ) -> Result<Self> {
        // Reject an output link before canonicalizing its destination.
        if exists(output)? {
            reject_link(output)?;
        }
        let output = resolve(output)?;
        let inputs = inputs
            .into_iter()
            .map(|p| fs::canonicalize(&p).map_err(|e| format!("{}: {e}", p.display())))
            .collect::<Result<Vec<_>>>()?;
        for input in &inputs {
            if overlaps(input, &output) {
                return Err("输出与原盘、译文或字库输入路径重叠".into());
            }
        }
        let previous = if exists(&output)? {
            if !overwrite && !is_empty_directory(&output)? {
                return Err("输出已存在；默认不覆盖".into());
            }
            Some(existing_output(&output)?)
        } else {
            None
        };
        for path in files.keys() {
            safe(path)?;
        }
        if files.contains_key(Path::new(MARKER)) {
            return Err("产品路径占用归属清单".into());
        }
        let ownership = Ownership {
            schema: "uma-output-v1".into(),
            files: files.keys().cloned().collect(),
        };
        files.insert(PathBuf::from(MARKER), json(&ownership)?);
        let preview = vn_cli::Preview {
            inputs,
            outputs: vec![output.clone()],
            details: vec![
                report.summary.clone(),
                format!("写出 {} 个文件；源数据只读", files.len()),
            ],
            steps: vec![
                "解析原盘与文本，完成格式、容量及字槽预检".into(),
                "在独立暂存目录写出全部结果".into(),
                "提交整个输出目录；不执行哈希校验".into(),
            ],
        };
        Ok(Self {
            preview,
            output,
            previous,
            files,
            report,
        })
    }

    pub fn execute(mut self, progress: &mut dyn vn_cli::Progress) -> Result<vn_cli::RunReport> {
        for input in &self.preview.inputs {
            if overlaps(&resolve(input)?, &resolve(&self.output)?) {
                return Err("执行时输入输出路径发生重叠".into());
            }
        }
        progress
            .report("写出预检快照，不复读原盘或检验哈希")
            .map_err(|e| e.to_string())?;
        let parent = self.output.parent().ok_or("输出缺少父目录")?;
        let stage = Stage::create(parent, "stage")?;
        let payload = stage.0.join("payload");
        fs::create_dir(&payload).map_err(|e| e.to_string())?;
        for (relative, bytes) in self.files {
            let path = payload.join(relative);
            fs::create_dir_all(path.parent().ok_or("产品路径无父目录")?)
                .map_err(|e| e.to_string())?;
            let mut file = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&path)
                .map_err(|e| e.to_string())?;
            file.write_all(&bytes)
                .and_then(|_| file.sync_all())
                .map_err(|e| e.to_string())?;
        }
        let current = if exists(&self.output)? {
            Some(existing_output(&self.output)?)
        } else {
            None
        };
        if current != self.previous {
            return Err("输出在预检后发生变化，保留现有目录".into());
        }
        let backup = if self.previous.is_some() {
            let backup = Stage::create(parent, "backup")?;
            fs::rename(&self.output, backup.0.join("previous")).map_err(|e| e.to_string())?;
            Some(backup)
        } else {
            None
        };
        if let Err(error) = fs::rename(&payload, &self.output) {
            if let Some(backup) = backup {
                if let Err(rollback) = fs::rename(backup.0.join("previous"), &self.output) {
                    let saved = backup.preserve();
                    return Err(format!(
                        "提交失败: {error}；恢复失败: {rollback}；旧结果保存在 {}",
                        saved.display()
                    ));
                }
            }
            return Err(format!("提交失败: {error}"));
        }
        if let Some(backup) = backup {
            if !self.previous.as_ref().is_some_and(Tree::is_empty) {
                let path = backup.preserve().join("previous");
                self.report
                    .warnings
                    .push(format!("旧输出保留在 {}", path.display()));
            }
        }
        self.report.outputs = vec![self.output];
        Ok(self.report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Quiet;
    impl vn_cli::Progress for Quiet {
        fn report(&mut self, _: &str) -> vn_cli::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn protect_inputs_and_reject_output_appearing_after_prepare() {
        let temp = Stage::create(&std::env::temp_dir(), "test").unwrap();
        let source = temp.0.join("source");
        fs::create_dir(&source).unwrap();
        let files = BTreeMap::from([(PathBuf::from("x/data"), b"new".to_vec())]);
        assert!(Job::new(
            vec![source.clone()],
            &source.join("out"),
            false,
            files.clone(),
            vn_cli::RunReport::default()
        )
        .is_err());
        let output = temp.0.join("output");
        let job = Job::new(
            vec![source.clone()],
            &output,
            false,
            files.clone(),
            vn_cli::RunReport::default(),
        )
        .unwrap();
        fs::create_dir(&output).unwrap();
        fs::write(output.join("keep"), b"old").unwrap();
        assert!(job.execute(&mut Quiet).is_err());
        assert_eq!(read(&output.join("keep")).unwrap(), b"old");
        assert!(Job::new(
            vec![source],
            &output,
            true,
            files,
            vn_cli::RunReport::default()
        )
        .is_err());
    }
    #[test]
    fn explicit_overwrite_retains_recoverable_previous_output() {
        let temp = Stage::create(&std::env::temp_dir(), "test").unwrap();
        let output = temp.0.join("output");
        for (value, overwrite) in [(b"old", false), (b"new", true)] {
            let files = BTreeMap::from([(PathBuf::from("x/data"), value.to_vec())]);
            Job::new(
                vec![],
                &output,
                overwrite,
                files,
                vn_cli::RunReport::default(),
            )
            .unwrap()
            .execute(&mut Quiet)
            .unwrap();
        }
        assert_eq!(read(&output.join("x/data")).unwrap(), b"new");
        let backup = fs::read_dir(&temp.0)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".uma-backup")
            })
            .unwrap();
        assert_eq!(read(&backup.join("previous/x/data")).unwrap(), b"old");
    }

    #[test]
    fn empty_output_beside_disks_is_allowed_but_source_files_stay_protected() {
        let temp = Stage::create(&std::env::temp_dir(), "test").unwrap();
        for disk in crate::DISKS {
            fs::write(temp.0.join(disk), b"original").unwrap();
        }
        let inputs = crate::workflow::source_inputs(&temp.0);
        let output = temp.0.join("新建文件夹");
        fs::create_dir(&output).unwrap();
        let files = BTreeMap::from([(PathBuf::from("translations/data.json"), b"[]".to_vec())]);
        for forbidden in [&temp.0, &temp.0.join(crate::DISKS[0])] {
            assert!(Job::new(
                inputs.clone(),
                forbidden,
                true,
                files.clone(),
                vn_cli::RunReport::default()
            )
            .is_err());
        }
        let job = Job::new(
            inputs.clone(),
            &output,
            false,
            files,
            vn_cli::RunReport::default(),
        )
        .unwrap();
        let report = job.execute(&mut Quiet).unwrap();
        assert!(report.warnings.is_empty());
        assert_eq!(read(&output.join("translations/data.json")).unwrap(), b"[]");
        for path in inputs {
            assert_eq!(read(&path).unwrap(), b"original");
        }
    }

    #[test]
    fn selected_empty_output_must_remain_empty_until_commit() {
        let temp = Stage::create(&std::env::temp_dir(), "test").unwrap();
        let output = temp.0.join("selected");
        fs::create_dir(&output).unwrap();
        let files = BTreeMap::from([(PathBuf::from("result"), b"new".to_vec())]);
        let job = Job::new(vec![], &output, false, files, vn_cli::RunReport::default()).unwrap();
        fs::write(output.join("keep"), b"keep").unwrap();
        assert!(job.execute(&mut Quiet).is_err());
        assert_eq!(read(&output.join("keep")).unwrap(), b"keep");
        assert!(!output.join("result").exists());
    }
}
