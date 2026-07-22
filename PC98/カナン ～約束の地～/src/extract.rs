use crate::logical_text::build_logical_script;
use crate::script::{
    build_cfg, cfg_text_candidates, is_main_story_script_name, validate_reachable_save_template,
    xor_script_body, ScriptError,
};
use crate::text_json::{TextJsonError, TranslationFile, TEXT_JSON_FORMAT};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExtractError {
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
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("invalid extract input: {0}")]
    Invalid(String),
    #[error("output already exists: {0}")]
    OutputExists(PathBuf),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExtractStats {
    pub scanned_files: usize,
    pub json_files: usize,
    pub extracted_entries: usize,
    pub skipped_entries: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone)]
struct PreparedJson {
    relative_output: PathBuf,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PreparedExtract {
    pub input: PathBuf,
    pub input_is_file: bool,
    pub stats: ExtractStats,
    files: Vec<PreparedJson>,
}

fn io_error(path: &Path, source: std::io::Error) -> ExtractError {
    ExtractError::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn collect_scripts(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), ExtractError> {
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
            return Err(ExtractError::Invalid(format!(
                "symbolic links are not supported: {}",
                path.display()
            )));
        }
        if file_type.is_dir() {
            collect_scripts(&path, output)?;
        } else if file_type.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("s"))
        {
            output.push(path);
        }
    }
    Ok(())
}

fn relative_json_name(relative_script: &Path) -> Result<PathBuf, ExtractError> {
    let name = relative_script
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ExtractError::Invalid(format!(
                "script filename is not valid Unicode: {}",
                relative_script.display()
            ))
        })?;
    Ok(relative_script.with_file_name(format!("{name}.json")))
}

fn portable_relative_path(path: &Path) -> Result<String, ExtractError> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => parts.push(value.to_str().ok_or_else(|| {
                ExtractError::Invalid(format!(
                    "relative path is not valid Unicode: {}",
                    path.display()
                ))
            })?),
            _ => {
                return Err(ExtractError::Invalid(format!(
                    "unsafe relative script path: {}",
                    path.display()
                )))
            }
        }
    }
    Ok(parts.join("/"))
}

fn extract_script(
    source_path: &Path,
    relative_path: &Path,
) -> Result<(TranslationFile, usize), ExtractError> {
    let source = fs::read(source_path).map_err(|source| io_error(source_path, source))?;
    let decoded = xor_script_body(&source)?;
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
        return Err(ExtractError::Invalid(format!(
            "{} has CFG warnings: {details}",
            source_path.display()
        )));
    }
    let candidates = cfg_text_candidates(&decoded, &cfg)?;
    let template = validate_reachable_save_template(&decoded, &cfg, &candidates)
        .map_err(|error| ExtractError::Invalid(format!("{}: {error}", source_path.display())))?;
    let excluded: BTreeSet<_> = template
        .map(|value| value.text_instructions.into_iter().collect())
        .unwrap_or_default();
    let file = portable_relative_path(relative_path)?;
    let logical = build_logical_script(&cfg, &candidates, &excluded)?;
    let entries = logical.translation_entries(&file);
    let unchanged = logical.prepare_patches(&file, &entries)?;
    if unchanged
        .iter()
        .any(|patch| patch.expected_stream != patch.replacement_stream)
    {
        return Err(ExtractError::Invalid(format!(
            "{} logical-page rendering is not byte-exact",
            source_path.display()
        )));
    }
    let skipped = excluded.len();
    Ok((
        TranslationFile {
            format: TEXT_JSON_FORMAT.to_owned(),
            file,
            entries,
        },
        skipped,
    ))
}

pub fn prepare_extract(input: &Path) -> Result<PreparedExtract, ExtractError> {
    if !input.exists() {
        return Err(ExtractError::Invalid(format!(
            "input does not exist: {}",
            input.display()
        )));
    }
    let input_is_file = input.is_file();
    let mut scripts = if input_is_file {
        vec![input.to_path_buf()]
    } else if input.is_dir() {
        let mut paths = Vec::new();
        collect_scripts(input, &mut paths)?;
        paths
    } else {
        return Err(ExtractError::Invalid(format!(
            "input is not a file or directory: {}",
            input.display()
        )));
    };
    scripts.sort();
    if scripts.is_empty() {
        return Err(ExtractError::Invalid(format!(
            "no .S scripts found under {}",
            input.display()
        )));
    }

    let mut stats = ExtractStats {
        scanned_files: scripts.len(),
        ..ExtractStats::default()
    };
    let mut files = Vec::new();
    for script in scripts {
        let name = script
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| {
                ExtractError::Invalid(format!(
                    "script filename is not valid Unicode: {}",
                    script.display()
                ))
            })?;
        if !is_main_story_script_name(name) {
            if input_is_file {
                return Err(ExtractError::Invalid(format!(
                    "{} is outside the confirmed csNN_NN.s main-story scope",
                    script.display()
                )));
            }
            let source = fs::read(&script).map_err(|source| io_error(&script, source))?;
            let decoded = xor_script_body(&source)?;
            let cfg = build_cfg(&decoded);
            stats.skipped_entries += cfg_text_candidates(&decoded, &cfg)?.len();
            continue;
        }
        let relative = if input_is_file {
            PathBuf::from(name)
        } else {
            script
                .strip_prefix(input)
                .map_err(|_| {
                    ExtractError::Invalid(format!(
                        "{} is outside input root {}",
                        script.display(),
                        input.display()
                    ))
                })?
                .to_path_buf()
        };
        let (translation, skipped) = extract_script(&script, &relative)?;
        stats.extracted_entries += translation.entries.len();
        stats.skipped_entries += skipped;
        stats.json_files += 1;
        let mut json = serde_json::to_string_pretty(&translation)?;
        json.push('\n');
        files.push(PreparedJson {
            relative_output: relative_json_name(&relative)?,
            bytes: json.into_bytes(),
        });
    }
    if files.is_empty() {
        return Err(ExtractError::Invalid(
            "no confirmed main-story scripts were found".to_owned(),
        ));
    }
    Ok(PreparedExtract {
        input: input.to_path_buf(),
        input_is_file,
        stats,
        files,
    })
}

impl PreparedExtract {
    pub fn write_to(&self, output: &Path) -> Result<(), ExtractError> {
        if output.exists() {
            return Err(ExtractError::OutputExists(output.to_path_buf()));
        }
        if self.input_is_file {
            let file = self.files.first().expect("file extraction has one JSON");
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent).map_err(|source| io_error(parent, source))?;
            }
            fs::write(output, &file.bytes).map_err(|source| io_error(output, source))?;
            return Ok(());
        }

        fs::create_dir_all(output).map_err(|source| io_error(output, source))?;
        let write_result = (|| {
            for file in &self.files {
                let target = output.join(&file.relative_output);
                let parent = target.parent().expect("relative JSON has a parent");
                fs::create_dir_all(parent).map_err(|source| io_error(parent, source))?;
                fs::write(&target, &file.bytes).map_err(|source| io_error(&target, source))?;
            }
            Ok(())
        })();
        if write_result.is_err() {
            let _ = fs::remove_dir_all(output);
        }
        write_result
    }
}
