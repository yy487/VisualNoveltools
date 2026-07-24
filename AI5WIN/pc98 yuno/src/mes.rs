use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

pub const FRAME_SIZE: usize = 0x1000;
const FRAME_MASK: usize = FRAME_SIZE - 1;
const FRAME_INIT: usize = 1;
const MAX_MATCH: usize = 17;
const MIN_MATCH: usize = 2;
const MAX_OUTPUT_SIZE: usize = 0xFFFF;

#[derive(Debug, Error)]
pub enum MesError {
    #[error("invalid PC-98 MES stream: {0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, MesError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeStats {
    pub output_size: usize,
    pub consumed_bits: usize,
    pub consumed_bytes: usize,
    pub trailing_bytes: usize,
    pub padding_bits: u8,
    pub padding_value: u8,
    pub entry_offset: u16,
    pub literal_tokens: usize,
    pub match_tokens: usize,
    pub initial_zero_bytes: usize,
    pub sha256: String,
}

struct BitReader<'a> {
    data: &'a [u8],
    bit_pos: usize,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, bit_pos: 0 }
    }

    fn read_bits(&mut self, count: u8, field: &str) -> Result<u16> {
        let count = usize::from(count);
        let end = self
            .bit_pos
            .checked_add(count)
            .ok_or_else(|| MesError::Invalid("bit position overflows usize".to_owned()))?;
        if end > self.data.len().saturating_mul(8) {
            return Err(MesError::Invalid(format!(
                "truncated {field} at bit offset 0x{:X}",
                self.bit_pos
            )));
        }

        let mut value = 0u16;
        for _ in 0..count {
            let byte = self.data[self.bit_pos / 8];
            let shift = 7 - (self.bit_pos % 8);
            value = (value << 1) | u16::from((byte >> shift) & 1);
            self.bit_pos += 1;
        }
        Ok(value)
    }
}

struct BitWriter {
    data: Vec<u8>,
    current: u8,
    used_bits: u8,
}

impl BitWriter {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            current: 0,
            used_bits: 0,
        }
    }

    fn write_bits(&mut self, value: u16, count: u8) {
        for shift in (0..count).rev() {
            let bit = ((value >> shift) & 1) as u8;
            self.current = (self.current << 1) | bit;
            self.used_bits += 1;
            if self.used_bits == 8 {
                self.data.push(self.current);
                self.current = 0;
                self.used_bits = 0;
            }
        }
    }

    fn finish(mut self) -> Vec<u8> {
        if self.used_bits != 0 {
            self.current <<= 8 - self.used_bits;
            self.data.push(self.current);
        }
        self.data
    }
}

pub fn decompress(input: &[u8]) -> Result<(Vec<u8>, DecodeStats)> {
    if input.is_empty() {
        return Err(MesError::Invalid("stream is empty".to_owned()));
    }

    let mut reader = BitReader::new(input);
    let mut frame = [0u8; FRAME_SIZE];
    let mut frame_written = [false; FRAME_SIZE];
    let mut frame_pos = FRAME_INIT;
    let mut output = Vec::new();
    let mut literal_tokens = 0usize;
    let mut match_tokens = 0usize;
    let mut initial_zero_bytes = 0usize;

    loop {
        let flag = reader.read_bits(1, "token flag")?;
        if flag != 0 {
            let value = reader.read_bits(8, "literal")? as u8;
            if output.len() == MAX_OUTPUT_SIZE {
                return Err(MesError::Invalid(format!(
                    "output exceeds the 0x{MAX_OUTPUT_SIZE:X}-byte script limit"
                )));
            }
            output.push(value);
            frame[frame_pos] = value;
            frame_written[frame_pos] = true;
            frame_pos = (frame_pos + 1) & FRAME_MASK;
            literal_tokens += 1;
            continue;
        }

        let source_pos = usize::from(reader.read_bits(12, "match index")?);
        if source_pos == 0 {
            break;
        }
        let length = usize::from(reader.read_bits(4, "match length")?) + MIN_MATCH;
        if output
            .len()
            .checked_add(length)
            .is_none_or(|size| size > MAX_OUTPUT_SIZE)
        {
            return Err(MesError::Invalid(format!(
                "match at output offset 0x{:X} exceeds the 0x{MAX_OUTPUT_SIZE:X}-byte script limit",
                output.len()
            )));
        }

        for offset in 0..length {
            let read_pos = (source_pos + offset) & FRAME_MASK;
            let value = frame[read_pos];
            if !frame_written[read_pos] {
                initial_zero_bytes += 1;
            }
            output.push(value);
            frame[frame_pos] = value;
            frame_written[frame_pos] = true;
            frame_pos = (frame_pos + 1) & FRAME_MASK;
        }
        match_tokens += 1;
    }

    if output.len() < 2 {
        return Err(MesError::Invalid(
            "decoded script is shorter than its 16-bit entry offset".to_owned(),
        ));
    }

    let entry_offset = u16::from_le_bytes([output[0], output[1]]);
    if usize::from(entry_offset) >= output.len() {
        return Err(MesError::Invalid(format!(
            "entry offset 0x{entry_offset:X} is outside the 0x{:X}-byte decoded script",
            output.len()
        )));
    }

    let consumed_bits = reader.bit_pos;
    let consumed_bytes = consumed_bits.div_ceil(8);
    let padding_bits = ((8 - consumed_bits % 8) % 8) as u8;
    let padding_value = if padding_bits == 0 {
        0
    } else {
        input[consumed_bytes - 1] & ((1u8 << padding_bits) - 1)
    };
    let trailing_bytes = input
        .len()
        .checked_sub(consumed_bytes)
        .expect("consumed byte count is bounded by the input length");

    let stats = DecodeStats {
        output_size: output.len(),
        consumed_bits,
        consumed_bytes,
        trailing_bytes,
        padding_bits,
        padding_value,
        entry_offset,
        literal_tokens,
        match_tokens,
        initial_zero_bytes,
        sha256: hex::encode(Sha256::digest(&output)),
    };
    Ok((output, stats))
}

fn add_position(index: &mut HashMap<u16, VecDeque<usize>>, data: &[u8], position: usize) {
    let Some(&next) = data.get(position + 1) else {
        return;
    };
    let key = u16::from_be_bytes([data[position], next]);
    index.entry(key).or_default().push_back(position);
}

fn trim_candidates(candidates: &mut VecDeque<usize>, current: usize) {
    while candidates
        .front()
        .is_some_and(|position| current.saturating_sub(*position) > FRAME_SIZE)
    {
        candidates.pop_front();
    }
}

fn find_match(
    input: &[u8],
    current: usize,
    positions: &mut HashMap<u16, VecDeque<usize>>,
) -> Option<(usize, usize)> {
    let max_len = (input.len() - current).min(MAX_MATCH);
    if max_len < MIN_MATCH {
        return None;
    }

    let key = u16::from_be_bytes([input[current], input[current + 1]]);
    let candidates = positions.get_mut(&key)?;
    trim_candidates(candidates, current);

    let mut best = None;
    for &candidate in candidates.iter().rev() {
        let distance = current - candidate;
        if distance == 0 || distance > FRAME_SIZE {
            continue;
        }
        let frame_pos = (FRAME_INIT + candidate) & FRAME_MASK;
        if frame_pos == 0 {
            continue;
        }

        let mut length = 0usize;
        while length < max_len && input[candidate + length] == input[current + length] {
            length += 1;
        }
        if length >= MIN_MATCH
            && best
                .as_ref()
                .is_none_or(|(_, best_length)| length > *best_length)
        {
            best = Some((frame_pos, length));
            if length == max_len {
                break;
            }
        }
    }
    best
}

pub fn compress(input: &[u8]) -> Result<Vec<u8>> {
    if input.len() < 2 {
        return Err(MesError::Invalid(
            "decoded script is shorter than its 16-bit entry offset".to_owned(),
        ));
    }
    if input.len() > MAX_OUTPUT_SIZE {
        return Err(MesError::Invalid(format!(
            "decoded script is {} bytes; runtime limit is 0x{MAX_OUTPUT_SIZE:X}",
            input.len()
        )));
    }
    let entry_offset = u16::from_le_bytes([input[0], input[1]]);
    if usize::from(entry_offset) >= input.len() {
        return Err(MesError::Invalid(format!(
            "entry offset 0x{entry_offset:X} is outside the 0x{:X}-byte decoded script",
            input.len()
        )));
    }

    let mut positions = HashMap::<u16, VecDeque<usize>>::new();
    let mut writer = BitWriter::new();
    let mut current = 0usize;

    while current < input.len() {
        let best = find_match(input, current, &mut positions);
        if let Some((source_pos, length)) = best {
            writer.write_bits(0, 1);
            writer.write_bits(source_pos as u16, 12);
            writer.write_bits((length - MIN_MATCH) as u16, 4);
            for position in current..current + length {
                add_position(&mut positions, input, position);
            }
            current += length;
        } else {
            writer.write_bits(1, 1);
            writer.write_bits(u16::from(input[current]), 8);
            add_position(&mut positions, input, current);
            current += 1;
        }
    }

    writer.write_bits(0, 1);
    writer.write_bits(0, 12);
    let packed = writer.finish();
    let (round_trip, stats) = decompress(&packed)?;
    if round_trip != input || stats.trailing_bytes != 0 || stats.padding_value != 0 {
        return Err(MesError::Invalid(
            "internal MES compression round-trip verification failed".to_owned(),
        ));
    }
    Ok(packed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_literal_match_and_terminator() {
        let mut writer = BitWriter::new();
        for value in [0x06u8, 0x00, b'A', b'B', b'C'] {
            writer.write_bits(1, 1);
            writer.write_bits(u16::from(value), 8);
        }
        writer.write_bits(0, 1);
        writer.write_bits(3, 12);
        writer.write_bits(1, 4);
        writer.write_bits(0, 1);
        writer.write_bits(0, 12);

        let (decoded, stats) = decompress(&writer.finish()).expect("valid stream");
        assert_eq!(decoded, b"\x06\x00ABCABC");
        assert_eq!(stats.literal_tokens, 5);
        assert_eq!(stats.match_tokens, 1);
        assert_eq!(stats.entry_offset, 6);
        assert_eq!(stats.trailing_bytes, 0);
    }

    #[test]
    fn compress_round_trips_repetition_and_direct_bytes() {
        let mut input = b"\x06\x00".to_vec();
        for _ in 0..400 {
            input.extend_from_slice(b"ABABABABABABABAB");
        }
        input.extend(0x61u8..=0x7f);
        input.extend(0xd0u8..=0xff);
        let packed = compress(&input).expect("compress");
        let (decoded, stats) = decompress(&packed).expect("decompress");
        assert_eq!(decoded, input);
        assert_eq!(stats.trailing_bytes, 0);
        assert!(packed.len() < input.len());
    }

    #[test]
    fn rejects_truncated_literal() {
        let error = decompress(&[0x80]).expect_err("literal is truncated");
        assert!(error.to_string().contains("truncated literal"));
    }

    #[test]
    fn rejects_entry_offset_outside_output() {
        let mut writer = BitWriter::new();
        for value in [0xffu8, 0xff, 0] {
            writer.write_bits(1, 1);
            writer.write_bits(u16::from(value), 8);
        }
        writer.write_bits(0, 1);
        writer.write_bits(0, 12);
        let error = decompress(&writer.finish()).expect_err("bad entry offset");
        assert!(error.to_string().contains("entry offset"));
    }
}
