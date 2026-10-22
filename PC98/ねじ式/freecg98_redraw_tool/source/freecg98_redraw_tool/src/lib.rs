use ab_glyph::{point, Font, FontRef};
use encoding_rs::SHIFT_JIS;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const EMBEDDED_BMP: &[u8] = include_bytes!("../assets/FREECG98.BMP");
pub const EMBEDDED_MAPPING: &[u8] = include_bytes!("../assets/subs_cn_jp.json");
pub const EMBEDDED_TTF: &[u8] = include_bytes!("../assets/simsun_cnjp.ttf");

const EXPECTED_WIDTH: u32 = 2048;
const EXPECTED_HEIGHT: u32 = 2048;
const GLYPH_SIDE: u32 = 16;
const COVERAGE_THRESHOLD: f32 = 0.35;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolError(pub String);

impl Display for ToolError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ToolError {}

pub type ToolResult<T> = Result<T, ToolError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlyphSlot {
    pub jis_row: u8,
    pub jis_cell: u8,
    pub tile_x: u8,
    pub tile_y: u8,
}

impl GlyphSlot {
    pub fn jis_code(self) -> u16 {
        u16::from_be_bytes([self.jis_row, self.jis_cell])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappingEntry {
    pub target: char,
    pub carrier: char,
    pub slot: GlyphSlot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BmpInfo {
    pub width: u32,
    pub height: u32,
    pub pixel_offset: usize,
    pub stride: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusEvidence {
    pub target: char,
    pub carrier: char,
    pub shift_jis: [u8; 2],
    pub slot: GlyphSlot,
    pub source_black_pixels: usize,
    pub rendered_black_pixels: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderReport {
    pub mapping_entries: usize,
    pub unique_slots: usize,
    pub source_nonempty_slots: usize,
    pub rendered_nonempty_slots: usize,
    pub output_bytes: usize,
    pub focus: Vec<FocusEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyReport {
    pub mapping_entries: usize,
    pub unique_slots: usize,
    pub nonempty_slots: usize,
    pub bmp_bytes: usize,
}

pub fn parse_mapping() -> ToolResult<Vec<MappingEntry>> {
    let raw: BTreeMap<String, String> = serde_json::from_slice(EMBEDDED_MAPPING)
        .map_err(|error| ToolError(format!("embedded mapping is invalid JSON: {error}")))?;
    let mut entries = Vec::with_capacity(raw.len());
    let mut carriers = BTreeSet::new();
    let mut slots = BTreeMap::new();

    for (target_text, carrier_text) in raw {
        let target = one_scalar("mapping target", &target_text)?;
        let carrier = one_scalar("mapping carrier", &carrier_text)?;
        if !carriers.insert(carrier) {
            return Err(ToolError(format!(
                "mapping carrier U+{:04X} {carrier:?} is assigned more than once",
                carrier as u32
            )));
        }
        let slot = carrier_to_slot(carrier)?;
        if let Some((previous_target, previous_carrier)) =
            slots.insert((slot.tile_x, slot.tile_y), (target, carrier))
        {
            return Err(ToolError(format!(
                "targets {previous_target:?}/{target:?} use carriers {previous_carrier:?}/{carrier:?} that collide at tile ({}, {})",
                slot.tile_x, slot.tile_y
            )));
        }
        entries.push(MappingEntry {
            target,
            carrier,
            slot,
        });
    }

    entries.sort_unstable_by_key(|entry| (entry.slot.tile_y, entry.slot.tile_x));
    Ok(entries)
}

pub fn carrier_to_slot(carrier: char) -> ToolResult<GlyphSlot> {
    let encoded_text = carrier.to_string();
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&encoded_text);
    if had_errors || encoded.len() != 2 {
        return Err(ToolError(format!(
            "carrier U+{:04X} {carrier:?} is not one strict two-byte CP932 character",
            carrier as u32
        )));
    }
    shift_jis_to_slot(encoded[0], encoded[1]).map_err(|error| {
        ToolError(format!(
            "carrier U+{:04X} {carrier:?}: {}",
            carrier as u32, error.0
        ))
    })
}

pub fn carrier_shift_jis(carrier: char) -> ToolResult<[u8; 2]> {
    let encoded_text = carrier.to_string();
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&encoded_text);
    if had_errors || encoded.len() != 2 {
        return Err(ToolError(format!(
            "carrier U+{:04X} {carrier:?} is not one strict two-byte CP932 character",
            carrier as u32
        )));
    }
    Ok([encoded[0], encoded[1]])
}

pub fn shift_jis_to_slot(lead: u8, trail: u8) -> ToolResult<GlyphSlot> {
    if !((0x81..=0x9f).contains(&lead) || (0xe0..=0xef).contains(&lead)) {
        return Err(ToolError(format!(
            "CP932 lead byte 0x{lead:02X} is outside standard JIS X 0208"
        )));
    }
    if !((0x40..=0x7e).contains(&trail) || (0x80..=0xfc).contains(&trail)) {
        return Err(ToolError(format!(
            "CP932 trail byte 0x{trail:02X} is invalid"
        )));
    }

    let mut row = if lead <= 0x9f {
        (lead - 0x81) * 2 + 0x21
    } else {
        (lead - 0xc1) * 2 + 0x21
    };
    let cell = if trail >= 0x9f {
        row += 1;
        trail - 0x7e
    } else if trail > 0x7f {
        trail - 0x20
    } else {
        trail - 0x1f
    };
    if !(0x21..=0x7e).contains(&row) || !(0x21..=0x7e).contains(&cell) {
        return Err(ToolError(format!(
            "CP932 bytes {lead:02X}{trail:02X} convert outside JIS X 0208: {row:02X}{cell:02X}"
        )));
    }

    // FREECG98 stores a 94-column JIS table with an intentional leading
    // column. The JIS row selects X after subtracting 0x20; the JIS cell is
    // the absolute Y tile. Using CP932 bytes directly writes outside it.
    Ok(GlyphSlot {
        jis_row: row,
        jis_cell: cell,
        tile_x: row - 0x20,
        tile_y: cell,
    })
}

pub fn parse_bmp(bytes: &[u8]) -> ToolResult<BmpInfo> {
    if bytes.len() < 62 || &bytes[0..2] != b"BM" {
        return Err(ToolError("font image is not a BMP file".to_string()));
    }
    let declared_size = read_u32(bytes, 2)? as usize;
    let pixel_offset = read_u32(bytes, 10)? as usize;
    let dib_size = read_u32(bytes, 14)?;
    let width = read_i32(bytes, 18)?;
    let height = read_i32(bytes, 22)?;
    let planes = read_u16(bytes, 26)?;
    let bits_per_pixel = read_u16(bytes, 28)?;
    let compression = read_u32(bytes, 30)?;
    if declared_size != bytes.len() {
        return Err(ToolError(format!(
            "BMP declares {declared_size} bytes but contains {}",
            bytes.len()
        )));
    }
    if dib_size != 40
        || width != EXPECTED_WIDTH as i32
        || height != EXPECTED_HEIGHT as i32
        || planes != 1
        || bits_per_pixel != 1
        || compression != 0
        || pixel_offset != 62
    {
        return Err(ToolError(format!(
            "unsupported BMP layout: DIB={dib_size}, size={width}x{height}, planes={planes}, bpp={bits_per_pixel}, compression={compression}, pixel_offset={pixel_offset}"
        )));
    }
    if bytes.get(54..62) != Some(&[0, 0, 0, 0, 255, 255, 255, 0]) {
        return Err(ToolError(
            "BMP palette must be black index 0 and white index 1".to_string(),
        ));
    }
    let stride = (EXPECTED_WIDTH as usize).div_ceil(32) * 4;
    let required = pixel_offset
        .checked_add(stride * EXPECTED_HEIGHT as usize)
        .ok_or_else(|| ToolError("BMP pixel extent overflow".to_string()))?;
    if required != bytes.len() {
        return Err(ToolError(format!(
            "BMP pixel extent ends at {required}, file ends at {}",
            bytes.len()
        )));
    }
    Ok(BmpInfo {
        width: EXPECTED_WIDTH,
        height: EXPECTED_HEIGHT,
        pixel_offset,
        stride,
    })
}

pub fn render_embedded() -> ToolResult<(Vec<u8>, RenderReport)> {
    let entries = parse_mapping()?;
    let info = parse_bmp(EMBEDDED_BMP)?;
    let font = FontRef::try_from_slice(EMBEDDED_TTF)
        .map_err(|error| ToolError(format!("embedded TTF is invalid: {error}")))?;
    let mut output = EMBEDDED_BMP.to_vec();
    let mut source_counts = BTreeMap::new();
    let mut source_nonempty_slots = 0;

    for entry in &entries {
        ensure_slot_in_bmp(info, entry.slot)?;
        let count = black_pixel_count(EMBEDDED_BMP, info, entry.slot);
        if count > 0 {
            source_nonempty_slots += 1;
        }
        source_counts.insert(entry.target, count);
        render_glyph(&mut output, info, entry.slot, entry.target, &font)?;
    }

    let rendered_nonempty_slots = entries
        .iter()
        .filter(|entry| black_pixel_count(&output, info, entry.slot) > 0)
        .count();
    if rendered_nonempty_slots != entries.len() {
        return Err(ToolError(format!(
            "rendered only {rendered_nonempty_slots}/{} nonempty mapped slots",
            entries.len()
        )));
    }

    let mut focus = Vec::new();
    for target in ['黑', '赶'] {
        let entry = entries
            .iter()
            .find(|entry| entry.target == target)
            .ok_or_else(|| ToolError(format!("embedded mapping has no {target:?} entry")))?;
        focus.push(FocusEvidence {
            target,
            carrier: entry.carrier,
            shift_jis: carrier_shift_jis(entry.carrier)?,
            slot: entry.slot,
            source_black_pixels: source_counts[&target],
            rendered_black_pixels: black_pixel_count(&output, info, entry.slot),
        });
    }

    Ok((
        output,
        RenderReport {
            mapping_entries: entries.len(),
            unique_slots: entries.len(),
            source_nonempty_slots,
            rendered_nonempty_slots,
            output_bytes: EMBEDDED_BMP.len(),
            focus,
        },
    ))
}

pub fn verify_font_bmp(bytes: &[u8]) -> ToolResult<VerifyReport> {
    let entries = parse_mapping()?;
    let info = parse_bmp(bytes)?;
    for entry in &entries {
        ensure_slot_in_bmp(info, entry.slot)?;
    }
    let nonempty_slots = entries
        .iter()
        .filter(|entry| black_pixel_count(bytes, info, entry.slot) > 0)
        .count();
    Ok(VerifyReport {
        mapping_entries: entries.len(),
        unique_slots: entries.len(),
        nonempty_slots,
        bmp_bytes: bytes.len(),
    })
}

pub fn black_pixel_count(bytes: &[u8], info: BmpInfo, slot: GlyphSlot) -> usize {
    let (origin_x, origin_y) = slot_origin(slot);
    let mut count = 0;
    for y in origin_y..origin_y + GLYPH_SIDE {
        for x in origin_x..origin_x + GLYPH_SIDE {
            if !pixel_is_white(bytes, info, x, y) {
                count += 1;
            }
        }
    }
    count
}

pub fn write_output(path: &Path, bytes: &[u8], overwrite: bool) -> ToolResult<()> {
    if path.exists() && !overwrite {
        return Err(ToolError(format!(
            "output already exists (use --overwrite only when intended): '{}'",
            path.display()
        )));
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    if let Some(parent) = parent {
        if !parent.is_dir() {
            return Err(ToolError(format!(
                "output directory does not exist: '{}'",
                parent.display()
            )));
        }
    }
    let temp_path = temporary_sibling(path)?;
    let result = (|| -> ToolResult<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|error| {
                ToolError(format!(
                    "cannot create temporary output '{}': {error}",
                    temp_path.display()
                ))
            })?;
        file.write_all(bytes).map_err(|error| {
            ToolError(format!(
                "cannot write temporary output '{}': {error}",
                temp_path.display()
            ))
        })?;
        file.sync_all().map_err(|error| {
            ToolError(format!(
                "cannot flush temporary output '{}': {error}",
                temp_path.display()
            ))
        })?;
        drop(file);
        if path.exists() {
            if !overwrite {
                return Err(ToolError(format!(
                    "output appeared while rendering: '{}'",
                    path.display()
                )));
            }
            fs::remove_file(path).map_err(|error| {
                ToolError(format!(
                    "cannot replace existing output '{}': {error}",
                    path.display()
                ))
            })?;
        }
        fs::rename(&temp_path, path).map_err(|error| {
            ToolError(format!(
                "cannot commit output '{}' to '{}': {error}",
                temp_path.display(),
                path.display()
            ))
        })?;
        Ok(())
    })();
    if result.is_err() && temp_path.exists() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

fn one_scalar(role: &str, text: &str) -> ToolResult<char> {
    let mut chars = text.chars();
    let character = chars
        .next()
        .ok_or_else(|| ToolError(format!("{role} is empty")))?;
    if chars.next().is_some() {
        return Err(ToolError(format!(
            "{role} {text:?} is not one Unicode scalar"
        )));
    }
    Ok(character)
}

fn ensure_slot_in_bmp(info: BmpInfo, slot: GlyphSlot) -> ToolResult<()> {
    let (x, y) = slot_origin(slot);
    if x + GLYPH_SIDE > info.width || y + GLYPH_SIDE > info.height {
        return Err(ToolError(format!(
            "JIS {:04X} maps outside BMP at tile ({}, {})",
            slot.jis_code(),
            slot.tile_x,
            slot.tile_y
        )));
    }
    Ok(())
}

fn render_glyph(
    bmp: &mut [u8],
    info: BmpInfo,
    slot: GlyphSlot,
    character: char,
    font: &FontRef<'_>,
) -> ToolResult<()> {
    let glyph_id = font.glyph_id(character);
    if glyph_id.0 == 0 || font.outline(glyph_id).is_none() {
        return Err(ToolError(format!(
            "embedded TTF has no outline for U+{:04X} {character:?}",
            character as u32
        )));
    }
    clear_slot(bmp, info, slot);
    let mut scale = 16.0;
    let chosen = loop {
        let glyph = glyph_id.with_scale_and_position(scale, point(0.0, 0.0));
        if let Some(outline) = font.outline_glyph(glyph) {
            let bounds = outline.px_bounds();
            if bounds.width() <= 15.0 && bounds.height() <= 15.0 {
                break Some((scale, bounds));
            }
        }
        scale -= 0.25;
        if scale < 7.0 {
            break None;
        }
    };
    let (scale, initial_bounds) = chosen.ok_or_else(|| {
        ToolError(format!(
            "U+{:04X} {character:?} cannot fit a 16x16 FREECG98 slot",
            character as u32
        ))
    })?;
    let (origin_x, origin_y) = slot_origin(slot);
    let target_x = origin_x as f32 + (GLYPH_SIDE as f32 - initial_bounds.width()) / 2.0;
    let target_y = origin_y as f32 + (GLYPH_SIDE as f32 - initial_bounds.height()) / 2.0;
    let position = point(
        target_x - initial_bounds.min.x,
        target_y - initial_bounds.min.y,
    );
    let glyph = glyph_id.with_scale_and_position(scale, position);
    let outline = font
        .outline_glyph(glyph)
        .ok_or_else(|| ToolError(format!("cannot outline {character:?} after positioning")))?;
    let bounds = outline.px_bounds();
    let base_x = bounds.min.x.floor() as i32;
    let base_y = bounds.min.y.floor() as i32;
    let mut coverage_map = [0.0f32; (GLYPH_SIDE * GLYPH_SIDE) as usize];
    outline.draw(|draw_x, draw_y, coverage| {
        let x = base_x + draw_x as i32;
        let y = base_y + draw_y as i32;
        if x >= origin_x as i32
            && y >= origin_y as i32
            && x < (origin_x + GLYPH_SIDE) as i32
            && y < (origin_y + GLYPH_SIDE) as i32
        {
            let local_x = x as u32 - origin_x;
            let local_y = y as u32 - origin_y;
            let index = (local_y * GLYPH_SIDE + local_x) as usize;
            coverage_map[index] = coverage_map[index].max(coverage);
        }
    });
    let maximum = coverage_map.iter().copied().fold(0.0f32, f32::max);
    if maximum <= 0.0 {
        return Err(ToolError(format!(
            "TTF outline for U+{:04X} {character:?} does not touch its 16x16 slot",
            character as u32
        )));
    }
    let threshold = COVERAGE_THRESHOLD.min(maximum * 0.5);
    for (index, coverage) in coverage_map.into_iter().enumerate() {
        if coverage >= threshold {
            let x = origin_x + index as u32 % GLYPH_SIDE;
            let y = origin_y + index as u32 / GLYPH_SIDE;
            set_black(bmp, info, x, y);
        }
    }
    let count = black_pixel_count(bmp, info, slot);
    if count == 0 {
        return Err(ToolError(format!(
            "rendered U+{:04X} {character:?} as an empty slot",
            character as u32
        )));
    }
    Ok(())
}

fn slot_origin(slot: GlyphSlot) -> (u32, u32) {
    (
        slot.tile_x as u32 * GLYPH_SIDE,
        slot.tile_y as u32 * GLYPH_SIDE,
    )
}

fn clear_slot(bytes: &mut [u8], info: BmpInfo, slot: GlyphSlot) {
    let (origin_x, origin_y) = slot_origin(slot);
    for y in origin_y..origin_y + GLYPH_SIDE {
        for x in origin_x..origin_x + GLYPH_SIDE {
            set_white(bytes, info, x, y);
        }
    }
}

fn pixel_location(info: BmpInfo, x: u32, y: u32) -> (usize, u8) {
    let file_row = info.height - 1 - y;
    let offset = info.pixel_offset + file_row as usize * info.stride + (x / 8) as usize;
    let mask = 0x80 >> (x % 8);
    (offset, mask)
}

fn pixel_is_white(bytes: &[u8], info: BmpInfo, x: u32, y: u32) -> bool {
    let (offset, mask) = pixel_location(info, x, y);
    bytes[offset] & mask != 0
}

fn set_black(bytes: &mut [u8], info: BmpInfo, x: u32, y: u32) {
    let (offset, mask) = pixel_location(info, x, y);
    bytes[offset] &= !mask;
}

fn set_white(bytes: &mut [u8], info: BmpInfo, x: u32, y: u32) {
    let (offset, mask) = pixel_location(info, x, y);
    bytes[offset] |= mask;
}

fn read_u16(bytes: &[u8], offset: usize) -> ToolResult<u16> {
    let raw = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| ToolError(format!("truncated BMP field at 0x{offset:X}")))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> ToolResult<u32> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| ToolError(format!("truncated BMP field at 0x{offset:X}")))?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn read_i32(bytes: &[u8], offset: usize) -> ToolResult<i32> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| ToolError(format!("truncated BMP field at 0x{offset:X}")))?;
    Ok(i32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn temporary_sibling(path: &Path) -> ToolResult<PathBuf> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .ok_or_else(|| ToolError(format!("output has no file name: '{}'", path.display())))?
        .to_string_lossy();
    for sequence in 0..1000u32 {
        let candidate = parent.join(format!(
            ".{file_name}.tmp-{}-{sequence}",
            std::process::id()
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(ToolError(format!(
        "cannot allocate a temporary output beside '{}'",
        path.display()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_bmp_layout_and_palette_are_exact() {
        let info = parse_bmp(EMBEDDED_BMP).unwrap();
        assert_eq!(info.width, 2048);
        assert_eq!(info.height, 2048);
        assert_eq!(info.pixel_offset, 62);
        assert_eq!(info.stride, 256);
        assert_eq!(EMBEDDED_BMP.len(), 524_350);
    }

    #[test]
    fn all_embedded_mapping_slots_are_unique_present_and_in_bounds() {
        let info = parse_bmp(EMBEDDED_BMP).unwrap();
        let entries = parse_mapping().unwrap();
        assert_eq!(entries.len(), 3025);
        let slots = entries
            .iter()
            .map(|entry| (entry.slot.tile_x, entry.slot.tile_y))
            .collect::<BTreeSet<_>>();
        assert_eq!(slots.len(), entries.len());
        for entry in entries {
            ensure_slot_in_bmp(info, entry.slot).unwrap();
            assert!(black_pixel_count(EMBEDDED_BMP, info, entry.slot) > 0);
        }
    }

    #[test]
    fn black_and_gan_use_jis_tiles_not_raw_cp932_bytes() {
        let entries = parse_mapping().unwrap();
        let info = parse_bmp(EMBEDDED_BMP).unwrap();
        let black = entries.iter().find(|entry| entry.target == '黑').unwrap();
        assert_eq!(black.carrier, '黒');
        assert_eq!(carrier_shift_jis(black.carrier).unwrap(), [0x8d, 0x95]);
        assert_eq!(black.slot.jis_code(), 0x3975);
        assert_eq!((black.slot.tile_x, black.slot.tile_y), (25, 117));
        assert_eq!(black_pixel_count(EMBEDDED_BMP, info, black.slot), 79);

        let gan = entries.iter().find(|entry| entry.target == '赶').unwrap();
        assert_eq!(gan.carrier, '骭');
        assert_eq!(carrier_shift_jis(gan.carrier).unwrap(), [0xe9, 0x8c]);
        assert_eq!(gan.slot.jis_code(), 0x716c);
        assert_eq!((gan.slot.tile_x, gan.slot.tile_y), (81, 108));
        assert_eq!(black_pixel_count(EMBEDDED_BMP, info, gan.slot), 79);
    }

    #[test]
    fn cp932_extensions_outside_standard_jis_are_rejected() {
        assert!(shift_jis_to_slot(0xf0, 0x40).is_err());
        assert!(shift_jis_to_slot(0x81, 0x7f).is_err());
    }

    #[test]
    fn full_render_preserves_container_and_populates_every_mapping_slot() {
        let (rendered, report) = render_embedded().unwrap();
        assert_eq!(report.mapping_entries, 3025);
        assert_eq!(report.unique_slots, 3025);
        assert_eq!(report.source_nonempty_slots, 3025);
        assert_eq!(report.rendered_nonempty_slots, 3025);
        assert_eq!(rendered.len(), EMBEDDED_BMP.len());
        assert_eq!(&rendered[..62], &EMBEDDED_BMP[..62]);
        let verified = verify_font_bmp(&rendered).unwrap();
        assert_eq!(verified.nonempty_slots, 3025);
    }

    #[test]
    fn malformed_bmp_is_rejected() {
        assert!(parse_bmp(b"not a bmp").is_err());
        let mut damaged = EMBEDDED_BMP.to_vec();
        damaged[28] = 8;
        assert!(parse_bmp(&damaged).is_err());
    }
}
