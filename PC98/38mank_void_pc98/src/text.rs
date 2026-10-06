//! Text records in `.STD` are a bytewise ROR1 wrapper around the game's VM.
//! Only instructions with a verified linear operand layout are traversed.
use crate::{sha256_hex, Result};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use vn_font::font_98;

const SCHEMA: &str = "38mank-void-std-text-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextDocument {
    pub schema: String,
    pub source_file: String,
    pub source_sha256: String,
    pub encoding: String,
    pub entries: Vec<TextEntry>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextEntry {
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_byte_length")]
    pub byte_length: usize,
    #[serde(rename = "_raw_hex")]
    pub raw_hex: String,
    #[serde(rename = "_kind")]
    pub kind: TextKind,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextKind {
    Pc98Words,
    Cp932Bytes,
}

struct Parsed {
    document: TextDocument,
    boundaries: Vec<usize>,
    jumps: Vec<Jump>,
}

struct Jump {
    operand_offset: usize,
    target_offset: usize,
}

fn decode_wrapped(bytes: &[u8]) -> Vec<u8> {
    bytes.iter().map(|byte| byte.rotate_left(1)).collect()
}

fn encode_wrapped(bytes: &[u8]) -> Vec<u8> {
    bytes.iter().map(|byte| byte.rotate_right(1)).collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}

fn read_u8(bytes: &[u8], cursor: &mut usize) -> Result<u8> {
    let value = *bytes.get(*cursor).ok_or("VM 指令在读取 8 位参数时越界")?;
    *cursor += 1;
    Ok(value)
}

fn read_u16_le(bytes: &[u8], cursor: &mut usize) -> Result<u16> {
    let end = cursor.checked_add(2).ok_or("VM 偏移溢出")?;
    let pair = bytes
        .get(*cursor..end)
        .ok_or("VM 指令在读取 16 位参数时越界")?;
    *cursor = end;
    Ok(u16::from_le_bytes([pair[0], pair[1]]))
}

fn skip_typed(bytes: &[u8], cursor: &mut usize) -> Result<()> {
    let kind = read_u8(bytes, cursor)?;
    match kind {
        0 | 1 => {
            read_u16_le(bytes, cursor)?;
            Ok(())
        }
        2 => {
            read_u16_le(bytes, cursor)?;
            read_u16_le(bytes, cursor)?;
            Ok(())
        }
        _ => Err(format!(
            "未知 typed operand {kind:02X}，停止于 0x{:X}",
            *cursor - 1
        )),
    }
}

fn take_terminated_bytes(bytes: &[u8], cursor: &mut usize) -> Result<(usize, usize)> {
    let start = *cursor;
    loop {
        if read_u8(bytes, cursor)? == 0 {
            return Ok((start, *cursor - 1));
        }
    }
}

fn skip_terminated_bytes(bytes: &[u8], cursor: &mut usize) -> Result<()> {
    take_terminated_bytes(bytes, cursor).map(|_| ())
}

fn take_text(bytes: &[u8], cursor: &mut usize) -> Result<(usize, usize)> {
    let start = *cursor;
    loop {
        let end = cursor.checked_add(2).ok_or("文本偏移溢出")?;
        let pair = bytes.get(*cursor..end).ok_or("文本串缺少 00 00 终止符")?;
        *cursor = end;
        if pair == [0, 0] {
            return Ok((start, end - 2));
        }
    }
}

fn text_view(bytes: &[u8]) -> String {
    let mut result = String::new();
    for pair in bytes.chunks_exact(2) {
        let word = u16::from_be_bytes([pair[0], pair[1]]);
        if word == 0x8197 {
            result.push('\n');
        } else if matches!(word, 0x814F | 0x818F | 0x81A5) {
            result.push_str(&format!("[[PC98:{word:04X}]]"));
        } else if pair[0] != 0 {
            if let Some(decoded) =
                SHIFT_JIS.decode_without_bom_handling_and_without_replacement(pair)
            {
                let (encoded, _, had_errors) = SHIFT_JIS.encode(&decoded);
                if !had_errors && encoded.as_ref() == pair {
                    result.push_str(&decoded);
                    continue;
                }
            }
            result.push_str(&format!("[[PC98:{word:04X}]]"));
        } else {
            result.push_str(&format!("[[PC98:{word:04X}]]"));
        }
    }
    result
}

fn cp932_view(bytes: &[u8]) -> Option<String> {
    let decoded = SHIFT_JIS.decode_without_bom_handling_and_without_replacement(bytes)?;
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&decoded);
    if had_errors || encoded.as_ref() != bytes {
        return None;
    }
    Some(decoded.into_owned())
}

fn encode_message(message: &str, plan: &font_98::EncodingPlan) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut cursor = 0;
    while cursor < message.len() {
        let rest = &message[cursor..];
        if rest.starts_with("[[PC98:") {
            let end = rest.find("]]").ok_or("PC98 原始字码标记缺少 ]] ")?;
            let digits = &rest[7..end];
            if digits.len() != 4 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(format!("无效 PC98 字码标记: {}", &rest[..end + 2]));
            }
            let word = u16::from_str_radix(digits, 16).map_err(|e| e.to_string())?;
            if word == 0 {
                return Err("文本中不能包含 00 00 终止符".into());
            }
            result.extend_from_slice(&word.to_be_bytes());
            cursor += end + 2;
            continue;
        }
        let ch = rest.chars().next().ok_or("无效 UTF-8 字符边界")?;
        if ch == '\n' {
            result.extend_from_slice(&0x8197u16.to_be_bytes());
        } else if ch.is_control() {
            return Err(format!("不支持控制字符 U+{:04X}", ch as u32));
        } else {
            let carrier = plan
                .carrier_for(ch)
                .map_err(|error| format!("字符 {ch:?} 无法分配 PC-98 字库字位: {error}"))?;
            result.extend_from_slice(&font_98::cp932_for_carrier(carrier)?);
        }
        cursor += ch.len_utf8();
    }
    Ok(result)
}

fn encode_cp932_bytes(message: &str, plan: &font_98::EncodingPlan) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    for ch in message.chars() {
        if ch.is_control() {
            return Err(format!("NUL 字节串中不支持控制字符 U+{:04X}", ch as u32));
        }
        if ch.is_ascii() {
            result.push(ch as u8);
        } else {
            let carrier = plan
                .carrier_for(ch)
                .map_err(|error| format!("字符 {ch:?} 无法分配 PC-98 字库字位: {error}"))?;
            result.extend_from_slice(&font_98::cp932_for_carrier(carrier)?);
        }
    }
    if result.contains(&0) {
        return Err("文本无法无损编码为 CP932 NUL 字节串".into());
    }
    Ok(result)
}

fn parse_decoded(bytes: &[u8], source_file: &str, raw_hash: &str) -> Parsed {
    let mut cursor = 0;
    let mut entries = Vec::new();
    let mut diagnostics = Vec::new();
    let mut boundaries = Vec::new();
    let mut jumps = Vec::new();
    while cursor < bytes.len() {
        let opcode_offset = cursor;
        boundaries.push(cursor);
        let opcode = bytes[cursor];
        cursor += 1;
        let parsed: Result<Vec<(usize, usize, TextKind)>> = (|| match opcode {
            0x00 => {
                if bytes[cursor..].iter().any(|byte| *byte != 0) {
                    return Err("VM 结束符后存在非零尾部；剩余内容未解析".into());
                }
                cursor = bytes.len();
                Ok(Vec::new())
            }
            0x01 => {
                for _ in 0..5 {
                    skip_typed(bytes, &mut cursor)?;
                }
                let (start, end) = take_terminated_bytes(bytes, &mut cursor)?;
                Ok(vec![(start, end, TextKind::Cp932Bytes)])
            }
            0x02 | 0x04 | 0x0A | 0x10 | 0x11 | 0x12 | 0x15 => {
                skip_typed(bytes, &mut cursor)?;
                Ok(Vec::new())
            }
            0x03 | 0x17 => {
                for _ in 0..3 {
                    skip_typed(bytes, &mut cursor)?;
                }
                Ok(Vec::new())
            }
            0x05 | 0x06 | 0x08 | 0x0B | 0x13 | 0x16 => Ok(Vec::new()),
            0x07 => {
                skip_typed(bytes, &mut cursor)?;
                skip_typed(bytes, &mut cursor)?;
                let operation = read_u8(bytes, &mut cursor)?;
                if operation != 0x0A {
                    skip_typed(bytes, &mut cursor)?;
                }
                Ok(Vec::new())
            }
            0x09 => {
                skip_typed(bytes, &mut cursor)?;
                read_u8(bytes, &mut cursor)?;
                skip_typed(bytes, &mut cursor)?;
                Ok(Vec::new())
            }
            0x0C => {
                let count = read_u8(bytes, &mut cursor)?;
                let mut strings = Vec::new();
                for _ in 0..count {
                    let (start, end) = take_terminated_bytes(bytes, &mut cursor)?;
                    strings.push((start, end, TextKind::Cp932Bytes));
                }
                Ok(strings)
            }
            0x0D => {
                skip_typed(bytes, &mut cursor)?;
                let (start, end) = take_text(bytes, &mut cursor)?;
                Ok(vec![(start, end, TextKind::Pc98Words)])
            }
            0x0E => {
                let (start, end) = take_text(bytes, &mut cursor)?;
                Ok(vec![(start, end, TextKind::Pc98Words)])
            }
            0x0F => {
                let operand_offset = cursor;
                let displacement = read_u16_le(bytes, &mut cursor)? as i16;
                let target = (cursor as isize)
                    .checked_add(displacement as isize)
                    .filter(|target| *target >= 0)
                    .ok_or("0x0F 跳转目标越界")? as usize;
                jumps.push(Jump {
                    operand_offset,
                    target_offset: target,
                });
                Ok(Vec::new())
            }
            0x14 | 0x18 | 0x19 | 0x1A => {
                skip_typed(bytes, &mut cursor)?;
                skip_typed(bytes, &mut cursor)?;
                Ok(Vec::new())
            }
            0x1B => {
                skip_typed(bytes, &mut cursor)?;
                skip_typed(bytes, &mut cursor)?;
                skip_terminated_bytes(bytes, &mut cursor)?;
                Ok(Vec::new())
            }
            0x1C => {
                for _ in 0..4 {
                    skip_typed(bytes, &mut cursor)?;
                }
                Ok(Vec::new())
            }
            0x1D => {
                skip_typed(bytes, &mut cursor)?;
                let count = usize::from(read_u8(bytes, &mut cursor)?);
                for _ in 0..count {
                    read_u16_le(bytes, &mut cursor)?;
                }
                Ok(Vec::new())
            }
            0x1E => {
                skip_terminated_bytes(bytes, &mut cursor)?;
                Ok(Vec::new())
            }
            _ => Err(format!("opcode {opcode:02X} 的边界尚未确认")),
        })();
        match parsed {
            Ok(spans) => {
                for (start, end, kind) in spans {
                    let payload = &bytes[start..end];
                    if payload.is_empty() {
                        continue;
                    }
                    let view = match kind {
                        TextKind::Pc98Words => Some(text_view(payload)),
                        TextKind::Cp932Bytes => cp932_view(payload),
                    };
                    if let Some(view) = view {
                        entries.push(TextEntry {
                            index: entries.len(),
                            offset: start,
                            byte_length: payload.len(),
                            raw_hex: hex(payload),
                            kind,
                            scr_msg: view.clone(),
                            message: view,
                        });
                    } else {
                        diagnostics.push(format!("0x{start:X}: 非可逆 CP932 字节串，保留原始数据"));
                    }
                }
            }
            Err(error) => {
                diagnostics.push(format!("0x{opcode_offset:X}: {error}"));
                break;
            }
        }
    }
    for jump in &jumps {
        if !boundaries.contains(&jump.target_offset) {
            diagnostics.push(format!(
                "0x{:X}: 跳转目标 0x{:X} 不是指令边界",
                jump.operand_offset, jump.target_offset
            ));
        }
    }
    Parsed {
        document: TextDocument {
            schema: SCHEMA.into(),
            source_file: source_file.into(),
            source_sha256: raw_hash.into(),
            encoding: "CP932 / bytewise ROR1 on disk".into(),
            entries,
            diagnostics,
        },
        boundaries,
        jumps,
    }
}

pub fn extract_document(raw: &[u8], source_file: &str) -> Result<Option<TextDocument>> {
    if !source_file.to_ascii_uppercase().ends_with(".STD") {
        return Ok(None);
    }
    let decoded = decode_wrapped(raw);
    let parsed = parse_decoded(&decoded, source_file, &sha256_hex(raw));
    Ok(Some(parsed.document))
}

pub fn merge_document(
    raw: &[u8],
    source_file: &str,
    supplied: &TextDocument,
) -> Result<TextDocument> {
    if !source_file.to_ascii_uppercase().ends_with(".STD") {
        return Err("源文件不是 STD 脚本".into());
    }
    let decoded = decode_wrapped(raw);
    let parsed = parse_decoded(&decoded, source_file, &sha256_hex(raw));
    let expected = &parsed.document;
    if supplied.schema != expected.schema
        || supplied.source_file != expected.source_file
        || supplied.source_sha256 != expected.source_sha256
        || supplied.encoding != expected.encoding
        || supplied.diagnostics != expected.diagnostics
        || supplied.entries.len() != expected.entries.len()
    {
        return Err(format!(
            "翻译 JSON 的来源元数据或文本条目数不匹配: {source_file}"
        ));
    }
    for (actual, baseline) in supplied.entries.iter().zip(expected.entries.iter()) {
        if actual.index != baseline.index
            || actual.offset != baseline.offset
            || actual.byte_length != baseline.byte_length
            || actual.raw_hex != baseline.raw_hex
            || actual.kind != baseline.kind
            || actual.scr_msg != baseline.scr_msg
        {
            return Err(format!(
                "{source_file}: 文本条目 {} 的原文或定位信息被修改",
                baseline.index
            ));
        }
    }
    let mut merged = expected.clone();
    for (actual, entry) in merged.entries.iter_mut().zip(&supplied.entries) {
        actual.message = entry.message.clone();
    }
    Ok(merged)
}

pub fn apply_document(
    raw: &[u8],
    source_file: &str,
    supplied: &TextDocument,
    plan: &font_98::EncodingPlan,
) -> Result<Vec<u8>> {
    let decoded = decode_wrapped(raw);
    let parsed = parse_decoded(&decoded, source_file, &sha256_hex(raw));
    let expected = &parsed.document;
    let merged = merge_document(raw, source_file, supplied)?;
    let mut edits = Vec::new();
    for (actual, baseline) in merged.entries.iter().zip(expected.entries.iter()) {
        if actual.message != actual.scr_msg {
            let replacement = match actual.kind {
                TextKind::Pc98Words => encode_message(&actual.message, plan),
                TextKind::Cp932Bytes => encode_cp932_bytes(&actual.message, plan),
            }
            .map_err(|e| format!("{source_file} 条目 {}: {e}", actual.index))?;
            let roundtrip = match actual.kind {
                TextKind::Pc98Words => plan.decode_carriers(&text_view(&replacement)),
                TextKind::Cp932Bytes => {
                    plan.decode_carriers(&cp932_view(&replacement).ok_or("CP932 回读失败")?)
                }
            };
            if roundtrip != actual.message {
                return Err(format!(
                    "{source_file} 条目 {}: 编码回读与译文不一致",
                    actual.index
                ));
            }
            edits.push(Edit {
                start: baseline.offset,
                end: baseline.offset + baseline.byte_length,
                replacement,
            });
        }
    }
    if edits.is_empty() {
        return Ok(raw.to_vec());
    }
    if !expected.diagnostics.is_empty() {
        return Err(format!("{source_file}: 源脚本未完整解析，拒绝注入"));
    }
    edits.sort_by_key(|edit| edit.start);
    let mut rebuilt = Vec::new();
    let mut source_cursor = 0;
    for edit in &edits {
        if edit.start < source_cursor || edit.end > decoded.len() {
            return Err(format!("{source_file}: 译文范围重叠或越界"));
        }
        rebuilt.extend_from_slice(&decoded[source_cursor..edit.start]);
        rebuilt.extend_from_slice(&edit.replacement);
        source_cursor = edit.end;
    }
    rebuilt.extend_from_slice(&decoded[source_cursor..]);
    if rebuilt.len() > u16::MAX as usize {
        return Err(format!("{source_file}: 新脚本超过 16 位段内偏移范围"));
    }
    for jump in &parsed.jumps {
        if !parsed.boundaries.contains(&jump.target_offset) {
            return Err(format!("{source_file}: 原始跳转目标不是指令边界"));
        }
        let operand = remap_offset(jump.operand_offset, &edits)?;
        let target = remap_offset(jump.target_offset, &edits)?;
        let displacement = (target as isize) - ((operand + 2) as isize);
        let displacement = i16::try_from(displacement)
            .map_err(|_| format!("{source_file}: 重定位后的跳转超出 i16 范围"))?;
        rebuilt[operand..operand + 2].copy_from_slice(&displacement.to_le_bytes());
    }
    let verified = parse_decoded(&rebuilt, source_file, "rebuild");
    if !verified.document.diagnostics.is_empty()
        || verified.document.entries.len() != supplied.entries.len()
        || verified.jumps.len() != parsed.jumps.len()
    {
        return Err(format!("{source_file}: 重建后脚本结构复核失败"));
    }
    for (actual, entry) in verified.document.entries.iter().zip(&merged.entries) {
        let actual_message = match actual.kind {
            TextKind::Pc98Words => plan.decode_carriers(&actual.scr_msg),
            TextKind::Cp932Bytes => plan.decode_carriers(&actual.scr_msg),
        };
        if actual_message != entry.message || actual.kind != entry.kind {
            return Err(format!(
                "{source_file}: 重建后条目 {} 回读不一致",
                entry.index
            ));
        }
    }
    Ok(encode_wrapped(&rebuilt))
}

struct Edit {
    start: usize,
    end: usize,
    replacement: Vec<u8>,
}

fn remap_offset(offset: usize, edits: &[Edit]) -> Result<usize> {
    let mut delta = 0isize;
    for edit in edits {
        if offset < edit.start {
            break;
        }
        if offset < edit.end {
            return Err(format!("偏移 0x{offset:X} 位于已替换文本内部"));
        }
        delta += edit.replacement.len() as isize - (edit.end - edit.start) as isize;
    }
    offset
        .checked_add_signed(delta)
        .ok_or_else(|| format!("偏移 0x{offset:X} 重定位溢出"))
}
