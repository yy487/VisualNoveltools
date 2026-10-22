use crate::{hex, ToolError, ToolResult};
use encoding_rs::SHIFT_JIS;
use std::collections::{HashMap, HashSet};

pub const MAX_RUNTIME_OPCODE: u16 = 97;
pub const VOICE_MARKER: &[u8; 2] = b"\\V";
pub const VOICE_SUFFIX_MAX_LEN: usize = 18;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbStringKind {
    Message,
    Choice,
    ChoicePrompt,
    Buffer,
    Resource,
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
    ConditionalPositionedChoice,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbString {
    pub offset: usize,
    pub raw: Vec<u8>,
    pub text: String,
    pub kind: AbStringKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbVoice {
    pub offset: usize,
    pub raw: Vec<u8>,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbTarget {
    pub field_offset: usize,
    pub value: u32,
    pub kind: AbTargetKind,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbPlainText {
    pub raw: Vec<u8>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbFile {
    Bytecode(AbScript),
    PlainCp932(AbPlainText),
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

    pub fn strings(&self) -> impl Iterator<Item = (&AbInstruction, &AbString)> {
        self.instructions.iter().flat_map(|instruction| {
            instruction
                .strings
                .iter()
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
            hex(raw)
        )));
    }
    let text = decoded.into_owned();
    let (encoded, _, encode_errors) = SHIFT_JIS.encode(&text);
    if encode_errors || encoded.as_ref() != raw {
        return Err(ToolError(format!(
            "{label} failed CP932 byte round-trip: {}",
            hex(raw)
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

fn read_voice(data: &[u8], offset: usize) -> ToolResult<(AbVoice, usize)> {
    if data.get(offset..offset + VOICE_MARKER.len()) != Some(VOICE_MARKER) {
        return Err(ToolError(format!("voice marker is absent at 0x{offset:x}")));
    }
    let suffix_start = checked_end(offset, VOICE_MARKER.len(), "voice marker")?;
    let suffix_end = checked_end(suffix_start, VOICE_SUFFIX_MAX_LEN, "voice id")?.min(data.len());
    let period_offset = data[suffix_start..suffix_end]
        .iter()
        .position(|byte| *byte == b'.')
        .map(|relative| suffix_start + relative)
        .ok_or_else(|| {
            ToolError(format!(
                "opcode 0 voice at 0x{offset:x} has no '.' within {VOICE_SUFFIX_MAX_LEN} bytes after \\V"
            ))
        })?;
    let end = checked_end(period_offset, 1, "voice id")?;
    let raw = data[offset..end].to_vec();
    let text = decode_cp932(&raw, "voice id")?;
    Ok((AbVoice { offset, raw, text }, end))
}

fn push_target(
    data: &[u8],
    targets: &mut Vec<AbTarget>,
    field_offset: usize,
    kind: AbTargetKind,
    label: &str,
) -> ToolResult<()> {
    let value = read_u32(data, field_offset, label)?;
    targets.push(AbTarget {
        field_offset,
        value,
        kind,
    });
    Ok(())
}

fn read_prefixed_string(
    data: &[u8],
    cursor: &mut usize,
    prefix_size: usize,
    kind: AbStringKind,
    label: &str,
) -> ToolResult<AbString> {
    require(data, *cursor, prefix_size, label)?;
    *cursor += prefix_size;
    let (string, end) = read_cstring(data, *cursor, kind, label)?;
    *cursor = end;
    Ok(string)
}

/// Parses all operand layouts accepted by the Kokoro 2 interpreter at
/// KKR2VOI.EXE 0x40A8E0. Unsupported dispatch-table values are rejected.
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
                    let (parsed_voice, end) = read_voice(data, cursor)?;
                    voice = Some(parsed_voice);
                    cursor = end;
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
            3 | 37 => {
                require(data, cursor, 6, "three-u16 operands")?;
                cursor += 6;
            }
            4 | 9 => {
                require(data, cursor, 5, "assignment operands")?;
                cursor += 5;
            }
            5 | 7 | 10 | 11 | 13 | 25 | 33 | 35 | 40 | 41 | 53 | 54 | 56 | 73 | 78 | 82 => {}
            6 => {
                let (string, end) =
                    read_cstring(data, cursor, AbStringKind::ChoicePrompt, "choice prompt")?;
                strings.push(string);
                cursor = end;
            }
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
                if last < first {
                    return Err(ToolError(format!(
                        "opcode 12 at 0x{offset:x} has descending range {first}..{last}"
                    )));
                }
                cursor += 6;
                let count = usize::from(last - first) + 1;
                for _ in 0..count {
                    let value = read_u32(data, cursor, "sort target")?;
                    cursor += 4;
                    if value == u32::MAX {
                        break;
                    }
                    targets.push(AbTarget {
                        field_offset: cursor - 4,
                        value,
                        kind: AbTargetKind::Sort,
                    });
                }
            }
            14 | 17 | 18 | 20 | 45 | 49 | 77 | 80 | 92..=97 => {
                require(data, cursor, 2, "u16 operand")?;
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
            19 | 24 | 30 | 34 | 38 | 70 | 71 | 72 | 79 => {
                let (string, end) =
                    read_cstring(data, cursor, AbStringKind::Resource, "resource string")?;
                strings.push(string);
                cursor = end;
            }
            21 | 23 => {
                require(data, cursor, 4, "four-byte operands")?;
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
            48 => {
                require(data, cursor, 8, "rectangle operands")?;
                cursor += 8;
            }
            50 | 75 => {
                let string = read_prefixed_string(
                    data,
                    &mut cursor,
                    2,
                    AbStringKind::Resource,
                    "u16-prefixed resource string",
                )?;
                strings.push(string);
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
            63 => {
                for index in 0..2 {
                    let (string, end) = read_cstring(
                        data,
                        cursor,
                        AbStringKind::Resource,
                        &format!("opcode 63 resource string {index}"),
                    )?;
                    strings.push(string);
                    cursor = end;
                }
            }
            64 => {
                require(data, cursor, 12, "positioned choice operands")?;
                push_target(
                    data,
                    &mut targets,
                    cursor + 8,
                    AbTargetKind::PositionedChoice,
                    "positioned choice target",
                )?;
                cursor += 12;
            }
            65 | 69 => {
                require(data, cursor, 4, "u32 operand")?;
                cursor += 4;
            }
            76 => {
                let string = read_prefixed_string(
                    data,
                    &mut cursor,
                    4,
                    AbStringKind::Resource,
                    "two-u16-prefixed resource string",
                )?;
                strings.push(string);
            }
            81 => {
                require(data, cursor, 14, "conditional positioned choice operands")?;
                push_target(
                    data,
                    &mut targets,
                    cursor + 10,
                    AbTargetKind::ConditionalPositionedChoice,
                    "conditional positioned choice target",
                )?;
                cursor += 14;
            }
            83 => {
                let string = read_prefixed_string(
                    data,
                    &mut cursor,
                    6,
                    AbStringKind::Resource,
                    "three-u16-prefixed resource string",
                )?;
                strings.push(string);
            }
            _ => {
                return Err(ToolError(format!(
                    "unsupported Kokoro 2 runtime opcode {opcode} at 0x{offset:x}"
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

pub fn parse_file(data: &[u8]) -> ToolResult<AbFile> {
    if data.len() >= 2 && read_u16(data, 0, "opcode")? <= MAX_RUNTIME_OPCODE {
        if let Ok(script) = parse_script(data) {
            return Ok(AbFile::Bytecode(script));
        }
    }
    if data.contains(&0) {
        return Err(ToolError(
            "file is neither supported bytecode nor NUL-free CP932 plain text".to_string(),
        ));
    }
    let text = decode_cp932(data, "plain .AB text")?;
    Ok(AbFile::PlainCp932(AbPlainText {
        raw: data.to_vec(),
        text,
    }))
}

#[derive(Debug)]
struct BytePatch {
    start: usize,
    end: usize,
    bytes: Vec<u8>,
}

/// Rebuilds a parsed bytecode script while replacing selected cstring payloads
/// and relocating every parsed absolute bytecode target.
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
        .strings()
        .map(|(_, string)| string.offset)
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
    fn parses_kokoro2_voice_and_extended_operands() {
        let mut data = Vec::new();
        push_u16(&mut data, 0);
        push_u16(&mut data, 7);
        data.extend_from_slice(b"\\VAB001.");
        data.extend_from_slice(b"Name\\NBody");
        data.push(0);

        let choice_offset = data.len();
        push_u16(&mut data, 1);
        let choice_target_field = data.len();
        push_u32(&mut data, 0);
        data.extend_from_slice(b"Choice");
        data.push(0);

        let positioned_offset = data.len();
        push_u16(&mut data, 81);
        for value in [1u16, 2, 3, 4, 5] {
            push_u16(&mut data, value);
        }
        let positioned_target_field = data.len();
        push_u32(&mut data, 0);

        let end_offset = data.len();
        push_u16(&mut data, 13);
        data[choice_target_field..choice_target_field + 4]
            .copy_from_slice(&(positioned_offset as u32).to_le_bytes());
        data[positioned_target_field..positioned_target_field + 4]
            .copy_from_slice(&(end_offset as u32).to_le_bytes());

        let script = parse_script(&data).expect("valid script");
        assert_eq!(script.instructions.len(), 4);
        assert_eq!(script.instructions[0].message_id, Some(7));
        assert_eq!(
            script.instructions[0].voice.as_ref().expect("voice").text,
            "\\VAB001."
        );
        assert_eq!(script.instructions[1].strings[0].kind, AbStringKind::Choice);
        assert_eq!(script.instructions[2].opcode, 81);
        assert_eq!(script.instructions[2].targets[0].value as usize, end_offset);
        assert_eq!(script.instructions[1].offset, choice_offset);
    }

    #[test]
    fn parses_opcode_75_and_83_resource_layouts() {
        let mut data = Vec::new();
        push_u16(&mut data, 75);
        push_u16(&mut data, 2);
        data.extend_from_slice(b"BG01");
        data.push(0);
        push_u16(&mut data, 83);
        push_u16(&mut data, 3);
        push_u16(&mut data, 100);
        push_u16(&mut data, 200);
        data.extend_from_slice(b"overlay");
        data.push(0);
        push_u16(&mut data, 13);

        let script = parse_script(&data).expect("valid script");
        assert_eq!(script.instructions.len(), 3);
        assert_eq!(script.instructions[0].strings[0].text, "BG01");
        assert_eq!(script.instructions[1].strings[0].text, "overlay");
    }

    #[test]
    fn rebuilds_longer_message_and_relocates_targets() {
        let mut data = Vec::new();
        push_u16(&mut data, 1);
        let target_field = data.len();
        push_u32(&mut data, 0);
        data.extend_from_slice(b"A");
        data.push(0);
        let message_offset = data.len();
        push_u16(&mut data, 0);
        push_u16(&mut data, 0);
        let string_offset = data.len();
        data.extend_from_slice(b"Old");
        data.push(0);
        let target = data.len();
        push_u16(&mut data, 13);
        data[target_field..target_field + 4].copy_from_slice(&(target as u32).to_le_bytes());

        let script = parse_script(&data).expect("valid script");
        let mut replacements = HashMap::new();
        replacements.insert(string_offset, b"A longer replacement".to_vec());
        let rebuilt = rebuild_script(&data, &script, &replacements).expect("valid rebuild");
        let reparsed = parse_script(&rebuilt).expect("valid rebuilt script");
        assert_eq!(reparsed.instructions[1].offset, message_offset);
        assert_eq!(
            reparsed.instructions[1].strings[0].text,
            "A longer replacement"
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
        push_u16(&mut data, 0);
        data.extend_from_slice(b"Text");
        data.push(0);
        push_u16(&mut data, 13);
        let script = parse_script(&data).expect("valid script");
        assert_eq!(
            rebuild_script(&data, &script, &HashMap::new()).expect("valid rebuild"),
            data
        );
    }

    #[test]
    fn parses_every_runtime_accepted_operand_layout() {
        let mut data = Vec::new();
        let mut target_fields = Vec::new();

        push_u16(&mut data, 0);
        push_u16(&mut data, 0);
        data.extend_from_slice(b"Text");
        data.push(0);

        push_u16(&mut data, 1);
        target_fields.push(data.len());
        push_u32(&mut data, 0);
        data.extend_from_slice(b"Choice");
        data.push(0);

        push_u16(&mut data, 2);
        target_fields.push(data.len());
        push_u32(&mut data, 0);

        for opcode in [3u16, 37] {
            push_u16(&mut data, opcode);
            for value in [1u16, 2, 3] {
                push_u16(&mut data, value);
            }
        }

        for opcode in [4u16, 9] {
            push_u16(&mut data, opcode);
            push_u16(&mut data, 1);
            data.push(b'+');
            push_u16(&mut data, 2);
        }

        for opcode in [
            5u16, 7, 10, 11, 13, 25, 33, 35, 40, 41, 53, 54, 56, 73, 78, 82,
        ] {
            push_u16(&mut data, opcode);
        }

        push_u16(&mut data, 6);
        data.extend_from_slice(b"Prompt");
        data.push(0);

        push_u16(&mut data, 8);
        push_u16(&mut data, 0);
        data.extend_from_slice(b"Buffer");
        data.push(0);

        push_u16(&mut data, 12);
        push_u16(&mut data, 0);
        push_u16(&mut data, 0);
        push_u16(&mut data, 0);
        target_fields.push(data.len());
        push_u32(&mut data, 0);

        for opcode in [14u16, 17, 18, 20, 45, 49, 77, 80, 92, 93, 94, 95, 96, 97] {
            push_u16(&mut data, opcode);
            push_u16(&mut data, 1);
        }

        push_u16(&mut data, 16);
        push_u16(&mut data, 1);
        data.push(b'=');
        push_u16(&mut data, 1);
        target_fields.push(data.len());
        push_u32(&mut data, 0);

        for opcode in [19u16, 24, 30, 34, 38, 70, 71, 72, 79] {
            push_u16(&mut data, opcode);
            data.extend_from_slice(b"resource");
            data.push(0);
        }

        for opcode in [21u16, 23] {
            push_u16(&mut data, opcode);
            data.extend_from_slice(&[1, 2, 3, 4]);
        }

        push_u16(&mut data, 32);
        target_fields.push(data.len());
        push_u32(&mut data, 0);

        push_u16(&mut data, 48);
        for value in [1u16, 2, 3, 4] {
            push_u16(&mut data, value);
        }

        for opcode in [50u16, 75] {
            push_u16(&mut data, opcode);
            push_u16(&mut data, 1);
            data.extend_from_slice(b"resource");
            data.push(0);
        }

        for opcode in [57u16, 58] {
            push_u16(&mut data, opcode);
            target_fields.push(data.len());
            push_u32(&mut data, 0);
            push_u16(&mut data, u16::MAX);
        }

        push_u16(&mut data, 63);
        data.extend_from_slice(b"left");
        data.push(0);
        data.extend_from_slice(b"right");
        data.push(0);

        push_u16(&mut data, 64);
        for value in [1u16, 2, 3, 4] {
            push_u16(&mut data, value);
        }
        target_fields.push(data.len());
        push_u32(&mut data, 0);

        for opcode in [65u16, 69] {
            push_u16(&mut data, opcode);
            push_u32(&mut data, 1);
        }

        push_u16(&mut data, 76);
        push_u16(&mut data, 1);
        push_u16(&mut data, 2);
        data.extend_from_slice(b"resource");
        data.push(0);

        push_u16(&mut data, 81);
        for value in [1u16, 2, 3, 4, 5] {
            push_u16(&mut data, value);
        }
        target_fields.push(data.len());
        push_u32(&mut data, 0);

        push_u16(&mut data, 83);
        for value in [1u16, 2, 3] {
            push_u16(&mut data, value);
        }
        data.extend_from_slice(b"resource");
        data.push(0);

        let end_offset = data.len() as u32;
        for field in target_fields {
            data[field..field + 4].copy_from_slice(&end_offset.to_le_bytes());
        }

        let script = parse_script(&data).expect("all accepted layouts parse");
        assert_eq!(script.file_size, data.len());
        assert_eq!(
            script.instruction_boundaries().len(),
            script.instructions.len() + 1
        );
        assert_eq!(
            rebuild_script(&data, &script, &HashMap::new()).expect("rebuild"),
            data
        );
    }

    #[test]
    fn classifies_plain_cp932_ab_file() {
        let bytes = encode_cp932("plain text", "test").expect("encodable");
        let parsed = parse_file(&bytes).expect("plain text");
        assert!(matches!(parsed, AbFile::PlainCp932(_)));
    }

    #[test]
    fn rejects_unsupported_runtime_opcode() {
        let error = parse_script(&15u16.to_le_bytes()).expect_err("unsupported opcode");
        assert!(error
            .to_string()
            .contains("unsupported Kokoro 2 runtime opcode 15"));
    }
}
