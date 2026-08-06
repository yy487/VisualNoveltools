use crate::{ToolError, ToolResult};
use encoding_rs::SHIFT_JIS;
use std::collections::{HashMap, HashSet};

pub const MESSAGE_SEPARATOR: &str = "\\N";
pub const VOICE_MARKER: &[u8; 2] = b"\\V";
pub const VOICE_ID_SIZE: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbStringKind {
    Message,
    Choice,
    Buffer,
    Resource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbString {
    pub offset: usize,
    pub raw: Vec<u8>,
    pub text: String,
    pub kind: AbStringKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbTargetKind {
    Choice,
    Jump,
    Sort,
    Conditional,
    Call,
    MultiConditional,
    PositionedChoice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbTarget {
    pub field_offset: usize,
    pub value: u32,
    pub kind: AbTargetKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbVoice {
    pub offset: usize,
    pub raw: Vec<u8>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbInstruction {
    pub offset: usize,
    pub end: usize,
    pub opcode: u16,
    pub strings: Vec<AbString>,
    pub targets: Vec<AbTarget>,
    pub message_id: Option<u16>,
    pub voice: Option<AbVoice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbScript {
    pub file_size: usize,
    pub instructions: Vec<AbInstruction>,
}

impl AbScript {
    pub fn instruction_boundaries(&self) -> HashSet<usize> {
        let mut boundaries = self
            .instructions
            .iter()
            .map(|instruction| instruction.offset)
            .collect::<HashSet<_>>();
        boundaries.insert(self.file_size);
        boundaries
    }

    pub fn text_strings(&self) -> impl Iterator<Item = (&AbInstruction, &AbString)> {
        self.instructions.iter().flat_map(|instruction| {
            instruction
                .strings
                .iter()
                .filter(|string| {
                    matches!(
                        string.kind,
                        AbStringKind::Message | AbStringKind::Choice | AbStringKind::Buffer
                    )
                })
                .map(move |string| (instruction, string))
        })
    }
}

fn checked_end(start: usize, size: usize, label: &str) -> ToolResult<usize> {
    start
        .checked_add(size)
        .ok_or_else(|| ToolError(format!("{label} range overflows usize")))
}

fn require(data: &[u8], offset: usize, size: usize, label: &str) -> ToolResult<()> {
    let end = checked_end(offset, size, label)?;
    if end > data.len() {
        return Err(ToolError(format!(
            "{label} at 0x{offset:x} is truncated: need {size} bytes, file size is 0x{:x}",
            data.len()
        )));
    }
    Ok(())
}

fn read_u16(data: &[u8], offset: usize, label: &str) -> ToolResult<u16> {
    require(data, offset, 2, label)?;
    Ok(u16::from_le_bytes([data[offset], data[offset + 1]]))
}

fn read_u32(data: &[u8], offset: usize, label: &str) -> ToolResult<u32> {
    require(data, offset, 4, label)?;
    Ok(u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

pub fn decode_cp932(raw: &[u8], label: &str) -> ToolResult<String> {
    let (decoded, _, had_errors) = SHIFT_JIS.decode(raw);
    if had_errors {
        return Err(ToolError(format!(
            "{label} is not valid CP932/Shift-JIS: {}",
            crate::hex(raw)
        )));
    }
    let text = decoded.into_owned();
    let (encoded, _, encode_errors) = SHIFT_JIS.encode(&text);
    if encode_errors || encoded.as_ref() != raw {
        return Err(ToolError(format!(
            "{label} failed CP932 byte round-trip: {}",
            crate::hex(raw)
        )));
    }
    Ok(text)
}

pub fn encode_cp932(text: &str, label: &str) -> ToolResult<Vec<u8>> {
    let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        let mut unsupported = Vec::new();
        for character in text.chars() {
            let value = character.to_string();
            let (_, _, character_errors) = SHIFT_JIS.encode(&value);
            if character_errors && !unsupported.contains(&character) {
                unsupported.push(character);
            }
        }
        let details = unsupported
            .iter()
            .map(|character| format!("{character:?} (U+{:04X})", u32::from(*character)))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(ToolError(format!(
            "{label} contains characters not encodable as CP932: {details}"
        )));
    }
    Ok(encoded.into_owned())
}

fn read_cstring(
    data: &[u8],
    offset: usize,
    kind: AbStringKind,
    label: &str,
) -> ToolResult<(AbString, usize)> {
    let relative_end = data
        .get(offset..)
        .ok_or_else(|| ToolError(format!("{label} starts beyond file at 0x{offset:x}")))?
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| ToolError(format!("{label} at 0x{offset:x} has no NUL terminator")))?;
    let end = checked_end(offset, relative_end, label)?;
    let raw = data[offset..end].to_vec();
    let text = decode_cp932(&raw, label)?;
    Ok((
        AbString {
            offset,
            raw,
            text,
            kind,
        },
        end + 1,
    ))
}

fn push_target(
    data: &[u8],
    targets: &mut Vec<AbTarget>,
    field_offset: usize,
    kind: AbTargetKind,
    label: &str,
) -> ToolResult<u32> {
    let value = read_u32(data, field_offset, label)?;
    targets.push(AbTarget {
        field_offset,
        value,
        kind,
    });
    Ok(value)
}

/// Parses every byte of a KOKOROV `.AB` script using the opcode operand layout
/// implemented by the game's interpreter at `KOKOROV.EXE:0x409B47`.
pub fn parse_script(data: &[u8]) -> ToolResult<AbScript> {
    let mut instructions = Vec::new();
    let mut cursor = 0usize;

    while cursor < data.len() {
        let offset = cursor;
        let opcode = read_u16(data, cursor, "opcode")?;
        cursor += 2;
        let mut strings = Vec::new();
        let mut targets = Vec::new();
        let mut message_id = None;
        let mut voice = None;

        match opcode {
            0 => {
                message_id = Some(read_u16(data, cursor, "message id")?);
                cursor += 2;
                if data.get(cursor..cursor + VOICE_MARKER.len()) == Some(VOICE_MARKER) {
                    cursor += VOICE_MARKER.len();
                    require(data, cursor, VOICE_ID_SIZE, "voice id")?;
                    let raw = data[cursor..cursor + VOICE_ID_SIZE].to_vec();
                    let text = decode_cp932(&raw, "voice id")?;
                    voice = Some(AbVoice {
                        offset: cursor,
                        raw,
                        text,
                    });
                    cursor += VOICE_ID_SIZE;
                }
                let (string, end) =
                    read_cstring(data, cursor, AbStringKind::Message, "message string")?;
                strings.push(string);
                cursor = end;
            }
            1 => {
                push_target(
                    data,
                    &mut targets,
                    cursor,
                    AbTargetKind::Choice,
                    "choice target",
                )?;
                cursor += 4;
                let (string, end) =
                    read_cstring(data, cursor, AbStringKind::Choice, "choice string")?;
                strings.push(string);
                cursor = end;
            }
            2 => {
                push_target(
                    data,
                    &mut targets,
                    cursor,
                    AbTargetKind::Jump,
                    "jump target",
                )?;
                cursor += 4;
            }
            3 => {
                require(data, cursor, 6, "opcode 3 operands")?;
                cursor += 6;
            }
            4 | 9 => {
                require(data, cursor, 5, "assignment operands")?;
                cursor += 5;
            }
            5 | 6 | 7 | 10 | 13 | 25 | 33 | 35 | 40 | 41 | 53 | 54 | 56 | 65 | 67 | 73 => {}
            8 => {
                let buffer_index = read_u16(data, cursor, "message buffer index")?;
                if buffer_index > 9 {
                    return Err(ToolError(format!(
                        "opcode 8 at 0x{offset:x} uses message buffer {buffer_index}, expected 0..9"
                    )));
                }
                cursor += 2;
                let (string, end) =
                    read_cstring(data, cursor, AbStringKind::Buffer, "message buffer string")?;
                if string.raw.len() > 16 {
                    return Err(ToolError(format!(
                        "opcode 8 at 0x{offset:x} message buffer string is {} bytes, runtime limit is 16",
                        string.raw.len()
                    )));
                }
                strings.push(string);
                cursor = end;
            }
            12 => {
                let first = read_u16(data, cursor, "sort first variable")?;
                let last = read_u16(data, cursor + 2, "sort last variable")?;
                require(data, cursor, 6, "sort header")?;
                cursor += 6;
                let count = last.checked_sub(first).ok_or_else(|| {
                    ToolError(format!(
                        "opcode 12 at 0x{offset:x} has descending range {first}..{last}"
                    ))
                })? as usize
                    + 1;
                for _ in 0..count {
                    let value = read_u32(data, cursor, "sort target")?;
                    if value == u32::MAX {
                        cursor += 4;
                        break;
                    }
                    push_target(
                        data,
                        &mut targets,
                        cursor,
                        AbTargetKind::Sort,
                        "sort target",
                    )?;
                    cursor += 4;
                }
            }
            14 => {
                require(data, cursor, 2, "engine command id")?;
                cursor += 2;
            }
            16 => {
                require(data, cursor, 9, "conditional operands")?;
                push_target(
                    data,
                    &mut targets,
                    cursor + 5,
                    AbTargetKind::Conditional,
                    "conditional target",
                )?;
                cursor += 9;
            }
            17 | 18 | 20 | 36 | 45 | 49 | 77 => {
                require(data, cursor, 2, "u16 operand")?;
                cursor += 2;
            }
            19 | 22 | 24 | 30 | 34 | 38 | 70 | 71 | 72 | 75 | 79 => {
                let (string, end) =
                    read_cstring(data, cursor, AbStringKind::Resource, "resource string")?;
                strings.push(string);
                cursor = end;
            }
            21 => {
                require(data, cursor, 4, "color operands")?;
                cursor += 4;
            }
            23 => {
                require(data, cursor, 4, "display mode operands")?;
                cursor += 4;
            }
            32 => {
                push_target(
                    data,
                    &mut targets,
                    cursor,
                    AbTargetKind::Call,
                    "call target",
                )?;
                cursor += 4;
            }
            37 => {
                require(data, cursor, 6, "random operands")?;
                cursor += 6;
            }
            48 => {
                require(data, cursor, 8, "rectangle operands")?;
                cursor += 8;
            }
            55 => {
                for index in 0..2 {
                    let (string, end) = read_cstring(
                        data,
                        cursor,
                        AbStringKind::Resource,
                        &format!("opcode 55 resource string {index}"),
                    )?;
                    strings.push(string);
                    cursor = end;
                }
            }
            57 | 58 => {
                push_target(
                    data,
                    &mut targets,
                    cursor,
                    AbTargetKind::MultiConditional,
                    "multi-condition target",
                )?;
                cursor += 4;
                for _ in 0..5 {
                    let variable = read_u16(data, cursor, "multi-condition variable")?;
                    cursor += 2;
                    if variable == u16::MAX {
                        break;
                    }
                    require(data, cursor, 3, "multi-condition expression")?;
                    cursor += 3;
                }
            }
            64 => {
                require(data, cursor, 12, "positioned choice operands")?;
                push_target(
                    data,
                    &mut targets,
                    cursor,
                    AbTargetKind::PositionedChoice,
                    "positioned choice target",
                )?;
                cursor += 12;
            }
            66 => {
                require(data, cursor, 4, "positioned text coordinates")?;
                cursor += 4;
                let (string, end) = read_cstring(
                    data,
                    cursor,
                    AbStringKind::Resource,
                    "positioned text string",
                )?;
                strings.push(string);
                cursor = end;
            }
            68 => {
                require(data, cursor, 2, "opcode 68 number")?;
                cursor += 2;
                let (string, end) =
                    read_cstring(data, cursor, AbStringKind::Resource, "opcode 68 string")?;
                strings.push(string);
                cursor = end;
            }
            69 => {
                require(data, cursor, 4, "opcode 69 operand")?;
                cursor += 4;
            }
            _ => {
                return Err(ToolError(format!(
                    "unknown AB opcode {opcode} at 0x{offset:x}"
                )));
            }
        }

        instructions.push(AbInstruction {
            offset,
            end: cursor,
            opcode,
            strings,
            targets,
            message_id,
            voice,
        });
    }

    let script = AbScript {
        file_size: data.len(),
        instructions,
    };
    let boundaries = script.instruction_boundaries();
    for instruction in &script.instructions {
        for target in &instruction.targets {
            let target_offset = usize::try_from(target.value).map_err(|_| {
                ToolError(format!(
                    "opcode {} at 0x{:x} target 0x{:x} does not fit usize",
                    instruction.opcode, instruction.offset, target.value
                ))
            })?;
            if !boundaries.contains(&target_offset) {
                return Err(ToolError(format!(
                    "opcode {} at 0x{:x} target field 0x{:x} points to non-instruction boundary 0x{:x}",
                    instruction.opcode, instruction.offset, target.field_offset, target.value
                )));
            }
        }
    }
    Ok(script)
}

#[derive(Debug)]
struct BytePatch {
    start: usize,
    end: usize,
    bytes: Vec<u8>,
}

/// Rebuilds a parsed script while replacing selected NUL-terminated string
/// payloads and relocating every absolute bytecode target.
pub fn rebuild_script(
    data: &[u8],
    script: &AbScript,
    replacements: &HashMap<usize, Vec<u8>>,
) -> ToolResult<Vec<u8>> {
    if data.len() != script.file_size {
        return Err(ToolError(format!(
            "source size {} does not match parsed script size {}",
            data.len(),
            script.file_size
        )));
    }

    let known_string_offsets = script
        .instructions
        .iter()
        .flat_map(|instruction| instruction.strings.iter().map(|string| string.offset))
        .collect::<HashSet<_>>();
    for (offset, replacement) in replacements {
        if !known_string_offsets.contains(offset) {
            return Err(ToolError(format!(
                "replacement at 0x{offset:x} is not a parsed string boundary"
            )));
        }
        if replacement.contains(&0) {
            return Err(ToolError(format!(
                "replacement at 0x{offset:x} contains NUL"
            )));
        }
    }

    let mut relocated_offsets = HashMap::new();
    let mut output_size = 0usize;
    for instruction in &script.instructions {
        relocated_offsets.insert(instruction.offset, output_size);
        let mut instruction_size = instruction.end - instruction.offset;
        for string in &instruction.strings {
            if let Some(replacement) = replacements.get(&string.offset) {
                instruction_size = instruction_size
                    .checked_sub(string.raw.len())
                    .and_then(|size| size.checked_add(replacement.len()))
                    .ok_or_else(|| {
                        ToolError(format!(
                            "opcode {} at 0x{:x} replacement size overflows",
                            instruction.opcode, instruction.offset
                        ))
                    })?;
            }
        }
        output_size = output_size
            .checked_add(instruction_size)
            .ok_or_else(|| ToolError("rebuilt script size overflows usize".to_string()))?;
    }
    relocated_offsets.insert(script.file_size, output_size);

    let mut output = Vec::with_capacity(output_size);
    for instruction in &script.instructions {
        let mut patches = Vec::new();
        for string in &instruction.strings {
            if let Some(replacement) = replacements.get(&string.offset) {
                patches.push(BytePatch {
                    start: string.offset,
                    end: checked_end(string.offset, string.raw.len(), "string patch")?,
                    bytes: replacement.clone(),
                });
            }
        }
        for target in &instruction.targets {
            let old_target = usize::try_from(target.value).map_err(|_| {
                ToolError(format!(
                    "target 0x{:x} at 0x{:x} does not fit usize",
                    target.value, target.field_offset
                ))
            })?;
            let new_target = *relocated_offsets.get(&old_target).ok_or_else(|| {
                ToolError(format!(
                    "target 0x{:x} at 0x{:x} has no relocated instruction boundary",
                    target.value, target.field_offset
                ))
            })?;
            let new_target = u32::try_from(new_target).map_err(|_| {
                ToolError(format!(
                    "relocated target 0x{new_target:x} does not fit u32"
                ))
            })?;
            patches.push(BytePatch {
                start: target.field_offset,
                end: checked_end(target.field_offset, 4, "target patch")?,
                bytes: new_target.to_le_bytes().to_vec(),
            });
        }
        patches.sort_by_key(|patch| patch.start);

        let mut source_cursor = instruction.offset;
        for patch in patches {
            if patch.start < source_cursor || patch.end < patch.start || patch.end > instruction.end
            {
                return Err(ToolError(format!(
                    "overlapping or out-of-range rebuild patch [0x{:x}, 0x{:x}) in opcode {} at 0x{:x}",
                    patch.start, patch.end, instruction.opcode, instruction.offset
                )));
            }
            output.extend_from_slice(&data[source_cursor..patch.start]);
            output.extend_from_slice(&patch.bytes);
            source_cursor = patch.end;
        }
        output.extend_from_slice(&data[source_cursor..instruction.end]);
    }
    if output.len() != output_size {
        return Err(ToolError(format!(
            "rebuilt size mismatch: calculated {output_size}, wrote {}",
            output.len()
        )));
    }

    let reparsed = parse_script(&output)?;
    if reparsed.instructions.len() != script.instructions.len()
        || reparsed
            .instructions
            .iter()
            .zip(&script.instructions)
            .any(|(new, old)| new.opcode != old.opcode)
    {
        return Err(ToolError(
            "rebuilt script changed the opcode sequence".to_string(),
        ));
    }
    Ok(output)
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

    #[test]
    fn parses_message_voice_choice_and_targets() {
        let mut data = Vec::new();
        push_u16(&mut data, 0);
        push_u16(&mut data, 7);
        data.extend_from_slice(VOICE_MARKER);
        data.extend_from_slice(b"J001_0003.");
        data.extend_from_slice(b"Mayumi\\N\"sample\"");
        data.push(0);
        let choice_offset = data.len();
        push_u16(&mut data, 1);
        let target_field = data.len();
        push_u32(&mut data, 0);
        data.extend_from_slice(" choice".as_bytes());
        data.push(0);
        let end_offset = data.len();
        push_u16(&mut data, 13);
        data[target_field..target_field + 4].copy_from_slice(&(end_offset as u32).to_le_bytes());

        let parsed = parse_script(&data).expect("valid script");
        assert_eq!(parsed.instructions.len(), 3);
        assert_eq!(parsed.instructions[0].message_id, Some(7));
        assert_eq!(
            parsed.instructions[0].voice.as_ref().unwrap().text,
            "J001_0003."
        );
        assert_eq!(
            parsed.instructions[0].strings[0].text,
            "Mayumi\\N\"sample\""
        );
        assert_eq!(parsed.instructions[1].offset, choice_offset);
        assert_eq!(parsed.instructions[1].targets[0].value, end_offset as u32);
    }

    #[test]
    fn parses_variable_multi_condition() {
        let mut data = Vec::new();
        push_u16(&mut data, 58);
        let target_field = data.len();
        push_u32(&mut data, 0);
        push_u16(&mut data, 3);
        data.push(b'=');
        push_u16(&mut data, 4);
        push_u16(&mut data, u16::MAX);
        let end_offset = data.len();
        push_u16(&mut data, 13);
        data[target_field..target_field + 4].copy_from_slice(&(end_offset as u32).to_le_bytes());

        let parsed = parse_script(&data).expect("valid script");
        assert_eq!(parsed.instructions.len(), 2);
        assert_eq!(parsed.instructions[0].end, end_offset);
        assert_eq!(parsed.instructions[0].targets[0].value, end_offset as u32);
    }

    #[test]
    fn rejects_invalid_target() {
        let mut data = Vec::new();
        push_u16(&mut data, 2);
        push_u32(&mut data, 1);
        push_u16(&mut data, 13);
        let error = parse_script(&data).unwrap_err().to_string();
        assert!(error.contains("non-instruction boundary"));
    }

    #[test]
    fn rejects_unknown_opcode() {
        let error = parse_script(&999u16.to_le_bytes()).unwrap_err().to_string();
        assert!(error.contains("unknown AB opcode 999"));
    }

    #[test]
    fn rejects_truncated_message() {
        let data = [0, 0, 1, 0, b'A'];
        let error = parse_script(&data).unwrap_err().to_string();
        assert!(error.contains("no NUL terminator"));
    }

    #[test]
    fn rebuilds_longer_string_and_relocates_target() {
        let mut data = Vec::new();
        push_u16(&mut data, 2);
        let target_field = data.len();
        push_u32(&mut data, 0);
        push_u16(&mut data, 0);
        push_u16(&mut data, 1);
        let string_offset = data.len();
        data.extend_from_slice(b"\\Nshort\0");
        let target = data.len();
        push_u16(&mut data, 13);
        data[target_field..target_field + 4].copy_from_slice(&(target as u32).to_le_bytes());

        let parsed = parse_script(&data).expect("valid script");
        let replacements = HashMap::from([(string_offset, b"\\Na much longer line".to_vec())]);
        let rebuilt = rebuild_script(&data, &parsed, &replacements).expect("valid rebuild");
        let reparsed = parse_script(&rebuilt).expect("valid rebuilt script");
        assert_eq!(
            reparsed.instructions[1].strings[0].text,
            "\\Na much longer line"
        );
        assert_eq!(
            reparsed.instructions[0].targets[0].value as usize,
            reparsed.instructions[2].offset
        );
    }

    #[test]
    fn identity_rebuild_is_byte_exact() {
        let mut data = Vec::new();
        push_u16(&mut data, 0);
        push_u16(&mut data, 1);
        data.extend_from_slice(b"\\Ntext\0");
        push_u16(&mut data, 13);
        let parsed = parse_script(&data).expect("valid script");
        let rebuilt = rebuild_script(&data, &parsed, &HashMap::new()).expect("valid rebuild");
        assert_eq!(rebuilt, data);
    }

    #[test]
    fn rejects_unencodable_cp932_text() {
        let error = encode_cp932("test\u{1f600}", "message")
            .unwrap_err()
            .to_string();
        assert!(error.contains("U+1F600"));
    }
}
