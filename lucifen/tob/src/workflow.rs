use crate::format::parse;
use crate::rebuild::rebuild;
use crate::translation::{extract, prepare_replacements, TranslationFile};
use crate::Result;
use serde::Serialize;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Default, Clone, Copy)]
pub struct Totals {
    pub files: usize,
    pub entries: usize,
    pub changed_files: usize,
    pub changed_entries: usize,
    pub verified_modifications: usize,
}

#[derive(Debug)]
struct PreparedFile {
    relative: PathBuf,
    bytes: Vec<u8>,
}

pub fn extract_path(input: &Path, output: &Path, overwrite: bool) -> Result<Totals> {
    let files = collect_tob_files(input)?;
    if files.is_empty() {
        return Err(format!("no TOB files found in {}", input.display()));
    }
    validate_separate_output(input, output)?;
    let mut prepared = Vec::with_capacity(files.len());
    let mut totals = Totals::default();
    for (source, relative) in files {
        let bytes =
            fs::read(&source).map_err(|error| format!("read {}: {error}", source.display()))?;
        let file = parse(bytes).map_err(|error| format!("{}: {error}", source.display()))?;
        let relative_name = portable_path(&relative)?;
        let translation = extract(&relative_name, &file)
            .map_err(|error| format!("{}: {error}", source.display()))?;
        totals.files += 1;
        totals.entries += translation.entries.len();
        let mut json = serde_json::to_vec_pretty(&translation)
            .map_err(|error| format!("serialize {relative_name}: {error}"))?;
        json.push(b'\n');
        let destination = if input.is_file() {
            PathBuf::from(output.file_name().ok_or("JSON output has no file name")?)
        } else {
            relative.with_extension("json")
        };
        prepared.push(PreparedFile {
            relative: destination,
            bytes: json,
        });
    }
    write_prepared(output, input.is_file(), prepared, overwrite)?;
    Ok(totals)
}

pub fn inject_path(
    input: &Path,
    translation_root: &Path,
    output: &Path,
    overwrite: bool,
    names_writable: bool,
) -> Result<Totals> {
    let files = collect_tob_files(input)?;
    if files.is_empty() {
        return Err(format!("no TOB files found in {}", input.display()));
    }
    validate_separate_output(input, output)?;
    validate_separate_output(translation_root, output)?;
    let mut prepared = Vec::with_capacity(files.len());
    let mut totals = Totals::default();
    for (source, relative) in files {
        let translation_path = if translation_root.is_file() {
            translation_root.to_path_buf()
        } else {
            safe_join(translation_root, &relative.with_extension("json"))?
        };
        let json = fs::read(&translation_path)
            .map_err(|error| format!("read {}: {error}", translation_path.display()))?;
        let translation: TranslationFile = serde_json::from_slice(&json)
            .map_err(|error| format!("parse {}: {error}", translation_path.display()))?;
        let bytes =
            fs::read(&source).map_err(|error| format!("read {}: {error}", source.display()))?;
        let file = parse(bytes).map_err(|error| format!("{}: {error}", source.display()))?;
        let relative_name = portable_path(&relative)?;
        let replacements =
            prepare_replacements(&relative_name, &file, &translation, names_writable)
                .map_err(|error| format!("{}: {error}", source.display()))?;
        let changed_entries = replacements
            .iter()
            .filter(|replacement| {
                file.bytes
                    .get(replacement.slot.range.clone())
                    .is_some_and(|source| source != replacement.bytes)
            })
            .count();
        let rebuilt = rebuild(&file, replacements)
            .map_err(|error| format!("{}: {error}", source.display()))?;
        totals.files += 1;
        totals.entries += translation.entries.len();
        totals.changed_entries += changed_entries;
        totals.changed_files += usize::from(rebuilt != file.bytes);
        prepared.push(PreparedFile {
            relative,
            bytes: rebuilt,
        });
    }

    if input.is_file() {
        write_prepared(output, true, prepared, overwrite)?;
    } else {
        write_injected_tree(input, output, prepared, overwrite)?;
    }
    Ok(totals)
}

pub fn verify_path(input: &Path) -> Result<Totals> {
    let files = collect_tob_files(input)?;
    if files.is_empty() {
        return Err(format!("no TOB files found in {}", input.display()));
    }
    let mut totals = Totals::default();
    let mut remaining_cases = vec!["title", "summary", "narration", "dialogue", "selection"];
    for (source, relative) in files {
        let bytes =
            fs::read(&source).map_err(|error| format!("read {}: {error}", source.display()))?;
        let file = parse(bytes).map_err(|error| format!("{}: {error}", source.display()))?;
        let relative_name = portable_path(&relative)?;
        let translation = extract(&relative_name, &file)
            .map_err(|error| format!("{}: {error}", source.display()))?;
        let replacements = prepare_replacements(&relative_name, &file, &translation, false)
            .map_err(|error| format!("{}: {error}", source.display()))?;
        let rebuilt = rebuild(&file, replacements)
            .map_err(|error| format!("{}: {error}", source.display()))?;
        if rebuilt != file.bytes {
            return Err(format!(
                "{} unchanged round trip is not byte-exact",
                source.display()
            ));
        }
        let mut case_index = 0usize;
        while case_index < remaining_cases.len() {
            let entry_type = remaining_cases[case_index];
            let Some(entry_index) = translation
                .entries
                .iter()
                .position(|entry| entry.entry_type == entry_type)
            else {
                case_index += 1;
                continue;
            };
            verify_modified_entry(&relative_name, &file, &translation, entry_index)?;
            totals.verified_modifications += 1;
            remaining_cases.remove(case_index);
        }
        totals.files += 1;
        totals.entries += translation.entries.len();
    }
    if !remaining_cases.is_empty() {
        return Err(format!(
            "real-sample modified verification did not find entry types: {}",
            remaining_cases.join(", ")
        ));
    }
    Ok(totals)
}

fn verify_modified_entry(
    file_name: &str,
    file: &crate::format::TobFile,
    source: &TranslationFile,
    entry_index: usize,
) -> Result<()> {
    let mut translation = source.clone();
    let entry_type = translation.entries[entry_index].entry_type.clone();
    let expected_single;
    let expected_parts;
    if let Some(message) = translation.entries[entry_index].message.as_mut() {
        let replacement = match entry_type.as_str() {
            "title" => "検証用の長いタイトルです",
            "summary" => "検証",
            "narration" => "短い検証文。",
            "dialogue" => "「これは長さを変えるための検証用台詞です」",
            other => return Err(format!("unsupported verification entry type {other}")),
        };
        *message = replacement.to_string();
        expected_single = Some(replacement.to_string());
        expected_parts = None;
    } else {
        let parts = translation.entries[entry_index]
            .message_parts
            .as_mut()
            .ok_or("multipart verification entry has no message_parts")?;
        for (index, part) in parts.iter_mut().enumerate() {
            part.push_str(&format!("・検証{}", index + 1));
        }
        expected_parts = Some(parts.clone());
        expected_single = None;
    }
    let replacements = prepare_replacements(file_name, file, &translation, false)?;
    let rebuilt = rebuild(file, replacements)?;
    if rebuilt == file.bytes {
        return Err(format!(
            "modified verification for {} did not change the TOB",
            entry_type
        ));
    }
    let reparsed = parse(rebuilt)?;
    let extracted = extract(file_name, &reparsed)?;
    let actual = extracted
        .entries
        .get(entry_index)
        .ok_or("modified verification entry disappeared")?;
    if actual.message != expected_single || actual.message_parts != expected_parts {
        return Err(format!(
            "modified verification for {} did not re-extract the requested text",
            entry_type
        ));
    }
    Ok(())
}

fn collect_tob_files(input: &Path) -> Result<Vec<(PathBuf, PathBuf)>> {
    if input.is_file() {
        if !has_extension(input, "tob") {
            return Err(format!("{} is not a .tob file", input.display()));
        }
        let name = input
            .file_name()
            .ok_or_else(|| format!("{} has no file name", input.display()))?;
        return Ok(vec![(input.to_path_buf(), PathBuf::from(name))]);
    }
    if !input.is_dir() {
        return Err(format!("input does not exist: {}", input.display()));
    }
    let mut files = Vec::new();
    collect_recursive(input, input, &mut files)?;
    files.sort_by(|left, right| left.1.cmp(&right.1));
    Ok(files)
}

fn collect_recursive(
    root: &Path,
    directory: &Path,
    output: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<()> {
    for entry in
        fs::read_dir(directory).map_err(|error| format!("read {}: {error}", directory.display()))?
    {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_dir() {
            collect_recursive(root, &path, output)?;
        } else if has_extension(&path, "tob") {
            let relative = path
                .strip_prefix(root)
                .map_err(|error| format!("make relative path: {error}"))?
                .to_path_buf();
            output.push((path, relative));
        }
    }
    Ok(())
}

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(expected))
}

fn write_prepared(
    output: &Path,
    single_file: bool,
    prepared: Vec<PreparedFile>,
    overwrite: bool,
) -> Result<()> {
    if output.exists() && !overwrite {
        return Err(format!(
            "output exists: {} (pass --overwrite)",
            output.display()
        ));
    }
    if single_file {
        let item = prepared
            .into_iter()
            .next()
            .ok_or("no prepared output file")?;
        atomic_write(output, &item.bytes, overwrite)
    } else {
        let staging = staging_path(output)?;
        if staging.exists() {
            return Err(format!(
                "staging path already exists: {}",
                staging.display()
            ));
        }
        fs::create_dir_all(&staging)
            .map_err(|error| format!("create {}: {error}", staging.display()))?;
        let result = (|| {
            for item in prepared {
                let destination = safe_join(&staging, &item.relative)?;
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|error| format!("create {}: {error}", parent.display()))?;
                }
                fs::write(&destination, item.bytes)
                    .map_err(|error| format!("write {}: {error}", destination.display()))?;
            }
            replace_directory(&staging, output, overwrite)
        })();
        if result.is_err() && staging.exists() {
            let _ = fs::remove_dir_all(&staging);
        }
        result
    }
}

fn write_injected_tree(
    input: &Path,
    output: &Path,
    prepared: Vec<PreparedFile>,
    overwrite: bool,
) -> Result<()> {
    if output.exists() && !overwrite {
        return Err(format!(
            "output exists: {} (pass --overwrite)",
            output.display()
        ));
    }
    let staging = staging_path(output)?;
    if staging.exists() {
        return Err(format!(
            "staging path already exists: {}",
            staging.display()
        ));
    }
    let result = (|| {
        copy_tree(input, &staging)?;
        for item in prepared {
            let destination = safe_join(&staging, &item.relative)?;
            fs::write(&destination, item.bytes)
                .map_err(|error| format!("write {}: {error}", destination.display()))?;
        }
        replace_directory(&staging, output, overwrite)
    })();
    if result.is_err() && staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)
        .map_err(|error| format!("create {}: {error}", destination.display()))?;
    for entry in
        fs::read_dir(source).map_err(|error| format!("read {}: {error}", source.display()))?
    {
        let path = entry.map_err(|error| error.to_string())?.path();
        let target = destination.join(path.file_name().ok_or("source entry has no file name")?);
        if path.is_dir() {
            copy_tree(&path, &target)?;
        } else {
            fs::copy(&path, &target)
                .map_err(|error| format!("copy {}: {error}", path.display()))?;
        }
    }
    Ok(())
}

fn replace_directory(staging: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if output.exists() {
        if !overwrite {
            return Err(format!(
                "output exists: {} (pass --overwrite)",
                output.display()
            ));
        }
        if output.is_dir() {
            fs::remove_dir_all(output)
                .map_err(|error| format!("remove old output {}: {error}", output.display()))?;
        } else {
            fs::remove_file(output)
                .map_err(|error| format!("remove old output {}: {error}", output.display()))?;
        }
    }
    fs::rename(staging, output).map_err(|error| {
        format!(
            "rename {} to {}: {error}",
            staging.display(),
            output.display()
        )
    })
}

fn atomic_write(output: &Path, bytes: &[u8], overwrite: bool) -> Result<()> {
    if output.exists() && !overwrite {
        return Err(format!(
            "output exists: {} (pass --overwrite)",
            output.display()
        ));
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    let temporary = staging_path(output)?;
    fs::write(&temporary, bytes)
        .map_err(|error| format!("write {}: {error}", temporary.display()))?;
    if output.exists() {
        fs::remove_file(output)
            .map_err(|error| format!("remove old output {}: {error}", output.display()))?;
    }
    fs::rename(&temporary, output).map_err(|error| {
        format!(
            "rename {} to {}: {error}",
            temporary.display(),
            output.display()
        )
    })
}

fn staging_path(output: &Path) -> Result<PathBuf> {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let name = output
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("output path has no UTF-8 file name: {}", output.display()))?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock error: {error}"))?
        .as_nanos();
    Ok(parent.join(format!(
        ".{name}.staging-{}-{timestamp}",
        std::process::id()
    )))
}

fn safe_join(root: &Path, relative: &Path) -> Result<PathBuf> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe relative path: {}", relative.display()));
    }
    Ok(root.join(relative))
}

fn portable_path(path: &Path) -> Result<String> {
    let components = path
        .components()
        .map(|component| match component {
            Component::Normal(value) => value
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("path is not valid UTF-8: {}", path.display())),
            _ => Err(format!(
                "path is not a safe relative path: {}",
                path.display()
            )),
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(components.join("/"))
}

fn validate_separate_output(input: &Path, output: &Path) -> Result<()> {
    let input =
        fs::canonicalize(input).map_err(|error| format!("resolve {}: {error}", input.display()))?;
    let output = resolve_future_path(output)?;
    if output == input || output.starts_with(&input) {
        return Err(format!(
            "output must not be the input or a child of it: {}",
            output.display()
        ));
    }
    Ok(())
}

fn resolve_future_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path)
            .map_err(|error| format!("resolve {}: {error}", path.display()));
    }
    let mut cursor = path;
    let mut missing = Vec::new();
    while !cursor.exists() {
        let name = cursor
            .file_name()
            .ok_or_else(|| format!("path has no existing ancestor: {}", path.display()))?;
        missing.push(name.to_os_string());
        cursor = cursor.parent().unwrap_or_else(|| Path::new("."));
    }
    let mut resolved = fs::canonicalize(cursor)
        .map_err(|error| format!("resolve {}: {error}", cursor.display()))?;
    for component in missing.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

pub fn to_pretty_json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}
