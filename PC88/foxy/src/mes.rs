use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub type MesResult<T> = Result<T, String>;

const TEXT_MARKER: &str = ".foxy_text_managed";
const LOCALIZED_MARKER: &str = ".foxy_localization_managed";
const EXPLICIT_NEWLINE: [u8; 2] = [0x81, 0x93];

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextSpan {
    offset: usize,
    size: usize,
    kind: EntryKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryKind {
    Message,
    Choice,
}

impl EntryKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Choice => "choice",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslationDocument {
    pub format: String,
    pub version: u32,
    pub profile: TranslationProfile,
    pub files: Vec<TranslationFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslationProfile {
    pub engine: String,
    pub encoding: String,
    pub speaker_policy: String,
    pub menu_policy: String,
    pub auto_wrap: AutoWrapProfile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoWrapProfile {
    pub detected: bool,
    pub fullwidth_columns: usize,
    pub default_mode: String,
    pub explicit_newline_sjis: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslationFile {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_sha256")]
    pub sha256: String,
    pub entries: Vec<TranslationEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslationEntry {
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scr_msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_parts: Option<Vec<MessagePart>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessagePart {
    Text {
        scr_msg: String,
        message: String,
    },
    Raw {
        #[serde(rename = "_hex")]
        hex: String,
    },
}

#[derive(Debug, Serialize)]
struct TextManifest {
    format: &'static str,
    tool: &'static str,
    source: String,
    file_count: usize,
    entry_count: usize,
    messages_path: &'static str,
}

#[derive(Debug, Serialize)]
pub struct InjectionReport {
    format: &'static str,
    source: String,
    translations: String,
    file_count: usize,
    changed_file_count: usize,
    changed_entry_count: usize,
    wrap_columns: Option<usize>,
    files: Vec<InjectedFile>,
}

#[derive(Debug, Serialize)]
struct InjectedFile {
    #[serde(rename = "_file")]
    file: String,
    original_sha256: String,
    injected_sha256: String,
    original_size: usize,
    injected_size: usize,
    changed_entries: usize,
}

pub struct InjectionSummary {
    pub file_count: usize,
    pub changed_file_count: usize,
    pub changed_entry_count: usize,
    pub report_path: PathBuf,
}

pub fn extract_workspace(input: &Path, output: &Path, overwrite: bool) -> MesResult<usize> {
    reject_nested_output(input, output)?;
    let files = discover_mes_files(input)?;
    if files.is_empty() {
        return Err(format!(
            "no decoded AI1 .MES files found under {}",
            input.display()
        ));
    }

    let mut translation_files = Vec::with_capacity(files.len());
    let mut entry_count = 0usize;
    for file in files {
        let bytes = fs::read(&file)
            .map_err(|error| format!("failed to read {}: {error}", file.display()))?;
        let relative = file
            .strip_prefix(input)
            .map_err(|_| format!("{} is outside {}", file.display(), input.display()))?;
        let entries = extract_entries(&bytes)
            .map_err(|error| format!("{} is not valid FOXY AI1 MES: {error}", file.display()))?;
        entry_count += entries.len();
        translation_files.push(TranslationFile {
            file: slash_path(relative),
            sha256: sha256_hex(&bytes),
            entries,
        });
    }

    let document = TranslationDocument {
        format: "FOXY AI1 translation JSON".to_string(),
        version: 1,
        profile: TranslationProfile {
            engine: "AI1".to_string(),
            encoding: "CP932 plus FOXY carrier mapping".to_string(),
            speaker_policy: "no name field; dialogue and narration use message".to_string(),
            menu_policy: "text inside command A1 menu blocks uses choice".to_string(),
            auto_wrap: AutoWrapProfile {
                detected: true,
                fullwidth_columns: 36,
                default_mode: "preserve; do not insert hard line breaks".to_string(),
                explicit_newline_sjis: "8193".to_string(),
            },
        },
        files: translation_files,
    };

    prepare_managed_directory(output, overwrite, TEXT_MARKER, "manifest.json")?;
    fs::write(
        output.join(TEXT_MARKER),
        b"foxy text extraction in progress\n",
    )
    .map_err(|error| format!("failed to mark {}: {error}", output.display()))?;
    let text_dir = output.join("text");
    fs::create_dir_all(&text_dir)
        .map_err(|error| format!("failed to create {}: {error}", text_dir.display()))?;
    write_json(&text_dir.join("messages.json"), &document)?;
    let manifest = TextManifest {
        format: "FOXY AI1 extracted text workspace",
        tool: "foxy_d88_tool",
        source: display_leaf(input),
        file_count: document.files.len(),
        entry_count,
        messages_path: "text/messages.json",
    };
    write_json(&output.join("manifest.json"), &manifest)?;
    fs::remove_file(output.join(TEXT_MARKER))
        .map_err(|error| format!("failed to clear output marker: {error}"))?;
    Ok(entry_count)
}

pub fn load_translation_document(path: &Path) -> MesResult<TranslationDocument> {
    let messages_path = if path.is_dir() {
        let nested = path.join("text").join("messages.json");
        if nested.is_file() {
            nested
        } else {
            path.join("messages.json")
        }
    } else {
        path.to_path_buf()
    };
    let bytes = fs::read(&messages_path)
        .map_err(|error| format!("failed to read {}: {error}", messages_path.display()))?;
    let document: TranslationDocument = serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse {}: {error}", messages_path.display()))?;
    validate_document_header(&document)?;
    Ok(document)
}

pub fn inject_workspace<F>(
    input: &Path,
    translations_path: &Path,
    output: &Path,
    overwrite: bool,
    wrap_columns: Option<usize>,
    mut encode_text: F,
) -> MesResult<InjectionSummary>
where
    F: FnMut(&str) -> MesResult<Vec<u8>>,
{
    if let Some(columns) = wrap_columns {
        if columns == 0 {
            return Err("--wrap-columns must be greater than zero".to_string());
        }
    }
    reject_nested_output(input, output)?;
    let document = load_translation_document(translations_path)?;
    let mut generated = Vec::with_capacity(document.files.len());
    let mut report_files = Vec::with_capacity(document.files.len());
    let mut changed_file_count = 0usize;
    let mut changed_entry_count = 0usize;

    for translation_file in &document.files {
        let relative = safe_relative_path(&translation_file.file)?;
        let source_path = input.join(&relative);
        let original = fs::read(&source_path)
            .map_err(|error| format!("failed to read {}: {error}", source_path.display()))?;
        let original_hash = sha256_hex(&original);
        if original_hash != translation_file.sha256 {
            return Err(format!(
                "source hash changed for {}: JSON has {}, input is {}",
                translation_file.file, translation_file.sha256, original_hash
            ));
        }
        let current = extract_entries(&original)
            .map_err(|error| format!("failed to parse {}: {error}", source_path.display()))?;
        validate_entries(&translation_file.entries, &current, &translation_file.file)?;

        let mut patches = Vec::with_capacity(translation_file.entries.len());
        let mut file_changes = 0usize;
        for entry in &translation_file.entries {
            let replacement = encode_entry(entry, wrap_columns, &mut encode_text)?;
            let original_slice = &original[entry.offset..entry.offset + entry.size];
            if replacement != original_slice {
                file_changes += 1;
            }
            patches.push((entry.offset, entry.size, replacement));
        }
        patches.sort_by_key(|patch| patch.0);
        for pair in patches.windows(2) {
            if pair[0].0 + pair[0].1 > pair[1].0 {
                return Err(format!(
                    "overlapping text entries in {}",
                    translation_file.file
                ));
            }
        }
        let mut injected = original.clone();
        for (offset, size, replacement) in patches.into_iter().rev() {
            injected.splice(offset..offset + size, replacement);
        }
        parse_text_spans(&injected).map_err(|error| {
            format!(
                "injected MES no longer parses ({}): {error}",
                translation_file.file
            )
        })?;

        if file_changes > 0 {
            changed_file_count += 1;
            changed_entry_count += file_changes;
        }
        report_files.push(InjectedFile {
            file: translation_file.file.clone(),
            original_sha256: original_hash,
            injected_sha256: sha256_hex(&injected),
            original_size: original.len(),
            injected_size: injected.len(),
            changed_entries: file_changes,
        });
        generated.push((relative, injected));
    }

    prepare_managed_directory(output, overwrite, LOCALIZED_MARKER, "manifest.json")?;
    fs::write(
        output.join(LOCALIZED_MARKER),
        b"foxy localization injection in progress\n",
    )
    .map_err(|error| format!("failed to mark {}: {error}", output.display()))?;
    copy_tree(input, output)?;
    for (relative, bytes) in generated {
        let destination = output.join(relative);
        fs::write(&destination, bytes)
            .map_err(|error| format!("failed to write {}: {error}", destination.display()))?;
    }
    let localization_dir = output.join("localization");
    fs::create_dir_all(&localization_dir)
        .map_err(|error| format!("failed to create {}: {error}", localization_dir.display()))?;
    let report_path = localization_dir.join("injection_report.json");
    let report = InjectionReport {
        format: "FOXY AI1 injection report",
        source: display_leaf(input),
        translations: display_leaf(translations_path),
        file_count: document.files.len(),
        changed_file_count,
        changed_entry_count,
        wrap_columns,
        files: report_files,
    };
    write_json(&report_path, &report)?;
    fs::remove_file(output.join(LOCALIZED_MARKER))
        .map_err(|error| format!("failed to clear output marker: {error}"))?;

    Ok(InjectionSummary {
        file_count: document.files.len(),
        changed_file_count,
        changed_entry_count,
        report_path,
    })
}

pub fn collect_editable_text(document: &TranslationDocument) -> Vec<&str> {
    let mut output = Vec::new();
    for file in &document.files {
        for entry in &file.entries {
            if let Some(message) = entry.message.as_deref() {
                output.push(message);
            }
            if let Some(parts) = &entry.message_parts {
                for part in parts {
                    if let MessagePart::Text { message, .. } = part {
                        output.push(message);
                    }
                }
            }
        }
    }
    output
}

pub fn collect_original_double_byte_codes(input: &Path) -> MesResult<Vec<u16>> {
    let files = discover_mes_files(input)?;
    let mut codes = std::collections::BTreeSet::new();
    for file in files {
        let bytes = fs::read(&file)
            .map_err(|error| format!("failed to read {}: {error}", file.display()))?;
        let spans = parse_text_spans(&bytes)
            .map_err(|error| format!("failed to parse {}: {error}", file.display()))?;
        for span in spans {
            for pair in bytes[span.offset..span.offset + span.size].chunks_exact(2) {
                let (_, had_errors) = SHIFT_JIS.decode_without_bom_handling(pair);
                if !had_errors {
                    codes.insert(u16::from_be_bytes([pair[0], pair[1]]));
                }
            }
        }
    }
    Ok(codes.into_iter().collect())
}

fn validate_document_header(document: &TranslationDocument) -> MesResult<()> {
    if document.format != "FOXY AI1 translation JSON" || document.version != 1 {
        return Err(format!(
            "unsupported translation document {} version {}",
            document.format, document.version
        ));
    }
    if document.profile.engine != "AI1" {
        return Err(format!(
            "translation engine must be AI1, got {}",
            document.profile.engine
        ));
    }
    if document.profile.speaker_policy != "no name field; dialogue and narration use message" {
        return Err(
            "translation profile speaker policy was changed; FOXY has no name field".to_string(),
        );
    }
    Ok(())
}

fn extract_entries(bytes: &[u8]) -> MesResult<Vec<TranslationEntry>> {
    let spans = parse_text_spans(bytes)?;
    spans
        .into_iter()
        .enumerate()
        .map(|(index, span)| {
            let raw = &bytes[span.offset..span.offset + span.size];
            let parts = decode_parts(raw);
            let has_raw = parts
                .iter()
                .any(|part| matches!(part, MessagePart::Raw { .. }));
            if has_raw {
                Ok(TranslationEntry {
                    index,
                    offset: span.offset,
                    size: span.size,
                    kind: span.kind.as_str().to_string(),
                    scr_msg: None,
                    message: None,
                    message_parts: Some(parts),
                })
            } else {
                let mut text = String::new();
                for part in parts {
                    if let MessagePart::Text { scr_msg, .. } = part {
                        text.push_str(&scr_msg);
                    }
                }
                Ok(TranslationEntry {
                    index,
                    offset: span.offset,
                    size: span.size,
                    kind: span.kind.as_str().to_string(),
                    scr_msg: Some(text.clone()),
                    message: Some(text),
                    message_parts: None,
                })
            }
        })
        .collect()
}

fn decode_parts(raw: &[u8]) -> Vec<MessagePart> {
    let mut parts = Vec::new();
    let mut text = String::new();
    for pair in raw.chunks_exact(2) {
        if pair == EXPLICIT_NEWLINE {
            text.push('\n');
            continue;
        }
        let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(pair);
        if had_errors {
            if !text.is_empty() {
                parts.push(MessagePart::Text {
                    scr_msg: text.clone(),
                    message: std::mem::take(&mut text),
                });
            }
            parts.push(MessagePart::Raw {
                hex: hex_bytes(pair),
            });
        } else {
            text.push_str(&decoded);
        }
    }
    if !text.is_empty() {
        parts.push(MessagePart::Text {
            scr_msg: text.clone(),
            message: text,
        });
    }
    parts
}

fn validate_entries(
    requested: &[TranslationEntry],
    current: &[TranslationEntry],
    file: &str,
) -> MesResult<()> {
    if requested.len() != current.len() {
        return Err(format!(
            "entry count changed for {file}: JSON has {}, input has {}",
            requested.len(),
            current.len()
        ));
    }
    for (wanted, actual) in requested.iter().zip(current) {
        if wanted.index != actual.index
            || wanted.offset != actual.offset
            || wanted.size != actual.size
            || wanted.kind != actual.kind
        {
            return Err(format!(
                "entry metadata changed for {file} index {}",
                wanted.index
            ));
        }
        match (
            &wanted.scr_msg,
            &wanted.message_parts,
            &actual.scr_msg,
            &actual.message_parts,
        ) {
            (Some(scr_msg), None, Some(actual_scr), None) if scr_msg == actual_scr => {}
            (None, Some(parts), None, Some(actual_parts)) => {
                if parts.len() != actual_parts.len() {
                    return Err(format!(
                        "message_parts structure changed for {file} index {}",
                        wanted.index
                    ));
                }
                for (wanted_part, actual_part) in parts.iter().zip(actual_parts) {
                    match (wanted_part, actual_part) {
                        (
                            MessagePart::Text { scr_msg, .. },
                            MessagePart::Text {
                                scr_msg: actual_scr,
                                ..
                            },
                        ) if scr_msg == actual_scr => {}
                        (MessagePart::Raw { hex }, MessagePart::Raw { hex: actual_hex })
                            if hex.eq_ignore_ascii_case(actual_hex) => {}
                        _ => {
                            return Err(format!(
                                "immutable message_parts source changed for {file} index {}",
                                wanted.index
                            ));
                        }
                    }
                }
            }
            _ => {
                return Err(format!(
                    "immutable scr_msg changed or message layout is invalid for {file} index {}",
                    wanted.index
                ));
            }
        }
    }
    Ok(())
}

fn encode_entry<F>(
    entry: &TranslationEntry,
    wrap_columns: Option<usize>,
    encode_text: &mut F,
) -> MesResult<Vec<u8>>
where
    F: FnMut(&str) -> MesResult<Vec<u8>>,
{
    match (&entry.message, &entry.message_parts) {
        (Some(message), None) if entry.scr_msg.is_some() => {
            encode_display_text(message, wrap_columns, encode_text)
        }
        (None, Some(parts)) if entry.scr_msg.is_none() => {
            let mut output = Vec::new();
            for part in parts {
                match part {
                    MessagePart::Text { message, .. } => {
                        output.extend(encode_display_text(message, wrap_columns, encode_text)?);
                    }
                    MessagePart::Raw { hex } => output.extend(decode_hex(hex)?),
                }
            }
            Ok(output)
        }
        _ => Err(format!(
            "entry {} must use either scr_msg/message or message_parts",
            entry.index
        )),
    }
}

fn encode_display_text<F>(
    message: &str,
    wrap_columns: Option<usize>,
    encode_text: &mut F,
) -> MesResult<Vec<u8>>
where
    F: FnMut(&str) -> MesResult<Vec<u8>>,
{
    let prepared = if let Some(columns) = wrap_columns {
        wrap_text(message, columns)
    } else {
        message.to_string()
    };
    let mut output = Vec::new();
    for (index, segment) in prepared.split('\n').enumerate() {
        if index > 0 {
            output.extend_from_slice(&EXPLICIT_NEWLINE);
        }
        output.extend(encode_text(segment)?);
    }
    Ok(output)
}

fn wrap_text(message: &str, columns: usize) -> String {
    let mut output = String::with_capacity(message.len());
    let mut column = 0usize;
    for character in message.chars() {
        if character == '\n' {
            output.push(character);
            column = 0;
        } else {
            if column == columns {
                output.push('\n');
                column = 0;
            }
            output.push(character);
            column += 1;
        }
    }
    output
}

fn parse_text_spans(bytes: &[u8]) -> MesResult<Vec<TextSpan>> {
    Parser::new(bytes).parse()
}

struct Parser<'a> {
    data: &'a [u8],
    position: usize,
    spans: Vec<TextSpan>,
}

impl<'a> Parser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            position: 0,
            spans: Vec::new(),
        }
    }

    fn parse(mut self) -> MesResult<Vec<TextSpan>> {
        self.parse_stmts(false)?;
        if self.peek() == Some(0x7D) {
            self.position += 1;
        }
        Ok(self.spans)
    }

    fn parse_stmts(&mut self, choice: bool) -> MesResult<()> {
        while self.peek_is_stmt() {
            self.parse_stmt(choice)?;
        }
        Ok(())
    }

    fn parse_stmt(&mut self, choice: bool) -> MesResult<()> {
        let byte = self
            .peek()
            .ok_or_else(|| "unexpected end of MES".to_string())?;
        match byte {
            0x7B => self.parse_block(choice),
            0x2C => {
                self.position += 1;
                Ok(())
            }
            _ if self.peek_is_op() => self.parse_op(choice),
            _ if self.peek_is_term() => self.parse_expr(),
            0x80..=0x98 => self.parse_chrs(choice),
            _ => Err(format!(
                "unexpected statement byte {byte:02X} at {:#x}",
                self.position
            )),
        }
    }

    fn parse_op(&mut self, choice: bool) -> MesResult<()> {
        let byte = self
            .peek()
            .ok_or_else(|| "unexpected end of MES".to_string())?;
        match byte {
            0x22 => self.parse_string(),
            0x9D => self.parse_conditional(choice),
            0x99..=0x9C | 0x9E..=0xBF => {
                self.position += 1;
                self.parse_params(choice || byte == 0xA1)
            }
            0xC0..=0xFF => {
                self.position += 1;
                Ok(())
            }
            _ => Err(format!(
                "unexpected operation byte {byte:02X} at {:#x}",
                self.position
            )),
        }
    }

    fn parse_conditional(&mut self, choice: bool) -> MesResult<()> {
        let saved_position = self.position;
        let saved_spans = self.spans.len();
        if self.parse_conditional_braced(choice).is_ok() {
            return Ok(());
        }
        self.position = saved_position;
        self.spans.truncate(saved_spans);
        self.expect(0x9D)?;
        self.parse_expr()?;
        while self.peek_is_op() || matches!(self.peek(), Some(0x80..=0x98)) {
            if self.peek_is_op() {
                self.parse_op(choice)?;
            } else {
                self.parse_chrs(choice)?;
            }
        }
        Ok(())
    }

    fn parse_conditional_braced(&mut self, choice: bool) -> MesResult<()> {
        self.parse_conditional_pair(choice)?;
        loop {
            if self.peek() == Some(0x2C) {
                let saved_position = self.position;
                let saved_spans = self.spans.len();
                self.position += 1;
                if self.peek() == Some(0x9D) {
                    if self.parse_conditional_pair(choice).is_ok() {
                        continue;
                    }
                } else if self.peek() == Some(0x7B) {
                    self.parse_block(choice)?;
                    break;
                } else {
                    break;
                }
                self.position = saved_position;
                self.spans.truncate(saved_spans);
                break;
            }
            if self.peek() == Some(0x7B) {
                self.parse_block(choice)?;
            }
            break;
        }
        Ok(())
    }

    fn parse_conditional_pair(&mut self, choice: bool) -> MesResult<()> {
        self.expect(0x9D)?;
        self.parse_expr()?;
        self.parse_block(choice)
    }

    fn parse_params(&mut self, choice: bool) -> MesResult<()> {
        if !self.peek_can_start_param() {
            return Ok(());
        }
        self.parse_param(choice)?;
        while self.peek() == Some(0x2C) {
            let saved_position = self.position;
            let saved_spans = self.spans.len();
            self.position += 1;
            if !self.peek_can_start_param() || self.parse_param(choice).is_err() {
                self.position = saved_position;
                self.spans.truncate(saved_spans);
                break;
            }
        }
        Ok(())
    }

    fn parse_param(&mut self, choice: bool) -> MesResult<()> {
        match self.peek() {
            Some(0x7B) => self.parse_block(choice),
            Some(0x22) => self.parse_string(),
            _ => self.parse_expr(),
        }
    }

    fn parse_block(&mut self, choice: bool) -> MesResult<()> {
        self.expect(0x7B)?;
        self.parse_stmts(choice)?;
        if self.position == self.data.len() {
            // The original B/OMAKE.MES is physically truncated inside a
            // terminal block. Preserve that known malformed ending.
            Ok(())
        } else {
            self.expect(0x7D)
        }
    }

    fn parse_expr(&mut self) -> MesResult<()> {
        if !self.peek_is_term() {
            return Err(format!("expected expression at {:#x}", self.position));
        }
        while self.peek_is_term() {
            let byte = self.data[self.position];
            self.position += match byte {
                0x00 | 0x10 => 2,
                0x08 | 0x18 => 3,
                _ => 1,
            };
            if self.position > self.data.len() {
                return Err("truncated expression token".to_string());
            }
        }
        Ok(())
    }

    fn parse_string(&mut self) -> MesResult<()> {
        self.expect(0x22)?;
        while self.peek() != Some(0x22) {
            if self.position >= self.data.len() {
                return Err("unterminated 22-delimited string".to_string());
            }
            self.position += 1;
        }
        self.position += 1;
        Ok(())
    }

    fn parse_chrs(&mut self, choice: bool) -> MesResult<()> {
        let start = self.position;
        while matches!(self.peek(), Some(0x80..=0x98)) {
            if self.position + 1 >= self.data.len() {
                // OMAKE.MES ends in one opaque 0x93 byte. The original AI1
                // parser also leaves such terminal garbage outside its AST.
                // Keep it byte-for-byte, but never expose it as editable text.
                self.position = self.data.len();
                break;
            }
            self.position += 2;
        }
        let complete_size = (self.position - start) & !1;
        if complete_size > 0 {
            self.spans.push(TextSpan {
                offset: start,
                size: complete_size,
                kind: if choice {
                    EntryKind::Choice
                } else {
                    EntryKind::Message
                },
            });
        }
        Ok(())
    }

    fn expect(&mut self, expected: u8) -> MesResult<()> {
        match self.peek() {
            Some(actual) if actual == expected => {
                self.position += 1;
                Ok(())
            }
            Some(actual) => Err(format!(
                "expected {expected:02X}, got {actual:02X} at {:#x}",
                self.position
            )),
            None => Err(format!("expected {expected:02X} at end of MES")),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.data.get(self.position).copied()
    }

    fn peek_is_term(&self) -> bool {
        matches!(
            self.peek(),
            Some(
                0x00..=0x08
                | 0x10..=0x18
                | 0x21
                | 0x23
                | 0x25
                | 0x26
                | 0x2A
                | 0x2B
                | 0x2D
                | 0x2F
                | 0x3C..=0x40
                | 0x41..=0x5A
                | 0x5C
                | 0x5E
                | 0x7C,
            )
        )
    }

    fn peek_is_op(&self) -> bool {
        matches!(self.peek(), Some(0x22 | 0x99..=0xFF))
    }

    fn peek_is_stmt(&self) -> bool {
        matches!(self.peek(), Some(0x7B | 0x2C | 0x80..=0x98))
            || self.peek_is_op()
            || self.peek_is_term()
    }

    fn peek_can_start_param(&self) -> bool {
        matches!(self.peek(), Some(0x7B | 0x22)) || self.peek_is_term()
    }
}

fn discover_mes_files(root: &Path) -> MesResult<Vec<PathBuf>> {
    if !root.is_dir() {
        return Err(format!("MES input is not a directory: {}", root.display()));
    }
    let managed = root.join("manifest.json").is_file() && root.join("volumes").is_dir();
    let mut output = Vec::new();
    collect_files(root, root, managed, &mut output)?;
    output.sort_by_key(|path| slash_path(path.strip_prefix(root).unwrap_or(path)));
    Ok(output)
}

fn collect_files(
    root: &Path,
    current: &Path,
    managed: bool,
    output: &mut Vec<PathBuf>,
) -> MesResult<()> {
    for entry in fs::read_dir(current)
        .map_err(|error| format!("failed to read directory {}: {error}", current.display()))?
    {
        let entry =
            entry.map_err(|error| format!("failed to enumerate {}: {error}", current.display()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("failed to inspect {}: {error}", entry.path().display()))?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            collect_files(root, &path, managed, output)?;
        } else if file_type.is_file()
            && path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("mes"))
        {
            let include = if managed {
                path.strip_prefix(root).ok().is_some_and(|relative| {
                    relative
                        .components()
                        .any(|part| part.as_os_str() == "files_decoded")
                })
            } else {
                true
            };
            if include {
                output.push(path);
            }
        }
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> MesResult<()> {
    for entry in fs::read_dir(source)
        .map_err(|error| format!("failed to read directory {}: {error}", source.display()))?
    {
        let entry =
            entry.map_err(|error| format!("failed to enumerate {}: {error}", source.display()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("failed to inspect {}: {error}", entry.path().display()))?;
        if file_type.is_symlink() {
            return Err(format!(
                "refusing to copy symlink {}",
                entry.path().display()
            ));
        }
        let target = destination.join(entry.file_name());
        if file_type.is_dir() {
            fs::create_dir_all(&target)
                .map_err(|error| format!("failed to create {}: {error}", target.display()))?;
            copy_tree(&entry.path(), &target)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &target)
                .map_err(|error| format!("failed to copy {}: {error}", target.display()))?;
        }
    }
    Ok(())
}

fn prepare_managed_directory(
    output: &Path,
    overwrite: bool,
    marker_name: &str,
    manifest_name: &str,
) -> MesResult<()> {
    if output.as_os_str().is_empty() || output.parent().is_none() {
        return Err("refusing to use a filesystem root as output".to_string());
    }
    if output.exists() {
        let nonempty = fs::read_dir(output)
            .map_err(|error| format!("failed to inspect {}: {error}", output.display()))?
            .next()
            .transpose()
            .map_err(|error| format!("failed to inspect {}: {error}", output.display()))?
            .is_some();
        if nonempty {
            if !overwrite {
                return Err(format!(
                    "output directory is not empty: {}",
                    output.display()
                ));
            }
            if !output.join(manifest_name).is_file() && !output.join(marker_name).is_file() {
                return Err(format!(
                    "refusing to replace unrecognized directory {}",
                    output.display()
                ));
            }
            fs::remove_dir_all(output)
                .map_err(|error| format!("failed to replace {}: {error}", output.display()))?;
        }
    }
    fs::create_dir_all(output)
        .map_err(|error| format!("failed to create {}: {error}", output.display()))
}

fn reject_nested_output(input: &Path, output: &Path) -> MesResult<()> {
    let input = fs::canonicalize(input)
        .map_err(|error| format!("failed to resolve {}: {error}", input.display()))?;
    let output_parent = output.parent().unwrap_or_else(|| Path::new("."));
    let resolved_parent =
        fs::canonicalize(output_parent).unwrap_or_else(|_| output_parent.to_path_buf());
    let candidate = resolved_parent.join(output.file_name().unwrap_or_default());
    if candidate.starts_with(&input) || input.starts_with(&candidate) {
        return Err("MES output and source workspace cannot contain each other".to_string());
    }
    Ok(())
}

fn safe_relative_path(value: &str) -> MesResult<PathBuf> {
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(format!("unsafe translation file path: {value}"));
    }
    Ok(path.to_path_buf())
}

fn decode_hex(value: &str) -> MesResult<Vec<u8>> {
    if !value.len().is_multiple_of(2) || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("invalid raw hex value: {value}"));
    }
    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16).map_err(|error| error.to_string())
        })
        .collect()
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn display_leaf(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("input")
        .to_string()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> MesResult<()> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(|error| format!("failed to write {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_message_without_name_field() {
        let bytes = [0x82, 0xA0, 0x82, 0xA2, 0xA6, 0x7D];
        let entries = extract_entries(&bytes).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].scr_msg.as_deref(), Some("あい"));
        assert_eq!(entries[0].message.as_deref(), Some("あい"));
        assert_eq!(entries[0].kind, "message");
    }

    #[test]
    fn command_a1_block_is_choice() {
        let bytes = [0xA1, 0x7B, 0x82, 0xA0, 0x2C, 0x82, 0xA2, 0x7D, 0x7D];
        let entries = extract_entries(&bytes).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|entry| entry.kind == "choice"));
    }

    #[test]
    fn raw_pairs_use_message_parts() {
        let bytes = [0x82, 0xA0, 0x83, 0xEB, 0x82, 0xA2, 0x7D];
        let entries = extract_entries(&bytes).unwrap();
        let parts = entries[0].message_parts.as_ref().unwrap();
        assert_eq!(parts.len(), 3);
        assert!(matches!(&parts[1], MessagePart::Raw { hex } if hex == "83EB"));
    }

    #[test]
    fn explicit_newline_round_trips_as_lf() {
        let bytes = [0x82, 0xA0, 0x81, 0x93, 0x82, 0xA2, 0x7D];
        let entries = extract_entries(&bytes).unwrap();
        assert_eq!(entries[0].message.as_deref(), Some("あ\nい"));
        let encoded = encode_entry(&entries[0], None, &mut |text| {
            let (bytes, _, had_errors) = SHIFT_JIS.encode(text);
            if had_errors {
                Err("encoding error".to_string())
            } else {
                Ok(bytes.into_owned())
            }
        })
        .unwrap();
        assert_eq!(encoded, &bytes[..6]);
    }

    #[test]
    fn optional_wrap_uses_display_columns() {
        assert_eq!(wrap_text("一二三四五", 3), "一二三\n四五");
        assert_eq!(wrap_text("一二\n三四", 3), "一二\n三四");
    }
}
