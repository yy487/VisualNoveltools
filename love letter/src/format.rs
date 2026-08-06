use crate::{Result, ToolError};
use std::collections::{BTreeMap, BTreeSet};

const MESSAGE_FUNCTION: u32 = 0x15C27;
const CHOICE_FUNCTION: u32 = 0x1870E;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextUse {
    Message,
    Choice,
}

#[derive(Debug, Clone)]
pub struct StringReference {
    pub opcode_offset: usize,
    pub consumer_offset: usize,
    pub consumer_opcode: u8,
    pub consumer_argument: u32,
}

impl StringReference {
    pub fn text_use(&self) -> Option<TextUse> {
        if self.consumer_opcode != 0x06 {
            return None;
        }
        match self.consumer_argument {
            MESSAGE_FUNCTION => Some(TextUse::Message),
            CHOICE_FUNCTION => Some(TextUse::Choice),
            _ => None,
        }
    }
}

/// One length-prefixed record in the `.o` string/argument table.
#[derive(Debug, Clone)]
pub struct Entry {
    pub index: usize,
    pub start: usize,
    pub data_offset: usize,
    pub original_len: usize,
    pub data: Vec<u8>,
    pub reference_offsets: Vec<usize>,
    pub references: Vec<StringReference>,
}

impl Entry {
    pub fn text_use(&self) -> Option<TextUse> {
        let mut result = None;
        for reference in &self.references {
            let current = reference.text_use()?;
            if result.is_some_and(|previous| previous != current) {
                return None;
            }
            result = Some(current);
        }
        result
    }

    pub fn has_text_reference(&self) -> bool {
        self.references
            .iter()
            .any(|reference| reference.text_use().is_some())
    }
}

/// Parsed `.o` structure. The instruction region is decoded according to
/// `sub_408DF0`; the suffix table and all unknown payload bytes are lossless.
#[derive(Debug, Clone)]
pub struct ParsedObj {
    pub bytes: Vec<u8>,
    pub table_offset: usize,
    pub entries: Vec<Entry>,
}

impl ParsedObj {
    pub fn entry_by_index(&self, index: usize) -> Option<&Entry> {
        self.entries.get(index)
    }

    pub fn entry_by_offset(&self, offset: usize) -> Option<&Entry> {
        self.entries
            .iter()
            .find(|entry| entry.data_offset == offset)
    }
}

#[derive(Debug)]
struct PendingReference {
    opcode_offset: usize,
    target: usize,
}

#[derive(Debug)]
struct DecodedReference {
    target: usize,
    reference: StringReference,
}

/// Parse the VM layout confirmed by `sub_408DF0`:
///
/// * opcodes 0, 1, 2, 3, 5, 6, and 7 carry a four-byte operand;
/// * opcode 8 and expression operators 0x80..=0xFF are one byte;
/// * opcode 2 points to a length-prefixed string record;
/// * opcodes 5 and 6 consume the values accumulated by the current statement.
pub fn parse_obj(bytes: &[u8]) -> Result<ParsedObj> {
    if bytes.len() < 8 {
        return Err(ToolError::Format(
            "file is shorter than the VM instruction/table layout".to_string(),
        ));
    }

    let (table_offset, decoded_refs) = decode_instruction_region(bytes)?;
    let starts = parse_record_chain(bytes, table_offset).ok_or_else(|| {
        ToolError::Format(format!(
            "record table at 0x{table_offset:X} does not reach EOF"
        ))
    })?;
    let record_starts: BTreeSet<usize> = starts.iter().copied().collect();
    let mut refs_by_target: BTreeMap<usize, Vec<StringReference>> = BTreeMap::new();
    for decoded in decoded_refs {
        if !record_starts.contains(&decoded.target) {
            return Err(ToolError::Format(format!(
                "opcode 0x02 reference at 0x{:X} targets non-record offset 0x{:X}",
                decoded.reference.opcode_offset, decoded.target
            )));
        }
        refs_by_target
            .entry(decoded.target)
            .or_default()
            .push(decoded.reference);
    }
    if let Some(unreferenced) = starts
        .iter()
        .find(|start| !refs_by_target.contains_key(start))
    {
        return Err(ToolError::Format(format!(
            "table record at 0x{unreferenced:X} has no instruction-boundary opcode 0x02 reference"
        )));
    }

    let entries = starts
        .into_iter()
        .enumerate()
        .map(|(index, start)| {
            let original_len = read_u32(bytes, start).expect("validated record length") as usize;
            let data_offset = start + 4;
            let references = refs_by_target.remove(&start).unwrap_or_default();
            let reference_offsets = references
                .iter()
                .map(|reference| reference.opcode_offset)
                .collect();
            Entry {
                index,
                start,
                data_offset,
                original_len,
                data: bytes[data_offset..data_offset + original_len].to_vec(),
                reference_offsets,
                references,
            }
        })
        .collect();

    Ok(ParsedObj {
        bytes: bytes.to_vec(),
        table_offset,
        entries,
    })
}

fn decode_instruction_region(bytes: &[u8]) -> Result<(usize, Vec<DecodedReference>)> {
    let mut cursor = 0usize;
    let mut table_offset = bytes.len();
    let mut pending = Vec::new();
    let mut references = Vec::new();

    while cursor < table_offset {
        let opcode = bytes[cursor];
        let has_operand = matches!(opcode, 0x00..=0x03 | 0x05..=0x07);
        let size = if has_operand { 5 } else { 1 };
        let end = cursor.checked_add(size).ok_or_else(|| {
            ToolError::Format(format!("instruction size overflow at 0x{cursor:X}"))
        })?;
        if end > table_offset {
            return Err(ToolError::Format(format!(
                "truncated opcode 0x{opcode:02X} at 0x{cursor:X}"
            )));
        }
        if !has_operand && opcode != 0x08 && opcode < 0x80 {
            return Err(ToolError::Format(format!(
                "invalid VM opcode 0x{opcode:02X} at instruction boundary 0x{cursor:X}"
            )));
        }

        let argument =
            has_operand.then(|| read_u32(bytes, cursor + 1).expect("validated operand bounds"));
        if opcode == 0x02 {
            let target = argument.expect("opcode 2 operand") as usize;
            if target < end || target > bytes.len().saturating_sub(4) {
                return Err(ToolError::Format(format!(
                    "opcode 0x02 at 0x{cursor:X} has invalid target 0x{target:X}"
                )));
            }
            let length = read_u32(bytes, target).expect("validated target header") as usize;
            let record_end = target
                .checked_add(4)
                .and_then(|value| value.checked_add(length))
                .ok_or_else(|| {
                    ToolError::Format(format!(
                        "string record size overflow for opcode 0x02 at 0x{cursor:X}"
                    ))
                })?;
            if record_end > bytes.len() {
                return Err(ToolError::Format(format!(
                    "opcode 0x02 at 0x{cursor:X} targets truncated record 0x{target:X}"
                )));
            }
            table_offset = table_offset.min(target);
            pending.push(PendingReference {
                opcode_offset: cursor,
                target,
            });
        }

        if matches!(opcode, 0x05 | 0x06) {
            let consumer_argument = argument.expect("consumer operand");
            references.extend(pending.drain(..).map(|pending| DecodedReference {
                target: pending.target,
                reference: StringReference {
                    opcode_offset: pending.opcode_offset,
                    consumer_offset: cursor,
                    consumer_opcode: opcode,
                    consumer_argument,
                },
            }));
        }
        cursor = end;
    }

    if table_offset == bytes.len() {
        return Err(ToolError::Format(
            "no instruction-boundary opcode 0x02 string reference found".to_string(),
        ));
    }
    if cursor != table_offset {
        return Err(ToolError::Format(format!(
            "instruction decoding stopped at 0x{cursor:X}, table starts at 0x{table_offset:X}"
        )));
    }
    if let Some(reference) = pending.first() {
        return Err(ToolError::Format(format!(
            "opcode 0x02 at 0x{:X} has no following native/call consumer",
            reference.opcode_offset
        )));
    }
    Ok((table_offset, references))
}

fn parse_record_chain(bytes: &[u8], start: usize) -> Option<Vec<usize>> {
    let mut records = Vec::new();
    let mut cursor = start;
    let mut seen = BTreeSet::new();
    while cursor < bytes.len() {
        if !seen.insert(cursor) {
            return None;
        }
        let length = read_u32(bytes, cursor)? as usize;
        let end = cursor.checked_add(4)?.checked_add(length)?;
        if end > bytes.len() {
            return None;
        }
        records.push(cursor);
        cursor = end;
    }
    (cursor == bytes.len()).then_some(records)
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let slice = bytes.get(offset..end)?;
    Some(u32::from_le_bytes(slice.try_into().ok()?))
}

pub(crate) fn write_u32(bytes: &mut [u8], offset: usize, value: usize) -> Result<()> {
    let value = u32::try_from(value)
        .map_err(|_| ToolError::Format(format!("offset 0x{value:X} does not fit u32")))?;
    let end = offset
        .checked_add(4)
        .ok_or_else(|| ToolError::Format("u32 write offset overflow".to_string()))?;
    let dst = bytes.get_mut(offset..end).ok_or_else(|| {
        ToolError::Format(format!("u32 write offset 0x{offset:X} is out of range"))
    })?;
    dst.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_file(payloads: &[&[u8]], consumers: &[u32]) -> Vec<u8> {
        assert_eq!(payloads.len(), consumers.len());
        let table_offset = payloads.len() * 10;
        let mut starts = Vec::with_capacity(payloads.len());
        let mut cursor = table_offset;
        for payload in payloads {
            starts.push(cursor);
            cursor += 4 + payload.len();
        }

        let mut bytes = Vec::with_capacity(cursor);
        for (&start, &consumer) in starts.iter().zip(consumers) {
            bytes.push(0x02);
            bytes.extend_from_slice(&(start as u32).to_le_bytes());
            bytes.push(0x06);
            bytes.extend_from_slice(&consumer.to_le_bytes());
        }
        for (payload, start) in payloads.iter().zip(starts) {
            bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            bytes.extend_from_slice(payload);
            assert_eq!(bytes.len(), start + 4 + payload.len());
        }
        bytes
    }

    #[test]
    fn parses_instruction_boundary_references_and_consumers() {
        let bytes = synthetic_file(&[b"abc", b"012345"], &[MESSAGE_FUNCTION, CHOICE_FUNCTION]);
        let parsed = parse_obj(&bytes).expect("synthetic table should parse");

        assert_eq!(parsed.table_offset, 20);
        assert_eq!(parsed.entries.len(), 2);
        assert_eq!(parsed.entries[0].data, b"abc");
        assert_eq!(parsed.entries[1].data, b"012345");
        assert_eq!(parsed.entries[0].reference_offsets, vec![0]);
        assert_eq!(parsed.entries[1].reference_offsets, vec![10]);
        assert_eq!(parsed.entries[0].text_use(), Some(TextUse::Message));
        assert_eq!(parsed.entries[1].text_use(), Some(TextUse::Choice));
        assert_eq!(parsed.entries[0].references[0].consumer_offset, 5);
    }

    #[test]
    fn rejects_invalid_opcode_and_truncated_record() {
        let mut invalid = synthetic_file(&[b"abc"], &[MESSAGE_FUNCTION]);
        invalid[0] = 0x04;
        assert!(parse_obj(&invalid).is_err());

        let mut truncated = synthetic_file(&[b"abc"], &[MESSAGE_FUNCTION]);
        truncated.truncate(truncated.len() - 1);
        assert!(parse_obj(&truncated).is_err());
    }
}
