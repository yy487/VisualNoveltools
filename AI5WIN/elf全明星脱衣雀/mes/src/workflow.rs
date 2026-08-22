use crate::encoding::{decode_text, encode_text, EncodingRoute};
use crate::mes::{Argument, InstructionKind, MesScript, Replacement, TextSlot};
use crate::{fail, sha256_hex, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationDocument {
    pub schema_version: u32,
    pub format: String,
    pub source_file: String,
    pub source_sha256: String,
    pub source_encoding: EncodingRoute,
    pub entries: Vec<TranslationEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationEntry {
    pub name: Option<String>,
    #[serde(rename = "_scr_name")]
    pub scr_name: Option<String>,
    pub scr_msg: String,
    pub message: String,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_message_id")]
    pub message_id: usize,
    #[serde(rename = "_offset")]
    pub offset: u64,
    #[serde(rename = "_inst_offset")]
    pub inst_offset: u32,
    #[serde(rename = "_size")]
    pub size: u32,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_voice", skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(rename = "_name_offset", skip_serializing_if = "Option::is_none")]
    pub name_offset: Option<u64>,
    #[serde(rename = "_name_size", skip_serializing_if = "Option::is_none")]
    pub name_size: Option<u32>,
    #[serde(rename = "_quote_policy")]
    pub quote_policy: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub format: String,
    pub source_encoding: EncodingRoute,
    pub files: Vec<ManifestFile>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestFile {
    pub source_file: String,
    pub source_sha256: String,
    pub translation_file: String,
    pub entry_count: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WorkflowStats {
    pub scanned_files: usize,
    pub script_files: usize,
    pub text_entries: usize,
}

pub fn extract_path(
    input: &Path,
    output: &Path,
    source_encoding: EncodingRoute,
    overwrite: bool,
) -> Result<WorkflowStats> {
    let input = absolute_existing(input)?;
    reject_nested_output(&input, output)?;
    check_destination(output, overwrite)?;
    let stage = staging_path(output)?;
    if stage.exists() {
        remove_path(&stage)?;
    }
    fs::create_dir_all(&stage)?;

    let result = (|| {
        let (root, files) = input_files(&input)?;
        let mut manifest = Manifest {
            schema_version: SCHEMA_VERSION,
            format: "ai5win-mes".to_owned(),
            source_encoding,
            files: Vec::new(),
        };
        let mut stats = WorkflowStats {
            scanned_files: files.len(),
            ..WorkflowStats::default()
        };
        for source_path in files {
            let bytes = fs::read(&source_path)?;
            let Ok(script) = MesScript::parse(&bytes) else {
                continue;
            };
            let relative = relative_name(&root, &source_path)?;
            let document = analyze_script(&script, &bytes, &relative, source_encoding)?;
            let translation_file = format!("{relative}.json");
            write_json(
                &stage.join(path_from_manifest(&translation_file)?),
                &document,
            )?;
            manifest.files.push(ManifestFile {
                source_file: relative,
                source_sha256: document.source_sha256.clone(),
                translation_file,
                entry_count: document.entries.len(),
            });
            stats.script_files += 1;
            stats.text_entries += document.entries.len();
        }
        manifest
            .files
            .sort_by(|left, right| left.source_file.cmp(&right.source_file));
        if manifest.files.is_empty() {
            return fail("no structurally valid AI5WIN script was found");
        }
        write_json(&stage.join("ai5win_manifest.json"), &manifest)?;
        commit_stage(&stage, output, overwrite)?;
        Ok(stats)
    })();
    if result.is_err() && stage.exists() {
        let _ = remove_path(&stage);
    }
    result
}

pub fn inject_path(
    source: &Path,
    translation: &Path,
    output: &Path,
    output_encoding: EncodingRoute,
    overwrite: bool,
) -> Result<WorkflowStats> {
    let source = absolute_existing(source)?;
    let translation = absolute_existing(translation)?;
    reject_nested_output(&source, output)?;
    reject_nested_output(&translation, output)?;
    check_destination(output, overwrite)?;
    let manifest: Manifest = read_json(&translation.join("ai5win_manifest.json"))?;
    validate_manifest(&manifest)?;
    let stage = staging_path(output)?;
    if stage.exists() {
        remove_path(&stage)?;
    }

    let result = (|| {
        if source.is_dir() {
            copy_tree(&source, &stage)?;
        } else {
            if manifest.files.len() != 1 {
                return fail("file source requires a manifest containing exactly one script");
            }
            if let Some(parent) = stage.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source, &stage)?;
        }
        let mut stats = WorkflowStats {
            scanned_files: manifest.files.len(),
            ..WorkflowStats::default()
        };
        for file in &manifest.files {
            let source_path = source_member(&source, &file.source_file)?;
            let destination_path = source_member(&stage, &file.source_file)?;
            let bytes = fs::read(&source_path)?;
            if sha256_hex(&bytes) != file.source_sha256 {
                return fail(format!(
                    "source hash mismatch for {}; use the exact files used for extraction",
                    file.source_file
                ));
            }
            let script = MesScript::parse(&bytes)
                .map_err(|error| format!("{}: {error}", file.source_file))?;
            let fresh =
                analyze_script(&script, &bytes, &file.source_file, manifest.source_encoding)?;
            let edited: TranslationDocument =
                read_json(&translation.join(path_from_manifest(&file.translation_file)?))?;
            let rebuilt = inject_document(&script, &fresh, &edited, output_encoding)?;
            MesScript::parse(&rebuilt).map_err(|error| {
                format!(
                    "rebuilt {} failed structural validation: {error}",
                    file.source_file
                )
            })?;
            verify_injected(&rebuilt, &file.source_file, &edited, output_encoding)?;
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&destination_path, rebuilt)?;
            stats.script_files += 1;
            stats.text_entries += edited.entries.len();
        }
        commit_stage(&stage, output, overwrite)?;
        Ok(stats)
    })();
    if result.is_err() && stage.exists() {
        let _ = remove_path(&stage);
    }
    result
}

pub fn verify_path(input: &Path, encoding: EncodingRoute) -> Result<WorkflowStats> {
    let input = absolute_existing(input)?;
    let (root, files) = input_files(&input)?;
    let mut stats = WorkflowStats {
        scanned_files: files.len(),
        ..WorkflowStats::default()
    };
    for path in files {
        let bytes = fs::read(&path)?;
        let Ok(script) = MesScript::parse(&bytes) else {
            continue;
        };
        let relative = relative_name(&root, &path)?;
        let document = analyze_script(&script, &bytes, &relative, encoding)?;
        stats.script_files += 1;
        stats.text_entries += document.entries.len();
    }
    if stats.script_files == 0 {
        return fail("no structurally valid AI5WIN script was found");
    }
    Ok(stats)
}

fn analyze_script(
    script: &MesScript,
    source: &[u8],
    relative: &str,
    encoding: EncodingRoute,
) -> Result<TranslationDocument> {
    let mut entries = Vec::new();
    for (message_id, range) in script.message_instruction_ranges().into_iter().enumerate() {
        let mut voice = None;
        let mut cursor = range.start;
        while cursor < range.end {
            let instruction = &script.instructions[cursor];
            if let InstructionKind::Command(command) = &instruction.kind {
                if command.command_id == Some(7) {
                    voice = command
                        .arguments
                        .iter()
                        .find_map(|argument| match argument {
                            Argument::String(slot) => {
                                decode_text(script.text_bytes(slot), encoding)
                                    .ok()
                                    .filter(|value| value.to_ascii_lowercase().ends_with(".ogg"))
                            }
                            Argument::Expression { .. } => None,
                        });
                }
            }
            let InstructionKind::Text(slot) = &instruction.kind else {
                cursor += 1;
                continue;
            };
            if slot.data_start == slot.data_end {
                cursor += 1;
                continue;
            }
            let text = decode_slot(script, slot, encoding, relative)?;

            if cursor + 2 < range.end {
                let separator = &script.instructions[cursor + 1];
                let body_instruction = &script.instructions[cursor + 2];
                if separator.opcode == 0x13 && script.instruction_bytes(separator) == [0x13, 0x00] {
                    if let InstructionKind::Text(body_slot) = &body_instruction.kind {
                        let body = decode_slot(script, body_slot, encoding, relative)?;
                        if !text.starts_with('「') && quote_policy(&body) != "none" {
                            entries.push(make_entry(
                                script,
                                relative,
                                entries.len(),
                                message_id,
                                body_instruction.offset,
                                body_slot,
                                body,
                                "dialogue",
                                voice.clone(),
                                Some((slot, text)),
                            )?);
                            cursor += 3;
                            continue;
                        }
                    }
                }
            }

            let entry_type = if is_option(script, cursor) {
                "option"
            } else if quote_policy(&text) != "none" {
                "dialogue"
            } else {
                "text"
            };
            entries.push(make_entry(
                script,
                relative,
                entries.len(),
                message_id,
                instruction.offset,
                slot,
                text,
                entry_type,
                voice.clone(),
                None,
            )?);
            cursor += 1;
        }
    }
    Ok(TranslationDocument {
        schema_version: SCHEMA_VERSION,
        format: "ai5win-mes".to_owned(),
        source_file: relative.to_owned(),
        source_sha256: sha256_hex(source),
        source_encoding: encoding,
        entries,
    })
}

#[allow(clippy::too_many_arguments)]
fn make_entry(
    script: &MesScript,
    relative: &str,
    index: usize,
    message_id: usize,
    inst_offset: u32,
    slot: &TextSlot,
    message: String,
    entry_type: &str,
    voice: Option<String>,
    name: Option<(&TextSlot, String)>,
) -> Result<TranslationEntry> {
    let (name_offset, name_size, editable_name, scr_name) = if let Some((name_slot, value)) = name {
        (
            Some(script.body_start as u64 + u64::from(name_slot.data_start)),
            Some(name_slot.data_end - name_slot.data_start),
            Some(value.clone()),
            Some(value),
        )
    } else {
        (None, None, None, None)
    };
    Ok(TranslationEntry {
        name: editable_name,
        scr_name,
        scr_msg: message.clone(),
        message: message.clone(),
        file: relative.to_owned(),
        index,
        message_id,
        offset: script.body_start as u64 + u64::from(slot.data_start),
        inst_offset,
        size: slot.data_end - slot.data_start,
        entry_type: entry_type.to_owned(),
        voice,
        name_offset,
        name_size,
        quote_policy: quote_policy(&message).to_owned(),
    })
}

fn is_option(script: &MesScript, index: usize) -> bool {
    let Some(previous) = index
        .checked_sub(1)
        .and_then(|old| script.instructions.get(old))
    else {
        return false;
    };
    let current = &script.instructions[index];
    previous.opcode == 0x10
        && previous.end == current.offset
        && script.reference_target_for_instruction(previous) == current.end.checked_add(1)
        && script.body().get(current.end as usize) == Some(&0)
}

fn quote_policy(text: &str) -> &'static str {
    if text.starts_with('「') && text.ends_with("」　") {
        "corner_fullwidth_space"
    } else if text.starts_with('「') && text.ends_with('」') {
        "corner"
    } else {
        "none"
    }
}

fn decode_slot(
    script: &MesScript,
    slot: &TextSlot,
    encoding: EncodingRoute,
    file: &str,
) -> Result<String> {
    decode_text(script.text_bytes(slot), encoding).map_err(|error| {
        format!(
            "{file}: text at body offset 0x{:X}: {error}",
            slot.data_start
        )
        .into()
    })
}

fn inject_document(
    script: &MesScript,
    fresh: &TranslationDocument,
    edited: &TranslationDocument,
    output_encoding: EncodingRoute,
) -> Result<Vec<u8>> {
    validate_document_header(fresh, edited)?;
    if fresh.entries.len() != edited.entries.len() {
        return fail(format!(
            "{}: entry count changed from {} to {}",
            fresh.source_file,
            fresh.entries.len(),
            edited.entries.len()
        ));
    }
    let force_reencode = output_encoding == EncodingRoute::Gbk;
    let mut replacements = Vec::new();
    let mut owned_offsets = HashSet::new();
    for (fresh_entry, edited_entry) in fresh.entries.iter().zip(&edited.entries) {
        validate_entry_metadata(fresh_entry, edited_entry)?;
        validate_quote_policy(edited_entry)?;
        let body_start = body_offset(script, fresh_entry.offset)?;
        if !owned_offsets.insert(body_start) {
            return fail(format!(
                "{}: duplicate text ownership at body offset 0x{body_start:X}",
                fresh.source_file
            ));
        }
        if force_reencode || edited_entry.message != fresh_entry.scr_msg {
            let encoded = encode_text(&edited_entry.message, output_encoding).map_err(|error| {
                format!(
                    "{} entry #{} message: {error}",
                    fresh.source_file, fresh_entry.index
                )
            })?;
            replacements.push(Replacement {
                start: body_start,
                end: body_start + fresh_entry.size,
                data: encoded,
            });
        }
        match (
            &fresh_entry.scr_name,
            &edited_entry.name,
            fresh_entry.name_offset,
        ) {
            (Some(original), Some(value), Some(file_offset)) => {
                let start = body_offset(script, file_offset)?;
                if !owned_offsets.insert(start) {
                    return fail(format!(
                        "{}: duplicate name ownership at body offset 0x{start:X}",
                        fresh.source_file
                    ));
                }
                if force_reencode || value != original {
                    let encoded = encode_text(value, output_encoding).map_err(|error| {
                        format!(
                            "{} entry #{} name: {error}",
                            fresh.source_file, fresh_entry.index
                        )
                    })?;
                    replacements.push(Replacement {
                        start,
                        end: start + fresh_entry.name_size.expect("validated name size"),
                        data: encoded,
                    });
                }
            }
            (None, None, None) => {}
            _ => {
                return fail(format!(
                    "{} entry #{} cannot add or remove the structural name slot",
                    fresh.source_file, fresh_entry.index
                ));
            }
        }
    }
    script.rebuild(replacements)
}

fn validate_document_header(
    fresh: &TranslationDocument,
    edited: &TranslationDocument,
) -> Result<()> {
    if edited.schema_version != SCHEMA_VERSION
        || edited.format != "ai5win-mes"
        || edited.source_file != fresh.source_file
        || edited.source_sha256 != fresh.source_sha256
        || edited.source_encoding != fresh.source_encoding
    {
        return fail(format!(
            "{}: translation document metadata does not match the extracted source",
            fresh.source_file
        ));
    }
    Ok(())
}

fn validate_entry_metadata(fresh: &TranslationEntry, edited: &TranslationEntry) -> Result<()> {
    let valid = edited.scr_msg == fresh.scr_msg
        && edited.scr_name == fresh.scr_name
        && edited.file == fresh.file
        && edited.index == fresh.index
        && edited.message_id == fresh.message_id
        && edited.offset == fresh.offset
        && edited.inst_offset == fresh.inst_offset
        && edited.size == fresh.size
        && edited.entry_type == fresh.entry_type
        && edited.voice == fresh.voice
        && edited.name_offset == fresh.name_offset
        && edited.name_size == fresh.name_size
        && edited.quote_policy == fresh.quote_policy;
    if !valid {
        return fail(format!(
            "{} entry #{}: immutable metadata, scr_msg, or _scr_name was changed",
            fresh.file, fresh.index
        ));
    }
    Ok(())
}

fn validate_quote_policy(entry: &TranslationEntry) -> Result<()> {
    if entry.quote_policy != quote_policy(&entry.message) {
        return fail(format!(
            "{} entry #{}: message must preserve its {:?} quote policy",
            entry.file, entry.index, entry.quote_policy
        ));
    }
    Ok(())
}

fn verify_injected(
    rebuilt: &[u8],
    relative: &str,
    edited: &TranslationDocument,
    encoding: EncodingRoute,
) -> Result<()> {
    let script = MesScript::parse(rebuilt)?;
    let checked = analyze_script(&script, rebuilt, relative, encoding)?;
    if checked.entries.len() != edited.entries.len() {
        return fail(format!(
            "{relative}: rebuilt text count changed from {} to {}",
            edited.entries.len(),
            checked.entries.len()
        ));
    }
    for (actual, expected) in checked.entries.iter().zip(&edited.entries) {
        if actual.message != expected.message || actual.name != expected.name {
            return fail(format!(
                "{relative} entry #{} failed Unicode verification after injection",
                expected.index
            ));
        }
    }
    Ok(())
}

fn body_offset(script: &MesScript, file_offset: u64) -> Result<u32> {
    let body_start = u64::try_from(script.body_start)?;
    let relative = file_offset
        .checked_sub(body_start)
        .ok_or("translation offset precedes MES body")?;
    u32::try_from(relative).map_err(|_| "translation offset exceeds u32".into())
}

fn validate_manifest(manifest: &Manifest) -> Result<()> {
    if manifest.schema_version != SCHEMA_VERSION || manifest.format != "ai5win-mes" {
        return fail("unsupported or malformed AI5WIN manifest");
    }
    let mut sources = HashSet::new();
    let mut translations = HashSet::new();
    for file in &manifest.files {
        path_from_manifest(&file.source_file)?;
        path_from_manifest(&file.translation_file)?;
        if !sources.insert(&file.source_file) || !translations.insert(&file.translation_file) {
            return fail("manifest contains duplicate source or translation paths");
        }
    }
    Ok(())
}

fn input_files(input: &Path) -> Result<(PathBuf, Vec<PathBuf>)> {
    if input.is_file() {
        let root = input
            .parent()
            .ok_or("input file has no parent directory")?
            .to_path_buf();
        return Ok((root, vec![input.to_path_buf()]));
    }
    let mut files = Vec::new();
    collect_files(input, &mut files)?;
    files.sort();
    Ok((input.to_path_buf(), files))
}

fn collect_files(root: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, output)?;
        } else if path.is_file() {
            output.push(path);
        }
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn relative_name(root: &Path, path: &Path) -> Result<String> {
    let relative = path.strip_prefix(root)?;
    let value = relative
        .to_str()
        .ok_or("non-Unicode paths are not supported in translation metadata")?;
    Ok(value.replace('\\', "/"))
}

fn path_from_manifest(value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        })
    {
        return fail(format!("unsafe relative path in manifest: {value:?}"));
    }
    Ok(path.to_path_buf())
}

fn source_member(root: &Path, relative: &str) -> Result<PathBuf> {
    if root.is_file() || (!root.exists() && root.extension().is_some()) {
        return Ok(root.to_path_buf());
    }
    Ok(root.join(path_from_manifest(relative)?))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut data = serde_json::to_vec_pretty(value)?;
    data.push(b'\n');
    fs::write(path, data)?;
    Ok(())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()).into())
}

fn absolute_existing(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).map_err(|error| format!("{}: {error}", path.display()).into())
}

fn check_destination(path: &Path, overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        return fail(format!(
            "output already exists: {}; pass --overwrite to replace it",
            path.display()
        ));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn commit_stage(stage: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if output.exists() {
        if !overwrite {
            return fail(format!(
                "output appeared while working: {}",
                output.display()
            ));
        }
        remove_path(output)?;
    }
    fs::rename(stage, output)?;
    Ok(())
}

fn remove_path(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn staging_path(output: &Path) -> Result<PathBuf> {
    let name = output
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("output path must have a Unicode file name")?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(output.with_file_name(format!(".{name}.ai5win-stage-{stamp}")))
}

fn reject_nested_output(source: &Path, output: &Path) -> Result<()> {
    let absolute_output = if output.is_absolute() {
        output.to_path_buf()
    } else {
        std::env::current_dir()?.join(output)
    };
    if absolute_output == source {
        return fail("output must be different from every input path");
    }
    if source.is_dir() && absolute_output.starts_with(source) {
        return fail("output must not be inside the source directory");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named_sample() -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(&[0x17, 0, 0, 0, 0, 0x01]);
        body.extend_from_slice(&encode_text("巴", EncodingRoute::Cp932).unwrap());
        body.extend_from_slice(&[0, 0x13, 0, 0x01]);
        body.extend_from_slice(&encode_text("「こんにちは」", EncodingRoute::Cp932).unwrap());
        body.extend_from_slice(&[0, 0]);
        let mut source = Vec::new();
        source.extend_from_slice(&1u32.to_le_bytes());
        source.extend_from_slice(&0u32.to_le_bytes());
        source.extend_from_slice(&body);
        source
    }

    #[test]
    fn manifest_paths_reject_escape() {
        assert!(path_from_manifest("../escape").is_err());
        assert!(path_from_manifest("C:/absolute").is_err());
        assert!(path_from_manifest("safe/name.MES.json").is_ok());
    }

    #[test]
    fn quote_policy_is_explicit() {
        assert_eq!(quote_policy("「text」"), "corner");
        assert_eq!(quote_policy("「text」　"), "corner_fullwidth_space");
        assert_eq!(quote_policy("text"), "none");
    }

    #[test]
    fn cp932_unchanged_is_byte_exact() {
        let source = named_sample();
        let script = MesScript::parse(&source).unwrap();
        let document = analyze_script(&script, &source, "sample", EncodingRoute::Cp932).unwrap();
        let rebuilt = inject_document(&script, &document, &document, EncodingRoute::Cp932).unwrap();
        assert_eq!(rebuilt, source);
    }

    #[test]
    fn gbk_reencodes_unchanged_name_and_message() {
        let source = named_sample();
        let script = MesScript::parse(&source).unwrap();
        let document = analyze_script(&script, &source, "sample", EncodingRoute::Cp932).unwrap();
        let rebuilt = inject_document(&script, &document, &document, EncodingRoute::Gbk).unwrap();
        assert_ne!(rebuilt, source);
        let reparsed = MesScript::parse(&rebuilt).unwrap();
        let checked = analyze_script(&reparsed, &rebuilt, "sample", EncodingRoute::Gbk).unwrap();
        assert_eq!(checked.entries[0].name.as_deref(), Some("巴"));
        assert_eq!(checked.entries[0].message, "「こんにちは」");
    }

    #[test]
    fn gbk_accepts_modified_chinese_and_keeps_immutable_source() {
        let source = named_sample();
        let script = MesScript::parse(&source).unwrap();
        let fresh = analyze_script(&script, &source, "sample", EncodingRoute::Cp932).unwrap();
        let mut edited = fresh.clone();
        edited.entries[0].name = Some("巴译".to_owned());
        edited.entries[0].message = "「中文测试」".to_owned();
        let rebuilt = inject_document(&script, &fresh, &edited, EncodingRoute::Gbk).unwrap();
        verify_injected(&rebuilt, "sample", &edited, EncodingRoute::Gbk).unwrap();
        assert_eq!(edited.entries[0].scr_name.as_deref(), Some("巴"));
        assert_eq!(edited.entries[0].scr_msg, "「こんにちは」");
    }
}
