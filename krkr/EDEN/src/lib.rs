use encoding_rs::SHIFT_JIS;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone)]
pub struct ExtractOptions {
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub macro_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct InjectOptions {
    pub json_dir: PathBuf,
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    pub macro_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RepairOptions {
    pub broken_json_dir: PathBuf,
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    pub macro_path: PathBuf,
    pub name_dictionary_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct InjectReport {
    pub json_files: usize,
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub output_files: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepairReport {
    pub json_files: usize,
    pub entries: usize,
    pub translated_messages: usize,
    pub multipart_entries: usize,
    pub repaired_inner_quotes: usize,
    pub dictionary_entries: usize,
    pub translated_speaker_names: usize,
    pub unmapped_speaker_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ControlRecord {
    pub after_part: usize,
    pub line: usize,
    pub tag: String,
    pub raw: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_line")]
    pub line: usize,
    #[serde(rename = "_end_line")]
    pub end_line: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_boundary")]
    pub boundary: String,
    #[serde(rename = "_source_kind")]
    pub source_kind: String,
    #[serde(rename = "_speaker_macro", skip_serializing_if = "Option::is_none")]
    pub speaker_macro: Option<String>,
    #[serde(rename = "_target", skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(rename = "_ruby_removed", default, skip_serializing_if = "is_zero")]
    pub ruby_removed: usize,
    #[serde(rename = "_controls", default, skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<ControlRecord>,
    #[serde(rename = "_scr_name", default, skip_serializing_if = "Option::is_none")]
    pub scr_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub scr_msg: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scr_msg_parts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_parts: Option<Vec<String>>,
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub severity: String,
    pub code: String,
    pub file: String,
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileReport {
    pub file: String,
    pub entries: usize,
    pub dialogue: usize,
    pub monologue: usize,
    pub choice: usize,
    pub ui: usize,
    pub name: usize,
    pub ruby_removed: usize,
    pub crlf_lines: usize,
    pub lf_lines: usize,
    pub cr_lines: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditReport {
    pub project: String,
    pub source_directory: String,
    pub source_encoding: String,
    pub scanned_files: usize,
    pub json_files: usize,
    pub extracted_entries: usize,
    pub dialogue_entries: usize,
    pub monologue_entries: usize,
    pub choice_entries: usize,
    pub ui_entries: usize,
    pub name_entries: usize,
    pub speaker_macros: usize,
    pub speaker_calls: usize,
    pub ruby_removed: usize,
    pub opaque_controls: usize,
    pub warning_count: usize,
    pub violation_count: usize,
    pub files: Vec<FileReport>,
    pub diagnostics: Vec<Diagnostic>,
}

impl AuditReport {
    fn new() -> Self {
        Self {
            project: "tongern".to_string(),
            source_directory: String::new(),
            source_encoding: "cp932".to_string(),
            scanned_files: 0,
            json_files: 0,
            extracted_entries: 0,
            dialogue_entries: 0,
            monologue_entries: 0,
            choice_entries: 0,
            ui_entries: 0,
            name_entries: 0,
            speaker_macros: 0,
            speaker_calls: 0,
            ruby_removed: 0,
            opaque_controls: 0,
            warning_count: 0,
            violation_count: 0,
            files: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn push_diagnostic(
        &mut self,
        severity: &str,
        code: &str,
        file: &str,
        line: usize,
        message: impl Into<String>,
    ) {
        if severity == "violation" {
            self.violation_count += 1;
        } else {
            self.warning_count += 1;
        }
        self.diagnostics.push(Diagnostic {
            severity: severity.to_string(),
            code: code.to_string(),
            file: file.to_string(),
            line,
            message: message.into(),
        });
    }

    fn account_entry(&mut self, entry: &TextEntry) {
        self.extracted_entries += 1;
        self.ruby_removed += entry.ruby_removed;
        self.opaque_controls += entry.controls.len();
        match entry.entry_type.as_str() {
            "dialogue" => self.dialogue_entries += 1,
            "monologue" => self.monologue_entries += 1,
            "choice" => self.choice_entries += 1,
            "ui" | "system" => self.ui_entries += 1,
            "name" => self.name_entries += 1,
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
struct SourceLine {
    number: usize,
    byte_offset: usize,
    text: String,
}

#[derive(Debug, Clone)]
struct SourceFile {
    relative_name: String,
    lines: Vec<SourceLine>,
    crlf_lines: usize,
    lf_lines: usize,
    cr_lines: usize,
}

fn decode_cp932_line(path: &Path, line_no: usize, bytes: &[u8]) -> Result<String> {
    let text = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(bytes)
        .ok_or_else(|| {
            format!(
                "{}:{} is not valid CP932/Shift_JIS",
                path.display(),
                line_no
            )
        })?
        .into_owned();
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&text);
    if had_errors || encoded.as_ref() != bytes {
        return Err(format!(
            "{}:{} failed byte-exact CP932 round trip",
            path.display(),
            line_no
        ));
    }
    Ok(text)
}

fn read_source(path: &Path, relative_name: &str) -> Result<SourceFile> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    if bytes.starts_with(&[0xef, 0xbb, 0xbf])
        || bytes.starts_with(&[0xff, 0xfe])
        || bytes.starts_with(&[0xfe, 0xff])
    {
        return Err(format!(
            "{} has an unsupported Unicode BOM; expected BOM-less CP932",
            path.display()
        ));
    }

    let mut lines = Vec::new();
    let mut start = 0usize;
    let mut cursor = 0usize;
    let mut line_no = 1usize;
    let mut crlf_lines = 0usize;
    let mut lf_lines = 0usize;
    let mut cr_lines = 0usize;

    while cursor < bytes.len() {
        let terminator_len = match bytes[cursor] {
            b'\r' if cursor + 1 < bytes.len() && bytes[cursor + 1] == b'\n' => {
                crlf_lines += 1;
                2usize
            }
            b'\r' => {
                cr_lines += 1;
                1usize
            }
            b'\n' => {
                lf_lines += 1;
                1usize
            }
            _ => {
                cursor += 1;
                continue;
            }
        };
        let text = decode_cp932_line(path, line_no, &bytes[start..cursor])?;
        lines.push(SourceLine {
            number: line_no,
            byte_offset: start,
            text,
        });
        cursor += terminator_len;
        start = cursor;
        line_no += 1;
    }

    if start < bytes.len() {
        let text = decode_cp932_line(path, line_no, &bytes[start..])?;
        lines.push(SourceLine {
            number: line_no,
            byte_offset: start,
            text,
        });
    } else if bytes.is_empty() {
        lines.push(SourceLine {
            number: 1,
            byte_offset: 0,
            text: String::new(),
        });
    }

    Ok(SourceFile {
        relative_name: relative_name.replace('\\', "/"),
        lines,
        crlf_lines,
        lf_lines,
        cr_lines,
    })
}

fn cp932_len(text: &str) -> Result<usize> {
    let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        return Err("internal text cannot be encoded as CP932".to_string());
    }
    Ok(encoded.len())
}

fn absolute_offset(line: &SourceLine, utf8_offset: usize) -> Result<usize> {
    Ok(line.byte_offset + cp932_len(&line.text[..utf8_offset])?)
}

fn ascii_trim_start(text: &str) -> &str {
    text.trim_start_matches([' ', '\t'])
}

fn ascii_trim(text: &str) -> &str {
    text.trim_matches([' ', '\t'])
}

#[derive(Debug, Clone)]
struct AttrToken {
    name: String,
    value: String,
}

#[derive(Debug, Clone)]
struct TagToken {
    raw: String,
    name: String,
    attrs: Vec<AttrToken>,
    end: usize,
}

impl TagToken {
    fn attr(&self, name: &str) -> Option<&AttrToken> {
        self.attrs
            .iter()
            .find(|attr| attr.name.eq_ignore_ascii_case(name))
    }
}

#[derive(Debug, Clone)]
enum InlineToken {
    Text {
        raw: String,
        start: usize,
        end: usize,
    },
    Tag(TagToken),
}

fn find_tag_end(text: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    for (relative, ch) in text[start + 1..].char_indices() {
        match quote {
            Some(open) if ch == open => quote = None,
            Some(_) => {}
            None if ch == '\'' || ch == '"' => quote = Some(ch),
            None if ch == ']' => return Some(start + 1 + relative + ch.len_utf8()),
            None => {}
        }
    }
    None
}

fn parse_attributes(inner: &str) -> Vec<AttrToken> {
    let mut attrs = Vec::new();
    let bytes = inner.as_bytes();
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let key_start = cursor;
        while cursor < bytes.len()
            && (bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'_')
        {
            cursor += 1;
        }
        if cursor == key_start {
            while cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            continue;
        }
        let key = &inner[key_start..cursor];
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] != b'=' {
            continue;
        }
        cursor += 1;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            break;
        }

        let (value_start, value_end) = if bytes[cursor] == b'"' || bytes[cursor] == b'\'' {
            let quote = bytes[cursor];
            cursor += 1;
            let start = cursor;
            while cursor < bytes.len() && bytes[cursor] != quote {
                if bytes[cursor] == b'\\' && cursor + 1 < bytes.len() {
                    cursor += 2;
                } else {
                    cursor += 1;
                }
            }
            let end = cursor.min(bytes.len());
            if cursor < bytes.len() {
                cursor += 1;
            }
            (start, end)
        } else {
            let start = cursor;
            while cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            (start, cursor)
        };
        attrs.push(AttrToken {
            name: key.to_ascii_lowercase(),
            value: inner[value_start..value_end].to_string(),
        });
    }

    attrs
}

fn parse_tag(raw: &str, end: usize) -> TagToken {
    let inner = &raw[1..raw.len() - 1];
    let trimmed = inner.trim_start();
    let name_end = trimmed
        .find(|ch: char| ch.is_whitespace())
        .unwrap_or(trimmed.len());
    let name = trimmed[..name_end].to_ascii_lowercase();
    let attrs_inner = &trimmed[name_end..];
    TagToken {
        raw: raw.to_string(),
        name,
        attrs: parse_attributes(attrs_inner),
        end,
    }
}

fn push_text_token(
    tokens: &mut Vec<InlineToken>,
    raw: &str,
    start: usize,
    end: usize,
    file: &str,
    line: usize,
    report: &mut AuditReport,
) {
    if raw.is_empty() {
        return;
    }
    if raw.contains(']') {
        report.push_diagnostic(
            "violation",
            "stray_closing_bracket",
            file,
            line,
            format!("stray ']' in text: {raw}"),
        );
    }
    if raw.chars().all(|ch| ch.is_whitespace() || ch == ']') {
        return;
    }
    tokens.push(InlineToken::Text {
        raw: raw.to_string(),
        start,
        end,
    });
}

fn tokenize_line(file: &str, line: &SourceLine, report: &mut AuditReport) -> Vec<InlineToken> {
    let text = &line.text;
    let mut tokens = Vec::new();
    let mut cursor = 0usize;

    while cursor < text.len() {
        let Some(relative) = text[cursor..].find('[') else {
            let raw = &text[cursor..];
            push_text_token(
                &mut tokens,
                raw,
                cursor,
                text.len(),
                file,
                line.number,
                report,
            );
            break;
        };
        let tag_start = cursor + relative;
        if tag_start > cursor {
            let raw = &text[cursor..tag_start];
            push_text_token(
                &mut tokens,
                raw,
                cursor,
                tag_start,
                file,
                line.number,
                report,
            );
        }
        let Some(tag_end) = find_tag_end(text, tag_start) else {
            report.push_diagnostic(
                "violation",
                "unclosed_tag",
                file,
                line.number,
                format!("unclosed tag starting at column {}", tag_start + 1),
            );
            tokens.push(InlineToken::Text {
                raw: text[tag_start..].to_string(),
                start: tag_start,
                end: text.len(),
            });
            break;
        };
        let raw = &text[tag_start..tag_end];
        tokens.push(InlineToken::Tag(parse_tag(raw, tag_end)));
        cursor = tag_end;
    }

    tokens
}

fn parse_command(line: &SourceLine) -> Option<TagToken> {
    let trimmed = ascii_trim_start(&line.text);
    if !trimmed.starts_with('@') {
        return None;
    }
    let indent = line.text.len() - trimmed.len();
    let command = trimmed.trim_end_matches('\\');
    let inner = &command[1..];
    let name_end = inner
        .find(|ch: char| ch.is_whitespace())
        .unwrap_or(inner.len());
    let name = inner[..name_end].to_ascii_lowercase();
    let attrs_inner = &inner[name_end..];
    Some(TagToken {
        raw: command.to_string(),
        name,
        attrs: parse_attributes(attrs_inner),
        end: indent + command.len(),
    })
}

fn is_comment_or_label(line: &str) -> bool {
    let trimmed = ascii_trim_start(line);
    trimmed.starts_with(';') || trimmed.starts_with('*')
}

fn is_continuation_line(line: &str) -> bool {
    line.trim_end_matches([' ', '\t']).ends_with('\\')
}

fn is_terminal_wait(
    tokens: &[InlineToken],
    token_index: usize,
    line_has_continuation: bool,
) -> bool {
    if line_has_continuation {
        return false;
    }
    matches!(
        tokens.get(token_index),
        Some(InlineToken::Tag(tag)) if tag.name == "l"
    ) && tokens[token_index + 1..].iter().all(|token| match token {
        InlineToken::Text { raw, .. } => raw.trim().is_empty(),
        InlineToken::Tag(_) => false,
    })
}

fn trim_continuation(raw: &str, is_last_token: bool, line_has_continuation: bool) -> &str {
    if !is_last_token || !line_has_continuation {
        return raw;
    }
    let trimmed = raw.trim_end_matches([' ', '\t']);
    trimmed.strip_suffix('\\').unwrap_or(raw)
}

fn visible_text_exists(text: &str) -> bool {
    text.chars().any(|ch| !ch.is_whitespace())
}

fn trim_outer_text(parts: &mut [String]) {
    if let Some(first) = parts.first_mut() {
        *first = first.trim_start().to_string();
    }
    if let Some(last) = parts.last_mut() {
        *last = last.trim_end().to_string();
    }
}

fn trim_unreferenced_trailing_parts(parts: &mut Vec<String>, controls: &[ControlRecord]) {
    let minimum_parts = controls
        .iter()
        .map(|control| control.after_part + 1)
        .max()
        .unwrap_or(1);
    while parts.last().is_some_and(String::is_empty) && parts.len() > minimum_parts {
        parts.pop();
    }
}

#[derive(Debug, Clone)]
struct SpeakerDefinition {
    macro_name: String,
    display_name: String,
    line: usize,
    end_line: usize,
    offset: usize,
    size: usize,
    parts: Vec<String>,
    controls: Vec<ControlRecord>,
}

#[derive(Debug, Clone)]
struct MacroCapture {
    macro_name: String,
    start_line: usize,
    body: Vec<SourceLine>,
}

fn macro_start(tag: &TagToken) -> Option<String> {
    if tag.name != "macro" {
        return None;
    }
    tag.attr("name").map(|attr| attr.value.clone())
}

fn line_primary_tag(line: &SourceLine) -> Option<TagToken> {
    if let Some(command) = parse_command(line) {
        return Some(command);
    }
    let trimmed = ascii_trim_start(&line.text);
    if !trimmed.starts_with('[') {
        return None;
    }
    let indent = line.text.len() - trimmed.len();
    let end = find_tag_end(&line.text, indent)?;
    let rest = ascii_trim(&line.text[end..]);
    if !rest.is_empty() && rest != "\\" {
        return None;
    }
    Some(parse_tag(&line.text[indent..end], end))
}

fn parse_speaker_definitions(
    source: &SourceFile,
    report: &mut AuditReport,
) -> Result<Vec<SpeakerDefinition>> {
    let mut definitions = Vec::new();
    let mut capture: Option<MacroCapture> = None;

    for line in &source.lines {
        let primary = line_primary_tag(line);
        if capture.is_none() {
            if let Some(tag) = primary.as_ref() {
                if let Some(macro_name) = macro_start(tag) {
                    capture = Some(MacroCapture {
                        macro_name,
                        start_line: line.number,
                        body: Vec::new(),
                    });
                }
            }
            continue;
        }

        if primary.as_ref().is_some_and(|tag| tag.name == "endmacro") {
            let completed = capture.take().expect("capture exists");
            if let Some(definition) =
                speaker_definition_from_capture(source, completed, line.number, report)?
            {
                definitions.push(definition);
            }
        } else if let Some(active) = capture.as_mut() {
            if primary.as_ref().is_some_and(|tag| tag.name == "macro") {
                report.push_diagnostic(
                    "violation",
                    "nested_macro",
                    &source.relative_name,
                    line.number,
                    "nested macro definition",
                );
            }
            active.body.push(line.clone());
        }
    }

    if let Some(active) = capture {
        report.push_diagnostic(
            "violation",
            "unclosed_macro",
            &source.relative_name,
            active.start_line,
            format!("macro '{}' has no endmacro", active.macro_name),
        );
    }

    Ok(definitions)
}

fn speaker_definition_from_capture(
    source: &SourceFile,
    capture: MacroCapture,
    end_line: usize,
    report: &mut AuditReport,
) -> Result<Option<SpeakerDefinition>> {
    let mut in_message1 = false;
    let mut saw_message1 = false;
    let mut saw_message0_after = false;
    let mut parts = vec![String::new()];
    let mut controls = Vec::new();
    let mut first_offset = None;
    let mut last_offset = None;
    let mut display_line = capture.start_line;

    for line in &capture.body {
        if is_comment_or_label(&line.text) || parse_command(line).is_some() {
            continue;
        }
        let tokens = tokenize_line(&source.relative_name, line, report);
        let continuation = is_continuation_line(&line.text);
        for (token_index, token) in tokens.iter().enumerate() {
            match token {
                InlineToken::Tag(tag) if tag.name == "current" => {
                    let layer = tag.attr("layer").map(|attr| attr.value.as_str());
                    match layer {
                        Some(value) if value.eq_ignore_ascii_case("message1") => {
                            in_message1 = true;
                            saw_message1 = true;
                        }
                        Some(value) if value.eq_ignore_ascii_case("message0") => {
                            if saw_message1 {
                                saw_message0_after = true;
                            }
                            in_message1 = false;
                        }
                        _ => {}
                    }
                }
                InlineToken::Tag(tag) if in_message1 => {
                    if visible_text_exists(parts.last().expect("part exists")) {
                        controls.push(ControlRecord {
                            after_part: parts.len() - 1,
                            line: line.number,
                            tag: tag.name.clone(),
                            raw: tag.raw.clone(),
                        });
                        parts.push(String::new());
                    }
                }
                InlineToken::Text { raw, start, end } if in_message1 => {
                    let text =
                        trim_continuation(raw, token_index + 1 == tokens.len(), continuation);
                    if text.chars().any(|ch| !ch.is_whitespace()) {
                        if first_offset.is_none() {
                            first_offset = Some(absolute_offset(line, *start)?);
                            display_line = line.number;
                        }
                        last_offset = Some(absolute_offset(line, *end)?);
                    }
                    parts.last_mut().expect("part exists").push_str(text);
                }
                _ => {}
            }
        }
    }

    if !(saw_message1 && saw_message0_after) {
        return Ok(None);
    }

    trim_outer_text(&mut parts);
    trim_unreferenced_trailing_parts(&mut parts, &controls);
    let display_name = parts.concat();
    if !visible_text_exists(&display_name) {
        report.push_diagnostic(
            "violation",
            "speaker_macro_without_name",
            &source.relative_name,
            capture.start_line,
            format!(
                "speaker-shaped macro '{}' has no display text",
                capture.macro_name
            ),
        );
        return Ok(None);
    }
    let offset = first_offset.unwrap_or(0);
    let end_offset = last_offset.unwrap_or(offset);
    Ok(Some(SpeakerDefinition {
        macro_name: capture.macro_name,
        display_name,
        line: display_line,
        end_line,
        offset,
        size: end_offset.saturating_sub(offset),
        parts,
        controls,
    }))
}

#[derive(Debug, Clone)]
struct MessageBuilder {
    parts: Vec<String>,
    controls: Vec<ControlRecord>,
    start_line: Option<usize>,
    end_line: usize,
    start_offset: Option<usize>,
    end_offset: usize,
    ruby_removed: usize,
    pending_ruby_line: Option<usize>,
}

impl MessageBuilder {
    fn new() -> Self {
        Self {
            parts: vec![String::new()],
            controls: Vec::new(),
            start_line: None,
            end_line: 0,
            start_offset: None,
            end_offset: 0,
            ruby_removed: 0,
            pending_ruby_line: None,
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn has_visible_text(&self) -> bool {
        self.parts
            .iter()
            .any(|part| visible_text_exists(part.as_str()))
    }

    fn mark_ruby(&mut self, file: &str, line: usize, report: &mut AuditReport) {
        if let Some(previous) = self.pending_ruby_line {
            report.push_diagnostic(
                "violation",
                "consecutive_ruby",
                file,
                line,
                format!("ruby tag appears before the ruby at line {previous} receives base text"),
            );
        }
        self.ruby_removed += 1;
        self.pending_ruby_line = Some(line);
    }

    fn add_text(&mut self, raw: &str, line: &SourceLine, start: usize, end: usize) -> Result<()> {
        if raw.is_empty() {
            return Ok(());
        }
        let contributes = raw.chars().any(|ch| !ch.is_whitespace() && ch != '\\');
        if contributes {
            if self.start_line.is_none() {
                self.start_line = Some(line.number);
                self.start_offset = Some(absolute_offset(line, start)?);
            }
            self.end_line = line.number;
            self.end_offset = absolute_offset(line, end)?;
            if self.pending_ruby_line.is_some()
                && raw.chars().any(|ch| !ch.is_whitespace() && ch != '\\')
            {
                self.pending_ruby_line = None;
            }
        }
        self.parts.last_mut().expect("part exists").push_str(raw);
        Ok(())
    }

    fn add_opaque_control(&mut self, tag: &TagToken, line: &SourceLine) -> Result<()> {
        if !self.has_visible_text() {
            return Ok(());
        }
        self.controls.push(ControlRecord {
            after_part: self.parts.len() - 1,
            line: line.number,
            tag: tag.name.clone(),
            raw: tag.raw.clone(),
        });
        self.parts.push(String::new());
        self.end_line = line.number;
        self.end_offset = absolute_offset(line, tag.end)?;
        Ok(())
    }

    fn finish(
        &mut self,
        context: EntryContext<'_>,
        boundary: &str,
        report: &mut AuditReport,
    ) -> Option<TextEntry> {
        if let Some(ruby_line) = self.pending_ruby_line {
            report.push_diagnostic(
                "violation",
                "orphan_ruby",
                context.file,
                ruby_line,
                "ruby tag has no following base character before the text boundary",
            );
        }

        if !self.has_visible_text() {
            if boundary == "p" {
                report.push_diagnostic(
                    "warning",
                    "empty_page",
                    context.file,
                    context.boundary_line,
                    "page break has no visible text",
                );
            }
            self.reset();
            return None;
        }

        trim_outer_text(&mut self.parts);
        trim_unreferenced_trailing_parts(&mut self.parts, &self.controls);
        let scr_msg = self.parts.concat();
        if !visible_text_exists(&scr_msg) {
            self.reset();
            return None;
        }
        if boundary != "p"
            && boundary != "link"
            && boundary != "attribute"
            && boundary != "string"
            && boundary != "macro_definition"
        {
            report.push_diagnostic(
                "warning",
                "implicit_text_boundary",
                context.file,
                context.boundary_line,
                format!("visible text ended by '{boundary}' instead of [p]"),
            );
        }

        let start = self.start_offset.unwrap_or(0);
        let parts = if self.controls.is_empty() {
            None
        } else {
            Some(self.parts.clone())
        };
        let entry = TextEntry {
            file: context.file.to_string(),
            index: 0,
            line: self.start_line.unwrap_or(context.boundary_line),
            end_line: self.end_line.max(context.boundary_line),
            offset: start,
            size: self.end_offset.saturating_sub(start),
            entry_type: context.entry_type.to_string(),
            encoding: "cp932".to_string(),
            boundary: boundary.to_string(),
            source_kind: context.source_kind.to_string(),
            speaker_macro: context.speaker_macro.map(str::to_string),
            target: context.target.map(str::to_string),
            ruby_removed: self.ruby_removed,
            controls: self.controls.clone(),
            scr_name: context.name.map(str::to_string),
            name: context.name.map(str::to_string),
            scr_msg: scr_msg.clone(),
            message: scr_msg,
            scr_msg_parts: parts.clone(),
            message_parts: parts,
        };
        self.reset();
        Some(entry)
    }
}

#[derive(Debug, Clone, Copy)]
struct EntryContext<'a> {
    file: &'a str,
    entry_type: &'a str,
    source_kind: &'a str,
    speaker_macro: Option<&'a str>,
    name: Option<&'a str>,
    target: Option<&'a str>,
    boundary_line: usize,
}

#[derive(Debug, Clone)]
struct LinkState {
    builder: MessageBuilder,
    target: Option<String>,
    start_line: usize,
    entry_type: Option<String>,
}

fn link_kind(file: &str) -> Option<&'static str> {
    if file.eq_ignore_ascii_case("7th.ks") {
        Some("choice")
    } else {
        None
    }
}

fn is_macro_definition_boundary(name: &str) -> bool {
    name == "macro" || name == "endmacro"
}

fn is_cm_boundary(name: &str) -> bool {
    matches!(name, "cm" | "cm1" | "cm2" | "ct")
}

fn is_er_boundary(name: &str) -> bool {
    matches!(name, "er" | "er1" | "er2")
}

fn is_stop_boundary(name: &str) -> bool {
    matches!(name, "s" | "return" | "jump")
}

fn is_speaker_invocation(
    tokens: &[InlineToken],
    speakers: &HashMap<String, SpeakerDefinition>,
) -> Option<String> {
    match tokens {
        [InlineToken::Tag(tag), InlineToken::Text { raw, .. }]
            if ascii_trim(raw) == "\\" && speakers.contains_key(&tag.name) =>
        {
            Some(tag.name.clone())
        }
        [InlineToken::Tag(tag)] if speakers.contains_key(&tag.name) => Some(tag.name.clone()),
        _ => None,
    }
}

fn initial_message_region(file: &str) -> bool {
    !matches!(
        file.to_ascii_lowercase().as_str(),
        "first.ks"
            | "first2nd.ks"
            | "about.ks"
            | "rclick_test.ks"
            | "gvolume.ks"
            | "exhistorylayer.ks"
            | "autoinsertlabel.ks"
            | "changecolorlink.ks"
            | "rain.ks"
            | "swaplinkcolor.ks"
            | "zoom.ks"
    )
}

fn label_message_region(file: &str, line: &str) -> Option<bool> {
    if !file.eq_ignore_ascii_case("first2nd.ks") {
        return None;
    }
    let label = ascii_trim_start(line)
        .strip_prefix('*')?
        .split('|')
        .next()?
        .trim();
    if label == "0001" {
        Some(true)
    } else if label.eq_ignore_ascii_case("config3") {
        Some(false)
    } else {
        None
    }
}

fn collect_ks_files(input_dir: &Path) -> Result<Vec<(PathBuf, String)>> {
    if !input_dir.is_dir() {
        return Err(format!("{} is not a directory", input_dir.display()));
    }
    let mut files = fs::read_dir(input_dir)
        .map_err(|error| format!("failed to read {}: {error}", input_dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read directory entry: {error}"))?;
    files.sort_by_key(|entry| entry.file_name());

    let mut result = Vec::new();
    for entry in files {
        let path = entry.path();
        if entry
            .file_type()
            .map_err(|error| format!("failed to stat {}: {error}", path.display()))?
            .is_file()
            && path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("ks"))
        {
            result.push((path, entry.file_name().to_string_lossy().into_owned()));
        }
    }
    Ok(result)
}

struct MessageEnvironment<'a> {
    file: &'a str,
    speakers: &'a HashMap<String, SpeakerDefinition>,
}

fn emit_current_message(
    builder: &mut MessageBuilder,
    speaker: Option<&str>,
    environment: &MessageEnvironment<'_>,
    boundary: &str,
    boundary_line: usize,
    entries: &mut Vec<TextEntry>,
    report: &mut AuditReport,
) -> bool {
    let definition = speaker.and_then(|name| environment.speakers.get(name));
    let entry_type = if definition.is_some() {
        "dialogue"
    } else {
        "monologue"
    };
    let context = EntryContext {
        file: environment.file,
        entry_type,
        source_kind: "kag_message",
        speaker_macro: definition.map(|definition| definition.macro_name.as_str()),
        name: definition.map(|definition| definition.display_name.as_str()),
        target: None,
        boundary_line,
    };
    if let Some(entry) = builder.finish(context, boundary, report) {
        let dialogue = entry.entry_type == "dialogue";
        entries.push(entry);
        dialogue
    } else {
        false
    }
}

fn emit_link(
    state: &mut LinkState,
    file: &str,
    boundary_line: usize,
    entries: &mut Vec<TextEntry>,
    report: &mut AuditReport,
) {
    let Some(entry_type) = state.entry_type.as_deref() else {
        state.builder.reset();
        return;
    };
    let context = EntryContext {
        file,
        entry_type,
        source_kind: "kag_link_body",
        speaker_macro: None,
        name: None,
        target: state.target.as_deref(),
        boundary_line,
    };
    if let Some(entry) = state.builder.finish(context, "link", report) {
        entries.push(entry);
    }
}

fn extract_file_entries(
    source: &SourceFile,
    speakers: &HashMap<String, SpeakerDefinition>,
    name_entries: Option<&[SpeakerDefinition]>,
    report: &mut AuditReport,
) -> Result<Vec<TextEntry>> {
    let mut entries = Vec::new();
    if let Some(definitions) = name_entries {
        for definition in definitions {
            let parts = if definition.controls.is_empty() {
                None
            } else {
                Some(definition.parts.clone())
            };
            entries.push(TextEntry {
                file: source.relative_name.clone(),
                index: 0,
                line: definition.line,
                end_line: definition.end_line,
                offset: definition.offset,
                size: definition.size,
                entry_type: "name".to_string(),
                encoding: "cp932".to_string(),
                boundary: "macro_definition".to_string(),
                source_kind: "speaker_macro_definition".to_string(),
                speaker_macro: Some(definition.macro_name.clone()),
                target: None,
                ruby_removed: 0,
                controls: definition.controls.clone(),
                scr_name: None,
                name: None,
                scr_msg: definition.display_name.clone(),
                message: definition.display_name.clone(),
                scr_msg_parts: parts.clone(),
                message_parts: parts,
            });
        }
    }

    let mut builder = MessageBuilder::new();
    let mut link_state: Option<LinkState> = None;
    let mut speaker: Option<String> = None;
    let mut speaker_line = 0usize;
    let mut speaker_has_text = true;
    let mut in_iscript = false;
    let mut in_macro = false;
    let mut message_region = initial_message_region(&source.relative_name);
    let message_environment = MessageEnvironment {
        file: &source.relative_name,
        speakers,
    };

    for line in &source.lines {
        let primary = line_primary_tag(line);
        if in_macro {
            if primary.as_ref().is_some_and(|tag| tag.name == "endmacro") {
                in_macro = false;
            }
            continue;
        }
        if primary.as_ref().is_some_and(|tag| tag.name == "macro") {
            in_macro = true;
            continue;
        }

        if in_iscript {
            if primary.as_ref().is_some_and(|tag| tag.name == "endscript") {
                in_iscript = false;
                continue;
            }
            continue;
        }
        if primary.as_ref().is_some_and(|tag| tag.name == "iscript") {
            in_iscript = true;
            continue;
        }

        if let Some(enabled) = label_message_region(&source.relative_name, &line.text) {
            message_region = enabled;
        }
        if is_comment_or_label(&line.text) || line.text.trim().is_empty() {
            continue;
        }

        if let Some(command) = parse_command(line) {
            if !message_region {
                continue;
            }
            if is_cm_boundary(&command.name) {
                let emitted = emit_current_message(
                    &mut builder,
                    speaker.as_deref(),
                    &message_environment,
                    "cm",
                    line.number,
                    &mut entries,
                    report,
                );
                speaker_has_text |= emitted;
                if speaker.is_some() && !speaker_has_text {
                    report.push_diagnostic(
                        "warning",
                        "speaker_without_text",
                        &source.relative_name,
                        speaker_line,
                        "speaker state was cleared before visible text was emitted",
                    );
                }
                speaker = None;
                speaker_has_text = true;
            } else if is_er_boundary(&command.name) {
                let emitted = emit_current_message(
                    &mut builder,
                    speaker.as_deref(),
                    &message_environment,
                    "er",
                    line.number,
                    &mut entries,
                    report,
                );
                speaker_has_text |= emitted;
            } else if is_stop_boundary(&command.name) {
                let emitted = emit_current_message(
                    &mut builder,
                    speaker.as_deref(),
                    &message_environment,
                    &command.name,
                    line.number,
                    &mut entries,
                    report,
                );
                if speaker.is_some() && !(speaker_has_text || emitted) {
                    report.push_diagnostic(
                        "warning",
                        "speaker_without_text",
                        &source.relative_name,
                        speaker_line,
                        "speaker state stopped before visible text was emitted",
                    );
                }
                speaker = None;
                speaker_has_text = true;
            }
            continue;
        }

        let tokens = tokenize_line(&source.relative_name, line, report);
        if message_region {
            if let Some(invocation) = is_speaker_invocation(&tokens, speakers) {
                if builder.has_visible_text() {
                    let emitted = emit_current_message(
                        &mut builder,
                        speaker.as_deref(),
                        &message_environment,
                        "new_speaker",
                        line.number,
                        &mut entries,
                        report,
                    );
                    speaker_has_text |= emitted;
                }
                if speaker.is_some() && !speaker_has_text {
                    report.push_diagnostic(
                        "warning",
                        "speaker_without_text",
                        &source.relative_name,
                        speaker_line,
                        "speaker was replaced before visible text was emitted",
                    );
                }
                if link_state.is_some() {
                    report.push_diagnostic(
                        "violation",
                        "speaker_inside_link",
                        &source.relative_name,
                        line.number,
                        "speaker invocation encountered before endlink",
                    );
                }
                speaker = Some(invocation);
                speaker_line = line.number;
                speaker_has_text = false;
                report.speaker_calls += 1;
                continue;
            }
        }

        let continuation = is_continuation_line(&line.text);
        let mut page_breaks = 0usize;

        for (token_index, token) in tokens.iter().enumerate() {
            match token {
                InlineToken::Tag(tag)
                    if matches!(tag.name.as_str(), "link" | "changecolorlink") =>
                {
                    if link_state.is_some() {
                        report.push_diagnostic(
                            "violation",
                            "nested_link",
                            &source.relative_name,
                            line.number,
                            "nested link start",
                        );
                    } else {
                        link_state = Some(LinkState {
                            builder: MessageBuilder::new(),
                            target: tag.attr("target").map(|attr| attr.value.clone()),
                            start_line: line.number,
                            entry_type: link_kind(&source.relative_name).map(str::to_string),
                        });
                    }
                }
                InlineToken::Tag(tag) if tag.name == "endlink" => {
                    if let Some(mut state) = link_state.take() {
                        emit_link(
                            &mut state,
                            &source.relative_name,
                            line.number,
                            &mut entries,
                            report,
                        );
                    } else {
                        report.push_diagnostic(
                            "violation",
                            "endlink_without_link",
                            &source.relative_name,
                            line.number,
                            "endlink appears without a matching link",
                        );
                    }
                }
                InlineToken::Text { raw, start, end } => {
                    let text =
                        trim_continuation(raw, token_index + 1 == tokens.len(), continuation);
                    if let Some(state) = link_state.as_mut() {
                        if state.entry_type.is_some() {
                            state.builder.add_text(text, line, *start, *end)?;
                        }
                    } else if message_region {
                        builder.add_text(text, line, *start, *end)?;
                    }
                }
                InlineToken::Tag(tag) if link_state.is_some() => {
                    let state = link_state.as_mut().expect("checked");
                    if state.entry_type.is_none() {
                        continue;
                    }
                    match tag.name.as_str() {
                        "ruby" => {
                            state
                                .builder
                                .mark_ruby(&source.relative_name, line.number, report)
                        }
                        "l" if is_terminal_wait(&tokens, token_index, continuation) => {
                            state.builder.add_opaque_control(tag, line)?;
                        }
                        "l" | "r" => {}
                        _ => state.builder.add_opaque_control(tag, line)?,
                    }
                }
                InlineToken::Tag(tag) if message_region => match tag.name.as_str() {
                    "ruby" => builder.mark_ruby(&source.relative_name, line.number, report),
                    "l" if is_terminal_wait(&tokens, token_index, continuation) => {
                        builder.add_opaque_control(tag, line)?;
                    }
                    "l" | "r" => {}
                    "p" => {
                        page_breaks += 1;
                        let emitted = emit_current_message(
                            &mut builder,
                            speaker.as_deref(),
                            &message_environment,
                            "p",
                            line.number,
                            &mut entries,
                            report,
                        );
                        speaker_has_text |= emitted;
                    }
                    name if is_er_boundary(name) => {
                        let emitted = emit_current_message(
                            &mut builder,
                            speaker.as_deref(),
                            &message_environment,
                            "er",
                            line.number,
                            &mut entries,
                            report,
                        );
                        speaker_has_text |= emitted;
                    }
                    name if is_cm_boundary(name) => {
                        let emitted = emit_current_message(
                            &mut builder,
                            speaker.as_deref(),
                            &message_environment,
                            "cm",
                            line.number,
                            &mut entries,
                            report,
                        );
                        speaker_has_text |= emitted;
                        if speaker.is_some() && !speaker_has_text {
                            report.push_diagnostic(
                                "warning",
                                "speaker_without_text",
                                &source.relative_name,
                                speaker_line,
                                "speaker state was cleared before visible text was emitted",
                            );
                        }
                        speaker = None;
                        speaker_has_text = true;
                    }
                    name if is_stop_boundary(name) => {
                        let emitted = emit_current_message(
                            &mut builder,
                            speaker.as_deref(),
                            &message_environment,
                            name,
                            line.number,
                            &mut entries,
                            report,
                        );
                        if speaker.is_some() && !(speaker_has_text || emitted) {
                            report.push_diagnostic(
                                "warning",
                                "speaker_without_text",
                                &source.relative_name,
                                speaker_line,
                                "speaker state stopped before visible text was emitted",
                            );
                        }
                        speaker = None;
                        speaker_has_text = true;
                    }
                    name if is_macro_definition_boundary(name) => {}
                    _ => builder.add_opaque_control(tag, line)?,
                },
                _ => {}
            }
        }

        if page_breaks > 1 {
            report.push_diagnostic(
                "warning",
                "multiple_page_breaks",
                &source.relative_name,
                line.number,
                format!("{page_breaks} [p] tags occur on one physical line"),
            );
        }
    }

    if in_iscript {
        report.push_diagnostic(
            "violation",
            "unclosed_iscript",
            &source.relative_name,
            source.lines.last().map_or(1, |line| line.number),
            "iscript block reaches end of file",
        );
    }
    if in_macro {
        report.push_diagnostic(
            "violation",
            "unclosed_macro",
            &source.relative_name,
            source.lines.last().map_or(1, |line| line.number),
            "macro block reaches end of file",
        );
    }
    if let Some(mut state) = link_state {
        report.push_diagnostic(
            "violation",
            "unclosed_link",
            &source.relative_name,
            state.start_line,
            "link reaches end of file without endlink",
        );
        emit_link(
            &mut state,
            &source.relative_name,
            source.lines.last().map_or(1, |line| line.number),
            &mut entries,
            report,
        );
    }
    let eof_line = source.lines.last().map_or(1, |line| line.number);
    let emitted = emit_current_message(
        &mut builder,
        speaker.as_deref(),
        &message_environment,
        "eof",
        eof_line,
        &mut entries,
        report,
    );
    speaker_has_text |= emitted;
    if speaker.is_some() && !speaker_has_text {
        report.push_diagnostic(
            "warning",
            "speaker_without_text",
            &source.relative_name,
            speaker_line,
            "speaker reaches end of file without visible text",
        );
    }

    for (index, entry) in entries.iter_mut().enumerate() {
        entry.index = index;
    }
    Ok(entries)
}

fn json_path(output_dir: &Path, source_name: &str) -> PathBuf {
    output_dir.join(format!("{source_name}.json"))
}

fn write_utf8_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut json = serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    json.push('\n');
    fs::write(path, json.as_bytes())
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn count_types(entries: &[TextEntry], entry_type: &str) -> usize {
    entries
        .iter()
        .filter(|entry| entry.entry_type == entry_type)
        .count()
}

fn read_utf8_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        return Err(format!(
            "{} is UTF-16; translation JSON must be UTF-8",
            path.display()
        ));
    }
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse UTF-8 JSON {}: {error}", path.display()))
}

fn json_source_path(path: &Path) -> Result<String> {
    let canonical = fs::canonicalize(path)
        .map_err(|error| format!("failed to resolve {}: {error}", path.display()))?;
    let value = canonical.to_string_lossy();
    #[cfg(windows)]
    {
        if let Some(stripped) = value.strip_prefix(r"\\?\UNC\") {
            return Ok(format!(r"\\{stripped}"));
        }
        if let Some(stripped) = value.strip_prefix(r"\\?\") {
            return Ok(stripped.to_string());
        }
    }
    Ok(value.into_owned())
}

struct ParsedDirectory {
    sources: Vec<(PathBuf, SourceFile)>,
    entries: BTreeMap<String, Vec<TextEntry>>,
    report: AuditReport,
}

fn parse_directory(input_dir: &Path, macro_path: &Path) -> Result<ParsedDirectory> {
    if !macro_path.is_file() {
        return Err(format!(
            "speaker macro file does not exist: {}",
            macro_path.display()
        ));
    }

    let file_paths = collect_ks_files(input_dir)?;
    if file_paths.is_empty() {
        return Err(format!("no .ks files found in {}", input_dir.display()));
    }

    let mut report = AuditReport::new();
    report.source_directory = json_source_path(input_dir)?;
    let mut sources = Vec::new();
    for (path, name) in &file_paths {
        sources.push((path.clone(), read_source(path, name)?));
    }
    report.scanned_files = sources.len();

    let macro_name = macro_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| format!("invalid macro path: {}", macro_path.display()))?;
    let canonical_macro = fs::canonicalize(macro_path)
        .map_err(|error| format!("failed to resolve {}: {error}", macro_path.display()))?;
    let macro_source = if let Some((_, source)) = sources.iter().find(|(path, _)| {
        fs::canonicalize(path).is_ok_and(|candidate| candidate == canonical_macro)
    }) {
        source.clone()
    } else {
        read_source(macro_path, &macro_name)?
    };
    let definitions = parse_speaker_definitions(&macro_source, &mut report)?;
    report.speaker_macros = definitions.len();

    let mut speakers = HashMap::new();
    for definition in &definitions {
        let key = definition.macro_name.to_ascii_lowercase();
        if let Some(previous) = speakers.insert(key.clone(), definition.clone()) {
            report.push_diagnostic(
                "violation",
                "duplicate_speaker_macro",
                &macro_source.relative_name,
                definition.line,
                format!(
                    "speaker macro '{}' duplicates definition at line {}",
                    definition.macro_name, previous.line
                ),
            );
        }
    }

    let mut all_entries: BTreeMap<String, Vec<TextEntry>> = BTreeMap::new();
    for (_, source) in &sources {
        let is_macro_source = source.relative_name.eq_ignore_ascii_case(&macro_name);
        let entries = extract_file_entries(
            source,
            &speakers,
            is_macro_source.then_some(definitions.as_slice()),
            &mut report,
        )?;
        let file_report = FileReport {
            file: source.relative_name.clone(),
            entries: entries.len(),
            dialogue: count_types(&entries, "dialogue"),
            monologue: count_types(&entries, "monologue"),
            choice: count_types(&entries, "choice"),
            ui: count_types(&entries, "ui") + count_types(&entries, "system"),
            name: count_types(&entries, "name"),
            ruby_removed: entries.iter().map(|entry| entry.ruby_removed).sum(),
            crlf_lines: source.crlf_lines,
            lf_lines: source.lf_lines,
            cr_lines: source.cr_lines,
        };
        for entry in &entries {
            report.account_entry(entry);
        }
        report.files.push(file_report);
        all_entries.insert(source.relative_name.clone(), entries);
    }

    report
        .diagnostics
        .sort_by(|a, b| (&a.file, a.line, &a.code).cmp(&(&b.file, b.line, &b.code)));
    report.json_files = all_entries.len();
    Ok(ParsedDirectory {
        sources,
        entries: all_entries,
        report,
    })
}

pub fn extract_directory(options: &ExtractOptions) -> Result<AuditReport> {
    if options.output_dir.exists() {
        return Err(format!(
            "refusing to overwrite existing output directory: {}",
            options.output_dir.display()
        ));
    }
    let parsed = parse_directory(&options.input_dir, &options.macro_path)?;

    fs::create_dir_all(&options.output_dir)
        .map_err(|error| format!("failed to create {}: {error}", options.output_dir.display()))?;
    for (source_name, entries) in &parsed.entries {
        write_utf8_json(&json_path(&options.output_dir, source_name), entries)?;
    }
    write_utf8_json(&options.output_dir.join("audit.json"), &parsed.report)?;
    Ok(parsed.report)
}

#[derive(Deserialize)]
struct AuditSource {
    source_directory: String,
}

pub fn audit_source_directory(json_dir: &Path) -> Result<PathBuf> {
    let audit_path = json_dir.join("audit.json");
    if !audit_path.is_file() {
        return Err(format!(
            "JSON directory has no audit.json: {}",
            json_dir.display()
        ));
    }
    let audit: AuditSource = read_utf8_json(&audit_path)?;
    if audit.source_directory.is_empty() {
        return Err(format!(
            "{} does not record source_directory; extract again with this tool",
            audit_path.display()
        ));
    }
    Ok(PathBuf::from(audit.source_directory))
}

fn take_quoted_string<'a>(input: &'a str, context: &str) -> Result<(String, &'a str)> {
    if !input.starts_with('"') {
        return Err(format!("{context}: expected a quoted string"));
    }
    let mut escaped = false;
    for (relative, ch) in input[1..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            let end = 1 + relative + ch.len_utf8();
            let value = serde_json::from_str::<String>(&input[..end])
                .map_err(|error| format!("{context}: invalid quoted string: {error}"))?;
            return Ok((value, &input[end..]));
        }
    }
    Err(format!("{context}: unclosed quoted string"))
}

fn parse_name_dictionary(path: &Path) -> Result<HashMap<String, String>> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {} as UTF-8: {error}", path.display()))?;
    let mut names = HashMap::new();
    for (line_index, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let context = format!("{}:{}", path.display(), line_index + 1);
        let (source, rest) = take_quoted_string(line, &context)?;
        let rest = rest
            .trim_start()
            .strip_prefix('=')
            .ok_or_else(|| format!("{context}: expected '='"))?
            .trim_start()
            .strip_prefix('[')
            .ok_or_else(|| format!("{context}: expected '['"))?
            .trim_start();
        let (target, rest) = take_quoted_string(rest, &context)?;
        let count_text = rest
            .trim_start()
            .strip_prefix(',')
            .ok_or_else(|| format!("{context}: expected dictionary count"))?
            .trim_start()
            .strip_suffix(']')
            .ok_or_else(|| format!("{context}: expected closing ']'"))?
            .trim();
        count_text
            .parse::<usize>()
            .map_err(|error| format!("{context}: invalid dictionary count: {error}"))?;
        if names.insert(source.clone(), target).is_some() {
            return Err(format!("{context}: duplicate name key '{source}'"));
        }
    }
    if names.is_empty() {
        return Err(format!("name dictionary is empty: {}", path.display()));
    }
    Ok(names)
}

fn decode_tolerant_json_inner(
    inner: &str,
    context: &str,
    repaired_quotes: &mut usize,
) -> Result<String> {
    let mut quoted = String::with_capacity(inner.len() + 2);
    quoted.push('"');
    let mut escaped = false;
    for ch in inner.chars() {
        if escaped {
            quoted.push(ch);
            escaped = false;
        } else if ch == '\\' {
            quoted.push(ch);
            escaped = true;
        } else if ch == '"' {
            quoted.push_str("\\\"");
            *repaired_quotes += 1;
        } else if ch.is_control() {
            let encoded = serde_json::to_string(&ch.to_string())
                .map_err(|error| format!("{context}: failed to encode control: {error}"))?;
            quoted.push_str(&encoded[1..encoded.len() - 1]);
        } else {
            quoted.push(ch);
        }
    }
    if escaped {
        return Err(format!("{context}: translated string ends with bare '\\'"));
    }
    quoted.push('"');
    serde_json::from_str(&quoted)
        .map_err(|error| format!("{context}: translated string cannot be repaired: {error}"))
}

fn extract_pretty_property_strings(path: &Path, property: &str) -> Result<(Vec<String>, usize)> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {} as UTF-8: {error}", path.display()))?;
    let prefix = format!("    \"{property}\": ");
    let mut values = Vec::new();
    let mut repaired_quotes = 0usize;
    for (line_index, raw_line) in text.lines().enumerate() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let Some(raw_value) = line.strip_prefix(&prefix) else {
            continue;
        };
        let without_comma = raw_value.strip_suffix(',').unwrap_or(raw_value);
        if !without_comma.starts_with('"') || !without_comma.ends_with('"') {
            return Err(format!(
                "{}:{}: {property} must remain on one quoted line",
                path.display(),
                line_index + 1
            ));
        }
        let inner = &without_comma[1..without_comma.len() - 1];
        values.push(decode_tolerant_json_inner(
            inner,
            &format!("{}:{} {property}", path.display(), line_index + 1),
            &mut repaired_quotes,
        )?);
    }
    Ok((values, repaired_quotes))
}

fn is_boundary_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '。' | '！'
            | '？'
            | '…'
            | '―'
            | '」'
            | '』'
            | '）'
            | '】'
            | '》'
            | '〉'
            | '”'
            | '’'
            | '、'
            | '，'
            | '；'
            | '：'
            | '.'
            | '!'
            | '?'
            | ')'
            | ']'
            | '"'
            | '\''
    )
}

fn split_translation_parts(source_parts: &[String], translation: &str) -> Vec<String> {
    if source_parts.len() <= 1 {
        return vec![translation.to_string()];
    }

    let translated = translation.chars().collect::<Vec<_>>();
    let source_lengths = source_parts
        .iter()
        .map(|part| part.chars().count())
        .collect::<Vec<_>>();
    let source_total = source_lengths.iter().sum::<usize>().max(1);
    let mut source_cumulative = 0usize;
    let mut previous = 0usize;
    let mut boundaries = Vec::with_capacity(source_parts.len() - 1);

    for part_index in 0..source_parts.len() - 1 {
        source_cumulative += source_lengths[part_index];
        let desired = ((source_cumulative as f64 / source_total as f64) * translated.len() as f64)
            .round() as usize;
        let source_before = source_parts[part_index].chars().last();
        let source_after = source_parts[part_index + 1].chars().next();
        let source_part_nonempty = source_lengths[part_index] > 0;
        let source_remaining = source_lengths[part_index + 1..].iter().sum::<usize>() > 0;

        let mut best_position = previous.min(translated.len());
        let mut best_score = i64::MAX;
        for position in previous..=translated.len() {
            let mut score = (position.abs_diff(desired) as i64) * 10;
            let translated_before = position
                .checked_sub(1)
                .and_then(|index| translated.get(index))
                .copied();
            let translated_after = translated.get(position).copied();

            if source_before.is_some() && source_before == translated_before {
                score -= 100;
            } else if source_before.is_some_and(is_boundary_punctuation)
                && translated_before.is_some_and(is_boundary_punctuation)
            {
                score -= 45;
            } else if source_before.is_some_and(is_boundary_punctuation) {
                score += 30;
            }
            if source_after.is_some() && source_after == translated_after {
                score -= 40;
            } else if source_after.is_some_and(is_boundary_punctuation)
                && translated_after.is_some_and(is_boundary_punctuation)
            {
                score -= 20;
            }
            if source_part_nonempty && position == previous && previous < translated.len() {
                score += 80;
            }
            if source_remaining && position == translated.len() {
                score += 80;
            }
            if score < best_score {
                best_score = score;
                best_position = position;
            }
        }
        boundaries.push(best_position);
        previous = best_position;
    }

    let mut parts = Vec::with_capacity(source_parts.len());
    let mut start = 0usize;
    for end in boundaries {
        parts.push(translated[start..end].iter().collect());
        start = end;
    }
    parts.push(translated[start..].iter().collect());
    parts
}

fn set_translated_message(
    entry: &mut TextEntry,
    source: &TextEntry,
    message: String,
) -> Result<()> {
    entry.message = message;
    match source.scr_msg_parts.as_ref() {
        Some(source_parts) => {
            let parts = split_translation_parts(source_parts, &entry.message);
            if parts.len() != source_parts.len() || parts.concat() != entry.message {
                return Err(format!(
                    "{} entry {}: failed to rebuild message_parts",
                    source.file, source.index
                ));
            }
            entry.message_parts = Some(parts);
        }
        None => entry.message_parts = None,
    }
    Ok(())
}

pub fn repair_json_directory(options: &RepairOptions) -> Result<RepairReport> {
    if options.output_dir.exists() {
        return Err(format!(
            "refusing to overwrite existing output directory: {}",
            options.output_dir.display()
        ));
    }
    let parsed = parse_directory(&options.source_dir, &options.macro_path)?;
    let name_dictionary = parse_name_dictionary(&options.name_dictionary_path)?;
    let mut available_json = collect_translation_json_names(&options.broken_json_dir)?;
    let mut repaired_entries = BTreeMap::<String, Vec<TextEntry>>::new();
    let mut repaired_inner_quotes = 0usize;

    for (_, source) in &parsed.sources {
        let expected_entries = parsed
            .entries
            .get(&source.relative_name)
            .expect("parsed source has entries");
        let json_file_name = format!("{}.json", source.relative_name);
        if !available_json.remove(&json_file_name.to_ascii_lowercase()) {
            return Err(format!(
                "missing translated JSON for {}: {}",
                source.relative_name,
                options.broken_json_dir.join(&json_file_name).display()
            ));
        }
        let (translated_messages, quote_repairs) = extract_pretty_property_strings(
            &options.broken_json_dir.join(&json_file_name),
            "message",
        )?;
        repaired_inner_quotes += quote_repairs;
        if translated_messages.len() != expected_entries.len() {
            return Err(format!(
                "{} has {} message lines but {} source entries",
                json_file_name,
                translated_messages.len(),
                expected_entries.len()
            ));
        }

        let mut entries = expected_entries.clone();
        for ((entry, expected), message) in entries
            .iter_mut()
            .zip(expected_entries)
            .zip(translated_messages)
        {
            set_translated_message(entry, expected, message)?;
        }
        repaired_entries.insert(source.relative_name.clone(), entries);
    }
    if !available_json.is_empty() {
        return Err(format!(
            "unexpected translated JSON files: {}",
            available_json.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }

    let mut speaker_names = HashMap::<String, String>::new();
    let mut unmapped_speaker_names = BTreeSet::new();
    let mut translated_speaker_names = 0usize;
    for entries in repaired_entries.values_mut() {
        for entry in entries
            .iter_mut()
            .filter(|entry| entry.entry_type == "name")
        {
            let macro_name = entry
                .speaker_macro
                .as_ref()
                .ok_or_else(|| {
                    format!("{} entry {} has no speaker macro", entry.file, entry.index)
                })?
                .to_ascii_lowercase();
            let translated = if let Some(value) = name_dictionary.get(&entry.scr_msg) {
                value.clone()
            } else {
                unmapped_speaker_names.insert(entry.scr_msg.clone());
                entry.scr_msg.clone()
            };
            if translated != entry.scr_msg {
                translated_speaker_names += 1;
            }
            let source = parsed
                .entries
                .get(&entry.file)
                .and_then(|source_entries| source_entries.get(entry.index))
                .expect("name source entry exists");
            set_translated_message(entry, source, translated.clone())?;
            if let Some(previous) = speaker_names.insert(macro_name.clone(), translated.clone()) {
                if previous != translated {
                    return Err(format!(
                        "speaker macro '{macro_name}' maps to both '{previous}' and '{translated}'"
                    ));
                }
            }
        }
    }

    for entries in repaired_entries.values_mut() {
        for entry in entries.iter_mut().filter(|entry| entry.scr_name.is_some()) {
            let macro_name = entry
                .speaker_macro
                .as_ref()
                .ok_or_else(|| {
                    format!("{} entry {} has no speaker macro", entry.file, entry.index)
                })?
                .to_ascii_lowercase();
            let translated = speaker_names.get(&macro_name).ok_or_else(|| {
                format!(
                    "{} entry {} references unknown speaker macro '{}'",
                    entry.file, entry.index, macro_name
                )
            })?;
            entry.name = Some(translated.clone());
        }
    }

    let entries = repaired_entries.values().map(Vec::len).sum();
    let translated_messages = repaired_entries
        .values()
        .flatten()
        .filter(|entry| entry.message != entry.scr_msg)
        .count();
    let multipart_entries = repaired_entries
        .values()
        .flatten()
        .filter(|entry| entry.message_parts.is_some())
        .count();

    let output_parent = options
        .output_dir
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(output_parent)
        .map_err(|error| format!("failed to create {}: {error}", output_parent.display()))?;
    let temporary = temporary_output_path(&options.output_dir)?;
    let write_result = (|| {
        fs::create_dir(&temporary)
            .map_err(|error| format!("failed to create {}: {error}", temporary.display()))?;
        for (source_name, entries) in &repaired_entries {
            write_utf8_json(&json_path(&temporary, source_name), entries)?;
        }
        write_utf8_json(&temporary.join("audit.json"), &parsed.report)?;
        fs::rename(&temporary, &options.output_dir).map_err(|error| {
            format!(
                "failed to finalize {}: {error}",
                options.output_dir.display()
            )
        })
    })();
    if let Err(error) = write_result {
        if temporary.exists() {
            let _ = fs::remove_dir_all(&temporary);
        }
        return Err(error);
    }

    Ok(RepairReport {
        json_files: repaired_entries.len(),
        entries,
        translated_messages,
        multipart_entries,
        repaired_inner_quotes,
        dictionary_entries: name_dictionary.len(),
        translated_speaker_names,
        unmapped_speaker_names: unmapped_speaker_names.into_iter().collect(),
    })
}

fn validate_immutable_entry(expected: &TextEntry, actual: &TextEntry) -> Result<()> {
    let context = format!("{} entry {}", expected.file, expected.index);
    macro_rules! immutable {
        ($field:ident, $json_name:literal) => {
            if actual.$field != expected.$field {
                return Err(format!(
                    "{context}: immutable field {} was changed",
                    $json_name
                ));
            }
        };
    }

    immutable!(file, "_file");
    immutable!(index, "_index");
    immutable!(line, "_line");
    immutable!(end_line, "_end_line");
    immutable!(offset, "_offset");
    immutable!(size, "_size");
    immutable!(entry_type, "_type");
    immutable!(encoding, "_encoding");
    immutable!(boundary, "_boundary");
    immutable!(source_kind, "_source_kind");
    immutable!(speaker_macro, "_speaker_macro");
    immutable!(target, "_target");
    immutable!(ruby_removed, "_ruby_removed");
    immutable!(controls, "_controls");
    immutable!(scr_name, "_scr_name");
    immutable!(scr_msg, "scr_msg");
    immutable!(scr_msg_parts, "scr_msg_parts");
    Ok(())
}

fn validate_translator_text(context: &str, field: &str, text: &str) -> Result<()> {
    if text.contains('\0') {
        return Err(format!("{context}: {field} contains NUL"));
    }
    if text.contains(['\r', '\n']) {
        return Err(format!("{context}: {field} contains a physical newline"));
    }
    if text.contains(['[', ']']) {
        return Err(format!(
            "{context}: {field} contains '[' or ']'; KAG syntax is not allowed in translator text"
        ));
    }
    Ok(())
}

fn encode_cp932_translation(context: &str, text: &str) -> Result<Vec<u8>> {
    let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
    if !had_errors {
        return Ok(encoded.into_owned());
    }

    let mut invalid = BTreeSet::new();
    for ch in text.chars() {
        let (_, _, char_error) = SHIFT_JIS.encode(&ch.to_string());
        if char_error {
            invalid.insert(ch);
        }
    }
    let listed = invalid
        .iter()
        .map(|ch| format!("U+{:04X} '{}'", *ch as u32, ch))
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!(
        "{context}: translation cannot be encoded as CP932: {listed}"
    ))
}

fn rebuild_controlled_text(
    context: &str,
    parts: &[String],
    controls: &[ControlRecord],
) -> Result<String> {
    for control in controls {
        if control.after_part >= parts.len() {
            return Err(format!(
                "{context}: protected control '{}' references missing message_parts[{}]",
                control.raw, control.after_part
            ));
        }
    }

    let mut rebuilt = String::new();
    for (part_index, part) in parts.iter().enumerate() {
        rebuilt.push_str(part);
        for control in controls
            .iter()
            .filter(|control| control.after_part == part_index)
        {
            rebuilt.push_str(&control.raw);
        }
    }
    Ok(rebuilt)
}

fn entry_replacement(expected: &TextEntry, actual: &TextEntry) -> Result<Option<Vec<u8>>> {
    validate_immutable_entry(expected, actual)?;
    let context = format!("{} entry {}", expected.file, expected.index);

    if expected.controls.is_empty() {
        if actual.message_parts.is_some() {
            return Err(format!(
                "{context}: message_parts is not allowed for an entry without protected controls"
            ));
        }
        if actual.message == expected.message {
            return Ok(None);
        }
        validate_translator_text(&context, "message", &actual.message)?;
        let trimmed = actual.message.trim_start_matches([' ', '\t']);
        if trimmed.starts_with(['@', '*', ';']) {
            return Err(format!(
                "{context}: message starts with KAG command, label, or comment syntax"
            ));
        }
        if actual.message.trim_end_matches([' ', '\t']).ends_with('\\') {
            return Err(format!(
                "{context}: message ends with a KAG physical-line continuation"
            ));
        }
        return encode_cp932_translation(&context, &actual.message).map(Some);
    }

    let expected_parts = expected
        .message_parts
        .as_ref()
        .ok_or_else(|| format!("{context}: extractor metadata has no message_parts"))?;
    let actual_parts = actual
        .message_parts
        .as_ref()
        .ok_or_else(|| format!("{context}: protected entry requires message_parts"))?;
    if actual_parts.len() != expected_parts.len() {
        return Err(format!(
            "{context}: message_parts count changed from {} to {}",
            expected_parts.len(),
            actual_parts.len()
        ));
    }

    let message_changed = actual.message != expected.message;
    let parts_changed = actual_parts != expected_parts;
    if !message_changed && !parts_changed {
        return Ok(None);
    }
    if message_changed && !parts_changed {
        return Err(format!(
            "{context}: edit message_parts for entries with protected controls; message alone is not injectable"
        ));
    }
    let joined = actual_parts.concat();
    if message_changed && actual.message != joined {
        return Err(format!(
            "{context}: changed message must equal concatenated message_parts"
        ));
    }
    for (index, part) in actual_parts.iter().enumerate() {
        validate_translator_text(&context, &format!("message_parts[{index}]"), part)?;
    }
    let rebuilt = rebuild_controlled_text(&context, actual_parts, &expected.controls)?;
    let trimmed = rebuilt.trim_start_matches([' ', '\t']);
    if trimmed.starts_with(['@', '*', ';']) {
        return Err(format!(
            "{context}: rebuilt text starts with KAG command, label, or comment syntax"
        ));
    }
    if rebuilt.trim_end_matches([' ', '\t']).ends_with('\\') {
        return Err(format!(
            "{context}: rebuilt text ends with a KAG physical-line continuation"
        ));
    }
    encode_cp932_translation(&context, &rebuilt).map(Some)
}

struct BytePatch {
    offset: usize,
    size: usize,
    bytes: Vec<u8>,
    index: usize,
}

fn apply_byte_patches(path: &Path, patches: &mut [BytePatch]) -> Result<Vec<u8>> {
    let mut bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    patches.sort_by_key(|patch| patch.offset);

    let mut previous_end = 0usize;
    for patch in patches.iter() {
        let end = patch
            .offset
            .checked_add(patch.size)
            .ok_or_else(|| format!("{} entry {} span overflows", path.display(), patch.index))?;
        if end > bytes.len() {
            return Err(format!(
                "{} entry {} span {}..{} exceeds {} bytes",
                path.display(),
                patch.index,
                patch.offset,
                end,
                bytes.len()
            ));
        }
        if patch.offset < previous_end {
            return Err(format!(
                "{} entry {} overlaps a previous text span",
                path.display(),
                patch.index
            ));
        }
        previous_end = end;
    }

    for patch in patches.iter().rev() {
        let end = patch.offset + patch.size;
        bytes.splice(patch.offset..end, patch.bytes.iter().copied());
    }
    Ok(bytes)
}

fn collect_translation_json_names(json_dir: &Path) -> Result<BTreeSet<String>> {
    if !json_dir.is_dir() {
        return Err(format!("{} is not a directory", json_dir.display()));
    }
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(json_dir)
        .map_err(|error| format!("failed to read {}: {error}", json_dir.display()))?
    {
        let entry = entry.map_err(|error| format!("failed to read JSON entry: {error}"))?;
        let path = entry.path();
        if entry
            .file_type()
            .map_err(|error| format!("failed to stat {}: {error}", path.display()))?
            .is_file()
            && path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
            && !entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("audit.json")
        {
            names.insert(entry.file_name().to_string_lossy().to_ascii_lowercase());
        }
    }
    Ok(names)
}

fn copy_tree(source: &Path, destination: &Path) -> Result<usize> {
    fs::create_dir(destination)
        .map_err(|error| format!("failed to create {}: {error}", destination.display()))?;
    let mut entries = fs::read_dir(source)
        .map_err(|error| format!("failed to read {}: {error}", source.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read directory entry: {error}"))?;
    entries.sort_by_key(|entry| entry.file_name());

    let mut copied_files = 0usize;
    for entry in entries {
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|error| format!("failed to stat {}: {error}", source_path.display()))?;
        if file_type.is_symlink() {
            return Err(format!(
                "refusing to copy symbolic link in source tree: {}",
                source_path.display()
            ));
        }
        if file_type.is_dir() {
            copied_files += copy_tree(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &destination_path).map_err(|error| {
                format!(
                    "failed to copy {} to {}: {error}",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
            copied_files += 1;
        } else {
            return Err(format!(
                "unsupported source tree entry: {}",
                source_path.display()
            ));
        }
    }
    Ok(copied_files)
}

fn future_absolute_path(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| format!("failed to resolve current directory: {error}"))?
            .join(path)
    };
    let mut cursor = absolute.as_path();
    let mut missing = Vec::new();
    while !cursor.exists() {
        let name = cursor
            .file_name()
            .ok_or_else(|| format!("cannot resolve future path {}", path.display()))?;
        missing.push(name.to_os_string());
        cursor = cursor
            .parent()
            .ok_or_else(|| format!("cannot resolve future path {}", path.display()))?;
    }
    let mut resolved = fs::canonicalize(cursor)
        .map_err(|error| format!("failed to resolve {}: {error}", cursor.display()))?;
    for component in missing.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn temporary_output_path(output: &Path) -> Result<PathBuf> {
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = output
        .file_name()
        .ok_or_else(|| format!("invalid output path: {}", output.display()))?
        .to_string_lossy();
    for attempt in 0..100usize {
        let candidate = parent.join(format!(
            ".{name}.tongern_tmp_{}_{}",
            std::process::id(),
            attempt
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "could not reserve a temporary output beside {}",
        output.display()
    ))
}

fn merge_requested_speaker_name(
    requested: &mut HashMap<String, String>,
    macro_name: &str,
    translated: &str,
    context: &str,
) -> Result<()> {
    let key = macro_name.to_ascii_lowercase();
    if let Some(previous) = requested.get(&key) {
        if previous != translated {
            return Err(format!(
                "{context}: speaker macro '{macro_name}' has conflicting names '{previous}' and '{translated}'"
            ));
        }
    } else {
        requested.insert(key, translated.to_string());
    }
    Ok(())
}

pub fn inject_directory(options: &InjectOptions) -> Result<InjectReport> {
    if options.output_dir.exists() {
        return Err(format!(
            "refusing to overwrite existing output directory: {}",
            options.output_dir.display()
        ));
    }
    let source_absolute = fs::canonicalize(&options.source_dir).map_err(|error| {
        format!(
            "failed to resolve source directory {}: {error}",
            options.source_dir.display()
        )
    })?;
    let output_absolute = future_absolute_path(&options.output_dir)?;
    if output_absolute.starts_with(&source_absolute) {
        return Err(format!(
            "output directory must be outside the source tree: {}",
            options.output_dir.display()
        ));
    }

    let _recorded_source = audit_source_directory(&options.json_dir)?;
    let parsed = parse_directory(&options.source_dir, &options.macro_path)?;
    let mut available_json = collect_translation_json_names(&options.json_dir)?;
    let mut modified_files = BTreeMap::<PathBuf, Vec<u8>>::new();
    let mut actual_by_file = BTreeMap::<String, Vec<TextEntry>>::new();
    let mut json_entries = 0usize;
    let mut patched = 0usize;
    let mut unchanged = 0usize;

    for (_, source) in &parsed.sources {
        let expected_entries = parsed
            .entries
            .get(&source.relative_name)
            .expect("parsed source has entries");
        let json_file_name = format!("{}.json", source.relative_name);
        if !available_json.remove(&json_file_name.to_ascii_lowercase()) {
            return Err(format!(
                "missing translation JSON for {}: {}",
                source.relative_name,
                options.json_dir.join(&json_file_name).display()
            ));
        }
        let actual_entries: Vec<TextEntry> =
            read_utf8_json(&options.json_dir.join(&json_file_name))?;
        if actual_entries.len() != expected_entries.len() {
            return Err(format!(
                "{} entry count changed from {} to {}",
                json_file_name,
                expected_entries.len(),
                actual_entries.len()
            ));
        }
        json_entries += actual_entries.len();
        actual_by_file.insert(source.relative_name.clone(), actual_entries);
    }
    if !available_json.is_empty() {
        return Err(format!(
            "unexpected translation JSON files: {}",
            available_json.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }

    let mut requested_speaker_names = HashMap::<String, String>::new();
    for (file, expected_entries) in &parsed.entries {
        let actual_entries = actual_by_file.get(file).expect("actual entries loaded");
        for (expected, actual) in expected_entries.iter().zip(actual_entries) {
            validate_immutable_entry(expected, actual)?;
            let context = format!("{} entry {}", expected.file, expected.index);
            match expected.scr_name.as_deref() {
                Some(source_name) => {
                    let translated_name = actual
                        .name
                        .as_deref()
                        .ok_or_else(|| format!("{context}: writable name is missing"))?;
                    validate_translator_text(&context, "name", translated_name)?;
                    encode_cp932_translation(&context, translated_name)?;
                    if translated_name != source_name {
                        let macro_name = expected.speaker_macro.as_deref().ok_or_else(|| {
                            format!("{context}: named dialogue has no _speaker_macro")
                        })?;
                        merge_requested_speaker_name(
                            &mut requested_speaker_names,
                            macro_name,
                            translated_name,
                            &context,
                        )?;
                    }
                }
                None if actual.name.is_some() => {
                    return Err(format!("{context}: unexpected name field"));
                }
                None => {}
            }
            if expected.entry_type == "name" && actual.message != expected.message {
                let macro_name = expected
                    .speaker_macro
                    .as_deref()
                    .ok_or_else(|| format!("{context}: name entry has no _speaker_macro"))?;
                merge_requested_speaker_name(
                    &mut requested_speaker_names,
                    macro_name,
                    &actual.message,
                    &context,
                )?;
            }
        }
    }

    for (source_path, source) in &parsed.sources {
        let expected_entries = parsed
            .entries
            .get(&source.relative_name)
            .expect("parsed source has entries");
        let actual_entries = actual_by_file
            .get(&source.relative_name)
            .expect("actual entries loaded");
        let mut patches = Vec::new();
        for (expected, actual) in expected_entries.iter().zip(actual_entries) {
            let mut effective = actual.clone();
            if expected.entry_type == "name" {
                if let Some(macro_name) = expected.speaker_macro.as_deref() {
                    if let Some(translated) =
                        requested_speaker_names.get(&macro_name.to_ascii_lowercase())
                    {
                        set_translated_message(&mut effective, expected, translated.clone())?;
                    }
                }
            }
            if let Some(bytes) = entry_replacement(expected, &effective)? {
                patches.push(BytePatch {
                    offset: expected.offset,
                    size: expected.size,
                    bytes,
                    index: expected.index,
                });
                patched += 1;
            } else {
                unchanged += 1;
            }
        }
        if !patches.is_empty() {
            modified_files.insert(
                PathBuf::from(&source.relative_name),
                apply_byte_patches(source_path, &mut patches)?,
            );
        }
    }

    let output_parent = options
        .output_dir
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(output_parent)
        .map_err(|error| format!("failed to create {}: {error}", output_parent.display()))?;
    let temporary = temporary_output_path(&options.output_dir)?;
    let write_result = (|| {
        let output_files = copy_tree(&options.source_dir, &temporary)?;
        for (relative, bytes) in &modified_files {
            let destination = temporary.join(relative);
            fs::write(&destination, bytes)
                .map_err(|error| format!("failed to write {}: {error}", destination.display()))?;
        }
        fs::rename(&temporary, &options.output_dir).map_err(|error| {
            format!(
                "failed to finalize {}: {error}",
                options.output_dir.display()
            )
        })?;
        Ok(output_files)
    })();

    let output_files = match write_result {
        Ok(count) => count,
        Err(error) => {
            if temporary.exists() {
                let _ = fs::remove_dir_all(&temporary);
            }
            return Err(error);
        }
    };

    Ok(InjectReport {
        json_files: parsed.entries.len(),
        json_entries,
        patched,
        unchanged,
        output_files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_DIRECTORY_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TestDirectory {
        path: PathBuf,
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn test_directory(label: &str) -> TestDirectory {
        let counter = TEST_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "tongern_ks_{label}_{}_{}",
            std::process::id(),
            counter
        ));
        fs::create_dir(&path).unwrap();
        TestDirectory { path }
    }

    fn write_cp932(path: &Path, text: &str) {
        let (bytes, _, had_errors) = SHIFT_JIS.encode(text);
        assert!(!had_errors);
        fs::write(path, bytes.as_ref()).unwrap();
    }

    fn filesystem_fixture(label: &str, script: &str) -> (TestDirectory, PathBuf, PathBuf, PathBuf) {
        let root = test_directory(label);
        let scenario = root.path.join("场景 & scenario");
        let json = root.path.join("翻译 JSON");
        let output = root.path.join("注回 output");
        fs::create_dir(&scenario).unwrap();
        write_cp932(
            &scenario.join("macro.ks"),
            "[macro name=男]\\\r\n[current layer=message1]\\\r\n男\r\n[current layer=message0]\\\r\n[endmacro]\\\r\n",
        );
        write_cp932(&scenario.join("test.ks"), script);
        fs::write(scenario.join("untouched.bin"), [0, 1, 2, 0xff]).unwrap();
        (root, scenario, json, output)
    }

    fn source(name: &str, text: &str) -> SourceFile {
        let mut lines = Vec::new();
        let mut byte_offset = 0usize;
        let mut crlf = 0usize;
        let mut lf = 0usize;
        let mut cr = 0usize;
        let mut remaining = text;
        let mut line_no = 1usize;

        while !remaining.is_empty() {
            let next = remaining
                .char_indices()
                .find_map(|(index, ch)| matches!(ch, '\r' | '\n').then_some((index, ch)));
            let (line_text, terminator, consumed) = match next {
                Some((index, '\r')) if remaining[index..].starts_with("\r\n") => {
                    crlf += 1;
                    (&remaining[..index], "\r\n", index + 2)
                }
                Some((index, '\r')) => {
                    cr += 1;
                    (&remaining[..index], "\r", index + 1)
                }
                Some((index, '\n')) => {
                    lf += 1;
                    (&remaining[..index], "\n", index + 1)
                }
                _ => (remaining, "", remaining.len()),
            };
            let byte_len = cp932_len(line_text).unwrap();
            lines.push(SourceLine {
                number: line_no,
                byte_offset,
                text: line_text.to_string(),
            });
            byte_offset += byte_len + terminator.len();
            remaining = &remaining[consumed..];
            line_no += 1;
        }

        SourceFile {
            relative_name: name.to_string(),
            lines,
            crlf_lines: crlf,
            lf_lines: lf,
            cr_lines: cr,
        }
    }

    fn speakers() -> (HashMap<String, SpeakerDefinition>, AuditReport) {
        let macro_source = source(
            "macro.ks",
            "[macro name=男]\\\r\n[current layer=message1]\\\r\n[font color=0xffffff]\\\r\n　　　男\r\n[current layer=message0]\\\r\n[endmacro]\\\r\n[macro name=リーゼ]\\\r\n[current layer=message1]\\\r\nリーゼロッテ\r\n[current layer=message0]\\\r\n[endmacro]\\\r\n",
        );
        let mut report = AuditReport::new();
        let definitions = parse_speaker_definitions(&macro_source, &mut report).unwrap();
        let map = definitions
            .into_iter()
            .map(|definition| (definition.macro_name.to_ascii_lowercase(), definition))
            .collect();
        (map, report)
    }

    #[test]
    fn finds_display_names_from_speaker_macros() {
        let (speakers, report) = speakers();
        assert_eq!(report.violation_count, 0);
        assert_eq!(speakers.len(), 2);
        assert_eq!(speakers["男"].display_name, "男");
        assert_eq!(speakers["リーゼ"].display_name, "リーゼロッテ");
    }

    #[test]
    fn extracts_dialogue_and_removes_ruby_reading() {
        let (speakers, mut report) = speakers();
        let script = source(
            "2nd.ks",
            "[男]\\\r\n「[ruby text=かん]漢字だ」[p]\r\n[cm1]\\\r\n",
        );
        let entries = extract_file_entries(&script, &speakers, None, &mut report).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "dialogue");
        assert_eq!(entries[0].name.as_deref(), Some("男"));
        assert_eq!(entries[0].scr_msg, "「漢字だ」");
        assert_eq!(entries[0].ruby_removed, 1);
        assert_eq!(entries[0].boundary, "p");
    }

    #[test]
    fn hides_newlines_and_returns_but_protects_terminal_wait() {
        let (speakers, mut report) = speakers();
        let script = source("2nd.ks", "一行目\r\n二[l]行目[l]\r\n三行目[r]四行目[p]\r\n");
        let entries = extract_file_entries(&script, &speakers, None, &mut report).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].scr_msg, "一行目二行目三行目四行目");
        assert!(!entries[0].message.contains(['\r', '\n', '[', ']']));
        assert_eq!(entries[0].controls.len(), 1);
        assert_eq!(entries[0].controls[0].tag, "l");
        assert_eq!(entries[0].controls[0].raw, "[l]");
    }

    #[test]
    fn er_retains_speaker_and_cm_resets_it() {
        let (speakers, mut report) = speakers();
        let script = source(
            "test.ks",
            "[男]\\\r\n一[p]\r\n[er1]\\\r\n二[p]\r\n[cm1]\\\r\n地の文[p]\r\n",
        );
        let entries = extract_file_entries(&script, &speakers, None, &mut report).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].name.as_deref(), Some("男"));
        assert_eq!(entries[1].name.as_deref(), Some("男"));
        assert_eq!(entries[2].name, None);
        assert_eq!(entries[2].entry_type, "monologue");
    }

    #[test]
    fn cm_flushes_text_without_page_break_and_reports_it() {
        let (speakers, mut report) = speakers();
        let script = source("test.ks", "[男]\\\r\n叫び声[l]\r\n[cm1]\\\r\n");
        let entries = extract_file_entries(&script, &speakers, None, &mut report).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].scr_msg, "叫び声");
        assert_eq!(entries[0].controls.len(), 1);
        assert_eq!(entries[0].controls[0].tag, "l");
        assert_eq!(entries[0].boundary, "cm");
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "implicit_text_boundary"));
    }

    #[test]
    fn handles_two_pages_and_reports_stray_bracket() {
        let (speakers, mut report) = speakers();
        let script = source("test.ks", "[男]\\\r\n一[p]二[p]]\r\n[cm1]\\\r\n");
        let entries = extract_file_entries(&script, &speakers, None, &mut report).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].scr_msg, "一");
        assert_eq!(entries[1].scr_msg, "二");
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "multiple_page_breaks"));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "stray_closing_bracket"));
    }

    #[test]
    fn extracts_choice_and_skips_tjs_ui_string() {
        let (speakers, mut report) = speakers();
        let script = source(
            "7th.ks",
            "[link target=*a]選択肢[endlink]\r\n[iscript]\r\nbutton.hint = \"閉じる\";\r\n[endscript]\r\n",
        );
        let entries = extract_file_entries(&script, &speakers, None, &mut report).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "choice");
        assert_eq!(entries[0].target.as_deref(), Some("*a"));
        assert_eq!(entries[0].scr_msg, "選択肢");
    }

    #[test]
    fn extracts_only_story_region_from_mixed_first2nd_script() {
        let (speakers, mut report) = speakers();
        let script = source(
            "first2nd.ks",
            "*first2nd\r\nメニュー[s]\r\n*0001\r\n地の文[p]\r\n*config3\r\nシステム設定[s]\r\n",
        );
        let entries = extract_file_entries(&script, &speakers, None, &mut report).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "monologue");
        assert_eq!(entries[0].scr_msg, "地の文");
    }

    #[test]
    fn skips_ui_only_kag_files() {
        let (speakers, mut report) = speakers();
        let script = source(
            "rclick_test.ks",
            "[title name=\"右クリック\"]\r\nシステム設定[s]\r\n",
        );
        let entries = extract_file_entries(&script, &speakers, None, &mut report).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn unchanged_directory_injection_is_byte_exact() {
        let (_root, scenario, json, output) =
            filesystem_fixture("unchanged", "一行目\r\n二行目[l]\r\n[p]\r\n");
        let extract = ExtractOptions {
            input_dir: scenario.clone(),
            output_dir: json.clone(),
            macro_path: scenario.join("macro.ks"),
        };
        extract_directory(&extract).unwrap();

        let inject = InjectOptions {
            json_dir: json,
            source_dir: scenario.clone(),
            output_dir: output.clone(),
            macro_path: scenario.join("macro.ks"),
        };
        let report = inject_directory(&inject).unwrap();
        assert_eq!(report.patched, 0);
        assert_eq!(report.json_entries, report.unchanged);
        for name in ["macro.ks", "test.ks", "untouched.bin"] {
            assert_eq!(
                fs::read(scenario.join(name)).unwrap(),
                fs::read(output.join(name)).unwrap()
            );
        }
    }

    #[test]
    fn modified_injection_restores_only_terminal_wait_and_opaque_controls() {
        let (_root, scenario, json, output) =
            filesystem_fixture("modified", "一[l]中[r]末[wait time=100]後[l]\r\n[p]\r\n");
        extract_directory(&ExtractOptions {
            input_dir: scenario.clone(),
            output_dir: json.clone(),
            macro_path: scenario.join("macro.ks"),
        })
        .unwrap();

        let json_path = json.join("test.ks.json");
        let mut entries: Vec<TextEntry> = read_utf8_json(&json_path).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].scr_msg, "一中末後");
        assert_eq!(
            entries[0]
                .controls
                .iter()
                .map(|control| control.tag.as_str())
                .collect::<Vec<_>>(),
            ["wait", "l"]
        );
        entries[0].message_parts = Some(vec!["甲".to_string(), "乙".to_string()]);
        write_utf8_json(&json_path, &entries).unwrap();

        let report = inject_directory(&InjectOptions {
            json_dir: json,
            source_dir: scenario.clone(),
            output_dir: output.clone(),
            macro_path: scenario.join("macro.ks"),
        })
        .unwrap();
        assert_eq!(report.patched, 1);
        let rebuilt = decode_cp932_line(
            &output.join("test.ks"),
            1,
            &fs::read(output.join("test.ks")).unwrap(),
        )
        .unwrap();
        assert_eq!(rebuilt, "甲[wait time=100]乙[l]\r\n[p]\r\n");
        assert_eq!(rebuilt.matches("[l]").count(), 1);
        assert!(!rebuilt.contains("[r]"));
    }

    #[test]
    fn injection_rejects_unencodable_text_before_creating_output() {
        let (_root, scenario, json, output) = filesystem_fixture("encoding", "書き換え対象[p]\r\n");
        extract_directory(&ExtractOptions {
            input_dir: scenario.clone(),
            output_dir: json.clone(),
            macro_path: scenario.join("macro.ks"),
        })
        .unwrap();

        let json_path = json.join("test.ks.json");
        let mut entries: Vec<TextEntry> = read_utf8_json(&json_path).unwrap();
        entries[0].message = "CP932外😀".to_string();
        write_utf8_json(&json_path, &entries).unwrap();
        let error = inject_directory(&InjectOptions {
            json_dir: json,
            source_dir: scenario.clone(),
            output_dir: output.clone(),
            macro_path: scenario.join("macro.ks"),
        })
        .unwrap_err();
        assert!(error.contains("U+1F600"));
        assert!(!output.exists());
    }

    #[test]
    fn resolves_bare_relative_future_output_against_current_directory() {
        let name = format!(
            "tongern_relative_output_{}_{}",
            std::process::id(),
            TEST_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let resolved = future_absolute_path(Path::new(&name)).unwrap();
        let expected = fs::canonicalize(std::env::current_dir().unwrap())
            .unwrap()
            .join(name);
        assert_eq!(resolved, expected);
    }

    #[test]
    fn repairs_invalid_json_multipart_messages_and_writable_names() {
        let (_root, scenario, extracted, injected) =
            filesystem_fixture("repair", "[男]\\\r\n一[l]\r\n二[p]\r\n");
        extract_directory(&ExtractOptions {
            input_dir: scenario.clone(),
            output_dir: extracted.clone(),
            macro_path: scenario.join("macro.ks"),
        })
        .unwrap();

        let broken = extracted.with_file_name("broken JSON");
        fs::create_dir(&broken).unwrap();
        for entry in fs::read_dir(&extracted).unwrap() {
            let entry = entry.unwrap();
            fs::copy(entry.path(), broken.join(entry.file_name())).unwrap();
        }
        let test_json_path = broken.join("test.ks.json");
        let test_json = fs::read_to_string(&test_json_path).unwrap();
        let broken_json = test_json
            .replace("\"name\": \"男\"", "\"name\": \"旧译名\"")
            .replace("\"message\": \"一二\"", "\"message\": \"甲\"乙\"");
        fs::write(&test_json_path, broken_json.as_bytes()).unwrap();
        let dictionary = broken.with_file_name("names.toml");
        fs::write(&dictionary, "\"男\" = [\"男人\", 1]\n".as_bytes()).unwrap();
        let repaired = broken.with_file_name("repaired JSON");

        let report = repair_json_directory(&RepairOptions {
            broken_json_dir: broken,
            source_dir: scenario.clone(),
            output_dir: repaired.clone(),
            macro_path: scenario.join("macro.ks"),
            name_dictionary_path: dictionary,
        })
        .unwrap();
        assert_eq!(report.repaired_inner_quotes, 1);
        assert_eq!(report.translated_speaker_names, 1);

        let entries: Vec<TextEntry> = read_utf8_json(&repaired.join("test.ks.json")).unwrap();
        assert_eq!(entries[0].scr_name.as_deref(), Some("男"));
        assert_eq!(entries[0].name.as_deref(), Some("男人"));
        assert_eq!(entries[0].message, "甲\"乙");
        assert_eq!(
            entries[0]
                .message_parts
                .as_ref()
                .expect("multipart translation")
                .concat(),
            entries[0].message
        );

        inject_directory(&InjectOptions {
            json_dir: repaired,
            source_dir: scenario.clone(),
            output_dir: injected.clone(),
            macro_path: scenario.join("macro.ks"),
        })
        .unwrap();
        let macro_text = decode_cp932_line(
            &injected.join("macro.ks"),
            1,
            &fs::read(injected.join("macro.ks")).unwrap(),
        )
        .unwrap();
        assert!(macro_text.contains("男人"));
        let script_text = decode_cp932_line(
            &injected.join("test.ks"),
            1,
            &fs::read(injected.join("test.ks")).unwrap(),
        )
        .unwrap();
        assert_eq!(script_text.matches("[l]").count(), 1);
        assert!(script_text.contains("甲\""));
        assert!(script_text.contains("乙[p]"));
    }
}
