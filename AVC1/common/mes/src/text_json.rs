use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::encoding::encode_cp932;
use crate::script::{ParsedScript, SlotKind, TextSlot, parse_script};
use crate::{ToolResult, error};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TextEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    pub scr_name: Option<String>,
    pub scr_msg: String,
    pub message: String,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_line")]
    pub line: usize,
    #[serde(rename = "_offset")]
    pub offset: u64,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub kind: String,
    #[serde(rename = "_rule")]
    pub rule: String,
    #[serde(rename = "_opcode", skip_serializing_if = "Option::is_none")]
    pub opcode: Option<String>,
    #[serde(rename = "_target", skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(rename = "_choice_index", skip_serializing_if = "Option::is_none")]
    pub choice_index: Option<usize>,
    #[serde(rename = "_encoding")]
    pub encoding: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InjectCounts {
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
}

#[derive(Debug)]
struct Replacement {
    range: Range<usize>,
    value: String,
}

pub fn entries_from_script(parsed: &ParsedScript, relative_file: &str) -> Vec<TextEntry> {
    parsed
        .slots
        .iter()
        .map(|slot| TextEntry {
            name: slot.scr_name.clone(),
            scr_name: slot.scr_name.clone(),
            scr_msg: slot.scr_msg.clone(),
            message: slot.scr_msg.clone(),
            file: relative_file.to_owned(),
            index: slot.index,
            line: slot.line,
            offset: slot.offset,
            size: slot.size,
            kind: slot.kind.as_str().to_owned(),
            rule: slot.rule.clone(),
            opcode: slot.opcode.clone(),
            target: slot.target.clone(),
            choice_index: slot.choice_index,
            encoding: "cp932".to_owned(),
        })
        .collect()
}

pub fn serialize_entries(entries: &[TextEntry]) -> ToolResult<Vec<u8>> {
    let mut json = serde_json::to_string_pretty(entries)?;
    json.push('\n');
    Ok(json.into_bytes())
}

pub fn deserialize_entries(bytes: &[u8], context: &str) -> ToolResult<Vec<TextEntry>> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| crate::error(format!("{context}: JSON is not UTF-8: {error}")))?;
    serde_json::from_str(text).map_err(|error| crate::error(format!("{context}: {error}")))
}

pub fn inject_script(
    parsed: ParsedScript,
    entries: &[TextEntry],
    relative_file: &str,
) -> ToolResult<(Vec<u8>, InjectCounts)> {
    if entries.len() != parsed.slots.len() {
        return Err(error(format!(
            "{relative_file}: JSON entry count {} does not match source slot count {}",
            entries.len(),
            parsed.slots.len()
        )));
    }

    let mut replacements = Vec::new();
    let mut counts = InjectCounts {
        json_entries: entries.len(),
        ..InjectCounts::default()
    };

    for (slot, entry) in parsed.slots.iter().zip(entries) {
        validate_metadata(slot, entry, relative_file)?;
        validate_translation(slot, entry, relative_file)?;

        let translated_name = match (&slot.scr_name, &entry.scr_name, &entry.name) {
            (Some(source), Some(validation), Some(translated)) => {
                if validation != source {
                    return Err(entry_error(
                        relative_file,
                        slot,
                        format!("_scr_name mismatch: expected {source:?}, got {validation:?}"),
                    ));
                }
                Some(translated)
            }
            (Some(_), _, _) => {
                return Err(entry_error(
                    relative_file,
                    slot,
                    "named entry requires both name and _scr_name",
                ));
            }
            (None, None, None) => None,
            (None, _, _) => {
                return Err(entry_error(
                    relative_file,
                    slot,
                    "unnamed entry must not contain name or _scr_name",
                ));
            }
        };

        let name_changed = translated_name
            .zip(slot.scr_name.as_ref())
            .is_some_and(|(translated, source)| translated != source);
        let message_changed = entry.message != slot.scr_msg;
        if name_changed || message_changed {
            counts.patched += 1;
        } else {
            counts.unchanged += 1;
        }

        if message_changed {
            replacements.push(Replacement {
                range: slot.message_range.clone(),
                value: entry.message.clone(),
            });
        }
        if name_changed {
            replacements.push(Replacement {
                range: slot.name_range.clone().expect("named slot has name range"),
                value: entry.name.clone().expect("validated name exists"),
            });
        }
    }

    replacements.sort_by(|left, right| right.range.start.cmp(&left.range.start));
    let mut next_start = parsed.text.len();
    for replacement in &replacements {
        if replacement.range.end > next_start {
            return Err(error(format!(
                "{relative_file}: internal error: overlapping replacement spans"
            )));
        }
        next_start = replacement.range.start;
    }

    let mut rebuilt = parsed.text;
    for replacement in replacements {
        rebuilt.replace_range(replacement.range, &replacement.value);
    }
    let bytes = encode_cp932(&rebuilt, relative_file)?;
    Ok((bytes, counts))
}

fn validate_metadata(slot: &TextSlot, entry: &TextEntry, relative_file: &str) -> ToolResult<()> {
    macro_rules! check {
        ($condition:expr, $message:expr) => {
            if !$condition {
                return Err(entry_error(relative_file, slot, $message));
            }
        };
    }

    check!(entry.file == relative_file, "_file mismatch");
    check!(entry.index == slot.index, "_index mismatch");
    check!(entry.line == slot.line, "_line mismatch");
    check!(entry.offset == slot.offset, "_offset mismatch");
    check!(entry.size == slot.size, "_size mismatch");
    check!(entry.kind == slot.kind.as_str(), "_type mismatch");
    check!(entry.rule == slot.rule, "_rule mismatch");
    check!(entry.opcode == slot.opcode, "_opcode mismatch");
    check!(entry.target == slot.target, "_target mismatch");
    check!(
        entry.choice_index == slot.choice_index,
        "_choice_index mismatch"
    );
    check!(
        entry.encoding.eq_ignore_ascii_case("cp932"),
        "_encoding must be cp932"
    );
    check!(entry.scr_msg == slot.scr_msg, "scr_msg mismatch");
    Ok(())
}

fn validate_translation(slot: &TextSlot, entry: &TextEntry, relative_file: &str) -> ToolResult<()> {
    validate_field(&entry.message, "message", relative_file, slot)?;
    if !slot.scr_msg.is_empty() && entry.message.is_empty() {
        return Err(entry_error(
            relative_file,
            slot,
            "message cannot be empty because the line would no longer match its source rule",
        ));
    }

    let source_controls = bracket_tokens(&slot.scr_msg);
    let translated_controls = bracket_tokens(&entry.message);
    if !control_sequence_allows_deletions(&source_controls, &translated_controls) {
        return Err(entry_error(
            relative_file,
            slot,
            format!(
                "control sequence invalid: source {source_controls:?}; translation {translated_controls:?}; controls may be deleted but retained controls must stay in order"
            ),
        ));
    }

    if slot.kind == SlotKind::Choice
        && (entry.message.contains('"') || entry.message.contains(",*"))
    {
        return Err(entry_error(
            relative_file,
            slot,
            "choice message must not contain a quote or the ',*' target delimiter",
        ));
    }

    if let Some(name) = &entry.name {
        validate_field(name, "name", relative_file, slot)?;
        if slot
            .scr_name
            .as_ref()
            .is_some_and(|source| !source.is_empty())
            && name.is_empty()
        {
            return Err(entry_error(relative_file, slot, "name cannot be empty"));
        }
        if name
            .chars()
            .any(|character| matches!(character, '【' | '】' | '@' | ','))
        {
            return Err(entry_error(
                relative_file,
                slot,
                "name contains a structural delimiter (【, 】, @, or comma)",
            ));
        }
    }

    encode_cp932(
        &entry.message,
        &format!("{relative_file}:{} message", slot.line),
    )?;
    if let Some(name) = &entry.name {
        encode_cp932(name, &format!("{relative_file}:{} name", slot.line))?;
    }
    Ok(())
}

fn validate_field(
    value: &str,
    field: &str,
    relative_file: &str,
    slot: &TextSlot,
) -> ToolResult<()> {
    if value.contains('\0') {
        return Err(entry_error(
            relative_file,
            slot,
            format!("{field} contains NUL"),
        ));
    }
    if value.contains('\r') || value.contains('\n') {
        return Err(entry_error(
            relative_file,
            slot,
            format!("{field} contains a real newline; use the literal [n] control code"),
        ));
    }
    Ok(())
}

fn bracket_tokens(value: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut rest = value;
    while let Some(open) = rest.find('[') {
        let after_open = &rest[open..];
        let Some(close) = after_open.find(']') else {
            break;
        };
        tokens.push(&after_open[..=close]);
        rest = &after_open[close + 1..];
    }
    tokens
}

fn control_sequence_allows_deletions(source: &[&str], translated: &[&str]) -> bool {
    let mut source_cursor = 0usize;
    for translated_token in translated {
        let Some(relative_position) = source[source_cursor..]
            .iter()
            .position(|source_token| source_token == translated_token)
        else {
            return false;
        };
        source_cursor += relative_position + 1;
    }
    true
}

fn entry_error(relative_file: &str, slot: &TextSlot, message: impl AsRef<str>) -> crate::ToolError {
    error(format!(
        "{relative_file}: entry {} (line {}): {}",
        slot.index,
        slot.line,
        message.as_ref()
    ))
}

pub fn parse_and_extract(source: String, relative_file: &str) -> ToolResult<Vec<TextEntry>> {
    let parsed = parse_script(source)?;
    Ok(entries_from_script(&parsed, relative_file))
}

#[cfg(test)]
mod tests {
    use crate::encoding::{decode_cp932, encode_cp932};

    use super::*;

    fn source_and_entries(source: &str) -> (ParsedScript, Vec<TextEntry>) {
        let parsed = parse_script(source.to_owned()).unwrap();
        let entries = entries_from_script(&parsed, "scene.txt");
        (parsed, entries)
    }

    #[test]
    fn unchanged_injection_is_byte_exact() {
        let source = "本文[n]続き\n【文太郎,S000】「台詞」\n";
        let original = encode_cp932(source, "test").unwrap();
        let (parsed, entries) = source_and_entries(source);
        let (rebuilt, counts) = inject_script(parsed, &entries, "scene.txt").unwrap();
        assert_eq!(rebuilt, original);
        assert_eq!(counts.unchanged, 2);
    }

    #[test]
    fn edits_only_captured_name_and_choice_text() {
        let source = concat!(
            "【砂雪@？？？,S001】「……」\n",
            "SELECT \"元の選択肢,*route\", \"そのまま,*other\"\n",
        );
        let (parsed, mut entries) = source_and_entries(source);
        entries[0].name = Some("謎の人".to_owned());
        entries[1].message = "翻訳した選択肢".to_owned();
        let (rebuilt, counts) = inject_script(parsed, &entries, "scene.txt").unwrap();
        let rebuilt = decode_cp932(&rebuilt, "test").unwrap();
        assert_eq!(
            rebuilt,
            "【砂雪@謎の人,S001】「……」\nSELECT \"翻訳した選択肢,*route\", \"そのまま,*other\"\n"
        );
        assert_eq!(counts.patched, 2);
    }

    #[test]
    fn rejects_modified_source_validation_fields() {
        let (parsed, mut entries) = source_and_entries("本文\n");
        entries[0].scr_msg = "別物".to_owned();
        let error = inject_script(parsed, &entries, "scene.txt").unwrap_err();
        assert!(error.0.contains("scr_msg mismatch"));
    }

    #[test]
    fn allows_deleted_control_codes() {
        let (parsed, mut entries) = source_and_entries("本文[n]続き\n");
        entries[0].message = "本文 続き".to_owned();
        let (rebuilt, counts) = inject_script(parsed, &entries, "scene.txt").unwrap();
        assert_eq!(decode_cp932(&rebuilt, "test").unwrap(), "本文 続き\n");
        assert_eq!(counts.patched, 1);
    }

    #[test]
    fn rejects_added_or_reordered_control_codes() {
        let (parsed, mut entries) = source_and_entries("本文[n]続き\n");
        entries[0].message = "本文[n][n]続き".to_owned();
        let error = inject_script(parsed, &entries, "scene.txt").unwrap_err();
        assert!(error.0.contains("control sequence invalid"));
    }

    #[test]
    fn rejects_control_code_reordering() {
        let (parsed, mut entries) = source_and_entries("前[n]中[r]后\n");
        entries[0].message = "前[r]中[n]后".to_owned();
        let error = inject_script(parsed, &entries, "scene.txt").unwrap_err();
        assert!(error.0.contains("control sequence invalid"));
    }

    #[test]
    fn rejects_modified_original_name() {
        let (parsed, mut entries) = source_and_entries("【文太郎,S000】「本文」\n");
        entries[0].scr_name = Some("別人".to_owned());
        let error = inject_script(parsed, &entries, "scene.txt").unwrap_err();
        assert!(error.0.contains("_scr_name mismatch"));
    }
}
