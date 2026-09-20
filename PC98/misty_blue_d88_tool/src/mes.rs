//! Misty Blue compiled `.MES` text projection and guarded rebuilding.

use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use vn_font::font_98::EncodingPlan;

pub const FORMAT: &str = "misty-blue-mes-text-v1";
const ENCODING: &str = "CP932 bytecode; 81..98 are display pairs; translated text uses compatible double-byte carriers";
const LEXER: &str =
    "MAIN.EXE 1EE61/1F043 profile; quotes and packed integers are immutable operands";
const SPECIAL_CONTROLS: [u16; 3] = [0x8193, 0x8194, 0x8197];

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenJson {
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_role")]
    pub role: String,
    #[serde(rename = "_raw")]
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryJson {
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
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MesJson {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_source_size")]
    pub source_size: usize,
    #[serde(rename = "_source_sha256")]
    pub source_sha256: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_lexer")]
    pub lexer: String,
    pub entries: Vec<EntryJson>,
    #[serde(rename = "_tokens")]
    pub tokens: Vec<TokenJson>,
}

fn is_display_lead(byte: u8) -> bool {
    (0x81..=0x98).contains(&byte)
}

fn strict_pair(bytes: &[u8], offset: usize) -> bool {
    if offset + 1 >= bytes.len() || !is_display_lead(bytes[offset]) {
        return false;
    }
    let trail = bytes[offset + 1];
    if !((0x40..=0x7e).contains(&trail) || (0x80..=0xfc).contains(&trail)) {
        return false;
    }
    SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(&bytes[offset..offset + 2])
        .is_some()
}

fn compact_size(bytes: &[u8], offset: usize) -> Result<usize> {
    let tag = bytes[offset];
    let size = if tag & 7 != 0 {
        1
    } else if tag & 8 != 0 {
        3
    } else {
        2
    };
    if offset + size > bytes.len() {
        Err(format!("MES+0x{offset:X}: 压缩整数被截断"))
    } else {
        Ok(size)
    }
}

fn lex(bytes: &[u8]) -> Result<Vec<TokenJson>> {
    let mut tokens = Vec::new();
    let mut offset = 0usize;
    while offset < bytes.len() {
        let start = offset;
        let byte = bytes[offset];
        let (size, role) = if (0x99..=0xef).contains(&byte) {
            (1, "opcode")
        } else if is_display_lead(byte) {
            if !strict_pair(bytes, offset) {
                return Err(format!("MES+0x{offset:X}: 无效或截断的 CP932 显示字符"));
            }
            let code = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
            (
                2,
                if SPECIAL_CONTROLS.contains(&code) {
                    "display_control"
                } else {
                    "text"
                },
            )
        } else if byte == 0x22 {
            let end = bytes[offset + 1..]
                .iter()
                .position(|value| *value == 0x22)
                .ok_or_else(|| format!("MES+0x{offset:X}: 引号串未终止"))?;
            (end + 2, "quoted_operand")
        } else if byte < 0x20 {
            (compact_size(bytes, offset)?, "packed_value")
        } else {
            (
                1,
                if matches!(
                    byte,
                    b'+' | b'-'
                        | b'*'
                        | b'/'
                        | b'%'
                        | b'&'
                        | b'|'
                        | b'^'
                        | b'>'
                        | b'<'
                        | b'!'
                        | b'='
                        | b'\\'
                        | b'#'
                ) {
                    "operator"
                } else if matches!(byte, b',' | b'{' | b'}') {
                    "delimiter"
                } else if byte == b'?' {
                    "random"
                } else if (b'@'..=b'Z').contains(&byte) {
                    "variable"
                } else {
                    "opaque"
                },
            )
        };
        offset += size;
        tokens.push(TokenJson {
            index: tokens.len(),
            offset: start,
            size,
            role: role.to_owned(),
            raw: hex_spaced(&bytes[start..offset]),
        });
    }
    Ok(tokens)
}

fn decode_text(bytes: &[u8], context: &str) -> Result<String> {
    SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(bytes)
        .map(|value| value.into_owned())
        .ok_or_else(|| format!("{context}: CP932 正文解码失败"))
}

pub fn parse_mes(bytes: &[u8], file: String) -> Result<MesJson> {
    let tokens = lex(bytes)?;
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
        let text = decode_text(&bytes[offset..end], &format!("{file}+0x{offset:X}"))?;
        entries.push(EntryJson {
            index: entries.len(),
            token_start,
            token_end,
            offset,
            size: end - offset,
            scr_msg: text.clone(),
            message: text,
        });
    }
    Ok(MesJson {
        format: FORMAT.to_owned(),
        file,
        source_size: bytes.len(),
        source_sha256: sha256(bytes),
        encoding: ENCODING.to_owned(),
        lexer: LEXER.to_owned(),
        entries,
        tokens,
    })
}

fn same_immutable_entry(actual: &EntryJson, expected: &EntryJson) -> bool {
    actual.index == expected.index
        && actual.token_start == expected.token_start
        && actual.token_end == expected.token_end
        && actual.offset == expected.offset
        && actual.size == expected.size
        && actual.scr_msg == expected.scr_msg
}

pub fn validate_mes(script: &MesJson, source: &[u8]) -> Result<MesJson> {
    if script.format != FORMAT || script.encoding != ENCODING || script.lexer != LEXER {
        return Err(format!("不支持或已修改的 MES JSON 元数据；需要 {FORMAT}"));
    }
    if script.source_size != source.len() || script.source_sha256 != sha256(source) {
        return Err("MES JSON 对应的源文件大小或 SHA-256 不匹配".to_owned());
    }
    let expected = parse_mes(source, script.file.clone())?;
    if script.tokens != expected.tokens
        || script.entries.len() != expected.entries.len()
        || script
            .entries
            .iter()
            .zip(&expected.entries)
            .any(|(actual, expected)| !same_immutable_entry(actual, expected))
    {
        return Err("entries/_tokens 的不可变结构或 scr_msg 已被修改".to_owned());
    }
    for entry in &script.entries {
        normalize_translation(&entry.message)
            .map_err(|error| format!("entry {}: {error}", entry.index))?;
    }
    Ok(expected)
}

pub fn normalize_translation(value: &str) -> Result<String> {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        let normalized = match character {
            '\0' | '\r' | '\n' => {
                return Err(
                    "MES message 不能包含 NUL、CR 或 LF；原脚本箭头控制符不可编辑".to_owned(),
                )
            }
            ' ' => '　',
            '!'..='~' => char::from_u32(character as u32 + 0xfee0).expect("ASCII 全角映射始终有效"),
            value if value.is_control() => {
                return Err(format!("MES message 含控制字符 U+{:04X}", value as u32))
            }
            value => value,
        };
        let text = normalized.to_string();
        let (encoded, _, errors) = SHIFT_JIS.encode(&text);
        if !errors && encoded.len() == 2 {
            let code = u16::from_be_bytes([encoded[0], encoded[1]]);
            if SPECIAL_CONTROLS.contains(&code) {
                return Err(format!(
                    "字符 {normalized:?} 是 MES 专用控制箭头，不能放入正文"
                ));
            }
        }
        output.push(normalized);
    }
    Ok(output)
}

pub fn reserved_control_cp932() -> impl Iterator<Item = u16> {
    SPECIAL_CONTROLS.into_iter()
}

pub fn changed_entries(script: &MesJson) -> usize {
    script
        .entries
        .iter()
        .filter(|entry| entry.message != entry.scr_msg)
        .count()
}

pub fn final_display_texts(script: &MesJson) -> Result<Vec<String>> {
    script
        .entries
        .iter()
        .filter(|entry| entry.message != entry.scr_msg)
        .map(|entry| normalize_translation(&entry.message))
        .collect()
}

pub fn source_reserved_cp932(script: &MesJson) -> Result<BTreeSet<u16>> {
    let mut reserved = BTreeSet::new();
    for character in script
        .entries
        .iter()
        .filter(|entry| entry.message == entry.scr_msg)
        .flat_map(|entry| entry.scr_msg.chars())
    {
        let text = character.to_string();
        let (encoded, _, errors) = SHIFT_JIS.encode(&text);
        if errors || encoded.len() != 2 {
            return Err(format!("源 MES 字符 {character:?} 不能表示为双字节 CP932"));
        }
        reserved.insert(u16::from_be_bytes([encoded[0], encoded[1]]));
    }
    Ok(reserved)
}

fn encode_translation(text: &str, plan: &EncodingPlan) -> Result<Vec<u8>> {
    let normalized = normalize_translation(text)?;
    let bytes = plan.encode_cp932(&normalized)?;
    for pair in bytes.chunks_exact(2) {
        if !crate::bun::compatible_pair([pair[0], pair[1]]) {
            return Err(format!(
                "载体 {:02X}{:02X} 会与 BUN/MES 控制码冲突",
                pair[0], pair[1]
            ));
        }
    }
    Ok(bytes)
}

pub fn rebuild_mes(script: &MesJson, source: &[u8], plan: &EncodingPlan) -> Result<Vec<u8>> {
    let expected = validate_mes(script, source)?;
    let mut output = Vec::with_capacity(source.len());
    let mut token_index = 0usize;
    let mut entry_index = 0usize;
    while token_index < expected.tokens.len() {
        if entry_index < script.entries.len()
            && script.entries[entry_index].token_start == token_index
        {
            let entry = &script.entries[entry_index];
            if entry.message == entry.scr_msg {
                output.extend_from_slice(&source[entry.offset..entry.offset + entry.size]);
            } else {
                output.extend_from_slice(&encode_translation(&entry.message, plan)?);
            }
            token_index = entry.token_end;
            entry_index += 1;
        } else {
            let token = &expected.tokens[token_index];
            output.extend_from_slice(&source[token.offset..token.offset + token.size]);
            token_index += 1;
        }
    }
    if entry_index != script.entries.len() {
        return Err(format!("entry {entry_index} 未在 MES token 流中到达"));
    }
    Ok(output)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:X}", Sha256::digest(bytes))
}

fn hex_spaced(bytes: &[u8]) -> String {
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
    fn quotes_and_controls_are_not_translation_entries() {
        let source = [0xae, b'"', b'A', b'.', b'P', b'"', 0x81, 0x93, 0x82, 0xa0];
        let parsed = parse_mes(&source, "B/files/T.MES".into()).unwrap();
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].scr_msg, "あ");
        assert_eq!(parsed.tokens[1].role, "quoted_operand");
        assert_eq!(parsed.tokens[2].role, "display_control");
    }
}
