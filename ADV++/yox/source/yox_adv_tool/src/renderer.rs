//! Windows GDI renderer bound to the installed 新宋体 (NSimSun) face.

use crate::carrier::cp932_decode;
use crate::font::YoxFont;
use std::ffi::c_void;
use std::ptr;

pub struct RedrawnFont {
    pub glyphs: Vec<(usize, Vec<u8>)>,
    pub advances: Vec<(usize, u16)>,
    pub face: String,
    pub fallbacks: Vec<FallbackRecord>,
}

pub struct RenderedSamples {
    pub face: String,
    pub glyphs: Vec<Vec<u8>>,
    pub advances: Vec<u16>,
}

#[derive(serde::Serialize)]
pub struct FallbackRecord {
    pub slot: usize,
    pub cp932: String,
    pub character: String,
    pub requested_face: String,
    pub actual_face: String,
}

#[cfg(windows)]
mod gdi {
    use super::*;
    use std::collections::BTreeMap;
    type Handle = *mut c_void;
    #[repr(C)]
    struct BitmapInfoHeader {
        size: u32,
        width: i32,
        height: i32,
        planes: u16,
        bit_count: u16,
        compression: u32,
        image_size: u32,
        xppm: i32,
        yppm: i32,
        colors_used: u32,
        important: u32,
    }
    #[repr(C)]
    struct RgbQuad {
        blue: u8,
        green: u8,
        red: u8,
        reserved: u8,
    }
    #[repr(C)]
    struct BitmapInfo {
        header: BitmapInfoHeader,
        colors: [RgbQuad; 1],
    }
    #[repr(C)]
    struct Size {
        cx: i32,
        cy: i32,
    }
    #[repr(C)]
    struct TextMetricW {
        height: i32,
        ascent: i32,
        descent: i32,
        internal_leading: i32,
        external_leading: i32,
        average_width: i32,
        max_width: i32,
        weight: i32,
        overhang: i32,
        aspect_x: i32,
        aspect_y: i32,
        first_char: u16,
        last_char: u16,
        default_char: u16,
        break_char: u16,
        italic: u8,
        underlined: u8,
        struck_out: u8,
        pitch_and_family: u8,
        charset: u8,
    }

    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn CreateCompatibleDC(dc: Handle) -> Handle;
        fn DeleteDC(dc: Handle) -> i32;
        fn CreateDIBSection(
            dc: Handle,
            info: *const BitmapInfo,
            usage: u32,
            bits: *mut *mut c_void,
            section: Handle,
            offset: u32,
        ) -> Handle;
        fn CreateFontW(
            height: i32,
            width: i32,
            escapement: i32,
            orientation: i32,
            weight: i32,
            italic: u32,
            underline: u32,
            strike: u32,
            charset: u32,
            output_precision: u32,
            clip_precision: u32,
            quality: u32,
            pitch: u32,
            face: *const u16,
        ) -> Handle;
        fn SelectObject(dc: Handle, object: Handle) -> Handle;
        fn SetTextAlign(dc: Handle, align: u32) -> u32;
        fn DeleteObject(object: Handle) -> i32;
        fn SetTextColor(dc: Handle, color: u32) -> u32;
        fn SetBkMode(dc: Handle, mode: i32) -> i32;
        fn TextOutW(dc: Handle, x: i32, y: i32, text: *const u16, len: i32) -> i32;
        fn GetTextExtentPoint32W(dc: Handle, text: *const u16, len: i32, size: *mut Size) -> i32;
        fn GetTextFaceW(dc: Handle, count: i32, face: *mut u16) -> i32;
        fn GetGlyphIndicesW(
            dc: Handle,
            text: *const u16,
            len: i32,
            glyphs: *mut u16,
            flags: u32,
        ) -> u32;
        fn GetTextMetricsW(dc: Handle, metrics: *mut TextMetricW) -> i32;
        fn GdiFlush() -> i32;
    }

    struct Canvas {
        dc: Handle,
        old_font: Handle,
        old_bitmap: Handle,
        font: Handle,
        bitmap: Handle,
        bits: *mut u32,
        width: usize,
        height: usize,
        face: String,
    }
    impl Drop for Canvas {
        fn drop(&mut self) {
            unsafe {
                SelectObject(self.dc, self.old_bitmap);
                SelectObject(self.dc, self.old_font);
                DeleteObject(self.bitmap);
                DeleteObject(self.font);
                DeleteDC(self.dc);
            }
        }
    }

    impl Canvas {
        fn new(width: usize, height: usize) -> Result<Self, String> {
            Self::new_for_face(width, height, "新宋体")
        }

        fn new_for_face(width: usize, height: usize, requested: &str) -> Result<Self, String> {
            let dc = unsafe { CreateCompatibleDC(ptr::null_mut()) };
            if dc.is_null() {
                return Err("GDI 无法创建内存DC".into());
            }
            let face: Vec<u16> = format!("{requested}\0").encode_utf16().collect();
            let font = unsafe {
                CreateFontW(
                    -(width.min(height) as i32),
                    0,
                    0,
                    0,
                    400,
                    0,
                    0,
                    0,
                    1,
                    4,
                    0,
                    4,
                    0,
                    face.as_ptr(),
                )
            };
            if font.is_null() {
                unsafe {
                    DeleteDC(dc);
                }
                return Err("GDI 无法创建新宋体字体".into());
            }
            let old_font = unsafe { SelectObject(dc, font) };
            let mut selected_face = [0u16; 64];
            let n =
                unsafe { GetTextFaceW(dc, selected_face.len() as i32, selected_face.as_mut_ptr()) };
            let actual = String::from_utf16_lossy(&selected_face[..n.max(0) as usize])
                .trim_end_matches('\0')
                .to_owned();
            let matches_requested =
                actual == requested || (requested == "新宋体" && actual == "NSimSun");
            if requested == "新宋体" && !matches_requested {
                unsafe {
                    SelectObject(dc, old_font);
                    DeleteObject(font);
                    DeleteDC(dc);
                }
                return Err(format!("请求字体 {requested:?} 但GDI选择了 {actual:?}"));
            }
            unsafe {
                SetBkMode(dc, 1);
                SetTextColor(dc, 0x00ff_ffff);
                SetTextAlign(dc, 0x0018);
            }
            let info = BitmapInfo {
                header: BitmapInfoHeader {
                    size: 40,
                    width: width as i32,
                    height: -(height as i32),
                    planes: 1,
                    bit_count: 32,
                    compression: 0,
                    image_size: (width * height * 4) as u32,
                    xppm: 0,
                    yppm: 0,
                    colors_used: 0,
                    important: 0,
                },
                colors: [RgbQuad {
                    blue: 0,
                    green: 0,
                    red: 0,
                    reserved: 0,
                }],
            };
            let mut bits = ptr::null_mut();
            let bitmap = unsafe { CreateDIBSection(dc, &info, 0, &mut bits, ptr::null_mut(), 0) };
            if bitmap.is_null() || bits.is_null() {
                unsafe {
                    SelectObject(dc, old_font);
                    DeleteObject(font);
                    DeleteDC(dc);
                }
                return Err("GDI 无法创建字形画布".into());
            }
            let old_bitmap = unsafe { SelectObject(dc, bitmap) };
            Ok(Self {
                dc,
                old_font,
                old_bitmap,
                font,
                bitmap,
                bits: bits.cast(),
                width,
                height,
                face: actual,
            })
        }
        fn draw(&mut self, ch: char) -> Result<(Vec<u8>, u16), String> {
            unsafe {
                GdiFlush();
            }
            unsafe {
                ptr::write_bytes(self.bits, 0, self.width * self.height);
            }
            let mut unit = [0u16; 2];
            let encoded = ch.encode_utf16(&mut unit);
            let mut glyph = [0u16; 2];
            let absent = unsafe {
                GetGlyphIndicesW(
                    self.dc,
                    encoded.as_ptr(),
                    encoded.len() as i32,
                    glyph.as_mut_ptr(),
                    1,
                )
            };
            if absent == u32::MAX || glyph[0] == 0xffff {
                return Err(format!("新宋体缺少字符 U+{:04X}", ch as u32));
            }
            let mut size = Size { cx: 0, cy: 0 };
            if unsafe {
                GetTextExtentPoint32W(self.dc, encoded.as_ptr(), encoded.len() as i32, &mut size)
            } == 0
            {
                return Err("GDI 无法测量字形".into());
            }
            let punctuation = matches!(ch, '、' | '。' | '，' | '．' | '｡' | '､' | ',' | '.');
            let x = 0;
            let mut metrics = TextMetricW {
                height: 0,
                ascent: 0,
                descent: 0,
                internal_leading: 0,
                external_leading: 0,
                average_width: 0,
                max_width: 0,
                weight: 0,
                overhang: 0,
                aspect_x: 0,
                aspect_y: 0,
                first_char: 0,
                last_char: 0,
                default_char: 0,
                break_char: 0,
                italic: 0,
                underlined: 0,
                struck_out: 0,
                pitch_and_family: 0,
                charset: 0,
            };
            if unsafe { GetTextMetricsW(self.dc, &mut metrics) } == 0 {
                return Err("GDI 无法读取新宋体度量".into());
            }
            let ink_height = metrics.ascent + metrics.descent;
            let top = ((self.height as i32 - ink_height).max(0)) / 2;
            let baseline = top + metrics.ascent;
            if unsafe { TextOutW(self.dc, x, baseline, encoded.as_ptr(), encoded.len() as i32) }
                == 0
            {
                return Err("GDI 无法绘制字形".into());
            }
            unsafe {
                GdiFlush();
            }
            let mut pixels: Vec<u8> =
                unsafe { std::slice::from_raw_parts(self.bits, self.width * self.height) }
                    .iter()
                    .map(|p| ((*p >> 16) & 0xff) as u8)
                    .collect();
            if punctuation {
                let right_edge = if matches!(ch, ',' | '.') {
                    (size.cx.max(1) as usize).min(self.width) - 1
                } else {
                    self.width - 1
                };
                let mut bounds = (self.width, self.height, 0usize, 0usize);
                for y in 0..self.height {
                    for x in 0..self.width {
                        if pixels[y * self.width + x] != 0 {
                            bounds.0 = bounds.0.min(x);
                            bounds.1 = bounds.1.min(y);
                            bounds.2 = bounds.2.max(x);
                            bounds.3 = bounds.3.max(y);
                        }
                    }
                }
                if bounds.0 < self.width {
                    let dx = right_edge as isize - bounds.2 as isize;
                    let dy = self.height - 1 - bounds.3;
                    let mut anchored = vec![0; pixels.len()];
                    for y in bounds.1..=bounds.3 {
                        for x in bounds.0..=bounds.2 {
                            let nx = x as isize + dx;
                            if nx >= 0 && nx <= right_edge as isize {
                                anchored[(y + dy) * self.width + nx as usize] =
                                    pixels[y * self.width + x];
                            }
                        }
                    }
                    pixels = anchored;
                }
            }
            Ok((pixels, size.cx.clamp(0, u16::MAX as i32) as u16))
        }

        fn has_glyph(&self, ch: char) -> bool {
            let mut units = [0u16; 2];
            let encoded = ch.encode_utf16(&mut units);
            let mut glyph = [0u16; 2];
            let result = unsafe {
                GetGlyphIndicesW(
                    self.dc,
                    encoded.as_ptr(),
                    encoded.len() as i32,
                    glyph.as_mut_ptr(),
                    1,
                )
            };
            result != u32::MAX && glyph[0] != 0xffff
        }
    }

    pub fn redraw(font: &YoxFont, carriers: &BTreeMap<char, u16>) -> Result<RedrawnFont, String> {
        let mut canvas = Canvas::new(font.width, font.height)?;
        let fallback_faces = [
            "MS Gothic",
            "Yu Mincho",
            "Meiryo",
            "Microsoft YaHei",
            "SimSun",
        ];
        let mut fallback_canvases = fallback_faces
            .iter()
            .filter_map(|face| Canvas::new_for_face(font.width, font.height, face).ok())
            .collect::<Vec<_>>();
        let translated: BTreeMap<u16, char> =
            carriers.iter().map(|(ch, code)| (*code, *ch)).collect();
        let mut glyphs = Vec::with_capacity(font.glyph_count);
        let mut advances = Vec::with_capacity(font.glyph_count);
        let mut fallbacks = Vec::new();
        for slot in 0..font.glyph_count {
            let Some(code) = crate::carrier::code_for_slot(slot, font.range_mask) else {
                return Err(format!("字形槽 {slot} 无CP932范围码"));
            };
            let Some(ch) = translated
                .get(&code)
                .copied()
                .or_else(|| cp932_decode(code))
            else {
                glyphs.push((slot, vec![0; font.width * font.height]));
                advances.push((slot, 0));
                continue;
            };
            let (pixels, advance) = if canvas.has_glyph(ch) {
                canvas
                    .draw(ch)
                    .map_err(|e| format!("槽 {slot} / CP932 {code:04X} / 新宋体: {e}"))?
            } else if let Some(fallback) = fallback_canvases
                .iter_mut()
                .find(|candidate| candidate.has_glyph(ch))
            {
                let (pixels, advance) = fallback.draw(ch).map_err(|e| {
                    format!("槽 {slot} / CP932 {code:04X} / {}: {e}", fallback.face)
                })?;
                fallbacks.push(FallbackRecord {
                    slot,
                    cp932: format!("{code:04X}"),
                    character: ch.to_string(),
                    requested_face: "新宋体".into(),
                    actual_face: fallback.face.clone(),
                });
                (pixels, advance)
            } else {
                return Err(format!(
                    "槽 {slot} / CP932 {code:04X} / U+{:04X}: 新宋体及候选备用字体均缺字",
                    ch as u32
                ));
            };
            glyphs.push((slot, pixels));
            advances.push((slot, advance));
        }
        Ok(RedrawnFont {
            glyphs,
            advances,
            face: canvas.face.clone(),
            fallbacks,
        })
    }

    pub fn render_samples(font: &YoxFont, chars: &[char]) -> Result<RenderedSamples, String> {
        let mut canvas = Canvas::new(font.width, font.height)?;
        let rendered = chars
            .iter()
            .map(|ch| canvas.draw(*ch))
            .collect::<Result<Vec<_>, _>>()?;
        let (glyphs, advances) = rendered.into_iter().unzip();
        Ok(RenderedSamples {
            face: canvas.face.clone(),
            glyphs,
            advances,
        })
    }
}

#[cfg(windows)]
pub fn redraw(
    font: &YoxFont,
    carriers: &std::collections::BTreeMap<char, u16>,
) -> Result<RedrawnFont, String> {
    gdi::redraw(font, carriers)
}

#[cfg(windows)]
pub fn render_samples(font: &YoxFont, chars: &[char]) -> Result<RenderedSamples, String> {
    gdi::render_samples(font, chars)
}

#[cfg(not(windows))]
pub fn redraw(
    _font: &YoxFont,
    _carriers: &std::collections::BTreeMap<char, u16>,
) -> Result<RedrawnFont, String> {
    Err("全量字库重绘只在Windows GDI环境可用".into())
}

#[cfg(not(windows))]
pub fn render_samples(_font: &YoxFont, _chars: &[char]) -> Result<RenderedSamples, String> {
    Err("字体预览只在Windows GDI环境可用".into())
}

pub fn preview_bmp(
    width: usize,
    height: usize,
    glyphs: &[(usize, Vec<u8>)],
    sample_slots: &[usize],
) -> Result<Vec<u8>, String> {
    let spacing = 2usize;
    let cols = 8usize;
    let rows = sample_slots.len().div_ceil(cols);
    let image_w = cols * width + (cols + 1) * spacing;
    let image_h = rows * height + (rows + 1) * spacing;
    let stride = (image_w * 3 + 3) & !3;
    let mut pixels = vec![0u8; stride * image_h];
    for (i, slot) in sample_slots.iter().enumerate() {
        let glyph = glyphs
            .get(*slot)
            .ok_or_else(|| format!("预览槽 {slot} 不存在"))?;
        let (x0, y0) = (
            spacing + (i % cols) * (width + spacing),
            spacing + (i / cols) * (height + spacing),
        );
        if glyph.1.len() != width * height {
            return Err("预览字形尺寸不符".into());
        }
        for y in 0..height {
            for x in 0..width {
                let val = glyph.1[y * width + x];
                let at = (y0 + y) * stride + (x0 + x) * 3;
                pixels[at..at + 3].fill(val);
            }
        }
    }
    let mut out = Vec::with_capacity(54 + pixels.len());
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&((54 + pixels.len()) as u32).to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(image_w as i32).to_le_bytes());
    out.extend_from_slice(&(-(image_h as i32)).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&[0; 24]);
    out.extend_from_slice(&pixels);
    Ok(out)
}

pub fn samples_bmp(width: usize, height: usize, glyphs: &[Vec<u8>]) -> Result<Vec<u8>, String> {
    let spacing = 3usize;
    let cols = 8usize;
    let rows = glyphs.len().div_ceil(cols);
    let image_w = cols * width + (cols + 1) * spacing;
    let image_h = rows * height + (rows + 1) * spacing;
    let stride = (image_w * 3 + 3) & !3;
    let mut pixels = vec![0u8; stride * image_h];
    for (i, glyph) in glyphs.iter().enumerate() {
        if glyph.len() != width * height {
            return Err("预览字形尺寸不符".into());
        }
        let (x0, y0) = (
            spacing + (i % cols) * (width + spacing),
            spacing + (i / cols) * (height + spacing),
        );
        for y in 0..height {
            for x in 0..width {
                let val = glyph[y * width + x];
                let at = (y0 + y) * stride + (x0 + x) * 3;
                pixels[at..at + 3].fill(val);
            }
        }
    }
    let mut out = Vec::with_capacity(54 + pixels.len());
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&((54 + pixels.len()) as u32).to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(image_w as i32).to_le_bytes());
    out.extend_from_slice(&(-(image_h as i32)).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&[0; 24]);
    out.extend_from_slice(&pixels);
    Ok(out)
}

pub fn line_preview_bmp(
    width: usize,
    height: usize,
    glyphs: &[Vec<u8>],
    advances: &[u16],
) -> Result<Vec<u8>, String> {
    if glyphs.len() != advances.len() || glyphs.iter().any(|g| g.len() != width * height) {
        return Err("连排预览字形/字宽数据不一致".into());
    }
    let advance_total: usize = advances.iter().map(|a| *a as usize).sum();
    let image_w = advance_total.checked_add(width).ok_or("预览行宽溢出")?;
    let stride = (image_w * 3 + 3) & !3;
    let mut pixels = vec![0u8; stride * height];
    let mut pen = 0usize;
    for (glyph, advance) in glyphs.iter().zip(advances) {
        for y in 0..height {
            for x in 0..width {
                let dest_x = pen + x;
                if dest_x >= image_w {
                    break;
                }
                let value = glyph[y * width + x];
                if value != 0 {
                    let at = y * stride + dest_x * 3;
                    pixels[at..at + 3].fill(value);
                }
            }
        }
        pen += *advance as usize;
    }
    let mut out = Vec::with_capacity(54 + pixels.len());
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&((54 + pixels.len()) as u32).to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(image_w as i32).to_le_bytes());
    out.extend_from_slice(&(-(height as i32)).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&[0; 24]);
    out.extend_from_slice(&pixels);
    Ok(out)
}
