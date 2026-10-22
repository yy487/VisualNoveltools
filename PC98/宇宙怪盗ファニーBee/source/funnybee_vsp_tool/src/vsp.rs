const HEADER_SIZE: usize = 0x3a;
const MAX_X_COLUMNS: u16 = 80;
const MAX_Y: u16 = 401;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VspHeader {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub reserved: u8,
    pub palette_bank: u8,
    pub palette: [[u8; 3]; 16],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedVsp {
    pub header: VspHeader,
    pub pixels: Vec<u8>,
    pub consumed_bytes: usize,
}

pub fn looks_like_vsp(source: &[u8]) -> bool {
    parse_header(source).is_ok()
}

pub fn decode_vsp(source: &[u8]) -> Result<DecodedVsp, String> {
    let header = parse_header(source)?;
    let width_columns = usize::from(header.width / 8);
    let height = usize::from(header.height);
    let width = usize::from(header.width);
    let pixel_count = width
        .checked_mul(height)
        .ok_or_else(|| "VSP 像素数量溢出".to_string())?;
    let mut pixels = vec![0u8; pixel_count];
    let mut current = [
        vec![0u8; height],
        vec![0u8; height],
        vec![0u8; height],
        vec![0u8; height],
    ];
    let mut previous = [
        vec![0u8; height],
        vec![0u8; height],
        vec![0u8; height],
        vec![0u8; height],
    ];
    let mut cursor = HEADER_SIZE;
    let mut mask = 0u8;

    for column in 0..width_columns {
        for plane in 0..4usize {
            let mut row = 0usize;
            while row < height {
                let command = take_byte(source, &mut cursor)?;
                match command {
                    0x08..=0xff => {
                        current[plane][row] = command;
                        row += 1;
                    }
                    0x00 => {
                        if column == 0 {
                            return Err(format!(
                                "VSP 首列在流偏移 0x{:X} 引用了前一列",
                                cursor - 1
                            ));
                        }
                        let count = usize::from(take_byte(source, &mut cursor)?) + 1;
                        ensure_room(row, count, height, cursor)?;
                        current[plane][row..row + count]
                            .copy_from_slice(&previous[plane][row..row + count]);
                        row += count;
                    }
                    0x01 => {
                        let count = usize::from(take_byte(source, &mut cursor)?) + 1;
                        let value = take_byte(source, &mut cursor)?;
                        ensure_room(row, count, height, cursor)?;
                        current[plane][row..row + count].fill(value);
                        row += count;
                    }
                    0x02 => {
                        let pairs = usize::from(take_byte(source, &mut cursor)?) + 1;
                        let first = take_byte(source, &mut cursor)?;
                        let second = take_byte(source, &mut cursor)?;
                        let count = pairs
                            .checked_mul(2)
                            .ok_or_else(|| "VSP 双字节 RLE 长度溢出".to_string())?;
                        ensure_room(row, count, height, cursor)?;
                        for _ in 0..pairs {
                            current[plane][row] = first;
                            current[plane][row + 1] = second;
                            row += 2;
                        }
                    }
                    0x03..=0x05 => {
                        let source_plane = usize::from(command - 0x03);
                        if source_plane >= plane {
                            return Err(format!(
                                "VSP 平面 {plane} 在流偏移 0x{:X} 引用了尚未解码的平面 {source_plane}",
                                cursor - 1
                            ));
                        }
                        let count = usize::from(take_byte(source, &mut cursor)?) + 1;
                        ensure_room(row, count, height, cursor)?;
                        let (decoded_planes, target_planes) = current.split_at_mut(plane);
                        let source_bytes = &decoded_planes[source_plane][row..row + count];
                        let target_bytes = &mut target_planes[0][row..row + count];
                        for (target, source) in target_bytes.iter_mut().zip(source_bytes) {
                            *target = *source ^ mask;
                        }
                        row += count;
                        mask = 0;
                    }
                    0x06 => mask = 0xff,
                    0x07 => {
                        current[plane][row] = take_byte(source, &mut cursor)?;
                        row += 1;
                    }
                }
            }
        }

        for (row, (((plane0, plane1), plane2), plane3)) in current[0]
            .iter()
            .zip(&current[1])
            .zip(&current[2])
            .zip(&current[3])
            .enumerate()
        {
            let destination = row * width + column * 8;
            for bit_index in 0..8usize {
                let bit = 7 - bit_index;
                let color = ((*plane0 >> bit) & 1)
                    | (((*plane1 >> bit) & 1) << 1)
                    | (((*plane2 >> bit) & 1) << 2)
                    | (((*plane3 >> bit) & 1) << 3);
                pixels[destination + bit_index] = color;
            }
        }
        std::mem::swap(&mut current, &mut previous);
    }

    Ok(DecodedVsp {
        header,
        pixels,
        consumed_bytes: cursor,
    })
}

fn parse_header(source: &[u8]) -> Result<VspHeader, String> {
    if source.len() < HEADER_SIZE {
        return Err(format!("文件短于 VSP 头: {} < {HEADER_SIZE}", source.len()));
    }
    let x0 = read_u16(source, 0);
    let y0 = read_u16(source, 2);
    let x1 = read_u16(source, 4);
    let y1 = read_u16(source, 6);
    let reserved = source[8];
    let palette_bank = source[9];
    if x0 >= x1 || x1 > MAX_X_COLUMNS || y0 >= y1 || y1 > MAX_Y || palette_bank > 15 {
        return Err(format!(
            "VSP 头范围无效: ({x0},{y0})-({x1},{y1}), bank={palette_bank}"
        ));
    }
    if source[10..HEADER_SIZE].iter().any(|value| *value > 15) {
        return Err("VSP 4-bit 调色板含有大于 15 的分量".to_string());
    }
    let width_columns = x1 - x0;
    let width = width_columns
        .checked_mul(8)
        .ok_or_else(|| "VSP 宽度溢出".to_string())?;
    let height = y1 - y0;
    let mut palette = [[0u8; 3]; 16];
    for (index, color) in palette.iter_mut().enumerate() {
        let offset = 10 + index * 3;
        let blue = source[offset] * 17;
        let red = source[offset + 1] * 17;
        let green = source[offset + 2] * 17;
        *color = [red, green, blue];
    }
    Ok(VspHeader {
        x: x0 * 8,
        y: y0,
        width,
        height,
        reserved,
        palette_bank,
        palette,
    })
}

fn ensure_room(row: usize, count: usize, height: usize, cursor: usize) -> Result<(), String> {
    if row.checked_add(count).is_none_or(|end| end > height) {
        return Err(format!(
            "VSP 解压在流偏移 0x{cursor:X} 越过图像高度: row={row}, count={count}, height={height}"
        ));
    }
    Ok(())
}

fn take_byte(source: &[u8], cursor: &mut usize) -> Result<u8, String> {
    let value = source
        .get(*cursor)
        .copied()
        .ok_or_else(|| format!("VSP 图像流在 0x{:X} 意外结束", *cursor))?;
    *cursor += 1;
    Ok(value)
}

fn read_u16(source: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([source[offset], source[offset + 1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_header(width_columns: u16, height: u16) -> Vec<u8> {
        let mut source = vec![0u8; HEADER_SIZE];
        source[4..6].copy_from_slice(&width_columns.to_le_bytes());
        source[6..8].copy_from_slice(&height.to_le_bytes());
        source
    }

    #[test]
    fn decodes_literal_planar_pixels() {
        let mut source = test_header(1, 1);
        source.extend_from_slice(&[0x07, 0x80, 0x07, 0x00, 0x07, 0x00, 0x07, 0x00]);
        let image = decode_vsp(&source).unwrap();
        assert_eq!(image.header.width, 8);
        assert_eq!(image.header.height, 1);
        assert_eq!(image.pixels, vec![1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(image.consumed_bytes, source.len());
    }

    #[test]
    fn decodes_rle_plane_copy_invert_and_previous_column() {
        let mut source = test_header(2, 4);
        source.extend_from_slice(&[
            0x01, 0x03, 0xaa, // plane 0: repeated byte
            0x02, 0x01, 0xf0, 0x0f, // plane 1: alternating pair
            0x03, 0x03, // plane 2: copy plane 0
            0x06, 0x05, 0x03, // plane 3: inverted plane 2
            0x00, 0x03, // second column, plane 0
            0x00, 0x03, // plane 1
            0x00, 0x03, // plane 2
            0x00, 0x03, // plane 3
        ]);
        let image = decode_vsp(&source).unwrap();
        assert_eq!(image.header.width, 16);
        for row in image.pixels.chunks_exact(16) {
            assert_eq!(&row[..8], &row[8..]);
        }
    }

    #[test]
    fn rejects_truncated_image_stream() {
        let source = test_header(1, 1);
        assert!(decode_vsp(&source).unwrap_err().contains("意外结束"));
    }
}
