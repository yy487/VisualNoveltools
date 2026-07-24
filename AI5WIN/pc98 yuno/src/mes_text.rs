use crate::mes::{compress, decompress, MesError, Result};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::borrow::Cow;

pub const JSON_FORMAT: &str = "yuno-pc98-mes-v1";
const TEXT_ENCODING: &str = "ai5-pc98-cp932-token";
const TEXT_POLICY: &str = "relocate";

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextPart {
    start: usize,
    end: usize,
    text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InlineControl {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScriptRecord {
    inst_offset: usize,
    bracketed: bool,
    static_name: Option<TextPart>,
    dynamic_name: bool,
    name_controls: Vec<InlineControl>,
    message_parts: Vec<TextPart>,
    message_controls: Vec<InlineControl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedName {
    static_name: Option<TextPart>,
    dynamic: bool,
    controls: Vec<InlineControl>,
    after_close: usize,
}

#[derive(Debug, Clone)]
struct ParsedScript {
    dictionary: Vec<String>,
    records: Vec<ScriptRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlMetadata {
    pub after_part: usize,
    pub offset: usize,
    pub hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextEntry {
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_inst_offset")]
    pub inst_offset: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub kind: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_policy")]
    pub policy: String,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    pub scr_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "_name_dynamic", default, skip_serializing_if = "is_false")]
    pub name_dynamic: bool,
    #[serde(
        rename = "_name_controls",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub name_controls: Vec<ControlMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scr_msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scr_msg_parts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_parts: Option<Vec<String>>,
    #[serde(
        rename = "_message_controls",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub message_controls: Vec<ControlMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextDocument {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_source_sha256")]
    pub source_sha256: String,
    pub entries: Vec<TextEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectResult {
    pub stored: Vec<u8>,
    pub json_entries: usize,
    pub patched_fields: usize,
    pub unchanged_fields: usize,
    pub source_bytes: usize,
    pub output_bytes: usize,
    pub decoded_source_bytes: usize,
    pub decoded_output_bytes: usize,
}

#[derive(Debug)]
struct Patch {
    start: usize,
    end: usize,
    replacement: Vec<u8>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn invalid(message: impl Into<String>) -> MesError {
    MesError::Invalid(message.into())
}

fn decode_exact_cp932(raw: &[u8], field: &str) -> Result<String> {
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(raw);
    if had_errors {
        return Err(invalid(format!("{field} is not valid CP932")));
    }
    let (encoded, _, encode_errors) = SHIFT_JIS.encode(&decoded);
    if encode_errors || encoded.as_ref() != raw {
        return Err(invalid(format!(
            "{field} does not round-trip through CP932"
        )));
    }
    Ok(decoded.into_owned())
}

fn parse_dictionary(data: &[u8]) -> Result<(usize, Vec<String>)> {
    if data.len() < 2 {
        return Err(invalid("decoded MES is shorter than its entry offset"));
    }
    let entry_offset = usize::from(u16::from_le_bytes([data[0], data[1]]));
    if entry_offset < 2 || entry_offset >= data.len() || (entry_offset - 2) % 2 != 0 {
        return Err(invalid(format!(
            "invalid MES entry offset 0x{entry_offset:X}"
        )));
    }

    let mut dictionary = Vec::with_capacity((entry_offset - 2) / 2);
    for position in (2..entry_offset).step_by(2) {
        let text = decode_exact_cp932(
            &data[position..position + 2],
            &format!("dictionary entry at 0x{position:X}"),
        )?;
        if text.chars().count() != 1 {
            return Err(invalid(format!(
                "dictionary entry at 0x{position:X} is not one character"
            )));
        }
        dictionary.push(text);
    }
    if dictionary.len() > 48 {
        return Err(invalid(format!(
            "dictionary contains {} entries; runtime supports at most 48",
            dictionary.len()
        )));
    }
    Ok((entry_offset, dictionary))
}

fn decode_text_token(
    data: &[u8],
    position: usize,
    dictionary: &[String],
) -> Option<(String, usize)> {
    let value = *data.get(position)?;
    if value >= 0xD0 {
        let index = usize::from(value - 0xD0);
        return dictionary
            .get(index)
            .cloned()
            .map(|text| (text, position + 1));
    }

    if (0x60..=0x7F).contains(&value) || (0xC0..=0xCF).contains(&value) {
        let trail = *data.get(position + 1)?;
        let raw = [value + 0x20, trail];
        let (text, had_errors) = SHIFT_JIS.decode_without_bom_handling(&raw);
        if had_errors || text.chars().count() != 1 {
            return None;
        }
        let (encoded, _, encode_errors) = SHIFT_JIS.encode(&text);
        if encode_errors || encoded.as_ref() != raw {
            return None;
        }
        return Some((text.into_owned(), position + 2));
    }
    None
}

fn skip_expression(data: &[u8], mut position: usize) -> Result<usize> {
    loop {
        let opcode = *data.get(position).ok_or_else(|| {
            invalid(format!(
                "truncated expression at decoded offset 0x{position:X}"
            ))
        })?;
        position += 1;
        match opcode {
            0x03 => return Ok(position),
            0x07 => position = position.checked_add(1).ok_or_else(|| invalid("overflow"))?,
            0x08 => position = position.checked_add(2).ok_or_else(|| invalid("overflow"))?,
            0x09 => position = position.checked_add(3).ok_or_else(|| invalid("overflow"))?,
            0x20..=0x5A => {}
            _ => return Ok(position),
        }
        if position > data.len() {
            return Err(invalid("truncated immediate inside expression"));
        }
    }
}

fn skip_delimited(data: &[u8], mut position: usize, delimiter: u8) -> Result<usize> {
    position += 1;
    while position < data.len() {
        if data[position] == delimiter {
            return Ok(position + 1);
        }
        position += 1;
    }
    Err(invalid(format!(
        "unterminated 0x{delimiter:02X} block in decoded MES"
    )))
}

fn skip_nested_block(data: &[u8], mut position: usize) -> Result<usize> {
    if data.get(position) != Some(&0x01) {
        return Err(invalid(format!(
            "nested block at 0x{position:X} does not start with 0x01"
        )));
    }
    position += 1;
    let mut depth = 1usize;
    while position < data.len() {
        match data[position] {
            0x01 => depth += 1,
            0x00 => {
                depth -= 1;
                if depth == 0 {
                    return Ok(position + 1);
                }
            }
            _ => {}
        }
        position += 1;
    }
    Err(invalid("unterminated nested 0x01/0x00 block"))
}

fn is_expression_start(value: u8) -> bool {
    matches!(value, 0x07..=0x09 | 0x20..=0x5A)
}

fn skip_argument_list(data: &[u8], mut position: usize) -> Result<usize> {
    loop {
        let value = *data.get(position).ok_or_else(|| {
            invalid(format!(
                "truncated command arguments at decoded offset 0x{position:X}"
            ))
        })?;
        position = match value {
            0x06 => skip_delimited(data, position, 0x06)?,
            0x01 => skip_nested_block(data, position)?,
            _ if is_expression_start(value) => skip_expression(data, position)?,
            _ => position + 1,
        };

        if data.get(position) == Some(&0x02) {
            position += 1;
            continue;
        }
        return Ok(position);
    }
}

fn consume_text_run(data: &[u8], mut position: usize, dictionary: &[String]) -> Option<TextPart> {
    let start = position;
    let mut text = String::new();
    while let Some((character, next)) = decode_text_token(data, position, dictionary) {
        text.push_str(&character);
        position = next;
    }
    if position == start {
        None
    } else {
        Some(TextPart {
            start,
            end: position,
            text,
        })
    }
}

fn consume_inline_controls_to_text(
    data: &[u8],
    position: usize,
    dictionary: &[String],
) -> Result<Option<(Vec<InlineControl>, TextPart)>> {
    let mut cursor = position;
    let mut controls = Vec::new();
    while matches!(data.get(cursor), Some(0x13 | 0x16)) {
        let start = cursor;
        cursor = skip_argument_list(data, cursor + 1)?;
        controls.push(InlineControl { start, end: cursor });
    }
    Ok(consume_text_run(data, cursor, dictionary).map(|part| (controls, part)))
}

fn parse_name(
    data: &[u8],
    mut position: usize,
    dictionary: &[String],
) -> Result<Option<ParsedName>> {
    let name_start = position;
    let mut text = String::new();
    let mut controls = Vec::new();
    loop {
        let Some((character, next)) = decode_text_token(data, position, dictionary) else {
            if data.get(position) != Some(&0x13) {
                return Ok(None);
            }
            let start = position;
            position = skip_argument_list(data, position + 1)?;
            controls.push(InlineControl {
                start,
                end: position,
            });
            continue;
        };
        if character == "】" {
            let static_name = if controls.is_empty() {
                Some(TextPart {
                    start: name_start,
                    end: position,
                    text,
                })
            } else {
                None
            };
            return Ok(Some(ParsedName {
                static_name,
                dynamic: !controls.is_empty(),
                controls,
                after_close: next,
            }));
        }
        text.push_str(&character);
        position = next;
    }
}

fn parse_message_parts(
    data: &[u8],
    position: usize,
    dictionary: &[String],
) -> Result<(Vec<TextPart>, Vec<InlineControl>, usize)> {
    let Some(first_part) = consume_text_run(data, position, dictionary) else {
        return Ok((Vec::new(), Vec::new(), position));
    };
    let mut cursor = first_part.end;
    let mut parts = vec![first_part];
    let mut controls = Vec::new();

    while let Some((next_controls, next_part)) =
        consume_inline_controls_to_text(data, cursor, dictionary)?
    {
        if next_controls.is_empty() {
            break;
        }
        controls.extend(next_controls);
        cursor = next_part.end;
        parts.push(next_part);
    }
    Ok((parts, controls, cursor))
}

fn advance_script_token(data: &[u8], position: usize, dictionary: &[String]) -> Result<usize> {
    let value = data[position];
    match value {
        0x06 => skip_delimited(data, position, 0x06),
        0x07..=0x09 => {
            let immediate_size = usize::from(value - 0x06);
            let end = position
                .checked_add(1 + immediate_size)
                .ok_or_else(|| invalid("script token position overflows usize"))?;
            if end > data.len() {
                return Err(invalid(format!(
                    "truncated 0x{value:02X} immediate at decoded offset 0x{position:X}"
                )));
            }
            Ok(end)
        }
        _ => Ok(decode_text_token(data, position, dictionary)
            .map(|(_, next)| next)
            .unwrap_or(position + 1)),
    }
}

fn parse_script(data: &[u8]) -> Result<ParsedScript> {
    let (entry_offset, dictionary) = parse_dictionary(data)?;
    let mut records = Vec::new();
    let mut position = entry_offset;
    while position < data.len() {
        if data[position] != 0x11 {
            position = advance_script_token(data, position, &dictionary)?;
            continue;
        }

        let inst_offset = position;
        let text_start = position + 1;
        let Some((first_character, first_end)) = decode_text_token(data, text_start, &dictionary)
        else {
            position += 1;
            continue;
        };

        let (bracketed, static_name, dynamic_name, name_controls, message_start) =
            if first_character == "【" {
                let Some(name) = parse_name(data, first_end, &dictionary)? else {
                    position += 1;
                    continue;
                };
                (
                    true,
                    name.static_name,
                    name.dynamic,
                    name.controls,
                    name.after_close,
                )
            } else {
                (false, None, false, Vec::new(), text_start)
            };

        let (message_parts, message_controls, end) =
            parse_message_parts(data, message_start, &dictionary)?;
        if message_parts.is_empty() {
            position += 1;
            continue;
        }
        records.push(ScriptRecord {
            inst_offset,
            bracketed,
            static_name,
            dynamic_name,
            name_controls,
            message_parts,
            message_controls,
        });
        position = end.max(position + 1);
    }

    Ok(ParsedScript {
        dictionary,
        records,
    })
}

fn control_metadata(
    data: &[u8],
    controls: &[InlineControl],
    parts: &[TextPart],
) -> Vec<ControlMetadata> {
    controls
        .iter()
        .map(|control| {
            let after_part = parts
                .iter()
                .position(|part| part.end == control.start)
                .unwrap_or_else(|| {
                    parts
                        .iter()
                        .take_while(|part| part.end <= control.start)
                        .count()
                        .saturating_sub(1)
                });
            ControlMetadata {
                after_part,
                offset: control.start,
                hex: hex::encode(&data[control.start..control.end]),
            }
        })
        .collect()
}

fn record_to_entry(data: &[u8], index: usize, record: &ScriptRecord) -> TextEntry {
    let first = &record.message_parts[0];
    let message_size = record
        .message_parts
        .iter()
        .map(|part| part.end - part.start)
        .sum();
    let message_texts: Vec<_> = record
        .message_parts
        .iter()
        .map(|part| part.text.clone())
        .collect();
    let multipart = message_texts.len() > 1;
    let static_name = record.static_name.as_ref().map(|part| part.text.clone());
    let kind = if record.bracketed { "dialogue" } else { "text" };

    TextEntry {
        index,
        inst_offset: record.inst_offset,
        offset: first.start,
        size: message_size,
        kind: kind.to_owned(),
        encoding: TEXT_ENCODING.to_owned(),
        policy: TEXT_POLICY.to_owned(),
        scr_name: static_name.clone(),
        name: static_name,
        name_dynamic: record.dynamic_name,
        name_controls: control_metadata(data, &record.name_controls, &[]),
        scr_msg: (!multipart).then(|| message_texts[0].clone()),
        message: (!multipart).then(|| message_texts[0].clone()),
        scr_msg_parts: multipart.then(|| message_texts.clone()),
        message_parts: multipart.then_some(message_texts),
        message_controls: control_metadata(data, &record.message_controls, &record.message_parts),
    }
}

pub fn extract_document(stored: &[u8], file: impl Into<String>) -> Result<TextDocument> {
    let (decoded, stats) = decompress(stored)?;
    if stats.trailing_bytes != 0 || stats.padding_value != 0 {
        return Err(invalid(
            "MES stream has trailing bytes or nonzero padding bits",
        ));
    }
    let parsed = parse_script(&decoded)?;
    let entries = parsed
        .records
        .iter()
        .enumerate()
        .map(|(index, record)| record_to_entry(&decoded, index, record))
        .collect();
    Ok(TextDocument {
        format: JSON_FORMAT.to_owned(),
        file: file.into(),
        source_sha256: hex::encode(Sha256::digest(stored)),
        entries,
    })
}

fn validate_text_input(text: &str, field: &str) -> Result<()> {
    if text.contains('\0') {
        return Err(invalid(format!("{field} contains NUL")));
    }
    if text.contains('\r') || text.contains('\n') {
        return Err(invalid(format!("{field} contains a newline")));
    }
    Ok(())
}

fn encode_text(text: &str, dictionary: &[String], field: &str) -> Result<Vec<u8>> {
    validate_text_input(text, field)?;
    let mut output = Vec::new();
    for character in text.chars() {
        let value = character.to_string();
        if let Some(index) = dictionary.iter().position(|entry| entry == &value) {
            output.push(0xD0 + u8::try_from(index).expect("dictionary is limited to 48 entries"));
            continue;
        }

        let (encoded, _, had_errors) = SHIFT_JIS.encode(&value);
        if had_errors {
            return Err(invalid(format!(
                "{field} contains a character that CP932 cannot encode: {character}"
            )));
        }
        let raw: Cow<'_, [u8]> = encoded;
        if raw.len() != 2 {
            return Err(invalid(format!(
                "{field} character must encode as two-byte CP932 or use the script dictionary: \
                 {character}"
            )));
        }
        let lead = raw[0];
        if !((0x81..=0x9F).contains(&lead) || (0xE0..=0xEF).contains(&lead)) {
            return Err(invalid(format!(
                "{field} has unsupported CP932 lead byte 0x{lead:02X} for {character}"
            )));
        }
        output.push(lead - 0x20);
        output.push(raw[1]);
    }
    Ok(output)
}

fn validate_common_entry(json: &TextEntry, expected: &TextEntry, index: usize) -> Result<()> {
    if json.index != index
        || json.inst_offset != expected.inst_offset
        || json.offset != expected.offset
        || json.size != expected.size
        || json.kind != expected.kind
        || json.encoding != TEXT_ENCODING
        || json.policy != TEXT_POLICY
        || json.name_dynamic != expected.name_dynamic
        || json.name_controls != expected.name_controls
        || json.message_controls != expected.message_controls
    {
        return Err(invalid(format!(
            "JSON metadata mismatch for entry {index} at instruction 0x{:X}",
            expected.inst_offset
        )));
    }
    Ok(())
}

fn prepare_entry_patches(
    decoded: &[u8],
    parsed: &ParsedScript,
    record: &ScriptRecord,
    json: &TextEntry,
    expected: &TextEntry,
    patches: &mut Vec<Patch>,
) -> Result<(usize, usize)> {
    let mut patched = 0usize;
    let mut unchanged = 0usize;

    match (&record.static_name, &expected.scr_name) {
        (Some(name_part), Some(source_name)) => {
            if json.scr_name.as_deref() != Some(source_name) {
                return Err(invalid(format!(
                    "entry {} _scr_name does not match the source",
                    expected.index
                )));
            }
            let translated = json
                .name
                .as_ref()
                .ok_or_else(|| invalid(format!("entry {} is missing name", expected.index)))?;
            if translated.contains('【') || translated.contains('】') {
                return Err(invalid(format!(
                    "entry {} name contains structural brackets",
                    expected.index
                )));
            }
            if translated == source_name {
                unchanged += 1;
            } else {
                patches.push(Patch {
                    start: name_part.start,
                    end: name_part.end,
                    replacement: encode_text(
                        translated,
                        &parsed.dictionary,
                        &format!("entry {} name", expected.index),
                    )?,
                });
                patched += 1;
            }
        }
        (None, None) => {
            if json.name.is_some() || json.scr_name.is_some() {
                return Err(invalid(format!(
                    "entry {} has a dynamic or absent name and cannot accept name text",
                    expected.index
                )));
            }
        }
        _ => {
            return Err(invalid(format!(
                "entry {} name metadata does not match the source",
                expected.index
            )));
        }
    }

    if record.message_parts.len() == 1 {
        let source = expected.scr_msg.as_ref().expect("single-part source text");
        if json.scr_msg.as_ref() != Some(source)
            || json.scr_msg_parts.is_some()
            || json.message_parts.is_some()
        {
            return Err(invalid(format!(
                "entry {} scr_msg or part layout does not match the source",
                expected.index
            )));
        }
        let translated = json
            .message
            .as_ref()
            .ok_or_else(|| invalid(format!("entry {} is missing message", expected.index)))?;
        if translated == source {
            unchanged += 1;
        } else {
            let part = &record.message_parts[0];
            patches.push(Patch {
                start: part.start,
                end: part.end,
                replacement: encode_text(
                    translated,
                    &parsed.dictionary,
                    &format!("entry {} message", expected.index),
                )?,
            });
            patched += 1;
        }
    } else {
        let source_parts = expected
            .scr_msg_parts
            .as_ref()
            .expect("multipart source text");
        if json.scr_msg_parts.as_ref() != Some(source_parts)
            || json.scr_msg.is_some()
            || json.message.is_some()
        {
            return Err(invalid(format!(
                "entry {} scr_msg_parts or field layout does not match the source",
                expected.index
            )));
        }
        let translated_parts = json
            .message_parts
            .as_ref()
            .ok_or_else(|| invalid(format!("entry {} is missing message_parts", expected.index)))?;
        if translated_parts.len() != record.message_parts.len() {
            return Err(invalid(format!(
                "entry {} message_parts count changed from {} to {}",
                expected.index,
                record.message_parts.len(),
                translated_parts.len()
            )));
        }
        for (part_index, ((part, source), translated)) in record
            .message_parts
            .iter()
            .zip(source_parts)
            .zip(translated_parts)
            .enumerate()
        {
            if translated == source {
                unchanged += 1;
            } else {
                patches.push(Patch {
                    start: part.start,
                    end: part.end,
                    replacement: encode_text(
                        translated,
                        &parsed.dictionary,
                        &format!("entry {} message_parts[{part_index}]", expected.index),
                    )?,
                });
                patched += 1;
            }
        }
    }

    let _ = decoded;
    Ok((patched, unchanged))
}

pub fn inject_document(
    source_stored: &[u8],
    document: &TextDocument,
    expected_file: &str,
) -> Result<InjectResult> {
    if document.format != JSON_FORMAT {
        return Err(invalid(format!(
            "unsupported JSON format {}; expected {JSON_FORMAT}",
            document.format
        )));
    }
    if document.file != expected_file {
        return Err(invalid(format!(
            "JSON _file is {}; expected {expected_file}",
            document.file
        )));
    }
    let source_sha256 = hex::encode(Sha256::digest(source_stored));
    if document.source_sha256 != source_sha256 {
        return Err(invalid(format!(
            "source MES SHA-256 mismatch for {expected_file}"
        )));
    }

    let (decoded, stats) = decompress(source_stored)?;
    if stats.trailing_bytes != 0 || stats.padding_value != 0 {
        return Err(invalid(
            "MES stream has trailing bytes or nonzero padding bits",
        ));
    }
    let parsed = parse_script(&decoded)?;
    if document.entries.len() != parsed.records.len() {
        return Err(invalid(format!(
            "JSON has {} entries but source MES has {}",
            document.entries.len(),
            parsed.records.len()
        )));
    }

    let expected_entries: Vec<_> = parsed
        .records
        .iter()
        .enumerate()
        .map(|(index, record)| record_to_entry(&decoded, index, record))
        .collect();
    let mut patches = Vec::new();
    let mut patched_fields = 0usize;
    let mut unchanged_fields = 0usize;
    for (index, ((json, record), expected)) in document
        .entries
        .iter()
        .zip(&parsed.records)
        .zip(&expected_entries)
        .enumerate()
    {
        validate_common_entry(json, expected, index)?;
        let (patched, unchanged) =
            prepare_entry_patches(&decoded, &parsed, record, json, expected, &mut patches)?;
        patched_fields += patched;
        unchanged_fields += unchanged;
    }

    if patches.is_empty() {
        return Ok(InjectResult {
            stored: source_stored.to_vec(),
            json_entries: document.entries.len(),
            patched_fields,
            unchanged_fields,
            source_bytes: source_stored.len(),
            output_bytes: source_stored.len(),
            decoded_source_bytes: decoded.len(),
            decoded_output_bytes: decoded.len(),
        });
    }

    patches.sort_by_key(|patch| patch.start);
    for pair in patches.windows(2) {
        if pair[0].end > pair[1].start {
            return Err(invalid(format!(
                "overlapping text patches at 0x{:X} and 0x{:X}",
                pair[0].start, pair[1].start
            )));
        }
    }

    let mut rebuilt = decoded.clone();
    for patch in patches.iter().rev() {
        rebuilt.splice(patch.start..patch.end, patch.replacement.iter().copied());
    }
    let stored = compress(&rebuilt)?;
    let output_bytes = stored.len();
    Ok(InjectResult {
        stored,
        json_entries: document.entries.len(),
        patched_fields,
        unchanged_fields,
        source_bytes: source_stored.len(),
        output_bytes,
        decoded_source_bytes: decoded.len(),
        decoded_output_bytes: rebuilt.len(),
    })
}

pub fn document_to_json(document: &TextDocument) -> Result<Vec<u8>> {
    let mut output = serde_json::to_string_pretty(document)
        .map_err(|error| invalid(format!("cannot serialize text JSON: {error}")))?;
    output.push('\n');
    Ok(output.into_bytes())
}

pub fn document_from_json(data: &[u8]) -> Result<TextDocument> {
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err(invalid("translation JSON must be UTF-8 without BOM"));
    }
    serde_json::from_slice(data)
        .map_err(|error| invalid(format!("invalid UTF-8 translation JSON: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mes::compress;

    fn encoded_character(character: char) -> Vec<u8> {
        let text = character.to_string();
        let (raw, _, errors) = SHIFT_JIS.encode(&text);
        assert!(!errors);
        assert_eq!(raw.len(), 2);
        vec![raw[0] - 0x20, raw[1]]
    }

    fn sample_plain() -> Vec<u8> {
        let dictionary_chars = ["【", "】", "亜", "由", "美", "。"];
        let entry_offset = 2 + dictionary_chars.len() * 2;
        let mut plain = (entry_offset as u16).to_le_bytes().to_vec();
        for character in dictionary_chars {
            let (raw, _, errors) = SHIFT_JIS.encode(character);
            assert!(!errors);
            plain.extend_from_slice(&raw);
        }

        plain.extend_from_slice(&[0x11, 0xD0, 0xD2, 0xD3, 0xD4, 0xD1]);
        plain.extend_from_slice(&encoded_character('こ'));
        plain.extend_from_slice(&encoded_character('ん'));
        plain.extend_from_slice(&encoded_character('に'));
        plain.extend_from_slice(&encoded_character('ち'));
        plain.extend_from_slice(&encoded_character('は'));
        plain.push(0xD5);
        plain.push(0x11);
        plain.extend_from_slice(&encoded_character('ガ'));
        plain.extend_from_slice(&encoded_character('チ'));
        plain.extend_from_slice(&encoded_character('ャ'));
        plain.push(0xD5);
        plain.push(0);
        plain
    }

    fn dynamic_multipart_plain() -> Vec<u8> {
        let dictionary_chars = ["【", "】", "《", "》"];
        let entry_offset = 2 + dictionary_chars.len() * 2;
        let mut plain = (entry_offset as u16).to_le_bytes().to_vec();
        for character in dictionary_chars {
            let (raw, _, errors) = SHIFT_JIS.encode(character);
            assert!(!errors);
            plain.extend_from_slice(&raw);
        }

        plain.extend_from_slice(&[0x11, 0xD0, 0x13, 0x30, 0x03, 0xD1, 0xD2, 0x13, 0x30, 0x03]);
        plain.extend_from_slice(&encoded_character('ク'));
        plain.extend_from_slice(&encoded_character('ン'));
        plain.push(0xD3);
        plain.push(0);
        plain
    }

    fn lexical_decoy_plain() -> Vec<u8> {
        let dictionary_chars = ["文", "。"];
        let entry_offset = 2 + dictionary_chars.len() * 2;
        let mut plain = (entry_offset as u16).to_le_bytes().to_vec();
        for character in dictionary_chars {
            let (raw, _, errors) = SHIFT_JIS.encode(character);
            assert!(!errors);
            plain.extend_from_slice(&raw);
        }

        plain.extend_from_slice(&[0x08, 0x11, 0xD0]);
        plain.extend_from_slice(&[0x06, b'A', 0x11, 0xD0, 0x06]);
        plain.extend_from_slice(&[0x11, 0xD0, 0xD1, 0]);
        plain
    }

    #[test]
    fn extracts_named_and_unbracketed_records() {
        let stored = compress(&sample_plain()).expect("compress");
        let document = extract_document(&stored, "YUNO_A/TEST.MES").expect("extract");
        assert_eq!(document.entries.len(), 2);
        assert_eq!(document.entries[0].name.as_deref(), Some("亜由美"));
        assert_eq!(document.entries[0].scr_msg.as_deref(), Some("こんにちは。"));
        assert_eq!(document.entries[0].kind, "dialogue");
        assert!(document.entries[1].name.is_none());
        assert_eq!(document.entries[1].scr_msg.as_deref(), Some("ガチャ。"));
        assert_eq!(document.entries[1].kind, "text");
    }

    #[test]
    fn unchanged_injection_reuses_original_compressed_bytes() {
        let stored = compress(&sample_plain()).expect("compress");
        let document = extract_document(&stored, "TEST.MES").expect("extract");
        let result = inject_document(&stored, &document, "TEST.MES").expect("inject");
        assert_eq!(result.stored, stored);
        assert_eq!(result.patched_fields, 0);
    }

    #[test]
    fn changes_static_name_and_message_and_reextracts() {
        let stored = compress(&sample_plain()).expect("compress");
        let mut document = extract_document(&stored, "TEST.MES").expect("extract");
        document.entries[0].name = Some("亜美".to_owned());
        document.entries[0].message = Some("こんにちは。こんにちは。".to_owned());
        let result = inject_document(&stored, &document, "TEST.MES").expect("inject");
        assert_eq!(result.patched_fields, 2);

        let updated = extract_document(&result.stored, "TEST.MES").expect("reextract");
        assert_eq!(updated.entries[0].name.as_deref(), Some("亜美"));
        assert_eq!(
            updated.entries[0].message.as_deref(),
            Some("こんにちは。こんにちは。")
        );
        assert_eq!(updated.entries[1].message.as_deref(), Some("ガチャ。"));
    }

    #[test]
    fn rejects_modified_source_validation_text() {
        let stored = compress(&sample_plain()).expect("compress");
        let mut document = extract_document(&stored, "TEST.MES").expect("extract");
        document.entries[0].scr_msg = Some("違う".to_owned());
        let error = inject_document(&stored, &document, "TEST.MES")
            .expect_err("modified scr_msg must fail");
        assert!(error.to_string().contains("scr_msg"));
    }

    #[test]
    fn preserves_dynamic_name_and_multipart_controls() {
        let stored = compress(&dynamic_multipart_plain()).expect("compress");
        let mut document = extract_document(&stored, "TEST.MES").expect("extract");
        assert_eq!(document.entries.len(), 1);
        let entry = &mut document.entries[0];
        assert!(entry.name_dynamic);
        assert!(entry.name.is_none());
        assert_eq!(entry.name_controls[0].hex, "133003");
        assert_eq!(
            entry.message_parts.as_ref().expect("parts"),
            &["《".to_owned(), "クン》".to_owned()]
        );
        entry.message_parts.as_mut().expect("parts")[1] = "クン、いい？》".to_owned();

        let result = inject_document(&stored, &document, "TEST.MES").expect("inject");
        let updated = extract_document(&result.stored, "TEST.MES").expect("reextract");
        assert!(updated.entries[0].name_dynamic);
        assert_eq!(updated.entries[0].name_controls[0].hex, "133003");
        assert_eq!(
            updated.entries[0]
                .message_parts
                .as_ref()
                .expect("updated parts")[1],
            "クン、いい？》"
        );
    }

    #[test]
    fn skips_text_like_bytes_inside_immediates_and_delimited_strings() {
        let stored = compress(&lexical_decoy_plain()).expect("compress");
        let document = extract_document(&stored, "TEST.MES").expect("extract");
        assert_eq!(document.entries.len(), 1);
        assert_eq!(document.entries[0].message.as_deref(), Some("文。"));
    }

    #[test]
    fn rejects_writable_name_for_dynamic_protagonist() {
        let stored = compress(&dynamic_multipart_plain()).expect("compress");
        let mut document = extract_document(&stored, "TEST.MES").expect("extract");
        document.entries[0].name = Some("たくや".to_owned());
        let error =
            inject_document(&stored, &document, "TEST.MES").expect_err("dynamic name must reject");
        assert!(error.to_string().contains("cannot accept name text"));
    }
}
