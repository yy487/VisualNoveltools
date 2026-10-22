//! Persistent PC-88 KANJI1 carrier mapping and deterministic ROM rebuilding.
//!
//! The game receives CP932 bytes and folds their JIS coordinates into a
//! physical KANJI1 slot.  A mapping entry therefore records both the visible
//! Unicode character and the carrier byte pair used by MSG injection.

#[cfg(not(windows))]
use ab_glyph::{point, Font, FontRef, ScaleFont};
use anyhow::{bail, ensure, Context, Result};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::msg_json::MsgTranslationFile;
use crate::util::{hex_encode, sha256_hex, write_output_file};

const FONT_MAP_FORMAT_V1: &str = "kuki-ayaka-kanji1-map-v1";
pub const FONT_MAP_FORMAT: &str = "kuki-ayaka-kanji1-map-v2";
const FONT_MAP_VERSION: u32 = 2;
const MAPPING_ID: &str = "kanji1-dynamic-v2";
const CARRIER_POLICY: &str =
    "all CJK ideographs and non-CP932 characters use unused CP932 carriers in JIS ku 16-47; START.ZIL E8/E9 address mapping; append-only lock";
pub const KANJI1_SIZE: usize = 0x20_000;
pub const KANJI1_SLOTS: usize = 4096;
pub const KANJI1_GLYPH_BYTES: usize = 0x20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontMap {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_version")]
    pub version: u32,
    #[serde(rename = "_mapping_id")]
    pub mapping_id: String,
    #[serde(rename = "_source_rom_sha256")]
    pub source_rom_sha256: String,
    #[serde(rename = "_ttf_sha256")]
    pub ttf_sha256: String,
    #[serde(rename = "_rom_size")]
    pub rom_size: usize,
    #[serde(rename = "_carrier_policy")]
    pub carrier_policy: String,
    pub entries: Vec<FontMapEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontMapEntry {
    pub character: String,
    pub carrier: String,
    #[serde(rename = "_carrier_cp932")]
    pub carrier_cp932: String,
    #[serde(rename = "_jis_ku")]
    pub jis_ku: u8,
    #[serde(rename = "_jis_ten")]
    pub jis_ten: u8,
    #[serde(rename = "_slot")]
    pub slot: usize,
    #[serde(rename = "_rom_offset")]
    pub rom_offset: usize,
    #[serde(rename = "_kind")]
    pub kind: String,
    #[serde(rename = "_usage_count")]
    pub usage_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontScanReport {
    pub json_files: usize,
    pub entries_scanned: usize,
    pub required_characters: usize,
    pub new_mappings: usize,
    pub repaired_mappings: usize,
    pub total_mappings: usize,
    pub output: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontRebuildReport {
    pub mappings: usize,
    pub changed_slots: usize,
    pub changed_bytes: usize,
    pub output: PathBuf,
    pub atlas: Option<PathBuf>,
}

impl FontMap {
    pub fn load(path: &Path) -> Result<Self> {
        let map = Self::read(path)?;
        map.validate()?;
        Ok(map)
    }

    fn load_for_scan(path: &Path) -> Result<(Self, usize)> {
        let mut map = Self::read(path)?;
        ensure!(
            (map.format == FONT_MAP_FORMAT_V1 && map.version == 1)
                || (map.format == FONT_MAP_FORMAT && map.version == FONT_MAP_VERSION),
            "unsupported font mapping format {:?} version {}",
            map.format,
            map.version
        );
        let repaired = map.repair_derived_mapping_fields()?;
        map.format = FONT_MAP_FORMAT.to_owned();
        map.version = FONT_MAP_VERSION;
        map.mapping_id = MAPPING_ID.to_owned();
        map.carrier_policy = CARRIER_POLICY.to_owned();
        map.validate()?;
        Ok((map, repaired))
    }

    fn read(path: &Path) -> Result<Self> {
        let bytes = fs::read(path)
            .with_context(|| format!("failed to read font mapping {}", path.display()))?;
        serde_json::from_slice(&bytes)
            .with_context(|| format!("invalid font mapping JSON {}", path.display()))
    }

    pub fn save(&self, path: &Path, overwrite: bool) -> Result<()> {
        let mut bytes = serde_json::to_vec_pretty(self).context("failed to serialize font map")?;
        bytes.push(b'\n');
        write_output_file(path, &bytes, overwrite)
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.format == FONT_MAP_FORMAT,
            "unsupported font mapping format {:?}",
            self.format
        );
        ensure!(
            self.version == FONT_MAP_VERSION,
            "unsupported font mapping version {}",
            self.version
        );
        ensure!(
            self.rom_size == KANJI1_SIZE,
            "font mapping ROM size must be 0x20000"
        );
        ensure!(
            self.carrier_policy == CARRIER_POLICY,
            "outdated font mapping carrier policy; rerun font scan before MSG injection or ROM rebuild"
        );
        let mut chars = HashSet::new();
        let mut carriers = HashSet::new();
        let mut slots: HashSet<usize> = HashSet::new();
        for entry in &self.entries {
            let character = one_char(&entry.character, "mapping character")?;
            let carrier = one_char(&entry.carrier, "mapping carrier")?;
            ensure!(
                chars.insert(character),
                "duplicate mapping character {:?}",
                character
            );
            ensure!(
                carriers.insert(carrier),
                "duplicate mapping carrier {:?}",
                carrier
            );
            ensure!(
                slots.insert(entry.slot),
                "duplicate mapping physical slot 0x{:X}",
                entry.slot
            );
            ensure!(
                entry.slot < KANJI1_SLOTS,
                "mapping slot out of range: 0x{:X}",
                entry.slot
            );
            ensure!(
                entry.rom_offset == entry.slot * KANJI1_GLYPH_BYTES,
                "mapping ROM offset does not match slot for {:?}",
                character
            );
            ensure!(
                (16..=47).contains(&entry.jis_ku),
                "mapping carrier JIS ku {} is outside KANJI1 carrier rows 16-47",
                entry.jis_ku
            );
            let bytes = cp932_bytes(carrier, "mapping carrier")?;
            ensure!(
                hex_encode(&bytes) == entry.carrier_cp932,
                "mapping carrier bytes mismatch for {:?}",
                character
            );
            let (ku, ten) = jis_from_cp932(&bytes)?;
            ensure!(
                ku == entry.jis_ku && ten == entry.jis_ten,
                "mapping JIS coordinates mismatch for {:?}",
                character
            );
            ensure!(
                kanji1_slot_from_jis_bytes(ku, ten) == entry.slot,
                "mapping physical slot mismatch for {:?}",
                character
            );
        }
        Ok(())
    }

    fn repair_derived_mapping_fields(&mut self) -> Result<usize> {
        let mut repaired = 0usize;
        for entry in &mut self.entries {
            let carrier = one_char(&entry.carrier, "mapping carrier")?;
            let bytes = cp932_bytes(carrier, "mapping carrier")?;
            let (ku, ten) = jis_from_cp932(&bytes)?;
            let slot = kanji1_slot_from_jis_bytes(ku, ten);
            let carrier_cp932 = hex_encode(&bytes);
            let rom_offset = slot * KANJI1_GLYPH_BYTES;
            if entry.carrier_cp932 != carrier_cp932
                || entry.jis_ku != ku
                || entry.jis_ten != ten
                || entry.slot != slot
                || entry.rom_offset != rom_offset
            {
                repaired += 1;
            }
            entry.carrier_cp932 = carrier_cp932;
            entry.jis_ku = ku;
            entry.jis_ten = ten;
            entry.slot = slot;
            entry.rom_offset = rom_offset;
        }
        Ok(repaired)
    }

    pub fn mapped_carrier(&self, character: char) -> Option<char> {
        self.entries
            .iter()
            .find(|entry| entry.character == character.to_string())
            .and_then(|entry| entry.carrier.chars().next())
    }

    /// Encode plain text using persistent carriers where a character is mapped.
    pub fn encode_plain(&self, value: &str, role: &str) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        let mut unsupported = Vec::new();
        for character in value.chars() {
            if character == '\0' || character.is_control() {
                bail!("{role} contains a forbidden Unicode control character");
            }
            if let Some(carrier) = self.mapped_carrier(character) {
                output.extend_from_slice(&cp932_bytes(carrier, role)?);
                continue;
            }
            let text = character.to_string();
            let (encoded, _, had_errors) = SHIFT_JIS.encode(&text);
            if had_errors {
                unsupported.push(format!("{character} (U+{:04X})", u32::from(character)));
            } else if encoded.contains(&0) {
                bail!("{role} encodes to the structural NUL terminator");
            } else {
                output.extend_from_slice(&encoded);
            }
        }
        if !unsupported.is_empty() {
            bail!("{role} contains characters not encodable as CP932 and missing from the font map: {}", unsupported.join(", "));
        }
        Ok(output)
    }
}

pub fn scan_translation_root(
    translation_root: &Path,
    rom_path: &Path,
    ttf_path: &Path,
    existing_path: Option<&Path>,
    output: &Path,
    overwrite: bool,
) -> Result<FontScanReport> {
    let rom = fs::read(rom_path)
        .with_context(|| format!("failed to read KANJI1 ROM {}", rom_path.display()))?;
    ensure!(
        rom.len() == KANJI1_SIZE,
        "KANJI1 ROM must be exactly 0x20000 bytes, got 0x{:X}",
        rom.len()
    );
    let ttf =
        fs::read(ttf_path).with_context(|| format!("failed to read TTF {}", ttf_path.display()))?;
    let source_rom_sha256 = sha256_hex(&rom);
    let ttf_sha256 = sha256_hex(&ttf);
    let (translations, json_files) = read_translations(translation_root)?;
    let mut counts = BTreeMap::<char, usize>::new();
    let mut reserved_carriers = HashSet::<char>::new();
    let mut entries_scanned = 0usize;
    for translation in &translations {
        for entry in &translation.entries {
            entries_scanned += 1;
            scan_editable_chars(&entry.message, &mut counts)?;
            reserve_source_carriers(&entry.message, &mut reserved_carriers)?;
            if let Some(name) = &entry.name {
                scan_editable_chars(name, &mut counts)?;
                for character in name.chars() {
                    if character != '\0' && !character.is_control() {
                        if let Some((_, _, slot)) = cp932_slot(character) {
                            if slot_is_carrier(slot) {
                                reserved_carriers.insert(character);
                            }
                        }
                    }
                }
            }
            reserve_source_carriers(&entry.scr_msg, &mut reserved_carriers)?;
            if let Some(name) = &entry.scr_name {
                reserve_source_carriers(name, &mut reserved_carriers)?;
            }
        }
    }
    let existing_path = existing_path.or_else(|| output.is_file().then_some(output));
    let (mut map, repaired_mappings) =
        if let Some(path) = existing_path.filter(|path| path.is_file()) {
            let (loaded, repaired) = FontMap::load_for_scan(path)?;
            ensure!(
                loaded.source_rom_sha256 == source_rom_sha256,
                "existing font map was built from a different KANJI1 ROM"
            );
            ensure!(
                loaded.ttf_sha256 == ttf_sha256,
                "existing font map was built from a different TTF"
            );
            (loaded, repaired)
        } else {
            (
                FontMap {
                    format: FONT_MAP_FORMAT.to_owned(),
                    version: FONT_MAP_VERSION,
                    mapping_id: MAPPING_ID.to_owned(),
                    source_rom_sha256: source_rom_sha256.clone(),
                    ttf_sha256: ttf_sha256.clone(),
                    rom_size: KANJI1_SIZE,
                    carrier_policy: CARRIER_POLICY.to_owned(),
                    entries: Vec::new(),
                },
                0,
            )
        };
    let old_mapping_count = map.entries.len();
    let mut existing_chars = HashSet::new();
    let mut existing_carriers = HashSet::new();
    let mut existing_slots = HashSet::new();
    for entry in &mut map.entries {
        let character = one_char(&entry.character, "mapping character")?;
        let carrier = one_char(&entry.carrier, "mapping carrier")?;
        existing_chars.insert(character);
        existing_carriers.insert(carrier);
        existing_slots.insert(entry.slot);
        entry.usage_count = counts.get(&character).copied().unwrap_or(entry.usage_count);
    }
    let mut candidates = carrier_candidates(&rom, &reserved_carriers, &existing_carriers)?;
    let mut missing = Vec::new();
    let mut required = 0usize;
    let mut requested: Vec<(char, usize)> = counts
        .iter()
        .filter_map(|(&character, &count)| {
            if !requires_font_mapping(character) || existing_chars.contains(&character) {
                None
            } else {
                Some((character, count))
            }
        })
        .collect();
    requested.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    for (character, count) in requested {
        required += 1;
        if existing_chars.contains(&character) {
            continue;
        }
        let candidate = candidates
            .iter()
            .position(|candidate| !existing_slots.contains(&candidate.slot));
        let Some(position) = candidate else {
            missing.push((character, count));
            continue;
        };
        let candidate = candidates.remove(position);
        existing_chars.insert(character);
        existing_carriers.insert(candidate.carrier);
        existing_slots.insert(candidate.slot);
        map.entries.push(FontMapEntry {
            character: character.to_string(),
            carrier: candidate.carrier.to_string(),
            carrier_cp932: hex_encode(&candidate.bytes),
            jis_ku: candidate.ku,
            jis_ten: candidate.ten,
            slot: candidate.slot,
            rom_offset: candidate.slot * KANJI1_GLYPH_BYTES,
            kind: "substitute".to_owned(),
            usage_count: count,
        });
    }
    if !missing.is_empty() {
        let details = missing
            .iter()
            .map(|(character, count)| {
                format!(
                    "{character} (U+{:04X}, {count} occurrence(s))",
                    u32::from(*character)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "KANJI1 carrier capacity exhausted; missing {} character(s): {details}",
            missing.len()
        );
    }
    map.validate()?;
    map.save(output, overwrite)?;
    Ok(FontScanReport {
        json_files,
        entries_scanned,
        required_characters: required,
        new_mappings: map.entries.len().saturating_sub(old_mapping_count),
        repaired_mappings,
        total_mappings: map.entries.len(),
        output: output.to_path_buf(),
    })
}

pub fn rebuild_kanji1(
    rom_path: &Path,
    map_path: &Path,
    ttf_path: &Path,
    output: &Path,
    atlas: Option<&Path>,
    overwrite: bool,
) -> Result<FontRebuildReport> {
    let source = fs::read(rom_path)
        .with_context(|| format!("failed to read KANJI1 ROM {}", rom_path.display()))?;
    ensure!(
        source.len() == KANJI1_SIZE,
        "KANJI1 ROM must be exactly 0x20000 bytes"
    );
    let ttf =
        fs::read(ttf_path).with_context(|| format!("failed to read TTF {}", ttf_path.display()))?;
    let map = FontMap::load(map_path)?;
    ensure!(
        map.source_rom_sha256 == sha256_hex(&source),
        "font map source ROM SHA-256 does not match input ROM"
    );
    ensure!(
        map.ttf_sha256 == sha256_hex(&ttf),
        "font map TTF SHA-256 does not match input TTF"
    );
    #[cfg(windows)]
    let renderer = windows_gdi::GdiGlyphRenderer::new(ttf_path, &ttf)?;
    #[cfg(not(windows))]
    let font = FontRef::try_from_slice(&ttf).context("failed to parse TTF")?;
    #[cfg(not(windows))]
    let scaled = font.as_scaled(16.0);
    let mut rebuilt = source.clone();
    let mut changed_slots = 0usize;
    let mut changed_bytes = 0usize;
    for entry in &map.entries {
        let character = one_char(&entry.character, "mapping character")?;
        #[cfg(windows)]
        let glyph = renderer.render(character)?;
        #[cfg(not(windows))]
        let glyph = render_glyph(&font, &scaled, character)?;
        let start = entry.slot * KANJI1_GLYPH_BYTES;
        let end = start + KANJI1_GLYPH_BYTES;
        ensure!(
            end <= rebuilt.len(),
            "mapping slot out of ROM bounds: 0x{:X}",
            entry.slot
        );
        if rebuilt[start..end] != glyph {
            changed_slots += 1;
            changed_bytes += rebuilt[start..end]
                .iter()
                .zip(glyph.iter())
                .filter(|(left, right)| left != right)
                .count();
            rebuilt[start..end].copy_from_slice(&glyph);
        }
    }
    write_output_file(output, &rebuilt, overwrite)?;
    let atlas_path = if let Some(path) = atlas {
        write_atlas_pgm(&rebuilt, path, overwrite)?;
        Some(path.to_path_buf())
    } else {
        None
    };
    Ok(FontRebuildReport {
        mappings: map.entries.len(),
        changed_slots,
        changed_bytes,
        output: output.to_path_buf(),
        atlas: atlas_path,
    })
}

#[cfg(not(windows))]
fn render_glyph<F: Font>(
    font: &F,
    scaled: &ab_glyph::PxScaleFont<&F>,
    character: char,
) -> Result<[u8; KANJI1_GLYPH_BYTES]> {
    let glyph_id = font.glyph_id(character);
    ensure!(
        glyph_id.0 != 0,
        "TTF has no glyph for {character} (U+{:04X})",
        u32::from(character)
    );
    let glyph = glyph_id.with_scale_and_position(16.0, point(1.0, scaled.ascent()));
    let Some(outlined) = font.outline_glyph(glyph) else {
        bail!(
            "TTF glyph outline is missing for {character} (U+{:04X})",
            u32::from(character)
        );
    };
    let bounds = outlined.px_bounds();
    let mut bitmap = [0u8; KANJI1_GLYPH_BYTES];
    outlined.draw(|x, y, coverage| {
        let px = bounds.min.x as i32 + x as i32;
        let py = bounds.min.y as i32 + y as i32;
        if (0..16).contains(&px) && (0..16).contains(&py) && coverage >= 0.5 {
            bitmap[py as usize * 2 + px as usize / 8] |= 0x80 >> (px as usize & 7);
        }
    });
    ensure!(
        bitmap.iter().any(|byte| *byte != 0),
        "TTF glyph rendered blank for {character} (U+{:04X})",
        u32::from(character)
    );
    Ok(bitmap)
}

#[cfg(windows)]
mod windows_gdi {
    use super::*;
    use std::ffi::{c_void, OsStr};
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use ttf_parser::{name_id, Face, PlatformId};

    const FR_PRIVATE: u32 = 0x10;
    const DEFAULT_CHARSET: u32 = 1;
    const OUT_TT_PRECIS: u32 = 4;
    const NONANTIALIASED_QUALITY: u32 = 3;
    const FW_NORMAL: i32 = 400;
    const GGO_BITMAP: u32 = 1;
    const GGO_GLYPH_INDEX: u32 = 0x80;
    const GGI_MARK_NONEXISTING_GLYPHS: u32 = 1;
    const GDI_ERROR: u32 = u32::MAX;

    type Handle = *mut c_void;

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct GlyphMetrics {
        black_box_x: u32,
        black_box_y: u32,
        glyph_origin: Point,
        cell_inc_x: i16,
        cell_inc_y: i16,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Fixed {
        fract: u16,
        value: i16,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Mat2 {
        m11: Fixed,
        m12: Fixed,
        m21: Fixed,
        m22: Fixed,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
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
        digitized_aspect_x: i32,
        digitized_aspect_y: i32,
        first_char: u16,
        last_char: u16,
        default_char: u16,
        break_char: u16,
        italic: u8,
        underlined: u8,
        struck_out: u8,
        pitch_and_family: u8,
        char_set: u8,
    }

    #[link(name = "gdi32")]
    unsafe extern "system" {
        #[link_name = "AddFontResourceExW"]
        fn add_font_resource_ex_w(path: *const u16, flags: u32, reserved: Handle) -> i32;
        #[link_name = "RemoveFontResourceExW"]
        fn remove_font_resource_ex_w(path: *const u16, flags: u32, reserved: Handle) -> i32;
        #[link_name = "CreateCompatibleDC"]
        fn create_compatible_dc(dc: Handle) -> Handle;
        #[link_name = "DeleteDC"]
        fn delete_dc(dc: Handle) -> i32;
        #[link_name = "CreateFontW"]
        fn create_font_w(
            height: i32,
            width: i32,
            escapement: i32,
            orientation: i32,
            weight: i32,
            italic: u32,
            underline: u32,
            strike_out: u32,
            char_set: u32,
            output_precision: u32,
            clip_precision: u32,
            quality: u32,
            pitch_and_family: u32,
            face: *const u16,
        ) -> Handle;
        #[link_name = "SelectObject"]
        fn select_object(dc: Handle, object: Handle) -> Handle;
        #[link_name = "DeleteObject"]
        fn delete_object(object: Handle) -> i32;
        #[link_name = "GetTextMetricsW"]
        fn get_text_metrics_w(dc: Handle, metrics: *mut TextMetricW) -> i32;
        #[link_name = "GetGlyphIndicesW"]
        fn get_glyph_indices_w(
            dc: Handle,
            text: *const u16,
            count: i32,
            glyphs: *mut u16,
            flags: u32,
        ) -> u32;
        #[link_name = "GetGlyphOutlineW"]
        fn get_glyph_outline_w(
            dc: Handle,
            character: u32,
            format: u32,
            metrics: *mut GlyphMetrics,
            buffer_size: u32,
            buffer: *mut c_void,
            matrix: *const Mat2,
        ) -> u32;
    }

    struct PrivateFont {
        path: Vec<u16>,
    }

    impl PrivateFont {
        fn add(path: &Path) -> Result<Self> {
            let wide = wide_null(path.as_os_str());
            let added =
                unsafe { add_font_resource_ex_w(wide.as_ptr(), FR_PRIVATE, ptr::null_mut()) };
            ensure!(added > 0, "failed to load private TTF {}", path.display());
            Ok(Self { path: wide })
        }
    }

    impl Drop for PrivateFont {
        fn drop(&mut self) {
            unsafe {
                remove_font_resource_ex_w(self.path.as_ptr(), FR_PRIVATE, ptr::null_mut());
            }
        }
    }

    pub(super) struct GdiGlyphRenderer {
        dc: Handle,
        font: Handle,
        old_font: Handle,
        ascent: i32,
        _private_font: PrivateFont,
    }

    impl GdiGlyphRenderer {
        pub(super) fn new(ttf_path: &Path, ttf: &[u8]) -> Result<Self> {
            let family = ttf_family_name(ttf)?;
            let private_font = PrivateFont::add(ttf_path)?;
            let dc = unsafe { create_compatible_dc(ptr::null_mut()) };
            ensure!(!dc.is_null(), "failed to create GDI font device context");
            let face = wide_null(OsStr::new(&family));
            let font = unsafe {
                create_font_w(
                    -16,
                    0,
                    0,
                    0,
                    FW_NORMAL,
                    0,
                    0,
                    0,
                    DEFAULT_CHARSET,
                    OUT_TT_PRECIS,
                    0,
                    NONANTIALIASED_QUALITY,
                    0,
                    face.as_ptr(),
                )
            };
            if font.is_null() {
                unsafe {
                    delete_dc(dc);
                }
                bail!("failed to create 16px GDI font for TTF family {family:?}");
            }
            let old_font = unsafe { select_object(dc, font) };
            if old_font.is_null() {
                unsafe {
                    delete_object(font);
                    delete_dc(dc);
                }
                bail!("failed to select GDI font family {family:?}");
            }
            let mut metrics = TextMetricW::default();
            if unsafe { get_text_metrics_w(dc, &mut metrics) } == 0 {
                unsafe {
                    select_object(dc, old_font);
                    delete_object(font);
                    delete_dc(dc);
                }
                bail!("failed to read GDI font metrics for {family:?}");
            }
            ensure!(
                metrics.height <= 16,
                "GDI font family {family:?} produced {}px metrics for a 16px cell",
                metrics.height
            );
            Ok(Self {
                dc,
                font,
                old_font,
                ascent: metrics.ascent,
                _private_font: private_font,
            })
        }

        pub(super) fn render(&self, character: char) -> Result<[u8; KANJI1_GLYPH_BYTES]> {
            let mut utf16 = [0u16; 2];
            let encoded = character.encode_utf16(&mut utf16);
            ensure!(
                encoded.len() == 1,
                "16x16 GDI renderer only supports BMP characters: {character} (U+{:X})",
                u32::from(character)
            );
            let mut glyph_index = u16::MAX;
            let status = unsafe {
                get_glyph_indices_w(
                    self.dc,
                    utf16.as_ptr(),
                    1,
                    &mut glyph_index,
                    GGI_MARK_NONEXISTING_GLYPHS,
                )
            };
            ensure!(
                status != GDI_ERROR && glyph_index != u16::MAX,
                "TTF has no glyph for {character} (U+{:04X})",
                u32::from(character)
            );

            let matrix = Mat2 {
                m11: Fixed { fract: 0, value: 1 },
                m22: Fixed { fract: 0, value: 1 },
                ..Mat2::default()
            };
            let mut metrics = GlyphMetrics::default();
            let format = GGO_BITMAP | GGO_GLYPH_INDEX;
            let size = unsafe {
                get_glyph_outline_w(
                    self.dc,
                    u32::from(glyph_index),
                    format,
                    &mut metrics,
                    0,
                    ptr::null_mut(),
                    &matrix,
                )
            };
            ensure!(
                size != GDI_ERROR && size > 0,
                "GDI glyph outline is missing for {character} (U+{:04X})",
                u32::from(character)
            );
            let mut raw = vec![0u8; size as usize];
            let rendered = unsafe {
                get_glyph_outline_w(
                    self.dc,
                    u32::from(glyph_index),
                    format,
                    &mut metrics,
                    size,
                    raw.as_mut_ptr().cast(),
                    &matrix,
                )
            };
            ensure!(
                rendered != GDI_ERROR,
                "failed to rasterize {character} with GDI"
            );

            let left = metrics.glyph_origin.x;
            let top = self.ascent - metrics.glyph_origin.y;
            let right = left + metrics.black_box_x as i32;
            let bottom = top + metrics.black_box_y as i32;
            ensure!(
                left >= 0 && top >= 0 && right <= 16 && bottom <= 16,
                "TTF glyph {character} exceeds the 16x16 cell: ({left},{top})..({right},{bottom})"
            );
            let stride = (metrics.black_box_x as usize).div_ceil(32) * 4;
            let mut bitmap = [0u8; KANJI1_GLYPH_BYTES];
            for y in 0..metrics.black_box_y as usize {
                for x in 0..metrics.black_box_x as usize {
                    if raw[y * stride + x / 8] & (0x80 >> (x & 7)) == 0 {
                        continue;
                    }
                    let px = left as usize + x;
                    let py = top as usize + y;
                    bitmap[py * 2 + px / 8] |= 0x80 >> (px & 7);
                }
            }
            ensure!(
                bitmap.iter().any(|byte| *byte != 0),
                "TTF glyph rendered blank for {character} (U+{:04X})",
                u32::from(character)
            );
            Ok(bitmap)
        }
    }

    impl Drop for GdiGlyphRenderer {
        fn drop(&mut self) {
            unsafe {
                select_object(self.dc, self.old_font);
                delete_object(self.font);
                delete_dc(self.dc);
            }
        }
    }

    fn ttf_family_name(ttf: &[u8]) -> Result<String> {
        let face = Face::parse(ttf, 0).context("failed to parse TTF name table")?;
        for name_id in [name_id::TYPOGRAPHIC_FAMILY, name_id::FAMILY] {
            for english_only in [true, false] {
                if let Some(name) = face.names().into_iter().find_map(|name| {
                    (name.name_id == name_id
                        && name.platform_id == PlatformId::Windows
                        && (!english_only || name.language_id == 0x0409))
                        .then(|| name.to_string())
                        .flatten()
                }) {
                    ensure!(!name.contains('\0'), "TTF family name contains NUL");
                    return Ok(name);
                }
            }
        }
        bail!("TTF does not contain a Windows Unicode family name")
    }

    fn wide_null(value: &OsStr) -> Vec<u16> {
        value.encode_wide().chain(std::iter::once(0)).collect()
    }
}

fn write_atlas_pgm(rom: &[u8], path: &Path, overwrite: bool) -> Result<()> {
    let width = 64 * 16;
    let height = 64 * 16;
    let mut bytes = format!("P5\n{width} {height}\n255\n").into_bytes();
    bytes.reserve(width * height);
    for y in 0..height {
        let slot_row = y / 16;
        let glyph_y = y % 16;
        for x in 0..width {
            let slot = slot_row * 64 + x / 16;
            let glyph_x = x % 16;
            let row = &rom[slot * KANJI1_GLYPH_BYTES + glyph_y * 2
                ..slot * KANJI1_GLYPH_BYTES + glyph_y * 2 + 2];
            bytes.push(if row[glyph_x / 8] & (0x80 >> (glyph_x & 7)) != 0 {
                0
            } else {
                255
            });
        }
    }
    write_output_file(path, &bytes, overwrite)
}

#[derive(Debug, Clone)]
struct CarrierCandidate {
    carrier: char,
    bytes: Vec<u8>,
    ku: u8,
    ten: u8,
    slot: usize,
}

fn carrier_candidates(
    rom: &[u8],
    reserved: &HashSet<char>,
    existing: &HashSet<char>,
) -> Result<Vec<CarrierCandidate>> {
    let mut output = Vec::new();
    let mut slots = HashSet::new();
    for ku in 16..=47 {
        for ten in 1..=94 {
            let bytes = jis_to_cp932(ku, ten)?;
            let Some(decoded) =
                SHIFT_JIS.decode_without_bom_handling_and_without_replacement(&bytes)
            else {
                continue;
            };
            let mut chars = decoded.chars();
            let Some(carrier) = chars.next() else {
                continue;
            };
            if chars.next().is_some() || reserved.contains(&carrier) || existing.contains(&carrier)
            {
                continue;
            }
            let slot = kanji1_slot_from_jis_bytes(ku, ten);
            if slot >= KANJI1_SLOTS || !slots.insert(slot) {
                continue;
            }
            // Prefer all-zero slots so a carrier does not discard an existing
            // glyph when a blank physical slot is available.
            output.push(CarrierCandidate {
                carrier,
                bytes,
                ku,
                ten,
                slot,
            });
        }
    }
    output.sort_by_key(|candidate| {
        let blank = rom
            [candidate.slot * KANJI1_GLYPH_BYTES..(candidate.slot + 1) * KANJI1_GLYPH_BYTES]
            .iter()
            .all(|byte| *byte == 0);
        (!blank, candidate.slot)
    });
    Ok(output)
}

fn read_translations(root: &Path) -> Result<(Vec<MsgTranslationFile>, usize)> {
    let paths = collect_json_paths(root)?;
    let mut translations: Vec<MsgTranslationFile> = Vec::new();
    for path in &paths {
        let bytes = fs::read(path)
            .with_context(|| format!("failed to read translation JSON {}", path.display()))?;
        let value: serde_json::Value = match serde_json::from_slice(&bytes) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if value.get("_format").and_then(serde_json::Value::as_str)
            != Some("kuki-ayaka-msg-json-v1")
        {
            continue;
        }
        translations.push(
            serde_json::from_value(value)
                .with_context(|| format!("invalid MSG translation JSON {}", path.display()))?,
        );
    }
    if translations.is_empty() {
        bail!(
            "translation root contains no kuki-ayaka-msg-json-v1 JSON files: {}",
            root.display()
        );
    }
    translations.sort_by(|left, right| left.file.cmp(&right.file));
    let valid_count = translations.len();
    Ok((translations, valid_count))
}

fn collect_json_paths(root: &Path) -> Result<Vec<PathBuf>> {
    if root.is_file() {
        return Ok(vec![root.to_path_buf()]);
    }
    ensure!(
        root.is_dir(),
        "translation root does not exist: {}",
        root.display()
    );
    let mut stack = vec![root.to_path_buf()];
    let mut paths = Vec::new();
    while let Some(directory) = stack.pop() {
        for entry in fs::read_dir(&directory)
            .with_context(|| format!("failed to read {}", directory.display()))?
        {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                stack.push(entry.path());
            } else if kind.is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                paths.push(entry.path());
            }
        }
    }
    paths.sort();
    Ok(paths)
}

fn scan_editable_chars(value: &str, counts: &mut BTreeMap<char, usize>) -> Result<()> {
    let mut chars = value.char_indices().peekable();
    while let Some((offset, character)) = chars.next() {
        if character != '[' {
            if character == '\0' || character.is_control() {
                bail!(
                    "translation contains forbidden U+{:04X} at character offset {offset}",
                    u32::from(character)
                );
            }
            if requires_font_mapping(character) {
                *counts.entry(character).or_insert(0) += 1;
            }
            continue;
        }
        if chars.peek().is_some_and(|(_, next)| *next == '[') {
            chars.next();
            continue;
        }
        let remaining = &value[offset..];
        let bytes = remaining.as_bytes();
        if bytes.len() >= 4
            && bytes[3] == b']'
            && bytes[1].is_ascii_hexdigit()
            && bytes[2].is_ascii_hexdigit()
        {
            chars.next();
            chars.next();
            chars.next();
            continue;
        }
        bail!("malformed control token at character offset {offset}");
    }
    Ok(())
}

fn requires_font_mapping(character: char) -> bool {
    let codepoint = u32::from(character);
    let is_cjk_ideograph = matches!(
        codepoint,
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF
    );
    is_cjk_ideograph || SHIFT_JIS.encode(&character.to_string()).2
}

fn reserve_source_carriers(value: &str, reserved: &mut HashSet<char>) -> Result<()> {
    let mut chars = value.char_indices().peekable();
    while let Some((offset, character)) = chars.next() {
        if character == '[' {
            if chars.peek().is_some_and(|(_, next)| *next == '[') {
                chars.next();
                continue;
            }
            let remaining = &value[offset..];
            let bytes = remaining.as_bytes();
            if bytes.len() >= 4
                && bytes[3] == b']'
                && bytes[1].is_ascii_hexdigit()
                && bytes[2].is_ascii_hexdigit()
            {
                chars.next();
                chars.next();
                chars.next();
                continue;
            }
            bail!("malformed control token at character offset {offset}");
        }
        if let Some((_, _, slot)) = cp932_slot(character) {
            if slot_is_carrier(slot) {
                reserved.insert(character);
            }
        }
    }
    Ok(())
}

fn one_char(value: &str, role: &str) -> Result<char> {
    let mut chars = value.chars();
    let character = chars
        .next()
        .with_context(|| format!("{role} must contain exactly one Unicode scalar"))?;
    ensure!(
        chars.next().is_none(),
        "{role} must contain exactly one Unicode scalar"
    );
    Ok(character)
}

fn cp932_bytes(character: char, role: &str) -> Result<Vec<u8>> {
    let text = character.to_string();
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&text);
    ensure!(
        !had_errors && encoded.len() == 2,
        "{role} {:?} is not a two-byte CP932 character",
        character
    );
    Ok(encoded.into_owned())
}

fn cp932_slot(character: char) -> Option<(u8, u8, usize)> {
    let bytes = cp932_bytes(character, "carrier").ok()?;
    let (ku, ten) = jis_from_cp932(&bytes).ok()?;
    Some((ku, ten, kanji1_slot_from_jis_bytes(ku, ten)))
}

fn jis_from_cp932(bytes: &[u8]) -> Result<(u8, u8)> {
    ensure!(bytes.len() == 2, "CP932 character must use two bytes");
    let lead = bytes[0];
    let trail = bytes[1];
    ensure!(
        (0x81..=0x9F).contains(&lead) || (0xE0..=0xEF).contains(&lead),
        "invalid CP932 lead byte 0x{lead:02X}"
    );
    ensure!(
        (0x40..=0x7E).contains(&trail) || (0x80..=0xFC).contains(&trail),
        "invalid CP932 trail byte 0x{trail:02X}"
    );
    let mut first = if lead < 0xA0 {
        lead - 0x81
    } else {
        lead - 0xC1
    } * 2
        + 0x21;
    let ten = if trail >= 0x9F {
        first += 1;
        trail - 0x9F + 1
    } else if trail >= 0x80 {
        trail - 0x80 + 0x40
    } else {
        trail - 0x40 + 1
    };
    Ok((first - 0x20, ten))
}

fn jis_to_cp932(ku: u8, ten: u8) -> Result<Vec<u8>> {
    ensure!(
        (1..=94).contains(&ku) && (1..=94).contains(&ten),
        "invalid JIS coordinates {ku}-{ten}"
    );
    let first = ku + 0x20;
    let second = ten + 0x20;
    let lead = (first - 0x21) / 2 + 0x81 + if first >= 0x60 { 0x40 } else { 0 };
    let trail = if first % 2 == 1 {
        let value = second + 0x1F;
        if value >= 0x7F {
            value + 1
        } else {
            value
        }
    } else {
        second + 0x7E
    };
    Ok(vec![lead, trail])
}

/// Physical KANJI1 slot for the game's JIS->address routine (KANJI1 level).
pub fn kanji1_slot_from_jis_bytes(ku: u8, ten: u8) -> usize {
    let jis_first = ku + 0x20;
    let jis_second = ten + 0x20;
    let level_bit = if jis_first & 0x20 != 0 { 0x10 } else { 0 };
    let high =
        ((jis_first & 0x0F) | (jis_second & 0x60) | level_bit) << 1 | ((jis_second >> 4) & 1);
    (usize::from(high) << 4) | usize::from(jis_second & 0x0F)
}

fn slot_is_carrier(slot: usize) -> bool {
    slot < KANJI1_SLOTS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jis_and_physical_slot_match_runtime_examples() {
        assert_eq!(jis_from_cp932(&[0x88, 0x9F]).unwrap(), (16, 1));
        assert_eq!(kanji1_slot_from_jis_bytes(16, 1), 0x601);
        assert_eq!(jis_from_cp932(&[0x8A, 0xA3]).unwrap(), (20, 5));
        assert_eq!(kanji1_slot_from_jis_bytes(20, 5), 0x685);
        assert_eq!(kanji1_slot_from_jis_bytes(32, 1), 0x401);
        assert_eq!(jis_from_cp932(&[0xEA, 0xA3]).unwrap(), (84, 5));
        assert_eq!(kanji1_slot_from_jis_bytes(84, 5), 0x685);
    }

    #[test]
    fn mapped_plain_encoding_uses_carrier_bytes() {
        let map = FontMap {
            format: FONT_MAP_FORMAT.to_owned(),
            version: FONT_MAP_VERSION,
            mapping_id: "test".to_owned(),
            source_rom_sha256: String::new(),
            ttf_sha256: String::new(),
            rom_size: KANJI1_SIZE,
            carrier_policy: String::new(),
            entries: vec![FontMapEntry {
                character: "你".to_owned(),
                carrier: "亜".to_owned(),
                carrier_cp932: "889F".to_owned(),
                jis_ku: 16,
                jis_ten: 1,
                slot: 0x601,
                rom_offset: 0x601 * KANJI1_GLYPH_BYTES,
                kind: "substitute".to_owned(),
                usage_count: 1,
            }],
        };
        assert_eq!(
            map.encode_plain("你A", "message").unwrap(),
            [0x88, 0x9F, b'A']
        );
    }

    #[test]
    fn scan_maps_cp932_encodable_cjk_ideographs() {
        assert!(!SHIFT_JIS.encode("做").2);
        assert!(requires_font_mapping('做'));
        assert!(!requires_font_mapping('A'));

        let mut counts = BTreeMap::new();
        scan_editable_chars("做A[0E]", &mut counts).unwrap();
        assert_eq!(counts.get(&'做'), Some(&1));
        assert!(!counts.contains_key(&'A'));
    }

    #[test]
    fn repairs_v1_slots_with_the_start_zil_address_formula() {
        let carrier_bytes = jis_to_cp932(32, 1).unwrap();
        let carrier = SHIFT_JIS
            .decode_without_bom_handling_and_without_replacement(&carrier_bytes)
            .unwrap()
            .into_owned();
        let mut map = FontMap {
            format: FONT_MAP_FORMAT_V1.to_owned(),
            version: 1,
            mapping_id: "kanji1-dynamic-v1".to_owned(),
            source_rom_sha256: String::new(),
            ttf_sha256: String::new(),
            rom_size: KANJI1_SIZE,
            carrier_policy: CARRIER_POLICY.to_owned(),
            entries: vec![FontMapEntry {
                character: "测".to_owned(),
                carrier,
                carrier_cp932: hex_encode(&carrier_bytes),
                jis_ku: 32,
                jis_ten: 1,
                slot: 0x601,
                rom_offset: 0x601 * KANJI1_GLYPH_BYTES,
                kind: "substitute".to_owned(),
                usage_count: 1,
            }],
        };

        assert_eq!(map.repair_derived_mapping_fields().unwrap(), 1);
        assert_eq!(map.entries[0].slot, 0x401);
        assert_eq!(map.entries[0].rom_offset, 0x401 * KANJI1_GLYPH_BYTES);
        map.format = FONT_MAP_FORMAT.to_owned();
        map.version = FONT_MAP_VERSION;
        map.validate().unwrap();
    }
}
