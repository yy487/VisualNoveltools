use crate::ab::{
    encode_cp932, parse_script, rebuild_script, AbInstruction, AbScript, AbString, AbStringKind,
    MESSAGE_SEPARATOR,
};
use crate::{ToolError, ToolResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const ENCODING: &str = "CP932";
const POLICY: &str = "relocate";
const FULLWIDTH_SPACE: char = '\u{3000}';

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextType {
    Dialogue,
    Monologue,
    Choice,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    pub scr_name: Option<String>,
    pub scr_msg: String,
    pub message: String,
    #[serde(rename = "_file")]
    pub source_file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_inst_offset")]
    pub instruction_offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub text_type: TextType,
    #[serde(rename = "_opcode")]
    pub opcode: u16,
    #[serde(rename = "_target", skip_serializing_if = "Option::is_none")]
    pub target: Option<u32>,
    #[serde(rename = "_message_id", skip_serializing_if = "Option::is_none")]
    pub message_id: Option<u16>,
    #[serde(rename = "_voice", skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(rename = "_buffer_index", skip_serializing_if = "Option::is_none")]
    pub buffer_index: Option<u16>,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_policy")]
    pub policy: String,
}

#[derive(Debug)]
struct TextSlot {
    string_offset: usize,
    entry: TranslationEntry,
}

#[derive(Debug)]
pub struct InjectedScript {
    pub bytes: Vec<u8>,
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
}

fn find_message_separator(raw: &[u8], instruction: &AbInstruction) -> ToolResult<usize> {
    let matches = raw
        .windows(MESSAGE_SEPARATOR.len())
        .enumerate()
        .filter_map(|(index, value)| (value == MESSAGE_SEPARATOR.as_bytes()).then_some(index))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(ToolError(format!(
            "opcode 0 at 0x{:x} has {} message separators, expected exactly one",
            instruction.offset,
            matches.len()
        )));
    }
    Ok(matches[0])
}

fn classify_named_body(body: &str) -> TextType {
    if body.starts_with(['\u{300c}', '\u{300a}']) {
        TextType::Dialogue
    } else {
        TextType::System
    }
}

fn validate_controls(text: &str, label: &str) -> ToolResult<()> {
    if text.contains('\0') {
        return Err(ToolError(format!("{label} contains NUL")));
    }
    if text.contains(['\r', '\n']) {
        return Err(ToolError(format!(
            "{label} contains a real CR/LF; KOKOROV text uses explicit controls"
        )));
    }

    let bytes = text.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => {
                let next = *bytes.get(cursor + 1).ok_or_else(|| {
                    ToolError(format!("{label} ends with an incomplete backslash control"))
                })?;
                if next.eq_ignore_ascii_case(&b'W') {
                    cursor += 2;
                } else {
                    return Err(ToolError(format!(
                        "{label} contains unsupported control \\{} at UTF-8 byte {cursor}",
                        char::from(next)
                    )));
                }
            }
            b'%' => {
                let kind = *bytes.get(cursor + 1).ok_or_else(|| {
                    ToolError(format!("{label} ends with an incomplete percent control"))
                })?;
                if !kind.eq_ignore_ascii_case(&b'M') {
                    return Err(ToolError(format!(
                        "{label} contains unsupported control %{} at UTF-8 byte {cursor}",
                        char::from(kind)
                    )));
                }
                let digits_start = cursor + 2;
                let mut end = digits_start;
                while bytes.get(end).is_some_and(u8::is_ascii_digit) {
                    end += 1;
                }
                if end == digits_start {
                    return Err(ToolError(format!(
                        "{label} contains %M without a buffer index at UTF-8 byte {cursor}"
                    )));
                }
                let digits = &text[digits_start..end];
                let index = digits.parse::<u16>().map_err(|error| {
                    ToolError(format!("{label} has invalid %M index {digits:?}: {error}"))
                })?;
                if index > 9 {
                    return Err(ToolError(format!(
                        "{label} references message buffer M{index}, expected M0..M9"
                    )));
                }
                cursor = end;
            }
            _ => {
                let character = text[cursor..].chars().next().ok_or_else(|| {
                    ToolError(format!("{label} has invalid UTF-8 boundary at {cursor}"))
                })?;
                cursor += character.len_utf8();
            }
        }
    }
    Ok(())
}

fn validate_name(name: &str, label: &str) -> ToolResult<()> {
    if name.is_empty() {
        return Err(ToolError(format!("{label} is empty")));
    }
    if name.contains(['\0', '\r', '\n', '\\', '%']) {
        return Err(ToolError(format!(
            "{label} contains a forbidden NUL, CR/LF, backslash, or percent sign"
        )));
    }
    Ok(())
}

fn make_message_slot(
    source_file: &str,
    index: usize,
    instruction: &AbInstruction,
    string: &AbString,
) -> ToolResult<TextSlot> {
    let separator = find_message_separator(&string.raw, instruction)?;
    let name_raw = &string.raw[..separator];
    let body_start = separator + MESSAGE_SEPARATOR.len();
    let body_raw = &string.raw[body_start..];
    if body_raw.is_empty() {
        return Err(ToolError(format!(
            "opcode 0 at 0x{:x} has an empty message body",
            instruction.offset
        )));
    }
    let name = if name_raw.is_empty() {
        None
    } else {
        Some(crate::ab::decode_cp932(name_raw, "message name")?)
    };
    let message = crate::ab::decode_cp932(body_raw, "message body")?;
    validate_controls(&message, "source message")?;
    if let Some(name) = &name {
        validate_name(name, "source name")?;
    }
    let text_type = match &name {
        Some(_) => classify_named_body(&message),
        None => TextType::Monologue,
    };
    Ok(TextSlot {
        string_offset: string.offset,
        entry: TranslationEntry {
            name: name.clone(),
            scr_name: name,
            scr_msg: message.clone(),
            message,
            source_file: source_file.to_string(),
            index,
            offset: string.offset + body_start,
            instruction_offset: instruction.offset,
            size: body_raw.len(),
            text_type,
            opcode: instruction.opcode,
            target: None,
            message_id: instruction.message_id,
            voice: instruction.voice.as_ref().map(|voice| voice.text.clone()),
            buffer_index: None,
            encoding: ENCODING.to_string(),
            policy: POLICY.to_string(),
        },
    })
}

fn make_plain_slot(
    source_file: &str,
    index: usize,
    data: &[u8],
    instruction: &AbInstruction,
    string: &AbString,
) -> ToolResult<TextSlot> {
    validate_controls(&string.text, "source text")?;
    let (text_type, target, buffer_index) = match string.kind {
        AbStringKind::Choice => (
            TextType::Choice,
            instruction.targets.first().map(|target| target.value),
            None,
        ),
        AbStringKind::Buffer => {
            let operand = instruction.offset.checked_add(2).ok_or_else(|| {
                ToolError(format!(
                    "opcode 8 at 0x{:x} operand offset overflows",
                    instruction.offset
                ))
            })?;
            let bytes = data.get(operand..operand + 2).ok_or_else(|| {
                ToolError(format!(
                    "opcode 8 at 0x{:x} buffer index is truncated",
                    instruction.offset
                ))
            })?;
            (
                TextType::System,
                None,
                Some(u16::from_le_bytes([bytes[0], bytes[1]])),
            )
        }
        _ => {
            return Err(ToolError(format!(
                "opcode {} at 0x{:x} is not an extractable plain text slot",
                instruction.opcode, instruction.offset
            )));
        }
    };
    Ok(TextSlot {
        string_offset: string.offset,
        entry: TranslationEntry {
            name: None,
            scr_name: None,
            scr_msg: string.text.clone(),
            message: string.text.clone(),
            source_file: source_file.to_string(),
            index,
            offset: string.offset,
            instruction_offset: instruction.offset,
            size: string.raw.len(),
            text_type,
            opcode: instruction.opcode,
            target,
            message_id: None,
            voice: None,
            buffer_index,
            encoding: ENCODING.to_string(),
            policy: POLICY.to_string(),
        },
    })
}

fn collect_slots(source_file: &str, data: &[u8], script: &AbScript) -> ToolResult<Vec<TextSlot>> {
    let mut slots = Vec::new();
    for instruction in &script.instructions {
        for string in &instruction.strings {
            let index = slots.len();
            let slot = match string.kind {
                AbStringKind::Message => {
                    make_message_slot(source_file, index, instruction, string)?
                }
                AbStringKind::Choice | AbStringKind::Buffer => {
                    make_plain_slot(source_file, index, data, instruction, string)?
                }
                AbStringKind::Resource => continue,
            };
            slots.push(slot);
        }
    }
    Ok(slots)
}

pub fn extract_entries(
    source_file: &str,
    data: &[u8],
    script: &AbScript,
) -> ToolResult<Vec<TranslationEntry>> {
    Ok(collect_slots(source_file, data, script)?
        .into_iter()
        .map(|slot| slot.entry)
        .collect())
}

pub fn serialize_entries(entries: &[TranslationEntry]) -> ToolResult<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(entries)
        .map_err(|error| ToolError(format!("cannot serialize translation JSON: {error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn deserialize_entries(bytes: &[u8], label: &str) -> ToolResult<Vec<TranslationEntry>> {
    serde_json::from_slice(bytes)
        .map_err(|error| ToolError(format!("cannot parse UTF-8 JSON {label}: {error}")))
}

fn validate_metadata(actual: &TranslationEntry, expected: &TranslationEntry) -> ToolResult<()> {
    let mismatch = if actual.source_file != expected.source_file {
        Some("_file")
    } else if actual.index != expected.index {
        Some("_index")
    } else if actual.offset != expected.offset {
        Some("_offset")
    } else if actual.instruction_offset != expected.instruction_offset {
        Some("_inst_offset")
    } else if actual.size != expected.size {
        Some("_size")
    } else if actual.text_type != expected.text_type {
        Some("_type")
    } else if actual.opcode != expected.opcode {
        Some("_opcode")
    } else if actual.target != expected.target {
        Some("_target")
    } else if actual.message_id != expected.message_id {
        Some("_message_id")
    } else if actual.voice != expected.voice {
        Some("_voice")
    } else if actual.buffer_index != expected.buffer_index {
        Some("_buffer_index")
    } else if actual.encoding != expected.encoding {
        Some("_encoding")
    } else if actual.policy != expected.policy {
        Some("_policy")
    } else {
        None
    };
    if let Some(field) = mismatch {
        return Err(ToolError(format!(
            "{} entry {} metadata {field} does not match source",
            expected.source_file, expected.index
        )));
    }
    Ok(())
}

fn validate_entry(actual: &TranslationEntry, expected: &TranslationEntry) -> ToolResult<()> {
    validate_metadata(actual, expected)?;
    if actual.scr_msg != expected.scr_msg {
        return Err(ToolError(format!(
            "{} entry {} scr_msg was modified or does not match source",
            expected.source_file, expected.index
        )));
    }
    if actual.scr_name != expected.scr_name {
        return Err(ToolError(format!(
            "{} entry {} _scr_name was modified or does not match source",
            expected.source_file, expected.index
        )));
    }
    match (&actual.name, &expected.name) {
        (Some(name), Some(_)) => validate_name(
            name,
            &format!("{} entry {} name", expected.source_file, expected.index),
        )?,
        (None, None) => {}
        _ => {
            return Err(ToolError(format!(
                "{} entry {} name presence does not match source",
                expected.source_file, expected.index
            )));
        }
    }
    if actual.message.is_empty() {
        return Err(ToolError(format!(
            "{} entry {} message is empty",
            expected.source_file, expected.index
        )));
    }
    validate_controls(
        &expected.scr_msg,
        &format!("{} entry {} scr_msg", expected.source_file, expected.index),
    )?;
    validate_controls(
        &actual.message,
        &format!("{} entry {} message", expected.source_file, expected.index),
    )?;
    if expected.scr_msg.starts_with(FULLWIDTH_SPACE) && !actual.message.starts_with(FULLWIDTH_SPACE)
    {
        return Err(ToolError(format!(
            "{} entry {} must preserve its leading fullwidth space",
            expected.source_file, expected.index
        )));
    }
    Ok(())
}

pub fn inject_entries(
    source_file: &str,
    data: &[u8],
    entries: &[TranslationEntry],
) -> ToolResult<InjectedScript> {
    let script = parse_script(data)?;
    let slots = collect_slots(source_file, data, &script)?;
    if entries.len() != slots.len() {
        return Err(ToolError(format!(
            "{source_file} JSON has {} entries, source has {} extractable entries",
            entries.len(),
            slots.len()
        )));
    }

    let mut replacements = HashMap::new();
    let mut patched = 0usize;
    let mut unchanged = 0usize;
    for (actual, slot) in entries.iter().zip(&slots) {
        validate_entry(actual, &slot.entry)?;
        let message_bytes = encode_cp932(
            &actual.message,
            &format!("{source_file} entry {} message", actual.index),
        )?;
        let replacement = match actual.opcode {
            0 => {
                let mut output = Vec::new();
                if let Some(name) = &actual.name {
                    output.extend_from_slice(&encode_cp932(
                        name,
                        &format!("{source_file} entry {} name", actual.index),
                    )?);
                }
                output.extend_from_slice(MESSAGE_SEPARATOR.as_bytes());
                output.extend_from_slice(&message_bytes);
                output
            }
            1 => message_bytes,
            8 => {
                if message_bytes.len() > 16 {
                    return Err(ToolError(format!(
                        "{source_file} entry {} opcode 8 message buffer value is {} bytes; runtime limit is 16",
                        actual.index,
                        message_bytes.len()
                    )));
                }
                message_bytes
            }
            opcode => {
                return Err(ToolError(format!(
                    "{source_file} entry {} has unsupported text opcode {opcode}",
                    actual.index
                )));
            }
        };
        if actual.message != slot.entry.scr_msg || actual.name != slot.entry.name {
            patched += 1;
        } else {
            unchanged += 1;
        }
        replacements.insert(slot.string_offset, replacement);
    }

    let bytes = rebuild_script(data, &script, &replacements)?;
    let reparsed = parse_script(&bytes)?;
    let rebuilt_slots = collect_slots(source_file, &bytes, &reparsed)?;
    for (actual, rebuilt) in entries.iter().zip(&rebuilt_slots) {
        if actual.name != rebuilt.entry.name || actual.message != rebuilt.entry.scr_msg {
            return Err(ToolError(format!(
                "{source_file} entry {} did not survive rebuild verification",
                actual.index
            )));
        }
    }
    Ok(InjectedScript {
        bytes,
        json_entries: entries.len(),
        patched,
        unchanged,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_u16(output: &mut Vec<u8>, value: u16) {
        output.extend_from_slice(&value.to_le_bytes());
    }

    fn push_u32(output: &mut Vec<u8>, value: u32) {
        output.extend_from_slice(&value.to_le_bytes());
    }

    fn sample_script() -> Vec<u8> {
        let mut data = Vec::new();
        push_u16(&mut data, 8);
        push_u16(&mut data, 0);
        data.extend_from_slice(b"I\0");

        push_u16(&mut data, 2);
        let jump_field = data.len();
        push_u32(&mut data, 0);

        push_u16(&mut data, 0);
        push_u16(&mut data, 3);
        data.extend_from_slice(b"Name\\N\"Hello %M0\\W\"\0");

        push_u16(&mut data, 1);
        let choice_field = data.len();
        push_u32(&mut data, 0);
        data.extend_from_slice(&encode_cp932("\u{3000}Choice", "test choice").unwrap());
        data.push(0);

        let end = data.len();
        push_u16(&mut data, 13);
        data[jump_field..jump_field + 4].copy_from_slice(&(end as u32).to_le_bytes());
        data[choice_field..choice_field + 4].copy_from_slice(&(end as u32).to_le_bytes());
        data
    }

    #[test]
    fn extracts_confirmed_text_contract() {
        let data = sample_script();
        let script = parse_script(&data).unwrap();
        let entries = extract_entries("sample.AB", &data, &script).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].text_type, TextType::System);
        assert_eq!(entries[0].buffer_index, Some(0));
        assert_eq!(entries[1].name.as_deref(), Some("Name"));
        assert_eq!(entries[1].scr_name.as_deref(), Some("Name"));
        assert_eq!(entries[1].scr_msg, "\"Hello %M0\\W\"");
        assert_eq!(entries[2].text_type, TextType::Choice);
        assert!(entries[2].scr_msg.starts_with(FULLWIDTH_SPACE));
    }

    #[test]
    fn unchanged_injection_is_byte_exact() {
        let data = sample_script();
        let script = parse_script(&data).unwrap();
        let entries = extract_entries("sample.AB", &data, &script).unwrap();
        let injected = inject_entries("sample.AB", &data, &entries).unwrap();
        assert_eq!(injected.bytes, data);
        assert_eq!(injected.patched, 0);
        assert_eq!(injected.unchanged, 3);
    }

    #[test]
    fn injects_longer_name_and_message_and_relocates_targets() {
        let data = sample_script();
        let script = parse_script(&data).unwrap();
        let mut entries = extract_entries("sample.AB", &data, &script).unwrap();
        entries[1].name = Some("Longer Name".to_string());
        entries[1].message = "\"A much longer %M0 line\\W\"".to_string();
        let injected = inject_entries("sample.AB", &data, &entries).unwrap();
        let reparsed = parse_script(&injected.bytes).unwrap();
        assert_eq!(injected.patched, 1);
        assert_eq!(
            reparsed.instructions[1].targets[0].value as usize,
            reparsed.instructions.last().unwrap().offset
        );
        assert_eq!(
            reparsed.instructions[3].targets[0].value as usize,
            reparsed.instructions.last().unwrap().offset
        );
    }

    #[test]
    fn rejects_modified_source_validation_fields() {
        let data = sample_script();
        let script = parse_script(&data).unwrap();
        let mut entries = extract_entries("sample.AB", &data, &script).unwrap();
        entries[1].scr_msg.push('x');
        let error = inject_entries("sample.AB", &data, &entries)
            .unwrap_err()
            .to_string();
        assert!(error.contains("scr_msg was modified"));
    }

    #[test]
    fn rejects_modified_source_name_validation_field() {
        let data = sample_script();
        let script = parse_script(&data).unwrap();
        let mut entries = extract_entries("sample.AB", &data, &script).unwrap();
        entries[1].scr_name = Some("Other".to_string());
        let error = inject_entries("sample.AB", &data, &entries)
            .unwrap_err()
            .to_string();
        assert!(error.contains("_scr_name was modified"));
    }

    #[test]
    fn permits_control_removal() {
        let data = sample_script();
        let script = parse_script(&data).unwrap();
        let mut entries = extract_entries("sample.AB", &data, &script).unwrap();
        entries[1].message = "\"Hello\"".to_string();
        inject_entries("sample.AB", &data, &entries).expect("controls may be removed");
    }

    #[test]
    fn permits_added_control() {
        let data = sample_script();
        let script = parse_script(&data).unwrap();
        let mut entries = extract_entries("sample.AB", &data, &script).unwrap();
        entries[1].message = "\"\\W\\WHello %M0\\W\"".to_string();
        inject_entries("sample.AB", &data, &entries).expect("controls may be added");
    }

    #[test]
    fn permits_control_repositioning() {
        let data = sample_script();
        let script = parse_script(&data).unwrap();
        let mut entries = extract_entries("sample.AB", &data, &script).unwrap();
        entries[1].message = "\"\\WHello %M0\"".to_string();
        inject_entries("sample.AB", &data, &entries).expect("control may move");
    }

    #[test]
    fn rejects_removed_fullwidth_indent() {
        let data = sample_script();
        let script = parse_script(&data).unwrap();
        let mut entries = extract_entries("sample.AB", &data, &script).unwrap();
        entries[2].message = "Choice".to_string();
        let error = inject_entries("sample.AB", &data, &entries)
            .unwrap_err()
            .to_string();
        assert!(error.contains("leading fullwidth space"));
    }

    #[test]
    fn rejects_oversized_message_buffer() {
        let data = sample_script();
        let script = parse_script(&data).unwrap();
        let mut entries = extract_entries("sample.AB", &data, &script).unwrap();
        entries[0].message = "12345678901234567".to_string();
        let error = inject_entries("sample.AB", &data, &entries)
            .unwrap_err()
            .to_string();
        assert!(error.contains("runtime limit is 16"));
    }

    #[test]
    fn rejects_nul_and_real_newline() {
        let data = sample_script();
        let script = parse_script(&data).unwrap();
        let mut entries = extract_entries("sample.AB", &data, &script).unwrap();
        entries[1].message = "bad\nline".to_string();
        let error = inject_entries("sample.AB", &data, &entries)
            .unwrap_err()
            .to_string();
        assert!(error.contains("real CR/LF"));

        entries = extract_entries("sample.AB", &data, &script).unwrap();
        entries[1].message = "bad\0text".to_string();
        let error = inject_entries("sample.AB", &data, &entries)
            .unwrap_err()
            .to_string();
        assert!(error.contains("contains NUL"));
    }

    #[test]
    fn rejects_unencodable_translation() {
        let data = sample_script();
        let script = parse_script(&data).unwrap();
        let mut entries = extract_entries("sample.AB", &data, &script).unwrap();
        entries[0].message = "\u{1f600}".to_string();
        let error = inject_entries("sample.AB", &data, &entries)
            .unwrap_err()
            .to_string();
        assert!(error.contains("not encodable as CP932"));
    }

    #[test]
    fn repeated_source_text_is_located_by_index() {
        let mut data = Vec::new();
        for _ in 0..2 {
            push_u16(&mut data, 0);
            push_u16(&mut data, 1);
            data.extend_from_slice(b"\x5cNsame ");
        }
        push_u16(&mut data, 13);
        let script = parse_script(&data).unwrap();
        let mut entries = extract_entries("repeat.AB", &data, &script).unwrap();
        entries[1].message = "changed".to_string();
        let injected = inject_entries("repeat.AB", &data, &entries).unwrap();
        let reparsed = parse_script(&injected.bytes).unwrap();
        assert_eq!(reparsed.instructions[0].strings[0].text, "\\Nsame");
        assert_eq!(reparsed.instructions[1].strings[0].text, "\\Nchanged");
    }
}
