use crate::model::GameId;
use anyhow::{bail, Context, Result};
use drrnger_d88_tool::font::{cp932_for_carrier, EncodingPlan};
use encoding_rs::SHIFT_JIS;

#[derive(Debug, Clone)]
pub struct TextSpan {
    pub start: usize,
    pub end: usize,
    pub name: Option<String>,
    pub text: String,
    pub command_text: bool,
}

#[derive(Debug, Clone)]
pub struct ExeTextSlot {
    pub offset: usize,
    pub capacity: usize,
    pub spans: Vec<TextSpan>,
}

pub fn scan_exe_text_slots(data: &[u8]) -> Vec<ExeTextSlot> {
    let mut result = Vec::new();
    let mut start = 0usize;
    while start < data.len() {
        while start < data.len() && data[start] == 0 {
            start += 1;
        }
        if start >= data.len() {
            break;
        }
        let end = data[start..]
            .iter()
            .position(|&byte| byte == 0)
            .map(|length| start + length)
            .unwrap_or(data.len());
        let candidate = &data[start..end];
        if candidate.len() >= 4 && candidate.len() <= 4096 && valid_exe_text(candidate) {
            let spans = find_text_spans(candidate)
                .into_iter()
                .filter(|span| contains_japanese_text(&span.text))
                .filter_map(|span| decorate_exe_span(candidate, span))
                .collect::<Vec<_>>();
            if !spans.is_empty() {
                result.push(ExeTextSlot {
                    offset: start,
                    capacity: candidate.len(),
                    spans,
                });
            }
        }
        start = end.saturating_add(1);
    }
    let data_start = result
        .iter()
        .filter(|slot| {
            slot.spans
                .iter()
                .any(|span| span.text.chars().filter(|&ch| is_kana(ch)).count() >= 2)
        })
        .map(|slot| slot.offset)
        .min();
    if let Some(data_start) = data_start {
        result.retain(|slot| slot.offset >= data_start);
    } else {
        result.clear();
    }
    result
}

fn decorate_exe_span(data: &[u8], span: TextSpan) -> Option<TextSpan> {
    let command_text = span.command_text;
    let start = if span.start > 0 && matches!(data[span.start - 1], b'"' | b'\'') {
        span.start - 1
    } else {
        span.start
    };
    make_span(data, start, span.end).ok().map(|mut result| {
        result.command_text = command_text;
        result
    })
}

fn valid_exe_text(data: &[u8]) -> bool {
    let mut pos = 0usize;
    let mut glyphs = 0usize;
    while pos < data.len() {
        let byte = data[pos];
        if byte.is_ascii_graphic() || matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | 0x1b) {
            pos += 1;
        } else if is_sjis_lead(byte) && pos + 1 < data.len() && is_sjis_trail(data[pos + 1]) {
            glyphs += 1;
            pos += 2;
        } else if (0xa0..=0xdf).contains(&byte) {
            glyphs += 1;
            pos += 1;
        } else {
            return false;
        }
    }
    glyphs >= 2
}

fn contains_japanese_text(text: &str) -> bool {
    if text.contains("<GLYPH:")
        || text.contains("<BYTE:")
        || text.chars().any(|ch| matches!(ch as u32, 0xff61..=0xff9f))
    {
        return false;
    }
    let core = text.chars().filter(|&ch| is_japanese_core(ch)).count();
    if core >= 3 {
        return true;
    }
    core == 2
        && text
            .chars()
            .all(|ch| is_japanese_core(ch) || "　、。！？…‥・「」『』（）【】ー～〜".contains(ch))
}

fn is_japanese_core(character: char) -> bool {
    matches!(
        character as u32,
        0x3040..=0x30ff | 0x3400..=0x4dbf | 0x4e00..=0x9fff
    )
}

fn is_kana(character: char) -> bool {
    matches!(character as u32, 0x3040..=0x30ff)
}

pub fn load_nooch1_map(data: &[u8]) -> Result<Vec<Vec<u8>>> {
    let count = read_u16(data, 0)? as usize;
    let header_end = data
        .get(2..)
        .and_then(|tail| tail.iter().position(|&b| b == b'\n'))
        .map(|at| at + 3)
        .context("TEXT.MAP 缺少头部换行")?;
    let mut entries = data[header_end..]
        .split(|&b| b == b'\n')
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line).to_vec())
        .collect::<Vec<_>>();
    if entries.last().is_some_and(Vec::is_empty) {
        entries.pop();
    }
    if entries.len() + 1 == count {
        entries.insert(0, b"<PLAYER_NAME>".to_vec());
    }
    if entries.len() != count {
        bail!("TEXT.MAP 声明 {count} 项，实际解析 {} 项", entries.len());
    }
    Ok(entries)
}

pub fn load_nooch2_map(data: &[u8]) -> Result<Vec<Vec<u8>>> {
    let marker = b"#MAP={\r\n";
    let start = find_bytes(data, marker, 0).context("SYSTEM.MAC 缺少 #MAP 块")? + marker.len();
    let end = find_bytes(data, b"};", start).context("SYSTEM.MAC 的 #MAP 块未闭合")?;
    Ok(data[start..end]
        .split(|&b| b == b'\n')
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

pub fn load_nooch3_map(data: &[u8]) -> Result<Vec<Vec<u8>>> {
    let count = read_u16(data, 0)? as usize;
    let mut pos = 4usize;
    let mut entries = Vec::with_capacity(count);
    for index in 0..count {
        let size = *data
            .get(pos)
            .with_context(|| format!("TEXTMAP.DAT 第 {index} 项缺少长度"))?
            as usize;
        pos += 1;
        let raw = data
            .get(pos..pos + size)
            .with_context(|| format!("TEXTMAP.DAT 第 {index} 项越界"))?;
        pos += size;
        entries.push(raw.strip_suffix(&[0]).unwrap_or(raw).to_vec());
    }
    Ok(entries)
}

pub fn expand12(game: GameId, raw: &[u8], map: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < raw.len() && raw[pos] != 0 {
        let b = raw[pos];
        if b == 0x01 && pos + 1 < raw.len() {
            let operand = raw[pos + 1];
            let index = match game {
                GameId::Nooch => operand.wrapping_sub(1) as usize,
                GameId::Nooch2 => (0xff - operand) as usize,
                GameId::Nooch3 => unreachable!(),
            };
            if let Some(value) = map.get(index) {
                out.extend_from_slice(value);
            } else {
                out.extend_from_slice(format!("<MAP:{index}>").as_bytes());
            }
            pos += 2;
        } else if b == 0x04 && pos + 1 < raw.len() {
            out.extend_from_slice(format!("<RUNTIME:{:02X}>", raw[pos + 1]).as_bytes());
            pos += 2;
        } else if is_sjis_lead(b) && pos + 1 < raw.len() {
            if game == GameId::Nooch2 && raw[pos..pos + 2] == [0x81, 0x96] {
                out.extend_from_slice(b"<PLAYER_NAME>");
            } else {
                out.extend_from_slice(&raw[pos..pos + 2]);
            }
            pos += 2;
        } else if let Some(pair) = kana_pair(b) {
            out.extend_from_slice(&pair);
            pos += 1;
        } else if b == 0x80 && pos + 2 < raw.len() {
            let selectors = u16::from_le_bytes([raw[pos + 1], raw[pos + 2]]);
            pos += 3;
            for shift in [14u32, 12, 10, 8, 6, 4, 2, 0] {
                let selector = ((selectors >> shift) & 3) as u8;
                if selector == 0 || pos >= raw.len() {
                    break;
                }
                out.extend_from_slice(&[0x80 + selector, raw[pos]]);
                pos += 1;
            }
        } else {
            out.push(b);
            pos += 1;
        }
    }
    out
}

pub fn expand3(raw: &[u8], map: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < raw.len() && raw[pos] != 0 {
        let b = raw[pos];
        if b == 0x7f && pos + 1 < raw.len() {
            let count = raw[pos + 1] as usize;
            let trails = raw.get(pos + 2..pos + 2 + count).unwrap_or(&[]);
            for &trail in trails {
                out.extend_from_slice(&[0x83, trail]);
            }
            pos += 2 + trails.len();
        } else if let Some(pair) = kana_pair(b) {
            out.extend_from_slice(&pair);
            pos += 1;
        } else if b == 0x7e && pos + 1 < raw.len() {
            let index = (0xff - raw[pos + 1]) as usize;
            if let Some(value) = map.get(index) {
                out.extend_from_slice(value);
            } else {
                out.extend_from_slice(&raw[pos..pos + 2]);
            }
            pos += 2;
        } else if b == 0x7c {
            out.extend_from_slice(b"<PLAYER_NAME>");
            pos += 1;
        } else if b == 0x7d {
            out.extend_from_slice(&[0x81, 0x79]);
            out.extend_from_slice(b"<PLAYER_NAME>");
            out.extend_from_slice(&[0x81, 0x7a]);
            pos += 1;
        } else if is_sjis_lead(b) && pos + 1 < raw.len() {
            out.extend_from_slice(&raw[pos..pos + 2]);
            pos += 2;
        } else {
            out.push(b);
            pos += 1;
        }
    }
    out
}

pub fn find_text_spans(expanded: &[u8]) -> Vec<TextSpan> {
    let mut ranges = Vec::new();
    let mut chunk_start = None;
    let mut has_visible = false;
    let mut pos = 0usize;
    let flush = |end: usize,
                 start: &mut Option<usize>,
                 visible: &mut bool,
                 ranges: &mut Vec<(usize, usize, bool)>| {
        if let Some(mut begin) = start.take() {
            let mut finish = end;
            while begin < finish && expanded[begin].is_ascii_whitespace() {
                begin += 1;
            }
            while finish > begin && expanded[finish - 1].is_ascii_whitespace() {
                finish -= 1;
            }
            if *visible && finish > begin {
                ranges.push((begin, finish, false));
            }
        }
        *visible = false;
    };
    while pos < expanded.len() {
        if expanded[pos] == b'(' && expanded.get(pos + 1).is_some_and(u8::is_ascii_alphabetic) {
            if let Some(close) = expanded[pos + 2..].iter().position(|&b| b == b')') {
                flush(pos, &mut chunk_start, &mut has_visible, &mut ranges);
                let command_end = pos + close + 2;
                if has_command_text_opcode(expanded, pos + 1, command_end) {
                    find_command_text_ranges(expanded, pos + 1, command_end, &mut ranges);
                }
                pos = command_end + 1;
                continue;
            }
        }
        if expanded[pos] < 0x20 || expanded[pos] == 0x7f || expanded[pos] == 0xff {
            flush(pos, &mut chunk_start, &mut has_visible, &mut ranges);
            pos += 1;
            continue;
        }
        chunk_start.get_or_insert(pos);
        if expanded[pos..].starts_with(b"<PLAYER_NAME>")
            || expanded[pos..].starts_with(b"<RUNTIME:")
        {
            has_visible = true;
            if let Some(close) = expanded[pos..].iter().position(|&b| b == b'>') {
                pos += close + 1;
                continue;
            }
        }
        if is_sjis_lead(expanded[pos])
            && pos + 1 < expanded.len()
            && is_sjis_trail(expanded[pos + 1])
        {
            has_visible = true;
            pos += 2;
        } else if expanded[pos] >= 0xa0 {
            has_visible = true;
            pos += 1;
        } else if expanded[pos].is_ascii_graphic() || expanded[pos] == b' ' {
            pos += 1;
        } else {
            flush(pos, &mut chunk_start, &mut has_visible, &mut ranges);
            pos += 1;
        }
    }
    flush(
        expanded.len(),
        &mut chunk_start,
        &mut has_visible,
        &mut ranges,
    );
    ranges
        .into_iter()
        .filter_map(|(start, end, command_text)| {
            visible_bounds(expanded, start, end).map(|(start, end)| (start, end, command_text))
        })
        .filter_map(|(start, end, command_text)| {
            make_span(expanded, start, end).ok().map(|mut span| {
                span.command_text = command_text;
                span
            })
        })
        .collect()
}

fn has_command_text_opcode(expanded: &[u8], start: usize, end: usize) -> bool {
    let mut pos = start;
    while pos < end {
        if expanded[pos] == b'<' {
            if let Some(close) = expanded[pos..end].iter().position(|&byte| byte == b'>') {
                pos += close + 1;
                continue;
            }
        }
        if is_sjis_lead(expanded[pos]) && pos + 1 < end && is_sjis_trail(expanded[pos + 1]) {
            pos += 2;
            continue;
        }
        if matches!(expanded[pos], b'T' | b't') {
            return true;
        }
        pos += 1;
    }
    false
}

fn find_command_text_ranges(
    expanded: &[u8],
    start: usize,
    end: usize,
    ranges: &mut Vec<(usize, usize, bool)>,
) {
    let mut pos = start;
    let mut chunk_start = None;
    let flush =
        |finish: usize, chunk_start: &mut Option<usize>, ranges: &mut Vec<(usize, usize, bool)>| {
            if let Some(begin) = chunk_start.take() {
                if finish > begin {
                    ranges.push((begin, finish, true));
                }
            }
        };

    while pos < end {
        if expanded[pos..end].starts_with(b"<PLAYER_NAME>") {
            chunk_start.get_or_insert(pos);
            pos += b"<PLAYER_NAME>".len();
        } else if is_sjis_lead(expanded[pos]) && pos + 1 < end && is_sjis_trail(expanded[pos + 1]) {
            chunk_start.get_or_insert(pos);
            pos += 2;
        } else if expanded[pos] >= 0xa0 {
            chunk_start.get_or_insert(pos);
            pos += 1;
        } else {
            flush(pos, &mut chunk_start, ranges);
            pos += 1;
        }
    }
    flush(end, &mut chunk_start, ranges);
}

pub fn find_nooch3_messages(expanded: &[u8]) -> Vec<TextSpan> {
    let mut result = Vec::new();
    let mut pos = 0usize;
    while let Some(relative) = find_bytes(expanded, b"\xff(", pos) {
        let start = relative + 2;
        let Some(end) = expanded[start..]
            .iter()
            .position(|&b| b == b')')
            .map(|n| start + n)
        else {
            break;
        };
        if let Ok(span) = make_span(expanded, start, end) {
            result.push(span);
        }
        pos = end + 1;
    }
    result
}

fn make_span(expanded: &[u8], start: usize, end: usize) -> Result<TextSpan> {
    let mut body_start = start;
    let mut name = None;
    if expanded.get(start..start + 2) == Some(&[0x81, 0x79]) {
        if let Some(close) = find_bytes(expanded, &[0x81, 0x7a], start + 2).filter(|&at| at < end) {
            name = Some(decode_display(&expanded[start + 2..close])?);
            body_start = close + 2;
        }
    }
    Ok(TextSpan {
        start,
        end,
        name,
        text: decode_display(&expanded[body_start..end])?,
        command_text: false,
    })
}

fn visible_bounds(data: &[u8], start: usize, end: usize) -> Option<(usize, usize)> {
    let mut first = None;
    let mut last = None;
    let mut pos = start;
    while pos < end {
        let b = data[pos];
        if is_sjis_lead(b) && pos + 1 < end && is_sjis_trail(data[pos + 1]) {
            first.get_or_insert(pos);
            last = Some(pos + 2);
            pos += 2;
        } else if b >= 0xa0 {
            first.get_or_insert(pos);
            last = Some(pos + 1);
            pos += 1;
        } else {
            pos += 1;
        }
    }
    Some((first?, last?))
}

pub fn decode_display(data: &[u8]) -> Result<String> {
    let mut out = String::new();
    let mut pos = 0usize;
    while pos < data.len() {
        if data[pos] == b'<' {
            if let Some(close) = data[pos..].iter().position(|&b| b == b'>') {
                if data[pos..pos + close + 1].is_ascii() {
                    out.push_str(std::str::from_utf8(&data[pos..pos + close + 1])?);
                    pos += close + 1;
                    continue;
                }
            }
        }
        let b = data[pos];
        if is_sjis_lead(b) && pos + 1 < data.len() && is_sjis_trail(data[pos + 1]) {
            let pair = [b, data[pos + 1]];
            if let Some(character) = decode_engine_pair(pair) {
                out.push(character);
            } else if b == 0x85 || b == 0x86 {
                out.push_str(&format!("<GLYPH:{b:02X}{:02X}>", pair[1]));
            } else {
                let (decoded, _, errors) = SHIFT_JIS.decode(&pair);
                if errors {
                    out.push_str(&format!("<GLYPH:{b:02X}{:02X}>", pair[1]));
                } else {
                    out.push_str(&decoded);
                }
            }
            pos += 2;
        } else {
            let (decoded, _, errors) = SHIFT_JIS.decode(&data[pos..pos + 1]);
            if errors {
                out.push_str(&format!("<BYTE:{b:02X}>"));
            } else {
                out.push_str(&decoded);
            }
            pos += 1;
        }
    }
    Ok(out)
}

pub fn encode_translation(text: &str, plan: &EncodingPlan) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < text.len() {
        let tail = &text[pos..];
        if tail.starts_with("<PLAYER_NAME>") {
            out.extend_from_slice(b"<PLAYER_NAME>");
            pos += "<PLAYER_NAME>".len();
            continue;
        }
        if tail.starts_with("<RUNTIME:") {
            let end = tail.find('>').context("未闭合的 <RUNTIME:XX> 标记")?;
            out.extend_from_slice(tail[..=end].as_bytes());
            pos += end + 1;
            continue;
        }
        if tail.starts_with("<GLYPH:") {
            let end = tail.find('>').context("未闭合的 <GLYPH:XXXX> 标记")?;
            let value = &tail[7..end];
            if value.len() != 4 {
                bail!("字形标记必须是 <GLYPH:XXXX>: {}", &tail[..=end]);
            }
            let word = u16::from_str_radix(value, 16).context("字形标记不是十六进制")?;
            out.extend_from_slice(&word.to_be_bytes());
            pos += end + 1;
            continue;
        }
        if tail.starts_with("<BYTE:") {
            let end = tail.find('>').context("未闭合的 <BYTE:XX> 标记")?;
            let value = &tail[6..end];
            let byte = u8::from_str_radix(value, 16).context("字节标记不是十六进制")?;
            out.push(byte);
            pos += end + 1;
            continue;
        }
        let character = tail.chars().next().expect("non-empty tail");
        let carrier = plan.carrier_for(character).map_err(anyhow::Error::msg)?;
        out.extend_from_slice(&cp932_for_carrier(carrier).map_err(anyhow::Error::msg)?);
        pos += character.len_utf8();
    }
    Ok(out)
}

pub fn serialize_expanded(game: GameId, data: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < data.len() {
        if game == GameId::Nooch3 {
            let full = [0x81, 0x79];
            if data[pos..].starts_with(&full)
                && data[pos + 2..].starts_with(b"<PLAYER_NAME>")
                && data.get(pos + 15..pos + 17) == Some(&[0x81, 0x7a])
            {
                out.push(0x7d);
                pos += 17;
                continue;
            }
        }
        if data[pos..].starts_with(b"<PLAYER_NAME>") {
            match game {
                GameId::Nooch => out.extend_from_slice(&[0x01, 0x01]),
                GameId::Nooch2 => out.extend_from_slice(&[0x81, 0x96]),
                GameId::Nooch3 => out.push(0x7c),
            }
            pos += 13;
            continue;
        }
        if data[pos..].starts_with(b"<RUNTIME:") {
            let close = data[pos..]
                .iter()
                .position(|&b| b == b'>')
                .context("未闭合的 RUNTIME 标记")?;
            let text = std::str::from_utf8(&data[pos + 9..pos + close])?;
            let operand = u8::from_str_radix(text, 16).context("RUNTIME 标记参数错误")?;
            if game == GameId::Nooch3 {
                bail!("三代脚本不支持 <RUNTIME:XX> 标记");
            }
            out.extend_from_slice(&[0x04, operand]);
            pos += close + 1;
            continue;
        }
        out.push(data[pos]);
        pos += 1;
    }
    Ok(out)
}

/// Serialize an expanded Nooch 2 line using only the generated full-character
/// dictionary and the engine's verified one-byte kana forms. Other Shift-JIS
/// characters are copied as complete pairs; new packed-SJIS groups are never
/// emitted.
pub fn serialize_nooch2(data: &[u8], map: &[Vec<u8>]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < data.len() {
        if data[pos..].starts_with(b"<PLAYER_NAME>") {
            out.extend_from_slice(&[0x81, 0x96]);
            pos += 13;
            continue;
        }
        if data[pos..].starts_with(b"<RUNTIME:") {
            let close = data[pos..]
                .iter()
                .position(|&byte| byte == b'>')
                .context("未闭合的 RUNTIME 标记")?;
            let text = std::str::from_utf8(&data[pos + 9..pos + close])?;
            let operand = u8::from_str_radix(text, 16).context("RUNTIME 标记参数错误")?;
            out.extend_from_slice(&[0x04, operand]);
            pos += close + 1;
            continue;
        }

        let dictionary = map
            .iter()
            .enumerate()
            .filter(|(_, value)| !value.is_empty() && data[pos..].starts_with(value.as_slice()))
            .max_by_key(|(_, value)| value.len());
        if let Some((index, value)) = dictionary {
            let index = u8::try_from(index).context("二代字典索引超出字节范围")?;
            let operand = 0xff - index;
            out.extend_from_slice(&[0x01, operand]);
            pos += value.len();
            continue;
        }

        if pos + 1 < data.len() {
            if let Some(code) = compact_kana_code([data[pos], data[pos + 1]]) {
                out.push(code);
                pos += 2;
                continue;
            }
        }

        // The translated stream must always advance on complete Shift-JIS
        // characters. Visiting a trail byte as a new position can accidentally
        // match a dictionary entry across glyph boundaries. Also avoid emitting
        // the engine's packed-SJIS form for new text; the generated Nooch 2
        // dictionary provides the required space savings without that ambiguity.
        if is_sjis_lead(data[pos]) && pos + 1 < data.len() && is_sjis_trail(data[pos + 1]) {
            out.extend_from_slice(&data[pos..pos + 2]);
            pos += 2;
            continue;
        }

        out.push(data[pos]);
        pos += 1;
    }
    Ok(out)
}

fn compact_kana_code(pair: [u8; 2]) -> Option<u8> {
    (0xa0u8..=0xdf).find(|&code| kana_pair(code) == Some(pair))
}

pub fn strip_tokens(text: &str) -> String {
    let mut out = String::new();
    let mut pos = 0usize;
    while pos < text.len() {
        let tail = &text[pos..];
        if (tail.starts_with("<PLAYER_NAME>")
            || tail.starts_with("<RUNTIME:")
            || tail.starts_with("<GLYPH:")
            || tail.starts_with("<BYTE:"))
            && tail.find('>').is_some()
        {
            pos += tail.find('>').unwrap() + 1;
        } else {
            let ch = tail.chars().next().unwrap();
            out.push(ch);
            pos += ch.len_utf8();
        }
    }
    out
}

pub fn lzss3_decode(data: &[u8]) -> Result<Vec<u8>> {
    if data.first() != Some(&0xff) {
        return Ok(data.to_vec());
    }
    let out_size = read_u16(data, 6)? as usize;
    let mut src = 10usize;
    let mut flags = 0u16;
    let mut ring = vec![0u8; 4096];
    let mut write_pos = 0xfeeusize;
    let mut out = Vec::with_capacity(out_size);
    while out.len() < out_size {
        flags >>= 1;
        if flags & 0x100 == 0 {
            flags = u16::from(*data.get(src).context("LZSS 标志越界")?) | 0xff00;
            src += 1;
        }
        if flags & 1 != 0 {
            let value = *data.get(src).context("LZSS 字面量越界")?;
            src += 1;
            out.push(value);
            ring[write_pos] = value;
            write_pos = (write_pos + 1) & 0xfff;
        } else {
            let lo = *data.get(src).context("LZSS 回引低字节越界")?;
            let hi = *data.get(src + 1).context("LZSS 回引高字节越界")?;
            src += 2;
            let read_pos = usize::from(lo) | (usize::from(hi & 0xf0) << 4);
            let length = usize::from(hi & 0x0f) + 3;
            for index in 0..length {
                let value = ring[(read_pos + index) & 0xfff];
                out.push(value);
                ring[write_pos] = value;
                write_pos = (write_pos + 1) & 0xfff;
                if out.len() == out_size {
                    break;
                }
            }
        }
    }
    Ok(out)
}

pub fn lzss3_compress(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() > u16::MAX as usize {
        bail!("三代单个场景解压后超过 65535 字节");
    }
    let mut out = vec![0xff, 0, 0, 0, 0, 0];
    out.extend_from_slice(&(data.len() as u16).to_le_bytes());
    out.extend_from_slice(&[0, 0]);

    let mut positions: [std::collections::VecDeque<usize>; 256] =
        std::array::from_fn(|_| std::collections::VecDeque::new());
    let mut pos = 0usize;
    while pos < data.len() {
        let flag_pos = out.len();
        out.push(0);
        let mut flags = 0u8;
        for bit in 0..8 {
            if pos >= data.len() {
                break;
            }
            let oldest = pos.saturating_sub(4096);
            let candidates = &mut positions[usize::from(data[pos])];
            while candidates
                .front()
                .is_some_and(|&candidate| candidate < oldest)
            {
                candidates.pop_front();
            }

            let mut best_at = 0usize;
            let mut best_len = 0usize;
            let max_len = 18.min(data.len() - pos);
            for &candidate in candidates.iter().rev().take(128) {
                let mut length = 1usize;
                while length < max_len && data[candidate + length] == data[pos + length] {
                    length += 1;
                }
                if length > best_len {
                    best_at = candidate;
                    best_len = length;
                    if length == max_len {
                        break;
                    }
                }
            }

            let consumed = if best_len >= 3 {
                let ring_pos = (0xfee + best_at) & 0xfff;
                out.push((ring_pos & 0xff) as u8);
                out.push((((ring_pos >> 4) & 0xf0) | (best_len - 3)) as u8);
                best_len
            } else {
                flags |= 1 << bit;
                out.push(data[pos]);
                1
            };
            for at in pos..pos + consumed {
                positions[usize::from(data[at])].push_back(at);
            }
            pos += consumed;
        }
        out[flag_pos] = flags;
    }
    Ok(out)
}

fn decode_engine_pair(pair: [u8; 2]) -> Option<char> {
    if pair[0] == 0x85 {
        let trails = (0x40..=0x7e).chain(0x80..=0x9e).collect::<Vec<_>>();
        if let Some(index) = trails.iter().position(|&trail| trail == pair[1]) {
            return char::from_u32('!' as u32 + index as u32);
        }
        return match pair[1] {
            0x9f => Some('｡'),
            0xa0 => Some('｢'),
            0xa1 => Some('｣'),
            0xa2 => Some('､'),
            0xa3 => Some('･'),
            _ => None,
        };
    }
    (pair == [0x86, 0x40]).then_some(' ')
}

fn kana_pair(b: u8) -> Option<[u8; 2]> {
    let trail = match b {
        0xa6 => 0xf0,
        0xa7 => 0x9f,
        0xa8 => 0xa1,
        0xa9 => 0xa3,
        0xaa => 0xa5,
        0xab => 0xa7,
        0xac => 0xe1,
        0xad => 0xe3,
        0xae => 0xe5,
        0xaf => 0xc1,
        0xb1 => 0xa0,
        0xb2 => 0xa2,
        0xb3 => 0xa4,
        0xb4 => 0xa6,
        0xb5 => 0xa8,
        0xb6 => 0xa9,
        0xb7 => 0xab,
        0xb8 => 0xad,
        0xb9 => 0xaf,
        0xba => 0xb1,
        0xbb => 0xb3,
        0xbc => 0xb5,
        0xbd => 0xb7,
        0xbe => 0xb9,
        0xbf => 0xbb,
        0xc0 => 0xbd,
        0xc1 => 0xbf,
        0xc2 => 0xc2,
        0xc3 => 0xc4,
        0xc4 => 0xc6,
        0xc5 => 0xc8,
        0xc6 => 0xc9,
        0xc7 => 0xca,
        0xc8 => 0xcb,
        0xc9 => 0xcc,
        0xca => 0xcd,
        0xcb => 0xd0,
        0xcc => 0xd3,
        0xcd => 0xd6,
        0xce => 0xd9,
        0xcf => 0xdc,
        0xd0 => 0xdd,
        0xd1 => 0xde,
        0xd2 => 0xdf,
        0xd3 => 0xe0,
        0xd4 => 0xe2,
        0xd5 => 0xe4,
        0xd6 => 0xe6,
        0xd7 => 0xe7,
        0xd8 => 0xe8,
        0xd9 => 0xe9,
        0xda => 0xea,
        0xdb => 0xeb,
        0xdc => 0xed,
        0xdd => 0xf1,
        _ => return None,
    };
    Some([0x82, trail])
}

fn is_sjis_lead(b: u8) -> bool {
    (0x81..=0x9f).contains(&b) || (0xe0..=0xfc).contains(&b)
}

fn is_sjis_trail(b: u8) -> bool {
    (0x40..=0x7e).contains(&b) || (0x80..=0xfc).contains(&b)
}

pub fn find_bytes(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    haystack
        .get(start..)?
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|at| at + start)
}

pub fn read_u16(data: &[u8], offset: usize) -> Result<u16> {
    let bytes = data.get(offset..offset + 2).context("读取 u16 越界")?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

pub fn read_u32(data: &[u8], offset: usize) -> Result<u32> {
    let bytes = data.get(offset..offset + 4).context("读取 u32 越界")?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_lzss_roundtrip() {
        let input = (0..513).map(|n| (n * 37) as u8).collect::<Vec<_>>();
        assert_eq!(
            lzss3_decode(&lzss3_compress(&input).unwrap()).unwrap(),
            input
        );
    }

    #[test]
    fn engine_ascii_mapping() {
        assert_eq!(decode_display(&[0x85, 0x40, 0x85, 0x9e]).unwrap(), "!~");
    }

    #[test]
    fn nooch2_dictionary_never_starts_on_a_trail_byte() {
        let input = [0x9e, 0xe0, 0x81, 0x40];
        let cross_boundary_dictionary = vec![vec![0xe0, 0x81]];
        assert_eq!(
            serialize_nooch2(&input, &cross_boundary_dictionary).unwrap(),
            input
        );
    }

    #[test]
    fn extracts_command_text_without_control_bytes() {
        let (encoded, _, had_errors) =
            SHIFT_JIS.encode("I(T1えばるなバカ！*RT*T1また会える日を楽しみに・・・・*Rt2*)通常文");
        assert!(!had_errors);
        let spans = find_text_spans(&encoded);
        assert_eq!(
            spans
                .iter()
                .map(|span| (span.text.as_str(), span.command_text))
                .collect::<Vec<_>>(),
            vec![
                ("えばるなバカ！", true),
                ("また会える日を楽しみに・・・・", true),
                ("通常文", false),
            ]
        );
    }
}
