use crate::script::{decode_cp932_exact, encode_cp932_exact, Script};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const FORMAT_ID: &str = "shangri-la1-mes-json-v1";
pub const MAX_SCRIPT_BYTES: usize = 64_000;
const OPEN_BRACKET: char = '\u{ff3b}';
const CLOSE_BRACKET: char = '\u{ff3d}';

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TranslationFile {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_file")]
    pub file: String,
    pub entries: Vec<TranslationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TranslationEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_inst_offset")]
    pub instruction_offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub kind: String,
    #[serde(rename = "_opcode")]
    pub opcode: u8,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_policy")]
    pub policy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractReport {
    pub scanned_files: usize,
    pub json_files: usize,
    pub extracted_entries: usize,
    pub skipped: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectReport {
    pub source_files: usize,
    pub json_files: usize,
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub output_files: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextJsonError(String);

impl TextJsonError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for TextJsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for TextJsonError {}

#[derive(Debug, Clone)]
struct ProjectedText {
    instruction_index: usize,
    entry: TranslationEntry,
}

#[derive(Debug, Clone)]
struct Projection {
    file: TranslationFile,
    sources: Vec<ProjectedText>,
    skipped: usize,
}

pub fn extract_input(input: &Path, output: &Path) -> Result<ExtractReport, TextJsonError> {
    let metadata = fs::metadata(input).map_err(|error| io_error("inspect input", input, error))?;
    if metadata.is_file() {
        extract_file(input, output)
    } else if metadata.is_dir() {
        extract_directory(input, output)
    } else {
        Err(TextJsonError::new(format!(
            "input is neither a regular file nor a directory: {}",
            input.display()
        )))
    }
}

pub fn inject_input(
    source: &Path,
    translations: &Path,
    output: &Path,
) -> Result<InjectReport, TextJsonError> {
    let metadata =
        fs::metadata(source).map_err(|error| io_error("inspect source", source, error))?;
    if metadata.is_file() {
        inject_file(source, translations, output)
    } else if metadata.is_dir() {
        inject_directory(source, translations, output)
    } else {
        Err(TextJsonError::new(format!(
            "source is neither a regular file nor a directory: {}",
            source.display()
        )))
    }
}

pub fn translation_to_json(file: &TranslationFile) -> Result<Vec<u8>, TextJsonError> {
    let mut json = serde_json::to_string_pretty(file).map_err(|error| {
        TextJsonError::new(format!("cannot serialize translation JSON: {error}"))
    })?;
    json.push('\n');
    Ok(json.into_bytes())
}

pub fn translation_from_json(bytes: &[u8], path: &Path) -> Result<TranslationFile, TextJsonError> {
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(TextJsonError::new(format!(
            "UTF-8 BOM is not allowed in {}",
            path.display()
        )));
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|error| TextJsonError::new(format!("{} is not UTF-8: {error}", path.display())))?;
    serde_json::from_str(text)
        .map_err(|error| TextJsonError::new(format!("invalid JSON in {}: {error}", path.display())))
}

fn extract_file(input: &Path, output: &Path) -> Result<ExtractReport, TextJsonError> {
    reject_existing_output(output)?;
    let file_name = source_file_name(input)?;
    let bytes = fs::read(input).map_err(|error| io_error("read script", input, error))?;
    let script = Script::parse(&bytes).map_err(|error| {
        TextJsonError::new(format!("cannot parse {}: {error}", input.display()))
    })?;
    let projection = project_script(&file_name, &script)?;
    let json = translation_to_json(&projection.file)?;
    write_staged_file(output, &json)?;
    Ok(ExtractReport {
        scanned_files: 1,
        json_files: 1,
        extracted_entries: projection.file.entries.len(),
        skipped: projection.skipped,
        warnings: 0,
    })
}

fn extract_directory(input: &Path, output: &Path) -> Result<ExtractReport, TextJsonError> {
    reject_existing_output(output)?;
    let files = collect_flat_files(input)?;
    let mut prepared = Vec::new();
    let mut report = ExtractReport {
        scanned_files: files.len(),
        json_files: 0,
        extracted_entries: 0,
        skipped: 0,
        warnings: 0,
    };
    for path in files {
        let file_name = source_file_name(&path)?;
        let bytes = fs::read(&path).map_err(|error| io_error("read script", &path, error))?;
        let script = Script::parse(&bytes).map_err(|error| {
            TextJsonError::new(format!("cannot parse {}: {error}", path.display()))
        })?;
        let projection = project_script(&file_name, &script)?;
        report.skipped += projection.skipped;
        report.extracted_entries += projection.file.entries.len();
        if projection.file.entries.is_empty() {
            continue;
        }
        let mut json_name = file_name;
        json_name.push_str(".json");
        prepared.push((json_name, translation_to_json(&projection.file)?));
    }
    report.json_files = prepared.len();
    write_staged_directory(output, &prepared)?;
    Ok(report)
}

fn inject_file(
    source: &Path,
    translations: &Path,
    output: &Path,
) -> Result<InjectReport, TextJsonError> {
    reject_existing_output(output)?;
    if !translations.is_file() {
        return Err(TextJsonError::new(format!(
            "translation input is not a JSON file: {}",
            translations.display()
        )));
    }
    let file_name = source_file_name(source)?;
    let source_bytes =
        fs::read(source).map_err(|error| io_error("read source script", source, error))?;
    let json_bytes = fs::read(translations)
        .map_err(|error| io_error("read translation JSON", translations, error))?;
    let translation = translation_from_json(&json_bytes, translations)?;
    let (rebuilt, patched, unchanged, entries) =
        apply_translation(&file_name, &source_bytes, &translation)?;
    write_staged_file(output, &rebuilt)?;
    Ok(InjectReport {
        source_files: 1,
        json_files: 1,
        json_entries: entries,
        patched,
        unchanged,
        output_files: 1,
        warnings: 0,
    })
}

fn inject_directory(
    source: &Path,
    translations: &Path,
    output: &Path,
) -> Result<InjectReport, TextJsonError> {
    reject_existing_output(output)?;
    if !translations.is_dir() {
        return Err(TextJsonError::new(format!(
            "translation input is not a directory: {}",
            translations.display()
        )));
    }

    let json_paths = collect_flat_files(translations)?;
    let mut json_by_file = HashMap::new();
    for path in &json_paths {
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            return Err(TextJsonError::new(format!(
                "translation directory contains a non-JSON file: {}",
                path.display()
            )));
        }
        let bytes =
            fs::read(path).map_err(|error| io_error("read translation JSON", path, error))?;
        let translation = translation_from_json(&bytes, path)?;
        let key = translation.file.to_ascii_uppercase();
        if json_by_file.insert(key, translation).is_some() {
            return Err(TextJsonError::new(format!(
                "duplicate translation source file in {}",
                path.display()
            )));
        }
    }

    let source_paths = collect_flat_files(source)?;
    let mut prepared = Vec::with_capacity(source_paths.len());
    let mut used_json = HashSet::new();
    let mut report = InjectReport {
        source_files: source_paths.len(),
        json_files: json_paths.len(),
        json_entries: 0,
        patched: 0,
        unchanged: 0,
        output_files: source_paths.len(),
        warnings: 0,
    };
    for path in source_paths {
        let file_name = source_file_name(&path)?;
        let key = file_name.to_ascii_uppercase();
        let bytes =
            fs::read(&path).map_err(|error| io_error("read source script", &path, error))?;
        let script = Script::parse(&bytes).map_err(|error| {
            TextJsonError::new(format!("cannot parse {}: {error}", path.display()))
        })?;
        let projection = project_script(&file_name, &script)?;
        let output_bytes = if projection.file.entries.is_empty() {
            if json_by_file.contains_key(&key) {
                return Err(TextJsonError::new(format!(
                    "translation JSON exists for source with no extractable entries: {file_name}"
                )));
            }
            bytes
        } else {
            let translation = json_by_file.get(&key).ok_or_else(|| {
                TextJsonError::new(format!("missing translation JSON for {file_name}"))
            })?;
            used_json.insert(key.clone());
            let (rebuilt, patched, unchanged, entries) =
                apply_projected_translation(&file_name, &script, projection, translation)?;
            report.patched += patched;
            report.unchanged += unchanged;
            report.json_entries += entries;
            rebuilt
        };
        prepared.push((file_name, output_bytes));
    }
    if used_json.len() != json_by_file.len() {
        let mut extras: Vec<_> = json_by_file
            .keys()
            .filter(|key| !used_json.contains(*key))
            .cloned()
            .collect();
        extras.sort_unstable();
        return Err(TextJsonError::new(format!(
            "translation JSON references source files not present in the source directory: {}",
            extras.join(", ")
        )));
    }
    write_staged_directory(output, &prepared)?;
    Ok(report)
}

fn apply_translation(
    file_name: &str,
    source_bytes: &[u8],
    translation: &TranslationFile,
) -> Result<(Vec<u8>, usize, usize, usize), TextJsonError> {
    let script = Script::parse(source_bytes).map_err(|error| {
        TextJsonError::new(format!("cannot parse source script {file_name}: {error}"))
    })?;
    let projection = project_script(file_name, &script)?;
    apply_projected_translation(file_name, &script, projection, translation)
}

fn apply_projected_translation(
    file_name: &str,
    script: &Script,
    projection: Projection,
    translation: &TranslationFile,
) -> Result<(Vec<u8>, usize, usize, usize), TextJsonError> {
    validate_translation_header(file_name, &projection.file, translation)?;
    let mut replacements = BTreeMap::new();
    let mut patched = 0usize;
    let mut unchanged = 0usize;
    for (source, edited) in projection.sources.iter().zip(&translation.entries) {
        validate_entry(&source.entry, edited)?;
        validate_message(&edited.message, edited.index, file_name)?;
        if edited.message == source.entry.scr_msg {
            unchanged += 1;
            continue;
        }
        let full_text = match &source.entry.name {
            Some(name) => format!("{OPEN_BRACKET}{name}{CLOSE_BRACKET}{}", edited.message),
            None => edited.message.clone(),
        };
        let encoded = encode_cp932_exact(&full_text).map_err(|error| {
            TextJsonError::new(format!(
                "{file_name} entry[{}] cannot be encoded: {error}",
                edited.index
            ))
        })?;
        replacements.insert(source.instruction_index, encoded);
        patched += 1;
    }

    let rebuilt = script.rebuild_with_texts(&replacements).map_err(|error| {
        TextJsonError::new(format!(
            "cannot rebuild translated script {file_name}: {error}"
        ))
    })?;
    if rebuilt.len() > MAX_SCRIPT_BYTES {
        return Err(TextJsonError::new(format!(
            "rebuilt {file_name} is {} bytes and exceeds the runtime's {MAX_SCRIPT_BYTES}-byte script buffer",
            rebuilt.len()
        )));
    }
    let reparsed = Script::parse(&rebuilt).map_err(|error| {
        TextJsonError::new(format!(
            "rebuilt script {file_name} does not parse: {error}"
        ))
    })?;
    if reparsed.rebuild().map_err(|error| {
        TextJsonError::new(format!("cannot verify rebuilt script {file_name}: {error}"))
    })? != rebuilt
    {
        return Err(TextJsonError::new(format!(
            "rebuilt script is not byte-exact under parse/rebuild verification: {file_name}"
        )));
    }
    let verified = project_script(file_name, &reparsed)?;
    if verified.file.entries.len() != translation.entries.len() {
        return Err(TextJsonError::new(format!(
            "rebuilt {file_name} text count changed: expected {}, got {}",
            translation.entries.len(),
            verified.file.entries.len()
        )));
    }
    for (actual, expected) in verified.file.entries.iter().zip(&translation.entries) {
        if actual.name != expected.name || actual.scr_msg != expected.message {
            return Err(TextJsonError::new(format!(
                "rebuilt {file_name} entry[{}] verification mismatch",
                expected.index
            )));
        }
    }
    Ok((rebuilt, patched, unchanged, translation.entries.len()))
}

fn project_script(file_name: &str, script: &Script) -> Result<Projection, TextJsonError> {
    let mut entries = Vec::new();
    let mut sources = Vec::new();
    let mut skipped = 0usize;
    for (instruction_index, instruction) in script.instructions.iter().enumerate() {
        if instruction.opcode != 0x01 {
            continue;
        }
        let raw = instruction
            .text
            .as_deref()
            .expect("opcode 0x01 always has parsed text");
        let text = decode_cp932_exact(raw).map_err(|error| {
            TextJsonError::new(format!(
                "cannot decode CP932 display text in {file_name} at 0x{:X}: {error}",
                instruction.offset
            ))
        })?;
        if text == "\u{25b2}" || text == "\u{25bc}" {
            skipped += 1;
            continue;
        }

        let (name, message, prefix_size, kind) = split_name_message(&text).map_err(|error| {
            TextJsonError::new(format!(
                "invalid name/message structure in {file_name} at 0x{:X}: {error}",
                instruction.offset
            ))
        })?;
        let message_bytes = encode_cp932_exact(message).map_err(|error| {
            TextJsonError::new(format!(
                "cannot validate CP932 body in {file_name} at 0x{:X}: {error}",
                instruction.offset
            ))
        })?;
        let index = entries.len();
        let entry = TranslationEntry {
            file: file_name.to_string(),
            index,
            offset: instruction.offset + 1 + prefix_size,
            instruction_offset: instruction.offset,
            size: message_bytes.len(),
            kind: kind.to_string(),
            opcode: 0x01,
            encoding: "CP932".to_string(),
            policy: "relocate".to_string(),
            name: name.map(str::to_string),
            scr_msg: message.to_string(),
            message: message.to_string(),
        };
        entries.push(entry.clone());
        sources.push(ProjectedText {
            instruction_index,
            entry,
        });
    }
    Ok(Projection {
        file: TranslationFile {
            format: FORMAT_ID.to_string(),
            file: file_name.to_string(),
            entries,
        },
        sources,
        skipped,
    })
}

fn split_name_message(text: &str) -> Result<(Option<&str>, &str, usize, &'static str), String> {
    if let Some(rest) = text.strip_prefix(OPEN_BRACKET) {
        let close = rest
            .find(CLOSE_BRACKET)
            .ok_or_else(|| "leading fullwidth name bracket has no closing bracket".to_string())?;
        let name = &rest[..close];
        let message = &rest[close + CLOSE_BRACKET.len_utf8()..];
        if name.is_empty() {
            return Err("speaker name is empty".to_string());
        }
        if message.is_empty() {
            return Err("message body is empty".to_string());
        }
        if name.contains(OPEN_BRACKET) || name.contains(CLOSE_BRACKET) {
            return Err("speaker name contains nested brackets".to_string());
        }
        let prefix = format!("{OPEN_BRACKET}{name}{CLOSE_BRACKET}");
        let prefix_size = encode_cp932_exact(&prefix)
            .map_err(|error| format!("speaker prefix is not CP932: {error}"))?
            .len();
        Ok((Some(name), message, prefix_size, "dialogue"))
    } else {
        if text.contains(OPEN_BRACKET) || text.contains(CLOSE_BRACKET) {
            return Err("bracket characters occur outside a leading speaker prefix".to_string());
        }
        if text.is_empty() {
            return Err("display string is empty".to_string());
        }
        Ok((None, text, 0, "unnamed"))
    }
}

fn validate_translation_header(
    file_name: &str,
    source: &TranslationFile,
    edited: &TranslationFile,
) -> Result<(), TextJsonError> {
    if edited.format != FORMAT_ID {
        return Err(TextJsonError::new(format!(
            "{file_name} has unsupported _format {:?}; expected {FORMAT_ID:?}",
            edited.format
        )));
    }
    if edited.file != source.file {
        return Err(TextJsonError::new(format!(
            "translation _file mismatch: expected {:?}, got {:?}",
            source.file, edited.file
        )));
    }
    if edited.entries.len() != source.entries.len() {
        return Err(TextJsonError::new(format!(
            "{file_name} entry count mismatch: expected {}, got {}",
            source.entries.len(),
            edited.entries.len()
        )));
    }
    Ok(())
}

fn validate_entry(
    source: &TranslationEntry,
    edited: &TranslationEntry,
) -> Result<(), TextJsonError> {
    let index = source.index;
    macro_rules! immutable {
        ($field:ident, $label:literal) => {
            if edited.$field != source.$field {
                return Err(TextJsonError::new(format!(
                    "{} entry[{index}] immutable {} mismatch: expected {:?}, got {:?}",
                    source.file, $label, source.$field, edited.$field
                )));
            }
        };
    }
    immutable!(file, "_file");
    immutable!(index, "_index");
    immutable!(offset, "_offset");
    immutable!(instruction_offset, "_inst_offset");
    immutable!(size, "_size");
    immutable!(kind, "_type");
    immutable!(opcode, "_opcode");
    immutable!(encoding, "_encoding");
    immutable!(policy, "_policy");
    immutable!(name, "name");
    immutable!(scr_msg, "scr_msg");
    Ok(())
}

fn validate_message(message: &str, index: usize, file_name: &str) -> Result<(), TextJsonError> {
    if message.contains('\0') {
        return Err(TextJsonError::new(format!(
            "{file_name} entry[{index}] message contains NUL"
        )));
    }
    if message.contains(['\r', '\n']) {
        return Err(TextJsonError::new(format!(
            "{file_name} entry[{index}] message contains CR/LF, which is not present in this format"
        )));
    }
    if message.is_empty() {
        return Err(TextJsonError::new(format!(
            "{file_name} entry[{index}] message is empty"
        )));
    }
    Ok(())
}

fn collect_flat_files(directory: &Path) -> Result<Vec<PathBuf>, TextJsonError> {
    let mut files = Vec::new();
    for item in
        fs::read_dir(directory).map_err(|error| io_error("read directory", directory, error))?
    {
        let item = item.map_err(|error| io_error("enumerate directory", directory, error))?;
        let path = item.path();
        if !item
            .file_type()
            .map_err(|error| io_error("inspect directory item", &path, error))?
            .is_file()
        {
            return Err(TextJsonError::new(format!(
                "directory must be flat and contain only files: {}",
                path.display()
            )));
        }
        files.push(path);
    }
    files.sort_unstable_by_key(|path| {
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_uppercase()
    });
    Ok(files)
}

fn source_file_name(path: &Path) -> Result<String, TextJsonError> {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            TextJsonError::new(format!(
                "source path has no valid Unicode filename: {}",
                path.display()
            ))
        })
}

fn reject_existing_output(path: &Path) -> Result<(), TextJsonError> {
    if path.exists() {
        Err(TextJsonError::new(format!(
            "output already exists: {}",
            path.display()
        )))
    } else {
        Ok(())
    }
}

fn staging_path(output: &Path) -> Result<PathBuf, TextJsonError> {
    let file_name = output.file_name().ok_or_else(|| {
        TextJsonError::new(format!(
            "output must name an item below a parent: {}",
            output.display()
        ))
    })?;
    let mut name = file_name.to_os_string();
    name.push(".partial");
    Ok(output.with_file_name(name))
}

fn write_staged_file(output: &Path, bytes: &[u8]) -> Result<(), TextJsonError> {
    let staging = staging_path(output)?;
    reject_existing_output(&staging)?;
    fs::write(&staging, bytes).map_err(|error| io_error("write staging file", &staging, error))?;
    if let Err(error) = fs::rename(&staging, output) {
        let _ = fs::remove_file(&staging);
        return Err(io_error("commit output file", output, error));
    }
    Ok(())
}

fn write_staged_directory(output: &Path, files: &[(String, Vec<u8>)]) -> Result<(), TextJsonError> {
    let staging = staging_path(output)?;
    reject_existing_output(&staging)?;
    fs::create_dir(&staging)
        .map_err(|error| io_error("create staging directory", &staging, error))?;
    let result = (|| {
        for (name, bytes) in files {
            let path = staging.join(name);
            fs::write(&path, bytes).map_err(|error| io_error("write output file", &path, error))?;
        }
        fs::rename(&staging, output)
            .map_err(|error| io_error("commit output directory", output, error))?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    Ok(())
}

fn io_error(action: &str, path: &Path, error: io::Error) -> TextJsonError {
    TextJsonError::new(format!("failed to {action} {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn script_bytes(texts: &[&str]) -> Vec<u8> {
        let mut bytes = 0u32.to_le_bytes().to_vec();
        for text in texts {
            bytes.push(0x01);
            bytes.extend_from_slice(&encode_cp932_exact(text).unwrap());
            bytes.push(0);
        }
        bytes.push(0);
        bytes
    }

    #[test]
    fn extracts_named_and_unnamed_text_and_skips_ui_arrows() {
        let bytes = script_bytes(&["［五月］はじめまして。", "地の文。", "▲"]);
        let script = Script::parse(&bytes).unwrap();
        let projection = project_script("A.MES", &script).unwrap();
        assert_eq!(projection.file.entries.len(), 2);
        assert_eq!(projection.skipped, 1);
        assert_eq!(projection.file.entries[0].name.as_deref(), Some("五月"));
        assert_eq!(projection.file.entries[0].scr_msg, "はじめまして。");
        assert_eq!(projection.file.entries[1].name, None);
        assert_eq!(projection.file.entries[1].kind, "unnamed");
    }

    #[test]
    fn unchanged_json_rebuilds_byte_exactly() {
        let bytes = script_bytes(&["［五月］はじめまして。", "地の文。"]);
        let script = Script::parse(&bytes).unwrap();
        let projection = project_script("A.MES", &script).unwrap();
        let (rebuilt, patched, unchanged, entries) =
            apply_projected_translation("A.MES", &script, projection.clone(), &projection.file)
                .unwrap();
        assert_eq!(rebuilt, bytes);
        assert_eq!((patched, unchanged, entries), (0, 2, 2));
    }

    #[test]
    fn changes_body_but_preserves_read_only_name() {
        let bytes = script_bytes(&["［五月］はじめまして。"]);
        let script = Script::parse(&bytes).unwrap();
        let projection = project_script("A.MES", &script).unwrap();
        let mut edited = projection.file.clone();
        edited.entries[0].message = "さようなら。".to_string();
        let (rebuilt, patched, unchanged, _) =
            apply_projected_translation("A.MES", &script, projection, &edited).unwrap();
        assert_eq!((patched, unchanged), (1, 0));
        let reparsed = Script::parse(&rebuilt).unwrap();
        let text = decode_cp932_exact(reparsed.instructions[0].text.as_deref().unwrap()).unwrap();
        assert_eq!(text, "［五月］さようなら。");
    }

    #[test]
    fn rejects_changed_source_and_name_fields() {
        let bytes = script_bytes(&["［五月］はじめまして。"]);
        let script = Script::parse(&bytes).unwrap();
        let projection = project_script("A.MES", &script).unwrap();
        let mut edited = projection.file.clone();
        edited.entries[0].scr_msg = "changed".to_string();
        assert!(apply_projected_translation("A.MES", &script, projection, &edited).is_err());

        let script = Script::parse(&bytes).unwrap();
        let projection = project_script("A.MES", &script).unwrap();
        let mut edited = projection.file.clone();
        edited.entries[0].name = Some("菜奈".to_string());
        assert!(apply_projected_translation("A.MES", &script, projection, &edited).is_err());
    }

    #[test]
    fn rejects_nul_newline_and_unencodable_messages() {
        let bytes = script_bytes(&["地の文。"]);
        for message in ["bad\0text", "bad\ntext", "简体中文"] {
            let script = Script::parse(&bytes).unwrap();
            let projection = project_script("A.MES", &script).unwrap();
            let mut edited = projection.file.clone();
            edited.entries[0].message = message.to_string();
            assert!(apply_projected_translation("A.MES", &script, projection, &edited).is_err());
        }
    }

    #[test]
    fn json_is_utf8_without_bom_and_round_trips() {
        let file = TranslationFile {
            format: FORMAT_ID.to_string(),
            file: "A.MES".to_string(),
            entries: Vec::new(),
        };
        let bytes = translation_to_json(&file).unwrap();
        assert!(!bytes.starts_with(&[0xef, 0xbb, 0xbf]));
        assert!(std::str::from_utf8(&bytes).unwrap().ends_with('\n'));
        assert_eq!(
            translation_from_json(&bytes, Path::new("A.MES.json")).unwrap(),
            file
        );
    }
}
