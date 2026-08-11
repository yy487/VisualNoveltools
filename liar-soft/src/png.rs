use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PngError(String);

impl PngError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for PngError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for PngError {}

/// Encodes BGRA8 pixels as a standards-compliant RGBA PNG.
///
/// Deflate stored blocks are used deliberately so the converter remains a
/// dependency-free, offline-capable recovery tool.
pub fn encode_bgra_png(width: u32, height: u32, bgra: &[u8]) -> Result<Vec<u8>, PngError> {
    let row_bytes = usize::try_from(width)
        .ok()
        .and_then(|w| w.checked_mul(4))
        .ok_or_else(|| PngError::new("PNG row size overflow"))?;
    let expected = usize::try_from(height)
        .ok()
        .and_then(|h| row_bytes.checked_mul(h))
        .ok_or_else(|| PngError::new("PNG image size overflow"))?;
    if bgra.len() != expected {
        return Err(PngError::new(format!(
            "BGRA buffer has {} bytes, expected {expected}",
            bgra.len()
        )));
    }

    let scanline_size = row_bytes
        .checked_add(1)
        .ok_or_else(|| PngError::new("PNG scanline size overflow"))?;
    let raw_size = scanline_size
        .checked_mul(height as usize)
        .ok_or_else(|| PngError::new("PNG filtered data size overflow"))?;
    let mut raw = Vec::with_capacity(raw_size);
    for row in bgra.chunks_exact(row_bytes) {
        raw.push(0); // PNG filter type: None
        for pixel in row.chunks_exact(4) {
            raw.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }
    }

    let compressed = zlib_stored(&raw);
    let mut png = Vec::with_capacity(compressed.len() + 128);
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // RGBA8, deflate, filter, no interlace
    append_chunk(&mut png, *b"IHDR", &ihdr);
    append_chunk(&mut png, *b"IDAT", &compressed);
    append_chunk(&mut png, *b"IEND", &[]);
    Ok(png)
}

fn zlib_stored(raw: &[u8]) -> Vec<u8> {
    let blocks = raw.len().div_ceil(u16::MAX as usize).max(1);
    let mut out = Vec::with_capacity(raw.len() + blocks * 5 + 6);
    out.extend_from_slice(&[0x78, 0x01]);

    if raw.is_empty() {
        out.extend_from_slice(&[1, 0, 0, 0xff, 0xff]);
    } else {
        let chunk_count = raw.len().div_ceil(u16::MAX as usize);
        for (index, chunk) in raw.chunks(u16::MAX as usize).enumerate() {
            out.push(if index + 1 == chunk_count { 1 } else { 0 });
            let len = chunk.len() as u16;
            out.extend_from_slice(&len.to_le_bytes());
            out.extend_from_slice(&(!len).to_le_bytes());
            out.extend_from_slice(chunk);
        }
    }
    out.extend_from_slice(&adler32(raw).to_be_bytes());
    out
}

fn append_chunk(output: &mut Vec<u8>, kind: [u8; 4], data: &[u8]) {
    output.extend_from_slice(&(data.len() as u32).to_be_bytes());
    output.extend_from_slice(&kind);
    output.extend_from_slice(data);
    let mut checksum_data = Vec::with_capacity(4 + data.len());
    checksum_data.extend_from_slice(&kind);
    checksum_data.extend_from_slice(data);
    output.extend_from_slice(&crc32(&checksum_data).to_be_bytes());
}

fn adler32(bytes: &[u8]) -> u32 {
    const MOD: u32 = 65_521;
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in bytes {
        a = (a + u32::from(byte)) % MOD;
        b = (b + a) % MOD;
    }
    (b << 16) | a
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_png_signature_and_ihdr_dimensions() {
        let png = encode_bgra_png(1, 1, &[0x11, 0x22, 0x33, 0x44]).unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(&png[12..16], b"IHDR");
        assert_eq!(&png[16..20], &1u32.to_be_bytes());
        assert_eq!(&png[20..24], &1u32.to_be_bytes());
        assert_eq!(&png[24..29], &[8, 6, 0, 0, 0]);
    }
}
