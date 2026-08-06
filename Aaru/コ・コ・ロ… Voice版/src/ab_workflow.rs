use crate::ab::parse_script;
use crate::ab_text::{
    deserialize_entries, extract_entries, inject_entries, serialize_entries, TranslationEntry,
};
use crate::{ToolError, ToolResult};
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
pub struct ExtractReport {
    pub input: PathBuf,
    pub output: PathBuf,
    pub scanned_files: usize,
    pub json_files: usize,
    pub extracted_entries: usize,
    pub warnings: usize,
}

#[derive(Debug)]
pub struct InjectReport {
    pub source: PathBuf,
    pub json: PathBuf,
    pub output: PathBuf,
    pub json_files: usize,
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub warnings: usize,
}

#[derive(Debug)]
struct PreparedFile {
    relative: PathBuf,
    bytes: Vec<u8>,
}

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(expected))
}

fn collect_files(root: &Path, extension: &str) -> ToolResult<Vec<PathBuf>> {
    fn visit(
        root: &Path,
        directory: &Path,
        extension: &str,
        output: &mut Vec<PathBuf>,
    ) -> ToolResult<()> {
        let mut entries = fs::read_dir(directory)
            .map_err(|error| ToolError(format!("cannot read '{}': {error}", directory.display())))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                ToolError(format!(
                    "cannot enumerate '{}': {error}",
                    directory.display()
                ))
            })?;
        entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_uppercase());
        for entry in entries {
            let path = entry.path();
            let file_type = entry.file_type().map_err(|error| {
                ToolError(format!("cannot inspect '{}': {error}", path.display()))
            })?;
            if file_type.is_symlink() {
                return Err(ToolError(format!(
                    "symbolic links are not supported in directory workflows: '{}'",
                    path.display()
                )));
            }
            if file_type.is_dir() {
                visit(root, &path, extension, output)?;
            } else if file_type.is_file() && has_extension(&path, extension) {
                output.push(
                    path.strip_prefix(root)
                        .map_err(|error| {
                            ToolError(format!(
                                "cannot make '{}' relative to '{}': {error}",
                                path.display(),
                                root.display()
                            ))
                        })?
                        .to_path_buf(),
                );
            }
        }
        Ok(())
    }

    let mut output = Vec::new();
    visit(root, root, extension, &mut output)?;
    output.sort_by_key(|path| {
        path.to_string_lossy()
            .replace('\\', "/")
            .to_ascii_uppercase()
    });
    Ok(output)
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .to_ascii_uppercase()
}

fn path_label(path: &Path) -> ToolResult<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => parts.push(value.to_str().ok_or_else(|| {
                ToolError(format!("path is not valid Unicode: '{}'", path.display()))
            })?),
            _ => {
                return Err(ToolError(format!(
                    "expected a relative file path, got '{}'",
                    path.display()
                )));
            }
        }
    }
    Ok(parts.join("/"))
}

fn append_json_extension(path: &Path) -> ToolResult<PathBuf> {
    let file_name = path
        .file_name()
        .ok_or_else(|| ToolError(format!("path has no file name: '{}'", path.display())))?;
    let mut json_name = OsString::from(file_name);
    json_name.push(".json");
    Ok(path.with_file_name(json_name))
}

fn default_directory_output(input: &Path, suffix: &str) -> PathBuf {
    let name = input
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "scripts".to_string());
    input.with_file_name(format!("{name}{suffix}"))
}

pub fn default_extract_output(input: &Path) -> ToolResult<PathBuf> {
    if input.is_file() {
        append_json_extension(input)
    } else {
        Ok(default_directory_output(input, "_json"))
    }
}

pub fn default_inject_output(source: &Path) -> PathBuf {
    if source.is_file() {
        let stem = source
            .file_stem()
            .map(|value| value.to_string_lossy().into_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "script".to_string());
        let extension = source
            .extension()
            .map(|value| value.to_string_lossy().into_owned());
        let file_name = match extension {
            Some(extension) => format!("{stem}_injected.{extension}"),
            None => format!("{stem}_injected"),
        };
        source.with_file_name(file_name)
    } else {
        default_directory_output(source, "_injected")
    }
}

fn normalized_absolute(path: &Path) -> ToolResult<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| ToolError(format!("cannot get current directory: {error}")))?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(ToolError(format!(
                        "cannot normalize output path '{}'",
                        path.display()
                    )));
                }
            }
        }
    }
    Ok(normalized)
}

fn ensure_separate_output(input: &Path, output: &Path, directory_input: bool) -> ToolResult<()> {
    let input = normalized_absolute(input)?;
    let output = normalized_absolute(output)?;
    let input_key = path_key(&input);
    let output_key = path_key(&output);
    if input_key == output_key {
        return Err(ToolError(format!(
            "output must not overwrite the source path '{}'",
            input.display()
        )));
    }
    if directory_input && output_key.starts_with(&format!("{input_key}/")) {
        return Err(ToolError(format!(
            "output directory '{}' must not be inside source directory '{}'",
            output.display(),
            input.display()
        )));
    }
    Ok(())
}

fn staging_path(output: &Path) -> ToolResult<PathBuf> {
    let file_name = output.file_name().ok_or_else(|| {
        ToolError(format!(
            "output path has no file name: '{}'",
            output.display()
        ))
    })?;
    let mut name = OsString::from(".");
    name.push(file_name);
    name.push(format!(".abtool-{}.tmp", std::process::id()));
    Ok(output.with_file_name(name))
}

fn remove_path(path: &Path) -> ToolResult<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|error| ToolError(format!("cannot remove '{}': {error}", path.display())))
    } else {
        fs::remove_file(path)
            .map_err(|error| ToolError(format!("cannot remove '{}': {error}", path.display())))
    }
}

fn prepare_staging(output: &Path, overwrite: bool) -> ToolResult<PathBuf> {
    if output.exists() && !overwrite {
        return Err(ToolError(format!(
            "refusing to overwrite existing output '{}'; pass --overwrite",
            output.display()
        )));
    }
    let staging = staging_path(output)?;
    if staging.exists() {
        return Err(ToolError(format!(
            "staging path already exists; remove it before retrying: '{}'",
            staging.display()
        )));
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| ToolError(format!("cannot create '{}': {error}", parent.display())))?;
    }
    Ok(staging)
}

fn commit_staging(staging: &Path, output: &Path, overwrite: bool) -> ToolResult<()> {
    if output.exists() {
        if !overwrite {
            return Err(ToolError(format!(
                "output appeared while processing: '{}'",
                output.display()
            )));
        }
        remove_path(output)?;
    }
    fs::rename(staging, output).map_err(|error| {
        ToolError(format!(
            "cannot move completed output '{}' to '{}': {error}",
            staging.display(),
            output.display()
        ))
    })
}

fn write_prepared_directory(staging: &Path, files: &[PreparedFile]) -> ToolResult<()> {
    fs::create_dir(staging)
        .map_err(|error| ToolError(format!("cannot create '{}': {error}", staging.display())))?;
    for file in files {
        let target = staging.join(&file.relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                ToolError(format!("cannot create '{}': {error}", parent.display()))
            })?;
        }
        fs::write(&target, &file.bytes)
            .map_err(|error| ToolError(format!("cannot write '{}': {error}", target.display())))?;
    }
    Ok(())
}

fn prepare_entries(source_path: &Path, source_file: &str) -> ToolResult<Vec<TranslationEntry>> {
    let data = fs::read(source_path)
        .map_err(|error| ToolError(format!("cannot read '{}': {error}", source_path.display())))?;
    let script = parse_script(&data)
        .map_err(|error| ToolError(format!("{}: {error}", source_path.display())))?;
    extract_entries(source_file, &data, &script)
        .map_err(|error| ToolError(format!("{}: {error}", source_path.display())))
}

pub fn extract_path(
    input: &Path,
    output_arg: Option<&Path>,
    overwrite: bool,
) -> ToolResult<ExtractReport> {
    if !input.exists() {
        return Err(ToolError(format!(
            "input does not exist: '{}'",
            input.display()
        )));
    }
    if input.is_file() && !has_extension(input, "ab") {
        return Err(ToolError(format!(
            "input file is not .AB: '{}'",
            input.display()
        )));
    }
    if !input.is_file() && !input.is_dir() {
        return Err(ToolError(format!(
            "input is not a file or directory: '{}'",
            input.display()
        )));
    }
    let output = output_arg
        .map(Path::to_path_buf)
        .unwrap_or(default_extract_output(input)?);
    ensure_separate_output(input, &output, input.is_dir())?;

    let mut prepared = Vec::new();
    let mut extracted_entries = 0usize;
    if input.is_file() {
        let relative =
            PathBuf::from(input.file_name().ok_or_else(|| {
                ToolError(format!("input has no file name: '{}'", input.display()))
            })?);
        let entries = prepare_entries(input, &path_label(&relative)?)?;
        extracted_entries += entries.len();
        prepared.push(PreparedFile {
            relative: PathBuf::new(),
            bytes: serialize_entries(&entries)?,
        });
    } else {
        let files = collect_files(input, "ab")?;
        if files.is_empty() {
            return Err(ToolError(format!(
                "directory contains no .AB files: '{}'",
                input.display()
            )));
        }
        for relative in files {
            let entries = prepare_entries(&input.join(&relative), &path_label(&relative)?)?;
            extracted_entries += entries.len();
            prepared.push(PreparedFile {
                relative: append_json_extension(&relative)?,
                bytes: serialize_entries(&entries)?,
            });
        }
    }

    let staging = prepare_staging(&output, overwrite)?;
    let write_result = if input.is_file() {
        fs::write(&staging, &prepared[0].bytes)
            .map_err(|error| ToolError(format!("cannot write '{}': {error}", staging.display())))
    } else {
        write_prepared_directory(&staging, &prepared)
    };
    if let Err(error) = write_result {
        let _ = remove_path(&staging);
        return Err(error);
    }
    if let Err(error) = commit_staging(&staging, &output, overwrite) {
        let _ = remove_path(&staging);
        return Err(error);
    }

    Ok(ExtractReport {
        input: input.to_path_buf(),
        output,
        scanned_files: prepared.len(),
        json_files: prepared.len(),
        extracted_entries,
        warnings: 0,
    })
}

fn copy_tree(source: &Path, target: &Path) -> ToolResult<()> {
    fs::create_dir(target)
        .map_err(|error| ToolError(format!("cannot create '{}': {error}", target.display())))?;
    let mut entries = fs::read_dir(source)
        .map_err(|error| ToolError(format!("cannot read '{}': {error}", source.display())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| ToolError(format!("cannot enumerate '{}': {error}", source.display())))?;
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_uppercase());
    for entry in entries {
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry.file_type().map_err(|error| {
            ToolError(format!(
                "cannot inspect '{}': {error}",
                source_path.display()
            ))
        })?;
        if file_type.is_symlink() {
            return Err(ToolError(format!(
                "symbolic links are not supported in directory workflows: '{}'",
                source_path.display()
            )));
        }
        if file_type.is_dir() {
            copy_tree(&source_path, &target_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &target_path).map_err(|error| {
                ToolError(format!(
                    "cannot copy '{}' to '{}': {error}",
                    source_path.display(),
                    target_path.display()
                ))
            })?;
        } else {
            return Err(ToolError(format!(
                "unsupported filesystem entry: '{}'",
                source_path.display()
            )));
        }
    }
    Ok(())
}

fn prepare_injected_file(
    source_path: &Path,
    json_path: &Path,
    source_file: &str,
) -> ToolResult<(PreparedFile, usize, usize, usize)> {
    let source_bytes = fs::read(source_path)
        .map_err(|error| ToolError(format!("cannot read '{}': {error}", source_path.display())))?;
    let json_bytes = fs::read(json_path)
        .map_err(|error| ToolError(format!("cannot read '{}': {error}", json_path.display())))?;
    let entries = deserialize_entries(&json_bytes, &format!("'{}'", json_path.display()))?;
    let injected = inject_entries(source_file, &source_bytes, &entries)
        .map_err(|error| ToolError(format!("{}: {error}", json_path.display())))?;
    Ok((
        PreparedFile {
            relative: PathBuf::new(),
            bytes: injected.bytes,
        },
        injected.json_entries,
        injected.patched,
        injected.unchanged,
    ))
}

pub fn inject_path(
    source: &Path,
    json: &Path,
    output_arg: Option<&Path>,
    overwrite: bool,
) -> ToolResult<InjectReport> {
    if !source.exists() {
        return Err(ToolError(format!(
            "source does not exist: '{}'",
            source.display()
        )));
    }
    if !json.exists() {
        return Err(ToolError(format!(
            "JSON input does not exist: '{}'",
            json.display()
        )));
    }
    if source.is_file() != json.is_file() || source.is_dir() != json.is_dir() {
        return Err(ToolError(
            "SOURCE and JSON must both be files or both be directories".to_string(),
        ));
    }
    if source.is_file() && !has_extension(source, "ab") {
        return Err(ToolError(format!(
            "source file is not .AB: '{}'",
            source.display()
        )));
    }
    if source.is_file() && !has_extension(json, "json") {
        return Err(ToolError(format!(
            "translation file is not .json: '{}'",
            json.display()
        )));
    }

    let output = output_arg
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_inject_output(source));
    ensure_separate_output(source, &output, source.is_dir())?;

    let mut prepared = Vec::new();
    let mut json_entries = 0usize;
    let mut patched = 0usize;
    let mut unchanged = 0usize;
    if source.is_file() {
        let relative = PathBuf::from(source.file_name().ok_or_else(|| {
            ToolError(format!("source has no file name: '{}'", source.display()))
        })?);
        let (mut file, count, changed, same) =
            prepare_injected_file(source, json, &path_label(&relative)?)?;
        file.relative = relative;
        prepared.push(file);
        json_entries += count;
        patched += changed;
        unchanged += same;
    } else {
        let json_files = collect_files(json, "json")?;
        if json_files.is_empty() {
            return Err(ToolError(format!(
                "directory contains no .json files: '{}'",
                json.display()
            )));
        }
        let mut destinations = HashSet::new();
        for relative_json in json_files {
            let relative_source = relative_json.with_extension("");
            if !has_extension(&relative_source, "ab") {
                return Err(ToolError(format!(
                    "translation path must end in .AB.json: '{}'",
                    json.join(&relative_json).display()
                )));
            }
            if !destinations.insert(path_key(&relative_source)) {
                return Err(ToolError(format!(
                    "duplicate case-insensitive translation target: '{}'",
                    relative_source.display()
                )));
            }
            let source_path = source.join(&relative_source);
            if !source_path.is_file() {
                return Err(ToolError(format!(
                    "translation has no matching source script: '{}'",
                    source_path.display()
                )));
            }
            let (mut file, count, changed, same) = prepare_injected_file(
                &source_path,
                &json.join(&relative_json),
                &path_label(&relative_source)?,
            )?;
            file.relative = relative_source;
            prepared.push(file);
            json_entries += count;
            patched += changed;
            unchanged += same;
        }
    }

    let staging = prepare_staging(&output, overwrite)?;
    let write_result = if source.is_file() {
        fs::write(&staging, &prepared[0].bytes)
            .map_err(|error| ToolError(format!("cannot write '{}': {error}", staging.display())))
    } else {
        copy_tree(source, &staging).and_then(|_| {
            for file in &prepared {
                let target = staging.join(&file.relative);
                fs::write(&target, &file.bytes).map_err(|error| {
                    ToolError(format!("cannot write '{}': {error}", target.display()))
                })?;
            }
            Ok(())
        })
    };
    if let Err(error) = write_result {
        let _ = remove_path(&staging);
        return Err(error);
    }
    if let Err(error) = commit_staging(&staging, &output, overwrite) {
        let _ = remove_path(&staging);
        return Err(error);
    }

    Ok(InjectReport {
        source: source.to_path_buf(),
        json: json.to_path_buf(),
        output,
        json_files: prepared.len(),
        json_entries,
        patched,
        unchanged,
        warnings: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ab::encode_cp932;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

    struct TempDirectory(PathBuf);

    impl TempDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "kokorov-ab-test-{}-{}",
                std::process::id(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn sample_script(message: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&encode_cp932(&format!("Name\\N{message}"), "test").unwrap());
        bytes.push(0);
        bytes.extend_from_slice(&13u16.to_le_bytes());
        bytes
    }

    #[test]
    fn directory_round_trip_preserves_source_tree() {
        let temp = TempDirectory::new();
        let source = temp.0.join("mes");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("ONE.AB"), sample_script("Text")).unwrap();
        fs::write(source.join("asset.bin"), b"opaque").unwrap();

        let json = temp.0.join("json");
        let extract = extract_path(&source, Some(&json), false).unwrap();
        assert_eq!(extract.extracted_entries, 1);
        let output = temp.0.join("injected");
        let inject = inject_path(&source, &json, Some(&output), false).unwrap();
        assert_eq!(inject.patched, 0);
        assert_eq!(
            fs::read(output.join("ONE.AB")).unwrap(),
            fs::read(source.join("ONE.AB")).unwrap()
        );
        assert_eq!(fs::read(output.join("asset.bin")).unwrap(), b"opaque");
    }

    #[test]
    fn refuses_existing_output_without_overwrite() {
        let temp = TempDirectory::new();
        let source = temp.0.join("ONE.AB");
        fs::write(&source, sample_script("Text")).unwrap();
        let output = temp.0.join("existing.json");
        fs::write(&output, b"keep").unwrap();
        let error = extract_path(&source, Some(&output), false)
            .unwrap_err()
            .to_string();
        assert!(error.contains("refusing to overwrite"));
        assert_eq!(fs::read(output).unwrap(), b"keep");
    }
}
