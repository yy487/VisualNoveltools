use crate::lzss;
use anyhow::{ensure, Context, Result};
use image::{DynamicImage, ImageFormat, RgbImage};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct G24Header {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

pub fn read_header(data: &[u8]) -> Result<G24Header> {
    ensure!(data.len() >= 8, "G24 is shorter than its 8-byte header");
    let x = i16::from_le_bytes([data[0], data[1]]);
    let y = i16::from_le_bytes([data[2], data[3]]);
    let width_i = i16::from_le_bytes([data[4], data[5]]);
    let height_i = i16::from_le_bytes([data[6], data[7]]);
    ensure!(
        width_i > 0 && height_i > 0,
        "invalid G24 dimensions: {width_i}x{height_i}"
    );
    Ok(G24Header {
        x,
        y,
        width: width_i as u16,
        height: height_i as u16,
    })
}

pub fn decode(data: &[u8]) -> Result<(G24Header, RgbImage)> {
    let header = read_header(data)?;
    let width = usize::from(header.width);
    let height = usize::from(header.height);
    let stride = width
        .checked_mul(3)
        .and_then(|value| value.checked_add(3))
        .map(|value| value & !3)
        .context("G24 stride overflow")?;
    let raw_len = stride
        .checked_mul(height)
        .context("G24 image size overflow")?;
    let raw = lzss::decompress(&data[8..], Some(raw_len))?;
    let mut rgb = vec![0u8; width * height * 3];

    for y in 0..height {
        let source = (height - 1 - y) * stride;
        let destination = y * width * 3;
        for x in 0..width {
            let src = source + x * 3;
            let dst = destination + x * 3;
            rgb[dst] = raw[src + 2];
            rgb[dst + 1] = raw[src + 1];
            rgb[dst + 2] = raw[src];
        }
    }

    let image = RgbImage::from_raw(u32::from(header.width), u32::from(header.height), rgb)
        .context("failed to construct decoded G24 image")?;
    Ok((header, image))
}

pub fn encode(image: &DynamicImage, x: i16, y: i16) -> Result<Vec<u8>> {
    let rgb = image.to_rgb8();
    let (width_u32, height_u32) = rgb.dimensions();
    ensure!(
        width_u32 > 0
            && width_u32 <= i16::MAX as u32
            && height_u32 > 0
            && height_u32 <= i16::MAX as u32,
        "G24 dimensions must fit positive int16: {width_u32}x{height_u32}"
    );
    let width = width_u32 as usize;
    let height = height_u32 as usize;
    let stride = (width * 3 + 3) & !3;
    let mut raw = vec![0u8; stride * height];
    let pixels = rgb.as_raw();

    for source_y in 0..height {
        let destination_y = height - 1 - source_y;
        for pixel_x in 0..width {
            let src = (source_y * width + pixel_x) * 3;
            let dst = destination_y * stride + pixel_x * 3;
            raw[dst] = pixels[src + 2];
            raw[dst + 1] = pixels[src + 1];
            raw[dst + 2] = pixels[src];
        }
    }

    let mut output = Vec::with_capacity(8 + raw.len());
    output.extend_from_slice(&x.to_le_bytes());
    output.extend_from_slice(&y.to_le_bytes());
    output.extend_from_slice(&(width_u32 as i16).to_le_bytes());
    output.extend_from_slice(&(height_u32 as i16).to_le_bytes());
    output.extend_from_slice(&lzss::compress(&raw));
    Ok(output)
}

pub fn decode_file(input: &Path, output: &Path) -> Result<G24Header> {
    let data = fs::read(input).with_context(|| format!("read G24: {}", input.display()))?;
    let (header, image) = decode(&data)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    image
        .save_with_format(output, ImageFormat::Png)
        .with_context(|| format!("write PNG: {}", output.display()))?;
    Ok(header)
}

pub fn encode_file(input: &Path, output: &Path, x: i16, y: i16) -> Result<G24Header> {
    let image = image::open(input).with_context(|| format!("read image: {}", input.display()))?;
    let data = encode(&image, x, y)?;
    let header = read_header(&data)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, data).with_context(|| format!("write G24: {}", output.display()))?;
    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    #[test]
    fn roundtrips_pixels_and_coordinates_with_padding() {
        let mut image = RgbImage::new(7, 5);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            *pixel = Rgb([(x * 31) as u8, (y * 47) as u8, (x * 13 + y * 19) as u8]);
        }
        let packed = encode(&DynamicImage::ImageRgb8(image.clone()), -12, 34).unwrap();
        let (header, decoded) = decode(&packed).unwrap();
        assert_eq!(
            (header.x, header.y, header.width, header.height),
            (-12, 34, 7, 5)
        );
        assert_eq!(decoded, image);
    }
}
