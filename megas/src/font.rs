use ab_glyph::{point, Font, FontArc};
use png::{BitDepth, ColorType, Decoder, Encoder};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::glyph::{GlyphDictionary, GLYPH_COUNT, GLYPH_RECORD_SIZE};
use crate::{ToolError, ToolResult};

const ATLAS_WIDTH: u32 = 4096;
const ATLAS_HEIGHT: u32 = 7546;

#[derive(Debug, Clone)]
pub struct FontBuildReport {
    pub input_bin: PathBuf,
    pub input_png: PathBuf,
    pub output_bin: PathBuf,
    pub output_png: PathBuf,
    pub rendered_slots: usize,
    pub mapping_entries: usize,
    pub donor_fonts: usize,
    pub donor_fonts_used: usize,
}

#[derive(Debug, Clone, Copy)]
struct GlyphRecord {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    advance_width: u16,
    advance_height: u16,
}

pub fn build_font_pair(
    input_bin: &Path,
    input_png: &Path,
    ttf: &Path,
    donors: &[PathBuf],
    output_bin: &Path,
    output_png: &Path,
    custom_mapping: Option<&Path>,
) -> ToolResult<FontBuildReport> {
    build_font_pair_with_targets(
        input_bin,
        input_png,
        ttf,
        donors,
        output_bin,
        output_png,
        custom_mapping,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn build_font_pair_for_targets(
    input_bin: &Path,
    input_png: &Path,
    ttf: &Path,
    donors: &[PathBuf],
    output_bin: &Path,
    output_png: &Path,
    custom_mapping: Option<&Path>,
    targets: &HashSet<char>,
) -> ToolResult<FontBuildReport> {
    build_font_pair_with_targets(
        input_bin,
        input_png,
        ttf,
        donors,
        output_bin,
        output_png,
        custom_mapping,
        Some(targets),
    )
}

pub fn missing_ttf_glyphs(
    ttf: &Path,
    targets: &std::collections::BTreeSet<char>,
) -> ToolResult<Vec<char>> {
    let font = load_font(ttf)?;
    Ok(targets
        .iter()
        .copied()
        .filter(|character| !has_glyph(&font, *character))
        .collect())
}

#[allow(clippy::too_many_arguments)]
fn build_font_pair_with_targets(
    input_bin: &Path,
    input_png: &Path,
    ttf: &Path,
    donors: &[PathBuf],
    output_bin: &Path,
    output_png: &Path,
    custom_mapping: Option<&Path>,
    target_filter: Option<&HashSet<char>>,
) -> ToolResult<FontBuildReport> {
    refuse_existing(output_bin)?;
    refuse_existing(output_png)?;
    let bin = fs::read(input_bin).map_err(|error| {
        ToolError(format!(
            "cannot read font BIN '{}': {error}",
            input_bin.display()
        ))
    })?;
    let records = parse_records(&bin)?;
    let (width, height, mut pixels) = read_png(input_png)?;
    if width != ATLAS_WIDTH || height != ATLAS_HEIGHT {
        return Err(ToolError(format!(
            "font atlas '{}' is {}x{}, expected {}x{}",
            input_png.display(),
            width,
            height,
            ATLAS_WIDTH,
            ATLAS_HEIGHT
        )));
    }
    let mut fonts = Vec::with_capacity(1 + donors.len());
    fonts.push((ttf.to_path_buf(), load_font(ttf)?));
    for donor in donors {
        fonts.push((donor.clone(), load_font(donor)?));
    }
    let dictionary = dictionary_with_custom_mapping(custom_mapping)?;
    let requested = requested_glyph_assignments(&dictionary, target_filter)?;
    let mut assignments = Vec::with_capacity(requested.len());
    let mut missing = Vec::new();
    for (target, index) in requested {
        let font_index = fonts.iter().position(|(_, font)| has_glyph(font, target));
        if let Some(font_index) = font_index {
            assignments.push((target, index, font_index));
        } else {
            missing.push(target);
        }
    }
    if !missing.is_empty() {
        let rendered = missing
            .iter()
            .map(|character| format!("U+{:04X} {:?}", *character as u32, character))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(ToolError(format!(
            "no supplied TTF contains {} requested glyph(s): {rendered}",
            missing.len()
        )));
    }
    let mapping_entries = assignments.len();
    if assignments.is_empty() {
        fs::copy(input_png, output_png).map_err(|error| {
            ToolError(format!(
                "cannot copy unchanged font PNG '{}' to '{}': {error}",
                input_png.display(),
                output_png.display()
            ))
        })?;
        fs::write(output_bin, bin).map_err(|error| {
            ToolError(format!(
                "cannot write font BIN '{}': {error}",
                output_bin.display()
            ))
        })?;
        return Ok(FontBuildReport {
            input_bin: input_bin.to_path_buf(),
            input_png: input_png.to_path_buf(),
            output_bin: output_bin.to_path_buf(),
            output_png: output_png.to_path_buf(),
            rendered_slots: 0,
            mapping_entries: 0,
            donor_fonts: donors.len(),
            donor_fonts_used: 0,
        });
    }
    let mut used_fonts = HashSet::new();
    let mut rendered_slots = 0;
    for (target, index, font_index) in assignments {
        used_fonts.insert(font_index);
        render_glyph(
            &mut pixels,
            width,
            height,
            records[index],
            target,
            &fonts[font_index].1,
        )?;
        rendered_slots += 1;
    }
    write_png(output_png, width, height, &pixels)?;
    fs::write(output_bin, bin).map_err(|error| {
        ToolError(format!(
            "cannot write font BIN '{}': {error}",
            output_bin.display()
        ))
    })?;
    Ok(FontBuildReport {
        input_bin: input_bin.to_path_buf(),
        input_png: input_png.to_path_buf(),
        output_bin: output_bin.to_path_buf(),
        output_png: output_png.to_path_buf(),
        rendered_slots,
        mapping_entries,
        donor_fonts: donors.len(),
        donor_fonts_used: used_fonts.into_iter().filter(|index| *index > 0).count(),
    })
}

fn requested_glyph_assignments(
    dictionary: &GlyphDictionary,
    target_filter: Option<&HashSet<char>>,
) -> ToolResult<Vec<(char, usize)>> {
    let mut requested = if let Some(targets) = target_filter {
        targets
            .iter()
            .copied()
            .map(|target| {
                dictionary
                    .index_for_translated_char(target)
                    .map(|index| (target, index as usize))
                    .ok_or_else(|| {
                        ToolError(format!(
                            "requested character U+{:04X} {target:?} has no glyph slot",
                            target as u32
                        ))
                    })
            })
            .collect::<ToolResult<Vec<_>>>()?
    } else {
        dictionary
            .carrier_table()
            .iter()
            .map(|(target, carrier)| {
                dictionary
                    .index_for_char(*carrier)
                    .map(|index| (*target, index as usize))
                    .ok_or_else(|| ToolError(format!("carrier {carrier:?} has no glyph index")))
            })
            .collect::<ToolResult<Vec<_>>>()?
    };
    requested.sort_unstable_by_key(|(target, index)| (*index, *target as u32));

    for pair in requested.windows(2) {
        if pair[0].1 == pair[1].1 {
            return Err(ToolError(format!(
                "multiple requested characters {:?} and {:?} target the same glyph slot 0x{:04X}",
                pair[0].0, pair[1].0, pair[0].1
            )));
        }
    }
    Ok(requested)
}

fn load_font(path: &Path) -> ToolResult<FontArc> {
    let data = fs::read(path)
        .map_err(|error| ToolError(format!("cannot read TTF '{}': {error}", path.display())))?;
    FontArc::try_from_vec(data)
        .map_err(|error| ToolError(format!("cannot parse TTF '{}': {error}", path.display())))
}

fn has_glyph(font: &FontArc, character: char) -> bool {
    let glyph_id = font.glyph_id(character);
    glyph_id.0 != 0 && font.outline(glyph_id).is_some()
}

fn dictionary_with_custom_mapping(custom_mapping: Option<&Path>) -> ToolResult<GlyphDictionary> {
    let built_in = GlyphDictionary::built_in()?;
    let mut mapping = built_in
        .carrier_table()
        .iter()
        .map(|(target, carrier)| (target.to_string(), carrier.to_string()))
        .collect::<HashMap<_, _>>();
    if let Some(path) = custom_mapping {
        let bytes = fs::read(path).map_err(|error| {
            ToolError(format!(
                "cannot read custom mapping '{}': {error}",
                path.display()
            ))
        })?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|error| {
            ToolError(format!(
                "cannot parse custom mapping '{}': {error}",
                path.display()
            ))
        })?;
        let object = value.as_object().ok_or_else(|| {
            ToolError(format!(
                "custom mapping '{}' must be a JSON object",
                path.display()
            ))
        })?;
        for (target, carrier) in object {
            let carrier = carrier.as_str().ok_or_else(|| {
                ToolError(format!(
                    "custom mapping target {target:?} does not have a string carrier"
                ))
            })?;
            mapping.insert(target.clone(), carrier.to_string());
        }
    }
    GlyphDictionary::from_mapping(mapping)
}

fn parse_records(bin: &[u8]) -> ToolResult<Vec<GlyphRecord>> {
    if bin.len() != GLYPH_COUNT * GLYPH_RECORD_SIZE {
        return Err(ToolError(format!(
            "font BIN has {} bytes, expected {}",
            bin.len(),
            GLYPH_COUNT * GLYPH_RECORD_SIZE
        )));
    }
    let mut records = Vec::with_capacity(GLYPH_COUNT);
    for index in 0..GLYPH_COUNT {
        let offset = index * GLYPH_RECORD_SIZE;
        records.push(GlyphRecord {
            x: u16::from_le_bytes([bin[offset + 4], bin[offset + 5]]),
            y: u16::from_le_bytes([bin[offset + 6], bin[offset + 7]]),
            width: u16::from_le_bytes([bin[offset + 8], bin[offset + 9]]),
            height: u16::from_le_bytes([bin[offset + 10], bin[offset + 11]]),
            advance_width: u16::from_le_bytes([bin[offset + 12], bin[offset + 13]]),
            advance_height: u16::from_le_bytes([bin[offset + 14], bin[offset + 15]]),
        });
    }
    Ok(records)
}

fn read_png(path: &Path) -> ToolResult<(u32, u32, Vec<u8>)> {
    let file = File::open(path).map_err(|error| {
        ToolError(format!(
            "cannot open font atlas '{}': {error}",
            path.display()
        ))
    })?;
    let decoder = Decoder::new(BufReader::new(file));
    let mut reader = decoder.read_info().map_err(|error| {
        ToolError(format!(
            "cannot read PNG header '{}': {error}",
            path.display()
        ))
    })?;
    let output_size = reader.output_buffer_size().ok_or_else(|| {
        ToolError(format!(
            "PNG '{}' has no decodable output size",
            path.display()
        ))
    })?;
    let mut buffer = vec![0; output_size];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|error| ToolError(format!("cannot decode PNG '{}': {error}", path.display())))?;
    if info.color_type != ColorType::Grayscale || info.bit_depth != BitDepth::Eight {
        return Err(ToolError(format!(
            "font atlas '{}' must be 8-bit grayscale (found {:?}/{:?})",
            path.display(),
            info.color_type,
            info.bit_depth
        )));
    }
    Ok((
        info.width,
        info.height,
        buffer[..info.buffer_size()].to_vec(),
    ))
}

fn write_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> ToolResult<()> {
    let file = File::create(path)
        .map_err(|error| ToolError(format!("cannot create PNG '{}': {error}", path.display())))?;
    let mut encoder = Encoder::new(file, width, height);
    encoder.set_color(ColorType::Grayscale);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|error| {
        ToolError(format!(
            "cannot write PNG header '{}': {error}",
            path.display()
        ))
    })?;
    writer
        .write_image_data(pixels)
        .map_err(|error| ToolError(format!("cannot write PNG '{}': {error}", path.display())))
}

fn glyph_position(
    record: GlyphRecord,
    glyph_width: f32,
    glyph_height: f32,
    origin: (f32, f32),
) -> (f32, f32) {
    let x = record.x as f32;
    let y = record.y as f32;
    let width = record.width as f32;
    let height = record.height as f32;
    let left = x + (width - glyph_width) / 2.0;
    let bottom_margin = if height > 2.0 { 1.0 } else { 0.0 };
    (
        left - origin.0,
        y + height - glyph_height - bottom_margin - origin.1,
    )
}

fn render_glyph(
    pixels: &mut [u8],
    atlas_width: u32,
    atlas_height: u32,
    record: GlyphRecord,
    character: char,
    font: &FontArc,
) -> ToolResult<()> {
    let x = record.x as u32;
    let y = record.y as u32;
    let width = record.width as u32;
    let height = record.height as u32;
    if width == 0 || height == 0 {
        return Err(ToolError(format!(
            "carrier slot for {character:?} has an empty atlas rectangle"
        )));
    }
    if x.checked_add(width).is_none_or(|end| end > atlas_width)
        || y.checked_add(height).is_none_or(|end| end > atlas_height)
    {
        return Err(ToolError(format!(
            "atlas rectangle for {character:?} lies outside the PNG"
        )));
    }
    let glyph_id = font.glyph_id(character);
    if glyph_id.0 == 0 || font.outline(glyph_id).is_none() {
        return Err(ToolError(format!(
            "TTF does not contain a glyph for U+{:04X} {:?}",
            character as u32, character
        )));
    }
    for row in y..y + height {
        let start = (row * atlas_width + x) as usize;
        let end = start + width as usize;
        pixels[start..end].fill(0);
    }

    let max_scale = (record.advance_width.max(record.advance_height).max(1)) as f32;
    let min_scale = (max_scale * 0.25).max(8.0);
    let mut scale = max_scale;
    let mut chosen = None;
    while scale >= min_scale {
        let glyph = glyph_id.with_scale_and_position(scale, point(0.0, 0.0));
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            if bounds.width() <= width as f32 - 2.0 && bounds.height() <= height as f32 - 2.0 {
                chosen = Some((
                    scale,
                    bounds.width(),
                    bounds.height(),
                    bounds.min.x,
                    bounds.min.y,
                ));
                break;
            }
        }
        scale -= 0.5;
    }
    let (scale, glyph_width, glyph_height, origin_x, origin_y) = chosen.ok_or_else(|| {
        ToolError(format!(
            "TTF glyph {character:?} cannot fit carrier crop {}x{}",
            width, height
        ))
    })?;
    // ab_glyph positions glyphs by their baseline. The atlas crop already
    // carries the original vertical anchor, so align the new outline to its
    // bottom edge instead of treating the baseline as a top-left point.
    let (position_x, position_y) =
        glyph_position(record, glyph_width, glyph_height, (origin_x, origin_y));
    let position = point(position_x, position_y);
    let glyph = glyph_id.with_scale_and_position(scale, position);
    let outlined = font
        .outline_glyph(glyph)
        .ok_or_else(|| ToolError(format!("cannot outline TTF glyph {character:?}")))?;
    let bounds = outlined.px_bounds();
    let base_x = bounds.min.x.floor() as i64;
    let base_y = bounds.min.y.floor() as i64;
    outlined.draw(|draw_x, draw_y, coverage| {
        let px = base_x + draw_x as i64;
        let py = base_y + draw_y as i64;
        if px >= x as i64 && py >= y as i64 && px < (x + width) as i64 && py < (y + height) as i64 {
            let offset = (py as u32 * atlas_width + px as u32) as usize;
            let value = (coverage.clamp(0.0, 1.0) * 255.0).round() as u8;
            pixels[offset] = pixels[offset].max(value);
        }
    });
    Ok(())
}

fn refuse_existing(path: &Path) -> ToolResult<()> {
    if path.exists() {
        return Err(ToolError(format!(
            "output already exists: '{}'",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_position_uses_outline_origin_and_crop_bottom() {
        let record = GlyphRecord {
            x: 2712,
            y: 1063,
            width: 59,
            height: 20,
            advance_width: 48,
            advance_height: 48,
        };
        let (x, y) = glyph_position(record, 44.0, 5.0, (2.0, -23.0));
        assert_eq!((x, y), (2717.5, 1100.0));
    }

    #[test]
    fn filtered_assignments_include_mapped_and_literal_characters() {
        let dictionary = GlyphDictionary::built_in().unwrap();
        let targets = HashSet::from(['测', '是', '。']);
        let assignments = requested_glyph_assignments(&dictionary, Some(&targets)).unwrap();

        for target in targets {
            let expected = dictionary.index_for_translated_char(target).unwrap() as usize;
            assert!(assignments.contains(&(target, expected)));
        }
    }

    #[test]
    fn filtered_assignments_reject_a_target_and_its_literal_carrier() {
        let dictionary = GlyphDictionary::built_in().unwrap();
        let carrier = dictionary.carrier_for('测').unwrap();
        let targets = HashSet::from(['测', carrier]);
        let error = requested_glyph_assignments(&dictionary, Some(&targets)).unwrap_err();

        assert!(error.0.contains("same glyph slot"));
    }
}
