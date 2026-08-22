use crate::{fail, Result};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct MesScript {
    original: Vec<u8>,
    pub body_start: usize,
    pub header_offsets: Vec<u32>,
    pub instructions: Vec<Instruction>,
    references: Vec<CodeReference>,
}

#[derive(Clone, Debug)]
pub struct Instruction {
    pub offset: u32,
    pub end: u32,
    pub opcode: u8,
    pub kind: InstructionKind,
}

#[derive(Clone, Debug)]
pub enum InstructionKind {
    Text(TextSlot),
    Command(Command),
    Marker { message_id: u32 },
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextSlot {
    pub data_start: u32,
    pub data_end: u32,
}

#[derive(Clone, Debug)]
pub struct Command {
    pub command_id: Option<u32>,
    pub arguments: Vec<Argument>,
}

#[derive(Clone, Debug)]
pub enum Argument {
    String(TextSlot),
    Expression { value: Option<u32> },
}

#[derive(Clone, Debug)]
struct CodeReference {
    field_offset: u32,
    target: u32,
    opcode: u8,
}

#[derive(Clone, Debug)]
pub struct Replacement {
    pub start: u32,
    pub end: u32,
    pub data: Vec<u8>,
}

impl MesScript {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return fail("MES is smaller than its four-byte message count");
        }
        let count = read_u32(data, 0)? as usize;
        let body_start = 4usize
            .checked_add(count.checked_mul(4).ok_or("MES header size overflow")?)
            .ok_or("MES header size overflow")?;
        if body_start > data.len() {
            return fail(format!(
                "MES message table ends at 0x{body_start:X}, beyond file size 0x{:X}",
                data.len()
            ));
        }
        let body = &data[body_start..];
        let mut header_offsets = Vec::with_capacity(count);
        let mut previous = None;
        for index in 0..count {
            let offset = read_u32(data, 4 + index * 4)?;
            if offset as usize >= body.len() {
                return fail(format!(
                    "MES message table entry #{index} targets 0x{offset:X}, outside body size 0x{:X}",
                    body.len()
                ));
            }
            if let Some(old) = previous {
                if offset <= old {
                    return fail(format!(
                        "MES message table is not strictly increasing at entry #{index}: 0x{old:X} then 0x{offset:X}"
                    ));
                }
            }
            let target = offset as usize;
            if target + 5 > body.len() || body[target] != 0x17 {
                return fail(format!(
                    "MES message table entry #{index} does not target a 0x17 marker at body offset 0x{offset:X}"
                ));
            }
            let message_id = read_u32(body, target + 1)?;
            if message_id as usize != index {
                return fail(format!(
                    "MES marker at body offset 0x{offset:X} has id {message_id}, expected {index}"
                ));
            }
            header_offsets.push(offset);
            previous = Some(offset);
        }

        let (instructions, references) = parse_body(body)?;
        let mut boundaries: HashSet<u32> = instructions.iter().map(|item| item.offset).collect();
        boundaries.insert(u32::try_from(body.len()).map_err(|_| "MES body exceeds u32")?);
        for (index, offset) in header_offsets.iter().enumerate() {
            if !boundaries.contains(offset) {
                return fail(format!(
                    "MES table entry #{index} targets 0x{offset:X}, which is not an instruction boundary"
                ));
            }
        }
        for reference in &references {
            if !boundaries.contains(&reference.target) {
                return fail(format!(
                    "MES opcode 0x{:02X} at body offset 0x{:X} targets 0x{:X}, which is not an instruction boundary",
                    reference.opcode,
                    reference.field_offset.saturating_sub(1),
                    reference.target
                ));
            }
        }

        Ok(Self {
            original: data.to_vec(),
            body_start,
            header_offsets,
            instructions,
            references,
        })
    }

    pub fn body(&self) -> &[u8] {
        &self.original[self.body_start..]
    }

    pub fn text_bytes(&self, slot: &TextSlot) -> &[u8] {
        &self.body()[slot.data_start as usize..slot.data_end as usize]
    }

    pub fn instruction_bytes(&self, instruction: &Instruction) -> &[u8] {
        &self.body()[instruction.offset as usize..instruction.end as usize]
    }

    pub fn reference_target_for_instruction(&self, instruction: &Instruction) -> Option<u32> {
        self.references
            .iter()
            .find(|reference| {
                reference.field_offset >= instruction.offset
                    && reference.field_offset < instruction.end
            })
            .map(|reference| reference.target)
    }

    pub fn rebuild(&self, mut replacements: Vec<Replacement>) -> Result<Vec<u8>> {
        if replacements.is_empty() {
            return Ok(self.original.clone());
        }
        replacements.sort_by_key(|replacement| replacement.start);
        let body = self.body();
        let mut previous_end = 0u32;
        for replacement in &replacements {
            if replacement.start > replacement.end || replacement.end as usize > body.len() {
                return fail(format!(
                    "invalid MES replacement range 0x{:X}..0x{:X} for body size 0x{:X}",
                    replacement.start,
                    replacement.end,
                    body.len()
                ));
            }
            if replacement.start < previous_end {
                return fail(format!(
                    "overlapping MES replacements at body offset 0x{:X}",
                    replacement.start
                ));
            }
            previous_end = replacement.end;
        }

        let mut rebuilt_body = Vec::new();
        let mut source_position = 0usize;
        for replacement in &replacements {
            rebuilt_body.extend_from_slice(&body[source_position..replacement.start as usize]);
            rebuilt_body.extend_from_slice(&replacement.data);
            source_position = replacement.end as usize;
        }
        rebuilt_body.extend_from_slice(&body[source_position..]);
        if rebuilt_body.len() > u32::MAX as usize {
            return fail("rebuilt MES body exceeds u32");
        }

        for reference in &self.references {
            reject_target_inside_replacement(reference.target, &replacements)?;
            let new_field = relocate(reference.field_offset, &replacements)? as usize;
            let new_target = relocate(reference.target, &replacements)?;
            write_u32(&mut rebuilt_body, new_field, new_target)?;
        }

        let mut output = Vec::with_capacity(self.body_start + rebuilt_body.len());
        output.extend_from_slice(&(self.header_offsets.len() as u32).to_le_bytes());
        for old_offset in &self.header_offsets {
            reject_target_inside_replacement(*old_offset, &replacements)?;
            output.extend_from_slice(&relocate(*old_offset, &replacements)?.to_le_bytes());
        }
        output.extend_from_slice(&rebuilt_body);
        Ok(output)
    }

    pub fn message_instruction_ranges(&self) -> Vec<std::ops::Range<usize>> {
        let mut instruction_by_offset = HashMap::with_capacity(self.instructions.len());
        for (index, instruction) in self.instructions.iter().enumerate() {
            instruction_by_offset.insert(instruction.offset, index);
        }
        self.header_offsets
            .iter()
            .enumerate()
            .map(|(message_index, start)| {
                let start_index = instruction_by_offset[start];
                let end_index = self
                    .header_offsets
                    .get(message_index + 1)
                    .map(|next| instruction_by_offset[next])
                    .unwrap_or(self.instructions.len());
                start_index..end_index
            })
            .collect()
    }
}

fn parse_body(body: &[u8]) -> Result<(Vec<Instruction>, Vec<CodeReference>)> {
    let mut cursor = 0usize;
    let mut instructions = Vec::new();
    let mut references = Vec::new();
    while cursor < body.len() {
        let start = cursor;
        let opcode = take_u8(body, &mut cursor, start, "opcode")?;
        let kind = match opcode {
            0x00 | 0x19 | 0x1A => InstructionKind::Other,
            0x01 => {
                let data_start = cursor;
                let data_end = take_cstring(body, &mut cursor, start, "0x01 text")?;
                InstructionKind::Text(TextSlot {
                    data_start: to_u32(data_start)?,
                    data_end: to_u32(data_end)?,
                })
            }
            0x03 => {
                take_bytes(body, &mut cursor, 2, start, "0x03 index")?;
                parse_expression(body, &mut cursor, start)?;
                parse_repeated_expressions(body, &mut cursor, start)?;
                InstructionKind::Other
            }
            0x04 | 0x16 => {
                take_bytes(body, &mut cursor, 1, start, "array selector")?;
                parse_expression(body, &mut cursor, start)?;
                parse_repeated_expressions(body, &mut cursor, start)?;
                InstructionKind::Other
            }
            0x05 | 0x07 | 0x0A => {
                parse_expression(body, &mut cursor, start)?;
                parse_expression(body, &mut cursor, start)?;
                while take_u8(body, &mut cursor, start, "expression-list delimiter")? != 0 {
                    parse_expression(body, &mut cursor, start)?;
                }
                InstructionKind::Other
            }
            0x06 | 0x08 => {
                parse_expression(body, &mut cursor, start)?;
                take_bytes(body, &mut cursor, 1, start, "array selector")?;
                parse_expression(body, &mut cursor, start)?;
                parse_repeated_expressions(body, &mut cursor, start)?;
                InstructionKind::Other
            }
            0x09 => {
                take_bytes(body, &mut cursor, 2, start, "0x09 index")?;
                parse_expression(body, &mut cursor, start)?;
                parse_repeated_expressions(body, &mut cursor, start)?;
                InstructionKind::Other
            }
            0x0B => {
                parse_expression(body, &mut cursor, start)?;
                parse_reference(body, &mut cursor, start, opcode, &mut references)?;
                InstructionKind::Other
            }
            0x0C => {
                parse_reference(body, &mut cursor, start, opcode, &mut references)?;
                InstructionKind::Other
            }
            0x0D => {
                let expression = parse_expression(body, &mut cursor, start)?;
                let arguments = parse_arguments(body, &mut cursor, start)?;
                InstructionKind::Command(Command {
                    command_id: expression.value,
                    arguments,
                })
            }
            0x0E | 0x0F | 0x11 | 0x15 | 0x1B | 0x1D => {
                parse_arguments(body, &mut cursor, start)?;
                InstructionKind::Other
            }
            0x10 => {
                parse_arguments(body, &mut cursor, start)?;
                parse_reference(body, &mut cursor, start, opcode, &mut references)?;
                InstructionKind::Other
            }
            0x13 => {
                take_bytes(body, &mut cursor, 1, start, "0x13 argument")?;
                InstructionKind::Other
            }
            0x14 | 0x1C => {
                parse_expression(body, &mut cursor, start)?;
                parse_reference(body, &mut cursor, start, opcode, &mut references)?;
                InstructionKind::Other
            }
            0x17 => {
                let message_id = take_u32(body, &mut cursor, start, "message id")?;
                InstructionKind::Marker { message_id }
            }
            0x18 => {
                parse_expression(body, &mut cursor, start)?;
                InstructionKind::Other
            }
            0x1F => {
                take_bytes(body, &mut cursor, 4, start, "0x1F payload")?;
                InstructionKind::Other
            }
            _ => {
                take_cstring(body, &mut cursor, start, "opaque display run")?;
                InstructionKind::Other
            }
        };
        if cursor <= start {
            return fail(format!(
                "MES parser did not advance at body offset 0x{start:X}"
            ));
        }
        instructions.push(Instruction {
            offset: to_u32(start)?,
            end: to_u32(cursor)?,
            opcode,
            kind,
        });
    }
    Ok((instructions, references))
}

#[derive(Clone, Copy, Debug)]
struct Expression {
    value: Option<u32>,
}

fn parse_expression(body: &[u8], cursor: &mut usize, inst: usize) -> Result<Expression> {
    let mut single_value = None;
    let mut token_count = 0usize;
    loop {
        let token = take_u8(body, cursor, inst, "expression token")?;
        if token == 0xFF {
            break;
        }
        token_count += 1;
        let value = match token {
            0x00..=0x7F => Some(token as u32),
            0x80 => Some(take_u8(body, cursor, inst, "0x80 operand")? as u32),
            0xA0 | 0xC0 | 0xF8 => {
                take_bytes(body, cursor, 1, inst, "expression operand")?;
                None
            }
            0xF1 | 0xF3 | 0xF6 => {
                let start = *cursor;
                take_bytes(body, cursor, 2, inst, "u16 expression operand")?;
                (token == 0xF1).then(|| u16::from_le_bytes([body[start], body[start + 1]]) as u32)
            }
            0xF2 => {
                let value = take_u32(body, cursor, inst, "u32 expression operand")?;
                Some(value)
            }
            0xF9 => {
                take_bytes(body, cursor, 1, inst, "0xF9 operand")?;
                None
            }
            _ => None,
        };
        if token_count == 1 {
            single_value = value;
        } else {
            single_value = None;
        }
    }
    Ok(Expression {
        value: (token_count == 1).then_some(single_value).flatten(),
    })
}

fn parse_arguments(body: &[u8], cursor: &mut usize, inst: usize) -> Result<Vec<Argument>> {
    let mut arguments = Vec::new();
    loop {
        let argument_type = take_u8(body, cursor, inst, "argument type")?;
        match argument_type {
            0 => break,
            1 => {
                let data_start = *cursor;
                let data_end = take_cstring(body, cursor, inst, "string argument")?;
                arguments.push(Argument::String(TextSlot {
                    data_start: to_u32(data_start)?,
                    data_end: to_u32(data_end)?,
                }));
            }
            2 => {
                let expression = parse_expression(body, cursor, inst)?;
                arguments.push(Argument::Expression {
                    value: expression.value,
                });
            }
            other => {
                return fail(format!(
                    "unknown MES argument type 0x{other:02X} in instruction at body offset 0x{inst:X}"
                ));
            }
        }
    }
    Ok(arguments)
}

fn parse_repeated_expressions(body: &[u8], cursor: &mut usize, inst: usize) -> Result<()> {
    while take_u8(body, cursor, inst, "expression-list delimiter")? != 0 {
        parse_expression(body, cursor, inst)?;
    }
    Ok(())
}

fn parse_reference(
    body: &[u8],
    cursor: &mut usize,
    inst: usize,
    opcode: u8,
    references: &mut Vec<CodeReference>,
) -> Result<()> {
    let field_offset = to_u32(*cursor)?;
    let target = take_u32(body, cursor, inst, "code target")?;
    references.push(CodeReference {
        field_offset,
        target,
        opcode,
    });
    Ok(())
}

fn take_cstring(body: &[u8], cursor: &mut usize, inst: usize, role: &str) -> Result<usize> {
    let start = *cursor;
    let relative = body[start..]
        .iter()
        .position(|&byte| byte == 0)
        .ok_or_else(|| {
            format!("unterminated {role} in MES instruction at body offset 0x{inst:X}")
        })?;
    let end = start + relative;
    *cursor = end + 1;
    Ok(end)
}

fn take_u8(body: &[u8], cursor: &mut usize, inst: usize, role: &str) -> Result<u8> {
    let byte = *body
        .get(*cursor)
        .ok_or_else(|| format!("truncated {role} in MES instruction at body offset 0x{inst:X}"))?;
    *cursor += 1;
    Ok(byte)
}

fn take_u32(body: &[u8], cursor: &mut usize, inst: usize, role: &str) -> Result<u32> {
    let value = read_u32(body, *cursor)
        .map_err(|_| format!("truncated {role} in MES instruction at body offset 0x{inst:X}"))?;
    *cursor += 4;
    Ok(value)
}

fn take_bytes(
    body: &[u8],
    cursor: &mut usize,
    count: usize,
    inst: usize,
    role: &str,
) -> Result<()> {
    let end = cursor
        .checked_add(count)
        .ok_or("MES instruction offset overflow")?;
    if end > body.len() {
        return fail(format!(
            "truncated {role} in MES instruction at body offset 0x{inst:X}"
        ));
    }
    *cursor = end;
    Ok(())
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32> {
    let end = offset.checked_add(4).ok_or("u32 offset overflow")?;
    let bytes = data
        .get(offset..end)
        .ok_or_else(|| format!("truncated u32 at offset 0x{offset:X}"))?;
    Ok(u32::from_le_bytes(
        bytes.try_into().expect("four-byte slice"),
    ))
}

fn write_u32(data: &mut [u8], offset: usize, value: u32) -> Result<()> {
    let end = offset.checked_add(4).ok_or("u32 offset overflow")?;
    let destination = data
        .get_mut(offset..end)
        .ok_or_else(|| format!("rebuilt u32 field at offset 0x{offset:X} is out of range"))?;
    destination.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn to_u32(value: usize) -> Result<u32> {
    u32::try_from(value).map_err(|_| "MES body offset exceeds u32".into())
}

fn relocate(offset: u32, replacements: &[Replacement]) -> Result<u32> {
    let mut relocated = i64::from(offset);
    for replacement in replacements {
        if replacement.end <= offset {
            relocated +=
                replacement.data.len() as i64 - i64::from(replacement.end - replacement.start);
        }
    }
    u32::try_from(relocated)
        .map_err(|_| format!("relocated MES offset is outside u32: {relocated}").into())
}

fn reject_target_inside_replacement(target: u32, replacements: &[Replacement]) -> Result<()> {
    for replacement in replacements {
        if target > replacement.start && target < replacement.end {
            return fail(format!(
                "code target 0x{target:X} points inside replaced text range 0x{:X}..0x{:X}",
                replacement.start, replacement.end
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(&[0x0C]);
        body.extend_from_slice(&5u32.to_le_bytes());
        body.extend_from_slice(&[0x17]);
        body.extend_from_slice(&0u32.to_le_bytes());
        body.extend_from_slice(&[0x01, 0x82, 0xA0, 0x00, 0x00]);
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&5u32.to_le_bytes());
        data.extend_from_slice(&body);
        data
    }

    #[test]
    fn unchanged_roundtrip_is_exact() {
        let source = sample();
        let script = MesScript::parse(&source).unwrap();
        assert_eq!(script.rebuild(Vec::new()).unwrap(), source);
    }

    #[test]
    fn relocation_updates_header_and_jump() {
        let source = sample();
        let script = MesScript::parse(&source).unwrap();
        let rebuilt = script
            .rebuild(vec![Replacement {
                start: 11,
                end: 13,
                data: vec![0x82, 0xA0, 0x82, 0xA2],
            }])
            .unwrap();
        let reparsed = MesScript::parse(&rebuilt).unwrap();
        assert_eq!(reparsed.header_offsets, vec![5]);
        assert_eq!(read_u32(reparsed.body(), 1).unwrap(), 5);
        assert_eq!(reparsed.body().len(), script.body().len() + 2);
    }
}
