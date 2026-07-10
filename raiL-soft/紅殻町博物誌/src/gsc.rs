use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};

pub const GSC_HEADER_SIZE: usize = 36;
const GSC_HEADER_SIZE_U32: u32 = 36;

const RESOURCE_TEXTS: &[&str] = &[
    "grpo", "grpo_bu", "grpo_ex", "REP001", "REP002", "REP003", "REP004", "REP005", "REP006",
    "REP007", "REP008",
];

#[derive(Debug)]
pub enum GscError {
    InvalidFormat(String),
    Encoding(String),
    InvalidTranslation(String),
}

impl GscError {
    fn invalid(message: impl Into<String>) -> Self {
        Self::InvalidFormat(message.into())
    }

    fn encoding(message: impl Into<String>) -> Self {
        Self::Encoding(message.into())
    }

    fn translation(message: impl Into<String>) -> Self {
        Self::InvalidTranslation(message.into())
    }
}

impl fmt::Display for GscError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat(message) => write!(formatter, "invalid GSC file: {message}"),
            Self::Encoding(message) => write!(formatter, "CP932 encoding error: {message}"),
            Self::InvalidTranslation(message) => {
                write!(formatter, "invalid GSC translation: {message}")
            }
        }
    }
}

impl Error for GscError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GscHeader {
    pub file_size: u32,
    pub header_size: u32,
    pub code_size: u32,
    pub text_index_size: u32,
    pub text_pool_size: u32,
    pub sequence_index_size: u32,
    pub sequence_pool_count: u32,
    pub symbol_table_size: u32,
    pub symbol_name_pool_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub scr_msg: String,
    pub message: String,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: u32,
    #[serde(rename = "_offset")]
    pub offset: u32,
    #[serde(rename = "_size")]
    pub size: u32,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_policy")]
    pub policy: String,
    #[serde(rename = "_inst_offset", skip_serializing_if = "Option::is_none")]
    pub instruction_offset: Option<u32>,
    #[serde(rename = "_opcode", skip_serializing_if = "Option::is_none")]
    pub opcode: Option<String>,
    #[serde(rename = "_target", skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(rename = "_choice_style", skip_serializing_if = "Option::is_none")]
    pub choice_style: Option<u32>,
}

#[derive(Debug)]
pub struct GscFile<'a> {
    pub header: GscHeader,
    data: &'a [u8],
    text_index_start: usize,
    text_pool_start: usize,
    text_pool_end: usize,
    text_offsets: Vec<u32>,
    text_bytes: Vec<&'a [u8]>,
}

#[derive(Debug, Clone)]
struct ChoiceReference {
    instruction_offset: u32,
    target: u32,
}

#[derive(Debug)]
enum TextContext<'a> {
    Choice {
        prefix: &'a str,
        visible: &'a str,
        style: u32,
    },
    Dialogue {
        prefix: &'a str,
        name: &'a str,
        body: &'a str,
    },
    Monologue(&'a str),
}

impl<'a> GscFile<'a> {
    /// Parse every declared GSC region and validate the text index/pool layout.
    ///
    /// # Errors
    ///
    /// Returns [`GscError::InvalidFormat`] when a header size, region boundary,
    /// text offset, or NUL-terminated text record is invalid.
    pub fn parse(data: &'a [u8]) -> Result<Self, GscError> {
        if data.len() < GSC_HEADER_SIZE {
            return Err(GscError::invalid(format!(
                "file is {} bytes, shorter than the {GSC_HEADER_SIZE}-byte header",
                data.len()
            )));
        }

        let header = GscHeader {
            file_size: read_u32(data, 0)?,
            header_size: read_u32(data, 4)?,
            code_size: read_u32(data, 8)?,
            text_index_size: read_u32(data, 12)?,
            text_pool_size: read_u32(data, 16)?,
            sequence_index_size: read_u32(data, 20)?,
            sequence_pool_count: read_u32(data, 24)?,
            symbol_table_size: read_u32(data, 28)?,
            symbol_name_pool_size: read_u32(data, 32)?,
        };

        if header.header_size != GSC_HEADER_SIZE_U32 {
            return Err(GscError::invalid(format!(
                "header size is {}, expected {GSC_HEADER_SIZE}",
                header.header_size
            )));
        }
        if header.file_size as usize != data.len() {
            return Err(GscError::invalid(format!(
                "declared file size is {}, physical size is {}",
                header.file_size,
                data.len()
            )));
        }
        if !header.text_index_size.is_multiple_of(4) {
            return Err(GscError::invalid("text index size is not divisible by 4"));
        }
        if !header.sequence_index_size.is_multiple_of(4) {
            return Err(GscError::invalid(
                "u16 sequence index size is not divisible by 4",
            ));
        }
        if !header.symbol_table_size.is_multiple_of(4) {
            return Err(GscError::invalid("symbol table size is not divisible by 4"));
        }

        let calculated_size = calculate_file_size(&header)?;
        if calculated_size != data.len() {
            return Err(GscError::invalid(format!(
                "region sizes total {calculated_size} bytes, physical size is {}",
                data.len()
            )));
        }

        let text_index_start = checked_add(GSC_HEADER_SIZE, header.code_size, "text index")?;
        let text_pool_start =
            checked_add(text_index_start, header.text_index_size, "text pool start")?;
        let text_pool_end = checked_add(text_pool_start, header.text_pool_size, "text pool end")?;
        let text_count = header.text_index_size as usize / 4;

        let mut text_offsets = Vec::with_capacity(text_count);
        let mut text_bytes = Vec::with_capacity(text_count);
        let mut cursor = 0_usize;
        for index in 0..text_count {
            let offset = read_u32(data, text_index_start + index * 4)?;
            if offset as usize != cursor {
                return Err(GscError::invalid(format!(
                    "text index {index} points to 0x{offset:x}, expected sequential offset 0x{cursor:x}"
                )));
            }
            if cursor >= header.text_pool_size as usize {
                return Err(GscError::invalid(format!(
                    "text index {index} starts outside the text pool"
                )));
            }

            let remaining = &data[text_pool_start + cursor..text_pool_end];
            let terminator = remaining
                .iter()
                .position(|byte| *byte == 0)
                .ok_or_else(|| {
                    GscError::invalid(format!("text index {index} is not NUL-terminated"))
                })?;
            text_offsets.push(offset);
            text_bytes.push(&remaining[..terminator]);
            cursor = cursor
                .checked_add(terminator + 1)
                .ok_or_else(|| GscError::invalid("text pool cursor overflow"))?;
        }
        if cursor != header.text_pool_size as usize {
            return Err(GscError::invalid(format!(
                "indexed strings consume {cursor} bytes, text pool declares {} bytes",
                header.text_pool_size
            )));
        }

        Ok(Self {
            header,
            data,
            text_index_start,
            text_pool_start,
            text_pool_end,
            text_offsets,
            text_bytes,
        })
    }

    #[must_use]
    pub fn text_count(&self) -> usize {
        self.text_bytes.len()
    }

    /// Export every localizable text-table entry using stable text indices.
    ///
    /// Empty strings and the game's confirmed resource identifiers are omitted.
    /// Dialogue names and choice style prefixes are exposed as metadata and are
    /// restored automatically during injection.
    ///
    /// # Errors
    ///
    /// Returns an error if any exported text is not valid CP932 or a structurally
    /// valid choice instruction reuses one text index with conflicting targets.
    pub fn extract_entries(&self, file_name: &str) -> Result<Vec<TextEntry>, GscError> {
        let choices = self.find_choice_references()?;
        let mut entries = Vec::new();

        for (index, raw) in self.text_bytes.iter().enumerate() {
            let decoded = decode_cp932(raw, index)?;
            if decoded.is_empty() || RESOURCE_TEXTS.contains(&decoded.as_str()) {
                continue;
            }

            let choice = choices.get(&index);
            let context = text_context(&decoded, choice.is_some())?;
            let physical_offset = self
                .text_pool_start
                .checked_add(self.text_offsets[index] as usize)
                .ok_or_else(|| GscError::invalid("physical text offset overflow"))?;
            let offset = u32::try_from(physical_offset)
                .map_err(|_| GscError::invalid("physical text offset exceeds u32"))?;
            let size = u32::try_from(raw.len())
                .map_err(|_| GscError::invalid("text byte length exceeds u32"))?;

            let mut entry = TextEntry {
                name: None,
                scr_msg: String::new(),
                message: String::new(),
                file: file_name.to_owned(),
                index: u32::try_from(index)
                    .map_err(|_| GscError::invalid("text index exceeds u32"))?,
                offset,
                size,
                entry_type: String::new(),
                encoding: "cp932".to_owned(),
                policy: "relocate".to_owned(),
                instruction_offset: None,
                opcode: None,
                target: None,
                choice_style: None,
            };

            match context {
                TextContext::Choice { visible, style, .. } => {
                    entry.scr_msg = script_text_to_json(visible);
                    "choice".clone_into(&mut entry.entry_type);
                    entry.choice_style = Some(style);
                    if let Some(reference) = choice {
                        entry.instruction_offset = Some(reference.instruction_offset);
                        entry.opcode = Some("0x000E".to_owned());
                        entry.target = Some(format!("0x{:08X}", reference.target));
                    }
                }
                TextContext::Dialogue { name, body, .. } => {
                    entry.name = Some(name.to_owned());
                    entry.scr_msg = script_text_to_json(body);
                    "dialogue".clone_into(&mut entry.entry_type);
                }
                TextContext::Monologue(text) => {
                    entry.scr_msg = script_text_to_json(text);
                    "monologue".clone_into(&mut entry.entry_type);
                }
            }
            entry.message.clone_from(&entry.scr_msg);
            entries.push(entry);
        }
        Ok(entries)
    }

    /// Rebuild the text offset table and CP932 text pool from translated JSON entries.
    ///
    /// All immutable locator fields and `scr_msg` values are checked against the
    /// current GSC before any output is produced. Code, sequence, and symbol
    /// regions are copied byte-for-byte.
    ///
    /// # Errors
    ///
    /// Returns an error for missing/duplicate entries, modified source metadata,
    /// NUL characters, unencodable text, or any rebuilt u32 size overflow.
    pub fn rebuild_from_entries(
        &self,
        file_name: &str,
        translated: &[TextEntry],
    ) -> Result<Vec<u8>, GscError> {
        let translated_by_index = self.validate_translations(file_name, translated)?;
        let (rebuilt_offsets, rebuilt_pool) =
            self.build_text_pool(file_name, &translated_by_index)?;

        let rebuilt_pool_size = u32::try_from(rebuilt_pool.len())
            .map_err(|_| GscError::translation("rebuilt text pool size exceeds u32"))?;
        let old_tail = &self.data[self.text_pool_end..];
        let new_file_size = self
            .text_index_start
            .checked_add(self.header.text_index_size as usize)
            .and_then(|size| size.checked_add(rebuilt_pool.len()))
            .and_then(|size| size.checked_add(old_tail.len()))
            .ok_or_else(|| GscError::translation("rebuilt GSC size overflows this platform"))?;
        let new_file_size_u32 = u32::try_from(new_file_size)
            .map_err(|_| GscError::translation("rebuilt GSC size exceeds u32"))?;

        let mut output = Vec::with_capacity(new_file_size);
        output.extend_from_slice(&self.data[..self.text_index_start]);
        for offset in rebuilt_offsets {
            output.extend_from_slice(&offset.to_le_bytes());
        }
        output.extend_from_slice(&rebuilt_pool);
        output.extend_from_slice(old_tail);
        write_u32(&mut output, 0, new_file_size_u32)?;
        write_u32(&mut output, 16, rebuilt_pool_size)?;

        let reparsed = GscFile::parse(&output)?;
        if reparsed.header.code_size != self.header.code_size
            || output[GSC_HEADER_SIZE..self.text_index_start]
                != self.data[GSC_HEADER_SIZE..self.text_index_start]
            || output[reparsed.text_pool_end..] != self.data[self.text_pool_end..]
        {
            return Err(GscError::translation(format!(
                "{file_name}: rebuilt non-text regions changed unexpectedly"
            )));
        }

        Ok(output)
    }

    fn validate_translations<'b>(
        &self,
        file_name: &str,
        translated: &'b [TextEntry],
    ) -> Result<HashMap<u32, &'b TextEntry>, GscError> {
        let expected = self.extract_entries(file_name)?;
        if translated.len() != expected.len() {
            return Err(GscError::translation(format!(
                "{file_name}: JSON contains {} entries, expected {}",
                translated.len(),
                expected.len()
            )));
        }

        let mut translated_by_index = HashMap::with_capacity(translated.len());
        for entry in translated {
            if translated_by_index.insert(entry.index, entry).is_some() {
                return Err(GscError::translation(format!(
                    "{file_name}: duplicate JSON text index {}",
                    entry.index
                )));
            }
        }

        let mut expected_indices = HashSet::with_capacity(expected.len());
        for source in &expected {
            expected_indices.insert(source.index);
            let translated_entry = translated_by_index.get(&source.index).ok_or_else(|| {
                GscError::translation(format!(
                    "{file_name}: missing JSON text index {}",
                    source.index
                ))
            })?;
            validate_immutable_fields(source, translated_entry)?;
        }
        if let Some(extra) = translated_by_index
            .keys()
            .find(|index| !expected_indices.contains(index))
        {
            return Err(GscError::translation(format!(
                "{file_name}: unexpected JSON text index {extra}"
            )));
        }
        Ok(translated_by_index)
    }

    fn build_text_pool(
        &self,
        file_name: &str,
        translated_by_index: &HashMap<u32, &TextEntry>,
    ) -> Result<(Vec<u32>, Vec<u8>), GscError> {
        let choices = self.find_choice_references()?;
        let mut rebuilt_offsets = Vec::with_capacity(self.text_bytes.len());
        let mut rebuilt_pool = Vec::with_capacity(self.header.text_pool_size as usize);

        for (index, original_raw) in self.text_bytes.iter().enumerate() {
            let offset = u32::try_from(rebuilt_pool.len())
                .map_err(|_| GscError::translation("rebuilt text pool offset exceeds u32"))?;
            rebuilt_offsets.push(offset);

            let index_u32 = u32::try_from(index)
                .map_err(|_| GscError::translation("text index exceeds u32"))?;
            if let Some(entry) = translated_by_index.get(&index_u32) {
                let decoded = decode_cp932(original_raw, index)?;
                let context = text_context(&decoded, choices.contains_key(&index))?;
                rebuilt_pool.extend_from_slice(&encode_visible_text(
                    original_raw,
                    &context,
                    &entry.message,
                    file_name,
                    index,
                )?);
            } else {
                rebuilt_pool.extend_from_slice(original_raw);
            }
            rebuilt_pool.push(0);
        }
        Ok((rebuilt_offsets, rebuilt_pool))
    }

    fn find_choice_references(&self) -> Result<HashMap<usize, ChoiceReference>, GscError> {
        let code = &self.data[GSC_HEADER_SIZE..self.text_index_start];
        let mut choices = HashMap::new();
        if code.len() < 60 {
            return Ok(choices);
        }

        for offset in 0..=code.len() - 60 {
            if read_u16_unchecked(code, offset) != 0x000e {
                continue;
            }
            let count_and_flags = read_u16_unchecked(code, offset + 2);
            let count = usize::from(count_and_flags & 0x00ff);
            if !(1..=5).contains(&count) {
                continue;
            }
            let prompt_index = read_u32_unchecked(code, offset + 4) as usize;
            if prompt_index >= self.text_count() {
                continue;
            }
            if (0..3).any(|index| read_u32_unchecked(code, offset + 0x30 + index * 4) != 0) {
                continue;
            }

            let mut slots = Vec::with_capacity(count);
            let mut valid = true;
            for slot in 0..count {
                let target = read_u32_unchecked(code, offset + 8 + slot * 4);
                let text_index = read_u32_unchecked(code, offset + 0x1c + slot * 4) as usize;
                if target as usize >= code.len()
                    || text_index == 0
                    || text_index >= self.text_count()
                {
                    valid = false;
                    break;
                }
                let decoded = decode_cp932(self.text_bytes[text_index], text_index)?;
                if parse_choice_prefix(&decoded).is_none() {
                    valid = false;
                    break;
                }
                slots.push((text_index, target));
            }
            if !valid {
                continue;
            }

            let physical_offset = u32::try_from(GSC_HEADER_SIZE + offset)
                .map_err(|_| GscError::invalid("choice instruction offset exceeds u32"))?;
            for (text_index, target) in slots {
                let reference = ChoiceReference {
                    instruction_offset: physical_offset,
                    target,
                };
                if let Some(previous) = choices.insert(text_index, reference) {
                    return Err(GscError::invalid(format!(
                        "choice text index {text_index} is referenced by multiple candidate instructions at 0x{:x} and 0x{physical_offset:x}",
                        previous.instruction_offset
                    )));
                }
            }
        }
        Ok(choices)
    }
}

fn calculate_file_size(header: &GscHeader) -> Result<usize, GscError> {
    let fields = [
        u64::from(header.header_size),
        u64::from(header.code_size),
        u64::from(header.text_index_size),
        u64::from(header.text_pool_size),
        u64::from(header.sequence_index_size),
        u64::from(header.sequence_pool_count) * 2,
        u64::from(header.symbol_table_size) * 2,
        u64::from(header.symbol_name_pool_size),
    ];
    let total = fields
        .into_iter()
        .try_fold(0_u64, u64::checked_add)
        .ok_or_else(|| GscError::invalid("declared region sizes overflow u64"))?;
    usize::try_from(total).map_err(|_| GscError::invalid("file size does not fit this platform"))
}

fn checked_add(base: usize, size: u32, label: &str) -> Result<usize, GscError> {
    base.checked_add(size as usize)
        .ok_or_else(|| GscError::invalid(format!("{label} overflows this platform")))
}

fn text_context(text: &str, is_choice: bool) -> Result<TextContext<'_>, GscError> {
    if is_choice {
        let Some((prefix, visible, style)) = parse_choice_prefix(text) else {
            return Err(GscError::invalid(
                "choice-referenced text has no valid <NN> style prefix",
            ));
        };
        return Ok(TextContext::Choice {
            prefix,
            visible,
            style,
        });
    }
    if let Some((prefix, name, body)) = split_speaker(text) {
        return Ok(TextContext::Dialogue { prefix, name, body });
    }
    Ok(TextContext::Monologue(text))
}

fn split_speaker(text: &str) -> Option<(&str, &str, &str)> {
    let rest = text.strip_prefix('【')?;
    let delimiter = rest.find("】^n")?;
    if delimiter == 0 {
        return None;
    }
    let prefix_end = '【'.len_utf8() + delimiter + "】^n".len();
    let name_start = '【'.len_utf8();
    let name_end = name_start + delimiter;
    Some((
        &text[..prefix_end],
        &text[name_start..name_end],
        &text[prefix_end..],
    ))
}

fn parse_choice_prefix(text: &str) -> Option<(&str, &str, u32)> {
    let rest = text.strip_prefix('<')?;
    let end = rest.find('>')?;
    let digits = &rest[..end];
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let style = digits.parse().ok()?;
    let prefix_end = 1 + end + 1;
    Some((&text[..prefix_end], &text[prefix_end..], style))
}

fn validate_immutable_fields(source: &TextEntry, translated: &TextEntry) -> Result<(), GscError> {
    let index = source.index;
    macro_rules! ensure_equal {
        ($field:ident) => {
            if source.$field != translated.$field {
                return Err(GscError::translation(format!(
                    "{} index {index}: immutable field {} changed",
                    source.file,
                    stringify!($field)
                )));
            }
        };
    }

    ensure_equal!(name);
    ensure_equal!(scr_msg);
    ensure_equal!(file);
    ensure_equal!(offset);
    ensure_equal!(size);
    ensure_equal!(entry_type);
    ensure_equal!(encoding);
    ensure_equal!(policy);
    ensure_equal!(instruction_offset);
    ensure_equal!(opcode);
    ensure_equal!(target);
    ensure_equal!(choice_style);
    Ok(())
}

fn decode_cp932(raw: &[u8], index: usize) -> Result<String, GscError> {
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(raw);
    if had_errors {
        return Err(GscError::encoding(format!(
            "text index {index} contains invalid CP932 bytes"
        )));
    }
    Ok(decoded.into_owned())
}

fn encode_cp932(text: &str, file_name: &str, index: usize) -> Result<Vec<u8>, GscError> {
    if text.contains('\0') {
        return Err(GscError::translation(format!(
            "{file_name} index {index}: message contains NUL"
        )));
    }
    let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        return Err(GscError::encoding(format!(
            "{file_name} index {index} contains characters that cannot be encoded as CP932"
        )));
    }
    Ok(encoded.into_owned())
}

fn encode_visible_text(
    original_raw: &[u8],
    context: &TextContext<'_>,
    message: &str,
    file_name: &str,
    index: usize,
) -> Result<Vec<u8>, GscError> {
    let script_message = json_text_to_script(message, file_name, index)?;
    match context {
        TextContext::Choice { prefix, .. } | TextContext::Dialogue { prefix, .. } => {
            let encoded_prefix = encode_cp932(prefix, file_name, index)?;
            let original_prefix = original_raw.get(..encoded_prefix.len()).ok_or_else(|| {
                GscError::translation(format!(
                    "{file_name} index {index}: original prefix is shorter than expected"
                ))
            })?;
            if decode_cp932(original_prefix, index)? != *prefix {
                return Err(GscError::translation(format!(
                    "{file_name} index {index}: original prefix bytes do not roundtrip"
                )));
            }
            let encoded_message = encode_cp932(&script_message, file_name, index)?;
            let mut rebuilt = Vec::with_capacity(original_prefix.len() + encoded_message.len());
            rebuilt.extend_from_slice(original_prefix);
            rebuilt.extend_from_slice(&encoded_message);
            Ok(rebuilt)
        }
        TextContext::Monologue(_) => encode_cp932(&script_message, file_name, index),
    }
}

/// Convert engine markup into the translator-facing representation.
///
/// Forced line breaks become JSON newlines. Ruby readings and their optional
/// span marker are intentionally discarded, leaving only the visible base text.
fn script_text_to_json(text: &str) -> String {
    let without_ruby = strip_ruby(text);
    without_ruby.replace("^n", "\n")
}

fn json_text_to_script(text: &str, file_name: &str, index: usize) -> Result<String, GscError> {
    let without_ruby = strip_ruby(text);
    let mut output = String::with_capacity(without_ruby.len());
    let mut chars = without_ruby.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '\r' => {
                if chars.next_if_eq(&'\n').is_none() {
                    return Err(GscError::translation(format!(
                        "{file_name} index {index}: message contains a bare carriage return"
                    )));
                }
                output.push_str("^n");
            }
            '\n' => output.push_str("^n"),
            _ => output.push(character),
        }
    }
    Ok(output)
}

fn strip_ruby(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0_usize;
    let mut ruby_base_available = false;

    while cursor < text.len() {
        let remainder = &text[cursor..];
        let character = remainder
            .chars()
            .next()
            .expect("cursor should remain on a character boundary");

        if character == '|' && pipe_starts_ruby_span(remainder) {
            cursor += character.len_utf8();
            continue;
        }

        if character == '['
            && ruby_base_available
            && let Some(closing_offset) = ruby_closing_offset(remainder)
        {
            cursor += closing_offset + ']'.len_utf8();
            ruby_base_available = false;
            continue;
        }

        if let Some(control_len) = script_control_len(remainder) {
            output.push_str(&remainder[..control_len]);
            cursor += control_len;
            ruby_base_available = false;
            continue;
        }

        output.push(character);
        cursor += character.len_utf8();
        ruby_base_available = can_precede_ruby(character);
    }

    output
}

fn pipe_starts_ruby_span(text: &str) -> bool {
    let Some(after_pipe) = text.strip_prefix('|') else {
        return false;
    };
    let mut base_len = 0_usize;
    for (offset, character) in after_pipe.char_indices() {
        if character == '[' {
            return base_len > 0 && ruby_closing_offset(&after_pipe[offset..]).is_some();
        }
        if matches!(character, '|' | ']' | '^' | '\r' | '\n') {
            return false;
        }
        base_len += character.len_utf8();
    }
    false
}

fn ruby_closing_offset(text: &str) -> Option<usize> {
    let after_open = text.strip_prefix('[')?;
    if after_open.is_empty() {
        return None;
    }
    for (offset, character) in after_open.char_indices() {
        match character {
            ']' if offset > 0 => return Some('['.len_utf8() + offset),
            '[' | '\r' | '\n' => return None,
            _ => {}
        }
    }
    None
}

fn can_precede_ruby(character: char) -> bool {
    !character.is_whitespace() && !matches!(character, '|' | '[' | ']' | '^')
}

fn script_control_len(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    match bytes {
        [b'^', b'n', ..] => Some(2),
        [b'^', b'd' | b's', digit, ..] if digit.is_ascii_digit() => Some(3),
        _ => None,
    }
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32, GscError> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or_else(|| GscError::invalid(format!("missing u32 at 0x{offset:x}")))?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_u16_unchecked(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn read_u32_unchecked(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

fn write_u32(data: &mut [u8], offset: usize, value: u32) -> Result<(), GscError> {
    let destination = data
        .get_mut(offset..offset + 4)
        .ok_or_else(|| GscError::invalid(format!("missing output u32 at 0x{offset:x}")))?;
    destination.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(text: &str) -> Vec<u8> {
        let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
        assert!(!had_errors);
        encoded.into_owned()
    }

    fn build_gsc() -> Vec<u8> {
        let texts = [
            "",
            "<01>|菅笠[すげかさ]^n娘[おなご]",
            "【智久】^n本文^n次行",
            "grpo",
        ];
        let mut code = vec![0_u8; 60];
        code[0..2].copy_from_slice(&0x000e_u16.to_le_bytes());
        code[2..4].copy_from_slice(&1_u16.to_le_bytes());
        code[0x1c..0x20].copy_from_slice(&1_u32.to_le_bytes());

        let mut offsets = Vec::new();
        let mut pool = Vec::new();
        for text in texts {
            offsets.push(u32::try_from(pool.len()).expect("test pool should fit u32"));
            pool.extend_from_slice(&encode(text));
            pool.push(0);
        }

        let file_size = GSC_HEADER_SIZE + code.len() + offsets.len() * 4 + pool.len();
        let mut output = Vec::new();
        output.extend_from_slice(
            &u32::try_from(file_size)
                .expect("test file should fit u32")
                .to_le_bytes(),
        );
        output.extend_from_slice(&GSC_HEADER_SIZE_U32.to_le_bytes());
        output.extend_from_slice(
            &u32::try_from(code.len())
                .expect("test code should fit u32")
                .to_le_bytes(),
        );
        output.extend_from_slice(
            &u32::try_from(offsets.len() * 4)
                .expect("test index should fit u32")
                .to_le_bytes(),
        );
        output.extend_from_slice(
            &u32::try_from(pool.len())
                .expect("test pool should fit u32")
                .to_le_bytes(),
        );
        output.extend_from_slice(&0_u32.to_le_bytes());
        output.extend_from_slice(&0_u32.to_le_bytes());
        output.extend_from_slice(&0_u32.to_le_bytes());
        output.extend_from_slice(&0_u32.to_le_bytes());
        output.extend_from_slice(&code);
        for offset in offsets {
            output.extend_from_slice(&offset.to_le_bytes());
        }
        output.extend_from_slice(&pool);
        output
    }

    #[test]
    fn extracts_choice_dialogue_and_skips_resources() {
        let data = build_gsc();
        let gsc = GscFile::parse(&data).expect("test GSC should parse");
        let entries = gsc
            .extract_entries("test.gsc")
            .expect("test text should extract");

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].entry_type, "choice");
        assert_eq!(entries[0].scr_msg, "菅笠\n娘");
        assert_eq!(entries[0].choice_style, Some(1));
        assert_eq!(entries[1].entry_type, "dialogue");
        assert_eq!(entries[1].name.as_deref(), Some("智久"));
        assert_eq!(entries[1].scr_msg, "本文\n次行");
    }

    #[test]
    fn zero_translation_rebuild_removes_ruby_and_is_stable() {
        let data = build_gsc();
        let gsc = GscFile::parse(&data).expect("test GSC should parse");
        let entries = gsc
            .extract_entries("test.gsc")
            .expect("test text should extract");
        let rebuilt = gsc
            .rebuild_from_entries("test.gsc", &entries)
            .expect("test GSC should rebuild");
        let reparsed = GscFile::parse(&rebuilt).expect("rebuilt GSC should parse");
        let rebuilt_entries = reparsed
            .extract_entries("test.gsc")
            .expect("rebuilt text should extract");

        let rebuilt_again = reparsed
            .rebuild_from_entries("test.gsc", &rebuilt_entries)
            .expect("normalized GSC should rebuild again");

        assert_ne!(rebuilt, data);
        assert_eq!(rebuilt_entries[0].scr_msg, entries[0].scr_msg);
        assert_eq!(rebuilt_entries[1].scr_msg, entries[1].scr_msg);
        assert_eq!(rebuilt_again, rebuilt);
        assert_eq!(
            &rebuilt[GSC_HEADER_SIZE..GSC_HEADER_SIZE + 60],
            &data[GSC_HEADER_SIZE..GSC_HEADER_SIZE + 60]
        );
        assert!(
            !decode_cp932(reparsed.text_bytes[1], 1)
                .expect("rebuilt choice should decode")
                .contains(['|', '[', ']'])
        );
    }

    #[test]
    fn variable_length_rebuild_preserves_code_and_metadata() {
        let data = build_gsc();
        let gsc = GscFile::parse(&data).expect("test GSC should parse");
        let mut entries = gsc
            .extract_entries("test.gsc")
            .expect("test text should extract");
        entries[0].message = "かなり長い選択肢".to_owned();
        entries[1].message = "長くなった本文です".to_owned();

        let rebuilt = gsc
            .rebuild_from_entries("test.gsc", &entries)
            .expect("test GSC should rebuild");
        let reparsed = GscFile::parse(&rebuilt).expect("rebuilt GSC should parse");
        let rebuilt_entries = reparsed
            .extract_entries("test.gsc")
            .expect("rebuilt text should extract");

        assert_eq!(
            &rebuilt[GSC_HEADER_SIZE..GSC_HEADER_SIZE + 60],
            &data[GSC_HEADER_SIZE..GSC_HEADER_SIZE + 60]
        );
        assert_eq!(rebuilt_entries[0].scr_msg, "かなり長い選択肢");
        assert_eq!(rebuilt_entries[1].scr_msg, "長くなった本文です");
    }

    #[test]
    fn normalizes_forced_breaks_and_ruby_markup() {
        assert_eq!(
            script_text_to_json("|菅笠[すげかさ]を娘[おなご]^n次^n"),
            "菅笠を娘\n次\n"
        );
        assert_eq!(
            json_text_to_script("菅笠を娘\r\n次\n", "test.gsc", 7)
                .expect("JSON line breaks should convert"),
            "菅笠を娘^n次^n"
        );
        assert_eq!(
            json_text_to_script("|外套[コート]\n", "test.gsc", 8)
                .expect("ruby added to a translation should be removed"),
            "外套^n"
        );
    }

    #[test]
    fn preserves_malformed_ruby_and_rejects_bare_carriage_returns() {
        assert_eq!(
            script_text_to_json("記号|だけ [未完 [] ^n[表示] ^d1[表示]"),
            "記号|だけ [未完 [] \n[表示] ^d1[表示]"
        );
        let error = json_text_to_script("一行\r二行", "test.gsc", 9)
            .expect_err("bare carriage return must fail");
        assert!(error.to_string().contains("bare carriage return"));
    }

    #[test]
    fn rejects_modified_source_text() {
        let data = build_gsc();
        let gsc = GscFile::parse(&data).expect("test GSC should parse");
        let mut entries = gsc
            .extract_entries("test.gsc")
            .expect("test text should extract");
        entries[0].scr_msg = "改変".to_owned();

        let error = gsc
            .rebuild_from_entries("test.gsc", &entries)
            .expect_err("modified source text must fail");
        assert!(error.to_string().contains("immutable field scr_msg"));
    }

    #[test]
    fn rejects_unencodable_message() {
        let data = build_gsc();
        let gsc = GscFile::parse(&data).expect("test GSC should parse");
        let mut entries = gsc
            .extract_entries("test.gsc")
            .expect("test text should extract");
        entries[0].message = "emoji: 😀".to_owned();

        let error = gsc
            .rebuild_from_entries("test.gsc", &entries)
            .expect_err("unencodable text must fail");
        assert!(error.to_string().contains("cannot be encoded"));
    }
}
