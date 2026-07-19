use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};

pub const FORMAT_V3: &str = "NECRONOMICON MES sub_D77C full-file bytecode IR v3";

const SCRIPT_ENCODING: &str = "CP932 + custom hiragana bytes 0x2D..0x7F -> 0x82,(byte+0x72)";
const LEXER_PROFILE: &str = "sub_D77C token-length rules, full-file coverage";
const HANDLER_PROFILE: &str =
    "A6..D8 runtime DS:DB5A jump table + static handler disassembly, 2026-07-18";
const ENTRY_ENCODING: &str = "CP932 + custom hiragana byte restoration";
const ENTRY_OPCODE: &str = "inline_text";
const ENTRY_POLICY: &str = "relocate";
const NAME_POLICY: &str =
    "confirmed: a leading corner-quoted prefix is writable name; _scr_name is immutable";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandlerProfile {
    pub opcode: u8,
    pub target: &'static str,
    pub grammar: &'static str,
}

// Targets are static-image offsets recovered from the live DS:DB5A table.
// Grammar descriptions state how each handler advances SI.  Structural
// handlers recursively invoke the same bytecode stream parser, so their
// nested bytes remain ordinary full-file tokens rather than opaque payloads.
const HANDLERS: &[HandlerProfile] = &[
    HandlerProfile {
        opcode: 0xA6,
        target: "C805",
        grammar: "expr x4",
    },
    HandlerProfile {
        opcode: 0xA7,
        target: "C839",
        grammar: "expr x2",
    },
    HandlerProfile {
        opcode: 0xA8,
        target: "C84E",
        grammar: "expr x2, then conditional expr",
    },
    HandlerProfile {
        opcode: 0xA9,
        target: "C86C",
        grammar: "expr x2",
    },
    HandlerProfile {
        opcode: 0xAA,
        target: "C87F",
        grammar: "optional expr",
    },
    HandlerProfile {
        opcode: 0xAB,
        target: "C898",
        grammar: "expr, then conditional expr",
    },
    HandlerProfile {
        opcode: 0xAC,
        target: "C8AB",
        grammar: "expr, then conditional expr",
    },
    HandlerProfile {
        opcode: 0xAD,
        target: "C8BC",
        grammar: "expr pairs until zero, then nested selection block",
    },
    HandlerProfile {
        opcode: 0xAE,
        target: "CCA8",
        grammar: "A0..A3 structural blocks and nested AD lists",
    },
    HandlerProfile {
        opcode: 0xAF,
        target: "CD9B",
        grammar: "table-driven conditional display block",
    },
    HandlerProfile {
        opcode: 0xB0,
        target: "CE1D",
        grammar: "quoted string, then up to two expr",
    },
    HandlerProfile {
        opcode: 0xB1,
        target: "CECA",
        grammar: "quoted string",
    },
    HandlerProfile {
        opcode: 0xB2,
        target: "CEEC",
        grammar: "conditional scanner and nested display block",
    },
    HandlerProfile {
        opcode: 0xB3,
        target: "CF16",
        grammar: "structural display block",
    },
    HandlerProfile {
        opcode: 0xB4,
        target: "CF26",
        grammar: "raw byte x2",
    },
    HandlerProfile {
        opcode: 0xB5,
        target: "CF38",
        grammar: "expr, raw selector, then structural display",
    },
    HandlerProfile {
        opcode: 0xB6,
        target: "CF4B",
        grammar: "quoted string or raw selector plus optional expr",
    },
    HandlerProfile {
        opcode: 0xB7,
        target: "CF8F",
        grammar: "expr, optional expr, then conditional expr",
    },
    HandlerProfile {
        opcode: 0xB8,
        target: "CFE1",
        grammar: "optional expr x2",
    },
    HandlerProfile {
        opcode: 0xB9,
        target: "CFF8",
        grammar: "expr, then structural block",
    },
    HandlerProfile {
        opcode: 0xBA,
        target: "D00F",
        grammar: "expr, then nested bytecode stream",
    },
    HandlerProfile {
        opcode: 0xBB,
        target: "D022",
        grammar: "expr-selected nested/conditional block",
    },
    HandlerProfile {
        opcode: 0xBC,
        target: "D04B",
        grammar: "condition list through A3, then structural block",
    },
    HandlerProfile {
        opcode: 0xBD,
        target: "D062",
        grammar: "condition list through A3",
    },
    HandlerProfile {
        opcode: 0xBE,
        target: "D06D",
        grammar: "optional expr",
    },
    HandlerProfile {
        opcode: 0xBF,
        target: "D096",
        grammar: "optional expr x2",
    },
    HandlerProfile {
        opcode: 0xC0,
        target: "D12C",
        grammar: "quoted string plus expr, or expr",
    },
    HandlerProfile {
        opcode: 0xC1,
        target: "D1BA",
        grammar: "expr, then quoted string",
    },
    HandlerProfile {
        opcode: 0xC2,
        target: "D1D6",
        grammar: "expr, then quoted string",
    },
    HandlerProfile {
        opcode: 0xC3,
        target: "D1E4",
        grammar: "expr x2",
    },
    HandlerProfile {
        opcode: 0xC4,
        target: "D1F2",
        grammar: "expr x2",
    },
    HandlerProfile {
        opcode: 0xC5,
        target: "D20A",
        grammar: "expr x2",
    },
    HandlerProfile {
        opcode: 0xC6,
        target: "D21D",
        grammar: "expr x5",
    },
    HandlerProfile {
        opcode: 0xC7,
        target: "D239",
        grammar: "optional expr",
    },
    HandlerProfile {
        opcode: 0xC8,
        target: "D27E",
        grammar: "quoted string, then optional expr",
    },
    HandlerProfile {
        opcode: 0xC9,
        target: "D296",
        grammar: "quoted string",
    },
    HandlerProfile {
        opcode: 0xCA,
        target: "D531",
        grammar: "raw selector byte, then expr x3",
    },
    HandlerProfile {
        opcode: 0xCB,
        target: "D5DE",
        grammar: "expr",
    },
    HandlerProfile {
        opcode: 0xCC,
        target: "D2AA",
        grammar: "expr, then optional expr",
    },
    HandlerProfile {
        opcode: 0xCD,
        target: "D2FB",
        grammar: "expr, then repeated quoted-string/expr operands",
    },
    HandlerProfile {
        opcode: 0xCE,
        target: "D377",
        grammar: "expr x2",
    },
    HandlerProfile {
        opcode: 0xCF,
        target: "D38F",
        grammar: "expr, then optional expr triple",
    },
    HandlerProfile {
        opcode: 0xD0,
        target: "D3D2",
        grammar: "subcommand expr/quoted-string branch",
    },
    HandlerProfile {
        opcode: 0xD1,
        target: "D46B",
        grammar: "subcommand-selected variable expr sequence",
    },
    HandlerProfile {
        opcode: 0xD2,
        target: "D4F2",
        grammar: "no stream operands",
    },
    HandlerProfile {
        opcode: 0xD3,
        target: "D524",
        grammar: "expr x2",
    },
    HandlerProfile {
        opcode: 0xD4,
        target: "D1A5",
        grammar: "no stream operands",
    },
    HandlerProfile {
        opcode: 0xD5,
        target: "C797",
        grammar: "expr; may transform remaining stream at runtime",
    },
    HandlerProfile {
        opcode: 0xD6,
        target: "D5F1",
        grammar: "return; no stream operands",
    },
    HandlerProfile {
        opcode: 0xD7,
        target: "D5F1",
        grammar: "return; no stream operands",
    },
    HandlerProfile {
        opcode: 0xD8,
        target: "D5F1",
        grammar: "return; no stream operands",
    },
];

pub fn opcode_handler_profile(opcode: u8) -> Option<HandlerProfile> {
    HANDLERS
        .iter()
        .copied()
        .find(|profile| profile.opcode == opcode)
}

fn is_sjis_lead(byte: u8) -> bool {
    (0x81..=0x9F).contains(&byte) || (0xE0..=0xFC).contains(&byte)
}

fn is_sjis_trail(byte: u8) -> bool {
    (0x40..=0x7E).contains(&byte) || (0x80..=0xFC).contains(&byte)
}

fn strict_sjis_pair(bytes: &[u8], offset: usize) -> bool {
    if offset + 1 >= bytes.len()
        || !is_sjis_lead(bytes[offset])
        || !is_sjis_trail(bytes[offset + 1])
    {
        return false;
    }
    let (_, had_errors) = SHIFT_JIS.decode_without_bom_handling(&bytes[offset..offset + 2]);
    !had_errors
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len().saturating_mul(3));
    for (index, byte) in bytes.iter().enumerate() {
        if index != 0 {
            output.push(' ');
        }
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02X}");
    }
    output
}

fn decode_text(bytes: &[u8]) -> Result<String, String> {
    let mut restored = Vec::with_capacity(bytes.len() * 2);
    let mut offset = 0usize;
    while offset < bytes.len() {
        if strict_sjis_pair(bytes, offset) {
            restored.extend_from_slice(&bytes[offset..offset + 2]);
            offset += 2;
        } else if (0x2D..=0x7F).contains(&bytes[offset]) {
            restored.push(0x82);
            restored.push(bytes[offset] + 0x72);
            offset += 1;
        } else {
            return Err(format!(
                "non-display byte {:02X} at relative offset 0x{offset:X}",
                bytes[offset]
            ));
        }
    }
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(&restored);
    if had_errors {
        return Err("CP932 decode failed after custom hiragana restoration".to_owned());
    }
    Ok(decoded.into_owned())
}

fn encode_text(value: &str) -> Result<Vec<u8>, String> {
    if value.contains('\0') {
        return Err("text contains NUL".to_owned());
    }
    if value.contains(['\r', '\n']) {
        return Err(
            "text contains CR/LF; MES display segments remain separate JSON entries".to_owned(),
        );
    }
    let (encoded, _, had_errors) = SHIFT_JIS.encode(value);
    if had_errors {
        let mut invalid = Vec::new();
        for character in value.chars() {
            let (_, _, character_error) = SHIFT_JIS.encode(&character.to_string());
            if character_error && !invalid.contains(&character) {
                invalid.push(character);
            }
        }
        let details = invalid
            .iter()
            .map(|character| format!("{character:?} (U+{:04X})", *character as u32))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!("text is not representable in CP932: {details}"));
    }

    let encoded = encoded.as_ref();
    let mut result = Vec::with_capacity(encoded.len());
    let mut offset = 0usize;
    while offset < encoded.len() {
        if encoded[offset] == 0x82
            && offset + 1 < encoded.len()
            && (0x9F..=0xF1).contains(&encoded[offset + 1])
        {
            result.push(encoded[offset + 1] - 0x72);
            offset += 2;
        } else if strict_sjis_pair(encoded, offset) {
            result.extend_from_slice(&encoded[offset..offset + 2]);
            offset += 2;
        } else {
            result.push(encoded[offset]);
            offset += 1;
        }
    }

    let roundtrip = decode_text(&result).map_err(|_| {
        "text contains a CP932 single-byte character that collides with MES bytecode".to_owned()
    })?;
    if roundtrip != value {
        return Err(
            "text does not round-trip through the MES display encoding because a single-byte character collides with MES bytecode"
                .to_owned(),
        );
    }
    Ok(result)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LexToken {
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub token_type: String,
    #[serde(rename = "_role")]
    pub role: String,
    #[serde(rename = "_raw")]
    pub raw: String,
    #[serde(rename = "_opcode", skip_serializing_if = "Option::is_none")]
    pub opcode: Option<String>,
    #[serde(rename = "_handler_target", skip_serializing_if = "Option::is_none")]
    pub handler_target: Option<String>,
    #[serde(rename = "_handler_grammar", skip_serializing_if = "Option::is_none")]
    pub handler_grammar: Option<String>,
    #[serde(rename = "_entry_index", skip_serializing_if = "Option::is_none")]
    pub entry_index: Option<usize>,
    #[serde(
        rename = "_owner_opcode_index",
        skip_serializing_if = "Option::is_none"
    )]
    pub owner_opcode_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_token_start")]
    pub token_start: usize,
    #[serde(rename = "_token_end")]
    pub token_end: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_opcode")]
    pub opcode: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_policy")]
    pub policy: String,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    pub scr_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptJson {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_source_size")]
    pub source_size: usize,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_lexer")]
    pub lexer: String,
    #[serde(rename = "_handler_profile")]
    pub handler_profile: String,
    #[serde(rename = "_name_policy")]
    pub name_policy: String,
    pub entries: Vec<TextEntry>,
    #[serde(rename = "_tokens")]
    pub tokens: Vec<LexToken>,
    #[serde(rename = "_warnings")]
    pub warnings: Vec<String>,
}

fn token_length(bytes: &[u8], offset: usize, warnings: &mut Vec<String>) -> usize {
    let remaining = bytes.len() - offset;
    let byte = bytes[offset];
    let required = match byte {
        0x21 => {
            if let Some(end) = bytes[offset + 1..].iter().position(|&value| value == 0) {
                return end + 2;
            }
            warnings.push(format!("unterminated 0x21 NUL string at 0x{offset:X}"));
            return remaining;
        }
        0x22 => {
            if let Some(end) = bytes[offset + 1..].iter().position(|&value| value == 0x22) {
                return end + 2;
            }
            warnings.push(format!("unterminated 0x22 quoted string at 0x{offset:X}"));
            return remaining;
        }
        // sub_D77C initializes CX=1 after consuming the opcode byte.
        // High nibble 0 consumes one additional byte; high nibble 1
        // consumes two additional bytes.
        0x00..=0x0F => 2,
        0x10..=0x1F => 3,
        0x23..=0x27 => 1,
        0x28 => 2,
        0x29..=0x2C => 3,
        value if is_sjis_lead(value) => 2,
        _ => 1,
    };
    if remaining < required {
        warnings.push(format!(
            "truncated sub_D77C token {:02X} at 0x{offset:X}: expected {required}, have {remaining}",
            byte
        ));
    }
    required.min(remaining)
}

fn classify_token(bytes: &[u8], offset: usize, size: usize) -> LexToken {
    let byte = bytes[offset];
    let mut token_type = "byte";
    let mut role = "opaque";
    let mut opcode = None;
    let mut handler_target = None;
    let mut handler_grammar = None;

    match byte {
        0x21 => {
            token_type = "nul_string";
            role = "operand_string";
        }
        0x22 => {
            token_type = "quoted_string";
            role = "operand_string";
        }
        0x00..=0x0F => {
            token_type = "control_2byte";
            role = "control";
        }
        0x10..=0x1F => {
            token_type = "control_3byte";
            role = "control";
        }
        0x23..=0x2C => {
            token_type = "immediate";
            role = "expression";
        }
        value if is_sjis_lead(value) => {
            token_type = "sjis_pair";
            role = if size == 2
                && matches!(
                    &bytes[offset..offset + 2],
                    [0x81, 0x97] | [0x81, 0x90] | [0x81, 0x6F] | [0x81, 0x70]
                ) {
                "special_control"
            } else if size == 2 && strict_sjis_pair(bytes, offset) {
                "text"
            } else {
                "invalid_sjis"
            };
        }
        0x2D..=0x7F => role = "text",
        0xA0..=0xA4 => role = "block_control",
        0xA5 => role = "line_break",
        0xA6..=0xD8 => {
            role = "opcode";
            opcode = Some(format!("{byte:02X}"));
            if let Some(profile) = opcode_handler_profile(byte) {
                handler_target = Some(profile.target.to_owned());
                handler_grammar = Some(profile.grammar.to_owned());
            }
        }
        0x20 | 0x80 | 0xD9..=0xDF | 0xFD..=0xFF => role = "control",
        _ => {}
    }

    LexToken {
        index: 0,
        offset,
        size,
        token_type: token_type.to_owned(),
        role: role.to_owned(),
        raw: hex(&bytes[offset..offset + size]),
        opcode,
        handler_target,
        handler_grammar,
        entry_index: None,
        owner_opcode_index: None,
    }
}

fn lex_full_file(bytes: &[u8]) -> (Vec<LexToken>, Vec<String>) {
    let mut tokens = Vec::new();
    let mut warnings = Vec::new();
    let mut offset = 0usize;
    while offset < bytes.len() {
        let size = token_length(bytes, offset, &mut warnings);
        let mut token = classify_token(bytes, offset, size);
        token.index = tokens.len();
        if token.role == "invalid_sjis" {
            warnings.push(format!(
                "undefined or truncated CP932 token {} at 0x{offset:X}; preserved losslessly",
                token.raw
            ));
        }
        tokens.push(token);
        offset += size;
    }
    (tokens, warnings)
}

fn immediate_size(bytes: &[u8], offset: usize) -> usize {
    match bytes.get(offset).copied() {
        Some(0x23..=0x27) => 1,
        Some(0x28) if offset + 1 < bytes.len() => 2,
        Some(0x29..=0x2C) if offset + 2 < bytes.len() => 3,
        _ => 0,
    }
}

fn mark_operand_range(tokens: &mut [LexToken], start: usize, end: usize, owner: usize, role: &str) {
    for token in tokens {
        let token_end = token.offset + token.size;
        if token.offset < end && token_end > start {
            token.owner_opcode_index.get_or_insert(owner);
            if matches!(
                token.role.as_str(),
                "text" | "expression" | "operand_string"
            ) {
                token.role = role.to_owned();
            }
        }
    }
}

fn claim_exprs(
    bytes: &[u8],
    tokens: &mut [LexToken],
    owner: usize,
    cursor: &mut usize,
    maximum: usize,
) {
    for _ in 0..maximum {
        let size = immediate_size(bytes, *cursor);
        if size == 0 {
            break;
        }
        mark_operand_range(
            tokens,
            *cursor,
            cursor.saturating_add(size),
            owner,
            "handler_expression",
        );
        *cursor += size;
    }
}

fn claim_quote(bytes: &[u8], tokens: &mut [LexToken], owner: usize, cursor: &mut usize) -> bool {
    if bytes.get(*cursor) != Some(&0x22) {
        return false;
    }
    let size = bytes[*cursor + 1..]
        .iter()
        .position(|&byte| byte == 0x22)
        .map_or(bytes.len() - *cursor, |relative| relative + 2);
    mark_operand_range(
        tokens,
        *cursor,
        cursor.saturating_add(size),
        owner,
        "handler_string",
    );
    *cursor += size;
    true
}

fn annotate_handler_operands(bytes: &[u8], tokens: &mut [LexToken]) {
    let opcodes = tokens
        .iter()
        .filter_map(|token| {
            token
                .opcode
                .as_ref()
                .and_then(|opcode| u8::from_str_radix(opcode, 16).ok())
                .map(|opcode| (token.index, opcode, token.offset + token.size))
        })
        .collect::<Vec<_>>();

    for (owner, opcode, mut cursor) in opcodes {
        match opcode {
            0xA6 => claim_exprs(bytes, tokens, owner, &mut cursor, 4),
            0xA7 => claim_exprs(bytes, tokens, owner, &mut cursor, 2),
            0xA8 => claim_exprs(bytes, tokens, owner, &mut cursor, 3),
            0xA9 => claim_exprs(bytes, tokens, owner, &mut cursor, 2),
            0xAA => claim_exprs(bytes, tokens, owner, &mut cursor, 1),
            0xAB | 0xAC => claim_exprs(bytes, tokens, owner, &mut cursor, 2),
            0xAD => claim_exprs(bytes, tokens, owner, &mut cursor, usize::MAX),
            0xAE | 0xAF | 0xB2 => {}
            0xB0 => {
                claim_quote(bytes, tokens, owner, &mut cursor);
                claim_exprs(bytes, tokens, owner, &mut cursor, 2);
            }
            0xB1 => {
                claim_quote(bytes, tokens, owner, &mut cursor);
            }
            0xB3 => mark_operand_range(
                tokens,
                cursor,
                cursor.saturating_add(1),
                owner,
                "handler_operand",
            ),
            0xB4 => mark_operand_range(
                tokens,
                cursor,
                cursor.saturating_add(2),
                owner,
                "handler_operand",
            ),
            0xB5 => {
                claim_exprs(bytes, tokens, owner, &mut cursor, 1);
                mark_operand_range(
                    tokens,
                    cursor,
                    cursor.saturating_add(1),
                    owner,
                    "handler_operand",
                );
            }
            0xB6 => {
                if !claim_quote(bytes, tokens, owner, &mut cursor) {
                    mark_operand_range(
                        tokens,
                        cursor,
                        cursor.saturating_add(1),
                        owner,
                        "handler_operand",
                    );
                    cursor = cursor.saturating_add(1);
                    claim_exprs(bytes, tokens, owner, &mut cursor, 1);
                }
            }
            0xB7 => claim_exprs(bytes, tokens, owner, &mut cursor, 3),
            0xB8 => claim_exprs(bytes, tokens, owner, &mut cursor, 2),
            0xB9..=0xBB => claim_exprs(bytes, tokens, owner, &mut cursor, 1),
            0xBC | 0xBD => {}
            0xBE => claim_exprs(bytes, tokens, owner, &mut cursor, 1),
            0xBF => claim_exprs(bytes, tokens, owner, &mut cursor, 2),
            0xC0 => {
                claim_quote(bytes, tokens, owner, &mut cursor);
                claim_exprs(bytes, tokens, owner, &mut cursor, 1);
            }
            0xC1 | 0xC2 => {
                claim_exprs(bytes, tokens, owner, &mut cursor, 1);
                claim_quote(bytes, tokens, owner, &mut cursor);
            }
            0xC3..=0xC5 => claim_exprs(bytes, tokens, owner, &mut cursor, 2),
            0xC6 => claim_exprs(bytes, tokens, owner, &mut cursor, 5),
            0xC7 => claim_exprs(bytes, tokens, owner, &mut cursor, 1),
            0xC8 => {
                claim_quote(bytes, tokens, owner, &mut cursor);
                claim_exprs(bytes, tokens, owner, &mut cursor, 1);
            }
            0xC9 => {
                claim_quote(bytes, tokens, owner, &mut cursor);
            }
            0xCA => {
                mark_operand_range(
                    tokens,
                    cursor,
                    cursor.saturating_add(1),
                    owner,
                    "handler_operand",
                );
                cursor = cursor.saturating_add(1);
                claim_exprs(bytes, tokens, owner, &mut cursor, 3);
            }
            0xCB => claim_exprs(bytes, tokens, owner, &mut cursor, 1),
            0xCC => claim_exprs(bytes, tokens, owner, &mut cursor, 2),
            0xCD => {
                claim_exprs(bytes, tokens, owner, &mut cursor, 1);
                loop {
                    if claim_quote(bytes, tokens, owner, &mut cursor) {
                        continue;
                    }
                    let before = cursor;
                    claim_exprs(bytes, tokens, owner, &mut cursor, 1);
                    if cursor == before {
                        break;
                    }
                }
            }
            0xCE => claim_exprs(bytes, tokens, owner, &mut cursor, 2),
            0xCF => claim_exprs(bytes, tokens, owner, &mut cursor, 4),
            0xD0 => {
                if bytes
                    .get(cursor..cursor.saturating_add(2))
                    .is_some_and(|value| value.eq_ignore_ascii_case(b"se"))
                {
                    mark_operand_range(tokens, cursor, cursor + 2, owner, "handler_operand");
                    cursor += 2;
                    while bytes.get(cursor) == Some(&0x20) {
                        mark_operand_range(tokens, cursor, cursor + 1, owner, "handler_operand");
                        cursor += 1;
                    }
                    if !claim_quote(bytes, tokens, owner, &mut cursor) {
                        claim_exprs(bytes, tokens, owner, &mut cursor, 1);
                    }
                } else if !claim_quote(bytes, tokens, owner, &mut cursor) {
                    claim_exprs(bytes, tokens, owner, &mut cursor, 2);
                }
            }
            0xD1 => claim_exprs(bytes, tokens, owner, &mut cursor, 4),
            0xD2 | 0xD4 | 0xD6..=0xD8 => {}
            0xD3 => claim_exprs(bytes, tokens, owner, &mut cursor, 2),
            0xD5 => claim_exprs(bytes, tokens, owner, &mut cursor, 1),
            _ => {}
        }
    }
}

fn split_speaker(text: &str) -> (Option<String>, String) {
    let Some(rest) = text.strip_prefix('「') else {
        return (None, text.to_owned());
    };
    let Some(closing) = rest.find('」') else {
        return (None, text.to_owned());
    };
    if closing == 0 {
        return (None, text.to_owned());
    }
    let after_closing = closing + '」'.len_utf8();
    (
        Some(rest[..closing].to_owned()),
        rest[after_closing..].to_owned(),
    )
}

pub fn parse_script(bytes: &[u8], file: String) -> Result<ScriptJson, String> {
    let (mut tokens, warnings) = lex_full_file(bytes);
    annotate_handler_operands(bytes, &mut tokens);
    let mut entries = Vec::new();
    let mut token_index = 0usize;

    while token_index < tokens.len() {
        if tokens[token_index].role != "text" {
            token_index += 1;
            continue;
        }
        let token_start = token_index;
        while token_index < tokens.len() && tokens[token_index].role == "text" {
            token_index += 1;
        }
        let token_end = token_index;
        let offset = tokens[token_start].offset;
        let end = tokens[token_end - 1].offset + tokens[token_end - 1].size;
        let decoded = decode_text(&bytes[offset..end])
            .map_err(|error| format!("{file}+0x{offset:X}: {error}"))?;
        let (scr_name, scr_msg) = split_speaker(&decoded);
        let index = entries.len();
        for token in &mut tokens[token_start..token_end] {
            token.entry_index = Some(index);
        }
        entries.push(TextEntry {
            file: file.clone(),
            index,
            token_start,
            token_end,
            offset,
            size: end - offset,
            entry_type: if scr_name.is_some() {
                "dialogue".to_owned()
            } else {
                "text".to_owned()
            },
            opcode: ENTRY_OPCODE.to_owned(),
            encoding: ENTRY_ENCODING.to_owned(),
            policy: ENTRY_POLICY.to_owned(),
            name: scr_name.clone(),
            scr_name,
            message: scr_msg.clone(),
            scr_msg,
        });
    }

    Ok(ScriptJson {
        format: FORMAT_V3.to_owned(),
        file,
        source_size: bytes.len(),
        encoding: SCRIPT_ENCODING.to_owned(),
        lexer: LEXER_PROFILE.to_owned(),
        handler_profile: HANDLER_PROFILE.to_owned(),
        name_policy: NAME_POLICY.to_owned(),
        entries,
        tokens,
        warnings,
    })
}

fn same_immutable_entry(actual: &TextEntry, expected: &TextEntry) -> bool {
    actual.file == expected.file
        && actual.index == expected.index
        && actual.token_start == expected.token_start
        && actual.token_end == expected.token_end
        && actual.offset == expected.offset
        && actual.size == expected.size
        && actual.entry_type == expected.entry_type
        && actual.opcode == expected.opcode
        && actual.encoding == expected.encoding
        && actual.policy == expected.policy
        && actual.scr_name == expected.scr_name
        && actual.scr_msg == expected.scr_msg
}

fn rebuild_entry(entry: &TextEntry, original: &[u8]) -> Result<Vec<u8>, String> {
    let decoded = decode_text(original)
        .map_err(|error| format!("entry {} source text is invalid: {error}", entry.index))?;
    let (source_name, source_message) = split_speaker(&decoded);
    if entry.scr_name != source_name {
        return Err(format!(
            "entry {} _scr_name does not match source text",
            entry.index
        ));
    }
    if entry.scr_msg != source_message {
        return Err(format!(
            "entry {} scr_msg does not match source text",
            entry.index
        ));
    }

    let translated = match (&source_name, &entry.name) {
        (Some(_), Some(name)) => {
            if name.is_empty() {
                return Err(format!("entry {} name must not be empty", entry.index));
            }
            format!("「{name}」{}", entry.message)
        }
        (Some(_), None) => {
            return Err(format!(
                "entry {} name is required because source has _scr_name",
                entry.index
            ));
        }
        (None, Some(_)) => {
            return Err(format!(
                "entry {} cannot add a name to an unnamed source segment",
                entry.index
            ));
        }
        (None, None) => entry.message.clone(),
    };

    if entry.name == source_name && entry.message == source_message {
        Ok(original.to_vec())
    } else {
        encode_text(&translated).map_err(|error| format!("entry {}: {error}", entry.index))
    }
}

/// Rebuild a MES file from its full-file lexer/IR projection.
///
/// Every source byte is re-lexed and compared with immutable JSON metadata.
/// Only `name` and `message` may change; all other token bytes are copied from
/// the verified source in original order.
pub fn rebuild_script(script: &ScriptJson, source: &[u8]) -> Result<Vec<u8>, String> {
    if script.format != FORMAT_V3
        || script.encoding != SCRIPT_ENCODING
        || script.lexer != LEXER_PROFILE
        || script.handler_profile != HANDLER_PROFILE
        || script.name_policy != NAME_POLICY
    {
        return Err(format!(
            "unsupported or modified MES JSON format metadata; expected {FORMAT_V3:?}"
        ));
    }
    if script.source_size != source.len() {
        return Err(format!(
            "source size mismatch: JSON says {}, source is {}",
            script.source_size,
            source.len()
        ));
    }

    let expected = parse_script(source, script.file.clone())?;
    if script.tokens != expected.tokens {
        return Err("_tokens metadata does not match the source sub_D77C stream".to_owned());
    }
    if script.warnings != expected.warnings {
        return Err("_warnings metadata does not match source analysis".to_owned());
    }
    if script.entries.len() != expected.entries.len() {
        return Err(format!(
            "entry count mismatch: JSON has {}, source has {}",
            script.entries.len(),
            expected.entries.len()
        ));
    }
    for (actual, source_entry) in script.entries.iter().zip(&expected.entries) {
        if !same_immutable_entry(actual, source_entry) {
            return Err(format!(
                "entry {} immutable metadata or source text was modified",
                actual.index
            ));
        }
    }

    let mut output = Vec::with_capacity(source.len());
    let mut token_index = 0usize;
    let mut entry_index = 0usize;
    while token_index < expected.tokens.len() {
        if entry_index < script.entries.len()
            && script.entries[entry_index].token_start == token_index
        {
            let entry = &script.entries[entry_index];
            let original = &source[entry.offset..entry.offset + entry.size];
            output.extend_from_slice(&rebuild_entry(entry, original)?);
            token_index = entry.token_end;
            entry_index += 1;
        } else {
            let token = &expected.tokens[token_index];
            output.extend_from_slice(&source[token.offset..token.offset + token.size]);
            token_index += 1;
        }
    }
    if entry_index != script.entries.len() {
        return Err(format!(
            "entry {entry_index} was not reached by the token stream"
        ));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded(value: &str) -> Vec<u8> {
        encode_text(value).unwrap()
    }

    fn sample() -> Vec<u8> {
        let mut bytes = vec![0xA6, 0x23, 0x23, 0x23, 0x23];
        bytes.extend_from_slice(&encoded("「男の声」そろそろだな"));
        bytes.push(0xA5);
        bytes.extend_from_slice(&encoded("次の行"));
        bytes.extend_from_slice(&[0xFE, 0xFF]);
        bytes
    }

    #[test]
    fn handler_table_covers_every_a6_through_d8_opcode() {
        assert_eq!(HANDLERS.len(), usize::from(0xD8_u8 - 0xA6_u8 + 1));
        for opcode in 0xA6..=0xD8 {
            let profile = opcode_handler_profile(opcode).unwrap();
            assert_eq!(profile.opcode, opcode);
            assert!(!profile.target.is_empty());
            assert!(!profile.grammar.is_empty());
        }
    }

    #[test]
    fn d77c_lengths_cover_all_token_classes() {
        let source = [
            0x21, b'A', 0x00, 0x22, b'B', 0x22, 0x0F, 0x99, 0x10, 0x88, 0x77, 0x23, 0x28, 0x44,
            0x29, 0x34, 0x12, 0x82, 0xA0, 0xA5,
        ];
        let (tokens, warnings) = lex_full_file(&source);
        assert!(warnings.is_empty());
        assert_eq!(
            tokens.iter().map(|token| token.size).collect::<Vec<_>>(),
            vec![3, 3, 2, 3, 1, 2, 3, 2, 1]
        );
        assert_eq!(
            tokens.iter().map(|token| token.size).sum::<usize>(),
            source.len()
        );
    }

    #[test]
    fn unterminated_variable_token_is_preserved_to_eof() {
        let source = [0xA6, 0x22, 0x82, 0xA0];
        let (tokens, warnings) = lex_full_file(&source);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[1].raw, "22 82 A0");
        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("unterminated"))
        );
    }

    #[test]
    fn ba_23_is_opcode_plus_expression_not_a_record_terminator() {
        let mut source = encoded("前");
        source.extend_from_slice(&[0xBA, 0x23]);
        source.extend_from_slice(&encoded("後"));
        let parsed = parse_script(&source, "TEST.MES".to_owned()).unwrap();
        assert_eq!(parsed.entries.len(), 2);
        assert_eq!(parsed.entries[0].scr_msg, "前");
        assert_eq!(parsed.entries[1].scr_msg, "後");
        let ba = parsed
            .tokens
            .iter()
            .find(|token| token.opcode.as_deref() == Some("BA"))
            .unwrap();
        assert_eq!(ba.handler_target.as_deref(), Some("D00F"));
        assert_eq!(parsed.tokens[ba.index + 1].role, "handler_expression");
        assert_eq!(
            parsed.tokens[ba.index + 1].owner_opcode_index,
            Some(ba.index)
        );
    }

    #[test]
    fn parsing_covers_every_source_byte_without_trailing_bucket() {
        let source = sample();
        let parsed = parse_script(&source, "TEST.MES".to_owned()).unwrap();
        let mut offset = 0usize;
        for token in &parsed.tokens {
            assert_eq!(token.offset, offset);
            offset += token.size;
        }
        assert_eq!(offset, source.len());
        assert_eq!(parsed.entries.len(), 2);
    }

    #[test]
    fn unchanged_rebuild_is_byte_exact() {
        let source = sample();
        let parsed = parse_script(&source, "TEST.MES".to_owned()).unwrap();
        assert_eq!(rebuild_script(&parsed, &source).unwrap(), source);
    }

    #[test]
    fn message_can_grow_and_shrink_without_touching_controls() {
        let source = sample();
        let mut parsed = parse_script(&source, "TEST.MES".to_owned()).unwrap();
        parsed.entries[0].message = "まだまだ、そろそろだな".to_owned();
        parsed.entries[1].message = "行".to_owned();
        let rebuilt = rebuild_script(&parsed, &source).unwrap();
        let reparsed = parse_script(&rebuilt, "TEST.MES".to_owned()).unwrap();
        assert_eq!(reparsed.entries[0].scr_msg, "まだまだ、そろそろだな");
        assert_eq!(reparsed.entries[1].scr_msg, "行");
        assert!(
            rebuilt
                .windows(5)
                .any(|window| window == [0xA6, 0x23, 0x23, 0x23, 0x23])
        );
        assert!(rebuilt.ends_with(&[0xFE, 0xFF]));
    }

    #[test]
    fn immutable_token_metadata_is_rejected_when_modified() {
        let source = sample();
        let mut parsed = parse_script(&source, "TEST.MES".to_owned()).unwrap();
        parsed.tokens[0].raw = "00".to_owned();
        let error = rebuild_script(&parsed, &source).unwrap_err();
        assert!(error.contains("_tokens"));
    }

    #[test]
    fn immutable_source_text_is_rejected_when_modified() {
        let source = sample();
        let mut parsed = parse_script(&source, "TEST.MES".to_owned()).unwrap();
        parsed.entries[0].scr_msg = "改竄".to_owned();
        let error = rebuild_script(&parsed, &source).unwrap_err();
        assert!(error.contains("immutable metadata"));
    }

    #[test]
    fn modified_scr_name_is_rejected() {
        let source = sample();
        let mut parsed = parse_script(&source, "TEST.MES".to_owned()).unwrap();
        parsed.entries[0].scr_name = Some("別人".to_owned());
        let error = rebuild_script(&parsed, &source).unwrap_err();
        assert!(error.contains("immutable metadata"));
    }

    #[test]
    fn unnamed_text_cannot_gain_a_name() {
        let source = encoded("本文");
        let mut parsed = parse_script(&source, "TEST.MES".to_owned()).unwrap();
        parsed.entries[0].name = Some("人物".to_owned());
        let error = rebuild_script(&parsed, &source).unwrap_err();
        assert!(error.contains("cannot add a name"));
    }

    #[test]
    fn newline_nul_unencodable_and_bytecode_colliding_text_are_rejected() {
        let source = encoded("本文");
        for (message, expected) in [
            ("改\n行", "CR/LF"),
            ("改\0行", "NUL"),
            ("😀", "U+1F600"),
            ("ASCII", "collides with MES bytecode"),
        ] {
            let mut parsed = parse_script(&source, "TEST.MES".to_owned()).unwrap();
            parsed.entries[0].message = message.to_owned();
            let error = rebuild_script(&parsed, &source).unwrap_err();
            assert!(error.contains(expected), "unexpected error: {error}");
        }
    }

    #[test]
    fn mapped_chinese_preserves_shift_jis_character_boundaries() {
        for value in [
            "像這樣定定地眺望着，那点想要出門游蕩的浮躁心气，似乎也漸漸衰竭了。",
            "那是一幅浮雕，描絵着不知是魚還是人的異形之物；加德納堅称，這就是這神殿供奉的神祇。",
        ] {
            let bytes = encode_text(value).unwrap();
            assert_eq!(decode_text(&bytes).unwrap(), value);
        }
    }

    #[test]
    fn translator_entries_do_not_contain_parts_arrays() {
        let json =
            serde_json::to_value(parse_script(&sample(), "TEST.MES".to_owned()).unwrap()).unwrap();
        let entry = &json["entries"][0];
        assert!(entry.get("scr_msg_parts").is_none());
        assert!(entry.get("message_parts").is_none());
        assert!(entry.get("_parts").is_none());
        assert!(json.get("_tokens").is_some());
        assert!(json.get("_records").is_none());
        assert!(json.get("_trailing").is_none());
    }
}
