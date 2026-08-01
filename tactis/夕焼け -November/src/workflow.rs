use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::model::ScriptJson;
use crate::script::{extract_document, inject_document};

#[derive(Debug, Default)]
pub struct ExtractReport {
    pub scanned_files: usize,
    pub json_files: usize,
    pub extracted_entries: usize,
    pub skipped_empty_pages: usize,
    pub warnings: usize,
    pub output: PathBuf,
}

#[derive(Debug, Default)]
pub struct InjectReport {
    pub json_files: usize,
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub warnings: usize,
    pub output: PathBuf,
}

pub fn default_extract_output(input: &Path) -> Result<PathBuf> {
    if input.is_dir() {
        append_name_suffix(input, "_json")
    } else {
        let mut name = input
            .file_name()
            .context("input file has no file name")?
            .to_os_string();
        name.push(".json");
        Ok(input.with_file_name(name))
    }
}

pub fn default_inject_output(source: &Path) -> Result<PathBuf> {
    if source.is_dir() {
        append_name_suffix(source, "_injected")
    } else {
        let stem = source
            .file_stem()
            .context("source file has no stem")?
            .to_os_string();
        let mut name = stem;
        name.push("_injected");
        if let Some(extension) = source.extension() {
            name.push(".");
            name.push(extension);
        }
        Ok(source.with_file_name(name))
    }
}

pub fn extract_path(input: &Path, output: &Path) -> Result<ExtractReport> {
    ensure_input(input)?;
    ensure_new_output(output)?;

    if input.is_file() {
        let bytes =
            fs::read(input).with_context(|| format!("failed to read input {}", input.display()))?;
        let relative = file_name_utf8(input)?;
        let (document, empty_pages) = extract_document(&bytes, &relative)
            .with_context(|| format!("failed to parse {}", input.display()))?;
        let json = serialize_document(&document)?;
        write_new_file(output, &json)?;
        return Ok(ExtractReport {
            scanned_files: 1,
            json_files: 1,
            extracted_entries: document.entries.len(),
            skipped_empty_pages: empty_pages,
            warnings: 0,
            output: output.to_owned(),
        });
    }

    let files = collect_files(input)?;
    let mut prepared = Vec::new();
    let mut report = ExtractReport {
        scanned_files: files.len(),
        output: output.to_owned(),
        ..ExtractReport::default()
    };

    for source_path in files {
        let relative_path = source_path
            .strip_prefix(input)
            .expect("collected path is below input");
        let relative = json_relative_path(relative_path)?;
        let bytes = fs::read(&source_path)
            .with_context(|| format!("failed to read input {}", source_path.display()))?;
        let (document, empty_pages) = extract_document(&bytes, &relative)
            .with_context(|| format!("failed to parse {}", source_path.display()))?;
        report.skipped_empty_pages += empty_pages;
        if document.entries.is_empty() {
            continue;
        }
        report.extracted_entries += document.entries.len();
        report.json_files += 1;
        let json = serialize_document(&document)?;
        let mut json_relative = relative_path.as_os_str().to_os_string();
        json_relative.push(".json");
        prepared.push((PathBuf::from(json_relative), json));
    }

    fs::create_dir(output)
        .with_context(|| format!("failed to create output directory {}", output.display()))?;
    for (relative, json) in prepared {
        let destination = output.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create output directory {}", parent.display())
            })?;
        }
        write_new_file(&destination, &json)?;
    }
    Ok(report)
}

pub fn inject_path(source: &Path, translations: &Path, output: &Path) -> Result<InjectReport> {
    ensure_input(source)?;
    ensure_input(translations)?;
    ensure_new_output(output)?;

    if source.is_file() != translations.is_file() {
        bail!("source and translations must both be files or both be directories");
    }

    if source.is_file() {
        let bytes = fs::read(source)
            .with_context(|| format!("failed to read source {}", source.display()))?;
        let document = read_document(translations)?;
        let relative = file_name_utf8(source)?;
        let (rebuilt, stats) = inject_document(&bytes, &relative, &document)
            .with_context(|| format!("failed to inject {}", source.display()))?;
        write_new_file(output, &rebuilt)?;
        return Ok(InjectReport {
            json_files: 1,
            json_entries: stats.entries,
            patched: stats.patched,
            unchanged: stats.unchanged,
            failed: 0,
            warnings: 0,
            output: output.to_owned(),
        });
    }

    let source_files = collect_files(source)?;
    let json_files = collect_files(translations)?;
    if json_files.is_empty() {
        bail!("translation directory contains no JSON files");
    }

    let source_relatives: BTreeSet<String> = source_files
        .iter()
        .map(|path| {
            let relative = path.strip_prefix(source).expect("source child");
            json_relative_path(relative)
        })
        .collect::<Result<_>>()?;

    let mut documents = BTreeMap::new();
    for json_path in &json_files {
        if json_path.extension().and_then(|value| value.to_str()) != Some("json") {
            bail!(
                "translation directory contains a non-JSON file: {}",
                json_path.display()
            );
        }
        let document = read_document(json_path)?;
        if !source_relatives.contains(&document.file) {
            bail!(
                "{} references missing source file {:?}",
                json_path.display(),
                document.file
            );
        }
        if documents.insert(document.file.clone(), document).is_some() {
            bail!("duplicate translation JSON for a source file");
        }
    }

    let mut prepared = Vec::with_capacity(source_files.len());
    let mut report = InjectReport {
        json_files: documents.len(),
        output: output.to_owned(),
        ..InjectReport::default()
    };

    for source_path in source_files {
        let relative_path = source_path
            .strip_prefix(source)
            .expect("collected path is below source");
        let relative = json_relative_path(relative_path)?;
        let bytes = fs::read(&source_path)
            .with_context(|| format!("failed to read source {}", source_path.display()))?;
        if let Some(document) = documents.get(&relative) {
            let (rebuilt, stats) = inject_document(&bytes, &relative, document)
                .with_context(|| format!("failed to inject {}", source_path.display()))?;
            report.json_entries += stats.entries;
            report.patched += stats.patched;
            report.unchanged += stats.unchanged;
            prepared.push((relative_path.to_owned(), rebuilt));
        } else {
            prepared.push((relative_path.to_owned(), bytes));
        }
    }

    fs::create_dir(output)
        .with_context(|| format!("failed to create output directory {}", output.display()))?;
    for (relative, bytes) in prepared {
        let destination = output.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create output directory {}", parent.display())
            })?;
        }
        write_new_file(&destination, &bytes)?;
    }
    Ok(report)
}

fn ensure_input(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("input does not exist: {}", path.display());
    }
    Ok(())
}

fn ensure_new_output(path: &Path) -> Result<()> {
    if path.exists() {
        bail!("output already exists: {}", path.display());
    }
    Ok(())
}

fn append_name_suffix(path: &Path, suffix: &str) -> Result<PathBuf> {
    let mut name: OsString = path
        .file_name()
        .context("path has no final component")?
        .to_os_string();
    name.push(suffix);
    Ok(path.with_file_name(name))
}

fn file_name_utf8(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(str::to_owned)
        .with_context(|| format!("file name is not valid Unicode: {}", path.display()))
}

fn json_relative_path(path: &Path) -> Result<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => parts.push(
                value
                    .to_str()
                    .with_context(|| format!("path is not valid Unicode: {}", path.display()))?,
            ),
            _ => bail!("unsafe relative path: {}", path.display()),
        }
    }
    if parts.is_empty() {
        bail!("empty relative path");
    }
    Ok(parts.join("/"))
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut output = Vec::new();
    collect_files_recursive(root, &mut output)?;
    output.sort();
    Ok(output)
}

fn collect_files_recursive(path: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(path)
        .with_context(|| format!("failed to read directory {}", path.display()))?
        .collect::<std::io::Result<_>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_type = entry.file_type()?;
        let child = entry.path();
        if file_type.is_symlink() {
            bail!("symbolic links are not supported: {}", child.display());
        }
        if file_type.is_dir() {
            collect_files_recursive(&child, output)?;
        } else if file_type.is_file() {
            output.push(child);
        }
    }
    Ok(())
}

fn serialize_document(document: &ScriptJson) -> Result<Vec<u8>> {
    let mut json = serde_json::to_string_pretty(document)?.into_bytes();
    json.push(b'\n');
    Ok(json)
}

fn read_document(path: &Path) -> Result<ScriptJson> {
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read translation JSON {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid translation JSON {}", path.display()))
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;

    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options
        .open(path)
        .with_context(|| format!("failed to create output file {}", path.display()))?;
    file.write_all(bytes)
        .with_context(|| format!("failed to write output file {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_ampersand_paths_are_supported() {
        let temporary = tempfile::tempdir().unwrap();
        let source = temporary.path().join("日本語 & test");
        fs::write(&source, [0xFF]).unwrap();
        let output = default_extract_output(&source).unwrap();
        let report = extract_path(&source, &output).unwrap();
        assert_eq!(report.scanned_files, 1);
        assert!(output.exists());
    }

    #[test]
    fn existing_outputs_are_rejected() {
        let temporary = tempfile::tempdir().unwrap();
        let source = temporary.path().join("source");
        let output = temporary.path().join("output.json");
        fs::write(&source, [0xFF]).unwrap();
        fs::write(&output, b"existing").unwrap();
        let error = extract_path(&source, &output).unwrap_err();
        assert!(error.to_string().contains("already exists"));
    }
}
