use encoding_rs::SHIFT_JIS;
use std::collections::{BTreeMap, HashSet};
use std::fmt;

const HEADER_SIZE: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    pub offsets: Vec<u32>,
    pub code_offset: usize,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub offset: usize,
    pub opcode: u8,
    pub raw: Vec<u8>,
    pub text: Option<Vec<u8>>,
    pub parameters: Vec<Parameter>,
    pub subcommand: Option<u32>,
    pub code_reference: Option<CodeReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeReference {
    pub field_offset: usize,
    pub target: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    pub offset: usize,
    pub raw: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub offset: usize,
    pub kind: u8,
    pub raw: Vec<u8>,
    pub value: ParameterValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterValue {
    String(Vec<u8>),
    Expression(Expression),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptError(String);

impl ScriptError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ScriptError {}

impl Expression {
    pub fn constant(&self) -> Option<u32> {
        let body = self.raw.strip_suffix(&[0xff])?;
        match body {
            [value @ 0x00..=0x7f] => Some(u32::from(*value)),
            [0xf1, low, high] => Some(u32::from(u16::from_le_bytes([*low, *high]))),
            [0xf2, b0, b1, b2, b3] => Some(u32::from_le_bytes([*b0, *b1, *b2, *b3])),
            _ => None,
        }
    }
}

impl Script {
    pub fn parse(bytes: &[u8]) -> Result<Self, ScriptError> {
        let mut cursor = Cursor::new(bytes);
        let count = cursor.read_u32("offset count")? as usize;
        let table_size = count
            .checked_mul(4)
            .and_then(|size| size.checked_add(HEADER_SIZE))
            .ok_or_else(|| ScriptError::new(format!("offset count is too large: {count}")))?;
        if table_size > bytes.len() {
            return Err(ScriptError::new(format!(
                "offset table is truncated: count={count}, table_end=0x{table_size:X}, file_size=0x{:X}",
                bytes.len()
            )));
        }

        let mut offsets = Vec::with_capacity(count);
        for index in 0..count {
            offsets.push(cursor.read_u32(&format!("offset[{index}]"))?);
        }
        let code_offset = cursor.position;
        let code_size = bytes.len() - code_offset;
        for (index, offset) in offsets.iter().enumerate() {
            if *offset as usize >= code_size {
                return Err(ScriptError::new(format!(
                    "relative offset[{index}] is outside code: 0x{offset:X}, code_size=0x{code_size:X}"
                )));
            }
        }

        let mut instructions = Vec::new();
        while cursor.position < bytes.len() {
            instructions.push(parse_instruction(&mut cursor)?);
        }

        let mut valid_targets = HashSet::with_capacity(instructions.len() + 1);
        for instruction in &instructions {
            valid_targets.insert(
                u32::try_from(instruction.offset - code_offset)
                    .map_err(|_| ScriptError::new("script code offset exceeds u32"))?,
            );
        }
        valid_targets.insert(
            u32::try_from(code_size)
                .map_err(|_| ScriptError::new("script code size exceeds u32"))?,
        );
        for (index, offset) in offsets.iter().enumerate() {
            if *offset == code_size as u32 || !valid_targets.contains(offset) {
                return Err(ScriptError::new(format!(
                    "relative offset[{index}] does not point to an instruction boundary: 0x{offset:X}"
                )));
            }
        }
        for instruction in &instructions {
            if let Some(reference) = &instruction.code_reference {
                if !valid_targets.contains(&reference.target) {
                    return Err(ScriptError::new(format!(
                        "opcode 0x{:02X} at 0x{:X} has a non-boundary code target 0x{:X}",
                        instruction.opcode, instruction.offset, reference.target
                    )));
                }
            }
        }

        Ok(Self {
            offsets,
            code_offset,
            instructions,
        })
    }

    pub fn rebuild(&self) -> Result<Vec<u8>, ScriptError> {
        self.rebuild_with_texts(&BTreeMap::new())
    }

    pub fn rebuild_with_texts(
        &self,
        replacements: &BTreeMap<usize, Vec<u8>>,
    ) -> Result<Vec<u8>, ScriptError> {
        for (index, raw) in replacements {
            let instruction = self.instructions.get(*index).ok_or_else(|| {
                ScriptError::new(format!(
                    "text replacement instruction index is invalid: {index}"
                ))
            })?;
            if instruction.opcode != 0x01 {
                return Err(ScriptError::new(format!(
                    "instruction[{index}] at 0x{:X} is opcode 0x{:02X}, not display-text opcode 0x01",
                    instruction.offset, instruction.opcode
                )));
            }
            if raw.contains(&0) {
                return Err(ScriptError::new(format!(
                    "instruction[{index}] replacement contains a NUL byte"
                )));
            }
        }

        let count = u32::try_from(self.offsets.len())
            .map_err(|_| ScriptError::new("too many offset entries to rebuild"))?;
        let mut rebuilt_instructions = Vec::with_capacity(self.instructions.len());
        for (index, instruction) in self.instructions.iter().enumerate() {
            if let Some(text) = replacements.get(&index) {
                let mut raw = Vec::with_capacity(text.len() + 2);
                raw.push(0x01);
                raw.extend_from_slice(text);
                raw.push(0);
                rebuilt_instructions.push(raw);
            } else {
                rebuilt_instructions.push(instruction.raw.clone());
            }
        }

        let instruction_bytes = rebuilt_instructions
            .iter()
            .try_fold(0usize, |total, raw| total.checked_add(raw.len()))
            .ok_or_else(|| ScriptError::new("rebuilt script size overflows usize"))?;
        let old_code_size = self
            .instructions
            .iter()
            .try_fold(0usize, |total, instruction| {
                total.checked_add(instruction.raw.len())
            })
            .ok_or_else(|| ScriptError::new("source script size overflows usize"))?;

        let mut relocation = BTreeMap::new();
        let mut new_relative = 0usize;
        for (instruction, raw) in self.instructions.iter().zip(&rebuilt_instructions) {
            let old_relative = instruction.offset - self.code_offset;
            relocation.insert(
                u32::try_from(old_relative)
                    .map_err(|_| ScriptError::new("source code offset exceeds u32"))?,
                u32::try_from(new_relative)
                    .map_err(|_| ScriptError::new("rebuilt code offset exceeds u32"))?,
            );
            new_relative = new_relative
                .checked_add(raw.len())
                .ok_or_else(|| ScriptError::new("rebuilt script size overflows usize"))?;
        }
        relocation.insert(
            u32::try_from(old_code_size)
                .map_err(|_| ScriptError::new("source code size exceeds u32"))?,
            u32::try_from(instruction_bytes)
                .map_err(|_| ScriptError::new("rebuilt code size exceeds u32"))?,
        );

        for (instruction, raw) in self.instructions.iter().zip(&mut rebuilt_instructions) {
            let Some(reference) = &instruction.code_reference else {
                continue;
            };
            let relocated = relocation.get(&reference.target).ok_or_else(|| {
                ScriptError::new(format!(
                    "cannot relocate opcode 0x{:02X} target 0x{:X} at 0x{:X}",
                    instruction.opcode, reference.target, instruction.offset
                ))
            })?;
            let field = raw
                .get_mut(reference.field_offset..reference.field_offset + 4)
                .ok_or_else(|| {
                    ScriptError::new("code-reference field moved outside instruction")
                })?;
            field.copy_from_slice(&relocated.to_le_bytes());
        }

        let capacity = HEADER_SIZE
            .checked_add(
                self.offsets
                    .len()
                    .checked_mul(4)
                    .ok_or_else(|| ScriptError::new("offset table size overflows usize"))?,
            )
            .and_then(|size| size.checked_add(instruction_bytes))
            .ok_or_else(|| ScriptError::new("rebuilt script size overflows usize"))?;
        let mut output = Vec::with_capacity(capacity);
        output.extend_from_slice(&count.to_le_bytes());
        for offset in &self.offsets {
            let relocated = relocation.get(offset).ok_or_else(|| {
                ScriptError::new(format!("cannot relocate table offset 0x{offset:X}"))
            })?;
            output.extend_from_slice(&relocated.to_le_bytes());
        }
        for raw in rebuilt_instructions {
            output.extend_from_slice(&raw);
        }
        Ok(output)
    }
}

pub fn decode_cp932_exact(raw: &[u8]) -> Result<String, ScriptError> {
    let text = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(raw)
        .ok_or_else(|| ScriptError::new("invalid CP932 byte sequence"))?;
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&text);
    if had_errors || encoded.as_ref() != raw {
        return Err(ScriptError::new("CP932 decode/encode round-trip mismatch"));
    }
    Ok(text.into_owned())
}

pub fn encode_cp932_exact(text: &str) -> Result<Vec<u8>, ScriptError> {
    let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        let unsupported: Vec<_> = text
            .chars()
            .filter(|character| {
                let value = character.to_string();
                let (_, _, failed) = SHIFT_JIS.encode(&value);
                failed
            })
            .collect();
        return Err(ScriptError::new(format!(
            "text contains characters not encodable as CP932: {:?}",
            unsupported
        )));
    }
    let bytes = encoded.into_owned();
    if decode_cp932_exact(&bytes)? != text {
        return Err(ScriptError::new("CP932 encode/decode round-trip mismatch"));
    }
    Ok(bytes)
}

fn parse_instruction(cursor: &mut Cursor<'_>) -> Result<Instruction, ScriptError> {
    let start = cursor.position;
    let opcode = cursor.read_u8("opcode")?;
    let mut text = None;
    let mut parameters = Vec::new();
    let mut subcommand = None;
    let mut code_reference = None;

    match opcode {
        0x01 | 0x02 => {
            text = Some(cursor.read_cstring(&format!("opcode 0x{opcode:02X} text"))?);
        }
        0x03 | 0x09 => {
            cursor.read_bytes(2, &format!("opcode 0x{opcode:02X} destination"))?;
            parse_expression_list(cursor)?;
        }
        0x04 | 0x16 => {
            cursor.read_u8(&format!("opcode 0x{opcode:02X} destination"))?;
            parse_expression_list(cursor)?;
        }
        0x05 | 0x0a => {
            parse_expression(cursor)?;
            parse_expression_list(cursor)?;
        }
        0x06..=0x08 => {
            parse_expression(cursor)?;
            cursor.read_u8(&format!("opcode 0x{opcode:02X} bank"))?;
            parse_expression_list(cursor)?;
        }
        0x0b => {
            parse_expression(cursor)?;
            code_reference = Some(parse_code_reference(cursor, start, "conditional target")?);
        }
        0x0c | 0x10 => {
            code_reference = Some(parse_code_reference(
                cursor,
                start,
                &format!("opcode 0x{opcode:02X} target"),
            )?);
        }
        0x0d => {
            let expression = parse_expression(cursor)?;
            subcommand = expression.constant();
            parameters = parse_parameters(cursor)?;
        }
        0x0e | 0x0f | 0x11 | 0x12 | 0x15 | 0x1b | 0x1d => {
            parameters = parse_parameters(cursor)?;
        }
        0x13 => {
            cursor.read_u8("opcode 0x13 flag")?;
        }
        0x14 | 0x1c => {
            parse_expression(cursor)?;
            code_reference = Some(parse_code_reference(
                cursor,
                start,
                &format!("opcode 0x{opcode:02X} target"),
            )?);
        }
        0x18 => {
            parse_expression(cursor)?;
        }
        0x1e => {
            text = Some(cursor.read_cstring("opcode 0x1E data")?);
        }
        0x17 | 0x1f => {
            cursor.read_bytes(4, &format!("opcode 0x{opcode:02X} data"))?;
        }
        0x00 | 0x19 | 0x1a => {}
        _ => {
            return Err(ScriptError::new(format!(
                "unexpected printable/data byte 0x{opcode:02X} at 0x{start:X}"
            )));
        }
    }

    Ok(Instruction {
        offset: start,
        opcode,
        raw: cursor.data[start..cursor.position].to_vec(),
        text,
        parameters,
        subcommand,
        code_reference,
    })
}

fn parse_code_reference(
    cursor: &mut Cursor<'_>,
    instruction_start: usize,
    field: &str,
) -> Result<CodeReference, ScriptError> {
    let field_offset = cursor.position - instruction_start;
    let target = cursor.read_u32(field)?;
    Ok(CodeReference {
        field_offset,
        target,
    })
}

fn parse_expression(cursor: &mut Cursor<'_>) -> Result<Expression, ScriptError> {
    let start = cursor.position;
    loop {
        let token = cursor.read_u8("expression token")?;
        if token == 0xff {
            break;
        }
        let immediate_size = match token {
            0x80 | 0xa0 | 0xc0 | 0xf8 | 0xf9 => 1,
            0xf1 | 0xf3 | 0xf6 => 2,
            0xf2 => 4,
            _ => 0,
        };
        cursor.read_bytes(immediate_size, &format!("0x{token:02X} expression operand"))?;
    }
    Ok(Expression {
        offset: start,
        raw: cursor.data[start..cursor.position].to_vec(),
    })
}

fn parse_expression_list(cursor: &mut Cursor<'_>) -> Result<(), ScriptError> {
    parse_expression(cursor)?;
    while cursor.read_u8("expression-list separator")? != 0 {
        parse_expression(cursor)?;
    }
    Ok(())
}

fn parse_parameters(cursor: &mut Cursor<'_>) -> Result<Vec<Parameter>, ScriptError> {
    let mut parameters = Vec::new();
    while cursor.peek_u8()? != 0 {
        let start = cursor.position;
        let kind = cursor.read_u8("parameter type")?;
        let value = match kind {
            1 => ParameterValue::String(cursor.read_cstring("string parameter")?),
            2 => ParameterValue::Expression(parse_expression(cursor)?),
            _ => ParameterValue::Unknown,
        };
        parameters.push(Parameter {
            offset: start,
            kind,
            raw: cursor.data[start..cursor.position].to_vec(),
            value,
        });
    }
    cursor.read_u8("parameter-list terminator")?;
    Ok(parameters)
}

struct Cursor<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    fn require(&self, size: usize, field: &str) -> Result<(), ScriptError> {
        let end = self
            .position
            .checked_add(size)
            .ok_or_else(|| ScriptError::new(format!("{field} range overflows usize")))?;
        if end > self.data.len() {
            return Err(ScriptError::new(format!(
                "truncated {field} at 0x{:X}: need {size} bytes, file ends at 0x{:X}",
                self.position,
                self.data.len()
            )));
        }
        Ok(())
    }

    fn peek_u8(&self) -> Result<u8, ScriptError> {
        self.require(1, "byte")?;
        Ok(self.data[self.position])
    }

    fn read_u8(&mut self, field: &str) -> Result<u8, ScriptError> {
        self.require(1, field)?;
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }

    fn read_u32(&mut self, field: &str) -> Result<u32, ScriptError> {
        let raw = self.read_bytes(4, field)?;
        Ok(u32::from_le_bytes(raw.try_into().expect("four-byte slice")))
    }

    fn read_bytes(&mut self, size: usize, field: &str) -> Result<&'a [u8], ScriptError> {
        self.require(size, field)?;
        let start = self.position;
        self.position += size;
        Ok(&self.data[start..self.position])
    }

    fn read_cstring(&mut self, field: &str) -> Result<Vec<u8>, ScriptError> {
        let relative_end = self.data[self.position..]
            .iter()
            .position(|byte| *byte == 0)
            .ok_or_else(|| {
                ScriptError::new(format!("unterminated {field} at 0x{:X}", self.position))
            })?;
        let start = self.position;
        let end = start + relative_end;
        self.position = end + 1;
        Ok(self.data[start..end].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn script_bytes(offsets: &[u32], code: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(offsets.len() as u32).to_le_bytes());
        for offset in offsets {
            bytes.extend_from_slice(&offset.to_le_bytes());
        }
        bytes.extend_from_slice(code);
        bytes
    }

    #[test]
    fn parses_structured_instructions_and_rebuilds_exactly() {
        let code = [
            0x03, 0xea, 0x03, 0x00, 0xff, 0x00, 0x0d, 0x13, 0xff, 0x02, 0x02, 0xff, 0x01, b'a',
            b'b', b'c', 0x00, 0x00, 0x01, b't', b'e', b'x', b't', 0x00, 0x00,
        ];
        let bytes = script_bytes(&[0, 18], &code);
        let script = Script::parse(&bytes).unwrap();
        assert_eq!(script.code_offset, 12);
        assert_eq!(script.instructions.len(), 4);
        assert_eq!(script.instructions[1].subcommand, Some(19));
        assert_eq!(script.instructions[1].parameters.len(), 2);
        assert_eq!(
            script.instructions[2].text.as_deref(),
            Some(b"text".as_slice())
        );
        assert_eq!(script.rebuild().unwrap(), bytes);
    }

    #[test]
    fn decodes_cp932_with_exact_round_trip() {
        let (encoded, _, had_errors) = SHIFT_JIS.encode("\u{ff3b}\u{4e94}\u{6708}\u{ff3d}\u{306f}\u{3058}\u{3081}\u{307e}\u{3057}\u{3066}\u{3002}");
        assert!(!had_errors);
        assert_eq!(
            decode_cp932_exact(encoded.as_ref()).unwrap(),
            "\u{ff3b}\u{4e94}\u{6708}\u{ff3d}\u{306f}\u{3058}\u{3081}\u{307e}\u{3057}\u{3066}\u{3002}"
        );
    }

    #[test]
    fn encodes_cp932_and_rejects_unsupported_characters() {
        assert_eq!(
            encode_cp932_exact("五月").unwrap(),
            [0x8c, 0xdc, 0x8c, 0x8e]
        );
        assert!(encode_cp932_exact("简体中文").is_err());
    }

    #[test]
    fn rejects_truncated_table() {
        let error = Script::parse(&1u32.to_le_bytes()).unwrap_err();
        assert!(error.to_string().contains("table is truncated"));
    }

    #[test]
    fn rejects_out_of_range_relative_offset() {
        let bytes = script_bytes(&[1], &[0x00]);
        let error = Script::parse(&bytes).unwrap_err();
        assert!(error.to_string().contains("outside code"));
    }

    #[test]
    fn rejects_unterminated_expression() {
        let bytes = script_bytes(&[], &[0x18, 0xf1, 0x34]);
        let error = Script::parse(&bytes).unwrap_err();
        assert!(error.to_string().contains("expression operand"));
    }

    #[test]
    fn rejects_unexpected_data_at_instruction_boundary() {
        let bytes = script_bytes(&[], &[0x80]);
        let error = Script::parse(&bytes).unwrap_err();
        assert!(error.to_string().contains("unexpected printable/data byte"));
    }

    #[test]
    fn relocates_table_and_jump_targets_after_text_changes() {
        let code = [0x01, b'a', 0x00, 0x0c, 0x08, 0x00, 0x00, 0x00, 0x00];
        let bytes = script_bytes(&[8], &code);
        let script = Script::parse(&bytes).unwrap();
        let replacements = BTreeMap::from([(0usize, b"longer".to_vec())]);
        let rebuilt = script.rebuild_with_texts(&replacements).unwrap();
        let reparsed = Script::parse(&rebuilt).unwrap();
        assert_eq!(reparsed.offsets, vec![13]);
        assert_eq!(
            reparsed.instructions[1]
                .code_reference
                .as_ref()
                .unwrap()
                .target,
            13
        );
        assert_eq!(
            reparsed.instructions[0].text.as_deref(),
            Some(b"longer".as_slice())
        );
    }
}
