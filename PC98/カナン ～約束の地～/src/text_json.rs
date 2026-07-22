use crate::script::{
    encode_cp932_text, shift_jis_pair_to_jis, GameGlyph, ScriptError, TextControl, TextPart,
    TextStream,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

pub const TEXT_JSON_FORMAT: &str = "canaan-sys98-text-v2";
pub const CHANNEL_8_LINE_UNITS: usize = 60;
pub const CHANNEL_9_LINE_UNITS: usize = 70;

#[derive(Debug, Error)]
pub enum TextJsonError {
    #[error(transparent)]
    Script(#[from] ScriptError),
    #[error("invalid translation JSON: {0}")]
    Invalid(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationFile {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_file")]
    pub file: String,
    pub entries: Vec<TranslationEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationSegment {
    #[serde(rename = "_inst_offset")]
    pub instruction_offset: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_opcode")]
    pub opcode: String,
    #[serde(rename = "_part_start")]
    pub part_start: usize,
    #[serde(rename = "_part_end")]
    pub part_end: usize,
    #[serde(rename = "_page_index")]
    pub page_index: usize,
    #[serde(rename = "_page_count")]
    pub page_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_type")]
    pub text_type: String,
    #[serde(rename = "_channel")]
    pub channel: u16,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_segments")]
    pub segments: Vec<TranslationSegment>,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedMessage {
    pub stream: Vec<u8>,
    pub max_line_units: usize,
    pub line_count: usize,
}

#[derive(Debug, Default)]
pub(crate) struct ParsedMessage {
    pub bytes: Vec<u8>,
    pub structural_controls: Vec<Vec<u8>>,
    pub newline_count: usize,
    pub page_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EncodedToken {
    pub bytes: Vec<u8>,
    pub units: usize,
    pub control: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EncodedPage {
    pub bytes: Vec<u8>,
    pub tokens: Vec<EncodedToken>,
    pub max_line_units: usize,
    pub line_count: usize,
}

fn line_limit(channel: u16) -> Result<usize, TextJsonError> {
    match channel {
        8 => Ok(CHANNEL_8_LINE_UNITS),
        9 => Ok(CHANNEL_9_LINE_UNITS),
        _ => Err(TextJsonError::Invalid(format!(
            "unsupported main-story text channel {channel}"
        ))),
    }
}

fn trailing_hidden_control_count(stream: &TextStream) -> usize {
    stream
        .parts
        .iter()
        .rev()
        .take_while(|part| {
            matches!(part, TextPart::Control(control) if matches!(control.code, 0x01 | 0x02))
        })
        .count()
}

pub(crate) fn control_marker(control: &TextControl) -> String {
    match (control.code, control.selector, control.arguments.as_slice()) {
        (0x01, None, []) => "[[PAGE]]".to_owned(),
        (0x02, None, []) => "[[WAIT]]".to_owned(),
        (0x04, None, [slot]) => format!("[[VAR:{slot:02X}]]"),
        _ => format!("[[CTRL:{}]]", hex::encode_upper(control.encoded())),
    }
}

pub(crate) fn glyph_marker(glyph: &GameGlyph) -> String {
    match glyph.jis_code {
        Some(code) => format!("[[G:{code:04X}]]"),
        None => format!("[[GRAW:{}]]", hex::encode_upper(&glyph.bytes)),
    }
}

pub fn render_message(stream: &TextStream) -> String {
    let visible_parts = stream
        .parts
        .len()
        .saturating_sub(trailing_hidden_control_count(stream));
    let mut output = String::new();
    for part in &stream.parts[..visible_parts] {
        match part {
            TextPart::Text(segment) => output.push_str(&segment.text),
            TextPart::Control(control) if control.code == 0x0D => output.push('\n'),
            TextPart::Control(control) => output.push_str(&control_marker(control)),
            TextPart::Glyph(glyph) => output.push_str(&glyph_marker(glyph)),
        }
    }
    output
}

fn source_structure(stream: &TextStream) -> (Vec<Vec<u8>>, usize, usize, Vec<u8>) {
    let hidden = trailing_hidden_control_count(stream);
    let visible_parts = stream.parts.len().saturating_sub(hidden);
    let mut controls = Vec::new();
    let mut newlines = 0usize;
    let mut pages = 0usize;
    for part in &stream.parts[..visible_parts] {
        if let TextPart::Control(control) = part {
            if control.code == 0x0D {
                newlines += 1;
            } else if control.code == 0x01 {
                pages += 1;
            } else {
                controls.push(control.encoded());
            }
        }
    }
    let mut suffix = Vec::new();
    for part in &stream.parts[visible_parts..] {
        if let TextPart::Control(control) = part {
            suffix.extend_from_slice(&control.encoded());
        }
    }
    (controls, newlines, pages, suffix)
}

fn parse_hex(marker: &str, context: &str) -> Result<Vec<u8>, TextJsonError> {
    if marker.is_empty() || !marker.len().is_multiple_of(2) || !marker.is_ascii() {
        return Err(TextJsonError::Invalid(format!(
            "{context} must contain an even number of hexadecimal digits"
        )));
    }
    hex::decode(marker).map_err(|_| {
        TextJsonError::Invalid(format!("{context} contains invalid hexadecimal digits"))
    })
}

fn jis_to_shift_jis(code: u16) -> Result<[u8; 2], TextJsonError> {
    let row = (code >> 8) as u8;
    let cell = code as u8;
    if !(0x21..=0x7E).contains(&row) || !(0x21..=0x7E).contains(&cell) {
        return Err(TextJsonError::Invalid(format!(
            "game glyph JIS code {code:04X} is outside 0x2121..0x7E7E"
        )));
    }
    let mut lead = 0x81u16 + u16::from((row - 0x21) / 2);
    if lead > 0x9F {
        lead += 0x40;
    }
    let trail = if row % 2 == 1 {
        cell + if cell <= 0x5F { 0x1F } else { 0x20 }
    } else {
        cell + 0x7E
    };
    let pair = [lead as u8, trail];
    if shift_jis_pair_to_jis(pair[0], pair[1]) != Some(code) {
        return Err(TextJsonError::Invalid(format!(
            "game glyph JIS code {code:04X} cannot be represented as Shift-JIS"
        )));
    }
    Ok(pair)
}

pub(crate) fn expected_control_size(raw: &[u8]) -> Option<usize> {
    let code = *raw.first()?;
    if code == 0x0F {
        return match raw.get(1) {
            Some(0x00) => Some(3),
            _ => None,
        };
    }
    let arguments = match code {
        0x01 | 0x02 | 0x07 | 0x09 | 0x0A | 0x0D => 0,
        0x03 | 0x04 | 0x06 | 0x08 | 0x0B | 0x0C | 0x0E => 1,
        0x05 => 2,
        _ => return None,
    };
    Some(1 + arguments)
}

fn parse_marker(marker: &str) -> Result<(Vec<u8>, bool), TextJsonError> {
    match marker {
        "PAGE" => return Ok((vec![0x01], true)),
        "WAIT" => return Ok((vec![0x02], true)),
        _ => {}
    }
    if let Some(value) = marker.strip_prefix("VAR:") {
        let raw = parse_hex(value, "VAR marker")?;
        if raw.len() != 1 {
            return Err(TextJsonError::Invalid(
                "VAR marker requires exactly two hexadecimal digits".to_owned(),
            ));
        }
        return Ok((vec![0x04, raw[0]], true));
    }
    if let Some(value) = marker.strip_prefix("G:") {
        if value.len() != 4 {
            return Err(TextJsonError::Invalid(
                "G marker requires exactly four hexadecimal digits".to_owned(),
            ));
        }
        let code = u16::from_str_radix(value, 16)
            .map_err(|_| TextJsonError::Invalid("G marker has an invalid JIS code".to_owned()))?;
        return Ok((jis_to_shift_jis(code)?.to_vec(), false));
    }
    if let Some(value) = marker.strip_prefix("GRAW:") {
        let raw = parse_hex(value, "GRAW marker")?;
        if raw.len() != 1 || !matches!(raw[0], 0x7F | 0x80 | 0xA0 | 0xFD..=0xFF) {
            return Err(TextJsonError::Invalid(
                "GRAW marker must encode one non-control single-byte game glyph".to_owned(),
            ));
        }
        return Ok((raw, false));
    }
    if let Some(value) = marker.strip_prefix("CTRL:") {
        let raw = parse_hex(value, "CTRL marker")?;
        if expected_control_size(&raw) != Some(raw.len()) {
            return Err(TextJsonError::Invalid(format!(
                "CTRL marker {} is not one complete supported control",
                hex::encode_upper(raw)
            )));
        }
        if raw[0] == 0 || raw[0] == 0x0D {
            return Err(TextJsonError::Invalid(
                "CTRL marker cannot encode NUL or newline".to_owned(),
            ));
        }
        return Ok((raw, true));
    }
    Err(TextJsonError::Invalid(format!(
        "unknown reserved marker [[{marker}]]"
    )))
}

fn flush_text(pending: &mut String, parsed: &mut ParsedMessage) -> Result<(), TextJsonError> {
    if pending.is_empty() {
        return Ok(());
    }
    parsed.bytes.extend_from_slice(&encode_cp932_text(pending)?);
    pending.clear();
    Ok(())
}

pub(crate) fn parse_message(message: &str) -> Result<ParsedMessage, TextJsonError> {
    if message.trim().is_empty() {
        return Err(TextJsonError::Invalid(
            "message must contain translatable text".to_owned(),
        ));
    }
    let mut parsed = ParsedMessage::default();
    let mut pending = String::new();
    let mut cursor = 0usize;
    while cursor < message.len() {
        let rest = &message[cursor..];
        if rest.starts_with("[[") {
            flush_text(&mut pending, &mut parsed)?;
            let Some(end) = rest.find("]]") else {
                return Err(TextJsonError::Invalid(
                    "unterminated reserved marker".to_owned(),
                ));
            };
            let marker = &rest[2..end];
            let (raw, structural) = parse_marker(marker)?;
            if raw == [0x01] {
                parsed.page_count += 1;
            } else if structural {
                parsed.structural_controls.push(raw.clone());
            }
            parsed.bytes.extend_from_slice(&raw);
            cursor += end + 2;
            continue;
        }
        let character = rest.chars().next().expect("cursor is in bounds");
        if character == '\n' {
            flush_text(&mut pending, &mut parsed)?;
            parsed.bytes.push(0x0D);
            parsed.newline_count += 1;
        } else if character == '\r' || character.is_control() {
            return Err(TextJsonError::Invalid(format!(
                "message contains unsupported control character U+{:04X}",
                character as u32
            )));
        } else {
            pending.push(character);
        }
        cursor += character.len_utf8();
    }
    flush_text(&mut pending, &mut parsed)?;
    Ok(parsed)
}

pub(crate) fn runtime_variable_width(slot: u8) -> usize {
    match slot {
        0 => 6, // カイト
        1 => 8, // ウルフィ
        _ => 19,
    }
}

pub(crate) fn validate_line_width(
    stream: &[u8],
    channel: u16,
) -> Result<(usize, usize), TextJsonError> {
    let limit = line_limit(channel)?;
    let mut cursor = 0usize;
    let mut line = 1usize;
    let mut line_width = 0usize;
    let mut max_width = 0usize;
    while cursor < stream.len() {
        let byte = stream[cursor];
        if byte == 0 {
            max_width = max_width.max(line_width);
            return Ok((max_width, line));
        }
        if byte < 0x10 {
            let size = expected_control_size(&stream[cursor..]).ok_or_else(|| {
                TextJsonError::Invalid(format!(
                    "unsupported control 0x{byte:02X} while measuring message"
                ))
            })?;
            if cursor + size > stream.len() {
                return Err(TextJsonError::Invalid(
                    "truncated control while measuring message".to_owned(),
                ));
            }
            match byte {
                0x01 => {
                    max_width = max_width.max(line_width);
                    line_width = 0;
                    line += 1;
                }
                0x04 => line_width += runtime_variable_width(stream[cursor + 1]),
                0x0D => {
                    max_width = max_width.max(line_width);
                    line_width = 0;
                    line += 1;
                }
                _ => {}
            }
            cursor += size;
        } else if (0x81..=0x9F).contains(&byte) || (0xE0..=0xFC).contains(&byte) {
            let trail = *stream.get(cursor + 1).ok_or_else(|| {
                TextJsonError::Invalid("truncated Shift-JIS character in message".to_owned())
            })?;
            let jis = shift_jis_pair_to_jis(byte, trail).ok_or_else(|| {
                TextJsonError::Invalid(format!(
                    "invalid Shift-JIS pair {byte:02X} {trail:02X} in message"
                ))
            })?;
            line_width += if (0x2921..=0x2B7E).contains(&jis) {
                1
            } else {
                2
            };
            cursor += 2;
        } else {
            line_width += 1;
            cursor += 1;
        }
        if line_width > limit {
            return Err(TextJsonError::Invalid(format!(
                "message line {line} is {line_width} display units, channel {channel} limit is {limit}; insert an explicit newline before the overflow"
            )));
        }
    }
    Err(TextJsonError::Invalid(
        "message stream is not NUL-terminated".to_owned(),
    ))
}

pub(crate) fn tokenize_encoded(bytes: &[u8]) -> Result<Vec<EncodedToken>, TextJsonError> {
    let mut tokens = Vec::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if byte < 0x10 {
            let size = expected_control_size(&bytes[cursor..]).ok_or_else(|| {
                TextJsonError::Invalid(format!(
                    "unsupported control 0x{byte:02X} while tokenizing message"
                ))
            })?;
            if cursor + size > bytes.len() {
                return Err(TextJsonError::Invalid(
                    "truncated control while tokenizing message".to_owned(),
                ));
            }
            let units = if byte == 0x04 {
                runtime_variable_width(bytes[cursor + 1])
            } else {
                0
            };
            tokens.push(EncodedToken {
                bytes: bytes[cursor..cursor + size].to_vec(),
                units,
                control: Some(byte),
            });
            cursor += size;
        } else if (0x81..=0x9F).contains(&byte) || (0xE0..=0xFC).contains(&byte) {
            let trail = *bytes.get(cursor + 1).ok_or_else(|| {
                TextJsonError::Invalid("truncated Shift-JIS character in message".to_owned())
            })?;
            let jis = shift_jis_pair_to_jis(byte, trail).ok_or_else(|| {
                TextJsonError::Invalid(format!(
                    "invalid Shift-JIS pair {byte:02X} {trail:02X} in message"
                ))
            })?;
            tokens.push(EncodedToken {
                bytes: bytes[cursor..cursor + 2].to_vec(),
                units: if (0x2921..=0x2B7E).contains(&jis) {
                    1
                } else {
                    2
                },
                control: None,
            });
            cursor += 2;
        } else {
            tokens.push(EncodedToken {
                bytes: vec![byte],
                units: 1,
                control: None,
            });
            cursor += 1;
        }
    }
    Ok(tokens)
}

pub(crate) fn token_line_units(tokens: &[EncodedToken]) -> Vec<usize> {
    let mut lines = vec![0usize];
    for token in tokens {
        if matches!(token.control, Some(0x01 | 0x0D)) {
            lines.push(0);
        } else {
            *lines.last_mut().expect("line vector is never empty") += token.units;
        }
    }
    lines
}

fn validate_page_line_widths(
    tokens: &[EncodedToken],
    channel: u16,
    source_line_units: &[usize],
) -> Result<(usize, usize), TextJsonError> {
    let line_units = token_line_units(tokens);
    let normal_limit = line_limit(channel)?;
    if channel == 8 && line_units.len() != source_line_units.len() {
        return Err(TextJsonError::Invalid(format!(
            "choice page has {} display lines, source requires {}",
            line_units.len(),
            source_line_units.len()
        )));
    }
    for (line_index, width) in line_units.iter().copied().enumerate() {
        let limit = if channel == 8 {
            normal_limit.max(source_line_units[line_index])
        } else {
            normal_limit
        };
        if width > limit {
            return Err(TextJsonError::Invalid(format!(
                "message line {} is {width} display units, channel {channel} limit is {limit}; insert an explicit newline before the overflow",
                line_index + 1
            )));
        }
    }
    Ok((
        line_units.iter().copied().max().unwrap_or(0),
        line_units.len(),
    ))
}

pub(crate) fn encode_page_message(
    source_controls: &[Vec<u8>],
    source_newlines: usize,
    source_line_units: &[usize],
    message: &str,
    channel: u16,
) -> Result<EncodedPage, TextJsonError> {
    let parsed = parse_message(message)?;
    if parsed.page_count != 0 {
        return Err(TextJsonError::Invalid(
            "PAGE markers are page boundaries in v2 JSON and cannot appear inside message"
                .to_owned(),
        ));
    }
    if parsed.structural_controls != source_controls {
        return Err(TextJsonError::Invalid(
            "message changed, removed, duplicated, or reordered a required structural marker"
                .to_owned(),
        ));
    }
    if channel == 8 && parsed.newline_count != source_newlines {
        return Err(TextJsonError::Invalid(format!(
            "choice page has {} newlines, source requires {source_newlines}",
            parsed.newline_count
        )));
    }
    let tokens = tokenize_encoded(&parsed.bytes)?;
    let (max_line_units, line_count) =
        validate_page_line_widths(&tokens, channel, source_line_units)?;
    Ok(EncodedPage {
        bytes: parsed.bytes,
        tokens,
        max_line_units,
        line_count,
    })
}

pub fn encode_message(
    source: &TextStream,
    message: &str,
    channel: u16,
) -> Result<EncodedMessage, TextJsonError> {
    let (source_controls, source_newlines, source_pages, suffix) = source_structure(source);
    let mut parsed = parse_message(message)?;
    if parsed.structural_controls != source_controls {
        return Err(TextJsonError::Invalid(
            "message changed, removed, duplicated, or reordered a required structural marker"
                .to_owned(),
        ));
    }
    if channel == 8 && parsed.newline_count != source_newlines {
        return Err(TextJsonError::Invalid(format!(
            "choice message has {} newlines, source requires {source_newlines}",
            parsed.newline_count
        )));
    }
    if channel == 8 && parsed.page_count != source_pages {
        return Err(TextJsonError::Invalid(format!(
            "choice message has {} PAGE markers, source requires {source_pages}",
            parsed.page_count
        )));
    }
    if channel == 9 && parsed.page_count < source_pages {
        return Err(TextJsonError::Invalid(format!(
            "message has {} PAGE markers, source requires at least {source_pages}",
            parsed.page_count
        )));
    }
    parsed.bytes.extend_from_slice(&suffix);
    parsed.bytes.push(0);
    let (max_line_units, line_count) = validate_line_width(&parsed.bytes, channel)?;
    Ok(EncodedMessage {
        stream: parsed.bytes,
        max_line_units,
        line_count,
    })
}

pub fn validate_file_header(file: &TranslationFile) -> Result<(), TextJsonError> {
    if file.format != TEXT_JSON_FORMAT {
        return Err(TextJsonError::Invalid(format!(
            "unsupported _format {:?}, expected {:?}",
            file.format, TEXT_JSON_FORMAT
        )));
    }
    let mut indices = BTreeSet::new();
    for entry in &file.entries {
        if !indices.insert(entry.index) {
            return Err(TextJsonError::Invalid(format!(
                "duplicate _index {}",
                entry.index
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::{parse_text_stream, SCRIPT_BODY_OFFSET};

    fn source_stream() -> TextStream {
        let mut data = vec![0; SCRIPT_BODY_OFFSET];
        data.extend_from_slice(&encode_cp932_text("原文").unwrap());
        data.push(0x0D);
        data.extend_from_slice(&encode_cp932_text("名前").unwrap());
        data.extend_from_slice(&[0x04, 0x01]);
        data.extend_from_slice(&[0xEB, 0xC2]);
        data.push(0x01);
        data.push(0);
        parse_text_stream(&data, SCRIPT_BODY_OFFSET).unwrap()
    }

    #[test]
    fn single_message_round_trips_controls_and_glyphs() {
        let source = source_stream();
        let rendered = render_message(&source);
        assert_eq!(rendered, "原文\n名前[[VAR:01]][[G:7644]]");
        let encoded = encode_message(&source, &rendered, 9).unwrap();
        assert_eq!(encoded.stream, source.encoded());
    }

    #[test]
    fn story_message_can_reflow_and_replace_a_game_glyph() {
        let source = source_stream();
        let translated = "短い\n訳[[VAR:01]]\nです";
        let encoded = encode_message(&source, translated, 9).unwrap();
        assert!(encoded.stream.windows(2).all(|pair| pair != [0xEB, 0xC2]));
        assert_eq!(
            encoded.stream.iter().filter(|byte| **byte == 0x0D).count(),
            2
        );
        assert_eq!(encoded.stream[encoded.stream.len() - 2..], [0x01, 0]);
    }

    #[test]
    fn choice_newline_count_is_immutable() {
        let source = source_stream();
        let error = encode_message(&source, "原文 名前[[VAR:01]][[G:7644]]", 8).unwrap_err();
        assert!(error.to_string().contains("source requires 1"));
    }

    #[test]
    fn display_width_limits_fullwidth_and_halfwidth_text() {
        let source = source_stream();
        let pass = format!("{}[[VAR:01]]", "あ".repeat(31));
        assert_eq!(
            encode_message(&source, &pass, 9).unwrap().max_line_units,
            70
        );
        let fail = format!("{}[[VAR:01]]", "あ".repeat(32));
        let error = encode_message(&source, &fail, 9).unwrap_err();
        assert!(error.to_string().contains("72 display units"));
    }

    #[test]
    fn choice_page_preserves_only_source_line_width_exceptions() {
        let accepted = encode_page_message(&[], 0, &[80], &"あ".repeat(40), 8).unwrap();
        assert_eq!(accepted.max_line_units, 80);

        let wider = encode_page_message(&[], 0, &[80], &"あ".repeat(41), 8).unwrap_err();
        assert!(wider.to_string().contains("limit is 80"));

        let ordinary = encode_page_message(&[], 0, &[40], &"あ".repeat(31), 8).unwrap_err();
        assert!(ordinary.to_string().contains("limit is 60"));
    }

    #[test]
    fn required_structural_markers_cannot_change() {
        let source = source_stream();
        let error = encode_message(&source, "原文\n名前", 9).unwrap_err();
        assert!(error.to_string().contains("required structural marker"));
    }

    #[test]
    fn story_message_can_add_pages_for_free_length_growth() {
        let source = source_stream();
        let translated = "一\n二\n三\n四[[PAGE]]五\n六[[VAR:01]]";
        let encoded = encode_message(&source, translated, 9).unwrap();
        assert!(encoded.stream.contains(&0x01));
    }

    #[test]
    fn legacy_v1_json_header_is_rejected() {
        let file = TranslationFile {
            format: "canaan-sys98-text-v1".to_owned(),
            file: "cs00_00.s".to_owned(),
            entries: Vec::new(),
        };
        let error = validate_file_header(&file).unwrap_err();
        assert!(error
            .to_string()
            .contains("expected \"canaan-sys98-text-v2\""));
    }
}
