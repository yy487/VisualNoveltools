//! Lossless SCR parser and translation round trip.
//
// CPMAIN keeps display strings in a shared payload and addresses them from
// opcode records. The public translation JSON intentionally contains only
// editable strings; records, encoded bytes, and resource names stay inside
// the source SCR and are re-read during injection.

use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SCHEMA: &str = "chrome-paradise-scr-translations-v2";
pub const STRING_OPCODES: [u8; 3] = [0x04, 0x06, 0x19];
const TEXT_OPCODE: u8 = 0x02;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScrDocument {
    pub schema: String,
    pub source_file: String,
    pub source_sha256: String,
    pub signature: String,
    pub count_a: u16,
    pub count_b: u16,
    pub strings: Vec<ScrString>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScrString {
    pub index: usize,
    pub offset: usize,
    pub scr_msg: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker_id: Option<u16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub halfwidth_kana: Vec<HalfwidthKanaSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HalfwidthKanaSpan {
    pub start: usize,
    pub end: usize,
    pub normalized: String,
    pub original: String,
}

#[derive(Debug, Clone)]
struct ScrRecord {
    opcode: u8,
    selector: u8,
    arg_a: u16,
    arg_b: u16,
}

#[derive(Debug, Clone)]
struct ParsedString {
    index: usize,
    offset: usize,
    bytes: Vec<u8>,
    message: String,
    halfwidth_kana: Vec<HalfwidthKanaSpan>,
}

#[derive(Debug, Clone)]
struct ParsedDocument {
    count_a: u16,
    count_b: u16,
    records: Vec<ScrRecord>,
    strings: Vec<ParsedString>,
    trailing: Vec<u8>,
    diagnostics: Vec<String>,
}

pub fn extract_document(raw: &[u8], source_file: impl Into<String>) -> Result<ScrDocument, String> {
    let source_file = source_file.into();
    let parsed = parse_document(raw, source_file.clone())?;
    let resource_offsets = resource_offsets(&parsed.records);
    let speaker_ids = speaker_ids(&parsed.records);
    let mut strings = Vec::new();
    let mut filtered = 0usize;
    for item in &parsed.strings {
        if resource_offsets.contains(&item.offset) || is_resource_identifier(&item.message) {
            filtered += 1;
            continue;
        }
        strings.push(ScrString {
            index: item.index,
            offset: item.offset,
            scr_msg: item.message.clone(),
            message: item.message.clone(),
            speaker_id: speaker_ids.get(&item.offset).copied(),
            halfwidth_kana: item.halfwidth_kana.clone(),
        });
    }
    let mut diagnostics = parsed.diagnostics;
    if filtered != 0 {
        diagnostics.push(format!("已过滤 {filtered} 个资源或内部字符串"));
    }
    Ok(ScrDocument {
        schema: SCHEMA.into(),
        source_file,
        source_sha256: fivec_new::sha256(raw),
        signature: "SCR:".into(),
        count_a: parsed.count_a,
        count_b: parsed.count_b,
        strings,
        diagnostics,
    })
}

pub fn apply_document(raw: &[u8], document: &ScrDocument) -> Result<Vec<u8>, String> {
    apply_document_internal(raw, document, None)
}

/// Apply already planned raw display bytes to a document. This is used when
/// the same CP932/font carrier plan produced the corresponding font artifact.
pub fn apply_document_with_encoded_strings(
    raw: &[u8],
    document: &ScrDocument,
    encoded_strings: &[Vec<u8>],
) -> Result<Vec<u8>, String> {
    apply_document_internal(raw, document, Some(encoded_strings))
}

pub fn encode_document_message(item: &ScrString) -> Result<Vec<u8>, String> {
    let restored = restore_halfwidth_kana(&item.message, &item.halfwidth_kana)?;
    encode_message(&restored)
}

pub fn restore_halfwidth_kana(text: &str, spans: &[HalfwidthKanaSpan]) -> Result<String, String> {
    let mut chars: Vec<char> = text.chars().collect();
    for span in spans.iter().rev() {
        if span.start > span.end {
            return Err(format!(
                "半角假名样式范围无效: {}..{}",
                span.start, span.end
            ));
        }
        // These spans describe positions in the source Japanese string. A
        // shorter translation may no longer contain that position, in which
        // case there is nothing to restore at the old offset.
        if span.end > chars.len() {
            continue;
        }
        let current: String = chars[span.start..span.end].iter().collect();
        if current == span.normalized {
            chars.splice(span.start..span.end, span.original.chars());
        }
    }
    Ok(chars.into_iter().collect())
}

fn apply_document_internal(
    raw: &[u8],
    document: &ScrDocument,
    planned_strings: Option<&[Vec<u8>]>,
) -> Result<Vec<u8>, String> {
    if document.schema != SCHEMA {
        return Err(format!("不支持的 SCR JSON schema: {}", document.schema));
    }
    if fivec_new::sha256(raw) != document.source_sha256 {
        return Err("SCR 原文件 SHA-256 与 JSON 不一致，拒绝注回".into());
    }
    let baseline = parse_document(raw, document.source_file.clone())?;
    let structural = blocking_diagnostics(&baseline.diagnostics);
    if !structural.is_empty() {
        return Err(format!(
            "原始 SCR 存在结构诊断，先处理: {}",
            structural.join("; ")
        ));
    }
    if baseline.count_a != document.count_a || baseline.count_b != document.count_b {
        return Err("SCR JSON 与原文件的记录数量不一致".into());
    }
    let baseline_public = public_strings(&baseline);
    if baseline_public.len() != document.strings.len() {
        return Err("SCR JSON 与原文件的可翻译字符串数量不一致".into());
    }
    if planned_strings.is_some_and(|planned| planned.len() != document.strings.len()) {
        return Err("字体计划的字符串数量与 SCR JSON 不一致".into());
    }
    let mut translated_by_index = BTreeMap::new();
    for (position, (baseline_item, translated)) in
        baseline_public.iter().zip(&document.strings).enumerate()
    {
        if baseline_item.index != translated.index
            || baseline_item.offset != translated.offset
            || baseline_item.scr_msg != translated.scr_msg
            || baseline_item.speaker_id != translated.speaker_id
            || baseline_item.halfwidth_kana != translated.halfwidth_kana
        {
            return Err(format!("SCR 字符串 {} 的来源字段被修改", translated.index));
        }
        let encoded = match planned_strings {
            Some(planned) => planned[position].clone(),
            None => encode_document_message(translated)?,
        };
        translated_by_index.insert(translated.index, encoded);
    }

    let mut payload = Vec::new();
    let mut new_offsets = BTreeMap::new();
    for item in &baseline.strings {
        new_offsets.insert(item.offset, payload.len());
        if let Some(encoded) = translated_by_index.get(&item.index) {
            payload.extend_from_slice(encoded);
        } else {
            payload.extend_from_slice(&item.bytes);
        }
        payload.push(0);
    }
    if payload.len() > usize::from(u16::MAX) {
        return Err("注回后的 SCR 字符串区超过 u16 长度".into());
    }
    let records_end = 8usize
        .checked_add(
            baseline
                .records
                .len()
                .checked_mul(6)
                .ok_or("SCR 记录区溢出")?,
        )
        .ok_or("SCR 记录区溢出")?;
    let mut output = Vec::with_capacity(records_end + 2 + payload.len() + baseline.trailing.len());
    output.extend_from_slice(&raw[..8]);
    for record in &baseline.records {
        let mut bytes = [
            record.opcode,
            record.selector,
            (record.arg_a & 0xff) as u8,
            (record.arg_a >> 8) as u8,
            (record.arg_b & 0xff) as u8,
            (record.arg_b >> 8) as u8,
        ];
        if let Some(old_offset) = record_string_offset(record) {
            if let Some(new_offset) = new_offsets.get(&usize::from(old_offset)) {
                if *new_offset > usize::from(u16::MAX) {
                    return Err("字符串偏移超过 u16 范围".into());
                }
                let value = *new_offset as u16;
                if record.opcode == TEXT_OPCODE {
                    bytes[4] = (value & 0xff) as u8;
                    bytes[5] = (value >> 8) as u8;
                } else {
                    bytes[2] = (value & 0xff) as u8;
                    bytes[3] = (value >> 8) as u8;
                }
            }
        }
        output.extend_from_slice(&bytes);
    }
    output.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    output.extend(payload.iter().map(|byte| byte ^ 0x7f));
    output.extend_from_slice(&baseline.trailing);
    let verified = parse_document(&output, document.source_file.clone())?;
    let verified_public = public_strings(&verified);
    if verified_public.len() != document.strings.len() {
        return Err("注回后可翻译字符串数量发生变化".into());
    }
    let verified_by_index = verified
        .strings
        .iter()
        .map(|item| (item.index, item))
        .collect::<BTreeMap<_, _>>();
    for (expected, actual) in document.strings.iter().zip(verified_public) {
        if planned_strings.is_some() {
            let encoded = translated_by_index
                .get(&expected.index)
                .ok_or("字体计划字符串缺失")?;
            let actual_bytes = verified_by_index
                .get(&actual.index)
                .ok_or("注回后字符串索引缺失")?
                .bytes
                .as_slice();
            if actual_bytes != encoded.as_slice() {
                return Err(format!(
                    "字符串 {} 原始编码注回后复核不一致",
                    expected.index
                ));
            }
        } else if expected.message != actual.message {
            return Err(format!("字符串 {} 注回后复核不一致", expected.index));
        }
    }
    Ok(output)
}

fn parse_document(raw: &[u8], _source_file: String) -> Result<ParsedDocument, String> {
    if raw.len() < 8 || &raw[..4] != b"SCR:" {
        return Err("不是 CPMAIN 识别的 SCR 文件".into());
    }
    let count_a = u16::from_le_bytes([raw[4], raw[5]]);
    let count_b = u16::from_le_bytes([raw[6], raw[7]]);
    let record_count = usize::from(count_a)
        .checked_add(usize::from(count_b))
        .ok_or_else(|| "SCR 记录数量溢出".to_string())?;
    let records_end = 8usize
        .checked_add(record_count.checked_mul(6).ok_or("SCR 记录区溢出")?)
        .ok_or("SCR 记录区溢出")?;
    if raw.len() < records_end + 2 {
        return Err("SCR 记录区截断".into());
    }
    let mut records = Vec::with_capacity(record_count);
    for index in 0..record_count {
        let offset = 8 + index * 6;
        let bytes = &raw[offset..offset + 6];
        records.push(ScrRecord {
            opcode: bytes[0],
            selector: bytes[1],
            arg_a: u16::from_le_bytes([bytes[2], bytes[3]]),
            arg_b: u16::from_le_bytes([bytes[4], bytes[5]]),
        });
    }
    let payload_length = usize::from(u16::from_le_bytes([raw[records_end], raw[records_end + 1]]));
    let payload_offset = records_end + 2;
    let payload_end = payload_offset
        .checked_add(payload_length)
        .ok_or_else(|| "SCR 字符串区溢出".to_string())?;
    if raw.len() < payload_end {
        return Err("SCR 字符串区截断".into());
    }
    let decoded_payload: Vec<u8> = raw[payload_offset..payload_end]
        .iter()
        .map(|byte| byte ^ 0x7f)
        .collect();
    let mut diagnostics = Vec::new();
    let mut strings = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    while cursor < decoded_payload.len() {
        let start = cursor;
        let end = decoded_payload[cursor..]
            .iter()
            .position(|byte| *byte == 0)
            .map(|relative| cursor + relative)
            .unwrap_or(decoded_payload.len());
        let bytes = &decoded_payload[start..end];
        let (decoded, _reversible, special_bytes) = decode_message(bytes);
        let (message, halfwidth_kana) = normalize_halfwidth_kana(&decoded);
        strings.push(ParsedString {
            index,
            offset: start,
            bytes: bytes.to_vec(),
            message,
            halfwidth_kana,
        });
        for (local_offset, code) in special_bytes {
            diagnostics.push(format!(
                "字符串区偏移 0x{:04X} 含 CPMAIN 扩展码位 {}",
                start + local_offset,
                code
            ));
        }
        index += 1;
        if end == decoded_payload.len() {
            diagnostics.push(format!("字符串 {index} 没有 NUL 终止符"));
            cursor = end;
        } else {
            cursor = end + 1;
        }
    }
    if decoded_payload.is_empty() {
        diagnostics.push("字符串区为空".into());
    }
    for (index, record) in records.iter().enumerate() {
        if record.opcode > 0x25 {
            diagnostics.push(format!(
                "记录 {index} 的 opcode 0x{:02X} 超出 CPMAIN 跳转表 0x00..0x25",
                record.opcode
            ));
        }
    }
    if payload_end < raw.len() {
        diagnostics.push(format!(
            "字符串区后有 {} 字节未解释数据",
            raw.len() - payload_end
        ));
    }
    Ok(ParsedDocument {
        count_a,
        count_b,
        records,
        strings,
        trailing: raw[payload_end..].to_vec(),
        diagnostics,
    })
}

fn public_strings(parsed: &ParsedDocument) -> Vec<ScrString> {
    let resource_offsets = resource_offsets(&parsed.records);
    let speaker_ids = speaker_ids(&parsed.records);
    parsed
        .strings
        .iter()
        .filter(|item| {
            !resource_offsets.contains(&item.offset) && !is_resource_identifier(&item.message)
        })
        .map(|item| ScrString {
            index: item.index,
            offset: item.offset,
            scr_msg: item.message.clone(),
            message: item.message.clone(),
            speaker_id: speaker_ids.get(&item.offset).copied(),
            halfwidth_kana: item.halfwidth_kana.clone(),
        })
        .collect()
}

fn resource_offsets(records: &[ScrRecord]) -> BTreeSet<usize> {
    records
        .iter()
        .filter(|record| STRING_OPCODES.contains(&record.opcode))
        .map(|record| usize::from(record.arg_a))
        .collect()
}

fn speaker_ids(records: &[ScrRecord]) -> BTreeMap<usize, u16> {
    let mut result = BTreeMap::new();
    for record in records {
        if record.opcode == TEXT_OPCODE && record.arg_a != u16::MAX {
            result
                .entry(usize::from(record.arg_b))
                .or_insert(record.arg_a);
        }
    }
    result
}

fn record_string_offset(record: &ScrRecord) -> Option<u16> {
    if record.opcode == TEXT_OPCODE {
        Some(record.arg_b)
    } else if STRING_OPCODES.contains(&record.opcode) {
        Some(record.arg_a)
    } else {
        None
    }
}

fn is_resource_identifier(text: &str) -> bool {
    let mut parts = text.split('_');
    let Some(first) = parts.next() else {
        return false;
    };
    let Some(second) = parts.next() else {
        return false;
    };
    parts.next().is_none()
        && !first.is_empty()
        && first.chars().all(|ch| ch.is_ascii_digit())
        && !second.is_empty()
        && second.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn normalize_halfwidth_kana(text: &str) -> (String, Vec<HalfwidthKanaSpan>) {
    let mut output = String::new();
    let mut spans = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0usize;
    while index < chars.len() {
        let Some(base) = halfwidth_to_fullwidth(chars[index]) else {
            output.push(chars[index]);
            index += 1;
            continue;
        };
        let mut original = chars[index].to_string();
        let mut normalized = base.to_string();
        if index + 1 < chars.len() && (chars[index + 1] == 'ﾞ' || chars[index + 1] == 'ﾟ') {
            original.push(chars[index + 1]);
            let mark = if chars[index + 1] == 'ﾞ' {
                '゛'
            } else {
                '゜'
            };
            normalized = compose_kana(base, mark)
                .map(|character| character.to_string())
                .unwrap_or_else(|| format!("{base}{mark}"));
            index += 1;
        }
        let start = output.chars().count();
        output.push_str(&normalized);
        let end = output.chars().count();
        spans.push(HalfwidthKanaSpan {
            start,
            end,
            normalized,
            original,
        });
        index += 1;
    }
    (output, spans)
}

fn halfwidth_to_fullwidth(ch: char) -> Option<char> {
    Some(match ch {
        '｡' => '。',
        '｢' => '「',
        '｣' => '」',
        '､' => '、',
        '･' => '・',
        'ｦ' => 'ヲ',
        'ｧ' => 'ァ',
        'ｨ' => 'ィ',
        'ｩ' => 'ゥ',
        'ｪ' => 'ェ',
        'ｫ' => 'ォ',
        'ｬ' => 'ャ',
        'ｭ' => 'ュ',
        'ｮ' => 'ョ',
        'ｯ' => 'ッ',
        'ｰ' => 'ー',
        'ｱ' => 'ア',
        'ｲ' => 'イ',
        'ｳ' => 'ウ',
        'ｴ' => 'エ',
        'ｵ' => 'オ',
        'ｶ' => 'カ',
        'ｷ' => 'キ',
        'ｸ' => 'ク',
        'ｹ' => 'ケ',
        'ｺ' => 'コ',
        'ｻ' => 'サ',
        'ｼ' => 'シ',
        'ｽ' => 'ス',
        'ｾ' => 'セ',
        'ｿ' => 'ソ',
        'ﾀ' => 'タ',
        'ﾁ' => 'チ',
        'ﾂ' => 'ツ',
        'ﾃ' => 'テ',
        'ﾄ' => 'ト',
        'ﾅ' => 'ナ',
        'ﾆ' => 'ニ',
        'ﾇ' => 'ヌ',
        'ﾈ' => 'ネ',
        'ﾉ' => 'ノ',
        'ﾊ' => 'ハ',
        'ﾋ' => 'ヒ',
        'ﾌ' => 'フ',
        'ﾍ' => 'ヘ',
        'ﾎ' => 'ホ',
        'ﾏ' => 'マ',
        'ﾐ' => 'ミ',
        'ﾑ' => 'ム',
        'ﾒ' => 'メ',
        'ﾓ' => 'モ',
        'ﾔ' => 'ヤ',
        'ﾕ' => 'ユ',
        'ﾖ' => 'ヨ',
        'ﾗ' => 'ラ',
        'ﾘ' => 'リ',
        'ﾙ' => 'ル',
        'ﾚ' => 'レ',
        'ﾛ' => 'ロ',
        'ﾜ' => 'ワ',
        'ﾝ' => 'ン',
        'ﾞ' => '゛',
        'ﾟ' => '゜',
        _ => return None,
    })
}

fn compose_kana(base: char, mark: char) -> Option<char> {
    Some(match (base, mark) {
        ('ウ', '゛') => 'ヴ',
        ('カ', '゛') => 'ガ',
        ('キ', '゛') => 'ギ',
        ('ク', '゛') => 'グ',
        ('ケ', '゛') => 'ゲ',
        ('コ', '゛') => 'ゴ',
        ('サ', '゛') => 'ザ',
        ('シ', '゛') => 'ジ',
        ('ス', '゛') => 'ズ',
        ('セ', '゛') => 'ゼ',
        ('ソ', '゛') => 'ゾ',
        ('タ', '゛') => 'ダ',
        ('チ', '゛') => 'ヂ',
        ('ツ', '゛') => 'ヅ',
        ('テ', '゛') => 'デ',
        ('ト', '゛') => 'ド',
        ('ハ', '゛') => 'バ',
        ('ヒ', '゛') => 'ビ',
        ('フ', '゛') => 'ブ',
        ('ヘ', '゛') => 'ベ',
        ('ホ', '゛') => 'ボ',
        ('ハ', '゜') => 'パ',
        ('ヒ', '゜') => 'ピ',
        ('フ', '゜') => 'プ',
        ('ヘ', '゜') => 'ペ',
        ('ホ', '゜') => 'ポ',
        _ => return None,
    })
}

fn decode_message(bytes: &[u8]) -> (String, bool, Vec<(usize, String)>) {
    let mut text = String::new();
    let mut reversible = true;
    let mut special_bytes = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == 0x0a {
            text.push('\n');
            index += 1;
            continue;
        }
        if byte == 0x02 {
            if let Some(&reference) = bytes.get(index + 1) {
                text.push_str(&format!("[[REF:{reference:02X}]]"));
                index += 2;
            } else {
                // CPMAIN consumes a one-byte reference argument after 0x02.
                // Keep a truncated marker losslessly for diagnostics.
                text.push_str("[[CTRL:02]]");
                reversible = false;
                index += 1;
            }
            continue;
        }
        let width = if (0x81..=0x9f).contains(&byte) || (0xe0..=0xfc).contains(&byte) {
            2
        } else {
            1
        };
        if width == 2 && index + width <= bytes.len() {
            let (decoded, had_errors) =
                SHIFT_JIS.decode_without_bom_handling(&bytes[index..index + width]);
            if !had_errors && !decoded.contains('\u{fffd}') {
                text.push_str(&decoded);
                index += width;
                continue;
            }
        }
        if byte != 0x80 && byte != 0xa0 && width == 1 {
            let (decoded, had_errors) =
                SHIFT_JIS.decode_without_bom_handling(&bytes[index..index + 1]);
            if !had_errors && !decoded.contains('\u{fffd}') {
                text.push_str(&decoded);
                index += 1;
                continue;
            }
        }
        if index + 1 < bytes.len()
            && (byte == 0x80 || byte >= 0xa0)
            && !matches!(bytes[index + 1], 0x00 | 0x02 | 0x0a)
        {
            let code = format!("{byte:02X}{:02X}", bytes[index + 1]);
            text.push_str(&format!("[[CODE:{code}]]"));
            reversible = false;
            special_bytes.push((index, code));
            index += 2;
            continue;
        }
        if index + width > bytes.len() {
            text.push_str(&format!("[[BYTE:{byte:02X}]]"));
            reversible = false;
            index += 1;
            continue;
        }
        if byte == 0x80 || byte == 0xa0 {
            text.push_str(&format!("[[BYTE:{byte:02X}]]"));
            reversible = false;
            index += 1;
            continue;
        }
        let (decoded, had_errors) =
            SHIFT_JIS.decode_without_bom_handling(&bytes[index..index + width]);
        if had_errors || decoded.contains('\u{fffd}') {
            text.push_str(&format!("[[BYTE:{byte:02X}]]"));
            reversible = false;
            index += 1;
        } else {
            text.push_str(&decoded);
            index += width;
        }
    }
    (text, reversible, special_bytes)
}

fn encode_message(text: &str) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let mut cursor = 0usize;
    while cursor < text.len() {
        let rest = &text[cursor..];
        if let Some(end) = rest.find("]]") {
            if rest.starts_with("[[BYTE:")
                || rest.starts_with("[[CTRL:")
                || rest.starts_with("[[CODE:")
                || rest.starts_with("[[REF:")
            {
                let token = &rest[..end + 2];
                let prefix = if rest.starts_with("[[BYTE:") {
                    "[[BYTE:"
                } else if rest.starts_with("[[CTRL:") {
                    "[[CTRL:"
                } else if rest.starts_with("[[REF:") {
                    "[[REF:"
                } else {
                    "[[CODE:"
                };
                let digits = if prefix == "[[CODE:" { 4 } else { 2 };
                if token.len() == prefix.len() + digits + 2
                    && token.ends_with("]]")
                    && (0..digits)
                        .all(|offset| token.as_bytes()[prefix.len() + offset].is_ascii_hexdigit())
                {
                    let value =
                        u16::from_str_radix(&token[prefix.len()..prefix.len() + digits], 16)
                            .map_err(|_| format!("非法 SCR 字节标记: {token}"))?;
                    if rest.starts_with("[[BYTE:")
                        || (rest.starts_with("[[CTRL:") && (value == 0x02 || value == 0x0a))
                    {
                        output.push(value as u8);
                    } else if rest.starts_with("[[REF:") && value <= 0xff {
                        output.extend_from_slice(&[0x02, value as u8]);
                    } else if rest.starts_with("[[CODE:") && value > 0xff {
                        output.push((value >> 8) as u8);
                        output.push(value as u8);
                    } else {
                        return Err(format!("不支持的 SCR 控制标记: {token}"));
                    }
                    cursor += token.len();
                    continue;
                }
            }
        }
        let character = text[cursor..]
            .chars()
            .next()
            .ok_or_else(|| "UTF-8 文本截断".to_string())?;
        cursor += character.len_utf8();
        if character == '\n' {
            output.push(0x0a);
            continue;
        }
        if character == '\0' || character == '\u{0002}' {
            return Err("翻译文本含未转义 NUL 或控制字符".into());
        }
        let mut buffer = [0u8; 4];
        let character_text = character.encode_utf8(&mut buffer);
        let (encoded, _encoding_used, had_errors) = SHIFT_JIS.encode(character_text);
        if had_errors {
            return Err(format!(
                "字符 {:?} 无法编码为 CP932；自定义字形请先使用 [[CODE:HHHH]] 映射",
                character
            ));
        }
        output.extend_from_slice(&encoded);
    }
    Ok(output)
}

fn blocking_diagnostics(diagnostics: &[String]) -> Vec<String> {
    diagnostics
        .iter()
        .filter(|message| {
            !message.contains("含 CPMAIN 扩展码位") && !message.starts_with("已过滤 ")
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic(records: &[u8], payload: &[u8]) -> Vec<u8> {
        assert_eq!(records.len() % 6, 0);
        let mut raw = Vec::new();
        raw.extend_from_slice(b"SCR:");
        raw.extend_from_slice(&((records.len() / 6) as u16).to_le_bytes());
        raw.extend_from_slice(&0u16.to_le_bytes());
        raw.extend_from_slice(records);
        raw.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        raw.extend(payload.iter().map(|byte| byte ^ 0x7f));
        raw
    }

    #[test]
    fn filters_resources_and_rewrites_text_pointer() {
        let records = [0x02, 0, 7, 0, 6, 0, 0x06, 0, 2, 0, 0, 0];
        let raw = synthetic(&records, b"A\0M_3\0B\0");
        let mut document = extract_document(&raw, "test.SCR").unwrap();
        assert_eq!(document.strings.len(), 2);
        assert_eq!(document.strings[0].message, "A");
        assert_eq!(document.strings[1].message, "B");
        assert_eq!(document.strings[1].speaker_id, Some(7));
        document.strings[0].message = "A-LONG".into();
        let rebuilt = apply_document(&raw, &document).unwrap();
        let parsed = parse_document(&rebuilt, "test.SCR".into()).unwrap();
        assert_eq!(parsed.records[0].arg_b, 11);
        assert_eq!(parsed.records[1].arg_a, 7);
        assert_eq!(parsed.strings[2].message, "B");
        assert_eq!(parsed.strings[2].bytes, b"B");
    }

    #[test]
    fn normalizes_and_restores_halfwidth_kana() {
        let raw = synthetic(&[0x02, 0, 0, 0, 0, 0], b"\xB6\xDE\xC5\0");
        let document = extract_document(&raw, "test.SCR").unwrap();
        assert_eq!(document.strings[0].message, "ガナ");
        assert_eq!(document.strings[0].scr_msg, "ガナ");
        assert_eq!(apply_document(&raw, &document).unwrap(), raw);
    }

    #[test]
    fn shorter_translation_ignores_out_of_range_source_style_spans() {
        let spans = [HalfwidthKanaSpan {
            start: 6,
            end: 7,
            normalized: "ナ".into(),
            original: "ﾅ".into(),
        }];
        assert_eq!(restore_halfwidth_kana("中文", &spans).unwrap(), "中文");
    }

    #[test]
    fn preserves_controls_and_non_cp932_bytes() {
        let raw = synthetic(&[0x02, 0, 0, 0, 0, 0], b"ABC\n\x02\x80\0");
        let document = extract_document(&raw, "test.SCR").unwrap();
        assert_eq!(document.strings[0].message, "ABC\n[[REF:80]]");
        assert_eq!(apply_document(&raw, &document).unwrap(), raw);
    }

    #[test]
    fn preserves_cpmain_extension_pair() {
        let raw = synthetic(&[0x02, 0, 0, 0, 0, 0], b"X\xeb\xaf\0");
        let document = extract_document(&raw, "test.SCR").unwrap();
        assert_eq!(document.strings[0].message, "X[[CODE:EBAF]]");
        assert_eq!(apply_document(&raw, &document).unwrap(), raw);
    }
}
