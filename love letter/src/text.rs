use crate::format::{write_u32, TextUse};
use crate::{Entry, ParsedObj, Result, ToolError};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub scr_msg: String,
    pub message: String,

    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_entry_offset")]
    pub entry_offset: usize,
    #[serde(rename = "_inst_offset")]
    pub inst_offset: usize,
    #[serde(
        rename = "_inst_offsets",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub inst_offsets: Vec<usize>,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub kind: String,
    #[serde(rename = "_opcode")]
    pub opcode: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_policy")]
    pub policy: String,
    #[serde(rename = "_scr_raw")]
    pub scr_raw: String,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    pub scr_name: Option<String>,
    #[serde(rename = "_tag", skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "_split")]
    pub split: bool,
    #[serde(rename = "_quoted")]
    pub quoted: bool,
    #[serde(rename = "_name_separator", skip_serializing_if = "Option::is_none")]
    pub name_separator: Option<String>,
    #[serde(rename = "_quote_open", skip_serializing_if = "Option::is_none")]
    pub quote_open: Option<String>,
    #[serde(rename = "_quote_close", skip_serializing_if = "Option::is_none")]
    pub quote_close: Option<String>,
    #[serde(rename = "_terminator_len")]
    pub terminator_len: usize,
}

#[derive(Debug, Default)]
pub struct ExtractReport {
    pub entries: Vec<TextEntry>,
    pub skipped: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InjectOptions {
    pub write_names: bool,
}

#[derive(Debug, Default)]
pub struct InjectReport {
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
struct SourceParts {
    raw: String,
    tag: Option<String>,
    id: Option<String>,
    prefix: String,
    display: String,
    name: Option<String>,
    message: String,
    split: bool,
    quoted: bool,
    suffix: String,
    terminator_len: usize,
}

pub fn extract_entries(parsed: &ParsedObj, file_name: &str) -> Result<ExtractReport> {
    let mut report = ExtractReport::default();
    for entry in &parsed.entries {
        let Some(text_use) = entry.text_use() else {
            report.skipped += 1;
            if entry.has_text_reference() {
                report.warnings.push(format!(
                    "{} entry {} at 0x{:X}: string is shared by incompatible text/non-text consumers",
                    file_name, entry.index, entry.data_offset
                ));
            }
            continue;
        };
        let parts = match source_parts(entry) {
            Ok(parts) => parts,
            Err(error) => {
                report.skipped += 1;
                report.warnings.push(format!(
                    "{} entry {} at 0x{:X}: {error}",
                    file_name, entry.index, entry.data_offset
                ));
                continue;
            }
        };
        if parts.display.trim().is_empty() {
            report.skipped += 1;
            continue;
        }
        let Some(&inst_offset) = entry.reference_offsets.first() else {
            report.skipped += 1;
            report.warnings.push(format!(
                "{} entry {} at 0x{:X}: no opcode 0x02 reference",
                file_name, entry.index, entry.data_offset
            ));
            continue;
        };

        let scr_msg = parts.message.clone();
        let name = parts.name.clone();
        let scr_name = name.clone();
        let kind = classify(&parts, text_use).to_string();
        report.entries.push(TextEntry {
            name,
            scr_msg: scr_msg.clone(),
            message: scr_msg,
            file: file_name.to_string(),
            index: entry.index,
            offset: entry.data_offset,
            entry_offset: entry.start,
            inst_offset,
            inst_offsets: if entry.reference_offsets.len() > 1 {
                entry.reference_offsets[1..].to_vec()
            } else {
                Vec::new()
            },
            size: entry.original_len,
            kind,
            opcode: "0x02".to_string(),
            encoding: "CP932".to_string(),
            policy: "relocate".to_string(),
            scr_raw: parts.raw,
            scr_name,
            tag: parts.tag,
            id: parts.id,
            split: parts.split,
            quoted: parts.quoted,
            name_separator: parts.split.then(|| "　".to_string()),
            quote_open: parts.quoted.then(|| "「".to_string()),
            quote_close: parts.quoted.then(|| "」".to_string()),
            terminator_len: parts.terminator_len,
        });
    }
    Ok(report)
}

pub fn inject_entries(
    parsed: &ParsedObj,
    file_name: &str,
    json_entries: &[TextEntry],
    options: InjectOptions,
) -> Result<(Vec<u8>, InjectReport)> {
    let mut report = InjectReport {
        json_entries: json_entries.len(),
        ..InjectReport::default()
    };
    let mut seen = BTreeSet::new();
    let mut replacement_data: Vec<Vec<u8>> =
        parsed.entries.iter().map(|e| e.data.clone()).collect();

    for item in json_entries {
        if !seen.insert(item.index) {
            return Err(ToolError::Text(format!(
                "{} JSON contains duplicate _index {}",
                file_name, item.index
            )));
        }
        if item.file != file_name {
            return Err(ToolError::Text(format!(
                "JSON _file {:?} does not match source file {:?} at _index {}",
                item.file, file_name, item.index
            )));
        }
        let source = parsed.entry_by_index(item.index).ok_or_else(|| {
            ToolError::Text(format!(
                "{} JSON _index {} is outside the source table",
                file_name, item.index
            ))
        })?;
        if source.data_offset != item.offset || source.start != item.entry_offset {
            return Err(ToolError::Text(format!(
                "{} _index {} location mismatch: JSON entry 0x{:X}/0x{:X}, source 0x{:X}/0x{:X}",
                file_name,
                item.index,
                item.entry_offset,
                item.offset,
                source.start,
                source.data_offset
            )));
        }
        let text_use = source.text_use().ok_or_else(|| {
            ToolError::Text(format!(
                "{} _index {} is not exclusively consumed by a confirmed text function",
                file_name, item.index
            ))
        })?;
        let parts = source_parts(source)?;
        let expected_kind = classify(&parts, text_use);
        let expected_offsets: Vec<usize> =
            source.reference_offsets.iter().copied().skip(1).collect();
        if source.original_len != item.size
            || source.reference_offsets.first().copied() != Some(item.inst_offset)
            || expected_offsets != item.inst_offsets
            || item.kind != expected_kind
            || item.opcode != "0x02"
            || item.encoding != "CP932"
            || item.policy != "relocate"
            || item.terminator_len != parts.terminator_len
        {
            return Err(ToolError::Text(format!(
                "{} _index {} instruction/type/size metadata mismatch",
                file_name, item.index
            )));
        }
        if parts.raw != item.scr_raw {
            return Err(ToolError::Text(format!(
                "{} _index {} _scr_raw mismatch at 0x{:X}",
                file_name, item.index, source.data_offset
            )));
        }
        if parts.message != item.scr_msg {
            return Err(ToolError::Text(format!(
                "{} _index {} scr_msg mismatch at 0x{:X}",
                file_name, item.index, source.data_offset
            )));
        }
        if parts.split != item.split || parts.quoted != item.quoted {
            return Err(ToolError::Text(format!(
                "{} _index {} split/quote metadata mismatch",
                file_name, item.index
            )));
        }
        let expected_separator = parts.split.then(|| "　".to_string());
        let expected_open = parts.quoted.then(|| "「".to_string());
        let expected_close = parts.quoted.then(|| "」".to_string());
        if item.tag != parts.tag
            || item.id != parts.id
            || item.name_separator != expected_separator
            || item.quote_open != expected_open
            || item.quote_close != expected_close
        {
            return Err(ToolError::Text(format!(
                "{} _index {} tag/name/quote metadata mismatch",
                file_name, item.index
            )));
        }
        let source_name = parts.name.clone();
        if source_name != item.scr_name {
            return Err(ToolError::Text(format!(
                "{} _index {} _scr_name mismatch",
                file_name, item.index
            )));
        }

        let new_message = item.message.clone();
        ensure_control_signature(
            &item.scr_msg,
            &new_message,
            "message",
            file_name,
            item.index,
        )?;
        let new_name = if parts.split {
            let Some(name) = item.name.as_deref() else {
                return Err(ToolError::Text(format!(
                    "{} _index {} has a split name but JSON name is missing",
                    file_name, item.index
                )));
            };
            let source_name = item.scr_name.as_deref().ok_or_else(|| {
                ToolError::Text(format!(
                    "{} _index {} has a split name but _scr_name is missing",
                    file_name, item.index
                ))
            })?;
            if name != source_name && !options.write_names {
                return Err(ToolError::Text(format!(
                    "{} _index {} changes name; rerun with --write-names after reviewing _scr_name",
                    file_name, item.index
                )));
            }
            ensure_control_signature(source_name, name, "name", file_name, item.index)?;
            Some(name.to_string())
        } else {
            if item.name.is_some() || item.scr_name.is_some() {
                return Err(ToolError::Text(format!(
                    "{} _index {} has name metadata but source is not split",
                    file_name, item.index
                )));
            }
            None
        };

        let unchanged =
            new_message == item.scr_msg && new_name.as_deref() == item.scr_name.as_deref();
        if unchanged {
            report.unchanged += 1;
            continue;
        }

        let translated_display = if parts.split {
            let close = if parts.quoted { "」" } else { "" };
            format!(
                "{}{}{}{}{}{}",
                new_name.as_deref().unwrap_or_default(),
                if parts.split { "　" } else { "" },
                if parts.quoted { "「" } else { "" },
                new_message,
                close,
                parts.suffix
            )
        } else {
            new_message
        };
        let translated = format!("{}{}", parts.prefix, translated_display);
        let mut encoded = encode_text_with_controls(&translated)?;
        encoded.extend(std::iter::repeat_n(0, parts.terminator_len));
        replacement_data[item.index] = encoded;
        report.patched += 1;
    }

    let mut new_offsets = Vec::with_capacity(parsed.entries.len());
    let mut cursor = parsed.table_offset;
    for data in &replacement_data {
        new_offsets.push(cursor);
        cursor = cursor
            .checked_add(4)
            .and_then(|value| value.checked_add(data.len()))
            .ok_or_else(|| ToolError::Format("rebuilt table size overflow".to_string()))?;
    }

    let mut output = parsed.bytes[..parsed.table_offset].to_vec();
    for data in &replacement_data {
        let length = u32::try_from(data.len())
            .map_err(|_| ToolError::Text("translated record exceeds u32 length".to_string()))?;
        output.extend_from_slice(&length.to_le_bytes());
        output.extend_from_slice(data);
    }

    for (entry, &new_start) in parsed.entries.iter().zip(&new_offsets) {
        for &reference_offset in &entry.reference_offsets {
            if reference_offset >= parsed.table_offset {
                return Err(ToolError::Format(format!(
                    "table reference 0x{reference_offset:X} is not in the instruction region"
                )));
            }
            write_u32(&mut output, reference_offset + 1, new_start)?;
        }
    }

    let rebuilt = crate::parse_obj(&output)?;
    if rebuilt.table_offset != parsed.table_offset || rebuilt.entries.len() != parsed.entries.len()
    {
        return Err(ToolError::Format(
            "rebuilt file no longer has the original table shape".to_string(),
        ));
    }
    for (expected, actual) in replacement_data.iter().zip(rebuilt.entries.iter()) {
        if expected != &actual.data {
            return Err(ToolError::Format(
                "rebuilt table payload differs from prepared replacement".to_string(),
            ));
        }
    }
    Ok((output, report))
}

fn source_parts(entry: &Entry) -> Result<SourceParts> {
    let terminator_len = entry
        .data
        .iter()
        .rev()
        .take_while(|&&byte| byte == 0)
        .count();
    let payload_end = entry
        .data
        .len()
        .checked_sub(terminator_len)
        .ok_or_else(|| {
            ToolError::Text("record terminator length exceeds payload length".to_string())
        })?;
    let raw_bytes = &entry.data[..payload_end];
    let raw = decode_text_with_controls(raw_bytes)?;
    let (tag, id, prefix, display) = parse_tag(&raw);
    let (name, message, split, quoted, suffix) = split_display(&display);
    Ok(SourceParts {
        raw,
        tag,
        id,
        prefix,
        display,
        name,
        message,
        split,
        quoted,
        suffix,
        terminator_len,
    })
}

fn parse_tag(raw: &str) -> (Option<String>, Option<String>, String, String) {
    if !raw.starts_with('!') {
        return (None, None, String::new(), raw.to_string());
    }
    let Some(slash) = raw.find('\\') else {
        return (None, None, String::new(), raw.to_string());
    };
    let Some(id_end_rel) = raw[slash + 1..].find("\\n") else {
        return (None, None, String::new(), raw.to_string());
    };
    let id_end = slash + 1 + id_end_rel;
    let tag = raw[1..slash].to_string();
    let id = raw[slash + 1..id_end].to_string();
    let prefix_end = id_end + 2;
    (
        Some(tag),
        Some(id),
        raw[..prefix_end].to_string(),
        raw[prefix_end..].to_string(),
    )
}

fn split_display(display: &str) -> (Option<String>, String, bool, bool, String) {
    let Some(separator) = display.find("　「") else {
        return (None, display.to_string(), false, false, String::new());
    };
    let name = &display[..separator];
    if name.is_empty() {
        return (None, display.to_string(), false, false, String::new());
    }
    let message_start = separator + "　「".len();
    let Some(close_rel) = display[message_start..].rfind('」') else {
        return (None, display.to_string(), false, false, String::new());
    };
    let close = message_start + close_rel;
    (
        Some(name.to_string()),
        display[message_start..close].to_string(),
        true,
        true,
        display[close + '」'.len_utf8()..].to_string(),
    )
}

fn classify(parts: &SourceParts, text_use: TextUse) -> &'static str {
    match text_use {
        TextUse::Choice => "choice",
        TextUse::Message if parts.tag.as_deref() == Some("se") => "effect",
        TextUse::Message => "dialogue",
    }
}

fn decode_cp932(bytes: &[u8]) -> Option<String> {
    let (text, _, had_errors) = SHIFT_JIS.decode(bytes);
    (!had_errors).then(|| text.into_owned())
}

fn encode_cp932(text: &str) -> Result<Vec<u8>> {
    let (bytes, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        let mut bad = Vec::new();
        for ch in text.chars() {
            let (_, _, errors) = SHIFT_JIS.encode(ch.encode_utf8(&mut [0; 4]));
            if errors && !bad.contains(&ch) {
                bad.push(ch);
            }
        }
        let chars = bad
            .into_iter()
            .map(|ch| format!("U+{:04X}", ch as u32))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(ToolError::Text(format!(
            "text contains characters not representable in CP932: {chars}"
        )));
    }
    Ok(bytes.into_owned())
}

fn decode_text_with_controls(bytes: &[u8]) -> Result<String> {
    let mut output = String::new();
    let mut plain_start = 0usize;
    let mut index = 0usize;
    while index < bytes.len() {
        let control_len = match bytes[index] {
            0x07 => Some(5usize),
            0x08 => Some(17usize),
            0x09 => Some(1usize),
            0x0A => Some(13usize),
            byte if byte < 0x20 && !matches!(byte, b'\t' | b'\n' | b'\r') => Some(1usize),
            _ => None,
        };
        let Some(control_len) = control_len else {
            index += 1;
            continue;
        };

        append_cp932_chunk(&mut output, &bytes[plain_start..index], plain_start)?;
        let end = index.checked_add(control_len).ok_or_else(|| {
            ToolError::Text(format!(
                "control structure size overflow at byte 0x{index:X}"
            ))
        })?;
        if end > bytes.len() {
            return Err(ToolError::Text(format!(
                "truncated control 0x{:02X} at payload byte 0x{index:X}",
                bytes[index]
            )));
        }

        match bytes[index] {
            0x07 => {
                let digits = &bytes[index + 1..end];
                validate_digits(digits, index)?;
                output.push_str(&format!(
                    "{{{{VAR:{}}}}}",
                    std::str::from_utf8(digits).expect("validated ASCII digits")
                ));
            }
            0x08 => {
                let width = i32::from_le_bytes(bytes[index + 1..index + 5].try_into().unwrap());
                let reserved = u32::from_le_bytes(bytes[index + 5..index + 9].try_into().unwrap());
                let wide = u32::from_le_bytes(bytes[index + 9..index + 13].try_into().unwrap());
                let digits = &bytes[index + 13..end];
                validate_digits(digits, index)?;
                output.push_str(&format!(
                    "{{{{VAR_FMT:{}:{width}:{reserved:08X}:{wide:08X}}}}}",
                    std::str::from_utf8(digits).expect("validated ASCII digits")
                ));
            }
            0x09 => output.push_str("{{STACK}}"),
            0x0A => {
                let width = i32::from_le_bytes(bytes[index + 1..index + 5].try_into().unwrap());
                let reserved = u32::from_le_bytes(bytes[index + 5..index + 9].try_into().unwrap());
                let wide = u32::from_le_bytes(bytes[index + 9..index + 13].try_into().unwrap());
                output.push_str(&format!(
                    "{{{{STACK_FMT:{width}:{reserved:08X}:{wide:08X}}}}}"
                ));
            }
            byte => output.push_str(&format!("{{{{CTRL:{byte:02X}}}}}")),
        }
        index = end;
        plain_start = end;
    }
    append_cp932_chunk(&mut output, &bytes[plain_start..], plain_start)?;
    Ok(output)
}

fn append_cp932_chunk(output: &mut String, bytes: &[u8], offset: usize) -> Result<()> {
    let decoded = decode_cp932(bytes).ok_or_else(|| {
        ToolError::Text(format!(
            "CP932 decode failed near payload byte 0x{offset:X}"
        ))
    })?;
    output.push_str(&decoded);
    Ok(())
}

fn validate_digits(bytes: &[u8], offset: usize) -> Result<()> {
    if bytes.len() == 4 && bytes.iter().all(u8::is_ascii_digit) {
        return Ok(());
    }
    Err(ToolError::Text(format!(
        "control at payload byte 0x{offset:X} does not contain a four-digit variable id"
    )))
}

fn encode_text_with_controls(text: &str) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut index = 0;
    while index < text.len() {
        let rest = &text[index..];
        let Some(open_rel) = rest.find("{{") else {
            output.extend_from_slice(&encode_cp932(rest)?);
            break;
        };
        let open = index + open_rel;
        output.extend_from_slice(&encode_cp932(&text[index..open])?);
        let Some(close_rel) = text[open + 2..].find("}}") else {
            return Err(ToolError::Text(format!(
                "unterminated control token near {:?}",
                &text[open..]
            )));
        };
        let close = open + 2 + close_rel;
        let token = &text[open + 2..close];
        if let Some(digits) = token.strip_prefix("VAR:") {
            if digits.len() != 4 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(ToolError::Text(format!(
                    "invalid variable token {{{{{token}}}}}"
                )));
            }
            output.push(0x07);
            output.extend_from_slice(digits.as_bytes());
        } else if let Some(fields) = token.strip_prefix("VAR_FMT:") {
            let fields: Vec<&str> = fields.split(':').collect();
            if fields.len() != 4
                || fields[0].len() != 4
                || !fields[0].bytes().all(|byte| byte.is_ascii_digit())
            {
                return Err(ToolError::Text(format!(
                    "invalid formatted variable token {{{{{token}}}}}"
                )));
            }
            let width = parse_width(fields[1], token)?;
            let reserved = parse_hex_u32(fields[2], token)?;
            let wide = parse_hex_u32(fields[3], token)?;
            output.push(0x08);
            output.extend_from_slice(&width.to_le_bytes());
            output.extend_from_slice(&reserved.to_le_bytes());
            output.extend_from_slice(&wide.to_le_bytes());
            output.extend_from_slice(fields[0].as_bytes());
        } else if token == "STACK" {
            output.push(0x09);
        } else if let Some(fields) = token.strip_prefix("STACK_FMT:") {
            let fields: Vec<&str> = fields.split(':').collect();
            if fields.len() != 3 {
                return Err(ToolError::Text(format!(
                    "invalid formatted stack token {{{{{token}}}}}"
                )));
            }
            let width = parse_width(fields[0], token)?;
            let reserved = parse_hex_u32(fields[1], token)?;
            let wide = parse_hex_u32(fields[2], token)?;
            output.push(0x0A);
            output.extend_from_slice(&width.to_le_bytes());
            output.extend_from_slice(&reserved.to_le_bytes());
            output.extend_from_slice(&wide.to_le_bytes());
        } else if let Some(hex) = token.strip_prefix("CTRL:") {
            if hex.len() != 2 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(ToolError::Text(format!(
                    "invalid control token {{{{{token}}}}}"
                )));
            }
            output.push(
                u8::from_str_radix(hex, 16).map_err(|_| {
                    ToolError::Text(format!("invalid control token {{{{{token}}}}}"))
                })?,
            );
        } else {
            return Err(ToolError::Text(format!(
                "unknown control token {{{{{token}}}}}"
            )));
        }
        index = close + 2;
    }
    Ok(output)
}

fn parse_width(value: &str, token: &str) -> Result<i32> {
    value.parse::<i32>().map_err(|_| {
        ToolError::Text(format!(
            "invalid signed width in control token {{{{{token}}}}}"
        ))
    })
}

fn parse_hex_u32(value: &str, token: &str) -> Result<u32> {
    if value.len() != 8 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ToolError::Text(format!(
            "invalid u32 field in control token {{{{{token}}}}}"
        )));
    }
    u32::from_str_radix(value, 16).map_err(|_| {
        ToolError::Text(format!(
            "invalid u32 field in control token {{{{{token}}}}}"
        ))
    })
}

fn control_signature(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut index = 0;
    while let Some(open_rel) = text[index..].find("{{") {
        let open = index + open_rel;
        let Some(close_rel) = text[open + 2..].find("}}") else {
            break;
        };
        let close = open + 2 + close_rel;
        let token = &text[open + 2..close];
        if token.starts_with("VAR:")
            || token.starts_with("VAR_FMT:")
            || token == "STACK"
            || token.starts_with("STACK_FMT:")
            || token.starts_with("CTRL:")
        {
            result.push(token.to_string());
        }
        index = close + 2;
    }
    result
}

fn ensure_control_signature(
    original: &str,
    replacement: &str,
    field: &str,
    file_name: &str,
    index: usize,
) -> Result<()> {
    let original_signature = control_signature(original);
    let replacement_signature = control_signature(replacement);
    if original_signature != replacement_signature {
        return Err(ToolError::Text(format!(
            "{} _index {} {} control tokens changed: {:?} -> {:?}",
            file_name, index, field, original_signature, replacement_signature
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_obj;

    const MESSAGE_FUNCTION: u32 = 0x15C27;
    const CHOICE_FUNCTION: u32 = 0x1870E;

    fn encode_test(text: &str) -> Vec<u8> {
        let (bytes, _, had_errors) = SHIFT_JIS.encode(text);
        assert!(!had_errors, "test text must be representable in CP932");
        bytes.into_owned()
    }

    fn synthetic_obj(strings: &[(&str, u8, u32)]) -> Vec<u8> {
        let table_offset = strings.len() * 10;
        let payloads: Vec<Vec<u8>> = strings
            .iter()
            .map(|(text, _, _)| {
                let mut bytes = encode_test(text);
                bytes.push(0);
                bytes
            })
            .collect();
        let mut starts = Vec::with_capacity(payloads.len());
        let mut cursor = table_offset;
        for payload in &payloads {
            starts.push(cursor);
            cursor += 4 + payload.len();
        }
        let mut bytes = Vec::with_capacity(cursor);
        for ((_, consumer_opcode, consumer_argument), start) in strings.iter().zip(&starts) {
            bytes.push(0x02);
            bytes.extend_from_slice(&(*start as u32).to_le_bytes());
            bytes.push(*consumer_opcode);
            bytes.extend_from_slice(&consumer_argument.to_le_bytes());
        }
        for (payload, start) in payloads.iter().zip(starts) {
            bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            bytes.extend_from_slice(payload);
            assert_eq!(bytes.len(), start + 4 + payload.len());
        }
        bytes
    }

    #[test]
    fn cp932_round_trip_is_byte_stable() {
        let text = "やよい　「瑞穂……」";
        let encoded = encode_test(text);
        assert_eq!(decode_cp932(&encoded).as_deref(), Some(text));
        assert_eq!(encode_cp932(text).expect("CP932 encode"), encoded);
    }

    #[test]
    fn splits_name_message_and_preserves_suffix() {
        let (name, message, split, quoted, suffix) = split_display("やよい　「瑞穂……」{{STACK}}");
        assert_eq!(name.as_deref(), Some("やよい"));
        assert_eq!(message, "瑞穂……");
        assert!(split);
        assert!(quoted);
        assert_eq!(suffix, "{{STACK}}");
    }

    #[test]
    fn structured_controls_round_trip_and_require_same_signature() {
        let mut source = encode_test("３月");
        source.push(0x08);
        source.extend_from_slice(&2i32.to_le_bytes());
        source.extend_from_slice(&0x11223344u32.to_le_bytes());
        source.extend_from_slice(&1u32.to_le_bytes());
        source.extend_from_slice(b"0267");
        source.extend_from_slice(&encode_test("日"));
        let tokenized = decode_text_with_controls(&source).expect("decode controls");
        assert_eq!(tokenized, "３月{{VAR_FMT:0267:2:11223344:00000001}}日");
        assert_eq!(
            encode_text_with_controls(&tokenized).expect("encode controls"),
            source
        );

        let variable = decode_text_with_controls(b"\x070083").expect("decode variable");
        assert_eq!(variable, "{{VAR:0083}}");
        assert_eq!(
            encode_text_with_controls(&variable).expect("encode variable"),
            b"\x070083"
        );
        assert!(ensure_control_signature(&tokenized, "３月日", "message", "test.o", 0).is_err());
        assert!(encode_text_with_controls("{{VAR:83}}").is_err());
    }

    #[test]
    fn extracts_only_confirmed_message_and_choice_consumers() {
        let bytes = synthetic_obj(&[
            ("!m\\yay01000\\nやよい　「瑞穂……」", 0x06, MESSAGE_FUNCTION),
            ("\u{7}0083　「もう朝か……」", 0x06, MESSAGE_FUNCTION),
            ("　……………………", 0x06, MESSAGE_FUNCTION),
            ("　１．進む", 0x06, CHOICE_FUNCTION),
            ("　", 0x06, CHOICE_FUNCTION),
            ("savefile\u{7}0111.dat", 0x05, 0x400),
            ("環境.dat", 0x05, 0x400),
        ]);
        let parsed = parse_obj(&bytes).expect("synthetic object should parse");
        let report = extract_entries(&parsed, "M_01.o").expect("extract");

        assert_eq!(report.entries.len(), 4);
        let named = &report.entries[0];
        assert_eq!(named.name.as_deref(), Some("やよい"));
        assert_eq!(named.scr_msg, "瑞穂……");
        assert_eq!(named.scr_raw, "!m\\yay01000\\nやよい　「瑞穂……」");
        assert!(named.split);

        let hero = &report.entries[1];
        assert_eq!(hero.name.as_deref(), Some("{{VAR:0083}}"));
        assert_eq!(hero.scr_msg, "もう朝か……");
        assert_eq!(hero.scr_name.as_deref(), Some("{{VAR:0083}}"));
        assert_eq!(report.entries[2].kind, "dialogue");
        assert_eq!(report.entries[2].scr_msg, "　……………………");
        assert_eq!(report.entries[3].kind, "choice");
        assert_eq!(report.entries[3].scr_msg, "　１．進む");
        assert!(!report
            .entries
            .iter()
            .any(|entry| entry.scr_msg.contains("savefile") || entry.scr_msg.contains("環境")));
    }

    #[test]
    fn effect_kind_comes_from_message_sink_and_tag() {
        let bytes = synthetic_obj(&[("!se\\se002\\n　チチチッ！", 0x06, MESSAGE_FUNCTION)]);
        let parsed = parse_obj(&bytes).expect("synthetic object should parse");
        let report = extract_entries(&parsed, "M_01.o").expect("extract");

        assert_eq!(report.entries.len(), 1);
        assert_eq!(report.entries[0].kind, "effect");
        assert_eq!(report.entries[0].scr_msg, "　チチチッ！");
    }

    #[test]
    fn inject_rebuilds_variable_length_records_and_offsets() {
        let bytes = synthetic_obj(&[
            ("!m\\yay01000\\nやよい　「短」", 0x06, MESSAGE_FUNCTION),
            ("!m\\miz01000\\n瑞穂　「二」", 0x06, MESSAGE_FUNCTION),
        ]);
        let parsed = parse_obj(&bytes).expect("synthetic object should parse");
        let extracted = extract_entries(&parsed, "M_01.o").expect("extract");
        assert_eq!(extracted.entries.len(), 2);

        let mut changed = extracted.entries[0].clone();
        changed.message = "これは長い翻訳文です".to_string();
        let unchanged = extracted.entries[1].clone();
        let (rebuilt, report) = inject_entries(
            &parsed,
            "M_01.o",
            &[changed, unchanged],
            InjectOptions::default(),
        )
        .expect("inject");

        assert_eq!(report.json_entries, 2);
        assert_eq!(report.patched, 1);
        assert_eq!(report.unchanged, 1);
        assert!(rebuilt.len() > bytes.len());

        let reparsed = parse_obj(&rebuilt).expect("rebuilt object should parse");
        assert!(reparsed.entries[1].data_offset > parsed.entries[1].data_offset);
        let target = u32::from_le_bytes(rebuilt[11..15].try_into().expect("reference bytes"));
        assert_eq!(target as usize, reparsed.entries[1].start);
        let translated = source_parts(&reparsed.entries[0]).expect("translated source");
        assert_eq!(translated.name.as_deref(), Some("やよい"));
        assert_eq!(translated.message, "これは長い翻訳文です");
    }

    #[test]
    fn rejects_json_location_mismatch_and_name_without_write_flag() {
        let bytes = synthetic_obj(&[("!m\\yay01000\\nやよい　「短」", 0x06, MESSAGE_FUNCTION)]);
        let parsed = parse_obj(&bytes).expect("synthetic object should parse");
        let extracted = extract_entries(&parsed, "M_01.o").expect("extract");

        let mut wrong_location = extracted.entries[0].clone();
        wrong_location.offset += 1;
        assert!(inject_entries(
            &parsed,
            "M_01.o",
            &[wrong_location],
            InjectOptions::default()
        )
        .is_err());

        let mut renamed = extracted.entries[0].clone();
        renamed.name = Some("別名".to_string());
        assert!(inject_entries(
            &parsed,
            "M_01.o",
            &[renamed.clone()],
            InjectOptions::default()
        )
        .is_err());
        let (rebuilt, report) = inject_entries(
            &parsed,
            "M_01.o",
            &[renamed],
            InjectOptions { write_names: true },
        )
        .expect("explicit name write should work");
        assert_eq!(report.patched, 1);
        let reparsed = parse_obj(&rebuilt).expect("renamed object should parse");
        assert_eq!(
            source_parts(&reparsed.entries[0]).unwrap().name.as_deref(),
            Some("別名")
        );
    }

    #[test]
    fn rejects_unrepresentable_text() {
        assert!(encode_cp932("😀").is_err());
    }
}
