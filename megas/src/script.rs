use crate::glyph::{project_units, scan_units, GlyphDictionary, UnitKind};
use crate::{hex_encode, ToolError, ToolResult};
use std::collections::HashMap;
use std::ops::Range;

const MSB_HEADER_SIZE: usize = 16;
const MSB_RECORD_SIZE: usize = 8;

#[derive(Debug, Clone)]
pub struct MsbRecord {
    pub id: u32,
    pub relative_offset: u32,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct MsbFile {
    pub header: [u8; MSB_HEADER_SIZE],
    pub records: Vec<MsbRecord>,
}

#[derive(Debug, Clone)]
pub struct TextSplit {
    pub has_name: bool,
    pub name: Option<String>,
    pub message: String,
    pub name_range: Option<Range<usize>>,
    pub message_range: Range<usize>,
    pub prefix: Vec<u8>,
    pub middle: Vec<u8>,
    pub suffix: Vec<u8>,
    pub terminator: Option<[u8; 2]>,
}

#[derive(Debug, Clone)]
pub struct ScxFile {
    pub original: Vec<u8>,
    pub fc: u32,
    pub f4: u32,
    pub f8: u32,
    pub pointers: Vec<u32>,
    pub vector: Vec<u8>,
    pub blocks: Vec<Vec<u8>>,
}

pub fn parse_msb(data: &[u8]) -> ToolResult<MsbFile> {
    if data.len() < MSB_HEADER_SIZE || &data[..4] != b"MES\0" {
        return Err(ToolError("not an MES\\0 MSB file".to_string()));
    }
    let version = read_u32(data, 4)?;
    if version != 1 {
        return Err(ToolError(format!("unsupported MSB version {version}")));
    }
    let count = read_u32(data, 8)? as usize;
    let body_base = read_u32(data, 12)? as usize;
    let expected_base = MSB_HEADER_SIZE
        .checked_add(
            count
                .checked_mul(MSB_RECORD_SIZE)
                .ok_or_else(|| ToolError("MSB record table size overflows".to_string()))?,
        )
        .ok_or_else(|| ToolError("MSB body base overflows".to_string()))?;
    if body_base != expected_base || body_base > data.len() {
        return Err(ToolError(format!(
            "invalid MSB body base 0x{body_base:X}, expected 0x{expected_base:X}"
        )));
    }

    let mut header = [0u8; MSB_HEADER_SIZE];
    header.copy_from_slice(&data[..MSB_HEADER_SIZE]);
    let mut table: Vec<MsbRecord> = Vec::with_capacity(count);
    let mut previous = 0u32;
    for index in 0..count {
        let offset = MSB_HEADER_SIZE + index * MSB_RECORD_SIZE;
        let id = read_u32(data, offset)?;
        let relative_offset = read_u32(data, offset + 4)?;
        if index > 0 && relative_offset < previous {
            return Err(ToolError(format!(
                "MSB record {index} offset 0x{relative_offset:X} moves backwards"
            )));
        }
        if relative_offset as usize > data.len() - body_base {
            return Err(ToolError(format!(
                "MSB record {index} offset 0x{relative_offset:X} is outside the body"
            )));
        }
        if index > 0 && id <= table[index - 1].id {
            return Err(ToolError(format!(
                "MSB record IDs are not strictly increasing at index {index}"
            )));
        }
        previous = relative_offset;
        table.push(MsbRecord {
            id,
            relative_offset,
            body: Vec::new(),
        });
    }

    for index in 0..count {
        let start = body_base + table[index].relative_offset as usize;
        let end = if index + 1 < count {
            body_base + table[index + 1].relative_offset as usize
        } else {
            data.len()
        };
        if end < start || end > data.len() {
            return Err(ToolError(format!(
                "MSB record {index} has invalid bounds 0x{start:X}..0x{end:X}"
            )));
        }
        table[index].body = data[start..end].to_vec();
    }
    Ok(MsbFile {
        header,
        records: table,
    })
}

pub fn rebuild_msb(file: &MsbFile) -> ToolResult<Vec<u8>> {
    let count = file.records.len();
    let body_base = MSB_HEADER_SIZE
        .checked_add(
            count
                .checked_mul(MSB_RECORD_SIZE)
                .ok_or_else(|| ToolError("MSB record table size overflows".to_string()))?,
        )
        .ok_or_else(|| ToolError("MSB body base overflows".to_string()))?;
    if body_base > u32::MAX as usize {
        return Err(ToolError("MSB body base exceeds u32".to_string()));
    }
    let mut header = file.header;
    header[..4].copy_from_slice(b"MES\0");
    header[4..8].copy_from_slice(&1u32.to_le_bytes());
    header[8..12].copy_from_slice(&(count as u32).to_le_bytes());
    header[12..16].copy_from_slice(&(body_base as u32).to_le_bytes());
    let total_body = file.records.iter().try_fold(0usize, |sum, record| {
        sum.checked_add(record.body.len())
            .ok_or_else(|| ToolError("MSB body size overflows".to_string()))
    })?;
    let mut output = Vec::with_capacity(body_base + total_body);
    output.extend_from_slice(&header);
    let mut relative = 0usize;
    for record in &file.records {
        if relative > u32::MAX as usize {
            return Err(ToolError("MSB record offset exceeds u32".to_string()));
        }
        output.extend_from_slice(&record.id.to_le_bytes());
        output.extend_from_slice(&(relative as u32).to_le_bytes());
        relative = relative
            .checked_add(record.body.len())
            .ok_or_else(|| ToolError("MSB body size overflows".to_string()))?;
    }
    for record in &file.records {
        output.extend_from_slice(&record.body);
    }
    Ok(output)
}

pub fn split_text(body: &[u8], dictionary: &GlyphDictionary) -> TextSplit {
    let units = scan_units(body);
    let terminator_start = if body.len() >= 2
        && matches!(body[body.len() - 2], 0x03 | 0x08)
        && body[body.len() - 1] == 0xFF
    {
        body.len() - 2
    } else {
        body.len()
    };
    let terminator = if terminator_start < body.len() {
        Some([body[terminator_start], body[terminator_start + 1]])
    } else {
        None
    };
    let name_separator = units
        .iter()
        .find(|unit| unit.offset < terminator_start && unit.kind == UnitKind::Byte(0x01));
    let message_separator = name_separator.and_then(|name| {
        units.iter().find(|unit| {
            unit.offset > name.offset
                && unit.offset < terminator_start
                && unit.kind == UnitKind::Byte(0x02)
        })
    });
    let has_name =
        name_separator.is_some_and(|unit| unit.offset == 0) && message_separator.is_some();
    let (name_range, message_range, prefix, middle) = if has_name {
        let name = name_separator.expect("checked above");
        let message = message_separator.expect("checked above");
        let name_start = name.offset + name.len;
        let name_end = message.offset;
        let message_start = message.offset + message.len;
        (
            Some(name_start..name_end),
            message_start..terminator_start,
            body[..name_start].to_vec(),
            body[name_end..message_start].to_vec(),
        )
    } else {
        (None, 0..terminator_start, Vec::new(), Vec::new())
    };
    let project_range = |range: &Range<usize>| {
        let selected = units
            .iter()
            .filter(|unit| unit.offset >= range.start && unit.offset < range.end)
            .copied()
            .collect::<Vec<_>>();
        project_units(&selected, dictionary)
    };
    let message_projection = project_range(&message_range);
    let name = name_range.as_ref().map(|range| project_range(range).text);
    TextSplit {
        has_name,
        name,
        message: message_projection.text,
        name_range,
        message_range,
        prefix,
        middle,
        suffix: body[terminator_start..].to_vec(),
        terminator,
    }
}

pub fn rebuild_text_body(
    original: &[u8],
    name: Option<&str>,
    message: &str,
    dictionary: &GlyphDictionary,
) -> ToolResult<Vec<u8>> {
    let split = split_text(original, dictionary);
    if split.name.as_deref() == name && split.message == message {
        return Ok(original.to_vec());
    }
    if split.has_name && name.is_none() {
        return Err(ToolError(
            "translated entry removed a required name".to_string(),
        ));
    }
    if !split.has_name && name.is_some_and(|value| !value.is_empty()) {
        return Err(ToolError(
            "translated entry added a name to a monologue without a name separator".to_string(),
        ));
    }
    let encoded_name = match name {
        Some(value) if split.has_name => dictionary.encode_text(value)?,
        _ => Vec::new(),
    };
    let encoded_message = dictionary.encode_text(message)?;
    if let Some(range) = &split.name_range {
        validate_control_sequence(&original[range.clone()], &encoded_name, "name")?;
    }
    validate_control_sequence(
        &original[split.message_range.clone()],
        &encoded_message,
        "message",
    )?;
    let mut output = Vec::with_capacity(
        split.prefix.len()
            + encoded_name.len()
            + split.middle.len()
            + encoded_message.len()
            + split.suffix.len(),
    );
    output.extend_from_slice(&split.prefix);
    output.extend_from_slice(&encoded_name);
    output.extend_from_slice(&split.middle);
    output.extend_from_slice(&encoded_message);
    output.extend_from_slice(&split.suffix);
    Ok(output)
}

fn validate_control_sequence(original: &[u8], replacement: &[u8], label: &str) -> ToolResult<()> {
    let original_controls = scan_units(original)
        .into_iter()
        .filter_map(|unit| match unit.kind {
            UnitKind::Byte(value) => Some(value),
            UnitKind::Glyph(_) => None,
        })
        .collect::<Vec<_>>();
    let replacement_controls = scan_units(replacement)
        .into_iter()
        .filter_map(|unit| match unit.kind {
            UnitKind::Byte(value) => Some(value),
            UnitKind::Glyph(_) => None,
        })
        .collect::<Vec<_>>();
    if original_controls != replacement_controls {
        return Err(ToolError(format!(
            "{label} control bytes changed: source={} replacement={}",
            hex_encode(&original_controls),
            hex_encode(&replacement_controls)
        )));
    }
    Ok(())
}

pub fn parse_scx(data: &[u8]) -> ToolResult<ScxFile> {
    if data.len() < 16 || &data[..4] != b"SC3\0" {
        return Err(ToolError("not an SC3\\0 SCX file".to_string()));
    }
    let f4 = read_u32(data, 4)?;
    let f8 = read_u32(data, 8)?;
    let fc = read_u32(data, 12)?;
    if !(16 <= fc && fc <= f4 && f4 <= f8 && (f8 as usize) <= data.len()) {
        return Err(ToolError(format!(
            "invalid SCX sections fc=0x{fc:X} f4=0x{f4:X} f8=0x{f8:X}"
        )));
    }
    if !(f8 - f4).is_multiple_of(4) {
        return Err(ToolError(
            "SCX pointer table is not u32 aligned".to_string(),
        ));
    }
    let pointer_count = ((f8 - f4) / 4) as usize;
    let mut pointers = Vec::with_capacity(pointer_count);
    for index in 0..pointer_count {
        pointers.push(read_u32(data, f4 as usize + index * 4)?);
    }
    let (vector_end, vector, blocks) = if let Some(first) = pointers.first().copied() {
        if first < f8 || first as usize > data.len() {
            return Err(ToolError(format!(
                "SCX first string pointer 0x{first:X} is outside the tail"
            )));
        }
        if pointers.windows(2).any(|pair| pair[0] > pair[1]) {
            return Err(ToolError("SCX string pointers are not sorted".to_string()));
        }
        if pointers
            .iter()
            .any(|pointer| *pointer as usize > data.len())
        {
            return Err(ToolError(
                "SCX string pointer is outside the file".to_string(),
            ));
        }
        let vector_end = first as usize;
        let vector = data[f8 as usize..vector_end].to_vec();
        let mut blocks = Vec::with_capacity(pointers.len());
        for (index, pointer) in pointers.iter().enumerate() {
            let start = *pointer as usize;
            let end = pointers
                .get(index + 1)
                .map(|value| *value as usize)
                .unwrap_or(data.len());
            if end < start {
                return Err(ToolError(format!("SCX block {index} has invalid bounds")));
            }
            blocks.push(data[start..end].to_vec());
        }
        (vector_end, vector, blocks)
    } else {
        (data.len(), data[f8 as usize..].to_vec(), Vec::new())
    };
    let _ = vector_end;
    Ok(ScxFile {
        original: data.to_vec(),
        fc,
        f4,
        f8,
        pointers,
        vector,
        blocks,
    })
}

pub fn rebuild_scx(file: &ScxFile, replacements: &HashMap<usize, Vec<u8>>) -> ToolResult<Vec<u8>> {
    if replacements.keys().any(|index| *index >= file.blocks.len()) {
        return Err(ToolError(
            "SCX replacement index is outside the pointer table".to_string(),
        ));
    }
    if file.pointers.is_empty() {
        if replacements.is_empty() {
            return Ok(file.original.clone());
        }
        return Err(ToolError(
            "cannot replace a block in an SCX file with an empty pointer table".to_string(),
        ));
    }
    let mut blocks = file.blocks.clone();
    for (index, replacement) in replacements {
        let old = blocks[*index].clone();
        let suffix_len = scx_suffix_len(&old);
        blocks[*index] = replacement.clone();
        if suffix_len > 0 {
            blocks[*index].extend_from_slice(&old[old.len() - suffix_len..]);
        }
    }
    let first = file.pointers[0] as usize;
    let mut pointers = Vec::with_capacity(blocks.len());
    let mut cursor = first;
    for block in &blocks {
        if cursor > u32::MAX as usize {
            return Err(ToolError("SCX relocated pointer exceeds u32".to_string()));
        }
        pointers.push(cursor as u32);
        cursor = cursor
            .checked_add(block.len())
            .ok_or_else(|| ToolError("SCX string tail size overflows".to_string()))?;
    }
    let mut output = file.original[..file.f8 as usize].to_vec();
    for (index, pointer) in pointers.iter().enumerate() {
        let at = file.f4 as usize + index * 4;
        output[at..at + 4].copy_from_slice(&pointer.to_le_bytes());
    }
    output.extend_from_slice(&file.vector);
    for block in blocks {
        output.extend_from_slice(&block);
    }
    Ok(output)
}

pub fn scx_content(body: &[u8], dictionary: &GlyphDictionary) -> (String, usize) {
    let suffix_len = scx_suffix_len(body);
    let end = body.len() - suffix_len;
    let projection = project_units(&scan_units(&body[..end]), dictionary);
    (projection.text, suffix_len)
}

pub fn encode_scx_content(
    original: &[u8],
    message: &str,
    dictionary: &GlyphDictionary,
) -> ToolResult<Vec<u8>> {
    let suffix_len = scx_suffix_len(original);
    let end = original.len() - suffix_len;
    let encoded = dictionary.encode_text(message)?;
    validate_control_sequence(&original[..end], &encoded, "SCX block")?;
    Ok(encoded)
}

fn scx_suffix_len(body: &[u8]) -> usize {
    usize::from(
        body.len() >= 2
            && matches!(body[body.len() - 2], 0x03 | 0x08)
            && body[body.len() - 1] == 0xFF,
    ) * 2
}

fn read_u32(data: &[u8], offset: usize) -> ToolResult<u32> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or_else(|| ToolError(format!("truncated u32 at 0x{offset:X}")))?;
    Ok(u32::from_le_bytes(bytes.try_into().expect("four bytes")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glyph::GlyphDictionary;

    #[test]
    fn msb_round_trip_and_relocation() {
        let dictionary = GlyphDictionary::built_in().unwrap();
        let mut source = b"MES\0".to_vec();
        source.extend_from_slice(&1u32.to_le_bytes());
        source.extend_from_slice(&1u32.to_le_bytes());
        source.extend_from_slice(&24u32.to_le_bytes());
        source.extend_from_slice(&0u32.to_le_bytes());
        source.extend_from_slice(&0u32.to_le_bytes());
        source.extend_from_slice(b"\x80\0\x02\x18\x03\xFF");
        let file = parse_msb(&source).unwrap();
        assert_eq!(rebuild_msb(&file).unwrap(), source);
        let replacement =
            rebuild_text_body(&file.records[0].body, None, "もも", &dictionary).unwrap();
        assert!(replacement.len() > file.records[0].body.len());
    }

    #[test]
    fn scx_empty_pointer_table_round_trip() {
        let mut source = b"SC3\0".to_vec();
        source.extend_from_slice(&16u32.to_le_bytes());
        source.extend_from_slice(&16u32.to_le_bytes());
        source.extend_from_slice(&16u32.to_le_bytes());
        source.extend_from_slice(b"opaque");
        let file = parse_scx(&source).unwrap();
        assert_eq!(rebuild_scx(&file, &HashMap::new()).unwrap(), source);
    }
}
