//! Lossless YOX script pool editing. Code and all instruction slots remain fixed.

use crate::carrier::{codes_in_text, cp932_encode, decode_text, encode_text};
use std::collections::BTreeMap;
use std::convert::TryInto;

const INLINE: [usize; 64] = [
    0, 1, 1, 1, 2, 3, 4, 1, 0, 1, 1, 1, 1, 0, 1, 1, 2, 1, 3, 3, 3, 3, 3, 3, 5, 1, 5, 4, 4, 5, 0, 0,
    0, 2, 3, 4, 1, 2, 2, 3, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 1, 2, 2, 0, 3, 1,
];

fn u32_at(data: &[u8], at: usize) -> Result<u32, String> {
    let b = data.get(at..at + 4).ok_or("脚本区段被截断")?;
    Ok(u32::from_le_bytes(b.try_into().unwrap()))
}

#[derive(Debug, Clone)]
pub struct StringSlot {
    pub bytes: Vec<u8>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedMessage {
    pub reference_index: usize,
    pub string_slot: usize,
    pub message: String,
    pub suffix: String,
    pub original: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProjectionStats {
    pub total_messages: usize,
    pub exported_messages: usize,
    pub filtered_layout: usize,
    pub filtered_blank: usize,
    pub filtered_control_only: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessageProjection {
    pub messages: Vec<ProjectedMessage>,
    pub stats: ProjectionStats,
}

#[derive(Debug, Clone)]
pub struct Script {
    header: [u8; 48],
    pub code: Vec<u8>,
    pub strings: Vec<StringSlot>,
    trailing: Vec<u8>,
    pub message_refs: Vec<usize>,
}

impl Script {
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        if data.len() < 48 || &data[..8] != b"YOX\0\0\0\0\x10" {
            return Err("不是 YOX 0x10000000 脚本条目".into());
        }
        let code_size = u32_at(data, 8)? as usize;
        let pool_size = u32_at(data, 12)? as usize;
        let index_size = u32_at(data, 16)? as usize;
        if !index_size.is_multiple_of(4) {
            return Err("脚本索引长度不是4的倍数".into());
        }
        let pool_start = 48usize.checked_add(code_size).ok_or("代码长度溢出")?;
        let index_start = pool_start
            .checked_add(pool_size)
            .ok_or("字符串池长度溢出")?;
        let end = index_start.checked_add(index_size).ok_or("索引长度溢出")?;
        if end > data.len() {
            return Err("脚本代码/字符串池/索引越界".into());
        }
        let code = data[48..pool_start].to_vec();
        let pool = &data[pool_start..index_start];
        let mut strings = Vec::with_capacity(index_size / 4);
        let mut covered = 0usize;
        for i in 0..index_size / 4 {
            let relative = u32_at(data, index_start + i * 4)? as usize;
            if relative >= pool.len() {
                return Err(format!("字符串槽 {i} 偏移越界"));
            }
            if relative != covered {
                return Err(format!("字符串槽 {i} 存在索引别名、重叠或池间隙"));
            }
            let rest = &pool[relative..];
            let n = rest
                .iter()
                .position(|b| *b == 0)
                .ok_or_else(|| format!("字符串槽 {i} 缺少NUL"))?;
            let bytes = rest[..n].to_vec();
            let text = decode_text(&bytes).map_err(|e| format!("字符串槽 {i}: {e}"))?;
            covered = relative + n + 1;
            strings.push(StringSlot { bytes, text });
        }
        if covered != pool.len() {
            return Err("字符串池尾部存在未引用数据".into());
        }
        let message_refs = find_message_refs(&code, strings.len())?;
        let mut header = [0u8; 48];
        header.copy_from_slice(&data[..48]);
        Ok(Self {
            header,
            code,
            strings,
            trailing: data[end..].to_vec(),
            message_refs,
        })
    }

    pub fn editable_entries(&self) -> Vec<vn_text::Entry> {
        self.message_refs
            .iter()
            .map(|slot| vn_text::Entry {
                name: None,
                message: merge_layout_lines(&self.strings[*slot].text),
            })
            .collect()
    }

    pub fn translation_projection(&self) -> Result<MessageProjection, String> {
        let mut projection = MessageProjection::default();
        for (reference_index, slot) in self.message_refs.iter().copied().enumerate() {
            let original = self.strings[slot].text.as_str();
            projection.stats.total_messages += 1;
            if has_layout_token(original) {
                projection.stats.filtered_layout += 1;
                continue;
            }
            let (body, suffix) = split_terminal_controls(original)?;
            let message = merge_layout_lines(body);
            match displayed_content(&message)? {
                DisplayContent::Blank => {
                    projection.stats.filtered_blank += 1;
                    continue;
                }
                DisplayContent::ControlOnly => {
                    projection.stats.filtered_control_only += 1;
                    continue;
                }
                DisplayContent::Visible => {}
            }
            projection.messages.push(ProjectedMessage {
                reference_index,
                string_slot: slot,
                message,
                suffix: suffix.to_owned(),
                original: original.to_owned(),
            });
        }
        projection.stats.exported_messages = projection.messages.len();
        Ok(projection)
    }

    pub fn projected_entries(&self) -> Result<Vec<vn_text::Entry>, String> {
        Ok(self
            .translation_projection()?
            .messages
            .into_iter()
            .map(|message| vn_text::Entry {
                name: None,
                message: message.message,
            })
            .collect())
    }

    pub fn apply_projected_messages(
        &mut self,
        entries: &[vn_text::Entry],
    ) -> Result<Vec<usize>, String> {
        let projection = self.translation_projection()?;
        let expected: Vec<_> = projection
            .messages
            .iter()
            .map(|message| vn_text::Entry {
                name: None,
                message: message.message.clone(),
            })
            .collect();
        vn_text::validate_shape(&expected, entries).map_err(|e| e.to_string())?;
        let full_messages: Vec<String> = projection
            .messages
            .iter()
            .zip(entries)
            .map(|(record, entry)| {
                if entry.message.contains('\0') {
                    return Err(format!("字符串槽 {}: 译文含NUL", record.string_slot));
                }
                Ok(format!("{}{}", entry.message, record.suffix))
            })
            .collect::<Result<_, String>>()?;
        self.apply_projected_full_messages(&projection, &full_messages)
    }

    fn apply_projected_full_messages(
        &mut self,
        projection: &MessageProjection,
        messages: &[String],
    ) -> Result<Vec<usize>, String> {
        if messages.len() != projection.messages.len() {
            return Err("完整译文记录数与稳定投影不一致".into());
        }
        let mut edits = BTreeMap::<usize, String>::new();
        for (record, combined) in projection.messages.iter().zip(messages) {
            if let Some(old) = edits.insert(record.string_slot, combined.clone()) {
                if old != *combined {
                    return Err(format!(
                        "字符串槽 {} 被不同消息引用且译文冲突",
                        record.string_slot
                    ));
                }
            }
        }
        let changed = edits
            .iter()
            .filter_map(|(slot, text)| (self.strings[*slot].text != *text).then_some(*slot))
            .collect();
        for (slot, text) in edits {
            self.strings[slot].text = text;
        }
        Ok(changed)
    }

    pub fn apply_projected_messages_reflow(
        &mut self,
        entries: &[vn_text::Entry],
        width_pixels: usize,
        font: &crate::font::YoxFont,
        carriers: &BTreeMap<char, u16>,
    ) -> Result<Vec<usize>, String> {
        let projection = self.translation_projection()?;
        if entries.len() != projection.messages.len() {
            return Err(format!(
                "翻译记录数{}与当前投影{}不一致",
                entries.len(),
                projection.messages.len()
            ));
        }
        let flowed = projection
            .messages
            .iter()
            .zip(entries)
            .map(|(record, entry)| {
                if entry.name.is_some() {
                    return Err("翻译视图不支持name字段".to_owned());
                }
                let combined = format!("{}{}", entry.message, record.suffix);
                validate_message_terminator(&combined)?;
                rewrap_message(&combined, width_pixels, font, carriers)
            })
            .collect::<Result<Vec<_>, String>>()?;
        self.apply_projected_full_messages(&projection, &flowed)
    }

    pub fn apply_messages(&mut self, entries: &[vn_text::Entry]) -> Result<(), String> {
        let expected = self.editable_entries();
        vn_text::validate_shape(&expected, entries).map_err(|e| e.to_string())?;
        let mut edits = BTreeMap::<usize, String>::new();
        for (slot, entry) in self.message_refs.iter().copied().zip(entries) {
            if entry.message.contains('\0') {
                return Err(format!("字符串槽 {slot}: 译文含NUL"));
            }
            if let Some(old) = edits.insert(slot, entry.message.clone()) {
                if old != entry.message {
                    return Err(format!("字符串槽 {slot} 被不同消息引用且译文冲突"));
                }
            }
        }
        for (slot, text) in edits {
            self.strings[slot].text = text;
        }
        Ok(())
    }

    pub fn apply_messages_reflow(
        &mut self,
        entries: &[vn_text::Entry],
        width_pixels: usize,
        font: &crate::font::YoxFont,
        carriers: &BTreeMap<char, u16>,
    ) -> Result<(), String> {
        let mut flowed = entries.to_vec();
        for (slot, entry) in self.message_refs.iter().copied().zip(&mut flowed) {
            validate_message_terminator(&entry.message)?;
            if self.strings[slot].text == entry.message && is_control_only_message(&entry.message) {
                continue;
            }
            entry.message = rewrap_message(&entry.message, width_pixels, font, carriers)?;
        }
        self.apply_messages(&flowed)
    }

    pub fn preserved_text(&self) -> impl Iterator<Item = &str> {
        self.strings.iter().map(|s| s.text.as_str())
    }

    pub fn raw_codes(&self) -> Result<Vec<u16>, String> {
        self.strings.iter().try_fold(Vec::new(), |mut all, slot| {
            all.extend(codes_in_text(&slot.bytes)?);
            Ok(all)
        })
    }

    pub fn rebuild(&self, carriers: &BTreeMap<char, u16>) -> Result<Vec<u8>, String> {
        let mut pool = Vec::new();
        let mut offsets = Vec::with_capacity(self.strings.len());
        for (slot, string) in self.strings.iter().enumerate() {
            if pool.len() > u32::MAX as usize {
                return Err("字符串池超过4GiB".into());
            }
            offsets.push(pool.len() as u32);
            let bytes = if string.text == string_text_from_bytes(&string.bytes)? {
                string.bytes.clone()
            } else {
                encode_text(&string.text, carriers).map_err(|e| format!("字符串槽 {slot}: {e}"))?
            };
            if bytes.contains(&0) {
                return Err(format!("字符串槽 {slot}: CP932文本编码结果含NUL"));
            }
            pool.extend_from_slice(&bytes);
            pool.push(0);
        }
        let index_size = offsets.len().checked_mul(4).ok_or("索引长度溢出")?;
        let mut out = self.header.to_vec();
        out[8..12].copy_from_slice(
            &u32::try_from(self.code.len())
                .map_err(|_| "代码超过4GiB")?
                .to_le_bytes(),
        );
        out[12..16].copy_from_slice(
            &u32::try_from(pool.len())
                .map_err(|_| "字符串池超过4GiB")?
                .to_le_bytes(),
        );
        out[16..20].copy_from_slice(
            &u32::try_from(index_size)
                .map_err(|_| "索引超过4GiB")?
                .to_le_bytes(),
        );
        out.extend_from_slice(&self.code);
        out.extend_from_slice(&pool);
        for offset in offsets {
            out.extend_from_slice(&offset.to_le_bytes());
        }
        out.extend_from_slice(&self.trailing);
        Ok(out)
    }
}

fn string_text_from_bytes(bytes: &[u8]) -> Result<String, String> {
    decode_text(bytes)
}

pub fn merge_layout_lines(text: &str) -> String {
    if is_control_only_message(text) {
        return text.to_owned();
    }
    if has_layout_token(text) {
        return text.to_owned();
    }
    let mut out = String::new();
    let mut at = 0usize;
    while at < text.len() {
        if text[at..].starts_with("@@") {
            out.push_str("@@");
            at += 2;
        } else if text[at..].starts_with("@L") {
            at += 2;
        } else {
            let ch = text[at..].chars().next().unwrap();
            out.push(ch);
            at += ch.len_utf8();
        }
    }
    out
}

fn is_control_only_message(text: &str) -> bool {
    let mut at = 0usize;
    while at < text.len() {
        if let Ok(Some(end)) = control_end(text, at) {
            at = end;
        } else {
            let ch = text[at..].chars().next().unwrap();
            if !ch.is_whitespace() {
                return false;
            }
            at += ch.len_utf8();
        }
    }
    true
}

fn has_layout_token(text: &str) -> bool {
    let mut at = 0usize;
    while at < text.len() {
        if let Ok(Some(end)) = control_end(text, at) {
            let token = &text[at..end];
            if token == "@K"
                || (token.starts_with("@B")
                    && token.len() == 4
                    && token[2..].bytes().all(|b| b.is_ascii_digit()))
            {
                return true;
            }
            at = end;
        } else {
            at += text[at..].chars().next().unwrap().len_utf8();
        }
    }
    false
}

fn split_terminal_controls(text: &str) -> Result<(&str, &str), String> {
    let mut controls = Vec::<(usize, &str)>::new();
    let mut at = 0usize;
    while at < text.len() {
        if let Some(end) = control_end(text, at)? {
            controls.push((at, &text[at..end]));
            at = end;
        } else {
            at += text[at..].chars().next().unwrap().len_utf8();
        }
    }
    if controls.last().is_none_or(|(_, token)| *token != "@P") {
        return Ok((text, ""));
    }
    let mut first = controls.len() - 1;
    while first > 0 && matches!(controls[first - 1].1, "@F" | "@I" | "@P") {
        first -= 1;
    }
    let suffix_at = controls[first].0;
    Ok((&text[..suffix_at], &text[suffix_at..]))
}

enum DisplayContent {
    Visible,
    Blank,
    ControlOnly,
}

fn displayed_content(text: &str) -> Result<DisplayContent, String> {
    let (mut at, mut visible, mut saw_whitespace) = (0usize, false, false);
    while at < text.len() {
        if let Some(end) = control_end(text, at)? {
            let token = &text[at..end];
            if token == "@@"
                || token.starts_with("@Vi")
                || token.starts_with("@Vc")
                || token.starts_with("@Vs")
                || ((token.as_bytes().get(1).is_some_and(u8::is_ascii_digit))
                    && token.contains("Vi"))
            {
                visible = true;
            }
            at = end;
        } else {
            let ch = text[at..].chars().next().unwrap();
            if ch.is_whitespace() {
                saw_whitespace = true;
            } else {
                visible = true;
            }
            at += ch.len_utf8();
        }
    }
    Ok(if visible {
        DisplayContent::Visible
    } else if saw_whitespace {
        DisplayContent::Blank
    } else {
        DisplayContent::ControlOnly
    })
}

fn control_end(text: &str, at: usize) -> Result<Option<usize>, String> {
    let tail = &text[at..];
    if tail.starts_with("@@") {
        return Ok(Some(at + 2));
    }
    if !tail.starts_with('@') {
        return Ok(None);
    }
    let rest = &tail[1..];
    let digits =
        |s: &str, n: usize| s.len() >= n && s.as_bytes()[..n].iter().all(u8::is_ascii_digit);
    let len = if rest.starts_with("Vs") || rest.starts_with("Vc") || rest.starts_with("Vi") {
        if digits(&rest[2..], 2) {
            5
        } else {
            0
        }
    } else if rest.len() >= 4
        && rest.as_bytes()[0].is_ascii_digit()
        && rest[1..].starts_with("Vi")
        && digits(&rest[3..], 2)
    {
        6
    } else if rest.len() >= 5
        && rest.as_bytes()[0] == b'0'
        && rest.as_bytes()[1].is_ascii_digit()
        && rest[2..].starts_with("Vi")
        && digits(&rest[4..], 2)
    {
        7
    } else if rest.starts_with("Ct") || rest.starts_with("Cs") {
        if digits(&rest[2..], 1) {
            4
        } else {
            0
        }
    } else if rest.starts_with("El") {
        3
    } else if rest.as_bytes().first().is_some_and(|b| b"RAB".contains(b)) && digits(&rest[1..], 2) {
        4
    } else if rest
        .as_bytes()
        .first()
        .is_some_and(|b| b"FIKLNPSTW".contains(b))
    {
        2
    } else {
        0
    };
    if len == 0 {
        Err(format!(
            "未知或损坏控制符：{}",
            tail.chars().take(8).collect::<String>()
        ))
    } else {
        Ok(Some(at + len))
    }
}

fn validate_message_terminator(message: &str) -> Result<(), String> {
    if message.contains(['\r', '\n']) {
        return Err("译文含CR/LF；消息必须使用@L排版换行".into());
    }
    let mut at = 0usize;
    let mut count = 0usize;
    let mut last_control = None;
    while at < message.len() {
        if let Some(end) = control_end(message, at)? {
            let token = &message[at..end];
            if token == "@P" {
                count += 1;
            }
            last_control = Some((token, end));
            at = end;
        } else {
            let ch = message[at..].chars().next().unwrap();
            at += ch.len_utf8();
            last_control = None;
        }
    }
    if count != 1 || last_control.is_none_or(|(token, end)| token != "@P" || end != message.len()) {
        return Err("每条3F消息必须恰有一个末尾@P；纯控制消息应保持为@P".into());
    }
    Ok(())
}

pub fn rewrap_message(
    message: &str,
    width_pixels: usize,
    font: &crate::font::YoxFont,
    carriers: &BTreeMap<char, u16>,
) -> Result<String, String> {
    if width_pixels == 0 {
        return Err("换行窗口宽度必须大于0像素".into());
    }
    if message.contains(['\r', '\n']) {
        return Err("译文不能包含CR/LF；请用排版换行控制由工具重新生成@L".into());
    }
    let input = merge_layout_lines(message);
    let protected = has_layout_token(&input);
    let mut output = String::new();
    let (mut line_width, mut block_bytes, mut blocks) = (0usize, 0usize, 1usize);
    let mut line_chars = Vec::<(char, usize, usize, usize)>::new();
    let max_digit = ('0'..='9')
        .filter_map(|c| code_advance(c, font, carriers).ok())
        .max()
        .unwrap_or(font.width as u16) as usize;
    let (max_single, max_double) = font.max_advances();
    let mut at = 0usize;
    while at < input.len() {
        if let Some(end) = control_end(&input, at)? {
            let token = &input[at..end];
            if token == "@@" {
                let advance = code_advance('@', font, carriers)? as usize;
                if !protected && (line_width + advance > width_pixels || block_bytes + 1 > 127) {
                    output.push_str("@L");
                    line_width = 0;
                    block_bytes = 0;
                    blocks += 1;
                }
                output.push_str("@@");
                line_width += advance;
                block_bytes += 1;
                line_chars.push(('@', advance, 1, output.len() - 2));
            } else if token == "@L" {
                if block_bytes == 0 {
                    return Err("@L不能提交空文本块".into());
                }
                output.push_str(token);
                line_width = 0;
                block_bytes = 0;
                blocks += 1;
                line_chars.clear();
            } else {
                let variable_width = if token.starts_with("@Vs") {
                    (64 * usize::from(max_single)).max(32 * usize::from(max_double))
                } else if token.starts_with("@Vc") {
                    14 * max_digit
                } else if token.contains("Vi") {
                    11 * max_digit
                } else {
                    0
                };
                let variable_bytes = if token.starts_with("@Vs") {
                    64
                } else if token.starts_with("@Vc") {
                    14
                } else if token.contains("Vi") {
                    11
                } else {
                    0
                };
                if variable_width > 0 && variable_width > width_pixels {
                    return Err(format!("变量控制符 {token} 的保守最大宽度 {variable_width}px 超过窗口宽度 {width_pixels}px"));
                }
                if !protected
                    && variable_width > 0
                    && (line_width + variable_width > width_pixels
                        || block_bytes + variable_bytes > 127)
                {
                    output.push_str("@L");
                    line_width = 0;
                    block_bytes = 0;
                    blocks += 1;
                }
                output.push_str(token);
                line_width = line_width.saturating_add(variable_width);
                block_bytes += variable_bytes;
                if (token == "@W" || token == "@P")
                    && block_bytes == 0
                    && !(token == "@P" && input == "@P")
                {
                    return Err(format!("{token}不能提交空文本块"));
                }
                if token == "@W" {
                    block_bytes = 0;
                    blocks += 1;
                }
                if token == "@P" {
                    block_bytes = 0;
                }
            }
            at = end;
        } else {
            let ch = input[at..].chars().next().unwrap();
            let advance = code_advance(ch, font, carriers)? as usize;
            if advance > width_pixels {
                return Err(format!(
                    "字符 U+{:04X} 的字宽 {advance}px 超过窗口宽度 {width_pixels}px",
                    ch as u32
                ));
            }
            let byte_count = encode_text(&ch.to_string(), carriers)?.len();
            let over_limit = line_width + advance > width_pixels || block_bytes + byte_count > 127;
            if !protected && over_limit {
                let mut move_count = 0usize;
                if is_closing_punctuation(ch) {
                    while line_chars
                        .get(line_chars.len().wrapping_sub(move_count + 1))
                        .is_some_and(|(last, ..)| is_closing_punctuation(*last))
                    {
                        move_count += 1;
                    }
                    if move_count < line_chars.len() {
                        move_count += 1;
                    }
                } else if line_chars
                    .last()
                    .is_some_and(|(last, ..)| is_opening_punctuation(*last))
                {
                    move_count = 1;
                }
                let can_move = move_count > 0
                    && move_count < line_chars.len()
                    && line_chars.get(line_chars.len() - move_count).is_some_and(
                        |(_, _, _, start)| {
                            output.len()
                                == *start
                                    + line_chars[line_chars.len() - move_count..]
                                        .iter()
                                        .map(|(c, ..)| c.len_utf8())
                                        .sum::<usize>()
                        },
                    );
                if can_move {
                    let split = line_chars.len() - move_count;
                    let start = line_chars[split].3;
                    let moved = line_chars.split_off(split);
                    output.truncate(start);
                    output.push_str("@L");
                    line_width = 0;
                    block_bytes = 0;
                    blocks += 1;
                    line_chars.clear();
                    for (moved_ch, moved_advance, moved_bytes, _) in moved {
                        let pos = output.len();
                        output.push(moved_ch);
                        line_chars.push((moved_ch, moved_advance, moved_bytes, pos));
                        line_width += moved_advance;
                        block_bytes += moved_bytes;
                    }
                } else if !is_closing_punctuation(ch)
                    && !line_chars
                        .last()
                        .is_some_and(|(last, ..)| is_opening_punctuation(*last))
                {
                    output.push_str("@L");
                    line_width = 0;
                    block_bytes = 0;
                    blocks += 1;
                    line_chars.clear();
                }
            }
            let char_start = output.len();
            output.push(ch);
            line_width += advance;
            block_bytes += byte_count;
            line_chars.push((ch, advance, byte_count, char_start));
            at += ch.len_utf8();
        }
        if !protected && line_width > width_pixels {
            return Err(format!(
                "标点无法在窗口宽度 {width_pixels}px 内找到安全断点，当前行宽 {line_width}px"
            ));
        }
        if block_bytes > 127 {
            return Err(format!(
                "文本块含NUL超出128字节容量（文本{block_bytes}字节）"
            ));
        }
        if blocks > 32 {
            return Err(format!("重新排版产生{blocks}个文本块，消息对象最多32块"));
        }
    }
    Ok(output)
}

fn is_closing_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '，' | '。'
            | '、'
            | '；'
            | '：'
            | '！'
            | '？'
            | ','
            | '.'
            | ';'
            | ':'
            | '!'
            | '?'
            | '）'
            | ')'
            | '】'
            | ']'
            | '》'
            | '〉'
            | '」'
            | '』'
            | '〕'
            | '〗'
            | '〙'
            | '〛'
            | '”'
            | '’'
            | '…'
    )
}

fn is_opening_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '（' | '('
            | '【'
            | '['
            | '《'
            | '〈'
            | '「'
            | '『'
            | '〔'
            | '〖'
            | '〘'
            | '〚'
            | '“'
            | '‘'
    )
}

fn code_advance(
    ch: char,
    font: &crate::font::YoxFont,
    carriers: &BTreeMap<char, u16>,
) -> Result<u16, String> {
    let code = carriers
        .get(&ch)
        .copied()
        .or_else(|| cp932_encode(ch))
        .ok_or_else(|| format!("换行测宽缺少字符 U+{:04X} 的CP932载体", ch as u32))?;
    font.advance_for_code(code)
        .ok_or_else(|| format!("CP932 {code:04X} 不在活动字库范围内"))
}

fn find_message_refs(code: &[u8], string_count: usize) -> Result<Vec<usize>, String> {
    let mut refs = Vec::new();
    let mut pc = 0usize;
    let mut boundaries = Vec::new();
    let mut labels = Vec::new();
    while pc < code.len() {
        let start = pc;
        boundaries.push(start);
        let physical = code[pc];
        pc += 1;
        let mut opcode = usize::from(physical);
        while opcode == 1 {
            if pc + 5 > code.len() {
                return Err(format!("{start:#x}: 扩展指令截断"));
            }
            let tag = code[pc];
            if tag != 1 {
                return Err(format!("{start:#x}: 扩展操作码标签错误"));
            }
            opcode = usize::try_from(u32_at(code, pc + 1)?).map_err(|_| "操作码溢出")?;
            if opcode > 255 {
                return Err(format!("{start:#x}: 操作码超出分发表"));
            }
            pc += 5;
        }
        let count = if opcode < 64 { INLINE[opcode] } else { 0 };
        for _ in 0..count {
            if pc + 5 > code.len() {
                return Err(format!("{start:#x}: 指令参数截断"));
            }
            let tag = code[pc];
            let value = u32_at(code, pc + 1)? as usize;
            if tag == 16 {
                labels.push((start, value));
            }
            if tag == 2 && value >= 16 {
                return Err(format!("{start:#x}: 寄存器索引 {value} 越界"));
            }
            if opcode == 0x3f && tag != 64 {
                return Err(format!("{start:#x}: 3F消息参数必须引用tag 40字符串槽"));
            }
            if tag == 64 {
                if value >= string_count {
                    return Err(format!("{start:#x}: 消息槽 {value} 越界"));
                }
                if opcode == 0x3f {
                    refs.push(value);
                }
            }
            if tag == 4 && value >= string_count {
                return Err(format!("{start:#x}: 字符串槽 {value} 越界"));
            }
            if !matches!(tag, 1 | 2 | 4 | 16 | 64) {
                return Err(format!("{start:#x}: 未知参数标签 {tag:#x}"));
            }
            pc += 5;
        }
    }
    boundaries.push(code.len());
    for (source, target) in labels {
        if boundaries.binary_search(&target).is_err() {
            return Err(format!("{source:#x}: 标签目标 {target:#x} 不是指令边界"));
        }
    }
    Ok(refs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font() -> crate::font::YoxFont {
        let count = 9589usize;
        let mut data = vec![0; 32 + count * 8 + count * 8];
        data[..4].copy_from_slice(b"YOX\0");
        data[4..8].copy_from_slice(&0x0100_0000u32.to_le_bytes());
        data[8..12].copy_from_slice(&0x3du32.to_le_bytes());
        data[16..20].copy_from_slice(&32u32.to_le_bytes());
        data[20..24].copy_from_slice(&(15u32 | (16u32 << 16)).to_le_bytes());
        data[24..28].copy_from_slice(&(count as u32).to_le_bytes());
        data[28..32].copy_from_slice(&((32 + count * 8) as u32).to_le_bytes());
        for i in 0..count {
            data[32 + count * 8 + i * 8..32 + count * 8 + i * 8 + 2]
                .copy_from_slice(&10u16.to_le_bytes());
        }
        crate::font::YoxFont::parse(data).unwrap()
    }

    #[test]
    fn merges_only_plain_layout_rows_and_preserves_escaped_at() {
        assert_eq!(merge_layout_lines("甲@L乙@@L丙@P"), "甲乙@@L丙@P");
        assert_eq!(merge_layout_lines("甲@@K乙@L丙@@B00@P"), "甲@@K乙丙@@B00@P");
        assert_eq!(merge_layout_lines("@L@F@I@P"), "@L@F@I@P");
        assert_eq!(control_end("@3Vi02x", 0).unwrap(), Some(6));
        assert_eq!(control_end("@03Vi02x", 0).unwrap(), Some(7));
        assert_eq!(merge_layout_lines("甲@@K乙@L丙@@B00@P"), "甲@@K乙丙@@B00@P");
        assert_eq!(merge_layout_lines("@Vi00日@L9:09@K@P"), "@Vi00日@L9:09@K@P");
    }

    #[test]
    fn translation_projection_filters_layout_blank_and_control_records_stably() {
        let texts = [
            "@Vi00日目@L15:43@K@P",
            "決戦の放課後@B00@I@P",
            "　@I@P",
            "@P",
            "普通の正文@I@P",
            "@Vi00日目@I@P",
            "@@K@P",
        ];
        let script = Script {
            header: [0; 48],
            code: Vec::new(),
            strings: texts
                .iter()
                .map(|text| StringSlot {
                    bytes: text.as_bytes().to_vec(),
                    text: (*text).to_owned(),
                })
                .collect(),
            trailing: Vec::new(),
            message_refs: (0..texts.len()).collect(),
        };
        let projection = script.translation_projection().unwrap();
        assert_eq!(
            projection
                .messages
                .iter()
                .map(|m| m.string_slot)
                .collect::<Vec<_>>(),
            [4, 5, 6]
        );
        assert_eq!(projection.messages[0].message, "普通の正文");
        assert_eq!(projection.messages[0].suffix, "@I@P");
        assert_eq!(projection.messages[1].message, "@Vi00日目");
        assert_eq!(projection.messages[2].message, "@@K");
        assert_eq!(projection.stats.filtered_layout, 2);
        assert_eq!(projection.stats.filtered_blank, 1);
        assert_eq!(projection.stats.filtered_control_only, 1);
        assert_eq!(projection.stats.exported_messages, 3);
    }

    #[test]
    fn projected_import_restores_suffix_and_does_not_touch_filtered_slots() {
        let texts = ["@Vi00日目@L15:43@K@P", "　@I@P", "普通@I@P"];
        let mut script = Script {
            header: [0; 48],
            code: Vec::new(),
            strings: texts
                .iter()
                .map(|text| StringSlot {
                    bytes: text.as_bytes().to_vec(),
                    text: (*text).to_owned(),
                })
                .collect(),
            trailing: Vec::new(),
            message_refs: vec![0, 1, 2],
        };
        let before = script
            .strings
            .iter()
            .map(|s| s.bytes.clone())
            .collect::<Vec<_>>();
        let changed = script
            .apply_projected_messages(&[vn_text::Entry {
                name: None,
                message: "翻译".into(),
            }])
            .unwrap();
        assert_eq!(changed, [2]);
        assert_eq!(script.strings[0].bytes, before[0]);
        assert_eq!(script.strings[1].bytes, before[1]);
        assert_eq!(script.strings[2].text, "翻译@I@P");
    }

    #[test]
    fn projected_reflow_appends_hidden_suffix_exactly_once() {
        let mut script = Script {
            header: [0; 48],
            code: Vec::new(),
            strings: vec![StringSlot {
                bytes: Vec::new(),
                text: "短句@I@P".into(),
            }],
            trailing: Vec::new(),
            message_refs: vec![0],
        };
        let translated = vec![vn_text::Entry {
            name: None,
            message: "短句".into(),
        }];
        script
            .apply_projected_messages_reflow(&translated, 20, &font(), &BTreeMap::new())
            .unwrap();
        assert_eq!(script.strings[0].text, "短句@I@P");
        assert_eq!(script.strings[0].text.matches("@P").count(), 1);
    }

    #[test]
    fn rewraps_on_measured_width_and_requires_message_terminator() {
        let result = rewrap_message("ABC@P", 20, &font(), &BTreeMap::new()).unwrap();
        assert_eq!(result, "AB@LC@P");
        assert!(validate_message_terminator("ABC").is_err());
        assert!(validate_message_terminator("@P").is_ok());
        assert!(rewrap_message("@B00@L@P", 20, &font(), &BTreeMap::new()).is_err());
        assert!(rewrap_message("@F@I@P", 20, &font(), &BTreeMap::new()).is_err());
    }

    #[test]
    fn wrap_moves_closing_punctuation_cluster_and_openers_to_safe_side() {
        let font = font();
        let closing = rewrap_message("甲乙丙丁，。后@P", 32, &font, &BTreeMap::new()).unwrap();
        assert_eq!(closing, "甲乙丙@L丁，。@L后@P");
        assert!(!closing.contains("，@L"));
        assert!(!closing.contains("@L。"));
        let opening = rewrap_message("甲乙丙（丁@P", 30, &font, &BTreeMap::new()).unwrap();
        assert_eq!(opening, "甲乙丙@L（丁@P");
        assert!(rewrap_message("甲，。@P", 10, &font, &BTreeMap::new()).is_err());
    }

    #[test]
    fn unchanged_control_only_source_message_keeps_original_line_control() {
        let mut script = Script {
            header: [0; 48],
            code: Vec::new(),
            strings: vec![StringSlot {
                bytes: b"@L@F@I@P".to_vec(),
                text: "@L@F@I@P".into(),
            }],
            trailing: Vec::new(),
            message_refs: vec![0],
        };
        let translation = vec![vn_text::Entry {
            name: None,
            message: "@L@F@I@P".into(),
        }];
        script
            .apply_messages_reflow(&translation, 20, &font(), &BTreeMap::new())
            .unwrap();
        assert_eq!(script.strings[0].text, "@L@F@I@P");
    }

    #[test]
    fn text_block_capacity_inserts_line_before_nul_limit() {
        let input = format!("{}@P", "漢".repeat(64));
        let result = rewrap_message(&input, 2000, &font(), &BTreeMap::new()).unwrap();
        assert!(result.contains("漢".repeat(63).as_str()));
        assert!(result.contains("@L漢"));
    }
}
