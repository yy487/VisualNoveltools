use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use encoding_rs::SHIFT_JIS;

use crate::error::{Error, Result};
use crate::model::{InjectionSummary, TranslationEntry};
use crate::normalization::TextPolicy;
use crate::scr::{RawRecord, has_version5_signature, parse_version5, rebuild_version5};

#[derive(Debug)]
struct SourceFile {
    path: PathBuf,
    relative_path: PathBuf,
    is_script: bool,
}

#[derive(Debug)]
struct PreparedScript {
    relative_path: PathBuf,
    bytes: Vec<u8>,
}

pub fn inject_directory(
    source: &Path,
    translation: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<InjectionSummary> {
    require_directory(source, "source")?;
    require_directory(translation, "translation")?;
    validate_output_scope(source, translation, output)?;

    let source_files = discover_source_files(source)?;
    let scripts: Vec<&SourceFile> = source_files.iter().filter(|file| file.is_script).collect();
    if scripts.is_empty() {
        return Err(Error::with_path(
            source.display(),
            "no SCR:2005 files found",
        ));
    }

    let mut expected_json = BTreeMap::new();
    for script in &scripts {
        let json_path = json_path_for_source(&script.relative_path);
        let key = path_key(&json_path);
        if expected_json.insert(key, *script).is_some() {
            return Err(Error::new(format!(
                "multiple source scripts map to translation {}",
                json_path.display()
            )));
        }
    }

    let translation_files = discover_translation_files(translation)?;
    let mut translated_keys = BTreeSet::new();
    for relative_path in &translation_files {
        let key = path_key(relative_path);
        if !expected_json.contains_key(&key) {
            return Err(Error::with_path(
                translation.join(relative_path).display(),
                "translation JSON has no matching SCR:2005 source script",
            ));
        }
        if !translated_keys.insert(key) {
            return Err(Error::with_path(
                translation.join(relative_path).display(),
                "duplicate translation path",
            ));
        }
    }

    let mut summary = InjectionSummary {
        source_files: source_files.len(),
        scripts: scripts.len(),
        translation_files: translation_files.len(),
        ..InjectionSummary::default()
    };
    let mut prepared = Vec::with_capacity(translation_files.len());

    for relative_json in &translation_files {
        let source_file = expected_json
            .get(&path_key(relative_json))
            .copied()
            .expect("validated translation mapping");
        let source_bytes = fs::read(&source_file.path)
            .map_err(|error| Error::with_path(source_file.path.display(), error))?;
        let label = display_path(&source_file.relative_path);
        let parsed = parse_version5(&source_bytes, &label)?;
        let translation_path = translation.join(relative_json);
        let entries = read_translation(&translation_path)?;
        let changes = validate_and_prepare(&label, &parsed.records, &entries)?;

        summary.entries += entries.len();
        summary.changed_entries += changes.changed_entries;
        summary.changed_messages += changes.changed_messages;
        summary.changed_names += changes.changed_names;
        let rebuilt = rebuild_version5(&source_bytes, &label, &changes.replacements)?;
        parse_version5(&rebuilt, &format!("rebuilt {label}"))?;
        if rebuilt == source_bytes {
            summary.byte_exact_scripts += 1;
        } else {
            summary.rebuilt_scripts += 1;
        }
        prepared.push(PreparedScript {
            relative_path: source_file.relative_path.clone(),
            bytes: rebuilt,
        });
    }

    install_output_tree(output, &source_files, &prepared, overwrite)?;
    Ok(summary)
}

#[derive(Debug, Default)]
struct PreparedChanges {
    replacements: BTreeMap<u32, Vec<u8>>,
    changed_entries: usize,
    changed_messages: usize,
    changed_names: usize,
}

fn validate_and_prepare(
    label: &str,
    records: &[RawRecord],
    entries: &[TranslationEntry],
) -> Result<PreparedChanges> {
    if entries.len() != records.len() {
        return Err(Error::new(format!(
            "{label}: translation has {} entries, source has {}",
            entries.len(),
            records.len()
        )));
    }

    let mut changes = PreparedChanges::default();
    for (index, (record, entry)) in records.iter().zip(entries).enumerate() {
        let context = format!("{label} entry {index}");
        validate_metadata(&context, label, index, record, entry)?;
        let policy = TextPolicy::from_label(&entry.policy).ok_or_else(|| {
            Error::new(format!("{context}: unsupported _policy {:?}", entry.policy))
        })?;
        let source_message = policy.apply(&record.message);
        if entry.scr_msg != source_message {
            return Err(Error::new(format!(
                "{context}: scr_msg does not match the current source script"
            )));
        }
        validate_editable_text(&context, "message", &entry.message)?;

        let message_changed = entry.message != entry.scr_msg;
        if message_changed {
            let mut encoded = encode_cp932(&context, "message", &entry.message)?;
            match record.terminator {
                Some("LF") => encoded.push(b'\n'),
                Some("CRLF") => encoded.extend_from_slice(b"\r\n"),
                Some(other) => {
                    return Err(Error::new(format!(
                        "{context}: unsupported source terminator {other:?}"
                    )));
                }
                None => {}
            }
            insert_replacement(
                &mut changes.replacements,
                record.string_offset,
                encoded,
                &context,
            )?;
            changes.changed_messages += 1;
        }

        let name_changed = match (&record.name, &record.name_string_offset) {
            (None, None) => {
                if entry.source_name.is_some() || entry.name.is_some() {
                    return Err(Error::new(format!(
                        "{context}: unnamed source entry cannot add name or _scr_name"
                    )));
                }
                false
            }
            (Some(raw_name), Some(name_offset)) => {
                let expected_name = policy.apply(raw_name);
                if entry.source_name.as_deref() != Some(expected_name.as_str()) {
                    return Err(Error::new(format!(
                        "{context}: _scr_name does not match the current source script"
                    )));
                }
                let name = entry.name.as_deref().ok_or_else(|| {
                    Error::new(format!("{context}: named source entry is missing name"))
                })?;
                validate_editable_text(&context, "name", name)?;
                if name != expected_name {
                    let encoded = encode_cp932(&context, "name", name)?;
                    insert_replacement(&mut changes.replacements, *name_offset, encoded, &context)?;
                    changes.changed_names += 1;
                    true
                } else {
                    false
                }
            }
            _ => {
                return Err(Error::new(format!(
                    "{context}: inconsistent source name metadata"
                )));
            }
        };

        if message_changed || name_changed {
            changes.changed_entries += 1;
        }
    }
    Ok(changes)
}

fn validate_metadata(
    context: &str,
    label: &str,
    index: usize,
    record: &RawRecord,
    entry: &TranslationEntry,
) -> Result<()> {
    let expected_terminator = record.terminator.map(str::to_owned);
    let checks = [
        (entry.file == label, "_file"),
        (entry.index == index, "_index"),
        (entry.offset == record.absolute_string_offset, "_offset"),
        (
            entry.string_offset == record.string_offset,
            "_string_offset",
        ),
        (
            entry.instruction_offset == record.instruction_offset,
            "_inst_offset",
        ),
        (entry.size == record.encoded_size, "_size"),
        (entry.entry_type == record.kind.label(), "_type"),
        (entry.opcode == record.opcode, "_opcode"),
        (entry.encoding == "CP932", "_encoding"),
        (entry.terminator == expected_terminator, "_terminator"),
    ];
    for (matches, field) in checks {
        if !matches {
            return Err(Error::new(format!(
                "{context}: {field} does not match the current source script"
            )));
        }
    }
    Ok(())
}

fn validate_editable_text(context: &str, field: &str, value: &str) -> Result<()> {
    if value.contains('\0') {
        return Err(Error::new(format!("{context}: {field} contains NUL")));
    }
    if value.contains('\r') {
        return Err(Error::new(format!(
            "{context}: {field} contains CR; only the confirmed LF display control is allowed"
        )));
    }
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            continue;
        }
        let escaped = characters.next().ok_or_else(|| {
            Error::new(format!(
                "{context}: {field} ends with a backslash control without a following character"
            ))
        })?;
        if !escaped.is_ascii() {
            return Err(Error::new(format!(
                "{context}: {field} backslash control is followed by non-ASCII {escaped:?}"
            )));
        }
    }
    Ok(())
}

fn encode_cp932(context: &str, field: &str, value: &str) -> Result<Vec<u8>> {
    let (encoded, _, had_errors) = SHIFT_JIS.encode(value);
    if !had_errors {
        return Ok(encoded.into_owned());
    }
    let mut unsupported = BTreeSet::new();
    for character in value.chars() {
        let mut buffer = [0u8; 4];
        let source = character.encode_utf8(&mut buffer);
        let (_, _, character_error) = SHIFT_JIS.encode(source);
        if character_error {
            unsupported.insert(format!("U+{:04X} {character:?}", character as u32));
        }
    }
    Err(Error::new(format!(
        "{context}: {field} contains characters not encodable as CP932: {}",
        unsupported.into_iter().collect::<Vec<_>>().join(", ")
    )))
}

fn insert_replacement(
    replacements: &mut BTreeMap<u32, Vec<u8>>,
    offset: u32,
    bytes: Vec<u8>,
    context: &str,
) -> Result<()> {
    if let Some(existing) = replacements.get(&offset) {
        if existing != &bytes {
            return Err(Error::new(format!(
                "{context}: shared string at offset 0x{offset:x} has conflicting translations"
            )));
        }
    } else {
        replacements.insert(offset, bytes);
    }
    Ok(())
}

fn read_translation(path: &Path) -> Result<Vec<TranslationEntry>> {
    let bytes = fs::read(path).map_err(|error| Error::with_path(path.display(), error))?;
    serde_json::from_slice(&bytes).map_err(|error| {
        Error::with_path(
            path.display(),
            format!("invalid UTF-8 translation JSON: {error}"),
        )
    })
}

fn require_directory(path: &Path, role: &str) -> Result<()> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| Error::with_path(path.display(), error))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(Error::with_path(
            path.display(),
            format!("{role} must be a real directory"),
        ));
    }
    Ok(())
}

fn discover_source_files(root: &Path) -> Result<Vec<SourceFile>> {
    let mut files = Vec::new();
    visit_source(root, root, &mut files)?;
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(files)
}

fn visit_source(root: &Path, directory: &Path, files: &mut Vec<SourceFile>) -> Result<()> {
    let mut children = read_children(directory)?;
    children.sort_by_key(|entry| entry.file_name());
    for child in children {
        let path = child.path();
        let file_type = child
            .file_type()
            .map_err(|error| Error::with_path(path.display(), error))?;
        if file_type.is_symlink() {
            return Err(Error::with_path(
                path.display(),
                "source tree contains a symbolic link",
            ));
        }
        if file_type.is_dir() {
            visit_source(root, &path, files)?;
        } else if file_type.is_file() {
            let relative_path = safe_relative(root, &path)?;
            let mut signature = [0u8; 8];
            let mut file =
                File::open(&path).map_err(|error| Error::with_path(path.display(), error))?;
            let read = file
                .read(&mut signature)
                .map_err(|error| Error::with_path(path.display(), error))?;
            files.push(SourceFile {
                path,
                relative_path,
                is_script: read == signature.len() && has_version5_signature(&signature),
            });
        }
    }
    Ok(())
}

fn discover_translation_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    visit_translation(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit_translation(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let mut children = read_children(directory)?;
    children.sort_by_key(|entry| entry.file_name());
    for child in children {
        let path = child.path();
        let file_type = child
            .file_type()
            .map_err(|error| Error::with_path(path.display(), error))?;
        if file_type.is_symlink() {
            return Err(Error::with_path(
                path.display(),
                "translation tree contains a symbolic link",
            ));
        }
        if file_type.is_dir() {
            visit_translation(root, &path, files)?;
        } else if file_type.is_file() && path.extension().and_then(OsStr::to_str) == Some("json") {
            files.push(safe_relative(root, &path)?);
        }
    }
    Ok(())
}

fn install_output_tree(
    output: &Path,
    source_files: &[SourceFile],
    prepared: &[PreparedScript],
    overwrite: bool,
) -> Result<()> {
    validate_existing_output(output, overwrite)?;
    let parent = parent_directory(output);
    fs::create_dir_all(parent).map_err(|error| Error::with_path(parent.display(), error))?;
    let output_name = output.file_name().ok_or_else(|| {
        Error::with_path(output.display(), "output must name a dedicated directory")
    })?;
    let staging = create_temporary_directory(parent, output_name, "tmp")?;

    let build_result = (|| -> Result<()> {
        for source_file in source_files {
            let destination = staging.join(&source_file.relative_path);
            let destination_parent = destination.parent().ok_or_else(|| {
                Error::with_path(destination.display(), "missing destination parent")
            })?;
            fs::create_dir_all(destination_parent)
                .map_err(|error| Error::with_path(destination_parent.display(), error))?;
            fs::copy(&source_file.path, &destination)
                .map_err(|error| Error::with_path(destination.display(), error))?;
        }
        for script in prepared {
            let destination = staging.join(&script.relative_path);
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&destination)
                .map_err(|error| Error::with_path(destination.display(), error))?;
            file.write_all(&script.bytes)
                .map_err(|error| Error::with_path(destination.display(), error))?;
            file.sync_all()
                .map_err(|error| Error::with_path(destination.display(), error))?;
        }
        Ok(())
    })();
    if let Err(error) = build_result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    let install_result = install_directory(&staging, output, overwrite);
    if install_result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    install_result
}

fn validate_existing_output(output: &Path, overwrite: bool) -> Result<()> {
    if !output.exists() {
        return Ok(());
    }
    let metadata =
        fs::symlink_metadata(output).map_err(|error| Error::with_path(output.display(), error))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(Error::with_path(
            output.display(),
            "output exists and is not a real directory",
        ));
    }
    if !overwrite {
        return Err(Error::with_path(
            output.display(),
            "output directory exists; pass --overwrite or confirm it interactively",
        ));
    }
    Ok(())
}

fn validate_output_scope(source: &Path, translation: &Path, output: &Path) -> Result<()> {
    let source = canonical(source)?;
    let translation = canonical(translation)?;
    let output = if output.exists() {
        canonical(output)?
    } else {
        let parent = canonical(parent_directory(output)).map_err(|_| {
            Error::with_path(
                output.display(),
                "output parent must already exist for scope validation",
            )
        })?;
        let name = output.file_name().ok_or_else(|| {
            Error::with_path(output.display(), "output must name a dedicated directory")
        })?;
        parent.join(name)
    };
    if source.starts_with(&output) || translation.starts_with(&output) {
        return Err(Error::new(format!(
            "output directory {} contains source or translation input; refusing replacement",
            output.display()
        )));
    }
    if output.starts_with(&source) || output.starts_with(&translation) {
        return Err(Error::new(format!(
            "output directory {} is inside source or translation input",
            output.display()
        )));
    }
    Ok(())
}

fn canonical(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).map_err(|error| Error::with_path(path.display(), error))
}

fn read_children(directory: &Path) -> Result<Vec<fs::DirEntry>> {
    fs::read_dir(directory)
        .map_err(|error| Error::with_path(directory.display(), error))?
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|error| Error::with_path(directory.display(), error))
}

fn safe_relative(root: &Path, path: &Path) -> Result<PathBuf> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| Error::new(format!("{} is outside input root", path.display())))?;
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(Error::new(format!(
            "unsafe relative path {}",
            relative.display()
        )));
    }
    Ok(relative.to_owned())
}

fn json_path_for_source(source: &Path) -> PathBuf {
    let mut output = source.to_owned();
    output.set_extension("json");
    output
}

fn path_key(path: &Path) -> String {
    display_path(path).to_lowercase()
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn parent_directory(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
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
    let name = output.file_name().ok_or_else(|| {
        Error::with_path(output.display(), "output must name a dedicated directory")
    })?;
    let backup = unique_sibling_path(parent, name, "replace")?;
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
                    "{}: install failed ({error}); restoring previous output also failed ({restore_error})",
                    output.display()
                )));
            }
            Err(Error::with_path(output.display(), error))
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_validation_allows_removal_and_valid_pairs() {
        assert!(validate_editable_text("entry", "message", "plain").is_ok());
        assert!(validate_editable_text("entry", "message", "[\\o\\y]").is_ok());
        assert!(validate_editable_text("entry", "message", "tail\\").is_err());
        assert!(validate_editable_text("entry", "message", "\\中").is_err());
    }

    #[test]
    fn json_mapping_is_one_to_one() {
        assert_eq!(
            json_path_for_source(Path::new("route/scene.scr")),
            PathBuf::from("route/scene.json")
        );
    }
}
