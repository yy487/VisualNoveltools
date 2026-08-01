use std::ops::Range;

use crate::encoding::cp932_len;
use crate::{ToolResult, error};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlotKind {
    Dialogue,
    Monologue,
    Choice,
    Name,
}

impl SlotKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dialogue => "dialogue",
            Self::Monologue => "monologue",
            Self::Choice => "choice",
            Self::Name => "name",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextSlot {
    pub index: usize,
    pub line: usize,
    pub message_range: Range<usize>,
    pub name_range: Option<Range<usize>>,
    pub scr_msg: String,
    pub scr_name: Option<String>,
    pub kind: SlotKind,
    pub rule: String,
    pub opcode: Option<String>,
    pub target: Option<String>,
    pub choice_index: Option<usize>,
    pub offset: u64,
    pub size: usize,
}

#[derive(Clone, Debug)]
pub struct ParsedScript {
    pub text: String,
    pub slots: Vec<TextSlot>,
}

#[derive(Debug)]
struct LocalSlot {
    message: Range<usize>,
    name: Option<Range<usize>>,
    kind: SlotKind,
    rule: String,
    opcode: Option<String>,
    target: Option<String>,
    choice_index: Option<usize>,
}

pub fn parse_script(text: String) -> ToolResult<ParsedScript> {
    if text.contains('\r') {
        return Err(error(
            "source contains CR characters; this format profile requires LF line endings",
        ));
    }
    if text.contains('\0') {
        return Err(error("source contains NUL"));
    }

    let mut slots = Vec::new();
    let mut utf8_base = 0usize;
    let mut cp932_base = 0usize;

    for (line_number, raw_line) in text.split_inclusive('\n').enumerate() {
        let line = raw_line.strip_suffix('\n').unwrap_or(raw_line);
        let local_slots = parse_line(line);
        for local in local_slots {
            let absolute_message =
                (utf8_base + local.message.start)..(utf8_base + local.message.end);
            let absolute_name = local
                .name
                .as_ref()
                .map(|span| (utf8_base + span.start)..(utf8_base + span.end));
            let scr_msg = line[local.message.clone()].to_owned();
            let scr_name = local
                .name
                .as_ref()
                .map(|span| line[span.clone()].to_owned());
            let offset = cp932_base + cp932_len(&line[..local.message.start])?;
            let size = cp932_len(&scr_msg)?;
            slots.push(TextSlot {
                index: slots.len(),
                line: line_number + 1,
                message_range: absolute_message,
                name_range: absolute_name,
                scr_msg,
                scr_name,
                kind: local.kind,
                rule: local.rule,
                opcode: local.opcode,
                target: local.target,
                choice_index: local.choice_index,
                offset: offset as u64,
                size,
            });
        }
        utf8_base += raw_line.len();
        cp932_base += cp932_len(raw_line)?;
    }

    Ok(ParsedScript { text, slots })
}

fn parse_line(line: &str) -> Vec<LocalSlot> {
    let choices = parse_choices(line);
    if !choices.is_empty() {
        return choices;
    }

    if line.is_empty() || is_skipped_line(line) {
        return Vec::new();
    }

    if let Some(slot) = parse_named_line(line) {
        return vec![slot];
    }
    if let Some(slot) = parse_font_color_line(line) {
        return vec![slot];
    }

    vec![LocalSlot {
        message: 0..line.len(),
        name: None,
        kind: SlotKind::Monologue,
        rule: "42".to_owned(),
        opcode: None,
        target: None,
        choice_index: None,
    }]
}

fn is_skipped_line(line: &str) -> bool {
    let Some(character) = line.trim_start_matches(char::is_whitespace).chars().next() else {
        return false;
    };
    character == ';'
        || character == '#'
        || character == '/'
        || character == '*'
        || character.is_ascii_alphabetic()
}

fn parse_named_line(line: &str) -> Option<LocalSlot> {
    let rest = line.strip_prefix('【')?;
    let close_in_rest = rest.find('】')?;
    let header = &rest[..close_in_rest];
    let header_start = '【'.len_utf8();
    let close = header_start + close_in_rest;
    let message_start = close + '】'.len_utf8();
    let has_message = message_start < line.len();

    let (rule, name_local) = if has_message {
        if let Some(position) = header.find("@,") {
            if position > 0 && position + 2 < header.len() {
                ("11", 0..position)
            } else {
                parse_named_with_message_fallback(header)?
            }
        } else if let Some(at) = header.find('@') {
            if at > 0 {
                if let Some(comma_after) = header[at + 1..].find(',') {
                    let comma = at + 1 + comma_after;
                    if comma > at + 1 && comma + 1 < header.len() {
                        ("12", (at + 1)..comma)
                    } else {
                        parse_named_with_message_fallback(header)?
                    }
                } else if header.ends_with('@') {
                    ("22", 0..(header.len() - 1))
                } else {
                    ("23", 0..header.len())
                }
            } else if header.ends_with('@') {
                ("22", 0..(header.len() - 1))
            } else {
                parse_named_with_message_fallback(header)?
            }
        } else {
            parse_named_with_message_fallback(header)?
        }
    } else if let Some(comma) = header.find(',') {
        if comma + 1 < header.len() {
            ("24", 0..comma)
        } else {
            ("25", 0..header.len())
        }
    } else {
        ("25", 0..header.len())
    };

    Some(LocalSlot {
        message: message_start..line.len(),
        name: Some((header_start + name_local.start)..(header_start + name_local.end)),
        kind: if has_message {
            SlotKind::Dialogue
        } else {
            SlotKind::Name
        },
        rule: rule.to_owned(),
        opcode: None,
        target: None,
        choice_index: None,
    })
}

fn parse_named_with_message_fallback(header: &str) -> Option<(&'static str, Range<usize>)> {
    if let Some(comma) = header.find(',')
        && comma + 1 < header.len()
    {
        return Some(("21", 0..comma));
    }
    Some(("23", 0..header.len()))
}

fn parse_font_color_line(line: &str) -> Option<LocalSlot> {
    let opening_start = line.starts_with("[setFontColor");
    let opening_end = opening_start
        .then(|| line.find("\\]"))
        .flatten()
        .map(|position| position + 2);
    let closing_start = line
        .rfind("[setFontColor")
        .filter(|position| line.ends_with(']') && *position > 0);

    let (rule, message) = match (opening_end, closing_start) {
        (Some(start), Some(end)) if start <= end => ("30", start..end),
        (Some(start), _) if start < line.len() => ("31", start..line.len()),
        (_, Some(end)) if end > 0 => ("32", 0..end),
        _ => return None,
    };

    Some(LocalSlot {
        message,
        name: None,
        kind: SlotKind::Monologue,
        rule: rule.to_owned(),
        opcode: None,
        target: None,
        choice_index: None,
    })
}

fn parse_choices(line: &str) -> Vec<LocalSlot> {
    let trimmed = line.trim_start_matches(char::is_whitespace);
    let opcode = if command_starts(trimmed, "SELECT_INIT") {
        Some("SELECT_INIT")
    } else if command_starts(trimmed, "SELECT") {
        Some("SELECT")
    } else if line.contains("g_select") && line.contains("$$") {
        Some("DYNAMIC_SELECT")
    } else {
        None
    };
    let Some(opcode) = opcode else {
        return Vec::new();
    };

    let mut result = Vec::new();
    for quoted in quoted_string_ranges(line) {
        let payload = &line[quoted.clone()];
        let Some(delimiter) = payload.rfind(",*") else {
            continue;
        };
        let target = &payload[delimiter + 1..];
        if target.len() < 2
            || target
                .chars()
                .any(|character| character == ',' || character == '"' || character.is_whitespace())
        {
            continue;
        }

        let choice_index = result.len();
        result.push(LocalSlot {
            message: quoted.start..(quoted.start + delimiter),
            name: None,
            kind: SlotKind::Choice,
            rule: if opcode == "DYNAMIC_SELECT" {
                "choice_dynamic".to_owned()
            } else {
                "choice_inline".to_owned()
            },
            opcode: Some(opcode.to_owned()),
            target: Some(target.to_owned()),
            choice_index: Some(choice_index),
        });
    }
    result
}

fn command_starts(line: &str, command: &str) -> bool {
    line.strip_prefix(command)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
}

fn quoted_string_ranges(line: &str) -> Vec<Range<usize>> {
    let bytes = line.as_bytes();
    let mut ranges = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] != b'"' || is_escaped(bytes, index) {
            index += 1;
            continue;
        }
        let start = index + 1;
        index = start;
        while index < bytes.len() {
            if bytes[index] == b'"' && !is_escaped(bytes, index) {
                ranges.push(start..index);
                index += 1;
                break;
            }
            index += 1;
        }
    }
    ranges
}

fn is_escaped(bytes: &[u8], position: usize) -> bool {
    let mut backslashes = 0usize;
    let mut cursor = position;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        backslashes += 1;
        cursor -= 1;
    }
    backslashes % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(line: &str) -> ParsedScript {
        parse_script(line.to_owned()).unwrap()
    }

    #[test]
    fn follows_confirmed_name_rules() {
        let source = concat!(
            "【文太郎,S000_A_0001】「本文」\n",
            "【砂雪@？？？,S001_A_0001】「……」\n",
            "【砂雪@,S001_C_0001】「遅い」\n",
            "【うぐいす】「本文」\n",
        );
        let parsed = parse(source);
        let names: Vec<_> = parsed
            .slots
            .iter()
            .map(|slot| {
                (
                    slot.rule.as_str(),
                    slot.scr_name.as_deref(),
                    slot.scr_msg.as_str(),
                )
            })
            .collect();
        assert_eq!(
            names,
            vec![
                ("21", Some("文太郎"), "「本文」"),
                ("12", Some("？？？"), "「……」"),
                ("11", Some("砂雪"), "「遅い」"),
                ("23", Some("うぐいす"), "「本文」"),
            ]
        );
    }

    #[test]
    fn parses_inline_and_dynamic_choices() {
        let source = concat!(
            "SELECT \"一つ目,*route_1\", \"二つ目,*route_2\"\n",
            "if(%0) cal $0=\"g_select\"+$(%0), $$0=\"動的,*route_3\":cal %0+=1\n",
        );
        let parsed = parse(source);
        assert_eq!(parsed.slots.len(), 3);
        assert_eq!(parsed.slots[0].scr_msg, "一つ目");
        assert_eq!(parsed.slots[1].target.as_deref(), Some("*route_2"));
        assert_eq!(parsed.slots[2].opcode.as_deref(), Some("DYNAMIC_SELECT"));
    }

    #[test]
    fn skips_commands_and_extracts_narration() {
        let parsed = parse("BG BLACK, 800@1\n本文[n]続き\n*label\n");
        assert_eq!(parsed.slots.len(), 1);
        assert_eq!(parsed.slots[0].scr_msg, "本文[n]続き");
        assert_eq!(parsed.slots[0].rule, "42");
    }

    #[test]
    fn supports_supplied_font_color_variants() {
        let parsed = parse(
            "[setFontColor red\\]中央[setFontColor reset]\n[setFontColor red\\]後半\n前半[setFontColor reset]\n",
        );
        let values: Vec<_> = parsed
            .slots
            .iter()
            .map(|slot| (slot.rule.as_str(), slot.scr_msg.as_str()))
            .collect();
        assert_eq!(values, vec![("30", "中央"), ("31", "後半"), ("32", "前半")]);
    }

    #[test]
    fn supports_name_only_variants() {
        let parsed = parse("【名前,voice】\n【名前】\n");
        assert_eq!(parsed.slots[0].rule, "24");
        assert_eq!(parsed.slots[1].rule, "25");
        assert!(parsed.slots.iter().all(|slot| slot.scr_msg.is_empty()));
    }
}
