use std::ops::Range;

use encoding_rs::SHIFT_JIS;
use vn_text::Entry;

const COMMANDS: &str = "add amsp bar bg bgm bgmfadeout bgmonce bgmvol blt br btn btndef btntime btnwait btnwait2 caption cl click csp csp2 date dec defaultfont defaultspeed defmp3vol defsevol delay deletescreenshot dim div dwave dwaveloop dwavestop effect end erasetextwindow exbtn exbtn_d flushout font for game getcursorpos getscreenshot gettext gettimer globalon gosub goto if inc intlimit isfull ispage kidokuskip killmenu layermessage ld len loadgame lookbackbutton lookbackcolor lr_trap lsp lsp2 lsph menu_full menu_window mid mod mode_wave_demo monocro mov movie msp msp2 mul nega next notif nsa numalias print quake quakex quakey repaint resettimer return rmode rnd rnd2 roff savefileexist savegame savename savenumber saveoff saveon select setlayer setwindow setwindow2 setwindow3 setwindow4 sevol shell spbtn stop stralias sub systemcall tal texec textbtnwait textclear textgosub textoff texton time underline usewheel versionstr vsp wait waittimer";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecordKind {
    BareText,
    QuotedText,
}

#[derive(Debug, Clone)]
struct TextRecord {
    start: usize,
    end: usize,
    line: usize,
    kind: RecordKind,
    original_message: String,
    window: Option<TextWindow>,
}

#[derive(Debug, Clone, Copy)]
struct TextWindow {
    columns: usize,
    rows: usize,
}

#[derive(Debug, Clone, Copy)]
struct TextWindowSpec {
    columns: usize,
    rows: usize,
}

#[derive(Debug, Clone, Copy)]
struct TextUnit {
    start: usize,
    end: usize,
    width: usize,
    break_after: bool,
}

#[derive(Debug, Clone)]
pub struct ScriptCatalog {
    plain: Vec<u8>,
    records: Vec<TextRecord>,
    entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub struct InjectionResult {
    pub plain: Vec<u8>,
    pub expected_entries: Vec<Entry>,
    pub normalized_literals: usize,
    pub normalized_literal_entries: Vec<usize>,
    pub wrapped_entries: Vec<usize>,
}

impl ScriptCatalog {
    pub fn parse(plain: &[u8]) -> Result<Self, String> {
        if !plain.starts_with(b"*define") {
            return Err("明文脚本未以 *define 开始；拒绝按本作脚本解析".into());
        }
        decode_shift_jis(plain, "脚本")?;

        let mut catalog = Self {
            plain: plain.to_vec(),
            records: Vec::new(),
            entries: Vec::new(),
        };
        let mut select_continuation = false;
        let mut text_window: Option<TextWindowSpec> = None;
        let mut offset = 0usize;
        let mut line_number = 1usize;

        while offset < plain.len() {
            let line_start = offset;
            let lf = plain[offset..].iter().position(|byte| *byte == b'\n');
            let (mut content_end, next_offset) = match lf {
                Some(relative) => (offset + relative, offset + relative + 1),
                None => (plain.len(), plain.len()),
            };
            if content_end > line_start && plain[content_end - 1] == b'\r' {
                content_end -= 1;
            }
            let line = &plain[line_start..content_end];

            if line_is_blank_or_comment(line) {
                offset = next_offset;
                line_number += 1;
                continue;
            }

            if select_continuation && starts_with_quote_or_space_then_quote(line) {
                let spans = quoted_spans(line, line_number)?;
                for span in spans {
                    catalog.add_record(
                        line_start + span.start,
                        line_start + span.end,
                        line_number,
                        RecordKind::QuotedText,
                        None,
                    )?;
                }
                select_continuation = ends_with_comma(line);
                offset = next_offset;
                line_number += 1;
                continue;
            }
            select_continuation = false;

            let significant = skip_ascii_indent(line);
            if significant < line.len() && line[significant] == b'*' {
                offset = next_offset;
                line_number += 1;
                continue;
            }

            if let Some((word, word_end)) = ascii_command_word(&line[significant..]) {
                if !is_known_command(word) {
                    return Err(format!(
                        "第 {line_number} 行以未知 ASCII 指令 ({}) 开始；为避免漏提文本，停止解析",
                        String::from_utf8_lossy(word)
                    ));
                }
                let command = std::str::from_utf8(word)
                    .map_err(|_| format!("第 {line_number} 行指令名不是 ASCII"))?
                    .to_ascii_lowercase();
                if matches!(command.as_str(), "setwindow" | "setwindow3") {
                    text_window = parse_text_window_spec(line, significant + word_end);
                } else if command == "setwindow4" {
                    text_window = None;
                }
                if command == "select" {
                    for span in quoted_spans(line, line_number)? {
                        catalog.add_record(
                            line_start + span.start,
                            line_start + span.end,
                            line_number,
                            RecordKind::QuotedText,
                            None,
                        )?;
                    }
                    select_continuation = ends_with_comma(line);
                } else if matches!(
                    command.as_str(),
                    "caption" | "savename" | "versionstr" | "if" | "mov" | "add"
                ) {
                    for span in quoted_spans(line, line_number)? {
                        let value = &line[span.start..span.end];
                        if contains_non_ascii(value) && !looks_like_resource_name(value) {
                            catalog.add_record(
                                line_start + span.start,
                                line_start + span.end,
                                line_number,
                                RecordKind::QuotedText,
                                None,
                            )?;
                        }
                    }
                } else if matches!(command.as_str(), "lsp" | "lsph") {
                    for span in quoted_spans(line, line_number)? {
                        let value = &line[span.start..span.end];
                        if let Some(payload) = string_sprite_payload(value) {
                            catalog.add_record(
                                line_start + span.start + payload.start,
                                line_start + span.start + payload.end,
                                line_number,
                                RecordKind::QuotedText,
                                None,
                            )?;
                        }
                    }
                }
            } else {
                let window = text_window.map(|spec| TextWindow {
                    columns: spec.columns,
                    rows: spec.rows,
                });
                catalog.add_bare_text_line(line, line_start, line_number, window)?;
            }

            offset = next_offset;
            line_number += 1;
        }

        Ok(catalog)
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn inject(&self, translations: &[Entry]) -> Result<InjectionResult, String> {
        if translations.len() != self.records.len() {
            return Err(format!(
                "译文有 {} 条，当前来源需要 {} 条",
                translations.len(),
                self.records.len()
            ));
        }

        let mut output = Vec::with_capacity(self.plain.len());
        let mut expected_entries = Vec::with_capacity(translations.len());
        let mut normalized_literals = 0usize;
        let mut normalized_literal_entries = Vec::new();
        let mut wrapped_entries = Vec::new();
        let mut cursor = 0usize;
        for (index, (record, translated)) in self.records.iter().zip(translations).enumerate() {
            if record.start < cursor || record.end < record.start || record.end > self.plain.len() {
                return Err(format!("第 {} 条记录定位重叠或越界", index + 1));
            }
            output.extend_from_slice(&self.plain[cursor..record.start]);
            let original = &self.plain[record.start..record.end];
            let (message, normalized_count) = if translated.message == record.original_message
                || record.kind != RecordKind::BareText
            {
                (translated.message.clone(), 0)
            } else {
                normalize_bare_literals(&translated.message)
            };
            normalized_literals += normalized_count;
            if normalized_count > 0 {
                normalized_literal_entries.push(index + 1);
            }
            let encoded = if message == record.original_message {
                original.to_vec()
            } else {
                encode_translation(&message, record.kind, index + 1, record.line)?
            };
            let chunks = match (record.kind, record.window) {
                (RecordKind::BareText, Some(window)) => {
                    wrap_text(&encoded, window.columns, window.rows).map_err(|error| {
                        format!("第 {} 条（源第 {} 行）{error}", index + 1, record.line)
                    })?
                }
                _ => vec![encoded.clone()],
            };
            if chunks.len() > 1 {
                wrapped_entries.push(index + 1);
                let line_ending = script_line_ending(&self.plain, record.end);
                for (chunk_index, chunk) in chunks.iter().enumerate() {
                    output.extend_from_slice(chunk);
                    expected_entries.push(Entry {
                        name: translated.name.clone(),
                        message: extracted_message_after_injection(
                            chunk,
                            record.kind,
                            index + 1,
                            record.line,
                        )?,
                    });
                    if chunk_index + 1 < chunks.len() {
                        output.extend_from_slice(line_ending);
                        output.extend_from_slice(b"br");
                        output.extend_from_slice(line_ending);
                    }
                }
            } else if message == record.original_message {
                output.extend_from_slice(original);
                expected_entries.push(self.entries[index].clone());
            } else {
                output.extend_from_slice(&encoded);
                expected_entries.push(Entry {
                    name: translated.name.clone(),
                    message: extracted_message_after_injection(
                        &encoded,
                        record.kind,
                        index + 1,
                        record.line,
                    )?,
                });
            }
            cursor = record.end;
        }
        output.extend_from_slice(&self.plain[cursor..]);
        Ok(InjectionResult {
            plain: output,
            expected_entries,
            normalized_literals,
            normalized_literal_entries,
            wrapped_entries,
        })
    }

    fn add_bare_text_line(
        &mut self,
        line: &[u8],
        line_start: usize,
        line_number: usize,
        window: Option<TextWindow>,
    ) -> Result<(), String> {
        let mut start = 0usize;
        while let Some(next) = fixed_prefix_control_end(line, start) {
            start = next;
        }
        let suffix_start = terminal_control_suffix_start(line).unwrap_or(line.len());
        if start >= suffix_start {
            return Ok(());
        }
        let visible = &line[start..suffix_start];
        if !has_renderable_content(visible) {
            return Ok(());
        }
        self.add_record(
            line_start + start,
            line_start + suffix_start,
            line_number,
            RecordKind::BareText,
            window,
        )
    }

    fn add_record(
        &mut self,
        start: usize,
        end: usize,
        line: usize,
        kind: RecordKind,
        window: Option<TextWindow>,
    ) -> Result<(), String> {
        if start >= end {
            return Ok(());
        }
        let message = decode_shift_jis(&self.plain[start..end], &format!("第 {line} 行文本"))?;
        if message.trim().is_empty() {
            return Ok(());
        }
        self.records.push(TextRecord {
            start,
            end,
            line,
            kind,
            original_message: message.clone(),
            window,
        });
        self.entries.push(Entry {
            name: None,
            message,
        });
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct Span {
    start: usize,
    end: usize,
}

fn decode_shift_jis(bytes: &[u8], label: &str) -> Result<String, String> {
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(bytes);
    if had_errors {
        return Err(format!("{label}不是有效的 Shift_JIS / CP932 字节序列"));
    }
    Ok(decoded.into_owned())
}

fn encode_translation(
    translated: &str,
    kind: RecordKind,
    index: usize,
    line: usize,
) -> Result<Vec<u8>, String> {
    if translated.contains(['\0', '\r', '\n']) {
        return Err(format!("第 {index} 条（源第 {line} 行）含 NUL 或换行"));
    }
    if kind == RecordKind::QuotedText && translated.contains('"') {
        return Err(format!(
            "第 {index} 条（源第 {line} 行）含双引号，会破坏命令字符串"
        ));
    }
    let (encoded, _, had_errors) = SHIFT_JIS.encode(translated);
    if had_errors {
        let unsupported = translated
            .chars()
            .find(|character| {
                let one = character.to_string();
                SHIFT_JIS.encode(&one).2
            })
            .map(|character| format!(" U+{:04X} ({character})", character as u32))
            .unwrap_or_default();
        return Err(format!(
            "第 {index} 条（源第 {line} 行）不能编码为 CP932；不支持的字符{unsupported}"
        ));
    }
    let encoded = encoded.into_owned();
    validate_control_syntax(&encoded, kind, index, line)?;
    Ok(encoded)
}

fn normalize_bare_literals(message: &str) -> (String, usize) {
    let characters: Vec<char> = message.chars().collect();
    let mut ruby_markers = vec![false; characters.len()];
    let mut cursor = 0usize;
    while cursor < characters.len() {
        if characters[cursor] != '(' {
            cursor += 1;
            continue;
        }
        let mut close = None;
        let mut slash = None;
        let mut slash_count = 0usize;
        let mut index = cursor + 1;
        while index < characters.len() && characters[index] != '(' {
            if characters[index] == '/' {
                slash_count += 1;
                slash = Some(index);
            }
            if characters[index] == ')' {
                close = Some(index);
                break;
            }
            index += 1;
        }
        if let (Some(close), Some(slash)) = (close, slash) {
            if slash_count == 1 {
                ruby_markers[cursor] = true;
                ruby_markers[slash] = true;
                ruby_markers[close] = true;
                cursor = close + 1;
                continue;
            }
        }
        cursor += 1;
    }

    let mut normalized = String::with_capacity(message.len());
    let mut replaced = 0usize;
    for (index, character) in characters.iter().copied().enumerate() {
        let previous = index
            .checked_sub(1)
            .and_then(|prior| characters.get(prior))
            .copied();
        let next = characters.get(index + 1).copied();
        let replacement = match character {
            '(' if !ruby_markers[index] => Some('（'),
            ')' if !ruby_markers[index] => Some('）'),
            '/' if !ruby_markers[index] => Some('／'),
            '\\' => Some('＼'),
            '@' if previous.is_some_and(is_ascii_word) && next.is_some_and(is_ascii_word) => {
                Some('＠')
            }
            '$' if !next.is_some_and(is_ascii_word) => Some('＄'),
            '%' if !next.is_some_and(is_ascii_word) => Some('％'),
            '#' if !has_hex_color(&characters, index) => Some('＃'),
            '&' if !next.is_some_and(|value| value.is_ascii_digit()) => Some('＆'),
            '!' if !is_known_bang_control(&characters, index) => Some('！'),
            _ => None,
        };
        if let Some(replacement) = replacement {
            normalized.push(replacement);
            replaced += 1;
        } else {
            normalized.push(character);
        }
    }
    (normalized, replaced)
}

fn is_ascii_word(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

fn has_hex_color(characters: &[char], start: usize) -> bool {
    start + 6 < characters.len()
        && characters[start + 1..=start + 6]
            .iter()
            .all(|character| character.is_ascii_hexdigit())
}

fn is_known_bang_control(characters: &[char], start: usize) -> bool {
    let Some(kind) = characters.get(start + 1).copied() else {
        return false;
    };
    if matches!(kind, 'd' | 'w') {
        return characters
            .get(start + 2)
            .is_some_and(|character| character.is_ascii_digit());
    }
    if kind == 's' {
        return characters
            .get(start + 2)
            .is_some_and(|character| *character == 'd' || character.is_ascii_digit());
    }
    matches!(kind, '@' | '/')
}

fn line_is_blank_or_comment(line: &[u8]) -> bool {
    let first = skip_ascii_indent(line);
    first >= line.len() || line[first] == b';'
}

fn skip_ascii_indent(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .position(|byte| !matches!(*byte, b' ' | b'\t'))
        .unwrap_or(bytes.len())
}

fn ascii_command_word(bytes: &[u8]) -> Option<(&[u8], usize)> {
    let start = skip_ascii_indent(bytes);
    if start >= bytes.len() || !bytes[start].is_ascii_alphabetic() {
        return None;
    }
    let mut end = start + 1;
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }
    Some((&bytes[start..end], end))
}

fn parse_text_window_spec(line: &[u8], arguments_start: usize) -> Option<TextWindowSpec> {
    let arguments: Vec<_> = line[arguments_start..]
        .split(|byte| *byte == b',')
        .collect();
    let argument = |index| parse_unsigned_ascii(arguments.get(index).copied()?);
    let text_x = argument(0)?;
    let columns = argument(2)?;
    let rows = argument(3)?;
    let font_width = argument(4)?;
    let font_height = argument(5)?;
    let spacing_x = argument(6)?;
    let spacing_y = argument(7)?;
    let window_right = argument(14)?;
    let window_bottom = argument(15)?;
    let horizontal_advance = font_width.saturating_add(spacing_x).max(1);
    let vertical_advance = font_height.saturating_add(spacing_y).max(1);
    let pixel_columns = window_right.saturating_add(1).saturating_sub(text_x) / horizontal_advance;
    let pixel_rows =
        window_bottom.saturating_add(1).saturating_sub(argument(1)?) / vertical_advance;
    let columns = columns.min(pixel_columns);
    let rows = rows.min(pixel_rows);
    (columns > 0 && rows > 0).then_some(TextWindowSpec { columns, rows })
}

fn parse_unsigned_ascii(bytes: &[u8]) -> Option<usize> {
    let start = bytes.iter().position(|byte| !byte.is_ascii_whitespace())?;
    let end = bytes.iter().rposition(|byte| !byte.is_ascii_whitespace())? + 1;
    let trimmed = &bytes[start..end];
    if trimmed.is_empty() || !trimmed.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(trimmed).ok()?.parse().ok()
}

fn script_line_ending(script: &[u8], from: usize) -> &'static [u8] {
    if let Some(relative_lf) = script
        .get(from..)
        .and_then(|tail| tail.iter().position(|b| *b == b'\n'))
    {
        if from + relative_lf > 0 && script[from + relative_lf - 1] == b'\r' {
            b"\r\n"
        } else {
            b"\n"
        }
    } else if script.windows(2).any(|pair| pair == b"\r\n") {
        b"\r\n"
    } else {
        b"\n"
    }
}

fn wrap_text(bytes: &[u8], columns: usize, window_rows: usize) -> Result<Vec<Vec<u8>>, String> {
    let max_width = columns.saturating_mul(2);
    let units = text_units(bytes)?;
    let width: usize = units.iter().map(|unit| unit.width).sum();
    if width <= max_width || units.is_empty() {
        return Ok(vec![bytes.to_vec()]);
    }
    let content_units_end = units
        .iter()
        .rposition(|unit| unit.width > 0)
        .map(|index| index + 1)
        .unwrap_or(0);

    let mut chunks = Vec::new();
    let mut first_unit = 0usize;
    while first_unit < content_units_end {
        let mut width = 0usize;
        let mut fit_end = first_unit;
        while fit_end < content_units_end {
            let next_width = width.saturating_add(units[fit_end].width);
            if next_width > max_width && fit_end > first_unit {
                break;
            }
            if next_width > max_width {
                return Err("单个注音或控制结构宽于文本窗口，无法安全自动换行".into());
            }
            width = next_width;
            fit_end += 1;
        }
        if fit_end == content_units_end {
            chunks.push(bytes[units[first_unit].start..units[fit_end - 1].end].to_vec());
            break;
        }
        let break_end = (first_unit..fit_end)
            .rev()
            .find(|index| units[*index].break_after)
            .map(|index| index + 1)
            .unwrap_or(fit_end);
        if break_end <= first_unit {
            return Err("译文无法在文本窗口宽度内安全拆分".into());
        }
        chunks.push(bytes[units[first_unit].start..units[break_end - 1].end].to_vec());
        first_unit = break_end;
    }
    if content_units_end < units.len() {
        if let Some(last_chunk) = chunks.last_mut() {
            last_chunk.extend_from_slice(
                &bytes[units[content_units_end].start..units[units.len() - 1].end],
            );
        }
    }

    if chunks.len() > window_rows {
        return Err(format!(
            "译文宽度需要 {} 行，超过文本窗口总行数 {} 行；请缩短译文",
            chunks.len(),
            window_rows
        ));
    }
    Ok(chunks)
}

fn text_units(bytes: &[u8]) -> Result<Vec<TextUnit>, String> {
    let mut units = Vec::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if let Some(end) = inline_control_end(bytes, cursor) {
            units.push(TextUnit {
                start: cursor,
                end,
                width: 0,
                break_after: false,
            });
            cursor = end;
            continue;
        }
        if matches!(ascii_at(bytes, cursor), Some(b'$' | b'%')) {
            let end = identifier_end(bytes, cursor + 1);
            if end > cursor + 1 {
                units.push(TextUnit {
                    start: cursor,
                    end,
                    width: (end - cursor).saturating_mul(2),
                    break_after: false,
                });
                cursor = end;
                continue;
            }
        }
        let (length, ascii) = unit_at(bytes, cursor);
        if ascii == Some(b'(') {
            if let Some((slash, close)) = ruby_span(bytes, cursor) {
                let ruby_width: usize = text_units(&bytes[cursor + 1..slash])?
                    .iter()
                    .map(|unit| unit.width)
                    .sum();
                units.push(TextUnit {
                    start: cursor,
                    end: close + 1,
                    width: ruby_width,
                    break_after: false,
                });
                cursor = close + 1;
                continue;
            }
        }
        let character = decode_shift_jis(&bytes[cursor..cursor + length], "回注文本")?
            .chars()
            .next()
            .ok_or("回注文本含空字符")?;
        let width = if ascii.is_some() { 1 } else { 2 };
        units.push(TextUnit {
            start: cursor,
            end: cursor + length,
            width,
            break_after: is_line_break_punctuation(character),
        });
        cursor += length;
    }
    Ok(units)
}

fn ruby_span(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    let mut slash = None;
    let mut cursor = start + 1;
    while cursor < bytes.len() {
        let (length, ascii) = unit_at(bytes, cursor);
        match ascii {
            Some(b'(') => return None,
            Some(b'/') if slash.is_none() => slash = Some(cursor),
            Some(b'/') => return None,
            Some(b')') => return slash.map(|slash| (slash, cursor)),
            _ => {}
        }
        cursor += length;
    }
    None
}

fn is_line_break_punctuation(character: char) -> bool {
    matches!(
        character,
        ',' | '.'
            | ';'
            | ':'
            | '!'
            | '?'
            | '，'
            | '。'
            | '、'
            | '；'
            | '：'
            | '！'
            | '？'
            | '…'
            | '―'
            | '—'
            | '－'
    )
}

fn is_known_command(word: &[u8]) -> bool {
    COMMANDS
        .split_ascii_whitespace()
        .any(|candidate| candidate.as_bytes().eq_ignore_ascii_case(word))
}

fn starts_with_quote_or_space_then_quote(line: &[u8]) -> bool {
    let start = skip_ascii_indent(line);
    line.get(start) == Some(&b'"')
}

fn ends_with_comma(line: &[u8]) -> bool {
    let end = line
        .iter()
        .rposition(|byte| !matches!(*byte, b' ' | b'\t'))
        .map(|index| index + 1)
        .unwrap_or(0);
    end > 0 && line[end - 1] == b','
}

fn quoted_spans(line: &[u8], line_number: usize) -> Result<Vec<Span>, String> {
    let mut spans = Vec::new();
    let mut cursor = 0usize;
    while cursor < line.len() {
        let (unit_len, ascii) = unit_at(line, cursor);
        match ascii {
            Some(b';') => break,
            Some(b'"') => {
                let start = cursor + 1;
                let mut end = start;
                let mut found = false;
                while end < line.len() {
                    let (length, value) = unit_at(line, end);
                    if value == Some(b'"') {
                        found = true;
                        break;
                    }
                    end += length;
                }
                if !found {
                    return Err(format!("第 {line_number} 行命令字符串没有闭合双引号"));
                }
                spans.push(Span { start, end });
                cursor = end + 1;
            }
            _ => cursor += unit_len,
        }
    }
    Ok(spans)
}

fn string_sprite_payload(value: &[u8]) -> Option<Range<usize>> {
    if !value.starts_with(b":s/") {
        return None;
    }
    let separator = value.iter().position(|byte| *byte == b';')?;
    let mut start = separator + 1;
    loop {
        if value.get(start) != Some(&b'#') || start + 7 > value.len() {
            break;
        }
        if value[start + 1..start + 7]
            .iter()
            .all(u8::is_ascii_hexdigit)
        {
            start += 7;
        } else {
            break;
        }
    }
    (start < value.len()).then_some(start..value.len())
}

fn contains_non_ascii(bytes: &[u8]) -> bool {
    bytes.iter().any(|byte| *byte >= 0x80)
}

fn looks_like_resource_name(bytes: &[u8]) -> bool {
    let lower = String::from_utf8_lossy(bytes).to_ascii_lowercase();
    [
        ".bmp", ".jpg", ".jpeg", ".png", ".gif", ".wav", ".mp3", ".ogg", ".dll", ".mpg", ".mpeg",
        ".avi", ".dat", ".nsa",
    ]
    .iter()
    .any(|extension| lower.contains(extension))
        || ["sys/", "bg/", "wav/", "mp3/", "movie/", "dll\\"]
            .iter()
            .any(|prefix| lower.starts_with(prefix))
}

fn fixed_prefix_control_end(bytes: &[u8], start: usize) -> Option<usize> {
    match ascii_at(bytes, start)? {
        b'@' => Some(start + 1),
        b'!' => {
            let kind = ascii_at(bytes, start + 1)?;
            let digits_start = start + 2;
            if kind == b's' && ascii_at(bytes, digits_start) == Some(b'd') {
                Some(digits_start + 1)
            } else if matches!(kind, b'd' | b'w' | b's') {
                let end = digits_end(bytes, digits_start);
                (end > digits_start).then_some(end)
            } else {
                None
            }
        }
        b'#' => {
            let end = start + 7;
            (end <= bytes.len() && bytes[start + 1..end].iter().all(u8::is_ascii_hexdigit))
                .then_some(end)
        }
        b'&' => {
            let end = digits_end(bytes, start + 1);
            (end > start + 1).then_some(end)
        }
        _ => None,
    }
}

fn terminal_control_suffix_start(bytes: &[u8]) -> Option<usize> {
    let mut end = bytes.len();
    while end > 0 && matches!(bytes[end - 1], b' ' | b'\t') {
        end -= 1;
    }
    let mut cursor = end;
    let mut found = false;
    loop {
        let Some((start, value)) = previous_unit(bytes, cursor) else {
            break;
        };
        if !matches!(value, Some(b'@' | b'/' | b'\\')) {
            break;
        }
        if value == Some(b'/') && inside_ruby(&bytes[..start]) {
            break;
        }
        found = true;
        cursor = start;
    }
    if found {
        while cursor > 0 && matches!(bytes[cursor - 1], b' ' | b'\t') {
            cursor -= 1;
        }
        Some(cursor)
    } else {
        None
    }
}

fn extracted_message_after_injection(
    bytes: &[u8],
    kind: RecordKind,
    index: usize,
    line: usize,
) -> Result<String, String> {
    let visible = if kind == RecordKind::BareText {
        let mut start = 0usize;
        while let Some(next) = fixed_prefix_control_end(bytes, start) {
            start = next;
        }
        let end = terminal_control_suffix_start(bytes).unwrap_or(bytes.len());
        if start < end {
            &bytes[start..end]
        } else {
            &bytes[start..start]
        }
    } else {
        bytes
    };
    decode_shift_jis(visible, &format!("第 {index} 条（源第 {line} 行）回注文本"))
}

fn inside_ruby(bytes: &[u8]) -> bool {
    let mut open = false;
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let (length, ascii) = unit_at(bytes, cursor);
        match ascii {
            Some(b'(') if !open => open = true,
            Some(b')') if open => open = false,
            _ => {}
        }
        cursor += length;
    }
    open
}

fn has_renderable_content(bytes: &[u8]) -> bool {
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if let Some(end) = inline_control_end(bytes, cursor) {
            cursor = end;
            continue;
        }
        let (length, ascii) = unit_at(bytes, cursor);
        if let Some(value) = ascii {
            if !value.is_ascii_whitespace() {
                return true;
            }
        } else {
            return true;
        }
        cursor += length;
    }
    false
}

fn inline_control_end(bytes: &[u8], start: usize) -> Option<usize> {
    match ascii_at(bytes, start)? {
        b'@' | b'\\' | b'/' => Some(start + 1),
        b'!' => {
            let kind = ascii_at(bytes, start + 1)?;
            if kind == b's' && ascii_at(bytes, start + 2) == Some(b'd') {
                Some(start + 3)
            } else if matches!(kind, b'd' | b'w' | b's') {
                let end = digits_end(bytes, start + 2);
                (end > start + 2).then_some(end)
            } else {
                None
            }
        }
        b'#' => {
            let end = start + 7;
            (end <= bytes.len() && bytes[start + 1..end].iter().all(u8::is_ascii_hexdigit))
                .then_some(end)
        }
        b'&' => {
            let end = digits_end(bytes, start + 1);
            (end > start + 1).then_some(end)
        }
        _ => None,
    }
}

fn validate_control_syntax(
    bytes: &[u8],
    kind: RecordKind,
    index: usize,
    line: usize,
) -> Result<(), String> {
    let mut ruby_state = 0u8; // 0=outside ruby, 1=base text, 2=reading text.
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let (length, ascii) = unit_at(bytes, cursor);
        if ascii == Some(b'!') {
            let marker = ascii_at(bytes, cursor + 1);
            let end = match marker {
                Some(b'd' | b'w') => {
                    let end = digits_end(bytes, cursor + 2);
                    if end == cursor + 2 {
                        return Err(format!(
                            "第 {index} 条（源第 {line} 行）含缺少数字参数的 `!d` / `!w` 控制符"
                        ));
                    }
                    Some(end)
                }
                Some(b's') if ascii_at(bytes, cursor + 2) == Some(b'd') => Some(cursor + 3),
                Some(b's') => {
                    let end = digits_end(bytes, cursor + 2);
                    if end == cursor + 2 {
                        return Err(format!(
                            "第 {index} 条（源第 {line} 行）含缺少数字参数的 `!s` 控制符"
                        ));
                    }
                    Some(end)
                }
                Some(b'@' | b'/') => Some(cursor + 2),
                _ => None,
            };
            if let Some(end) = end {
                cursor = end;
                continue;
            }
        }
        if kind == RecordKind::BareText {
            match ascii {
                Some(b'#') => {
                    let end = cursor + 7;
                    if end > bytes.len()
                        || !bytes[cursor + 1..end].iter().all(u8::is_ascii_hexdigit)
                    {
                        return Err(format!(
                            "第 {index} 条（源第 {line} 行）含无效颜色标记；使用 #RRGGBB"
                        ));
                    }
                    cursor = end;
                    continue;
                }
                Some(b'&') => {
                    let end = digits_end(bytes, cursor + 1);
                    if end == cursor + 1 {
                        return Err(format!(
                            "第 {index} 条（源第 {line} 行）含缺少编号的 `&` 调色板标记"
                        ));
                    }
                    cursor = end;
                    continue;
                }
                Some(b'$' | b'%') => {
                    let end = identifier_end(bytes, cursor + 1);
                    if end == cursor + 1 {
                        return Err(format!(
                            "第 {index} 条（源第 {line} 行）含不完整的变量替换标记"
                        ));
                    }
                    cursor = end;
                    continue;
                }
                _ => {}
            }
        }
        match ascii {
            Some(b'(') if kind == RecordKind::BareText => {
                if ruby_state != 0 {
                    return Err(format!("第 {index} 条（源第 {line} 行）含嵌套注音括号"));
                }
                ruby_state = 1;
                cursor += length;
            }
            Some(b'/') if kind == RecordKind::BareText => {
                if ruby_state != 1 {
                    return Err(format!(
                        "第 {index} 条（源第 {line} 行）含未配对的 `/`；普通对白终止符由源脚本恢复"
                    ));
                }
                ruby_state = 2;
                cursor += length;
            }
            Some(b')') if kind == RecordKind::BareText => {
                if ruby_state != 2 {
                    return Err(format!("第 {index} 条（源第 {line} 行）含不完整注音结构"));
                }
                ruby_state = 0;
                cursor += length;
            }
            Some(b'\\') if kind == RecordKind::BareText => {
                return Err(format!(
                    "第 {index} 条（源第 {line} 行）含行内页末符 `\\`；固定句末控制符由源脚本恢复"
                ));
            }
            _ => cursor += length,
        }
    }
    if ruby_state != 0 {
        return Err(format!("第 {index} 条（源第 {line} 行）含未闭合注音结构"));
    }
    Ok(())
}

fn digits_end(bytes: &[u8], mut cursor: usize) -> usize {
    while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
        cursor += 1;
    }
    cursor
}

fn identifier_end(bytes: &[u8], mut cursor: usize) -> usize {
    while cursor < bytes.len() && (bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'_') {
        cursor += 1;
    }
    cursor
}

fn ascii_at(bytes: &[u8], index: usize) -> Option<u8> {
    if index >= bytes.len() || is_cp932_lead(bytes[index]) {
        None
    } else {
        Some(bytes[index])
    }
}

fn unit_at(bytes: &[u8], index: usize) -> (usize, Option<u8>) {
    let byte = bytes[index];
    if is_cp932_lead(byte) {
        (2, None)
    } else {
        (1, Some(byte))
    }
}

fn previous_unit(bytes: &[u8], end: usize) -> Option<(usize, Option<u8>)> {
    let mut cursor = 0usize;
    let mut previous = None;
    while cursor < end {
        let (length, ascii) = unit_at(bytes, cursor);
        if cursor + length > end {
            return None;
        }
        previous = Some((cursor, ascii));
        cursor += length;
    }
    (cursor == end).then_some(previous).flatten()
}

fn is_cp932_lead(byte: u8) -> bool {
    (0x81..=0x9f).contains(&byte) || (0xe0..=0xfc).contains(&byte)
}

#[cfg(test)]
mod tests {
    use super::{decode_shift_jis, terminal_control_suffix_start, ScriptCatalog};
    use encoding_rs::SHIFT_JIS;

    #[test]
    fn strips_fixed_line_ending_controls_but_not_cp932_trail_bytes() {
        assert_eq!(terminal_control_suffix_start(b"abc\\"), Some(3));
        let line = [0x81, 0x40, b'\\'];
        assert_eq!(terminal_control_suffix_start(&line), Some(2));
        let trail_at = [0x81, 0x40];
        assert_eq!(terminal_control_suffix_start(&trail_at), None);
    }

    #[test]
    fn extracts_bare_text_and_multiline_select_choices_in_source_order() {
        let source = concat!(
            "*define\n",
            "*start\n",
            "「こんにちは」\\\n",
            "select \"１．はい\",*yes,\n",
            "\"２．いいえ\",*no\n",
            "end\n"
        );
        let (source, _, errors) = SHIFT_JIS.encode(source);
        assert!(!errors);
        let catalog = ScriptCatalog::parse(&source).unwrap();
        let messages: Vec<_> = catalog
            .entries()
            .iter()
            .map(|entry| entry.message.as_str())
            .collect();
        assert_eq!(messages, ["「こんにちは」", "１．はい", "２．いいえ"]);
        let rebuilt = catalog.inject(catalog.entries()).unwrap();
        assert_eq!(rebuilt.plain, source.as_ref());
    }

    #[test]
    fn keeps_terminal_wait_in_source_when_translated_message_changes() {
        let source = "*define\n*start\n「こんにちは」\\\nend\n";
        let (source, _, errors) = SHIFT_JIS.encode(source);
        assert!(!errors);
        let catalog = ScriptCatalog::parse(&source).unwrap();
        let mut entries = catalog.entries().to_vec();
        entries[0].message = "「訳文」".into();
        let rebuilt = catalog.inject(&entries).unwrap();
        let decoded = decode_shift_jis(&rebuilt.plain, "test").unwrap();
        assert!(decoded.contains("「訳文」\\\n"));
    }

    #[test]
    fn allows_delay_edits_and_fullwidth_escapes_malformed_bang_text() {
        let source = "*define\n*start\n「待って!d500続く」@\nend\n";
        let (source, _, errors) = SHIFT_JIS.encode(source);
        assert!(!errors);
        let catalog = ScriptCatalog::parse(&source).unwrap();
        let mut entries = catalog.entries().to_vec();
        entries[0].message = "「待って続く」".into();
        let rebuilt = catalog.inject(&entries).unwrap();
        assert!(decode_shift_jis(&rebuilt.plain, "test")
            .unwrap()
            .contains("「待って続く」@"));
        entries[0].message = "「待って!d続く」".into();
        let escaped = catalog.inject(&entries).unwrap();
        assert_eq!(escaped.entries[0].message, "「待って！d続く」");
        entries[0].message = "无法编码🙂".into();
        assert!(catalog.inject(&entries).unwrap_err().contains("不能编码"));
    }

    #[test]
    fn fullwidth_normalizes_literal_ascii_reserved_punctuation() {
        let source = "*define\n*start\n原文。\\\nend\n";
        let (source, _, errors) = SHIFT_JIS.encode(source);
        assert!(!errors);
        let catalog = ScriptCatalog::parse(&source).unwrap();
        let mut entries = catalog.entries().to_vec();
        entries[0].message = "割合１００%，括弧(説明)".into();
        let injection = catalog.inject(&entries).unwrap();
        assert_eq!(injection.normalized_literals, 3);
        assert_eq!(injection.entries[0].message, "割合１００％，括弧（説明）");
        let decoded = decode_shift_jis(&injection.plain, "test").unwrap();
        assert!(decoded.contains("割合１００％，括弧（説明）\\\n"));
    }
}
