use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use crate::Result;

pub const TEXT_OPCODE: u8 = 0x58;
pub const MAX_SCRIPT_SIZE: usize = 0x1_0000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    pub range: Range<usize>,
    pub width: usize,
    pub raw: u16,
    pub signed: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    pub range: Range<usize>,
    pub literal: Option<Literal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    RawU16,
    Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub field: Range<usize>,
    pub width: usize,
    pub target: usize,
    pub target_bias: isize,
    pub kind: ReferenceKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextOperand {
    pub instruction_offset: usize,
    pub encoded_range: Range<usize>,
    pub plaintext: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub offset: usize,
    pub end: usize,
    pub opcode: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    pub instructions: Vec<Instruction>,
    pub texts: Vec<TextOperand>,
    pub references: Vec<Reference>,
    pub parsed_end: usize,
}

struct Parser<'a> {
    data: &'a [u8],
    instructions: Vec<Instruction>,
    texts: Vec<TextOperand>,
    references: Vec<Reference>,
}

impl<'a> Parser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            instructions: Vec::new(),
            texts: Vec::new(),
            references: Vec::new(),
        }
    }

    fn require(&self, offset: usize, size: usize, context: &str) -> Result<()> {
        if offset
            .checked_add(size)
            .is_some_and(|end| end <= self.data.len())
        {
            Ok(())
        } else {
            Err(format!(
                "truncated {context} at 0x{offset:04X}: need {size} byte(s), file size is 0x{:04X}",
                self.data.len()
            ))
        }
    }

    fn expression(&self, start: usize) -> Result<Expression> {
        let mut cursor = start;
        let mut token_count = 0usize;
        let mut only_literal = None;

        loop {
            self.require(cursor, 1, "expression token")?;
            let tag = self.data[cursor];
            cursor += 1;
            token_count += 1;

            match tag & 0xBF {
                0x81 | 0x90 | 0xA0 => {
                    self.require(cursor, 2, "expression operand")?;
                    cursor += 2;
                }
                0x84 => {
                    self.require(cursor, 1, "expression operator")?;
                    cursor += 1;
                }
                _ if tag & 2 != 0 => {
                    self.require(cursor, 1, "8-bit expression literal")?;
                    let raw = self.data[cursor] as u16;
                    only_literal = Some(Literal {
                        range: cursor..cursor + 1,
                        width: 1,
                        raw,
                        signed: i8::from_le_bytes([raw as u8]) as i32,
                    });
                    cursor += 1;
                }
                _ => {
                    self.require(cursor, 2, "16-bit expression literal")?;
                    let raw = u16::from_le_bytes([self.data[cursor], self.data[cursor + 1]]);
                    only_literal = Some(Literal {
                        range: cursor..cursor + 2,
                        width: 2,
                        raw,
                        signed: i16::from_le_bytes(raw.to_le_bytes()) as i32,
                    });
                    cursor += 2;
                }
            }

            if tag & 0x40 == 0 {
                break;
            }
        }

        Ok(Expression {
            range: start..cursor,
            literal: (token_count == 1).then_some(only_literal).flatten(),
        })
    }

    fn expressions(&self, mut cursor: usize, count: usize) -> Result<(usize, Vec<Expression>)> {
        let mut expressions = Vec::with_capacity(count);
        for _ in 0..count {
            let expression = self.expression(cursor)?;
            cursor = expression.range.end;
            expressions.push(expression);
        }
        Ok((cursor, expressions))
    }

    fn expression_reference(&mut self, expression: &Expression, label: &str) -> Result<()> {
        let literal = expression.literal.as_ref().ok_or_else(|| {
            format!(
                "{label} at 0x{:04X} is not a direct literal expression; variable-length relocation is unsafe",
                expression.range.start
            )
        })?;
        if literal.raw == u16::MAX {
            return Ok(());
        }
        self.references.push(Reference {
            field: literal.range.clone(),
            width: literal.width,
            target: literal.raw as usize,
            target_bias: 0,
            kind: ReferenceKind::Expression,
        });
        Ok(())
    }

    fn raw_u16_reference(&mut self, field: usize, bias: isize) -> Result<usize> {
        self.require(field, 2, "16-bit script reference")?;
        let raw = u16::from_le_bytes([self.data[field], self.data[field + 1]]) as usize;
        if raw == usize::from(u16::MAX) {
            return Ok(field + 2);
        }
        let target = raw.checked_add_signed(bias).ok_or_else(|| {
            format!("invalid biased target at 0x{field:04X}: raw=0x{raw:04X}, bias={bias}")
        })?;
        self.references.push(Reference {
            field: field..field + 2,
            width: 2,
            target,
            target_bias: bias,
            kind: ReferenceKind::RawU16,
        });
        Ok(field + 2)
    }

    fn encrypted_operand(&self, opcode_offset: usize) -> Result<(usize, Vec<u8>)> {
        let mut key = self.data[opcode_offset];
        let mut cursor = opcode_offset + 1;
        let mut plaintext = Vec::new();

        loop {
            self.require(cursor, 1, "encrypted string")?;
            let byte = self.data[cursor] ^ key;
            cursor += 1;
            if byte == 0 {
                break;
            }
            plaintext.push(byte);
            key = key.wrapping_add(byte);
        }

        Ok((cursor, plaintext))
    }

    fn menu(&mut self, start: usize) -> Result<usize> {
        let mut cursor = start + 1;
        loop {
            self.require(cursor, 1, "menu entry or terminator")?;
            if self.data[cursor] == 0x35 {
                return Ok(cursor + 1);
            }
            let (end, expressions) = self.expressions(cursor, 8)?;
            // sub_403B50 reaches three distinct menu paths by evaluating fields 5, 6, and 7.
            self.expression_reference(&expressions[4], "menu target field 5")?;
            self.expression_reference(&expressions[5], "menu target field 6")?;
            self.expression_reference(&expressions[6], "menu target field 7")?;
            cursor = end;
        }
    }

    fn animation_definition(&self, start: usize) -> Result<usize> {
        let (mut cursor, _) = self.expressions(start + 1, 1)?;
        loop {
            let marker = self.expression(cursor)?;
            cursor = marker.range.end;
            let value = marker.literal.as_ref().map(|literal| literal.signed);
            if value == Some(-1) {
                break;
            }
            if value.is_none() {
                return Err(format!(
                    "animation definition marker at 0x{:04X} is not a literal",
                    marker.range.start
                ));
            }
            cursor = self.expressions(cursor, 4)?.0;
        }
        Ok(self.expressions(cursor, 1)?.0)
    }

    fn parse_instruction(&mut self, start: usize) -> Result<usize> {
        self.require(start, 1, "opcode")?;
        let opcode = self.data[start];

        if opcode & 0x80 != 0 && opcode != 0xFF {
            return Ok(self.expression(start)?.range.end);
        }

        match opcode {
            0x10 => {
                let expression = self.expression(start + 1)?;
                self.raw_u16_reference(expression.range.end, 1)
            }
            0x11 | 0x1E => Ok(start + 1),
            0x12 | 0x1A | 0x1C => {
                let expression = self.expression(start + 1)?;
                self.expression_reference(&expression, "branch target")?;
                Ok(expression.range.end)
            }
            0x14 | 0x15 => {
                let expression = self.expression(start + 1)?;
                self.raw_u16_reference(expression.range.end, 0)
            }
            0x20 => Ok(self.expressions(start + 1, 6)?.0),
            0x22 => Ok(self.expressions(start + 1, 2)?.0),
            0x24 => Ok(self.expressions(start + 1, 10)?.0),
            0x26 => Ok(self.expressions(start + 1, 6)?.0),
            0x28 => Ok(self.expressions(start + 1, 3)?.0),
            0x30 | 0x41 => {
                self.require(start + 1, 3, "four-byte system instruction")?;
                Ok(start + 4)
            }
            0x32 => {
                let (end, expressions) = self.expressions(start + 1, 3)?;
                self.expression_reference(&expressions[0], "input branch target")?;
                self.expression_reference(&expressions[1], "input alternate target")?;
                Ok(end)
            }
            0x34 => self.menu(start),
            0x36 => {
                let (end, expressions) = self.expressions(start + 1, 2)?;
                self.expression_reference(&expressions[0], "menu cancel target")?;
                Ok(end)
            }
            0x3A => Ok(self.expressions(start + 1, 1)?.0),
            0x40 => Ok(self.expressions(start + 1, 2)?.0),
            0x44 => Ok(self.expressions(start + 1, 3)?.0),
            0x4A => Ok(self.encrypted_operand(start)?.0),
            0x4B => Ok(self.expressions(start + 1, 2)?.0),
            0x4C | 0x4D | 0x56 => Ok(self.expressions(start + 1, 1)?.0),
            TEXT_OPCODE => {
                let (end, plaintext) = self.encrypted_operand(start)?;
                self.texts.push(TextOperand {
                    instruction_offset: start,
                    encoded_range: start + 1..end,
                    plaintext,
                });
                Ok(end)
            }
            0x5A => Ok(self.expressions(start + 1, 5)?.0),
            0x5D => Ok(self.expressions(start + 1, 8)?.0),
            0x5E => Ok(self.expressions(start + 1, 9)?.0),
            0x5F => Ok(self.expressions(start + 1, 5)?.0),
            0x60 | 0x64 => self.animation_definition(start),
            0x62 => Ok(self.expressions(start + 1, 1)?.0),
            0x6E => Ok(self.expressions(start + 1, 6)?.0),
            0x70..=0x73 => Ok(self.expressions(start + 1, 2)?.0),
            0xFF => Ok(start + 1),
            _ => Err(format!(
                "unknown ADV opcode 0x{opcode:02X} at 0x{start:04X}"
            )),
        }
    }

    fn parse(mut self) -> Result<Script> {
        if self.data.is_empty() {
            return Err("empty ADV file".to_owned());
        }
        if self.data.len() > MAX_SCRIPT_SIZE {
            return Err(format!(
                "ADV file is {} bytes; KEISEI reads at most {MAX_SCRIPT_SIZE} bytes",
                self.data.len()
            ));
        }

        let mut cursor = 0usize;
        let mut terminators = 0usize;
        while cursor < self.data.len() {
            let opcode = self.data[cursor];
            let end = self.parse_instruction(cursor)?;
            if end <= cursor {
                return Err(format!("parser made no progress at 0x{cursor:04X}"));
            }
            self.instructions.push(Instruction {
                offset: cursor,
                end,
                opcode,
            });
            cursor = end;
            if opcode == 0xFF {
                terminators += 1;
            }
        }

        if terminators == 0 {
            return Err("ADV script has no 0xFF section terminator".to_owned());
        }

        let mut reference_fields = BTreeSet::new();
        let instruction_offsets = self
            .instructions
            .iter()
            .map(|instruction| instruction.offset)
            .collect::<BTreeSet<_>>();
        for reference in &self.references {
            if !reference_fields.insert(reference.field.start) {
                return Err(format!(
                    "duplicate script reference at 0x{:04X}",
                    reference.field.start
                ));
            }
            if reference.target > self.data.len() {
                return Err(format!(
                    "script reference at 0x{:04X} targets 0x{:04X}, beyond file size 0x{:04X}",
                    reference.field.start,
                    reference.target,
                    self.data.len()
                ));
            }
            if reference.target != self.data.len()
                && !instruction_offsets.contains(&reference.target)
            {
                return Err(format!(
                    "script reference at 0x{:04X} targets 0x{:04X}, which is not an instruction boundary",
                    reference.field.start, reference.target
                ));
            }
        }

        Ok(Script {
            instructions: self.instructions,
            texts: self.texts,
            references: self.references,
            parsed_end: cursor,
        })
    }
}

pub fn parse(data: &[u8]) -> Result<Script> {
    Parser::new(data).parse()
}

pub fn encode_encrypted(opcode: u8, plaintext: &[u8]) -> Result<Vec<u8>> {
    if plaintext.contains(&0) {
        return Err("encrypted ADV strings cannot contain NUL".to_owned());
    }
    let mut key = opcode;
    let mut encoded = Vec::with_capacity(plaintext.len() + 1);
    for &byte in plaintext {
        encoded.push(byte ^ key);
        key = key.wrapping_add(byte);
    }
    encoded.push(key);
    Ok(encoded)
}

#[derive(Debug, Clone)]
struct EncodedReplacement {
    old_range: Range<usize>,
    bytes: Vec<u8>,
}

fn map_offset(old: usize, replacements: &[EncodedReplacement]) -> Result<usize> {
    let mut delta = 0isize;
    for replacement in replacements {
        if old < replacement.old_range.start {
            break;
        }
        if old == replacement.old_range.start {
            return old
                .checked_add_signed(delta)
                .ok_or_else(|| format!("offset mapping underflow at 0x{old:04X}"));
        }
        if old < replacement.old_range.end {
            return Err(format!(
                "script offset 0x{old:04X} points inside replaced encrypted text 0x{:04X}..0x{:04X}",
                replacement.old_range.start, replacement.old_range.end
            ));
        }
        delta += replacement.bytes.len() as isize - replacement.old_range.len() as isize;
    }
    old.checked_add_signed(delta)
        .ok_or_else(|| format!("offset mapping underflow at 0x{old:04X}"))
}

pub fn rebuild(
    source: &[u8],
    script: &Script,
    plaintext_by_instruction: &BTreeMap<usize, Vec<u8>>,
) -> Result<Vec<u8>> {
    let known_texts = script
        .texts
        .iter()
        .map(|text| text.instruction_offset)
        .collect::<BTreeSet<_>>();
    for instruction_offset in plaintext_by_instruction.keys() {
        if !known_texts.contains(instruction_offset) {
            return Err(format!(
                "replacement targets non-text instruction 0x{instruction_offset:04X}"
            ));
        }
    }

    let mut replacements = Vec::new();
    for text in &script.texts {
        let Some(plaintext) = plaintext_by_instruction.get(&text.instruction_offset) else {
            continue;
        };
        replacements.push(EncodedReplacement {
            old_range: text.encoded_range.clone(),
            bytes: encode_encrypted(TEXT_OPCODE, plaintext)?,
        });
    }
    replacements.sort_by_key(|replacement| replacement.old_range.start);
    for pair in replacements.windows(2) {
        if pair[0].old_range.end > pair[1].old_range.start {
            return Err(format!(
                "overlapping text ranges 0x{:04X}..0x{:04X} and 0x{:04X}..0x{:04X}",
                pair[0].old_range.start,
                pair[0].old_range.end,
                pair[1].old_range.start,
                pair[1].old_range.end
            ));
        }
    }

    let estimated_size = source.len() as isize
        + replacements
            .iter()
            .map(|replacement| {
                replacement.bytes.len() as isize - replacement.old_range.len() as isize
            })
            .sum::<isize>();
    if !(0..=MAX_SCRIPT_SIZE as isize).contains(&estimated_size) {
        return Err(format!(
            "rebuilt ADV size {estimated_size} exceeds the runtime limit of {MAX_SCRIPT_SIZE} bytes"
        ));
    }

    let mut output = Vec::with_capacity(estimated_size as usize);
    let mut cursor = 0usize;
    for replacement in &replacements {
        output.extend_from_slice(&source[cursor..replacement.old_range.start]);
        output.extend_from_slice(&replacement.bytes);
        cursor = replacement.old_range.end;
    }
    output.extend_from_slice(&source[cursor..]);

    for reference in &script.references {
        let new_field = map_offset(reference.field.start, &replacements)?;
        let new_target = map_offset(reference.target, &replacements)?;
        let stored = new_target
            .checked_add_signed(-reference.target_bias)
            .ok_or_else(|| {
                format!(
                    "relocated target 0x{new_target:04X} cannot apply bias {}",
                    reference.target_bias
                )
            })?;
        match reference.width {
            1 if stored <= u8::MAX as usize => output[new_field] = stored as u8,
            2 if stored <= u16::MAX as usize => {
                output[new_field..new_field + 2].copy_from_slice(&(stored as u16).to_le_bytes());
            }
            1 | 2 => {
                return Err(format!(
                    "relocated target 0x{stored:04X} does not fit the {}-byte field at old offset 0x{:04X}",
                    reference.width,
                    reference.field.start
                ));
            }
            width => return Err(format!("unsupported reference width {width}")),
        }
    }

    let reparsed =
        parse(&output).map_err(|error| format!("rebuilt ADV validation failed: {error}"))?;
    if reparsed.texts.len() != script.texts.len() {
        return Err(format!(
            "rebuilt ADV text count changed from {} to {}",
            script.texts.len(),
            reparsed.texts.len()
        ));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cumulative_xor_round_trip() {
        let plain = b"test\\k\\*";
        let encoded = encode_encrypted(TEXT_OPCODE, plain).unwrap();
        let mut script = vec![TEXT_OPCODE];
        script.extend_from_slice(&encoded);
        script.push(0xFF);
        let parsed = parse(&script).unwrap();
        assert_eq!(parsed.texts.len(), 1);
        assert_eq!(parsed.texts[0].plaintext, plain);
    }

    #[test]
    fn parses_expression_sizes() {
        let parser = Parser::new(&[0x82, 7, 0x80, 0x34, 0x12, 0x84, 15]);
        let one = parser.expression(0).unwrap();
        assert_eq!(one.range, 0..2);
        assert_eq!(one.literal.unwrap().signed, 7);
        let two = parser.expression(2).unwrap();
        assert_eq!(two.range, 2..5);
        assert_eq!(two.literal.unwrap().raw, 0x1234);
        let op = parser.expression(5).unwrap();
        assert_eq!(op.range, 5..7);
        assert!(op.literal.is_none());
    }

    #[test]
    fn relocates_raw_branch_across_longer_text() {
        let encrypted = encode_encrypted(TEXT_OPCODE, b"a").unwrap();
        let target = 5 + 1 + encrypted.len();
        let mut source = vec![0x14, 0x82, 1];
        source.extend_from_slice(&(target as u16).to_le_bytes());
        source.push(TEXT_OPCODE);
        source.extend_from_slice(&encrypted);
        source.extend_from_slice(&[0x11, 0xFF]);

        let script = parse(&source).unwrap();
        let replacements = BTreeMap::from([(5usize, b"long".to_vec())]);
        let output = rebuild(&source, &script, &replacements).unwrap();
        let relocated = u16::from_le_bytes([output[3], output[4]]) as usize;
        assert_eq!(relocated, target + 3);
        assert_eq!(parse(&output).unwrap().texts[0].plaintext, b"long");
    }

    #[test]
    fn relocates_all_menu_targets_across_longer_choice_text() {
        let mut source = vec![0x34];
        for _ in 0..8 {
            source.extend_from_slice(&[0x80, 0, 0]);
        }
        source.push(0x35);
        let text_offset = source.len();
        let encrypted = encode_encrypted(TEXT_OPCODE, b"\\=1;short\\k\\*").unwrap();
        source.push(TEXT_OPCODE);
        source.extend_from_slice(&encrypted);
        let target_5 = source.len();
        source.push(0x11);
        let target_6 = source.len();
        source.push(0x11);
        let target_7 = source.len();
        source.extend_from_slice(&[0x11, 0xFF]);

        source[14..16].copy_from_slice(&(target_5 as u16).to_le_bytes());
        source[17..19].copy_from_slice(&(target_6 as u16).to_le_bytes());
        source[20..22].copy_from_slice(&(target_7 as u16).to_le_bytes());

        let script = parse(&source).unwrap();
        let replacement = b"\\=1;a much longer choice\\k\\*".to_vec();
        let delta = replacement.len() - b"\\=1;short\\k\\*".len();
        let output = rebuild(
            &source,
            &script,
            &BTreeMap::from([(text_offset, replacement.clone())]),
        )
        .unwrap();

        assert_eq!(
            u16::from_le_bytes([output[14], output[15]]) as usize,
            target_5 + delta
        );
        assert_eq!(
            u16::from_le_bytes([output[17], output[18]]) as usize,
            target_6 + delta
        );
        assert_eq!(
            u16::from_le_bytes([output[20], output[21]]) as usize,
            target_7 + delta
        );
        assert_eq!(parse(&output).unwrap().texts[0].plaintext, replacement);
    }

    #[test]
    fn identical_text_is_replaced_by_instruction_offset() {
        let encrypted = encode_encrypted(TEXT_OPCODE, b"same").unwrap();
        let mut source = vec![TEXT_OPCODE];
        source.extend_from_slice(&encrypted);
        let second_offset = source.len();
        source.push(TEXT_OPCODE);
        source.extend_from_slice(&encrypted);
        source.push(0xFF);

        let script = parse(&source).unwrap();
        let output = rebuild(
            &source,
            &script,
            &BTreeMap::from([(second_offset, b"changed".to_vec())]),
        )
        .unwrap();
        let texts = parse(&output).unwrap().texts;
        assert_eq!(texts[0].plaintext, b"same");
        assert_eq!(texts[1].plaintext, b"changed");
    }
}
