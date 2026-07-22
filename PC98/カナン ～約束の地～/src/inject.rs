use crate::logical_text::{build_logical_script, LogicalScript};
use crate::script::{
    build_cfg, cfg_text_candidates, is_main_story_script_name, patch_script_streams,
    validate_reachable_save_template, xor_script_body, ScriptError,
};
use crate::text_json::{validate_file_header, TextJsonError, TranslationFile};
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InjectError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Script(#[from] ScriptError),
    #[error(transparent)]
    TextJson(#[from] TextJsonError),
    #[error("invalid JSON at {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid injection input: {0}")]
    Invalid(String),
    #[error("output already exists: {0}")]
    OutputExists(PathBuf),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InjectStats {
    pub json_files: usize,
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub in_place: usize,
    pub relocated_entries: usize,
    pub appended_streams: usize,
    pub appended_bytes: usize,
    pub output_script_bytes: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone)]
struct PreparedScript {
    relative_path: PathBuf,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PreparedInject {
    pub source: PathBuf,
    pub translations: PathBuf,
    pub source_is_file: bool,
    pub stats: InjectStats,
    scripts: Vec<PreparedScript>,
}

fn io_error(path: &Path, source: std::io::Error) -> InjectError {
    InjectError::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn collect_json_files(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), InjectError> {
    let mut entries = fs::read_dir(root)
        .map_err(|source| io_error(root, source))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| io_error(root, source))?;
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_lowercase());
    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| io_error(&path, source))?;
        if file_type.is_symlink() {
            return Err(InjectError::Invalid(format!(
                "symbolic links are not supported: {}",
                path.display()
            )));
        }
        if file_type.is_dir() {
            collect_json_files(&path, output)?;
        } else if file_type.is_file()
            && path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.to_ascii_lowercase().ends_with(".s.json"))
        {
            output.push(path);
        }
    }
    Ok(())
}

fn safe_relative_path(value: &str) -> Result<PathBuf, InjectError> {
    if value.is_empty() || value.contains('\\') {
        return Err(InjectError::Invalid(format!(
            "_file must be a nonempty forward-slash relative path: {value:?}"
        )));
    }
    let path = Path::new(value);
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let text = part.to_str().ok_or_else(|| {
                    InjectError::Invalid(format!("_file is not valid Unicode: {value:?}"))
                })?;
                if text.contains(':') {
                    return Err(InjectError::Invalid(format!(
                        "_file contains an unsafe path component: {value:?}"
                    )));
                }
                result.push(part);
            }
            _ => {
                return Err(InjectError::Invalid(format!(
                    "_file is not a safe relative path: {value:?}"
                )))
            }
        }
    }
    if result.as_os_str().is_empty() {
        return Err(InjectError::Invalid(format!("_file is empty: {value:?}")));
    }
    Ok(result)
}

fn read_translation(path: &Path) -> Result<TranslationFile, InjectError> {
    let text = fs::read_to_string(path).map_err(|source| io_error(path, source))?;
    let file: TranslationFile =
        serde_json::from_str(&text).map_err(|source| InjectError::Json {
            path: path.to_path_buf(),
            source,
        })?;
    validate_file_header(&file)?;
    Ok(file)
}

fn story_model(source_path: &Path, encrypted: &[u8]) -> Result<LogicalScript, InjectError> {
    let decoded = xor_script_body(encrypted)?;
    let cfg = build_cfg(&decoded);
    if !cfg.warnings.is_empty() {
        let details = cfg
            .warnings
            .iter()
            .take(8)
            .map(|warning| {
                format!(
                    "0x{:04X} {:?}: {}",
                    warning.offset, warning.kind, warning.detail
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        return Err(InjectError::Invalid(format!(
            "{} has CFG warnings: {details}",
            source_path.display()
        )));
    }
    let candidates = cfg_text_candidates(&decoded, &cfg)?;
    let template = validate_reachable_save_template(&decoded, &cfg, &candidates)
        .map_err(|error| InjectError::Invalid(format!("{}: {error}", source_path.display())))?;
    let excluded: BTreeSet<_> = template
        .map(|value| value.text_instructions.into_iter().collect())
        .unwrap_or_default();
    Ok(build_logical_script(&cfg, &candidates, &excluded)?)
}

fn prepare_script(
    source_path: &Path,
    relative_path: &Path,
    translation: &TranslationFile,
) -> Result<(PreparedScript, crate::script::ScriptPatchStats), InjectError> {
    let expected_file = relative_path
        .components()
        .map(|component| match component {
            Component::Normal(value) => value.to_string_lossy().into_owned(),
            _ => String::new(),
        })
        .collect::<Vec<_>>()
        .join("/");
    if translation.file != expected_file {
        return Err(InjectError::Invalid(format!(
            "translation _file {:?} does not match source path {:?}",
            translation.file, expected_file
        )));
    }
    let source = fs::read(source_path).map_err(|source| io_error(source_path, source))?;
    let logical = story_model(source_path, &source)?;
    let patches = logical
        .prepare_patches(&translation.file, &translation.entries)
        .map_err(|error| InjectError::Invalid(format!("{}: {error}", translation.file)))?;
    let patched = patch_script_streams(&source, &patches)?;
    Ok((
        PreparedScript {
            relative_path: relative_path.to_path_buf(),
            bytes: patched.bytes,
        },
        patched.stats,
    ))
}

pub fn prepare_inject(source: &Path, translations: &Path) -> Result<PreparedInject, InjectError> {
    if !source.exists() {
        return Err(InjectError::Invalid(format!(
            "source does not exist: {}",
            source.display()
        )));
    }
    if !translations.exists() {
        return Err(InjectError::Invalid(format!(
            "translation input does not exist: {}",
            translations.display()
        )));
    }
    let source_is_file = source.is_file();
    if !source_is_file && !source.is_dir() {
        return Err(InjectError::Invalid(format!(
            "source is not a file or directory: {}",
            source.display()
        )));
    }
    let mut json_paths = if translations.is_file() {
        vec![translations.to_path_buf()]
    } else if translations.is_dir() {
        let mut paths = Vec::new();
        collect_json_files(translations, &mut paths)?;
        paths
    } else {
        return Err(InjectError::Invalid(format!(
            "translation input is not a file or directory: {}",
            translations.display()
        )));
    };
    json_paths.sort();
    if json_paths.is_empty() {
        return Err(InjectError::Invalid(format!(
            "no .S.json files found under {}",
            translations.display()
        )));
    }
    if source_is_file && json_paths.len() != 1 {
        return Err(InjectError::Invalid(
            "a source script requires exactly one translation JSON".to_owned(),
        ));
    }

    let source_name = source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            InjectError::Invalid(format!("invalid source filename: {}", source.display()))
        })?;
    if source_is_file && !is_main_story_script_name(source_name) {
        return Err(InjectError::Invalid(format!(
            "{} is outside the confirmed csNN_NN.s main-story scope",
            source.display()
        )));
    }

    let mut stats = InjectStats::default();
    let mut scripts = Vec::new();
    let mut seen_files = HashSet::new();
    for json_path in json_paths {
        let translation = read_translation(&json_path)?;
        let relative = safe_relative_path(&translation.file)?;
        let script_name = relative
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| {
                InjectError::Invalid(format!(
                    "translation _file has an invalid filename: {:?}",
                    translation.file
                ))
            })?;
        if !is_main_story_script_name(script_name) {
            return Err(InjectError::Invalid(format!(
                "translation _file is outside the confirmed main-story scope: {:?}",
                translation.file
            )));
        }
        let key = translation.file.to_ascii_lowercase();
        if !seen_files.insert(key) {
            return Err(InjectError::Invalid(format!(
                "duplicate translation _file {:?}",
                translation.file
            )));
        }
        let (source_path, expected_relative) = if source_is_file {
            (source.to_path_buf(), PathBuf::from(source_name))
        } else {
            (source.join(&relative), relative)
        };
        if !source_path.is_file() {
            return Err(InjectError::Invalid(format!(
                "translation source script does not exist: {}",
                source_path.display()
            )));
        }
        let (prepared, patch_stats) =
            prepare_script(&source_path, &expected_relative, &translation)?;
        stats.json_files += 1;
        stats.json_entries += translation.entries.len();
        stats.patched += patch_stats.patched;
        stats.unchanged += patch_stats.unchanged;
        stats.in_place += patch_stats.in_place;
        stats.relocated_entries += patch_stats.relocated_entries;
        stats.appended_streams += patch_stats.appended_streams;
        stats.appended_bytes += patch_stats.appended_bytes;
        stats.output_script_bytes += patch_stats.output_bytes;
        scripts.push(prepared);
    }

    Ok(PreparedInject {
        source: source.to_path_buf(),
        translations: translations.to_path_buf(),
        source_is_file,
        stats,
        scripts,
    })
}

fn copy_tree(source: &Path, output: &Path) -> Result<(), InjectError> {
    fs::create_dir(output).map_err(|source_error| io_error(output, source_error))?;
    let mut entries = fs::read_dir(source)
        .map_err(|source_error| io_error(source, source_error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source_error| io_error(source, source_error))?;
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_lowercase());
    for entry in entries {
        let source_path = entry.path();
        let output_path = output.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|source_error| io_error(&source_path, source_error))?;
        if file_type.is_symlink() {
            return Err(InjectError::Invalid(format!(
                "symbolic links are not supported: {}",
                source_path.display()
            )));
        }
        if file_type.is_dir() {
            copy_tree(&source_path, &output_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &output_path)
                .map_err(|source_error| io_error(&output_path, source_error))?;
        } else {
            return Err(InjectError::Invalid(format!(
                "unsupported filesystem entry: {}",
                source_path.display()
            )));
        }
    }
    Ok(())
}

impl PreparedInject {
    pub fn write_to(&self, output: &Path) -> Result<(), InjectError> {
        if output.exists() {
            return Err(InjectError::OutputExists(output.to_path_buf()));
        }
        if self.source_is_file {
            let script = self.scripts.first().expect("file injection has one script");
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent).map_err(|source| io_error(parent, source))?;
            }
            fs::write(output, &script.bytes).map_err(|source| io_error(output, source))?;
            return Ok(());
        }
        if output.starts_with(&self.source) {
            return Err(InjectError::Invalid(format!(
                "output directory cannot be inside source directory: {}",
                output.display()
            )));
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|source| io_error(parent, source))?;
        }
        let write_result = (|| {
            copy_tree(&self.source, output)?;
            for script in &self.scripts {
                let target = output.join(&script.relative_path);
                fs::write(&target, &script.bytes).map_err(|source| io_error(&target, source))?;
            }
            Ok(())
        })();
        if write_result.is_err() && output.exists() {
            let _ = fs::remove_dir_all(output);
        }
        write_result
    }
}
