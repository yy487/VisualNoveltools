use crate::glyph::GlyphDictionary;
use crate::{ToolError, ToolResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    #[serde(rename = "_offset", default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
    #[serde(rename = "_size", default, skip_serializing_if = "Option::is_none")]
    pub size: Option<usize>,
    #[serde(rename = "_type", default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(rename = "_encoding", default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    #[serde(rename = "_policy", default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "_scr_name", default, skip_serializing_if = "Option::is_none")]
    pub scr_name: Option<String>,
    pub scr_msg: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_parts: Option<Vec<String>>,
    #[serde(rename = "_raw_body", default, skip_serializing_if = "Option::is_none")]
    pub raw_body: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TranslationEditEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    message: String,
}

pub fn read_entries(path: &Path) -> ToolResult<Vec<TextEntry>> {
    let bytes = fs::read(path)
        .map_err(|error| ToolError(format!("cannot read JSON '{}': {error}", path.display())))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| ToolError(format!("cannot parse JSON '{}': {error}", path.display())))?;
    if !value.is_array() {
        return Err(ToolError(format!(
            "translation JSON '{}' must contain a top-level array",
            path.display()
        )));
    }
    serde_json::from_value(value).map_err(|error| {
        ToolError(format!(
            "translation JSON '{}' has an invalid entry: {error}",
            path.display()
        ))
    })
}

pub fn write_entries(path: &Path, entries: &[TextEntry]) -> ToolResult<()> {
    let mut bytes = serde_json::to_vec_pretty(entries)?;
    bytes.push(b'\n');
    fs::write(path, bytes)
        .map_err(|error| ToolError(format!("cannot write JSON '{}': {error}", path.display())))
}

/// Build the translator-facing projection. The complete source entries remain
/// in a separate internal tree and are merged back before injection.
pub fn write_translation_view_tree(
    source_root: &Path,
    output_root: &Path,
    dictionary: &GlyphDictionary,
) -> ToolResult<usize> {
    if output_root.exists() {
        return Err(ToolError(format!(
            "translation JSON output already exists: '{}'",
            output_root.display()
        )));
    }
    fs::create_dir_all(output_root).map_err(|error| {
        ToolError(format!(
            "cannot create translation JSON directory '{}': {error}",
            output_root.display()
        ))
    })?;
    let source_files = collect_json_tree(source_root)?;
    let mut total_entries = 0;
    for (relative, source) in source_files {
        if !is_editable_source_json(&relative) {
            continue;
        }
        let entries = read_entries(&source)?;
        let projected = entries
            .iter()
            .map(|entry| {
                let name = entry
                    .name
                    .as_deref()
                    .map(|value| dictionary.project_translation_text(value))
                    .transpose()?;
                let message = dictionary
                    .project_translation_text(&entry.message)?
                    .trim_start_matches('\u{3000}')
                    .to_string();
                Ok(TranslationEditEntry { name, message })
            })
            .collect::<ToolResult<Vec<_>>>()?;
        let destination = output_root.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                ToolError(format!(
                    "cannot create translation JSON directory '{}': {error}",
                    parent.display()
                ))
            })?;
        }
        write_json(&destination, &projected)?;
        total_entries += projected.len();
    }
    Ok(total_entries)
}

/// Merge the editable name/message projection with immutable source metadata.
/// File sets, entry counts, and name presence are validated before injection.
pub fn merge_translation_view_tree(
    source_root: &Path,
    translation_root: &Path,
    dictionary: &GlyphDictionary,
) -> ToolResult<Vec<TextEntry>> {
    let source_files = collect_json_tree(source_root)?
        .into_iter()
        .filter(|(relative, _)| is_editable_source_json(relative))
        .collect::<BTreeMap<_, _>>();
    let translation_files = collect_json_tree(translation_root)?;
    let source_set = source_files.keys().collect::<Vec<_>>();
    let translation_set = translation_files.keys().collect::<Vec<_>>();
    if source_set != translation_set {
        let missing = source_files
            .keys()
            .filter(|path| !translation_files.contains_key(*path))
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>();
        let extra = translation_files
            .keys()
            .filter(|path| !source_files.contains_key(*path))
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>();
        return Err(ToolError(format!(
            "translation JSON file set changed; missing=[{}] extra=[{}]",
            missing.join(", "),
            extra.join(", ")
        )));
    }

    let mut merged = Vec::new();
    for (relative, source) in source_files {
        let mut source_entries = read_entries(&source)?;
        let translation = translation_files
            .get(&relative)
            .expect("validated translation JSON file set");
        let edit_entries: Vec<TranslationEditEntry> = read_json(translation)?;
        if source_entries.len() != edit_entries.len() {
            return Err(ToolError(format!(
                "translation JSON '{}' has {} entries, expected {}; entries must stay in their original order",
                translation.display(),
                edit_entries.len(),
                source_entries.len()
            )));
        }
        for (index, (source_entry, edit_entry)) in
            source_entries.iter_mut().zip(edit_entries).enumerate()
        {
            match (&source_entry.name, edit_entry.name) {
                (Some(source_name), Some(name)) => {
                    let source_name_view = dictionary.project_translation_text(source_name)?;
                    if name != source_name_view {
                        source_entry.name =
                            Some(restore_internal_markup(source_name, &name, dictionary)?);
                    }
                }
                (None, None) => {}
                (Some(_), None) => {
                    return Err(ToolError(format!(
                        "translation JSON '{}' entry {} removed its name field",
                        translation.display(),
                        index
                    )))
                }
                (None, Some(_)) => {
                    return Err(ToolError(format!(
                        "translation JSON '{}' entry {} added a name to a nameless source entry",
                        translation.display(),
                        index
                    )))
                }
            }
            let source_message_view = dictionary.project_translation_text(&source_entry.message)?;
            let prefix_len = source_message_view
                .chars()
                .take_while(|character| *character == '\u{3000}')
                .count();
            let edited_message_view =
                "\u{3000}".repeat(prefix_len) + edit_entry.message.trim_start_matches('\u{3000}');
            if edited_message_view != source_message_view {
                source_entry.message = restore_internal_markup(
                    &source_entry.message,
                    &edited_message_view,
                    dictionary,
                )?;
            }
        }
        merged.extend(source_entries);
    }
    Ok(merged)
}

fn is_editable_source_json(relative: &Path) -> bool {
    relative
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name.to_ascii_lowercase().ends_with(".msb.json"))
}

#[derive(Debug)]
struct ControlEvent {
    position: usize,
    markup: String,
    line_breaks: usize,
}

/// Reattach opaque source controls after the translator edits the projected
/// text. Newline-bearing control runs must keep the same number of visible
/// breaks; other controls retain their relative position by scaled glyph
/// ordinal. The original source is returned unchanged by the caller when the
/// projected text was not edited, which preserves exact duplicate glyph slots.
fn restore_internal_markup(
    source: &str,
    edited: &str,
    dictionary: &GlyphDictionary,
) -> ToolResult<String> {
    let mut events = Vec::new();
    let mut visible_position = 0usize;
    let mut source_pos = 0usize;
    while source_pos < source.len() {
        let character = source[source_pos..]
            .chars()
            .next()
            .expect("valid UTF-8 character boundary");
        if character == '<' {
            if let Some((end, _)) = crate::glyph::parse_markup_at(source, source_pos, "g:", 4)? {
                let projected = dictionary.project_translation_text(&source[source_pos..end])?;
                visible_position += projected.chars().count();
                source_pos = end;
                continue;
            }
            if let Some((end, value)) = crate::glyph::parse_markup_at(source, source_pos, "b:", 2)?
            {
                let mut end_run = end;
                let mut controls = vec![(source[source_pos..end].to_string(), value)];
                while end_run < source.len() {
                    let Some((next_end, next_value)) =
                        crate::glyph::parse_markup_at(source, end_run, "b:", 2)?
                    else {
                        break;
                    };
                    controls.push((source[end_run..next_end].to_string(), next_value));
                    end_run = next_end;
                }
                let mut markup = String::new();
                let mut has_line_break = false;
                for (raw, value) in controls {
                    if value == 0 && has_line_break {
                        events.push(ControlEvent {
                            position: visible_position,
                            markup,
                            line_breaks: 1,
                        });
                        markup = String::new();
                    }
                    markup.push_str(&raw);
                    has_line_break |= value == 0;
                }
                events.push(ControlEvent {
                    position: visible_position,
                    markup,
                    line_breaks: usize::from(has_line_break),
                });
                source_pos = end_run;
                continue;
            }
        }
        visible_position += usize::from(character != '\n');
        source_pos += character.len_utf8();
    }

    let source_breaks = events.iter().map(|event| event.line_breaks).sum::<usize>();
    let edited_breaks = edited
        .chars()
        .filter(|character| *character == '\n')
        .count();
    if source_breaks != edited_breaks {
        return Err(ToolError(format!(
            "translated text changed control-line count: source={source_breaks} replacement={edited_breaks}"
        )));
    }
    let source_visible_len = visible_position.max(1);
    let edited_visible_len = edited
        .chars()
        .filter(|character| *character != '\n')
        .count();
    let mut ordinary_events = events
        .iter()
        .filter(|event| event.line_breaks == 0)
        .collect::<Vec<_>>();
    ordinary_events.sort_by_key(|event| event.position);
    let mut event_index = 0usize;
    let mut line_event_index = 0usize;
    let line_events = events
        .iter()
        .filter(|event| event.line_breaks > 0)
        .collect::<Vec<_>>();
    let mut output = String::new();
    let mut edited_position = 0usize;
    for character in edited.chars() {
        if character == '\n' {
            let event = line_events.get(line_event_index).ok_or_else(|| {
                ToolError("translated text contains an unexpected control line break".to_string())
            })?;
            output.push_str(&event.markup);
            line_event_index += 1;
            continue;
        }
        while let Some(event) = ordinary_events.get(event_index) {
            let target = event.position.saturating_mul(edited_visible_len) / source_visible_len;
            if target > edited_position {
                break;
            }
            output.push_str(&event.markup);
            event_index += 1;
        }
        output.push(character);
        edited_position += 1;
    }
    while let Some(event) = ordinary_events.get(event_index) {
        output.push_str(&event.markup);
        event_index += 1;
    }
    if line_event_index != line_events.len() {
        return Err(ToolError(
            "translated text did not retain all source control line breaks".to_string(),
        ));
    }
    Ok(output)
}

fn collect_json_tree(root: &Path) -> ToolResult<BTreeMap<PathBuf, PathBuf>> {
    if !root.is_dir() {
        return Err(ToolError(format!(
            "JSON tree '{}' is not a directory",
            root.display()
        )));
    }
    let mut output = BTreeMap::new();
    collect_json_tree_inner(root, root, &mut output)?;
    Ok(output)
}

fn collect_json_tree_inner(
    root: &Path,
    current: &Path,
    output: &mut BTreeMap<PathBuf, PathBuf>,
) -> ToolResult<()> {
    let mut entries = fs::read_dir(current)
        .map_err(|error| ToolError(format!("cannot list '{}': {error}", current.display())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            ToolError(format!(
                "cannot read directory entry in '{}': {error}",
                current.display()
            ))
        })?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_json_tree_inner(root, &path, output)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        {
            let relative = path.strip_prefix(root).map_err(|error| {
                ToolError(format!(
                    "cannot make '{}' relative to '{}': {error}",
                    path.display(),
                    root.display()
                ))
            })?;
            output.insert(relative.to_path_buf(), path);
        }
    }
    Ok(())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> ToolResult<T> {
    let bytes = fs::read(path)
        .map_err(|error| ToolError(format!("cannot read JSON '{}': {error}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| ToolError(format!("cannot parse JSON '{}': {error}", path.display())))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> ToolResult<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)
        .map_err(|error| ToolError(format!("cannot write JSON '{}': {error}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translation_view_only_exposes_editable_fields_and_restores_indent() {
        let root = std::env::temp_dir().join(format!(
            "merry-translation-view-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source = root.join("source");
        let translation = root.join("translation");
        fs::create_dir_all(&source).unwrap();
        let entries = vec![TextEntry {
            file: "sample.msb".to_string(),
            index: 0,
            id: Some(7),
            offset: Some(16),
            size: Some(8),
            kind: Some("dialogue".to_string()),
            encoding: Some("glyph-index".to_string()),
            policy: Some("relocate".to_string()),
            name: Some("原名".to_string()),
            scr_name: Some("原名".to_string()),
            scr_msg: "\u{3000}\u{3000}<g:0113><g:2001><g:0221>".to_string(),
            message: "\u{3000}\u{3000}<g:0113><g:2001><g:0221>".to_string(),
            message_parts: None,
            raw_body: Some("00".to_string()),
            extra: BTreeMap::new(),
        }];
        let dictionary = GlyphDictionary::built_in().unwrap();
        write_entries(&source.join("sample.msb.json"), &entries).unwrap();
        write_json(&source.join("sample.scx.json"), &Vec::<Value>::new()).unwrap();
        assert_eq!(
            write_translation_view_tree(&source, &translation, &dictionary).unwrap(),
            1
        );

        let path = translation.join("sample.msb.json");
        let value: Value = read_json(&path).unwrap();
        let object = value[0].as_object().unwrap();
        assert_eq!(object.len(), 2);
        assert_eq!(object["name"], "原名");
        assert_eq!(object["message"], "――わ");

        let edited = vec![TranslationEditEntry {
            name: Some("译名".to_string()),
            message: "译文".to_string(),
        }];
        write_json(&path, &edited).unwrap();
        let merged = merge_translation_view_tree(&source, &translation, &dictionary).unwrap();
        assert_eq!(merged[0].name.as_deref(), Some("译名"));
        assert_eq!(merged[0].scr_name.as_deref(), Some("原名"));
        assert_eq!(
            merged[0].scr_msg,
            "\u{3000}\u{3000}<g:0113><g:2001><g:0221>"
        );
        assert_eq!(merged[0].message, "\u{3000}\u{3000}译文");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn restores_adjacent_controls_before_multibyte_text() {
        let dictionary = GlyphDictionary::built_in().unwrap();
        let source = "<b:11><b:00><b:42>……でも、やっぱり";
        assert_eq!(
            restore_internal_markup(source, "\n不过，果然", &dictionary).unwrap(),
            "<b:11><b:00><b:42>不过，果然"
        );
    }

    #[test]
    fn restores_multiple_line_breaks_in_one_control_run() {
        let dictionary = GlyphDictionary::built_in().unwrap();
        let source = "<b:11><b:00><b:42><b:00><b:04>原文";
        assert_eq!(
            restore_internal_markup(source, "\n甲\n乙", &dictionary).unwrap(),
            "<b:11><b:00><b:42>甲<b:00><b:04>乙"
        );
    }
}
