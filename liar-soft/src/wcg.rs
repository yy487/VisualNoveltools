use std::error::Error;
use std::fmt;

const HEADER_SIZE: usize = 16;
const STREAM_HEADER_SIZE: usize = 12;
const MAX_IMAGE_BYTES: usize = 512 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WcgImage {
    pub width: u32,
    pub height: u32,
    /// Pixels in BGRA byte order, matching the original engine decoder.
    pub bgra: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WcgError(String);

impl WcgError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for WcgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for WcgError {}

pub fn looks_like_wcg(bytes: &[u8]) -> bool {
    bytes.len() >= HEADER_SIZE
        && bytes[0] == b'W'
        && bytes[1] == b'G'
        && (u16::from_le_bytes([bytes[2], bytes[3]]) & 0x0f) == 1
        && bytes[4] == 0x20
        && bytes[5] == 0
}

pub fn decode_wcg(bytes: &[u8]) -> Result<WcgImage, WcgError> {
    if !looks_like_wcg(bytes) {
        return Err(WcgError::new("not a supported Liar-soft WCG stream"));
    }

    let width = read_u32(bytes, 8)?;
    let height = read_u32(bytes, 12)?;
    if width == 0 || height == 0 {
        return Err(WcgError::new("WCG dimensions must be non-zero"));
    }
    let pixels = usize::try_from(width)
        .ok()
        .and_then(|w| usize::try_from(height).ok().and_then(|h| w.checked_mul(h)))
        .ok_or_else(|| WcgError::new("WCG dimensions overflow this platform"))?;
    let image_bytes = pixels
        .checked_mul(4)
        .ok_or_else(|| WcgError::new("WCG pixel buffer size overflow"))?;
    if image_bytes > MAX_IMAGE_BYTES {
        return Err(WcgError::new(format!(
            "WCG image requires {image_bytes} bytes, above the safety limit"
        )));
    }

    let mut bgra = vec![0; image_bytes];
    let next = decode_stream(bytes, HEADER_SIZE, pixels, 2, &mut bgra)?;
    let _end = decode_stream(bytes, next, pixels, 0, &mut bgra)?;
    for alpha in bgra.iter_mut().skip(3).step_by(4) {
        *alpha = !*alpha;
    }

    Ok(WcgImage {
        width,
        height,
        bgra,
    })
}

fn decode_stream(
    bytes: &[u8],
    start: usize,
    pixels: usize,
    channel_offset: usize,
    output: &mut [u8],
) -> Result<usize, WcgError> {
    let header_end = start
        .checked_add(STREAM_HEADER_SIZE)
        .ok_or_else(|| WcgError::new("WCG stream header offset overflow"))?;
    if header_end > bytes.len() {
        return Err(WcgError::new("truncated WCG stream header"));
    }

    let unpacked_size = usize::try_from(read_u32(bytes, start)?)
        .map_err(|_| WcgError::new("WCG unpacked size does not fit this platform"))?;
    let expected_size = pixels
        .checked_mul(2)
        .ok_or_else(|| WcgError::new("WCG plane size overflow"))?;
    if unpacked_size != expected_size {
        return Err(WcgError::new(format!(
            "invalid WCG plane size: declared {unpacked_size}, expected {expected_size}"
        )));
    }

    let data_size = usize::try_from(read_u32(bytes, start + 4)?)
        .map_err(|_| WcgError::new("WCG compressed size does not fit this platform"))?;
    let index_size = usize::from(read_u16(bytes, start + 8)?);
    if index_size == 0 {
        return Err(WcgError::new("WCG palette is empty"));
    }
    let palette_bytes = index_size
        .checked_mul(2)
        .ok_or_else(|| WcgError::new("WCG palette size overflow"))?;
    let palette_end = header_end
        .checked_add(palette_bytes)
        .ok_or_else(|| WcgError::new("WCG palette offset overflow"))?;
    let data_end = palette_end
        .checked_add(data_size)
        .ok_or_else(|| WcgError::new("WCG compressed data offset overflow"))?;
    if data_end > bytes.len() {
        return Err(WcgError::new("truncated WCG palette or compressed data"));
    }

    let mut palette = Vec::with_capacity(index_size);
    for pos in (header_end..palette_end).step_by(2) {
        palette.push(u16::from_le_bytes([bytes[pos], bytes[pos + 1]]));
    }

    let small_index = index_size < 0x1002;
    let index_length_bits = if small_index { 3 } else { 4 };
    let index_length_limit = if small_index { 6 } else { 14 };
    let mut bits = MsbBits::new(&bytes[palette_end..data_end]);
    let mut pixel = 0usize;

    while pixel < pixels {
        let mut count = 1usize;
        let mut index_length = bits.get_bits(index_length_bits)? as usize;
        if index_length == 0 {
            count = bits.get_bits(4)? as usize + 2;
            index_length = bits.get_bits(index_length_bits)? as usize;
        }
        if index_length == 0 {
            return Err(WcgError::new("invalid zero-length WCG palette index"));
        }
        if count > pixels - pixel {
            return Err(WcgError::new("WCG run exceeds the destination plane"));
        }

        let index = decode_index(&mut bits, index_length, index_length_limit)?;
        let word = *palette
            .get(index)
            .ok_or_else(|| WcgError::new(format!("WCG palette index {index} is out of range")))?;
        let [low, high] = word.to_le_bytes();
        for _ in 0..count {
            let dst = pixel * 4 + channel_offset;
            output[dst] = low;
            output[dst + 1] = high;
            pixel += 1;
        }
    }

    Ok(data_end)
}

fn decode_index(
    bits: &mut MsbBits<'_>,
    mut count: usize,
    index_length_limit: usize,
) -> Result<usize, WcgError> {
    count -= 1;
    if count == 0 {
        return Ok(bits.get_bit()? as usize);
    }
    if count < index_length_limit {
        return Ok((1usize << count) | bits.get_bits(count)? as usize);
    }
    while bits.get_bit()? != 0 {
        if count >= 16 {
            return Err(WcgError::new("invalid WCG extended palette index length"));
        }
        count += 1;
    }
    Ok((1usize << count) | bits.get_bits(count)? as usize)
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, WcgError> {
    let end = offset
        .checked_add(2)
        .ok_or_else(|| WcgError::new("WCG read offset overflow"))?;
    let value = bytes
        .get(offset..end)
        .ok_or_else(|| WcgError::new("truncated WCG integer"))?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, WcgError> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| WcgError::new("WCG read offset overflow"))?;
    let value = bytes
        .get(offset..end)
        .ok_or_else(|| WcgError::new("truncated WCG integer"))?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

struct MsbBits<'a> {
    bytes: &'a [u8],
    bit_position: usize,
}

impl<'a> MsbBits<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bit_position: 0,
        }
    }

    fn get_bit(&mut self) -> Result<u8, WcgError> {
        if self.bit_position >= self.bytes.len().saturating_mul(8) {
            return Err(WcgError::new("truncated WCG bitstream"));
        }
        let byte = self.bytes[self.bit_position / 8];
        let shift = 7 - (self.bit_position % 8);
        self.bit_position += 1;
        Ok((byte >> shift) & 1)
    }

    fn get_bits(&mut self, count: usize) -> Result<u32, WcgError> {
        if count > 32 {
            return Err(WcgError::new("internal WCG bit request exceeds 32 bits"));
        }
        let mut value = 0u32;
        for _ in 0..count {
            value = (value << 1) | u32::from(self.get_bit()?);
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_pixel_stream(word: u16) -> Vec<u8> {
        let mut stream = Vec::new();
        stream.extend_from_slice(&2u32.to_le_bytes());
        stream.extend_from_slice(&1u32.to_le_bytes());
        stream.extend_from_slice(&1u16.to_le_bytes());
        stream.extend_from_slice(&7u16.to_le_bytes());
        stream.extend_from_slice(&word.to_le_bytes());
        // index-length 1 (001), then palette index 0; padded to one byte.
        stream.push(0b0010_0000);
        stream
    }

    #[test]
    fn decodes_two_wcg_planes_and_inverts_alpha() {
        let mut input = b"WGq\x02\x20\0\0\x40\x01\0\0\0\x01\0\0\0".to_vec();
        input.extend_from_slice(&one_pixel_stream(0xbb33));
        input.extend_from_slice(&one_pixel_stream(0x2211));

        let image = decode_wcg(&input).unwrap();
        assert_eq!((image.width, image.height), (1, 1));
        assert_eq!(image.bgra, [0x11, 0x22, 0x33, 0x44]);
    }

    #[test]
    fn rejects_truncated_stream() {
        assert!(decode_wcg(b"WGq\x02\x20\0\0\x40").is_err());
    }
}
