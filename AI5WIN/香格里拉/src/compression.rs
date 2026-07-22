use std::collections::BTreeSet;
use std::fmt;

const WINDOW_SIZE: usize = 0x1000;
const WINDOW_MASK: usize = WINDOW_SIZE - 1;
const INITIAL_WINDOW_POSITION: usize = 0x0fee;
const MIN_MATCH: usize = 3;
const MAX_MATCH: usize = 18;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecompressError(String);

impl DecompressError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for DecompressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DecompressError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressError(String);

impl CompressError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for CompressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CompressError {}

pub fn decompress_lzss(input: &[u8], max_output: usize) -> Result<Vec<u8>, DecompressError> {
    let mut window = [0u8; WINDOW_SIZE];
    let mut window_position = INITIAL_WINDOW_POSITION;
    let mut input_position = 0usize;
    let mut flags = 0u16;
    let mut output = Vec::new();

    while input_position < input.len() {
        flags >>= 1;
        if flags & 0x100 == 0 {
            let flag_byte = read_byte(input, &mut input_position, "flag byte")?;
            flags = u16::from(flag_byte) | 0xff00;
        }

        if flags & 1 != 0 {
            let value = read_byte(input, &mut input_position, "literal byte")?;
            push_byte(
                &mut output,
                &mut window,
                &mut window_position,
                value,
                max_output,
            )?;
        } else {
            let low = read_byte(input, &mut input_position, "back-reference low byte")?;
            let high_and_length = read_byte(
                input,
                &mut input_position,
                "back-reference high/length byte",
            )?;
            let offset = usize::from(low) | (usize::from(high_and_length & 0xf0) << 4);
            let length = usize::from(high_and_length & 0x0f) + 3;
            for index in 0..length {
                let value = window[(offset + index) & WINDOW_MASK];
                push_byte(
                    &mut output,
                    &mut window,
                    &mut window_position,
                    value,
                    max_output,
                )?;
            }
        }
    }

    Ok(output)
}

pub fn compress_lzss(input: &[u8]) -> Result<Vec<u8>, CompressError> {
    let mut window = [0u8; WINDOW_SIZE];
    let mut positions: [BTreeSet<usize>; 256] = std::array::from_fn(|_| BTreeSet::new());
    positions[0].extend(0..WINDOW_SIZE);
    let mut window_position = INITIAL_WINDOW_POSITION;
    let mut input_position = 0usize;
    let capacity = input
        .len()
        .checked_add(input.len().div_ceil(8))
        .ok_or_else(|| CompressError::new("compressed output capacity overflows usize"))?;
    let mut output = Vec::with_capacity(capacity);

    while input_position < input.len() {
        let flag_position = output.len();
        output.push(0);
        let mut flags = 0u8;

        for bit in 0..8 {
            if input_position >= input.len() {
                break;
            }
            let (match_offset, match_length) =
                find_match(input, input_position, &window, &positions, window_position);
            if match_length >= MIN_MATCH {
                output.push(match_offset as u8);
                output.push(
                    (((match_offset >> 4) & 0xf0) as u8)
                        | u8::try_from(match_length - MIN_MATCH)
                            .expect("LZSS match length fits in four bits"),
                );
                for value in &input[input_position..input_position + match_length] {
                    update_window(&mut window, &mut positions, window_position, *value);
                    window_position = (window_position + 1) & WINDOW_MASK;
                }
                input_position += match_length;
            } else {
                flags |= 1 << bit;
                let value = input[input_position];
                output.push(value);
                update_window(&mut window, &mut positions, window_position, value);
                window_position = (window_position + 1) & WINDOW_MASK;
                input_position += 1;
            }
        }
        output[flag_position] = flags;
    }

    Ok(output)
}

fn find_match(
    input: &[u8],
    input_position: usize,
    window: &[u8; WINDOW_SIZE],
    positions: &[BTreeSet<usize>; 256],
    window_position: usize,
) -> (usize, usize) {
    let max_length = MAX_MATCH.min(input.len() - input_position);
    if max_length < MIN_MATCH {
        return (0, 0);
    }

    let mut best_offset = 0usize;
    let mut best_length = 0usize;
    for &offset in &positions[usize::from(input[input_position])] {
        let mut length = 1usize;
        while length < max_length {
            let source = (offset + length) & WINDOW_MASK;
            let written_distance = (source + WINDOW_SIZE - window_position) & WINDOW_MASK;
            let value = if written_distance < length {
                input[input_position + written_distance]
            } else {
                window[source]
            };
            if value != input[input_position + length] {
                break;
            }
            length += 1;
        }
        if length > best_length {
            best_offset = offset;
            best_length = length;
            if length == max_length {
                break;
            }
        }
    }
    (best_offset, best_length)
}

fn update_window(
    window: &mut [u8; WINDOW_SIZE],
    positions: &mut [BTreeSet<usize>; 256],
    offset: usize,
    value: u8,
) {
    let previous = window[offset];
    if previous == value {
        return;
    }
    positions[usize::from(previous)].remove(&offset);
    positions[usize::from(value)].insert(offset);
    window[offset] = value;
}

fn read_byte(input: &[u8], input_position: &mut usize, field: &str) -> Result<u8, DecompressError> {
    let value = input.get(*input_position).copied().ok_or_else(|| {
        DecompressError::new(format!(
            "truncated LZSS {field} at compressed offset 0x{:X}",
            *input_position
        ))
    })?;
    *input_position += 1;
    Ok(value)
}

fn push_byte(
    output: &mut Vec<u8>,
    window: &mut [u8; WINDOW_SIZE],
    window_position: &mut usize,
    value: u8,
    max_output: usize,
) -> Result<(), DecompressError> {
    if output.len() >= max_output {
        return Err(DecompressError::new(format!(
            "decompressed data exceeds the configured limit of {max_output} bytes"
        )));
    }
    output.push(value);
    window[*window_position] = value;
    *window_position = (*window_position + 1) & WINDOW_MASK;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_literal_group() {
        let output = decompress_lzss(b"\x07ABC", 100).unwrap();
        assert_eq!(output, b"ABC");
    }

    #[test]
    fn decodes_overlapping_back_reference() {
        let output = decompress_lzss(b"\x01A\xEE\xF1", 100).unwrap();
        assert_eq!(output, b"AAAAA");
    }

    #[test]
    fn rejects_truncated_literal() {
        let error = decompress_lzss(b"\x01", 100).unwrap_err();
        assert!(error.to_string().contains("literal byte"));
    }

    #[test]
    fn rejects_truncated_back_reference() {
        let error = decompress_lzss(b"\x00\xEE", 100).unwrap_err();
        assert!(error.to_string().contains("high/length byte"));
    }

    #[test]
    fn enforces_output_limit() {
        let error = decompress_lzss(b"\x01A\xEE\xFF", 10).unwrap_err();
        assert!(error.to_string().contains("configured limit"));
    }

    #[test]
    fn compresses_and_decompresses_literals_and_repetitions() {
        for input in [
            b"".as_slice(),
            b"ABC",
            b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            b"0123456789-0123456789-0123456789",
            b"\0\0\0\0\0\0\0\0\0\0",
        ] {
            let compressed = compress_lzss(input).unwrap();
            assert_eq!(decompress_lzss(&compressed, 1000).unwrap(), input);
        }
    }

    #[test]
    fn compressor_uses_back_references() {
        let input = b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let compressed = compress_lzss(input).unwrap();
        assert!(compressed.len() < input.len());
    }
}
