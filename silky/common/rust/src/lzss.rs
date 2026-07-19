use anyhow::{bail, Result};

const N: usize = 4096;
const F: usize = 18;
const THRESHOLD: usize = 2;
const NIL: usize = N;

struct Encoder<'a> {
    input: &'a [u8],
    text: Vec<u8>,
    match_position: usize,
    match_length: usize,
    lson: Vec<usize>,
    rson: Vec<usize>,
    dad: Vec<usize>,
}

impl<'a> Encoder<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            text: vec![0; N + F - 1],
            match_position: 0,
            match_length: 0,
            lson: vec![0; N + 1],
            rson: vec![0; N + 257],
            dad: vec![0; N + 1],
        }
    }

    fn init_tree(&mut self) {
        for value in &mut self.rson[N + 1..N + 257] {
            *value = NIL;
        }
        for value in &mut self.dad[..N] {
            *value = NIL;
        }
    }

    fn insert_node(&mut self, r: usize) {
        let mut cmp = 1i16;
        let mut p = N + 1 + self.text[r] as usize;
        self.rson[r] = NIL;
        self.lson[r] = NIL;
        self.match_length = 0;

        loop {
            if cmp >= 0 {
                if self.rson[p] != NIL {
                    p = self.rson[p];
                } else {
                    self.rson[p] = r;
                    self.dad[r] = p;
                    return;
                }
            } else if self.lson[p] != NIL {
                p = self.lson[p];
            } else {
                self.lson[p] = r;
                self.dad[r] = p;
                return;
            }

            let mut matched = F;
            for i in 1..F {
                cmp = self.text[r + i] as i16 - self.text[p + i] as i16;
                if cmp != 0 {
                    matched = i;
                    break;
                }
            }
            if matched > self.match_length {
                self.match_position = p;
                self.match_length = matched;
                if matched >= F {
                    break;
                }
            }
        }

        self.dad[r] = self.dad[p];
        self.lson[r] = self.lson[p];
        self.rson[r] = self.rson[p];
        self.dad[self.lson[p]] = r;
        self.dad[self.rson[p]] = r;
        if self.rson[self.dad[p]] == p {
            self.rson[self.dad[p]] = r;
        } else {
            self.lson[self.dad[p]] = r;
        }
        self.dad[p] = NIL;
    }

    fn delete_node(&mut self, p: usize) {
        if self.dad[p] == NIL {
            return;
        }
        let q = if self.rson[p] == NIL {
            self.lson[p]
        } else if self.lson[p] == NIL {
            self.rson[p]
        } else {
            let mut q = self.lson[p];
            if self.rson[q] != NIL {
                q = self.rson[q];
                while self.rson[q] != NIL {
                    q = self.rson[q];
                }
                self.rson[self.dad[q]] = self.lson[q];
                self.dad[self.lson[q]] = self.dad[q];
                self.lson[q] = self.lson[p];
                self.dad[self.lson[p]] = q;
            }
            self.rson[q] = self.rson[p];
            self.dad[self.rson[p]] = q;
            q
        };
        self.dad[q] = self.dad[p];
        if self.rson[self.dad[p]] == p {
            self.rson[self.dad[p]] = q;
        } else {
            self.lson[self.dad[p]] = q;
        }
        self.dad[p] = NIL;
    }

    fn encode(mut self) -> Vec<u8> {
        if self.input.is_empty() {
            return Vec::new();
        }
        self.init_tree();
        let mut s = 0usize;
        let mut r = N - F;
        self.text[..r].fill(0);

        let mut length = self.input.len().min(F);
        self.text[r..r + length].copy_from_slice(&self.input[..length]);
        for i in 1..=F {
            self.insert_node(r - i);
        }
        self.insert_node(r);

        let mut input_pos = length;
        let mut code = [0u8; 17];
        let mut code_len = 1usize;
        let mut mask = 1u8;
        let mut output = Vec::new();

        loop {
            if self.match_length > length {
                self.match_length = length;
            }
            if self.match_length <= THRESHOLD {
                self.match_length = 1;
                code[0] |= mask;
                code[code_len] = self.text[r];
                code_len += 1;
            } else {
                code[code_len] = (self.match_position & 0xff) as u8;
                code_len += 1;
                code[code_len] = (((self.match_position >> 4) & 0xf0)
                    | (self.match_length - (THRESHOLD + 1))) as u8;
                code_len += 1;
            }

            mask = mask.wrapping_shl(1);
            if mask == 0 {
                output.extend_from_slice(&code[..code_len]);
                code.fill(0);
                code_len = 1;
                mask = 1;
            }

            let last_match_length = self.match_length;
            let mut consumed = 0usize;
            while consumed < last_match_length && input_pos < self.input.len() {
                self.delete_node(s);
                let byte = self.input[input_pos];
                input_pos += 1;
                self.text[s] = byte;
                if s < F - 1 {
                    self.text[s + N] = byte;
                }
                s = (s + 1) & (N - 1);
                r = (r + 1) & (N - 1);
                self.insert_node(r);
                consumed += 1;
            }
            while consumed < last_match_length {
                self.delete_node(s);
                s = (s + 1) & (N - 1);
                r = (r + 1) & (N - 1);
                length -= 1;
                if length != 0 {
                    self.insert_node(r);
                }
                consumed += 1;
            }
            if length == 0 {
                break;
            }
        }

        if code_len > 1 {
            output.extend_from_slice(&code[..code_len]);
        }
        output
    }
}

pub fn compress(data: &[u8]) -> Vec<u8> {
    Encoder::new(data).encode()
}

pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    decompress_checked(data, None)
}

pub fn decompress_exact(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    decompress_checked(data, Some(expected_size))
}

fn decompress_checked(data: &[u8], expected_size: Option<usize>) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(expected_size.unwrap_or(0));
    let mut text = vec![0u8; N + F - 1];
    let mut r = N - F;
    let mut flags = 0u16;
    let mut pos = 0usize;

    while pos < data.len() {
        flags >>= 1;
        if flags & 0x100 == 0 {
            flags = data[pos] as u16 | 0xff00;
            pos += 1;
        }
        if flags & 1 != 0 {
            if pos >= data.len() {
                bail!("truncated LZSS literal");
            }
            let byte = data[pos];
            pos += 1;
            output.push(byte);
            text[r] = byte;
            r = (r + 1) & (N - 1);
        } else {
            if pos + 1 >= data.len() {
                bail!("truncated LZSS back-reference");
            }
            let mut source = data[pos] as usize;
            let length_byte = data[pos + 1] as usize;
            pos += 2;
            source |= (length_byte & 0xf0) << 4;
            let length = (length_byte & 0x0f) + THRESHOLD + 1;
            for index in 0..length {
                let byte = text[(source + index) & (N - 1)];
                output.push(byte);
                text[r] = byte;
                r = (r + 1) & (N - 1);
            }
        }
        if let Some(expected) = expected_size {
            if output.len() > expected {
                bail!(
                    "LZSS output exceeds expected size: {} > {expected}",
                    output.len()
                );
            }
            if output.len() == expected {
                if pos != data.len() {
                    bail!("LZSS stream has trailing bytes after expected output");
                }
                return Ok(output);
            }
        }
    }

    if let Some(expected) = expected_size {
        if output.len() != expected {
            bail!("LZSS output size mismatch: {} != {expected}", output.len());
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_varied_inputs() {
        let inputs = [
            Vec::new(),
            vec![0],
            b"abcdefghijklmnopqr".to_vec(),
            b"abcabcabcabcabcabcabcabc".to_vec(),
            (0..=255).cycle().take(8193).collect(),
        ];
        for input in inputs {
            let encoded = compress(&input);
            let decoded = decompress_exact(&encoded, input.len()).unwrap();
            assert_eq!(decoded, input);
        }
    }

    #[test]
    fn compressor_matches_python_reference_vectors() {
        let vectors: &[(&[u8], &[u8])] = &[
            (b"", b""),
            (b"a", &[0x01, 0x61]),
            (
                b"abcdefghijklmnopqr",
                &[
                    0xff, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0xff, 0x69, 0x6a, 0x6b,
                    0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x03, 0x71, 0x72,
                ],
            ),
            (
                b"abcabcabcabcabcabcabcabc",
                &[0x07, 0x61, 0x62, 0x63, 0xee, 0xff, 0xf4, 0xf0],
            ),
        ];
        for (source, expected) in vectors {
            assert_eq!(&compress(source), expected);
        }
    }

    #[test]
    fn rejects_truncated_stream() {
        assert!(decompress_exact(&[0, 1], 10).is_err());
    }
}
