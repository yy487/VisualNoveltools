use encoding_rs::SHIFT_JIS;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone)]
pub struct ExtractOptions {
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct InjectOptions {
    pub json_dir: PathBuf,
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct InjectReport {
    pub json_files: usize,
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub output_files: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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
    #[serde(rename = "_target", default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(
        rename = "_condition",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub condition: Option<String>,
    #[serde(rename = "_quote", default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
    #[serde(
        rename = "_display_prefix",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub display_prefix: Option<String>,
    #[serde(
        rename = "_display_suffix",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub display_suffix: Option<String>,
    pub scr_msg: String,
    pub message: String,
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
    pub body: usize,
    pub choice: usize,
    pub crlf_lines: usize,
    pub lf_lines: usize,
    pub cr_lines: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditReport {
    pub project: String,
    pub source_directory: String,
    pub source_encoding: String,
    pub line_ending_policy: String,
    pub name_policy: String,
    pub scanned_files: usize,
    pub json_files: usize,
    pub extracted_entries: usize,
    pub body_entries: usize,
    pub choice_entries: usize,
    pub tracked_controls: usize,
    pub dash_controls: usize,
    pub wait_controls: usize,
    pub ruby_controls: usize,
    pub font_controls: usize,
    pub resetfont_controls: usize,
    pub emb_controls: usize,
    pub l_controls: usize,
    pub r_controls: usize,
    pub at_r_controls: usize,
    pub warning_count: usize,
    pub files: Vec<FileReport>,
    pub diagnostics: Vec<Diagnostic>,
}

impl AuditReport {
    fn new() -> Self {
        Self {
            project: "diannao".to_string(),
            source_directory: String::new(),
            source_encoding: "cp932".to_string(),
            line_ending_policy:
                "physical CRLF is protected source structure and omitted from preview text"
                    .to_string(),
            name_policy: "no speaker-name macro; entries have no name field".to_string(),
            scanned_files: 0,
            json_files: 0,
            extracted_entries: 0,
            body_entries: 0,
            choice_entries: 0,
            tracked_controls: 0,
            dash_controls: 0,
            wait_controls: 0,
            ruby_controls: 0,
            font_controls: 0,
            resetfont_controls: 0,
            emb_controls: 0,
            l_controls: 0,
            r_controls: 0,
            at_r_controls: 0,
            warning_count: 0,
            files: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn warning(&mut self, code: &str, file: &str, line: usize, message: impl Into<String>) {
        self.warning_count += 1;
        self.diagnostics.push(Diagnostic {
            severity: "warning".to_string(),
            code: code.to_string(),
            file: file.to_string(),
            line,
            message: message.into(),
        });
    }

    fn account_entry(&mut self, entry: &TextEntry) {
        self.extracted_entries += 1;
        match entry.entry_type.as_str() {
            "body" => self.body_entries += 1,
            "choice" => self.choice_entries += 1,
            _ => {}
        }
    }

    fn account_control(&mut self, tag: &str) {
        let matched = match tag {
            "dash" => {
                self.dash_controls += 1;
                true
            }
            "wait" => {
                self.wait_controls += 1;
                true
            }
            "ruby" => {
                self.ruby_controls += 1;
                true
            }
            "font" => {
                self.font_controls += 1;
                true
            }
            "resetfont" => {
                self.resetfont_controls += 1;
                true
            }
            "emb" => {
                self.emb_controls += 1;
                true
            }
            "l" => {
                self.l_controls += 1;
                true
            }
            "r" => {
                self.r_controls += 1;
                true
            }
            "@r" => {
                self.at_r_controls += 1;
                true
            }
            _ => false,
        };
        if matched {
            self.tracked_controls += 1;
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
    #[cfg(test)]
    bytes: Vec<u8>,
    lines: Vec<SourceLine>,
    crlf_lines: usize,
    lf_lines: usize,
    cr_lines: usize,
}

fn decode_cp932(path: &Path, context: &str, bytes: &[u8]) -> Result<String> {
    let text = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(bytes)
        .ok_or_else(|| format!("{} {context} is not valid CP932/Shift_JIS", path.display()))?
        .into_owned();
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&text);
    if had_errors || encoded.as_ref() != bytes {
        return Err(format!(
            "{} {context} failed byte-exact CP932 round trip",
            path.display()
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
                2
            }
            b'\r' => {
                cr_lines += 1;
                1
            }
            b'\n' => {
                lf_lines += 1;
                1
            }
            _ => {
                cursor += 1;
                continue;
            }
        };
        let text = decode_cp932(path, &format!("line {line_no}"), &bytes[start..cursor])?;
        let newline_end = cursor + terminator_len;
        lines.push(SourceLine {
            number: line_no,
            byte_offset: start,
            text,
        });
        cursor = newline_end;
        start = cursor;
        line_no += 1;
    }
    if start < bytes.len() {
        let text = decode_cp932(path, &format!("line {line_no}"), &bytes[start..])?;
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
        #[cfg(test)]
        bytes,
        lines,
        crlf_lines,
        lf_lines,
        cr_lines,
    })
}

fn cp932_len(text: &str) -> Result<usize> {
    let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        return Err("internal source text cannot be encoded as CP932".to_string());
    }
    Ok(encoded.len())
}

fn absolute_offset(line: &SourceLine, utf8_offset: usize) -> Result<usize> {
    Ok(line.byte_offset + cp932_len(&line.text[..utf8_offset])?)
}

#[cfg(test)]
fn raw_range(source: &SourceFile, start: usize, end: usize) -> Result<String> {
    if start > end || end > source.bytes.len() {
        return Err(format!(
            "{} has invalid source range {start}..{end}",
            source.relative_name
        ));
    }
    SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(&source.bytes[start..end])
        .map(|text| text.into_owned())
        .ok_or_else(|| {
            format!(
                "{} range {start}..{end} is not valid CP932",
                source.relative_name
            )
        })
}

fn ascii_trim_start(text: &str) -> &str {
    text.trim_start_matches([' ', '\t'])
}

#[derive(Debug, Clone)]
struct AttrToken {
    name: String,
    value: String,
    value_start: usize,
    value_end: usize,
    quote: Option<char>,
}

#[derive(Debug, Clone)]
struct TagToken {
    name: String,
    attrs: Vec<AttrToken>,
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

fn parse_attributes(inner: &str, base: usize) -> Vec<AttrToken> {
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
        let (value_start, value_end, quote) = if bytes[cursor] == b'"' || bytes[cursor] == b'\'' {
            let quote = bytes[cursor] as char;
            cursor += 1;
            let start = cursor;
            while cursor < bytes.len() && bytes[cursor] != quote as u8 {
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
            (start, end, Some(quote))
        } else {
            let start = cursor;
            while cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            (start, cursor, None)
        };
        attrs.push(AttrToken {
            name: key.to_ascii_lowercase(),
            value: inner[value_start..value_end].to_string(),
            value_start: base + value_start,
            value_end: base + value_end,
            quote,
        });
    }
    attrs
}

fn parse_tag(text: &str, start: usize, end: usize) -> TagToken {
    let raw = &text[start..end];
    let inner = &raw[1..raw.len() - 1];
    let leading = inner.len() - inner.trim_start().len();
    let trimmed = inner.trim_start();
    let name_end = trimmed
        .find(|ch: char| ch.is_whitespace())
        .unwrap_or(trimmed.len());
    let name = trimmed[..name_end].to_ascii_lowercase();
    let attrs_inner = &trimmed[name_end..];
    let attrs_base = start + 1 + leading + name_end;
    TagToken {
        name,
        attrs: parse_attributes(attrs_inner, attrs_base),
    }
}

fn tokenize_line(
    source: &SourceFile,
    line: &SourceLine,
    report: &mut AuditReport,
) -> Vec<InlineToken> {
    let text = &line.text;
    let mut tokens = Vec::new();
    let mut cursor = 0usize;
    while cursor < text.len() {
        let Some(relative) = text[cursor..].find('[') else {
            if cursor < text.len() {
                tokens.push(InlineToken::Text {
                    raw: text[cursor..].to_string(),
                    start: cursor,
                    end: text.len(),
                });
            }
            break;
        };
        let tag_start = cursor + relative;
        if tag_start > cursor {
            tokens.push(InlineToken::Text {
                raw: text[cursor..tag_start].to_string(),
                start: cursor,
                end: tag_start,
            });
        }
        let Some(tag_end) = find_tag_end(text, tag_start) else {
            report.warning(
                "unclosed_tag",
                &source.relative_name,
                line.number,
                format!("unclosed '[' at UTF-8 column {}", tag_start + 1),
            );
            tokens.push(InlineToken::Text {
                raw: text[tag_start..].to_string(),
                start: tag_start,
                end: text.len(),
            });
            break;
        };
        tokens.push(InlineToken::Tag(parse_tag(text, tag_start, tag_end)));
        cursor = tag_end;
    }
    tokens
}

#[derive(Debug, Clone)]
struct CommandToken {
    name: String,
}

fn parse_command(line: &SourceLine) -> Option<CommandToken> {
    let trimmed = ascii_trim_start(&line.text);
    if !trimmed.starts_with('@') {
        return None;
    }
    let inner = &trimmed[1..];
    let name_end = inner
        .find(|ch: char| ch.is_whitespace())
        .unwrap_or(inner.len());
    Some(CommandToken {
        name: inner[..name_end].to_ascii_lowercase(),
    })
}

fn is_comment_or_label(line: &str) -> bool {
    let trimmed = ascii_trim_start(line);
    trimmed.starts_with(';') || trimmed.starts_with('*')
}

fn visible_text(text: &str) -> bool {
    text.chars().any(|ch| !ch.is_whitespace())
}

fn tag_display(tag: &TagToken) -> String {
    match tag.name.as_str() {
        "dash" => "──".to_string(),
        "emb" => {
            let expression = tag
                .attr("exp")
                .map(|attr| attr.value.as_str())
                .unwrap_or("?");
            format!("{{{{emb:{expression}}}}}")
        }
        _ => String::new(),
    }
}

#[derive(Debug, Clone)]
struct MessageBuilder {
    spans: Vec<BodySpan>,
    pending_display: String,
}

#[derive(Debug, Clone)]
struct BodySpan {
    text: String,
    offset: usize,
    size: usize,
    line: usize,
    display_prefix: Option<String>,
    display_suffix: Option<String>,
}

impl MessageBuilder {
    fn new() -> Self {
        Self {
            spans: Vec::new(),
            pending_display: String::new(),
        }
    }

    fn has_visible_text(&self) -> bool {
        !self.spans.is_empty()
    }

    fn add_text(&mut self, line: &SourceLine, raw: &str, start: usize, end: usize) -> Result<()> {
        if raw.is_empty() || !visible_text(raw) {
            return Ok(());
        }
        let absolute_start = absolute_offset(line, start)?;
        let absolute_end = absolute_offset(line, end)?;
        let display_prefix = if self.pending_display.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.pending_display))
        };
        self.spans.push(BodySpan {
            text: raw.to_string(),
            offset: absolute_start,
            size: absolute_end.saturating_sub(absolute_start),
            line: line.number,
            display_prefix,
            display_suffix: None,
        });
        Ok(())
    }

    fn add_control(&mut self, display: &str) {
        self.pending_display.push_str(display);
    }

    fn finish(
        &mut self,
        source: &SourceFile,
        boundary_line: usize,
        boundary: &str,
        report: &mut AuditReport,
    ) -> Vec<TextEntry> {
        if !self.pending_display.is_empty() {
            if let Some(last) = self.spans.last_mut() {
                last.display_suffix = Some(std::mem::take(&mut self.pending_display));
            } else {
                report.warning(
                    "orphan_display_control",
                    &source.relative_name,
                    boundary_line,
                    format!(
                        "visible control {:?} has no adjacent text before {boundary}",
                        self.pending_display
                    ),
                );
            }
        }
        self.pending_display.clear();
        self.spans
            .drain(..)
            .map(|span| {
                let mut scr_msg = span.display_prefix.clone().unwrap_or_default();
                scr_msg.push_str(&span.text);
                scr_msg.push_str(span.display_suffix.as_deref().unwrap_or_default());
                TextEntry {
                    file: source.relative_name.clone(),
                    index: 0,
                    line: span.line,
                    end_line: span.line,
                    offset: span.offset,
                    size: span.size,
                    entry_type: "body".to_string(),
                    encoding: "cp932".to_string(),
                    boundary: boundary.to_string(),
                    source_kind: "body_part".to_string(),
                    target: None,
                    condition: None,
                    quote: None,
                    display_prefix: span.display_prefix,
                    display_suffix: span.display_suffix,
                    scr_msg: scr_msg.clone(),
                    message: scr_msg,
                }
            })
            .collect()
    }
}

fn is_boundary(name: &str) -> Option<&'static str> {
    match name {
        "p" => Some("p"),
        "cm" => Some("cm"),
        "select" => Some("select"),
        "jump" => Some("jump"),
        _ => None,
    }
}

fn choice_entry(source: &SourceFile, line: &SourceLine, tag: &TagToken) -> Result<TextEntry> {
    let text = tag.attr("text").ok_or_else(|| {
        format!(
            "{}:{} [seladd] has no text attribute",
            source.relative_name, line.number
        )
    })?;
    let offset = absolute_offset(line, text.value_start)?;
    let end = absolute_offset(line, text.value_end)?;
    let target = tag
        .attr("target")
        .or_else(|| tag.attr("storage"))
        .map(|attr| attr.value.clone());
    let condition = tag.attr("exp").map(|attr| attr.value.clone());
    Ok(TextEntry {
        file: source.relative_name.clone(),
        index: 0,
        line: line.number,
        end_line: line.number,
        offset,
        size: end.saturating_sub(offset),
        entry_type: "choice".to_string(),
        encoding: "cp932".to_string(),
        boundary: "select".to_string(),
        source_kind: "seladd_text_attribute".to_string(),
        target,
        condition,
        quote: text.quote.map(|quote| quote.to_string()),
        display_prefix: None,
        display_suffix: None,
        scr_msg: text.value.clone(),
        message: text.value.clone(),
    })
}

fn finish_message(
    entries: &mut Vec<TextEntry>,
    builder: &mut MessageBuilder,
    source: &SourceFile,
    boundary_line: usize,
    boundary: &str,
    report: &mut AuditReport,
) {
    entries.extend(builder.finish(source, boundary_line, boundary, report));
}

fn extract_file_entries(source: &SourceFile, report: &mut AuditReport) -> Result<Vec<TextEntry>> {
    let mut entries = Vec::new();
    let mut builder = MessageBuilder::new();
    let mut in_macro = false;
    let mut in_script = false;

    for line in &source.lines {
        if is_comment_or_label(&line.text) || line.text.trim().is_empty() {
            continue;
        }

        if let Some(command) = parse_command(line) {
            match command.name.as_str() {
                "macro" => {
                    finish_message(
                        &mut entries,
                        &mut builder,
                        source,
                        line.number,
                        "macro",
                        report,
                    );
                    in_macro = true;
                    continue;
                }
                "endmacro" => {
                    in_macro = false;
                    continue;
                }
                "iscript" => {
                    finish_message(
                        &mut entries,
                        &mut builder,
                        source,
                        line.number,
                        "iscript",
                        report,
                    );
                    in_script = true;
                    continue;
                }
                "endscript" => {
                    in_script = false;
                    continue;
                }
                _ => {}
            }
            if in_macro || in_script {
                continue;
            }
            if let Some(boundary) = is_boundary(&command.name) {
                finish_message(
                    &mut entries,
                    &mut builder,
                    source,
                    line.number,
                    boundary,
                    report,
                );
            } else {
                if command.name == "r" {
                    report.account_control("@r");
                }
                builder.add_control("");
            }
            continue;
        }

        let tokens = tokenize_line(source, line, report);
        for token in tokens {
            match token {
                InlineToken::Text { raw, start, end } => {
                    if !in_macro && !in_script {
                        builder.add_text(line, &raw, start, end)?;
                    }
                }
                InlineToken::Tag(tag) => {
                    match tag.name.as_str() {
                        "macro" => {
                            finish_message(
                                &mut entries,
                                &mut builder,
                                source,
                                line.number,
                                "macro",
                                report,
                            );
                            in_macro = true;
                            continue;
                        }
                        "endmacro" => {
                            in_macro = false;
                            continue;
                        }
                        "iscript" => {
                            finish_message(
                                &mut entries,
                                &mut builder,
                                source,
                                line.number,
                                "iscript",
                                report,
                            );
                            in_script = true;
                            continue;
                        }
                        "endscript" => {
                            in_script = false;
                            continue;
                        }
                        _ => {}
                    }
                    if in_macro || in_script {
                        continue;
                    }
                    report.account_control(&tag.name);
                    if tag.name == "seladd" {
                        if builder.has_visible_text() {
                            finish_message(
                                &mut entries,
                                &mut builder,
                                source,
                                line.number,
                                "select",
                                report,
                            );
                        }
                        entries.push(choice_entry(source, line, &tag)?);
                        continue;
                    }
                    if let Some(boundary) = is_boundary(&tag.name) {
                        finish_message(
                            &mut entries,
                            &mut builder,
                            source,
                            line.number,
                            boundary,
                            report,
                        );
                        continue;
                    }
                    builder.add_control(&tag_display(&tag));
                }
            }
        }
    }

    let eof_line = source.lines.last().map(|line| line.number).unwrap_or(1);
    if builder.has_visible_text() {
        report.warning(
            "eof_text_boundary",
            &source.relative_name,
            eof_line,
            "visible body text reached EOF without @p, @cm, [select], or [jump]",
        );
    }
    finish_message(&mut entries, &mut builder, source, eof_line, "eof", report);

    entries.sort_by_key(|entry| entry.offset);
    let mut previous_end = 0usize;
    for (index, entry) in entries.iter_mut().enumerate() {
        if entry.offset < previous_end {
            return Err(format!(
                "{} entry at offset {} overlaps a previous entry ending at {}",
                source.relative_name, entry.offset, previous_end
            ));
        }
        previous_end = entry.offset + entry.size;
        entry.index = index;
    }
    Ok(entries)
}

fn collect_ks_files(input_dir: &Path) -> Result<Vec<(PathBuf, String)>> {
    if !input_dir.is_dir() {
        return Err(format!("{} is not a directory", input_dir.display()));
    }

    fn visit(root: &Path, directory: &Path, output: &mut Vec<(PathBuf, String)>) -> Result<()> {
        let mut entries = fs::read_dir(directory)
            .map_err(|error| format!("failed to read {}: {error}", directory.display()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| format!("failed to read directory entry: {error}"))?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| format!("failed to stat {}: {error}", path.display()))?;
            if file_type.is_symlink() {
                return Err(format!(
                    "refusing symbolic link in source tree: {}",
                    path.display()
                ));
            }
            if file_type.is_dir() {
                visit(root, &path, output)?;
            } else if file_type.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("ks"))
            {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|error| {
                        format!("failed to make {} relative: {error}", path.display())
                    })?
                    .to_string_lossy()
                    .replace('\\', "/");
                output.push((path, relative));
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    visit(input_dir, input_dir, &mut files)?;
    files.sort_by(|left, right| left.1.cmp(&right.1));
    if files.is_empty() {
        return Err(format!("{} contains no .ks files", input_dir.display()));
    }
    Ok(files)
}

#[derive(Debug)]
struct ParsedDirectory {
    sources: Vec<(PathBuf, SourceFile)>,
    entries: BTreeMap<String, Vec<TextEntry>>,
    report: AuditReport,
}

fn parse_directory(input_dir: &Path) -> Result<ParsedDirectory> {
    let files = collect_ks_files(input_dir)?;
    let source_directory = fs::canonicalize(input_dir)
        .map_err(|error| format!("failed to resolve {}: {error}", input_dir.display()))?;
    let mut report = AuditReport::new();
    report.source_directory = source_directory.to_string_lossy().into_owned();
    let mut sources = Vec::new();
    let mut entries_by_file = BTreeMap::new();

    for (path, relative) in files {
        let source = read_source(&path, &relative)?;
        if source.lf_lines > 0 || source.cr_lines > 0 {
            report.warning(
                "non_crlf_line_ending",
                &relative,
                1,
                format!(
                    "file contains CRLF={}, LF={}, CR={}",
                    source.crlf_lines, source.lf_lines, source.cr_lines
                ),
            );
        }
        let entries = extract_file_entries(&source, &mut report)?;
        for entry in &entries {
            report.account_entry(entry);
        }
        report.files.push(FileReport {
            file: relative.clone(),
            entries: entries.len(),
            body: entries
                .iter()
                .filter(|entry| entry.entry_type == "body")
                .count(),
            choice: entries
                .iter()
                .filter(|entry| entry.entry_type == "choice")
                .count(),
            crlf_lines: source.crlf_lines,
            lf_lines: source.lf_lines,
            cr_lines: source.cr_lines,
        });
        entries_by_file.insert(relative, entries);
        sources.push((path, source));
    }
    report.scanned_files = sources.len();
    report.json_files = sources.len();
    Ok(ParsedDirectory {
        sources,
        entries: entries_by_file,
        report,
    })
}

fn json_path(root: &Path, source_name: &str) -> PathBuf {
    root.join(format!("{source_name}.json"))
}

fn write_utf8_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn read_utf8_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(format!("{} must be UTF-8 without BOM", path.display()));
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| format!("{} is not valid UTF-8: {error}", path.display()))?;
    serde_json::from_str(text)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
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
            ".{name}.diannao_tmp_{}_{}",
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

fn prepare_output(output: &Path) -> Result<PathBuf> {
    if output.exists() {
        return Err(format!(
            "refusing to overwrite existing output directory: {}",
            output.display()
        ));
    }
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    let temporary = temporary_output_path(output)?;
    fs::create_dir(&temporary)
        .map_err(|error| format!("failed to create {}: {error}", temporary.display()))?;
    Ok(temporary)
}

fn commit_output(temporary: &Path, output: &Path) -> Result<()> {
    fs::rename(temporary, output).map_err(|error| {
        format!(
            "failed to move {} to {}: {error}",
            temporary.display(),
            output.display()
        )
    })
}

pub fn extract_directory(options: &ExtractOptions) -> Result<AuditReport> {
    if options.output_dir.exists() {
        return Err(format!(
            "refusing to overwrite existing output directory: {}",
            options.output_dir.display()
        ));
    }
    let source_absolute = fs::canonicalize(&options.input_dir).map_err(|error| {
        format!(
            "failed to resolve source directory {}: {error}",
            options.input_dir.display()
        )
    })?;
    let output_absolute = future_absolute_path(&options.output_dir)?;
    if output_absolute.starts_with(&source_absolute) {
        return Err(format!(
            "output directory must be outside the source tree: {}",
            options.output_dir.display()
        ));
    }

    let parsed = parse_directory(&options.input_dir)?;
    let temporary = prepare_output(&options.output_dir)?;
    let write_result = (|| -> Result<()> {
        for (_, source) in &parsed.sources {
            let entries = parsed
                .entries
                .get(&source.relative_name)
                .expect("parsed source has entries");
            write_utf8_json(&json_path(&temporary, &source.relative_name), entries)?;
        }
        write_utf8_json(&temporary.join("audit.json"), &parsed.report)
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&temporary);
        return Err(error);
    }
    commit_output(&temporary, &options.output_dir)?;
    Ok(parsed.report)
}

#[derive(Debug, Deserialize)]
struct AuditSource {
    source_directory: String,
}

pub fn audit_source_directory(json_dir: &Path) -> Result<PathBuf> {
    let audit: AuditSource = read_utf8_json(&json_dir.join("audit.json"))?;
    let source = PathBuf::from(audit.source_directory);
    if !source.is_dir() {
        return Err(format!(
            "audit.json source directory does not exist: {}",
            source.display()
        ));
    }
    Ok(source)
}

fn validate_immutable_entry(expected: &TextEntry, actual: &TextEntry) -> Result<()> {
    macro_rules! immutable {
        ($field:ident, $name:literal) => {
            if expected.$field != actual.$field {
                return Err(format!(
                    "{} entry {} immutable {} changed",
                    expected.file, expected.index, $name
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
    immutable!(target, "_target");
    immutable!(condition, "_condition");
    immutable!(quote, "_quote");
    immutable!(display_prefix, "_display_prefix");
    immutable!(display_suffix, "_display_suffix");
    immutable!(scr_msg, "scr_msg");
    Ok(())
}

fn validate_body_text(context: &str, field: &str, text: &str) -> Result<()> {
    if text.contains('\0') {
        return Err(format!("{context}: {field} contains NUL"));
    }
    if text.contains(['\r', '\n']) {
        return Err(format!(
            "{context}: {field} contains a physical newline; use protected [r]/@r controls"
        ));
    }
    if text.contains(['[', ']']) {
        return Err(format!(
            "{context}: {field} contains '[' or ']'; KAG tag syntax is not allowed"
        ));
    }
    Ok(())
}

fn validate_choice_text(context: &str, entry: &TextEntry, text: &str) -> Result<()> {
    if text.contains('\0') || text.contains(['\r', '\n']) {
        return Err(format!(
            "{context}: choice contains NUL or physical newline"
        ));
    }
    if let Some(quote) = entry.quote.as_deref() {
        if text.contains(quote) {
            return Err(format!(
                "{context}: choice contains its source quote delimiter {quote:?}"
            ));
        }
    } else if text.chars().any(char::is_whitespace) {
        return Err(format!(
            "{context}: unquoted choice attribute cannot contain whitespace"
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

fn entry_replacement(expected: &TextEntry, actual: &TextEntry) -> Result<Option<Vec<u8>>> {
    validate_immutable_entry(expected, actual)?;
    let context = format!("{} entry {}", expected.file, expected.index);
    if expected.entry_type == "choice" {
        if actual.message == expected.message {
            return Ok(None);
        }
        validate_choice_text(&context, expected, &actual.message)?;
        return encode_cp932_translation(&context, &actual.message).map(Some);
    }

    if actual.message == expected.message {
        return Ok(None);
    }
    let without_prefix = if let Some(prefix) = expected.display_prefix.as_deref() {
        actual.message.strip_prefix(prefix).ok_or_else(|| {
            format!("{context}: message must retain immutable display prefix {prefix:?}")
        })?
    } else {
        actual.message.as_str()
    };
    let translated = if let Some(suffix) = expected.display_suffix.as_deref() {
        without_prefix.strip_suffix(suffix).ok_or_else(|| {
            format!("{context}: message must retain immutable display suffix {suffix:?}")
        })?
    } else {
        without_prefix
    };
    validate_body_text(&context, "message", translated)?;
    encode_cp932_translation(&context, translated).map(Some)
}

#[derive(Debug)]
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

    fn visit(root: &Path, directory: &Path, names: &mut BTreeSet<String>) -> Result<()> {
        let mut entries = fs::read_dir(directory)
            .map_err(|error| format!("failed to read {}: {error}", directory.display()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| format!("failed to read JSON directory entry: {error}"))?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| format!("failed to stat {}: {error}", path.display()))?;
            if file_type.is_symlink() {
                return Err(format!(
                    "refusing symbolic link in JSON tree: {}",
                    path.display()
                ));
            }
            if file_type.is_dir() {
                visit(root, &path, names)?;
            } else if file_type.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
            {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|error| {
                        format!("failed to make {} relative: {error}", path.display())
                    })?
                    .to_string_lossy()
                    .replace('\\', "/")
                    .to_ascii_lowercase();
                if relative != "audit.json" {
                    names.insert(relative);
                }
            }
        }
        Ok(())
    }

    let mut names = BTreeSet::new();
    visit(json_dir, json_dir, &mut names)?;
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
                "refusing symbolic link in source tree: {}",
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
        }
    }
    Ok(copied_files)
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

    let recorded_source = audit_source_directory(&options.json_dir)?;
    let recorded_absolute = fs::canonicalize(&recorded_source).map_err(|error| {
        format!(
            "failed to resolve recorded source {}: {error}",
            recorded_source.display()
        )
    })?;
    if recorded_absolute != source_absolute {
        return Err(format!(
            "audit.json source {} does not match --source {}",
            recorded_absolute.display(),
            source_absolute.display()
        ));
    }

    let parsed = parse_directory(&options.source_dir)?;
    let mut available_json = collect_translation_json_names(&options.json_dir)?;
    let mut actual_by_file = BTreeMap::<String, Vec<TextEntry>>::new();
    let mut json_entries = 0usize;
    for (_, source) in &parsed.sources {
        let relative_json = format!("{}.json", source.relative_name);
        if !available_json.remove(&relative_json.to_ascii_lowercase()) {
            return Err(format!(
                "missing translation JSON for {}: {}",
                source.relative_name,
                json_path(&options.json_dir, &source.relative_name).display()
            ));
        }
        let actual: Vec<TextEntry> =
            read_utf8_json(&json_path(&options.json_dir, &source.relative_name))?;
        let expected = parsed
            .entries
            .get(&source.relative_name)
            .expect("parsed source has entries");
        if actual.len() != expected.len() {
            return Err(format!(
                "{} entry count changed from {} to {}",
                relative_json,
                expected.len(),
                actual.len()
            ));
        }
        json_entries += actual.len();
        actual_by_file.insert(source.relative_name.clone(), actual);
    }
    if !available_json.is_empty() {
        return Err(format!(
            "unexpected translation JSON files: {}",
            available_json.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }

    let mut modified_files = BTreeMap::<PathBuf, Vec<u8>>::new();
    let mut patched = 0usize;
    let mut unchanged = 0usize;
    for (source_path, source) in &parsed.sources {
        let expected_entries = parsed
            .entries
            .get(&source.relative_name)
            .expect("parsed source has entries");
        let actual_entries = actual_by_file
            .get(&source.relative_name)
            .expect("translation JSON loaded");
        let mut patches = Vec::new();
        for (expected, actual) in expected_entries.iter().zip(actual_entries) {
            if let Some(bytes) = entry_replacement(expected, actual)? {
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
    let output_files = match copy_tree(&options.source_dir, &temporary) {
        Ok(count) => count,
        Err(error) => {
            let _ = fs::remove_dir_all(&temporary);
            return Err(error);
        }
    };
    let write_result = (|| -> Result<()> {
        for (relative, bytes) in modified_files {
            let destination = temporary.join(relative);
            fs::write(&destination, bytes)
                .map_err(|error| format!("failed to write {}: {error}", destination.display()))?;
        }
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&temporary);
        return Err(error);
    }
    commit_output(&temporary, &options.output_dir)?;
    Ok(InjectReport {
        json_files: parsed.sources.len(),
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

    static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

    struct TestTree {
        root: PathBuf,
    }

    impl TestTree {
        fn new(label: &str) -> Self {
            let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "diannao_ks_{label}_{}_{}",
                std::process::id(),
                id
            ));
            if root.exists() {
                fs::remove_dir_all(&root).unwrap();
            }
            fs::create_dir(&root).unwrap();
            Self { root }
        }

        fn path(&self, name: &str) -> PathBuf {
            self.root.join(name)
        }
    }

    impl Drop for TestTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn encode_cp932(text: &str) -> Vec<u8> {
        let (bytes, _, had_errors) = SHIFT_JIS.encode(text);
        assert!(!had_errors);
        bytes.into_owned()
    }

    fn fixture() -> &'static str {
        concat!(
            "@cm\r\n",
            "[r]\r\n",
            "[dash]最初[l][r]\r\n",
            "@if exp=\"f.flag\"\r\n",
            "条件一[wait time=500]・[r]\r\n",
            "@else\r\n",
            "[font size=30]条件二[resetfont][l]\r\n",
            "@endif\r\n",
            "@p\r\n",
            "@cm\r\n",
            "動的[emb exp=\"f.next\"]値[l][r]\r\n",
            "@cm\r\n",
            "終了条件[l]\r\n",
            "@cm\r\n",
            "ジャンプ前[l]\r\n",
            "[jump storage=\"next.ks\"]\r\n",
            "@cm\r\n",
            "選択してください[l][r]\r\n",
            "[seladd text=\"１．はい\" target=\"*yes\" exp=\"f.a=1\"]\r\n",
            "[seladd text=\"２．いいえ\" storage=\"no.ks\"]\r\n",
            "[select]\r\n",
        )
    }

    fn make_source(tree: &TestTree) -> PathBuf {
        let source = tree.path("scenario");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("test.ks"), encode_cp932(fixture())).unwrap();
        fs::write(source.join("asset.bin"), [0, 1, 2, 3]).unwrap();
        source
    }

    #[test]
    fn state_machine_extracts_boundaries_choices_and_displays() {
        let tree = TestTree::new("state");
        let source_dir = make_source(&tree);
        let mut report = AuditReport::new();
        let source = read_source(&source_dir.join("test.ks"), "test.ks").unwrap();
        let entries = extract_file_entries(&source, &mut report).unwrap();
        assert_eq!(entries.len(), 11);
        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.entry_type == "body")
                .count(),
            9
        );
        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.entry_type == "choice")
                .count(),
            2
        );
        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.entry_type == "body")
                .map(|entry| entry.boundary.as_str())
                .collect::<Vec<_>>(),
            ["p", "p", "p", "p", "cm", "cm", "cm", "jump", "select"]
        );
        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.entry_type == "body")
                .map(|entry| entry.scr_msg.as_str())
                .collect::<Vec<_>>(),
            [
                "──最初",
                "条件一",
                "・",
                "条件二",
                "動的",
                "{{emb:f.next}}値",
                "終了条件",
                "ジャンプ前",
                "選択してください",
            ]
        );
        assert_eq!(entries[0].display_prefix.as_deref(), Some("──"));
        assert_eq!(
            entries
                .iter()
                .find(|entry| entry.scr_msg == "{{emb:f.next}}値")
                .unwrap()
                .display_prefix
                .as_deref(),
            Some("{{emb:f.next}}")
        );
        let json = serde_json::to_value(&entries).unwrap();
        let first = json.as_array().unwrap()[0].as_object().unwrap();
        assert!(!first.contains_key("scr_msg_parts"));
        assert!(!first.contains_key("message_parts"));
        assert!(!first.contains_key("_controls"));
        let choice = entries
            .iter()
            .find(|entry| entry.scr_msg == "１．はい")
            .unwrap();
        assert_eq!(choice.target.as_deref(), Some("*yes"));
        assert_eq!(choice.condition.as_deref(), Some("f.a=1"));
        assert_eq!(choice.quote.as_deref(), Some("\""));
        assert_eq!(report.dash_controls, 1);
        assert_eq!(report.wait_controls, 1);
        assert_eq!(report.font_controls, 1);
        assert_eq!(report.resetfont_controls, 1);
        assert_eq!(report.emb_controls, 1);
        assert_eq!(report.r_controls, 5);
    }

    #[test]
    fn body_entries_cover_only_their_source_text_bytes() {
        let tree = TestTree::new("rebuild");
        let source_dir = make_source(&tree);
        let source = read_source(&source_dir.join("test.ks"), "test.ks").unwrap();
        let mut report = AuditReport::new();
        let entries = extract_file_entries(&source, &mut report).unwrap();
        for entry in entries.iter().filter(|entry| entry.entry_type == "body") {
            let raw = raw_range(&source, entry.offset, entry.offset + entry.size).unwrap();
            let visible = entry
                .scr_msg
                .strip_prefix(entry.display_prefix.as_deref().unwrap_or_default())
                .unwrap();
            assert_eq!(raw, visible, "entry {}", entry.index);
            assert!(!raw.contains(['[', ']']));
            assert!(!raw.contains(['\r', '\n']));
        }
    }

    #[test]
    fn no_change_directory_injection_is_byte_exact() {
        let tree = TestTree::new("roundtrip");
        let source_dir = make_source(&tree);
        let json_dir = tree.path("scenario_json");
        let output_dir = tree.path("scenario_injected");
        let extract = extract_directory(&ExtractOptions {
            input_dir: source_dir.clone(),
            output_dir: json_dir.clone(),
        })
        .unwrap();
        assert_eq!(extract.scanned_files, 1);
        let inject = inject_directory(&InjectOptions {
            json_dir,
            source_dir: source_dir.clone(),
            output_dir: output_dir.clone(),
        })
        .unwrap();
        assert_eq!(inject.patched, 0);
        assert_eq!(inject.unchanged, extract.extracted_entries);
        assert_eq!(
            fs::read(source_dir.join("test.ks")).unwrap(),
            fs::read(output_dir.join("test.ks")).unwrap()
        );
        assert_eq!(
            fs::read(source_dir.join("asset.bin")).unwrap(),
            fs::read(output_dir.join("asset.bin")).unwrap()
        );
    }

    #[test]
    fn modified_body_and_choice_preserve_commands() {
        let tree = TestTree::new("modified");
        let source_dir = make_source(&tree);
        let json_dir = tree.path("scenario_json");
        let output_dir = tree.path("scenario_injected");
        extract_directory(&ExtractOptions {
            input_dir: source_dir.clone(),
            output_dir: json_dir.clone(),
        })
        .unwrap();

        let json_path = json_dir.join("test.ks.json");
        let mut entries: Vec<TextEntry> = read_utf8_json(&json_path).unwrap();
        let first = entries
            .iter_mut()
            .find(|entry| entry.scr_msg == "──最初")
            .unwrap();
        first.message = "──変更後".to_string();
        let choice = entries
            .iter_mut()
            .find(|entry| entry.scr_msg == "１．はい")
            .unwrap();
        choice.message = "１．承知".to_string();
        write_utf8_json(&json_path, &entries).unwrap();

        let inject = inject_directory(&InjectOptions {
            json_dir,
            source_dir,
            output_dir: output_dir.clone(),
        })
        .unwrap();
        assert_eq!(inject.patched, 2);
        let output = read_source(&output_dir.join("test.ks"), "test.ks").unwrap();
        let decoded = raw_range(&output, 0, output.bytes.len()).unwrap();
        assert!(decoded.contains("[dash]変更後[l][r]"));
        assert!(decoded.contains("@if exp=\"f.flag\""));
        assert!(decoded.contains("[wait time=500]"));
        assert!(decoded.contains("[font size=30]条件二[resetfont]"));
        assert!(decoded.contains("[seladd text=\"１．承知\" target=\"*yes\""));
    }

    #[test]
    fn changed_or_missing_display_prefix_is_rejected() {
        let tree = TestTree::new("prefix");
        let source_dir = make_source(&tree);
        let source = read_source(&source_dir.join("test.ks"), "test.ks").unwrap();
        let mut report = AuditReport::new();
        let entries = extract_file_entries(&source, &mut report).unwrap();
        let expected = entries
            .iter()
            .find(|entry| entry.scr_msg == "──最初")
            .unwrap();
        let mut actual = expected.clone();
        actual.message = "変更後".to_string();
        let error = entry_replacement(expected, &actual).unwrap_err();
        assert!(error.contains("retain immutable display prefix"));

        let mut metadata_changed = expected.clone();
        metadata_changed.display_prefix = Some("--".to_string());
        assert!(entry_replacement(expected, &metadata_changed)
            .unwrap_err()
            .contains("_display_prefix"));
    }

    #[test]
    fn trailing_display_control_is_an_immutable_suffix() {
        let tree = TestTree::new("suffix");
        let path = tree.path("suffix.ks");
        fs::write(&path, encode_cp932("末尾[dash]\r\n@p\r\n")).unwrap();
        let source = read_source(&path, "suffix.ks").unwrap();
        let mut report = AuditReport::new();
        let entries = extract_file_entries(&source, &mut report).unwrap();
        assert_eq!(entries.len(), 1);
        let expected = &entries[0];
        assert_eq!(expected.scr_msg, "末尾──");
        assert_eq!(expected.display_suffix.as_deref(), Some("──"));
        assert_eq!(report.warning_count, 0);

        let mut actual = expected.clone();
        actual.message = "変更後──".to_string();
        assert_eq!(
            entry_replacement(expected, &actual).unwrap().unwrap(),
            encode_cp932("変更後")
        );

        actual.message = "変更後".to_string();
        assert!(entry_replacement(expected, &actual)
            .unwrap_err()
            .contains("retain immutable display suffix"));
    }

    #[test]
    fn immutable_source_and_unencodable_translation_are_rejected() {
        let tree = TestTree::new("validation");
        let source_dir = make_source(&tree);
        let source = read_source(&source_dir.join("test.ks"), "test.ks").unwrap();
        let mut report = AuditReport::new();
        let entries = extract_file_entries(&source, &mut report).unwrap();
        let expected = entries
            .iter()
            .find(|entry| entry.entry_type == "choice")
            .unwrap();
        let mut metadata_changed = expected.clone();
        metadata_changed.scr_msg.push('改');
        assert!(entry_replacement(expected, &metadata_changed)
            .unwrap_err()
            .contains("scr_msg"));

        let mut unencodable = expected.clone();
        unencodable.message = "😀".to_string();
        assert!(entry_replacement(expected, &unencodable)
            .unwrap_err()
            .contains("U+1F600"));
    }

    #[test]
    fn unicode_bom_and_invalid_cp932_are_rejected() {
        let tree = TestTree::new("encoding");
        let bom = tree.path("bom.ks");
        fs::write(&bom, [0xef, 0xbb, 0xbf, b'a']).unwrap();
        assert!(read_source(&bom, "bom.ks")
            .unwrap_err()
            .contains("unsupported Unicode BOM"));

        let invalid = tree.path("invalid.ks");
        fs::write(&invalid, [0x81]).unwrap();
        assert!(read_source(&invalid, "invalid.ks")
            .unwrap_err()
            .contains("not valid CP932"));
    }
}
