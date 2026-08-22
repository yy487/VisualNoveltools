use crate::{fail, Result};
use std::collections::{HashMap, HashSet, VecDeque};

const FRAME_SIZE: usize = 0x1000;
const FRAME_MASK: usize = FRAME_SIZE - 1;
const FRAME_START: usize = 0xFEE;
pub const RUNTIME_DECODED_CAPACITY: usize = 0x153D8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjStorage {
    Plain,
    Lzss,
}

impl ObjStorage {
    pub fn label(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Lzss => "lzss",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ObjContainer {
    pub stored: Vec<u8>,
    pub storage: ObjStorage,
    pub script: ObjScript,
}

impl ObjContainer {
    pub fn parse(stored: &[u8]) -> Result<Self> {
        if let Ok(script) = ObjScript::parse(stored) {
            return Ok(Self {
                stored: stored.to_vec(),
                storage: ObjStorage::Plain,
                script,
            });
        }
        let decoded = lzss_decompress(stored)?;
        let script = ObjScript::parse(&decoded)
            .map_err(|error| format!("not a plain or LZSS-compressed Mahjong OBJ: {error}"))?;
        Ok(Self {
            stored: stored.to_vec(),
            storage: ObjStorage::Lzss,
            script,
        })
    }

    pub fn store_rebuilt(&self, decoded: Vec<u8>) -> Result<Vec<u8>> {
        if decoded == self.script.original {
            return Ok(self.stored.clone());
        }
        match self.storage {
            ObjStorage::Plain => Ok(decoded),
            ObjStorage::Lzss => {
                let compressed = lzss_compress_greedy(&decoded);
                if lzss_decompress(&compressed)? != decoded {
                    return fail("internal LZSS verification failed");
                }
                Ok(compressed)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ObjScript {
    pub original: Vec<u8>,
    pub instructions: Vec<ObjInstruction>,
    pub references: Vec<ObjReference>,
}

#[derive(Clone, Debug)]
pub struct ObjInstruction {
    pub offset: u32,
    pub end: u32,
    pub opcode: u8,
    pub kind: ObjInstructionKind,
}

#[derive(Clone, Debug)]
pub enum ObjInstructionKind {
    Command(Vec<ObjExpression>),
    Other,
}

#[derive(Clone, Debug)]
pub struct ObjExpression {
    pub tokens: Vec<ObjToken>,
}

#[derive(Clone, Debug)]
pub enum ObjToken {
    Integer(i32),
    String(ObjTextSlot),
    Operator(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjTextSlot {
    pub data_start: u32,
    pub data_end: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct ObjReference {
    pub field_offset: u32,
    pub target: u32,
}

#[derive(Clone, Debug)]
pub struct ObjReplacement {
    pub start: u32,
    pub end: u32,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
enum ConstantValue {
    Integer(i32),
    String(Vec<u8>),
}

impl ObjExpression {
    pub fn constant_integer(&self) -> Option<i32> {
        match evaluate_constant(self)? {
            ConstantValue::Integer(value) => Some(value),
            ConstantValue::String(_) => None,
        }
    }

    pub fn constant_string(&self, script: &ObjScript) -> Option<Vec<u8>> {
        let mut stack = Vec::new();
        for token in &self.tokens {
            match token {
                ObjToken::Integer(value) => stack.push(ConstantValue::Integer(*value)),
                ObjToken::String(slot) => {
                    stack.push(ConstantValue::String(script.text_bytes(slot).to_vec()))
                }
                ObjToken::Operator(0xE0) => {
                    let right = stack.pop()?;
                    let left = stack.pop()?;
                    match (left, right) {
                        (ConstantValue::String(mut left), ConstantValue::String(right)) => {
                            left.extend_from_slice(&right);
                            stack.push(ConstantValue::String(left));
                        }
                        (ConstantValue::Integer(left), ConstantValue::Integer(right)) => {
                            stack.push(ConstantValue::Integer(left.wrapping_add(right)));
                        }
                        _ => return None,
                    }
                }
                ObjToken::Operator(_) => return None,
            }
        }
        match stack.as_slice() {
            [ConstantValue::String(value)] => Some(value.clone()),
            _ => None,
        }
    }

    pub fn direct_string_slot(&self) -> Option<ObjTextSlot> {
        match self.tokens.as_slice() {
            [ObjToken::String(slot)] => Some(*slot),
            _ => None,
        }
    }
}

fn evaluate_constant(expression: &ObjExpression) -> Option<ConstantValue> {
    match expression.tokens.as_slice() {
        [ObjToken::Integer(value)] => Some(ConstantValue::Integer(*value)),
        _ => None,
    }
}

impl ObjScript {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return fail("empty OBJ bytecode");
        }
        let mut cursor = 0usize;
        let mut instructions = Vec::new();
        let mut references = Vec::new();
        while cursor < data.len() {
            let start = cursor;
            let opcode = take_u8(data, &mut cursor, start, "opcode")?;
            let kind = match opcode {
                0x00 | 0x06 => ObjInstructionKind::Other,
                0x01 => {
                    parse_expression(data, &mut cursor, start)?;
                    references.push(parse_reference(data, &mut cursor, start)?);
                    ObjInstructionKind::Other
                }
                0x02 => {
                    references.push(parse_reference(data, &mut cursor, start)?);
                    ObjInstructionKind::Other
                }
                0x03 => {
                    ObjInstructionKind::Command(parse_expression_list(data, &mut cursor, start)?)
                }
                0x04 => {
                    parse_expression(data, &mut cursor, start)?;
                    references.push(parse_reference(data, &mut cursor, start)?);
                    ObjInstructionKind::Other
                }
                0x05 => {
                    parse_expression_list(data, &mut cursor, start)?;
                    ObjInstructionKind::Other
                }
                0x07 | 0x08 | 0x09 => {
                    parse_expression(data, &mut cursor, start)?;
                    parse_expression(data, &mut cursor, start)?;
                    ObjInstructionKind::Other
                }
                _ => ObjInstructionKind::Other,
            };
            instructions.push(ObjInstruction {
                offset: to_u32(start)?,
                end: to_u32(cursor)?,
                opcode,
                kind,
            });
        }

        let boundaries: HashSet<u32> = instructions
            .iter()
            .map(|instruction| instruction.offset)
            .chain(std::iter::once(to_u32(data.len())?))
            .collect();
        for reference in &references {
            if !boundaries.contains(&reference.target) {
                return fail(format!(
                    "OBJ reference at 0x{:X} targets non-instruction boundary 0x{:X}",
                    reference.field_offset, reference.target
                ));
            }
        }
        if instructions.first().map(|instruction| instruction.opcode) != Some(0x04) {
            return fail("OBJ bytecode does not begin with a function definition (opcode 0x04)");
        }
        if references.is_empty() {
            return fail("OBJ bytecode has no validated code references");
        }
        let script = Self {
            original: data.to_vec(),
            instructions,
            references,
        };
        if script.message_slots()?.is_empty() {
            return fail("OBJ bytecode has no direct command-2 message strings");
        }
        Ok(script)
    }

    pub fn text_bytes(&self, slot: &ObjTextSlot) -> &[u8] {
        &self.original[slot.data_start as usize..slot.data_end as usize]
    }

    pub fn message_slots(&self) -> Result<Vec<ObjMessageSlot>> {
        let mut messages = Vec::new();
        let mut pending_voice = None;
        let mut command_two_count = 0usize;
        for instruction in &self.instructions {
            let ObjInstructionKind::Command(arguments) = &instruction.kind else {
                continue;
            };
            let Some(command_id) = arguments.first().and_then(ObjExpression::constant_integer)
            else {
                continue;
            };
            if command_id == 3 {
                pending_voice = arguments
                    .iter()
                    .skip(1)
                    .find_map(|argument| argument.constant_string(self))
                    .filter(|value| value.to_ascii_lowercase().ends_with(b".ogg"));
                continue;
            }
            if command_id != 2 {
                continue;
            }
            command_two_count += 1;
            let message = arguments.get(1).and_then(ObjExpression::direct_string_slot);
            let Some(message) = message else {
                return fail(format!(
                    "command 2 at OBJ offset 0x{:X} does not use one direct string expression",
                    instruction.offset
                ));
            };
            messages.push(ObjMessageSlot {
                instruction_offset: instruction.offset,
                text: message,
                speaker_id: arguments.get(2).and_then(ObjExpression::constant_integer),
                voice: pending_voice.take(),
            });
        }
        if messages.len() != command_two_count {
            return fail("not every command-2 message could be represented structurally");
        }
        Ok(messages)
    }

    pub fn rebuild(&self, mut replacements: Vec<ObjReplacement>) -> Result<Vec<u8>> {
        if replacements.is_empty() {
            return Ok(self.original.clone());
        }
        replacements.sort_by_key(|replacement| replacement.start);
        let mut previous_end = 0u32;
        for replacement in &replacements {
            if replacement.start > replacement.end || replacement.end as usize > self.original.len()
            {
                return fail(format!(
                    "invalid OBJ replacement range 0x{:X}..0x{:X}",
                    replacement.start, replacement.end
                ));
            }
            if replacement.start < previous_end {
                return fail(format!(
                    "overlapping OBJ replacement at 0x{:X}",
                    replacement.start
                ));
            }
            previous_end = replacement.end;
        }
        for reference in &self.references {
            reject_inside_replacement(reference.target, &replacements)?;
        }

        let mut rebuilt = Vec::new();
        let mut source_position = 0usize;
        for replacement in &replacements {
            rebuilt.extend_from_slice(&self.original[source_position..replacement.start as usize]);
            rebuilt.extend_from_slice(&replacement.data);
            source_position = replacement.end as usize;
        }
        rebuilt.extend_from_slice(&self.original[source_position..]);
        if rebuilt.len() > u32::MAX as usize {
            return fail("rebuilt OBJ exceeds u32 address space");
        }
        if rebuilt.len() > RUNTIME_DECODED_CAPACITY {
            return fail(format!(
                "rebuilt OBJ is {} bytes, exceeding the stock runtime decode buffer of {} bytes (0x{:X}); enlarge the executable allocation before injecting this file",
                rebuilt.len(),
                RUNTIME_DECODED_CAPACITY,
                RUNTIME_DECODED_CAPACITY
            ));
        }
        for reference in &self.references {
            let field = relocate(reference.field_offset, &replacements)? as usize;
            let target = relocate(reference.target, &replacements)?;
            write_u32_be(&mut rebuilt, field, target)?;
        }
        ObjScript::parse(&rebuilt)
            .map_err(|error| format!("rebuilt OBJ failed structural validation: {error}"))?;
        Ok(rebuilt)
    }
}

#[derive(Clone, Debug)]
pub struct ObjMessageSlot {
    pub instruction_offset: u32,
    pub text: ObjTextSlot,
    pub speaker_id: Option<i32>,
    pub voice: Option<Vec<u8>>,
}

fn parse_expression_list(
    data: &[u8],
    cursor: &mut usize,
    start: usize,
) -> Result<Vec<ObjExpression>> {
    let mut expressions = Vec::new();
    loop {
        if data.get(*cursor) == Some(&0) {
            *cursor += 1;
            return Ok(expressions);
        }
        if *cursor >= data.len() {
            return fail(format!(
                "truncated expression list in instruction at 0x{start:X}"
            ));
        }
        expressions.push(parse_expression(data, cursor, start)?);
    }
}

fn parse_expression(data: &[u8], cursor: &mut usize, start: usize) -> Result<ObjExpression> {
    let mut tokens = Vec::new();
    loop {
        let token = take_u8(data, cursor, start, "expression token")?;
        match token {
            0xFF => return Ok(ObjExpression { tokens }),
            0x01 => {
                let value = take_u32_be(data, cursor, start, "integer literal")? as i32;
                tokens.push(ObjToken::Integer(value));
            }
            0x02 => {
                let data_start = *cursor;
                let data_end = take_cstring(data, cursor, start, "string literal")?;
                tokens.push(ObjToken::String(ObjTextSlot {
                    data_start: to_u32(data_start)?,
                    data_end: to_u32(data_end)?,
                }));
            }
            operator => tokens.push(ObjToken::Operator(operator)),
        }
    }
}

fn parse_reference(data: &[u8], cursor: &mut usize, start: usize) -> Result<ObjReference> {
    let field_offset = to_u32(*cursor)?;
    let target = take_u32_be(data, cursor, start, "code target")?;
    Ok(ObjReference {
        field_offset,
        target,
    })
}

fn take_u8(data: &[u8], cursor: &mut usize, start: usize, what: &str) -> Result<u8> {
    let value = data.get(*cursor).copied().ok_or_else(|| {
        format!("truncated {what} in instruction beginning at OBJ offset 0x{start:X}")
    })?;
    *cursor += 1;
    Ok(value)
}

fn take_u32_be(data: &[u8], cursor: &mut usize, start: usize, what: &str) -> Result<u32> {
    if data.len().saturating_sub(*cursor) < 4 {
        return fail(format!(
            "truncated {what} in instruction beginning at OBJ offset 0x{start:X}"
        ));
    }
    let value = u32::from_be_bytes(data[*cursor..*cursor + 4].try_into()?);
    *cursor += 4;
    Ok(value)
}

fn take_cstring(data: &[u8], cursor: &mut usize, start: usize, what: &str) -> Result<usize> {
    let relative_end = data[*cursor..]
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| {
            format!("unterminated {what} in instruction beginning at OBJ offset 0x{start:X}")
        })?;
    let end = *cursor + relative_end;
    *cursor = end + 1;
    Ok(end)
}

fn to_u32(value: usize) -> Result<u32> {
    u32::try_from(value).map_err(|_| "OBJ offset exceeds u32".into())
}

fn relocate(offset: u32, replacements: &[ObjReplacement]) -> Result<u32> {
    let mut delta = 0i64;
    for replacement in replacements {
        if replacement.end <= offset {
            delta += replacement.data.len() as i64 - i64::from(replacement.end - replacement.start);
        } else {
            break;
        }
    }
    let relocated = i64::from(offset) + delta;
    u32::try_from(relocated).map_err(|_| "relocated OBJ offset exceeds u32".into())
}

fn reject_inside_replacement(target: u32, replacements: &[ObjReplacement]) -> Result<()> {
    if let Some(replacement) = replacements
        .iter()
        .find(|replacement| target > replacement.start && target < replacement.end)
    {
        return fail(format!(
            "OBJ target 0x{target:X} lands inside replaced text 0x{:X}..0x{:X}",
            replacement.start, replacement.end
        ));
    }
    Ok(())
}

fn write_u32_be(data: &mut [u8], offset: usize, value: u32) -> Result<()> {
    let Some(field) = data.get_mut(offset..offset + 4) else {
        return fail(format!(
            "OBJ target field 0x{offset:X} is outside rebuilt data"
        ));
    };
    field.copy_from_slice(&value.to_be_bytes());
    Ok(())
}

pub fn lzss_decompress(source: &[u8]) -> Result<Vec<u8>> {
    let mut frame = [0u8; FRAME_SIZE];
    let mut frame_position = FRAME_START;
    let mut output = Vec::new();
    let mut cursor = 0usize;
    while cursor < source.len() {
        let control = source[cursor];
        cursor += 1;
        let mut bit = 1u16;
        while bit != 0x100 && cursor < source.len() {
            if control & bit as u8 != 0 {
                let value = source[cursor];
                cursor += 1;
                frame[frame_position & FRAME_MASK] = value;
                frame_position = (frame_position + 1) & FRAME_MASK;
                output.push(value);
            } else {
                if source.len() - cursor < 2 {
                    return fail("truncated LZSS back-reference");
                }
                let low = source[cursor] as usize;
                let high = source[cursor + 1] as usize;
                cursor += 2;
                let mut offset = ((high & 0xF0) << 4) | low;
                let count = 3 + (high & 0x0F);
                for _ in 0..count {
                    let value = frame[offset & FRAME_MASK];
                    offset = (offset + 1) & FRAME_MASK;
                    frame[frame_position & FRAME_MASK] = value;
                    frame_position = (frame_position + 1) & FRAME_MASK;
                    output.push(value);
                }
            }
            bit <<= 1;
        }
    }
    Ok(output)
}

pub fn lzss_compress_greedy(source: &[u8]) -> Vec<u8> {
    if source.len() < 3 {
        return lzss_compress_literal(source);
    }
    let mut positions: HashMap<[u8; 3], VecDeque<usize>> = HashMap::new();
    let mut output = Vec::new();
    let mut cursor = 0usize;
    while cursor < source.len() {
        let control_offset = output.len();
        output.push(0);
        let mut control = 0u8;
        for bit in 0..8 {
            if cursor >= source.len() {
                break;
            }
            let mut best_position = None;
            let mut best_length = 0usize;
            if cursor + 3 <= source.len() {
                let key = [source[cursor], source[cursor + 1], source[cursor + 2]];
                if let Some(candidates) = positions.get(&key) {
                    let minimum = cursor.saturating_sub(FRAME_SIZE);
                    for &candidate in candidates.iter().rev() {
                        if candidate < minimum {
                            continue;
                        }
                        let maximum = 18.min(source.len() - cursor).min(cursor - candidate);
                        if maximum < 3 {
                            continue;
                        }
                        let mut length = 3;
                        while length < maximum
                            && source[candidate + length] == source[cursor + length]
                        {
                            length += 1;
                        }
                        if length > best_length {
                            best_length = length;
                            best_position = Some(candidate);
                            if length == 18 {
                                break;
                            }
                        }
                    }
                }
            }
            if let Some(position) = best_position.filter(|_| best_length >= 3) {
                let offset = (FRAME_START + position) & FRAME_MASK;
                output.push((offset & 0xFF) as u8);
                output.push((((offset >> 4) & 0xF0) | (best_length - 3)) as u8);
                for expanded in cursor..cursor + best_length {
                    add_position(source, expanded, &mut positions);
                }
                cursor += best_length;
            } else {
                control |= 1 << bit;
                output.push(source[cursor]);
                add_position(source, cursor, &mut positions);
                cursor += 1;
            }
        }
        output[control_offset] = control;
    }
    output
}

fn add_position(source: &[u8], position: usize, positions: &mut HashMap<[u8; 3], VecDeque<usize>>) {
    if position + 3 > source.len() {
        return;
    }
    let key = [source[position], source[position + 1], source[position + 2]];
    let values = positions.entry(key).or_default();
    if values.len() == 128 {
        values.pop_front();
    }
    values.push_back(position);
}

fn lzss_compress_literal(source: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(source.len() + source.len().div_ceil(8));
    for chunk in source.chunks(8) {
        output.push(((1u16 << chunk.len()) - 1) as u8);
        output.extend_from_slice(chunk);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expression_integer(value: u32) -> Vec<u8> {
        let mut result = vec![0x01];
        result.extend_from_slice(&value.to_be_bytes());
        result.push(0xFF);
        result
    }

    fn sample(message: &[u8]) -> Vec<u8> {
        let mut data = vec![0x04];
        data.extend_from_slice(&expression_integer(0));
        data.extend_from_slice(&11u32.to_be_bytes());
        data.push(0x03);
        data.extend_from_slice(&expression_integer(2));
        data.push(0x02);
        data.extend_from_slice(message);
        data.extend_from_slice(&[0, 0xFF]);
        data.extend_from_slice(&expression_integer(1));
        data.push(0);
        let goto_offset = data.len();
        data.push(0x02);
        let target_field = data.len();
        data.extend_from_slice(&0u32.to_be_bytes());
        let stop_offset = data.len() as u32;
        data.push(0);
        data[target_field..target_field + 4].copy_from_slice(&stop_offset.to_be_bytes());
        assert_eq!(goto_offset as u32 + 5, stop_offset);
        data
    }

    #[test]
    fn lzss_roundtrip() {
        let source = b"abcabcabcabcabc---0123456789---abcabcabc";
        let compressed = lzss_compress_greedy(source);
        assert_eq!(lzss_decompress(&compressed).unwrap(), source);
    }

    #[test]
    fn parses_and_relocates_message() {
        let source = sample(b"old");
        let script = ObjScript::parse(&source).unwrap();
        let messages = script.message_slots().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(script.text_bytes(&messages[0].text), b"old");
        let old_target = script.references[1].target;
        let rebuilt = script
            .rebuild(vec![ObjReplacement {
                start: messages[0].text.data_start,
                end: messages[0].text.data_end,
                data: b"a longer message".to_vec(),
            }])
            .unwrap();
        let checked = ObjScript::parse(&rebuilt).unwrap();
        assert_eq!(
            checked.text_bytes(&checked.message_slots().unwrap()[0].text),
            b"a longer message"
        );
        assert!(checked.references[1].target > old_target);
    }

    #[test]
    fn rejects_stock_runtime_buffer_overflow() {
        let source = sample(b"short");
        let script = ObjScript::parse(&source).unwrap();
        let slot = script.message_slots().unwrap()[0].text;
        let error = script
            .rebuild(vec![ObjReplacement {
                start: slot.data_start,
                end: slot.data_end,
                data: vec![b'A'; RUNTIME_DECODED_CAPACITY],
            }])
            .unwrap_err()
            .to_string();
        assert!(error.contains("stock runtime decode buffer"));
    }
}
