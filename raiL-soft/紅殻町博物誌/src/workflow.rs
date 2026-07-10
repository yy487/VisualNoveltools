use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::gsc::{GscError, GscFile, TextEntry};
use crate::xfl::{PackStats, XflArchive, XflError, pack_directory};

pub const GSC_MANIFEST_NAME: &str = ".gsc-manifest.json";

const GSC_MANIFEST_FORMAT: &str = "railsoft-gsc-json-v1";

#[derive(Debug)]
pub enum WorkflowError {
    Message(String),
    Io {
        context: String,
        source: io::Error,
    },
    Json {
        context: String,
        source: serde_json::Error,
    },
    Xfl(XflError),
    Gsc {
        file: String,
        source: GscError,
    },
}

impl WorkflowError {
    fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }

    fn io(context: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    fn json(context: impl Into<String>, source: serde_json::Error) -> Self {
        Self::Json {
            context: context.into(),
            source,
        }
    }

    fn gsc(file: impl Into<String>, source: GscError) -> Self {
        Self::Gsc {
            file: file.into(),
            source,
        }
    }
}

impl fmt::Display for WorkflowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => formatter.write_str(message),
            Self::Io { context, source } => write!(formatter, "{context}: {source}"),
            Self::Json { context, source } => write!(formatter, "{context}: {source}"),
            Self::Xfl(source) => source.fmt(formatter),
            Self::Gsc { file, source } => write!(formatter, "{file}: {source}"),
        }
    }
}

impl Error for WorkflowError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Message(_) => None,
            Self::Io { source, .. } => Some(source),
            Self::Json { source, .. } => Some(source),
            Self::Xfl(source) => Some(source),
            Self::Gsc { source, .. } => Some(source),
        }
    }
}

impl From<XflError> for WorkflowError {
    fn from(source: XflError) -> Self {
        Self::Xfl(source)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XflUnpackResult {
    pub output: PathBuf,
    pub files: usize,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XflPackResult {
    pub output: PathBuf,
    pub stats: PackStats,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GscExtractResult {
    pub output: PathBuf,
    pub files: usize,
    pub entries: usize,
    pub skipped_text_records: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GscInjectResult {
    pub output: PathBuf,
    pub files: usize,
    pub entries: usize,
    pub edited_entries: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct GscManifest {
    format: String,
    source_dir: String,
    files: Vec<GscManifestFile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GscManifestFile {
    source: String,
    json: String,
    entries: usize,
}

#[derive(Debug)]
struct PreparedJson {
    relative_path: PathBuf,
    data: Vec<u8>,
}

#[derive(Debug)]
struct PreparedInjection {
    files: Vec<(PathBuf, Vec<u8>)>,
    entries: usize,
    edited_entries: usize,
}

/// Unpack an XFL beside the input as `<stem>_unpacked`.
///
/// # Errors
///
/// Returns an error if the input is not a file, the output already exists, or
/// parsing/extraction fails.
pub fn unpack_xfl_default(input: &Path) -> Result<XflUnpackResult, WorkflowError> {
    if !input.is_file() {
        return Err(WorkflowError::message(format!(
            "XFL input is not a file: {}",
            input.display()
        )));
    }
    let stem = input
        .file_stem()
        .ok_or_else(|| WorkflowError::message("XFL input has no filename stem"))?;
    let mut output_name = stem.to_os_string();
    output_name.push("_unpacked");
    let output = input.with_file_name(output_name);
    if output.exists() {
        return Err(WorkflowError::message(format!(
            "output already exists: {}",
            output.display()
        )));
    }

    let data = read_file(input)?;
    let archive = XflArchive::parse(&data)?;
    let stats = archive.extract_to(&output, false)?;
    Ok(XflUnpackResult {
        output,
        files: stats.extracted_files,
        bytes: stats.extracted_bytes,
    })
}

/// Pack a directory beside itself using a non-overwriting `_rebuilt.xfl` name.
///
/// # Errors
///
/// Returns an error if directory parsing fails or the output already exists.
pub fn pack_xfl_default(input: &Path) -> Result<XflPackResult, WorkflowError> {
    let output = default_pack_output(input)?;
    if output.exists() {
        return Err(WorkflowError::message(format!(
            "output already exists: {}",
            output.display()
        )));
    }
    let (archive, stats) = pack_directory(input)?;
    fs::write(&output, archive).map_err(|error| {
        WorkflowError::io(format!("failed to write {}", output.display()), error)
    })?;
    Ok(XflPackResult { output, stats })
}

/// Extract all GSC text tables below a directory to a sibling UTF-8 JSON tree.
///
/// # Errors
///
/// Returns an error before creating output when any GSC or JSON serialization
/// fails, or when the default output directory already exists.
pub fn extract_gsc_default(input: &Path) -> Result<GscExtractResult, WorkflowError> {
    if !input.is_dir() {
        return Err(WorkflowError::message(format!(
            "GSC input is not a directory: {}",
            input.display()
        )));
    }
    let directory_name = input
        .file_name()
        .ok_or_else(|| WorkflowError::message("GSC input directory has no name"))?;
    let mut output_name = directory_name.to_os_string();
    output_name.push("_json");
    let output = input.with_file_name(output_name);
    if output.exists() {
        return Err(WorkflowError::message(format!(
            "output already exists: {}",
            output.display()
        )));
    }

    let gsc_files = collect_files(input, Some("gsc"))?;
    if gsc_files.is_empty() {
        return Err(WorkflowError::message(format!(
            "input directory contains no .gsc files: {}",
            input.display()
        )));
    }

    let mut prepared = Vec::with_capacity(gsc_files.len());
    let mut manifest_files = Vec::with_capacity(gsc_files.len());
    let mut total_entries = 0_usize;
    let mut skipped_text_records = 0_usize;
    for relative in &gsc_files {
        let source_path = input.join(relative);
        let source_name = path_to_slashes(relative)?;
        let data = read_file(&source_path)?;
        let gsc = GscFile::parse(&data).map_err(|error| WorkflowError::gsc(&source_name, error))?;
        let entries = gsc
            .extract_entries(&source_name)
            .map_err(|error| WorkflowError::gsc(&source_name, error))?;
        skipped_text_records += gsc.text_count().saturating_sub(entries.len());
        total_entries += entries.len();

        let json_relative = append_extension(relative, ".json");
        let json_name = path_to_slashes(&json_relative)?;
        let mut json_data = serde_json::to_vec_pretty(&entries).map_err(|error| {
            WorkflowError::json(format!("failed to serialize {json_name}"), error)
        })?;
        json_data.push(b'\n');
        prepared.push(PreparedJson {
            relative_path: json_relative,
            data: json_data,
        });
        manifest_files.push(GscManifestFile {
            source: source_name,
            json: json_name,
            entries: entries.len(),
        });
    }

    let source_dir = relative_source_for_default_output(input)?;
    let manifest = GscManifest {
        format: GSC_MANIFEST_FORMAT.to_owned(),
        source_dir,
        files: manifest_files,
    };
    let mut manifest_data = serde_json::to_vec_pretty(&manifest).map_err(|error| {
        WorkflowError::json("failed to serialize GSC extraction manifest", error)
    })?;
    manifest_data.push(b'\n');

    fs::create_dir(&output).map_err(|error| {
        WorkflowError::io(
            format!("failed to create output directory {}", output.display()),
            error,
        )
    })?;
    for item in prepared {
        let destination = output.join(&item.relative_path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                WorkflowError::io(
                    format!("failed to create directory {}", parent.display()),
                    error,
                )
            })?;
        }
        fs::write(&destination, item.data).map_err(|error| {
            WorkflowError::io(format!("failed to write {}", destination.display()), error)
        })?;
    }
    let manifest_path = output.join(GSC_MANIFEST_NAME);
    fs::write(&manifest_path, manifest_data).map_err(|error| {
        WorkflowError::io(
            format!("failed to write {}", manifest_path.display()),
            error,
        )
    })?;

    Ok(GscExtractResult {
        output,
        files: gsc_files.len(),
        entries: total_entries,
        skipped_text_records,
    })
}

/// Rebuild the source GSC directory identified by an extraction manifest.
///
/// The translated JSON directory is validated completely before the sibling
/// rebuilt directory is created. Non-GSC files are copied unchanged.
///
/// # Errors
///
/// Returns an error for a missing/invalid manifest, stale JSON locator fields,
/// unencodable messages, missing source files, or an existing output directory.
pub fn inject_gsc_default(json_dir: &Path) -> Result<GscInjectResult, WorkflowError> {
    if !json_dir.is_dir() {
        return Err(WorkflowError::message(format!(
            "translation input is not a directory: {}",
            json_dir.display()
        )));
    }
    let manifest_path = json_dir.join(GSC_MANIFEST_NAME);
    let manifest_data = read_file(&manifest_path)?;
    let manifest: GscManifest = parse_json(&manifest_data, &manifest_path)?;
    if manifest.format != GSC_MANIFEST_FORMAT {
        return Err(WorkflowError::message(format!(
            "unsupported GSC manifest format {:?}",
            manifest.format
        )));
    }

    let source_candidate = resolve_manifest_source(json_dir, &manifest.source_dir)?;
    if !source_candidate.is_dir() {
        return Err(WorkflowError::message(format!(
            "manifest source directory does not exist: {}",
            source_candidate.display()
        )));
    }
    let source_dir = normalize_path(&source_candidate)?;
    let output = default_rebuilt_directory(&source_dir)?;
    if output.exists() {
        return Err(WorkflowError::message(format!(
            "output already exists: {}",
            output.display()
        )));
    }

    let prepared = prepare_injection(json_dir, &source_dir, &manifest)?;
    write_prepared_files(&output, prepared.files)?;

    Ok(GscInjectResult {
        output,
        files: manifest.files.len(),
        entries: prepared.entries,
        edited_entries: prepared.edited_entries,
    })
}

fn prepare_injection(
    json_dir: &Path,
    source_dir: &Path,
    manifest: &GscManifest,
) -> Result<PreparedInjection, WorkflowError> {
    let mut rebuilt_files = HashMap::with_capacity(manifest.files.len());
    let mut seen_sources = HashSet::with_capacity(manifest.files.len());
    let mut total_entries = 0_usize;
    let mut edited_entries = 0_usize;
    for item in &manifest.files {
        let source_relative = safe_relative_path(&item.source)?;
        let source_key = path_key(&source_relative)?;
        if !seen_sources.insert(source_key.clone()) {
            return Err(WorkflowError::message(format!(
                "duplicate source path in GSC manifest: {}",
                item.source
            )));
        }
        let json_path = json_dir.join(safe_relative_path(&item.json)?);
        let source_data = read_file(&source_dir.join(&source_relative))?;
        let gsc = GscFile::parse(&source_data)
            .map_err(|error| WorkflowError::gsc(&item.source, error))?;
        let json_data = read_file(&json_path)?;
        let entries: Vec<TextEntry> = parse_json(&json_data, &json_path)?;
        if entries.len() != item.entries {
            return Err(WorkflowError::message(format!(
                "{} contains {} entries, manifest expects {}",
                json_path.display(),
                entries.len(),
                item.entries
            )));
        }
        total_entries += entries.len();
        edited_entries += entries
            .iter()
            .filter(|entry| entry.message != entry.scr_msg)
            .count();
        let rebuilt = gsc
            .rebuild_from_entries(&item.source, &entries)
            .map_err(|error| WorkflowError::gsc(&item.source, error))?;
        rebuilt_files.insert(source_key, rebuilt);
    }

    let mut files = Vec::new();
    for relative in collect_files(source_dir, None)? {
        let key = path_key(&relative)?;
        let data = rebuilt_files
            .remove(&key)
            .map_or_else(|| read_file(&source_dir.join(&relative)), Ok)?;
        files.push((relative, data));
    }
    if !rebuilt_files.is_empty() {
        return Err(WorkflowError::message(
            "one or more manifest GSC files were not present in the source directory",
        ));
    }
    Ok(PreparedInjection {
        files,
        entries: total_entries,
        edited_entries,
    })
}

fn write_prepared_files(
    output: &Path,
    prepared: Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), WorkflowError> {
    fs::create_dir(output).map_err(|error| {
        WorkflowError::io(
            format!("failed to create output directory {}", output.display()),
            error,
        )
    })?;
    for (relative, data) in prepared {
        let destination = output.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                WorkflowError::io(
                    format!("failed to create directory {}", parent.display()),
                    error,
                )
            })?;
        }
        fs::write(&destination, data).map_err(|error| {
            WorkflowError::io(format!("failed to write {}", destination.display()), error)
        })?;
    }
    Ok(())
}

fn default_pack_output(input: &Path) -> Result<PathBuf, WorkflowError> {
    if !input.is_dir() {
        return Err(WorkflowError::message(format!(
            "XFL pack input is not a directory: {}",
            input.display()
        )));
    }
    let name = input
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| WorkflowError::message("pack directory name is not valid Unicode"))?;
    Ok(input.with_file_name(pack_output_name(name)))
}

fn pack_output_name(name: &str) -> String {
    if let Some(stem) = name.strip_suffix("_unpacked") {
        format!("{stem}_rebuilt.xfl")
    } else if name.ends_with("_rebuilt") {
        format!("{name}.xfl")
    } else {
        format!("{name}_rebuilt.xfl")
    }
}

fn default_rebuilt_directory(source: &Path) -> Result<PathBuf, WorkflowError> {
    let name = source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| WorkflowError::message("source directory name is not valid Unicode"))?;
    let output_name = name.strip_suffix("_unpacked").map_or_else(
        || format!("{name}_rebuilt"),
        |stem| format!("{stem}_rebuilt"),
    );
    Ok(source.with_file_name(output_name))
}

fn relative_source_for_default_output(input: &Path) -> Result<String, WorkflowError> {
    let name = input
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| WorkflowError::message("source directory name is not valid Unicode"))?;
    Ok(format!("../{name}"))
}

fn resolve_manifest_source(json_dir: &Path, source: &str) -> Result<PathBuf, WorkflowError> {
    if source.is_empty() || source.chars().any(char::is_control) {
        return Err(WorkflowError::message(
            "manifest source directory is empty or contains control characters",
        ));
    }
    let source_path = Path::new(source);
    if source_path.is_absolute() {
        Ok(source_path.to_path_buf())
    } else {
        Ok(json_dir.join(source_path))
    }
}

fn normalize_path(path: &Path) -> Result<PathBuf, WorkflowError> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(WorkflowError::message(format!(
                        "path escapes its filesystem root: {}",
                        path.display()
                    )));
                }
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    Ok(normalized)
}

fn append_extension(path: &Path, suffix: &str) -> PathBuf {
    let mut value: OsString = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

fn collect_files(root: &Path, extension: Option<&str>) -> Result<Vec<PathBuf>, WorkflowError> {
    fn visit(
        root: &Path,
        current: &Path,
        extension: Option<&str>,
        output: &mut Vec<PathBuf>,
    ) -> Result<(), WorkflowError> {
        let read_dir = fs::read_dir(current).map_err(|error| {
            WorkflowError::io(
                format!("failed to read directory {}", current.display()),
                error,
            )
        })?;
        for item in read_dir {
            let item = item.map_err(|error| {
                WorkflowError::io(
                    format!("failed to enumerate directory {}", current.display()),
                    error,
                )
            })?;
            let path = item.path();
            let file_type = item.file_type().map_err(|error| {
                WorkflowError::io(format!("failed to inspect {}", path.display()), error)
            })?;
            if file_type.is_symlink() {
                return Err(WorkflowError::message(format!(
                    "symbolic links are not supported: {}",
                    path.display()
                )));
            }
            if file_type.is_dir() {
                visit(root, &path, extension, output)?;
            } else if file_type.is_file()
                && extension.is_none_or(|expected| {
                    path.extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
                })
            {
                let relative = path.strip_prefix(root).map_err(|_| {
                    WorkflowError::message(format!(
                        "enumerated path escaped root: {}",
                        path.display()
                    ))
                })?;
                output.push(relative.to_path_buf());
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    visit(root, root, extension, &mut files)?;
    files
        .sort_by_key(|path| path_key(path).unwrap_or_else(|_| path.to_string_lossy().into_owned()));
    Ok(files)
}

fn safe_relative_path(value: &str) -> Result<PathBuf, WorkflowError> {
    let path = PathBuf::from(value.replace('/', "\\"));
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(WorkflowError::message(format!(
            "unsafe relative path in manifest: {value:?}"
        )));
    }
    Ok(path)
}

fn path_to_slashes(path: &Path) -> Result<String, WorkflowError> {
    path.to_str()
        .map(|value| value.replace('\\', "/"))
        .ok_or_else(|| {
            WorkflowError::message(format!("path is not valid Unicode: {}", path.display()))
        })
}

fn path_key(path: &Path) -> Result<String, WorkflowError> {
    Ok(path_to_slashes(path)?.to_lowercase())
}

fn parse_json<T: for<'de> Deserialize<'de>>(data: &[u8], path: &Path) -> Result<T, WorkflowError> {
    let data = data.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(data);
    serde_json::from_slice(data)
        .map_err(|error| WorkflowError::json(format!("failed to parse {}", path.display()), error))
}

fn read_file(path: &Path) -> Result<Vec<u8>, WorkflowError> {
    fs::read(path)
        .map_err(|error| WorkflowError::io(format!("failed to read {}", path.display()), error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_names_form_a_drag_and_drop_pipeline() {
        let root = Path::new("C:\\game");
        let unpacked = root.join("scr_unpacked");
        let rebuilt = default_rebuilt_directory(&unpacked).expect("name should be valid");
        assert_eq!(rebuilt, root.join("scr_rebuilt"));
        assert_eq!(pack_output_name("scr_unpacked"), "scr_rebuilt.xfl");
        assert_eq!(pack_output_name("scr_rebuilt"), "scr_rebuilt.xfl");
    }

    #[test]
    fn manifest_paths_reject_parent_traversal() {
        assert!(safe_relative_path("../escape.gsc").is_err());
        assert!(safe_relative_path("dir/file.gsc").is_ok());
    }

    #[test]
    fn normalizes_manifest_source_without_verbatim_prefix() {
        let path = Path::new("E:\\work\\json\\..\\source");
        assert_eq!(
            normalize_path(path).expect("path should normalize"),
            PathBuf::from("E:\\work\\source")
        );
    }
}
