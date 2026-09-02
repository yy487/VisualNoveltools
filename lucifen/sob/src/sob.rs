use crate::common::{
    decode_with, encode_with, read_json, write_json, EncodingChoice, Entry, Result,
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct SobFile {
    bytes: Vec<u8>,
    script_begin: usize,
    payload_len_pos: usize,
    offsets: Vec<u32>,
    offset_positions: Vec<usize>,
    offset_indexes: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepairIssue {
    pub file: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub indexes: Vec<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub context: Vec<ContextEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextEntry {
    pub index: usize,
    pub row_offset: u64,
    pub chs_offset: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chs_text: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RepairSummary {
    pub changed: usize,
    pub issues: Vec<RepairIssue>,
}

#[derive(Debug, Clone)]
struct Span {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RecordKind {
    Sentence,
    Selection,
}

impl RecordKind {
    fn label(self) -> &'static str {
        match self {
            Self::Sentence => "sentence",
            Self::Selection => "selection",
        }
    }
}

#[derive(Debug, Clone)]
struct Record {
    kind: RecordKind,
    raw_start: usize,
    text_span: Span,
    text: String,
    message: String,
    name: Option<String>,
    tag: Option<String>,
    separator: String,
    has_terminator: bool,
    selection_spans: Vec<Span>,
}

#[derive(Debug, Clone)]
struct CompositeMember {
    row_offset: u32,
    indexes: Vec<usize>,
}

#[derive(Debug, Clone)]
struct CompositeGroup {
    parent_offset: u32,
    parent_indexes: Vec<usize>,
    members: Vec<CompositeMember>,
    end_offset: u32,
}

fn u16_at(b: &[u8], p: usize) -> Option<u16> {
    b.get(p..p + 2).map(|x| u16::from_le_bytes([x[0], x[1]]))
}

fn u32_at(b: &[u8], p: usize) -> Option<u32> {
    b.get(p..p + 4)
        .map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]))
}

fn put_u32(b: &mut [u8], p: usize, v: u32) {
    b[p..p + 4].copy_from_slice(&v.to_le_bytes());
}

fn parse(data: Vec<u8>) -> Result<SobFile> {
    if data.len() < 0x18 || &data[0..3] != b"SOB" || !matches!(data[3], b'0' | b'1' | b'2') {
        return Err("not a supported SOB0/SOB1/SOB2 file or file is truncated".into());
    }
    let relocation_len = u32_at(&data, 4).ok_or("truncated SOB header")? as usize;
    let metadata_start = 8usize
        .checked_add(relocation_len)
        .ok_or("SOB relocation table length overflow")?;
    let payload_len_pos = metadata_start
        .checked_add(4)
        .ok_or("SOB metadata offset overflow")?;
    let script_begin = payload_len_pos
        .checked_add(4)
        .ok_or("SOB payload offset overflow")?;
    if metadata_start < 0x10 || script_begin > data.len() {
        return Err("invalid SOB relocation table length".into());
    }
    let payload_len = u32_at(&data, payload_len_pos).ok_or("truncated SOB metadata")? as usize;
    // Variable/configuration SOB resources have no relocation groups. Preserve
    // them byte-for-byte instead of interpreting their trailing opaque data as
    // script payload.
    if u32_at(&data, 8) == Some(0) {
        return Ok(SobFile {
            bytes: data,
            script_begin,
            payload_len_pos,
            offsets: Vec::new(),
            offset_positions: Vec::new(),
            offset_indexes: Vec::new(),
        });
    }
    if payload_len != data.len() - script_begin {
        return Err(format!(
            "SOB payload length mismatch: header {payload_len:#x}, file {:#x}",
            data.len() - script_begin
        ));
    }

    let mut groups = Vec::new();
    let mut pos = 8usize;
    while pos < metadata_start {
        let count = u32_at(&data, pos).ok_or("truncated SOB relocation group")? as usize;
        let start = pos.checked_add(4).ok_or("SOB relocation offset overflow")?;
        let end = start
            .checked_add(
                count
                    .checked_mul(8)
                    .ok_or("SOB relocation count overflow")?,
            )
            .ok_or("SOB relocation group overflow")?;
        if end > metadata_start {
            return Err("SOB relocation group exceeds metadata".into());
        }
        groups.push((pos, start, count));
        pos = end;
        if groups.len() > 16 {
            return Err("too many SOB relocation groups".into());
        }
    }
    if pos != metadata_start || groups.len() < 5 {
        return Err("SOB relocation groups are incomplete".into());
    }
    let (_, group5_start, group5_count) = groups[4];
    let mut offsets = Vec::with_capacity(group5_count);
    let mut offset_positions = Vec::with_capacity(group5_count);
    let mut offset_indexes = Vec::with_capacity(group5_count);
    for i in 0..group5_count {
        let target_pos = group5_start + i * 8 + 4;
        offsets.push(u32_at(&data, target_pos).ok_or("truncated SOB internal relocation")?);
        offset_positions.push(target_pos);
        offset_indexes.push((target_pos - 0x10) / 4);
    }
    Ok(SobFile {
        bytes: data,
        script_begin,
        payload_len_pos,
        offsets,
        offset_positions,
        offset_indexes,
    })
}

fn decode_slot(requested: EncodingChoice, raw: &[u8]) -> Option<(String, EncodingChoice)> {
    let (text, errors) = decode_with(requested, raw).ok()?;
    if errors || !valid_text(&text) {
        return None;
    }
    Some((text, requested))
}

fn valid_text(text: &str) -> bool {
    !text.is_empty()
        && !text
            .chars()
            .any(|c| c == '\0' || (c.is_control() && c != '\r' && c != '\n' && c != '\t'))
}

fn looks_like_script_text(text: &str) -> bool {
    text.chars().any(|c| {
        ('\u{3000}'..='\u{9fff}').contains(&c)
            || ('\u{3040}'..='\u{30ff}').contains(&c)
            || ('\u{ff00}'..='\u{ffef}').contains(&c)
    })
}

fn split_sentence_layout(text: &str) -> (Option<String>, Option<String>, String, String, bool) {
    let (separator, separator_len, marker) = if let Some(pos) = text.find("$&") {
        (pos, 2, "$&")
    } else if text.starts_with('#') {
        if let Some(pos) = text.find("＄＆") {
            (pos, "＄＆".len(), "＄＆")
        } else if let Some(pos) = text.find('&') {
            (pos, 1, "&")
        } else {
            (usize::MAX, 0, "")
        }
    } else {
        (usize::MAX, 0, "")
    };
    if separator == usize::MAX {
        let has_terminator = text.ends_with('$');
        let body = if has_terminator {
            text[..text.len() - 1].to_owned()
        } else {
            text.to_owned()
        };
        return (None, None, body, String::new(), has_terminator);
    }
    let prefix = &text[..separator];
    let body_with_suffix = &text[separator + separator_len..];
    let has_terminator = body_with_suffix.ends_with('$');
    let body = if has_terminator {
        body_with_suffix[..body_with_suffix.len() - 1].to_owned()
    } else {
        body_with_suffix.to_owned()
    };
    let (name, tag) = match prefix.strip_prefix('#') {
        Some(value) => match value.find('\\') {
            Some(pos) => (
                Some(value[..pos].to_owned()),
                Some(value[pos + 1..].to_owned()),
            ),
            None => (Some(value.to_owned()), None),
        },
        None if !prefix.is_empty() => (None, Some(prefix.to_owned())),
        _ => (None, None),
    };
    (name, tag, body, marker.to_owned(), has_terminator)
}

#[cfg(test)]
fn split_sentence(text: &str) -> (Option<String>, Option<String>, String, bool) {
    let (name, tag, body, _separator, has_terminator) = split_sentence_layout(text);
    (name, tag, body, has_terminator)
}

fn parse_selection(
    file: &SobFile,
    start: usize,
    requested: EncodingChoice,
) -> Option<(Record, EncodingChoice)> {
    let b = &file.bytes;
    if start + 5 > b.len() || b[start] != 0 || b[start + 2] != 0 || b[start + 3] == 0 {
        return None;
    }
    let count = u16_at(b, start + 3)? as usize;
    if count == 0 || count > 128 {
        return None;
    }
    let mut p = start + 5;
    let mut spans = Vec::with_capacity(count);
    let mut values = Vec::with_capacity(count);
    let mut actual = None;
    for _ in 0..count {
        let len = u16_at(b, p)? as usize;
        if len < 3 || p.checked_add(len)? > b.len() || b[p + len - 1] != 0 {
            return None;
        }
        let span = Span {
            start: p + 2,
            end: p + len - 1,
        };
        let (value, enc) = decode_slot(requested, &b[span.start..span.end])?;
        if !looks_like_script_text(&value) {
            return None;
        }
        actual.get_or_insert(enc);
        values.push(value);
        spans.push(span);
        p += len;
    }
    let encoding = actual?;
    let message = format!("Select:{}", values.join("|||||"));
    Some((
        Record {
            kind: RecordKind::Selection,
            raw_start: start,
            text_span: Span { start, end: p },
            text: message.clone(),
            message,
            name: None,
            tag: None,
            separator: String::new(),
            has_terminator: false,
            selection_spans: spans,
        },
        encoding,
    ))
}

fn classify_unchecked(
    file: &SobFile,
    offset: u32,
    requested: EncodingChoice,
) -> Option<(Record, EncodingChoice)> {
    let start = file.script_begin.checked_add(offset as usize)?;
    let b = &file.bytes;
    if start + 4 > b.len() || b[start] != 0 {
        return None;
    }
    if let Some(selection) = parse_selection(file, start, requested) {
        return Some(selection);
    }

    // Sentence records start with the three-byte command/length prefix. A
    // preceding NUL may be the terminator of the previous record; accepting a
    // four-byte fallback would reinterpret that terminator as a new sentence.
    let text_start = start + 3;
    if text_start >= b.len()
        || b[text_start] == 0
        || (b[text_start] < 0x20 && b[text_start] != b'#' && b[text_start] != b'$')
    {
        return None;
    }
    let text_end = b[text_start..].iter().position(|x| *x == 0)? + text_start;
    if text_end == text_start {
        return None;
    }
    let (text, actual) = decode_slot(requested, &b[text_start..text_end])?;
    let sentence = text.contains("$&") || text.ends_with('$');
    if !sentence {
        return None;
    }
    let (name, tag, message, separator, has_terminator) = split_sentence_layout(&text);
    if message.trim().is_empty() {
        return None;
    }
    Some((
        Record {
            kind: RecordKind::Sentence,
            raw_start: start,
            text_span: Span {
                start: text_start,
                end: text_end,
            },
            text,
            message,
            name,
            tag,
            separator,
            has_terminator,
            selection_spans: Vec::new(),
        },
        actual,
    ))
}

fn preceding_enclosing_record(
    file: &SobFile,
    offset: u32,
    requested: EncodingChoice,
) -> Option<(u32, Record, EncodingChoice)> {
    let previous_offset = offset.checked_sub(2)?;
    let (record, encoding) = classify_unchecked(file, previous_offset, requested)?;
    let start = file.script_begin.checked_add(offset as usize)?;
    if record.raw_start < start && record.text_span.end > start {
        Some((previous_offset, record, encoding))
    } else {
        None
    }
}

fn classify(
    file: &SobFile,
    offset: u32,
    requested: EncodingChoice,
) -> Option<(Record, EncodingChoice)> {
    classify_unchecked(file, offset, requested)
}

fn target_pos(file: &SobFile, index: usize) -> Option<usize> {
    file.offset_positions.get(index).copied()
}

fn json_index(file: &SobFile, index: usize) -> Option<usize> {
    file.offset_indexes.get(index).copied()
}

fn composite_groups(file: &SobFile, requested: EncodingChoice) -> Vec<CompositeGroup> {
    let mut indexes_by_offset = HashMap::<u32, Vec<usize>>::new();
    for (index, offset) in file.offsets.iter().copied().enumerate() {
        indexes_by_offset.entry(offset).or_default().push(index);
    }

    let mut groups = Vec::new();
    let mut seen_parents = HashSet::new();
    for (parent_index, parent_offset) in file.offsets.iter().copied().enumerate() {
        if !seen_parents.insert(parent_offset) {
            continue;
        }
        let parent_abs = match file.script_begin.checked_add(parent_offset as usize) {
            Some(value) => value,
            None => continue,
        };
        let Some(parent_end) = parent_abs.checked_add(2) else {
            continue;
        };
        if file.bytes.get(parent_abs..parent_end) != Some([0x30, 0].as_slice()) {
            continue;
        }

        let mut members = Vec::new();
        let mut cursor = match parent_offset.checked_add(2) {
            Some(value) => value,
            None => continue,
        };
        while let Some((record, _)) = classify_unchecked(file, cursor, requested) {
            if record.kind != RecordKind::Sentence {
                break;
            }
            let Some(indexes) = indexes_by_offset.get(&cursor) else {
                break;
            };
            let Some(end_abs) = record.text_span.end.checked_add(1) else {
                break;
            };
            let Some(end_rel) = end_abs.checked_sub(file.script_begin) else {
                break;
            };
            if end_rel <= cursor as usize {
                break;
            }
            members.push(CompositeMember {
                row_offset: cursor,
                indexes: indexes.clone(),
            });
            cursor = match u32::try_from(end_rel) {
                Ok(value) => value,
                Err(_) => break,
            };
        }

        // A single sentence preceded by 30 00 is not enough evidence of the
        // compound command. Real blocks have multiple table-addressed children.
        if members.len() >= 2 {
            let parent_indexes = indexes_by_offset
                .get(&parent_offset)
                .cloned()
                .unwrap_or_else(|| vec![parent_index]);
            let end_offset = (parent_abs.saturating_add(2)..file.bytes.len())
                .find(|absolute| {
                    file.bytes.get(*absolute..absolute.saturating_add(2))
                        == Some([0x30, 0].as_slice())
                })
                .and_then(|absolute| absolute.checked_sub(file.script_begin))
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or_else(|| {
                    u32::try_from(file.bytes.len() - file.script_begin).unwrap_or(u32::MAX)
                });
            groups.push(CompositeGroup {
                parent_offset,
                parent_indexes,
                members,
                end_offset,
            });
        }
    }
    groups
}

fn composite_is_contiguous(
    file: &SobFile,
    group: &CompositeGroup,
    requested: EncodingChoice,
) -> bool {
    let Some(&parent_index) = group.parent_indexes.first() else {
        return false;
    };
    let Some(&parent_offset) = file.offsets.get(parent_index) else {
        return false;
    };
    let Some(parent_abs) = file.script_begin.checked_add(parent_offset as usize) else {
        return false;
    };
    let Some(parent_end) = parent_abs.checked_add(2) else {
        return false;
    };
    if file.bytes.get(parent_abs..parent_end) != Some([0x30, 0].as_slice()) {
        return false;
    }
    let mut cursor = match parent_offset.checked_add(2) {
        Some(value) => value,
        None => return false,
    };
    for member in &group.members {
        let Some(&index) = member.indexes.first() else {
            return false;
        };
        if file.offsets.get(index).copied() != Some(cursor) {
            return false;
        }
        let Some((record, _)) = classify_unchecked(file, cursor, requested)
            .or_else(|| classify_unchecked(file, cursor, EncodingChoice::Sjis))
        else {
            return false;
        };
        let Some(end_abs) = record.text_span.end.checked_add(1) else {
            return false;
        };
        let Some(end_rel) = end_abs.checked_sub(file.script_begin) else {
            return false;
        };
        cursor = match u32::try_from(end_rel) {
            Ok(value) => value,
            Err(_) => return false,
        };
    }
    true
}

fn composite_record_bytes(
    file: &SobFile,
    offset: u32,
    encoding: EncodingChoice,
) -> Option<Vec<u8>> {
    let (record, _) = classify_unchecked(file, offset, encoding)?;
    let end = record.text_span.end.checked_add(1)?;
    Some(file.bytes.get(record.raw_start..end)?.to_vec())
}

fn rebuild_composite_bytes(
    file: &SobFile,
    group: &CompositeGroup,
    requested: EncodingChoice,
    replacements: &HashMap<u32, Vec<u8>>,
) -> Option<Vec<u8>> {
    let first = group.members.first()?;
    let parent_start = file
        .script_begin
        .checked_add(group.parent_offset as usize)?;
    let first_start = file.script_begin.checked_add(first.row_offset as usize)?;
    let mut block = file.bytes.get(parent_start..first_start)?.to_vec();
    let mut cursor = first.row_offset;
    for member in &group.members {
        if member.row_offset != cursor {
            return None;
        }
        if let Some(bytes) = replacements.get(&member.row_offset) {
            block.extend_from_slice(bytes);
        } else {
            block.extend_from_slice(&composite_record_bytes(file, member.row_offset, requested)?);
        }
        let (record, _) = classify(file, member.row_offset, requested)?;
        let end = record.text_span.end.checked_add(1)?;
        cursor = u32::try_from(end.checked_sub(file.script_begin)?).ok()?;
    }
    let suffix_start = file.script_begin.checked_add(cursor as usize)?;
    let suffix_end = file.script_begin.checked_add(group.end_offset as usize)?;
    if suffix_start > suffix_end {
        return None;
    }
    block.extend_from_slice(file.bytes.get(suffix_start..suffix_end)?);
    Some(block)
}

fn chs_composite_record_bytes(file: &SobFile, offset: u32) -> Option<Vec<u8>> {
    composite_record_bytes(file, offset, EncodingChoice::Gbk)
        .or_else(|| composite_record_bytes(file, offset, EncodingChoice::Sjis))
}

fn record_entry(
    file_name: &str,
    index: usize,
    offset: u32,
    record: &Record,
    encoding: EncodingChoice,
) -> Entry {
    Entry {
        file: file_name.into(),
        index,
        offset: offset as u64,
        entry_type: record.kind.label().into(),
        encoding: encoding.label().into(),
        name_index: None,
        name: record.name.clone(),
        scr_name: record.name.clone(),
        scr_tag: record.tag.clone(),
        scr_msg: record.message.clone(),
        message: record.message.clone(),
    }
}

pub fn extract_file(
    input: &Path,
    output: &Path,
    file_name: &str,
    requested: EncodingChoice,
    overwrite: bool,
) -> Result<usize> {
    let file = parse(fs::read(input).map_err(|e| format!("read {}: {e}", input.display()))?)?;
    // A relocated CHS record can collide with an old out-of-range table value.
    // Keep the highest table index for each offset: it is the canonical script
    // reference, while stale low indices otherwise make the record appear at
    // the beginning of the extracted list.
    let mut canonical = HashMap::<u32, usize>::new();
    for (index, offset) in file.offsets.iter().enumerate() {
        canonical
            .entry(*offset)
            .and_modify(|current| *current = (*current).max(index))
            .or_insert(index);
    }
    let mut entries = Vec::new();
    for (target_index, offset) in file.offsets.iter().enumerate() {
        if canonical.get(offset) != Some(&target_index) {
            continue;
        }
        let index = file.offset_indexes[target_index];
        if let Some((record, actual)) = classify(&file, *offset, requested) {
            entries.push(record_entry(file_name, index, *offset, &record, actual));
        }
    }
    write_json(output, &entries, overwrite)?;
    Ok(entries.len())
}

fn record_key(record: &Record) -> (RecordKind, String) {
    (record.kind, record.text.clone())
}

fn issue_for_record(
    file_name: &str,
    index: usize,
    offset: u32,
    reason: impl Into<String>,
    source: Option<&Record>,
    message: Option<&Record>,
    context: Vec<ContextEntry>,
) -> RepairIssue {
    RepairIssue {
        file: file_name.to_owned(),
        indexes: vec![index],
        offset: Some(offset as u64),
        reason: reason.into(),
        source: source.map(|record| record.text.clone()),
        message: message.map(|record| record.message.clone()),
        context,
    }
}

fn context_entry(row: &SobFile, chs: &SobFile, index: usize) -> Option<ContextEntry> {
    let row_offset = row.offsets[index];
    let chs_offset = chs.offsets[index];
    let row_text = classify(row, row_offset, EncodingChoice::Sjis).map(|(record, _)| record.text);
    let chs_text = if chs_offset != row_offset {
        classify(chs, chs_offset, EncodingChoice::Gbk)
            .or_else(|| classify(chs, chs_offset, EncodingChoice::Sjis))
            .map(|(record, _)| record.text)
    } else {
        classify(row, row_offset, EncodingChoice::Sjis)
            .or_else(|| classify(chs, chs_offset, EncodingChoice::Gbk))
            .map(|(record, _)| record.text)
    };
    if row_text.is_none() && chs_text.is_none() {
        return None;
    }
    Some(ContextEntry {
        index: json_index(row, index).unwrap_or(index),
        row_offset: row_offset as u64,
        chs_offset: chs_offset as u64,
        row_text,
        chs_text,
    })
}

fn context_for_index(row: &SobFile, chs: &SobFile, center: usize) -> Vec<ContextEntry> {
    if center >= row.offsets.len() {
        return Vec::new();
    }
    let center_row_offset = row.offsets[center];
    let center_chs_offset = chs.offsets[center];
    let mut context = Vec::new();
    if let Some(entry) = context_entry(row, chs, center) {
        context.push(entry);
    }
    let mut before = Vec::new();
    let mut index = center;
    while index > 0 && before.len() < 2 {
        index -= 1;
        if row.offsets[index] == center_row_offset && chs.offsets[index] == center_chs_offset {
            continue;
        }
        if let Some(entry) = context_entry(row, chs, index) {
            before.push(entry);
        }
    }
    before.reverse();
    let mut after = Vec::new();
    let mut index = center + 1;
    while index < row.offsets.len() && after.len() < 2 {
        if row.offsets[index] == center_row_offset && chs.offsets[index] == center_chs_offset {
            index += 1;
            continue;
        }
        if let Some(entry) = context_entry(row, chs, index) {
            after.push(entry);
        }
        index += 1;
    }
    before.extend(after);
    context.extend(before);
    context
}

fn rebuild_composite_groups(
    row: &SobFile,
    chs: &SobFile,
    out: &mut Vec<u8>,
    file_name: &str,
    summary: &mut RepairSummary,
    handled: &mut HashSet<usize>,
) {
    for group in composite_groups(row, EncodingChoice::Sjis) {
        let all_indexes = group
            .parent_indexes
            .iter()
            .copied()
            .chain(
                group
                    .members
                    .iter()
                    .flat_map(|member| member.indexes.iter().copied()),
            )
            .collect::<Vec<_>>();

        // Do not append a duplicate block when a previous repair already put
        // the parent and all children back into their original contiguous form.
        if composite_is_contiguous(chs, &group, EncodingChoice::Gbk) {
            handled.extend(all_indexes);
            continue;
        }

        let Some(first_member) = group.members.first() else {
            continue;
        };
        let Some(parent_start) = row.script_begin.checked_add(group.parent_offset as usize) else {
            continue;
        };
        let Some(first_start) = row
            .script_begin
            .checked_add(first_member.row_offset as usize)
        else {
            continue;
        };
        let Some(prefix) = row.bytes.get(parent_start..first_start) else {
            continue;
        };

        let mut block = prefix.to_vec();
        let mut failed = None;
        for member in &group.members {
            let Some(&index) = member.indexes.first() else {
                failed = Some("composite child has no table index");
                break;
            };
            let Some(&chs_offset) = chs.offsets.get(index) else {
                failed = Some("composite child table index is out of range");
                break;
            };
            let Some(bytes) = chs_composite_record_bytes(chs, chs_offset) else {
                failed = Some("translated composite child is not a valid sentence");
                break;
            };
            block.extend_from_slice(&bytes);
        }
        if let Some(reason) = failed {
            let parent_index = group.parent_indexes.first().copied().unwrap_or_default();
            summary.issues.push(issue_for_record(
                file_name,
                json_index(row, parent_index).unwrap_or(parent_index),
                group.parent_offset,
                reason,
                None,
                None,
                context_for_index(row, chs, parent_index),
            ));
            continue;
        }
        let suffix_start = group
            .members
            .last()
            .and_then(|member| classify(row, member.row_offset, EncodingChoice::Sjis))
            .and_then(|(record, _)| record.text_span.end.checked_add(1))
            .and_then(|absolute| absolute.checked_sub(row.script_begin));
        let suffix_end = usize::try_from(group.end_offset).ok();
        if let (Some(start), Some(end)) = (suffix_start, suffix_end) {
            if start <= end {
                let abs_start = chs.script_begin.saturating_add(start);
                let abs_end = chs.script_begin.saturating_add(end);
                if let Some(suffix) = chs.bytes.get(abs_start..abs_end) {
                    block.extend_from_slice(suffix);
                }
            }
        }

        let Some(new_offset) = out
            .len()
            .checked_sub(chs.script_begin)
            .and_then(|value| u32::try_from(value).ok())
        else {
            let parent_index = group.parent_indexes.first().copied().unwrap_or_default();
            summary.issues.push(issue_for_record(
                file_name,
                json_index(row, parent_index).unwrap_or(parent_index),
                group.parent_offset,
                "rebuilt composite block exceeds 4 GiB",
                None,
                None,
                context_for_index(row, chs, parent_index),
            ));
            continue;
        };
        out.extend_from_slice(&block);

        for index in &group.parent_indexes {
            if *index >= chs.offsets.len() {
                continue;
            }
            if let Some(position) = target_pos(chs, *index) {
                put_u32(out, position, new_offset);
            }
            summary.changed += 1;
            handled.insert(*index);
        }
        let mut relative = prefix.len();
        for member in &group.members {
            let child_offset = match new_offset.checked_add(relative as u32) {
                Some(value) => value,
                None => break,
            };
            for index in &member.indexes {
                if *index >= chs.offsets.len() {
                    continue;
                }
                if let Some(position) = target_pos(chs, *index) {
                    put_u32(out, position, child_offset);
                }
                summary.changed += 1;
                handled.insert(*index);
            }
            let Some(&index) = member.indexes.first() else {
                break;
            };
            let Some(&chs_offset) = chs.offsets.get(index) else {
                break;
            };
            let Some(bytes) = chs_composite_record_bytes(chs, chs_offset) else {
                break;
            };
            relative = match relative.checked_add(bytes.len()) {
                Some(value) => value,
                None => break,
            };
        }
    }
}

/// Repair stale CHS table entries using the corresponding ROW table.
///
/// A changed ROW/CHS index is treated as the authoritative relocation for its
/// source record. Unchanged indices with the same source record are moved to
/// that translated offset. Entries whose ROW offset is outside the ROW payload
/// are retained and reported because no source-side identity is available.
pub fn repair_file(
    chs_input: &Path,
    row_input: &Path,
    output: &Path,
    file_name: &str,
    overwrite: bool,
) -> Result<RepairSummary> {
    let chs =
        parse(fs::read(chs_input).map_err(|e| format!("read {}: {e}", chs_input.display()))?)?;
    let row =
        parse(fs::read(row_input).map_err(|e| format!("read {}: {e}", row_input.display()))?)?;
    if chs.script_begin != row.script_begin || chs.offsets.len() != row.offsets.len() {
        return Err(format!(
            "{}: ROW/CHS offset tables differ (ROW {} entries at 0x{:x}, CHS {} entries at 0x{:x})",
            file_name,
            row.offsets.len(),
            row.script_begin,
            chs.offsets.len(),
            chs.script_begin
        ));
    }

    let mut targets = HashMap::<(RecordKind, String), Vec<u32>>::new();
    for index in 0..row.offsets.len() {
        let row_offset = row.offsets[index];
        let chs_offset = chs.offsets[index];
        if row_offset == chs_offset {
            continue;
        }
        let Some((source, _)) = classify(&row, row_offset, EncodingChoice::Sjis) else {
            continue;
        };
        let Some((translated, _)) = classify(&chs, chs_offset, EncodingChoice::Gbk) else {
            continue;
        };
        if source.kind == translated.kind {
            targets
                .entry(record_key(&source))
                .or_default()
                .push(chs_offset);
        }
    }
    for values in targets.values_mut() {
        values.sort_unstable();
        values.dedup();
    }
    let translated_offsets: HashSet<u32> = targets
        .values()
        .flat_map(|values| values.iter().copied())
        .collect();

    let mut out = chs.bytes.clone();
    let mut summary = RepairSummary::default();
    let mut composite_indexes = HashSet::new();
    rebuild_composite_groups(
        &row,
        &chs,
        &mut out,
        file_name,
        &mut summary,
        &mut composite_indexes,
    );
    for index in 0..row.offsets.len() {
        if composite_indexes.contains(&index) {
            continue;
        }
        let row_offset = row.offsets[index];
        let chs_offset = chs.offsets[index];
        if row_offset != chs_offset {
            continue;
        }
        let source = classify(&row, row_offset, EncodingChoice::Sjis).map(|(record, _)| record);
        let Some(source) = source else {
            let row_abs = row.script_begin.saturating_add(row_offset as usize);
            if row_abs >= row.bytes.len() {
                if classify_unchecked(&chs, chs_offset, EncodingChoice::Gbk).is_some() {
                    if let Some((target, _, _)) =
                        preceding_enclosing_record(&chs, chs_offset, EncodingChoice::Gbk)
                    {
                        if let Some(position) = target_pos(&chs, index) {
                            put_u32(&mut out, position, target);
                        }
                        summary.changed += 1;
                        continue;
                    }
                }
                let translated =
                    classify(&chs, chs_offset, EncodingChoice::Gbk).map(|(record, _)| record);
                if translated.is_some() && !translated_offsets.contains(&chs_offset) {
                    summary.issues.push(issue_for_record(
                        file_name,
                        json_index(&row, index).unwrap_or(index),
                        chs_offset,
                        "ROW offset is outside the ROW payload; no source record to map",
                        None,
                        translated.as_ref(),
                        context_for_index(&row, &chs, index),
                    ));
                }
            }
            continue;
        };
        let key = record_key(&source);
        match targets.get(&key) {
            Some(values) if values.len() == 1 => {
                let target = values[0];
                if target != chs_offset {
                    if let Some(position) = target_pos(&chs, index) {
                        put_u32(&mut out, position, target);
                    }
                    summary.changed += 1;
                }
            }
            Some(values) if values.len() > 1 => {
                let translated =
                    classify(&chs, chs_offset, EncodingChoice::Gbk).map(|(record, _)| record);
                summary.issues.push(issue_for_record(
                    file_name,
                    json_index(&row, index).unwrap_or(index),
                    chs_offset,
                    format!("source record has multiple CHS relocation targets: {values:?}"),
                    Some(&source),
                    translated.as_ref(),
                    context_for_index(&row, &chs, index),
                ));
            }
            _ => {}
        }
    }

    if !chs.offsets.is_empty() {
        let payload_len = out
            .len()
            .checked_sub(chs.script_begin)
            .ok_or("SOB output payload underflow")?;
        let payload_len =
            u32::try_from(payload_len).map_err(|_| "SOB payload exceeds 4 GiB".to_string())?;
        put_u32(&mut out, chs.payload_len_pos, payload_len);
    }

    if output.exists() && !overwrite {
        return Err(format!(
            "output exists: {} (use --overwrite)",
            output.display()
        ));
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(output, out).map_err(|e| format!("write {}: {e}", output.display()))?;
    Ok(summary)
}

fn build_sentence(
    file: &SobFile,
    record: &Record,
    entry: &Entry,
    encoding: EncodingChoice,
) -> Result<Vec<u8>> {
    if entry.message.contains('\0') {
        return Err("NUL is not allowed in SOB message".into());
    }
    let mut text = if record.name.is_some() {
        let name = entry
            .name
            .as_deref()
            .ok_or("named SOB sentence requires name")?;
        if name.contains('\0') || name.contains('\\') || name.contains("$&") {
            return Err("SOB name contains a structural delimiter or NUL".into());
        }
        let mut value = String::from("#");
        value.push_str(name);
        if let Some(tag) = entry.scr_tag.as_deref() {
            value.push('\\');
            value.push_str(tag);
        }
        value.push_str(if record.separator.is_empty() {
            "$&"
        } else {
            &record.separator
        });
        value
    } else if let Some(tag) = entry.scr_tag.as_deref() {
        let mut value = String::new();
        value.push_str(tag);
        value.push_str(if record.separator.is_empty() {
            "$&"
        } else {
            &record.separator
        });
        value
    } else if record.text.starts_with("$&") {
        if record.separator.is_empty() {
            "$&".to_owned()
        } else {
            record.separator.clone()
        }
    } else {
        String::new()
    };
    text.push_str(&entry.message);
    if record.has_terminator {
        text.push('$');
    }
    let text_start = record.text_span.start;
    let mut out = file.bytes[record.raw_start..text_start].to_vec();
    out.extend(encode_with(encoding, &text)?);
    out.push(0);
    Ok(out)
}

fn build_selection(
    file: &SobFile,
    record: &Record,
    entry: &Entry,
    encoding: EncodingChoice,
) -> Result<Vec<u8>> {
    let parts: Vec<&str> = entry
        .message
        .strip_prefix("Select:")
        .ok_or("selection message must start with Select:")?
        .split("|||||")
        .collect();
    if parts.len() != record.selection_spans.len() {
        return Err("selection option count cannot change".into());
    }
    let count =
        u16_at(&file.bytes, record.text_span.start + 3).ok_or("truncated selection header")?;
    if count as usize != parts.len() {
        return Err("selection option count cannot change".into());
    }
    let mut out = file.bytes[record.raw_start..record.raw_start + 5].to_vec();
    for part in parts {
        if part.contains('\0') {
            return Err("NUL is not allowed in selection text".into());
        }
        let bytes = encode_with(encoding, part)?;
        let len = bytes
            .len()
            .checked_add(3)
            .ok_or("selection length overflow")?;
        if len > u16::MAX as usize {
            return Err("selection text is too long".into());
        }
        out.extend_from_slice(&(len as u16).to_le_bytes());
        out.extend(bytes);
        out.push(0);
    }
    Ok(out)
}

fn build_record(
    file: &SobFile,
    record: &Record,
    entry: &Entry,
    encoding: EncodingChoice,
) -> Result<Vec<u8>> {
    match record.kind {
        RecordKind::Sentence => build_sentence(file, record, entry, encoding),
        RecordKind::Selection => build_selection(file, record, entry, encoding),
    }
}

pub fn inject_file(
    input: &Path,
    translation: &Path,
    output: &Path,
    requested: EncodingChoice,
    overwrite: bool,
) -> Result<usize> {
    let file = parse(fs::read(input).map_err(|e| format!("read {}: {e}", input.display()))?)?;
    let entries = read_json(translation)?;
    let mut by_offset = HashMap::<u32, &Entry>::new();
    let mut pending = Vec::<(u32, &Entry, Record, Vec<u8>)>::new();
    for entry in &entries {
        let offset =
            u32::try_from(entry.offset).map_err(|_| "_offset does not fit u32".to_string())?;
        if by_offset.insert(offset, entry).is_some() {
            return Err(format!("duplicate translation offset 0x{offset:x}"));
        }
        let (record, _) = classify(&file, offset, requested)
            .ok_or_else(|| format!("translation offset 0x{offset:x} is not a recognized record"))?;
        if record.kind.label() != entry.entry_type || record.message != entry.scr_msg {
            return Err(format!("SOB source mismatch at offset 0x{offset:x}"));
        }
        if record.name != entry.scr_name {
            return Err(format!("SOB scr_name mismatch at offset 0x{offset:x}"));
        }
        if record.tag != entry.scr_tag {
            return Err(format!("SOB scr_tag mismatch at offset 0x{offset:x}"));
        }
        if entry.message == entry.scr_msg && entry.name == entry.scr_name {
            continue;
        }
        let encoding = if entry.encoding.eq_ignore_ascii_case("gbk") {
            EncodingChoice::Gbk
        } else if entry.encoding.eq_ignore_ascii_case("sjis") {
            EncodingChoice::Sjis
        } else {
            return Err("translation _encoding must be sjis or gbk".into());
        };
        let replacement = build_record(&file, &record, entry, encoding)?;
        pending.push((offset, entry, record, replacement));
    }

    let mut out = file.bytes.clone();
    let mut changed = 0usize;
    pending.sort_by_key(|(offset, _, _, _)| *offset);
    let mut relocated = HashMap::<u32, u32>::new();
    let mut replacement_map = HashMap::<u32, Vec<u8>>::new();
    for (offset, _entry, _record, replacement) in &pending {
        let new_offset = out
            .len()
            .checked_sub(file.script_begin)
            .ok_or("SOB output offset underflow")?;
        let new_offset =
            u32::try_from(new_offset).map_err(|_| "SOB output exceeds 4 GiB".to_string())?;
        out.extend_from_slice(replacement);
        relocated.insert(*offset, new_offset);
        replacement_map.insert(*offset, replacement.clone());
        changed += 1;
    }

    for group in composite_groups(&file, requested) {
        if !group
            .members
            .iter()
            .any(|member| replacement_map.contains_key(&member.row_offset))
        {
            continue;
        }
        let Some(block) = rebuild_composite_bytes(&file, &group, requested, &replacement_map)
        else {
            return Err(format!(
                "failed to rebuild SOB 30 00 block at offset 0x{:x}",
                group.parent_offset
            ));
        };
        let new_offset = u32::try_from(
            out.len()
                .checked_sub(file.script_begin)
                .ok_or("SOB output offset underflow")?,
        )
        .map_err(|_| "SOB output exceeds 4 GiB".to_string())?;
        out.extend_from_slice(&block);
        relocated.insert(group.parent_offset, new_offset);
    }

    for (index, old) in file.offsets.iter().enumerate() {
        if let Some(new_offset) = relocated.get(old) {
            if let Some(position) = target_pos(&file, index) {
                put_u32(&mut out, position, *new_offset);
            }
        }
    }
    if !file.offsets.is_empty() {
        let new_payload_len = out
            .len()
            .checked_sub(file.script_begin)
            .ok_or("SOB output payload underflow")?;
        let new_payload_len =
            u32::try_from(new_payload_len).map_err(|_| "SOB payload exceeds 4 GiB".to_string())?;
        put_u32(&mut out, file.payload_len_pos, new_payload_len);
    }
    if output.exists() && !overwrite {
        return Err(format!(
            "output exists: {} (use --overwrite)",
            output.display()
        ));
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(output, out).map_err(|e| format!("write {}: {e}", output.display()))?;
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::{
        classify, rebuild_composite_groups, split_sentence, split_sentence_layout, RepairSummary,
        SobFile,
    };
    use crate::common::EncodingChoice;
    use std::collections::HashSet;

    fn composite_fixture(child_one: &[u8], child_two: &[u8], offsets: [u32; 3]) -> SobFile {
        let script_begin = 28;
        let mut bytes = vec![0u8; script_begin];
        bytes[..4].copy_from_slice(b"SOB0");
        bytes[4..8].copy_from_slice(&12u32.to_le_bytes());
        for (index, offset) in offsets.iter().copied().enumerate() {
            bytes[16 + index * 4..20 + index * 4].copy_from_slice(&offset.to_le_bytes());
        }
        bytes.extend_from_slice(&[0x30, 0]);
        bytes.extend_from_slice(child_one);
        bytes.extend_from_slice(child_two);
        SobFile {
            bytes,
            script_begin,
            payload_len_pos: 24,
            offsets: offsets.to_vec(),
            offset_positions: vec![16, 20, 24],
            offset_indexes: offsets.into_iter().enumerate().map(|(i, _)| i).collect(),
        }
    }

    #[test]
    fn splits_named_sentence_without_structural_delimiters() {
        let (name, tag, message, terminated) = split_sentence("#鹰志\\TAK1016$&正文$");
        assert_eq!(name.as_deref(), Some("鹰志"));
        assert_eq!(tag.as_deref(), Some("TAK1016"));
        assert_eq!(message, "正文");
        assert!(terminated);
    }

    #[test]
    fn splits_speakerless_sentence() {
        let (name, tag, message, terminated) = split_sentence("（旁白）$");
        assert!(name.is_none());
        assert!(tag.is_none());
        assert_eq!(message, "（旁白）");
        assert!(terminated);
    }

    #[test]
    fn preserves_empty_name_and_non_tak_tag() {
        let (name, tag, message, terminated) = split_sentence("#\\SIN2781$&正文$");
        assert_eq!(name.as_deref(), Some(""));
        assert_eq!(tag.as_deref(), Some("SIN2781"));
        assert_eq!(message, "正文");
        assert!(terminated);
    }

    #[test]
    fn preserves_non_speaker_prefix_as_tag() {
        let (name, tag, message, terminated) = split_sentence("DJC1152$&正文$");
        assert!(name.is_none());
        assert_eq!(tag.as_deref(), Some("DJC1152"));
        assert_eq!(message, "正文");
        assert!(terminated);
    }

    #[test]
    fn splits_chs_ascii_ampersand_layout() {
        let (name, tag, message, separator, terminated) =
            split_sentence_layout("#香烟店的老婆婆&「…………？」$");
        assert_eq!(name.as_deref(), Some("香烟店的老婆婆"));
        assert!(tag.is_none());
        assert_eq!(message, "「…………？」");
        assert_eq!(separator, "&");
        assert!(terminated);
    }

    #[test]
    fn splits_chs_fullwidth_delimiter_after_ampersand_name() {
        let (name, tag, message, separator, terminated) =
            split_sentence_layout("#鹫介&英里子\\Ｘ０１０００１＄＆「正经的！」$");
        assert_eq!(name.as_deref(), Some("鹫介&英里子"));
        assert_eq!(tag.as_deref(), Some("Ｘ０１０００１"));
        assert_eq!(message, "「正经的！」");
        assert_eq!(separator, "＄＆");
        assert!(terminated);
    }

    #[test]
    fn rejects_previous_record_terminator_as_sentence() {
        let mut bytes = vec![0u8, 0, 0x7c, 0];
        bytes.extend_from_slice(b"#N$&M$");
        bytes.push(0);
        let file = SobFile {
            bytes,
            script_begin: 0,
            payload_len_pos: 0,
            offsets: Vec::new(),
            offset_positions: Vec::new(),
            offset_indexes: Vec::new(),
        };
        assert!(classify(&file, 0, EncodingChoice::Sjis).is_none());
    }

    #[test]
    fn detects_sentence_start_shifted_into_an_existing_record() {
        let bytes = vec![0, 8, 0, b'X', b'X', b'b', b'o', b'd', b'y', b'$', 0];
        let file = SobFile {
            bytes,
            script_begin: 0,
            payload_len_pos: 0,
            offsets: Vec::new(),
            offset_positions: Vec::new(),
            offset_indexes: Vec::new(),
        };
        assert!(classify(&file, 0, EncodingChoice::Sjis).is_some());
        assert!(classify(&file, 2, EncodingChoice::Sjis).is_some());
        assert_eq!(
            super::preceding_enclosing_record(&file, 2, EncodingChoice::Sjis).map(|x| x.0),
            Some(0)
        );
    }

    #[test]
    fn rebuilds_30_wrapper_and_children_in_row_order() {
        let row_child_one = [0, 1, 0, b'X', b'$', 0];
        let row_child_two = [0, 2, 0, b'Y', b'$', 0];
        let row = composite_fixture(&row_child_one, &row_child_two, [0, 2, 8]);

        // CHS children are translated and appended in reverse order, leaving
        // the original 30 00 wrapper disconnected from the table entries.
        let script_begin = 28;
        let mut chs_bytes = vec![0u8; script_begin + 32];
        chs_bytes[..4].copy_from_slice(b"SOB0");
        chs_bytes[4..8].copy_from_slice(&12u32.to_le_bytes());
        let chs_offsets = [0u32, 26, 20];
        for (index, offset) in chs_offsets.iter().copied().enumerate() {
            chs_bytes[16 + index * 4..20 + index * 4].copy_from_slice(&offset.to_le_bytes());
        }
        chs_bytes[script_begin..script_begin + 2].copy_from_slice(&[0x30, 0]);
        chs_bytes[script_begin + 20..script_begin + 26].copy_from_slice(&[0, 2, 0, b'B', b'$', 0]);
        chs_bytes[script_begin + 26..script_begin + 32].copy_from_slice(&[0, 1, 0, b'A', b'$', 0]);
        let chs = SobFile {
            bytes: chs_bytes,
            script_begin,
            payload_len_pos: 24,
            offsets: chs_offsets.to_vec(),
            offset_positions: vec![16, 20, 24],
            offset_indexes: chs_offsets
                .into_iter()
                .enumerate()
                .map(|(i, _)| i)
                .collect(),
        };

        let mut out = chs.bytes.clone();
        let mut summary = RepairSummary::default();
        let mut handled = HashSet::new();
        rebuild_composite_groups(
            &row,
            &chs,
            &mut out,
            "fixture.sob",
            &mut summary,
            &mut handled,
        );

        assert_eq!(summary.changed, 3);
        assert_eq!(handled, HashSet::from([0usize, 1, 2]));
        let new_parent = u32::from_le_bytes(out[16..20].try_into().unwrap());
        let new_child_one = u32::from_le_bytes(out[20..24].try_into().unwrap());
        let new_child_two = u32::from_le_bytes(out[24..28].try_into().unwrap());
        assert_eq!((new_parent, new_child_one, new_child_two), (32, 34, 40));
        assert_eq!(&out[script_begin + 32..script_begin + 34], &[0x30, 0]);
        assert_eq!(
            &out[script_begin + 34..script_begin + 40],
            &[0, 1, 0, b'A', b'$', 0]
        );
        assert_eq!(
            &out[script_begin + 40..script_begin + 46],
            &[0, 2, 0, b'B', b'$', 0]
        );
    }
}
