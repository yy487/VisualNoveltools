use anyhow::{bail, Result};
use std::collections::{HashMap, VecDeque};

const WINDOW: usize = 4096;
const WINDOW_MASK: usize = WINDOW - 1;
const MAX_MATCH: usize = 18;
const MIN_MATCH: usize = 3;
const INITIAL_POS: usize = 0xFEE;

pub fn decompress(src: &[u8], expected_size: Option<usize>) -> Result<Vec<u8>> {
    let mut window = [0u8; WINDOW];
    let mut write_pos = INITIAL_POS;
    let mut input_pos = 0usize;
    let mut output = Vec::with_capacity(expected_size.unwrap_or(src.len().saturating_mul(4)));

    while input_pos < src.len() {
        let flags = src[input_pos];
        input_pos += 1;

        for bit in 0..8 {
            if input_pos >= src.len() {
                break;
            }

            if flags & (1 << bit) != 0 {
                let value = src[input_pos];
                input_pos += 1;
                output.push(value);
                window[write_pos] = value;
                write_pos = (write_pos + 1) & WINDOW_MASK;
            } else {
                if input_pos + 1 >= src.len() {
                    bail!("truncated LZSS back-reference at input offset {input_pos:#x}");
                }
                let low = src[input_pos] as usize;
                let high = src[input_pos + 1] as usize;
                input_pos += 2;
                let read_pos = low | ((high & 0xF0) << 4);
                let length = (high & 0x0F) + MIN_MATCH;

                for index in 0..length {
                    let value = window[(read_pos + index) & WINDOW_MASK];
                    output.push(value);
                    window[write_pos] = value;
                    write_pos = (write_pos + 1) & WINDOW_MASK;
                    if expected_size == Some(output.len()) {
                        return Ok(output);
                    }
                }
            }

            if expected_size == Some(output.len()) {
                return Ok(output);
            }
        }
    }

    if let Some(expected) = expected_size {
        if output.len() != expected {
            bail!(
                "LZSS output size mismatch: decoded {} bytes, expected {}",
                output.len(),
                expected
            );
        }
    }
    Ok(output)
}

pub fn compress(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(data.len());
    let mut positions: HashMap<[u8; 3], VecDeque<usize>> = HashMap::new();
    let mut pos = 0usize;

    while pos < data.len() {
        let flag_pos = output.len();
        output.push(0);
        let mut flags = 0u8;

        for bit in 0..8 {
            if pos >= data.len() {
                break;
            }

            let mut best_len = 0usize;
            let mut best_pos = 0usize;
            if let Some(key) = key_at(data, pos) {
                if let Some(candidates) = positions.get_mut(&key) {
                    while candidates.front().is_some_and(|&old| pos - old > WINDOW) {
                        candidates.pop_front();
                    }

                    for &candidate in candidates.iter().rev().take(128) {
                        let distance = pos - candidate;
                        if distance == 0 || distance > WINDOW {
                            continue;
                        }
                        let mut length = 0usize;
                        while length < MAX_MATCH
                            && pos + length < data.len()
                            && data[candidate + length] == data[pos + length]
                        {
                            length += 1;
                        }
                        if length > best_len {
                            best_len = length;
                            best_pos = candidate;
                            if length == MAX_MATCH {
                                break;
                            }
                        }
                    }
                }
            }

            if best_len >= MIN_MATCH {
                let ring_pos = (INITIAL_POS + best_pos) & WINDOW_MASK;
                output.push(ring_pos as u8);
                output.push((((ring_pos >> 4) & 0xF0) | (best_len - MIN_MATCH)) as u8);
                for inserted in pos..pos + best_len {
                    add_position(data, inserted, &mut positions);
                }
                pos += best_len;
            } else {
                flags |= 1 << bit;
                output.push(data[pos]);
                add_position(data, pos, &mut positions);
                pos += 1;
            }
        }

        output[flag_pos] = flags;
    }

    output
}

fn key_at(data: &[u8], pos: usize) -> Option<[u8; 3]> {
    (pos + 2 < data.len()).then(|| [data[pos], data[pos + 1], data[pos + 2]])
}

fn add_position(data: &[u8], pos: usize, positions: &mut HashMap<[u8; 3], VecDeque<usize>>) {
    let Some(key) = key_at(data, pos) else {
        return;
    };
    let candidates = positions.entry(key).or_default();
    candidates.push_back(pos);
    while candidates.front().is_some_and(|&old| pos - old > WINDOW) {
        candidates.pop_front();
    }
    while candidates.len() > 256 {
        candidates.pop_front();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_repetitive_and_binary_data() {
        let mut data = Vec::new();
        for index in 0..20_000usize {
            data.push(if index % 97 < 70 {
                (index % 11) as u8
            } else {
                (index.wrapping_mul(73) & 0xFF) as u8
            });
        }
        let packed = compress(&data);
        assert_eq!(decompress(&packed, Some(data.len())).unwrap(), data);
    }

    #[test]
    fn roundtrips_short_tail() {
        for length in 0..40usize {
            let data: Vec<u8> = (0..length).map(|x| (x * 17) as u8).collect();
            assert_eq!(decompress(&compress(&data), None).unwrap(), data);
        }
    }
}
