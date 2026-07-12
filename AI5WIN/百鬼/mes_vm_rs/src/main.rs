use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::Path;

use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};

const LZSS_N: usize = 4096;
const LZSS_F: usize = 18;
const LZSS_INIT_POS: usize = 0xfee;
const TEXT_STRING_CTRL: u8 = 0x01;

#[derive(Debug, Serialize, Deserialize)]
struct TextSegment {
    #[serde(rename = "_inst_offset")]
    inst_offset: usize,
    #[serde(rename = "_offset")]
    offset: usize,
    #[serde(rename = "_size")]
    size: usize,
    #[serde(rename = "_raw_hex")]
    raw_hex: String,
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TextEntry {
    #[serde(rename = "_file")]
    file: String,
    #[serde(rename = "_index")]
    index: usize,
    #[serde(rename = "_inst_offset")]
    inst_offset: usize,
    #[serde(rename = "_offset")]
    offset: usize,
    #[serde(rename = "_size")]
    size: usize,
    #[serde(rename = "_encoding")]
    encoding: String,
    #[serde(rename = "_raw_hex")]
    raw_hex: String,
    #[serde(rename = "_name_inst_offset", skip_serializing_if = "Option::is_none")]
    name_inst_offset: Option<usize>,
    #[serde(rename = "_name_offset", skip_serializing_if = "Option::is_none")]
    name_offset: Option<usize>,
    #[serde(rename = "_name_size", skip_serializing_if = "Option::is_none")]
    name_size: Option<usize>,
    #[serde(rename = "_name_raw_hex", skip_serializing_if = "Option::is_none")]
    name_raw_hex: Option<String>,
    #[serde(rename = "_inline_name", skip_serializing_if = "Option::is_none")]
    inline_name: Option<String>,
    #[serde(rename = "_message_segments", skip_serializing_if = "Option::is_none")]
    message_segments: Option<Vec<TextSegment>>,
    name: String,
    scr_msg: String,
    message: String,
}

fn read_u32le(data: &[u8], pos: usize) -> Option<u32> {
    data.get(pos..pos + 4)
        .map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]))
}

fn lzss_decompress(src: &[u8]) -> Vec<u8> {
    let mut text_buf = [0u8; LZSS_N];
    let mut r = LZSS_INIT_POS;
    let mut flags: u16 = 0;
    let mut ip = 0usize;
    let mut out = Vec::new();
    while ip < src.len() {
        flags >>= 1;
        if flags & 0x100 == 0 {
            if ip >= src.len() {
                break;
            }
            flags = src[ip] as u16 | 0xff00;
            ip += 1;
        }
        if flags & 1 != 0 {
            if ip >= src.len() {
                break;
            }
            let c = src[ip];
            ip += 1;
            out.push(c);
            text_buf[r] = c;
            r = (r + 1) & 0xfff;
        } else {
            if ip + 1 >= src.len() {
                break;
            }
            let lo = src[ip] as usize;
            let hi = src[ip + 1] as usize;
            ip += 2;
            let pos = lo | ((hi & 0xf0) << 4);
            let len = (hi & 0x0f) + 3;
            for k in 0..len {
                let c = text_buf[(pos + k) & 0xfff];
                out.push(c);
                text_buf[r] = c;
                r = (r + 1) & 0xfff;
            }
        }
    }
    out
}

fn lzss_compress(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < data.len() {
        let flag_pos = out.len();
        out.push(0);
        let mut flags = 0u8;
        for bit in 0..8 {
            if pos >= data.len() {
                break;
            }
            let window_start = pos.saturating_sub(LZSS_N);
            let mut best_len = 0usize;
            let mut best_abs = 0usize;
            if pos + 2 < data.len() {
                let mut cand = pos;
                while cand > window_start {
                    cand -= 1;
                    if data[cand] != data[pos] {
                        continue;
                    }
                    let mut len = 0usize;
                    while len < LZSS_F
                        && pos + len < data.len()
                        && data[cand + len] == data[pos + len]
                    {
                        len += 1;
                    }
                    if len > best_len {
                        best_len = len;
                        best_abs = cand;
                        if len == LZSS_F {
                            break;
                        }
                    }
                }
            }
            if best_len >= 3 {
                let ring_pos = (LZSS_INIT_POS + best_abs) & 0xfff;
                out.push((ring_pos & 0xff) as u8);
                out.push((((ring_pos >> 4) & 0xf0) | (best_len - 3)) as u8);
                pos += best_len;
            } else {
                flags |= 1 << bit;
                out.push(data[pos]);
                pos += 1;
            }
        }
        out[flag_pos] = flags;
    }
    out
}

fn detect_count_table(data: &[u8]) -> Option<(usize, Vec<u32>)> {
    let count = read_u32le(data, 0)? as usize;
    if count == 0 || count > 10_000 {
        return None;
    }
    let table_end = 4 + count * 4;
    if table_end + 4 > data.len() {
        return None;
    }
    if &data[table_end..table_end + 4] != b"\x12\x7f\xff\x0e" {
        return None;
    }
    let mut values = Vec::with_capacity(count);
    for i in 0..count {
        values.push(read_u32le(data, 4 + i * 4)?);
    }
    Some((table_end, values))
}

fn detect_entry_table(data: &[u8]) -> Option<(usize, Vec<u32>)> {
    let count = read_u32le(data, 0)? as usize;
    if count == 0 {
        return Some((4, Vec::new()));
    }
    if count > 4096 {
        return None;
    }
    let table_end = 4 + count * 4;
    if table_end > data.len() {
        return None;
    }
    let mut values = Vec::with_capacity(count);
    for i in 0..count {
        let value = read_u32le(data, 4 + i * 4)?;
        if value as usize >= data.len() {
            return None;
        }
        values.push(value);
    }
    if values.windows(2).any(|w| w[0] >= w[1]) {
        return None;
    }
    if (values.first().copied().unwrap_or(table_end as u32) as usize) < table_end {
        return None;
    }
    Some((table_end, values))
}

fn detect_script_table(data: &[u8]) -> Option<(usize, Vec<u32>)> {
    detect_count_table(data).or_else(|| detect_entry_table(data))
}

fn is_sjis_lead(b: u8) -> bool {
    (0x81..=0x9f).contains(&b) || (0xe0..=0xef).contains(&b) || (0xfa..=0xfc).contains(&b)
}

fn hex_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 2);
    for b in data {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

fn decode_text(raw: &[u8]) -> Option<String> {
    let (cow, _enc, had_errors) = SHIFT_JIS.decode(raw);
    if had_errors {
        return None;
    }
    Some(cow.into_owned())
}

fn encode_text(text: &str) -> Result<Vec<u8>, String> {
    let (cow, _enc, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        return Err(format!(
            "text is not encodable as cp932/shift-jis: {}",
            text
        ));
    }
    Ok(cow.into_owned())
}

fn is_resource_name(text: &str) -> bool {
    let lower = text.to_ascii_lowercase().replace('\\', "/");
    [
        ".gpr", ".wav", ".mam", ".mes", ".msk", ".bmp", ".dat", ".avi",
    ]
    .iter()
    .any(|suffix| lower.ends_with(suffix))
}

fn is_translatable_text(text: &str) -> bool {
    if text.is_empty() || text.is_ascii() || is_resource_name(text) {
        return false;
    }
    text.chars().any(|ch| {
        let c = ch as u32;
        (0x3040..=0x30ff).contains(&c)
            || (0x3400..=0x9fff).contains(&c)
            || (0xff00..=0xffef).contains(&c)
    })
}

fn collect_cstrings(data: &[u8], start: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = start;
    while i < data.len() {
        if data[i] != TEXT_STRING_CTRL {
            i += 1;
            continue;
        }
        let mut end = i + 1;
        while end < data.len() && data[end] != 0 {
            end += 1;
        }
        if end < data.len() && end > i + 1 {
            out.push((i, end + 1));
            i = end + 1;
        } else {
            i += 1;
        }
    }
    out
}

#[derive(Debug, Clone)]
struct CStringEntry {
    start: usize,
    raw: Vec<u8>,
    text: String,
}

#[allow(dead_code)]
fn starts_with_dialogue_quote(text: &str) -> bool {
    text.starts_with('「') || text.starts_with('『')
}

#[allow(dead_code)]
fn is_name_like(text: &str) -> bool {
    let len = text.chars().count();
    if len == 0 || len > 12 || is_resource_name(text) {
        return false;
    }
    if text
        .chars()
        .any(|ch| "「」『』。、，,.!?！？…・（）()[]【】".contains(ch))
    {
        return false;
    }
    text.chars().any(|ch| {
        let c = ch as u32;
        (0x3040..=0x30ff).contains(&c)
            || (0x3400..=0x9fff).contains(&c)
            || (0xff00..=0xffef).contains(&c)
    })
}

fn starts_with_dialogue_quote_clean(text: &str) -> bool {
    text.starts_with('\u{300c}') || text.starts_with('\u{300e}')
}

fn is_name_forbidden_char_clean(ch: char) -> bool {
    matches!(
        ch,
        '\u{300c}'
            | '\u{300d}'
            | '\u{300e}'
            | '\u{300f}'
            | '\u{3002}'
            | '\u{3001}'
            | '\u{ff0c}'
            | '\u{2026}'
            | '\u{30fb}'
            | '\u{ff08}'
            | '\u{ff09}'
            | '\u{3010}'
            | '\u{3011}'
            | '\u{ff01}'
            | '\u{ff1f}'
            | '\u{ff1a}'
            | '\u{ff1b}'
            | '\u{ff0f}'
            | '.'
            | ','
            | '!'
            | '?'
            | ';'
            | ':'
            | '/'
            | '\\'
            | '('
            | ')'
            | '['
            | ']'
    ) || ch.is_whitespace()
}

fn is_name_like_clean(text: &str) -> bool {
    let len = text.chars().count();
    if len == 0 || len > 12 || is_resource_name(text) {
        return false;
    }
    if text.chars().any(is_name_forbidden_char_clean) {
        return false;
    }
    text.chars().any(|ch| {
        let c = ch as u32;
        (0x3040..=0x30ff).contains(&c)
            || (0x3400..=0x9fff).contains(&c)
            || (0xff00..=0xffef).contains(&c)
    })
}

fn is_known_speaker_marker(text: &str) -> bool {
    matches!(
        text,
        "\u{7adc}\u{4e00}" // 竜一
            | "\u{9ad8}\u{5fd7}" // 高志
            | "\u{82e5}\u{8449}" // 若葉
            | "\u{5b8f}\u{7f8e}" // 宏美
            | "\u{7965}\u{5b50}" // 祥子
            | "\u{7a63}" // 穣
            | "\u{6075}\u{7f8e}" // 恵美
            | "\u{4f0a}\u{85e4}" // 伊藤
            | "\u{5927}\u{77f3}" // 大石
            | "\u{7950}\u{53f8}" // 祐司
            | "\u{3068}\u{3082}\u{5b50}" // とも子
            | "\u{30d0}\u{30d0}\u{30a1}" // ババァ
            | "\u{3042}\u{304b}\u{308a}" // あかり
            | "\u{89aa}\u{7236}" // 親父
            | "\u{7fa9}\u{660c}" // 義昌
            | "\u{8b66}\u{5b98}\u{9054}" // 警官達
            | "\u{732b}" // 猫
            | "\u{82e5}\u{8449}\u{ff06}\u{5b8f}\u{7f8e}" // 若葉＆宏美
            | "\u{5b8f}\u{7f8e}\u{ff06}\u{7965}\u{5b50}" // 宏美＆祥子
            | "\u{7a63}\u{ff06}\u{82e5}\u{8449}" // 穣＆若葉
            | "\u{7adc}\u{4e00}\u{ff06}\u{9ad8}\u{5fd7}" // 竜一＆高志
            | "\u{5b8f}\u{7f8e}\u{ff06}\u{9ad8}\u{5fd7}" // 宏美＆高志
            | "\u{82e5}\u{8449}\u{ff06}\u{5b8f}\u{7f8e}\u{ff06}\u{7965}\u{5b50}" // 若葉＆宏美＆祥子
            | "\u{7adc}\u{4e00}\u{ff06}\u{5b8f}\u{7f8e}\u{ff06}\u{7965}\u{5b50}" // 竜一＆宏美＆祥子
            | "\u{7adc}\u{4e00}\u{ff06}\u{5b8f}\u{7f8e}" // 竜一＆宏美
    )
}

fn split_inline_speaker_text(text: &str) -> Option<(&'static str, String)> {
    const SPEAKERS: &[&str] = &[
        "\u{82e5}\u{8449}\u{ff06}\u{5b8f}\u{7f8e}\u{ff06}\u{7965}\u{5b50}", // 若葉＆宏美＆祥子
        "\u{7adc}\u{4e00}\u{ff06}\u{5b8f}\u{7f8e}\u{ff06}\u{7965}\u{5b50}", // 竜一＆宏美＆祥子
        "\u{5b8f}\u{7f8e}\u{ff06}\u{7965}\u{5b50}",                         // 宏美＆祥子
        "\u{82e5}\u{8449}\u{ff06}\u{5b8f}\u{7f8e}",                         // 若葉＆宏美
        "\u{7a63}\u{ff06}\u{82e5}\u{8449}",                                 // 穣＆若葉
        "\u{7adc}\u{4e00}\u{ff06}\u{9ad8}\u{5fd7}",                         // 竜一＆高志
        "\u{5b8f}\u{7f8e}\u{ff06}\u{9ad8}\u{5fd7}",                         // 宏美＆高志
        "\u{7adc}\u{4e00}\u{ff06}\u{5b8f}\u{7f8e}",                         // 竜一＆宏美
        "\u{8b66}\u{5b98}\u{9054}",                                         // 警官達
        "\u{30d0}\u{30d0}\u{30a1}",                                         // ババァ
        "\u{3068}\u{3082}\u{5b50}",                                         // とも子
        "\u{3042}\u{304b}\u{308a}",                                         // あかり
        "\u{7adc}\u{4e00}",                                                 // 竜一
        "\u{9ad8}\u{5fd7}",                                                 // 高志
        "\u{82e5}\u{8449}",                                                 // 若葉
        "\u{5b8f}\u{7f8e}",                                                 // 宏美
        "\u{7965}\u{5b50}",                                                 // 祥子
        "\u{7a63}",                                                         // 穣
        "\u{6075}\u{7f8e}",                                                 // 恵美
        "\u{4f0a}\u{85e4}",                                                 // 伊藤
        "\u{5927}\u{77f3}",                                                 // 大石
        "\u{7950}\u{53f8}",                                                 // 祐司
        "\u{89aa}\u{7236}",                                                 // 親父
        "\u{7fa9}\u{660c}",                                                 // 義昌
        "\u{732b}",                                                         // 猫
    ];
    let trimmed = text.trim_start_matches(|ch: char| ch.is_whitespace() || ch == '\u{3000}');
    for speaker in SPEAKERS {
        if let Some(rest) = trimmed.strip_prefix(speaker) {
            if rest.starts_with('\u{300c}') || rest.starts_with('\u{300e}') {
                return Some((*speaker, rest.to_string()));
            }
        }
    }
    None
}

fn import_text_pair(entry: &TextEntry, old_text: &str) -> (String, String) {
    if !entry.name.is_empty() && entry.name_inst_offset.is_none() {
        let old_inline_name = entry
            .inline_name
            .clone()
            .or_else(|| {
                old_text
                    .strip_suffix(&entry.scr_msg)
                    .map(|prefix| prefix.to_string())
            })
            .unwrap_or_else(|| entry.name.clone());
        (
            format!("{}{}", old_inline_name, entry.scr_msg),
            format!("{}{}", entry.name, entry.message),
        )
    } else {
        (entry.scr_msg.clone(), entry.message.clone())
    }
}

fn is_junk_fragment(text: &str) -> bool {
    matches!(
        text,
        "\u{ff96}" // ﾖ
            | "\u{30e7}" // ョ
            | "\u{7724}" // 眤
            | "\u{72e2}" // 狢
            | "\u{71ff}" // 燿
            | "\u{ff66}" // ｦ
            | "\u{ff86}" // ﾆ
            | "\u{ff9e}" // ﾞ
    )
}

fn quote_delta(text: &str, open: char, close: char) -> i32 {
    text.chars().fold(0, |acc, ch| {
        if ch == open {
            acc + 1
        } else if ch == close {
            acc - 1
        } else {
            acc
        }
    })
}

fn primary_quote_pair(text: &str) -> Option<(char, char)> {
    if text.starts_with('\u{300c}') {
        Some(('\u{300c}', '\u{300d}'))
    } else if text.starts_with('\u{300e}') {
        Some(('\u{300e}', '\u{300f}'))
    } else {
        None
    }
}

fn make_segment(s: &CStringEntry) -> TextSegment {
    TextSegment {
        inst_offset: s.start,
        offset: s.start + 1,
        size: s.raw.len(),
        raw_hex: hex_encode(&s.raw),
        text: s.text.clone(),
    }
}

fn has_following_dialogue(
    strings: &[CStringEntry],
    text_indices: &[usize],
    rel_pos: usize,
    consumed: &[bool],
) -> bool {
    let next_rel = rel_pos + 1;
    if next_rel >= text_indices.len() {
        return false;
    }
    let next_idx = text_indices[next_rel];
    !consumed[next_idx] && starts_with_dialogue_quote_clean(&strings[next_idx].text)
}

fn collect_message_group(
    strings: &[CStringEntry],
    text_indices: &[usize],
    start_rel: usize,
    consumed: &[bool],
) -> (Vec<usize>, usize) {
    let mut group = vec![text_indices[start_rel]];
    let first = &strings[text_indices[start_rel]].text;
    let Some((open, close)) = primary_quote_pair(first) else {
        return (group, start_rel);
    };

    let mut balance = quote_delta(first, open, close);
    let mut rel = start_rel;
    while balance > 0 && rel + 1 < text_indices.len() {
        let next_rel = rel + 1;
        let next_idx = text_indices[next_rel];
        if consumed[next_idx] {
            break;
        }
        let next_text = &strings[next_idx].text;
        if is_name_like_clean(next_text)
            && has_following_dialogue(strings, text_indices, next_rel, consumed)
        {
            break;
        }
        group.push(next_idx);
        balance += quote_delta(next_text, open, close);
        rel = next_rel;
    }
    (group, rel)
}

fn build_text_entry(
    source_name: &str,
    index: usize,
    name_idx: Option<usize>,
    msg_indices: &[usize],
    strings: &[CStringEntry],
) -> TextEntry {
    let first_msg = &strings[msg_indices[0]];
    let segments: Vec<TextSegment> = msg_indices
        .iter()
        .map(|&idx| make_segment(&strings[idx]))
        .collect();
    let mut scr_msg = msg_indices
        .iter()
        .map(|&idx| strings[idx].text.as_str())
        .collect::<String>();
    let (name_inst_offset, name_offset, name_size, name_raw_hex, mut name) =
        if let Some(idx) = name_idx {
            let s = &strings[idx];
            (
                Some(s.start),
                Some(s.start + 1),
                Some(s.raw.len()),
                Some(hex_encode(&s.raw)),
                s.text.clone(),
            )
        } else {
            (None, None, None, None, String::new())
        };
    let mut inline_name = None;
    if name_idx.is_none() {
        if let Some((speaker, message)) = split_inline_speaker_text(&scr_msg) {
            name = speaker.to_string();
            inline_name = Some(speaker.to_string());
            scr_msg = message;
        }
    }

    TextEntry {
        file: source_name.to_string(),
        index,
        inst_offset: first_msg.start,
        offset: first_msg.start + 1,
        size: first_msg.raw.len(),
        encoding: "cp932".to_string(),
        raw_hex: hex_encode(&first_msg.raw),
        name_inst_offset,
        name_offset,
        name_size,
        name_raw_hex,
        inline_name,
        message_segments: if segments.len() > 1 {
            Some(segments)
        } else {
            None
        },
        name,
        scr_msg: scr_msg.clone(),
        message: scr_msg,
    }
}

fn collect_text_entries(data: &[u8], source_name: &str) -> Result<Vec<TextEntry>, String> {
    let (table_end, _entries) = detect_script_table(data).ok_or("script table not detected")?;
    let mut strings = Vec::new();
    for (start, end) in collect_cstrings(data, table_end) {
        let raw = &data[start + 1..end - 1];
        if raw.iter().any(|&b| b < 0x09) {
            continue;
        }
        let Some(text) = decode_text(raw) else {
            continue;
        };
        strings.push(CStringEntry {
            start,
            raw: raw.to_vec(),
            text,
        });
    }

    let text_indices: Vec<usize> = strings
        .iter()
        .enumerate()
        .filter_map(|(idx, s)| {
            (is_translatable_text(&s.text) && !is_junk_fragment(&s.text)).then_some(idx)
        })
        .collect();
    let no_consumed = vec![false; strings.len()];
    let speaker_names: BTreeSet<String> = text_indices
        .iter()
        .enumerate()
        .filter_map(|(rel, &idx)| {
            (is_name_like_clean(&strings[idx].text)
                && has_following_dialogue(&strings, &text_indices, rel, &no_consumed))
            .then(|| strings[idx].text.clone())
        })
        .collect();
    let mut out = Vec::new();
    let mut consumed = vec![false; strings.len()];
    let mut rel = 0usize;
    while rel < text_indices.len() {
        let i = text_indices[rel];
        if consumed[i] {
            rel += 1;
            continue;
        }

        if is_name_like_clean(&strings[i].text)
            && has_following_dialogue(&strings, &text_indices, rel, &consumed)
        {
            let start_rel = rel + 1;
            let (msg_indices, end_rel) =
                collect_message_group(&strings, &text_indices, start_rel, &consumed);
            let index = out.len();
            out.push(build_text_entry(
                source_name,
                index,
                Some(i),
                &msg_indices,
                &strings,
            ));
            consumed[i] = true;
            for &idx in &msg_indices {
                consumed[idx] = true;
            }
            rel = end_rel + 1;
            continue;
        }

        if starts_with_dialogue_quote_clean(&strings[i].text) {
            let (msg_indices, end_rel) =
                collect_message_group(&strings, &text_indices, rel, &consumed);
            let index = out.len();
            out.push(build_text_entry(
                source_name,
                index,
                None,
                &msg_indices,
                &strings,
            ));
            for &idx in &msg_indices {
                consumed[idx] = true;
            }
            rel = end_rel + 1;
            continue;
        }

        if speaker_names.contains(&strings[i].text) || is_known_speaker_marker(&strings[i].text) {
            consumed[i] = true;
            rel += 1;
            continue;
        }

        let index = out.len();
        out.push(build_text_entry(source_name, index, None, &[i], &strings));
        consumed[i] = true;
        rel += 1;
    }
    Ok(out)
}

fn parse_hex_bytes(s: &str) -> Result<Vec<u8>, String> {
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.len() % 2 != 0 {
        return Err("hex byte string length must be even".to_string());
    }
    let mut out = Vec::new();
    for i in (0..compact.len()).step_by(2) {
        let b = u8::from_str_radix(&compact[i..i + 2], 16)
            .map_err(|_| format!("bad hex byte: {}", &compact[i..i + 2]))?;
        out.push(b);
    }
    Ok(out)
}

fn parse_num(s: &str) -> Result<usize, String> {
    if let Some(hex) = s.strip_prefix("0x") {
        usize::from_str_radix(hex, 16).map_err(|e| e.to_string())
    } else {
        s.parse::<usize>().map_err(|e| e.to_string())
    }
}

fn map_copied_range(old_to_new: &mut [usize], old_start: usize, old_end: usize, new_start: usize) {
    for old_pos in old_start..old_end {
        old_to_new[old_pos] = new_start + (old_pos - old_start);
    }
}

fn rebuild_inline(
    data: &[u8],
    patches: &BTreeMap<usize, Vec<u8>>,
) -> Result<(Vec<u8>, usize, usize), String> {
    let (table_end, entries) = detect_script_table(data).ok_or("script table not detected")?;
    let mut out = Vec::with_capacity(data.len());
    let mut old_to_new = vec![0usize; data.len() + 1];
    let mut cursor = 0usize;

    for (&inst_offset, new_raw) in patches {
        if inst_offset < table_end {
            return Err(format!(
                "patch offset 0x{:x} is inside count table",
                inst_offset
            ));
        }
        if inst_offset < cursor {
            return Err(format!("overlapping patch at 0x{:x}", inst_offset));
        }
        if inst_offset >= data.len() || data[inst_offset] != TEXT_STRING_CTRL {
            return Err(format!(
                "patch offset 0x{:x} does not point to a text command",
                inst_offset
            ));
        }
        let term_rel = data[inst_offset + 1..]
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| format!("unterminated text at 0x{:x}", inst_offset))?;
        let old_end = inst_offset + 1 + term_rel + 1;

        let new_start = out.len();
        out.extend_from_slice(&data[cursor..inst_offset]);
        map_copied_range(&mut old_to_new, cursor, inst_offset, new_start);

        let patched_start = out.len();
        out.push(TEXT_STRING_CTRL);
        out.extend_from_slice(new_raw);
        out.push(0);

        let new_len = new_raw.len() + 2;
        for old_pos in inst_offset..old_end {
            old_to_new[old_pos] = patched_start + (old_pos - inst_offset).min(new_len - 1);
        }
        cursor = old_end;
    }

    let tail_start = out.len();
    out.extend_from_slice(&data[cursor..]);
    map_copied_range(&mut old_to_new, cursor, data.len(), tail_start);
    old_to_new[data.len()] = out.len();

    let mut fixes = 0usize;
    for (index, &old_target) in entries.iter().enumerate() {
        let old_target = old_target as usize;
        let new_target = if old_target < table_end || old_target > data.len() {
            old_target
        } else {
            old_to_new[old_target]
        };
        if new_target != old_target {
            fixes += 1;
        }
        out[4 + index * 4..4 + index * 4 + 4].copy_from_slice(&(new_target as u32).to_le_bytes());
    }
    // F3 is a variable-table lookup prefix, not a jump opcode. The formats of
    // EF FF and 0A are also not modeled as address-bearing instructions. Do
    // not rewrite opcode operands by byte pattern; only the confirmed leading
    // MES entry table is relocated here.
    Ok((out, fixes, 0))
}

fn write_recompressed(output: &Path, plain: &[u8]) -> Result<(), String> {
    let packed = lzss_compress(plain);
    if lzss_decompress(&packed) != plain {
        return Err("internal lzss verification failed".to_string());
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(output, packed).map_err(|e| e.to_string())
}

fn dump(path: &Path) -> Result<(), String> {
    let packed = fs::read(path).map_err(|e| e.to_string())?;
    let data = lzss_decompress(&packed);
    let (table_end, entries) = detect_count_table(&data).ok_or("count table not detected")?;
    let strings = collect_cstrings(&data, table_end);
    println!("{{");
    println!("  \"file\": \"{}\",", path.display());
    println!("  \"packed_size\": {},", packed.len());
    println!("  \"plain_size\": {},", data.len());
    println!("  \"count_table_end\": {},", table_end);
    println!("  \"entry_count\": {},", entries.len());
    println!("  \"cstring_count\": {},", strings.len());
    println!("  \"first_entries\": [");
    for (i, v) in entries.iter().take(32).enumerate() {
        let comma = if i + 1 == entries.len().min(32) {
            ""
        } else {
            ","
        };
        println!("    {{\"index\": {}, \"offset\": {}}}{}", i, v, comma);
    }
    println!("  ],");
    println!("  \"first_strings\": [");
    for (i, (start, end)) in strings.iter().take(32).enumerate() {
        let comma = if i + 1 == strings.len().min(32) {
            ""
        } else {
            ","
        };
        let first = data.get(start + 1).copied().unwrap_or(0);
        println!(
            "    {{\"offset\": {}, \"size\": {}, \"sjis_lead\": {}}}{}",
            start,
            end - start,
            is_sjis_lead(first),
            comma
        );
    }
    println!("  ]");
    println!("}}");
    Ok(())
}

fn sorted_mes_files(input_dir: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut files = Vec::new();
    for item in fs::read_dir(input_dir).map_err(|e| e.to_string())? {
        let path = item.map_err(|e| e.to_string())?.path();
        if path
            .extension()
            .and_then(|x| x.to_str())
            .map(|x| x.eq_ignore_ascii_case("mes"))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
    files.sort_by_key(|p| p.file_name().map(|x| x.to_ascii_lowercase()));
    Ok(files)
}

fn export_dir(input_dir: &Path, json_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(json_dir).map_err(|e| e.to_string())?;
    let mut files = 0usize;
    let mut total_entries = 0usize;
    for path in sorted_mes_files(input_dir)? {
        let packed = fs::read(&path).map_err(|e| e.to_string())?;
        let data = lzss_decompress(&packed);
        let source_name = path
            .file_name()
            .and_then(|x| x.to_str())
            .ok_or("bad source filename")?;
        let entries = collect_text_entries(&data, source_name)?;
        let out_path = json_dir.join(format!(
            "{}.json",
            path.file_stem()
                .and_then(|x| x.to_str())
                .ok_or("bad source stem")?
        ));
        let text = serde_json::to_string_pretty(&entries).map_err(|e| e.to_string())?;
        fs::write(out_path, text).map_err(|e| e.to_string())?;
        files += 1;
        total_entries += entries.len();
    }
    eprintln!("exported files={} entries={}", files, total_entries);
    Ok(())
}

fn raw_text_at<'a>(data: &'a [u8], start: usize, label: &str) -> Result<&'a [u8], String> {
    if start >= data.len() || data[start] != TEXT_STRING_CTRL {
        return Err(format!("{} 0x{:x} is not a text command", label, start));
    }
    let term = data[start + 1..]
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| format!("unterminated text at 0x{:x}", start))?;
    Ok(&data[start + 1..start + 1 + term])
}

fn char_len(text: &str) -> usize {
    text.chars().count()
}

fn is_soft_split_char(ch: char) -> bool {
    matches!(
        ch,
        '\u{3002}'
            | '\u{3001}'
            | '\u{ff0c}'
            | '\u{ff01}'
            | '\u{ff1f}'
            | '\u{2026}'
            | '\u{300d}'
            | '\u{300f}'
            | '.'
            | ','
            | '!'
            | '?'
    )
}

fn split_message_for_segments(message: &str, segments: &[TextSegment]) -> Vec<String> {
    if segments.len() <= 1 {
        return vec![message.to_string()];
    }
    let chars: Vec<char> = message.chars().collect();
    if chars.is_empty() {
        return (0..segments.len()).map(|_| String::new()).collect();
    }
    let old_total = segments
        .iter()
        .map(|s| char_len(&s.text))
        .sum::<usize>()
        .max(1);
    let mut cuts = Vec::new();
    let mut old_acc = 0usize;
    let mut prev_cut = 0usize;
    for (idx, seg) in segments.iter().enumerate().take(segments.len() - 1) {
        old_acc += char_len(&seg.text);
        let remaining_parts = segments.len() - idx - 1;
        let min_cut = prev_cut;
        let max_cut = chars.len().saturating_sub(remaining_parts);
        let mut target = ((chars.len() * old_acc) + (old_total / 2)) / old_total;
        target = target.clamp(min_cut, max_cut);

        let window = 8usize;
        let lo = target.saturating_sub(window).max(min_cut);
        let hi = (target + window).min(max_cut);
        let mut best = target;
        let mut best_dist = usize::MAX;
        for pos in lo..=hi {
            if pos > 0 && is_soft_split_char(chars[pos - 1]) {
                let dist = pos.abs_diff(target);
                if dist < best_dist {
                    best = pos;
                    best_dist = dist;
                }
            }
        }
        cuts.push(best);
        prev_cut = best;
    }

    let mut parts = Vec::with_capacity(segments.len());
    let mut start = 0usize;
    for cut in cuts {
        parts.push(chars[start..cut].iter().collect());
        start = cut;
    }
    parts.push(chars[start..].iter().collect());
    parts
}

fn import_one(
    input_path: &Path,
    json_path: Option<&Path>,
    output_path: &Path,
) -> Result<(usize, usize, usize), String> {
    let packed = fs::read(input_path).map_err(|e| e.to_string())?;
    let data = lzss_decompress(&packed);
    let mut patches = BTreeMap::new();
    if let Some(json_path) = json_path {
        if json_path.is_file() {
            let text = fs::read_to_string(json_path).map_err(|e| e.to_string())?;
            let entries: Vec<TextEntry> = serde_json::from_str(&text).map_err(|e| e.to_string())?;
            for entry in entries {
                if let Some(segments) = &entry.message_segments {
                    let mut old_text = String::new();
                    for segment in segments {
                        let old_raw = raw_text_at(&data, segment.inst_offset, "_message_segments")?;
                        if hex_encode(old_raw) != segment.raw_hex {
                            return Err(format!(
                                "{}: segment raw bytes changed at 0x{:x}",
                                input_path.display(),
                                segment.inst_offset
                            ));
                        }
                        let decoded = decode_text(old_raw).ok_or_else(|| {
                            format!("segment text decode failed at 0x{:x}", segment.inst_offset)
                        })?;
                        if decoded != segment.text {
                            return Err(format!(
                                "{}: segment text mismatch at 0x{:x}",
                                input_path.display(),
                                segment.inst_offset
                            ));
                        }
                        old_text.push_str(&decoded);
                    }
                    let (expected_old_text, output_message) = import_text_pair(&entry, &old_text);
                    if old_text != expected_old_text {
                        return Err(format!(
                            "{}: grouped scr_msg mismatch at 0x{:x}",
                            input_path.display(),
                            entry.inst_offset
                        ));
                    }
                    let new_parts = if entry.message == entry.scr_msg {
                        segments.iter().map(|s| s.text.clone()).collect::<Vec<_>>()
                    } else {
                        split_message_for_segments(&output_message, segments)
                    };
                    for (segment, new_text) in segments.iter().zip(new_parts.iter()) {
                        let old_raw = raw_text_at(&data, segment.inst_offset, "_message_segments")?;
                        let encoded = encode_text(new_text)?;
                        if encoded != old_raw {
                            patches.insert(segment.inst_offset, encoded);
                        }
                    }
                } else {
                    let old_start = entry.inst_offset;
                    let old_raw = raw_text_at(&data, old_start, "_inst_offset")?;
                    if hex_encode(old_raw) != entry.raw_hex {
                        return Err(format!(
                            "{}: raw bytes changed at 0x{:x}",
                            input_path.display(),
                            old_start
                        ));
                    }
                    let old_text = decode_text(old_raw)
                        .ok_or_else(|| format!("old text decode failed at 0x{:x}", old_start))?;
                    let (expected_old_text, output_message) = import_text_pair(&entry, &old_text);
                    if old_text != expected_old_text {
                        return Err(format!(
                            "{}: scr_msg mismatch at 0x{:x}",
                            input_path.display(),
                            old_start
                        ));
                    }
                    let encoded = encode_text(&output_message)?;
                    if encoded != old_raw {
                        patches.insert(entry.inst_offset, encoded);
                    }
                }

                if let Some(name_start) = entry.name_inst_offset {
                    let name_raw = raw_text_at(&data, name_start, "_name_inst_offset")?;
                    if let Some(expected) = &entry.name_raw_hex {
                        if hex_encode(name_raw) != *expected {
                            return Err(format!(
                                "{}: name raw bytes changed at 0x{:x}",
                                input_path.display(),
                                name_start
                            ));
                        }
                    }
                    let encoded_name = encode_text(&entry.name)?;
                    if encoded_name != name_raw {
                        patches.insert(name_start, encoded_name);
                    }
                }
            }
        }
    }
    let (plain, table_fixes, vm_fixes) = if patches.is_empty() {
        (data, 0, 0)
    } else {
        rebuild_inline(&data, &patches)?
    };
    write_recompressed(output_path, &plain)?;
    Ok((patches.len(), table_fixes, vm_fixes))
}

fn import_dir(input_dir: &Path, json_dir: &Path, output_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
    let mut files = 0usize;
    let mut patched_files = 0usize;
    let mut patches = 0usize;
    let mut table_fixes = 0usize;
    let mut vm_fixes = 0usize;
    for input_path in sorted_mes_files(input_dir)? {
        let stem = input_path
            .file_stem()
            .and_then(|x| x.to_str())
            .ok_or("bad source stem")?;
        let name = input_path
            .file_name()
            .and_then(|x| x.to_str())
            .ok_or("bad source filename")?;
        let json_path = json_dir.join(format!("{}.json", stem));
        let output_path = output_dir.join(name);
        let (file_patches, file_table_fixes, file_vm_fixes) = import_one(
            &input_path,
            if json_path.is_file() {
                Some(json_path.as_path())
            } else {
                None
            },
            &output_path,
        )?;
        files += 1;
        if file_patches != 0 {
            patched_files += 1;
        }
        patches += file_patches;
        table_fixes += file_table_fixes;
        vm_fixes += file_vm_fixes;
    }
    eprintln!(
        "imported files={} patched_files={} patches={} table_fixes={} vm_fixes={}",
        files, patched_files, patches, table_fixes, vm_fixes
    );
    Ok(())
}

fn patch_inline(
    input: &Path,
    output: &Path,
    text_offset: usize,
    append: &[u8],
) -> Result<(), String> {
    let packed = fs::read(input).map_err(|e| e.to_string())?;
    let data = lzss_decompress(&packed);
    if text_offset >= data.len() || data[text_offset] != TEXT_STRING_CTRL {
        return Err("text offset does not point at a 0x01 cstring".to_string());
    }
    let end = data[text_offset + 1..]
        .iter()
        .position(|&b| b == 0)
        .map(|p| text_offset + 1 + p)
        .ok_or("cstring terminator not found")?;
    let mut raw = data[text_offset + 1..end].to_vec();
    raw.extend_from_slice(append);
    let mut patches = BTreeMap::new();
    patches.insert(text_offset, raw);
    let (plain, table_fixes, vm_fixes) = rebuild_inline(&data, &patches)?;
    write_recompressed(output, &plain)?;
    eprintln!(
        "inline offset=0x{:x} append_len={} table_fixes={} vm_fixes={}",
        text_offset,
        append.len(),
        table_fixes,
        vm_fixes
    );
    Ok(())
}

fn patch_blockpool(
    input: &Path,
    output: &Path,
    text_offset: usize,
    append: &[u8],
) -> Result<(), String> {
    let packed = fs::read(input).map_err(|e| e.to_string())?;
    let mut data = lzss_decompress(&packed);
    let (_table_end, entries) = detect_count_table(&data).ok_or("count table not detected")?;
    let entry_index = entries
        .iter()
        .enumerate()
        .filter(|(_, &v)| (v as usize) <= text_offset)
        .max_by_key(|(_, &v)| v)
        .map(|(i, _)| i)
        .ok_or("text offset is before first entry")?;
    let block_start = entries[entry_index] as usize;
    let block_end = entries
        .iter()
        .filter_map(|&v| {
            let x = v as usize;
            if x > block_start && x <= data.len() {
                Some(x)
            } else {
                None
            }
        })
        .min()
        .ok_or("no following entry for block end")?;
    let rel = text_offset
        .checked_sub(block_start)
        .ok_or("text offset is outside selected block")?;
    let mut block = data[block_start..block_end].to_vec();
    if rel >= block.len() || block[rel] != 0x01 {
        return Err("text offset does not point at a 0x01 cstring".to_string());
    }
    let end = block[rel + 1..]
        .iter()
        .position(|&b| b == 0)
        .map(|p| rel + 1 + p)
        .ok_or("cstring terminator not found")?;
    block.splice(end..end, append.iter().copied());
    let new_block_off = data.len();
    data.extend_from_slice(&block);
    data[4 + entry_index * 4..4 + entry_index * 4 + 4]
        .copy_from_slice(&(new_block_off as u32).to_le_bytes());
    let out = lzss_compress(&data);
    if lzss_decompress(&out) != data {
        return Err("internal lzss verification failed".to_string());
    }
    fs::write(output, out).map_err(|e| e.to_string())?;
    eprintln!(
        "entry={} old_block=0x{:x}..0x{:x} new_block=0x{:x} append_len={}",
        entry_index,
        block_start,
        block_end,
        new_block_off,
        append.len()
    );
    Ok(())
}

fn usage() {
    eprintln!("usage:");
    eprintln!("  mes_vm_rs dump <input.MES>");
    eprintln!("  mes_vm_rs export-dir <input_mes_dir> <json_dir>");
    eprintln!("  mes_vm_rs import-dir <input_mes_dir> <json_dir> <output_mes_dir>");
    eprintln!(
        "  mes_vm_rs patch-inline <input.MES> <output.MES> --offset 0xEC5 --append-hex 928695b6"
    );
    eprintln!(
        "  mes_vm_rs patch-blockpool <input.MES> <output.MES> --offset 0x891 --append-hex 928695b6"
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        Some("dump") if args.len() == 3 => dump(Path::new(&args[2])),
        Some("export-dir") if args.len() == 4 => {
            export_dir(Path::new(&args[2]), Path::new(&args[3]))
        }
        Some("import-dir") if args.len() == 5 => import_dir(
            Path::new(&args[2]),
            Path::new(&args[3]),
            Path::new(&args[4]),
        ),
        Some("patch-inline") if args.len() == 8 => {
            let input = Path::new(&args[2]);
            let output = Path::new(&args[3]);
            if args[4] != "--offset" || args[6] != "--append-hex" {
                usage();
                std::process::exit(2);
            }
            let offset = parse_num(&args[5]).unwrap_or_else(|e| {
                eprintln!("bad offset: {}", e);
                std::process::exit(2);
            });
            let append = parse_hex_bytes(&args[7]).unwrap_or_else(|e| {
                eprintln!("{}", e);
                std::process::exit(2);
            });
            patch_inline(input, output, offset, &append)
        }
        Some("patch-blockpool") if args.len() == 8 => {
            let input = Path::new(&args[2]);
            let output = Path::new(&args[3]);
            if args[4] != "--offset" || args[6] != "--append-hex" {
                usage();
                std::process::exit(2);
            }
            let offset = parse_num(&args[5]).unwrap_or_else(|e| {
                eprintln!("bad offset: {}", e);
                std::process::exit(2);
            });
            let append = parse_hex_bytes(&args[7]).unwrap_or_else(|e| {
                eprintln!("{}", e);
                std::process::exit(2);
            });
            patch_blockpool(input, output, offset, &append)
        }
        _ => {
            usage();
            std::process::exit(2);
        }
    };
    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_index_is_not_relocated_when_text_grows() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&8u32.to_le_bytes());
        data.extend_from_slice(&[0xf3, 0x0c, 0x00]);
        data.extend_from_slice(&[TEXT_STRING_CTRL, b'A', 0x00]);

        let mut patches = BTreeMap::new();
        patches.insert(11, b"longer".to_vec());
        let (rebuilt, table_fixes, vm_fixes) = rebuild_inline(&data, &patches).unwrap();

        assert_eq!(&rebuilt[8..11], &[0xf3, 0x0c, 0x00]);
        assert_eq!(table_fixes, 0);
        assert_eq!(vm_fixes, 0);
    }
}
