use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::adv;
use crate::text::{self, TranslationEntry};
use crate::Result;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExtractReport {
    pub scanned_files: usize,
    pub json_files: usize,
    pub extracted_entries: usize,
    pub skipped: usize,
    pub warnings: usize,
    pub output: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InjectReport {
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub warnings: usize,
    pub copied_files: usize,
    pub output: PathBuf,
}

fn is_adv(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("adv"))
}

fn append_to_name(path: &Path, suffix: &str) -> Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| format!("path has no file name: {}", path.display()))?;
    let mut output_name = OsString::from(name);
    output_name.push(suffix);
    Ok(path.with_file_name(output_name))
}

fn injected_name(path: &Path) -> Result<PathBuf> {
    if path.is_dir() {
        return append_to_name(path, "_injected");
    }
    let stem = path
        .file_stem()
        .ok_or_else(|| format!("path has no file stem: {}", path.display()))?;
    let mut name = OsString::from(stem);
    name.push("_injected");
    if let Some(extension) = path.extension() {
        name.push(".");
        name.push(extension);
    }
    Ok(path.with_file_name(name))
}

fn relative_json_path(relative_source: &Path) -> Result<PathBuf> {
    let name = relative_source
        .file_name()
        .ok_or_else(|| format!("path has no file name: {}", relative_source.display()))?;
    let mut json_name = OsString::from(name);
    json_name.push(".json");
    Ok(relative_source.with_file_name(json_name))
}

fn normalized_relative(path: &Path) -> Result<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => parts.push(
                value
                    .to_str()
                    .ok_or_else(|| format!("path is not valid Unicode: {}", path.display()))?,
            ),
            _ => {
                return Err(format!(
                    "expected a safe relative path, got {}",
                    path.display()
                ));
            }
        }
    }
    Ok(parts.join("/"))
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>> {
    fn visit(root: &Path, current: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        let mut entries = fs::read_dir(current)
            .map_err(|error| format!("{}: {error}", current.display()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| format!("{}: {error}", current.display()))?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let metadata = entry
                .metadata()
                .map_err(|error| format!("{}: {error}", path.display()))?;
            if metadata.is_dir() {
                visit(root, &path, files)?;
            } else if metadata.is_file() {
                files.push(
                    path.strip_prefix(root)
                        .map_err(|error| format!("{}: {error}", path.display()))?
                        .to_owned(),
                );
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    visit(root, root, &mut files)?;
    Ok(files)
}

fn json_bytes(entries: &[TranslationEntry]) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(entries)
        .map_err(|error| format!("failed to serialize translation JSON: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn read_json(path: &Path) -> Result<Vec<TranslationEntry>> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
}

fn ensure_new_output(output: &Path) -> Result<()> {
    if output.exists() {
        return Err(format!(
            "output already exists; refusing to overwrite: {}",
            output.display()
        ));
    }
    Ok(())
}

fn safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn atomic_write_file(output: &Path, bytes: &[u8]) -> Result<()> {
    ensure_new_output(output)?;
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    let mut temporary_name = output
        .file_name()
        .ok_or_else(|| format!("path has no file name: {}", output.display()))?
        .to_os_string();
    temporary_name.push(format!(".tmp-{}", std::process::id()));
    let temporary = output.with_file_name(temporary_name);
    if temporary.exists() {
        return Err(format!(
            "temporary output already exists: {}",
            temporary.display()
        ));
    }
    if let Err(error) = fs::write(&temporary, bytes) {
        return Err(format!("{}: {error}", temporary.display()));
    }
    if let Err(error) = fs::rename(&temporary, output) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("failed to finalize {}: {error}", output.display()));
    }
    Ok(())
}

fn atomic_write_tree(output: &Path, files: &[(PathBuf, Vec<u8>)]) -> Result<()> {
    ensure_new_output(output)?;
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    let mut temporary_name = output
        .file_name()
        .ok_or_else(|| format!("path has no file name: {}", output.display()))?
        .to_os_string();
    temporary_name.push(format!(".tmp-{}", std::process::id()));
    let temporary = output.with_file_name(temporary_name);
    if temporary.exists() {
        return Err(format!(
            "temporary output already exists: {}",
            temporary.display()
        ));
    }

    let result = (|| -> Result<()> {
        fs::create_dir(&temporary).map_err(|error| format!("{}: {error}", temporary.display()))?;
        for (relative, bytes) in files {
            if !safe_relative(relative) {
                return Err(format!("unsafe output path: {}", relative.display()));
            }
            let destination = temporary.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("{}: {error}", parent.display()))?;
            }
            fs::write(&destination, bytes)
                .map_err(|error| format!("{}: {error}", destination.display()))?;
        }
        fs::rename(&temporary, output)
            .map_err(|error| format!("failed to finalize {}: {error}", output.display()))?;
        Ok(())
    })();

    if result.is_err() && temporary.exists() {
        let _ = fs::remove_dir_all(&temporary);
    }
    result
}

fn extract_one(data: &[u8], source_file: &str) -> Result<(Vec<TranslationEntry>, usize)> {
    let script = adv::parse(data)?;
    let entries = text::extract_entries(&script, source_file)?;
    let skipped = script.texts.len() - entries.len();
    Ok((entries, skipped))
}

pub fn extract(input: &Path, output: Option<&Path>) -> Result<ExtractReport> {
    if !input.exists() {
        return Err(format!("input does not exist: {}", input.display()));
    }

    if input.is_file() {
        if !is_adv(input) {
            return Err(format!("expected an .ADV file: {}", input.display()));
        }
        let output = output
            .map(Path::to_owned)
            .unwrap_or(append_to_name(input, ".json")?);
        let data = fs::read(input).map_err(|error| format!("{}: {error}", input.display()))?;
        let source_file = input
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("file name is not valid Unicode: {}", input.display()))?;
        let (entries, skipped) = extract_one(&data, source_file)
            .map_err(|error| format!("{}: {error}", input.display()))?;
        atomic_write_file(&output, &json_bytes(&entries)?)?;
        return Ok(ExtractReport {
            scanned_files: 1,
            json_files: 1,
            extracted_entries: entries.len(),
            skipped,
            warnings: 0,
            output,
        });
    }

    let output = output
        .map(Path::to_owned)
        .unwrap_or(append_to_name(input, "_json")?);
    ensure_new_output(&output)?;
    let source_files = collect_files(input)?;
    let mut prepared = Vec::new();
    let mut report = ExtractReport {
        output: output.clone(),
        ..ExtractReport::default()
    };
    for relative in source_files.into_iter().filter(|path| is_adv(path)) {
        let source = input.join(&relative);
        let data = fs::read(&source).map_err(|error| format!("{}: {error}", source.display()))?;
        let source_file = normalized_relative(&relative)?;
        let (entries, skipped) = extract_one(&data, &source_file)
            .map_err(|error| format!("{}: {error}", source.display()))?;
        prepared.push((relative_json_path(&relative)?, json_bytes(&entries)?));
        report.scanned_files += 1;
        report.json_files += 1;
        report.extracted_entries += entries.len();
        report.skipped += skipped;
    }
    if report.scanned_files == 0 {
        return Err(format!("no .ADV files found under {}", input.display()));
    }
    atomic_write_tree(&output, &prepared)?;
    Ok(report)
}

fn inject_one(
    source_data: &[u8],
    entries: &[TranslationEntry],
    source_file: &str,
) -> Result<(Vec<u8>, usize, usize)> {
    let script = adv::parse(source_data)?;
    let expected = text::extract_entries(&script, source_file)?;
    if entries.len() != expected.len() {
        return Err(format!(
            "JSON has {} entries, source ADV has {} extractable entries",
            entries.len(),
            expected.len()
        ));
    }

    let text_by_offset = script
        .texts
        .iter()
        .map(|operand| (operand.instruction_offset, operand))
        .collect::<BTreeMap<_, _>>();
    let mut replacements = BTreeMap::new();
    let mut patched = 0usize;
    let mut unchanged = 0usize;
    for (index, (entry, expected_entry)) in entries.iter().zip(&expected).enumerate() {
        if entry._index != expected_entry._index {
            return Err(format!(
                "JSON entry at array position {index} has _index {}, expected {}",
                entry._index, expected_entry._index
            ));
        }
        if entry._inst_offset != expected_entry._inst_offset {
            return Err(format!(
                "entry {index}: _inst_offset is 0x{:04X}, expected 0x{:04X}",
                entry._inst_offset, expected_entry._inst_offset
            ));
        }
        let operand = text_by_offset.get(&entry._inst_offset).ok_or_else(|| {
            format!(
                "entry {index}: no X instruction at 0x{:04X}",
                entry._inst_offset
            )
        })?;
        let plaintext = text::rebuild_plaintext(operand, entry, source_file)?;
        if plaintext == operand.plaintext {
            unchanged += 1;
        } else {
            patched += 1;
        }
        replacements.insert(entry._inst_offset, plaintext);
    }
    let output = adv::rebuild(source_data, &script, &replacements)?;
    Ok((output, patched, unchanged))
}

pub fn inject(source: &Path, translations: &Path, output: Option<&Path>) -> Result<InjectReport> {
    if !source.exists() {
        return Err(format!("source does not exist: {}", source.display()));
    }
    if !translations.exists() {
        return Err(format!(
            "translation input does not exist: {}",
            translations.display()
        ));
    }

    if source.is_file() {
        if !translations.is_file() || !is_adv(source) {
            return Err("single-file injection requires SOURCE.ADV and SOURCE.ADV.json".to_owned());
        }
        let output = output.map(Path::to_owned).unwrap_or(injected_name(source)?);
        let source_data =
            fs::read(source).map_err(|error| format!("{}: {error}", source.display()))?;
        let entries = read_json(translations)?;
        let source_file = source
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("file name is not valid Unicode: {}", source.display()))?;
        let (rebuilt, patched, unchanged) = inject_one(&source_data, &entries, source_file)
            .map_err(|error| format!("{}: {error}", source.display()))?;
        atomic_write_file(&output, &rebuilt)?;
        return Ok(InjectReport {
            json_entries: entries.len(),
            patched,
            unchanged,
            failed: 0,
            warnings: 0,
            copied_files: 1,
            output,
        });
    }

    if !translations.is_dir() {
        return Err(
            "directory injection requires a source directory and JSON directory".to_owned(),
        );
    }
    let output = output.map(Path::to_owned).unwrap_or(injected_name(source)?);
    ensure_new_output(&output)?;

    let source_files = collect_files(source)?;
    let translation_files = collect_files(translations)?;
    let mut json_sources = BTreeMap::<PathBuf, PathBuf>::new();
    for relative_json in translation_files {
        if !relative_json
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        {
            continue;
        }
        let file_name = relative_json
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                format!(
                    "JSON file name is not valid Unicode: {}",
                    relative_json.display()
                )
            })?;
        let source_name = file_name
            .strip_suffix(".json")
            .ok_or_else(|| format!("invalid JSON name: {}", relative_json.display()))?;
        let relative_source = relative_json.with_file_name(source_name);
        if !is_adv(&relative_source) {
            return Err(format!(
                "translation JSON does not map to an .ADV file: {}",
                relative_json.display()
            ));
        }
        if json_sources
            .insert(relative_source.clone(), relative_json.clone())
            .is_some()
        {
            return Err(format!(
                "duplicate translation mapping for {}",
                relative_source.display()
            ));
        }
    }

    let source_set = source_files.iter().cloned().collect::<BTreeSet<_>>();
    for relative_source in json_sources.keys() {
        if !source_set.contains(relative_source) {
            return Err(format!(
                "translation has no matching source file: {}",
                relative_source.display()
            ));
        }
    }

    let mut prepared = Vec::with_capacity(source_files.len());
    let mut report = InjectReport {
        output: output.clone(),
        ..InjectReport::default()
    };
    for relative in source_files {
        let source_path = source.join(&relative);
        let source_data = fs::read(&source_path)
            .map_err(|error| format!("{}: {error}", source_path.display()))?;
        let output_data = if let Some(relative_json) = json_sources.get(&relative) {
            let json_path = translations.join(relative_json);
            let entries = read_json(&json_path)?;
            let source_file = normalized_relative(&relative)?;
            let (rebuilt, patched, unchanged) = inject_one(&source_data, &entries, &source_file)
                .map_err(|error| format!("{}: {error}", source_path.display()))?;
            report.json_entries += entries.len();
            report.patched += patched;
            report.unchanged += unchanged;
            rebuilt
        } else {
            source_data
        };
        prepared.push((relative, output_data));
        report.copied_files += 1;
    }
    atomic_write_tree(&output, &prepared)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn output_names_do_not_overwrite_source() {
        assert_eq!(
            append_to_name(Path::new("A01.ADV"), ".json").unwrap(),
            PathBuf::from("A01.ADV.json")
        );
        assert_eq!(
            injected_name(Path::new("A01.ADV")).unwrap(),
            PathBuf::from("A01_injected.ADV")
        );
    }

    #[test]
    fn existing_output_is_never_overwritten() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("A01.ADV");
        let output = directory.path().join("A01.ADV.json");
        let mut bytes = vec![adv::TEXT_OPCODE];
        bytes.extend_from_slice(&adv::encode_encrypted(adv::TEXT_OPCODE, b"text").unwrap());
        bytes.push(0xFF);
        fs::write(&source, bytes).unwrap();
        fs::write(&output, b"keep").unwrap();

        let error = extract(&source, Some(&output)).unwrap_err();
        assert!(error.contains("refusing to overwrite"));
        assert_eq!(fs::read(output).unwrap(), b"keep");
    }
}
