//! Text extraction and variable-length rebuilding for the game's `*_MSG.CHN`
//! archives. The `.CRS` code layout is intentionally not handled here until
//! its variable-length references have been confirmed.

use crate::{sha256, Result};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use vn_font::font_98::EncodingPlan;

const SCHEMA: &str = "galaxy-railway-pc98-msg-v1";
const ENCODING: &str = "CP932";
const CHN_HEADER_SIZE: usize = 2;
const CHN_ROW_SIZE: usize = 22;
const CHN_NAME_SIZE: usize = 14;
const MSG_SLOT_COUNT: usize = 400;
const MSG_SLOT_SIZE: usize = 8;
const MSG_TABLE_SIZE: usize = MSG_SLOT_COUNT * MSG_SLOT_SIZE;
// The first data pointer is 0x0c80, while its block begins at ptr - 2.
const MSG_DATA_START: usize = MSG_TABLE_SIZE - 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TextDocument {
    pub schema: String,
    pub source_file: String,
    pub source_sha256: String,
    pub encoding: String,
    pub entries: Vec<TextEntry>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TextEntry {
    #[serde(rename = "_index")]
    pub index: usize,
    /// Offset of the text block within the embedded MSG member.
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_byte_length")]
    pub byte_length: usize,
    #[serde(rename = "_kind")]
    pub kind: u16,
    #[serde(rename = "_key")]
    pub key: u16,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Clone)]
struct ChnMember {
    name: String,
    row_offset: usize,
    flags: u16,
    terminal_size_exception: bool,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
struct ChnArchive {
    table_end: usize,
    members: Vec<ChnMember>,
    diagnostics: Vec<String>,
}

#[derive(Debug, Clone)]
struct MsgPart {
    relative_offset: usize,
    bytes: Vec<u8>,
    text: String,
}

#[derive(Debug, Clone)]
struct MsgRecord {
    record_index: usize,
    ptr: u16,
    span_len: u16,
    kind: u16,
    key: u16,
    block_start: usize,
    block_len: usize,
    parts: Vec<MsgPart>,
}

#[derive(Debug, Clone)]
struct MsgMember {
    data_start: usize,
    records: Vec<MsgRecord>,
    opaque_tail: Vec<u8>,
    diagnostics: Vec<String>,
}

fn read_u16(bytes: &[u8], offset: usize, what: &str) -> Result<u16> {
    let end = offset
        .checked_add(2)
        .ok_or_else(|| format!("{what} offset overflow"))?;
    let pair = bytes
        .get(offset..end)
        .ok_or_else(|| format!("{what} is truncated at 0x{offset:X}"))?;
    Ok(u16::from_le_bytes([pair[0], pair[1]]))
}

fn read_u32(bytes: &[u8], offset: usize, what: &str) -> Result<u32> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| format!("{what} offset overflow"))?;
    let word = bytes
        .get(offset..end)
        .ok_or_else(|| format!("{what} is truncated at 0x{offset:X}"))?;
    Ok(u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16, what: &str) -> Result<()> {
    let end = offset
        .checked_add(2)
        .ok_or_else(|| format!("{what} offset overflow"))?;
    let pair = bytes
        .get_mut(offset..end)
        .ok_or_else(|| format!("{what} is outside the rebuilt table"))?;
    pair.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32, what: &str) -> Result<()> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| format!("{what} offset overflow"))?;
    let word = bytes
        .get_mut(offset..end)
        .ok_or_else(|| format!("{what} is outside the rebuilt table"))?;
    word.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn strict_cp932_decode(bytes: &[u8]) -> Option<String> {
    let decoded = SHIFT_JIS.decode_without_bom_handling_and_without_replacement(bytes)?;
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&decoded);
    if had_errors || encoded.as_ref() != bytes {
        return None;
    }
    Some(decoded.into_owned())
}

fn parse_member_name(bytes: &[u8], row_index: usize) -> Result<String> {
    if bytes.len() != CHN_NAME_SIZE {
        return Err(format!("CHN row {row_index} has an invalid name field"));
    }
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    if bytes[end..].iter().any(|byte| *byte != 0) {
        return Err(format!(
            "CHN row {row_index} has nonzero bytes after its filename terminator"
        ));
    }
    if end == 0 {
        return Err(format!("CHN row {row_index} has an empty filename"));
    }
    let name = std::str::from_utf8(&bytes[..end])
        .map_err(|_| format!("CHN row {row_index} filename is not ASCII"))?;
    if !name.is_ascii() {
        return Err(format!("CHN row {row_index} filename is not ASCII"));
    }
    Ok(name.to_owned())
}

fn parse_chn(raw: &[u8]) -> Result<ChnArchive> {
    if raw.len() < CHN_HEADER_SIZE {
        return Err("CHN is shorter than its 2-byte member count".into());
    }
    let count = usize::from(read_u16(raw, 0, "CHN member count")?);
    let table_end = CHN_HEADER_SIZE
        .checked_add(
            count
                .checked_mul(CHN_ROW_SIZE)
                .ok_or("CHN row count overflows table size")?,
        )
        .ok_or("CHN table size overflow")?;
    if table_end > raw.len() {
        return Err(format!(
            "CHN table needs {table_end} bytes but source has {}",
            raw.len()
        ));
    }

    let mut names = HashSet::new();
    let mut rows = Vec::with_capacity(count);
    for row_index in 0..count {
        let row_offset = CHN_HEADER_SIZE + row_index * CHN_ROW_SIZE;
        let name = parse_member_name(&raw[row_offset..row_offset + CHN_NAME_SIZE], row_index)?;
        if !names.insert(name.to_ascii_uppercase()) {
            return Err(format!("CHN contains duplicate member name {name}"));
        }
        let fields_offset = row_offset + CHN_NAME_SIZE;
        let relative_offset = read_u32(raw, fields_offset, "CHN member offset")?;
        let declared_size = read_u16(raw, fields_offset + 4, "CHN member size")?;
        let flags = read_u16(raw, fields_offset + 6, "CHN member flags")?;
        rows.push((name, row_offset, relative_offset, declared_size, flags));
    }

    if count == 0 {
        if raw.len() != table_end {
            return Err("empty CHN has an unparsed trailing region".into());
        }
        return Ok(ChnArchive {
            table_end,
            members: Vec::new(),
            diagnostics: Vec::new(),
        });
    }

    let first_expected = table_end
        .checked_sub(CHN_HEADER_SIZE)
        .ok_or("invalid CHN table base")?;
    if rows[0].2 as usize != first_expected {
        return Err(format!(
            "first CHN member begins at relative offset 0x{:X}, expected 0x{first_expected:X}",
            rows[0].2
        ));
    }

    let mut members = Vec::with_capacity(count);
    let mut diagnostics = Vec::new();
    for (row_index, (name, row_offset, relative_offset, declared_size, flags)) in
        rows.into_iter().enumerate()
    {
        let absolute_offset = CHN_HEADER_SIZE
            .checked_add(relative_offset as usize)
            .ok_or_else(|| format!("{name}: CHN member offset overflow"))?;
        if absolute_offset > raw.len() {
            return Err(format!("{name}: CHN member offset exceeds archive EOF"));
        }
        let declared_end = absolute_offset
            .checked_add(usize::from(declared_size))
            .ok_or_else(|| format!("{name}: CHN member end overflow"))?;
        let is_last = row_index + 1 == count;
        let terminal_size_exception =
            is_last && flags == 1 && declared_end == raw.len().saturating_add(2);
        let actual_end = if declared_end <= raw.len() {
            declared_end
        } else if terminal_size_exception {
            diagnostics.push(format!(
                "member {name}: final flags=1 row declares two bytes past EOF; only the {declared_size_minus_two} bytes present in source are retained",
                declared_size_minus_two = declared_size.saturating_sub(2)
            ));
            raw.len()
        } else {
            return Err(format!(
                "{name}: declared member end 0x{declared_end:X} exceeds CHN EOF 0x{:X}",
                raw.len()
            ));
        };
        if actual_end < absolute_offset {
            return Err(format!("{name}: member has an inverted range"));
        }
        if row_index + 1 < count {
            let next_offset = rows_offset(raw, row_index + 1)?;
            if declared_end != next_offset {
                return Err(format!(
                    "{name}: member end 0x{declared_end:X} does not meet next member at 0x{next_offset:X}"
                ));
            }
        } else if !terminal_size_exception && actual_end != raw.len() {
            return Err(format!(
                "{name}: unparsed bytes remain after final CHN member ({} bytes)",
                raw.len() - actual_end
            ));
        }
        let bytes = raw[absolute_offset..actual_end].to_vec();
        if !name.to_ascii_uppercase().ends_with(".MSG") {
            diagnostics.push(format!(
                "member {name}: non-MSG member is preserved opaquely"
            ));
        }
        members.push(ChnMember {
            name,
            row_offset,
            flags,
            terminal_size_exception,
            bytes,
        });
    }

    Ok(ChnArchive {
        table_end,
        members,
        diagnostics,
    })
}

fn rows_offset(raw: &[u8], row_index: usize) -> Result<usize> {
    let row_offset = CHN_HEADER_SIZE + row_index * CHN_ROW_SIZE;
    let fields_offset = row_offset + CHN_NAME_SIZE;
    let relative = read_u32(raw, fields_offset, "CHN member offset")?;
    CHN_HEADER_SIZE
        .checked_add(relative as usize)
        .ok_or_else(|| "CHN member offset overflow".into())
}

fn parse_msg_member(bytes: &[u8], member_name: &str) -> Result<MsgMember> {
    if bytes.len() < MSG_TABLE_SIZE {
        return Err(format!(
            "{member_name}: MSG table is truncated ({} < {MSG_TABLE_SIZE} bytes)",
            bytes.len()
        ));
    }

    let mut records = Vec::new();
    for record_index in 0..MSG_SLOT_COUNT {
        let slot = record_index * MSG_SLOT_SIZE;
        let ptr = read_u16(bytes, slot, "MSG text pointer")?;
        let span_len = read_u16(bytes, slot + 2, "MSG span length")?;
        let kind = read_u16(bytes, slot + 4, "MSG kind")?;
        let key = read_u16(bytes, slot + 6, "MSG key")?;
        if ptr == 0 && span_len == 0 && kind == 0 {
            // The final nominal slot overlaps the first two text bytes. Its
            // first three fields remain zero and its key field is not a key.
            continue;
        }
        if ptr < 2 || span_len == 0 || kind == 0 {
            return Err(format!(
                "{member_name}: slot {record_index} has a partial zero pointer/length/kind tuple"
            ));
        }
        if record_index * MSG_SLOT_SIZE + MSG_SLOT_SIZE > MSG_DATA_START {
            return Err(format!(
                "{member_name}: active slot {record_index} overlaps the MSG text pool"
            ));
        }
        let block_start = usize::from(ptr) - 2;
        let block_len = usize::from(span_len);
        let block_end = block_start
            .checked_add(block_len)
            .ok_or_else(|| format!("{member_name}: slot {record_index} span overflow"))?;
        if block_start < MSG_DATA_START || block_end > bytes.len() {
            return Err(format!(
                "{member_name}: slot {record_index} span 0x{block_start:X}..0x{block_end:X} is outside the MSG data region"
            ));
        }
        let block = &bytes[block_start..block_end];
        let mut parts = Vec::new();
        let mut cursor = 0;
        while cursor < block.len() {
            if block[cursor] == 0 {
                cursor += 1;
                continue;
            }
            let start = cursor;
            while cursor < block.len() && block[cursor] != 0 {
                cursor += 1;
            }
            if cursor == block.len() {
                return Err(format!(
                    "{member_name}: slot {record_index} has a non-NUL-terminated text part"
                ));
            }
            let raw_part = &block[start..cursor];
            let text = strict_cp932_decode(raw_part).ok_or_else(|| {
                format!(
                    "{member_name}: slot {record_index} part at +0x{start:X} is not reversible CP932"
                )
            })?;
            if text.chars().any(|ch| ch.is_control()) {
                return Err(format!(
                    "{member_name}: slot {record_index} part at +0x{start:X} contains a control character"
                ));
            }
            parts.push(MsgPart {
                relative_offset: start,
                bytes: raw_part.to_vec(),
                text,
            });
        }
        records.push(MsgRecord {
            record_index,
            ptr,
            span_len,
            kind,
            key,
            block_start,
            block_len,
            parts,
        });
    }

    if records.is_empty() {
        return Ok(MsgMember {
            data_start: MSG_DATA_START,
            records,
            opaque_tail: bytes[MSG_DATA_START..].to_vec(),
            diagnostics: vec![format!(
                "member {member_name}: no active MSG records; payload preserved opaquely"
            )],
        });
    }
    if records[0].block_start != MSG_DATA_START {
        return Err(format!(
            "{member_name}: first text block starts at 0x{:X}, expected 0x{MSG_DATA_START:X}",
            records[0].block_start
        ));
    }
    for pair in records.windows(2) {
        if pair[0].block_start + pair[0].block_len != pair[1].block_start {
            return Err(format!(
                "{member_name}: MSG spans at slots {} and {} are not contiguous in table order",
                pair[0].record_index, pair[1].record_index
            ));
        }
        if usize::from(pair[0].ptr) + usize::from(pair[0].span_len) != usize::from(pair[1].ptr) {
            return Err(format!(
                "{member_name}: slot {} span does not lead to slot {} pointer",
                pair[0].record_index, pair[1].record_index
            ));
        }
    }
    let last = records.last().ok_or("MSG record list unexpectedly empty")?;
    let text_end = last
        .block_start
        .checked_add(last.block_len)
        .ok_or_else(|| format!("{member_name}: final text span overflow"))?;
    let opaque_tail = bytes[text_end..].to_vec();
    let mut diagnostics = Vec::new();
    if !opaque_tail.is_empty() {
        diagnostics.push(format!(
            "member {member_name}: opaque {len}-byte MSG suffix retained exactly after final span",
            len = opaque_tail.len()
        ));
    }
    Ok(MsgMember {
        data_start: MSG_DATA_START,
        records,
        opaque_tail,
        diagnostics,
    })
}

fn make_documents(archive: &ChnArchive) -> Result<Vec<TextDocument>> {
    let mut documents = Vec::new();
    for member in &archive.members {
        if !member.name.to_ascii_uppercase().ends_with(".MSG") {
            continue;
        }
        let parsed = parse_msg_member(&member.bytes, &member.name)?;
        let mut entries = Vec::new();
        let mut diagnostics = archive.diagnostics.clone();
        diagnostics.extend(parsed.diagnostics.iter().cloned());
        for record in &parsed.records {
            let message = record
                .parts
                .iter()
                .map(|part| part.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            entries.push(TextEntry {
                index: record.record_index,
                offset: record.block_start,
                byte_length: record.block_len,
                kind: record.kind,
                key: record.key,
                scr_msg: message.clone(),
                message,
            });
        }
        documents.push(TextDocument {
            schema: SCHEMA.into(),
            source_file: member.name.clone(),
            source_sha256: sha256(&member.bytes),
            encoding: ENCODING.into(),
            entries,
            diagnostics,
        });
    }
    Ok(documents)
}

/// Extract one translation document per embedded MSG member. The caller's
/// archive path is accepted for a consistent extractor interface; each JSON
/// file instead binds directly to its uniquely named embedded member.
pub fn extract_chn(raw: &[u8], _archive_source: &str) -> Result<Vec<TextDocument>> {
    let archive = parse_chn(raw)?;
    make_documents(&archive)
}

fn validate_document(
    supplied: &TextDocument,
    expected: &TextDocument,
) -> Result<HashMap<(String, usize), String>> {
    if supplied.schema != expected.schema
        || supplied.source_file != expected.source_file
        || supplied.source_sha256 != expected.source_sha256
        || supplied.encoding != ENCODING
        || supplied.diagnostics != expected.diagnostics
        || supplied.entries.len() != expected.entries.len()
    {
        return Err(format!(
            "translation JSON metadata, diagnostics, or entry count does not match {}",
            expected.source_file
        ));
    }

    let mut translations = HashMap::new();
    for (actual, baseline) in supplied.entries.iter().zip(&expected.entries) {
        if actual.index != baseline.index
            || actual.offset != baseline.offset
            || actual.byte_length != baseline.byte_length
            || actual.kind != baseline.kind
            || actual.key != baseline.key
            || actual.scr_msg != baseline.scr_msg
        {
            return Err(format!(
                "entry {} original text or location metadata was modified",
                baseline.index
            ));
        }
        if actual.message.contains('\r') {
            return Err(format!(
                "{} entry {} message contains CR/CRLF",
                expected.source_file, baseline.index
            ));
        }
        if !actual.message.is_empty() && actual.message.split('\n').any(str::is_empty) {
            return Err(format!(
                "{} entry {} has an empty line that cannot be distinguished from NUL padding",
                expected.source_file, baseline.index
            ));
        }
        translations.insert(
            (expected.source_file.to_ascii_uppercase(), baseline.index),
            actual.message.clone(),
        );
    }
    Ok(translations)
}

fn build_record_block(
    original: &[u8],
    record: &MsgRecord,
    translated: Option<&str>,
    encoding: &EncodingPlan,
    member_name: &str,
) -> Result<Vec<u8>> {
    let Some(translated) = translated else {
        return Ok(original.to_vec());
    };
    let original_text = record
        .parts
        .iter()
        .map(|part| part.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    if translated == original_text {
        return Ok(original.to_vec());
    }

    let lines: Vec<&str> = if translated.is_empty() {
        Vec::new()
    } else {
        translated.split('\n').collect()
    };
    if lines.iter().any(|line| line.is_empty()) {
        return Err(format!(
            "{member_name} slot {} contains an empty translated line, which this MSG layout cannot distinguish from NUL padding",
            record.record_index
        ));
    }

    let Some(first) = record.parts.first() else {
        if original.iter().any(|byte| *byte != 0) {
            return Err(format!(
                "{member_name}: empty slot {} has nonzero opaque bytes",
                record.record_index
            ));
        }
        let mut rebuilt = Vec::new();
        for (line_index, line) in lines.iter().enumerate() {
            rebuilt.extend_from_slice(&crate::font_plan::encode_display(line, encoding).map_err(
                |error| {
                    format!(
                        "{member_name} slot {} line {line_index}: {error}",
                        record.record_index
                    )
                },
            )?);
            if line_index + 1 < lines.len() {
                rebuilt.push(0);
            }
        }
        rebuilt.extend_from_slice(original);
        return Ok(rebuilt);
    };
    let last = record.parts.last().expect("nonempty parts checked above");
    let last_end = last
        .relative_offset
        .checked_add(last.bytes.len())
        .ok_or_else(|| {
            format!(
                "{member_name}: slot {} part end overflow",
                record.record_index
            )
        })?;
    if last_end > original.len() || first.relative_offset > original.len() {
        return Err(format!(
            "{member_name}: slot {} text extends beyond its source span",
            record.record_index
        ));
    }
    let prefix = &original[..first.relative_offset];
    let text_bytes = record
        .parts
        .iter()
        .map(|part| part.bytes.len())
        .sum::<usize>();
    let native_separators = record.parts.len().saturating_sub(1);
    let padding_len = original
        .len()
        .checked_sub(prefix.len() + text_bytes + native_separators)
        .ok_or_else(|| {
            format!(
                "{member_name}: slot {} has invalid NUL padding",
                record.record_index
            )
        })?;
    if original.iter().enumerate().any(|(position, byte)| {
        *byte != 0
            && !record.parts.iter().any(|part| {
                position >= part.relative_offset
                    && position < part.relative_offset + part.bytes.len()
            })
    }) {
        return Err(format!(
            "{member_name}: slot {} contains non-text bytes outside its CP932 segments",
            record.record_index
        ));
    }
    let mut rebuilt = Vec::new();
    rebuilt.extend_from_slice(prefix);
    for (line_index, line) in lines.iter().enumerate() {
        rebuilt.extend_from_slice(&crate::font_plan::encode_display(line, encoding).map_err(
            |error| {
                format!(
                    "{member_name} slot {} line {line_index}: {error}",
                    record.record_index
                )
            },
        )?);
        if line_index + 1 < lines.len() {
            // One JSON newline is exactly one native NUL. Any extra NULs in
            // the source gaps are included in padding below.
            rebuilt.push(0);
        }
    }
    // Keep all bytes that were not text or one source line delimiter. This
    // preserves leading zeros and total trailing/inter-line padding while the
    // new message's newline count controls the actual separators.
    rebuilt.resize(rebuilt.len() + padding_len, 0);
    Ok(rebuilt)
}

fn rebuild_msg_member(
    original: &[u8],
    parsed: &MsgMember,
    translations: &HashMap<(String, usize), String>,
    encoding: &EncodingPlan,
    member_name: &str,
) -> Result<Vec<u8>> {
    if parsed.records.is_empty() {
        return Ok(original.to_vec());
    }
    let mut table_prefix = original
        .get(..parsed.data_start)
        .ok_or_else(|| format!("{member_name}: MSG table prefix is truncated"))?
        .to_vec();
    let mut body = Vec::new();
    let mut cursor = parsed.data_start;
    for record in &parsed.records {
        let original_end = record
            .block_start
            .checked_add(record.block_len)
            .ok_or_else(|| format!("{member_name}: slot {} span overflow", record.record_index))?;
        let block = original
            .get(record.block_start..original_end)
            .ok_or_else(|| {
                format!(
                    "{member_name}: slot {} span is outside source",
                    record.record_index
                )
            })?;
        let translated = translations.get(&(member_name.to_ascii_uppercase(), record.record_index));
        let new_block = build_record_block(
            block,
            record,
            translated.map(String::as_str),
            encoding,
            member_name,
        )?;
        let new_ptr = cursor
            .checked_add(2)
            .ok_or_else(|| format!("{member_name}: rebuilt pointer overflow"))?;
        let new_end = cursor
            .checked_add(new_block.len())
            .ok_or_else(|| format!("{member_name}: rebuilt span overflow"))?;
        if new_ptr > usize::from(u16::MAX)
            || new_block.is_empty()
            || new_block.len() > usize::from(u16::MAX)
            || new_end > usize::from(u16::MAX) + 1
        {
            return Err(format!(
                "{member_name}: reflowed slot {} exceeds 16-bit MSG pointer/span range",
                record.record_index
            ));
        }
        let slot = record.record_index * MSG_SLOT_SIZE;
        write_u16(
            &mut table_prefix,
            slot,
            new_ptr as u16,
            "rebuilt MSG text pointer",
        )?;
        write_u16(
            &mut table_prefix,
            slot + 2,
            new_block.len() as u16,
            "rebuilt MSG span length",
        )?;
        body.extend_from_slice(&new_block);
        cursor = new_end;
    }
    table_prefix.extend_from_slice(&body);
    table_prefix.extend_from_slice(&parsed.opaque_tail);
    let rebuilt = table_prefix;
    if rebuilt.len() > usize::from(u16::MAX) {
        return Err(format!(
            "{member_name}: rebuilt MSG member exceeds CHN's 16-bit size field"
        ));
    }
    Ok(rebuilt)
}

/// Apply a collection of per-member translation documents to one CHN archive.
/// Documents for other CHN archives may be included and are ignored here.
pub fn apply_chn(
    raw: &[u8],
    supplied: &[TextDocument],
    encoding: &EncodingPlan,
) -> Result<Vec<u8>> {
    let archive = parse_chn(raw)?;
    let expected = make_documents(&archive)?;
    let mut supplied_by_name: HashMap<String, &TextDocument> = HashMap::new();
    for document in supplied {
        let key = document.source_file.to_ascii_uppercase();
        if key.is_empty() || supplied_by_name.insert(key.clone(), document).is_some() {
            return Err(format!(
                "translation collection has a duplicate or empty source_file {key:?}"
            ));
        }
    }

    let mut translations = HashMap::new();
    for expected_document in &expected {
        let key = expected_document.source_file.to_ascii_uppercase();
        let actual = supplied_by_name.get(&key).ok_or_else(|| {
            format!(
                "translation collection is missing {}",
                expected_document.source_file
            )
        })?;
        if actual.source_file != expected_document.source_file {
            return Err(format!(
                "translation source_file must exactly match {}",
                expected_document.source_file
            ));
        }
        translations.extend(validate_document(actual, expected_document)?);
    }

    let any_change = expected.iter().any(|document| {
        document.entries.iter().any(|baseline| {
            translations
                .get(&(document.source_file.to_ascii_uppercase(), baseline.index))
                .is_some_and(|message| message != &baseline.scr_msg)
        })
    });
    if !any_change {
        return Ok(raw.to_vec());
    }

    let table_end = archive.table_end;
    let mut rebuilt = raw
        .get(..table_end)
        .ok_or("CHN table is truncated during rebuild")?
        .to_vec();
    let mut next_relative_offset = table_end
        .checked_sub(CHN_HEADER_SIZE)
        .ok_or("CHN member base underflow")?;
    for (member_index, member) in archive.members.iter().enumerate() {
        if rebuilt.len().checked_sub(CHN_HEADER_SIZE) != Some(next_relative_offset) {
            return Err(format!(
                "{}: output member cursor is inconsistent",
                member.name
            ));
        }
        let new_member = if member.name.to_ascii_uppercase().ends_with(".MSG") {
            let parsed = parse_msg_member(&member.bytes, &member.name)?;
            rebuild_msg_member(
                &member.bytes,
                &parsed,
                &translations,
                encoding,
                &member.name,
            )?
        } else {
            member.bytes.clone()
        };
        let new_relative = u32::try_from(next_relative_offset)
            .map_err(|_| format!("{}: rebuilt CHN offset exceeds u32", member.name))?;
        let new_size_usize = new_member.len();
        let declared_size_usize = if member.terminal_size_exception {
            if member_index + 1 != archive.members.len() || member.flags != 1 {
                return Err(format!(
                    "{}: terminal +2 size convention moved away from the final flags=1 row",
                    member.name
                ));
            }
            new_size_usize
                .checked_add(2)
                .ok_or_else(|| format!("{}: terminal declared size overflow", member.name))?
        } else {
            new_size_usize
        };
        let new_size = u16::try_from(declared_size_usize)
            .map_err(|_| format!("{}: rebuilt declared member size exceeds u16", member.name))?;
        let row_fields = member.row_offset + CHN_NAME_SIZE;
        write_u32(
            &mut rebuilt,
            row_fields,
            new_relative,
            "rebuilt CHN member offset",
        )?;
        write_u16(
            &mut rebuilt,
            row_fields + 4,
            new_size,
            "rebuilt CHN member size",
        )?;
        // Preserve the flag word exactly, including the confirmed terminal 1.
        write_u16(
            &mut rebuilt,
            row_fields + 6,
            member.flags,
            "rebuilt CHN member flags",
        )?;
        rebuilt.extend_from_slice(&new_member);
        next_relative_offset = next_relative_offset
            .checked_add(new_size_usize)
            .ok_or_else(|| format!("{}: rebuilt CHN cursor overflow", member.name))?;
    }
    Ok(rebuilt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vn_font::font_98::SubstitutionMap;

    fn cp932(text: &str) -> Vec<u8> {
        let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
        assert!(!had_errors);
        encoded.into_owned()
    }

    fn build_msg(blocks: &[(u16, u16, Vec<u8>)], suffix: &[u8]) -> Vec<u8> {
        let mut table = vec![0u8; MSG_TABLE_SIZE];
        let mut body = Vec::new();
        let mut cursor = MSG_DATA_START;
        for (record_index, (kind, key, block)) in blocks.iter().enumerate() {
            let slot = record_index * MSG_SLOT_SIZE;
            table[slot..slot + 2].copy_from_slice(&((cursor + 2) as u16).to_le_bytes());
            table[slot + 2..slot + 4].copy_from_slice(&(block.len() as u16).to_le_bytes());
            table[slot + 4..slot + 6].copy_from_slice(&kind.to_le_bytes());
            table[slot + 6..slot + 8].copy_from_slice(&key.to_le_bytes());
            body.extend_from_slice(block);
            cursor += block.len();
        }
        table.truncate(MSG_DATA_START);
        table.extend_from_slice(&body);
        table.extend_from_slice(suffix);
        table
    }

    fn build_single_member_chn(name: &str, member: &[u8], terminal_exception: bool) -> Vec<u8> {
        build_chn(&[(name, member.to_vec(), u16::from(terminal_exception))])
    }

    fn build_chn(members: &[(&str, Vec<u8>, u16)]) -> Vec<u8> {
        let table_end = CHN_HEADER_SIZE + members.len() * CHN_ROW_SIZE;
        let mut raw = vec![0u8; table_end];
        raw[..2].copy_from_slice(&(members.len() as u16).to_le_bytes());
        let mut absolute = table_end;
        for (index, (name, bytes, flags)) in members.iter().enumerate() {
            let row = CHN_HEADER_SIZE + index * CHN_ROW_SIZE;
            let name_bytes = name.as_bytes();
            assert!(name_bytes.len() <= CHN_NAME_SIZE);
            raw[row..row + name_bytes.len()].copy_from_slice(name_bytes);
            let relative = (absolute - CHN_HEADER_SIZE) as u32;
            raw[row + CHN_NAME_SIZE..row + CHN_NAME_SIZE + 4]
                .copy_from_slice(&relative.to_le_bytes());
            let declared_size =
                bytes.len() + usize::from(index + 1 == members.len() && *flags == 1) * 2;
            raw[row + CHN_NAME_SIZE + 4..row + CHN_NAME_SIZE + 6]
                .copy_from_slice(&(declared_size as u16).to_le_bytes());
            raw[row + CHN_NAME_SIZE + 6..row + CHN_NAME_SIZE + 8]
                .copy_from_slice(&flags.to_le_bytes());
            raw.extend_from_slice(bytes);
            absolute += bytes.len();
        }
        raw
    }

    fn plan(texts: &[&str]) -> EncodingPlan {
        let substitutions = SubstitutionMap::embedded().expect("embedded substitutions");
        EncodingPlan::build(
            &substitutions,
            std::iter::empty::<u16>(),
            texts.iter().copied(),
        )
        .expect("synthetic encoding plan")
    }

    fn only_doc(source: &[u8]) -> TextDocument {
        let mut documents = extract_chn(source, "disk-01/TEST_MSG.CHN").unwrap();
        assert_eq!(documents.len(), 1);
        documents.remove(0)
    }

    fn verify_terminal_size_and_tail(rebuilt: &[u8]) {
        let row = CHN_HEADER_SIZE;
        let fields = row + CHN_NAME_SIZE;
        let offset = CHN_HEADER_SIZE + read_u32(rebuilt, fields, "member offset").unwrap() as usize;
        let physical_size = rebuilt.len() - offset;
        assert_eq!(
            read_u16(rebuilt, fields + 4, "member size").unwrap() as usize,
            physical_size + 2
        );
        assert_eq!(read_u16(rebuilt, fields + 6, "member flags").unwrap(), 1);
        assert!(rebuilt.ends_with(&[0x1a]));
        assert!(!rebuilt.ends_with(&[0x1a, 0x01, 0x00]));
    }

    #[test]
    fn json_shape_is_per_embedded_member_and_identity_is_byte_exact() {
        let first = [cp932("あ"), vec![0, 0], cp932("い"), vec![0, 0, 0]].concat();
        let second = [cp932("う"), vec![0, 0, 0]].concat();
        let msg = build_msg(&[(3, 2, first), (6, 3, second)], &[0x1a]);
        let source = build_single_member_chn("TEST_MSG.MSG", &msg, true);

        let documents =
            extract_chn(&source, "disk-01/TEST_MSG.CHN").expect("extract synthetic CHN");
        assert_eq!(documents.len(), 1);
        let document = &documents[0];
        assert_eq!(document.source_file, "TEST_MSG.MSG");
        assert_eq!(document.encoding, "CP932");
        assert_eq!(document.entries.len(), 2);
        assert_eq!(document.entries[0].index, 0);
        assert_eq!(document.entries[0].offset, MSG_DATA_START);
        assert_eq!(document.entries[0].byte_length, 9);
        assert_eq!(document.entries[0].scr_msg, "あ\nい");
        let json = serde_json::to_value(document).unwrap();
        assert!(json["entries"][0].get("_index").is_some());
        assert!(json["entries"][0].get("_offset").is_some());
        assert!(json["entries"][0].get("_byte_length").is_some());
        assert!(json["entries"][0].get("_kind").is_some());
        assert!(json["entries"][0].get("_key").is_some());
        assert!(json["entries"][0].get("parts").is_none());
        assert_eq!(
            apply_chn(&source, &documents, &plan(&["あ"])).unwrap(),
            source
        );
    }

    #[test]
    fn terminal_plus_two_convention_is_preserved_for_opaque_resource_member() {
        let msg = build_msg(&[(3, 2, [cp932("客室"), vec![0, 0, 0]].concat())], &[]);
        let source = build_chn(&[("TEST_MSG.MSG", msg, 0), ("STAR1.PRS", vec![0x1a], 1)]);
        let mut documents = extract_chn(&source, "TEST_MSG.CHN").unwrap();
        assert_eq!(documents.len(), 1);
        assert!(documents[0]
            .diagnostics
            .iter()
            .any(|line| line.contains("STAR1.PRS") && line.contains("two bytes past EOF")));
        assert_eq!(
            apply_chn(&source, &documents, &plan(&["客室"])).unwrap(),
            source
        );

        documents[0].entries[0].message = "车厢".into();
        let rebuilt = apply_chn(&source, &documents, &plan(&["车厢"])).unwrap();
        let reparsed = parse_chn(&rebuilt).unwrap();
        assert_eq!(reparsed.members[1].name, "STAR1.PRS");
        assert_eq!(reparsed.members[1].bytes, [0x1a]);
        assert!(reparsed.members[1].terminal_size_exception);
    }

    #[test]
    fn reflow_handles_short_long_added_and_removed_lines_and_rereads_terminal_member() {
        let first = [cp932("あ"), vec![0, 0], cp932("い"), vec![0, 0, 0]].concat();
        let second = [cp932("う"), vec![0, 0, 0]].concat();
        let msg = build_msg(&[(3, 2, first), (6, 3, second)], &[0x1a]);
        let source = build_single_member_chn("TEST_MSG.MSG", &msg, true);

        // Shorter text and fewer source lines.
        let mut short_docs = extract_chn(&source, "disk-01/TEST_MSG.CHN").unwrap();
        short_docs[0].entries[0].message = "短".into();
        let short = apply_chn(&source, &short_docs, &plan(&["短"])).unwrap();
        assert_eq!(short.len(), source.len() - 3);
        verify_terminal_size_and_tail(&short);
        let short_read = only_doc(&short);
        assert_eq!(short_read.entries[0].message, "短");
        assert_eq!(short_read.entries[1].message, "う");

        // Longer lines retain the same line count and use encoding-plan bytes.
        let mut long_docs = extract_chn(&source, "disk-01/TEST_MSG.CHN").unwrap();
        long_docs[0].entries[0].message = "長い文章\nさらに長い文".into();
        let long = apply_chn(&source, &long_docs, &plan(&["長い文章", "さらに長い文"])).unwrap();
        verify_terminal_size_and_tail(&long);
        let long_read = only_doc(&long);
        assert_eq!(long_read.entries[0].message, "長い文章\nさらに長い文");

        // Adding a line writes one native NUL per JSON newline. The source's
        // extra NUL bytes remain as trailing record padding.
        let mut added_docs = extract_chn(&source, "disk-01/TEST_MSG.CHN").unwrap();
        added_docs[0].entries[0].message = "あ\nい\nう".into();
        let added = apply_chn(&source, &added_docs, &plan(&["あ", "い", "う"])).unwrap();
        verify_terminal_size_and_tail(&added);
        let added_read = only_doc(&added);
        assert_eq!(added_read.entries[0].message, "あ\nい\nう");
        let member_offset = CHN_HEADER_SIZE
            + read_u32(&added, CHN_HEADER_SIZE + CHN_NAME_SIZE, "member offset").unwrap() as usize;
        let new_block = &added[member_offset + MSG_DATA_START..member_offset + MSG_DATA_START + 12];
        assert_eq!(
            new_block,
            [
                cp932("あ"),
                vec![0],
                cp932("い"),
                vec![0],
                cp932("う"),
                vec![0, 0, 0, 0]
            ]
            .concat()
        );

        // Deleting a line reflows the next 16-bit pointer and rereads cleanly.
        let mut deleted_docs = extract_chn(&source, "disk-01/TEST_MSG.CHN").unwrap();
        deleted_docs[0].entries[0].message = "い".into();
        let deleted = apply_chn(&source, &deleted_docs, &plan(&["い"])).unwrap();
        verify_terminal_size_and_tail(&deleted);
        let deleted_read = only_doc(&deleted);
        assert_eq!(deleted_read.entries[0].message, "い");
        assert_eq!(deleted_read.entries[1].message, "う");
        let member_offset = CHN_HEADER_SIZE
            + read_u32(&deleted, CHN_HEADER_SIZE + CHN_NAME_SIZE, "member offset").unwrap()
                as usize;
        let first_ptr = read_u16(&deleted[member_offset..], 0, "first pointer").unwrap();
        let first_len = read_u16(&deleted[member_offset..], 2, "first span").unwrap();
        let second_ptr =
            read_u16(&deleted[member_offset..], MSG_SLOT_SIZE, "second pointer").unwrap();
        assert_eq!(second_ptr, first_ptr + first_len);
    }

    #[test]
    fn chn_rejects_unverified_terminal_overrun() {
        let msg = build_msg(&[(3, 2, [cp932("あ"), vec![0, 0, 0]].concat())], &[0x1a]);
        let mut source = build_single_member_chn("TEST_MSG.MSG", &msg, true);
        let flags = CHN_HEADER_SIZE + CHN_NAME_SIZE + 4 + 2;
        source[flags..flags + 2].copy_from_slice(&0u16.to_le_bytes());
        assert!(extract_chn(&source, "disk-01/TEST_MSG.CHN").is_err());
    }

    #[test]
    fn chn_exports_all_reversible_active_records() {
        let japanese = [cp932("客室"), vec![0, 0, 0]].concat();
        let ascii = [b"ROOM".to_vec(), vec![0, 0, 0]].concat();
        let msg = build_msg(&[(3, 2, japanese), (3, 3, ascii)], &[]);
        let source = build_single_member_chn("TEST_MSG.MSG", &msg, false);
        let documents = extract_chn(&source, "disk-01/TEST_MSG.CHN").unwrap();
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].entries.len(), 2);
        assert_eq!(documents[0].entries[0].scr_msg, "客室");
        assert_eq!(documents[0].entries[1].scr_msg, "ROOM");
    }

    #[test]
    fn extraction_creates_one_document_per_msg_member() {
        let first = build_msg(&[(3, 2, [cp932("客室"), vec![0, 0, 0]].concat())], &[]);
        let second = build_msg(&[(3, 3, [cp932("寝台"), vec![0, 0, 0]].concat())], &[]);
        let source = build_chn(&[("HAK1_MSG.MSG", first, 0), ("HAK2_MSG.MSG", second, 0)]);
        let docs = extract_chn(&source, "disk-01/TEST_MSG.CHN").unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].source_file, "HAK1_MSG.MSG");
        assert_eq!(docs[1].source_file, "HAK2_MSG.MSG");
        assert_eq!(docs[0].entries[0].message, "客室");
        assert_eq!(docs[1].entries[0].message, "寝台");
    }
}
