use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::model::{Entry, ExtractedFile, Extraction, Summary};
use crate::normalization::TextPolicy;
use crate::scr::{SIGNATURE, has_version5_signature, parse_version5};

#[derive(Debug)]
struct SourceFile {
    path: PathBuf,
    relative_path: PathBuf,
    relative_name: String,
}

pub fn extract(input: &Path, policy: TextPolicy) -> Result<Extraction> {
    let (sources, files_scanned, mut warnings) = discover_sources(input)?;
    if sources.is_empty() {
        return Err(Error::with_path(input.display(), "no SCR:2005 files found"));
    }

    let mut files = Vec::with_capacity(sources.len());
    let mut output_keys = BTreeSet::new();
    let mut summary = Summary {
        files_scanned,
        scripts_parsed: sources.len(),
        json_files: sources.len(),
        ..Summary::default()
    };

    for source in sources {
        let bytes = fs::read(&source.path)
            .map_err(|error| Error::with_path(source.path.display(), error))?;
        let parsed = parse_version5(&bytes, &source.relative_name)?;
        summary.commands += parsed.stats.commands;
        summary.strings += parsed.stats.strings;
        summary.messages += parsed.stats.messages;
        summary.named_messages += parsed.stats.named_messages;
        summary.choices += parsed.stats.choices;

        for warning in parsed.warnings {
            warnings.push(format!("{}: {warning}", source.relative_name));
        }

        let mut entries = Vec::with_capacity(parsed.records.len());
        for (index, record) in parsed.records.into_iter().enumerate() {
            let source_name = record.name.as_deref().map(|name| policy.apply(name));
            let message = policy.apply(&record.message);
            summary.internal_newlines += message.matches('\n').count();
            summary.ruby_controls += message
                .chars()
                .filter(|character| matches!(character, '\u{0004}' | '\u{0005}' | '\u{0006}'))
                .count();
            summary.backslashes += message.matches('\\').count();

            entries.push(Entry {
                file: source.relative_name.clone(),
                index,
                offset: record.absolute_string_offset,
                string_offset: record.string_offset,
                instruction_offset: record.instruction_offset,
                size: record.encoded_size,
                entry_type: record.kind.label(),
                opcode: record.opcode,
                encoding: "CP932",
                policy: policy.label(),
                terminator: record.terminator,
                source_name: source_name.clone(),
                name: source_name,
                scr_msg: message.clone(),
                message,
            });
        }

        summary.entries += entries.len();
        let relative_path = json_path_for_source(&source.relative_path);
        let collision_key = relative_path
            .to_string_lossy()
            .replace('\\', "/")
            .to_lowercase();
        if !output_keys.insert(collision_key) {
            return Err(Error::new(format!(
                "multiple source scripts map to output {}",
                relative_path.display()
            )));
        }
        files.push(ExtractedFile {
            relative_path,
            entries,
        });
    }

    summary.warnings = warnings.len();
    Ok(Extraction {
        files,
        summary,
        warnings,
    })
}

pub fn write_json_directory(
    input: &Path,
    output: &Path,
    files: &[ExtractedFile],
    overwrite: bool,
) -> Result<()> {
    validate_output_scope(input, output)?;
    if output.exists() {
        let metadata = fs::symlink_metadata(output)
            .map_err(|error| Error::with_path(output.display(), error))?;
        if metadata.file_type().is_symlink() {
            return Err(Error::with_path(
                output.display(),
                "symbolic-link output is not supported",
            ));
        }
        if !metadata.is_dir() {
            return Err(Error::with_path(
                output.display(),
                "output path exists and is not a directory",
            ));
        }
        if !overwrite {
            return Err(Error::with_path(
                output.display(),
                "output directory exists; pass --overwrite or confirm it interactively",
            ));
        }
    }

    let parent = parent_directory(output);
    let output_name = output.file_name().ok_or_else(|| {
        Error::with_path(output.display(), "output must name a dedicated directory")
    })?;
    fs::create_dir_all(parent).map_err(|error| Error::with_path(parent.display(), error))?;
    let staging = create_temporary_directory(parent, output_name, "tmp")?;

    if let Err(error) = build_output_tree(&staging, files) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    let install_result = install_directory(&staging, output, overwrite);
    if install_result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    install_result
}

fn build_output_tree(staging: &Path, files: &[ExtractedFile]) -> Result<()> {
    for extracted in files {
        if extracted.relative_path.is_absolute()
            || extracted
                .relative_path
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(Error::new(format!(
                "unsafe relative output path: {}",
                extracted.relative_path.display()
            )));
        }

        let destination = staging.join(&extracted.relative_path);
        let parent = destination
            .parent()
            .ok_or_else(|| Error::with_path(destination.display(), "missing parent directory"))?;
        fs::create_dir_all(parent).map_err(|error| Error::with_path(parent.display(), error))?;

        let mut encoded = serde_json::to_vec_pretty(&extracted.entries)?;
        encoded.push(b'\n');
        let _: serde_json::Value = serde_json::from_slice(&encoded)?;

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
            .map_err(|error| Error::with_path(destination.display(), error))?;
        file.write_all(&encoded)
            .map_err(|error| Error::with_path(destination.display(), error))?;
        file.sync_all()
            .map_err(|error| Error::with_path(destination.display(), error))?;
    }
    Ok(())
}

fn install_directory(staging: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if !output.exists() {
        return fs::rename(staging, output)
            .map_err(|error| Error::with_path(output.display(), error));
    }
    if !overwrite {
        return Err(Error::with_path(
            output.display(),
            "output directory already exists",
        ));
    }

    let parent = parent_directory(output);
    let output_name = output.file_name().ok_or_else(|| {
        Error::with_path(output.display(), "output must name a dedicated directory")
    })?;
    let backup = unique_sibling_path(parent, output_name, "replace")?;

    fs::rename(output, &backup).map_err(|error| Error::with_path(output.display(), error))?;
    match fs::rename(staging, output) {
        Ok(()) => {
            fs::remove_dir_all(&backup)
                .map_err(|error| Error::with_path(backup.display(), error))?;
            Ok(())
        }
        Err(error) => {
            let restore_error = fs::rename(&backup, output).err();
            if let Some(restore_error) = restore_error {
                return Err(Error::new(format!(
                    "{}: install failed ({error}); restoring the previous output also failed ({restore_error})",
                    output.display()
                )));
            }
            Err(Error::with_path(output.display(), error))
        }
    }
}

fn validate_output_scope(input: &Path, output: &Path) -> Result<()> {
    if !output.exists() {
        return Ok(());
    }
    let input =
        fs::canonicalize(input).map_err(|error| Error::with_path(input.display(), error))?;
    let output =
        fs::canonicalize(output).map_err(|error| Error::with_path(output.display(), error))?;
    if input.starts_with(&output) {
        return Err(Error::new(format!(
            "output directory {} contains the input {}; refusing destructive replacement",
            output.display(),
            input.display()
        )));
    }
    Ok(())
}

fn discover_sources(input: &Path) -> Result<(Vec<SourceFile>, usize, Vec<String>)> {
    let metadata =
        fs::symlink_metadata(input).map_err(|error| Error::with_path(input.display(), error))?;
    if metadata.file_type().is_symlink() {
        return Err(Error::with_path(
            input.display(),
            "symbolic-link input is not supported",
        ));
    }
    if metadata.is_file() {
        let mut signature = [0u8; 8];
        let mut file =
            File::open(input).map_err(|error| Error::with_path(input.display(), error))?;
        file.read_exact(&mut signature)
            .map_err(|error| Error::with_path(input.display(), error))?;
        if signature != *SIGNATURE {
            return Err(Error::with_path(
                input.display(),
                "expected SCR:2005 signature",
            ));
        }
        let relative_path = PathBuf::from(
            input
                .file_name()
                .unwrap_or_else(|| OsStr::new("script.scr")),
        );
        let relative_name = display_path(&relative_path);
        return Ok((
            vec![SourceFile {
                path: input.to_owned(),
                relative_path,
                relative_name,
            }],
            1,
            Vec::new(),
        ));
    }
    if !metadata.is_dir() {
        return Err(Error::with_path(
            input.display(),
            "input is not a file or directory",
        ));
    }

    let mut files_scanned = 0usize;
    let mut sources = Vec::new();
    let mut warnings = Vec::new();
    visit_directory(
        input,
        input,
        &mut files_scanned,
        &mut sources,
        &mut warnings,
    )?;
    sources.sort_by(|left, right| left.relative_name.cmp(&right.relative_name));
    Ok((sources, files_scanned, warnings))
}

fn visit_directory(
    root: &Path,
    directory: &Path,
    files_scanned: &mut usize,
    sources: &mut Vec<SourceFile>,
    warnings: &mut Vec<String>,
) -> Result<()> {
    let mut children = fs::read_dir(directory)
        .map_err(|error| Error::with_path(directory.display(), error))?
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|error| Error::with_path(directory.display(), error))?;
    children.sort_by_key(|entry| entry.file_name());

    for child in children {
        let path = child.path();
        let file_type = child
            .file_type()
            .map_err(|error| Error::with_path(path.display(), error))?;
        if file_type.is_symlink() {
            warnings.push(format!(
                "{}: skipped symbolic link",
                display_relative(root, &path)
            ));
        } else if file_type.is_dir() {
            visit_directory(root, &path, files_scanned, sources, warnings)?;
        } else if file_type.is_file() {
            *files_scanned += 1;
            let mut file =
                File::open(&path).map_err(|error| Error::with_path(path.display(), error))?;
            let mut signature = [0u8; 8];
            let read = file
                .read(&mut signature)
                .map_err(|error| Error::with_path(path.display(), error))?;
            if read == signature.len() && has_version5_signature(&signature) {
                let relative_path = path
                    .strip_prefix(root)
                    .map_err(|_| Error::new(format!("{} is outside input root", path.display())))?;
                sources.push(SourceFile {
                    relative_name: display_path(relative_path),
                    relative_path: relative_path.to_owned(),
                    path,
                });
            }
        }
    }
    Ok(())
}

fn json_path_for_source(source: &Path) -> PathBuf {
    let mut output = source.to_owned();
    output.set_extension("json");
    output
}

fn display_relative(root: &Path, path: &Path) -> String {
    display_path(path.strip_prefix(root).unwrap_or(path))
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn create_temporary_directory(parent: &Path, name: &OsStr, role: &str) -> Result<PathBuf> {
    for attempt in 0..100u32 {
        let path = sibling_path(parent, name, role, attempt);
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(Error::with_path(path.display(), error)),
        }
    }
    Err(Error::with_path(
        parent.display(),
        "could not allocate a temporary output directory",
    ))
}

fn unique_sibling_path(parent: &Path, name: &OsStr, role: &str) -> Result<PathBuf> {
    for attempt in 0..100u32 {
        let path = sibling_path(parent, name, role, attempt);
        if !path.exists() {
            return Ok(path);
        }
    }
    Err(Error::with_path(
        parent.display(),
        "could not allocate a replacement backup path",
    ))
}

fn sibling_path(parent: &Path, name: &OsStr, role: &str, attempt: u32) -> PathBuf {
    parent.join(format!(
        ".{}.{}-{}-{attempt}",
        name.to_string_lossy(),
        role,
        std::process::id()
    ))
}

fn parent_directory(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_each_source_to_a_sibling_json_path() {
        assert_eq!(
            json_path_for_source(Path::new("route/00_yuk.scr")),
            PathBuf::from("route/00_yuk.json")
        );
        assert_eq!(
            json_path_for_source(Path::new("script-without-extension")),
            PathBuf::from("script-without-extension.json")
        );
    }
}
