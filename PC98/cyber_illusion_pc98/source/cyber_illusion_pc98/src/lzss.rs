//! `eLZ0` LZSS wrapper used by Cyber Illusion's TEXTxx modules.

use crate::Result;

const MAGIC: &[u8; 4] = b"eLZ0";
const HEADER_SIZE: usize = 8;
const WINDOW_SIZE: usize = 0x1000;
const WINDOW_MASK: usize = WINDOW_SIZE - 1;
const INITIAL_WRITE_POS: usize = 1;

/// PS1SYS reads a module into a `0xE000`-byte buffer.
const MAX_OUTPUT_SIZE: usize = 0xE000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decoded {
    /// The original `eLZ0` signature and little-endian output-size field.
    pub header: [u8; HEADER_SIZE],
    /// The expanded module bytes.
    pub bytes: Vec<u8>,
    /// Exclusive input offset after the terminating zero token.
    pub stream_end: usize,
}

/// Decode one complete `eLZ0` file.
///
/// The LZSS stream ends with a zero little-endian token. Cyber Illusion's
/// shipped TEXTxx files place that terminator at EOF, so trailing input is
/// rejected instead of being silently discarded.
pub fn decode_file(input: &[u8]) -> Result<Decoded> {
    if input.len() < HEADER_SIZE {
        return Err(format!(
            "eLZ0 header is truncated: got {} bytes, need {HEADER_SIZE}",
            input.len()
        ));
    }

    if input.get(..MAGIC.len()) != Some(&MAGIC[..]) {
        return Err("invalid eLZ0 signature".to_owned());
    }

    let header: [u8; HEADER_SIZE] = input[..HEADER_SIZE]
        .try_into()
        .map_err(|_| "invalid eLZ0 header length".to_owned())?;
    let declared_len = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
    if declared_len > MAX_OUTPUT_SIZE {
        return Err(format!(
            "eLZ0 output size 0x{declared_len:X} exceeds the PS1SYS module limit 0x{MAX_OUTPUT_SIZE:X}"
        ));
    }

    let mut output = Vec::new();
    output
        .try_reserve_exact(declared_len)
        .map_err(|error| format!("cannot allocate eLZ0 output buffer: {error}"))?;

    let mut window = [0u8; WINDOW_SIZE];
    let mut write_pos = INITIAL_WRITE_POS;
    let mut cursor = HEADER_SIZE;
    let stream_end = 'stream: loop {
        let flags = read_byte(input, &mut cursor, "flag byte")?;

        // AE has a fast path for eight consecutive literals. This is also
        // exactly the meaning of a flag byte whose eight bits are all set.
        if flags == 0xFF {
            ensure_output_room(output.len(), 8, declared_len, "0xFF literal group")?;
            let end = cursor
                .checked_add(8)
                .ok_or_else(|| "eLZ0 source offset overflow".to_owned())?;
            let literals = input.get(cursor..end).ok_or_else(|| {
                format!("eLZ0 0xFF literal group is truncated at input offset 0x{cursor:X}")
            })?;
            for &byte in literals {
                push_byte(byte, &mut output, &mut window, &mut write_pos);
            }
            cursor = end;
            continue;
        }

        for bit in 0..8 {
            if flags & (1 << bit) != 0 {
                let byte = read_byte(input, &mut cursor, "literal byte")?;
                ensure_output_room(output.len(), 1, declared_len, "literal byte")?;
                push_byte(byte, &mut output, &mut window, &mut write_pos);
                continue;
            }

            let token_offset = cursor;
            let lo = read_byte(input, &mut cursor, "reference token")?;
            let hi = read_byte(input, &mut cursor, "reference token")?;
            let token = u16::from_le_bytes([lo, hi]);
            if token == 0 {
                break 'stream cursor;
            }

            let source = usize::from(token >> 4);
            let length = usize::from(token & 0x000F) + 3;
            ensure_output_room(
                output.len(),
                length,
                declared_len,
                &format!("back-reference at input offset 0x{token_offset:X}"),
            )?;

            for index in 0..length {
                let byte = window[(source + index) & WINDOW_MASK];
                push_byte(byte, &mut output, &mut window, &mut write_pos);
            }
        }
    };

    if output.len() != declared_len {
        return Err(format!(
            "eLZ0 output length mismatch: header declares 0x{declared_len:X}, stream produced 0x{:X}",
            output.len()
        ));
    }
    if stream_end != input.len() {
        return Err(format!(
            "eLZ0 terminator ends at input offset 0x{stream_end:X}, but file has {} trailing bytes",
            input.len() - stream_end
        ));
    }

    Ok(Decoded {
        header,
        bytes: output,
        stream_end,
    })
}

/// Encode bytes as an `eLZ0` file containing literal groups only.
///
/// The result is decoded again before return. That runtime check protects the
/// wrapper, flag grouping, declared size, and end marker from drifting apart.
pub fn encode_literal(bytes: &[u8]) -> Result<Vec<u8>> {
    if bytes.len() > MAX_OUTPUT_SIZE {
        return Err(format!(
            "eLZ0 output size 0x{:X} exceeds the PS1SYS module limit 0x{MAX_OUTPUT_SIZE:X}",
            bytes.len()
        ));
    }

    let declared_len = u32::try_from(bytes.len())
        .map_err(|_| "eLZ0 output size does not fit its 32-bit header".to_owned())?;
    let full_groups = bytes.len() / 8;
    let remainder = bytes.len() % 8;
    let partial_group = if remainder == 0 { 0 } else { 1 };
    let terminator_group = if remainder == 0 { 1 } else { 0 };
    let flag_groups = full_groups
        .checked_add(partial_group)
        .and_then(|groups| groups.checked_add(terminator_group))
        .ok_or_else(|| "eLZ0 encoded size overflow".to_owned())?;
    let capacity = HEADER_SIZE
        .checked_add(bytes.len())
        .and_then(|size| size.checked_add(flag_groups))
        .and_then(|size| size.checked_add(2))
        .ok_or_else(|| "eLZ0 encoded size overflow".to_owned())?;

    let mut encoded = Vec::new();
    encoded
        .try_reserve_exact(capacity)
        .map_err(|error| format!("cannot allocate eLZ0 output buffer: {error}"))?;
    encoded.extend_from_slice(MAGIC);
    encoded.extend_from_slice(&declared_len.to_le_bytes());

    let mut offset = 0usize;
    while offset + 8 <= bytes.len() {
        encoded.push(0xFF);
        encoded.extend_from_slice(&bytes[offset..offset + 8]);
        offset += 8;
    }

    if offset < bytes.len() {
        let remaining = bytes.len() - offset;
        encoded.push(((1u16 << remaining) - 1) as u8);
        encoded.extend_from_slice(&bytes[offset..]);
        encoded.extend_from_slice(&[0, 0]);
    } else {
        // The final full literal group (if any) used the 0xFF fast path, so
        // the terminator starts a fresh group. This also handles empty input.
        encoded.extend_from_slice(&[0, 0, 0]);
    }

    let decoded = decode_file(&encoded)?;
    if decoded.bytes != bytes || decoded.stream_end != encoded.len() {
        return Err("eLZ0 literal encoder round-trip validation failed".to_owned());
    }

    Ok(encoded)
}

fn read_byte(input: &[u8], cursor: &mut usize, what: &str) -> Result<u8> {
    let offset = *cursor;
    let byte = input
        .get(offset)
        .copied()
        .ok_or_else(|| format!("eLZ0 {what} is truncated at input offset 0x{offset:X}"))?;
    *cursor = (*cursor)
        .checked_add(1)
        .ok_or_else(|| "eLZ0 source offset overflow".to_owned())?;
    Ok(byte)
}

fn ensure_output_room(current: usize, amount: usize, declared: usize, what: &str) -> Result<()> {
    let end = current
        .checked_add(amount)
        .ok_or_else(|| "eLZ0 output size overflow".to_owned())?;
    if end > declared {
        return Err(format!(
            "eLZ0 {what} would exceed declared output size 0x{declared:X}"
        ));
    }
    Ok(())
}

fn push_byte(
    byte: u8,
    output: &mut Vec<u8>,
    window: &mut [u8; WINDOW_SIZE],
    write_pos: &mut usize,
) {
    output.push(byte);
    window[*write_pos] = byte;
    *write_pos = (*write_pos + 1) & WINDOW_MASK;
}
