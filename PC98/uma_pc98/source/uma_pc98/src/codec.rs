use crate::Result;
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use vn_font::font_98::EncodingPlan;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub offset: usize,
    pub raw: Vec<u8>,
    pub message: String,
    /// Native expression glyphs are absent from the translation, restored at line ends.
    pub expression_lines: Vec<usize>,
}

pub fn is_lead(byte: u8) -> bool {
    (0x81..=0x9f).contains(&byte) || (0xe0..=0xfc).contains(&byte)
}

pub fn decode(raw: &[u8], filter_expression: bool) -> Result<(String, Vec<usize>)> {
    let mut text = String::new();
    let mut expressions = Vec::new();
    let (mut pos, mut line) = (0, 0);
    while pos < raw.len() {
        let byte = raw[pos];
        if filter_expression && raw.get(pos..pos + 2) == Some(&[0x86, 0xab]) {
            if raw.get(pos + 2).is_some_and(|b| *b != b'\n') {
                return Err("86 AB 不在已确认的行尾位置，拒绝猜测表情位置".into());
            }
            expressions.push(line);
            pos += 2;
            continue;
        }
        if (0x10..=0x1f).contains(&byte) {
            text.push_str(&format!("<COLOR:{:02X}>", byte - 0x10));
            pos += 1;
            continue;
        }
        if byte == b'\n' {
            line += 1;
        } else if byte < 0x20 || byte == 0x7f {
            return Err(format!("未支持的文本字节 0x{byte:02X}，位置 0x{pos:X}"));
        }
        let width = if is_lead(byte) { 2 } else { 1 };
        let unit = raw
            .get(pos..pos + width)
            .ok_or("文本末尾缺少双字节尾字节")?;
        if width == 2 && (!(0x40..=0xfc).contains(&unit[1]) || unit[1] == 0x7f) {
            return Err(format!("损坏的双字节字符，位置 0x{pos:X}"));
        }
        if let Some(decoded) = SHIFT_JIS.decode_without_bom_handling_and_without_replacement(unit) {
            text.push_str(&decoded);
        } else if width == 2 {
            text.push_str(&format!("<SJIS:{:02X}{:02X}>", unit[0], unit[1]));
        } else {
            return Err(format!("无效 CP932 字节 0x{byte:02X}"));
        }
        pos += width;
    }
    Ok((text, expressions))
}

pub fn mes_records(bytes: &[u8], count: usize) -> Result<Vec<Record>> {
    if bytes.len() != count * 112 {
        return Err(format!(
            "MES 大小应为 {} 字节，实际 {}",
            count * 112,
            bytes.len()
        ));
    }
    bytes
        .chunks_exact(112)
        .enumerate()
        .map(|(index, raw)| {
            let decoded: Vec<_> = raw.iter().map(|b| b ^ 0xff).collect();
            let end = decoded
                .iter()
                .rposition(|b| *b != b' ')
                .map_or(0, |p| p + 1);
            let (message, expression_lines) = decode(&decoded[..end], true)?;
            Ok(Record {
                offset: index * 112,
                raw: raw.to_vec(),
                message,
                expression_lines,
            })
        })
        .collect()
}

/// Extract protected native markers; their bytes are never sent to the font planner.
pub(crate) fn marker(text: &str) -> Result<Option<(usize, Vec<u8>)>> {
    for (prefix, digits) in [("<SJIS:", 4), ("<COLOR:", 2)] {
        if let Some(rest) = text.strip_prefix(prefix) {
            let value = rest.get(..digits).ok_or("原生字形/颜色标记不完整")?;
            if rest.as_bytes().get(digits) != Some(&b'>')
                || !value.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err("原生字形/颜色标记格式错误".into());
            }
            let number = u16::from_str_radix(value, 16).map_err(|e| e.to_string())?;
            let bytes = if digits == 4 {
                let bytes = number.to_be_bytes();
                if !is_lead(bytes[0]) || !(0x40..=0xfc).contains(&bytes[1]) || bytes[1] == 0x7f {
                    return Err("原生双字节标记越界".into());
                }
                bytes.to_vec()
            } else {
                if number > 15 {
                    return Err("颜色索引超出 0..15".into());
                }
                vec![0x10 + number as u8]
            };
            return Ok(Some((prefix.len() + digits + 1, bytes)));
        }
    }
    Ok(None)
}

pub fn format_signature(text: &str, mes: bool) -> Result<Vec<String>> {
    let bytes = text.as_bytes();
    let mut result = Vec::new();
    let mut pos = 0;
    while pos < bytes.len() {
        let rest = &text[pos..];
        if let Some((len, native)) = marker(rest)? {
            if native.len() == 2 {
                result.push(rest[..len].to_string());
            }
            pos += len;
            continue;
        }
        let byte = bytes[pos];
        if byte < 0x20 && byte != b'\n' || byte == 0x7f {
            return Err("文本只允许物理 LF 换行，不能包含 NUL、CR 或其他控制字节".into());
        }
        if mes && byte == b'%' {
            return Err("MES 使用 $ 占位符，不能直接加入 %".into());
        }
        if byte != if mes { b'$' } else { b'%' } {
            pos += rest.chars().next().ok_or("文本截断")?.len_utf8();
            continue;
        }
        let start = pos;
        pos += 1;
        if !mes && bytes.get(pos) == Some(&b'%') {
            pos += 1;
        } else {
            while bytes
                .get(pos)
                .is_some_and(|b| b"-+ #0".contains(b) || b.is_ascii_digit())
            {
                pos += 1;
            }
            if bytes.get(pos) == Some(&b'.') {
                pos += 1;
                while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
                    pos += 1;
                }
            }
            if bytes.get(pos).is_some_and(|b| b"hlL".contains(b)) {
                pos += 1;
            }
            if !bytes
                .get(pos)
                .is_some_and(|b| b"diuoxXscfFeEgG".contains(b))
            {
                return Err(format!("不支持或损坏的格式占位符，UTF-8 位置 {start}"));
            }
            pos += 1;
        }
        result.push(text[start..pos].to_string());
    }
    Ok(result)
}

pub fn validate_translation(record: &Record, translated: &str, mes: bool) -> Result<()> {
    if format_signature(&record.message, mes)? != format_signature(translated, mes)? {
        return Err("占位符/原生标记的类型、宽度、数量或顺序发生变化".into());
    }
    if translated.ends_with(' ') && mes {
        return Err("MES 译文不能以半角填充空格结尾".into());
    }
    let lines = translated.split('\n').count();
    if record.expression_lines.iter().any(|line| *line >= lines) {
        return Err("译文删除了保留表情字形所在的行".into());
    }
    Ok(())
}

pub fn font_text(text: &str) -> Result<String> {
    let mut display = String::new();
    let mut pos = 0;
    while pos < text.len() {
        let rest = &text[pos..];
        if let Some((len, _)) = marker(rest)? {
            pos += len;
            continue;
        }
        let ch = rest.chars().next().ok_or("文本截断")?;
        let s = ch.to_string();
        let (encoded, _, errors) = SHIFT_JIS.encode(&s);
        if ch != '\n' && (errors || encoded.len() != 1) {
            display.push(ch);
        }
        pos += ch.len_utf8();
    }
    Ok(display)
}

pub(crate) fn encode(text: &str, plan: &EncodingPlan) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut pos = 0;
    while pos < text.len() {
        let rest = &text[pos..];
        if let Some((len, bytes)) = marker(rest)? {
            output.extend(bytes);
            pos += len;
            continue;
        }
        let ch = rest.chars().next().ok_or("文本截断")?;
        let s = ch.to_string();
        let (encoded, _, errors) = SHIFT_JIS.encode(&s);
        if !errors && encoded.len() == 1 {
            output.extend_from_slice(&encoded);
        } else {
            output.extend(plan.encode_cp932(&s)?);
        }
        pos += ch.len_utf8();
    }
    Ok(output)
}

pub fn rebuild(
    record: &Record,
    translated: &str,
    mes: bool,
    plan: &EncodingPlan,
) -> Result<Vec<u8>> {
    validate_translation(record, translated, mes)?;
    if translated == record.message {
        return Ok(record.raw.clone());
    }
    let mut encoded = Vec::new();
    for (line, text) in translated.split('\n').enumerate() {
        if line > 0 {
            encoded.push(b'\n');
        }
        encoded.extend(encode(text, plan)?);
        for _ in record.expression_lines.iter().filter(|n| **n == line) {
            encoded.extend([0x86, 0xab]);
        }
    }
    let capacity = if mes {
        record.raw.len()
    } else {
        record.raw.len() - 1
    };
    if encoded.len() > capacity {
        return Err(format!(
            "译文占 {} 字节，槽位最多 {capacity} 字节（含保留表情）",
            encoded.len()
        ));
    }
    if mes {
        encoded.resize(capacity, b' ');
        encoded.iter_mut().for_each(|b| *b ^= 0xff);
    } else {
        encoded.resize(record.raw.len(), 0);
    }
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vn_font::font_98::SubstitutionMap;
    fn plan(text: &str) -> EncodingPlan {
        EncodingPlan::build(
            &SubstitutionMap::embedded().unwrap(),
            [],
            [font_text(text).unwrap().as_str()],
        )
        .unwrap()
    }
    #[test]
    fn hidden_expression_and_padding_survive_translation() {
        let mut source = b"test".to_vec();
        source.extend([0x86, 0xab, b'\n', b'$', b'd']);
        source.resize(112, b' ');
        source.iter_mut().for_each(|b| *b ^= 0xff);
        let r = mes_records(&source, 1).unwrap().remove(0);
        assert_eq!(r.message, "test\n$d");
        assert_eq!(
            rebuild(&r, &r.message, true, &plan(&r.message)).unwrap(),
            source
        );
        let changed = rebuild(&r, "ok\n$d", true, &plan("ok\n$d")).unwrap();
        assert_eq!(
            changed[..7].iter().map(|b| b ^ 0xff).collect::<Vec<_>>(),
            b"ok\x86\xab\n$d"
        );
        assert!(rebuild(&r, "$s", true, &plan("$s")).is_err());
        assert!(rebuild(
            &r,
            &format!("{}\n$d", "a".repeat(110)),
            true,
            &plan("a\n$d")
        )
        .is_err());
    }
    #[test]
    fn fixed_exe_slots_and_native_markers_are_protected() {
        let r = Record {
            offset: 0,
            raw: vec![b'a', 0xeb, 0x9f, 0],
            message: "a<SJIS:EB9F>".into(),
            expression_lines: vec![],
        };
        let bytes = rebuild(&r, "b<SJIS:EB9F>", false, &plan("b")).unwrap();
        assert_eq!(bytes, [b'b', 0xeb, 0x9f, 0]);
        assert!(rebuild(&r, "b", false, &plan("b")).is_err());
        assert!(format_signature("%n", false).is_err());
        assert!(format_signature("%", false).is_err());
        assert!(format_signature("$s\0", true).is_err());
        assert_eq!(
            format_signature("$7ld $s $d", true).unwrap(),
            ["$7ld", "$s", "$d"]
        );
        assert_eq!(
            format_signature("<COLOR:0F>$s<COLOR:02>", true).unwrap(),
            ["$s"]
        );
    }

    #[cfg(windows)]
    #[test]
    fn chinese_injection_and_font_use_the_same_carriers() {
        use vn_font::font_98;
        let source = Record {
            offset: 0,
            raw: b"original\0".to_vec(),
            message: "original".into(),
            expression_lines: vec![],
        };
        let reserved = std::collections::BTreeSet::from([
            u16::from_be_bytes(font_98::cp932_for_carrier('日').unwrap()),
            0x86ab,
        ]);
        let plan = EncodingPlan::build(
            &SubstitutionMap::embedded().unwrap(),
            reserved.iter().copied(),
            ["你好"],
        )
        .unwrap();
        let encoded = rebuild(&source, "你好", false, &plan).unwrap();
        assert_eq!(&encoded[..4], plan.encode_cp932("你好").unwrap());
        let font = font_98::prepare_font(
            font_98::EMBEDDED_FONT,
            &plan.requests(),
            &reserved,
            font_98::FONT_FACE,
        )
        .unwrap();
        assert!(font.patched_glyphs >= 1);
        assert_eq!(
            font_98::read_glyph(font_98::EMBEDDED_FONT, '日').unwrap(),
            font_98::read_glyph(&font.bytes, '日').unwrap()
        );
    }
}
