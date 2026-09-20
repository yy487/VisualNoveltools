//! Misty Blue `.BUN` dialogue projection and guarded rebuilding.

use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use vn_font::font_98::{jis_to_cp932, EncodingPlan};

pub const FORMAT: &str = "misty-blue-bun-text-v1";
const ENCODING: &str =
    "Misty Blue BUN compact JIS; translated text uses compatible double-byte CP932 carriers";
const STRUCTURE: &str =
    "u16le body size; repeated style:u8, attribute:u8, text, terminator:00..04; 02 ends record";

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentJson {
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_style")]
    pub style: u8,
    #[serde(rename = "_attribute")]
    pub attribute: u8,
    #[serde(rename = "_terminator")]
    pub terminator: u8,
    #[serde(rename = "_source_offset")]
    pub source_offset: usize,
    #[serde(rename = "_source_size")]
    pub source_size: usize,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordJson {
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_source_offset")]
    pub source_offset: usize,
    #[serde(rename = "_source_body_size")]
    pub source_body_size: usize,
    pub segments: Vec<SegmentJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BunJson {
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
    #[serde(rename = "_structure")]
    pub structure: String,
    #[serde(rename = "_logical_end")]
    pub logical_end: usize,
    pub records: Vec<RecordJson>,
}

fn decode_jis(jis: [u8; 2], context: &str) -> Result<char> {
    // The shipped scripts use the unassigned JIS 26-6E slot three times as a
    // visibly blank separator. Preserve its display semantics in editable text;
    // the raw 83EC slot is reserved separately during font planning.
    if jis == [0x26, 0x6e] {
        return Ok('　');
    }
    let cp932 = jis_to_cp932(jis).map_err(|error| format!("{context}: {error}"))?;
    let decoded = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(&cp932)
        .ok_or_else(|| format!("{context}: JIS {:02X}{:02X} 不能解码", jis[0], jis[1]))?;
    let mut chars = decoded.chars();
    let character = chars
        .next()
        .ok_or_else(|| format!("{context}: JIS 解码为空"))?;
    if chars.next().is_some() {
        return Err(format!("{context}: JIS 解码得到多个字符"));
    }
    Ok(character)
}

fn decode_run(bytes: &[u8], absolute_offset: usize) -> Result<(String, usize)> {
    let mut output = String::new();
    let mut offset = 0usize;
    let mut katakana = false;
    while offset < bytes.len() {
        let lead = bytes[offset];
        if lead < 0x05 {
            break;
        }
        offset += 1;
        match lead {
            0x6c => output.push('\n'),
            0x85 => katakana = false,
            0x86 => katakana = true,
            0x05..=0x48 => output.push(decode_jis(
                [0x21, lead + 1],
                &format!("BUN+0x{:X}", absolute_offset + offset - 1),
            )?),
            0xab..=0xff => {
                let jis = if katakana {
                    let mut trail = lead - 0x6b;
                    if trail >= 0x80 {
                        trail -= 1;
                    }
                    [
                        0x25,
                        trail.checked_sub(0x1f).ok_or_else(|| {
                            format!("BUN+0x{:X}: 片假名压缩码无效", absolute_offset + offset - 1)
                        })?,
                    ]
                } else {
                    [0x24, lead - 0x8a]
                };
                output.push(decode_jis(
                    jis,
                    &format!("BUN+0x{:X}", absolute_offset + offset - 1),
                )?);
            }
            _ => {
                let trail = *bytes.get(offset).ok_or_else(|| {
                    format!("BUN+0x{:X}: 双字节字符被截断", absolute_offset + offset - 1)
                })?;
                offset += 1;
                let trail = if lead == 0x97 && trail == 0x54 {
                    0x53
                } else {
                    trail
                };
                let mut row = (i16::from(lead) - 0x71) * 2 + 1;
                let column = if trail >= 0x9f {
                    row += 1;
                    trail - 0x7e
                } else {
                    let adjusted = if trail >= 0x80 { trail - 1 } else { trail };
                    adjusted.checked_sub(0x1f).ok_or_else(|| {
                        format!(
                            "BUN+0x{:X}: Shift-JIS trail 无效",
                            absolute_offset + offset - 1
                        )
                    })?
                };
                if !(0x21..=0x7e).contains(&row) || !(0x21..=0x7e).contains(&i16::from(column)) {
                    return Err(format!(
                        "BUN+0x{:X}: 字节 {lead:02X}{trail:02X} 得到越界 JIS",
                        absolute_offset + offset - 2
                    ));
                }
                output.push(decode_jis(
                    [row as u8, column],
                    &format!("BUN+0x{:X}", absolute_offset + offset - 2),
                )?);
            }
        }
    }
    Ok((output, offset))
}

pub fn parse_bun(bytes: &[u8], file: String) -> Result<BunJson> {
    let mut records = Vec::new();
    let mut offset = 0usize;
    let logical_end = loop {
        let raw = bytes
            .get(offset..offset + 2)
            .ok_or_else(|| format!("{file}: BUN 缺少零长度结束记录"))?;
        let body_size = usize::from(u16::from_le_bytes([raw[0], raw[1]]));
        let record_offset = offset;
        offset += 2;
        if body_size == 0 {
            break record_offset;
        }
        let body_end = offset
            .checked_add(body_size)
            .ok_or_else(|| format!("{file}+0x{record_offset:X}: 记录长度溢出"))?;
        if body_end > bytes.len() {
            return Err(format!(
                "{file}+0x{record_offset:X}: 记录声明 {body_size} 字节并越过文件末尾"
            ));
        }
        let mut body_cursor = offset;
        let mut segments = Vec::new();
        loop {
            if body_cursor + 2 > body_end {
                return Err(format!("{file}+0x{body_cursor:X}: 段头被截断"));
            }
            let style = bytes[body_cursor];
            let attribute = bytes[body_cursor + 1];
            body_cursor += 2;
            let text_offset = body_cursor;
            let (text, consumed) = decode_run(&bytes[body_cursor..body_end], body_cursor)?;
            body_cursor += consumed;
            let terminator = *bytes
                .get(body_cursor)
                .ok_or_else(|| format!("{file}+0x{body_cursor:X}: 段缺少终止控制码"))?;
            if terminator > 0x04 {
                return Err(format!(
                    "{file}+0x{body_cursor:X}: 段终止码 {terminator:02X} 不在 00..04"
                ));
            }
            body_cursor += 1;
            segments.push(SegmentJson {
                index: segments.len(),
                style,
                attribute,
                terminator,
                source_offset: text_offset,
                source_size: consumed,
                scr_msg: text.clone(),
                message: text,
            });
            if terminator == 0x02 || body_cursor == body_end {
                break;
            }
        }
        if body_cursor != body_end {
            return Err(format!(
                "{file}+0x{record_offset:X}: 记录在 end=02 后还剩 {} 字节",
                body_end - body_cursor
            ));
        }
        records.push(RecordJson {
            index: records.len(),
            source_offset: record_offset,
            source_body_size: body_size,
            segments,
        });
        offset = body_end;
    };
    Ok(BunJson {
        format: FORMAT.to_owned(),
        file,
        source_size: bytes.len(),
        source_sha256: sha256(bytes),
        encoding: ENCODING.to_owned(),
        structure: STRUCTURE.to_owned(),
        logical_end,
        records,
    })
}

fn same_immutable_segment(actual: &SegmentJson, expected: &SegmentJson) -> bool {
    actual.index == expected.index
        && actual.style == expected.style
        && actual.attribute == expected.attribute
        && actual.terminator == expected.terminator
        && actual.source_offset == expected.source_offset
        && actual.source_size == expected.source_size
        && actual.scr_msg == expected.scr_msg
}

pub fn validate_bun(script: &BunJson, source: &[u8]) -> Result<BunJson> {
    if script.format != FORMAT || script.encoding != ENCODING || script.structure != STRUCTURE {
        return Err(format!("不支持或已修改的 BUN JSON 元数据；需要 {FORMAT}"));
    }
    if script.source_size != source.len() || script.source_sha256 != sha256(source) {
        return Err("BUN JSON 对应的源文件大小或 SHA-256 不匹配".to_owned());
    }
    let expected = parse_bun(source, script.file.clone())?;
    if script.logical_end != expected.logical_end
        || script.records.len() != expected.records.len()
        || script
            .records
            .iter()
            .zip(&expected.records)
            .any(|(actual, expected)| {
                actual.index != expected.index
                    || actual.source_offset != expected.source_offset
                    || actual.source_body_size != expected.source_body_size
                    || actual.segments.len() != expected.segments.len()
                    || actual
                        .segments
                        .iter()
                        .zip(&expected.segments)
                        .any(|(actual, expected)| !same_immutable_segment(actual, expected))
            })
    {
        return Err("records/segments 的不可变元数据或 scr_msg 已被修改".to_owned());
    }
    for record in &script.records {
        for segment in &record.segments {
            normalize_translation(&segment.message).map_err(|error| {
                format!("record {} segment {}: {error}", record.index, segment.index)
            })?;
        }
    }
    Ok(expected)
}

pub fn normalize_translation(value: &str) -> Result<String> {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        let normalized = match character {
            '\n' => '\n',
            '\0' | '\r' => return Err("message 不能包含 NUL 或 CR；换行请使用 LF".to_owned()),
            ' ' => '　',
            '!'..='~' => char::from_u32(character as u32 + 0xfee0).expect("ASCII 全角映射始终有效"),
            value if value.is_control() => {
                return Err(format!("message 含控制字符 U+{:04X}", value as u32))
            }
            value => value,
        };
        output.push(normalized);
    }
    Ok(output)
}

pub fn entry_count(script: &BunJson) -> usize {
    script
        .records
        .iter()
        .map(|record| record.segments.len())
        .sum()
}

pub fn changed_entries(script: &BunJson) -> usize {
    script
        .records
        .iter()
        .flat_map(|record| &record.segments)
        .filter(|segment| segment.message != segment.scr_msg)
        .count()
}

pub fn final_display_texts(script: &BunJson) -> Result<Vec<String>> {
    script
        .records
        .iter()
        .flat_map(|record| &record.segments)
        .filter(|segment| segment.message != segment.scr_msg)
        .map(|segment| normalize_translation(&segment.message).map(|text| text.replace('\n', "")))
        .collect()
}

pub fn source_reserved_cp932(script: &BunJson) -> Result<BTreeSet<u16>> {
    let mut reserved = BTreeSet::new();
    reserved.insert(0x83ec);
    for segment in script
        .records
        .iter()
        .flat_map(|record| &record.segments)
        .filter(|segment| segment.message == segment.scr_msg)
    {
        for character in segment
            .scr_msg
            .chars()
            .filter(|character| *character != '\n')
        {
            let text = character.to_string();
            let (encoded, _, errors) = SHIFT_JIS.encode(&text);
            if errors || encoded.len() != 2 {
                return Err(format!("源 BUN 字符 {character:?} 不能表示为双字节 CP932"));
            }
            reserved.insert(u16::from_be_bytes([encoded[0], encoded[1]]));
        }
    }
    Ok(reserved)
}

pub fn compatible_pair(pair: [u8; 2]) -> bool {
    (0x81..=0x98).contains(&pair[0])
        && !matches!(pair[0], 0x85 | 0x86)
        && pair != [0x97, 0x54]
        && !matches!(pair, [0x81, 0x93] | [0x81, 0x94] | [0x81, 0x97])
}

fn encode_translation(text: &str, plan: &EncodingPlan) -> Result<Vec<u8>> {
    let normalized = normalize_translation(text)?;
    let mut output = Vec::with_capacity(normalized.len() * 2);
    for character in normalized.chars() {
        if character == '\n' {
            output.push(0x6c);
            continue;
        }
        let pair = plan.encode_cp932(&character.to_string())?;
        let pair: [u8; 2] = pair
            .as_slice()
            .try_into()
            .map_err(|_| format!("字符 {character:?} 未编码为一个双字节载体"))?;
        if !compatible_pair(pair) {
            return Err(format!(
                "字符 {character:?} 的载体 {:02X}{:02X} 会与 BUN/MES 控制码冲突",
                pair[0], pair[1]
            ));
        }
        output.extend_from_slice(&pair);
    }
    Ok(output)
}

pub fn rebuild_bun(script: &BunJson, source: &[u8], plan: &EncodingPlan) -> Result<Vec<u8>> {
    let expected = validate_bun(script, source)?;
    let mut output = Vec::with_capacity(source.len());
    for (record, expected_record) in script.records.iter().zip(&expected.records) {
        let mut body = Vec::new();
        for (segment, expected_segment) in record.segments.iter().zip(&expected_record.segments) {
            body.push(segment.style);
            body.push(segment.attribute);
            if segment.message == segment.scr_msg {
                body.extend_from_slice(
                    &source[expected_segment.source_offset
                        ..expected_segment.source_offset + expected_segment.source_size],
                );
            } else {
                body.extend_from_slice(&encode_translation(&segment.message, plan)?);
            }
            body.push(segment.terminator);
        }
        let body_size = u16::try_from(body.len())
            .map_err(|_| format!("record {} 重建后超过 BUN 的 65535 字节上限", record.index))?;
        output.extend_from_slice(&body_size.to_le_bytes());
        output.extend_from_slice(&body);
    }
    output.extend_from_slice(&0u16.to_le_bytes());
    let tail_start = expected
        .logical_end
        .checked_add(2)
        .ok_or_else(|| "BUN 结束偏移溢出".to_owned())?;
    output.extend_from_slice(&source[tail_start..]);
    Ok(output)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:X}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_compact_bun_and_preserves_unchanged_bytes() {
        let source = vec![4, 0, 1, 0xff, 0xac, 0x02, 0, 0, 0, 0];
        let parsed = parse_bun(&source, "A/files/T.BUN".into()).unwrap();
        assert_eq!(parsed.records.len(), 1);
        assert_eq!(parsed.records[0].segments[0].scr_msg, "あ");
        let substitutions = vn_font::font_98::SubstitutionMap::embedded().unwrap();
        let reserved = source_reserved_cp932(&parsed).unwrap();
        let texts = final_display_texts(&parsed).unwrap();
        let plan = EncodingPlan::build(&substitutions, reserved, texts.iter().map(String::as_str))
            .unwrap();
        assert_eq!(rebuild_bun(&parsed, &source, &plan).unwrap(), source);
    }
}
