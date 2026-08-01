use anyhow::{bail, Context, Result};
use encoding_rs::SHIFT_JIS;
use sha2::{Digest, Sha256};

use crate::model::{ScriptJson, TextEntry, FORMAT_NAME};

const MESSAGE_OPCODE: u8 = 0x15;
const NOOP_PAIR: [u8; 2] = [0x03, 0x03];

#[derive(Debug, Clone)]
pub struct MessageRecord {
    pub inst_offset: usize,
    pub payload_offset: usize,
    pub payload_end: usize,
    pub page: u16,
    pub parts: Vec<String>,
    pub part_ranges: Vec<(usize, usize)>,
    pub layout: Vec<Vec<u8>>,
    pub payload_sha256: String,
}

impl MessageRecord {
    pub fn payload_size(&self) -> usize {
        self.payload_end - self.payload_offset
    }

    #[cfg(test)]
    fn layout_hex(&self) -> Vec<String> {
        self.layout.iter().map(|bytes| hex_bytes(bytes)).collect()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PatchStats {
    pub entries: usize,
    pub patched: usize,
    pub unchanged: usize,
}

pub fn parse_message_records(data: &[u8]) -> Result<Vec<MessageRecord>> {
    let mut records = Vec::new();
    let mut cursor = 0usize;

    while cursor + 3 < data.len() {
        if data[cursor] != MESSAGE_OPCODE || data[cursor + 3] != 0x01 {
            cursor += 1;
            continue;
        }

        let payload_offset = cursor + 3;
        let Some(payload_end) = read_script_string_end(data, payload_offset) else {
            cursor += 1;
            continue;
        };
        let payload = &data[payload_offset..payload_end];
        let Ok((parts, part_ranges, layout)) = split_payload(payload) else {
            cursor += 1;
            continue;
        };

        let page = u16::from_le_bytes([data[cursor + 1], data[cursor + 2]]);
        records.push(MessageRecord {
            inst_offset: cursor,
            payload_offset,
            payload_end,
            page,
            parts,
            part_ranges,
            layout,
            payload_sha256: sha256_hex(payload),
        });
        cursor = payload_end + 1;
    }

    for pair in records.windows(2) {
        let expected = pair[0].page.checked_add(1).with_context(|| {
            format!(
                "page number overflow after record at 0x{:X}",
                pair[0].inst_offset
            )
        })?;
        if pair[1].page != expected {
            bail!(
                "message page sequence break: page {} at 0x{:X}, then page {} at 0x{:X}",
                pair[0].page,
                pair[0].inst_offset,
                pair[1].page,
                pair[1].inst_offset
            );
        }
    }

    Ok(records)
}

pub fn extract_document(data: &[u8], relative_file: &str) -> Result<(ScriptJson, usize)> {
    let records = parse_message_records(data)?;
    let empty_pages = records
        .iter()
        .filter(|record| record.parts.is_empty())
        .count();
    let mut entries = Vec::new();
    for record in records.iter().filter(|record| !record.parts.is_empty()) {
        for part_index in 0..record.parts.len() {
            let index = entries.len();
            entries.push(record_part_to_entry(
                record,
                relative_file,
                index,
                part_index,
            ));
        }
    }

    Ok((
        ScriptJson {
            format: FORMAT_NAME.to_owned(),
            file: relative_file.to_owned(),
            entries,
        },
        empty_pages,
    ))
}

pub fn inject_document(
    source: &[u8],
    relative_file: &str,
    document: &ScriptJson,
) -> Result<(Vec<u8>, PatchStats)> {
    validate_document_header(relative_file, document)?;
    let records = parse_message_records(source)?;
    let expected_entries: usize = records.iter().map(|record| record.parts.len()).sum();

    if document.entries.len() != expected_entries {
        bail!(
            "{}: JSON entry count {} does not match source entry count {}",
            relative_file,
            document.entries.len(),
            expected_entries
        );
    }

    let mut output = source.to_vec();
    let mut stats = PatchStats {
        entries: document.entries.len(),
        ..PatchStats::default()
    };

    let mut entry_cursor = 0usize;
    for record in records.iter().filter(|record| !record.parts.is_empty()) {
        let mut encoded_parts = Vec::with_capacity(record.parts.len());
        let mut page_changed = false;

        for part_index in 0..record.parts.len() {
            let expected_index = entry_cursor + part_index;
            let entry = &document.entries[expected_index];
            validate_entry(relative_file, expected_index, entry, record, part_index)?;
            validate_message(relative_file, expected_index, &entry.message)?;

            if entry.message == entry.scr_msg {
                stats.unchanged += 1;
                let (start, end) = record.part_ranges[part_index];
                encoded_parts.push(
                    source[record.payload_offset + start..record.payload_offset + end].to_vec(),
                );
            } else {
                page_changed = true;
                stats.patched += 1;
                encoded_parts.push(encode_cp932(&entry.message).with_context(|| {
                    format!(
                        "{} entry {} cannot be encoded as CP932",
                        relative_file, expected_index
                    )
                })?);
            }
        }

        entry_cursor += record.parts.len();
        if !page_changed {
            continue;
        }

        let mut rebuilt = Vec::new();
        for (part_index, bytes) in encoded_parts.iter().enumerate() {
            rebuilt.extend_from_slice(&record.layout[part_index]);
            rebuilt.extend_from_slice(bytes);
        }

        let suffix = record.layout.last().expect("layout always has a suffix");
        let base_size = rebuilt.len() + suffix.len();
        let original_size = record.payload_size();
        if base_size > original_size {
            bail!(
                "{} page {}: translated sentences use {} bytes, exceeding the {}-byte in-place page slot by {} bytes",
                relative_file,
                record.page,
                base_size,
                original_size,
                base_size - original_size
            );
        }
        let padding = original_size - base_size;
        if padding % NOOP_PAIR.len() != 0 {
            bail!(
                "{} page {}: translated sentences leave an odd {}-byte gap; safe 03 03 padding requires an even gap",
                relative_file,
                record.page,
                padding
            );
        }
        for _ in 0..(padding / NOOP_PAIR.len()) {
            rebuilt.extend_from_slice(&NOOP_PAIR);
        }
        rebuilt.extend_from_slice(suffix);
        debug_assert_eq!(rebuilt.len(), original_size);

        output[record.payload_offset..record.payload_end].copy_from_slice(&rebuilt);
    }

    Ok((output, stats))
}

fn validate_document_header(relative_file: &str, document: &ScriptJson) -> Result<()> {
    if document.format != FORMAT_NAME {
        bail!(
            "{}: unsupported JSON format {:?}; expected {:?}",
            relative_file,
            document.format,
            FORMAT_NAME
        );
    }
    if document.file != relative_file {
        bail!(
            "{}: JSON _file is {:?}, expected {:?}",
            relative_file,
            document.file,
            relative_file
        );
    }
    Ok(())
}

fn validate_entry(
    relative_file: &str,
    expected_index: usize,
    entry: &TextEntry,
    record: &MessageRecord,
    part_index: usize,
) -> Result<()> {
    if entry.file != relative_file {
        bail!(
            "{} entry {}: _file is {:?}",
            relative_file,
            expected_index,
            entry.file
        );
    }
    if entry.index != expected_index {
        bail!(
            "{} entry {}: _index is {}",
            relative_file,
            expected_index,
            entry.index
        );
    }
    let (part_start, part_end) = record.part_ranges[part_index];
    let expected_offset = record.payload_offset + part_start;
    let expected_size = part_end - part_start;
    if entry.inst_offset != record.inst_offset
        || entry.offset != expected_offset
        || entry.size != expected_size
        || entry.payload_offset != record.payload_offset
        || entry.payload_size != record.payload_size()
    {
        bail!(
            "{} entry {}: source offsets or size no longer match (_inst_offset=0x{:X}, _offset=0x{:X}, _size={})",
            relative_file,
            expected_index,
            record.inst_offset,
            expected_offset,
            expected_size
        );
    }
    if entry.entry_type != "message"
        || entry.opcode != "0x15"
        || entry.encoding != "CP932"
        || entry.policy != "in_place"
    {
        bail!(
            "{} entry {}: immutable type/opcode/encoding/policy metadata was changed",
            relative_file,
            expected_index
        );
    }
    if entry.page != record.page || entry.part_index != part_index {
        bail!(
            "{} entry {}: page/part metadata does not match source page {} part {}",
            relative_file,
            expected_index,
            record.page,
            part_index
        );
    }
    if entry.scr_msg != record.parts[part_index] {
        bail!(
            "{} entry {}: scr_msg does not match the source script",
            relative_file,
            expected_index
        );
    }
    if entry.control_before != hex_bytes(&record.layout[part_index])
        || entry.control_after != hex_bytes(&record.layout[part_index + 1])
    {
        bail!(
            "{} entry {}: control metadata does not match the source",
            relative_file,
            expected_index
        );
    }
    if entry.payload_sha256 != record.payload_sha256 {
        bail!(
            "{} entry {}: _payload_sha256 does not match the source",
            relative_file,
            expected_index
        );
    }
    Ok(())
}

fn validate_message(relative_file: &str, index: usize, message: &str) -> Result<()> {
    if message.contains('\0') {
        bail!("{} entry {}: NUL is not allowed", relative_file, index);
    }
    if message.contains('\r') || message.contains('\n') {
        bail!(
            "{} entry {}: CR/LF is not allowed inside a sentence",
            relative_file,
            index
        );
    }
    Ok(())
}

fn record_part_to_entry(
    record: &MessageRecord,
    relative_file: &str,
    index: usize,
    part_index: usize,
) -> TextEntry {
    let (part_start, part_end) = record.part_ranges[part_index];
    TextEntry {
        file: relative_file.to_owned(),
        index,
        inst_offset: record.inst_offset,
        offset: record.payload_offset + part_start,
        size: part_end - part_start,
        payload_offset: record.payload_offset,
        payload_size: record.payload_size(),
        entry_type: "message".to_owned(),
        opcode: "0x15".to_owned(),
        page: record.page,
        part_index,
        encoding: "CP932".to_owned(),
        policy: "in_place".to_owned(),
        control_before: hex_bytes(&record.layout[part_index]),
        control_after: hex_bytes(&record.layout[part_index + 1]),
        payload_sha256: record.payload_sha256.clone(),
        scr_msg: record.parts[part_index].clone(),
        message: record.parts[part_index].clone(),
    }
}

fn read_script_string_end(data: &[u8], start: usize) -> Option<usize> {
    let mut cursor = start;
    while cursor < data.len() {
        let byte = data[cursor];
        if byte == 0 {
            return Some(cursor);
        }
        if is_script_pair_lead(byte) {
            if cursor + 1 >= data.len() {
                return None;
            }
            cursor += 2;
        } else {
            cursor += 1;
        }
    }
    None
}

type PayloadParts = (Vec<String>, Vec<(usize, usize)>, Vec<Vec<u8>>);

fn split_payload(payload: &[u8]) -> Result<PayloadParts> {
    if payload.first() != Some(&0x01) {
        bail!("message payload does not start with control 01");
    }

    let mut parts = Vec::new();
    let mut ranges = Vec::new();
    let mut layout = Vec::new();
    let mut pending_controls = Vec::new();
    let mut cursor = 0usize;

    while cursor < payload.len() {
        if is_control(payload[cursor]) {
            let byte = payload[cursor];
            pending_controls.push(byte);
            cursor += 1;
            if matches!(byte, 0x03 | 0x04) {
                if cursor >= payload.len() {
                    bail!("truncated two-byte control {:02X}", byte);
                }
                pending_controls.push(payload[cursor]);
                cursor += 1;
            }
            continue;
        }

        layout.push(std::mem::take(&mut pending_controls));
        let start = cursor;
        while cursor < payload.len() && !is_control(payload[cursor]) {
            let byte = payload[cursor];
            if is_cp932_lead(byte) {
                if cursor + 1 >= payload.len() {
                    bail!("truncated CP932 character at payload offset 0x{cursor:X}");
                }
                cursor += 2;
            } else {
                cursor += 1;
            }
        }
        let decoded = decode_cp932(&payload[start..cursor])
            .with_context(|| format!("invalid CP932 text at payload offset 0x{start:X}"))?;
        parts.push(decoded);
        ranges.push((start, cursor));
    }
    layout.push(pending_controls);

    if layout.len() != parts.len() + 1 {
        bail!("internal payload layout error");
    }
    Ok((parts, ranges, layout))
}

fn is_script_pair_lead(byte: u8) -> bool {
    matches!(byte, 0x03 | 0x04) || is_cp932_lead(byte)
}

fn is_cp932_lead(byte: u8) -> bool {
    (0x81..=0x9F).contains(&byte) || byte >= 0xE0
}

fn is_control(byte: u8) -> bool {
    byte < 0x20
}

fn decode_cp932(bytes: &[u8]) -> Result<String> {
    let decoded = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(bytes)
        .context("CP932 decoder rejected the byte sequence")?;
    Ok(decoded.into_owned())
}

fn encode_cp932(text: &str) -> Result<Vec<u8>> {
    let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
    if !had_errors {
        return Ok(encoded.into_owned());
    }

    let unsupported: String = text
        .chars()
        .filter(|ch| {
            let (_, _, error) = SHIFT_JIS.encode(&ch.to_string());
            error
        })
        .collect();
    let unique = unsupported.chars().fold(String::new(), |mut output, ch| {
        if !output.contains(ch) {
            output.push(ch);
        }
        output
    });
    bail!("unencodable characters: {unique:?}")
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(text: &str) -> Vec<u8> {
        let (bytes, _, errors) = SHIFT_JIS.encode(text);
        assert!(!errors);
        bytes.into_owned()
    }

    fn fixture() -> Vec<u8> {
        let mut bytes = vec![0x35, 0x99, 0x6B, 0x05, 0x15, 0x14, 0x00, 0x01, 0x02, 0x00];
        bytes.extend_from_slice(&[0x15, 0x15, 0x00, 0x01]);
        bytes.extend_from_slice(&encode("一行目"));
        bytes.extend_from_slice(&[0x05, 0x01]);
        bytes.extend_from_slice(&encode("二行目。"));
        bytes.extend_from_slice(&[0x05, 0x02, 0x00, 0xFF]);
        bytes
    }

    #[test]
    fn parses_runtime_message_records_and_skips_control_only_pages() {
        let bytes = fixture();
        let records = parse_message_records(&bytes).unwrap();
        assert_eq!(records.len(), 2);
        assert!(records[0].parts.is_empty());
        assert_eq!(records[1].page, 21);
        assert_eq!(records[1].parts, ["一行目", "二行目。"]);
        assert_eq!(records[1].layout_hex(), ["01", "05 01", "05 02"]);
    }

    #[test]
    fn unchanged_injection_is_byte_exact() {
        let bytes = fixture();
        let (document, _) = extract_document(&bytes, "ha01-1").unwrap();
        let (rebuilt, stats) = inject_document(&bytes, "ha01-1", &document).unwrap();
        assert_eq!(rebuilt, bytes);
        assert_eq!(stats.patched, 0);
        assert_eq!(stats.unchanged, 2);
    }

    #[test]
    fn shorter_even_translation_uses_runtime_noop_pairs() {
        let bytes = fixture();
        let (mut document, _) = extract_document(&bytes, "ha01-1").unwrap();
        document.entries[0].message = "一行".to_owned();
        let (rebuilt, stats) = inject_document(&bytes, "ha01-1", &document).unwrap();
        assert_eq!(stats.patched, 1);
        assert_eq!(rebuilt.len(), bytes.len());
        assert!(rebuilt.windows(2).any(|pair| pair == NOOP_PAIR));

        let (reextracted, _) = extract_document(&rebuilt, "ha01-1").unwrap();
        assert_eq!(reextracted.entries[0].scr_msg, "一行");
        assert_eq!(reextracted.entries[1].scr_msg, "二行目。");
    }

    #[test]
    fn rejects_odd_padding_gap() {
        let bytes = fixture();
        let (mut document, _) = extract_document(&bytes, "ha01-1").unwrap();
        document.entries[0].message = "A".to_owned();
        let error = inject_document(&bytes, "ha01-1", &document).unwrap_err();
        assert!(error.to_string().contains("odd"));
    }

    #[test]
    fn rejects_longer_translation() {
        let bytes = fixture();
        let (mut document, _) = extract_document(&bytes, "ha01-1").unwrap();
        document.entries[0].message = "一行目一行目".to_owned();
        let error = inject_document(&bytes, "ha01-1", &document).unwrap_err();
        assert!(error.to_string().contains("exceeding"));
    }

    #[test]
    fn rejects_unencodable_characters() {
        let bytes = fixture();
        let (mut document, _) = extract_document(&bytes, "ha01-1").unwrap();
        document.entries[0].message = "🙂".to_owned();
        let error = inject_document(&bytes, "ha01-1", &document).unwrap_err();
        assert!(format!("{error:#}").contains("unencodable characters"));
    }

    #[test]
    fn rejects_newlines_inside_message() {
        let bytes = fixture();
        let (mut document, _) = extract_document(&bytes, "ha01-1").unwrap();
        document.entries[0].message = "一行\n改行".to_owned();
        let error = inject_document(&bytes, "ha01-1", &document).unwrap_err();
        assert!(error.to_string().contains("LF is not allowed"));
    }

    #[test]
    fn duplicate_text_is_patched_by_record_index() {
        let text = encode("同じ");
        let mut bytes = Vec::new();
        for page in [20u16, 21u16] {
            bytes.push(MESSAGE_OPCODE);
            bytes.extend_from_slice(&page.to_le_bytes());
            bytes.push(0x01);
            bytes.extend_from_slice(&text);
            bytes.extend_from_slice(&[0x05, 0x02, 0x00]);
        }
        let original_records = parse_message_records(&bytes).unwrap();
        let first_payload =
            bytes[original_records[0].payload_offset..original_records[0].payload_end].to_vec();
        let (mut document, _) = extract_document(&bytes, "duplicate").unwrap();
        document.entries[1].message = "同".to_owned();

        let (rebuilt, stats) = inject_document(&bytes, "duplicate", &document).unwrap();
        assert_eq!(stats.patched, 1);
        assert_eq!(
            &rebuilt[original_records[0].payload_offset..original_records[0].payload_end],
            first_payload
        );
        let (reextracted, _) = extract_document(&rebuilt, "duplicate").unwrap();
        assert_eq!(reextracted.entries[0].scr_msg, "同じ");
        assert_eq!(reextracted.entries[1].scr_msg, "同");
    }
}
