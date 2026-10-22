use super::model::{TextControl, TextEntry};
use std::collections::HashSet;

const SIMPLE_CONTROLS: &str = "BbLlUuRrDd*+";

#[derive(Default)]
struct SourceLine {
    raw: String,
    clean: String,
    controls: Vec<TextControl>,
}

struct Segment {
    lines: Vec<SourceLine>,
    page_start: bool,
    boundary_after: String,
}

struct EntryBuilder {
    name: Option<String>,
    message: String,
    raw: String,
    controls: Vec<TextControl>,
    first_line: u32,
}

pub fn extract_records(
    file: &str,
    block: u32,
    string_index: u32,
    string_offset: u64,
    source: &str,
    inline_speakers: &HashSet<String>,
) -> (Vec<TextEntry>, Vec<String>) {
    let (segments, warnings) = parse_segments(source);
    let mut speakers = inline_speakers.clone();
    speakers.extend(collect_explicit_speakers_from_segments(&segments));
    let mut entries = Vec::new();
    let mut page = 0u64;
    let mut record_index = 0u32;
    for segment in segments {
        if segment.page_start {
            page += 1;
        }
        let start = entries.len();
        let mut builders = split_lines(segment.lines, &speakers);
        for builder in builders.drain(..) {
            if builder.message.is_empty() {
                continue;
            }
            let kind = if builder.name.is_some() {
                "dialogue"
            } else {
                "text"
            };
            let name = builder.name.filter(|value| !value.is_empty());
            entries.push(TextEntry {
                _file: file.into(),
                _index: 0,
                _type: kind.into(),
                _block: Some(block),
                _string_index: Some(string_index),
                _string_offset: Some(string_offset),
                _record_index: Some(record_index),
                _offset: None,
                _size: None,
                _container_offset: None,
                _group: None,
                _item_index: None,
                _page: Some(page),
                _boundary_before: None,
                _boundary_after: None,
                _source_raw: Some(builder.raw),
                _controls: builder
                    .controls
                    .into_iter()
                    .map(|mut control| {
                        control.line -= builder.first_line;
                        control
                    })
                    .collect(),
                _scr_name: name.clone(),
                name,
                scr_msg: builder.message.clone(),
                message: builder.message,
            });
            record_index += 1;
        }
        if start < entries.len() {
            if segment.page_start {
                entries[start]._boundary_before = Some("page".into());
            }
            if segment.boundary_after != "end" {
                entries
                    .last_mut()
                    .expect("segment has records")
                    ._boundary_after = Some(segment.boundary_after);
            }
        }
    }
    (entries, warnings)
}

pub fn collect_explicit_speakers(source: &str) -> HashSet<String> {
    let (segments, _) = parse_segments(source);
    collect_explicit_speakers_from_segments(&segments)
}

fn collect_explicit_speakers_from_segments(segments: &[Segment]) -> HashSet<String> {
    let mut speakers = HashSet::new();
    for line in segments.iter().flat_map(|segment| &segment.lines) {
        let Some(tab) = line.clean.find('\t') else {
            continue;
        };
        let candidate = line.clean[..tab].trim_matches([' ', '\u{3000}']);
        let body = compact(&line.clean[tab + 1..]);
        if !candidate.is_empty() && body.starts_with(['「', '『']) {
            speakers.insert(candidate.into());
        }
    }
    speakers
}

fn parse_segments(source: &str) -> (Vec<Segment>, Vec<String>) {
    let chars = source.chars().collect::<Vec<_>>();
    let mut pos = 0usize;
    let mut segments = Vec::new();
    let mut lines = Vec::new();
    let mut line = SourceLine::default();
    let mut page_start = false;
    let mut warnings = Vec::new();

    let finish_line = |line: &mut SourceLine, lines: &mut Vec<SourceLine>| {
        if !line.raw.is_empty() || !line.clean.is_empty() || !line.controls.is_empty() {
            lines.push(std::mem::take(line));
        }
    };
    let finish_segment = |boundary: &str,
                          page_start_value: bool,
                          line: &mut SourceLine,
                          lines: &mut Vec<SourceLine>,
                          segments: &mut Vec<Segment>| {
        finish_line(line, lines);
        if !lines.is_empty() {
            segments.push(Segment {
                lines: std::mem::take(lines),
                page_start: page_start_value,
                boundary_after: boundary.into(),
            });
        }
    };

    while pos < chars.len() {
        let ch = chars[pos];
        if ch == '\r' {
            line.raw.push(ch);
            pos += 1;
            continue;
        }
        if ch == '\n' {
            line.raw.push(ch);
            finish_line(&mut line, &mut lines);
            pos += 1;
            continue;
        }
        if ch == '_' {
            let at = line.clean.chars().count() as u32;
            line.raw.push(ch);
            line.controls.push(TextControl {
                line: lines.len() as u32,
                at,
                raw: "_".into(),
            });
            pos += 1;
            continue;
        }
        if ch != '\\' {
            line.raw.push(ch);
            if ch >= ' ' || ch == '\t' {
                line.clean.push(ch);
            }
            pos += 1;
            continue;
        }
        if pos + 1 >= chars.len() {
            line.raw.push('\\');
            line.clean.push('\\');
            warnings.push("字符串末尾存在单独的反斜杠，按字面量保留".into());
            break;
        }

        let command = chars[pos + 1];
        if command == '/' || matches!(command, 'k' | 'K') {
            let boundary = if command == '/' { "page" } else { "wait" };
            finish_segment(boundary, page_start, &mut line, &mut lines, &mut segments);
            page_start = command == '/';
            pos += 2;
            continue;
        }

        let at = line.clean.chars().count() as u32;
        let mut raw = String::from("\\");
        raw.push(command);
        line.raw.push('\\');
        line.raw.push(command);
        pos += 2;
        if command == '\\'
            && pos < chars.len()
            && matches!(chars[pos], 'w' | 'W' | 'p' | 'P' | 'C' | 'c' | 'S' | 's')
        {
            let escaped_command = chars[pos];
            raw.push(escaped_command);
            line.raw.push(escaped_command);
            pos += 1;
            if matches!(escaped_command, 'w' | 'W' | 'p' | 'P') {
                while pos < chars.len() && chars[pos].is_ascii_digit() {
                    raw.push(chars[pos]);
                    line.raw.push(chars[pos]);
                    pos += 1;
                }
            } else if pos < chars.len() && chars[pos].is_ascii_hexdigit() {
                raw.push(chars[pos]);
                line.raw.push(chars[pos]);
                pos += 1;
            }
            while pos < chars.len() && chars[pos] == '_' {
                raw.push('_');
                line.raw.push('_');
                pos += 1;
            }
            line.controls.push(TextControl {
                line: lines.len() as u32,
                at,
                raw,
            });
            continue;
        }
        match command {
            '\\' => line.clean.push('\\'),
            '-' => line.clean.push('ー'),
            'w' | 'W' | 'p' | 'P' => {
                while pos < chars.len() && chars[pos].is_ascii_digit() {
                    raw.push(chars[pos]);
                    line.raw.push(chars[pos]);
                    pos += 1;
                }
            }
            'C' | 'c' | 'S' | 's' => {
                if pos < chars.len() && chars[pos].is_ascii_hexdigit() {
                    raw.push(chars[pos]);
                    line.raw.push(chars[pos]);
                    pos += 1;
                }
            }
            _ if SIMPLE_CONTROLS.contains(command) => {}
            _ if command.is_ascii_digit() => {
                while pos < chars.len() && chars[pos].is_ascii_digit() {
                    raw.push(chars[pos]);
                    line.raw.push(chars[pos]);
                    pos += 1;
                }
                warnings.push(format!(
                    "疑似漏写命令字母的计时控制 {raw}，净化文本按控制符移除"
                ));
            }
            _ => {
                line.clean.push(command);
                warnings.push(format!("未知控制符 \\{command}，净化文本保留命令字符"));
            }
        }
        while pos < chars.len() && chars[pos] == '_' {
            raw.push('_');
            line.raw.push('_');
            pos += 1;
        }
        line.controls.push(TextControl {
            line: lines.len() as u32,
            at,
            raw,
        });
    }
    finish_segment("end", page_start, &mut line, &mut lines, &mut segments);
    (segments, warnings)
}

fn split_lines(lines: Vec<SourceLine>, inline_speakers: &HashSet<String>) -> Vec<EntryBuilder> {
    let mut entries = Vec::new();
    let mut active: Option<EntryBuilder> = None;
    for (line_number, line) in lines.into_iter().enumerate() {
        let Some((explicit_name, name, body)) = classify_line(&line.clean, inline_speakers) else {
            if let Some(entry) = active.as_mut() {
                entry.raw.push_str(&line.raw);
                entry.controls.extend(line.controls);
            }
            continue;
        };
        if explicit_name {
            if let Some(entry) = active.take() {
                entries.push(entry);
            }
        }
        if active.is_none() {
            active = Some(EntryBuilder {
                name,
                message: String::new(),
                raw: String::new(),
                controls: Vec::new(),
                first_line: line_number as u32,
            });
        }
        let entry = active.as_mut().expect("active entry");
        entry.message.push_str(&body);
        entry.raw.push_str(&line.raw);
        entry.controls.extend(line.controls);
    }
    if let Some(entry) = active {
        entries.push(entry);
    }
    entries
}

fn classify_line(
    line: &str,
    inline_speakers: &HashSet<String>,
) -> Option<(bool, Option<String>, String)> {
    if let Some(tab) = line.find('\t') {
        let prefix = line[..tab].trim_matches([' ', '\u{3000}']);
        let body = compact(&line[tab + 1..]);
        if body.is_empty() {
            return None;
        }
        if prefix.is_empty() {
            return Some((false, None, body));
        }
        return Some((true, Some(prefix.into()), body));
    }

    let compacted = compact(line);
    if compacted.is_empty() {
        return None;
    }
    if let Some(quote) = compacted.find(['「', '『']) {
        let candidate = compacted[..quote].trim_matches([' ', '\u{3000}']);
        if !candidate.is_empty()
            && candidate.chars().count() <= 16
            && !candidate.chars().any(char::is_whitespace)
            && inline_speakers.contains(candidate)
        {
            return Some((true, Some(candidate.into()), compacted[quote..].into()));
        }
    }
    Some((false, None, compacted))
}

fn compact(value: &str) -> String {
    value
        .replace('\t', "")
        .trim_matches([' ', '\u{3000}', '\r', '\n'])
        .to_string()
}

pub fn render_edited_record(
    source_raw: &str,
    source_name: Option<&str>,
    new_name: Option<&str>,
    source_message: &str,
    new_message: &str,
) -> Result<String, String> {
    if source_name == new_name && source_message == new_message {
        return Ok(source_raw.into());
    }
    validate_edit("message", new_message)?;
    match (source_name, new_name) {
        (Some(_), Some(name)) if !name.is_empty() => validate_edit("name", name)?,
        (Some(_), _) => return Err("带姓名的原文记录必须保留非空 name 字段".into()),
        (None, Some(_)) => return Err("无姓名的原文记录不能新增 name 字段".into()),
        (None, None) => {}
    }

    let (segments, _) = parse_segments(source_raw);
    if segments.len() != 1 {
        return Err("记录原始文本含意外的分页或等待边界".into());
    }
    let lines = &segments[0].lines;
    let mut prefix_controls = Vec::new();
    let mut body_controls: Vec<(usize, String)> = Vec::new();
    let mut reconstructed_source = String::new();
    let mut prior_body_chars = 0usize;
    let inline_name = source_name.is_some()
        && lines
            .first()
            .and_then(|line| line.clean.find('\t'))
            .is_none();

    for (line_index, line) in lines.iter().enumerate() {
        let clean_chars = line.clean.chars().collect::<Vec<_>>();
        let body_start = line
            .clean
            .find('\t')
            .map(|byte| line.clean[..byte].chars().count() + 1)
            .or_else(|| {
                if line_index == 0 && inline_name {
                    let quote = line.clean.find(['「', '『'])?;
                    let name = source_name?;
                    (line.clean[..quote].trim() == name)
                        .then_some(line.clean[..quote].chars().count())
                } else {
                    None
                }
            })
            .unwrap_or(0);
        let body = compact(&clean_chars[body_start..].iter().collect::<String>());
        reconstructed_source.push_str(&body);
        for control in &line.controls {
            if control.raw == "\\-" || control.raw == "\\\\" {
                continue;
            }
            let normalized = normalize_control(&control.raw);
            if line_index == 0 && control.at as usize <= body_start {
                prefix_controls.push(normalized);
                continue;
            }
            let relative = (control.at as usize).saturating_sub(body_start);
            let before = clean_chars[body_start..clean_chars.len().min(body_start + relative)]
                .iter()
                .collect::<String>();
            let anchor = prior_body_chars + compact(&before).chars().count();
            body_controls.push((
                anchor.min(prior_body_chars + body.chars().count()),
                normalized,
            ));
        }
        prior_body_chars += body.chars().count();
    }
    if reconstructed_source != source_message {
        return Err(format!(
            "记录原始文本净化结果与 scr_msg 不一致：得到 {:?}，预期 {:?}",
            reconstructed_source, source_message
        ));
    }

    let source_len = source_message.chars().count();
    let translated = new_message.chars().collect::<Vec<_>>();
    let translated_len = translated.len();
    let mut mapped = body_controls
        .into_iter()
        .enumerate()
        .map(|(order, (anchor, raw))| {
            let position = if source_len == 0 {
                0
            } else {
                anchor
                    .saturating_mul(translated_len)
                    .saturating_add(source_len / 2)
                    / source_len
            };
            (position.min(translated_len), order, raw)
        })
        .collect::<Vec<_>>();
    mapped.sort_by_key(|(position, order, _)| (*position, *order));

    let mut result = prefix_controls.concat();
    if let Some(name) = new_name {
        result.push_str(name);
        if !inline_name {
            result.push('\t');
        }
    }
    let mut control_index = 0usize;
    for position in 0..=translated_len {
        while control_index < mapped.len() && mapped[control_index].0 == position {
            result.push_str(&mapped[control_index].2);
            control_index += 1;
        }
        if let Some(ch) = translated.get(position) {
            result.push(*ch);
        }
    }
    let trailing = source_raw
        .chars()
        .rev()
        .take_while(|ch| matches!(ch, '\r' | '\n'))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    result.push_str(&trailing);
    Ok(result)
}

fn normalize_control(raw: &str) -> String {
    let chars = raw.chars().collect::<Vec<_>>();
    if chars.len() >= 3
        && chars[0] == '\\'
        && chars[1] == '\\'
        && matches!(chars[2], 'w' | 'W' | 'p' | 'P' | 'C' | 'c' | 'S' | 's')
    {
        chars[1..].iter().collect()
    } else {
        raw.into()
    }
}

fn validate_edit(field: &str, value: &str) -> Result<(), String> {
    if let Some(ch) = value
        .chars()
        .find(|ch| matches!(ch, '\0' | '\r' | '\n' | '\\' | '_'))
    {
        return Err(format!(
            "{field} 含不允许的结构字符 {:?}；换行和控制符由工具元数据管理",
            ch
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{extract_records, render_edited_record};
    use std::collections::HashSet;

    #[test]
    fn repeated_named_lines_are_independent_records() {
        let source = "\\/謎の声\t「おや。\\p15_もう、見つかってしまいましたか。」\\p40_\n謎の声\t「あははははっ☆鬼ゴッコぉ☆」\\p40_\n謎の声\t「ちぇっ。\\p15_ラクラク逃げきれると思ったんやけどな。」\\k\n\\/少女\t「次」";
        let (entries, warnings) =
            extract_records("SCENARIO.GDM", 1, 0, 0x90B, source, &HashSet::new());
        assert!(warnings.is_empty());
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].name.as_deref(), Some("謎の声"));
        assert_eq!(
            entries[0].message,
            "「おや。もう、見つかってしまいましたか。」"
        );
        assert_eq!(entries[1].name.as_deref(), Some("謎の声"));
        assert_eq!(entries[2]._boundary_after.as_deref(), Some("wait"));
        assert_eq!(entries[3]._boundary_before.as_deref(), Some("page"));
    }

    #[test]
    fn wrapped_phone_call_stays_one_record() {
        let source = "\\/\\S0\\D\\w0主人公\t「もしもし、おれだけど\n\t  あ、\\p15_それでさ・・・・うん、そう・・・。\n\t  ・・・えっ\\p15_おまえも予定あるの？」\\k\n";
        let (entries, warnings) =
            extract_records("SCENARIO.GDM", 1, 3, 0xBEC, source, &HashSet::new());
        assert!(warnings.is_empty());
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name.as_deref(), Some("主人公"));
        assert_eq!(
            entries[0].message,
            "「もしもし、おれだけどあ、それでさ・・・・うん、そう・・・。・・・えっおまえも予定あるの？」"
        );
    }

    #[test]
    fn escaped_and_malformed_timing_controls_do_not_leak_into_message() {
        let source = "\\/\\w0「\\w10・・・\\\\p20_\\w2はい\\8_・・・\\w0」\\k";
        let (entries, warnings) =
            extract_records("SCENARIO.GDM", 29, 1, 0xC31, source, &HashSet::new());
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].message, "「・・・はい・・・」");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("\\8"));
    }

    #[test]
    fn prose_before_quote_is_not_invented_as_a_speaker() {
        let source = "\\/まさに「そうだ。」\\k";
        let (entries, _) = extract_records("SCENARIO.GDM", 1, 1, 0, source, &HashSet::new());
        assert_eq!(entries.len(), 1);
        assert!(entries[0].name.is_none());
        assert_eq!(entries[0].message, "まさに「そうだ。」");
    }

    #[test]
    fn edited_record_preserves_timing_and_line_separator() {
        let source = "謎の声\t「おや。\\p15_もう。」\\p40_\n";
        let rendered = render_edited_record(
            source,
            Some("謎の声"),
            Some("声音"),
            "「おや。もう。」",
            "「你好。找到了。」",
        )
        .expect("render");
        assert!(rendered.starts_with("声音\t"));
        assert!(rendered.contains("\\p15_"));
        assert!(rendered.ends_with("\\p40_\n"));
    }

    #[test]
    fn edited_inline_name_preserves_inline_separator() {
        let source = "\\S0\\D\\w0リュシィ「\\w10・・・\\w0うん！」";
        let rendered = render_edited_record(
            source,
            Some("リュシィ"),
            Some("露希"),
            "「・・・うん！」",
            "「你好！」",
        )
        .expect("render inline");
        assert!(rendered.starts_with("\\S0\\D\\w0露希"));
        assert!(!rendered.contains("露希\t"));
    }
}
