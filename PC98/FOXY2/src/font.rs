use encoding_rs::SHIFT_JIS;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::mes::FontPatchRequest;

const BMP_FILE_SIZE: usize = 0x8003E;
const BMP_PIXEL_OFFSET: usize = 0x3E;
const BMP_WIDTH: usize = 2048;
const BMP_HEIGHT: usize = 2048;
const BMP_STRIDE: usize = 256;
const GLYPH_WIDTH: usize = 16;
const GLYPH_HEIGHT: usize = 16;
const GLYPH_BYTES_PER_ROW: usize = 2;
const GLYPH_BYTES: usize = GLYPH_HEIGHT * GLYPH_BYTES_PER_ROW;

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BmpLayout {
    pixel_offset: usize,
    stride: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FontSlot {
    page: u8,
    cell: u8,
    x: usize,
    y: usize,
}

pub(crate) fn redraw_font(
    font_source: &Path,
    requests: &[FontPatchRequest],
    face: &str,
) -> Result<Vec<u8>> {
    let source = fs::read(font_source)
        .map_err(|error| format!("failed to read {}: {error}", font_source.display()))?;
    let layout = validate_font_tmp(&source)?;
    if requests.is_empty() {
        return Ok(source);
    }
    if face.is_empty() {
        return Err("font face must not be empty when redrawing glyphs".to_string());
    }

    let mut slots = Vec::with_capacity(requests.len());
    let mut seen = HashMap::with_capacity(requests.len());
    for request in requests {
        let slot = slot_for_carrier(request.carrier)?;
        if let Some(previous) = seen.insert(slot, request.replacement) {
            if previous != request.replacement {
                return Err(format!(
                    "carrier {} resolves to a slot requested for both {} and {}",
                    request.carrier, previous, request.replacement
                ));
            }
        }
        slots.push((slot, request.replacement));
    }

    let mut output = source.clone();
    let mut allowed = vec![false; output.len()];
    for (slot, replacement) in &slots {
        let glyph = render_glyph(*replacement, face)?;
        write_slot(&mut output, layout, *slot, &glyph)?;
        mark_slot_bytes(&mut allowed, layout, *slot)?;
        let actual = read_slot(&output, layout, *slot)?;
        if actual != glyph {
            return Err(format!(
                "font slot page {:#x} cell {:#x} failed pixel-exact readback",
                slot.page, slot.cell
            ));
        }
    }
    let _ = validate_font_tmp(&output)?;
    for (index, (before, after)) in source.iter().zip(&output).enumerate() {
        if before != after && !allowed[index] {
            return Err(format!(
                "font redraw changed byte {index:#x} outside requested 16x16 slots"
            ));
        }
    }
    Ok(output)
}

fn validate_font_tmp(bytes: &[u8]) -> Result<BmpLayout> {
    if bytes.len() != BMP_FILE_SIZE {
        return Err(format!(
            "font.tmp must be {BMP_FILE_SIZE} bytes, got {}",
            bytes.len()
        ));
    }
    if &bytes[0..2] != b"BM" {
        return Err("font.tmp is not a BMP file".to_string());
    }
    // The supplied NP2 font leaves the optional BMP file-size field as zero;
    // preserve it rather than requiring a conventional header value.
    if read_u32(bytes, 10)? as usize != BMP_PIXEL_OFFSET {
        return Err("font.tmp BMP pixel offset is not 0x3e".to_string());
    }
    if read_u32(bytes, 14)? != 40 {
        return Err("font.tmp does not use a 40-byte BITMAPINFOHEADER".to_string());
    }
    if read_i32(bytes, 18)? != BMP_WIDTH as i32 || read_i32(bytes, 22)? != BMP_HEIGHT as i32 {
        return Err("font.tmp must be a 2048x2048 bottom-up bitmap".to_string());
    }
    if read_u16(bytes, 26)? != 1 || read_u16(bytes, 28)? != 1 {
        return Err("font.tmp must be a 1-plane 1bpp bitmap".to_string());
    }
    if read_u32(bytes, 30)? != 0 {
        return Err("font.tmp must use uncompressed BI_RGB pixels".to_string());
    }
    if read_u32(bytes, 34)? as usize != BMP_STRIDE * BMP_HEIGHT {
        return Err("font.tmp BMP image size is not 0x80000".to_string());
    }
    if bytes[54..58] != [0, 0, 0, 0] || bytes[58..62] != [255, 255, 255, 0] {
        return Err("font.tmp must use black and white palette entries".to_string());
    }
    if BMP_PIXEL_OFFSET + BMP_STRIDE * BMP_HEIGHT != bytes.len() {
        return Err("font.tmp pixel payload has an unexpected size".to_string());
    }
    Ok(BmpLayout {
        pixel_offset: BMP_PIXEL_OFFSET,
        stride: BMP_STRIDE,
    })
}

fn slot_for_carrier(carrier: char) -> Result<FontSlot> {
    let encoded = carrier.to_string();
    let (bytes, _, had_errors) = SHIFT_JIS.encode(&encoded);
    if had_errors || bytes.len() != 2 {
        return Err(format!(
            "carrier {carrier} is not a two-byte CP932 character"
        ));
    }
    let lead = bytes[0];
    let trail = bytes[1];
    if !((0x81..=0x9F).contains(&lead) || (0xE0..=0xEF).contains(&lead))
        || !((0x40..=0x7E).contains(&trail) || (0x80..=0xFC).contains(&trail))
        || trail == 0x7F
    {
        return Err(format!(
            "carrier {carrier} encoded to invalid CP932 pair {lead:02X} {trail:02X}"
        ));
    }
    let row_base = if lead <= 0x9F {
        lead - 0x81
    } else {
        lead - 0xC1
    };
    let row = row_base * 2 + 0x21 + u8::from(trail >= 0x9F);
    let cell = if trail >= 0x9F {
        trail - 0x7E
    } else if trail < 0x7F {
        trail - 0x1F
    } else {
        trail - 0x20
    };
    let page = row
        .checked_sub(0x20)
        .ok_or_else(|| format!("carrier {carrier} maps before JIS row 0x21"))?;
    let page_loaded = (0x01..=0x55).contains(&page) || (0x58..=0x5F).contains(&page);
    if !page_loaded || !(0x01..=0x7F).contains(&cell) {
        return Err(format!(
            "carrier {carrier} maps to unsupported NP2 JIS page {page:#x}, cell {cell:#x}"
        ));
    }
    let x = usize::from(page) * GLYPH_WIDTH;
    let y = usize::from(cell) * GLYPH_HEIGHT;
    if x + GLYPH_WIDTH > BMP_WIDTH || y + GLYPH_HEIGHT > BMP_HEIGHT {
        return Err(format!(
            "carrier {carrier} maps outside the font.tmp atlas at ({x},{y})"
        ));
    }
    Ok(FontSlot { page, cell, x, y })
}

pub(crate) fn has_loaded_np2_slot(carrier: char) -> bool {
    slot_for_carrier(carrier).is_ok()
}

fn write_slot(
    bytes: &mut [u8],
    layout: BmpLayout,
    slot: FontSlot,
    glyph: &[u8; GLYPH_BYTES],
) -> Result<()> {
    for row in 0..GLYPH_HEIGHT {
        let offset = row_offset(layout, slot, row)?;
        bytes[offset..offset + GLYPH_BYTES_PER_ROW]
            .copy_from_slice(&glyph[row * GLYPH_BYTES_PER_ROW..(row + 1) * GLYPH_BYTES_PER_ROW]);
    }
    Ok(())
}

fn read_slot(bytes: &[u8], layout: BmpLayout, slot: FontSlot) -> Result<[u8; GLYPH_BYTES]> {
    let mut glyph = [0u8; GLYPH_BYTES];
    for row in 0..GLYPH_HEIGHT {
        let offset = row_offset(layout, slot, row)?;
        glyph[row * GLYPH_BYTES_PER_ROW..(row + 1) * GLYPH_BYTES_PER_ROW]
            .copy_from_slice(&bytes[offset..offset + GLYPH_BYTES_PER_ROW]);
    }
    Ok(glyph)
}

fn mark_slot_bytes(allowed: &mut [bool], layout: BmpLayout, slot: FontSlot) -> Result<()> {
    for row in 0..GLYPH_HEIGHT {
        let offset = row_offset(layout, slot, row)?;
        for value in &mut allowed[offset..offset + GLYPH_BYTES_PER_ROW] {
            *value = true;
        }
    }
    Ok(())
}

fn row_offset(layout: BmpLayout, slot: FontSlot, row: usize) -> Result<usize> {
    let atlas_y = slot.y + row;
    let storage_y = BMP_HEIGHT
        .checked_sub(1 + atlas_y)
        .ok_or_else(|| "font slot row is outside the BMP".to_string())?;
    layout
        .pixel_offset
        .checked_add(storage_y * layout.stride)
        .and_then(|offset| offset.checked_add(slot.x / 8))
        .filter(|offset| offset + GLYPH_BYTES_PER_ROW <= BMP_FILE_SIZE)
        .ok_or_else(|| "font slot byte offset overflows the BMP".to_string())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| format!("u16 at {offset:#x} exceeds font.tmp"))?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| format!("u32 at {offset:#x} exceeds font.tmp"))?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn read_i32(bytes: &[u8], offset: usize) -> Result<i32> {
    Ok(read_u32(bytes, offset)? as i32)
}

#[cfg(windows)]
fn render_glyph(replacement: char, face: &str) -> Result<[u8; GLYPH_BYTES]> {
    render_glyph_windows(replacement, face)
}

#[cfg(not(windows))]
fn render_glyph(_replacement: char, _face: &str) -> Result<[u8; GLYPH_BYTES]> {
    Err("font redraw requires Windows GDI; this build cannot render font.tmp glyphs".to_string())
}

#[cfg(windows)]
mod windows_gdi {
    use std::ffi::c_void;

    pub type Hdc = *mut c_void;
    pub type Hfont = *mut c_void;
    pub type Hbitmap = *mut c_void;
    pub type Hgdiobj = *mut c_void;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct BitmapInfoHeader {
        pub size: u32,
        pub width: i32,
        pub height: i32,
        pub planes: u16,
        pub bit_count: u16,
        pub compression: u32,
        pub size_image: u32,
        pub x_pels_per_meter: i32,
        pub y_pels_per_meter: i32,
        pub clr_used: u32,
        pub clr_important: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct RgbQuad {
        pub blue: u8,
        pub green: u8,
        pub red: u8,
        pub reserved: u8,
    }

    #[repr(C)]
    pub struct BitmapInfo {
        pub header: BitmapInfoHeader,
        pub colors: [RgbQuad; 2],
    }

    pub const DIB_RGB_COLORS: u32 = 0;
    pub const DEFAULT_CHARSET: u32 = 1;
    pub const OUT_DEFAULT_PRECIS: u32 = 0;
    pub const CLIP_DEFAULT_PRECIS: u32 = 0;
    pub const NONANTIALIASED_QUALITY: u32 = 3;
    pub const FIXED_PITCH: u32 = 1;
    pub const FW_NORMAL: i32 = 400;
    pub const OPAQUE: i32 = 2;
    pub const CLR_INVALID: u32 = 0xFFFF_FFFF;

    #[link(name = "gdi32")]
    extern "system" {
        pub fn CreateCompatibleDC(hdc: Hdc) -> Hdc;
        pub fn DeleteDC(hdc: Hdc) -> i32;
        pub fn CreateDIBSection(
            hdc: Hdc,
            bitmap_info: *const BitmapInfo,
            usage: u32,
            bits: *mut *mut c_void,
            section: *mut c_void,
            offset: u32,
        ) -> Hbitmap;
        pub fn DeleteObject(object: Hgdiobj) -> i32;
        pub fn SelectObject(hdc: Hdc, object: Hgdiobj) -> Hgdiobj;
        pub fn CreateFontW(
            height: i32,
            width: i32,
            escapement: i32,
            orientation: i32,
            weight: i32,
            italic: u32,
            underline: u32,
            strike_out: u32,
            charset: u32,
            output_precision: u32,
            clip_precision: u32,
            quality: u32,
            pitch_and_family: u32,
            face: *const u16,
        ) -> Hfont;
        pub fn SetBkColor(hdc: Hdc, color: u32) -> u32;
        pub fn SetTextColor(hdc: Hdc, color: u32) -> u32;
        pub fn SetBkMode(hdc: Hdc, mode: i32) -> i32;
        pub fn TextOutW(hdc: Hdc, x: i32, y: i32, text: *const u16, count: i32) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn GetLastError() -> u32;
    }
}

#[cfg(windows)]
fn render_glyph_windows(replacement: char, face: &str) -> Result<[u8; GLYPH_BYTES]> {
    use std::ffi::c_void;
    use std::ptr;
    use windows_gdi::*;

    if replacement as u32 > 0xFFFF || replacement.is_control() {
        return Err(format!(
            "replacement character U+{:04X} cannot be rendered as one BMP glyph",
            replacement as u32
        ));
    }
    let mut face_w: Vec<u16> = face.encode_utf16().collect();
    face_w.push(0);
    let text_w = [replacement as u16];
    let bitmap_info = BitmapInfo {
        header: BitmapInfoHeader {
            size: 40,
            width: GLYPH_WIDTH as i32,
            height: -(GLYPH_HEIGHT as i32),
            planes: 1,
            bit_count: 1,
            compression: 0,
            size_image: (GLYPH_HEIGHT * 4) as u32,
            x_pels_per_meter: 0,
            y_pels_per_meter: 0,
            clr_used: 2,
            clr_important: 2,
        },
        colors: [
            RgbQuad {
                blue: 0,
                green: 0,
                red: 0,
                reserved: 0,
            },
            RgbQuad {
                blue: 255,
                green: 255,
                red: 255,
                reserved: 0,
            },
        ],
    };

    let mut dc: Hdc = ptr::null_mut();
    let mut bitmap: Hbitmap = ptr::null_mut();
    let mut font: Hfont = ptr::null_mut();
    let mut old_bitmap: Hgdiobj = ptr::null_mut();
    let mut old_font: Hgdiobj = ptr::null_mut();
    let result = (|| unsafe {
        dc = CreateCompatibleDC(ptr::null_mut());
        if dc.is_null() {
            return Err(gdi_error("CreateCompatibleDC"));
        }
        let mut bits: *mut c_void = ptr::null_mut();
        bitmap = CreateDIBSection(
            dc,
            &bitmap_info,
            DIB_RGB_COLORS,
            &mut bits,
            ptr::null_mut(),
            0,
        );
        if bitmap.is_null() || bits.is_null() {
            return Err(gdi_error("CreateDIBSection"));
        }
        std::slice::from_raw_parts_mut(bits as *mut u8, GLYPH_HEIGHT * 4).fill(0xFF);
        font = CreateFontW(
            GLYPH_HEIGHT as i32,
            0,
            0,
            0,
            FW_NORMAL,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            NONANTIALIASED_QUALITY,
            FIXED_PITCH,
            face_w.as_ptr(),
        );
        if font.is_null() {
            return Err(gdi_error("CreateFontW"));
        }
        old_bitmap = SelectObject(dc, bitmap as Hgdiobj);
        if old_bitmap.is_null() {
            return Err(gdi_error("SelectObject(bitmap)"));
        }
        old_font = SelectObject(dc, font as Hgdiobj);
        if old_font.is_null() {
            return Err(gdi_error("SelectObject(font)"));
        }
        if SetBkMode(dc, OPAQUE) == 0 {
            return Err(gdi_error("SetBkMode"));
        }
        if SetBkColor(dc, 0x00FF_FFFF) == CLR_INVALID {
            return Err(gdi_error("SetBkColor"));
        }
        if SetTextColor(dc, 0x0000_0000) == CLR_INVALID {
            return Err(gdi_error("SetTextColor"));
        }
        if TextOutW(dc, 0, 0, text_w.as_ptr(), 1) == 0 {
            return Err(gdi_error("TextOutW"));
        }
        let raw = std::slice::from_raw_parts(bits as *const u8, GLYPH_HEIGHT * 4);
        let mut glyph = [0u8; GLYPH_BYTES];
        for row in 0..GLYPH_HEIGHT {
            glyph[row * 2..row * 2 + 2].copy_from_slice(&raw[row * 4..row * 4 + 2]);
        }
        Ok(glyph)
    })();
    unsafe {
        if !old_font.is_null() {
            let _ = SelectObject(dc, old_font);
        }
        if !old_bitmap.is_null() {
            let _ = SelectObject(dc, old_bitmap);
        }
        if !font.is_null() {
            let _ = DeleteObject(font as Hgdiobj);
        }
        if !bitmap.is_null() {
            let _ = DeleteObject(bitmap as Hgdiobj);
        }
        if !dc.is_null() {
            let _ = DeleteDC(dc);
        }
    }
    result
}

#[cfg(windows)]
fn gdi_error(operation: &str) -> String {
    let code = unsafe { windows_gdi::GetLastError() };
    format!("{operation} failed (GetLastError={code})")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jis_slot_matches_verified_rin_coordinates() {
        let slot = slot_for_carrier('凛').expect("凛 should be CP932 encodable");
        assert_eq!(slot.page, 0x31);
        assert_eq!(slot.cell, 0x5B);
        assert_eq!((slot.x, slot.y), (784, 1456));
    }

    #[test]
    fn extended_cp932_carrier_uses_second_jis_row_branch() {
        let slot = slot_for_carrier('凜').expect("凜 should be CP932 encodable");
        assert_eq!(slot.page, 0x54);
        assert_eq!(slot.cell, 0x25);
        assert_eq!((slot.x, slot.y), (1344, 592));
    }

    #[test]
    fn rejects_cp932_extension_outside_loaded_np2_pages() {
        let error = slot_for_carrier('羡').expect_err("羡 must not use an unloaded NP2 page");
        assert!(error.contains("invalid CP932 pair"));
        assert!(!has_loaded_np2_slot('羡'));
        assert!(has_loaded_np2_slot('羨'));
    }

    #[test]
    fn rejects_single_byte_carrier() {
        let error = slot_for_carrier('A').expect_err("ASCII must not select a double-byte slot");
        assert!(error.contains("two-byte CP932"));
    }

    #[test]
    fn validates_profile_geometry() {
        let mut bytes = vec![0u8; BMP_FILE_SIZE];
        bytes[0..2].copy_from_slice(b"BM");
        bytes[2..6].copy_from_slice(&(BMP_FILE_SIZE as u32).to_le_bytes());
        bytes[10..14].copy_from_slice(&(BMP_PIXEL_OFFSET as u32).to_le_bytes());
        bytes[14..18].copy_from_slice(&40u32.to_le_bytes());
        bytes[18..22].copy_from_slice(&(BMP_WIDTH as i32).to_le_bytes());
        bytes[22..26].copy_from_slice(&(BMP_HEIGHT as i32).to_le_bytes());
        bytes[26..28].copy_from_slice(&1u16.to_le_bytes());
        bytes[28..30].copy_from_slice(&1u16.to_le_bytes());
        bytes[30..34].copy_from_slice(&0u32.to_le_bytes());
        bytes[34..38].copy_from_slice(&((BMP_STRIDE * BMP_HEIGHT) as u32).to_le_bytes());
        bytes[54..58].copy_from_slice(&[0, 0, 0, 0]);
        bytes[58..62].copy_from_slice(&[255, 255, 255, 0]);
        assert!(validate_font_tmp(&bytes).is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn redraws_a_real_gdi_glyph_without_out_of_slot_changes() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut bytes = vec![0xFFu8; BMP_FILE_SIZE];
        bytes[0..2].copy_from_slice(b"BM");
        bytes[2..6].copy_from_slice(&(BMP_FILE_SIZE as u32).to_le_bytes());
        bytes[10..14].copy_from_slice(&(BMP_PIXEL_OFFSET as u32).to_le_bytes());
        bytes[14..18].copy_from_slice(&40u32.to_le_bytes());
        bytes[18..22].copy_from_slice(&(BMP_WIDTH as i32).to_le_bytes());
        bytes[22..26].copy_from_slice(&(BMP_HEIGHT as i32).to_le_bytes());
        bytes[26..28].copy_from_slice(&1u16.to_le_bytes());
        bytes[28..30].copy_from_slice(&1u16.to_le_bytes());
        bytes[30..34].copy_from_slice(&0u32.to_le_bytes());
        bytes[34..38].copy_from_slice(&((BMP_STRIDE * BMP_HEIGHT) as u32).to_le_bytes());
        bytes[54..58].copy_from_slice(&[0, 0, 0, 0]);
        bytes[58..62].copy_from_slice(&[255, 255, 255, 0]);
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("foxy2-font-test-{suffix}.tmp"));
        fs::write(&path, &bytes).expect("write temporary font");
        let result = redraw_font(
            &path,
            &[FontPatchRequest {
                carrier: '凛',
                replacement: '你',
            }],
            "新宋体",
        );
        let _ = fs::remove_file(&path);
        let output = result.expect("Windows GDI should render 新宋体");
        assert!(output
            .iter()
            .zip(&bytes)
            .enumerate()
            .any(|(index, (after, before))| after != before && index >= BMP_PIXEL_OFFSET));
        let slot = slot_for_carrier('凛').expect("凛 slot");
        let glyph = read_slot(
            &output,
            validate_font_tmp(&output).expect("output geometry"),
            slot,
        )
        .expect("readback slot");
        assert!(glyph.iter().any(|byte| *byte != 0xFF));
    }
}
