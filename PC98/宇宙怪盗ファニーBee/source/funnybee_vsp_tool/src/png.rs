use crate::DecodedVsp;

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

pub fn encode_png(image: &DecodedVsp) -> Result<Vec<u8>, String> {
    let width = usize::from(image.header.width);
    let height = usize::from(image.header.height);
    let expected_pixels = width
        .checked_mul(height)
        .ok_or_else(|| "PNG 像素数量溢出".to_string())?;
    if image.pixels.len() != expected_pixels {
        return Err(format!(
            "PNG 像素数量不匹配: {} != {expected_pixels}",
            image.pixels.len()
        ));
    }
    if width % 2 != 0 {
        return Err(format!("4-bit PNG 宽度必须为偶数: {width}"));
    }
    if image.pixels.iter().any(|pixel| *pixel > 15) {
        return Err("PNG 索引像素超出 16 色调色板".to_string());
    }

    let mut png = Vec::new();
    png.extend_from_slice(PNG_SIGNATURE);
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&u32::from(image.header.width).to_be_bytes());
    ihdr.extend_from_slice(&u32::from(image.header.height).to_be_bytes());
    ihdr.extend_from_slice(&[4, 3, 0, 0, 0]);
    write_chunk(&mut png, *b"IHDR", &ihdr);

    if image.header.x != 0 || image.header.y != 0 {
        let mut offsets = Vec::with_capacity(9);
        offsets.extend_from_slice(&i32::from(image.header.x).to_be_bytes());
        offsets.extend_from_slice(&i32::from(image.header.y).to_be_bytes());
        offsets.push(0);
        write_chunk(&mut png, *b"oFFs", &offsets);
    }
    if image.header.palette_bank != 0 {
        write_chunk(&mut png, *b"pbNk", &[image.header.palette_bank]);
    }

    let mut palette = Vec::with_capacity(48);
    for color in image.header.palette {
        palette.extend_from_slice(&color);
    }
    write_chunk(&mut png, *b"PLTE", &palette);

    let row_bytes = width / 2;
    let mut scanlines = Vec::with_capacity((row_bytes + 1) * height);
    for row in image.pixels.chunks_exact(width) {
        scanlines.push(0);
        for pair in row.chunks_exact(2) {
            scanlines.push((pair[0] << 4) | pair[1]);
        }
    }
    let compressed = zlib_store(&scanlines);
    write_chunk(&mut png, *b"IDAT", &compressed);
    write_chunk(&mut png, *b"IEND", &[]);
    Ok(png)
}

fn zlib_store(source: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(source.len() + source.len() / 65_535 * 5 + 11);
    output.extend_from_slice(&[0x78, 0x01]);
    if source.is_empty() {
        output.extend_from_slice(&[1, 0, 0, 0xff, 0xff]);
    } else {
        let mut offset = 0usize;
        while offset < source.len() {
            let count = (source.len() - offset).min(65_535);
            let final_block = offset + count == source.len();
            output.push(u8::from(final_block));
            let length = count as u16;
            output.extend_from_slice(&length.to_le_bytes());
            output.extend_from_slice(&(!length).to_le_bytes());
            output.extend_from_slice(&source[offset..offset + count]);
            offset += count;
        }
    }
    output.extend_from_slice(&adler32(source).to_be_bytes());
    output
}

fn write_chunk(output: &mut Vec<u8>, kind: [u8; 4], data: &[u8]) {
    output.extend_from_slice(&(data.len() as u32).to_be_bytes());
    output.extend_from_slice(&kind);
    output.extend_from_slice(data);
    let mut crc_source = Vec::with_capacity(4 + data.len());
    crc_source.extend_from_slice(&kind);
    crc_source.extend_from_slice(data);
    output.extend_from_slice(&crc32(&crc_source).to_be_bytes());
}

fn adler32(source: &[u8]) -> u32 {
    const MODULUS: u32 = 65_521;
    let mut first = 1u32;
    let mut second = 0u32;
    for byte in source {
        first = (first + u32::from(*byte)) % MODULUS;
        second = (second + first) % MODULUS;
    }
    (second << 16) | first
}

fn crc32(source: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in source {
        crc ^= u32::from(*byte);
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
    use crate::VspHeader;

    #[test]
    fn writes_indexed_png_with_offsets_palette_and_bank() {
        let image = DecodedVsp {
            header: VspHeader {
                x: 16,
                y: 3,
                width: 8,
                height: 1,
                reserved: 0,
                palette_bank: 2,
                palette: [[0; 3]; 16],
            },
            pixels: vec![0, 1, 2, 3, 4, 5, 6, 7],
            consumed_bytes: 0,
        };
        let png = encode_png(&image).unwrap();
        assert!(png.starts_with(PNG_SIGNATURE));
        assert!(png.windows(4).any(|window| window == b"oFFs"));
        assert!(png.windows(4).any(|window| window == b"pbNk"));
        assert!(png.windows(4).any(|window| window == b"PLTE"));
        assert!(png.windows(4).any(|window| window == b"IDAT"));
        assert!(png.windows(4).any(|window| window == b"IEND"));
    }
}
