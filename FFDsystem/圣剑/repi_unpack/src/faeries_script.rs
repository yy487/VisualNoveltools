use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    Message,
    Choice,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub kind: EntryKind,
    /// `-message id` or `-case to`, retained as read-only context.
    pub name: Option<String>,
    /// Message body or choice display text, normalized to LF for JSON.
    pub text: String,
    /// Byte range of the body or the quoted `-case text` value.
    pub text_range: Range<usize>,
}

#[derive(Clone, Debug)]
pub struct Script {
    pub entries: Vec<Entry>,
    pub message_count: usize,
    pub choice_count: usize,
    line_ending: Vec<u8>,
}

struct Attribute {
    value: Range<usize>,
    quoted: bool,
}

#[derive(Clone, Copy)]
struct Line {
    start: usize,
    content_end: usize,
    full_end: usize,
}

pub fn parse(bytes: &[u8], source: &str) -> Result<Script, String> {
    let lines = split_lines(bytes);
    let mut entries = Vec::new();
    let mut preferred_eol = None;
    let mut crlf_count = 0usize;
    let mut lf_count = 0usize;

    for line in &lines {
        if line.full_end > line.content_end {
            if line.full_end - line.content_end == 2 {
                crlf_count += 1;
                preferred_eol.get_or_insert_with(|| b"\r\n".to_vec());
            } else {
                lf_count += 1;
                preferred_eol.get_or_insert_with(|| b"\n".to_vec());
            }
        }
    }
    let line_ending = if crlf_count > lf_count {
        b"\r\n".to_vec()
    } else {
        preferred_eol.unwrap_or_else(|| b"\r\n".to_vec())
    };

    let mut index = 0usize;
    let mut in_select = false;
    while index < lines.len() {
        let line = lines[index];
        let header = &bytes[line.start..line.content_end];
        let trimmed = trim_ascii_space(header);
        if is_select_directive(trimmed) {
            in_select = true;
            index += 1;
            continue;
        }

        if in_select && (trimmed.is_empty() || trimmed.starts_with(b"//")) {
            index += 1;
            continue;
        }
        if in_select && is_case_directive(trimmed) {
            entries.push(parse_choice(bytes, line, source, index + 1)?);
            index += 1;
            continue;
        }
        if in_select {
            in_select = false;
        }
        if !is_message_directive(header) {
            index += 1;
            continue;
        }
        if line.full_end == line.content_end {
            return Err(format!("{source}: -message 指令缺少消息正文行"));
        }

        let name = parse_message_name(header, source, index + 1)?;
        let body_start = line.full_end;
        let mut terminator = None;
        let mut scan = index + 1;
        while scan < lines.len() {
            let candidate = lines[scan];
            let text = trim_ascii_space(&bytes[candidate.start..candidate.content_end]);
            if text == b"\\" {
                terminator = Some((scan, candidate.start));
                break;
            }
            scan += 1;
        }
        let Some((terminator_line, terminator_start)) = terminator else {
            return Err(format!(
                "{source}: 第 {} 行 -message 没有独立的反斜线结束符",
                index + 1
            ));
        };

        // The line break immediately before the standalone terminator is syntax,
        // not part of the editable body. Interior line breaks remain in the text.
        let mut body_end = terminator_start;
        if body_end > body_start && bytes[body_end - 1] == b'\n' {
            body_end -= 1;
            if body_end > body_start && bytes[body_end - 1] == b'\r' {
                body_end -= 1;
            }
        }
        if body_end < body_start {
            return Err(format!(
                "{source}: 第 {} 行 -message 正文范围无效",
                index + 1
            ));
        }

        let raw_text = &bytes[body_start..body_end];
        let text = normalize_newlines(&decode_cp932(raw_text).map_err(|error| {
            format!(
                "{source}: 第 {} 行消息正文无法按 CP932 解码: {error}",
                index + 1
            )
        })?);
        entries.push(Entry {
            kind: EntryKind::Message,
            name,
            text,
            text_range: body_start..body_end,
        });
        index = terminator_line + 1;
    }

    let message_count = entries
        .iter()
        .filter(|entry| entry.kind == EntryKind::Message)
        .count();
    let choice_count = entries.len() - message_count;
    Ok(Script {
        entries,
        message_count,
        choice_count,
        line_ending,
    })
}

/// Replace message bodies and choice display strings only. Branch targets,
/// command attributes, surrounding commands, and terminators remain untouched.
pub fn rewrite(bytes: &[u8], script: &Script, translated: &[String]) -> Result<Vec<u8>, String> {
    if translated.len() != script.entries.len() {
        return Err(format!(
            "译文条目数为 {}，脚本文本记录数为 {}",
            translated.len(),
            script.entries.len()
        ));
    }

    let mut output = Vec::with_capacity(bytes.len());
    let mut cursor = 0usize;
    for (entry, translated_text) in script.entries.iter().zip(translated) {
        if entry.text_range.start < cursor || entry.text_range.end > bytes.len() {
            return Err("文本定位重叠或超出脚本范围".into());
        }
        output.extend_from_slice(&bytes[cursor..entry.text_range.start]);
        if translated_text == &entry.text {
            // Avoid normalizing untouched CP932 bytes or mixed line endings.
            output.extend_from_slice(&bytes[entry.text_range.clone()]);
        } else {
            let encoded = match entry.kind {
                EntryKind::Message => encode_message(translated_text, &script.line_ending)?,
                EntryKind::Choice => encode_choice(translated_text)?,
            };
            output.extend_from_slice(&encoded);
        }
        cursor = entry.text_range.end;
    }
    output.extend_from_slice(&bytes[cursor..]);
    Ok(output)
}

fn split_lines(bytes: &[u8]) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut start = 0usize;
    while start < bytes.len() {
        let Some(relative_end) = bytes[start..].iter().position(|byte| *byte == b'\n') else {
            lines.push(Line {
                start,
                content_end: bytes.len(),
                full_end: bytes.len(),
            });
            break;
        };
        let newline = start + relative_end;
        let content_end = if newline > start && bytes[newline - 1] == b'\r' {
            newline - 1
        } else {
            newline
        };
        lines.push(Line {
            start,
            content_end,
            full_end: newline + 1,
        });
        start = newline + 1;
    }
    lines
}

fn is_message_directive(line: &[u8]) -> bool {
    has_command(line, b"-message")
}

fn is_case_directive(line: &[u8]) -> bool {
    has_command(line, b"-case")
}

fn is_select_directive(line: &[u8]) -> bool {
    has_command(line, b"@select")
}

fn has_command(line: &[u8], command: &[u8]) -> bool {
    let line = trim_ascii_space(line);
    line.strip_prefix(command).is_some_and(|rest| {
        rest.first()
            .is_none_or(|byte| byte.is_ascii_whitespace() || *byte == b'=')
    })
}

fn parse_message_name(
    header: &[u8],
    source: &str,
    line_number: usize,
) -> Result<Option<String>, String> {
    let Some(attribute) = find_attribute(header, b"-message", b"id", source, line_number)? else {
        return Ok(None);
    };
    let value = &header[attribute.value];
    decode_cp932(value)
        .map(Some)
        .map_err(|error| format!("{source}: 第 {line_number} 行 id 属性无法按 CP932 解码: {error}"))
}

fn parse_choice(
    bytes: &[u8],
    line: Line,
    source: &str,
    line_number: usize,
) -> Result<Entry, String> {
    let header = &bytes[line.start..line.content_end];
    let text = find_attribute(header, b"-case", b"text", source, line_number)?
        .ok_or_else(|| format!("{source}: 第 {line_number} 行 -case 缺少 text 属性"))?;
    let target = find_attribute(header, b"-case", b"to", source, line_number)?
        .ok_or_else(|| format!("{source}: 第 {line_number} 行 -case 缺少 to 属性"))?;
    if !text.quoted || !target.quoted {
        return Err(format!(
            "{source}: 第 {line_number} 行 -case 的 text/to 属性必须使用引号"
        ));
    }
    let raw_text = &header[text.value.clone()];
    let raw_target = &header[target.value.clone()];
    if raw_text.contains(&b'\\') || raw_target.contains(&b'\\') {
        return Err(format!(
            "{source}: 第 {line_number} 行 -case 含有未确认的反斜线转义"
        ));
    }
    let text_range = (line.start + text.value.start)..(line.start + text.value.end);
    let display_text = normalize_newlines(&decode_cp932(raw_text).map_err(|error| {
        format!("{source}: 第 {line_number} 行选项文字无法按 CP932 解码: {error}")
    })?);
    decode_cp932(raw_target).map_err(|error| {
        format!("{source}: 第 {line_number} 行分支目标无法按 CP932 解码: {error}")
    })?;
    Ok(Entry {
        kind: EntryKind::Choice,
        // A choice has no message id; its `to` value is structural and stays
        // in the source instead of being duplicated into the translation view.
        name: None,
        text: display_text,
        text_range,
    })
}

fn find_attribute(
    line: &[u8],
    directive: &[u8],
    wanted: &[u8],
    source: &str,
    line_number: usize,
) -> Result<Option<Attribute>, String> {
    let mut leading = 0usize;
    while leading < line.len() && line[leading].is_ascii_whitespace() {
        leading += 1;
    }
    let trimmed = &line[leading..];
    let rest = trimmed.strip_prefix(directive).ok_or_else(|| {
        format!(
            "{source}: 第 {line_number} 行缺少预期指令 {}",
            String::from_utf8_lossy(directive)
        )
    })?;
    let rest_offset = leading + directive.len();
    let mut cursor = 0usize;
    while cursor < rest.len() {
        while cursor < rest.len() && (rest[cursor].is_ascii_whitespace() || rest[cursor] == b',') {
            cursor += 1;
        }
        if cursor >= rest.len() {
            break;
        }

        let key_start = cursor;
        while cursor < rest.len()
            && rest[cursor] != b'='
            && !rest[cursor].is_ascii_whitespace()
            && rest[cursor] != b','
        {
            cursor += if is_sjis_lead(rest[cursor]) { 2 } else { 1 };
            cursor = cursor.min(rest.len());
        }
        let key_end = cursor;
        while cursor < rest.len() && rest[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= rest.len() || rest[cursor] != b'=' {
            while cursor < rest.len() && rest[cursor] != b',' {
                cursor += if is_sjis_lead(rest[cursor]) { 2 } else { 1 };
                cursor = cursor.min(rest.len());
            }
            continue;
        }
        let matches = &rest[key_start..key_end] == wanted;
        cursor += 1;
        while cursor < rest.len() && rest[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor < rest.len() && rest[cursor] == b'"' {
            cursor += 1;
            let value_start = cursor;
            let mut closed = false;
            while cursor < rest.len() {
                if is_sjis_lead(rest[cursor]) {
                    cursor = (cursor + 2).min(rest.len());
                } else if rest[cursor] == b'\\' && cursor + 1 < rest.len() {
                    cursor += 2;
                } else if rest[cursor] == b'"' {
                    closed = true;
                    break;
                } else {
                    cursor += 1;
                }
            }
            if !closed {
                return Err(format!("{source}: 第 {line_number} 行属性引号未闭合"));
            }
            if matches {
                return Ok(Some(Attribute {
                    value: (rest_offset + value_start)..(rest_offset + cursor),
                    quoted: true,
                }));
            }
            cursor += 1;
        } else {
            let value_start = cursor;
            while cursor < rest.len() && rest[cursor] != b',' && !rest[cursor].is_ascii_whitespace()
            {
                cursor += if is_sjis_lead(rest[cursor]) { 2 } else { 1 };
                cursor = cursor.min(rest.len());
            }
            if matches {
                return Ok(Some(Attribute {
                    value: (rest_offset + value_start)..(rest_offset + cursor),
                    quoted: false,
                }));
            }
        }
    }
    Ok(None)
}

fn encode_message(text: &str, line_ending: &[u8]) -> Result<Vec<u8>, String> {
    if text.contains('\0') {
        return Err("message 不允许包含 NUL 字符".into());
    }
    let normalized = normalize_newlines(text);
    if normalized
        .split('\n')
        .any(|line| trim_unicode_space(line) == "\\")
    {
        return Err("message 不能包含独占一行的反斜线（脚本消息结束符）".into());
    }

    let mut output = Vec::new();
    for (index, line) in normalized.split('\n').enumerate() {
        if index != 0 {
            output.extend_from_slice(line_ending);
        }
        output.extend_from_slice(
            &encode_cp932(line)
                .map_err(|error| format!("message 含有无法编码为 CP932 的字符: {error}"))?,
        );
    }
    Ok(output)
}

fn encode_choice(text: &str) -> Result<Vec<u8>, String> {
    if text.contains('\0') || text.contains(['\r', '\n', '"', '\\']) {
        return Err("选项文字不能包含 NUL、换行、双引号或反斜线（-case 属性语法）".into());
    }
    encode_cp932(text).map_err(|error| format!("选项文字含有无法编码为 CP932 的字符: {error}"))
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn trim_ascii_space(mut bytes: &[u8]) -> &[u8] {
    while bytes.first().is_some_and(u8::is_ascii_whitespace) {
        bytes = &bytes[1..];
    }
    while bytes.last().is_some_and(u8::is_ascii_whitespace) {
        bytes = &bytes[..bytes.len() - 1];
    }
    bytes
}

fn trim_unicode_space(text: &str) -> &str {
    text.trim_matches(char::is_whitespace)
}

fn is_sjis_lead(byte: u8) -> bool {
    (0x81..=0x9f).contains(&byte) || (0xe0..=0xfc).contains(&byte)
}

fn decode_cp932(bytes: &[u8]) -> Result<String, String> {
    if bytes.is_empty() {
        return Ok(String::new());
    }
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn MultiByteToWideChar(
                code_page: u32,
                flags: u32,
                source: *const i8,
                source_len: i32,
                destination: *mut u16,
                destination_len: i32,
            ) -> i32;
        }
        let source_len = i32::try_from(bytes.len()).map_err(|_| "CP932 输入过长")?;
        let required = unsafe {
            MultiByteToWideChar(
                932,
                8, // MB_ERR_INVALID_CHARS: reject damaged source text.
                bytes.as_ptr() as *const i8,
                source_len,
                std::ptr::null_mut(),
                0,
            )
        };
        if required <= 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let mut wide = vec![0u16; required as usize];
        let written = unsafe {
            MultiByteToWideChar(
                932,
                8,
                bytes.as_ptr() as *const i8,
                source_len,
                wide.as_mut_ptr(),
                required,
            )
        };
        if written != required {
            return Err(std::io::Error::last_os_error().to_string());
        }
        Ok(OsString::from_wide(&wide).to_string_lossy().into_owned())
    }
    #[cfg(not(windows))]
    {
        String::from_utf8(bytes.to_vec())
            .map_err(|error| format!("当前平台没有 CP932 解码器，且内容不是 UTF-8: {error}"))
    }
}

fn encode_cp932(text: &str) -> Result<Vec<u8>, String> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn WideCharToMultiByte(
                code_page: u32,
                flags: u32,
                source: *const u16,
                source_len: i32,
                destination: *mut i8,
                destination_len: i32,
                default_char: *const i8,
                used_default: *mut i32,
            ) -> i32;
        }
        let wide: Vec<u16> = std::ffi::OsStr::new(text).encode_wide().collect();
        let source_len = i32::try_from(wide.len()).map_err(|_| "message 太长，无法编码")?;
        let default_char = b"?"[0] as i8;
        let mut used_default = 0i32;
        let flags = 0x400; // WC_NO_BEST_FIT_CHARS.
        let required = unsafe {
            WideCharToMultiByte(
                932,
                flags,
                wide.as_ptr(),
                source_len,
                std::ptr::null_mut(),
                0,
                &default_char,
                &mut used_default,
            )
        };
        if required <= 0 || used_default != 0 {
            return Err("CP932 编码失败或需要替换字符".into());
        }
        let mut bytes = vec![0u8; required as usize];
        used_default = 0;
        let written = unsafe {
            WideCharToMultiByte(
                932,
                flags,
                wide.as_ptr(),
                source_len,
                bytes.as_mut_ptr() as *mut i8,
                required,
                &default_char,
                &mut used_default,
            )
        };
        if written != required || used_default != 0 {
            return Err("CP932 编码时发生了字符替换".into());
        }
        Ok(bytes)
    }
    #[cfg(not(windows))]
    {
        text.is_ascii()
            .then(|| text.as_bytes().to_vec())
            .ok_or_else(|| "当前平台没有 CP932 编码器".into())
    }
}
