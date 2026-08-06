use crate::extract::{extract_path_with_progress, ExtractReport};
use crate::glyph::GlyphDictionary;
use crate::text_json::{merge_translation_view_tree, write_translation_view_tree, TextEntry};
use crate::{parse_archive, unpack_archive, ToolError, ToolResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const WORKSPACE_SCHEMA: &str = "merry-mpk-translation-workspace-v1";
pub const INTERNAL_DIR: &str = ".mpk_tool";
pub const TRANSLATION_DIR: &str = "translation_json";
const SOURCE_DIR: &str = "source";
const SOURCE_JSON_DIR: &str = "source_json";
const WORKSPACE_MANIFEST: &str = "workspace.json";
const ARCHIVE_NAMES: [(&str, &str); 3] = [
    ("mes00.mpk", "mes00"),
    ("script.mpk", "script"),
    ("system_win.mpk", "system_win"),
];

#[derive(Debug, Clone)]
pub struct TranslationWorkspacePlan {
    pub input: PathBuf,
    pub output: PathBuf,
    pub package_directories: Vec<PathBuf>,
    pub archives: usize,
    pub archive_members: usize,
}

#[derive(Debug, Clone)]
pub struct TranslationWorkspaceReport {
    pub input: PathBuf,
    pub output: PathBuf,
    pub translation_json: PathBuf,
    pub package_directories: usize,
    pub archives: usize,
    pub extracted_members: usize,
    pub scanned_scripts: usize,
    pub json_files: usize,
    pub translation_entries: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum TranslationWorkspaceProgress {
    Unpacking {
        current: usize,
        total: usize,
        archive: PathBuf,
    },
    Extracting {
        current: usize,
        total: usize,
        script: PathBuf,
    },
    WritingTranslationView,
    Finalizing,
}

#[derive(Debug, Clone)]
pub struct TranslationWorkspacePaths {
    pub root: PathBuf,
    pub source_root: PathBuf,
    pub source_json_root: PathBuf,
    pub translation_root: PathBuf,
}

#[derive(Debug, Clone)]
struct ArchiveSet {
    directory: PathBuf,
    relative_directory: PathBuf,
    archives: Vec<ArchiveSource>,
}

#[derive(Debug, Clone)]
struct ArchiveSource {
    kind: String,
    path: PathBuf,
    members: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceManifest {
    schema: String,
    source_input: String,
    archives: Vec<WorkspaceArchive>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceArchive {
    source: String,
    unpacked: String,
    members: usize,
}

type WorkspaceOperation = (
    ExtractReport,
    usize,
    usize,
    Vec<String>,
    Vec<WorkspaceArchive>,
);

pub fn default_translation_workspace_output(input: &Path) -> ToolResult<PathBuf> {
    let name = input
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ToolError(format!(
                "cannot derive translation workspace name from '{}'",
                input.display()
            ))
        })?;
    Ok(input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{name}_translation")))
}

pub fn plan_translation_workspace(input: &Path) -> ToolResult<TranslationWorkspacePlan> {
    if !input.is_dir() {
        return Err(ToolError(format!(
            "automatic archive extraction input '{}' is not a directory",
            input.display()
        )));
    }
    let sets = find_archive_sets(input)?;
    if sets.is_empty() {
        return Err(ToolError(format!(
            "no directory containing mes00.mpk, script.mpk, and system_win.mpk was found under '{}'",
            input.display()
        )));
    }
    let archive_members = sets
        .iter()
        .flat_map(|set| &set.archives)
        .map(|archive| archive.members)
        .sum();
    Ok(TranslationWorkspacePlan {
        input: input.to_path_buf(),
        output: default_translation_workspace_output(input)?,
        package_directories: sets.iter().map(|set| set.directory.clone()).collect(),
        archives: sets.len() * ARCHIVE_NAMES.len(),
        archive_members,
    })
}

pub fn prepare_translation_workspace_with_progress(
    input: &Path,
    output: Option<&Path>,
    dictionary: &GlyphDictionary,
    progress: &mut dyn FnMut(TranslationWorkspaceProgress),
) -> ToolResult<TranslationWorkspaceReport> {
    let sets = find_archive_sets(input)?;
    if sets.is_empty() {
        return Err(ToolError(format!(
            "no directory containing mes00.mpk, script.mpk, and system_win.mpk was found under '{}'",
            input.display()
        )));
    }
    let output = output
        .map(Path::to_path_buf)
        .unwrap_or(default_translation_workspace_output(input)?);
    refuse_existing(&output)?;
    let output_parent = output.parent().unwrap_or_else(|| Path::new("."));
    if !output_parent.is_dir() {
        return Err(ToolError(format!(
            "translation workspace parent '{}' does not exist",
            output_parent.display()
        )));
    }
    let stage = temporary_sibling(&output)?;
    refuse_existing(&stage)?;
    let internal = stage.join(INTERNAL_DIR);
    let source_root = internal.join(SOURCE_DIR);
    let source_json_root = internal.join(SOURCE_JSON_DIR);
    let translation_root = stage.join(TRANSLATION_DIR);
    fs::create_dir_all(&source_root).map_err(|error| {
        ToolError(format!(
            "cannot create translation workspace staging directory '{}': {error}",
            source_root.display()
        ))
    })?;

    let operation = (|| -> ToolResult<WorkspaceOperation> {
        let total_archives = sets.len() * ARCHIVE_NAMES.len();
        let mut archive_index = 0;
        let mut extracted_members = 0;
        let mut warnings = Vec::new();
        let mut manifest_archives = Vec::new();
        for set in &sets {
            for archive in &set.archives {
                archive_index += 1;
                progress(TranslationWorkspaceProgress::Unpacking {
                    current: archive_index,
                    total: total_archives,
                    archive: archive.path.clone(),
                });
                let relative_output = set.relative_directory.join(&archive.kind);
                let unpacked = source_root.join(&relative_output);
                if let Some(parent) = unpacked.parent() {
                    fs::create_dir_all(parent).map_err(|error| {
                        ToolError(format!(
                            "cannot create archive output parent '{}': {error}",
                            parent.display()
                        ))
                    })?;
                }
                let report = unpack_archive(&archive.path, Some(&unpacked))?;
                extracted_members += report.extracted_files;
                warnings.extend(
                    report
                        .warning_messages
                        .into_iter()
                        .map(|warning| format!("{}: {warning}", archive.path.display())),
                );
                manifest_archives.push(WorkspaceArchive {
                    source: archive.path.display().to_string(),
                    unpacked: relative_output.display().to_string(),
                    members: archive.members,
                });
            }
        }

        fs::create_dir_all(&internal).map_err(|error| {
            ToolError(format!(
                "cannot create internal workspace directory '{}': {error}",
                internal.display()
            ))
        })?;
        let mut extract_progress = |current: usize, total: usize, path: &Path| {
            progress(TranslationWorkspaceProgress::Extracting {
                current,
                total,
                script: path.to_path_buf(),
            });
        };
        let extract = extract_path_with_progress(
            &source_root,
            Some(&source_json_root),
            dictionary,
            &mut extract_progress,
        )?;
        warnings.extend(extract.warnings.iter().cloned());
        progress(TranslationWorkspaceProgress::WritingTranslationView);
        let projected_entries =
            write_translation_view_tree(&source_json_root, &translation_root, dictionary)?;
        Ok((
            extract,
            extracted_members,
            projected_entries,
            warnings,
            manifest_archives,
        ))
    })();

    let (extract, extracted_members, translation_entries, warnings, manifest_archives) =
        match operation {
            Ok(value) => value,
            Err(error) => {
                let _ = fs::remove_dir_all(&stage);
                return Err(error);
            }
        };
    let manifest = WorkspaceManifest {
        schema: WORKSPACE_SCHEMA.to_string(),
        source_input: input.display().to_string(),
        archives: manifest_archives,
    };
    let mut manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    manifest_bytes.push(b'\n');
    fs::write(internal.join(WORKSPACE_MANIFEST), manifest_bytes).map_err(|error| {
        let _ = fs::remove_dir_all(&stage);
        ToolError(format!(
            "cannot write workspace manifest '{}': {error}",
            internal.join(WORKSPACE_MANIFEST).display()
        ))
    })?;
    progress(TranslationWorkspaceProgress::Finalizing);
    fs::rename(&stage, &output).map_err(|error| {
        let _ = fs::remove_dir_all(&stage);
        ToolError(format!(
            "cannot finalize translation workspace '{}': {error}",
            output.display()
        ))
    })?;
    Ok(TranslationWorkspaceReport {
        input: input.to_path_buf(),
        translation_json: output.join(TRANSLATION_DIR),
        output,
        package_directories: sets.len(),
        archives: sets.len() * ARCHIVE_NAMES.len(),
        extracted_members,
        scanned_scripts: extract.scanned_files,
        json_files: extract.json_files,
        translation_entries,
        warnings,
    })
}

pub fn resolve_translation_workspace(
    input: &Path,
) -> ToolResult<Option<TranslationWorkspacePaths>> {
    let start = if input.is_file() {
        input.parent().unwrap_or_else(|| Path::new("."))
    } else {
        input
    };
    let mut current = Some(start);
    while let Some(candidate) = current {
        let manifest_path = candidate.join(INTERNAL_DIR).join(WORKSPACE_MANIFEST);
        if manifest_path.is_file() {
            let bytes = fs::read(&manifest_path).map_err(|error| {
                ToolError(format!(
                    "cannot read translation workspace manifest '{}': {error}",
                    manifest_path.display()
                ))
            })?;
            let manifest: WorkspaceManifest = serde_json::from_slice(&bytes).map_err(|error| {
                ToolError(format!(
                    "cannot parse translation workspace manifest '{}': {error}",
                    manifest_path.display()
                ))
            })?;
            if manifest.schema != WORKSPACE_SCHEMA {
                return Err(ToolError(format!(
                    "translation workspace '{}' uses unsupported schema '{}'",
                    candidate.display(),
                    manifest.schema
                )));
            }
            let paths = TranslationWorkspacePaths {
                root: candidate.to_path_buf(),
                source_root: candidate.join(INTERNAL_DIR).join(SOURCE_DIR),
                source_json_root: candidate.join(INTERNAL_DIR).join(SOURCE_JSON_DIR),
                translation_root: candidate.join(TRANSLATION_DIR),
            };
            for required in [
                &paths.source_root,
                &paths.source_json_root,
                &paths.translation_root,
            ] {
                if !required.is_dir() {
                    return Err(ToolError(format!(
                        "translation workspace is missing directory '{}'",
                        required.display()
                    )));
                }
            }
            return Ok(Some(paths));
        }
        current = candidate.parent();
    }
    Ok(None)
}

pub fn load_workspace_translation_entries(
    paths: &TranslationWorkspacePaths,
    dictionary: &GlyphDictionary,
) -> ToolResult<Vec<TextEntry>> {
    merge_translation_view_tree(&paths.source_json_root, &paths.translation_root, dictionary)
}

fn find_archive_sets(root: &Path) -> ToolResult<Vec<ArchiveSet>> {
    let mut directories = Vec::new();
    find_archive_directories(root, &mut directories)?;
    directories.sort();
    let mut sets = Vec::with_capacity(directories.len());
    for directory in directories {
        let relative_directory = directory
            .strip_prefix(root)
            .map_err(|error| {
                ToolError(format!(
                    "cannot make package directory '{}' relative to '{}': {error}",
                    directory.display(),
                    root.display()
                ))
            })?
            .to_path_buf();
        let files = directory_files(&directory)?;
        let mut archives = Vec::with_capacity(ARCHIVE_NAMES.len());
        for (filename, kind) in ARCHIVE_NAMES {
            let path = files.get(filename).expect("validated archive set").clone();
            let members = parse_archive(&path)?.entries.len();
            archives.push(ArchiveSource {
                kind: kind.to_string(),
                path,
                members,
            });
        }
        sets.push(ArchiveSet {
            directory,
            relative_directory,
            archives,
        });
    }
    Ok(sets)
}

fn find_archive_directories(root: &Path, output: &mut Vec<PathBuf>) -> ToolResult<()> {
    let files = directory_files(root)?;
    if ARCHIVE_NAMES
        .iter()
        .all(|(filename, _)| files.contains_key(*filename))
    {
        output.push(root.to_path_buf());
        return Ok(());
    }
    let mut directories = fs::read_dir(root)
        .map_err(|error| ToolError(format!("cannot list '{}': {error}", root.display())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            ToolError(format!(
                "cannot read directory entry in '{}': {error}",
                root.display()
            ))
        })?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && !skip_directory(path))
        .collect::<Vec<_>>();
    directories.sort();
    for directory in directories {
        find_archive_directories(&directory, output)?;
    }
    Ok(())
}

fn directory_files(directory: &Path) -> ToolResult<HashMap<String, PathBuf>> {
    let mut files = HashMap::new();
    for entry in fs::read_dir(directory)
        .map_err(|error| ToolError(format!("cannot list '{}': {error}", directory.display())))?
    {
        let entry = entry.map_err(|error| {
            ToolError(format!(
                "cannot read directory entry in '{}': {error}",
                directory.display()
            ))
        })?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|value| value.to_str()) {
                files.insert(name.to_ascii_lowercase(), path);
            }
        }
    }
    Ok(files)
}

fn skip_directory(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| {
            name.eq_ignore_ascii_case("work")
                || name.eq_ignore_ascii_case("chs")
                || name.eq_ignore_ascii_case("target")
                || name.eq_ignore_ascii_case(INTERNAL_DIR)
                || name.eq_ignore_ascii_case(TRANSLATION_DIR)
                || name.starts_with('.')
                || name.to_ascii_lowercase().ends_with("_translation")
        })
}

fn refuse_existing(path: &Path) -> ToolResult<()> {
    if path.exists() {
        return Err(ToolError(format!(
            "output already exists: '{}'",
            path.display()
        )));
    }
    Ok(())
}

fn temporary_sibling(path: &Path) -> ToolResult<PathBuf> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ToolError(format!("invalid output name '{}'", path.display())))?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| ToolError(format!("system clock is before UNIX epoch: {error}")))?
        .as_nanos();
    Ok(parent.join(format!(".{name}.tmp-{}-{stamp}", std::process::id())))
}
