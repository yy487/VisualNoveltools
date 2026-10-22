use encoding_rs::SHIFT_JIS;

use crate::error::{PmxError, Result};

const PUT_TEXT_PREFIX: &[u8] = b"PutText ";

#[derive(Debug)]
pub(crate) struct ParsedScript {
    pub entries: Vec<ScriptText>,
    pub warnings: Vec<ScriptWarning>,
}

#[derive(Clone, Debug)]
pub(crate) struct ScriptText {
    pub index: usize,
    pub line: usize,
    pub inst_offset: usize,
    pub name_offset: Option<usize>,
    pub name_end: Option<usize>,
    pub message_offset: usize,
    pub message_end: usize,
    pub name: Option<String>,
    pub message: String,
}

#[derive(Debug)]
pub(crate) struct ScriptWarning {
    pub line: usize,
    pub message: String,
}

pub(crate) fn parse_script(data: &[u8]) -> Result<ParsedScript> {
    let mut entries = Vec::new();
    let mut warnings = Vec::new();
    let mut line_start = 0usize;
    let mut line_number = 1usize;

    while line_start < data.len() {
        let newline = data[line_start..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|relative| line_start + relative);
        let physical_end = newline.unwrap_or(data.len());
        let content_end = if physical_end > line_start && data[physical_end - 1] == b'\r' {
            physical_end - 1
        } else {
            physical_end
        };

        parse_line(
            &data[line_start..content_end],
            line_start,
            line_number,
            &mut entries,
            &mut warnings,
        )?;

        match newline {
            Some(position) => {
                line_start = position + 1;
                line_number += 1;
            }
            None => break,
        }
    }

    Ok(ParsedScript { entries, warnings })
}

pub(crate) fn encode_cp932(text: &str) -> Result<Vec<u8>> {
    let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        let mut unencodable = Vec::new();
        for character in text.chars() {
            let (_, _, character_error) = SHIFT_JIS.encode(&character.to_string());
            if character_error && !unencodable.contains(&character) {
                unencodable.push(character);
            }
        }
        let display: String = unencodable.iter().collect();
        return Err(PmxError::new(format!(
            "文本不能按 CP932 编码，不可编码字符：{display}"
        )));
    }
    Ok(encoded.into_owned())
}

fn parse_line(
    line: &[u8],
    line_offset: usize,
    line_number: usize,
    entries: &mut Vec<ScriptText>,
    warnings: &mut Vec<ScriptWarning>,
) -> Result<()> {
    if !line.starts_with(PUT_TEXT_PREFIX) {
        return Ok(());
    }

    let first_comma = line
        .iter()
        .position(|byte| *byte == b',')
        .ok_or_else(|| PmxError::new(format!("第 {line_number} 行 PutText 缺少第一个逗号")))?;
    let second_comma = line[first_comma + 1..]
        .iter()
        .position(|byte| *byte == b',')
        .map(|relative| first_comma + 1 + relative)
        .ok_or_else(|| PmxError::new(format!("第 {line_number} 行 PutText 缺少第二个逗号")))?;
    let payload_offset_in_line = second_comma + 1;
    let payload_bytes = &line[payload_offset_in_line..];
    if payload_bytes.is_empty() {
        return Err(PmxError::new(format!(
            "第 {line_number} 行 PutText 文本参数为空"
        )));
    }

    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(payload_bytes);
    if had_errors {
        return Err(PmxError::new(format!(
            "第 {line_number} 行 PutText 文本不能按 CP932 解码"
        )));
    }
    let payload = decoded.into_owned();
    let roundtrip = encode_cp932(&payload)?;
    if roundtrip != payload_bytes {
        return Err(PmxError::new(format!(
            "第 {line_number} 行 PutText 文本不能进行 CP932 字节精确往返"
        )));
    }

    let leading_controls_end = skip_leading_controls(&payload, 0);
    let mut name = None;
    let mut name_span = None;
    let mut message_start = leading_controls_end;
    let after_controls = &payload[leading_controls_end..];
    if let Some(without_open) = after_controls.strip_prefix('[') {
        if let Some(separator) = without_open.find("]\\n") {
            let parsed_name = &without_open[..separator];
            if parsed_name.is_empty() {
                warnings.push(ScriptWarning {
                    line: line_number,
                    message: "角色名为空，按无名文本处理".to_owned(),
                });
                message_start = 0;
            } else {
                name = Some(parsed_name.to_owned());
                name_span = Some((
                    leading_controls_end + 1,
                    leading_controls_end + 1 + separator,
                ));
                message_start = leading_controls_end + 1 + separator + 3;
            }
        } else {
            warnings.push(ScriptWarning {
                line: line_number,
                message: "文本以 [ 开头但缺少 ]\\n 分隔，按无名文本处理".to_owned(),
            });
            message_start = 0;
        }
    }

    message_start = skip_leading_controls(&payload, message_start);
    let message_end = trim_trailing_controls(&payload, message_start, payload.len());
    if message_start >= message_end {
        return Err(PmxError::new(format!(
            "第 {line_number} 行 PutText 去除控制码后正文为空"
        )));
    }

    let prefix_bytes = encode_cp932(&payload[..message_start])?;
    let message_bytes = encode_cp932(&payload[message_start..message_end])?;
    let payload_absolute = line_offset + payload_offset_in_line;
    let (name_offset, name_end) = match name_span {
        Some((start, end)) => {
            let prefix = encode_cp932(&payload[..start])?;
            let name_bytes = encode_cp932(&payload[start..end])?;
            let absolute_start = payload_absolute
                .checked_add(prefix.len())
                .ok_or_else(|| PmxError::new("PutText 角色名偏移溢出"))?;
            let absolute_end = absolute_start
                .checked_add(name_bytes.len())
                .ok_or_else(|| PmxError::new("PutText 角色名结尾溢出"))?;
            (Some(absolute_start), Some(absolute_end))
        }
        None => (None, None),
    };
    let message_offset = payload_absolute
        .checked_add(prefix_bytes.len())
        .ok_or_else(|| PmxError::new("PutText 正文偏移溢出"))?;
    let message_absolute_end = message_offset
        .checked_add(message_bytes.len())
        .ok_or_else(|| PmxError::new("PutText 正文结尾溢出"))?;

    if data_slice_mismatch(
        line,
        payload_offset_in_line + prefix_bytes.len(),
        &message_bytes,
    ) {
        return Err(PmxError::new(format!(
            "第 {line_number} 行 PutText 正文偏移校验失败"
        )));
    }

    entries.push(ScriptText {
        index: entries.len(),
        line: line_number,
        inst_offset: line_offset,
        name_offset,
        name_end,
        message_offset,
        message_end: message_absolute_end,
        name,
        message: payload[message_start..message_end].to_owned(),
    });
    Ok(())
}

fn skip_leading_controls(text: &str, mut offset: usize) -> usize {
    while let Some(length) = control_token_length(&text[offset..]) {
        offset += length;
    }
    offset
}

fn trim_trailing_controls(text: &str, minimum: usize, mut end: usize) -> usize {
    loop {
        let mut trailing = None;
        for (offset, _) in text[minimum..end].match_indices('\\') {
            let absolute = minimum + offset;
            if let Some(length) = control_token_length(&text[absolute..end]) {
                if absolute + length == end {
                    trailing = Some(absolute);
                }
            }
        }
        match trailing {
            Some(start) => end = start,
            None => return end,
        }
    }
}

fn control_token_length(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.first() != Some(&b'\\') {
        return None;
    }
    let mut offset = 1usize;
    while offset < bytes.len() && bytes[offset].is_ascii_alphabetic() {
        offset += 1;
    }
    if offset == 1 || bytes.get(offset) != Some(&b'(') {
        return None;
    }
    offset += 1;
    while offset < bytes.len() && bytes[offset] != b')' {
        if bytes[offset] == b'\r' || bytes[offset] == b'\n' {
            return None;
        }
        offset += 1;
    }
    (bytes.get(offset) == Some(&b')')).then_some(offset + 1)
}

fn data_slice_mismatch(data: &[u8], offset: usize, expected: &[u8]) -> bool {
    data.get(offset..offset + expected.len()) != Some(expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_named_put_text() {
        let data = encode_cp932("PutText 0,,[静　麻]\\n「……いや、無いんですけどね」\r\n").unwrap();
        let parsed = parse_script(&data).unwrap();

        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].name.as_deref(), Some("静　麻"));
        assert_eq!(parsed.entries[0].message, "「……いや、無いんですけどね」");
        assert_eq!(
            &data[parsed.entries[0].name_offset.unwrap()..parsed.entries[0].name_end.unwrap()],
            encode_cp932("静　麻").unwrap().as_slice()
        );
        assert_eq!(
            &data[parsed.entries[0].message_offset..parsed.entries[0].message_end],
            encode_cp932("「……いや、無いんですけどね」")
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn preserves_wrapper_controls_outside_message() {
        let data =
            encode_cp932("PutText 0,,\\i(235)[梨　花]\\n\\i(238)「きゃあ！」\\i(237)\r\n").unwrap();
        let parsed = parse_script(&data).unwrap();

        assert_eq!(parsed.entries[0].name.as_deref(), Some("梨　花"));
        assert_eq!(parsed.entries[0].message, "「きゃあ！」");
    }

    #[test]
    fn keeps_literal_newline_in_unnamed_message() {
        let data = b"PutText 0,,first\\nsecond\r\n";
        let parsed = parse_script(data).unwrap();

        assert_eq!(parsed.entries[0].name, None);
        assert_eq!(parsed.entries[0].message, "first\\nsecond");
    }

    #[test]
    fn bracket_without_separator_is_not_guessed_as_name() {
        let data = encode_cp932("PutText 0,,[　声　]「……」\r\n").unwrap();
        let parsed = parse_script(&data).unwrap();

        assert_eq!(parsed.entries[0].name, None);
        assert_eq!(parsed.entries[0].message, "[　声　]「……」");
        assert_eq!(parsed.warnings.len(), 1);
    }
}
