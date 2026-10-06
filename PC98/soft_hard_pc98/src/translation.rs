//! Editable script templates and safe CP932/PC-98 control-word translation.
use crate::text_vm::{self, DecodeDocument, Instruction};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use vn_font::font_98::{self, EncodingPlan};

pub const WORKSPACE_SCHEMA: &str = "soft-hard-pc98-translation-workspace-v1";
pub const TEMPLATE_SCHEMA: &str = "soft-hard-pc98-script-translation-v2";
const TEMPLATE_ENCODING: &str = "CP932";

const NEWLINE_WORD: u16 = 0x8197;
const CONTROL_MARKER: &str = "[[PC98:";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceManifest {
    pub schema: String,
    pub tool_version: String,
    pub disks: Vec<DiskBaseline>,
    pub scripts: Vec<ScriptBaseline>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiskBaseline {
    pub id: String,
    pub source_name: String,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptBaseline {
    pub disk_id: String,
    pub path: String,
    pub byte_length: u64,
    pub sha256: String,
    pub template_path: String,
    pub instruction_count: usize,
    pub string_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationTemplate {
    #[serde(rename = "_format")]
    format: String,
    #[serde(rename = "_file")]
    file: String,
    #[serde(rename = "_disk")]
    disk: String,
    #[serde(rename = "_source_sha256")]
    source_sha256: String,
    #[serde(rename = "_encoding")]
    encoding: String,
    pub entries: Vec<TranslationEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationEntry {
    #[serde(rename = "_index")]
    pub index: usize,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct StringEdit {
    pub instruction_offset: usize,
    pub string_index: usize,
    pub text: String,
}

#[derive(Clone, Debug)]
pub enum TextPiece {
    Text(String),
    RawWord(u16),
}

pub fn make_template(
    disk_id: &str,
    path: &str,
    bytes: &[u8],
    source_sha256: String,
) -> TranslationTemplate {
    let mut decoded = text_vm::parse(bytes, format!("{disk_id}/{path}"));
    make_template_from_decoded(disk_id, path, bytes, source_sha256, &mut decoded)
}

pub fn make_template_from_decoded(
    disk_id: &str,
    path: &str,
    bytes: &[u8],
    source_sha256: String,
    decoded: &mut DecodeDocument,
) -> TranslationTemplate {
    let mut entries = Vec::new();
    for instruction in &mut decoded.instructions {
        for payload in &mut instruction.strings {
            let raw = &bytes[payload.offset..payload.end_offset];
            payload.translation = translation_view(raw);
            entries.push(TranslationEntry {
                index: entries.len(),
                scr_msg: payload.translation.clone(),
                message: payload.translation.clone(),
            });
        }
    }
    TranslationTemplate {
        format: TEMPLATE_SCHEMA.into(),
        file: path.rsplit(['/', '\\']).next().unwrap_or(path).into(),
        disk: disk_id.into(),
        source_sha256,
        encoding: TEMPLATE_ENCODING.into(),
        entries,
    }
}

/// Read only editable `message` values after verifying the source metadata,
/// ordered indices, and immutable original strings against a fresh parse.
pub fn edits_from_template(
    supplied: &[u8],
    expected: &TranslationTemplate,
    decoded: &DecodeDocument,
) -> Result<Vec<StringEdit>, String> {
    let supplied: TranslationTemplate =
        serde_json::from_slice(supplied).map_err(|e| format!("翻译 JSON 格式无效: {e}"))?;
    if supplied.format != expected.format
        || supplied.file != expected.file
        || supplied.disk != expected.disk
        || supplied.source_sha256 != expected.source_sha256
        || supplied.encoding != expected.encoding
    {
        return Err("翻译 JSON 的文件标识或源哈希与当前原盘不匹配".into());
    }
    if supplied.entries.len() != expected.entries.len() {
        return Err("翻译 JSON 的文本条目数量与原文件不匹配".into());
    }

    let mut coordinates = Vec::with_capacity(expected.entries.len());
    for instruction in &decoded.instructions {
        for payload in &instruction.strings {
            coordinates.push((
                instruction.offset,
                payload.index.unwrap_or(0),
                payload.translation.as_str(),
            ));
        }
    }
    if coordinates.len() != expected.entries.len() {
        return Err("内部错误：文本条目与解码结构数量不匹配".into());
    }
    let mut edits = Vec::new();
    for (
        position,
        ((expected_entry, supplied_entry), (instruction_offset, string_index, original)),
    ) in expected
        .entries
        .iter()
        .zip(&supplied.entries)
        .zip(&coordinates)
        .enumerate()
    {
        if supplied_entry.index != position
            || expected_entry.index != position
            || supplied_entry.scr_msg != expected_entry.scr_msg
            || supplied_entry.scr_msg != *original
        {
            return Err(format!(
                "文本条目 {position} 的 _index/scr_msg 已修改或与原文不匹配"
            ));
        }
        edits.push(StringEdit {
            instruction_offset: *instruction_offset,
            string_index: *string_index,
            text: supplied_entry.message.clone(),
        });
    }
    Ok(edits)
}

pub fn translation_view(bytes: &[u8]) -> String {
    // SOG consumes each string as a sequence of BE u16 character words. Do not
    // treat 00 xx as a one-byte CP932 character; preserve it as a raw word.
    let mut out = String::new();
    for pair in bytes.chunks_exact(2) {
        let word = u16::from_be_bytes([pair[0], pair[1]]);
        if word == NEWLINE_WORD {
            out.push('\n');
            continue;
        }
        if pair[0] == 0 {
            push_raw_word(&mut out, word);
            continue;
        }
        if let Some(text) =
            encoding_rs::SHIFT_JIS.decode_without_bom_handling_and_without_replacement(pair)
        {
            out.push_str(&text);
        } else {
            push_raw_word(&mut out, word);
        }
    }
    if !bytes.len().is_multiple_of(2) {
        push_raw_word(&mut out, u16::from(bytes[bytes.len() - 1]));
    }
    out
}

pub fn template_for_script(
    disk_id: &str,
    path: &str,
    bytes: &[u8],
    source_sha256: String,
) -> TranslationTemplate {
    make_template(disk_id, path, bytes, source_sha256)
}

pub fn split_translation(text: &str) -> Result<Vec<TextPiece>, String> {
    let mut pieces = Vec::new();
    let mut ordinary = String::new();
    let mut index = 0usize;
    while index < text.len() {
        let rest = &text[index..];
        if rest.starts_with(CONTROL_MARKER) {
            if !ordinary.is_empty() {
                pieces.push(TextPiece::Text(std::mem::take(&mut ordinary)));
            }
            let close = rest.find("]]").ok_or("PC-98 原始字码标记缺少 ]] 结束符")?;
            let digits = &rest[CONTROL_MARKER.len()..close];
            if digits.len() != 4 || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(format!("无效 PC-98 原始字码标记: {}", &rest[..close + 2]));
            }
            let word = u16::from_str_radix(digits, 16).map_err(|e| e.to_string())?;
            if word == 0 {
                return Err("00 00 是脚本串终止符，不能放进 translation".into());
            }
            pieces.push(TextPiece::RawWord(word));
            index += close + 2;
            continue;
        }
        let ch = rest.chars().next().ok_or("文本编码错误")?;
        if ch == '\n' {
            if !ordinary.is_empty() {
                pieces.push(TextPiece::Text(std::mem::take(&mut ordinary)));
            }
            pieces.push(TextPiece::RawWord(NEWLINE_WORD));
        } else if ch == '\r' || ch.is_control() {
            return Err(format!(
                "translation 含不支持的控制字符 U+{:04X}",
                ch as u32
            ));
        } else {
            ordinary.push(ch);
        }
        index += ch.len_utf8();
    }
    if !ordinary.is_empty() {
        pieces.push(TextPiece::Text(ordinary));
    }
    Ok(pieces)
}

pub fn plan_text(text: &str) -> Result<String, String> {
    let mut out = String::new();
    for piece in split_translation(text)? {
        if let TextPiece::Text(text) = piece {
            out.push_str(&text);
        }
    }
    Ok(out)
}

pub fn encode_translation(text: &str, plan: &EncodingPlan) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    for piece in split_translation(text)? {
        match piece {
            TextPiece::Text(text) => {
                let encoded = plan
                    .encode_cp932(&text)
                    .map_err(|error| format!("文本编码失败: {error}"))?;
                let expected_len = text.chars().count().checked_mul(2).ok_or("文本长度溢出")?;
                if encoded.len() != expected_len {
                    return Err("文本编码未生成每字符一个 16 位字".into());
                }
                out.extend_from_slice(&encoded);
            }
            TextPiece::RawWord(word) => out.extend_from_slice(&word.to_be_bytes()),
        }
    }
    if out.len() % 2 != 0 {
        return Err("生成文本不是 16 位字序列".into());
    }
    Ok(out)
}

pub fn decoded_translation(bytes: &[u8], plan: &EncodingPlan) -> Result<String, String> {
    if !bytes.len().is_multiple_of(2) {
        return Err("回读文本不是 16 位字序列".into());
    }
    let mut out = String::new();
    for pair in bytes.chunks_exact(2) {
        let word = u16::from_be_bytes([pair[0], pair[1]]);
        if word == NEWLINE_WORD {
            out.push('\n');
        } else if pair[0] == 0 {
            push_raw_word(&mut out, word);
        } else if let Some(decoded) =
            encoding_rs::SHIFT_JIS.decode_without_bom_handling_and_without_replacement(pair)
        {
            for character in decoded.chars() {
                out.push(plan.display_for_carrier(character));
            }
        } else {
            push_raw_word(&mut out, word);
        }
    }
    Ok(out)
}

pub fn canonical_translation(text: &str) -> Result<String, String> {
    // Newline input is a convenient spelling for the confirmed 0x8197 word.
    let mut out = String::new();
    for piece in split_translation(text)? {
        match piece {
            TextPiece::Text(value) => out.push_str(&value),
            TextPiece::RawWord(word) if word == NEWLINE_WORD => out.push('\n'),
            TextPiece::RawWord(word) => {
                let bytes = word.to_be_bytes();
                if let Some(decoded) = encoding_rs::SHIFT_JIS
                    .decode_without_bom_handling_and_without_replacement(&bytes)
                {
                    out.push_str(&decoded);
                } else {
                    push_raw_word(&mut out, word);
                }
            }
        }
    }
    Ok(out)
}

pub fn collect_reserved_cp932(instructions: &[Instruction], bytes: &[u8]) -> BTreeSet<u16> {
    let mut reserved = BTreeSet::new();
    for instruction in instructions {
        for payload in &instruction.strings {
            if let Some(terminator) = payload.terminator_offset {
                let raw = &bytes[payload.offset..terminator];
                for pair in raw.chunks_exact(2) {
                    if pair[0] == 0 {
                        continue;
                    }
                    let Some(_) = encoding_rs::SHIFT_JIS
                        .decode_without_bom_handling_and_without_replacement(pair)
                    else {
                        continue;
                    };
                    let jis = [pair[0], pair[1]];
                    if font_98::cp932_to_jis(jis).is_ok() {
                        reserved.insert(u16::from_be_bytes(jis));
                    }
                }
            }
        }
    }
    reserved
}

fn push_raw_word(out: &mut String, word: u16) {
    use std::fmt::Write as _;
    let _ = write!(out, "[[PC98:{word:04X}]]");
}
