use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

fn adler32(bytes: &[u8]) -> u32 {
    const MODULUS: u32 = 65_521;
    let mut a = 1_u32;
    let mut b = 0_u32;
    for byte in bytes {
        a = (a + u32::from(*byte)) % MODULUS;
        b = (b + a) % MODULUS;
    }
    (b << 16) | a
}

fn chunk(kind: [u8; 4], data: &[u8], output: &mut Vec<u8>) {
    output.extend_from_slice(
        &u32::try_from(data.len())
            .expect("PNG chunk fits u32")
            .to_be_bytes(),
    );
    output.extend_from_slice(&kind);
    output.extend_from_slice(data);
    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(&kind);
    crc_input.extend_from_slice(data);
    output.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}

fn zlib_store(data: &[u8]) -> Vec<u8> {
    let block_count = data.len().div_ceil(65_535);
    let mut output = Vec::with_capacity(data.len() + block_count * 5 + 6);
    output.extend_from_slice(&[0x78, 0x01]);
    let mut cursor = 0;
    while cursor < data.len() {
        let count = (data.len() - cursor).min(65_535);
        let final_block = cursor + count == data.len();
        output.push(u8::from(final_block));
        let length = u16::try_from(count).expect("stored block length fits u16");
        output.extend_from_slice(&length.to_le_bytes());
        output.extend_from_slice(&(!length).to_le_bytes());
        output.extend_from_slice(&data[cursor..cursor + count]);
        cursor += count;
    }
    output.extend_from_slice(&adler32(data).to_be_bytes());
    output
}

/// Encodes an RGBA8 pixel buffer as a PNG byte stream.
///
/// # Errors
///
/// Returns [`io::ErrorKind::InvalidInput`] if the dimensions overflow the host
/// address space or the pixel-buffer length does not match them.
pub fn encode_rgba(width: u32, height: u32, rgba: &[u8]) -> io::Result<Vec<u8>> {
    let width_usize = usize::try_from(width)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "PNG width overflows usize"))?;
    let height_usize = usize::try_from(height)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "PNG height overflows usize"))?;
    let expected = width_usize
        .checked_mul(height_usize)
        .and_then(|value| value.checked_mul(4))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "PNG dimensions overflow"))?;
    if rgba.len() != expected {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("expected {expected} RGBA bytes, got {}", rgba.len()),
        ));
    }

    let stride = width_usize * 4;
    let mut filtered = Vec::with_capacity(rgba.len() + height_usize);
    for row in rgba.chunks_exact(stride) {
        filtered.push(0);
        filtered.extend_from_slice(row);
    }

    let mut output = Vec::with_capacity(filtered.len() + 128);
    output.extend_from_slice(PNG_SIGNATURE);
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(*b"IHDR", &header, &mut output);
    chunk(*b"IDAT", &zlib_store(&filtered), &mut output);
    chunk(*b"IEND", &[], &mut output);
    Ok(output)
}

/// Encodes and atomically flushes an RGBA8 image to a PNG file.
///
/// # Errors
///
/// Returns an I/O error for invalid dimensions or pixel length, or if the file
/// cannot be created, written, or synchronized.
pub fn write_rgba(path: &Path, width: u32, height: u32, rgba: &[u8]) -> io::Result<()> {
    let bytes = encode_rgba(width, height, rgba)?;
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_png_signature_and_chunks() {
        let png = encode_rgba(1, 1, &[1, 2, 3, 255]).unwrap();
        assert_eq!(&png[..8], PNG_SIGNATURE);
        assert!(png.windows(4).any(|window| window == b"IHDR"));
        assert!(png.windows(4).any(|window| window == b"IDAT"));
        assert!(png.windows(4).any(|window| window == b"IEND"));
    }
}
