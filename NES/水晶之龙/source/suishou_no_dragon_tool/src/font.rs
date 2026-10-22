use crate::{game::Game, Result};
use std::collections::BTreeMap;

pub fn export(game: &Game) -> Result<BTreeMap<String, Vec<u8>>> {
    let f = game.disk.file("CRAM_BG")?;
    Ok(BTreeMap::from([(
        "fonts/original_bg_chr.bin".into(),
        game.disk.payload(f).to_vec(),
    )]))
}

#[cfg(windows)]
mod gdi {
    use super::*;
    use std::ffi::c_void;
    type Handle = *mut c_void;
    #[repr(C)]
    struct Info {
        size: u32,
        width: i32,
        height: i32,
        planes: u16,
        bits: u16,
        compression: u32,
        bytes: u32,
        x: i32,
        y: i32,
        colors: u32,
        important: u32,
    }
    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn CreateCompatibleDC(dc: Handle) -> Handle;
        fn DeleteDC(dc: Handle) -> i32;
        fn CreateDIBSection(
            dc: Handle,
            info: *const Info,
            usage: u32,
            bits: *mut *mut c_void,
            section: Handle,
            offset: u32,
        ) -> Handle;
        fn CreateFontW(
            h: i32,
            w: i32,
            e: i32,
            o: i32,
            weight: i32,
            italic: u32,
            underline: u32,
            strike: u32,
            charset: u32,
            out: u32,
            clip: u32,
            quality: u32,
            family: u32,
            face: *const u16,
        ) -> Handle;
        fn SelectObject(dc: Handle, object: Handle) -> Handle;
        fn DeleteObject(object: Handle) -> i32;
        fn SetTextColor(dc: Handle, color: u32) -> u32;
        fn SetBkColor(dc: Handle, color: u32) -> u32;
        fn TextOutW(dc: Handle, x: i32, y: i32, text: *const u16, len: i32) -> i32;
        fn GetGlyphIndicesW(
            dc: Handle,
            text: *const u16,
            count: i32,
            indices: *mut u16,
            flags: u32,
        ) -> u32;
        fn PatBlt(dc: Handle, x: i32, y: i32, w: i32, h: i32, op: u32) -> i32;
        fn GdiFlush() -> i32;
    }
    pub struct Raster {
        dc: Handle,
        bitmap: Handle,
        font: Handle,
        old_bitmap: Handle,
        old_font: Handle,
        bits: *mut c_void,
    }
    impl Raster {
        pub fn new(face: &str) -> Result<Self> {
            if face.is_empty() || face.contains('\0') {
                return Err("字库字体名称非法".into());
            }
            let face = face.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
            unsafe {
                let dc = CreateCompatibleDC(std::ptr::null_mut());
                if dc.is_null() {
                    return Err("CreateCompatibleDC 失败".into());
                }
                let info = Info {
                    size: 40,
                    width: 16,
                    height: -16,
                    planes: 1,
                    bits: 32,
                    compression: 0,
                    bytes: 1024,
                    x: 0,
                    y: 0,
                    colors: 0,
                    important: 0,
                };
                let mut bits = std::ptr::null_mut();
                let bitmap = CreateDIBSection(dc, &info, 0, &mut bits, std::ptr::null_mut(), 0);
                let font = CreateFontW(-16, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 3, 0, face.as_ptr());
                if bitmap.is_null() || font.is_null() || bits.is_null() {
                    if !bitmap.is_null() {
                        DeleteObject(bitmap);
                    }
                    if !font.is_null() {
                        DeleteObject(font);
                    }
                    DeleteDC(dc);
                    return Err("创建 16×16 字模画布失败".into());
                }
                let old_bitmap = SelectObject(dc, bitmap);
                let old_font = SelectObject(dc, font);
                SetTextColor(dc, 0xFFFFFF);
                SetBkColor(dc, 0);
                Ok(Self {
                    dc,
                    bitmap,
                    font,
                    old_bitmap,
                    old_font,
                    bits,
                })
            }
        }
        pub fn glyph(&self, ch: char) -> Result<[u8; 32]> {
            if ch == ' ' {
                return Ok([0; 32]);
            }
            if ch as u32 > 0xFFFF {
                return Err(format!("16×16 字库当前只支持 BMP 字符: {ch}").into());
            }
            let c = ch as u16;
            let mut index = 0;
            unsafe {
                if GetGlyphIndicesW(self.dc, &c, 1, &mut index, 1) == u32::MAX || index == 0xFFFF {
                    return Err(format!("所选字体缺少字符: {ch}").into());
                }
                if PatBlt(self.dc, 0, 0, 16, 16, 0x42) == 0
                    || TextOutW(self.dc, 0, 0, &c, 1) == 0
                    || GdiFlush() == 0
                {
                    return Err("GDI 字模绘制失败".into());
                }
                let pixels = std::slice::from_raw_parts(self.bits as *const u32, 256);
                let mut result = [0; 32];
                for y in 0..16 {
                    for x in 0..16 {
                        if pixels[y * 16 + x] & 0xFFFFFF != 0 {
                            result[y * 2 + x / 8] |= 0x80 >> (x % 8);
                        }
                    }
                }
                if result == [0; 32] {
                    return Err(format!("字符被裁成空白: {ch}").into());
                }
                Ok(result)
            }
        }
    }
    impl Drop for Raster {
        fn drop(&mut self) {
            unsafe {
                SelectObject(self.dc, self.old_font);
                SelectObject(self.dc, self.old_bitmap);
                DeleteObject(self.font);
                DeleteObject(self.bitmap);
                DeleteDC(self.dc);
            }
        }
    }
}

#[cfg(windows)]
pub use gdi::Raster;

#[cfg(not(windows))]
pub struct Raster;
#[cfg(not(windows))]
impl Raster {
    pub fn new(_: &str) -> Result<Self> {
        Err("新增字模需要 Windows GDI 字体后端".into())
    }
    pub fn glyph(&self, _: char) -> Result<[u8; 32]> {
        Err("新增字模需要 Windows GDI 字体后端".into())
    }
}

pub fn atlas(glyphs: &BTreeMap<char, [u8; 32]>) -> Result<Vec<u8>> {
    let width = 16 * 16;
    let height = glyphs.len().max(1).div_ceil(16) * 16;
    let mut pixels = vec![0; width * height];
    for (i, bits) in glyphs.values().enumerate() {
        for y in 0..16 {
            for x in 0..16 {
                pixels[(i / 16 * 16 + y) * width + i % 16 * 16 + x] =
                    if bits[y * 2 + x / 8] & (0x80 >> (x % 8)) != 0 {
                        255
                    } else {
                        0
                    };
            }
        }
    }
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, width as u32, height as u32);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header()?.write_image_data(&pixels)?;
    }
    Ok(bytes)
}
