use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail, ensure};
use tempfile::Builder;

use crate::{
    codec::{CharacterMap, CharacterSubstitutions},
    model::TranslationFile,
    sdt::parse_sdt,
};

const SCHEMA: &str = "liena-sdt-v1";

#[derive(Debug, Default)]
pub struct OperationSummary {
    pub files_scanned: usize,
    pub json_files: usize,
    pub entries: usize,
    pub changed: usize,
    pub substituted_characters: usize,
    pub warnings: Vec<String>,
}

type TranslationsBySource = BTreeMap<String, (PathBuf, TranslationFile)>;

pub fn extract_directory(
    source: &Path,
    output: &Path,
    mapping: Option<&Path>,
    overwrite: bool,
) -> Result<OperationSummary> {
    ensure!(
        source.is_dir(),
        "source is not a directory: {}",
        source.display()
    );
    let map = CharacterMap::load(mapping)?;
    let protected = protected_paths(source, None, mapping);
    let output = validate_output_path(output, overwrite, &protected)?;
    let files = collect_files(source)?;
    let mut prepared = Vec::new();
    let mut summary = OperationSummary::default();

    for path in files {
        if !is_sdt(&path) {
            continue;
        }
        summary.files_scanned += 1;
        let relative = path.strip_prefix(source).expect("collected below source");
        let bytes =
            fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
        let parsed = parse_sdt(relative, bytes, &map)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        summary.entries += parsed.translation.entries.len();
        if parsed.translation.entries.is_empty() {
            continue;
        }
        let mut json = serde_json::to_vec_pretty(&parsed.translation)?;
        json.push(b'\n');
        prepared.push((json_relative_path(relative), json));
        summary.json_files += 1;
    }

    write_output_directory(&output, overwrite, |stage| {
        for (relative, bytes) in &prepared {
            let target = stage.join(relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&target, bytes)
                .with_context(|| format!("failed to write {}", target.display()))?;
        }
        Ok(())
    })?;
    Ok(summary)
}

pub fn inject_directory(
    source: &Path,
    translations: &Path,
    output: &Path,
    mapping: Option<&Path>,
    overwrite: bool,
) -> Result<OperationSummary> {
    ensure!(
        source.is_dir(),
        "source is not a directory: {}",
        source.display()
    );
    ensure!(
        translations.is_dir(),
        "translation input is not a directory: {}",
        translations.display()
    );
    let map = CharacterMap::load(mapping)?;
    let substitutions = CharacterSubstitutions::built_in()?;
    let protected = protected_paths(source, Some(translations), mapping);
    let output = validate_output_path(output, overwrite, &protected)?;
    let (mut translation_by_source, json_files) = load_translations(translations)?;
    let source_files = collect_files(source)?;
    let mut rebuilt = Vec::new();
    let mut summary = OperationSummary {
        json_files,
        ..OperationSummary::default()
    };

    for path in &source_files {
        if !is_sdt(path) {
            continue;
        }
        summary.files_scanned += 1;
        let relative = path.strip_prefix(source).expect("collected below source");
        let key = normalized_relative(&relative.to_string_lossy())?;
        let Some((json_path, translation)) = translation_by_source.remove(&key) else {
            continue;
        };
        let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
        let parsed = parse_sdt(relative, bytes, &map)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        let result = parsed
            .inject_with_substitutions(&translation, &map, &substitutions)
            .with_context(|| format!("failed to apply {}", json_path.display()))?;
        summary.entries += translation.entries.len();
        summary.changed += result.changed_entries;
        summary.substituted_characters += result.substituted_characters;
        rebuilt.push((relative.to_path_buf(), result.bytes));
    }
    ensure_all_sources_found(&translation_by_source)?;

    write_output_directory(&output, overwrite, |stage| {
        copy_tree(source, stage)?;
        for (relative, bytes) in &rebuilt {
            let target = stage.join(relative);
            fs::write(&target, bytes)
                .with_context(|| format!("failed to write {}", target.display()))?;
        }
        Ok(())
    })?;
    Ok(summary)
}

pub fn verify_directory(
    source: &Path,
    translations: &Path,
    mapping: Option<&Path>,
) -> Result<OperationSummary> {
    ensure!(
        source.is_dir(),
        "source is not a directory: {}",
        source.display()
    );
    ensure!(
        translations.is_dir(),
        "translation input is not a directory: {}",
        translations.display()
    );
    let map = CharacterMap::load(mapping)?;
    let (mut translation_by_source, json_files) = load_translations(translations)?;
    let mut summary = OperationSummary {
        json_files,
        ..OperationSummary::default()
    };

    for path in collect_files(source)? {
        if !is_sdt(&path) {
            continue;
        }
        summary.files_scanned += 1;
        let relative = path.strip_prefix(source).expect("collected below source");
        let key = normalized_relative(&relative.to_string_lossy())?;
        let Some((json_path, translation)) = translation_by_source.remove(&key) else {
            continue;
        };
        let bytes =
            fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
        let parsed = parse_sdt(relative, bytes.clone(), &map)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        let result = parsed
            .inject(&translation, &map)
            .with_context(|| format!("failed to apply {}", json_path.display()))?;
        ensure!(
            result.bytes == bytes,
            "unchanged round trip differs: {}",
            relative.display()
        );
        ensure!(
            result.changed_entries == 0,
            "verify accepts only unchanged translations: {}",
            json_path.display()
        );
        summary.entries += translation.entries.len();
    }
    ensure_all_sources_found(&translation_by_source)?;
    Ok(summary)
}

fn load_translations(translations: &Path) -> Result<(TranslationsBySource, usize)> {
    let mut by_source = BTreeMap::new();
    let mut json_files = 0;
    for json_path in collect_files(translations)? {
        if !has_extension(&json_path, "json") {
            continue;
        }
        json_files += 1;
        let raw = fs::read(&json_path)
            .with_context(|| format!("failed to read {}", json_path.display()))?;
        let translation: TranslationFile = serde_json::from_slice(&raw)
            .with_context(|| format!("invalid translation JSON {}", json_path.display()))?;
        ensure!(
            translation.schema == SCHEMA,
            "unsupported schema {:?} in {}",
            translation.schema,
            json_path.display()
        );
        let key = normalized_relative(&translation.source_file)?;
        if by_source
            .insert(key.clone(), (json_path, translation))
            .is_some()
        {
            bail!("multiple translation JSON files target {key:?}");
        }
    }
    ensure!(
        json_files > 0,
        "translation directory contains no JSON files: {}",
        translations.display()
    );
    Ok((by_source, json_files))
}

fn ensure_all_sources_found(remaining: &TranslationsBySource) -> Result<()> {
    if !remaining.is_empty() {
        let missing = remaining.keys().cloned().collect::<Vec<_>>().join(", ");
        bail!("translation JSON references missing source file(s): {missing}");
    }
    Ok(())
}

fn protected_paths<'a>(
    source: &'a Path,
    translations: Option<&'a Path>,
    mapping: Option<&'a Path>,
) -> Vec<&'a Path> {
    let mut paths = vec![source];
    paths.extend(translations);
    paths.extend(mapping);
    paths
}

fn validate_output_path(output: &Path, overwrite: bool, protected: &[&Path]) -> Result<PathBuf> {
    ensure!(
        output.file_name().is_some(),
        "refusing broad output path {}",
        output.display()
    );
    if output.exists() {
        let metadata = fs::symlink_metadata(output)?;
        ensure!(
            !metadata.file_type().is_symlink(),
            "output cannot be a symbolic link: {}",
            output.display()
        );
    }
    let resolved_output = resolve_path(output)?;
    for input in protected {
        let resolved_input = resolve_path(input)?;
        ensure!(
            !paths_overlap(&resolved_output, &resolved_input),
            "output {} overlaps input {}",
            output.display(),
            input.display()
        );
    }
    if resolved_output.exists() {
        ensure!(
            overwrite,
            "output already exists (pass --overwrite): {}",
            output.display()
        );
        ensure!(
            resolved_output.is_dir(),
            "output exists and is not a directory: {}",
            output.display()
        );
    }
    Ok(resolved_output)
}

fn resolve_path(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut cursor = absolute.as_path();
    let mut missing: Vec<OsString> = Vec::new();
    while !cursor.exists() {
        let name = cursor.file_name().with_context(|| {
            format!(
                "cannot resolve path with no existing ancestor: {}",
                path.display()
            )
        })?;
        missing.push(name.to_os_string());
        cursor = cursor.parent().with_context(|| {
            format!(
                "cannot resolve path with no existing ancestor: {}",
                path.display()
            )
        })?;
    }
    let mut resolved = fs::canonicalize(cursor)
        .with_context(|| format!("failed to resolve path {}", path.display()))?;
    for component in missing.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    let left = comparison_components(left);
    let right = comparison_components(right);
    starts_with_components(&left, &right) || starts_with_components(&right, &left)
}

fn comparison_components(path: &Path) -> Vec<String> {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
        .collect()
}

fn starts_with_components(path: &[String], prefix: &[String]) -> bool {
    path.len() >= prefix.len() && path[..prefix.len()] == *prefix
}

fn write_output_directory<F>(output: &Path, overwrite: bool, build: F) -> Result<()>
where
    F: FnOnce(&Path) -> Result<()>,
{
    let parent = output
        .parent()
        .with_context(|| format!("output has no parent directory: {}", output.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create output parent {}", parent.display()))?;
    let stage = Builder::new()
        .prefix(".liena-sdt-stage-")
        .tempdir_in(parent)
        .with_context(|| format!("failed to create staging directory in {}", parent.display()))?;
    build(stage.path())?;
    let stage_path = stage.keep();
    let cleanup_stage = |cause: anyhow::Error| match fs::remove_dir_all(&stage_path) {
        Ok(()) => cause,
        Err(cleanup_error) => cause.context(format!(
            "also failed to remove staging directory {} ({cleanup_error})",
            stage_path.display()
        )),
    };

    let backup = if output.exists() {
        if !overwrite {
            return Err(cleanup_stage(anyhow!(
                "output already exists: {}",
                output.display()
            )));
        }
        let backup = match unused_backup_path(parent) {
            Ok(backup) => backup,
            Err(error) => return Err(cleanup_stage(error)),
        };
        if let Err(error) = fs::rename(output, &backup) {
            let error = anyhow!(error).context(format!(
                "failed to move existing output {} to backup {}",
                output.display(),
                backup.display()
            ));
            return Err(cleanup_stage(error));
        }
        Some(backup)
    } else {
        None
    };

    if let Err(error) = fs::rename(&stage_path, output) {
        let _ = fs::remove_dir_all(&stage_path);
        if let Some(backup) = &backup
            && let Err(restore_error) = fs::rename(backup, output)
        {
            bail!(
                "failed to install staged output ({error}); also failed to restore backup {} ({restore_error})",
                backup.display()
            );
        }
        return Err(error)
            .with_context(|| format!("failed to install staged output at {}", output.display()));
    }

    if let Some(backup) = backup {
        fs::remove_dir_all(&backup).with_context(|| {
            format!(
                "output succeeded but failed to remove backup {}",
                backup.display()
            )
        })?;
    }
    Ok(())
}

fn unused_backup_path(parent: &Path) -> Result<PathBuf> {
    for attempt in 0..1000_u32 {
        let candidate = parent.join(format!(
            ".liena-sdt-backup-{}-{attempt}",
            std::process::id()
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    bail!(
        "could not allocate a backup directory in {}",
        parent.display()
    )
}

fn copy_tree(source: &Path, output: &Path) -> Result<()> {
    let mut pending = vec![(source.to_path_buf(), output.to_path_buf())];
    while let Some((source_dir, output_dir)) = pending.pop() {
        fs::create_dir_all(&output_dir)?;
        for entry in fs::read_dir(&source_dir)
            .with_context(|| format!("failed to list {}", source_dir.display()))?
        {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let source_path = entry.path();
            let output_path = output_dir.join(entry.file_name());
            if file_type.is_symlink() {
                bail!(
                    "symbolic links are not supported: {}",
                    source_path.display()
                );
            }
            if file_type.is_dir() {
                pending.push((source_path, output_path));
            } else if file_type.is_file() {
                fs::copy(&source_path, &output_path).with_context(|| {
                    format!(
                        "failed to copy {} to {}",
                        source_path.display(),
                        output_path.display()
                    )
                })?;
            }
        }
    }
    Ok(())
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)
            .with_context(|| format!("failed to list {}", directory.display()))?
        {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                bail!(
                    "symbolic links are not supported: {}",
                    entry.path().display()
                );
            }
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file() {
                files.push(entry.path());
            }
        }
    }
    files.sort();
    Ok(files)
}

fn is_sdt(path: &Path) -> bool {
    has_extension(path, "sdt")
}

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case(expected))
}

fn json_relative_path(source: &Path) -> PathBuf {
    let mut result = source.to_path_buf();
    let name = source
        .file_name()
        .unwrap_or_else(|| OsStr::new("script.sdt"))
        .to_string_lossy();
    result.set_file_name(format!("{name}.json"));
    result
}

fn normalized_relative(value: &str) -> Result<String> {
    let path = Path::new(value);
    ensure!(
        !path.is_absolute(),
        "source_file must be relative: {value:?}"
    );
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            _ => bail!("unsafe source_file path: {value:?}"),
        }
    }
    ensure!(!parts.is_empty(), "empty source_file path");
    Ok(parts.join("/").to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_must_not_overlap_inputs() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        fs::create_dir(&source).unwrap();
        let output = source.join("output");
        let error = validate_output_path(&output, false, &[&source]).unwrap_err();
        assert!(error.to_string().contains("overlaps input"));
    }

    #[test]
    fn failed_staging_preserves_existing_output() {
        let root = tempfile::tempdir().unwrap();
        let output = root.path().join("output");
        fs::create_dir(&output).unwrap();
        fs::write(output.join("old.txt"), b"old").unwrap();
        let result = write_output_directory(&output, true, |stage| {
            fs::write(stage.join("new.txt"), b"new")?;
            bail!("synthetic failure")
        });
        assert!(result.is_err());
        assert_eq!(fs::read(output.join("old.txt")).unwrap(), b"old");
        assert!(!output.join("new.txt").exists());
    }

    #[test]
    fn extension_matching_is_case_insensitive() {
        assert!(has_extension(Path::new("SCRIPT.JSON"), "json"));
        assert!(is_sdt(Path::new("SCRIPT.sDt")));
    }

    #[test]
    fn relative_source_rejects_parent_traversal() {
        assert!(normalized_relative("../LC00.SDT").is_err());
        assert!(normalized_relative("sub/LC00.SDT").is_ok());
    }
}
