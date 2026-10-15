use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use vn_text::Entry;

pub type Result<T> = std::result::Result<T, String>;
pub const SPECIAL: [u16; 5] = [0x8197, 0x8190, 0x816f, 0x8170, 0x8196];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NameSource {
    pub call: Span,
    pub prefix: Vec<u8>,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub span: Span,
    pub name_source: Option<NameSource>,
    pub entry: Entry,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Primary {
    pub file: String,
    pub original: Vec<u8>,
    pub records: Vec<Record>,
    pub pc98_glyphs: Vec<Span>,
}

#[derive(Clone, Debug)]
pub enum Kind {
    Text(String),
    Newline,
    Op(u8, Vec<Arg>),
    Boundary(u8),
    Pc98Glyph,
    Control,
}

#[derive(Clone, Debug)]
pub enum Arg {
    Number(u16),
    String(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct Token {
    pub span: Span,
    pub kind: Kind,
}

#[derive(Clone, Debug)]
pub struct Macro {
    pub body: Vec<u8>,
    pub name: Option<(String, Vec<u8>)>,
    pub plain: Option<String>,
}

pub type Macros = BTreeMap<u16, Macro>;

pub fn is_lead(b: u8) -> bool {
    (0x81..=0x9f).contains(&b) || (0xe0..=0xfc).contains(&b)
}

fn decode(bytes: &[u8]) -> Result<String> {
    SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(bytes)
        .map(|s| s.into_owned())
        .ok_or_else(|| format!("非法 CP932 字节: {bytes:02X?}"))
}

pub fn number(bytes: &[u8], p: &mut usize) -> Result<Option<u16>> {
    let Some(&b) = bytes.get(*p) else {
        return Ok(None);
    };
    let (len, n) = match b {
        0x23..=0x27 => (1, u16::from(b - 0x23)),
        0x28 => (2, u16::from(*bytes.get(*p + 1).ok_or("截断的 28 数值")?)),
        0x29..=0x2c => {
            let tail = bytes.get(*p + 1..*p + 3).ok_or("截断的长数值")?;
            if tail.iter().any(|b| *b > 0x7f) {
                return Err("长数值的后两字节必须是 7 位编码".into());
            }
            (
                3,
                (u16::from(b - 0x29) << 14) | (u16::from(tail[0]) << 7) | u16::from(tail[1]),
            )
        }
        _ => return Ok(None),
    };
    *p += len;
    Ok(Some(n))
}

fn args(bytes: &[u8], p: &mut usize, minimum: usize, maximum: usize) -> Result<Vec<Arg>> {
    let mut out = Vec::new();
    for _ in 0..maximum {
        match number(bytes, p)? {
            Some(n) => out.push(Arg::Number(n)),
            None => break,
        }
    }
    if out.len() < minimum {
        return Err(format!("缺少数值参数，至少需要 {minimum} 个"));
    }
    Ok(out)
}

fn quote(bytes: &[u8], p: &mut usize) -> Result<Arg> {
    if bytes.get(*p) != Some(&0x22) {
        return Err("缺少 22 引号参数".into());
    }
    *p += 1;
    let end = bytes[*p..]
        .iter()
        .position(|b| *b == 0x22)
        .ok_or("引号参数未结束")?
        + *p;
    let value = bytes[*p..end].to_vec();
    *p = end + 1;
    Ok(Arg::String(value))
}

pub fn lex(bytes: &[u8]) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut p = 0;
    while p < bytes.len() {
        let start = p;
        let b = bytes[p];
        p += 1;
        let kind = match b {
            0x00..=0x1f => {
                bytes.get(p).ok_or("截断的两字节标志")?;
                p += 1;
                Kind::Control
            }
            0x21 => {
                let end = bytes[p..]
                    .iter()
                    .position(|b| *b == 0)
                    .ok_or("21 字符串未结束")?
                    + p;
                let text = decode(&bytes[p..end])?;
                p = end + 1;
                Kind::Text(text)
            }
            0x2d..=0x7f => Kind::Text(decode(&[0x82, b + 0x72])?),
            0xa0..=0xa4 => Kind::Boundary(b),
            0xa5 => Kind::Newline,
            b if is_lead(b) => {
                let next = *bytes.get(p).ok_or("截断的双字节字符")?;
                p += 1;
                let code = u16::from_be_bytes([b, next]);
                if SPECIAL[..4].contains(&code) {
                    Kind::Control
                } else if code == 0x8196 && bytes.get(p..p + 4) == Some(&[0x81, 0x96, 0x81, 0x96]) {
                    p += 4;
                    Kind::Control
                } else if code == 0x8640 {
                    Kind::Pc98Glyph
                } else {
                    Kind::Text(decode(&[b, next])?)
                }
            }
            0xff => {
                if !matches!(bytes.get(p), Some(0xff | 0xfe)) {
                    return Err(format!("0x{start:X}: 非法 FF 结束标记"));
                }
                p += 1;
                Kind::Boundary(0xff)
            }
            0xa6..=0xd8 => {
                let mut a = match b {
                    0xa6 => args(bytes, &mut p, 4, 4)?,
                    0xa7 | 0xa9 | 0xc3 | 0xc4 | 0xc5 | 0xce | 0xd3 => args(bytes, &mut p, 2, 2)?,
                    0xa8 => args(bytes, &mut p, 1, 3)?,
                    0xaa | 0xbe | 0xc7 | 0xca | 0xcb => args(bytes, &mut p, 0, 1)?,
                    0xab | 0xac | 0xb8 | 0xbf | 0xcc => args(bytes, &mut p, 0, 2)?,
                    0xad | 0xae => args(bytes, &mut p, 0, 32)?,
                    0xb0 | 0xb1 | 0xc0 | 0xc8 | 0xc9 => {
                        let mut a = vec![quote(bytes, &mut p)?];
                        let n = match b {
                            0xb0 => 2,
                            0xc8 => 1,
                            _ => 0,
                        };
                        a.extend(args(bytes, &mut p, 0, n)?);
                        a
                    }
                    0xb4 => {
                        p += 1;
                        bytes.get(p - 1).ok_or("缺少 B4 字母变量")?;
                        Vec::new()
                    }
                    0xb5 | 0xb9 | 0xba | 0xbb | 0xd5 => args(bytes, &mut p, 1, 1)?,
                    0xb6 => {
                        if bytes.get(p) == Some(&0x22) {
                            vec![quote(bytes, &mut p)?]
                        } else {
                            bytes.get(p).ok_or("缺少 B6 变量")?;
                            p += 1;
                            args(bytes, &mut p, 0, 1)?
                        }
                    }
                    0xb7 => args(bytes, &mut p, 1, 3)?,
                    0xc1 | 0xc2 => {
                        let mut a = args(bytes, &mut p, 1, 1)?;
                        a.push(quote(bytes, &mut p)?);
                        a
                    }
                    0xc6 => args(bytes, &mut p, 5, 5)?,
                    0xcd => {
                        let mut a = args(bytes, &mut p, 1, 1)?;
                        loop {
                            if bytes.get(p) == Some(&0x22) {
                                a.push(quote(bytes, &mut p)?);
                            } else if let Some(n) = number(bytes, &mut p)? {
                                a.push(Arg::Number(n));
                            } else {
                                break;
                            }
                        }
                        a
                    }
                    0xcf => {
                        let mut a = args(bytes, &mut p, 1, 2)?;
                        let tail = args(bytes, &mut p, 0, 2)?;
                        if tail.len() == 1 {
                            return Err(format!("0x{start:X}: CF 坐标缺少第二值"));
                        }
                        a.extend(tail);
                        a
                    }
                    0xd0 => {
                        if bytes.get(p) == Some(&0x22) {
                            vec![quote(bytes, &mut p)?]
                        } else if bytes
                            .get(p..p + 2)
                            .is_some_and(|v| v.eq_ignore_ascii_case(b"SE"))
                        {
                            p += 2;
                            while bytes.get(p) == Some(&b' ') {
                                p += 1;
                            }
                            if bytes.get(p) == Some(&0x22) {
                                vec![quote(bytes, &mut p)?]
                            } else {
                                args(bytes, &mut p, 1, 1)?
                            }
                        } else {
                            let mut a = args(bytes, &mut p, 1, 1)?;
                            if matches!(a[0], Arg::Number(1..=3)) {
                                a.extend(args(bytes, &mut p, 0, 1)?);
                            }
                            a
                        }
                    }
                    0xd1 => {
                        let mut a = args(bytes, &mut p, 1, 1)?;
                        let count = if matches!(a[0], Arg::Number(2)) { 2 } else { 1 };
                        a.extend(args(bytes, &mut p, 0, count)?);
                        a
                    }
                    _ => Vec::new(),
                };
                if b == 0xd5 && !matches!(a.first(), Some(Arg::Number(0))) {
                    return Err("加密 MES 必须先完成本作 D5 解密，拒绝扫描密文".into());
                }
                a.shrink_to_fit();
                Kind::Op(b, a)
            }
            _ => return Err(format!("0x{start:X}: 不属于本作已确认语法的字节 {b:02X}")),
        };
        tokens.push(Token {
            span: Span { start, end: p },
            kind,
        });
    }
    Ok(tokens)
}

fn first_number(a: &[Arg]) -> Option<u16> {
    match a.first() {
        Some(Arg::Number(n)) => Some(*n),
        _ => None,
    }
}

pub fn definitions(bytes: &[u8], tokens: &[Token]) -> Result<Vec<(usize, usize, u16, Macro)>> {
    let mut found = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if let Kind::Op(0xb9, a) = &token.kind {
            let slot = first_number(a).ok_or("B9 缺少槽号")?;
            if slot >= 20 {
                return Err(format!("B9 槽 {slot} 超出本作 20 槽范围"));
            }
            if !matches!(
                tokens.get(i + 1).map(|t| &t.kind),
                Some(Kind::Boundary(0xa2))
            ) {
                return Err("B9 缺少 A2".into());
            }
            let mut depth = 1;
            let mut end = None;
            for (j, t) in tokens.iter().enumerate().skip(i + 2) {
                match t.kind {
                    Kind::Boundary(0xa2) => depth += 1,
                    Kind::Boundary(0xa3) => {
                        depth -= 1;
                        if depth == 0 {
                            end = Some(j);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let j = end.ok_or("B9 代码块未结束")?;
            let body_start = tokens[i + 1].span.end;
            let body_end = tokens[j].span.start;
            if body_end - body_start + 1 > 76 {
                return Err(format!("B9 槽 {slot} 超过 76 字节"));
            }
            let body = bytes[body_start..body_end].to_vec();
            let inside = &tokens[i + 2..j];
            let text = inside
                .iter()
                .filter_map(|t| match &t.kind {
                    Kind::Text(s) => Some(s.as_str()),
                    _ => None,
                })
                .collect::<String>();
            let name = if text.starts_with('［')
                && text.ends_with('］')
                && inside
                    .iter()
                    .all(|t| matches!(t.kind, Kind::Text(_) | Kind::Newline | Kind::Op(0xa8, _)))
            {
                let first = inside
                    .iter()
                    .find(|t| matches!(t.kind, Kind::Text(_)))
                    .ok_or("空姓名片段")?;
                Some((
                    text[3..text.len() - 3].to_owned(),
                    bytes[body_start..first.span.start].to_vec(),
                ))
            } else {
                None
            };
            let plain = if inside
                .iter()
                .all(|t| matches!(t.kind, Kind::Text(_) | Kind::Newline))
            {
                Some(
                    inside
                        .iter()
                        .map(|t| match &t.kind {
                            Kind::Text(s) => s.clone(),
                            _ => "\n".into(),
                        })
                        .collect(),
                )
            } else {
                None
            };
            found.push((i, j, slot, Macro { body, name, plain }));
        }
    }
    Ok(found)
}

pub fn startup_macros(bytes: &[u8]) -> Result<Macros> {
    let tokens = lex(bytes)?;
    Ok(definitions(bytes, &tokens)?
        .into_iter()
        .map(|(_, _, slot, m)| (slot, m))
        .collect())
}

fn literal_name(tokens: &[Token], start: usize) -> Option<(usize, NameSource)> {
    let mut text = String::new();
    for (i, token) in tokens.iter().enumerate().skip(start) {
        let Kind::Text(part) = &token.kind else {
            return None;
        };
        text.push_str(part);
        if !text.starts_with('［') {
            return None;
        }
        if text.ends_with('］') {
            let newline = tokens.get(i + 1)?;
            if !matches!(newline.kind, Kind::Newline) || text.len() <= 6 {
                return None;
            }
            return Some((
                i + 2,
                NameSource {
                    call: Span {
                        start: tokens[start].span.start,
                        end: newline.span.end,
                    },
                    prefix: Vec::new(),
                    name: text[3..text.len() - 3].into(),
                },
            ));
        }
    }
    None
}

pub fn parse(file: &str, bytes: &[u8], common: &Macros) -> Result<Primary> {
    if !bytes.ends_with(&[0xff, 0xff]) && !bytes.ends_with(&[0xff, 0xfe]) {
        return Err(format!("{file}: 缺少完整的脚本结束标记"));
    }
    let tokens = lex(bytes).map_err(|e| format!("{file}: {e}"))?;
    let defs = definitions(bytes, &tokens)?;
    let by_start: BTreeMap<_, _> = defs.iter().map(|d| (d.0, d)).collect();
    let mut macros = common.clone();
    let mut records = Vec::new();
    let mut pending: Option<NameSource> = None;
    let mut i = 0;
    while i < tokens.len() {
        if let Some((_, end, slot, m)) = by_start.get(&i) {
            macros.insert(*slot, m.clone());
            if m.name.is_some() {
                i = end + 1;
                pending = None;
                continue;
            }
        }
        let t = &tokens[i];
        if let Some((next, name)) = literal_name(&tokens, i) {
            pending = Some(name);
            i = next;
            continue;
        }
        if let Kind::Op(0xba, a) = &t.kind {
            if let Some((name, prefix)) = first_number(a)
                .and_then(|n| macros.get(&n))
                .and_then(|m| m.name.clone())
            {
                pending = Some(NameSource {
                    call: t.span.clone(),
                    prefix,
                    name,
                });
                i += 1;
                continue;
            }
            if first_number(a)
                .and_then(|n| macros.get(&n))
                .is_none_or(|m| m.plain.is_none())
            {
                pending = None;
            }
        }
        let piece = |t: &Token, macros: &Macros| -> Option<String> {
            match &t.kind {
                Kind::Text(s) => Some(s.clone()),
                Kind::Newline => Some("\n".into()),
                Kind::Op(0xba, a) => first_number(a)
                    .and_then(|n| macros.get(&n))
                    .and_then(|m| m.plain.clone()),
                _ => None,
            }
        };
        if piece(t, &macros).is_some_and(|s| !s.trim().is_empty()) {
            let start = t.span.start;
            let mut end = start;
            let mut message = String::new();
            while let Some(t) = tokens.get(i) {
                let Some(s) = piece(t, &macros) else { break };
                message.push_str(&s);
                end = t.span.end;
                i += 1;
            }
            // Trailing newlines belong to the following layout, not to the sentence.
            while message.ends_with('\n') {
                let last = &tokens[i - 1];
                let s = piece(last, &macros).unwrap_or_default();
                if s.chars().all(|c| c == '\n') {
                    message.truncate(message.len() - s.len());
                    end = last.span.start;
                    i -= 1;
                } else {
                    break;
                }
            }
            if !message.is_empty() {
                let name = pending.take();
                records.push(Record {
                    span: Span { start, end },
                    entry: Entry {
                        name: name.as_ref().map(|n| n.name.clone()),
                        message,
                    },
                    name_source: name,
                });
            }
            continue;
        }
        if matches!(
            t.kind,
            Kind::Boundary(_) | Kind::Op(0xa6 | 0xa7 | 0xaa | 0xb1 | 0xc0 | 0xd4, _)
        ) {
            pending = None;
        }
        i += 1;
    }
    Ok(Primary {
        file: file.into(),
        original: bytes.to_vec(),
        records,
        pc98_glyphs: tokens
            .iter()
            .filter(|t| matches!(t.kind, Kind::Pc98Glyph))
            .map(|t| t.span.clone())
            .collect(),
    })
}

pub fn normalize(text: &str, name: bool) -> Result<String> {
    if name && (text.contains(['\n', '\r']) || text.is_empty()) {
        return Err("姓名不能为空或带换行".into());
    }
    let mut out = String::new();
    for c in text.chars() {
        if c == '\n' && !name {
            out.push(c);
        } else if c.is_control() {
            return Err(format!("不支持的控制字符 U+{:04X}", c as u32));
        } else if c == ' ' {
            out.push('　');
        } else if ('!'..='~').contains(&c) {
            out.push(char::from_u32(c as u32 + 0xfee0).ok_or("全角转换失败")?);
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

pub fn text_changes(primary: &Primary, supplied: &[Entry]) -> Result<Vec<String>> {
    vn_text::validate_shape(
        &primary
            .records
            .iter()
            .map(|r| r.entry.clone())
            .collect::<Vec<_>>(),
        supplied,
    )
    .map_err(|e| e.to_string())?;
    let mut texts = Vec::new();
    for (r, e) in primary.records.iter().zip(supplied) {
        if r.entry.message != e.message {
            texts.push(normalize(&e.message, false)?.replace('\n', ""));
        }
        if r.entry.name != e.name {
            texts.push(format!(
                "［{}］",
                normalize(e.name.as_deref().ok_or("姓名字段缺失")?, true)?
            ));
        }
    }
    Ok(texts)
}

pub fn reserved_codes(bytes: &[u8]) -> BTreeSet<u16> {
    let mut set = BTreeSet::new();
    for pair in bytes.windows(2) {
        if is_lead(pair[0])
            && let Ok(text) = decode(pair)
            && text.chars().count() == 1
            && text.chars().all(vn_font::font_98::has_loaded_np2_slot)
            && text.chars().next().is_some_and(|c| {
                vn_font::font_98::cp932_for_carrier(c).is_ok_and(|canonical| canonical == pair)
            })
        {
            set.insert(u16::from_be_bytes([pair[0], pair[1]]));
        }
    }
    set
}

pub fn rebuild(
    primary: &Primary,
    supplied: &[Entry],
    encode: &impl Fn(&str) -> Result<Vec<u8>>,
) -> Result<Vec<u8>> {
    text_changes(primary, supplied)?;
    let mut patches: Vec<(Span, Vec<u8>)> = Vec::new();
    for (index, (r, e)) in primary.records.iter().zip(supplied).enumerate() {
        let result = (|| {
            if e.message != r.entry.message {
                let text = normalize(&e.message, false)?;
                let mut out = Vec::new();
                for (i, line) in text.split('\n').enumerate() {
                    if i != 0 {
                        out.push(0xa5);
                    }
                    out.extend(encode(line)?);
                }
                patches.push((r.span.clone(), out));
            }
            if e.name != r.entry.name {
                let n = r.name_source.as_ref().ok_or("本条没有姓名来源")?;
                let name = normalize(e.name.as_deref().ok_or("姓名字段缺失")?, true)?;
                let mut out = n.prefix.clone();
                out.extend(encode(&format!("［{name}］"))?);
                out.push(0xa5);
                patches.push((n.call.clone(), out));
            }
            Ok::<(), String>(())
        })();
        result.map_err(|e| format!("{} 第 {} 条: {e}", primary.file, index + 1))?;
    }
    patches.sort_by_key(|(s, _)| s.start);
    let mut output = Vec::new();
    let mut cursor = 0;
    for (span, replacement) in patches {
        if span.start < cursor || span.end > primary.original.len() {
            return Err("回注定位重叠或越界".into());
        }
        output.extend_from_slice(&primary.original[cursor..span.start]);
        output.extend(replacement);
        cursor = span.end;
    }
    output.extend_from_slice(&primary.original[cursor..]);
    let tokens = lex(&output)?;
    definitions(&output, &tokens)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operands_are_owned_and_numeric_pages_are_decoded() {
        let data = [
            0xc9, 0x22, b'A', b'5', 0x22, 0xcd, 0x2a, 0x20, 0, 0x22, b'X', 0x22, 0xff, 0xff,
        ];
        let tokens = lex(&data).unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(
            matches!(&tokens[1].kind, Kind::Op(0xcd, a) if matches!(a[0], Arg::Number(0x5000)))
        );
        assert!(lex(&[0xc9, 0x22, b'A']).is_err());
    }

    #[test]
    fn name_is_separate_and_changed_name_gets_one_a5() {
        let name_bytes = SHIFT_JIS.encode("［翔］").0.into_owned();
        let mut definition = vec![0xb9, 0x23, 0xa2, 0xa8, 0x23];
        definition.extend(&name_bytes);
        definition.extend([0xa5, 0xa3, 0xff, 0xff]);
        let common = startup_macros(&definition).unwrap();
        let source = [0xba, 0x23, 0x45, 0x4f, 0xa5, 0x7f, 0xff, 0xff];
        let parsed = parse("sample", &source, &common).unwrap();
        assert_eq!(parsed.records[0].entry.name.as_deref(), Some("翔"));
        assert_eq!(parsed.records[0].entry.message, "すっ\nん");
        let supplied = vec![Entry {
            name: Some("甲".into()),
            message: "日本\n世界".into(),
        }];
        let out = rebuild(&parsed, &supplied, &|s| {
            let (b, _, fail) = SHIFT_JIS.encode(s);
            if fail {
                Err("encoding".into())
            } else {
                Ok(b.into_owned())
            }
        })
        .unwrap();
        assert_eq!(out.iter().filter(|b| **b == 0xa5).count(), 2);
        assert!(normalize("甲\n", true).is_err());
    }

    #[test]
    fn literal_name_header_is_bound_to_the_following_message() {
        let mut source = SHIFT_JIS.encode("［翔］").0.into_owned();
        source.extend([0xa5, 0x45, 0x4f, 0xff, 0xff]);
        let parsed = parse("literal", &source, &Macros::new()).unwrap();
        assert_eq!(parsed.records[0].entry.name.as_deref(), Some("翔"));
        assert_eq!(parsed.records[0].entry.message, "すっ");
    }

    #[test]
    fn font_reservations_ignore_ibm_alias_bytes() {
        assert!(!reserved_codes(&[0xfa, 0x4c]).contains(&0xfa4c));
        assert!(reserved_codes(&[0x82, 0xa0]).contains(&0x82a0));
        assert!(matches!(
            lex(&[0x86, 0x40]).unwrap()[0].kind,
            Kind::Pc98Glyph
        ));
        assert!(parse("truncated", &[0x45, 0x4f], &Macros::new()).is_err());
    }
}
