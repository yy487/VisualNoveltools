use crate::font::{cp932_for_carrier, cp932_to_jis, jis_to_cp932, EncodingPlan};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};

const FORMAT: &str = "tauhido-ag00-text-v1";
const RECORD_SIZE: usize = 256;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ag00Header {
    pub max_variables: u16,
    pub verb_count: u16,
    pub object_count: u16,
    pub flag_count: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ag00Entry {
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: u64,
    #[serde(rename = "_size")]
    pub size: u32,
    #[serde(rename = "_type")]
    pub entry_type: String,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ag00Document {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_source_file")]
    pub source_file: String,
    #[serde(rename = "_source_sha256")]
    pub source_sha256: String,
    pub header: Ag00Header,
    pub entries: Vec<Ag00Entry>,
}

#[derive(Clone, Debug)]
struct ParsedEntry {
    offset: usize,
    size: usize,
    text: String,
    structural: bool,
}

#[derive(Clone, Debug)]
struct ParsedAg00 {
    header_end: usize,
    header: Ag00Header,
    entries: Vec<ParsedEntry>,
}

pub fn extract_document(source: &[u8], source_name: &str) -> Result<Ag00Document> {
    let parsed = parse(source)?;
    let mut entries = Vec::with_capacity(parsed.entries.len());
    for (index, entry) in parsed.entries.iter().enumerate() {
        let entry_type = if entry.structural {
            "object_wildcard"
        } else if index < usize::from(parsed.header.verb_count) {
            "verb"
        } else {
            "object"
        };
        entries.push(Ag00Entry {
            index,
            offset: entry.offset as u64,
            size: entry.size as u32,
            entry_type: entry_type.to_string(),
            scr_msg: entry.text.clone(),
            message: entry.text.clone(),
        });
    }
    Ok(Ag00Document {
        format: FORMAT.to_string(),
        source_file: source_name.to_string(),
        source_sha256: crate::sha256_hex(source),
        header: parsed.header,
        entries,
    })
}

pub fn messages(document: &Ag00Document) -> impl Iterator<Item = &str> {
    document
        .entries
        .iter()
        .filter(|entry| entry.entry_type != "object_wildcard")
        .map(|entry| entry.message.as_str())
}

pub fn rebuild(
    source: &[u8],
    source_name: &str,
    document: &Ag00Document,
    plan: &EncodingPlan,
) -> Result<(Vec<u8>, usize)> {
    if document.format != FORMAT
        || document.source_file != source_name
        || document.source_sha256 != crate::sha256_hex(source)
    {
        return Err(format!("{source_name}: AG00 JSON 与源文件不匹配"));
    }
    let parsed = parse(source)?;
    if document.header.max_variables != parsed.header.max_variables
        || document.header.verb_count != parsed.header.verb_count
        || document.header.object_count != parsed.header.object_count
        || document.header.flag_count != parsed.header.flag_count
        || document.entries.len() != parsed.entries.len()
    {
        return Err(format!("{source_name}: AG00 头或条目数已改变"));
    }

    let mut output = source[..parsed.header_end].to_vec();
    let mut changed = 0usize;
    for (index, (actual, expected)) in document.entries.iter().zip(&parsed.entries).enumerate() {
        let expected_type = if expected.structural {
            "object_wildcard"
        } else if index < usize::from(parsed.header.verb_count) {
            "verb"
        } else {
            "object"
        };
        if actual.index != index
            || actual.offset != expected.offset as u64
            || actual.size != expected.size as u32
            || actual.entry_type != expected_type
            || actual.scr_msg != expected.text
        {
            return Err(format!(
                "{source_name} entry {index}: 元数据或 scr_msg 已改变"
            ));
        }
        if expected.structural {
            if actual.message != "*" {
                return Err(format!(
                    "{source_name} entry {index}: 通配对象 * 是结构项，不能翻译"
                ));
            }
            output.extend_from_slice(b"*\r");
            continue;
        }
        if actual.message != actual.scr_msg {
            changed += 1;
        }
        output.extend_from_slice(b"\x1B\x4B");
        for character in actual.message.chars() {
            let carrier = plan.carrier_for(character)?;
            let cp932 = cp932_for_carrier(carrier)?;
            output.extend_from_slice(&cp932_to_jis(cp932)?);
        }
        output.extend_from_slice(b"\x1B\x48\r");
    }

    let minimum = output
        .len()
        .checked_add(1)
        .ok_or_else(|| "AG00 重建长度溢出".to_string())?;
    let final_len = source.len().max(round_up(minimum, RECORD_SIZE)?);
    output.resize(final_len, 0);
    output[final_len - 1] = 0x1A;

    let verified = parse(&output)?;
    for (index, (actual, extracted)) in document.entries.iter().zip(&verified.entries).enumerate() {
        if extracted.structural {
            if actual.message != extracted.text {
                return Err(format!(
                    "{source_name} entry {index}: AG00 结构项重建后不一致"
                ));
            }
            continue;
        }
        let expected = plan.normalize_text(&actual.message)?;
        let decoded = plan.decode_carriers(&extracted.text);
        if expected != decoded {
            return Err(format!(
                "{source_name} entry {index}: AG00 重建后文本复查不一致"
            ));
        }
    }
    Ok((output, changed))
}

fn parse(source: &[u8]) -> Result<ParsedAg00> {
    if source.len() < RECORD_SIZE || !source.len().is_multiple_of(RECORD_SIZE) {
        return Err("AG00 不是完整的 256 字节记录".to_string());
    }
    let newline = source
        .windows(2)
        .position(|window| window == b"\r\n")
        .ok_or_else(|| "AG00 缺少 ASCII 头行 CRLF".to_string())?;
    let header_text =
        std::str::from_utf8(&source[..newline]).map_err(|_| "AG00 头行不是 ASCII".to_string())?;
    let values = header_text
        .split(',')
        .map(|part| {
            part.parse::<u16>()
                .map_err(|_| format!("AG00 头字段无效: {part:?}"))
        })
        .collect::<Result<Vec<_>>>()?;
    if values.len() != 4 {
        return Err("AG00 头必须恰有 4 个十进制字段".to_string());
    }
    let header = Ag00Header {
        max_variables: values[0],
        verb_count: values[1],
        object_count: values[2],
        flag_count: values[3],
    };
    let expected = usize::from(header.verb_count) + usize::from(header.object_count);
    let header_end = newline + 2;
    let mut pos = header_end;
    let mut entries = Vec::with_capacity(expected);
    for index in 0..expected {
        let entry_start = pos;
        if source.get(pos) == Some(&b'*') {
            pos += 1;
            if source.get(pos) != Some(&b'\r') {
                return Err(format!("AG00 entry {index}: * 后缺少 CR"));
            }
            pos += 1;
            entries.push(ParsedEntry {
                offset: entry_start,
                size: 1,
                text: "*".to_string(),
                structural: true,
            });
            continue;
        }
        if source.get(pos..pos + 2) != Some(b"\x1B\x4B") {
            return Err(format!("AG00 entry {index}: 缺少 ESC K 起始符"));
        }
        pos += 2;
        let body_start = pos;
        let mut text = String::new();
        loop {
            if source.get(pos..pos + 2) == Some(b"\x1B\x48") {
                break;
            }
            let pair = source
                .get(pos..pos + 2)
                .ok_or_else(|| format!("AG00 entry {index}: JIS 字符被截断"))?;
            let cp932 = jis_to_cp932([pair[0], pair[1]])?;
            let decoded = SHIFT_JIS
                .decode_without_bom_handling_and_without_replacement(&cp932)
                .ok_or_else(|| format!("AG00 entry {index}: JIS 字符不能转为 CP932"))?;
            let mut chars = decoded.chars();
            let character = chars.next().ok_or_else(|| "AG00 解码为空".to_string())?;
            if chars.next().is_some() {
                return Err(format!("AG00 entry {index}: 一个 JIS 单元解码为多个字符"));
            }
            text.push(character);
            pos += 2;
        }
        let body_size = pos - body_start;
        pos += 2;
        if source.get(pos) != Some(&b'\r') {
            return Err(format!("AG00 entry {index}: ESC H 后缺少 CR"));
        }
        pos += 1;
        entries.push(ParsedEntry {
            offset: body_start,
            size: body_size,
            text,
            structural: false,
        });
    }
    if source[pos..].iter().any(|byte| !matches!(*byte, 0 | 0x1A)) {
        return Err("AG00 条目结束后的填充含未知数据".to_string());
    }
    if source.last() != Some(&0x1A) {
        return Err("AG00 文件末字节不是 0x1A".to_string());
    }
    Ok(ParsedAg00 {
        header_end,
        header,
        entries,
    })
}

fn round_up(value: usize, unit: usize) -> Result<usize> {
    value
        .checked_add(unit - 1)
        .map(|sum| sum / unit * unit)
        .ok_or_else(|| "长度向上取整溢出".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jis_cp932_round_trip() {
        for character in ['見', 'る', '凜', 'Ａ'] {
            let cp932 = cp932_for_carrier(character).unwrap();
            assert_eq!(jis_to_cp932(cp932_to_jis(cp932).unwrap()).unwrap(), cp932);
        }
    }
}
