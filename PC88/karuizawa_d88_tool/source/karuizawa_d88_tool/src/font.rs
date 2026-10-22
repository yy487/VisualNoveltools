use crate::n88::N88Disk;
use crate::text_compression::TextDictionary;
use crate::Result;
use serde::Serialize;

const M2_START: usize = 0xD990;
const GLYPH_TABLE: usize = 0xDB50;
const GLYPH_SLOT_BYTES: usize = 18;
const GLYPH_SLOTS: usize = 133;
const M3_START: usize = 0xBD00;
const M3_RENDER_ENTRY: usize = 0xBD93;

#[derive(Debug, Clone, Serialize)]
pub struct FontPatchReport {
    pub font_bank: String,
    pub ascii_first: String,
    pub ascii_last: String,
    pub direct_code_first: String,
    pub direct_code_last: String,
    pub glyph_slots: usize,
    pub glyph_slot_bytes: usize,
    pub m3_patches: Vec<String>,
}

pub struct FontPatches {
    pub m2data: Vec<u8>,
    pub m3data: Vec<u8>,
    pub m3dict: Option<Vec<u8>>,
    pub report: FontPatchReport,
    pub decoder_address: Option<u16>,
}

pub fn build_font_patches(
    disk: &N88Disk,
    font_rom: &[u8],
    dictionary: &TextDictionary,
    large_font: bool,
) -> Result<FontPatches> {
    if font_rom.len() < 0x800 {
        return Err(format!(
            "FONT.ROM 只有 {} 字节，缺少 0x800 字节 8×8 字库区",
            font_rom.len()
        ));
    }
    let m2 = patch_m2data(&disk.file("M2DATA")?.data, font_rom, large_font)?;
    let (m3dict, decoder_address) = patch_m3dict(dictionary)?;
    let m3 = patch_m3data(&disk.file("M3DATA")?.data, decoder_address, large_font)?;
    Ok(FontPatches {
        m2data: m2,
        m3data: m3,
        m3dict,
        report: FontPatchReport {
            font_bank: if large_font { "FONT.ROM 8x8 scaled to 16x12" } else { "FONT.ROM 0x0000..0x07FF, 256 x 8-byte 8x8" }.into(),
            ascii_first: "0x21 (!)".into(),
            ascii_last: "0x7E (~)".into(),
            direct_code_first: "0x86".into(),
            direct_code_last: "0xE3".into(),
            glyph_slots: if large_font { 94 } else { GLYPH_SLOTS },
            glyph_slot_bytes: if large_font { 24 } else { GLYPH_SLOT_BYTES },
            m3_patches: vec![
                if large_font { "16x12 glyphs: 24-byte stride, 2-byte advance, 12 rows; native digits/punctuation alias ASCII slots" } else { "BD53: second INC HL -> NOP; BE87: bytes per glyph row 2 -> 1" }.into(),
                "BDA6: high-byte glyphs always use the direct ASCII bank".into(),
                decoder_address.map_or_else(
                    || "text dictionary decoder not installed".into(),
                    |address| format!("BD93: shared renderer entry -> {address:04X}h dictionary decoder (USR and BE65 paths)"),
                ),
            ],
        },
        decoder_address,
    })
}

fn patch_m2data(source: &[u8], font_rom: &[u8], large_font: bool) -> Result<Vec<u8>> {
    require_load_header(source, M2_START, "M2DATA")?;
    let table_offset = 4 + GLYPH_TABLE - M2_START;
    let table_end = table_offset + GLYPH_SLOTS * GLYPH_SLOT_BYTES;
    if table_end > source.len() {
        return Err("M2DATA 的原始 133×18 字形区越界".into());
    }
    let mut output = source.to_vec();
    output[table_offset..table_end].fill(0);

    if large_font {
        for ascii in b'!'..=b'~' {
            let start = table_offset + usize::from(ascii - b'!') * 24;
            output[start..start + 24].copy_from_slice(&enlarge_glyph(glyph(font_rom, ascii)?));
        }
        return Ok(output);
    }

    // Every printable non-space ASCII byte maps linearly to a direct game code:
    // game_code = 86h + (ascii - 21h).  Spaces remain the native 20h advance.
    for ascii in b'!'..=b'~' {
        let game_code = 0x86u8 + (ascii - b'!');
        let slot = usize::from(game_code - 0x86);
        write_glyph(&mut output, table_offset, slot, glyph(font_rom, ascii)?)?;
    }

    // Keep native low-byte punctuation/digit controls usable for unedited engine
    // fragments.  FEh/FFh resolve to slots 120/121; 0..9 wrap to 122..131.
    write_glyph(&mut output, table_offset, 120, glyph(font_rom, b'!')?)?;
    write_glyph(&mut output, table_offset, 121, glyph(font_rom, b'?')?)?;
    for digit in b'0'..=b'9' {
        write_glyph(
            &mut output,
            table_offset,
            122 + usize::from(digit - b'0'),
            glyph(font_rom, digit)?,
        )?;
    }
    Ok(output)
}

fn patch_m3data(source: &[u8], decoder_address: Option<u16>, large_font: bool) -> Result<Vec<u8>> {
    require_load_header(source, M3_START, "M3DATA")?;
    let mut output = source.to_vec();
    if large_font {
        patch_byte(&mut output, 0xBD3C, 0x12, 0x18, "M3DATA 24 字节字形步长")?;
        patch_byte(&mut output, 0xBE86, 0x09, 0x0C, "M3DATA 12 行字形高度")?;
        patch_byte(&mut output, 0xBDB9, 0xD0, 0x65, "M3DATA 数字字形别名")?;
        patch_byte(&mut output, 0xBE31, 0xFE, 0x86, "M3DATA 感叹号字形别名")?;
        patch_byte(&mut output, 0xBE3B, 0xFF, 0xA4, "M3DATA 问号字形别名")?;
    } else {
        patch_byte(&mut output, 0xBD53, 0x23, 0x00, "M3DATA 横向步进")?;
        patch_byte(&mut output, 0xBE87, 0x02, 0x01, "M3DATA 行字节数")?;
    }
    patch_byte(&mut output, 0xBDA6, 0x64, 0x5C, "M3DATA ASCII 直接字形模式")?;
    let entry_offset = 4 + M3_RENDER_ENTRY - M3_START;
    let entry = output
        .get_mut(entry_offset..entry_offset + 5)
        .ok_or("M3DATA 文本解码入口地址越界")?;
    if entry != [0x3E, 0x00, 0x32, 0x8F, 0xBE] {
        return Err(format!(
            "M3DATA BD93h 为 {:02X?}，预期 LD A,0 / LD (BE8Fh),A",
            entry
        ));
    }
    if let Some(decoder_address) = decoder_address {
        let [decoder_low, decoder_high] = decoder_address.to_le_bytes();
        entry.copy_from_slice(&[0xC3, decoder_low, decoder_high, 0x00, 0x00]);
    }
    Ok(output)
}

const M3DICT_LOAD_ADDRESS: u16 = 0xF200;

fn patch_m3dict(dictionary: &TextDictionary) -> Result<(Option<Vec<u8>>, Option<u16>)> {
    if dictionary.is_empty() {
        return Ok((None, None));
    }
    let runtime = dictionary.runtime_patch(M3DICT_LOAD_ADDRESS)?;
    if runtime.is_empty() {
        return Ok((None, None));
    }
    let end = M3DICT_LOAD_ADDRESS
        .checked_add(u16::try_from(runtime.len() - 1).map_err(|_| "M3DICT 字典长度溢出")?)
        .ok_or("M3DICT 字典地址溢出")?;
    if end >= 0xF800 {
        return Err(format!("M3DICT 字典补丁结束于 {end:04X}h，超出保留区"));
    }
    let mut output = Vec::with_capacity(runtime.len() + 4);
    output.extend_from_slice(&M3DICT_LOAD_ADDRESS.to_le_bytes());
    output.extend_from_slice(&end.to_le_bytes());
    output.extend_from_slice(&runtime);
    Ok((Some(output), Some(M3DICT_LOAD_ADDRESS)))
}

fn enlarge_glyph(source: &[u8]) -> [u8; 24] {
    let mut output = [0; 24];
    for row in 0..12 {
        let mut pixels = 0u16;
        for column in 0..8 {
            if source[row * 8 / 12] & (0x80 >> column) != 0 {
                pixels |= 3 << (14 - column * 2);
            }
        }
        output[row * 2..row * 2 + 2].copy_from_slice(&pixels.to_be_bytes());
    }
    output
}

fn patch_byte(
    output: &mut [u8],
    address: usize,
    expected: u8,
    replacement: u8,
    label: &str,
) -> Result<()> {
    let offset = 4 + address - M3_START;
    let actual = output
        .get_mut(offset)
        .ok_or_else(|| format!("{label} 地址 {address:04X}h 越界"))?;
    if *actual != expected {
        return Err(format!(
            "{label} 地址 {address:04X}h 为 {:02X}，预期 {expected:02X}",
            *actual
        ));
    }
    *actual = replacement;
    Ok(())
}

fn require_load_header(source: &[u8], expected_start: usize, name: &str) -> Result<()> {
    if source.len() < 4 {
        return Err(format!("{name} 小于四字节 BLOAD 头"));
    }
    let start = usize::from(u16::from_le_bytes([source[0], source[1]]));
    if start != expected_start {
        return Err(format!(
            "{name} 的 BLOAD 起始地址为 {start:04X}h，预期 {expected_start:04X}h"
        ));
    }
    Ok(())
}

fn glyph(font_rom: &[u8], ascii: u8) -> Result<&[u8]> {
    let start = usize::from(ascii) * 8;
    let value = font_rom
        .get(start..start + 8)
        .ok_or_else(|| format!("FONT.ROM 缺少 ASCII {ascii:02X}h 字形"))?;
    if !value.iter().any(|byte| *byte != 0) {
        return Err(format!("FONT.ROM 的 ASCII {ascii:02X}h 字形为空"));
    }
    Ok(value)
}

fn write_glyph(output: &mut [u8], table_offset: usize, slot: usize, glyph: &[u8]) -> Result<()> {
    if slot >= GLYPH_SLOTS || glyph.len() != 8 {
        return Err(format!("字形槽 {slot} 或字形长度无效"));
    }
    let start = table_offset + slot * GLYPH_SLOT_BYTES;
    let target = output
        .get_mut(start..start + GLYPH_SLOT_BYTES)
        .ok_or_else(|| format!("字形槽 {slot} 越界"))?;
    target[..8].copy_from_slice(glyph);
    target[8..].fill(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enlarged_diagonal_has_two_columns_and_twelve_rows() {
        let glyph = enlarge_glyph(&[0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1]);
        let rows = glyph
            .chunks_exact(2)
            .map(|row| u16::from_be_bytes([row[0], row[1]]))
            .collect::<Vec<_>>();
        assert_eq!(
            rows,
            [
                0xC000, 0xC000, 0x3000, 0x0C00, 0x0C00, 0x0300, 0x00C0, 0x00C0, 0x0030, 0x000C,
                0x000C, 0x0003
            ]
        );
    }
}
