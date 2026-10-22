//! Inner YOX bitmap font reader and lossless block/index rebuilder.

use std::convert::TryInto;
use std::io::Write;

fn u32_at(data: &[u8], at: usize) -> Result<u32, String> {
    let bytes = data.get(at..at + 4).ok_or("字体头或索引被截断")?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

#[derive(Clone, Debug)]
pub struct YoxFont {
    data: Vec<u8>,
    pub version: u32,
    pub range_mask: u32,
    pub width: usize,
    pub height: usize,
    pub glyph_count: usize,
    index: Vec<(usize, usize)>,
    width_gap: Vec<u8>,
    width_table: Vec<u8>,
    max_single_advance: u16,
    max_double_advance: u16,
}

impl YoxFont {
    pub fn parse(data: Vec<u8>) -> Result<Self, String> {
        if data.len() < 32 || &data[..4] != b"YOX\0" {
            return Err("不是内层 YOX 字库".into());
        }
        let version = u32_at(&data, 4)?;
        let range_mask = u32_at(&data, 8)?;
        let bitmap_size = u32_at(&data, 12)? as usize;
        let index_off = u32_at(&data, 16)? as usize;
        let wh = u32_at(&data, 20)?;
        let width = (wh & 0xffff) as usize;
        let height = (wh >> 16) as usize;
        let glyph_count = u32_at(&data, 24)? as usize;
        let width_off = u32_at(&data, 28)? as usize;
        let bitmap_end = 32usize.checked_add(bitmap_size).ok_or("字形区长度溢出")?;
        let index_bytes = glyph_count.checked_mul(8).ok_or("索引长度溢出")?;
        let index_end = index_off.checked_add(index_bytes).ok_or("索引区长度溢出")?;
        if bitmap_end != index_off || index_end > data.len() || width == 0 || height == 0 {
            return Err("YOX 字库区段偏移或尺寸无效".into());
        }
        let mut index = Vec::with_capacity(glyph_count);
        for i in 0..glyph_count {
            let at = index_off + i * 8;
            let off = u32_at(&data, at)? as usize;
            let size = u32_at(&data, at + 4)? as usize;
            let end = off.checked_add(size).ok_or("字形块偏移溢出")?;
            if end > bitmap_size {
                return Err(format!("字形槽 {i} 的块超出字形区"));
            }
            index.push((off, size));
        }
        // In these YOX files width_off follows the index. The reference tool
        // sliced in the opposite direction and consequently lost this table.
        if version != 0x0100_0000 || range_mask != 0x3d || glyph_count != 9589 {
            return Err("不支持的字库版本、CP932范围掩码或槽数".into());
        }
        if width_off != 0 && (width_off < index_end || width_off > data.len()) {
            return Err("宽度表偏移不在索引之后或超出文件".into());
        }
        let (width_gap, width_table) = if width_off == 0 {
            if index_end != data.len() {
                return Err("无宽度表标志但索引后仍有数据".into());
            }
            (Vec::new(), Vec::new())
        } else {
            let widths = data[width_off..].to_vec();
            if widths.len() != glyph_count.checked_mul(8).ok_or("宽度表长度溢出")? {
                return Err("宽度表长度不是每槽8字节".into());
            }
            (data[index_end..width_off].to_vec(), widths)
        };
        let mut font = Self {
            data,
            version,
            range_mask,
            width,
            height,
            glyph_count,
            index,
            width_gap,
            width_table,
            max_single_advance: 0,
            max_double_advance: 0,
        };
        for slot in 0..font.glyph_count {
            let Some(code) = crate::carrier::code_for_slot(slot, font.range_mask) else {
                continue;
            };
            if crate::carrier::cp932_decode(code).is_none() {
                continue;
            }
            if let Some(advance) = font.advance_for_code(code) {
                if code <= 0xff {
                    font.max_single_advance = font.max_single_advance.max(advance);
                } else {
                    font.max_double_advance = font.max_double_advance.max(advance);
                }
            }
        }
        Ok(font)
    }

    pub fn width_table(&self) -> &[u8] {
        &self.width_table
    }

    pub fn max_advances(&self) -> (u16, u16) {
        (self.max_single_advance, self.max_double_advance)
    }

    pub fn advance_for_code(&self, code: u16) -> Option<u16> {
        let mut slot = crate::carrier::slot_for_code(code, self.range_mask)?;
        if slot == 0 || slot == 95 {
            slot += 1;
        }
        if self.width_table.is_empty() {
            return Some(if code <= 0xff {
                self.width.div_ceil(2) as u16
            } else {
                self.width as u16
            });
        }
        let at = slot.checked_mul(8)?;
        Some(u16::from_le_bytes(
            self.width_table.get(at..at + 2)?.try_into().ok()?,
        ))
    }

    /// Replaces selected raw grayscale glyphs, retaining every other compressed block verbatim.
    pub fn rebuild(
        &self,
        replacements: &[(usize, Vec<u8>)],
        advances: &[(usize, u16)],
    ) -> Result<Vec<u8>, String> {
        let mut replacement_map = std::collections::BTreeMap::new();
        for (slot, pixels) in replacements {
            if *slot >= self.glyph_count {
                return Err(format!("字形槽 {slot} 越界"));
            }
            if pixels.len() != self.width * self.height {
                return Err(format!("字形槽 {slot} 像素数不符"));
            }
            if replacement_map.insert(*slot, pixels.as_slice()).is_some() {
                return Err(format!("字形槽 {slot} 被重复分配"));
            }
        }
        let mut bitmap = Vec::new();
        let mut index = Vec::with_capacity(self.glyph_count);
        for (slot, (offset, size)) in self.index.iter().copied().enumerate() {
            let start = 32 + offset;
            let block = if let Some(pixels) = replacement_map.get(&slot) {
                encode_glyph(pixels)?
            } else {
                let end = start.checked_add(size).ok_or("字形块范围溢出")?;
                self.data.get(start..end).ok_or("字形块超出输入")?.to_vec()
            };
            let next = u32::try_from(bitmap.len()).map_err(|_| "字形区超过 4 GiB")?;
            let len = u32::try_from(block.len()).map_err(|_| "字形块超过 4 GiB")?;
            index.push((next, len));
            bitmap.extend_from_slice(&block);
        }
        let index_off = 32usize.checked_add(bitmap.len()).ok_or("字形区长度溢出")?;
        let width_off = if self.width_table.is_empty() {
            0
        } else {
            index_off + self.glyph_count * 8 + self.width_gap.len()
        };
        let mut out = Vec::with_capacity(index_off + self.glyph_count * 8 + self.width_table.len());
        out.extend_from_slice(b"YOX\0");
        put_u32(&mut out, self.version);
        put_u32(&mut out, self.range_mask);
        put_u32(
            &mut out,
            u32::try_from(bitmap.len()).map_err(|_| "字形区超过 4 GiB")?,
        );
        put_u32(
            &mut out,
            u32::try_from(index_off).map_err(|_| "索引偏移超过 4 GiB")?,
        );
        put_u32(&mut out, (self.width as u32) | ((self.height as u32) << 16));
        put_u32(
            &mut out,
            u32::try_from(self.glyph_count).map_err(|_| "字形数过多")?,
        );
        put_u32(
            &mut out,
            u32::try_from(width_off).map_err(|_| "宽度表偏移超过 4 GiB")?,
        );
        out.extend_from_slice(&bitmap);
        for (offset, size) in index {
            put_u32(&mut out, offset);
            put_u32(&mut out, size);
        }
        out.extend_from_slice(&self.width_gap);
        let mut widths = self.width_table.clone();
        for (slot, advance) in advances {
            if *slot >= self.glyph_count || widths.is_empty() {
                return Err(format!("字宽槽 {slot} 越界或字库无宽度表"));
            }
            let at = slot * 8;
            widths[at..at + 2].copy_from_slice(&advance.to_le_bytes());
            // The renderer applies negative i16 x/y offsets from +4/+6.
            // A common-origin redraw must clear those legacy offsets; +2 is
            // retained because the confirmed draw path does not consume it.
            widths[at + 4..at + 8].fill(0);
        }
        out.extend_from_slice(&widths);
        Ok(out)
    }
}

fn encode_glyph(pixels: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(pixels).map_err(|e| e.to_string())?;
    let z = encoder.finish().map_err(|e| e.to_string())?;
    let mut block = Vec::with_capacity(16 + z.len());
    block.extend_from_slice(b"YOX\0");
    put_u32(&mut block, 2);
    put_u32(
        &mut block,
        u32::try_from(pixels.len()).map_err(|_| "字形像素过多")?,
    );
    put_u32(&mut block, 0);
    block.extend_from_slice(&z);
    Ok(block)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebuild_preserves_width_table_and_unmodified_blocks() {
        let pixels = vec![12u8; 8];
        let glyph = encode_glyph(&pixels).unwrap();
        let mut data = vec![0u8; 32];
        data[..4].copy_from_slice(b"YOX\0");
        data[4..8].copy_from_slice(&0x0100_0000u32.to_le_bytes());
        data[8..12].copy_from_slice(&0x3du32.to_le_bytes());
        data[12..16].copy_from_slice(&(glyph.len() as u32).to_le_bytes());
        data[16..20].copy_from_slice(&((32 + glyph.len()) as u32).to_le_bytes());
        data[20..24].copy_from_slice(&(2u32 | (4u32 << 16)).to_le_bytes());
        let count = 9589usize;
        data[24..28].copy_from_slice(&(count as u32).to_le_bytes());
        data.extend_from_slice(&glyph);
        let index_off = data.len();
        for _ in 0..count {
            data.extend_from_slice(&0u32.to_le_bytes());
            data.extend_from_slice(&(glyph.len() as u32).to_le_bytes());
        }
        data.extend_from_slice(&[0x55; 128]);
        let width_off = data.len();
        data.extend_from_slice(&[0; 8 * 9589]);
        data[28..32].copy_from_slice(&(width_off as u32).to_le_bytes());
        let font = YoxFont::parse(data).unwrap();
        assert_eq!(font.width_table().len(), 8 * 9589);
        let rebuilt = font.rebuild(&[], &[(0, 9)]).unwrap();
        let again = YoxFont::parse(rebuilt).unwrap();
        assert_eq!(again.width_table()[..2], 9u16.to_le_bytes());
        assert_eq!(again.width_gap, vec![0x55; 128]);
        assert_eq!(again.index[0], (0, glyph.len()));
        assert!(index_off > 0);
    }
}
