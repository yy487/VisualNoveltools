use std::borrow::Cow;

use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};

use crate::adv::{Script, TextOperand};
use crate::Result;

const DIALOGUE_OPEN: &str = "【";
const DIALOGUE_CLOSE: &str = "】";
const TERMINAL_SUFFIX: &str = "\\k\\*";
const IGNORED_LINE_BREAK: &str = "\\n";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TranslationEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _scr_name: Option<String>,
    pub scr_msg: String,
    pub message: String,
    pub _file: String,
    pub _index: usize,
    pub _offset: usize,
    pub _inst_offset: usize,
    pub _size: usize,
    pub _type: String,
    pub _opcode: String,
    pub _encoding: String,
    pub _policy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _name_writable: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub prefix: String,
    pub name: Option<String>,
    pub body: String,
    pub suffix: String,
    pub name_writable: bool,
    pub body_byte_offset: usize,
    pub body_byte_size: usize,
    pub kind: &'static str,
}

pub fn decode_cp932(bytes: &[u8], context: &str) -> Result<String> {
    let decoded = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(bytes)
        .ok_or_else(|| format!("{context}: invalid CP932 byte sequence"))?;
    let text = decoded.into_owned();
    let encoded = encode_cp932(&text, context)?;
    if encoded != bytes {
        return Err(format!(
            "{context}: CP932 decode/encode round-trip mismatch"
        ));
    }
    Ok(text)
}

pub fn encode_cp932(text: &str, context: &str) -> Result<Vec<u8>> {
    if text.contains('\0') {
        return Err(format!("{context}: NUL is not allowed"));
    }
    let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        let unsupported = unsupported_characters(text);
        return Err(format!(
            "{context}: not representable in CP932: {}",
            unsupported.join(" ")
        ));
    }
    Ok(match encoded {
        Cow::Borrowed(bytes) => bytes.to_vec(),
        Cow::Owned(bytes) => bytes,
    })
}

fn unsupported_characters(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    for character in text.chars() {
        let mut encoded_utf8 = [0u8; 4];
        let value = character.encode_utf8(&mut encoded_utf8);
        if SHIFT_JIS.encode(value).2 {
            let item = format!("U+{:04X}({character})", character as u32);
            if !result.contains(&item) {
                result.push(item);
            }
        }
    }
    result
}

fn choice_prefix(text: &str) -> Option<&str> {
    if !text.starts_with("\\=") {
        return None;
    }
    text.find(';').map(|end| &text[..=end])
}

pub fn analyze_layout(plaintext: &[u8], context: &str) -> Result<Layout> {
    let decoded = decode_cp932(plaintext, context)?;
    let (without_suffix, suffix) = decoded
        .strip_suffix(TERMINAL_SUFFIX)
        .map_or((decoded.as_str(), ""), |body| (body, TERMINAL_SUFFIX));

    let prefix = choice_prefix(without_suffix).unwrap_or("");
    let visible = &without_suffix[prefix.len()..];

    let (name, source_body, name_writable) = if let Some(rest) = visible.strip_prefix(DIALOGUE_OPEN)
    {
        let close = rest
            .find(DIALOGUE_CLOSE)
            .ok_or_else(|| format!("{context}: dialogue starts with 【 but has no closing 】"))?;
        let name = &rest[..close];
        let source_body = &rest[close + DIALOGUE_CLOSE.len()..];
        (Some(name.to_owned()), source_body, !name.contains('\\'))
    } else {
        (None, visible, false)
    };
    let body = source_body.replace(IGNORED_LINE_BREAK, "");

    let mut before_body = prefix.to_owned();
    if let Some(name) = &name {
        before_body.push_str(DIALOGUE_OPEN);
        before_body.push_str(name);
        before_body.push_str(DIALOGUE_CLOSE);
    }
    let body_byte_offset = encode_cp932(&before_body, context)?.len();
    let body_byte_size = encode_cp932(source_body, context)?.len();
    let kind = if !prefix.is_empty() {
        "choice"
    } else if name.is_some() {
        "dialogue"
    } else {
        "monologue"
    };

    Ok(Layout {
        prefix: prefix.to_owned(),
        name,
        body,
        suffix: suffix.to_owned(),
        name_writable,
        body_byte_offset,
        body_byte_size,
        kind,
    })
}

pub fn extract_entries(script: &Script, source_file: &str) -> Result<Vec<TranslationEntry>> {
    let mut entries = Vec::new();
    for text in &script.texts {
        let context = format!("{source_file}@0x{:04X}", text.instruction_offset);
        let layout = analyze_layout(&text.plaintext, &context)?;
        if layout.body.is_empty() {
            continue;
        }
        let index = entries.len();
        if !has_translatable_text(&layout.body) {
            entries.push(None);
            continue;
        }
        entries.push(Some(TranslationEntry {
            name: layout.name.clone(),
            _scr_name: layout.name.clone(),
            scr_msg: layout.body.clone(),
            message: layout.body,
            _file: source_file.to_owned(),
            _index: index,
            _offset: text.instruction_offset + 1 + layout.body_byte_offset,
            _inst_offset: text.instruction_offset,
            _size: layout.body_byte_size,
            _type: layout.kind.to_owned(),
            _opcode: "X".to_owned(),
            _encoding: "CP932".to_owned(),
            _policy: "relocate".to_owned(),
            _name_writable: layout.name.as_ref().map(|_| layout.name_writable),
        }));
    }
    Ok(entries.into_iter().flatten().collect())
}

pub fn rebuild_plaintext(
    source: &TextOperand,
    entry: &TranslationEntry,
    source_file: &str,
) -> Result<Vec<u8>> {
    let context = format!("{source_file} entry {}", entry._index);
    if entry._file != source_file {
        return Err(format!(
            "{context}: _file is {:?}, expected {:?}",
            entry._file, source_file
        ));
    }
    if entry._inst_offset != source.instruction_offset {
        return Err(format!(
            "{context}: _inst_offset is 0x{:04X}, expected 0x{:04X}",
            entry._inst_offset, source.instruction_offset
        ));
    }

    let layout = analyze_layout(&source.plaintext, &context)?;
    if entry.scr_msg != layout.body {
        return Err(format!(
            "{context}: scr_msg does not match the source ADV text"
        ));
    }
    if entry._size != layout.body_byte_size {
        return Err(format!(
            "{context}: _size is {}, expected {}",
            entry._size, layout.body_byte_size
        ));
    }
    let expected_offset = source.instruction_offset + 1 + layout.body_byte_offset;
    if entry._offset != expected_offset {
        return Err(format!(
            "{context}: _offset is 0x{:04X}, expected 0x{expected_offset:04X}",
            entry._offset
        ));
    }
    if entry._type != layout.kind {
        return Err(format!(
            "{context}: _type is {:?}, expected {:?}",
            entry._type, layout.kind
        ));
    }
    if entry._opcode != "X" || entry._encoding != "CP932" || entry._policy != "relocate" {
        return Err(format!(
            "{context}: expected _opcode=X, _encoding=CP932, _policy=relocate"
        ));
    }
    let expected_name_writable = layout.name.as_ref().map(|_| layout.name_writable);
    if entry._name_writable != expected_name_writable {
        return Err(format!(
            "{context}: _name_writable is {:?}, expected {:?}",
            entry._name_writable, expected_name_writable
        ));
    }

    let source_controls = control_tokens(&entry.scr_msg);
    let translated_controls = control_tokens(&entry.message);
    if source_controls != translated_controls {
        return Err(format!(
            "{context}: body controls changed: source={source_controls:?}, translation={translated_controls:?}"
        ));
    }

    let mut rebuilt = layout.prefix;
    match (&layout.name, &entry.name, &entry._scr_name) {
        (Some(source_name), Some(name), Some(scr_name)) => {
            if scr_name != source_name {
                return Err(format!("{context}: _scr_name does not match source name"));
            }
            if !layout.name_writable && name != source_name {
                return Err(format!(
                    "{context}: dynamic control-bearing name is immutable"
                ));
            }
            if layout.name_writable && name.contains('\\') {
                return Err(format!(
                    "{context}: a static translated name cannot introduce renderer controls"
                ));
            }
            rebuilt.push_str(DIALOGUE_OPEN);
            rebuilt.push_str(name);
            rebuilt.push_str(DIALOGUE_CLOSE);
        }
        (None, None, None) => {}
        (None, None, Some(_)) => {
            return Err(format!("{context}: narration must not contain _scr_name"));
        }
        (None, Some(_), _) => return Err(format!("{context}: narration must not contain name")),
        (Some(_), _, _) => {
            return Err(format!(
                "{context}: dialogue requires name and _scr_name fields"
            ));
        }
    }
    if entry.message == entry.scr_msg && entry.name == layout.name {
        return Ok(source.plaintext.clone());
    }
    rebuilt.push_str(&entry.message);
    rebuilt.push_str(&layout.suffix);
    encode_cp932(&rebuilt, &context)
}

pub fn control_tokens(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut result = Vec::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor] != b'\\' || cursor + 1 >= bytes.len() {
            cursor += 1;
            continue;
        }
        let start = cursor;
        cursor = control_token_end(bytes, cursor);
        result.push(text[start..cursor].to_owned());
    }
    result
}

fn control_token_end(bytes: &[u8], start: usize) -> usize {
    let mut cursor = start + 2;
    let parameter_start = cursor;
    while cursor < bytes.len() && matches!(bytes[cursor], b'0'..=b'9' | b'+' | b'-' | b',') {
        cursor += 1;
    }
    if cursor > parameter_start && cursor < bytes.len() && bytes[cursor] == b';' {
        cursor + 1
    } else {
        parameter_start
    }
}

fn has_translatable_text(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor] == b'\\' && cursor + 1 < bytes.len() {
            cursor = control_token_end(bytes, cursor);
            continue;
        }
        let character = text[cursor..]
            .chars()
            .next()
            .expect("cursor is within the string");
        if !character.is_whitespace() {
            return true;
        }
        cursor += character.len_utf8();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_dialogue_and_terminal_controls() {
        let source = encode_cp932("【女】お兄さん、遊んでかない？\\k\\*", "test").unwrap();
        let layout = analyze_layout(&source, "test").unwrap();
        assert_eq!(layout.name.as_deref(), Some("女"));
        assert_eq!(layout.body, "お兄さん、遊んでかない？");
        assert_eq!(layout.suffix, "\\k\\*");
        assert!(layout.name_writable);
    }

    #[test]
    fn recognizes_choice_and_dynamic_name() {
        let choice = analyze_layout(b"\\=14;choice", "test").unwrap();
        assert_eq!(choice.prefix, "\\=14;");
        assert_eq!(choice.body, "choice");
        assert_eq!(choice.kind, "choice");

        let source = encode_cp932("【\\_\\%70,8;\\ 】message\\k\\*", "test").unwrap();
        let dynamic = analyze_layout(&source, "test").unwrap();
        assert_eq!(dynamic.name.as_deref(), Some("\\_\\%70,8;\\ "));
        assert!(!dynamic.name_writable);
    }

    #[test]
    fn hides_explicit_line_breaks_and_drops_them_from_modified_text() {
        let plaintext = encode_cp932("【少女】一行目\\n二行目\\k\\*", "test").unwrap();
        let layout = analyze_layout(&plaintext, "test").unwrap();
        assert_eq!(layout.body, "一行目二行目");
        assert_eq!(layout.body_byte_size, 14);

        let source = TextOperand {
            instruction_offset: 0x100,
            encoded_range: 0x101..0x102,
            plaintext: plaintext.clone(),
        };
        let script = Script {
            instructions: Vec::new(),
            texts: vec![source.clone()],
            references: Vec::new(),
            parsed_end: 0,
        };
        let mut entry = extract_entries(&script, "TEST.ADV").unwrap().remove(0);

        assert_eq!(
            rebuild_plaintext(&source, &entry, "TEST.ADV").unwrap(),
            plaintext
        );
        entry.message = "差し替えた長い文章".to_owned();
        let rebuilt = rebuild_plaintext(&source, &entry, "TEST.ADV").unwrap();
        assert_eq!(
            decode_cp932(&rebuilt, "test").unwrap(),
            "【少女】差し替えた長い文章\\k\\*"
        );
    }

    #[test]
    fn tokenizes_renderer_controls() {
        assert_eq!(
            control_tokens("a\\n\\%70,8;b\\k\\*"),
            ["\\n", "\\%70,8;", "\\k", "\\*"]
        );
    }

    #[test]
    fn skips_control_only_renderer_strings() {
        assert!(!has_translatable_text(
            "\\/\\l20;\\<\\s80,400,62,3;\\c255,255,255,0,0,0;\\y0,0,0;\\w2;\\]\\+"
        ));
        assert!(!has_translatable_text("\\!464,40,4,6,0,4;"));
        assert!(!has_translatable_text(
            "\\c0,0,0,255,255,255;\\y0,0,0;\\?14;"
        ));
        assert!(has_translatable_text("\\c255,255,255,0,0,0;表示する文章"));
        assert!(has_translatable_text("‥‥‥"));

        let script = Script {
            instructions: Vec::new(),
            texts: vec![
                TextOperand {
                    instruction_offset: 0x100,
                    encoded_range: 0x101..0x102,
                    plaintext: b"\\!464,40,4,6,0,4;".to_vec(),
                },
                TextOperand {
                    instruction_offset: 0x200,
                    encoded_range: 0x201..0x202,
                    plaintext: encode_cp932("表示する文章", "test").unwrap(),
                },
            ],
            references: Vec::new(),
            parsed_end: 0,
        };
        let entries = extract_entries(&script, "TEST.ADV").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]._index, 1);
    }

    #[test]
    fn rejects_unrepresentable_characters() {
        let error = encode_cp932("简体中文", "test").unwrap_err();
        assert!(error.contains("not representable in CP932"));
        assert!(error.contains("U+7B80"));
    }

    #[test]
    fn rejects_changed_source_text_and_removed_controls() {
        let plaintext = encode_cp932("値は\\%70,8;です。\\k\\*", "test").unwrap();
        let source = TextOperand {
            instruction_offset: 0x100,
            encoded_range: 0x101..0x102,
            plaintext,
        };
        let script = Script {
            instructions: Vec::new(),
            texts: vec![source.clone()],
            references: Vec::new(),
            parsed_end: 0,
        };
        let entry = extract_entries(&script, "TEST.ADV").unwrap().remove(0);

        let mut changed_source = entry.clone();
        changed_source.scr_msg.push('改');
        assert!(rebuild_plaintext(&source, &changed_source, "TEST.ADV")
            .unwrap_err()
            .contains("scr_msg does not match"));

        let mut removed_control = entry;
        removed_control.message = "値はです。".to_owned();
        assert!(rebuild_plaintext(&source, &removed_control, "TEST.ADV")
            .unwrap_err()
            .contains("body controls changed"));
    }

    #[test]
    fn rejects_changed_dynamic_name() {
        let plaintext = encode_cp932("【\\_\\%70,8;\\ 】本文\\k\\*", "test").unwrap();
        let source = TextOperand {
            instruction_offset: 0x100,
            encoded_range: 0x101..0x102,
            plaintext,
        };
        let script = Script {
            instructions: Vec::new(),
            texts: vec![source.clone()],
            references: Vec::new(),
            parsed_end: 0,
        };
        let mut entry = extract_entries(&script, "TEST.ADV").unwrap().remove(0);
        entry.name = Some("別名".to_owned());
        assert!(rebuild_plaintext(&source, &entry, "TEST.ADV")
            .unwrap_err()
            .contains("dynamic control-bearing name is immutable"));
    }
}
