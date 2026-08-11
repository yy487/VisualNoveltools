use crate::gsc::{GscFile, TextEntry, looks_like_gsc};
use crate::speaker::SpeakerMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const MANIFEST_NAME: &str = ".sbridge-gsc-manifest.json";
const MANIFEST_FORMAT: &str = "sbridge-gsc-json-v1";

#[derive(Debug)]
pub struct WorkflowError(String);

impl WorkflowError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for WorkflowError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractResult {
    pub files: usize,
    pub entries: usize,
    pub opaque_tail_files: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectResult {
    pub files: usize,
    pub entries: usize,
    pub edited_entries: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    format: String,
    source: SourceRole,
    speaker_map: Option<String>,
    files: Vec<ManifestFile>,
    injection_policy: InjectionPolicy,
}

#[derive(Debug, Serialize, Deserialize)]
struct SourceRole {
    input: String,
    root: String,
    kind: String,
    read_only: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestFile {
    source: String,
    json: String,
    entries: usize,
    declared_size: u32,
    physical_size: usize,
    opaque_tail_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct InjectionPolicy {
    writable_fields: Vec<String>,
    name_context_only: bool,
    preserve_opaque_tail: bool,
}

struct PreparedFile {
    relative: PathBuf,
    bytes: Vec<u8>,
}

pub fn extract(
    input: &Path,
    output: &Path,
    speaker_map: Option<&SpeakerMap>,
    speaker_map_path: Option<&Path>,
) -> Result<ExtractResult, WorkflowError> {
    if output.exists() {
        return Err(WorkflowError::new(format!(
            "output already exists: {}",
            output.display()
        )));
    }
    let (source_root, input_kind, candidates) = source_candidates(input)?;
    let mut prepared = Vec::new();
    let mut manifest_files = Vec::new();
    let mut total_entries = 0usize;
    let mut opaque_tail_files = 0usize;

    for relative in candidates {
        let source_path = source_root.join(&relative);
        let data = read(&source_path)?;
        if !looks_like_gsc(&data) {
            if input_kind == "file" {
                return Err(WorkflowError::new(format!(
                    "{} is not a supported GSC file",
                    source_path.display()
                )));
            }
            continue;
        }
        let source_name = path_to_slashes(&relative)?;
        let gsc = GscFile::parse(&data)
            .map_err(|error| WorkflowError::new(format!("{source_name}: {error}")))?;
        let entries = gsc
            .extract_entries(&source_name, speaker_map)
            .map_err(|error| WorkflowError::new(format!("{source_name}: {error}")))?;
        let json_relative = append_extension(&relative, ".json")?;
        let json_name = path_to_slashes(&json_relative)?;
        let mut bytes = serde_json::to_vec_pretty(&entries).map_err(|error| {
            WorkflowError::new(format!("failed to serialize {json_name}: {error}"))
        })?;
        bytes.push(b'\n');
        total_entries += entries.len();
        if gsc.opaque_tail_size() != 0 {
            opaque_tail_files += 1;
        }
        manifest_files.push(ManifestFile {
            source: source_name,
            json: json_name,
            entries: entries.len(),
            declared_size: gsc.header.file_size,
            physical_size: gsc.physical_size(),
            opaque_tail_size: gsc.opaque_tail_size(),
        });
        prepared.push(PreparedFile {
            relative: json_relative,
            bytes,
        });
    }
    if manifest_files.is_empty() {
        return Err(WorkflowError::new(format!(
            "no supported GSC files found at {}",
            input.display()
        )));
    }

    manifest_files.sort_by(|a, b| a.source.cmp(&b.source));
    prepared.sort_by(|a, b| a.relative.cmp(&b.relative));
    let manifest = Manifest {
        format: MANIFEST_FORMAT.to_owned(),
        source: SourceRole {
            input: absolute_path(input)?.to_string_lossy().into_owned(),
            root: absolute_path(&source_root)?.to_string_lossy().into_owned(),
            kind: input_kind,
            read_only: true,
        },
        speaker_map: speaker_map_path.map(|path| path.to_string_lossy().into_owned()),
        files: manifest_files,
        injection_policy: InjectionPolicy {
            writable_fields: vec!["message".to_owned()],
            name_context_only: true,
            preserve_opaque_tail: true,
        },
    };
    let mut manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| WorkflowError::new(format!("failed to serialize manifest: {error}")))?;
    manifest_bytes.push(b'\n');

    fs::create_dir_all(output).map_err(|error| {
        WorkflowError::new(format!("failed to create {}: {error}", output.display()))
    })?;
    for item in prepared {
        write_new(&output.join(item.relative), &item.bytes)?;
    }
    write_new(&output.join(MANIFEST_NAME), &manifest_bytes)?;
    Ok(ExtractResult {
        files: manifest.files.len(),
        entries: total_entries,
        opaque_tail_files,
    })
}

pub fn inject(
    translation_dir: &Path,
    output: &Path,
    source_override: Option<&Path>,
) -> Result<InjectResult, WorkflowError> {
    if output.exists() {
        return Err(WorkflowError::new(format!(
            "output already exists: {}",
            output.display()
        )));
    }
    let manifest_path = translation_dir.join(MANIFEST_NAME);
    let manifest: Manifest = serde_json::from_slice(&read(&manifest_path)?).map_err(|error| {
        WorkflowError::new(format!(
            "failed to parse {}: {error}",
            manifest_path.display()
        ))
    })?;
    validate_manifest(&manifest)?;
    let source_root = source_override
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(&manifest.source.root));
    if !source_root.is_dir() {
        return Err(WorkflowError::new(format!(
            "source root is not a directory: {}",
            source_root.display()
        )));
    }

    let mut prepared_by_path = HashMap::new();
    let mut listed_sources = HashSet::new();
    let mut entries = 0usize;
    let mut edited_entries = 0usize;
    for item in &manifest.files {
        let source_relative = safe_relative(&item.source)?;
        let json_relative = safe_relative(&item.json)?;
        if !listed_sources.insert(source_relative.clone()) {
            return Err(WorkflowError::new(format!(
                "duplicate source in manifest: {}",
                item.source
            )));
        }
        let source_path = source_root.join(&source_relative);
        let source_data = read(&source_path)?;
        let gsc = GscFile::parse(&source_data)
            .map_err(|error| WorkflowError::new(format!("{}: {error}", item.source)))?;
        if gsc.header.file_size != item.declared_size
            || gsc.physical_size() != item.physical_size
            || gsc.opaque_tail_size() != item.opaque_tail_size
        {
            return Err(WorkflowError::new(format!(
                "{} no longer matches manifest size metadata",
                item.source
            )));
        }
        let json_path = translation_dir.join(json_relative);
        let translated: Vec<TextEntry> =
            serde_json::from_slice(&read(&json_path)?).map_err(|error| {
                WorkflowError::new(format!("failed to parse {}: {error}", json_path.display()))
            })?;
        if translated.len() != item.entries {
            return Err(WorkflowError::new(format!(
                "{} contains {} entries, manifest expects {}",
                json_path.display(),
                translated.len(),
                item.entries
            )));
        }
        edited_entries += translated
            .iter()
            .filter(|entry| entry.message != entry.scr_msg)
            .count();
        entries += translated.len();
        let rebuilt = gsc
            .rebuild_from_entries(&item.source, &translated)
            .map_err(|error| WorkflowError::new(format!("{}: {error}", item.source)))?;
        prepared_by_path.insert(source_relative, rebuilt);
    }

    let mut prepared = Vec::new();
    if manifest.source.kind == "file" {
        for (relative, bytes) in prepared_by_path {
            prepared.push(PreparedFile { relative, bytes });
        }
    } else {
        for relative in walk_files(&source_root)? {
            let bytes = if let Some(rebuilt) = prepared_by_path.remove(&relative) {
                rebuilt
            } else {
                let source = read(&source_root.join(&relative))?;
                if looks_like_gsc(&source) {
                    return Err(WorkflowError::new(format!(
                        "GSC source is missing from manifest: {}",
                        relative.display()
                    )));
                }
                source
            };
            prepared.push(PreparedFile { relative, bytes });
        }
        if let Some(missing) = prepared_by_path.keys().next() {
            return Err(WorkflowError::new(format!(
                "manifest source file is missing: {}",
                missing.display()
            )));
        }
    }
    prepared.sort_by(|a, b| a.relative.cmp(&b.relative));

    fs::create_dir_all(output).map_err(|error| {
        WorkflowError::new(format!("failed to create {}: {error}", output.display()))
    })?;
    for item in prepared {
        write_new(&output.join(item.relative), &item.bytes)?;
    }
    Ok(InjectResult {
        files: manifest.files.len(),
        entries,
        edited_entries,
    })
}

fn validate_manifest(manifest: &Manifest) -> Result<(), WorkflowError> {
    if manifest.format != MANIFEST_FORMAT {
        return Err(WorkflowError::new(format!(
            "unsupported manifest format {:?}",
            manifest.format
        )));
    }
    if !manifest.source.read_only
        || !manifest.injection_policy.name_context_only
        || !manifest.injection_policy.preserve_opaque_tail
        || manifest.injection_policy.writable_fields != ["message"]
    {
        return Err(WorkflowError::new(
            "manifest write policy does not match the supported safe profile",
        ));
    }
    if !matches!(manifest.source.kind.as_str(), "file" | "directory") {
        return Err(WorkflowError::new("manifest has an invalid source kind"));
    }
    Ok(())
}

fn source_candidates(input: &Path) -> Result<(PathBuf, String, Vec<PathBuf>), WorkflowError> {
    if input.is_file() {
        let root = input
            .parent()
            .ok_or_else(|| WorkflowError::new("input file has no parent directory"))?
            .to_owned();
        let name = input
            .file_name()
            .ok_or_else(|| WorkflowError::new("input file has no name"))?;
        Ok((root, "file".to_owned(), vec![PathBuf::from(name)]))
    } else if input.is_dir() {
        Ok((input.to_owned(), "directory".to_owned(), walk_files(input)?))
    } else {
        Err(WorkflowError::new(format!(
            "input does not exist: {}",
            input.display()
        )))
    }
}

fn walk_files(root: &Path) -> Result<Vec<PathBuf>, WorkflowError> {
    let mut files = Vec::new();
    walk_directory(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_directory(
    root: &Path,
    current: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), WorkflowError> {
    let mut entries: Vec<_> = fs::read_dir(current)
        .map_err(|error| {
            WorkflowError::new(format!("failed to read {}: {error}", current.display()))
        })?
        .collect::<Result<_, _>>()
        .map_err(|error| {
            WorkflowError::new(format!(
                "failed to enumerate {}: {error}",
                current.display()
            ))
        })?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let kind = entry.file_type().map_err(|error| {
            WorkflowError::new(format!(
                "failed to inspect {}: {error}",
                entry.path().display()
            ))
        })?;
        if kind.is_dir() {
            walk_directory(root, &entry.path(), files)?;
        } else if kind.is_file() {
            files.push(
                entry
                    .path()
                    .strip_prefix(root)
                    .map_err(|error| WorkflowError::new(error.to_string()))?
                    .to_owned(),
            );
        }
    }
    Ok(())
}

fn safe_relative(value: &str) -> Result<PathBuf, WorkflowError> {
    let path = PathBuf::from(value);
    if path.as_os_str().is_empty()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(WorkflowError::new(format!(
            "manifest path is not a safe relative path: {value:?}"
        )));
    }
    Ok(path)
}

fn path_to_slashes(path: &Path) -> Result<String, WorkflowError> {
    let value = path
        .to_str()
        .ok_or_else(|| WorkflowError::new(format!("path is not Unicode: {}", path.display())))?;
    Ok(value.replace('\\', "/"))
}

fn append_extension(path: &Path, suffix: &str) -> Result<PathBuf, WorkflowError> {
    let mut name: OsString = path
        .file_name()
        .ok_or_else(|| WorkflowError::new("source path has no file name"))?
        .to_os_string();
    name.push(suffix);
    Ok(path.with_file_name(name))
}

fn absolute_path(path: &Path) -> Result<PathBuf, WorkflowError> {
    if path.is_absolute() {
        Ok(path.to_owned())
    } else {
        std::env::current_dir()
            .map(|current| current.join(path))
            .map_err(|error| WorkflowError::new(format!("failed to resolve path: {error}")))
    }
}

fn read(path: &Path) -> Result<Vec<u8>, WorkflowError> {
    fs::read(path)
        .map_err(|error| WorkflowError::new(format!("failed to read {}: {error}", path.display())))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), WorkflowError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            WorkflowError::new(format!("failed to create {}: {error}", parent.display()))
        })?;
    }
    if path.exists() {
        return Err(WorkflowError::new(format!(
            "refusing to overwrite {}",
            path.display()
        )));
    }
    fs::write(path, bytes)
        .map_err(|error| WorkflowError::new(format!("failed to write {}: {error}", path.display())))
}
