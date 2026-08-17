use crate::archive::{DrsArchive, prepare_directory, read_archive, write_new_or_overwrite};
use crate::script::{IsfScript, Replacement};
use crate::text::{
    EncodingRoute, decode_raw_cp932, encode_lien_text, encode_raw_text, parse_0x2b_stream,
};
use crate::{Result, fail, sha256_hex};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Component, Path};

pub const TRANSLATION_MANIFEST: &str = "lien-translation-manifest.json";

#[derive(Clone, Debug, Default)]
pub struct ExtractStats {
    pub scanned_files: usize,
    pub json_files: usize,
    pub extracted_entries: usize,
    pub excluded_opcodes: usize,
    pub multipart_opcodes: usize,
    pub warnings: usize,
}

#[derive(Clone, Debug, Default)]
pub struct InjectStats {
    pub json_files: usize,
    pub json_entries: usize,
    pub patched_messages: usize,
    pub patched_names: usize,
    pub unchanged: usize,
    pub rebuilt_scripts: usize,
    pub output_bytes: usize,
}

#[derive(Clone, Debug, Default)]
pub struct VerifyStats {
    pub archive_files: usize,
    pub parsed_scripts: usize,
    pub opcode_count: usize,
    pub text_entries: usize,
    pub excluded_opcodes: usize,
    pub script_roundtrip_exact: usize,
    pub archive_roundtrip_exact: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TranslationManifest {
    pub schema_version: u32,
    pub format: String,
    pub project: String,
    pub source_archive_file: String,
    pub source_archive_sha256: String,
    pub scripts: Vec<TranslationScriptRef>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TranslationScriptRef {
    pub archive_index: usize,
    pub source_file: String,
    pub source_sha256: String,
    pub json_file: String,
    pub entry_count: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TranslationDocument {
    pub schema_version: u32,
    pub format: String,
    pub source_file: String,
    pub source_sha256: String,
    pub entries: Vec<TranslationEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TranslationEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    pub scr_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scr_msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scr_msg_parts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_parts: Option<Vec<String>>,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_op_index")]
    pub op_index: usize,
    #[serde(rename = "_inst_offset")]
    pub inst_offset: u32,
    #[serde(rename = "_offset", skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
    #[serde(rename = "_size", skip_serializing_if = "Option::is_none")]
    pub size: Option<usize>,
    #[serde(rename = "_part_offsets", skip_serializing_if = "Option::is_none")]
    pub part_offsets: Option<Vec<usize>>,
    #[serde(rename = "_part_sizes", skip_serializing_if = "Option::is_none")]
    pub part_sizes: Option<Vec<usize>>,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_opcode")]
    pub opcode: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_policy")]
    pub policy: String,
    #[serde(rename = "_part_count")]
    pub part_count: usize,
    #[serde(rename = "_name_op_index", skip_serializing_if = "Option::is_none")]
    pub name_op_index: Option<usize>,
    #[serde(rename = "_name_offset", skip_serializing_if = "Option::is_none")]
    pub name_offset: Option<usize>,
    #[serde(rename = "_name_size", skip_serializing_if = "Option::is_none")]
    pub name_size: Option<usize>,
}

#[derive(Clone, Debug)]
struct PendingName {
    value: String,
    op_index: usize,
    offset: usize,
    size: usize,
}

pub fn unpack_archive(input: &Path, output: &Path, overwrite: bool) -> Result<usize> {
    let (source, archive) = read_archive(input)?;
    let count = archive.entries.len();
    archive.unpack_to(&source, output, overwrite)?;
    Ok(count)
}

pub fn pack_archive(
    input_directory: &Path,
    manifest_path: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<(usize, usize)> {
    ensure_distinct_paths(manifest_path, output, "DRS manifest", "pack output")?;
    let archive = DrsArchive::from_unpacked(input_directory, manifest_path)?;
    for entry in &archive.entries {
        ensure_distinct_paths(
            &input_directory.join(&entry.name),
            output,
            "DRS member",
            "pack output",
        )?;
    }
    let member_count = archive.entries.len();
    let data = archive.build()?;
    let output_bytes = data.len();
    write_new_or_overwrite(output, &data, overwrite)?;
    Ok((member_count, output_bytes))
}

pub fn extract_archive(input: &Path, output: &Path, overwrite: bool) -> Result<ExtractStats> {
    let (source_data, archive) = read_archive_source(input)?;
    let mut stats = ExtractStats {
        scanned_files: archive.entries.len(),
        ..ExtractStats::default()
    };
    let mut documents = Vec::new();
    let mut refs = Vec::new();

    for (archive_index, member) in archive.entries.iter().enumerate() {
        let (document, file_stats) = extract_script(&member.name, &member.data)?;
        stats.excluded_opcodes += file_stats.excluded_opcodes;
        stats.multipart_opcodes += file_stats.multipart_opcodes;
        stats.warnings += file_stats.warnings;
        if document.entries.is_empty() {
            continue;
        }
        let json_file = format!("{archive_index:03}_{}.json", member.name);
        validate_manifest_relative_file(&json_file)?;
        stats.json_files += 1;
        stats.extracted_entries += document.entries.len();
        refs.push(TranslationScriptRef {
            archive_index,
            source_file: member.name.clone(),
            source_sha256: sha256_hex(&member.data),
            json_file: json_file.clone(),
            entry_count: document.entries.len(),
        });
        documents.push((json_file, document));
    }

    let source_archive_file = input
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ISF".to_owned());
    let manifest = TranslationManifest {
        schema_version: 1,
        format: "lien-isf-translation-workspace".to_owned(),
        project: "Lien".to_owned(),
        source_archive_file,
        source_archive_sha256: sha256_hex(&source_data),
        scripts: refs,
    };

    prepare_directory(output, overwrite)?;
    for (json_file, document) in documents {
        write_json(&output.join(json_file), &document, overwrite)?;
    }
    write_json(&output.join(TRANSLATION_MANIFEST), &manifest, overwrite)?;
    Ok(stats)
}

pub fn inject_archive(
    source_path: &Path,
    translation_directory: &Path,
    output: &Path,
    route: EncodingRoute,
    overwrite: bool,
) -> Result<InjectStats> {
    ensure_distinct_paths(source_path, output, "source archive", "inject output")?;
    let (source_data, mut archive) = read_archive_source(source_path)?;
    let manifest_path = translation_directory.join(TRANSLATION_MANIFEST);
    let manifest: TranslationManifest = read_json(&manifest_path)?;
    validate_translation_manifest(&manifest)?;
    let actual_archive_hash = sha256_hex(&source_data);
    if manifest.source_archive_sha256 != actual_archive_hash {
        return fail(format!(
            "translation manifest source SHA-256 mismatch: manifest={}, source={actual_archive_hash}",
            manifest.source_archive_sha256
        ));
    }

    let mut stats = InjectStats::default();
    let mut seen_archive_indices = HashSet::with_capacity(manifest.scripts.len());
    for script_ref in &manifest.scripts {
        if !seen_archive_indices.insert(script_ref.archive_index) {
            return fail(format!(
                "translation manifest repeats archive index {}",
                script_ref.archive_index
            ));
        }
        validate_manifest_relative_file(&script_ref.json_file)?;
        let member = archive
            .entries
            .get_mut(script_ref.archive_index)
            .ok_or_else(|| {
                format!(
                    "translation manifest archive index {} is outside the source archive",
                    script_ref.archive_index
                )
            })?;
        if member.name != script_ref.source_file {
            return fail(format!(
                "translation manifest member mismatch at archive index {}: manifest={:?}, source={:?}",
                script_ref.archive_index, script_ref.source_file, member.name
            ));
        }
        let member_hash = sha256_hex(&member.data);
        if member_hash != script_ref.source_sha256 {
            return fail(format!(
                "source script SHA-256 mismatch for {}: manifest={}, source={member_hash}",
                member.name, script_ref.source_sha256
            ));
        }
        let document_path = translation_directory.join(&script_ref.json_file);
        let document: TranslationDocument = read_json(&document_path)?;
        if document.entries.len() != script_ref.entry_count {
            return fail(format!(
                "translation entry count mismatch for {}: manifest={}, JSON={}",
                member.name,
                script_ref.entry_count,
                document.entries.len()
            ));
        }
        let result = apply_document(&member.name, &member.data, &document, route)?;
        member.data = result.data;
        stats.json_files += 1;
        stats.json_entries += document.entries.len();
        stats.patched_messages += result.patched_messages;
        stats.patched_names += result.patched_names;
        stats.unchanged += result.unchanged;
        if result.rebuilt {
            stats.rebuilt_scripts += 1;
        }
    }

    let output_data = archive.build()?;
    stats.output_bytes = output_data.len();
    write_new_or_overwrite(output, &output_data, overwrite)?;
    Ok(stats)
}

pub fn verify_archive(input: &Path, verify_source_text: bool) -> Result<VerifyStats> {
    let (source_data, archive) = read_archive_source(input)?;
    let mut stats = VerifyStats {
        archive_files: archive.entries.len(),
        ..VerifyStats::default()
    };
    for member in &archive.entries {
        let script = IsfScript::parse(&member.data)
            .map_err(|error| format!("failed to parse {}: {error}", member.name))?;
        stats.parsed_scripts += 1;
        stats.opcode_count += script.opcodes.len();
        let rebuilt = script.rebuild()?;
        if rebuilt != member.data {
            return fail(format!(
                "structured no-change rebuild is not byte-exact for {}",
                member.name
            ));
        }
        stats.script_roundtrip_exact += 1;
        if verify_source_text {
            let (document, file_stats) = extract_script(&member.name, &member.data)?;
            stats.text_entries += document.entries.len();
            stats.excluded_opcodes += file_stats.excluded_opcodes;
        }
    }
    let rebuilt_archive = archive.build()?;
    if rebuilt_archive != source_data {
        return fail("DRS no-change rebuild is not byte-exact");
    }
    stats.archive_roundtrip_exact = true;
    Ok(stats)
}

fn read_archive_source(input: &Path) -> Result<(Vec<u8>, DrsArchive)> {
    if input.is_dir() {
        let manifest = input.join("lien-drs-manifest.json");
        if !manifest.is_file() {
            return fail(format!(
                "input is an unpacked directory but has no lien-drs-manifest.json: {}",
                input.display()
            ));
        }
        let archive = DrsArchive::from_unpacked(input, &manifest)?;
        let rebuilt = archive.build()?;
        return Ok((rebuilt, archive));
    }
    if !input.is_file() {
        return fail(format!(
            "input path is not a file or directory: {}",
            input.display()
        ));
    }
    read_archive(input)
}

#[derive(Default)]
struct PerFileExtractStats {
    excluded_opcodes: usize,
    multipart_opcodes: usize,
    warnings: usize,
}

fn extract_script(
    file_name: &str,
    data: &[u8],
) -> Result<(TranslationDocument, PerFileExtractStats)> {
    let script =
        IsfScript::parse(data).map_err(|error| format!("failed to parse {file_name}: {error}"))?;
    let mut entries = Vec::new();
    let mut pending_name: Option<PendingName> = None;
    let mut stats = PerFileExtractStats::default();

    for (op_index, opcode) in script.opcodes.iter().enumerate() {
        match opcode.opcode {
            0x2B => {
                let stream = parse_0x2b_stream(&opcode.content).map_err(|error| {
                    format!(
                        "{file_name}: opcode #{op_index} at body offset 0x{:X}: {error}",
                        opcode.original_offset
                    )
                })?;
                if stream.has_excluded_0x02 {
                    stats.excluded_opcodes += 1;
                    pending_name = None;
                    continue;
                }
                if stream.segments.len() > 1 {
                    stats.multipart_opcodes += 1;
                }
                let mut message_segments = Vec::new();
                for segment in stream.segments {
                    if let Some(name) = standalone_name(&segment.text) {
                        if !message_segments.is_empty() {
                            return fail(format!(
                                "{file_name}: opcode #{op_index} contains a name marker after message text"
                            ));
                        }
                        if pending_name.is_some() {
                            stats.warnings += 1;
                        }
                        pending_name = Some(PendingName {
                            value: name.to_owned(),
                            op_index,
                            offset: segment.start,
                            size: segment.end - segment.start,
                        });
                        continue;
                    }
                    message_segments.push(segment);
                }
                if !message_segments.is_empty() {
                    let name = pending_name.take();
                    let index = entries.len();
                    let part_count = message_segments.len();
                    let texts: Vec<String> = message_segments
                        .iter()
                        .map(|segment| segment.text.clone())
                        .collect();
                    let offsets: Vec<usize> = message_segments
                        .iter()
                        .map(|segment| segment.start)
                        .collect();
                    let sizes: Vec<usize> = message_segments
                        .iter()
                        .map(|segment| segment.end - segment.start)
                        .collect();
                    let multipart = part_count > 1;
                    entries.push(TranslationEntry {
                        name: name.as_ref().map(|record| record.value.clone()),
                        scr_name: name.as_ref().map(|record| record.value.clone()),
                        scr_msg: Some(texts.concat()),
                        message: Some(texts.concat()),
                        scr_msg_parts: multipart.then(|| texts.clone()),
                        message_parts: multipart.then_some(texts),
                        file: file_name.to_owned(),
                        index,
                        op_index,
                        inst_offset: opcode.original_offset,
                        offset: (!multipart).then_some(offsets[0]),
                        size: (!multipart).then_some(sizes[0]),
                        part_offsets: multipart.then_some(offsets),
                        part_sizes: multipart.then_some(sizes),
                        entry_type: "dialogue".to_owned(),
                        opcode: "0x2B".to_owned(),
                        encoding: "lien_cp932".to_owned(),
                        policy: "relocate".to_owned(),
                        part_count,
                        name_op_index: name.as_ref().map(|record| record.op_index),
                        name_offset: name.as_ref().map(|record| record.offset),
                        name_size: name.as_ref().map(|record| record.size),
                    });
                }
            }
            0x15 => {
                if let Some((start, end, text)) =
                    extract_0x15(&opcode.content).map_err(|error| {
                        format!(
                            "{file_name}: opcode #{op_index} at body offset 0x{:X}: {error}",
                            opcode.original_offset
                        )
                    })?
                {
                    if pending_name.take().is_some() {
                        stats.warnings += 1;
                    }
                    let index = entries.len();
                    entries.push(TranslationEntry {
                        name: None,
                        scr_name: None,
                        scr_msg: Some(text.clone()),
                        message: Some(text),
                        scr_msg_parts: None,
                        message_parts: None,
                        file: file_name.to_owned(),
                        index,
                        op_index,
                        inst_offset: opcode.original_offset,
                        offset: Some(start),
                        size: Some(end - start),
                        part_offsets: None,
                        part_sizes: None,
                        entry_type: "choice".to_owned(),
                        opcode: "0x15".to_owned(),
                        encoding: "cp932".to_owned(),
                        policy: "relocate".to_owned(),
                        part_count: 1,
                        name_op_index: None,
                        name_offset: None,
                        name_size: None,
                    });
                }
            }
            _ => {}
        }
    }
    if pending_name.is_some() {
        stats.warnings += 1;
    }

    Ok((
        TranslationDocument {
            schema_version: 1,
            format: "lien-isf-translation".to_owned(),
            source_file: file_name.to_owned(),
            source_sha256: sha256_hex(data),
            entries,
        },
        stats,
    ))
}

fn extract_0x15(content: &[u8]) -> Result<Option<(usize, usize, String)>> {
    const START: usize = 17;
    if content.len() <= START {
        return Ok(None);
    }
    let end = content[START..]
        .iter()
        .position(|&byte| byte == 0)
        .map(|relative| START + relative)
        .unwrap_or(content.len());
    if end == START {
        return Ok(None);
    }
    let text = decode_raw_cp932(&content[START..end])?;
    Ok(Some((START, end, text)))
}

fn standalone_name(text: &str) -> Option<&str> {
    let inner = text.strip_prefix('【')?.strip_suffix('】')?;
    if inner.is_empty() || inner.contains(['【', '】', '\r', '\n']) {
        None
    } else {
        Some(inner)
    }
}

struct ApplyResult {
    data: Vec<u8>,
    patched_messages: usize,
    patched_names: usize,
    unchanged: usize,
    rebuilt: bool,
}

fn apply_document(
    file_name: &str,
    source_data: &[u8],
    document: &TranslationDocument,
    route: EncodingRoute,
) -> Result<ApplyResult> {
    if document.schema_version != 1 || document.format != "lien-isf-translation" {
        return fail(format!(
            "unsupported translation JSON format for {file_name}"
        ));
    }
    if document.source_file != file_name {
        return fail(format!(
            "translation JSON source mismatch: JSON={:?}, source={file_name:?}",
            document.source_file
        ));
    }
    let source_hash = sha256_hex(source_data);
    if document.source_sha256 != source_hash {
        return fail(format!(
            "translation JSON source SHA-256 mismatch for {file_name}: JSON={}, source={source_hash}",
            document.source_sha256
        ));
    }
    let (current, _) = extract_script(file_name, source_data)?;
    if current.entries.len() != document.entries.len() {
        return fail(format!(
            "translation JSON entry count mismatch for {file_name}: JSON={}, source={}",
            document.entries.len(),
            current.entries.len()
        ));
    }

    let mut script = IsfScript::parse(source_data)?;
    let mut replacements: BTreeMap<usize, Vec<Replacement>> = BTreeMap::new();
    let mut name_writes: HashMap<(usize, usize, usize), String> = HashMap::new();
    let mut patched_messages = 0usize;
    let mut unchanged = 0usize;

    for (expected_index, (translated, source)) in document
        .entries
        .iter()
        .zip(current.entries.iter())
        .enumerate()
    {
        validate_entry_metadata(file_name, expected_index, translated, source)?;
        if translated.scr_msg != source.scr_msg || translated.scr_msg_parts != source.scr_msg_parts
        {
            return fail(format!(
                "{file_name}: entry #{expected_index} scr_msg/scr_msg_parts was modified or does not match the source"
            ));
        }
        if translated.scr_name != source.scr_name {
            return fail(format!(
                "{file_name}: entry #{expected_index} _scr_name was modified or does not match the source"
            ));
        }
        let translated_name = match (&translated.name, &source.name) {
            (None, None) => None,
            (Some(value), Some(_)) => Some(value),
            _ => {
                return fail(format!(
                    "{file_name}: entry #{expected_index} cannot add or remove a name field"
                ));
            }
        };
        if let (Some(name), Some(source_name)) = (translated_name, source.name.as_ref())
            && should_reencode(route, name, source_name)
        {
            validate_name(name, file_name, expected_index)?;
            let key = (
                source.name_op_index.expect("source name op index"),
                source.name_offset.expect("source name offset"),
                source.name_size.expect("source name size"),
            );
            if let Some(existing) = name_writes.insert(key, name.clone())
                && existing != *name
            {
                return fail(format!(
                    "{file_name}: conflicting writes for the same name slot: {existing:?} and {name:?}"
                ));
            }
        }

        if source.part_count == 1 {
            if translated.message_parts.is_some() || translated.scr_msg_parts.is_some() {
                return fail(format!(
                    "{file_name}: entry #{expected_index} is single-part and cannot use message_parts"
                ));
            }
            let scr_msg = translated.scr_msg.as_ref().ok_or_else(|| {
                format!("{file_name}: entry #{expected_index} is missing scr_msg")
            })?;
            let message = translated.message.as_ref().ok_or_else(|| {
                format!("{file_name}: entry #{expected_index} is missing message")
            })?;
            if !should_reencode(route, message, scr_msg) {
                unchanged += 1;
            } else {
                let encoded = encode_entry_text(
                    file_name,
                    expected_index,
                    &translated.opcode,
                    message,
                    route,
                )?;
                let offset = source.offset.expect("single-part source offset");
                let size = source.size.expect("single-part source size");
                replacements
                    .entry(source.op_index)
                    .or_default()
                    .push(Replacement {
                        start: offset,
                        end: offset + size,
                        data: encoded,
                    });
                patched_messages += 1;
            }
        } else {
            let scr_parts = translated.scr_msg_parts.as_ref().ok_or_else(|| {
                format!("{file_name}: entry #{expected_index} is missing scr_msg_parts")
            })?;
            let message_parts = translated.message_parts.as_ref().ok_or_else(|| {
                format!("{file_name}: entry #{expected_index} is missing message_parts")
            })?;
            let scr_msg = translated.scr_msg.as_ref().ok_or_else(|| {
                format!("{file_name}: entry #{expected_index} is missing scr_msg")
            })?;
            let message = translated.message.as_ref().ok_or_else(|| {
                format!("{file_name}: entry #{expected_index} is missing message")
            })?;
            if scr_msg != &scr_parts.concat() {
                return fail(format!(
                    "{file_name}: entry #{expected_index} scr_msg does not match scr_msg_parts"
                ));
            }
            if message != &message_parts.concat() {
                return fail(format!(
                    "{file_name}: entry #{expected_index} message does not match message_parts"
                ));
            }
            let offsets = source
                .part_offsets
                .as_ref()
                .expect("multipart source offsets");
            let sizes = source.part_sizes.as_ref().expect("multipart source sizes");
            if scr_parts.len() != source.part_count
                || message_parts.len() != source.part_count
                || offsets.len() != source.part_count
                || sizes.len() != source.part_count
            {
                return fail(format!(
                    "{file_name}: entry #{expected_index} must retain exactly {} message parts",
                    source.part_count
                ));
            }
            let mut changed = false;
            for part_index in 0..source.part_count {
                if !should_reencode(route, &message_parts[part_index], &scr_parts[part_index]) {
                    continue;
                }
                let encoded = encode_entry_text(
                    file_name,
                    expected_index,
                    &translated.opcode,
                    &message_parts[part_index],
                    route,
                )
                .map_err(|error| format!("{error}; message_parts index {part_index}"))?;
                replacements
                    .entry(source.op_index)
                    .or_default()
                    .push(Replacement {
                        start: offsets[part_index],
                        end: offsets[part_index] + sizes[part_index],
                        data: encoded,
                    });
                changed = true;
            }
            if changed {
                patched_messages += 1;
            } else {
                unchanged += 1;
            }
        }
    }

    let patched_names = name_writes.len();
    for ((op_index, offset, size), name) in name_writes {
        let wrapped = format!("【{name}】");
        let encoded = encode_lien_text(&wrapped, route)
            .map_err(|error| format!("{file_name}: name {name:?}: {error}"))?;
        replacements.entry(op_index).or_default().push(Replacement {
            start: offset,
            end: offset + size,
            data: encoded,
        });
    }

    if replacements.is_empty() {
        return Ok(ApplyResult {
            data: source_data.to_vec(),
            patched_messages,
            patched_names,
            unchanged,
            rebuilt: false,
        });
    }
    for (op_index, op_replacements) in replacements {
        script.apply_replacements(op_index, op_replacements)?;
    }
    let rebuilt = script.rebuild()?;
    IsfScript::parse(&rebuilt)
        .map_err(|error| format!("rebuilt {file_name} failed structural validation: {error}"))?;
    Ok(ApplyResult {
        data: rebuilt,
        patched_messages,
        patched_names,
        unchanged,
        rebuilt: true,
    })
}

fn encode_entry_text(
    file_name: &str,
    entry_index: usize,
    opcode: &str,
    text: &str,
    route: EncodingRoute,
) -> Result<Vec<u8>> {
    match opcode {
        "0x2B" => encode_lien_text(text, route),
        "0x15" => encode_raw_text(text, route),
        other => fail(format!(
            "{file_name}: entry #{entry_index} has unsupported opcode {other}"
        )),
    }
    .map_err(|error| format!("{file_name}: entry #{entry_index}: {error}").into())
}

fn should_reencode(route: EncodingRoute, translated: &str, source: &str) -> bool {
    route == EncodingRoute::Gbk || translated != source
}

fn validate_entry_metadata(
    file_name: &str,
    expected_index: usize,
    translated: &TranslationEntry,
    source: &TranslationEntry,
) -> Result<()> {
    let valid = translated.file == file_name
        && translated.index == expected_index
        && translated.index == source.index
        && translated.op_index == source.op_index
        && translated.inst_offset == source.inst_offset
        && translated.offset == source.offset
        && translated.size == source.size
        && translated.part_offsets == source.part_offsets
        && translated.part_sizes == source.part_sizes
        && translated.entry_type == source.entry_type
        && translated.opcode == source.opcode
        && translated.encoding == source.encoding
        && translated.policy == source.policy
        && translated.part_count == source.part_count
        && translated.name_op_index == source.name_op_index
        && translated.name_offset == source.name_offset
        && translated.name_size == source.name_size;
    if !valid {
        return fail(format!(
            "{file_name}: entry #{expected_index} location or validation metadata does not match the source"
        ));
    }
    Ok(())
}

fn validate_name(name: &str, file_name: &str, entry_index: usize) -> Result<()> {
    if name.is_empty() || name.contains(['【', '】', '\0', '\r', '\n']) {
        return fail(format!(
            "{file_name}: entry #{entry_index} has an invalid name; names must be non-empty and cannot contain brackets, NUL, CR, or LF"
        ));
    }
    Ok(())
}

fn validate_translation_manifest(manifest: &TranslationManifest) -> Result<()> {
    if manifest.schema_version != 1
        || manifest.format != "lien-isf-translation-workspace"
        || manifest.project != "Lien"
    {
        return fail("unsupported or invalid Lien translation manifest");
    }
    Ok(())
}

fn validate_manifest_relative_file(value: &str) -> Result<()> {
    let path = Path::new(value);
    let mut components = path.components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(()),
        _ => fail(format!("unsafe manifest-relative file path: {value:?}")),
    }
}

fn ensure_distinct_paths(
    left: &Path,
    right: &Path,
    left_role: &str,
    right_role: &str,
) -> Result<()> {
    let left_key = comparable_path(left)?;
    let right_key = comparable_path(right)?;
    if left_key == right_key {
        return fail(format!(
            "{right_role} must not overwrite {left_role}: {}",
            right.display()
        ));
    }
    Ok(())
}

fn comparable_path(path: &Path) -> Result<String> {
    let absolute = if path.exists() {
        fs::canonicalize(path)?
    } else {
        std::path::absolute(path)?
    };
    Ok(absolute.to_string_lossy().replace('/', "\\").to_lowercase())
}

fn write_json<T: Serialize>(path: &Path, value: &T, overwrite: bool) -> Result<()> {
    let mut data = serde_json::to_vec_pretty(value)?;
    data.push(b'\n');
    write_new_or_overwrite(path, &data, overwrite)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let data = fs::read(path)
        .map_err(|error| format!("failed to read JSON {}: {error}", path.display()))?;
    serde_json::from_slice(&data)
        .map_err(|error| format!("invalid JSON {}: {error}", path.display()).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encrypted_script_with_name_and_punctuation() -> Vec<u8> {
        let name = encode_lien_text("【玄照】", EncodingRoute::Cp932Native).unwrap();
        let message = encode_lien_text("。", EncodingRoute::Cp932Native).unwrap();
        let mut body = Vec::new();
        for payload in [name, message] {
            let mut content = vec![0x00, 0xFF];
            content.extend_from_slice(&payload);
            content.push(0x00);
            body.push(0x2B);
            body.push((content.len() + 2) as u8);
            body.extend_from_slice(&content);
        }
        let mut decoded = Vec::new();
        decoded.extend_from_slice(&8u32.to_le_bytes());
        decoded.extend_from_slice(&[0x95, 0x97, 0x00, 0x00]);
        decoded.extend_from_slice(&body);
        for byte in &mut decoded[8..] {
            *byte = byte.rotate_left(2);
        }
        decoded
    }

    #[test]
    fn name_markers_are_not_part_of_name() {
        assert_eq!(standalone_name("【晶】"), Some("晶"));
        assert_eq!(standalone_name("【】"), None);
        assert_eq!(standalone_name("【晶】本文"), None);
    }

    #[test]
    fn manifest_paths_must_be_single_relative_files() {
        assert!(validate_manifest_relative_file("001_A.ISF.json").is_ok());
        assert!(validate_manifest_relative_file("../A.json").is_err());
        assert!(validate_manifest_relative_file("folder/A.json").is_err());
    }

    #[test]
    fn gbk_reencodes_unchanged_names_and_punctuation() {
        let source = encrypted_script_with_name_and_punctuation();
        let (document, _) = extract_script("TEST.isf", &source).unwrap();

        let cp932 =
            apply_document("TEST.isf", &source, &document, EncodingRoute::Cp932Native).unwrap();
        assert_eq!(cp932.data, source);
        assert_eq!(cp932.patched_messages, 0);
        assert_eq!(cp932.patched_names, 0);
        assert_eq!(cp932.unchanged, 1);

        let gbk = apply_document("TEST.isf", &source, &document, EncodingRoute::Gbk).unwrap();
        assert_ne!(gbk.data, source);
        assert_eq!(gbk.patched_messages, 1);
        assert_eq!(gbk.patched_names, 1);
        assert_eq!(gbk.unchanged, 0);

        let rebuilt = IsfScript::parse(&gbk.data).unwrap();
        let expected_name = encode_lien_text("【玄照】", EncodingRoute::Gbk).unwrap();
        let expected_message = encode_lien_text("。", EncodingRoute::Gbk).unwrap();
        assert_eq!(
            &rebuilt.opcodes[0].content[2..2 + expected_name.len()],
            expected_name.as_slice()
        );
        assert_eq!(
            &rebuilt.opcodes[1].content[2..2 + expected_message.len()],
            expected_message.as_slice()
        );
    }
}
