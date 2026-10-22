//! Loss-preserving parser and rebuilder for the game's XOR-obfuscated MES
//! text files.
//!
//! Parsing retains the complete decoded file. Only message/extra bodies and
//! individual choice strings are exposed as translatable units; index fields,
//! segment markers, choice counts, terminators, gaps, and unknown bytes remain
//! owned by the document and are copied unchanged by MesDocument::rebuild.

use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const XOR_KEY: u8 = 0x54;
const MAX_FILE_LENGTH: usize = 0xC350;
const INDEX_ENTRY_LENGTH: usize = 4;
const SPEAKER_COUNT: u8 = 24;

/// Decode the byte-level XOR transform used by MES files.
///
/// Raw 00 and 54 bytes are sentinels and are copied unchanged; every other
/// byte is XORed with 54. The operation is its own inverse.
pub fn decode_file(encoded: &[u8]) -> Result<Vec<u8>, String> {
    if encoded.len() > MAX_FILE_LENGTH {
        return Err(format!(
            "encoded MES length {} exceeds {} bytes",
            encoded.len(),
            MAX_FILE_LENGTH
        ));
    }
    Ok(encoded.iter().copied().map(xor_byte).collect())
}

/// Encode decoded MES bytes with the game's XOR transform.
pub fn encode_file(decoded: &[u8]) -> Result<Vec<u8>, String> {
    if decoded.len() > MAX_FILE_LENGTH {
        return Err(format!(
            "decoded MES length {} exceeds {} bytes",
            decoded.len(),
            MAX_FILE_LENGTH
        ));
    }
    Ok(decoded.iter().copied().map(xor_byte).collect())
}

fn xor_byte(byte: u8) -> u8 {
    if byte == 0 || byte == XOR_KEY {
        byte
    } else {
        byte ^ XOR_KEY
    }
}

/// Stable source locator for one translatable unit.
///
/// segment_index is the zero-based ordinal of every dollar-prefixed segment
/// in the record, starting with its $L label. choice_index is None for $M/$X
/// bodies and Some(n) for the nth $C option.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UnitLocator {
    pub record_index: usize,
    pub segment_index: usize,
    pub choice_index: Option<usize>,
}

/// The native kind of a translatable MES unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextUnitKind {
    Message,
    Extra,
    Choice,
}

/// One translation-facing unit in stable source order.
///
/// message uses the template controls documented by MesDocument::encode_template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextUnit {
    pub locator: UnitLocator,
    pub record_key: u16,
    pub kind: TextUnitKind,
    /// Exact decoded template, including display controls and half-width kana.
    pub source_message: String,
    /// Translator-facing template with redundant KANA toggles hidden and
    /// half-width katakana normalized to full-width forms.
    pub message: String,
}

/// Return visible text from a template, omitting game control tokens.
///
/// Newlines are retained. Valid CP932 bytes escaped as RAW tokens are decoded
/// back to Unicode; undecodable raw bytes are omitted from the font-planning
/// view while remaining available to encode_template for lossless rebuilding.
pub fn plain_text(template: &str) -> String {
    let mut output = String::new();
    let mut cursor = 0usize;
    while cursor < template.len() {
        let rest = &template[cursor..];
        if let Some((token, token_len)) = parse_template_token(rest) {
            if let TemplateToken::Raw(bytes) = token {
                let (text, had_errors) = SHIFT_JIS.decode_without_bom_handling(&bytes);
                if !had_errors {
                    output.push_str(&text);
                }
            }
            cursor += token_len;
            continue;
        }
        let Some(character) = rest.chars().next() else {
            break;
        };
        output.push(character);
        cursor += character.len_utf8();
    }
    output
}

#[derive(Debug, Clone)]
struct IndexEntry {
    key: u16,
    offset: usize,
}

#[derive(Debug, Clone)]
struct Record {
    key: u16,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone)]
struct UnitSpan {
    locator: UnitLocator,
    record_key: u16,
    kind: TextUnitKind,
    start: usize,
    end: usize,
}

/// Parsed MES data with original decoded bytes retained for safe rebuilding.
#[derive(Debug, Clone)]
pub struct MesDocument {
    decoded: Vec<u8>,
    table_end: usize,
    records: Vec<Record>,
    units: Vec<TextUnit>,
    spans: BTreeMap<UnitLocator, UnitSpan>,
}

/// Decode and parse an encoded MES file.
///
/// The index consists of big-endian (key, offset) pairs and ends at the first
/// all-zero pair. Every offset must be strictly greater than its predecessor,
/// land after the index, and point to $L followed by the same big-endian key.
pub fn parse(encoded: &[u8]) -> Result<MesDocument, String> {
    let decoded = decode_file(encoded)?;
    if decoded.len() < INDEX_ENTRY_LENGTH {
        return Err(file_error(0, "file is too short to contain an index entry"));
    }

    let mut entries = Vec::new();
    let mut table_cursor = 0usize;
    let table_end = loop {
        let Some(entry_end) = table_cursor.checked_add(INDEX_ENTRY_LENGTH) else {
            return Err(file_error(table_cursor, "index position overflow"));
        };
        if entry_end > decoded.len() {
            return Err(file_error(
                table_cursor,
                "index has no all-zero terminator entry",
            ));
        }
        let key = read_be_u16(&decoded, table_cursor)?;
        let offset = read_be_u16(&decoded, table_cursor + 2)?;
        if key == 0 && offset == 0 {
            break entry_end;
        }
        if offset == 0 {
            return Err(file_error(
                table_cursor,
                "non-terminal index entry has a zero offset",
            ));
        }
        entries.push(IndexEntry {
            key,
            offset: usize::from(offset),
        });
        table_cursor = entry_end;
    };

    if entries.is_empty() {
        return Err(file_error(0, "index contains no records"));
    }
    let first_offset = entries[0].offset;
    if first_offset < table_end {
        return Err(file_error(
            first_offset,
            "first record overlaps the index table",
        ));
    }
    if first_offset >= decoded.len() {
        return Err(file_error(
            first_offset,
            "first record offset is outside the file",
        ));
    }

    let mut previous_offset = None;
    for entry in &entries {
        if let Some(previous) = previous_offset {
            if entry.offset <= previous {
                return Err(file_error(
                    entry.offset,
                    "record offsets are not strictly increasing",
                ));
            }
        }
        if entry.offset < table_end {
            return Err(file_error(
                entry.offset,
                "record offset overlaps the index table",
            ));
        }
        if entry.offset >= decoded.len() {
            return Err(file_error(
                entry.offset,
                "record offset is outside the file",
            ));
        }
        let Some(label_end) = entry.offset.checked_add(4) else {
            return Err(file_error(entry.offset, "label position overflow"));
        };
        if label_end > decoded.len()
            || decoded.get(entry.offset..entry.offset + 2) != Some(b"$L".as_slice())
        {
            return Err(record_error(
                entry.key,
                entry.offset,
                "index offset does not point to a complete $L label",
            ));
        }
        let label_key = read_be_u16(&decoded, entry.offset + 2)?;
        if label_key != entry.key {
            return Err(record_error(
                entry.key,
                entry.offset,
                &format!("$L contains key 0x{label_key:04X}"),
            ));
        }
        previous_offset = Some(entry.offset);
    }

    let mut records = Vec::with_capacity(entries.len());
    let mut units = Vec::new();
    let mut spans = BTreeMap::new();
    for (record_index, entry) in entries.iter().enumerate() {
        let end = entries
            .get(record_index + 1)
            .map_or(decoded.len(), |next| next.offset);
        let record = Record {
            key: entry.key,
            start: entry.offset,
            end,
        };
        parse_record(&decoded, record_index, &record, &mut units, &mut spans)?;
        records.push(record);
    }

    Ok(MesDocument {
        decoded,
        table_end,
        records,
        units,
        spans,
    })
}

impl MesDocument {
    /// Return translatable $M, $X, and $C units in file order.
    pub fn units(&self) -> Vec<TextUnit> {
        self.units.clone()
    }

    /// Convert one template string to native bytes.
    ///
    /// Ordinary text is passed to encode_text, which should map each character
    /// to a valid two-byte CP932 carrier. `$M` accepts [[SPK:NN]], [[WAIT]],
    /// [[KANA]], [[ATTR:d]], and LF. `$X` accepts LF only; `$C` options accept
    /// neither controls nor line breaks. All kinds accept `[[RAW:HH...]]`.
    ///
    /// If the template is unchanged, the exact original body is returned,
    /// including non-canonical attribute encodings and undecodable bytes.
    pub fn encode_template<F>(
        &self,
        locator: &UnitLocator,
        template: &str,
        mut encode_text: F,
    ) -> Result<Vec<u8>, String>
    where
        F: FnMut(&str) -> Result<Vec<u8>, String>,
    {
        let span = self
            .spans
            .get(locator)
            .ok_or_else(|| format!("unknown MES unit locator {locator:?}"))?;
        let source_unit = self
            .units
            .iter()
            .find(|unit| unit.locator == *locator)
            .ok_or_else(|| format!("MES unit metadata is missing for {locator:?}"))?;
        if template == source_unit.message {
            return Ok(self.decoded[span.start..span.end].to_vec());
        }

        let mut output = Vec::new();
        let mut cursor = 0usize;
        let mut plain_start = 0usize;
        while cursor < template.len() {
            let rest = &template[cursor..];
            if let Some((token, token_len)) = parse_template_token(rest) {
                append_encoded_text(
                    &template[plain_start..cursor],
                    &mut encode_text,
                    &mut output,
                )?;
                match token {
                    TemplateToken::Raw(bytes) => output.extend(bytes),
                    TemplateToken::Speaker(index) if span.kind == TextUnitKind::Message => {
                        output.extend([b'#', index]);
                    }
                    TemplateToken::Wait if span.kind == TextUnitKind::Message => output.push(b'@'),
                    TemplateToken::Kana if span.kind == TextUnitKind::Message => output.push(b'*'),
                    TemplateToken::Attribute(digit) if span.kind == TextUnitKind::Message => {
                        output.extend([b'^', digit]);
                    }
                    _ => {
                        return Err(format!(
                            "MES record key 0x{:04X}: control token is not valid in {:?} at template byte 0x{cursor:X}",
                            span.record_key, span.kind
                        ));
                    }
                }
                cursor += token_len;
                plain_start = cursor;
                continue;
            }

            if rest.starts_with("[[") {
                return Err(format!(
                    "MES record key 0x{:04X}: malformed or unknown control token at template byte 0x{cursor:X}",
                    span.record_key
                ));
            }

            let character = rest
                .chars()
                .next()
                .ok_or_else(|| "invalid UTF-8 cursor in translation template".to_owned())?;
            if character == '\n' {
                if span.kind == TextUnitKind::Choice {
                    return Err(format!(
                        "MES record key 0x{:04X}: line breaks are not valid in a choice string",
                        span.record_key
                    ));
                }
                append_encoded_text(
                    &template[plain_start..cursor],
                    &mut encode_text,
                    &mut output,
                )?;
                output.push(0x0D);
                cursor += character.len_utf8();
                plain_start = cursor;
            } else {
                cursor += character.len_utf8();
            }
        }
        append_encoded_text(&template[plain_start..], &mut encode_text, &mut output)?;
        validate_replacement(span.kind, &output).map_err(|error| {
            record_error(
                span.record_key,
                span.start,
                &format!("invalid replacement for {locator:?}: {error}"),
            )
        })?;
        Ok(output)
    }

    /// Rebuild the complete encoded file after replacing selected unit bodies.
    ///
    /// Replacement values are native bytes for a unit body, for example those
    /// produced by encode_template. The method leaves omitted units and all
    /// non-text bytes unchanged, preserves segment and choice terminators,
    /// updates big-endian index offsets, then applies the XOR transform.
    pub fn rebuild(
        &self,
        replacements: &BTreeMap<UnitLocator, Vec<u8>>,
    ) -> Result<Vec<u8>, String> {
        for locator in replacements.keys() {
            if !self.spans.contains_key(locator) {
                return Err(format!(
                    "replacement refers to unknown MES unit {locator:?}"
                ));
            }
        }

        let first_record = self
            .records
            .first()
            .ok_or_else(|| "MES document has no records".to_owned())?;
        let mut decoded = self.decoded[..first_record.start].to_vec();
        if self.table_end > decoded.len() {
            return Err(file_error(
                self.table_end,
                "index table extends beyond the first record",
            ));
        }

        let mut new_offsets = Vec::with_capacity(self.records.len());
        for (record_index, record) in self.records.iter().enumerate() {
            let new_offset = decoded.len();
            if new_offset > usize::from(u16::MAX) {
                return Err(record_error(
                    record.key,
                    new_offset,
                    "rebuilt record offset does not fit in u16",
                ));
            }
            new_offsets.push(new_offset);

            let record_spans = self
                .spans
                .values()
                .filter(|span| span.locator.record_index == record_index)
                .collect::<Vec<_>>();
            let mut cursor = record.start;
            for span in record_spans {
                if span.start < cursor || span.end > record.end {
                    return Err(record_error(
                        record.key,
                        span.start,
                        "unit span overlaps or exceeds its record",
                    ));
                }
                decoded.extend_from_slice(&self.decoded[cursor..span.start]);
                if let Some(replacement) = replacements.get(&span.locator) {
                    validate_replacement(span.kind, replacement).map_err(|error| {
                        record_error(
                            record.key,
                            span.start,
                            &format!("invalid replacement for {:?}: {error}", span.locator),
                        )
                    })?;
                    decoded.extend_from_slice(replacement);
                } else {
                    decoded.extend_from_slice(&self.decoded[span.start..span.end]);
                }
                cursor = span.end;
            }
            decoded.extend_from_slice(&self.decoded[cursor..record.end]);
            if decoded.len() > MAX_FILE_LENGTH {
                return Err(record_error(
                    record.key,
                    new_offset,
                    &format!("rebuilt MES length exceeds {} bytes", MAX_FILE_LENGTH),
                ));
            }
        }

        if decoded.len() > MAX_FILE_LENGTH {
            return Err(file_error(
                decoded.len(),
                &format!("rebuilt MES length exceeds {} bytes", MAX_FILE_LENGTH),
            ));
        }
        for (index, offset) in new_offsets.iter().copied().enumerate() {
            if offset > usize::from(u16::MAX) {
                return Err(file_error(
                    offset,
                    "rebuilt record offset does not fit in u16",
                ));
            }
            let entry_start = index
                .checked_mul(INDEX_ENTRY_LENGTH)
                .ok_or_else(|| file_error(index, "index entry position overflow"))?;
            let offset_start = entry_start
                .checked_add(2)
                .ok_or_else(|| file_error(entry_start, "index offset position overflow"))?;
            let offset_end = offset_start
                .checked_add(2)
                .ok_or_else(|| file_error(offset_start, "index offset range overflow"))?;
            if offset_end > self.table_end || offset_end > decoded.len() {
                return Err(file_error(
                    offset_start,
                    "rebuilt index offset lies outside the index table",
                ));
            }
            decoded[offset_start..offset_end].copy_from_slice(&(offset as u16).to_be_bytes());
        }
        for (index, (record, offset)) in self
            .records
            .iter()
            .zip(new_offsets.iter().copied())
            .enumerate()
        {
            let label_end = offset.checked_add(4).ok_or_else(|| {
                record_error(record.key, offset, "rebuilt label position overflow")
            })?;
            if label_end > decoded.len()
                || decoded.get(offset..offset + 2) != Some(b"$L".as_slice())
                || read_be_u16(&decoded, offset + 2)? != record.key
            {
                return Err(record_error(
                    record.key,
                    offset,
                    &format!("rebuilt index entry {index} does not point to its $L label"),
                ));
            }
            if let Some(next) = new_offsets.get(index + 1) {
                if offset >= *next {
                    return Err(record_error(
                        record.key,
                        offset,
                        "rebuilt record offsets are not strictly increasing",
                    ));
                }
            }
        }
        encode_file(&decoded)
    }
}

fn parse_record(
    decoded: &[u8],
    record_index: usize,
    record: &Record,
    units: &mut Vec<TextUnit>,
    spans: &mut BTreeMap<UnitLocator, UnitSpan>,
) -> Result<(), String> {
    let mut cursor = record.start;
    let mut segment_index = 0usize;
    while cursor < record.end {
        match decoded[cursor] {
            0x1A => break,
            b'$' => {
                if cursor + 1 >= record.end {
                    break;
                }
                match decoded[cursor + 1] {
                    b'L' => {
                        if cursor + 4 > record.end {
                            return Err(record_error(record.key, cursor, "truncated $L segment"));
                        }
                        cursor += 4;
                        segment_index += 1;
                    }
                    b'M' | b'X' => {
                        let tag = decoded[cursor + 1];
                        let body_start = cursor + 2;
                        // Some verified records end at the next indexed $L
                        // label without an in-record terminator. In that case
                        // the index boundary is the text body's boundary.
                        let body_end =
                            find_body_end(decoded, body_start, record.end).unwrap_or(record.end);
                        let kind = if tag == b'M' {
                            TextUnitKind::Message
                        } else {
                            TextUnitKind::Extra
                        };
                        let locator = UnitLocator {
                            record_index,
                            segment_index,
                            choice_index: None,
                        };
                        push_unit(
                            decoded, record, locator, kind, body_start, body_end, units, spans,
                        )?;
                        cursor = body_end;
                        segment_index += 1;
                    }
                    b'C' => {
                        let count_at = cursor + 2;
                        if count_at >= record.end {
                            return Err(record_error(
                                record.key,
                                cursor,
                                "truncated $C choice count",
                            ));
                        }
                        let count = usize::from(decoded[count_at]);
                        let mut choice_cursor = count_at + 1;
                        for choice_index in 0..count {
                            let body_start = choice_cursor;
                            let Some(body_end) = find_choice_end(decoded, body_start, record.end)
                            else {
                                return Err(record_error(
                                    record.key,
                                    body_start,
                                    &format!(
                                        "choice {choice_index} has no single-byte 0x00 terminator"
                                    ),
                                ));
                            };
                            let locator = UnitLocator {
                                record_index,
                                segment_index,
                                choice_index: Some(choice_index),
                            };
                            push_unit(
                                decoded,
                                record,
                                locator,
                                TextUnitKind::Choice,
                                body_start,
                                body_end,
                                units,
                                spans,
                            )?;
                            choice_cursor = body_end + 1;
                        }
                        cursor = choice_cursor;
                        segment_index += 1;
                    }
                    _ => {
                        // Unknown markers are kept in the original byte stream.
                        // Advancing over the marker avoids interpreting its tag
                        // byte as a separate top-level segment.
                        cursor += 2;
                        segment_index += 1;
                    }
                }
            }
            _ => cursor += 1,
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn push_unit(
    decoded: &[u8],
    record: &Record,
    locator: UnitLocator,
    kind: TextUnitKind,
    start: usize,
    end: usize,
    units: &mut Vec<TextUnit>,
    spans: &mut BTreeMap<UnitLocator, UnitSpan>,
) -> Result<(), String> {
    if start > end || end > record.end {
        return Err(record_error(
            record.key,
            start,
            "translatable unit extends beyond its record",
        ));
    }
    let span = UnitSpan {
        locator: locator.clone(),
        record_key: record.key,
        kind,
        start,
        end,
    };
    let source_message = template_from_bytes(&decoded[start..end], kind);
    let message = translation_message(&source_message, kind);
    if spans.insert(locator.clone(), span).is_some() {
        return Err(record_error(
            record.key,
            start,
            &format!("duplicate text-unit locator {locator:?}"),
        ));
    }
    units.push(TextUnit {
        locator,
        record_key: record.key,
        kind,
        source_message,
        message,
    });
    Ok(())
}

fn find_body_end(decoded: &[u8], start: usize, record_end: usize) -> Option<usize> {
    decoded
        .get(start..record_end)?
        .iter()
        .position(|byte| *byte == b'$' || *byte == 0x1A)
        .map(|relative| start + relative)
}

/// Find a choice's single-byte NUL terminator while stepping over 2-byte chars.
fn find_choice_end(decoded: &[u8], start: usize, record_end: usize) -> Option<usize> {
    let mut cursor = start;
    while cursor < record_end {
        if decoded[cursor] == 0 {
            return Some(cursor);
        }
        let pair_end = cursor.checked_add(2)?;
        if pair_end > record_end || decoded[cursor + 1] == 0 {
            return None;
        }
        cursor = pair_end;
    }
    None
}

fn template_from_bytes(bytes: &[u8], kind: TextUnitKind) -> String {
    if kind != TextUnitKind::Message {
        return template_from_dbcs_pairs(bytes, kind == TextUnitKind::Extra);
    }
    let mut output = String::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor] == b'#' && cursor + 1 < bytes.len() && bytes[cursor + 1] < SPEAKER_COUNT {
            output.push_str(&format!("[[SPK:{:02}]]", bytes[cursor + 1]));
            cursor += 2;
            continue;
        }
        match bytes[cursor] {
            b'@' => {
                output.push_str("[[WAIT]]");
                cursor += 1;
                continue;
            }
            b'*' => {
                output.push_str("[[KANA]]");
                cursor += 1;
                continue;
            }
            b'^' if cursor + 1 < bytes.len() && bytes[cursor + 1].is_ascii_digit() => {
                output.push_str(&format!("[[ATTR:{}]]", bytes[cursor + 1] as char));
                cursor += 2;
                continue;
            }
            0x81 if cursor + 2 < bytes.len()
                && bytes[cursor + 1] == 0x4F
                && bytes[cursor + 2].is_ascii_digit() =>
            {
                output.push_str(&format!("[[ATTR:{}]]", bytes[cursor + 2] as char));
                cursor += 3;
                continue;
            }
            0x0D => {
                output.push('\n');
                cursor += 1;
                continue;
            }
            b'[' => {
                // Escape a source literal that could be confused for a control
                // template when the text is later passed to encode_template.
                output.push_str("[[RAW:5B]]");
                cursor += 1;
                continue;
            }
            0x0A => {
                output.push_str("[[RAW:0A]]");
                cursor += 1;
                continue;
            }
            _ => {}
        }

        let length = if is_sjis_lead(bytes[cursor]) && cursor + 1 < bytes.len() {
            2
        } else {
            1
        };
        let character = &bytes[cursor..cursor + length];
        let (text, had_errors) = SHIFT_JIS.decode_without_bom_handling(character);
        if had_errors {
            output.push_str(&format!("[[RAW:{:02X}]]", bytes[cursor]));
            cursor += 1;
        } else {
            output.push_str(&text);
            cursor += length;
        }
    }
    output
}

fn translation_message(source: &str, kind: TextUnitKind) -> String {
    if kind != TextUnitKind::Message {
        return source.to_owned();
    }
    normalize_halfwidth_katakana(&source.replace("[[KANA]]", ""))
}

/// Map half-width katakana and punctuation to full-width Japanese forms.
/// Half-width dakuten/handakuten compose with the preceding kana where possible.
pub fn normalize_halfwidth_katakana(text: &str) -> String {
    const FULLWIDTH: &str = "。「」、・ヲァィゥェォャュョッーアイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワン";
    let mut output = String::with_capacity(text.len());
    for character in text.chars() {
        if character == '\u{FF9E}' || character == '\u{FF9F}' {
            let voiced = character == '\u{FF9E}';
            if let Some(previous) = output.pop() {
                if let Some(composed) = compose_katakana(previous, voiced) {
                    output.push(composed);
                    continue;
                }
                output.push(previous);
            }
            output.push(if voiced { '\u{309B}' } else { '\u{309C}' });
            continue;
        }
        let mapped = if ('\u{FF61}'..='\u{FF9D}').contains(&character) {
            FULLWIDTH
                .chars()
                .nth((character as u32 - 0xFF61) as usize)
                .unwrap_or(character)
        } else {
            character
        };
        output.push(mapped);
    }
    output
}

fn compose_katakana(base: char, voiced: bool) -> Option<char> {
    if voiced {
        match base {
            'ウ' => Some('ヴ'),
            'カ' => Some('ガ'),
            'キ' => Some('ギ'),
            'ク' => Some('グ'),
            'ケ' => Some('ゲ'),
            'コ' => Some('ゴ'),
            'サ' => Some('ザ'),
            'シ' => Some('ジ'),
            'ス' => Some('ズ'),
            'セ' => Some('ゼ'),
            'ソ' => Some('ゾ'),
            'タ' => Some('ダ'),
            'チ' => Some('ヂ'),
            'ツ' => Some('ヅ'),
            'テ' => Some('デ'),
            'ト' => Some('ド'),
            'ハ' => Some('バ'),
            'ヒ' => Some('ビ'),
            'フ' => Some('ブ'),
            'ヘ' => Some('ベ'),
            'ホ' => Some('ボ'),
            'ワ' => Some('ヷ'),
            'ヰ' => Some('ヸ'),
            'ヱ' => Some('ヹ'),
            'ヲ' => Some('ヺ'),
            _ => None,
        }
    } else {
        match base {
            'ハ' => Some('パ'),
            'ヒ' => Some('ピ'),
            'フ' => Some('プ'),
            'ヘ' => Some('ペ'),
            'ホ' => Some('ポ'),
            _ => None,
        }
    }
}

fn template_from_dbcs_pairs(bytes: &[u8], cr_is_newline: bool) -> String {
    let mut output = String::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if cr_is_newline && bytes[cursor] == 0x0D {
            output.push('\n');
            cursor += 1;
            continue;
        }
        if cursor + 1 >= bytes.len() {
            output.push_str(&format!("[[RAW:{:02X}]]", bytes[cursor]));
            cursor += 1;
            continue;
        }
        let pair = &bytes[cursor..cursor + 2];
        let (text, had_errors) = SHIFT_JIS.decode_without_bom_handling(pair);
        if had_errors {
            output.push_str(&format!("[[RAW:{:02X}{:02X}]]", pair[0], pair[1]));
        } else {
            output.push_str(&text);
        }
        cursor += 2;
    }
    output
}

fn is_sjis_lead(byte: u8) -> bool {
    (0x81..=0x9F).contains(&byte) || (0xE0..=0xFC).contains(&byte)
}

fn is_sjis_trail(byte: u8) -> bool {
    (0x40..=0x7E).contains(&byte) || (0x80..=0xFC).contains(&byte)
}

fn append_encoded_text<F>(
    text: &str,
    encode_text: &mut F,
    output: &mut Vec<u8>,
) -> Result<(), String>
where
    F: FnMut(&str) -> Result<Vec<u8>, String>,
{
    if text.is_empty() {
        return Ok(());
    }
    let encoded = encode_text(text)?;
    if encoded.len() % 2 != 0 {
        return Err(format!(
            "ordinary text encoder returned {} bytes; expected complete two-byte carriers",
            encoded.len()
        ));
    }
    for pair in encoded.chunks_exact(2) {
        if !is_sjis_lead(pair[0]) || !is_sjis_trail(pair[1]) {
            return Err(format!(
                "ordinary text encoder returned non-CP932 carrier {:02X}{:02X}",
                pair[0], pair[1]
            ));
        }
    }
    output.extend_from_slice(&encoded);
    Ok(())
}

enum TemplateToken {
    Speaker(u8),
    Wait,
    Kana,
    Attribute(u8),
    Raw(Vec<u8>),
}

fn parse_template_token(input: &str) -> Option<(TemplateToken, usize)> {
    if !input.starts_with("[[") {
        return None;
    }
    let close = input.find("]]")?;
    let interior = &input[2..close];
    let token_length = close.checked_add(2)?;
    if interior == "WAIT" {
        return Some((TemplateToken::Wait, token_length));
    }
    if interior == "KANA" {
        return Some((TemplateToken::Kana, token_length));
    }
    if let Some(digits) = interior.strip_prefix("SPK:") {
        if digits.len() == 2 && digits.bytes().all(|byte| byte.is_ascii_digit()) {
            let index = digits.parse::<u8>().ok()?;
            if index < SPEAKER_COUNT {
                return Some((TemplateToken::Speaker(index), token_length));
            }
        }
        return None;
    }
    if let Some(digit) = interior.strip_prefix("ATTR:") {
        if digit.len() == 1 && digit.as_bytes()[0].is_ascii_digit() {
            return Some((TemplateToken::Attribute(digit.as_bytes()[0]), token_length));
        }
        return None;
    }
    if let Some(hex) = interior.strip_prefix("RAW:") {
        if hex.is_empty() || hex.len() % 2 != 0 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return None;
        }
        let mut bytes = Vec::with_capacity(hex.len() / 2);
        for pair in hex.as_bytes().chunks_exact(2) {
            let high = hex_nibble(pair[0])?;
            let low = hex_nibble(pair[1])?;
            bytes.push((high << 4) | low);
        }
        return Some((TemplateToken::Raw(bytes), token_length));
    }
    None
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn validate_replacement(kind: TextUnitKind, bytes: &[u8]) -> Result<(), String> {
    match kind {
        TextUnitKind::Message => {
            if let Some(offset) = bytes.iter().position(|byte| *byte == b'$' || *byte == 0x1A) {
                return Err(format!(
                    "body contains structural terminator byte 0x{:02X} at replacement offset 0x{offset:X}",
                    bytes[offset]
                ));
            }
        }
        TextUnitKind::Extra => {
            if let Some(offset) = bytes.iter().position(|byte| *byte == b'$' || *byte == 0x1A) {
                return Err(format!(
                    "$X body contains structural terminator byte 0x{:02X} at replacement offset 0x{offset:X}",
                    bytes[offset]
                ));
            }
            let mut run_length = 0usize;
            for byte in bytes {
                if *byte == 0x0D {
                    if !run_length.is_multiple_of(2) {
                        return Err("$X text has an incomplete DBCS pair before CR".into());
                    }
                    run_length = 0;
                } else {
                    run_length += 1;
                }
            }
            if !run_length.is_multiple_of(2) {
                return Err("$X text has an incomplete trailing DBCS pair".into());
            }
        }
        TextUnitKind::Choice => {
            if let Some(offset) = bytes.iter().position(|byte| *byte == 0) {
                return Err(format!(
                    "choice contains its NUL terminator at replacement offset 0x{offset:X}"
                ));
            }
            if !bytes.len().is_multiple_of(2) {
                return Err(
                    "choice string has an odd byte length; expected complete DBCS pairs".into(),
                );
            }
        }
    }
    Ok(())
}

fn read_be_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let Some(end) = offset.checked_add(2) else {
        return Err(file_error(offset, "u16 range overflow"));
    };
    let Some(pair) = bytes.get(offset..end) else {
        return Err(file_error(offset, "truncated big-endian u16"));
    };
    Ok(u16::from_be_bytes([pair[0], pair[1]]))
}

fn file_error(offset: usize, message: &str) -> String {
    format!("MES file offset 0x{offset:X}: {message}")
}

fn record_error(key: u16, offset: usize, message: &str) -> String {
    format!("MES record key 0x{key:04X} at file offset 0x{offset:X}: {message}")
}
