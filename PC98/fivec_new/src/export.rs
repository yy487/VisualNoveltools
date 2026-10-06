use crate::{read_spans, sha256, Archive, FileSpan, Inspection, Result, Volume};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

const MANIFEST: &str = ".fivec_manifest.json";
const SCHEMA: &str = "fivec-new-export-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFile {
    pub path: String,
    pub volume_id: String,
    pub entry_id: String,
    pub size: usize,
    pub sha256: String,
    pub source_ranges: Vec<FileSpan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportManifest {
    pub schema: String,
    pub inspection: Inspection,
    pub selected_volumes: Vec<String>,
    pub directories: Vec<String>,
    pub files: Vec<ExportFile>,
}

#[derive(Debug)]
pub struct ExportReport {
    pub output: PathBuf,
    pub files: usize,
    pub bytes: usize,
    pub warnings: Vec<String>,
}

pub struct PreparedExport {
    source: Arc<[u8]>,
    source_path: Option<PathBuf>,
    output: PathBuf,
    original: Option<BTreeMap<String, Option<String>>>,
    manifest: ExportManifest,
}

impl Archive {
    /// `all` means every discovered volume; unsupported volumes cause an error.
    pub fn selected_volumes(&self, selection: &str) -> Result<Vec<&Volume>> {
        let volumes: Vec<_> = if selection == "all" {
            self.volumes().iter().collect()
        } else {
            vec![self
                .volumes()
                .iter()
                .find(|v| v.id == selection)
                .ok_or_else(|| format!("unknown volume: {selection}"))?]
        };
        if volumes.is_empty() {
            return Err("no volumes were discovered".into());
        }
        for volume in &volumes {
            if volume.filesystem.is_none() {
                return Err(format!(
                    "volume {} has no supported filesystem: {}",
                    volume.id,
                    volume.diagnostics.join("; ")
                ));
            }
        }
        Ok(volumes)
    }

    /// Prepare a snapshot-based transactional export into a new, separate folder.
    /// Its parent must exist. Overwrite accepts only unchanged tool-owned trees.
    pub fn prepare_export(
        &self,
        selection: &str,
        output: impl AsRef<Path>,
        overwrite: bool,
    ) -> Result<PreparedExport> {
        let selected = self.selected_volumes(selection)?;
        let absolute = std::path::absolute(output.as_ref()).map_err(|e| e.to_string())?;
        let parent = fs::canonicalize(absolute.parent().ok_or("output needs a parent directory")?)
            .map_err(|e| e.to_string())?;
        let name = absolute
            .file_name()
            .ok_or("output needs a directory name")?;
        let output = parent.join(name);
        protect_source(self.source_path.as_deref(), &output)?;
        let original = if exists(&output)? {
            if !overwrite {
                return Err("output already exists; overwrite is disabled".into());
            }
            Some(owned_tree(&output)?)
        } else {
            None
        };
        let mut directories = BTreeSet::new();
        let mut files = Vec::new();
        let mut used = BTreeSet::new();
        let selected_volumes = selected.iter().map(|v| v.id.clone()).collect();
        for volume in selected {
            directories.insert(volume.id.clone());
            let filesystem = volume.filesystem.as_ref().unwrap();
            for directory in &filesystem.directories {
                let path = format!("{}/{}", volume.id, directory.export_path);
                safe_relative(&path)?;
                directories.insert(path);
            }
            for file in &filesystem.files {
                let path = format!("{}/{}", volume.id, file.export_path);
                safe_relative(&path)?;
                if !used.insert(path.to_lowercase()) {
                    return Err("duplicate export path".into());
                }
                files.push(ExportFile {
                    path,
                    volume_id: volume.id.clone(),
                    entry_id: file.id.clone(),
                    size: file.size,
                    sha256: file.sha256.clone(),
                    source_ranges: file.source_ranges.clone(),
                });
            }
        }
        for directory in &directories {
            if !used.insert(directory.to_lowercase()) {
                return Err("file/directory export path collision".into());
            }
        }
        Ok(PreparedExport {
            source: self.bytes.clone(),
            source_path: self.source_path.clone(),
            output,
            original,
            manifest: ExportManifest {
                schema: SCHEMA.into(),
                inspection: self.inspection.clone(),
                selected_volumes,
                directories: directories.into_iter().collect(),
                files,
            },
        })
    }
}

impl PreparedExport {
    pub fn output(&self) -> &Path {
        &self.output
    }
    pub fn manifest(&self) -> &ExportManifest {
        &self.manifest
    }

    pub fn execute(self) -> Result<ExportReport> {
        protect_source(self.source_path.as_deref(), &self.output)?;
        if let Some(source) = &self.source_path {
            let current = fs::read(source).map_err(|e| e.to_string())?;
            if sha256(&current) != self.manifest.inspection.source_sha256 {
                return Err("source image changed after preflight".into());
            }
        }
        let parent = self.output.parent().ok_or("output has no parent")?;
        let stage = TemporaryDirectory::create(parent, "stage")?;
        let payload = stage.path.join("payload");
        fs::create_dir(&payload).map_err(|e| e.to_string())?;
        for directory in &self.manifest.directories {
            fs::create_dir_all(payload.join(safe_relative(directory)?))
                .map_err(|e| e.to_string())?;
        }
        let mut bytes = 0;
        for file in &self.manifest.files {
            let data = read_spans(&self.source, &file.source_ranges, file.size)?;
            if sha256(&data) != file.sha256 {
                return Err(format!("source mapping hash mismatch: {}", file.path));
            }
            write_new(&payload.join(safe_relative(&file.path)?), &data)?;
            bytes += data.len();
        }
        let manifest = serde_json::to_vec_pretty(&self.manifest).map_err(|e| e.to_string())?;
        write_new(&payload.join(MANIFEST), &manifest)?;
        // Verify actual staged bytes, not just the in-memory mapping.
        owned_tree(&payload)?;
        protect_source(self.source_path.as_deref(), &self.output)?;
        let current = if exists(&self.output)? {
            Some(owned_tree(&self.output)?)
        } else {
            None
        };
        if current != self.original {
            return Err("output changed after preflight; prepare again".into());
        }
        let backup = commit_directory(&payload, &self.output, self.original.as_ref())?;
        let mut warnings: Vec<_> = self
            .manifest
            .inspection
            .volumes
            .iter()
            .filter(|v| self.manifest.selected_volumes.contains(&v.id))
            .flat_map(|v| {
                v.filesystem
                    .iter()
                    .flat_map(|f| f.diagnostics.iter().map(|d| format!("{}: {d}", v.id)))
            })
            .collect();
        if let Some(path) = backup {
            warnings.push(format!(
                "previous output retained for recovery: {}",
                path.join("previous").display()
            ));
        }
        Ok(ExportReport {
            output: self.output,
            files: self.manifest.files.len(),
            bytes,
            warnings,
        })
    }
}

fn commit_directory(
    payload: &Path,
    output: &Path,
    original: Option<&BTreeMap<String, Option<String>>>,
) -> Result<Option<PathBuf>> {
    let parent = output.parent().ok_or("output has no parent")?;
    let backup = if original.is_some() {
        Some(TemporaryDirectory::create(parent, "backup")?)
    } else {
        None
    };
    let previous = backup.as_ref().map(|b| b.path.join("previous"));
    if let Some(previous) = &previous {
        fs::rename(output, previous).map_err(|e| e.to_string())?;
        let isolated = owned_tree(previous);
        if isolated.as_ref().ok() != original {
            if let Err(rollback) = fs::rename(previous, output) {
                let saved = backup.unwrap().preserve();
                return Err(format!("output changed during commit; rollback failed: {rollback}; previous output preserved at {}", saved.display()));
            }
            return Err("output changed during commit; original output restored".into());
        }
    }
    if let Err(error) = fs::rename(payload, output) {
        if let Some(previous) = &previous {
            if let Err(rollback) = fs::rename(previous, output) {
                let saved = backup.unwrap().preserve();
                return Err(format!("commit failed: {error}; rollback failed: {rollback}; previous output preserved at {}", saved.display()));
            }
        }
        return Err(format!("commit failed: {error}"));
    }
    Ok(backup.map(TemporaryDirectory::preserve))
}

fn safe_relative(path: &str) -> Result<PathBuf> {
    if path.is_empty() || path.contains('\\') {
        return Err("invalid relative export path".into());
    }
    for part in path.split('/') {
        if part.is_empty()
            || part == "."
            || part == ".."
            || part.ends_with(['.', ' '])
            || part
                .chars()
                .any(|c| c.is_control() || "<>:\"|?*".contains(c))
        {
            return Err(format!("unsafe export path: {path}"));
        }
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || ["COM", "LPT"].iter().any(|p| {
                stem.strip_prefix(p).is_some_and(|n| {
                    matches!(
                        n,
                        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                    )
                })
            })
        {
            return Err(format!("reserved host filename: {path}"));
        }
    }
    let path = PathBuf::from(path);
    if path
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("export path is not relative".into());
    }
    Ok(path)
}

fn protect_source(source: Option<&Path>, output: &Path) -> Result<()> {
    if let Some(source) = source {
        let resolved = if exists(output)? {
            fs::canonicalize(output).map_err(|e| e.to_string())?
        } else {
            output.to_path_buf()
        };
        #[cfg(windows)]
        let contained = {
            let source: Vec<_> = source
                .components()
                .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
                .collect();
            let output: Vec<_> = resolved
                .components()
                .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
                .collect();
            source.starts_with(&output)
        };
        #[cfg(not(windows))]
        let contained = source.starts_with(&resolved);
        if contained {
            return Err("output contains or equals the source image".into());
        }
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

fn reject_link(metadata: &fs::Metadata) -> Result<()> {
    if metadata.file_type().is_symlink() {
        return Err("links are not permitted in export trees".into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err("reparse points are not permitted in export trees".into());
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct Ownership {
    schema: String,
    directories: Vec<String>,
    files: Vec<ExportFile>,
}

fn owned_tree(root: &Path) -> Result<BTreeMap<String, Option<String>>> {
    let metadata = fs::symlink_metadata(root).map_err(|e| e.to_string())?;
    reject_link(&metadata)?;
    if !metadata.is_dir() {
        return Err("output is not a tool-owned directory".into());
    }
    let mut actual = BTreeMap::new();
    let mut pending = vec![PathBuf::new()];
    while let Some(relative) = pending.pop() {
        for entry in fs::read_dir(root.join(&relative)).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = relative.join(entry.file_name());
            let metadata = fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
            reject_link(&metadata)?;
            let key = path
                .to_str()
                .ok_or("non-Unicode file in output tree")?
                .replace('\\', "/");
            if metadata.is_dir() {
                actual.insert(key, None);
                pending.push(path);
            } else if metadata.is_file() {
                actual.insert(
                    key,
                    Some(sha256(&fs::read(entry.path()).map_err(|e| e.to_string())?)),
                );
            } else {
                return Err("special file in output tree".into());
            }
        }
    }
    let data = fs::read(root.join(MANIFEST))
        .map_err(|_| "existing output has no fivec_new ownership manifest")?;
    let owned: Ownership =
        serde_json::from_slice(&data).map_err(|e| format!("invalid ownership manifest: {e}"))?;
    if owned.schema != SCHEMA {
        return Err("unrecognized output ownership schema".into());
    }
    let mut expected = BTreeMap::from([(MANIFEST.to_string(), Some(sha256(&data)))]);
    for dir in owned.directories {
        safe_relative(&dir)?;
        if expected.insert(dir, None).is_some() {
            return Err("duplicate owned path".into());
        }
    }
    for file in owned.files {
        safe_relative(&file.path)?;
        if expected.insert(file.path, Some(file.sha256)).is_some() {
            return Err("duplicate owned path".into());
        }
    }
    if actual != expected {
        return Err(
            "output contains modified, missing or unrelated entries; overwrite refused".into(),
        );
    }
    Ok(actual)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())
}

struct TemporaryDirectory {
    path: PathBuf,
}
impl TemporaryDirectory {
    fn create(parent: &Path, role: &str) -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let path = parent.join(format!(
                ".fivec-{role}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.to_string()),
            }
        }
        Err("could not allocate staging directory".into())
    }
    fn preserve(self) -> PathBuf {
        let path = self.path.clone();
        std::mem::forget(self);
        path
    }
}
impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_directory_commit_restores_previous_tree() {
        let temp = TemporaryDirectory::create(&std::env::temp_dir(), "rollback-test").unwrap();
        let output = temp.path.join("out");
        fs::create_dir(&output).unwrap();
        write_new(
            &output.join(MANIFEST),
            br#"{"schema":"fivec-new-export-v1","directories":[],"files":[]}"#,
        )
        .unwrap();
        let original = owned_tree(&output).unwrap();
        // A missing prepared payload simulates a filesystem commit failure after
        // the previous tree has already been isolated into its recovery folder.
        let error = commit_directory(&temp.path.join("missing-payload"), &output, Some(&original))
            .unwrap_err();
        assert!(error.contains("commit failed"));
        assert_eq!(owned_tree(&output).unwrap(), original);
        assert_eq!(fs::read_dir(&temp.path).unwrap().count(), 1);
    }

    #[test]
    fn change_detected_after_isolation_is_restored_without_deletion() {
        let temp = TemporaryDirectory::create(&std::env::temp_dir(), "isolation-test").unwrap();
        let output = temp.path.join("out");
        fs::create_dir(&output).unwrap();
        write_new(
            &output.join(MANIFEST),
            br#"{"schema":"fivec-new-export-v1","directories":[],"files":[]}"#,
        )
        .unwrap();
        let original = owned_tree(&output).unwrap();
        write_new(&output.join("concurrent.txt"), b"preserve this").unwrap();
        let error = commit_directory(&temp.path.join("missing-payload"), &output, Some(&original))
            .unwrap_err();
        assert!(error.contains("original output restored"));
        assert_eq!(
            fs::read(output.join("concurrent.txt")).unwrap(),
            b"preserve this"
        );
    }
}
