use std::collections::BTreeMap;

use encoding_rs::SHIFT_JIS;

use crate::error::{Error, Result};

pub const SIGNATURE: &[u8; 8] = b"SCR:2005";
const HEADER_SIZE: usize = 0x10;
const SECTION_LENGTH_SIZE: usize = 4;
const XOR_KEY: u8 = 0x7f;
const IGNORE_PARAMETER: u32 = 0xffff_ffff;

const MESSAGE_POLICY: &[i8] = &[1, -1, 1];
const CHOICE_POLICY: &[i8] = &[0, 1];
const POINTER_POLICY: &[i8] = &[-1];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    Message,
    Choice,
}

impl RecordKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Choice => "choice",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawRecord {
    pub kind: RecordKind,
    pub opcode: u8,
    pub instruction_offset: u64,
    pub string_offset: u32,
    pub absolute_string_offset: u64,
    pub encoded_size: usize,
    pub terminator: Option<&'static str>,
    pub name_string_offset: Option<u32>,
    pub name: Option<String>,
    pub message: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ParseStats {
    pub commands: usize,
    pub strings: usize,
    pub messages: usize,
    pub named_messages: usize,
    pub choices: usize,
}

#[derive(Debug)]
pub struct ParsedScript {
    pub records: Vec<RawRecord>,
    pub stats: ParseStats,
    pub warnings: Vec<String>,
}

pub fn rebuild_version5(
    bytes: &[u8],
    label: &str,
    replacements: &BTreeMap<u32, Vec<u8>>,
) -> Result<Vec<u8>> {
    if replacements.is_empty() {
        return Ok(bytes.to_vec());
    }
    let layout = read_layout(bytes, label)?;
    let mut decrypted = bytes[layout.string_start..layout.string_end].to_vec();
    for byte in &mut decrypted {
        *byte ^= XOR_KEY;
    }
    if decrypted.last() != Some(&0) {
        return Err(Error::new(format!(
            "{label}: decrypted string section has no trailing NUL"
        )));
    }

    let raw_strings = split_raw_strings(&decrypted, label)?;
    let mut offset_to_index = BTreeMap::new();
    for (index, slot) in raw_strings.iter().enumerate() {
        offset_to_index.insert(slot.offset, index);
    }
    for offset in replacements.keys() {
        if !offset_to_index.contains_key(offset) {
            return Err(Error::new(format!(
                "{label}: replacement offset 0x{offset:x} is not a string boundary"
            )));
        }
    }

    let mut rebuilt_strings = Vec::new();
    let mut new_offsets = Vec::with_capacity(raw_strings.len());
    for slot in &raw_strings {
        let new_offset = u32::try_from(rebuilt_strings.len())
            .map_err(|_| Error::new(format!("{label}: rebuilt string table exceeds u32")))?;
        new_offsets.push(new_offset);
        let value = replacements
            .get(&slot.offset)
            .map(Vec::as_slice)
            .unwrap_or(slot.bytes);
        if value.contains(&0) {
            return Err(Error::new(format!(
                "{label}: replacement at 0x{:x} contains NUL",
                slot.offset
            )));
        }
        rebuilt_strings.extend_from_slice(value);
        rebuilt_strings.push(0);
    }

    let mut rebuilt_commands = bytes[layout.command_start..layout.command_end].to_vec();
    let mut position = 0usize;
    while position < rebuilt_commands.len() {
        if position + 2 > rebuilt_commands.len() {
            return Err(Error::new(format!(
                "{label}: truncated command at 0x{:x}",
                layout.command_start + position
            )));
        }
        let opcode = rebuilt_commands[position];
        let length = rebuilt_commands[position + 1] as usize;
        if length < 4 || !length.is_multiple_of(4) || position + length > rebuilt_commands.len() {
            return Err(Error::new(format!(
                "{label}: invalid command length {length} at 0x{:x}",
                layout.command_start + position
            )));
        }
        if let Some(policy) = string_policy(opcode) {
            let parameter_count = length / 4 - 1;
            if parameter_count > policy.len() {
                return Err(Error::new(format!(
                    "{label}: opcode 0x{opcode:02x} has {parameter_count} parameters, version 5 defines {}",
                    policy.len()
                )));
            }
            for (index, parameter_policy) in
                policy.iter().copied().enumerate().take(parameter_count)
            {
                if parameter_policy == 0 {
                    continue;
                }
                let parameter_offset = position + (index + 1) * 4;
                let old_offset = read_u32(&rebuilt_commands, parameter_offset, label)?;
                if old_offset == IGNORE_PARAMETER {
                    continue;
                }
                let string_index = offset_to_index.get(&old_offset).copied().ok_or_else(|| {
                    Error::new(format!(
                        "{label}: string offset 0x{old_offset:x} at instruction 0x{:x} is not on a string boundary",
                        layout.command_start + position
                    ))
                })?;
                rebuilt_commands[parameter_offset..parameter_offset + 4]
                    .copy_from_slice(&new_offsets[string_index].to_le_bytes());
            }
        }
        position += length;
    }

    for byte in &mut rebuilt_strings {
        *byte ^= XOR_KEY;
    }
    let command_length = u32::try_from(rebuilt_commands.len())
        .map_err(|_| Error::new(format!("{label}: rebuilt command section exceeds u32")))?;
    let string_length = u32::try_from(rebuilt_strings.len())
        .map_err(|_| Error::new(format!("{label}: rebuilt string section exceeds u32")))?;
    let mut rebuilt =
        Vec::with_capacity(HEADER_SIZE + 8 + rebuilt_commands.len() + rebuilt_strings.len());
    rebuilt.extend_from_slice(&bytes[..HEADER_SIZE]);
    rebuilt.extend_from_slice(&command_length.to_le_bytes());
    rebuilt.extend_from_slice(&rebuilt_commands);
    rebuilt.extend_from_slice(&string_length.to_le_bytes());
    rebuilt.extend_from_slice(&rebuilt_strings);
    Ok(rebuilt)
}

#[derive(Debug)]
struct Layout {
    command_start: usize,
    command_end: usize,
    string_start: usize,
    string_end: usize,
}

#[derive(Debug)]
struct RawString<'a> {
    offset: u32,
    bytes: &'a [u8],
}

fn read_layout(bytes: &[u8], label: &str) -> Result<Layout> {
    if bytes.len() < HEADER_SIZE + SECTION_LENGTH_SIZE * 2 {
        return Err(Error::new(format!("{label}: file is too short")));
    }
    if !has_version5_signature(bytes) {
        return Err(Error::new(format!("{label}: expected SCR:2005 signature")));
    }
    let command_length = read_u32(bytes, HEADER_SIZE, label)? as usize;
    let command_start = HEADER_SIZE + SECTION_LENGTH_SIZE;
    let command_end = command_start
        .checked_add(command_length)
        .ok_or_else(|| Error::new(format!("{label}: command section length overflow")))?;
    let string_length = read_u32(bytes, command_end, label)? as usize;
    let string_start = command_end + SECTION_LENGTH_SIZE;
    let string_end = string_start
        .checked_add(string_length)
        .ok_or_else(|| Error::new(format!("{label}: string section length overflow")))?;
    if string_end != bytes.len() {
        return Err(Error::new(format!(
            "{label}: declared sections end at 0x{string_end:x}, file ends at 0x{:x}",
            bytes.len()
        )));
    }
    Ok(Layout {
        command_start,
        command_end,
        string_start,
        string_end,
    })
}

fn split_raw_strings<'a>(decrypted: &'a [u8], label: &str) -> Result<Vec<RawString<'a>>> {
    let mut strings = Vec::new();
    let mut start = 0usize;
    for end in 0..decrypted.len() {
        if decrypted[end] != 0 {
            continue;
        }
        strings.push(RawString {
            offset: u32::try_from(start)
                .map_err(|_| Error::new(format!("{label}: string offset exceeds u32")))?,
            bytes: &decrypted[start..end],
        });
        start = end + 1;
    }
    if start != decrypted.len() {
        return Err(Error::new(format!(
            "{label}: string parser did not consume the full section"
        )));
    }
    Ok(strings)
}

#[derive(Debug)]
struct StringSlot {
    relative_offset: u32,
    absolute_offset: u64,
    encoded_size: usize,
    text: String,
}

pub fn has_version5_signature(bytes: &[u8]) -> bool {
    bytes.starts_with(SIGNATURE)
}

pub fn parse_version5(bytes: &[u8], label: &str) -> Result<ParsedScript> {
    let layout = read_layout(bytes, label)?;
    let command_start = layout.command_start;
    let command_end = layout.command_end;
    let string_start = layout.string_start;
    let string_end = layout.string_end;

    let mut decrypted = bytes[string_start..string_end].to_vec();
    for byte in &mut decrypted {
        *byte ^= XOR_KEY;
    }
    if decrypted.last() != Some(&0) {
        return Err(Error::new(format!(
            "{label}: decrypted string section has no trailing NUL"
        )));
    }

    let (strings, string_by_offset) = parse_strings(&decrypted, string_start, label)?;
    let command_bytes = &bytes[command_start..command_end];
    let mut records = Vec::new();
    let mut warnings = Vec::new();
    let mut referenced = vec![false; strings.len()];
    let mut command_position = 0usize;
    let mut command_count = 0usize;

    while command_position < command_bytes.len() {
        if command_position + 2 > command_bytes.len() {
            return Err(Error::new(format!(
                "{label}: truncated command header at 0x{:x}",
                command_start + command_position
            )));
        }
        let opcode = command_bytes[command_position];
        let length = command_bytes[command_position + 1] as usize;
        if length < 4
            || !length.is_multiple_of(4)
            || command_position + length > command_bytes.len()
        {
            return Err(Error::new(format!(
                "{label}: invalid command length {length} at 0x{:x}",
                command_start + command_position
            )));
        }

        let parameter_count = length / 4 - 1;
        let mut parameters = Vec::with_capacity(parameter_count);
        for index in 0..parameter_count {
            parameters.push(read_u32(
                command_bytes,
                command_position + (index + 1) * 4,
                label,
            )?);
        }

        if let Some(policy) = string_policy(opcode) {
            if parameter_count > policy.len() {
                return Err(Error::new(format!(
                    "{label}: opcode 0x{opcode:02x} at 0x{:x} has {parameter_count} parameters, version 5 defines {}",
                    command_start + command_position,
                    policy.len()
                )));
            }
            for (index, parameter) in parameters.iter().copied().enumerate() {
                if policy[index] == 0 || parameter == IGNORE_PARAMETER {
                    continue;
                }
                let slot_index = resolve_string(
                    &string_by_offset,
                    parameter,
                    label,
                    command_start + command_position,
                )?;
                referenced[slot_index] = true;
            }
        }

        let instruction_offset = (command_start + command_position) as u64;
        match opcode {
            0x5e => {
                if parameters.len() != 3 {
                    return Err(Error::new(format!(
                        "{label}: message command at 0x{instruction_offset:x} does not have three parameters"
                    )));
                }
                let message_index = resolve_string(
                    &string_by_offset,
                    parameters[2],
                    label,
                    command_start + command_position,
                )?;
                let message_slot = &strings[message_index];
                let name = if parameters[0] == IGNORE_PARAMETER {
                    None
                } else {
                    let name_index = resolve_string(
                        &string_by_offset,
                        parameters[0],
                        label,
                        command_start + command_position,
                    )?;
                    Some(strings[name_index].text.clone())
                };
                let (message, terminator) = strip_message_terminator(&message_slot.text);
                if terminator.is_none() {
                    warnings.push(format!(
                        "message at instruction 0x{instruction_offset:x} has no trailing LF"
                    ));
                }
                records.push(RawRecord {
                    kind: RecordKind::Message,
                    opcode,
                    instruction_offset,
                    string_offset: message_slot.relative_offset,
                    absolute_string_offset: message_slot.absolute_offset,
                    encoded_size: message_slot.encoded_size,
                    terminator,
                    name_string_offset: (parameters[0] != IGNORE_PARAMETER)
                        .then_some(parameters[0]),
                    name,
                    message,
                });
            }
            0x64 => {
                if parameters.len() != 2 {
                    return Err(Error::new(format!(
                        "{label}: choice command at 0x{instruction_offset:x} does not have two parameters"
                    )));
                }
                let message_index = resolve_string(
                    &string_by_offset,
                    parameters[1],
                    label,
                    command_start + command_position,
                )?;
                let message_slot = &strings[message_index];
                records.push(RawRecord {
                    kind: RecordKind::Choice,
                    opcode,
                    instruction_offset,
                    string_offset: message_slot.relative_offset,
                    absolute_string_offset: message_slot.absolute_offset,
                    encoded_size: message_slot.encoded_size,
                    terminator: None,
                    name_string_offset: None,
                    name: None,
                    message: message_slot.text.clone(),
                });
            }
            _ => {}
        }

        command_count += 1;
        command_position += length;
    }

    if command_position != command_bytes.len() {
        return Err(Error::new(format!(
            "{label}: command parser did not end on the section boundary"
        )));
    }

    for (index, was_referenced) in referenced.iter().copied().enumerate() {
        if !was_referenced {
            warnings.push(format!(
                "string {} at relative offset 0x{:x} is not referenced by the version 5 table",
                index, strings[index].relative_offset
            ));
        }
    }

    let messages = records
        .iter()
        .filter(|record| record.kind == RecordKind::Message)
        .count();
    let named_messages = records
        .iter()
        .filter(|record| record.kind == RecordKind::Message && record.name.is_some())
        .count();
    let choices = records
        .iter()
        .filter(|record| record.kind == RecordKind::Choice)
        .count();

    Ok(ParsedScript {
        records,
        stats: ParseStats {
            commands: command_count,
            strings: strings.len(),
            messages,
            named_messages,
            choices,
        },
        warnings,
    })
}

fn parse_strings(
    decrypted: &[u8],
    absolute_start: usize,
    label: &str,
) -> Result<(Vec<StringSlot>, BTreeMap<u32, usize>)> {
    let mut strings = Vec::new();
    let mut by_offset = BTreeMap::new();
    let mut start = 0usize;

    for end in 0..decrypted.len() {
        if decrypted[end] != 0 {
            continue;
        }
        let source = &decrypted[start..end];
        let text = decode_cp932_roundtrip(source, label, start)?;
        let relative_offset = u32::try_from(start)
            .map_err(|_| Error::new(format!("{label}: string offset exceeds u32")))?;
        let slot = StringSlot {
            relative_offset,
            absolute_offset: (absolute_start + start) as u64,
            encoded_size: source.len(),
            text,
        };
        let slot_index = strings.len();
        if by_offset.insert(relative_offset, slot_index).is_some() {
            return Err(Error::new(format!(
                "{label}: duplicate string offset 0x{relative_offset:x}"
            )));
        }
        strings.push(slot);
        start = end + 1;
    }

    if start != decrypted.len() {
        return Err(Error::new(format!(
            "{label}: string parser did not consume the full section"
        )));
    }
    Ok((strings, by_offset))
}

fn decode_cp932_roundtrip(source: &[u8], label: &str, relative_offset: usize) -> Result<String> {
    let (decoded, had_decode_errors) = SHIFT_JIS.decode_without_bom_handling(source);
    if had_decode_errors {
        return Err(Error::new(format!(
            "{label}: invalid CP932 at string offset 0x{relative_offset:x}"
        )));
    }
    let (encoded, _, had_encode_errors) = SHIFT_JIS.encode(&decoded);
    if had_encode_errors || encoded.as_ref() != source {
        return Err(Error::new(format!(
            "{label}: CP932 string at offset 0x{relative_offset:x} is not byte-exact after re-encoding"
        )));
    }
    Ok(decoded.into_owned())
}

fn strip_message_terminator(text: &str) -> (String, Option<&'static str>) {
    if let Some(without_lf) = text.strip_suffix('\n') {
        if let Some(without_crlf) = without_lf.strip_suffix('\r') {
            return (without_crlf.to_owned(), Some("CRLF"));
        }
        return (without_lf.to_owned(), Some("LF"));
    }
    (text.to_owned(), None)
}

fn resolve_string(
    string_by_offset: &BTreeMap<u32, usize>,
    offset: u32,
    label: &str,
    instruction_offset: usize,
) -> Result<usize> {
    if offset == IGNORE_PARAMETER {
        return Err(Error::new(format!(
            "{label}: required string is ignored at instruction 0x{instruction_offset:x}"
        )));
    }
    string_by_offset.get(&offset).copied().ok_or_else(|| {
        Error::new(format!(
            "{label}: string offset 0x{offset:x} at instruction 0x{instruction_offset:x} is not on a string boundary"
        ))
    })
}

fn string_policy(opcode: u8) -> Option<&'static [i8]> {
    match opcode {
        0x5e => Some(MESSAGE_POLICY),
        0x64 => Some(CHOICE_POLICY),
        0x60 | 0x6d | 0x6f | 0x71 | 0x72 | 0x74 | 0x7c | 0x7d | 0x8a => Some(POINTER_POLICY),
        _ => None,
    }
}

fn read_u32(bytes: &[u8], offset: usize, label: &str) -> Result<u32> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| Error::new(format!("{label}: integer offset overflow")))?;
    let source = bytes
        .get(offset..end)
        .ok_or_else(|| Error::new(format!("{label}: truncated u32 at 0x{offset:x}")))?;
    Ok(u32::from_le_bytes(
        source.try_into().expect("four-byte slice"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cp932(text: &str) -> Vec<u8> {
        let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
        assert!(!had_errors);
        encoded.into_owned()
    }

    fn fixture() -> Vec<u8> {
        let source_strings = ["ﾕｶﾘ", "｢ﾊｲ!｣\n", "ｲｸ?"];
        let mut string_section = Vec::new();
        let mut offsets = Vec::new();
        for text in source_strings {
            offsets.push(string_section.len() as u32);
            string_section.extend(cp932(text));
            string_section.push(0);
        }

        let mut commands = Vec::new();
        commands.extend([0x5e, 0x10, 0, 0]);
        commands.extend(offsets[0].to_le_bytes());
        commands.extend(IGNORE_PARAMETER.to_le_bytes());
        commands.extend(offsets[1].to_le_bytes());
        commands.extend([0x64, 0x0c, 0, 0]);
        commands.extend(7u32.to_le_bytes());
        commands.extend(offsets[2].to_le_bytes());

        for byte in &mut string_section {
            *byte ^= XOR_KEY;
        }

        let mut file = b"SCR:2005fixture\0".to_vec();
        assert_eq!(file.len(), HEADER_SIZE);
        file.extend((commands.len() as u32).to_le_bytes());
        file.extend(commands);
        file.extend((string_section.len() as u32).to_le_bytes());
        file.extend(string_section);
        file
    }

    #[test]
    fn parses_message_name_and_choice() {
        let parsed = parse_version5(&fixture(), "fixture.scr").unwrap();
        assert_eq!(parsed.stats.commands, 2);
        assert_eq!(parsed.stats.strings, 3);
        assert_eq!(parsed.stats.messages, 1);
        assert_eq!(parsed.stats.named_messages, 1);
        assert_eq!(parsed.stats.choices, 1);
        assert!(parsed.warnings.is_empty());
        assert_eq!(parsed.records[0].name.as_deref(), Some("ﾕｶﾘ"));
        assert_eq!(parsed.records[0].message, "｢ﾊｲ!｣");
        assert_eq!(parsed.records[0].terminator, Some("LF"));
        assert_eq!(parsed.records[1].message, "ｲｸ?");
    }

    #[test]
    fn rejects_bad_signature() {
        let mut bytes = fixture();
        bytes[0] = 0;
        let error = parse_version5(&bytes, "bad.scr").unwrap_err();
        assert!(error.to_string().contains("SCR:2005"));
    }

    #[test]
    fn rejects_non_boundary_string_pointer() {
        let mut bytes = fixture();
        bytes[0x18..0x1c].copy_from_slice(&1u32.to_le_bytes());
        let error = parse_version5(&bytes, "bad.scr").unwrap_err();
        assert!(error.to_string().contains("not on a string boundary"));
    }

    #[test]
    fn rejects_truncated_section() {
        let mut bytes = fixture();
        bytes.pop();
        let error = parse_version5(&bytes, "bad.scr").unwrap_err();
        assert!(error.to_string().contains("declared sections end"));
    }

    #[test]
    fn rebuilds_variable_length_strings_and_all_pointers() {
        let source = fixture();
        let parsed = parse_version5(&source, "fixture.scr").unwrap();
        let mut replacements = BTreeMap::new();
        let mut message = cp932("｢ﾓｯﾄ長ｲ文章!｣");
        message.push(b'\n');
        replacements.insert(parsed.records[0].string_offset, message);
        replacements.insert(
            parsed.records[0].name_string_offset.unwrap(),
            cp932("新名前"),
        );
        replacements.insert(parsed.records[1].string_offset, cp932("別ﾉ選択肢?"));

        let rebuilt = rebuild_version5(&source, "fixture.scr", &replacements).unwrap();
        let reparsed = parse_version5(&rebuilt, "fixture.scr").unwrap();
        assert_eq!(reparsed.records[0].name.as_deref(), Some("新名前"));
        assert_eq!(reparsed.records[0].message, "｢ﾓｯﾄ長ｲ文章!｣");
        assert_eq!(reparsed.records[1].message, "別ﾉ選択肢?");
        assert_ne!(rebuilt, source);
    }

    #[test]
    fn unchanged_rebuild_is_byte_exact() {
        let source = fixture();
        assert_eq!(
            rebuild_version5(&source, "fixture.scr", &BTreeMap::new()).unwrap(),
            source
        );
    }
}
