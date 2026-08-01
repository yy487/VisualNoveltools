use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use crate::encoding::decode_cp932;
use crate::script::parse_script;
use crate::text_json::{
    InjectCounts, deserialize_entries, entries_from_script, inject_script, serialize_entries,
};
use crate::{ToolResult, error};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExtractStats {
    pub scanned_files: usize,
    pub json_files: usize,
    pub extracted_entries: usize,
    pub warnings: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InjectStats {
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub warnings: usize,
}

struct PreparedFile {
    relative: PathBuf,
    bytes: Vec<u8>,
}

pub fn default_extract_output(source: &Path) -> ToolResult<PathBuf> {
    if source.is_file() {
        let name = source
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| error("input file name is not valid Unicode"))?;
        Ok(source.with_file_name(format!("{name}.json")))
    } else {
        sibling_with_suffix(source, "_json")
    }
}

pub fn default_inject_output(source: &Path) -> ToolResult<PathBuf> {
    if source.is_file() {
        let stem = source
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| error("input file stem is not valid Unicode"))?;
        let extension = source.extension().and_then(OsStr::to_str).unwrap_or("");
        let name = if extension.is_empty() {
            format!("{stem}_injected")
        } else {
            format!("{stem}_injected.{extension}")
        };
        Ok(source.with_file_name(name))
    } else {
        sibling_with_suffix(source, "_injected")
    }
}

pub fn extract(source: &Path, output: &Path) -> ToolResult<ExtractStats> {
    validate_source(source)?;
    reject_existing_output(output)?;

    if source.is_file() {
        require_txt(source)?;
        let relative = file_name_path(source)?;
        let relative_json = append_json_extension(&relative)?;
        let prepared = prepare_extraction(source, &relative, &relative_json)?;
        write_new_file(output, &prepared.bytes)?;
        return Ok(ExtractStats {
            scanned_files: 1,
            json_files: 1,
            extracted_entries: count_json_entries(&prepared.bytes)?,
            warnings: 0,
        });
    }

    reject_nested_output(source, output)?;
    let all_files = collect_files(source)?;
    let mut scanned = 0usize;
    let mut entries = 0usize;
    let mut prepared = Vec::new();
    for relative in all_files.iter().filter(|path| is_txt(path)) {
        scanned += 1;
        let source_file = source.join(relative);
        let relative_json = append_json_extension(relative)?;
        let item = prepare_extraction(&source_file, relative, &relative_json)?;
        let item_entries = count_json_entries(&item.bytes)?;
        if item_entries > 0 {
            entries += item_entries;
            prepared.push(item);
        }
    }

    create_directory_transaction(output, |created| {
        for item in &prepared {
            let destination = created.join(&item.relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|source_error| {
                    error(format!("{}: {source_error}", parent.display()))
                })?;
            }
            fs::write(&destination, &item.bytes).map_err(|source_error| {
                error(format!("{}: {source_error}", destination.display()))
            })?;
        }
        Ok(())
    })?;

    Ok(ExtractStats {
        scanned_files: scanned,
        json_files: prepared.len(),
        extracted_entries: entries,
        warnings: 0,
    })
}

pub fn inject(source: &Path, translations: &Path, output: &Path) -> ToolResult<InjectStats> {
    validate_source(source)?;
    validate_source(translations)?;
    reject_existing_output(output)?;

    if source.is_file() {
        require_txt(source)?;
        if !translations.is_file() {
            return Err(error("file injection requires a JSON file"));
        }
        let relative = file_name_path(source)?;
        let (bytes, counts) = prepare_injection(source, translations, &relative)?;
        write_new_file(output, &bytes)?;
        return Ok(inject_stats(counts));
    }

    if !translations.is_dir() {
        return Err(error("directory injection requires a JSON directory"));
    }
    reject_nested_output(source, output)?;

    let mut patches = Vec::new();
    let mut totals = InjectCounts::default();
    for json_relative in collect_files(translations)?
        .into_iter()
        .filter(|path| is_json(path))
    {
        let source_relative = remove_json_extension(&json_relative)?;
        require_txt(&source_relative)?;
        let source_file = source.join(&source_relative);
        if !source_file.is_file() {
            return Err(error(format!(
                "{}: matching source file does not exist: {}",
                translations.join(&json_relative).display(),
                source_file.display()
            )));
        }
        let (bytes, counts) = prepare_injection(
            &source_file,
            &translations.join(&json_relative),
            &source_relative,
        )?;
        totals.json_entries += counts.json_entries;
        totals.patched += counts.patched;
        totals.unchanged += counts.unchanged;
        patches.push(PreparedFile {
            relative: source_relative,
            bytes,
        });
    }
    if patches.is_empty() {
        return Err(error(format!(
            "{}: no translation JSON files found",
            translations.display()
        )));
    }

    create_directory_transaction(output, |created| {
        copy_tree(source, created)?;
        for patch in &patches {
            let destination = created.join(&patch.relative);
            fs::write(&destination, &patch.bytes).map_err(|source_error| {
                error(format!("{}: {source_error}", destination.display()))
            })?;
        }
        Ok(())
    })?;
    Ok(inject_stats(totals))
}

fn prepare_extraction(
    source: &Path,
    relative: &Path,
    relative_json: &Path,
) -> ToolResult<PreparedFile> {
    let context = source.display().to_string();
    let source_bytes =
        fs::read(source).map_err(|source_error| error(format!("{context}: {source_error}")))?;
    let text = decode_cp932(&source_bytes, &context)?;
    let parsed =
        parse_script(text).map_err(|source_error| error(format!("{context}: {source_error}")))?;
    let relative_string = relative_path_string(relative)?;
    let entries = entries_from_script(&parsed, &relative_string);
    Ok(PreparedFile {
        relative: relative_json.to_owned(),
        bytes: serialize_entries(&entries)?,
    })
}

fn prepare_injection(
    source: &Path,
    json: &Path,
    relative: &Path,
) -> ToolResult<(Vec<u8>, InjectCounts)> {
    let source_context = source.display().to_string();
    let source_bytes = fs::read(source)
        .map_err(|source_error| error(format!("{source_context}: {source_error}")))?;
    let text = decode_cp932(&source_bytes, &source_context)?;
    let parsed = parse_script(text)
        .map_err(|source_error| error(format!("{source_context}: {source_error}")))?;
    let json_context = json.display().to_string();
    let json_bytes =
        fs::read(json).map_err(|source_error| error(format!("{json_context}: {source_error}")))?;
    let entries = deserialize_entries(&json_bytes, &json_context)?;
    inject_script(parsed, &entries, &relative_path_string(relative)?)
}

fn inject_stats(counts: InjectCounts) -> InjectStats {
    InjectStats {
        json_entries: counts.json_entries,
        patched: counts.patched,
        unchanged: counts.unchanged,
        failed: 0,
        warnings: 0,
    }
}

fn count_json_entries(bytes: &[u8]) -> ToolResult<usize> {
    let value: Vec<serde_json::Value> = serde_json::from_slice(bytes)?;
    Ok(value.len())
}

fn validate_source(path: &Path) -> ToolResult<()> {
    if !path.exists() {
        return Err(error(format!("path does not exist: {}", path.display())));
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|source_error| error(format!("{}: {source_error}", path.display())))?;
    if metadata.file_type().is_symlink() {
        return Err(error(format!(
            "symlinks are not supported: {}",
            path.display()
        )));
    }
    if !metadata.is_file() && !metadata.is_dir() {
        return Err(error(format!("unsupported path type: {}", path.display())));
    }
    Ok(())
}

fn require_txt(path: &Path) -> ToolResult<()> {
    if !is_txt(path) {
        return Err(error(format!("expected a .txt script: {}", path.display())));
    }
    Ok(())
}

fn is_txt(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("txt"))
}

fn is_json(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
}

fn append_json_extension(path: &Path) -> ToolResult<PathBuf> {
    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| error(format!("invalid Unicode file name: {}", path.display())))?;
    Ok(path.with_file_name(format!("{name}.json")))
}

fn remove_json_extension(path: &Path) -> ToolResult<PathBuf> {
    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| error(format!("invalid Unicode file name: {}", path.display())))?;
    let Some(source_name) = name
        .strip_suffix(".json")
        .or_else(|| name.strip_suffix(".JSON"))
    else {
        return Err(error(format!("expected .json file: {}", path.display())));
    };
    Ok(path.with_file_name(source_name))
}

fn file_name_path(path: &Path) -> ToolResult<PathBuf> {
    path.file_name()
        .map(PathBuf::from)
        .ok_or_else(|| error(format!("path has no file name: {}", path.display())))
}

fn sibling_with_suffix(path: &Path, suffix: &str) -> ToolResult<PathBuf> {
    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| error(format!("path has no Unicode file name: {}", path.display())))?;
    Ok(path.with_file_name(format!("{name}{suffix}")))
}

fn reject_existing_output(output: &Path) -> ToolResult<()> {
    if output.exists() {
        return Err(error(format!(
            "output already exists; refusing to overwrite: {}",
            output.display()
        )));
    }
    Ok(())
}

fn reject_nested_output(source: &Path, output: &Path) -> ToolResult<()> {
    let source = absolute_normalized(source)?;
    let output = absolute_normalized(output)?;
    let source_key = source.to_string_lossy().to_lowercase();
    let output_key = output.to_string_lossy().to_lowercase();
    let separator = std::path::MAIN_SEPARATOR;
    let prefix = format!("{source_key}{separator}");
    if output_key == source_key || output_key.starts_with(&prefix) {
        return Err(error(format!(
            "output must not be inside the source directory: {}",
            output.display()
        )));
    }
    Ok(())
}

fn absolute_normalized(path: &Path) -> ToolResult<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut result = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            other => result.push(other.as_os_str()),
        }
    }
    Ok(result)
}

fn collect_files(root: &Path) -> ToolResult<Vec<PathBuf>> {
    let mut result = Vec::new();
    collect_files_inner(root, Path::new(""), &mut result)?;
    result.sort_by_key(|path| {
        relative_path_string(path).unwrap_or_else(|_| path.display().to_string())
    });
    Ok(result)
}

fn collect_files_inner(root: &Path, relative: &Path, output: &mut Vec<PathBuf>) -> ToolResult<()> {
    let directory = root.join(relative);
    let mut entries = fs::read_dir(&directory)
        .map_err(|source_error| error(format!("{}: {source_error}", directory.display())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source_error| error(format!("{}: {source_error}", directory.display())))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let child_relative = relative.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|source_error| error(format!("{}: {source_error}", entry.path().display())))?;
        if file_type.is_symlink() {
            return Err(error(format!(
                "symlinks are not supported: {}",
                entry.path().display()
            )));
        }
        if file_type.is_dir() {
            collect_files_inner(root, &child_relative, output)?;
        } else if file_type.is_file() {
            output.push(child_relative);
        }
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> ToolResult<()> {
    fs::create_dir_all(destination)
        .map_err(|source_error| error(format!("{}: {source_error}", destination.display())))?;
    copy_tree_inner(source, destination)
}

fn copy_tree_inner(source: &Path, destination: &Path) -> ToolResult<()> {
    let mut entries = fs::read_dir(source)
        .map_err(|source_error| error(format!("{}: {source_error}", source.display())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source_error| error(format!("{}: {source_error}", source.display())))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|source_error| error(format!("{}: {source_error}", source_path.display())))?;
        if file_type.is_symlink() {
            return Err(error(format!(
                "symlinks are not supported: {}",
                source_path.display()
            )));
        }
        if file_type.is_dir() {
            fs::create_dir_all(&destination_path).map_err(|source_error| {
                error(format!("{}: {source_error}", destination_path.display()))
            })?;
            copy_tree_inner(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &destination_path).map_err(|source_error| {
                error(format!("{}: {source_error}", destination_path.display()))
            })?;
        }
    }
    Ok(())
}

fn create_directory_transaction<F>(output: &Path, operation: F) -> ToolResult<()>
where
    F: FnOnce(&Path) -> ToolResult<()>,
{
    fs::create_dir_all(output)
        .map_err(|source_error| error(format!("{}: {source_error}", output.display())))?;
    if let Err(operation_error) = operation(output) {
        let cleanup = fs::remove_dir_all(output);
        if let Err(cleanup_error) = cleanup {
            return Err(error(format!(
                "{operation_error}; also failed to remove incomplete output {}: {cleanup_error}",
                output.display()
            )));
        }
        return Err(operation_error);
    }
    Ok(())
}

fn write_new_file(output: &Path, bytes: &[u8]) -> ToolResult<()> {
    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .map_err(|source_error| error(format!("{}: {source_error}", parent.display())))?;
    }
    let result = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .and_then(|mut file| file.write_all(bytes));
    if let Err(source_error) = result {
        let _ = fs::remove_file(output);
        return Err(error(format!("{}: {source_error}", output.display())));
    }
    Ok(())
}

fn relative_path_string(path: &Path) -> ToolResult<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => {
                parts.push(value.to_str().ok_or_else(|| {
                    error(format!("path is not valid Unicode: {}", path.display()))
                })?)
            }
            _ => {
                return Err(error(format!(
                    "expected a safe relative path: {}",
                    path.display()
                )));
            }
        }
    }
    Ok(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::encoding::encode_cp932;

    use super::*;

    fn temp_path(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "ianzhong_mes_text_{label}_{}_{}",
            std::process::id(),
            nonce
        ))
    }

    #[test]
    fn directory_no_change_round_trip_copies_all_files() {
        let root = temp_path("roundtrip");
        let source = root.join("源 & scripts");
        let json = root.join("json");
        let output = root.join("injected");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(source.join("empty")).unwrap();
        let script = encode_cp932("本文[n]続き\n【文太郎,S000】「台詞」\n", "test").unwrap();
        fs::write(source.join("場面.txt"), &script).unwrap();
        fs::write(source.join("manifest.json"), b"{}\n").unwrap();

        let extract_stats = extract(&source, &json).unwrap();
        assert_eq!(extract_stats.extracted_entries, 2);
        let inject_stats = inject(&source, &json, &output).unwrap();
        assert_eq!(inject_stats.unchanged, 2);
        assert_eq!(fs::read(output.join("場面.txt")).unwrap(), script);
        assert_eq!(fs::read(output.join("manifest.json")).unwrap(), b"{}\n");
        assert!(output.join("empty").is_dir());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_existing_output() {
        let root = temp_path("existing");
        fs::create_dir_all(&root).unwrap();
        let source = root.join("scene.txt");
        let output = root.join("scene.txt.json");
        fs::write(&source, encode_cp932("本文\n", "test").unwrap()).unwrap();
        fs::write(&output, b"existing").unwrap();
        let result = extract(&source, &output);
        assert!(result.unwrap_err().0.contains("refusing to overwrite"));
        fs::remove_dir_all(root).unwrap();
    }
}
