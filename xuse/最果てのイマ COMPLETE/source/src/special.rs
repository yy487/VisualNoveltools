use crate::encoding::TextEncoding;
use crate::hash::crc16_ccitt;
use crate::io_util::{checked_add, checked_mul, read_u16, read_u32};
use crate::ToolResult;

const VARIABLE_KEY: [u8; 16] = [
    0x8A, 0x71, 0x37, 0xF7, 0xFE, 0xD0, 0x11, 0xFA, 0x92, 0x60, 0x15, 0xBE, 0x1F, 0x4B, 0xAC, 0x6D,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelStats {
    pub blocks: u32,
    pub labels: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VariableStats {
    pub variables: u16,
    pub normal: u16,
    pub high_bit: u16,
}

pub fn decrypt_labels(bytes: &[u8]) -> Vec<u8> {
    let mut out = bytes.to_vec();
    let mut key = 27u8;
    for (index, byte) in out.iter_mut().enumerate() {
        *byte ^= key;
        key = ((index + (((key ^ 0xA1) as usize) >> 1)) & 0xFF) as u8;
    }
    out
}

pub fn verify_label_database(bytes: &[u8], context: &str) -> ToolResult<LabelStats> {
    let plain = decrypt_labels(bytes);
    if plain.len() < 6 {
        return Err(format!("{context}: label database is truncated"));
    }
    let count = read_u32(&plain, 0, context)? as usize;
    if count == 0 || count > 65535 {
        return Err(format!("{context}: implausible label block count {count}"));
    }
    let offsets_size = checked_mul(count, 4, context)?;
    let header_end = checked_add(checked_add(4, offsets_size, context)?, 2, context)?;
    if header_end > plain.len() {
        return Err(format!("{context}: label database header is truncated"));
    }
    if crc16_ccitt(&plain[..header_end]) != 0 {
        return Err(format!("{context}: label database header CRC16 mismatch"));
    }
    let mut block_offsets = Vec::with_capacity(count);
    for index in 0..count {
        block_offsets.push(read_u32(&plain, 4 + index * 4, context)? as usize);
    }
    if block_offsets.first().copied().unwrap_or(0) < header_end {
        return Err(format!("{context}: first label block overlaps the header"));
    }

    let mut labels = 0u64;
    for (block_index, &block_start) in block_offsets.iter().enumerate() {
        let block_end = block_offsets
            .get(block_index + 1)
            .copied()
            .unwrap_or(plain.len());
        if block_start >= block_end || block_end > plain.len() {
            return Err(format!(
                "{context}: invalid bounds for label block {block_index}"
            ));
        }
        let label_count = read_u32(&plain, block_start, context)? as usize;
        let table_size = checked_mul(label_count, 4, context)?;
        let block_header_end = checked_add(
            checked_add(block_start + 4, table_size, context)?,
            2,
            context,
        )?;
        if block_header_end > block_end {
            return Err(format!(
                "{context}: label block {block_index} header is truncated"
            ));
        }
        if crc16_ccitt(&plain[block_start..block_header_end]) != 0 {
            return Err(format!(
                "{context}: label block {block_index} header CRC16 mismatch"
            ));
        }
        let mut record_offsets = Vec::with_capacity(label_count);
        for label_index in 0..label_count {
            let relative = read_u32(&plain, block_start + 4 + label_index * 4, context)? as usize;
            let absolute = checked_add(block_start, relative, context)?;
            if absolute < block_header_end || absolute >= block_end {
                return Err(format!(
                    "{context}: label block {block_index} record {label_index} offset is invalid"
                ));
            }
            record_offsets.push(absolute);
        }
        for (label_index, &record_start) in record_offsets.iter().enumerate() {
            let name_len = read_u32(&plain, record_start, context)? as usize;
            if name_len == 0 {
                return Err(format!(
                    "{context}: label block {block_index} record {label_index} has an empty source name field"
                ));
            }
            let record_size = checked_add(name_len, 22, context)?;
            let record_end = checked_add(record_start, record_size, context)?;
            let expected_end = record_offsets
                .get(label_index + 1)
                .copied()
                .unwrap_or(block_end);
            if record_end != expected_end {
                return Err(format!(
                    "{context}: label block {block_index} record {label_index} size mismatch"
                ));
            }
            if plain[record_start + 4 + name_len - 1] != 0 {
                return Err(format!(
                    "{context}: label block {block_index} record {label_index} name lacks NUL"
                ));
            }
            TextEncoding::Cp932.decode(
                &plain[record_start + 4..record_start + 4 + name_len - 1],
                &format!("{context}: label block {block_index} record {label_index}"),
            )?;
            if crc16_ccitt(&plain[record_start..record_end]) != 0 {
                return Err(format!(
                    "{context}: label block {block_index} record {label_index} CRC16 mismatch"
                ));
            }
        }
        labels += label_count as u64;
    }
    Ok(LabelStats {
        blocks: count as u32,
        labels,
    })
}

pub fn is_label_database(bytes: &[u8]) -> bool {
    let plain = decrypt_labels(bytes);
    if plain.len() < 10 {
        return false;
    }
    let count = u32::from_le_bytes(plain[0..4].try_into().unwrap()) as usize;
    if count == 0 || count > 65535 {
        return false;
    }
    let Some(header_end) = count
        .checked_mul(4)
        .and_then(|size| 4usize.checked_add(size))
        .and_then(|size| size.checked_add(2))
    else {
        return false;
    };
    if header_end > plain.len() {
        return false;
    }
    let first_offset = u32::from_le_bytes(plain[4..8].try_into().unwrap()) as usize;
    first_offset >= header_end && first_offset < plain.len()
}

pub fn decrypt_variables(bytes: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .enumerate()
        .map(|(index, byte)| *byte ^ VARIABLE_KEY[index % VARIABLE_KEY.len()])
        .collect()
}

pub fn is_variable_database(bytes: &[u8]) -> bool {
    bytes.len() >= 4
        && bytes
            .iter()
            .take(4)
            .enumerate()
            .all(|(index, byte)| *byte ^ VARIABLE_KEY[index] == b"sVAR"[index])
}

pub fn verify_variable_database(bytes: &[u8], context: &str) -> ToolResult<VariableStats> {
    let plain = decrypt_variables(bytes);
    if plain.len() < 8 || &plain[..4] != b"sVAR" {
        return Err(format!("{context}: variable database magic mismatch"));
    }
    if crc16_ccitt(&plain[..8]) != 0 {
        return Err(format!(
            "{context}: variable database header CRC16 mismatch"
        ));
    }
    let count = read_u16(&plain, 4, context)?;
    let mut cursor = 8usize;
    let mut normal = 0u16;
    let mut high_bit = 0u16;
    for index in 0..count as usize {
        if plain.get(cursor..cursor + 4) != Some(b"eVAR") {
            return Err(format!("{context}: variable {index} magic mismatch"));
        }
        let raw_id = read_u16(&plain, cursor + 4, context)?;
        let name_len = read_u16(&plain, cursor + 6, context)? as usize;
        if name_len > 64 {
            return Err(format!(
                "{context}: variable {index} name length {name_len} exceeds 64"
            ));
        }
        let record_end = checked_add(cursor, checked_add(10, name_len, context)?, context)?;
        if record_end > plain.len() {
            return Err(format!("{context}: variable {index} is truncated"));
        }
        TextEncoding::Cp932.decode(
            &plain[cursor + 8..cursor + 8 + name_len],
            &format!("{context}: variable {index}"),
        )?;
        if crc16_ccitt(&plain[cursor..record_end]) != 0 {
            return Err(format!("{context}: variable {index} CRC16 mismatch"));
        }
        if raw_id & 0x8000 != 0 {
            high_bit += 1;
        } else {
            normal += 1;
        }
        cursor = record_end;
    }
    if cursor != plain.len() {
        return Err(format!(
            "{context}: variable database has {} trailing bytes",
            plain.len() - cursor
        ));
    }
    Ok(VariableStats {
        variables: count,
        normal,
        high_bit,
    })
}
