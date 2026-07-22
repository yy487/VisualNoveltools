use encoding_rs::SHIFT_JIS;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use thiserror::Error;

pub const SCRIPT_BODY_OFFSET: usize = 0x100;
pub const CONSERVATIVE_SCRIPT_LIMIT: usize = 0xBC00;
pub const MAX_OPCODE: u8 = 0x8E;

const SAVE_MENU_TEXT: &str = "    セーブ    ロードトップメニュー ＤＯＳに戻る";
const SAVE_DONE_TEXT: &str = "セーブしました。";
const SAVE_FILE_TEXTS: [&str; 7] = [
    "ファイル１に",
    "ファイル２に",
    "ファイル３に",
    "ファイル４に",
    "ファイル５に",
    "ファイル６に",
    "ファイル７に",
];
const SIMPLE_SAVE_PREFIX: [u8; 7] = [0x4A, 0x5A, 0x59, 0x60, 0x53, 0x59, 0x54];
const EXTENDED_SAVE_PREFIX: [u8; 8] = [0x4A, 0x5A, 0x59, 0x60, 0x53, 0x59, 0x54, 0x2F];

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("script is shorter than the 0x100-byte prefix: {length} bytes")]
    ScriptTooShort { length: usize },
    #[error("text pointer 0x{offset:04X} is outside the script body (length 0x{length:X})")]
    InvalidTextPointer { offset: usize, length: usize },
    #[error("text stream at 0x{offset:04X} is not NUL-terminated")]
    UnterminatedTextStream { offset: usize },
    #[error("unknown primary text control 0x{code:02X} at 0x{offset:04X}")]
    UnknownPrimaryControl { offset: usize, code: u8 },
    #[error("unknown extended text control 0x0F 0x{selector:02X} at 0x{offset:04X}")]
    UnknownExtendedControl { offset: usize, selector: u8 },
    #[error("truncated text control {code} at 0x{offset:04X}")]
    TruncatedControl { offset: usize, code: String },
    #[error("invalid CP932 bytes at 0x{offset:04X} (length {length})")]
    InvalidCp932 { offset: usize, length: usize },
    #[error("CP932 bytes at 0x{offset:04X} do not round trip exactly (length {length})")]
    Cp932RoundTrip { offset: usize, length: usize },
    #[error("decoded CP932 segment at 0x{offset:04X} contains U+{character:04X}")]
    UnexpectedCharacter { offset: usize, character: u32 },
    #[error("truncated Shift-JIS character at 0x{offset:04X}")]
    TruncatedCharacter { offset: usize },
    #[error("invalid Shift-JIS trail byte 0x{trail:02X} after 0x{lead:02X} at 0x{offset:04X}")]
    InvalidShiftJisPair { offset: usize, lead: u8, trail: u8 },
    #[error("invalid script CFG: {0}")]
    InvalidCfg(String),
    #[error("invalid text patch: {0}")]
    InvalidPatch(String),
    #[error("text cannot be encoded as CP932; unmappable characters: {characters}")]
    UnencodableText { characters: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSegment {
    pub offset: usize,
    pub end_offset: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextControl {
    pub offset: usize,
    pub code: u8,
    pub selector: Option<u8>,
    pub arguments: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameGlyph {
    pub offset: usize,
    pub bytes: Vec<u8>,
    pub jis_code: Option<u16>,
}

impl GameGlyph {
    pub fn gao4_index(&self) -> Option<usize> {
        match self.jis_code? {
            0x7621..=0x767E => Some(usize::from(self.jis_code? - 0x7621)),
            0x7721..=0x777E => Some(94 + usize::from(self.jis_code? - 0x7721)),
            _ => None,
        }
    }
}

impl TextControl {
    pub fn code_string(&self) -> String {
        match self.selector {
            Some(selector) => format!("{:02X} {selector:02X}", self.code),
            None => format!("{:02X}", self.code),
        }
    }

    pub fn encoded(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(2 + self.arguments.len());
        bytes.push(self.code);
        if let Some(selector) = self.selector {
            bytes.push(selector);
        }
        bytes.extend_from_slice(&self.arguments);
        bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextPart {
    Text(TextSegment),
    Control(TextControl),
    Glyph(GameGlyph),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextStream {
    pub offset: usize,
    pub end_offset: usize,
    pub text: String,
    pub parts: Vec<TextPart>,
}

impl TextStream {
    pub fn controls(&self) -> impl Iterator<Item = &TextControl> {
        self.parts.iter().filter_map(|part| match part {
            TextPart::Control(control) => Some(control),
            TextPart::Text(_) | TextPart::Glyph(_) => None,
        })
    }

    pub fn glyphs(&self) -> impl Iterator<Item = &GameGlyph> {
        self.parts.iter().filter_map(|part| match part {
            TextPart::Glyph(glyph) => Some(glyph),
            TextPart::Text(_) | TextPart::Control(_) => None,
        })
    }

    pub fn encoded(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for part in &self.parts {
            match part {
                TextPart::Text(segment) => {
                    let (encoded, _, had_errors) = SHIFT_JIS.encode(&segment.text);
                    debug_assert!(!had_errors);
                    bytes.extend_from_slice(encoded.as_ref());
                }
                TextPart::Control(control) => bytes.extend_from_slice(&control.encoded()),
                TextPart::Glyph(glyph) => bytes.extend_from_slice(&glyph.bytes),
            }
        }
        bytes.push(0);
        bytes
    }
}

pub fn xor_script_body(source: &[u8]) -> Result<Vec<u8>, ScriptError> {
    if source.len() < SCRIPT_BODY_OFFSET {
        return Err(ScriptError::ScriptTooShort {
            length: source.len(),
        });
    }
    let mut result = source.to_vec();
    for byte in &mut result[SCRIPT_BODY_OFFSET..] {
        *byte ^= 0x01;
    }
    Ok(result)
}

fn primary_control_argument_size(code: u8) -> Option<usize> {
    match code {
        0x01 | 0x02 | 0x07 | 0x09 | 0x0A | 0x0D => Some(0),
        0x03 | 0x04 | 0x06 | 0x08 | 0x0B | 0x0C | 0x0E => Some(1),
        0x05 => Some(2),
        _ => None,
    }
}

fn decode_cp932_segment(segment: &[u8], offset: usize) -> Result<String, ScriptError> {
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(segment);
    if had_errors {
        return Err(ScriptError::InvalidCp932 {
            offset,
            length: segment.len(),
        });
    }
    let (encoded, _, encode_errors) = SHIFT_JIS.encode(decoded.as_ref());
    if encode_errors || encoded.as_ref() != segment {
        return Err(ScriptError::Cp932RoundTrip {
            offset,
            length: segment.len(),
        });
    }
    Ok(decoded.into_owned())
}

fn is_shift_jis_lead(byte: u8) -> bool {
    (0x81..=0x9F).contains(&byte) || (0xE0..=0xFC).contains(&byte)
}

fn is_shift_jis_trail(byte: u8) -> bool {
    (0x40..=0x7E).contains(&byte) || (0x80..=0xFC).contains(&byte)
}

pub fn shift_jis_pair_to_jis(lead: u8, trail: u8) -> Option<u16> {
    if !is_shift_jis_lead(lead) || !is_shift_jis_trail(trail) {
        return None;
    }
    let lead_base = if lead <= 0x9F { 0x81 } else { 0xC1 };
    let mut row = u16::from(lead - lead_base) * 2 + 0x21;
    let cell = if trail < 0x9F {
        u16::from(trail - if trail < 0x7F { 0x1F } else { 0x20 })
    } else {
        row += 1;
        u16::from(trail - 0x7E)
    };
    Some((row << 8) | cell)
}

fn is_private_use(character: char) -> bool {
    matches!(character as u32, 0xE000..=0xF8FF | 0xF0000..=0xFFFFD | 0x100000..=0x10FFFD)
}

enum TokenKind {
    Text(usize),
    Glyph(usize, Option<u16>),
}

fn classify_text_token(data: &[u8], offset: usize) -> Result<TokenKind, ScriptError> {
    let byte = data[offset];
    if is_shift_jis_lead(byte) {
        let trail = *data
            .get(offset + 1)
            .ok_or(ScriptError::TruncatedCharacter { offset })?;
        let jis_code =
            shift_jis_pair_to_jis(byte, trail).ok_or(ScriptError::InvalidShiftJisPair {
                offset,
                lead: byte,
                trail,
            })?;
        let token = &data[offset..offset + 2];
        let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(token);
        if had_errors || decoded.chars().any(is_private_use) {
            return Ok(TokenKind::Glyph(2, Some(jis_code)));
        }
        let (encoded, _, encode_errors) = SHIFT_JIS.encode(decoded.as_ref());
        if encode_errors || encoded.as_ref() != token {
            return Ok(TokenKind::Glyph(2, Some(jis_code)));
        }
        return Ok(TokenKind::Text(2));
    }

    if byte < 0x20 || byte == 0x7F || byte == 0x80 || byte == 0xA0 || byte >= 0xFD {
        return Ok(TokenKind::Glyph(1, None));
    }
    Ok(TokenKind::Text(1))
}

fn push_text_segment(
    data: &[u8],
    start: usize,
    end: usize,
    text: &mut String,
    parts: &mut Vec<TextPart>,
) -> Result<(), ScriptError> {
    if start == end {
        return Ok(());
    }
    let decoded = decode_cp932_segment(&data[start..end], start)?;
    text.push_str(&decoded);
    parts.push(TextPart::Text(TextSegment {
        offset: start,
        end_offset: end,
        text: decoded,
    }));
    Ok(())
}

pub fn parse_text_stream(data: &[u8], offset: usize) -> Result<TextStream, ScriptError> {
    if !(SCRIPT_BODY_OFFSET..data.len()).contains(&offset) {
        return Err(ScriptError::InvalidTextPointer {
            offset,
            length: data.len(),
        });
    }

    let mut cursor = offset;
    let mut segment_start = offset;
    let mut text = String::new();
    let mut parts = Vec::new();

    while cursor < data.len() {
        let value = data[cursor];
        if value >= 0x10 {
            match classify_text_token(data, cursor)? {
                TokenKind::Text(length) => cursor += length,
                TokenKind::Glyph(length, jis_code) => {
                    push_text_segment(data, segment_start, cursor, &mut text, &mut parts)?;
                    parts.push(TextPart::Glyph(GameGlyph {
                        offset: cursor,
                        bytes: data[cursor..cursor + length].to_vec(),
                        jis_code,
                    }));
                    cursor += length;
                    segment_start = cursor;
                }
            }
            continue;
        }

        push_text_segment(data, segment_start, cursor, &mut text, &mut parts)?;

        if value == 0 {
            return Ok(TextStream {
                offset,
                end_offset: cursor + 1,
                text,
                parts,
            });
        }

        let control_offset = cursor;
        cursor += 1;
        let (selector, argument_size, code_string) = if value == 0x0F {
            if cursor >= data.len() {
                return Err(ScriptError::TruncatedControl {
                    offset: control_offset,
                    code: "0F".to_owned(),
                });
            }
            let selector = data[cursor];
            cursor += 1;
            let argument_size = match selector {
                0x00 => 1,
                _ => {
                    return Err(ScriptError::UnknownExtendedControl {
                        offset: control_offset,
                        selector,
                    });
                }
            };
            (Some(selector), argument_size, format!("0F {selector:02X}"))
        } else {
            let argument_size =
                primary_control_argument_size(value).ok_or(ScriptError::UnknownPrimaryControl {
                    offset: control_offset,
                    code: value,
                })?;
            (None, argument_size, format!("{value:02X}"))
        };

        if cursor + argument_size > data.len() {
            return Err(ScriptError::TruncatedControl {
                offset: control_offset,
                code: code_string,
            });
        }
        let arguments = data[cursor..cursor + argument_size].to_vec();
        cursor += argument_size;
        parts.push(TextPart::Control(TextControl {
            offset: control_offset,
            code: value,
            selector,
            arguments,
        }));
        segment_start = cursor;
    }

    Err(ScriptError::UnterminatedTextStream { offset })
}

pub fn is_localizable_text(stream: &TextStream) -> bool {
    !stream.text.trim().is_empty()
        && stream
            .text
            .chars()
            .all(|character| !character.is_control() || matches!(character, '\r' | '\n' | '\t'))
}

fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
    let bytes = data.get(offset..offset + 2)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn effective_opcode(raw_opcode: u16) -> u8 {
    if raw_opcode < 0x100 {
        raw_opcode as u8
    } else {
        0
    }
}

pub fn fixed_instruction_size(opcode: u8) -> Option<usize> {
    match opcode {
        0x02 | 0x03 => Some(14),
        0x04
        | 0x05
        | 0x09
        | 0x0D..=0x12
        | 0x1A
        | 0x1D
        | 0x1E
        | 0x1F
        | 0x21
        | 0x22
        | 0x33
        | 0x34
        | 0x42
        | 0x4E
        | 0x4F
        | 0x54
        | 0x5A
        | 0x72
        | 0x73
        | 0x75
        | 0x7A
        | 0x7F
        | 0x83 => Some(4),
        0x06 | 0x1B | 0x1C | 0x20 | 0x36 | 0x55 | 0x76 | 0x77 | 0x82 | 0x8E => Some(2),
        0x07
        | 0x08
        | 0x0B
        | 0x0C
        | 0x15
        | 0x19
        | 0x27
        | 0x28
        | 0x2D
        | 0x38
        | 0x39
        | 0x3D..=0x3F
        | 0x43
        | 0x4A..=0x4C
        | 0x51..=0x53
        | 0x56
        | 0x59
        | 0x5B
        | 0x62
        | 0x68
        | 0x6E
        | 0x74
        | 0x78
        | 0x7B
        | 0x81 => Some(6),
        0x16 | 0x57 | 0x58 | 0x60 => Some(8),
        0x14 | 0x40 | 0x41 | 0x6A | 0x6B => Some(10),
        0x26 | 0x3C | 0x5E => Some(12),
        0x2F | 0x3A | 0x3B | 0x63 | 0x64 => Some(18),
        _ => None,
    }
}

pub fn instruction_size(data: &[u8], offset: usize, opcode: u8) -> Option<usize> {
    if matches!(opcode, 0x18 | 0x24 | 0x25) {
        let first_operand = read_u16(data, offset + 2)?;
        Some(if first_operand >= 0x400 { 8 } else { 6 })
    } else {
        fixed_instruction_size(opcode)
    }
}

fn is_default_terminator(opcode: u8) -> bool {
    matches!(
        opcode,
        0x00
            | 0x01
            | 0x17
            | 0x23
            | 0x2B
            | 0x2C
            | 0x2E
            | 0x30
            | 0x31
            | 0x44..=0x49
            | 0x5C
            | 0x5D
            | 0x65
            | 0x66
            | 0x6C
            | 0x6D
            | 0x6F..=0x71
            | 0x7C..=0x7E
    )
}

pub fn instruction_successors(instruction: &Instruction) -> Vec<usize> {
    if is_default_terminator(instruction.opcode) || matches!(instruction.opcode, 0x1F | 0x55) {
        return Vec::new();
    }
    if matches!(instruction.opcode, 0x0A | 0x13) {
        return instruction
            .targets
            .iter()
            .map(|target| usize::from(*target))
            .collect();
    }
    let mut successors = vec![instruction.offset + instruction.size];
    successors.extend(
        instruction
            .targets
            .iter()
            .map(|target| usize::from(*target)),
    );
    successors.sort_unstable();
    successors.dedup();
    successors
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CfgWarningKind {
    InvalidRoot,
    InvalidTarget,
    Truncated,
    UnknownOpcode,
    JumpTableLimit,
    JumpTableEmpty,
    InstructionOverlap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CfgWarning {
    pub offset: usize,
    pub kind: CfgWarningKind,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub offset: usize,
    pub raw_opcode: u16,
    pub opcode: u8,
    pub size: usize,
    pub roots: Vec<u16>,
    pub targets: Vec<u16>,
}

#[derive(Debug, Clone, Default)]
pub struct Cfg {
    pub instructions: BTreeMap<usize, Instruction>,
    pub warnings: Vec<CfgWarning>,
}

fn parse_jump_table(data: &[u8], offset: usize) -> (usize, Vec<u16>, Vec<CfgWarning>) {
    let table_start = offset + 4;
    let mut table_cursor = table_start;
    let mut boundary = data.len();
    let mut targets = Vec::new();
    let mut warnings = Vec::new();
    let mut found_forward_target = false;
    let mut hit_limit = true;

    for _ in 0..256 {
        if table_cursor + 1 >= data.len() || table_cursor >= boundary {
            hit_limit = false;
            break;
        }
        let target = read_u16(data, table_cursor).expect("bounds checked");
        if found_forward_target && usize::from(target) < table_start {
            hit_limit = false;
            break;
        }
        table_cursor += 2;
        if (SCRIPT_BODY_OFFSET..data.len()).contains(&usize::from(target)) {
            targets.push(target);
            if usize::from(target) >= table_start {
                found_forward_target = true;
                boundary = boundary.min(usize::from(target));
            }
        }
    }
    if hit_limit {
        warnings.push(CfgWarning {
            offset,
            kind: CfgWarningKind::JumpTableLimit,
            detail: "more than 256 entries".to_owned(),
        });
    }
    if targets.is_empty() {
        warnings.push(CfgWarning {
            offset,
            kind: CfgWarningKind::JumpTableEmpty,
            detail: "no in-body targets".to_owned(),
        });
    }
    targets.sort_unstable();
    targets.dedup();
    (table_cursor - offset, targets, warnings)
}

fn add_target(
    queue: &mut VecDeque<usize>,
    warnings: &mut Vec<CfgWarning>,
    data: &[u8],
    source: usize,
    target: usize,
    kind: &str,
) {
    if (SCRIPT_BODY_OFFSET..data.len()).contains(&target) {
        queue.push_back(target);
    } else {
        warnings.push(CfgWarning {
            offset: source,
            kind: CfgWarningKind::InvalidTarget,
            detail: format!("{kind} target 0x{target:04X}"),
        });
    }
}

pub fn build_cfg(data: &[u8]) -> Cfg {
    let mut cfg = Cfg::default();
    let mut queue = VecDeque::from([SCRIPT_BODY_OFFSET]);

    while let Some(offset) = queue.pop_front() {
        if cfg.instructions.contains_key(&offset) {
            continue;
        }
        let Some(raw_opcode) = read_u16(data, offset) else {
            cfg.warnings.push(CfgWarning {
                offset,
                kind: CfgWarningKind::InvalidRoot,
                detail: "outside script body".to_owned(),
            });
            continue;
        };
        if offset < SCRIPT_BODY_OFFSET {
            cfg.warnings.push(CfgWarning {
                offset,
                kind: CfgWarningKind::InvalidRoot,
                detail: "outside script body".to_owned(),
            });
            continue;
        }

        let opcode = effective_opcode(raw_opcode);
        let mut roots = Vec::new();
        let mut targets = Vec::new();
        let size;

        if opcode == 0x0A {
            let Some(target) = read_u16(data, offset + 2) else {
                cfg.warnings.push(CfgWarning {
                    offset,
                    kind: CfgWarningKind::Truncated,
                    detail: "opcode 0x0A".to_owned(),
                });
                continue;
            };
            size = 4;
            targets.push(target);
        } else if opcode == 0x13 {
            let (table_size, table_targets, table_warnings) = parse_jump_table(data, offset);
            size = table_size;
            targets = table_targets;
            cfg.warnings.extend(table_warnings);
        } else if is_default_terminator(opcode) {
            size = 2;
        } else {
            let Some(opcode_size) = instruction_size(data, offset, opcode) else {
                cfg.warnings.push(CfgWarning {
                    offset,
                    kind: CfgWarningKind::UnknownOpcode,
                    detail: format!("raw=0x{raw_opcode:04X}"),
                });
                continue;
            };
            size = opcode_size;
            if offset + size > data.len() {
                cfg.warnings.push(CfgWarning {
                    offset,
                    kind: CfgWarningKind::Truncated,
                    detail: format!("opcode 0x{opcode:02X} size {size}"),
                });
                continue;
            }

            if (0x0D..=0x12).contains(&opcode) {
                targets.push(read_u16(data, offset + 2).expect("instruction bounds checked"));
            } else if matches!(opcode, 0x74 | 0x7B) {
                targets.push(read_u16(data, offset + 4).expect("instruction bounds checked"));
            } else if opcode == 0x54 {
                roots.push(read_u16(data, offset + 2).expect("instruction bounds checked"));
            } else if opcode == 0x40 {
                roots.push(read_u16(data, offset + 6).expect("instruction bounds checked"));
                roots.push(read_u16(data, offset + 8).expect("instruction bounds checked"));
            } else if opcode == 0x41 {
                roots.push(read_u16(data, offset + 8).expect("instruction bounds checked"));
            }
        }

        cfg.instructions.insert(
            offset,
            Instruction {
                offset,
                raw_opcode,
                opcode,
                size,
                roots: roots.clone(),
                targets: targets.clone(),
            },
        );

        for target in targets {
            add_target(
                &mut queue,
                &mut cfg.warnings,
                data,
                offset,
                usize::from(target),
                "branch",
            );
        }
        for root in roots {
            add_target(
                &mut queue,
                &mut cfg.warnings,
                data,
                offset,
                usize::from(root),
                "callback",
            );
        }

        let no_fallthrough =
            is_default_terminator(opcode) || matches!(opcode, 0x0A | 0x13 | 0x1F | 0x55);
        if !no_fallthrough {
            add_target(
                &mut queue,
                &mut cfg.warnings,
                data,
                offset,
                offset + size,
                "fallthrough",
            );
        }
    }

    let mut occupied = HashMap::new();
    for instruction in cfg.instructions.values() {
        for byte_offset in instruction.offset..instruction.offset + instruction.size {
            let previous = occupied.entry(byte_offset).or_insert(instruction.offset);
            if *previous != instruction.offset {
                cfg.warnings.push(CfgWarning {
                    offset: instruction.offset,
                    kind: CfgWarningKind::InstructionOverlap,
                    detail: format!("overlaps instruction at 0x{:04X}", *previous),
                });
                break;
            }
        }
    }

    cfg
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextCandidate {
    pub instruction_offset: usize,
    pub opcode: u8,
    pub operands: Vec<u16>,
    pub stream: TextStream,
}

pub fn cfg_text_candidates(data: &[u8], cfg: &Cfg) -> Result<Vec<TextCandidate>, ScriptError> {
    let mut candidates = Vec::new();
    for instruction in cfg.instructions.values() {
        let (operands, text_offset) = match instruction.opcode {
            0x15 => {
                let operands = vec![
                    read_u16(data, instruction.offset + 2).expect("instruction bounds checked"),
                    read_u16(data, instruction.offset + 4).expect("instruction bounds checked"),
                ];
                let text_offset = usize::from(operands[1]);
                (operands, text_offset)
            }
            0x16 => {
                let operands = vec![
                    read_u16(data, instruction.offset + 2).expect("instruction bounds checked"),
                    read_u16(data, instruction.offset + 4).expect("instruction bounds checked"),
                    read_u16(data, instruction.offset + 6).expect("instruction bounds checked"),
                ];
                let text_offset = usize::from(operands[2]);
                (operands, text_offset)
            }
            _ => continue,
        };
        let stream = parse_text_stream(data, text_offset)?;
        if is_localizable_text(&stream) {
            candidates.push(TextCandidate {
                instruction_offset: instruction.offset,
                opcode: instruction.opcode,
                operands,
                stream,
            });
        }
    }
    Ok(candidates)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SaveBlockVariant {
    Simple,
    Extended,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveTemplate {
    pub menu_instruction: usize,
    pub switch_instruction: usize,
    pub block_variant: SaveBlockVariant,
    pub block_starts: Vec<usize>,
    pub text_instructions: Vec<usize>,
    pub continuation_target: usize,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("invalid compiler save template: {detail}")]
pub struct SaveTemplateError {
    pub detail: String,
}

fn save_error(detail: impl Into<String>) -> SaveTemplateError {
    SaveTemplateError {
        detail: detail.into(),
    }
}

fn opcode_at(data: &[u8], offset: usize) -> Option<u8> {
    read_u16(data, offset).map(effective_opcode)
}

fn candidate_text(candidate: &TextCandidate) -> &str {
    &candidate.stream.text
}

pub fn validate_reachable_save_template(
    data: &[u8],
    cfg: &Cfg,
    candidates: &[TextCandidate],
) -> Result<Option<SaveTemplate>, SaveTemplateError> {
    let menu: Vec<_> = candidates
        .iter()
        .filter(|candidate| candidate_text(candidate) == SAVE_MENU_TEXT)
        .collect();
    let done: Vec<_> = candidates
        .iter()
        .filter(|candidate| candidate_text(candidate) == SAVE_DONE_TEXT)
        .collect();
    let labels: Vec<Vec<&TextCandidate>> = SAVE_FILE_TEXTS
        .iter()
        .map(|text| {
            candidates
                .iter()
                .filter(|candidate| candidate_text(candidate) == *text)
                .collect()
        })
        .collect();
    let related_count = menu.len() + done.len() + labels.iter().map(Vec::len).sum::<usize>();
    if related_count == 0 {
        return Ok(None);
    }
    if menu.len() != 1 || done.len() != 7 || labels.iter().any(|items| items.len() != 1) {
        return Err(save_error(format!(
            "text cardinality mismatch: menu={}, done={}, labels={:?}",
            menu.len(),
            done.len(),
            labels.iter().map(Vec::len).collect::<Vec<_>>()
        )));
    }

    let menu_candidate = menu[0];
    if menu_candidate.operands.first().copied() != Some(8) {
        return Err(save_error("save menu channel is not 8"));
    }
    let setup_start = menu_candidate
        .instruction_offset
        .checked_sub(fixed_instruction_size(0x03).expect("known opcode"))
        .ok_or_else(|| save_error("save menu is before setup"))?;
    let setup_expected = [
        (setup_start, 0x03),
        (menu_candidate.instruction_offset, 0x15),
        (menu_candidate.instruction_offset + 6, 0x6A),
        (menu_candidate.instruction_offset + 16, 0x3F),
        (menu_candidate.instruction_offset + 22, 0x40),
    ];
    for (offset, expected) in setup_expected {
        if opcode_at(data, offset) != Some(expected) {
            return Err(save_error(format!(
                "setup opcode mismatch at 0x{offset:04X}: {:?} != 0x{expected:02X}",
                opcode_at(data, offset)
            )));
        }
    }

    let done_by_offset: BTreeMap<_, _> = done
        .iter()
        .map(|candidate| (candidate.instruction_offset, *candidate))
        .collect();
    let mut block_starts = Vec::new();
    let mut variants = BTreeSet::new();
    let mut text_instructions = vec![menu_candidate.instruction_offset];
    let mut branch_continuation_targets = BTreeSet::new();

    for (block_index, label_items) in labels.iter().enumerate() {
        let label = label_items[0];
        let mut matches = Vec::new();
        for (variant, prefix) in [
            (SaveBlockVariant::Simple, SIMPLE_SAVE_PREFIX.as_slice()),
            (SaveBlockVariant::Extended, EXTENDED_SAVE_PREFIX.as_slice()),
        ] {
            let prefix_size: usize = prefix
                .iter()
                .map(|opcode| fixed_instruction_size(*opcode).expect("known save opcode"))
                .sum();
            let Some(candidate_start) = label.instruction_offset.checked_sub(prefix_size) else {
                continue;
            };
            let mut cursor = candidate_start;
            let valid = prefix.iter().all(|opcode| {
                if opcode_at(data, cursor) != Some(*opcode) {
                    return false;
                }
                cursor += fixed_instruction_size(*opcode).expect("known save opcode");
                true
            });
            if valid && cursor == label.instruction_offset {
                matches.push((variant, candidate_start));
            }
        }
        if matches.len() != 1 {
            return Err(save_error(format!(
                "label at 0x{:04X} matches {} block variants",
                label.instruction_offset,
                matches.len()
            )));
        }
        let (variant, block_start) = matches[0];
        variants.insert(variant);
        block_starts.push(block_start);

        let done_candidate = done_by_offset
            .get(&(label.instruction_offset + 6))
            .ok_or_else(|| {
                save_error(format!(
                    "label at 0x{:04X} lacks adjacent done text",
                    label.instruction_offset
                ))
            })?;
        if label.operands.first().copied() != Some(9)
            || done_candidate.operands.first().copied() != Some(9)
        {
            return Err(save_error("save file/done text channel is not 9"));
        }
        let cursor = label.instruction_offset + 12;
        if block_index < 6 {
            if opcode_at(data, cursor) != Some(0x0A) {
                return Err(save_error(format!(
                    "save block at 0x{block_start:04X} lacks continuation jump"
                )));
            }
            let target = read_u16(data, cursor + 2)
                .ok_or_else(|| save_error("truncated save continuation jump"))?;
            branch_continuation_targets.insert(usize::from(target));
        }
        text_instructions.extend([label.instruction_offset, done_candidate.instruction_offset]);
    }

    if variants.len() != 1 {
        return Err(save_error("save blocks mix variants"));
    }
    let block_variant = *variants.iter().next().expect("one variant");
    let continuation_target = labels[6][0].instruction_offset + 12;
    if branch_continuation_targets != BTreeSet::from([continuation_target]) {
        return Err(save_error(format!(
            "first six save blocks do not share tail 0x{continuation_target:04X}: {:?}",
            branch_continuation_targets
        )));
    }

    let (tail_expected, tail_jump_operand): (&[(usize, u8)], usize) = match block_variant {
        SaveBlockVariant::Simple => (
            &[
                (continuation_target, 0x04),
                (continuation_target + 4, 0x18),
                (continuation_target + 10, 0x0A),
            ],
            continuation_target + 12,
        ),
        SaveBlockVariant::Extended => (
            &[(continuation_target, 0x54), (continuation_target + 4, 0x0A)],
            continuation_target + 6,
        ),
    };
    for (offset, expected) in tail_expected {
        if opcode_at(data, *offset) != Some(*expected) {
            return Err(save_error(format!(
                "shared tail opcode mismatch at 0x{offset:04X}, expected 0x{expected:02X}"
            )));
        }
    }
    if read_u16(data, tail_jump_operand).map(usize::from) != Some(setup_start) {
        return Err(save_error(
            "save shared tail does not jump back to menu setup",
        ));
    }

    let expected_blocks: BTreeSet<_> = block_starts.iter().copied().collect();
    let switch_matches: Vec<_> = cfg
        .instructions
        .values()
        .filter(|instruction| {
            instruction.opcode == 0x13
                && instruction.offset < block_starts[0]
                && instruction
                    .targets
                    .iter()
                    .copied()
                    .map(usize::from)
                    .collect::<BTreeSet<_>>()
                    == expected_blocks
        })
        .collect();
    if switch_matches.len() != 1 {
        return Err(save_error(format!(
            "expected one reachable seven-way switch, found {}",
            switch_matches.len()
        )));
    }

    text_instructions.sort_unstable();
    Ok(Some(SaveTemplate {
        menu_instruction: menu_candidate.instruction_offset,
        switch_instruction: switch_matches[0].offset,
        block_variant,
        block_starts,
        text_instructions,
        continuation_target,
    }))
}

pub fn is_main_story_script_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() == 9
        && bytes[0].eq_ignore_ascii_case(&b'c')
        && bytes[1].eq_ignore_ascii_case(&b's')
        && bytes[2].is_ascii_digit()
        && bytes[3].is_ascii_digit()
        && bytes[4] == b'_'
        && bytes[5].is_ascii_digit()
        && bytes[6].is_ascii_digit()
        && bytes[7] == b'.'
        && bytes[8].eq_ignore_ascii_case(&b's')
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptTextPatch {
    pub instruction_offset: usize,
    pub expected_text_offset: usize,
    pub expected_size: usize,
    pub source_parts: Vec<String>,
    pub replacement_parts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptStreamPatch {
    pub instruction_offset: usize,
    pub expected_text_offset: usize,
    pub expected_size: usize,
    pub expected_stream: Vec<u8>,
    pub replacement_stream: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScriptPatchStats {
    pub requested: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub in_place: usize,
    pub relocated_entries: usize,
    pub appended_streams: usize,
    pub appended_bytes: usize,
    pub output_bytes: usize,
    pub byte_exact: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchedScript {
    pub bytes: Vec<u8>,
    pub stats: ScriptPatchStats,
}

pub fn text_parts(stream: &TextStream) -> Vec<&str> {
    stream
        .parts
        .iter()
        .filter_map(|part| match part {
            TextPart::Text(segment) => Some(segment.text.as_str()),
            TextPart::Control(_) | TextPart::Glyph(_) => None,
        })
        .collect()
}

pub fn encode_cp932_text(text: &str) -> Result<Vec<u8>, ScriptError> {
    if let Some(character) = text.chars().find(|character| character.is_control()) {
        return Err(ScriptError::InvalidPatch(format!(
            "translated text contains control character U+{:04X}",
            character as u32
        )));
    }
    let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        let mut unmappable = BTreeSet::new();
        for character in text.chars() {
            let value = character.to_string();
            let (_, _, character_error) = SHIFT_JIS.encode(&value);
            if character_error {
                unmappable.insert(character);
            }
        }
        let characters = unmappable
            .into_iter()
            .map(|character| format!("{character} (U+{:04X})", character as u32))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(ScriptError::UnencodableText { characters });
    }
    Ok(encoded.into_owned())
}

pub fn encode_replacement_stream(
    stream: &TextStream,
    replacement_parts: &[String],
) -> Result<Vec<u8>, ScriptError> {
    let expected_parts = text_parts(stream).len();
    if replacement_parts.len() != expected_parts {
        return Err(ScriptError::InvalidPatch(format!(
            "text stream 0x{:04X} has {expected_parts} editable parts, patch has {}",
            stream.offset,
            replacement_parts.len()
        )));
    }
    let mut output = Vec::new();
    let mut replacement_index = 0usize;
    for part in &stream.parts {
        match part {
            TextPart::Text(_) => {
                output
                    .extend_from_slice(&encode_cp932_text(&replacement_parts[replacement_index])?);
                replacement_index += 1;
            }
            TextPart::Control(control) => output.extend_from_slice(&control.encoded()),
            TextPart::Glyph(glyph) => output.extend_from_slice(&glyph.bytes),
        }
    }
    output.push(0);
    Ok(output)
}

fn validate_encoded_stream(stream: &[u8]) -> Result<(), ScriptError> {
    if stream.last() != Some(&0) {
        return Err(ScriptError::InvalidPatch(
            "replacement text stream is not NUL-terminated".to_owned(),
        ));
    }
    let mut fixture = vec![0; SCRIPT_BODY_OFFSET];
    fixture.extend_from_slice(stream);
    let parsed = parse_text_stream(&fixture, SCRIPT_BODY_OFFSET)?;
    if parsed.end_offset != fixture.len() || parsed.encoded() != stream {
        return Err(ScriptError::InvalidPatch(
            "replacement text stream has trailing or non-canonical bytes".to_owned(),
        ));
    }
    Ok(())
}

fn patch_range_mut<'a>(
    data: &'a mut [u8],
    offset: usize,
    length: usize,
    context: &str,
) -> Result<&'a mut [u8], ScriptError> {
    let data_length = data.len();
    data.get_mut(offset..offset.saturating_add(length))
        .ok_or_else(|| {
            ScriptError::InvalidPatch(format!(
                "{context} range 0x{offset:X}..0x{:X} exceeds script length 0x{data_length:X}",
                offset.saturating_add(length)
            ))
        })
}

pub fn patch_script_streams(
    encrypted_source: &[u8],
    patches: &[ScriptStreamPatch],
) -> Result<PatchedScript, ScriptError> {
    let mut stats = ScriptPatchStats {
        requested: patches.len(),
        ..ScriptPatchStats::default()
    };
    let mut decoded = xor_script_body(encrypted_source)?;
    if decoded.len() > CONSERVATIVE_SCRIPT_LIMIT {
        return Err(ScriptError::InvalidPatch(format!(
            "source length 0x{:X} exceeds conservative limit 0x{CONSERVATIVE_SCRIPT_LIMIT:X}",
            decoded.len()
        )));
    }
    let cfg = build_cfg(&decoded);
    if !cfg.warnings.is_empty() {
        let details = cfg
            .warnings
            .iter()
            .take(8)
            .map(|warning| {
                format!(
                    "0x{:04X} {:?}: {}",
                    warning.offset, warning.kind, warning.detail
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        return Err(ScriptError::InvalidCfg(details));
    }
    let candidates = cfg_text_candidates(&decoded, &cfg)?;
    let candidate_by_instruction: BTreeMap<_, _> = candidates
        .iter()
        .map(|candidate| (candidate.instruction_offset, candidate))
        .collect();
    let mut reference_counts: HashMap<usize, usize> = HashMap::new();
    for instruction in cfg
        .instructions
        .values()
        .filter(|instruction| matches!(instruction.opcode, 0x15 | 0x16))
    {
        let pointer_operand = if instruction.opcode == 0x15 { 4 } else { 6 };
        let pointer = usize::from(
            read_u16(&decoded, instruction.offset + pointer_operand)
                .expect("validated text instruction"),
        );
        *reference_counts.entry(pointer).or_default() += 1;
    }

    struct PreparedPatch {
        instruction_offset: usize,
        pointer_operand: usize,
        original_offset: usize,
        original_size: usize,
        encoded: Vec<u8>,
        unchanged: bool,
    }

    let mut seen_instructions = HashSet::new();
    let mut prepared = Vec::new();
    for patch in patches {
        if !seen_instructions.insert(patch.instruction_offset) {
            return Err(ScriptError::InvalidPatch(format!(
                "duplicate instruction offset 0x{:04X}",
                patch.instruction_offset
            )));
        }
        let candidate = candidate_by_instruction
            .get(&patch.instruction_offset)
            .ok_or_else(|| {
                ScriptError::InvalidPatch(format!(
                    "no localizable text instruction at 0x{:04X}",
                    patch.instruction_offset
                ))
            })?;
        if candidate.stream.offset != patch.expected_text_offset {
            return Err(ScriptError::InvalidPatch(format!(
                "instruction 0x{:04X} text pointer is 0x{:04X}, expected 0x{:04X}",
                patch.instruction_offset, candidate.stream.offset, patch.expected_text_offset
            )));
        }
        let original_size = candidate.stream.end_offset - candidate.stream.offset;
        if original_size != patch.expected_size {
            return Err(ScriptError::InvalidPatch(format!(
                "instruction 0x{:04X} text size is {original_size}, expected {}",
                patch.instruction_offset, patch.expected_size
            )));
        }
        let actual_stream = candidate.stream.encoded();
        if actual_stream != patch.expected_stream {
            return Err(ScriptError::InvalidPatch(format!(
                "instruction 0x{:04X} immutable source stream does not match",
                patch.instruction_offset
            )));
        }
        validate_encoded_stream(&patch.replacement_stream)?;
        prepared.push(PreparedPatch {
            instruction_offset: patch.instruction_offset,
            pointer_operand: if candidate.opcode == 0x15 { 4 } else { 6 },
            original_offset: candidate.stream.offset,
            original_size,
            encoded: patch.replacement_stream.clone(),
            unchanged: patch.replacement_stream == patch.expected_stream,
        });
    }

    let mut appended: HashMap<(usize, Vec<u8>), u16> = HashMap::new();
    for patch in prepared {
        if patch.unchanged {
            stats.unchanged += 1;
            continue;
        }
        stats.patched += 1;
        let unique_reference = reference_counts.get(&patch.original_offset) == Some(&1);
        if unique_reference && patch.encoded.len() <= patch.original_size {
            patch_range_mut(
                &mut decoded,
                patch.original_offset,
                patch.encoded.len(),
                "in-place text patch",
            )?
            .copy_from_slice(&patch.encoded);
            stats.in_place += 1;
            continue;
        }

        let key = (patch.original_offset, patch.encoded.clone());
        let new_pointer = if let Some(pointer) = appended.get(&key) {
            *pointer
        } else {
            let new_offset = decoded.len();
            let new_length = new_offset
                .checked_add(patch.encoded.len())
                .ok_or_else(|| ScriptError::InvalidPatch("script length overflow".to_owned()))?;
            if new_length > CONSERVATIVE_SCRIPT_LIMIT {
                return Err(ScriptError::InvalidPatch(format!(
                    "relocation would grow script to 0x{new_length:X}, limit is 0x{CONSERVATIVE_SCRIPT_LIMIT:X}"
                )));
            }
            let pointer = u16::try_from(new_offset).map_err(|_| {
                ScriptError::InvalidPatch(format!(
                    "relocated stream offset 0x{new_offset:X} does not fit u16"
                ))
            })?;
            decoded.extend_from_slice(&patch.encoded);
            stats.appended_streams += 1;
            stats.appended_bytes += patch.encoded.len();
            appended.insert(key, pointer);
            pointer
        };
        let pointer_offset = patch.instruction_offset + patch.pointer_operand;
        patch_range_mut(&mut decoded, pointer_offset, 2, "text pointer patch")?
            .copy_from_slice(&new_pointer.to_le_bytes());
        stats.relocated_entries += 1;
    }

    let bytes = if stats.patched == 0 {
        encrypted_source.to_vec()
    } else {
        xor_script_body(&decoded)?
    };
    stats.output_bytes = bytes.len();
    stats.byte_exact = bytes == encrypted_source;
    Ok(PatchedScript { bytes, stats })
}

pub fn patch_script(
    encrypted_source: &[u8],
    patches: &[ScriptTextPatch],
) -> Result<PatchedScript, ScriptError> {
    let decoded = xor_script_body(encrypted_source)?;
    let cfg = build_cfg(&decoded);
    if !cfg.warnings.is_empty() {
        let details = cfg
            .warnings
            .iter()
            .take(8)
            .map(|warning| {
                format!(
                    "0x{:04X} {:?}: {}",
                    warning.offset, warning.kind, warning.detail
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        return Err(ScriptError::InvalidCfg(details));
    }
    let candidates = cfg_text_candidates(&decoded, &cfg)?;
    let candidate_by_instruction: BTreeMap<_, _> = candidates
        .iter()
        .map(|candidate| (candidate.instruction_offset, candidate))
        .collect();
    let mut stream_patches = Vec::with_capacity(patches.len());
    for patch in patches {
        let candidate = candidate_by_instruction
            .get(&patch.instruction_offset)
            .ok_or_else(|| {
                ScriptError::InvalidPatch(format!(
                    "no localizable text instruction at 0x{:04X}",
                    patch.instruction_offset
                ))
            })?;
        let actual_source_parts: Vec<_> = text_parts(&candidate.stream)
            .into_iter()
            .map(str::to_owned)
            .collect();
        if actual_source_parts != patch.source_parts {
            return Err(ScriptError::InvalidPatch(format!(
                "instruction 0x{:04X} immutable source parts do not match",
                patch.instruction_offset
            )));
        }
        stream_patches.push(ScriptStreamPatch {
            instruction_offset: patch.instruction_offset,
            expected_text_offset: patch.expected_text_offset,
            expected_size: patch.expected_size,
            expected_stream: candidate.stream.encoded(),
            replacement_stream: encode_replacement_stream(
                &candidate.stream,
                &patch.replacement_parts,
            )?,
        });
    }
    patch_script_streams(encrypted_source, &stream_patches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn xor_preserves_prefix_and_is_symmetric() {
        let mut source = vec![0xA5; SCRIPT_BODY_OFFSET + 4];
        source[SCRIPT_BODY_OFFSET..].copy_from_slice(&[0x00, 0x01, 0xFE, 0xFF]);
        let decoded = xor_script_body(&source).unwrap();
        assert_eq!(
            &decoded[..SCRIPT_BODY_OFFSET],
            &source[..SCRIPT_BODY_OFFSET]
        );
        assert_eq!(&decoded[SCRIPT_BODY_OFFSET..], &[0x01, 0x00, 0xFF, 0xFE]);
        assert_eq!(xor_script_body(&decoded).unwrap(), source);
    }

    #[test]
    fn parses_and_rebuilds_text_parts_exactly() {
        let (encoded, _, had_errors) = SHIFT_JIS.encode("主人公です");
        assert!(!had_errors);
        let mut data = vec![0; SCRIPT_BODY_OFFSET];
        data.extend_from_slice(&encoded[..4]);
        data.extend_from_slice(&[0x04, 0x02, 0x0D]);
        data.extend_from_slice(&encoded[4..]);
        data.extend_from_slice(&[0x01, 0x00]);

        let stream = parse_text_stream(&data, SCRIPT_BODY_OFFSET).unwrap();
        assert_eq!(stream.text, "主人公です");
        assert_eq!(stream.controls().count(), 3);
        assert_eq!(stream.encoded(), data[SCRIPT_BODY_OFFSET..]);
    }

    #[test]
    fn preserves_game_font_glyphs_with_jis_locations() {
        let (encoded, _, had_errors) = SHIFT_JIS.encode("続く");
        assert!(!had_errors);
        let mut data = vec![0; SCRIPT_BODY_OFFSET];
        data.extend_from_slice(&encoded);
        data.extend_from_slice(&[0xEB, 0xB5, 0x00]);

        let stream = parse_text_stream(&data, SCRIPT_BODY_OFFSET).unwrap();
        assert_eq!(stream.text, "続く");
        let glyph = stream.glyphs().next().unwrap();
        assert_eq!(glyph.jis_code, Some(0x7637));
        assert_eq!(glyph.gao4_index(), Some(22));
        assert_eq!(stream.encoded(), data[SCRIPT_BODY_OFFSET..]);
    }

    #[test]
    fn rejects_unknown_control_and_invalid_pointer() {
        let mut data = vec![0; SCRIPT_BODY_OFFSET + 3];
        data[SCRIPT_BODY_OFFSET..].copy_from_slice(&[b'A', 0x0F, 0x01]);
        assert!(matches!(
            parse_text_stream(&data, SCRIPT_BODY_OFFSET),
            Err(ScriptError::UnknownExtendedControl { selector: 1, .. })
        ));
        assert!(matches!(
            parse_text_stream(&data, 0xFF),
            Err(ScriptError::InvalidTextPointer { .. })
        ));
    }

    fn encrypted_patch_fixture(shared_pointer: bool) -> Vec<u8> {
        let mut decoded = vec![0u8; 0x130];
        decoded[0x100..0x106].copy_from_slice(&[0x15, 0, 9, 0, 0x20, 0x01]);
        if shared_pointer {
            decoded[0x106..0x10C].copy_from_slice(&[0x15, 0, 9, 0, 0x20, 0x01]);
            decoded[0x10C..0x10E].copy_from_slice(&[0, 0]);
        } else {
            decoded[0x106..0x108].copy_from_slice(&[0, 0]);
        }
        let (source, _, had_errors) =
            SHIFT_JIS.encode(if shared_pointer { "元" } else { "原文" });
        assert!(!had_errors);
        let end = 0x120 + source.len();
        decoded[0x120..end].copy_from_slice(&source);
        decoded[end..end + 2].copy_from_slice(&[0x01, 0x00]);
        xor_script_body(&decoded).unwrap()
    }

    #[test]
    fn unchanged_patch_is_byte_exact_and_unique_short_text_is_in_place() {
        let source = encrypted_patch_fixture(false);
        let unchanged = patch_script(
            &source,
            &[ScriptTextPatch {
                instruction_offset: 0x100,
                expected_text_offset: 0x120,
                expected_size: 6,
                source_parts: vec!["原文".to_owned()],
                replacement_parts: vec!["原文".to_owned()],
            }],
        )
        .unwrap();
        assert!(unchanged.stats.byte_exact);
        assert_eq!(unchanged.bytes, source);

        let changed = patch_script(
            &source,
            &[ScriptTextPatch {
                instruction_offset: 0x100,
                expected_text_offset: 0x120,
                expected_size: 6,
                source_parts: vec!["原文".to_owned()],
                replacement_parts: vec!["短".to_owned()],
            }],
        )
        .unwrap();
        assert_eq!(changed.stats.in_place, 1);
        assert_eq!(changed.stats.relocated_entries, 0);
        assert_eq!(changed.bytes.len(), source.len());
        let decoded = xor_script_body(&changed.bytes).unwrap();
        assert_eq!(read_u16(&decoded, 0x104), Some(0x120));
        assert_eq!(parse_text_stream(&decoded, 0x120).unwrap().text, "短");
    }

    #[test]
    fn shared_pointer_changes_relocate_and_reuse_identical_stream() {
        let source = encrypted_patch_fixture(true);
        let patch = |instruction_offset| ScriptTextPatch {
            instruction_offset,
            expected_text_offset: 0x120,
            expected_size: 4,
            source_parts: vec!["元".to_owned()],
            replacement_parts: vec!["変更".to_owned()],
        };
        let changed = patch_script(&source, &[patch(0x100), patch(0x106)]).unwrap();
        assert_eq!(changed.stats.relocated_entries, 2);
        assert_eq!(changed.stats.appended_streams, 1);
        let decoded = xor_script_body(&changed.bytes).unwrap();
        let first_pointer = read_u16(&decoded, 0x104).unwrap();
        let second_pointer = read_u16(&decoded, 0x10A).unwrap();
        assert_eq!(first_pointer, second_pointer);
        assert_eq!(usize::from(first_pointer), source.len());
        let stream = parse_text_stream(&decoded, usize::from(first_pointer)).unwrap();
        assert_eq!(stream.text, "変更");
        assert_eq!(stream.controls().next().unwrap().code, 0x01);
    }

    #[test]
    fn patch_rejects_changed_source_and_unmapped_text() {
        let source = encrypted_patch_fixture(false);
        let wrong_source = ScriptTextPatch {
            instruction_offset: 0x100,
            expected_text_offset: 0x120,
            expected_size: 6,
            source_parts: vec!["違う".to_owned()],
            replacement_parts: vec!["短".to_owned()],
        };
        assert!(matches!(
            patch_script(&source, &[wrong_source]),
            Err(ScriptError::InvalidPatch(_))
        ));
        let unmapped = ScriptTextPatch {
            instruction_offset: 0x100,
            expected_text_offset: 0x120,
            expected_size: 6,
            source_parts: vec!["原文".to_owned()],
            replacement_parts: vec!["汉".to_owned()],
        };
        assert!(matches!(
            patch_script(&source, &[unmapped]),
            Err(ScriptError::UnencodableText { .. })
        ));
    }

    #[test]
    fn follows_branch_and_odd_callback_roots() {
        let mut data = vec![0; 0x130];
        data[0x100..0x104].copy_from_slice(&[0x0A, 0x00, 0x11, 0x01]);
        data[0x111..0x11B].copy_from_slice(&[0x40, 0x00, 0, 0, 0, 0, 0x21, 0x01, 0x23, 0x01]);
        data[0x11B..0x11D].copy_from_slice(&[0x00, 0x00]);
        data[0x121..0x123].copy_from_slice(&[0x00, 0x00]);
        data[0x123..0x125].copy_from_slice(&[0x00, 0x00]);
        let cfg = build_cfg(&data);
        assert!(cfg.warnings.is_empty(), "{:?}", cfg.warnings);
        assert_eq!(
            cfg.instructions.keys().copied().collect::<Vec<_>>(),
            vec![0x100, 0x111, 0x11B, 0x121, 0x123]
        );
    }

    #[test]
    fn recognizes_only_cs_number_number_scripts() {
        assert!(is_main_story_script_name("cs01_01.s"));
        assert!(is_main_story_script_name("CS99_99.S"));
        assert!(!is_main_story_script_name("prologue.s"));
        assert!(!is_main_story_script_name("cs1_01.s"));
    }

    fn collect_scripts(root: &Path, output: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect_scripts(&path, output);
            } else if path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("s"))
            {
                output.push(path);
            }
        }
    }

    #[test]
    #[ignore = "requires CANAAN_SCRIPT_ROOT pointing to the unpacked real corpus"]
    fn real_corpus_matches_confirmed_profile() {
        let root = PathBuf::from(
            std::env::var_os("CANAAN_SCRIPT_ROOT")
                .expect("set CANAAN_SCRIPT_ROOT to the unpacked script root"),
        );
        let mut paths = Vec::new();
        collect_scripts(&root, &mut paths);
        paths.sort();

        let mut instruction_count = 0usize;
        let mut candidate_count = 0usize;
        let mut template_count = 0usize;
        let mut simple_templates = 0usize;
        let mut extended_templates = 0usize;
        let mut save_candidate_count = 0usize;
        let mut main_story_count = 0usize;
        let mut text_instruction_count = 0usize;
        let mut main_story_glyph_entries = 0usize;
        let mut main_story_glyph_occurrences = 0usize;
        let mut main_story_gao4_occurrences = 0usize;
        let mut shared_pointer_groups = 0usize;
        let mut entries_on_shared_pointers = 0usize;
        let mut overlapping_text_spans = 0usize;
        let mut unchanged_patch_entries = 0usize;

        for path in &paths {
            let source = fs::read(path).unwrap();
            let data = xor_script_body(&source).unwrap();
            let cfg = build_cfg(&data);
            assert!(
                cfg.warnings.is_empty(),
                "{}: {:?}",
                path.display(),
                cfg.warnings
            );
            instruction_count += cfg.instructions.len();
            for instruction in cfg
                .instructions
                .values()
                .filter(|instruction| matches!(instruction.opcode, 0x15 | 0x16))
            {
                text_instruction_count += 1;
                let pointer_operand = if instruction.opcode == 0x15 { 4 } else { 6 };
                let pointer = usize::from(
                    read_u16(&data, instruction.offset + pointer_operand)
                        .expect("text instruction is complete"),
                );
                let stream = parse_text_stream(&data, pointer).unwrap();
                assert_eq!(
                    stream.encoded(),
                    data[stream.offset..stream.end_offset],
                    "{}: text stream 0x{:04X}",
                    path.display(),
                    stream.offset
                );
            }
            let candidates = cfg_text_candidates(&data, &cfg).unwrap();
            candidate_count += candidates.len();
            let template = validate_reachable_save_template(&data, &cfg, &candidates).unwrap();
            let excluded: BTreeSet<_> = template
                .as_ref()
                .map(|value| value.text_instructions.iter().copied().collect())
                .unwrap_or_default();
            if let Some(template) = template {
                template_count += 1;
                save_candidate_count += template.text_instructions.len();
                match template.block_variant {
                    SaveBlockVariant::Simple => simple_templates += 1,
                    SaveBlockVariant::Extended => extended_templates += 1,
                }
            }
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(is_main_story_script_name)
            {
                let main_candidates: Vec<_> = candidates
                    .iter()
                    .filter(|candidate| !excluded.contains(&candidate.instruction_offset))
                    .collect();
                let mut by_pointer: BTreeMap<usize, usize> = BTreeMap::new();
                let mut spans = BTreeSet::new();
                for candidate in &main_candidates {
                    *by_pointer.entry(candidate.stream.offset).or_default() += 1;
                    spans.insert((candidate.stream.offset, candidate.stream.end_offset));
                }
                for references in by_pointer.values().filter(|references| **references > 1) {
                    shared_pointer_groups += 1;
                    entries_on_shared_pointers += references;
                }
                let spans: Vec<_> = spans.into_iter().collect();
                overlapping_text_spans += spans
                    .windows(2)
                    .filter(|pair| pair[1].0 < pair[0].1)
                    .count();

                let unchanged_patches: Vec<_> = main_candidates
                    .iter()
                    .map(|candidate| {
                        let source_parts: Vec<_> = text_parts(&candidate.stream)
                            .into_iter()
                            .map(str::to_owned)
                            .collect();
                        ScriptTextPatch {
                            instruction_offset: candidate.instruction_offset,
                            expected_text_offset: candidate.stream.offset,
                            expected_size: candidate.stream.end_offset - candidate.stream.offset,
                            replacement_parts: source_parts.clone(),
                            source_parts,
                        }
                    })
                    .collect();
                let round_trip = patch_script(&source, &unchanged_patches).unwrap();
                assert!(round_trip.stats.byte_exact, "{}", path.display());
                assert_eq!(round_trip.bytes, source, "{}", path.display());
                assert_eq!(
                    round_trip.stats.unchanged,
                    main_candidates.len(),
                    "{}",
                    path.display()
                );
                unchanged_patch_entries += round_trip.stats.unchanged;

                for candidate in main_candidates {
                    main_story_count += 1;
                    let glyph_count = candidate.stream.glyphs().count();
                    if glyph_count != 0 {
                        main_story_glyph_entries += 1;
                    }
                    main_story_glyph_occurrences += glyph_count;
                    main_story_gao4_occurrences += candidate
                        .stream
                        .glyphs()
                        .filter(|glyph| glyph.gao4_index().is_some())
                        .count();
                }
            }
        }

        assert_eq!(paths.len(), 476);
        assert_eq!(instruction_count, 442_570);
        assert_eq!(text_instruction_count, 31_155);
        assert_eq!(candidate_count, 30_215);
        assert_eq!(template_count, 430);
        assert_eq!(simple_templates, 378);
        assert_eq!(extended_templates, 52);
        assert_eq!(save_candidate_count, 6_450);
        assert_eq!(main_story_count, 23_615);
        assert_eq!(main_story_glyph_entries, 6_085);
        assert_eq!(main_story_glyph_occurrences, 7_115);
        assert_eq!(main_story_gao4_occurrences, 7_027);
        assert_eq!(unchanged_patch_entries, 23_615);
        assert_eq!(shared_pointer_groups, 2_676);
        assert_eq!(entries_on_shared_pointers, 5_408);
        assert_eq!(overlapping_text_spans, 0);
        println!(
            "shared_pointer_groups={shared_pointer_groups} entries_on_shared_pointers={entries_on_shared_pointers} overlapping_text_spans={overlapping_text_spans}"
        );
    }

    #[test]
    #[ignore = "requires CANAAN_SCRIPT_ROOT pointing to the unpacked real corpus"]
    fn real_script_short_and_long_modifications() {
        let root = PathBuf::from(
            std::env::var_os("CANAAN_SCRIPT_ROOT")
                .expect("set CANAAN_SCRIPT_ROOT to the unpacked script root"),
        );
        let path = root.join("DISK_J").join("cs01_04.s");
        let source = fs::read(&path).unwrap();
        let decoded = xor_script_body(&source).unwrap();
        let cfg = build_cfg(&decoded);
        assert!(cfg.warnings.is_empty());
        let candidates = cfg_text_candidates(&decoded, &cfg).unwrap();
        let candidate = candidates
            .iter()
            .find(|candidate| candidate.instruction_offset == 0x0BFC)
            .expect("confirmed dynamic-name sample");
        let source_parts: Vec<_> = text_parts(&candidate.stream)
            .into_iter()
            .map(str::to_owned)
            .collect();
        assert_eq!(source_parts.len(), 2);
        let original_controls: Vec<_> = candidate
            .stream
            .controls()
            .map(TextControl::encoded)
            .collect();
        let base = ScriptTextPatch {
            instruction_offset: candidate.instruction_offset,
            expected_text_offset: candidate.stream.offset,
            expected_size: candidate.stream.end_offset - candidate.stream.offset,
            source_parts,
            replacement_parts: Vec::new(),
        };

        let mut short_patch = base.clone();
        short_patch.replacement_parts = vec!["僕".to_owned(), "旅が始まった。".to_owned()];
        let short = patch_script(&source, &[short_patch.clone()]).unwrap();
        assert_eq!(short.stats.in_place, 1);
        assert_eq!(short.stats.relocated_entries, 0);
        assert_eq!(short.bytes.len(), source.len());
        let short_decoded = xor_script_body(&short.bytes).unwrap();
        let short_stream = parse_text_stream(&short_decoded, candidate.stream.offset).unwrap();
        assert_eq!(
            text_parts(&short_stream),
            short_patch
                .replacement_parts
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            short_stream
                .controls()
                .map(TextControl::encoded)
                .collect::<Vec<_>>(),
            original_controls
        );

        let mut long_patch = base;
        long_patch.replacement_parts = vec!["僕".to_owned(), "長い".repeat(200)];
        let long = patch_script(&source, &[long_patch.clone()]).unwrap();
        assert_eq!(long.stats.in_place, 0);
        assert_eq!(long.stats.relocated_entries, 1);
        assert_eq!(long.stats.appended_streams, 1);
        assert!(long.bytes.len() > source.len());
        let long_decoded = xor_script_body(&long.bytes).unwrap();
        let new_pointer = usize::from(read_u16(&long_decoded, 0x0C00).unwrap());
        assert_eq!(new_pointer, source.len());
        let long_stream = parse_text_stream(&long_decoded, new_pointer).unwrap();
        assert_eq!(
            text_parts(&long_stream),
            long_patch
                .replacement_parts
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            long_stream
                .controls()
                .map(TextControl::encoded)
                .collect::<Vec<_>>(),
            original_controls
        );
        for offset in 0..decoded.len() {
            if !(0x0C00..0x0C02).contains(&offset) {
                assert_eq!(long_decoded[offset], decoded[offset], "offset 0x{offset:X}");
            }
        }
    }
}
