use crate::Result;

pub const MAGIC: &[u8; 4] = b"BIZ2";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnmInfo {
    pub width: u16,
    pub height: u16,
    pub compressed_bytes: usize,
    pub decoded_bytes: usize,
    pub frame_bytes: usize,
    pub frames: usize,
}

pub fn inspect(data: &[u8]) -> Result<AnmInfo> {
    if data.len() < 8 {
        return Err(format!("truncated ANM header: {} byte(s)", data.len()));
    }
    if &data[..4] != MAGIC {
        return Err(format!(
            "unsupported ANM magic {:02X?}; expected BIZ2",
            &data[..4]
        ));
    }
    let width = u16::from_le_bytes([data[4], data[5]]);
    let height = u16::from_le_bytes([data[6], data[7]]);
    if width == 0 || height == 0 {
        return Err(format!("invalid ANM dimensions {width}x{height}"));
    }

    let mut ring = [0u8; 4096];
    let mut ring_cursor = 4078usize;
    let mut cursor = 8usize;
    let mut flags = 0u16;
    let mut decoded_bytes = 0usize;

    while cursor < data.len() {
        flags >>= 1;
        if flags & 0x100 == 0 {
            flags = u16::from(data[cursor]) | 0xFF00;
            cursor += 1;
        }
        if flags & 1 != 0 {
            if cursor >= data.len() {
                return Err("truncated ANM literal".to_owned());
            }
            let value = data[cursor];
            cursor += 1;
            ring[ring_cursor] = value;
            ring_cursor = (ring_cursor + 1) & 0xFFF;
            decoded_bytes = decoded_bytes
                .checked_add(1)
                .ok_or_else(|| "ANM decoded size overflow".to_owned())?;
        } else {
            if cursor + 1 >= data.len() {
                return Err("truncated ANM back-reference".to_owned());
            }
            let offset = usize::from(data[cursor]) | ((usize::from(data[cursor + 1]) & 0xF0) << 4);
            let length = usize::from(data[cursor + 1] & 0x0F) + 3;
            cursor += 2;
            for index in 0..length {
                let value = ring[(offset + index) & 0xFFF];
                ring[ring_cursor] = value;
                ring_cursor = (ring_cursor + 1) & 0xFFF;
            }
            decoded_bytes = decoded_bytes
                .checked_add(length)
                .ok_or_else(|| "ANM decoded size overflow".to_owned())?;
        }
    }

    let frame_bytes = usize::from(width)
        .checked_mul(usize::from(height))
        .and_then(|pixels| pixels.checked_mul(3))
        .ok_or_else(|| "ANM frame size overflow".to_owned())?;
    if !decoded_bytes.is_multiple_of(frame_bytes) {
        return Err(format!(
            "ANM decoded size {decoded_bytes} is not a whole number of {width}x{height}x24-bit frames"
        ));
    }

    Ok(AnmInfo {
        width,
        height,
        compressed_bytes: data.len() - 8,
        decoded_bytes,
        frame_bytes,
        frames: decoded_bytes / frame_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_magic() {
        let error = inspect(b"NOPE\x01\x00\x01\x00").unwrap_err();
        assert!(error.contains("magic"));
    }
}
