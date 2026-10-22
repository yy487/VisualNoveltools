//! Loss-preserving projection of the PS1SYS slot-54 text byte stream.
//!
//! The parser operates on the expanded module bytes and must start at a pointer
//! established by code/data analysis. It is intentionally not a CP932 string
//! scanner: the runtime consumes lead-byte pairs before testing the trail byte
//! for controls or NUL, and maps single bytes through its own JIS tables.

use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};

/// PS1SYS table at linear address 0x14834, indexed by bytes 0x20..=0x7F.
/// Values are JIS row/cell codes read from little-endian words.
const ASCII_JIS: [u16; 96] = [
    0x2121, 0x212A, 0x2149, 0x2174, 0x2170, 0x2173, 0x2175, 0x212D, 0x214A, 0x214B, 0x2176, 0x215C,
    0x2124, 0x213D, 0x2125, 0x213F, 0x2330, 0x2331, 0x2332, 0x2333, 0x2334, 0x2335, 0x2336, 0x2337,
    0x2338, 0x2339, 0x2127, 0x2128, 0x2163, 0x2161, 0x2164, 0x2129, 0x2177, 0x2341, 0x2342, 0x2343,
    0x2344, 0x2345, 0x2346, 0x2347, 0x2348, 0x2349, 0x234A, 0x234B, 0x234C, 0x234D, 0x234E, 0x234F,
    0x2350, 0x2351, 0x2352, 0x2353, 0x2354, 0x2355, 0x2356, 0x2357, 0x2358, 0x2359, 0x235A, 0x214E,
    0x216F, 0x214F, 0x2130, 0x2132, 0x212E, 0x2361, 0x2362, 0x2363, 0x2364, 0x2365, 0x2366, 0x2367,
    0x2368, 0x2369, 0x236A, 0x236B, 0x236C, 0x236D, 0x236E, 0x236F, 0x2370, 0x2371, 0x2372, 0x2373,
    0x2374, 0x2375, 0x2376, 0x2377, 0x2378, 0x2379, 0x237A, 0x2150, 0x2143, 0x2151, 0x2131, 0x2121,
];

/// PS1SYS table at linear address 0x148F4, indexed by bytes 0xA0..=0xDF.
const KANA_JIS: [u16; 64] = [
    0x2121, 0x2123, 0x2156, 0x2157, 0x2122, 0x2126, 0x2472, 0x2421, 0x2423, 0x2425, 0x2427, 0x2429,
    0x2463, 0x2465, 0x2467, 0x2443, 0x213C, 0x2422, 0x2424, 0x2426, 0x2428, 0x242A, 0x242B, 0x242D,
    0x242F, 0x2431, 0x2433, 0x2435, 0x2437, 0x2439, 0x243B, 0x243D, 0x243F, 0x2441, 0x2444, 0x2446,
    0x2448, 0x244A, 0x244B, 0x244C, 0x244D, 0x244E, 0x244F, 0x2452, 0x2455, 0x2458, 0x245B, 0x245E,
    0x245F, 0x2460, 0x2461, 0x2462, 0x2464, 0x2466, 0x2468, 0x2469, 0x246A, 0x246B, 0x246C, 0x246D,
    0x246F, 0x2473, 0x212B, 0x212C,
];

const DEFAULT_SINGLE_BYTE_JIS: u16 = 0x2223;

/// Complete slot-54 string record, located in an expanded module.
///
/// `raw_bytes` excludes the outer NUL terminator but includes any NUL byte
/// consumed as the trail of a preceding CP932 lead byte. `tokens` partition
/// those bytes in the exact order consumed by PS1SYS. `source_end` points just
/// past the outer terminator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextRecord {
    pub source_offset: usize,
    pub source_end: usize,
    pub terminator_offset: usize,
    pub terminator: u8,
    pub raw_bytes: Vec<u8>,
    pub tokens: Vec<TextToken>,
    /// Reversible translator-facing projection. Runtime substitutions remain
    /// explicit tokens and are never expanded during parsing.
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextToken {
    /// Absolute offset within the expanded module.
    pub source_offset: usize,
    /// Exact source bytes consumed by this token.
    pub raw_bytes: Vec<u8>,
    pub kind: TextTokenKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TextTokenKind {
    Glyph { jis_code: u16, text: String },
    CursorAlign,
    ClearWindow,
    WaitInput,
    Substitution { slot: u8 },
    WaitParameter { value: u8 },
    GlyphWaitMode,
    AlternateWaitMode,
    Newline,
    Raw { reason: RawReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawReason {
    UnmappedGlyph,
    SkippedControl,
    MalformedControlPrefix,
}

/// Parse one NUL-terminated PS1SYS text argument from a fully expanded module.
///
/// The caller must provide a pointer obtained from a verified API call/table.
/// The parser stops at the first NUL seen by the outer scanner. A NUL directly
/// following a CP932 lead byte is consumed as that pair's second byte, exactly
/// as in `sub_11A49`.
pub fn parse_message(source: &[u8], start_offset: usize) -> Result<TextRecord, String> {
    if start_offset > source.len() {
        return Err(format!(
            "expanded-module offset 0x{start_offset:X} is past length 0x{:X}",
            source.len()
        ));
    }

    let mut cursor = start_offset;
    let mut tokens = Vec::new();
    let mut message = String::new();

    loop {
        let Some(&byte) = source.get(cursor) else {
            return Err(format!(
                "expanded-module offset 0x{start_offset:X}: text has no outer NUL before module end 0x{:X}",
                source.len()
            ));
        };
        if byte == 0 {
            return Ok(TextRecord {
                source_offset: start_offset,
                source_end: cursor + 1,
                terminator_offset: cursor,
                terminator: 0,
                raw_bytes: source[start_offset..cursor].to_vec(),
                tokens,
                message,
            });
        }

        // The game tests for a DBCS lead before looking for any controls. It
        // consumes the following byte unconditionally, including NUL or a
        // byte that would otherwise be a control character.
        if is_cp932_lead(byte) {
            let Some(&trail) = source.get(cursor + 1) else {
                return Err(format!(
                    "expanded-module offset 0x{cursor:X}: CP932 lead byte {byte:02X} has no following byte"
                ));
            };
            let pair = [byte, trail];
            if let Some((jis_code, text)) = decode_cp932_pair(pair) {
                push_token(
                    &mut tokens,
                    &mut message,
                    cursor,
                    &pair,
                    TextTokenKind::Glyph { jis_code, text },
                );
            } else {
                push_raw(
                    &mut tokens,
                    &mut message,
                    cursor,
                    &pair,
                    RawReason::UnmappedGlyph,
                );
            }
            cursor += 2;
            continue;
        }

        match byte {
            b'!' => {
                push_token(
                    &mut tokens,
                    &mut message,
                    cursor,
                    &[byte],
                    TextTokenKind::CursorAlign,
                );
                cursor += 1;
            }
            b'#' => {
                push_token(
                    &mut tokens,
                    &mut message,
                    cursor,
                    &[byte],
                    TextTokenKind::ClearWindow,
                );
                cursor += 1;
            }
            b'@' => {
                push_token(
                    &mut tokens,
                    &mut message,
                    cursor,
                    &[byte],
                    TextTokenKind::WaitInput,
                );
                cursor += 1;
            }
            b'$' => {
                if let Some(&digit @ b'0'..=b'9') = source.get(cursor + 1) {
                    push_token(
                        &mut tokens,
                        &mut message,
                        cursor,
                        &[byte, digit],
                        TextTokenKind::Substitution { slot: digit - b'0' },
                    );
                    cursor += 2;
                } else {
                    cursor = consume_malformed_prefix(source, cursor, &mut tokens, &mut message)?;
                }
            }
            0x04 => {
                if let Some(&digit @ b'0'..=b'9') = source.get(cursor + 1) {
                    push_token(
                        &mut tokens,
                        &mut message,
                        cursor,
                        &[byte, digit],
                        TextTokenKind::WaitParameter {
                            value: digit - b'0',
                        },
                    );
                    cursor += 2;
                } else {
                    cursor = consume_malformed_prefix(source, cursor, &mut tokens, &mut message)?;
                }
            }
            0x05 => {
                push_token(
                    &mut tokens,
                    &mut message,
                    cursor,
                    &[byte],
                    TextTokenKind::GlyphWaitMode,
                );
                cursor += 1;
            }
            0x06 => {
                push_token(
                    &mut tokens,
                    &mut message,
                    cursor,
                    &[byte],
                    TextTokenKind::AlternateWaitMode,
                );
                cursor += 1;
            }
            0x0A => {
                push_token(
                    &mut tokens,
                    &mut message,
                    cursor,
                    &[byte],
                    TextTokenKind::Newline,
                );
                cursor += 1;
            }
            0x01..=0x1F => {
                push_raw(
                    &mut tokens,
                    &mut message,
                    cursor,
                    &[byte],
                    RawReason::SkippedControl,
                );
                cursor += 1;
            }
            _ => {
                let jis_code = single_byte_jis(byte);
                if let Some(text) = decode_jis_code(jis_code) {
                    push_token(
                        &mut tokens,
                        &mut message,
                        cursor,
                        &[byte],
                        TextTokenKind::Glyph { jis_code, text },
                    );
                } else {
                    push_raw(
                        &mut tokens,
                        &mut message,
                        cursor,
                        &[byte],
                        RawReason::UnmappedGlyph,
                    );
                }
                cursor += 1;
            }
        }
    }
}

/// Encode a translator-facing message back to the parser's source bytes.
///
/// The returned bytes exclude the outer NUL. An unchanged message reuses the
/// exact original payload. `capacity` is the maximum payload size supplied by
/// the caller; pass the original byte length when enabling fixed-span edits.
/// The result is parsed again and rejected unless both its visible/token view
/// and complete payload boundary round-trip.
pub fn encode_message(
    message: &str,
    original: &TextRecord,
    capacity: usize,
) -> Result<Vec<u8>, String> {
    if message == original.message {
        if original.raw_bytes.len() > capacity {
            return Err(format!(
                "text at expanded-module offset 0x{:X} exceeds capacity: {} > {} bytes",
                original.source_offset,
                original.raw_bytes.len(),
                capacity
            ));
        }
        return Ok(original.raw_bytes.clone());
    }
    if original.tokens.iter().any(|token| {
        matches!(
            &token.kind,
            TextTokenKind::Raw {
                reason: RawReason::MalformedControlPrefix
            }
        )
    }) {
        return Err(format!(
            "text at expanded-module offset 0x{:X} contains a malformed control prefix and is read-only",
            original.source_offset
        ));
    }

    let mut output = Vec::new();
    let mut canonical = String::with_capacity(message.len());
    let mut cursor = 0usize;

    while cursor < message.len() {
        let rest = &message[cursor..];
        if rest.starts_with("[[") {
            let close = rest.find("]]").ok_or_else(|| {
                format!("malformed token at message byte 0x{cursor:X}: missing ]]")
            })?;
            let token_end = close + 2;
            let interior = &rest[2..close];
            let (encoded, normalized) = encode_token(interior, cursor)?;
            output.extend_from_slice(&encoded);
            canonical.push_str(&normalized);
            cursor += token_end;
            continue;
        }

        let character = rest
            .chars()
            .next()
            .ok_or_else(|| format!("invalid UTF-8 cursor at message byte 0x{cursor:X}"))?;
        if character == '\n' {
            output.push(0x0A);
            canonical.push('\n');
        } else if character.is_control() {
            return Err(format!(
                "unsupported control character U+{:04X} at message byte 0x{cursor:X}; use a documented CYBER token or RAW marker",
                character as u32
            ));
        } else {
            output.extend_from_slice(&encode_visible_character(character, cursor)?);
            canonical.push(character);
        }
        cursor += character.len_utf8();
    }

    if output.len() > capacity {
        return Err(format!(
            "encoded message exceeds capacity: {} > {capacity} bytes",
            output.len()
        ));
    }

    let mut terminated = output.clone();
    terminated.push(0);
    let roundtrip = parse_message(&terminated, 0)
        .map_err(|error| format!("encoded message is not valid PS1SYS text: {error}"))?;
    if roundtrip.raw_bytes != output || roundtrip.message != canonical {
        return Err(
            "encoded message does not round-trip through the PS1SYS CP932/JIS and control parser"
                .into(),
        );
    }
    Ok(output)
}

/// Encode one translator-facing text line as editable PS1SYS glyph atoms.
///
/// Presentation controls live in the source-bound projection plan and are not
/// accepted here. `{{SUB:n}}` is an inline runtime substitution; `[[RAW:HEX]]`
/// is accepted only when those exact bytes were already present in the source
/// segment. The caller inserts the returned atoms around anchored controls.
pub fn encode_projection_atoms(
    message: &str,
    allowed_raw: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>, String> {
    encode_projection_atoms_with(message, allowed_raw, encode_visible_character)
}

/// Encode one projection using a caller-supplied glyph encoder. The source-bound
/// adapter can use this to map unsupported Unicode glyphs onto patched PC-98
/// font carriers while keeping SUB and RAW tokens under this parser's control.
pub fn encode_projection_atoms_with(
    message: &str,
    allowed_raw: &[Vec<u8>],
    mut encode_character: impl FnMut(char, usize) -> Result<Vec<u8>, String>,
) -> Result<Vec<Vec<u8>>, String> {
    let mut atoms = Vec::new();
    let mut cursor = 0usize;

    while cursor < message.len() {
        let rest = &message[cursor..];
        if let Some(after_prefix) = rest.strip_prefix("{{SUB:") {
            let close = after_prefix.find("}}").ok_or_else(|| {
                format!("malformed substitution placeholder at message byte 0x{cursor:X}")
            })?;
            let value = &after_prefix[..close];
            if value.len() != 1 || !value.as_bytes()[0].is_ascii_digit() {
                return Err(format!(
                    "invalid substitution placeholder {{{{SUB:{value}}}}} at message byte 0x{cursor:X}"
                ));
            }
            atoms.push(vec![b'$', value.as_bytes()[0]]);
            cursor += "{{SUB:".len() + close + "}}".len();
            continue;
        }

        if let Some(after_prefix) = rest.strip_prefix("[[RAW:") {
            let close = after_prefix
                .find("]]")
                .ok_or_else(|| format!("malformed RAW marker at message byte 0x{cursor:X}"))?;
            let hex = &after_prefix[..close];
            let raw = parse_hex_bytes(hex)
                .ok_or_else(|| format!("invalid RAW marker [[RAW:{hex}]] at byte 0x{cursor:X}"))?;
            if !allowed_raw.iter().any(|allowed| allowed == &raw) {
                return Err(format!(
                    "RAW bytes [[RAW:{}]] were not present in this source segment",
                    hex_bytes(&raw)
                ));
            }
            atoms.push(raw);
            cursor += "[[RAW:".len() + close + "]]".len();
            continue;
        }

        if rest.starts_with("[[CYBER:") {
            return Err(format!(
                "presentation control markers do not belong in the translation message at byte 0x{cursor:X}"
            ));
        }

        let character = rest
            .chars()
            .next()
            .ok_or_else(|| format!("invalid UTF-8 cursor at message byte 0x{cursor:X}"))?;
        if character == '\n' || character.is_control() {
            return Err(format!(
                "line breaks and control characters must remain outside text atoms (U+{:04X} at byte 0x{cursor:X})",
                character as u32
            ));
        }
        atoms.push(encode_character(character, cursor)?);
        cursor += character.len_utf8();
    }

    Ok(atoms)
}

pub(crate) fn encode_native_projection_character(
    character: char,
    message_offset: usize,
) -> Result<Vec<u8>, String> {
    encode_visible_character(character, message_offset)
}

fn encode_token(interior: &str, message_offset: usize) -> Result<(Vec<u8>, String), String> {
    let bad = || {
        format!("unknown or malformed token [[{interior}]] at message byte 0x{message_offset:X}")
    };
    let token = match interior {
        "CYBER:CURSOR_ALIGN" => return Ok((vec![b'!'], "[[CYBER:CURSOR_ALIGN]]".into())),
        "CYBER:CLEAR_WINDOW" => return Ok((vec![b'#'], "[[CYBER:CLEAR_WINDOW]]".into())),
        "CYBER:WAIT_INPUT" => return Ok((vec![b'@'], "[[CYBER:WAIT_INPUT]]".into())),
        "CYBER:GLYPH_WAIT_MODE" => return Ok((vec![0x05], "[[CYBER:GLYPH_WAIT_MODE]]".into())),
        "CYBER:ALTERNATE_WAIT_MODE" => {
            return Ok((vec![0x06], "[[CYBER:ALTERNATE_WAIT_MODE]]".into()))
        }
        _ => interior,
    };

    if let Some(value) = token.strip_prefix("CYBER:SUB:") {
        if value.len() == 1 && value.as_bytes()[0].is_ascii_digit() {
            let digit = value.as_bytes()[0];
            return Ok((
                vec![b'$', digit],
                format!("[[CYBER:SUB:{}]]", digit as char),
            ));
        }
        return Err(bad());
    }
    if let Some(value) = token.strip_prefix("CYBER:WAIT_PARAMETER:") {
        if value.len() == 1 && value.as_bytes()[0].is_ascii_digit() {
            let digit = value.as_bytes()[0];
            return Ok((
                vec![0x04, digit],
                format!("[[CYBER:WAIT_PARAMETER:{}]]", digit as char),
            ));
        }
        return Err(bad());
    }
    if let Some(hex) = token.strip_prefix("RAW:") {
        let bytes = parse_hex_bytes(hex).ok_or_else(bad)?;
        let normalized = format!("[[RAW:{}]]", hex_bytes(&bytes));
        return Ok((bytes, normalized));
    }
    Err(bad())
}

fn parse_hex_bytes(hex: &str) -> Option<Vec<u8>> {
    if hex.is_empty() || hex.len() % 2 != 0 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let mut result = Vec::with_capacity(hex.len() / 2);
    for pair in hex.as_bytes().chunks_exact(2) {
        let high = hex_nibble(pair[0])?;
        let low = hex_nibble(pair[1])?;
        result.push((high << 4) | low);
    }
    Some(result)
}

fn encode_visible_character(character: char, message_offset: usize) -> Result<Vec<u8>, String> {
    let value = character.to_string();
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&value);
    if had_errors {
        return Err(format!(
            "character {character:?} at message byte 0x{message_offset:X} is not representable in CP932"
        ));
    }

    let bytes = encoded.as_ref();
    if bytes.len() == 1 && matches!(bytes[0], b'!' | b'#' | b'@' | b'$') {
        return Err(format!(
            "character {character:?} at message byte 0x{message_offset:X} is parsed as a PS1SYS control; use its full-width glyph"
        ));
    }
    let jis_code = match bytes {
        [single] => single_byte_jis(*single),
        [lead, trail] if is_cp932_lead(*lead) && is_cp932_trail(*trail) => {
            cp932_to_jis([*lead, *trail]).ok_or_else(|| {
                format!(
                    "character {character:?} at message byte 0x{message_offset:X} has an unsupported CP932 pair"
                )
            })?
        }
        _ => {
            return Err(format!(
                "character {character:?} at message byte 0x{message_offset:X} does not encode as one PS1SYS glyph"
            ));
        }
    };

    let Some(display) = decode_jis_code(jis_code) else {
        return Err(format!(
            "character {character:?} at message byte 0x{message_offset:X} maps to unsupported JIS code {jis_code:04X}"
        ));
    };
    let display_character = display.chars().next();
    if display.chars().count() != 1
        || (display_character != Some(character)
            && !is_fullwidth_ascii_alias(character, display_character))
    {
        return Err(format!(
            "character {character:?} at message byte 0x{message_offset:X} is not reproduced by the PS1SYS JIS mapping (it displays as {display:?})"
        ));
    }

    if bytes.len() == 2 {
        let roundtrip = jis_to_cp932(jis_code).ok_or_else(|| {
            format!(
                "character {character:?} at message byte 0x{message_offset:X} has no standard JIS-to-CP932 encoding"
            )
        })?;
        if roundtrip != [bytes[0], bytes[1]] {
            return Err(format!(
                "character {character:?} at message byte 0x{message_offset:X} uses a non-canonical CP932 extension; use a source RAW token where applicable"
            ));
        }
    }

    Ok(bytes.to_vec())
}

fn is_fullwidth_ascii_alias(character: char, display: Option<char>) -> bool {
    if !character.is_ascii() {
        return false;
    }
    let expected = if character == ' ' {
        Some('　')
    } else if character == '-' {
        // PS1SYS maps the single-byte hyphen to JIS 0x213D (horizontal bar).
        Some('―')
    } else if ('!'..='~').contains(&character) {
        char::from_u32(character as u32 + 0xFEE0)
    } else {
        None
    };
    expected.is_some() && display == expected
}

fn push_token(
    tokens: &mut Vec<TextToken>,
    message: &mut String,
    source_offset: usize,
    raw_bytes: &[u8],
    kind: TextTokenKind,
) {
    message.push_str(&token_view(&kind, raw_bytes));
    tokens.push(TextToken {
        source_offset,
        raw_bytes: raw_bytes.to_vec(),
        kind,
    });
}

fn push_raw(
    tokens: &mut Vec<TextToken>,
    message: &mut String,
    source_offset: usize,
    raw_bytes: &[u8],
    reason: RawReason,
) {
    push_token(
        tokens,
        message,
        source_offset,
        raw_bytes,
        TextTokenKind::Raw { reason },
    );
}

fn consume_malformed_prefix(
    source: &[u8],
    cursor: usize,
    tokens: &mut Vec<TextToken>,
    message: &mut String,
) -> Result<usize, String> {
    let following = cursor + 1;
    let end = match source.get(following).copied() {
        None => {
            return Err(format!(
                "malformed control prefix at expanded-module offset 0x{cursor:X} has no following byte"
            ));
        }
        Some(0) => following,
        Some(byte) if is_cp932_lead(byte) => {
            let end = following + 2;
            if end > source.len() {
                return Err(format!(
                    "malformed control prefix at expanded-module offset 0x{cursor:X} has a truncated following glyph"
                ));
            }
            end
        }
        Some(_) => following + 1,
    };

    push_raw(
        tokens,
        message,
        cursor,
        &source[cursor..end],
        RawReason::MalformedControlPrefix,
    );
    Ok(end)
}

fn token_view(kind: &TextTokenKind, raw_bytes: &[u8]) -> String {
    match kind {
        TextTokenKind::Glyph { text, .. } => text.clone(),
        TextTokenKind::CursorAlign => "[[CYBER:CURSOR_ALIGN]]".into(),
        TextTokenKind::ClearWindow => "[[CYBER:CLEAR_WINDOW]]".into(),
        TextTokenKind::WaitInput => "[[CYBER:WAIT_INPUT]]".into(),
        TextTokenKind::Substitution { slot } => format!("[[CYBER:SUB:{slot}]]"),
        TextTokenKind::WaitParameter { value } => format!("[[CYBER:WAIT_PARAMETER:{value}]]"),
        TextTokenKind::GlyphWaitMode => "[[CYBER:GLYPH_WAIT_MODE]]".into(),
        TextTokenKind::AlternateWaitMode => "[[CYBER:ALTERNATE_WAIT_MODE]]".into(),
        TextTokenKind::Newline => "\n".into(),
        TextTokenKind::Raw { .. } => format!("[[RAW:{}]]", hex_bytes(raw_bytes)),
    }
}

fn single_byte_jis(byte: u8) -> u16 {
    match byte {
        0x20..=0x7F => ASCII_JIS[usize::from(byte - 0x20)],
        0xA0..=0xDF => KANA_JIS[usize::from(byte - 0xA0)],
        _ => DEFAULT_SINGLE_BYTE_JIS,
    }
}

fn decode_cp932_pair(pair: [u8; 2]) -> Option<(u16, String)> {
    if !is_cp932_lead(pair[0]) || !is_cp932_trail(pair[1]) {
        return None;
    }
    let jis_code = cp932_to_jis(pair)?;
    let canonical_pair = jis_to_cp932(jis_code)?;
    if canonical_pair != pair {
        return None;
    }
    let text = decode_jis_code(jis_code)?;
    if text.chars().count() != 1 {
        return None;
    }
    Some((jis_code, text))
}

fn decode_jis_code(jis_code: u16) -> Option<String> {
    let pair = jis_to_cp932(jis_code)?;
    let decoded = SHIFT_JIS.decode_without_bom_handling_and_without_replacement(&pair)?;
    if decoded.chars().count() != 1 {
        return None;
    }
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&decoded);
    if had_errors || encoded.as_ref() != pair {
        return None;
    }
    Some(decoded.into_owned())
}

/// Convert one standard CP932/Shift-JIS double-byte pair to a JIS row/cell.
fn cp932_to_jis(pair: [u8; 2]) -> Option<u16> {
    let [lead, trail] = pair;
    if !is_cp932_lead(lead) || !is_cp932_trail(trail) {
        return None;
    }

    let lead_index = if lead <= 0x9F {
        lead.checked_sub(0x81)?
    } else {
        lead.checked_sub(0xC1)?
    };
    let mut row = 0x21u8.checked_add(lead_index.checked_mul(2)?)?;
    let cell = if trail >= 0x9F {
        row = row.checked_add(1)?;
        trail.checked_sub(0x7E)?
    } else if trail > 0x7E {
        trail.checked_sub(0x20)?
    } else {
        trail.checked_sub(0x1F)?
    };
    Some((u16::from(row) << 8) | u16::from(cell))
}

/// Convert a standard JIS X 0208 row/cell code to its canonical CP932 pair.
fn jis_to_cp932(jis_code: u16) -> Option<[u8; 2]> {
    let row = (jis_code >> 8) as u8;
    let cell = jis_code as u8;
    if !(0x21..=0x7E).contains(&row) || !(0x21..=0x7E).contains(&cell) {
        return None;
    }

    let lead_base = 0x81u8.checked_add((row - 0x21) / 2)?;
    let lead = if lead_base >= 0xA0 {
        lead_base.checked_add(0x40)?
    } else {
        lead_base
    };
    let trail = if row & 1 == 1 {
        if cell < 0x60 {
            cell.checked_add(0x1F)?
        } else {
            cell.checked_add(0x20)?
        }
    } else {
        cell.checked_add(0x7E)?
    };
    Some([lead, trail])
}

fn is_cp932_lead(byte: u8) -> bool {
    (0x81..=0x9F).contains(&byte) || (0xE0..=0xFC).contains(&byte)
}

fn is_cp932_trail(byte: u8) -> bool {
    (0x40..=0x7E).contains(&byte) || (0x80..=0xFC).contains(&byte)
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(result, "{byte:02X}");
    }
    result
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
