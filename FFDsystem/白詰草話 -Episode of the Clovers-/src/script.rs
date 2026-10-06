use std::ops::Range;

use repi_unpack_v2::{decode_cp932, encode_cp932};

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextKind {
    Dialogue,
    Choice,
}

#[derive(Clone, Debug)]
struct FormatTag {
    visible_offset: usize,
    raw: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct TextRecord {
    pub span: Range<usize>,
    pub message: String,
    pub kind: TextKind,
    original_body: Vec<u8>,
    format_tags: Vec<FormatTag>,
}

#[derive(Clone, Debug, Default)]
pub struct ParsedScript {
    pub records: Vec<TextRecord>,
    pub nonliteral_text_args: usize,
}

impl TextRecord {
    fn render(&self, translated: &str) -> Result<Vec<u8>> {
        if translated == self.message {
            return Ok(self.original_body.clone());
        }

        let normalized = translated.replace("\r\n", "\n").replace('\r', "\n");
        let characters: Vec<char> = normalized.chars().collect();
        let old_len = self.message.chars().count();
        let new_len = characters.len();
        let mut tags: Vec<(usize, usize, &[u8])> = self
            .format_tags
            .iter()
            .enumerate()
            .map(|(sequence, tag)| {
                let position = if old_len == 0 {
                    0
                } else {
                    ((tag.visible_offset.saturating_mul(new_len) + old_len / 2) / old_len)
                        .min(new_len)
                };
                let position = keep_markup_intact(position, &characters);
                (position, sequence, tag.raw.as_slice())
            })
            .collect();
        tags.sort_by_key(|(position, sequence, _)| (*position, *sequence));

        let mut output = Vec::new();
        let mut tag_index = 0usize;
        for (position, character) in characters.iter().enumerate() {
            while tag_index < tags.len() && tags[tag_index].0 == position {
                output.extend_from_slice(tags[tag_index].2);
                tag_index += 1;
            }
            if *character == '\n' {
                output.extend_from_slice(b"<BR>");
            } else {
                let encoded = encode_cp932(&character.to_string())?;
                append_escaped_cp932(&mut output, &encoded);
            }
        }
        while tag_index < tags.len() {
            output.extend_from_slice(tags[tag_index].2);
            tag_index += 1;
        }
        Ok(output)
    }
}

/// Keep relocated source formatting tags out of markup tags retained in the
/// translated message, such as `<B>` and `</B>`.
fn keep_markup_intact(position: usize, characters: &[char]) -> usize {
    let mut cursor = 0;
    while cursor < characters.len() {
        if characters[cursor] != '<' {
            cursor += 1;
            continue;
        }

        let mut name_start = cursor + 1;
        let is_closing = characters.get(name_start) == Some(&'/');
        if is_closing {
            name_start += 1;
        }
        if !characters
            .get(name_start)
            .is_some_and(|character| character.is_ascii_alphabetic())
        {
            cursor += 1;
            continue;
        }

        let Some(offset) = characters[name_start..]
            .iter()
            .position(|character| *character == '>')
        else {
            cursor += 1;
            continue;
        };
        let after_tag = name_start + offset + 1;
        if cursor < position && position < after_tag {
            return if is_closing { cursor } else { after_tag };
        }
        cursor = after_tag;
    }
    position
}

/// Find static text arguments in the script text constructors. Balloon text is
/// the ninth argument, `CreateText` text is the seventh, and `AddText` text is
/// the second.
pub fn parse(bytes: &[u8]) -> Result<ParsedScript> {
    let mut parsed = ParsedScript::default();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if let Some(end) = skip_comment(bytes, cursor)? {
            cursor = end;
            continue;
        }
        if bytes[cursor] == b'"' {
            cursor = skip_string(bytes, cursor)?;
            continue;
        }
        if is_sjis_lead(bytes[cursor]) {
            cursor = (cursor + 2).min(bytes.len());
            continue;
        }
        if !is_identifier_start(bytes[cursor]) {
            cursor += 1;
            continue;
        }

        let identifier_start = cursor;
        cursor += 1;
        while cursor < bytes.len() && is_identifier_continue(bytes[cursor]) {
            cursor += 1;
        }
        let identifier = std::str::from_utf8(&bytes[identifier_start..cursor])
            .map_err(|_| "脚本指令名不是 ASCII".to_owned())?;
        let constructor = match identifier {
            "CreateBalloon" | "CreateBalloonEx" => Some((TextKind::Dialogue, 8)),
            "CreateBalloonBie" => Some((TextKind::Choice, 8)),
            "CreateText" => Some((TextKind::Dialogue, 6)),
            "AddText" => Some((TextKind::Dialogue, 1)),
            _ => None,
        };
        let Some((kind, text_argument)) = constructor else {
            continue;
        };

        let mut after_name = cursor;
        skip_space_and_comments(bytes, &mut after_name)?;
        if bytes.get(after_name) != Some(&b'(') {
            continue;
        }
        let (arguments, call_end) = parse_call(bytes, after_name)?;
        if arguments.len() <= text_argument {
            parsed.nonliteral_text_args += 1;
            cursor = call_end;
            continue;
        }
        let Some(span) = string_argument(bytes, arguments[text_argument].clone())? else {
            parsed.nonliteral_text_args += 1;
            cursor = call_end;
            continue;
        };
        let original_body = bytes[span.clone()].to_vec();
        let (message, format_tags) = normalize_message(&original_body);
        parsed.records.push(TextRecord {
            span,
            message,
            kind,
            original_body,
            format_tags,
        });
        cursor = call_end;
    }
    Ok(parsed)
}

/// Replace only the recorded string-body spans. All code, comments, object IDs,
/// voice references, and unrelated source bytes remain in their original order.
pub fn inject(bytes: &[u8], records: &[TextRecord], translations: &[String]) -> Result<Vec<u8>> {
    if records.len() != translations.len() {
        return Err(format!(
            "译文条数为 {}，脚本文本记录数为 {}",
            translations.len(),
            records.len()
        ));
    }
    let mut output = bytes.to_vec();
    for (record, translation) in records.iter().zip(translations).rev() {
        let replacement = record.render(translation)?;
        if record.span.end > output.len() || record.span.start > record.span.end {
            return Err("脚本文本定位超出输入范围".into());
        }
        output.splice(record.span.clone(), replacement);
    }
    Ok(output)
}

fn normalize_message(raw: &[u8]) -> (String, Vec<FormatTag>) {
    let mut visible = Vec::with_capacity(raw.len());
    let mut tags = Vec::new();
    let mut visible_offset = 0usize;
    let mut cursor = 0usize;
    while cursor < raw.len() {
        if raw[cursor] == b'<' {
            if let Some(end) = tag_end(raw, cursor) {
                let tag = &raw[cursor..end];
                match classify_tag(tag) {
                    Some(TagKind::Break) => {
                        visible.push(b'\n');
                        visible_offset += 1;
                        cursor = end;
                        continue;
                    }
                    Some(TagKind::Formatting) => {
                        tags.push(FormatTag {
                            visible_offset,
                            raw: tag.to_vec(),
                        });
                        cursor = end;
                        continue;
                    }
                    None => {}
                }
            }
        }

        if is_sjis_lead(raw[cursor]) {
            let end = (cursor + 2).min(raw.len());
            visible.extend_from_slice(&raw[cursor..end]);
            visible_offset += 1;
            cursor = end;
        } else if raw[cursor] == b'\r' {
            visible.push(b'\n');
            visible_offset += 1;
            cursor += if raw.get(cursor + 1) == Some(&b'\n') {
                2
            } else {
                1
            };
        } else if raw[cursor] == b'\n' {
            visible.push(b'\n');
            visible_offset += 1;
            cursor += 1;
        } else if raw[cursor] == b'\\'
            && raw
                .get(cursor + 1)
                .is_some_and(|next| matches!(next, b'\\' | b'"'))
        {
            visible.push(raw[cursor + 1]);
            visible_offset += 1;
            cursor += 2;
        } else {
            visible.push(raw[cursor]);
            visible_offset += 1;
            cursor += 1;
        }
    }
    (decode_cp932(&visible), tags)
}

#[derive(Clone, Copy)]
enum TagKind {
    Break,
    Formatting,
}

fn classify_tag(tag: &[u8]) -> Option<TagKind> {
    if tag.len() < 3 || tag.first() != Some(&b'<') || tag.last() != Some(&b'>') {
        return None;
    }
    let mut content = &tag[1..tag.len() - 1];
    if content.first() == Some(&b'/') {
        content = &content[1..];
    }
    let name_end = content
        .iter()
        .position(|byte| byte.is_ascii_whitespace() || *byte == b'/')
        .unwrap_or(content.len());
    let name = &content[..name_end];
    if name.eq_ignore_ascii_case(b"BR") {
        Some(TagKind::Break)
    } else if name.eq_ignore_ascii_case(b"SOUND")
        || name.eq_ignore_ascii_case(b"TYPE")
        || name.eq_ignore_ascii_case(b"FONT")
    {
        Some(TagKind::Formatting)
    } else {
        None
    }
}

fn tag_end(bytes: &[u8], start: usize) -> Option<usize> {
    bytes[start + 1..]
        .iter()
        .position(|byte| *byte == b'>')
        .map(|offset| start + offset + 2)
}

fn append_escaped_cp932(output: &mut Vec<u8>, encoded: &[u8]) {
    let mut cursor = 0usize;
    while cursor < encoded.len() {
        if is_sjis_lead(encoded[cursor]) {
            let end = (cursor + 2).min(encoded.len());
            output.extend_from_slice(&encoded[cursor..end]);
            cursor = end;
        } else {
            if matches!(encoded[cursor], b'\\' | b'"') {
                output.push(b'\\');
            }
            output.push(encoded[cursor]);
            cursor += 1;
        }
    }
}

fn string_argument(bytes: &[u8], range: Range<usize>) -> Result<Option<Range<usize>>> {
    let mut cursor = range.start;
    skip_space_and_comments(bytes, &mut cursor)?;
    if bytes.get(cursor) != Some(&b'"') {
        return Ok(None);
    }
    let end = skip_string(bytes, cursor)?;
    let mut trailing = end;
    skip_space_and_comments(bytes, &mut trailing)?;
    if trailing != range.end {
        return Ok(None);
    }
    Ok(Some(cursor + 1..end - 1))
}

fn parse_call(bytes: &[u8], open: usize) -> Result<(Vec<Range<usize>>, usize)> {
    let mut arguments = Vec::new();
    let mut argument_start = open + 1;
    let mut cursor = argument_start;
    let mut depth = 1usize;
    while cursor < bytes.len() {
        if let Some(end) = skip_comment(bytes, cursor)? {
            cursor = end;
            continue;
        }
        if bytes[cursor] == b'"' {
            cursor = skip_string(bytes, cursor)?;
            continue;
        }
        if is_sjis_lead(bytes[cursor]) {
            cursor = (cursor + 2).min(bytes.len());
            continue;
        }
        match bytes[cursor] {
            b'(' => {
                depth += 1;
                cursor += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    arguments.push(argument_start..cursor);
                    return Ok((arguments, cursor + 1));
                }
                cursor += 1;
            }
            b',' if depth == 1 => {
                arguments.push(argument_start..cursor);
                cursor += 1;
                argument_start = cursor;
            }
            _ => cursor += 1,
        }
    }
    Err(format!("从字节偏移 0x{open:X} 开始的指令括号未闭合"))
}

fn skip_space_and_comments(bytes: &[u8], cursor: &mut usize) -> Result<()> {
    loop {
        while *cursor < bytes.len() && bytes[*cursor].is_ascii_whitespace() {
            *cursor += 1;
        }
        if let Some(end) = skip_comment(bytes, *cursor)? {
            *cursor = end;
        } else {
            return Ok(());
        }
    }
}

fn skip_comment(bytes: &[u8], start: usize) -> Result<Option<usize>> {
    if bytes.get(start..start + 2) == Some(b"//") {
        let mut cursor = start + 2;
        while cursor < bytes.len() && !matches!(bytes[cursor], b'\r' | b'\n') {
            if is_sjis_lead(bytes[cursor]) {
                cursor = (cursor + 2).min(bytes.len());
            } else {
                cursor += 1;
            }
        }
        return Ok(Some(cursor));
    }
    if bytes.get(start..start + 2) == Some(b"/*") {
        let mut cursor = start + 2;
        while cursor + 1 < bytes.len() {
            if is_sjis_lead(bytes[cursor]) {
                cursor = (cursor + 2).min(bytes.len());
            } else if bytes.get(cursor..cursor + 2) == Some(b"*/") {
                return Ok(Some(cursor + 2));
            } else {
                cursor += 1;
            }
        }
        return Err(format!("从字节偏移 0x{start:X} 开始的块注释未闭合"));
    }
    Ok(None)
}

fn skip_string(bytes: &[u8], start: usize) -> Result<usize> {
    let mut cursor = start + 1;
    while cursor < bytes.len() {
        if is_sjis_lead(bytes[cursor]) {
            cursor = (cursor + 2).min(bytes.len());
        } else if bytes[cursor] == b'\\' {
            cursor += 1;
            if cursor < bytes.len() {
                cursor = if is_sjis_lead(bytes[cursor]) {
                    (cursor + 2).min(bytes.len())
                } else {
                    cursor + 1
                };
            }
        } else if bytes[cursor] == b'"' {
            return Ok(cursor + 1);
        } else {
            cursor += 1;
        }
    }
    Err(format!("从字节偏移 0x{start:X} 开始的字符串未闭合"))
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn is_sjis_lead(byte: u8) -> bool {
    (0x81..=0x9f).contains(&byte) || (0xe0..=0xfc).contains(&byte)
}
