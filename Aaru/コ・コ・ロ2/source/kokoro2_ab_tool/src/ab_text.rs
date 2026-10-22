use crate::ab::{
    decode_cp932, encode_cp932, parse_file, rebuild_script, AbFile, AbInstruction, AbPlainText,
    AbScript, AbString, AbStringKind,
};
use crate::{ToolError, ToolResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const ENCODING: &str = "CP932";
const RELOCATE_POLICY: &str = "relocate";
const WHOLE_FILE_POLICY: &str = "whole_file";
const MESSAGE_SEPARATOR: &[u8] = b"\\N";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextType {
    Dialogue,
    Monologue,
    Choice,
    ChoicePrompt,
    System,
    Plaintext,
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
    #[serde(rename = "_inst_offset", skip_serializing_if = "Option::is_none")]
    pub instruction_offset: Option<usize>,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub text_type: TextType,
    #[serde(rename = "_opcode", skip_serializing_if = "Option::is_none")]
    pub opcode: Option<u16>,
    #[serde(rename = "_target", skip_serializing_if = "Option::is_none")]
    pub target: Option<u32>,
    #[serde(rename = "_message_id", skip_serializing_if = "Option::is_none")]
    pub message_id: Option<u16>,
    #[serde(rename = "_voice", skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(rename = "_buffer_index", skip_serializing_if = "Option::is_none")]
    pub buffer_index: Option<u16>,
    #[serde(rename = "_name_split", skip_serializing_if = "Option::is_none")]
    pub name_split: Option<bool>,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_policy")]
    pub policy: String,
}

#[derive(Debug)]
struct TextSlot {
    string_offset: usize,
    has_name_separator: bool,
    entry: TranslationEntry,
}

#[derive(Debug)]
pub struct InjectedFile {
    pub bytes: Vec<u8>,
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
}

fn separator_offsets(raw: &[u8]) -> Vec<usize> {
    raw.windows(MESSAGE_SEPARATOR.len())
        .enumerate()
        .filter_map(|(offset, value)| (value == MESSAGE_SEPARATOR).then_some(offset))
        .collect()
}

fn validate_inline_text(text: &str, label: &str) -> ToolResult<()> {
    if text.contains('\0') {
        return Err(ToolError(format!("{label} contains NUL")));
    }
    if text.contains(['\r', '\n']) {
        return Err(ToolError(format!(
            "{label} contains a real CR/LF; Kokoro 2 bytecode text uses explicit controls"
        )));
    }

    let bytes = text.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor] == b'\\' {
            let control = *bytes.get(cursor + 1).ok_or_else(|| {
                ToolError(format!("{label} ends with an incomplete backslash control"))
            })?;
            if !matches!(control, b'N' | b'W' | b'K') {
                return Err(ToolError(format!(
                    "{label} contains unsupported control \\{} at UTF-8 byte {cursor}",
                    char::from(control)
                )));
            }
            cursor += 2;
        } else {
            let character = text[cursor..].chars().next().ok_or_else(|| {
                ToolError(format!("{label} has an invalid UTF-8 boundary at {cursor}"))
            })?;
            cursor += character.len_utf8();
        }
    }
    Ok(())
}

fn validate_plain_text(text: &str, label: &str) -> ToolResult<()> {
    if text.contains('\0') {
        return Err(ToolError(format!("{label} contains NUL")));
    }
    Ok(())
}

fn validate_name(name: &str, label: &str) -> ToolResult<()> {
    if name.is_empty() {
        return Err(ToolError(format!("{label} is empty")));
    }
    if name.contains(['\0', '\r', '\n', '\\']) {
        return Err(ToolError(format!(
            "{label} contains a forbidden NUL, CR/LF, or backslash"
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
    let separators = separator_offsets(&string.raw);
    let (name, message, body_start, has_name_separator) = if separators.len() == 1 {
        let separator = separators[0];
        let name_raw = &string.raw[..separator];
        let body_start = separator + MESSAGE_SEPARATOR.len();
        let name = (!name_raw.is_empty())
            .then(|| decode_cp932(name_raw, "opcode 0 speaker name"))
            .transpose()?;
        let message = decode_cp932(&string.raw[body_start..], "opcode 0 message body")?;
        (name, message, body_start, true)
    } else {
        (None, string.text.clone(), 0, false)
    };
    if let Some(name) = &name {
        validate_name(name, "source name")?;
    }
    validate_inline_text(&message, "source message")?;

    Ok(TextSlot {
        string_offset: string.offset,
        has_name_separator,
        entry: TranslationEntry {
            name: name.clone(),
            scr_name: name,
            scr_msg: message.clone(),
            message,
            source_file: source_file.to_string(),
            index,
            offset: string.offset + body_start,
            instruction_offset: Some(instruction.offset),
            size: string.raw.len() - body_start,
            text_type: if has_name_separator {
                TextType::Dialogue
            } else {
                TextType::Monologue
            },
            opcode: Some(instruction.opcode),
            target: None,
            message_id: instruction.message_id,
            voice: instruction.voice.as_ref().map(|voice| voice.text.clone()),
            buffer_index: None,
            name_split: Some(has_name_separator),
            encoding: ENCODING.to_string(),
            policy: RELOCATE_POLICY.to_string(),
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
    validate_inline_text(&string.text, "source text")?;
    let (text_type, target, buffer_index) = match string.kind {
        AbStringKind::Choice => (
            TextType::Choice,
            instruction.targets.first().map(|target| target.value),
            None,
        ),
        AbStringKind::ChoicePrompt => (TextType::ChoicePrompt, None, None),
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
                "opcode {} at 0x{:x} is not an extractable text slot",
                instruction.opcode, instruction.offset
            )));
        }
    };

    Ok(TextSlot {
        string_offset: string.offset,
        has_name_separator: false,
        entry: TranslationEntry {
            name: None,
            scr_name: None,
            scr_msg: string.text.clone(),
            message: string.text.clone(),
            source_file: source_file.to_string(),
            index,
            offset: string.offset,
            instruction_offset: Some(instruction.offset),
            size: string.raw.len(),
            text_type,
            opcode: Some(instruction.opcode),
            target,
            message_id: None,
            voice: None,
            buffer_index,
            name_split: None,
            encoding: ENCODING.to_string(),
            policy: RELOCATE_POLICY.to_string(),
        },
    })
}

fn collect_bytecode_slots(
    source_file: &str,
    data: &[u8],
    script: &AbScript,
) -> ToolResult<Vec<TextSlot>> {
    let mut slots = Vec::new();
    for instruction in &script.instructions {
        for string in &instruction.strings {
            let index = slots.len();
            let slot = match string.kind {
                AbStringKind::Message => {
                    make_message_slot(source_file, index, instruction, string)?
                }
                AbStringKind::Choice | AbStringKind::ChoicePrompt | AbStringKind::Buffer => {
                    make_plain_slot(source_file, index, data, instruction, string)?
                }
                AbStringKind::Resource => continue,
            };
            slots.push(slot);
        }
    }
    Ok(slots)
}

fn make_plaintext_entry(
    source_file: &str,
    data: &[u8],
    plain: &AbPlainText,
) -> ToolResult<TranslationEntry> {
    validate_plain_text(&plain.text, "source plain text")?;
    Ok(TranslationEntry {
        name: None,
        scr_name: None,
        scr_msg: plain.text.clone(),
        message: plain.text.clone(),
        source_file: source_file.to_string(),
        index: 0,
        offset: 0,
        instruction_offset: None,
        size: data.len(),
        text_type: TextType::Plaintext,
        opcode: None,
        target: None,
        message_id: None,
        voice: None,
        buffer_index: None,
        name_split: None,
        encoding: ENCODING.to_string(),
        policy: WHOLE_FILE_POLICY.to_string(),
    })
}

pub fn extract_entries(source_file: &str, data: &[u8]) -> ToolResult<Vec<TranslationEntry>> {
    match parse_file(data)? {
        AbFile::Bytecode(script) => Ok(collect_bytecode_slots(source_file, data, &script)?
            .into_iter()
            .map(|slot| slot.entry)
            .collect()),
        AbFile::PlainCp932(plain) => Ok(vec![make_plaintext_entry(source_file, data, &plain)?]),
    }
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
    } else if actual.name_split != expected.name_split {
        Some("_name_split")
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

    match expected.text_type {
        TextType::Plaintext => {
            validate_plain_text(
                &expected.scr_msg,
                &format!("{} entry {} scr_msg", expected.source_file, expected.index),
            )?;
            validate_plain_text(
                &actual.message,
                &format!("{} entry {} message", expected.source_file, expected.index),
            )?;
        }
        _ => {
            validate_inline_text(
                &expected.scr_msg,
                &format!("{} entry {} scr_msg", expected.source_file, expected.index),
            )?;
            validate_inline_text(
                &actual.message,
                &format!("{} entry {} message", expected.source_file, expected.index),
            )?;
        }
    }
    Ok(())
}

fn extractable_strings(script: &AbScript) -> Vec<&AbString> {
    script
        .instructions
        .iter()
        .flat_map(|instruction| instruction.strings.iter())
        .filter(|string| {
            matches!(
                string.kind,
                AbStringKind::Message
                    | AbStringKind::Choice
                    | AbStringKind::ChoicePrompt
                    | AbStringKind::Buffer
            )
        })
        .collect()
}

fn inject_bytecode_entries(
    source_file: &str,
    data: &[u8],
    script: &AbScript,
    entries: &[TranslationEntry],
) -> ToolResult<InjectedFile> {
    let slots = collect_bytecode_slots(source_file, data, script)?;
    if entries.len() != slots.len() {
        return Err(ToolError(format!(
            "{source_file} JSON has {} entries, source has {} extractable entries",
            entries.len(),
            slots.len()
        )));
    }

    let mut replacements = HashMap::new();
    let mut expected_bytes = Vec::with_capacity(slots.len());
    let mut patched = 0usize;
    let mut unchanged = 0usize;
    for (actual, slot) in entries.iter().zip(&slots) {
        validate_entry(actual, &slot.entry)?;
        let message_bytes = encode_cp932(
            &actual.message,
            &format!("{source_file} entry {} message", actual.index),
        )?;
        let replacement = if slot.has_name_separator {
            let mut output = Vec::new();
            if let Some(name) = &actual.name {
                output.extend_from_slice(&encode_cp932(
                    name,
                    &format!("{source_file} entry {} name", actual.index),
                )?);
            }
            output.extend_from_slice(MESSAGE_SEPARATOR);
            output.extend_from_slice(&message_bytes);
            output
        } else {
            message_bytes
        };
        if slot.entry.opcode == Some(8) && replacement.len() > 16 {
            return Err(ToolError(format!(
                "{source_file} entry {} opcode 8 message buffer value is {} bytes; runtime limit is 16",
                actual.index,
                replacement.len()
            )));
        }
        if actual.message != slot.entry.scr_msg || actual.name != slot.entry.name {
            patched += 1;
        } else {
            unchanged += 1;
        }
        replacements.insert(slot.string_offset, replacement.clone());
        expected_bytes.push(replacement);
    }

    let bytes = rebuild_script(data, script, &replacements)?;
    let rebuilt_script = match parse_file(&bytes)? {
        AbFile::Bytecode(script) => script,
        AbFile::PlainCp932(_) => {
            return Err(ToolError(format!(
                "{source_file} rebuilt bytecode was classified as plain text"
            )));
        }
    };
    let rebuilt_strings = extractable_strings(&rebuilt_script);
    if rebuilt_strings.len() != expected_bytes.len() {
        return Err(ToolError(format!(
            "{source_file} rebuilt script changed its extractable string count"
        )));
    }
    for (index, (string, expected)) in rebuilt_strings.iter().zip(&expected_bytes).enumerate() {
        if string.raw != *expected {
            return Err(ToolError(format!(
                "{source_file} entry {index} did not survive rebuild verification"
            )));
        }
    }

    Ok(InjectedFile {
        bytes,
        json_entries: entries.len(),
        patched,
        unchanged,
    })
}

fn inject_plaintext_entries(
    source_file: &str,
    data: &[u8],
    plain: &AbPlainText,
    entries: &[TranslationEntry],
) -> ToolResult<InjectedFile> {
    let expected = make_plaintext_entry(source_file, data, plain)?;
    if entries.len() != 1 {
        return Err(ToolError(format!(
            "{source_file} JSON has {} entries, plain text source has 1 entry",
            entries.len()
        )));
    }
    let actual = &entries[0];
    validate_entry(actual, &expected)?;
    let bytes = encode_cp932(
        &actual.message,
        &format!("{source_file} entry {} message", actual.index),
    )?;
    let rebuilt = decode_cp932(&bytes, "rebuilt plain .AB text")?;
    if rebuilt != actual.message {
        return Err(ToolError(format!(
            "{source_file} plain text did not survive rebuild verification"
        )));
    }
    let patched = usize::from(actual.message != expected.scr_msg);
    Ok(InjectedFile {
        bytes,
        json_entries: 1,
        patched,
        unchanged: 1 - patched,
    })
}

pub fn inject_entries(
    source_file: &str,
    data: &[u8],
    entries: &[TranslationEntry],
) -> ToolResult<InjectedFile> {
    match parse_file(data)? {
        AbFile::Bytecode(script) => inject_bytecode_entries(source_file, data, &script, entries),
        AbFile::PlainCp932(plain) => inject_plaintext_entries(source_file, data, &plain, entries),
    }
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
        push_u16(&mut data, 6);
        data.extend_from_slice(b"Question?");
        data.push(0);

        push_u16(&mut data, 1);
        let choice_target = data.len();
        push_u32(&mut data, 0);
        data.extend_from_slice(b"Choice");
        data.push(0);

        push_u16(&mut data, 0);
        push_u16(&mut data, 4);
        data.extend_from_slice(b"Name\\NLine\\W");
        data.push(0);

        push_u16(&mut data, 0);
        push_u16(&mut data, 5);
        data.extend_from_slice(b"Ending\\Npart one\\Npart two");
        data.push(0);

        let end = data.len();
        push_u16(&mut data, 13);
        data[choice_target..choice_target + 4].copy_from_slice(&(end as u32).to_le_bytes());
        data
    }

    #[test]
    fn extracts_dialogue_choices_prompts_and_exceptional_controls() {
        let data = sample_script();
        let entries = extract_entries("sample.AB", &data).unwrap();
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].text_type, TextType::ChoicePrompt);
        assert_eq!(entries[1].text_type, TextType::Choice);
        assert_eq!(entries[2].name.as_deref(), Some("Name"));
        assert_eq!(entries[2].scr_name.as_deref(), Some("Name"));
        assert_eq!(entries[2].scr_msg, "Line\\W");
        assert_eq!(entries[2].name_split, Some(true));
        assert_eq!(entries[3].name, None);
        assert_eq!(entries[3].scr_msg, "Ending\\Npart one\\Npart two");
        assert_eq!(entries[3].name_split, Some(false));
    }

    #[test]
    fn unchanged_injection_is_byte_exact() {
        let data = sample_script();
        let entries = extract_entries("sample.AB", &data).unwrap();
        let injected = inject_entries("sample.AB", &data, &entries).unwrap();
        assert_eq!(injected.bytes, data);
        assert_eq!(injected.patched, 0);
        assert_eq!(injected.unchanged, 4);
    }

    #[test]
    fn injects_longer_name_body_and_prompt_and_relocates_targets() {
        let data = sample_script();
        let mut entries = extract_entries("sample.AB", &data).unwrap();
        entries[0].message = "A much longer choice prompt".to_string();
        entries[2].name = Some("Longer Name".to_string());
        entries[2].message = "Body with an added \\N control\\K".to_string();
        let injected = inject_entries("sample.AB", &data, &entries).unwrap();
        let reparsed = match parse_file(&injected.bytes).unwrap() {
            AbFile::Bytecode(script) => script,
            AbFile::PlainCp932(_) => panic!("expected bytecode"),
        };
        assert_eq!(injected.patched, 2);
        assert_eq!(
            reparsed.instructions[1].targets[0].value as usize,
            reparsed.instructions.last().unwrap().offset
        );
    }

    #[test]
    fn rejects_modified_source_validation_fields() {
        let data = sample_script();
        let mut entries = extract_entries("sample.AB", &data).unwrap();
        entries[2].scr_msg.push('x');
        let error = inject_entries("sample.AB", &data, &entries)
            .unwrap_err()
            .to_string();
        assert!(error.contains("scr_msg was modified"));
    }

    #[test]
    fn rejects_modified_source_name_validation_field() {
        let data = sample_script();
        let mut entries = extract_entries("sample.AB", &data).unwrap();
        entries[2].scr_name = Some("Other".to_string());
        let error = inject_entries("sample.AB", &data, &entries)
            .unwrap_err()
            .to_string();
        assert!(error.contains("_scr_name was modified"));
    }

    #[test]
    fn permits_known_control_changes_and_rejects_invalid_controls() {
        let data = sample_script();
        let mut entries = extract_entries("sample.AB", &data).unwrap();
        entries[2].message = "Reordered\\K controls\\N".to_string();
        inject_entries("sample.AB", &data, &entries).unwrap();

        entries[2].message = "Bad\\Xcontrol".to_string();
        let error = inject_entries("sample.AB", &data, &entries)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unsupported control"));
    }

    #[test]
    fn extracts_and_injects_plaintext_ab() {
        let data = encode_cp932("Thank you.\r\nLine two.\r\n", "test").unwrap();
        let mut entries = extract_entries("THANKS.AB", &data).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].text_type, TextType::Plaintext);
        assert_eq!(entries[0].policy, WHOLE_FILE_POLICY);
        entries[0].message = "Updated.\r\n".to_string();
        let injected = inject_entries("THANKS.AB", &data, &entries).unwrap();
        assert_eq!(
            decode_cp932(&injected.bytes, "test output").unwrap(),
            "Updated.\r\n"
        );
    }

    #[test]
    fn preserves_empty_name_separator_and_message_buffers() {
        let mut data = Vec::new();
        push_u16(&mut data, 0);
        push_u16(&mut data, 1);
        data.extend_from_slice(b"\\NBody");
        data.push(0);
        push_u16(&mut data, 8);
        push_u16(&mut data, 3);
        data.extend_from_slice(b"Short");
        data.push(0);
        push_u16(&mut data, 13);

        let mut entries = extract_entries("sample.AB", &data).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, None);
        assert_eq!(entries[0].name_split, Some(true));
        assert_eq!(entries[0].scr_msg, "Body");
        assert_eq!(entries[1].buffer_index, Some(3));

        entries[0].message = "New\\Nbody".to_string();
        entries[1].message = "Changed".to_string();
        let injected = inject_entries("sample.AB", &data, &entries).unwrap();
        let script = match parse_file(&injected.bytes).unwrap() {
            AbFile::Bytecode(script) => script,
            AbFile::PlainCp932(_) => panic!("expected bytecode"),
        };
        assert_eq!(script.instructions[0].strings[0].raw, b"\\NNew\\Nbody");
        assert_eq!(script.instructions[1].strings[0].text, "Changed");
    }

    #[test]
    fn rejects_oversized_message_buffer() {
        let mut data = Vec::new();
        push_u16(&mut data, 8);
        push_u16(&mut data, 0);
        data.extend_from_slice(b"Short");
        data.push(0);
        push_u16(&mut data, 13);

        let mut entries = extract_entries("sample.AB", &data).unwrap();
        entries[0].message = "12345678901234567".to_string();
        let error = inject_entries("sample.AB", &data, &entries)
            .unwrap_err()
            .to_string();
        assert!(error.contains("runtime limit is 16"));
    }
}
