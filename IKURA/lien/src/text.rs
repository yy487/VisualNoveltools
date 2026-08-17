use crate::{Result, fail};
use encoding_rs::{Encoding, GBK, SHIFT_JIS};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

const LIEN_DICTIONARY: [u8; 258] = [
    0x81, 0x40, 0x81, 0x40, 0x81, 0x41, 0x81, 0x42, 0x81, 0x45, 0x81, 0x48, 0x81, 0x49, 0x81, 0x69,
    0x81, 0x6A, 0x81, 0x75, 0x81, 0x76, 0x82, 0x4F, 0x82, 0x50, 0x82, 0x51, 0x82, 0x52, 0x82, 0x53,
    0x82, 0x54, 0x82, 0x55, 0x82, 0x56, 0x82, 0x57, 0x82, 0x58, 0x82, 0xA0, 0x82, 0xA2, 0x82, 0xA4,
    0x82, 0xA6, 0x82, 0xA8, 0x82, 0xA9, 0x82, 0xAA, 0x82, 0xAB, 0x82, 0xAC, 0x82, 0xAD, 0x82, 0xAE,
    0x81, 0x40, 0x82, 0xB0, 0x82, 0xB1, 0x82, 0xB2, 0x82, 0xB3, 0x82, 0xB4, 0x82, 0xB5, 0x82, 0xB6,
    0x82, 0xB7, 0x82, 0xB8, 0x82, 0xB9, 0x82, 0xBA, 0x82, 0xBB, 0x82, 0xBC, 0x82, 0xBD, 0x82, 0xBE,
    0x82, 0xBF, 0x82, 0xC0, 0x82, 0xC1, 0x82, 0xC2, 0x82, 0xC3, 0x82, 0xC4, 0x82, 0xC5, 0x82, 0xC6,
    0x82, 0xC7, 0x82, 0xC8, 0x82, 0xC9, 0x82, 0xCA, 0x82, 0xCB, 0x82, 0xCC, 0x82, 0xCD, 0x82, 0xCE,
    0x82, 0xD0, 0x82, 0xD1, 0x82, 0xD3, 0x82, 0xD4, 0x82, 0xD6, 0x82, 0xD7, 0x82, 0xD9, 0x82, 0xDA,
    0x82, 0xDC, 0x82, 0xDD, 0x82, 0xDE, 0x82, 0xDF, 0x82, 0xE0, 0x82, 0xE1, 0x82, 0xE2, 0x82, 0xE3,
    0x82, 0xE4, 0x82, 0xE5, 0x82, 0xE6, 0x82, 0xE7, 0x82, 0xE8, 0x82, 0xE9, 0x82, 0xEA, 0x82, 0xEB,
    0x82, 0xED, 0x82, 0xF0, 0x82, 0xF1, 0x83, 0x41, 0x83, 0x43, 0x83, 0x45, 0x83, 0x47, 0x83, 0x49,
    0x83, 0x4A, 0x83, 0x4C, 0x83, 0x4E, 0x83, 0x50, 0x83, 0x52, 0x83, 0x54, 0x83, 0x56, 0x83, 0x58,
    0x83, 0x5A, 0x83, 0x5C, 0x83, 0x5E, 0x83, 0x60, 0x83, 0x62, 0x83, 0x63, 0x83, 0x65, 0x83, 0x67,
    0x83, 0x69, 0x83, 0x6A, 0x82, 0xAF, 0x83, 0x6C, 0x83, 0x6D, 0x83, 0x6E, 0x83, 0x71, 0x83, 0x74,
    0x83, 0x77, 0x83, 0x7A, 0x83, 0x7D, 0x83, 0x7E, 0x83, 0x80, 0x83, 0x81, 0x83, 0x82, 0x83, 0x84,
    0x00, 0x00,
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EncodingRoute {
    Cp932Native,
    Gbk,
}

impl EncodingRoute {
    pub fn label(self) -> &'static str {
        match self {
            Self::Cp932Native => "cp932_native",
            Self::Gbk => "gbk",
        }
    }
}

impl fmt::Display for EncodingRoute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

impl FromStr for EncodingRoute {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "cp932" | "cp932_native" => Ok(Self::Cp932Native),
            "gbk" => Ok(Self::Gbk),
            _ => Err(format!("unsupported encoding route: {value}")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextSegment {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextStream {
    pub segments: Vec<TextSegment>,
    pub has_excluded_0x02: bool,
}

pub fn decode_lien_text(data: &[u8]) -> Result<String> {
    let mut expanded = Vec::with_capacity(data.len() * 2);
    let mut position = 0;
    while position < data.len() {
        let byte = data[position];
        if byte <= 0x80 {
            let table_offset = byte as usize * 2;
            let pair = &LIEN_DICTIONARY[table_offset..table_offset + 2];
            if pair == [0, 0] {
                return fail(format!(
                    "Lien text token 0x{byte:02X} at byte {position} maps to NUL"
                ));
            }
            expanded.extend_from_slice(pair);
            position += 1;
        } else {
            if position + 1 >= data.len() {
                return fail(format!(
                    "truncated Lien double-byte token at byte {position}: 0x{byte:02X}"
                ));
            }
            expanded.extend_from_slice(&data[position..position + 2]);
            position += 2;
        }
    }
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(&expanded);
    if had_errors {
        return fail("expanded Lien text is not valid CP932");
    }
    let text = decoded.into_owned();
    let (roundtrip, _, encode_errors) = SHIFT_JIS.encode(&text);
    if encode_errors || roundtrip.as_ref() != expanded {
        return fail("expanded Lien text does not round-trip through CP932");
    }
    Ok(text)
}

pub fn parse_0x2b_stream(content: &[u8]) -> Result<TextStream> {
    if content.is_empty() {
        return fail("0x2B content is missing its channel byte");
    }
    let mut position = 1;
    let mut segments = Vec::new();
    let mut has_excluded_0x02 = false;
    while position < content.len() {
        let command_offset = position;
        let command = content[position];
        position += 1;
        match command {
            0x01 | 0x08 | 0x0A | 0x11 => {
                advance_control(content, &mut position, 4, command_offset)?
            }
            0x04 | 0x09 => advance_control(content, &mut position, 1, command_offset)?,
            0x0B | 0x0C | 0x10 => advance_control(content, &mut position, 2, command_offset)?,
            0x02 => has_excluded_0x02 = true,
            0xFF => {
                let start = position;
                let relative_end = content[start..]
                    .iter()
                    .position(|&byte| byte == 0)
                    .ok_or_else(|| {
                        format!(
                            "unterminated 0x2B text payload introduced at content offset 0x{command_offset:X}"
                        )
                    })?;
                let end = start + relative_end;
                if end > start {
                    let text = decode_lien_text(&content[start..end]).map_err(|error| {
                        format!(
                            "invalid 0x2B text payload at content range 0x{start:X}..0x{end:X}: {error}"
                        )
                    })?;
                    segments.push(TextSegment { start, end, text });
                }
                position = end + 1;
            }
            _ => {}
        }
    }
    Ok(TextStream {
        segments,
        has_excluded_0x02,
    })
}

fn advance_control(
    content: &[u8],
    position: &mut usize,
    parameter_bytes: usize,
    command_offset: usize,
) -> Result<()> {
    let end = position
        .checked_add(parameter_bytes)
        .ok_or("0x2B control offset overflow")?;
    if end > content.len() {
        return fail(format!(
            "truncated 0x2B control at content offset 0x{command_offset:X}: needs {parameter_bytes} parameter bytes"
        ));
    }
    *position = end;
    Ok(())
}

pub fn decode_raw_cp932(data: &[u8]) -> Result<String> {
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(data);
    if had_errors {
        return fail("text is not valid CP932");
    }
    let text = decoded.into_owned();
    let (roundtrip, _, encode_errors) = SHIFT_JIS.encode(&text);
    if encode_errors || roundtrip.as_ref() != data {
        return fail("text does not round-trip through CP932");
    }
    Ok(text)
}

pub fn encode_lien_text(text: &str, route: EncodingRoute) -> Result<Vec<u8>> {
    reject_forbidden_text(text)?;
    let mut output = Vec::with_capacity(text.len() * 2);
    for (char_index, character) in text.chars().enumerate() {
        let encoded = encode_character(character, route, char_index)?;
        match route {
            EncodingRoute::Cp932Native => {
                if encoded.len() == 2 && encoded[0] > 0x80 {
                    output.extend_from_slice(&encoded);
                } else if let Some(token) = dictionary_token(&encoded) {
                    output.push(token);
                } else {
                    return fail(format!(
                        "character {character:?} at character index {char_index} encodes as {}, which the Lien 0x2B decoder cannot represent",
                        format_bytes(&encoded)
                    ));
                }
            }
            EncodingRoute::Gbk => {
                if encoded.len() == 2 && encoded[0] > 0x80 {
                    output.extend_from_slice(&encoded);
                } else {
                    return fail(format!(
                        "character {character:?} at character index {char_index} encodes as {}, but GBK 0x2B text requires a double-byte token with lead byte above 0x80",
                        format_bytes(&encoded)
                    ));
                }
            }
        }
    }
    Ok(output)
}

pub fn encode_raw_text(text: &str, route: EncodingRoute) -> Result<Vec<u8>> {
    reject_forbidden_text(text)?;
    let encoding = route_encoding(route);
    let mut output = Vec::with_capacity(text.len() * 2);
    for (char_index, character) in text.chars().enumerate() {
        output.extend_from_slice(&encode_with(character, encoding, route, char_index)?);
    }
    Ok(output)
}

fn encode_character(character: char, route: EncodingRoute, char_index: usize) -> Result<Vec<u8>> {
    encode_with(character, route_encoding(route), route, char_index)
}

fn encode_with(
    character: char,
    encoding: &'static Encoding,
    route: EncodingRoute,
    char_index: usize,
) -> Result<Vec<u8>> {
    let mut buffer = [0u8; 4];
    let text = character.encode_utf8(&mut buffer);
    let (encoded, _, had_errors) = encoding.encode(text);
    if had_errors {
        return fail(format!(
            "character {character:?} (U+{:04X}) at character index {char_index} is not encodable as {route}",
            character as u32
        ));
    }
    Ok(encoded.into_owned())
}

fn route_encoding(route: EncodingRoute) -> &'static Encoding {
    match route {
        EncodingRoute::Cp932Native => SHIFT_JIS,
        EncodingRoute::Gbk => GBK,
    }
}

fn dictionary_token(encoded: &[u8]) -> Option<u8> {
    if encoded.len() != 2 {
        return None;
    }
    (1u8..=0x7F).find(|&token| {
        let offset = token as usize * 2;
        LIEN_DICTIONARY[offset..offset + 2] == *encoded
    })
}

fn reject_forbidden_text(text: &str) -> Result<()> {
    for (index, character) in text.chars().enumerate() {
        if matches!(character, '\0' | '\r' | '\n') {
            return fail(format!(
                "forbidden character U+{:04X} at character index {index}",
                character as u32
            ));
        }
    }
    Ok(())
}

fn format_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_dictionary_and_cp932_tokens_decode() {
        let decoded = decode_lien_text(&[0x15, 0x82, 0xA0]).unwrap();
        assert_eq!(decoded, "ああ");
    }

    #[test]
    fn runtime_0x7f_is_dictionary_not_ascii_escape() {
        assert_eq!(decode_lien_text(&[0x7F]).unwrap(), "ヤ");
    }

    #[test]
    fn cp932_encoder_rejects_ascii_in_0x2b() {
        assert!(encode_lien_text("A", EncodingRoute::Cp932Native).is_err());
    }

    #[test]
    fn cp932_encoder_accepts_double_byte_kanji() {
        let encoded = encode_lien_text("晶", EncodingRoute::Cp932Native).unwrap();
        assert_eq!(decode_lien_text(&encoded).unwrap(), "晶");
    }

    #[test]
    fn parses_multiple_payloads_and_excluded_control() {
        let content = [
            0x00, 0xFF, 0x8F, 0xBB, 0x00, 0x06, 0xFF, 0x82, 0xA0, 0x00, 0x02, 0x00,
        ];
        let parsed = parse_0x2b_stream(&content).unwrap();
        assert_eq!(parsed.segments.len(), 2);
        assert!(parsed.has_excluded_0x02);
    }
}
