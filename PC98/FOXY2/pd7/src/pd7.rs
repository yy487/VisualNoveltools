use std::array;
use std::error::Error;
use std::fmt;

pub const SCREEN_WIDTH: usize = 640;
pub const SCREEN_HEIGHT: usize = 400;
const ROW_BYTES: usize = SCREEN_WIDTH / 8;
const PLANE_SIZE: usize = ROW_BYTES * SCREEN_HEIGHT;
const BLOCK_HEADER_SIZE: usize = 8 + 32 + 16;

const BLUE: usize = 0;
const RED: usize = 1;
const GREEN: usize = 2;
const INTENSITY: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pd7Error {
    pub offset: usize,
    pub message: String,
}

impl Pd7Error {
    fn new(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            message: message.into(),
        }
    }
}

impl fmt::Display for Pd7Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PD7 error at 0x{:X}: {}", self.offset, self.message)
    }
}

impl Error for Pd7Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockInfo {
    pub offset: usize,
    pub x_byte: u16,
    pub y: u16,
    pub width_bytes: u16,
    pub stored_height: u16,
    pub decoded_height: u16,
    pub stream_offset: usize,
    pub stream_end: usize,
    pub limit: usize,
}

impl BlockInfo {
    #[must_use]
    pub fn x_pixels(&self) -> usize {
        usize::from(self.x_byte) * 8
    }

    #[must_use]
    pub fn width_pixels(&self) -> usize {
        usize::from(self.width_bytes) * 8
    }

    #[must_use]
    pub fn trailing_bytes(&self) -> usize {
        self.limit.saturating_sub(self.stream_end)
    }
}

#[derive(Debug, Clone)]
pub struct DecodedImage {
    pub split_offset: Option<usize>,
    pub palette_words: [u16; 16],
    pub palette_rgb: [[u8; 3]; 16],
    pub blocks: Vec<BlockInfo>,
    pub rgba: Vec<u8>,
}

impl DecodedImage {
    #[must_use]
    pub fn crop_bounds(&self) -> (usize, usize, usize, usize) {
        let min_x = self
            .blocks
            .iter()
            .map(BlockInfo::x_pixels)
            .min()
            .unwrap_or(0);
        let min_y = self
            .blocks
            .iter()
            .map(|block| usize::from(block.y))
            .min()
            .unwrap_or(0);
        let max_x = self
            .blocks
            .iter()
            .map(|block| block.x_pixels() + block.width_pixels())
            .max()
            .unwrap_or(SCREEN_WIDTH);
        let max_y = self
            .blocks
            .iter()
            .map(|block| usize::from(block.y) + usize::from(block.decoded_height))
            .max()
            .unwrap_or(SCREEN_HEIGHT);
        (min_x, min_y, max_x, max_y)
    }

    #[must_use]
    /// Returns the decoded pixels cropped to the union of all block rectangles.
    ///
    /// # Panics
    ///
    /// Panics only on a target whose `u32` cannot represent the fixed 640x400
    /// PC-98 canvas dimensions.
    pub fn cropped_rgba(&self) -> (u32, u32, Vec<u8>) {
        let (min_x, min_y, max_x, max_y) = self.crop_bounds();
        let width = max_x - min_x;
        let height = max_y - min_y;
        let mut pixels = Vec::with_capacity(width * height * 4);
        for y in min_y..max_y {
            let start = (y * SCREEN_WIDTH + min_x) * 4;
            let end = start + width * 4;
            pixels.extend_from_slice(&self.rgba[start..end]);
        }
        (
            u32::try_from(width).expect("crop width fits u32"),
            u32::try_from(height).expect("crop height fits u32"),
            pixels,
        )
    }
}

struct Reader<'a> {
    data: &'a [u8],
    cursor: usize,
    limit: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8], cursor: usize, limit: usize) -> Result<Self, Pd7Error> {
        if cursor > limit || limit > data.len() {
            return Err(Pd7Error::new(cursor, "invalid reader bounds"));
        }
        Ok(Self {
            data,
            cursor,
            limit,
        })
    }

    fn byte(&mut self, context: &str) -> Result<u8, Pd7Error> {
        if self.cursor >= self.limit {
            return Err(Pd7Error::new(
                self.cursor,
                format!("unexpected end while reading {context}"),
            ));
        }
        let value = self.data[self.cursor];
        self.cursor += 1;
        Ok(value)
    }
}

struct BlockState {
    x_byte: usize,
    y: usize,
    width_bytes: usize,
    height: usize,
    current_x: usize,
    current_y: usize,
    emitted: usize,
}

impl BlockState {
    fn destination_offset(&self) -> usize {
        (self.y + self.current_y) * ROW_BYTES + self.x_byte + self.current_x
    }

    fn source_offset(&self, previous_column: bool, source_row: u8) -> Result<usize, Pd7Error> {
        let source_x = if previous_column {
            self.current_x.checked_sub(1).ok_or_else(|| {
                Pd7Error::new(0, "back-reference uses previous column before column zero")
            })?
        } else {
            self.current_x
        };
        let source_y = self.y + usize::from(source_row);
        let absolute_x = self.x_byte + source_x;
        if absolute_x >= ROW_BYTES || source_y >= SCREEN_HEIGHT {
            return Err(Pd7Error::new(
                0,
                format!("back-reference outside VRAM at byte-column {absolute_x}, row {source_y}"),
            ));
        }
        Ok(source_y * ROW_BYTES + absolute_x)
    }

    fn advance(&mut self) -> bool {
        self.emitted += 1;
        self.current_y += 1;
        if self.current_y == self.height {
            self.current_y = 0;
            self.current_x += 1;
        }
        self.emitted == self.width_bytes * self.height
    }
}

fn read_u16(data: &[u8], offset: usize, context: &str) -> Result<u16, Pd7Error> {
    let bytes = data
        .get(offset..offset + 2)
        .ok_or_else(|| Pd7Error::new(offset, format!("truncated {context}")))?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_palette(data: &[u8], offset: usize) -> Result<[u16; 16], Pd7Error> {
    let mut palette = [0_u16; 16];
    for (index, entry) in palette.iter_mut().enumerate() {
        *entry = read_u16(data, offset + index * 2, "palette")?;
        if *entry > 0x0FFF {
            return Err(Pd7Error::new(
                offset + index * 2,
                format!("palette entry 0x{entry:04X} exceeds 12 bits"),
            ));
        }
    }
    Ok(palette)
}

fn map_component(component: u16) -> u8 {
    const LEVELS: [u8; 8] = [0, 2, 4, 6, 8, 10, 12, 15];
    LEVELS[usize::from(component >> 1)] * 17
}

fn convert_palette(words: &[u16; 16]) -> [[u8; 3]; 16] {
    array::from_fn(|index| {
        let word = words[index];
        let blue = map_component(word & 0xF);
        let red = map_component((word >> 4) & 0xF);
        let green = map_component((word >> 8) & 0xF);
        [red, green, blue]
    })
}

fn emit(planes: &mut [Vec<u8>; 4], state: &mut BlockState, values: [u8; 4]) -> bool {
    let offset = state.destination_offset();
    for (plane, value) in planes.iter_mut().zip(values) {
        plane[offset] = value;
    }
    state.advance()
}

fn copy_back_reference(
    planes: &mut [Vec<u8>; 4],
    state: &mut BlockState,
    previous_column: bool,
    source_row: u8,
    count: usize,
) -> Result<(), Pd7Error> {
    let source_start = state.source_offset(previous_column, source_row)?;
    for index in 0..count {
        let source = source_start + index * ROW_BYTES;
        if source >= PLANE_SIZE {
            return Err(Pd7Error::new(
                0,
                format!("back-reference source 0x{source:X} exceeds VRAM"),
            ));
        }
        let values = array::from_fn(|plane| planes[plane][source]);
        if emit(planes, state, values) {
            break;
        }
    }
    Ok(())
}

fn compact_values(index: usize, first: u8, second: u8, third: Option<u8>) -> [u8; 4] {
    match index {
        9 => [first, second, third.expect("opcode 9 has three values"), 0],
        10 => [first, first, second, 0],
        11 => [second, first, first, 0],
        12 => [first, second, first, 0],
        13 => [0, first, second, 0],
        14 => [first, 0, second, 0],
        15 => [first, second, 0, 0],
        _ => unreachable!("compact opcode index"),
    }
}

#[allow(clippy::too_many_lines)]
fn decode_block(
    data: &[u8],
    offset: usize,
    limit: usize,
    height_override: Option<u16>,
    planes: &mut [Vec<u8>; 4],
) -> Result<BlockInfo, Pd7Error> {
    if offset + BLOCK_HEADER_SIZE > limit {
        return Err(Pd7Error::new(offset, "truncated block header"));
    }

    let x_byte = read_u16(data, offset, "block x")?;
    let y = read_u16(data, offset + 2, "block y")?;
    let width_bytes = read_u16(data, offset + 4, "block width")?;
    let stored_height = read_u16(data, offset + 6, "block height")?;
    let decoded_height = height_override.unwrap_or(stored_height);

    if width_bytes == 0 || decoded_height == 0 {
        return Err(Pd7Error::new(offset, "zero-sized block"));
    }
    if usize::from(x_byte) + usize::from(width_bytes) > ROW_BYTES {
        return Err(Pd7Error::new(
            offset,
            format!("block exceeds {ROW_BYTES} VRAM bytes per row"),
        ));
    }
    if usize::from(y) + usize::from(decoded_height) > SCREEN_HEIGHT {
        return Err(Pd7Error::new(
            offset + 2,
            format!("block exceeds {SCREEN_HEIGHT} scanlines"),
        ));
    }

    read_palette(data, offset + 8)?;
    let opcode_offset = offset + 40;
    let mut opcodes = [0_u8; 16];
    opcodes.copy_from_slice(&data[opcode_offset..opcode_offset + 16]);

    let stream_offset = offset + BLOCK_HEADER_SIZE;
    let mut reader = Reader::new(data, stream_offset, limit)?;
    let mut state = BlockState {
        x_byte: usize::from(x_byte),
        y: usize::from(y),
        width_bytes: usize::from(width_bytes),
        height: usize::from(decoded_height),
        current_x: 0,
        current_y: 0,
        emitted: 0,
    };

    while state.emitted < state.width_bytes * state.height {
        let command_offset = reader.cursor;
        let command = reader.byte("command")?;
        let opcode_index = opcodes.iter().position(|candidate| *candidate == command);

        let Some(index) = opcode_index else {
            let blue = reader.byte("raw blue plane")?;
            let red = reader.byte("raw red plane")?;
            let green = reader.byte("raw green plane")?;
            emit(planes, &mut state, [blue, red, green, command]);
            continue;
        };

        let first = reader.byte("opcode payload or escape")?;
        if first == command {
            let blue = reader.byte("escaped raw blue plane")?;
            let red = reader.byte("escaped raw red plane")?;
            let green = reader.byte("escaped raw green plane")?;
            emit(planes, &mut state, [blue, red, green, command]);
            continue;
        }

        match index {
            0 => {
                let length = reader.byte("variable back-reference length")?;
                let previous_column = length & 0x80 != 0;
                let count = usize::from(length & 0x7F) + 1;
                copy_back_reference(planes, &mut state, previous_column, first, count)?;
            }
            1..=4 => {
                copy_back_reference(planes, &mut state, false, first, index)?;
            }
            5..=8 => {
                copy_back_reference(planes, &mut state, true, first, index - 4)?;
            }
            9..=15 => {
                let second = reader.byte("compact plane payload")?;
                let third = if index == 9 {
                    Some(reader.byte("compact green plane payload")?)
                } else {
                    None
                };
                let values = compact_values(index, first, second, third);
                if emit(planes, &mut state, values) {
                    return Err(Pd7Error::new(
                        command_offset,
                        "two-output opcode crosses the end of a block",
                    ));
                }

                let first = reader.byte("second compact plane payload")?;
                let second = reader.byte("second compact plane payload")?;
                let third = if index == 9 {
                    Some(reader.byte("second compact green plane payload")?)
                } else {
                    None
                };
                emit(
                    planes,
                    &mut state,
                    compact_values(index, first, second, third),
                );
            }
            _ => unreachable!(),
        }
    }

    Ok(BlockInfo {
        offset,
        x_byte,
        y,
        width_bytes,
        stored_height,
        decoded_height,
        stream_offset,
        stream_end: reader.cursor,
        limit,
    })
}

fn planes_to_rgba(planes: &[Vec<u8>; 4], palette: &[[u8; 3]; 16]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(SCREEN_WIDTH * SCREEN_HEIGHT * 4);
    for y in 0..SCREEN_HEIGHT {
        for x_byte in 0..ROW_BYTES {
            let offset = y * ROW_BYTES + x_byte;
            for bit in (0..8).rev() {
                let color_index = ((planes[BLUE][offset] >> bit) & 1)
                    | (((planes[RED][offset] >> bit) & 1) << 1)
                    | (((planes[GREEN][offset] >> bit) & 1) << 2)
                    | (((planes[INTENSITY][offset] >> bit) & 1) << 3);
                let [red, green, blue] = palette[usize::from(color_index)];
                rgba.extend_from_slice(&[red, green, blue, 255]);
            }
        }
    }
    rgba
}

/// Decodes one complete FOXY2 PD7 file into a 640x400 RGBA image.
///
/// # Errors
///
/// Returns [`Pd7Error`] when the input is truncated, has invalid geometry or
/// split offsets, or contains a command that would read outside decoded VRAM.
pub fn decode(data: &[u8]) -> Result<DecodedImage, Pd7Error> {
    if data.len() < 2 + BLOCK_HEADER_SIZE {
        return Err(Pd7Error::new(0, "file is too short for a PD7 block"));
    }

    let split_word = read_u16(data, 0, "split offset")?;
    let palette_words = read_palette(data, 10)?;
    let palette_rgb = convert_palette(&palette_words);
    let mut planes: [Vec<u8>; 4] = array::from_fn(|_| vec![0_u8; PLANE_SIZE]);
    let mut blocks = Vec::with_capacity(if split_word == 0 { 1 } else { 2 });

    let split_offset = if split_word == 0 {
        blocks.push(decode_block(data, 2, data.len(), None, &mut planes)?);
        None
    } else {
        let split = usize::from(split_word);
        if split < 2 + BLOCK_HEADER_SIZE || split + BLOCK_HEADER_SIZE > data.len() {
            return Err(Pd7Error::new(
                0,
                format!("invalid second-block offset 0x{split:X}"),
            ));
        }
        blocks.push(decode_block(data, 2, split, Some(200), &mut planes)?);
        blocks.push(decode_block(data, split, data.len(), None, &mut planes)?);
        Some(split)
    };

    let rgba = planes_to_rgba(&planes, &palette_rgb);
    Ok(DecodedImage {
        split_offset,
        palette_words,
        palette_rgb,
        blocks,
        rgba,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_byte_file(stream: &[u8], opcode0: u8) -> Vec<u8> {
        let mut data = vec![0_u8; 2 + BLOCK_HEADER_SIZE];
        data[6..8].copy_from_slice(&1_u16.to_le_bytes());
        data[8..10].copy_from_slice(&1_u16.to_le_bytes());
        for (index, opcode) in data[42..58].iter_mut().enumerate() {
            *opcode = opcode0.wrapping_add(u8::try_from(index).unwrap());
        }
        data.extend_from_slice(stream);
        data
    }

    #[test]
    fn decodes_raw_literal() {
        let data = one_byte_file(&[0x00, 0x80, 0x00, 0x00], 0xC8);
        let image = decode(&data).unwrap();
        assert_eq!(image.blocks[0].stream_end, data.len());
        assert_eq!(&image.rgba[..4], &[0, 0, 0, 255]);
    }

    #[test]
    fn decodes_escaped_literal() {
        let data = one_byte_file(&[0xC8, 0xC8, 0x80, 0x00, 0x00], 0xC8);
        let image = decode(&data).unwrap();
        assert_eq!(image.blocks[0].stream_end, data.len());
    }

    #[test]
    fn rejects_truncated_stream() {
        let data = one_byte_file(&[0x00, 0x80], 0xC8);
        let error = decode(&data).unwrap_err();
        assert!(error.message.contains("unexpected end"));
    }

    #[test]
    fn palette_layout_is_grb() {
        let mut words = [0_u16; 16];
        words[1] = 0x0F00;
        words[2] = 0x00F0;
        words[3] = 0x000F;
        let palette = convert_palette(&words);
        assert_eq!(palette[1], [0, 255, 0]);
        assert_eq!(palette[2], [255, 0, 0]);
        assert_eq!(palette[3], [0, 0, 255]);
    }
}
