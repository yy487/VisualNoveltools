use crate::lzss;
use anyhow::{bail, ensure, Context, Result};
use image::{DynamicImage, GrayImage, ImageFormat};
use serde::Serialize;
use std::fs;
use std::path::Path;

pub const TITLE_PT_WIDTH: u32 = 624;
pub const TITLE_PT_HEIGHT: u32 = 580;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum MskKind {
    TypeA { width: u16, height: u16 },
    Raw8 { width: u32, height: u32 },
    TitlePt,
}

#[derive(Debug)]
pub struct DecodedMask {
    pub kind: MskKind,
    pub image: GrayImage,
}

pub fn is_title_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    upper == "TITLE_PT_M.MSK" || upper == "TITLE_PT_M_FULL_624X580.PNG"
}

pub fn decode(data: &[u8], name: &str, raw_dimensions: Option<(u32, u32)>) -> Result<DecodedMask> {
    let raw = lzss::decompress(data, None).with_context(|| format!("decompress MSK {name}"))?;

    if let Some((width, height, body)) = parse_type_a(&raw) {
        let pixels: Vec<u8> = body
            .iter()
            .map(|&value| if value == 16 { 255 } else { value * 16 })
            .collect();
        let image = GrayImage::from_raw(u32::from(width), u32::from(height), pixels)
            .context("failed to construct Type A mask")?;
        return Ok(DecodedMask {
            kind: MskKind::TypeA { width, height },
            image,
        });
    }

    let (kind, width, height) = if is_title_name(name) {
        (MskKind::TitlePt, TITLE_PT_WIDTH, TITLE_PT_HEIGHT)
    } else if let Some((width, height)) = raw_dimensions {
        (MskKind::Raw8 { width, height }, width, height)
    } else {
        bail!(
            "headerless MSK requires width and height: {name} (decoded {} bytes)",
            raw.len()
        );
    };
    ensure!(
        raw.len() == width as usize * height as usize,
        "MSK dimensions do not match decoded size: {name}, {width}x{height} != {} bytes",
        raw.len()
    );
    let image = GrayImage::from_raw(width, height, raw).context("failed to construct raw mask")?;
    Ok(DecodedMask { kind, image })
}

pub fn classify_template(data: &[u8], name: &str, raw_dimensions: (u32, u32)) -> Result<MskKind> {
    let raw =
        lzss::decompress(data, None).with_context(|| format!("decompress MSK template {name}"))?;
    if let Some((width, height, _)) = parse_type_a(&raw) {
        return Ok(MskKind::TypeA { width, height });
    }
    if is_title_name(name) {
        ensure!(
            raw.len() == (TITLE_PT_WIDTH * TITLE_PT_HEIGHT) as usize,
            "TITLE_PT_M template size is {}, expected {}",
            raw.len(),
            TITLE_PT_WIDTH * TITLE_PT_HEIGHT
        );
        return Ok(MskKind::TitlePt);
    }
    let (width, height) = raw_dimensions;
    ensure!(
        raw.len() == width as usize * height as usize,
        "raw MSK template size mismatch for {name}: {} != {width}x{height}",
        raw.len()
    );
    Ok(MskKind::Raw8 { width, height })
}

pub fn encode(image: &DynamicImage, kind: MskKind) -> Result<Vec<u8>> {
    let gray = image.to_luma8();
    let (width, height) = gray.dimensions();
    let mut raw = Vec::new();

    match kind {
        MskKind::TypeA {
            width: expected_width,
            height: expected_height,
        } => {
            ensure!(
                (width, height) == (u32::from(expected_width), u32::from(expected_height)),
                "Type A mask dimensions changed: {width}x{height}, expected {expected_width}x{expected_height}"
            );
            raw.extend_from_slice(&expected_width.to_le_bytes());
            raw.extend_from_slice(&expected_height.to_le_bytes());
            raw.extend(
                gray.as_raw()
                    .iter()
                    .map(|&value| ((u16::from(value) * 16 + 127) / 255) as u8),
            );
        }
        MskKind::Raw8 {
            width: expected_width,
            height: expected_height,
        } => {
            ensure!(
                (width, height) == (expected_width, expected_height),
                "raw mask dimensions changed: {width}x{height}, expected {expected_width}x{expected_height}"
            );
            raw.extend_from_slice(gray.as_raw());
        }
        MskKind::TitlePt => {
            ensure!(
                (width, height) == (TITLE_PT_WIDTH, TITLE_PT_HEIGHT),
                "TITLE_PT_M must be {}x{}, got {width}x{height}",
                TITLE_PT_WIDTH,
                TITLE_PT_HEIGHT
            );
            raw.extend_from_slice(gray.as_raw());
        }
    }

    Ok(lzss::compress(&raw))
}

pub fn decode_file(
    input: &Path,
    output: &Path,
    raw_dimensions: Option<(u32, u32)>,
) -> Result<MskKind> {
    let data = fs::read(input).with_context(|| format!("read MSK: {}", input.display()))?;
    let name = input
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let decoded = decode(&data, name, raw_dimensions)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    decoded
        .image
        .save_with_format(output, ImageFormat::Png)
        .with_context(|| format!("write mask PNG: {}", output.display()))?;
    Ok(decoded.kind)
}

pub fn encode_file(input: &Path, output: &Path, kind: MskKind) -> Result<()> {
    let image =
        image::open(input).with_context(|| format!("read mask image: {}", input.display()))?;
    let data = encode(&image, kind)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, data).with_context(|| format!("write MSK: {}", output.display()))?;
    Ok(())
}

fn parse_type_a(raw: &[u8]) -> Option<(u16, u16, &[u8])> {
    if raw.len() < 5 {
        return None;
    }
    let width = u16::from_le_bytes([raw[0], raw[1]]);
    let height = u16::from_le_bytes([raw[2], raw[3]]);
    let body = &raw[4..];
    if width == 0
        || height == 0
        || width > 4096
        || height > 4096
        || body.len() != usize::from(width) * usize::from(height)
        || body.iter().any(|&value| value > 16)
    {
        return None;
    }
    Some((width, height, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GrayImage, Luma};

    #[test]
    fn raw_mask_roundtrips() {
        let mut image = GrayImage::new(19, 7);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            *pixel = Luma([((x * 17 + y * 29) & 255) as u8]);
        }
        let kind = MskKind::Raw8 {
            width: 19,
            height: 7,
        };
        let packed = encode(&DynamicImage::ImageLuma8(image.clone()), kind).unwrap();
        assert_eq!(
            decode(&packed, "TEST_M.MSK", Some((19, 7))).unwrap().image,
            image
        );
    }

    #[test]
    fn type_a_preserves_canonical_levels() {
        let values: Vec<u8> = (0..=16)
            .map(|value| if value == 16 { 255 } else { value * 16 })
            .collect();
        let image = GrayImage::from_raw(17, 1, values).unwrap();
        let kind = MskKind::TypeA {
            width: 17,
            height: 1,
        };
        let packed = encode(&DynamicImage::ImageLuma8(image.clone()), kind).unwrap();
        assert_eq!(decode(&packed, "A.MSK", None).unwrap().image, image);
    }
}
