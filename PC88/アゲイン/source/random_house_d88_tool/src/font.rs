//! Persistent KANJI1 mapping for the Random House two-byte renderer.
//!
//! The scan/rebuild/lock workflow follows the proven `kuki-disk-msg` font
//! module.  This engine's carrier is different: the lead byte selects one of
//! sixteen safe slot pages and the trail byte is the physical slot low byte.
//! The BASIC `SYMBOL` path consumes control-valued trail bytes before the
//! resident RST 18h hook can see them, so current mappings use printable ASCII
//! trails only and exclude the quoted-string delimiter.

use crate::pc88::native_byte_for_glyph;
use anyhow::{bail, ensure, Context, Result};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const TRANSLATION_FORMAT: &str = "random-house-basic-json-v1";
pub const FONT_MAP_FORMAT: &str = "random-house-kanji1-map-v1";
const FONT_MAP_VERSION: u32 = 6;
const MAPPING_ID: &str = "random-house-printable-trail-v5";
const LEGACY_MAPPING_ID: &str = "random-house-safe-lead-v1";
const LEGACY_V3_MAPPING_ID: &str = "random-house-printable-trail-v2";
const LEGACY_V4_MAPPING_ID: &str = "random-house-printable-trail-v3";
const LEGACY_V5_MAPPING_ID: &str = "random-house-printable-trail-v4";
const LEGACY_V1_CARRIER_POLICY: &str = "all multibyte or non-CP932 translation characters use the validated safe lead table; trail 00 and KANJI1 slots 080-BF are reserved; append-only lock";
const LEGACY_V2_CARRIER_POLICY: &str = "multibyte or non-CP932 translation characters use the validated safe lead table except native PC-88 CGROM glyphs, which keep their original single byte; trail 00 and KANJI1 slots 080-BF are reserved; append-only lock";
const LEGACY_V3_CARRIER_POLICY: &str = "multibyte or non-CP932 translation characters use the validated safe lead table except native PC-88 CGROM glyphs, which keep their original single byte; trail is printable ASCII 20-7E except quote 22; append-only lock";
const LEGACY_V4_CARRIER_POLICY: &str = "translated visible ASCII is normalized to fullwidth glyphs; multibyte or non-CP932 translation characters use the validated safe lead table except native PC-88 CGROM glyphs, which keep their original single byte; trail is printable ASCII 20-7E except quote 22 and reserved page-wait carrier F87E; append-only lock";
const LEGACY_V5_CARRIER_POLICY: &str = "visible ASCII 21-7E stays one byte and is rendered as fullwidth from reserved E020-E021/E023-E07E slots; other multibyte or non-CP932 translation characters use printable carriers from later safe-lead pages except quote 22 and reserved page-wait carrier F87E; native PC-88 CGROM glyphs keep their original single byte; append-only character set with policy migration";
const CARRIER_POLICY: &str = "visible ASCII 21-7E stays one byte and is rendered as fullwidth from reserved E020-E021/E023-E07E slots; other multibyte or non-CP932 translation characters use printable carriers from later safe-lead pages except quote 22, reserved native-ASCII carrier F87D, and reserved page-clear carrier F87E; native PC-88 CGROM glyphs keep their original single byte; append-only character set with policy migration";
pub const KANJI1_SIZE: usize = 0x20_000;
pub const KANJI1_SLOTS: usize = 4096;
pub const KANJI1_GLYPH_BYTES: usize = 0x20;
pub const SAFE_LEADS: [u8; 16] = [
    0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE6, 0xE7, 0xE8, 0xEB, 0xEC, 0xED, 0xEE, 0xEF, 0xF6, 0xF7, 0xF8,
];
const CGROM_SLOT_START: usize = 0x080;
const CGROM_SLOT_END: usize = 0x0C0;
const PRINTABLE_TRAIL_START: u8 = 0x20;
const PRINTABLE_TRAIL_END: u8 = 0x7E;
const BASIC_STRING_QUOTE: u8 = b'"';
const NATIVE_ASCII_SENTINEL_SLOT: usize = 0xF7D;
const PAGE_WAIT_SENTINEL_SLOT: usize = 0xF7E;
const DIRECT_ASCII_COUNT: usize = 0x7F - 0x21;
const PRINTABLE_TRAILS_PER_PAGE: usize =
    (PRINTABLE_TRAIL_END - PRINTABLE_TRAIL_START + 1) as usize - 1;
const PRINTABLE_CARRIER_CAPACITY: usize = SAFE_LEADS.len() * PRINTABLE_TRAILS_PER_PAGE - 2;
const GENERAL_CARRIER_CAPACITY: usize = PRINTABLE_CARRIER_CAPACITY - DIRECT_ASCII_COUNT;

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
    pub remapped_mappings: usize,
    pub total_mappings: usize,
    pub output: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontRebuildReport {
    pub mappings: usize,
    pub native_glyphs: usize,
    pub changed_slots: usize,
    pub changed_bytes: usize,
    pub output: PathBuf,
    pub atlas: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontEncodeReport {
    pub characters: usize,
    pub bytes: usize,
    pub output: PathBuf,
}

#[derive(Debug, Deserialize)]
struct TranslationFile {
    #[serde(rename = "_format")]
    format: String,
    entries: Vec<TranslationEntry>,
}

#[derive(Debug, Deserialize)]
struct TranslationEntry {
    #[serde(default)]
    scr_msg: Option<String>,
    message: String,
}

impl FontMap {
    pub fn load(path: &Path) -> Result<Self> {
        let bytes = fs::read(path)
            .with_context(|| format!("failed to read font mapping {}", path.display()))?;
        let map: Self = serde_json::from_slice(&bytes)
            .with_context(|| format!("invalid font mapping JSON {}", path.display()))?;
        map.validate()?;
        Ok(map)
    }

    pub fn save(&self, path: &Path, overwrite: bool) -> Result<()> {
        let mut bytes = serde_json::to_vec_pretty(self).context("failed to serialize font map")?;
        bytes.push(b'\n');
        write_output_file(path, &bytes, overwrite)
    }

    pub fn validate(&self) -> Result<()> {
        let current = self.is_current_policy();
        let legacy_v1 = self.version == 1
            && self.mapping_id == LEGACY_MAPPING_ID
            && self.carrier_policy == LEGACY_V1_CARRIER_POLICY;
        let legacy_v2 = self.version == 2
            && self.mapping_id == LEGACY_MAPPING_ID
            && self.carrier_policy == LEGACY_V2_CARRIER_POLICY;
        let legacy_v3 = self.version == 3
            && self.mapping_id == LEGACY_V3_MAPPING_ID
            && self.carrier_policy == LEGACY_V3_CARRIER_POLICY;
        let legacy_v4 = self.version == 4
            && self.mapping_id == LEGACY_V4_MAPPING_ID
            && self.carrier_policy == LEGACY_V4_CARRIER_POLICY;
        let legacy_v5 = self.version == 5
            && self.mapping_id == LEGACY_V5_MAPPING_ID
            && self.carrier_policy == LEGACY_V5_CARRIER_POLICY;
        ensure!(
            self.format == FONT_MAP_FORMAT
                && (current || legacy_v1 || legacy_v2 || legacy_v3 || legacy_v4 || legacy_v5),
            "unsupported font mapping format {:?}, version {}, id {:?}",
            self.format,
            self.version,
            self.mapping_id
        );
        ensure!(
            self.rom_size == KANJI1_SIZE,
            "font mapping ROM size must be 0x20000"
        );
        let mut characters = HashSet::new();
        let mut carriers = HashSet::new();
        let mut slots = HashSet::new();
        for entry in &self.entries {
            let character = one_char(&entry.character, "mapping character")?;
            ensure!(
                characters.insert(character),
                "duplicate mapping character {character:?}"
            );
            let bytes = decode_carrier(&entry.carrier)?;
            ensure!(
                carriers.insert(bytes),
                "duplicate mapping carrier {}",
                entry.carrier
            );
            ensure!(
                slots.insert(entry.slot),
                "duplicate mapping physical slot 0x{:03X}",
                entry.slot
            );
            ensure!(
                if current {
                    slot_is_assignable(entry.slot)
                } else {
                    legacy_slot_is_assignable(entry.slot)
                },
                "mapping slot 0x{:03X} is not allowed by mapping version {}",
                entry.slot,
                self.version,
            );
            ensure!(
                carrier_for_slot(entry.slot) == bytes,
                "mapping carrier does not match slot 0x{:03X}",
                entry.slot
            );
            ensure!(
                entry.rom_offset == entry.slot * KANJI1_GLYPH_BYTES,
                "mapping ROM offset does not match slot for {character:?}"
            );
        }
        if current {
            for byte in b'!'..=b'~' {
                let character = fullwidth_ascii(byte);
                let slot = direct_ascii_slot(character)
                    .expect("fullwidth ASCII must have a direct renderer slot");
                ensure!(
                    self.entries.iter().any(|entry| {
                        entry.character == character.to_string() && entry.slot == slot
                    }),
                    "current font map is missing direct fullwidth ASCII glyph {character:?} at slot 0x{slot:03X}"
                );
            }
            for entry in &self.entries {
                if entry.slot >> 8 == 0 {
                    let character = one_char(&entry.character, "mapping character")?;
                    ensure!(
                        direct_ascii_slot(character) == Some(entry.slot),
                        "page-zero slot 0x{:03X} is reserved for direct fullwidth ASCII",
                        entry.slot
                    );
                }
            }
        }
        Ok(())
    }

    fn is_current_policy(&self) -> bool {
        self.version == FONT_MAP_VERSION
            && self.mapping_id == MAPPING_ID
            && self.carrier_policy == CARRIER_POLICY
    }

    pub fn ensure_runtime_safe(&self) -> Result<()> {
        ensure!(
            self.is_current_policy(),
            "font map version {} uses a legacy carrier policy that can black-screen the game; run `font scan --existing <old-map> ...` and rebuild both the D88 and kanji1.rom from the migrated map",
            self.version
        );
        Ok(())
    }

    pub fn mapped_carrier(&self, character: char) -> Option<[u8; 2]> {
        self.entries
            .iter()
            .find(|entry| entry.character == character.to_string())
            .and_then(|entry| decode_carrier(&entry.carrier).ok())
    }

    /// Encode text for the installed renderer.  Every character that would use
    /// more than one CP932 byte must have a locked two-byte carrier.
    pub fn encode_plain(&self, value: &str, role: &str) -> Result<Vec<u8>> {
        self.ensure_runtime_safe()?;
        let mut output = Vec::new();
        let mut unsupported = Vec::new();
        for source_character in value.chars() {
            if source_character == '\0' || source_character.is_control() {
                bail!("{role} contains a forbidden Unicode control character");
            }
            if ('!'..='~').contains(&source_character) {
                output.push(source_character as u8);
                continue;
            }
            let character = normalize_display_character(source_character);
            if let Some(byte) = native_byte_for_glyph(character) {
                output.push(byte);
                continue;
            }
            if let Some(carrier) = self.mapped_carrier(character) {
                output.extend_from_slice(&carrier);
                continue;
            }
            let text = character.to_string();
            let (encoded, _, had_errors) = SHIFT_JIS.encode(&text);
            if had_errors || encoded.len() != 1 || SAFE_LEADS.contains(&encoded[0]) {
                unsupported.push(format!("{character} (U+{:04X})", u32::from(character)));
            } else if encoded[0] == 0 {
                bail!("{role} encodes to the structural NUL terminator");
            } else {
                output.push(encoded[0]);
            }
        }
        if !unsupported.is_empty() {
            bail!(
                "{role} contains characters missing from the font map: {}",
                unsupported.join(", ")
            );
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
    let ttf = fs::read(ttf_path)
        .with_context(|| format!("failed to read font {}", ttf_path.display()))?;
    let source_rom_sha256 = sha256_hex(&rom);
    let ttf_sha256 = sha256_hex(&ttf);
    let (translations, json_files) = read_translations(translation_root)?;
    let mut counts = BTreeMap::<char, usize>::new();
    let mut entries_scanned = 0usize;
    for translation in &translations {
        for entry in &translation.entries {
            entries_scanned += 1;
            if entry.scr_msg.as_deref() == Some(entry.message.as_str()) {
                continue;
            }
            scan_editable_chars(&entry.message, &mut counts)?;
        }
    }

    let existing_path = existing_path.or_else(|| output.is_file().then_some(output));
    let mut map = if let Some(path) = existing_path.filter(|path| path.is_file()) {
        let loaded = FontMap::load(path)?;
        ensure!(
            loaded.source_rom_sha256 == source_rom_sha256,
            "existing font map was built from a different KANJI1 ROM"
        );
        ensure!(
            loaded.ttf_sha256 == ttf_sha256,
            "existing font map was built from a different font"
        );
        loaded
    } else {
        FontMap {
            format: FONT_MAP_FORMAT.to_owned(),
            version: FONT_MAP_VERSION,
            mapping_id: MAPPING_ID.to_owned(),
            source_rom_sha256,
            ttf_sha256,
            rom_size: KANJI1_SIZE,
            carrier_policy: CARRIER_POLICY.to_owned(),
            entries: Vec::new(),
        }
    };
    let old_mapping_count = map.entries.len();
    let remapped_mappings = migrate_to_current_policy(&mut map)?;
    let mut existing_characters = HashSet::new();
    let mut existing_slots = HashSet::new();
    for entry in &mut map.entries {
        let character = one_char(&entry.character, "mapping character")?;
        existing_characters.insert(character);
        existing_slots.insert(entry.slot);
        entry.usage_count = counts.get(&character).copied().unwrap_or(0);
    }

    let required_characters = counts.len();
    let mut requested = counts
        .iter()
        .filter_map(|(&character, &count)| {
            (!existing_characters.contains(&character)).then_some((character, count))
        })
        .collect::<Vec<_>>();
    requested.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    let mut candidates = carrier_candidates(&existing_slots);
    let mut missing = Vec::new();
    for (character, usage_count) in requested {
        let Some(candidate) = candidates.pop() else {
            missing.push((character, usage_count));
            continue;
        };
        existing_characters.insert(character);
        existing_slots.insert(candidate);
        let carrier = carrier_for_slot(candidate);
        map.entries.push(FontMapEntry {
            character: character.to_string(),
            carrier: hex_encode(&carrier),
            slot: candidate,
            rom_offset: candidate * KANJI1_GLYPH_BYTES,
            kind: "substitute".to_owned(),
            usage_count,
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
        required_characters,
        new_mappings: map.entries.len().saturating_sub(old_mapping_count),
        remapped_mappings,
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
    let ttf = fs::read(ttf_path)
        .with_context(|| format!("failed to read font {}", ttf_path.display()))?;
    let map = FontMap::load(map_path)?;
    map.ensure_runtime_safe()?;
    ensure!(
        map.source_rom_sha256 == sha256_hex(&source),
        "font map source ROM SHA-256 does not match input ROM"
    );
    ensure!(
        map.ttf_sha256 == sha256_hex(&ttf),
        "font map font SHA-256 does not match input font"
    );
    #[cfg(windows)]
    let renderer = windows_gdi::GdiGlyphRenderer::new(ttf_path, &ttf)?;
    #[cfg(not(windows))]
    bail!("KANJI1 font rebuilding currently requires Windows GDI");

    let mut rebuilt = source.clone();
    let mut native_glyphs = 0usize;
    let mut changed_slots = 0usize;
    let mut changed_bytes = 0usize;
    for entry in &map.entries {
        let character = one_char(&entry.character, "mapping character")?;
        if native_byte_for_glyph(character).is_some() {
            native_glyphs += 1;
            continue;
        }
        #[cfg(windows)]
        let glyph = renderer.render(character)?;
        #[cfg(not(windows))]
        let glyph = [0u8; KANJI1_GLYPH_BYTES];
        let start = entry.slot * KANJI1_GLYPH_BYTES;
        let end = start + KANJI1_GLYPH_BYTES;
        ensure!(end <= rebuilt.len(), "mapping slot is outside KANJI1 ROM");
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
    let atlas = if let Some(path) = atlas {
        write_atlas_pgm(&rebuilt, path, overwrite)?;
        Some(path.to_path_buf())
    } else {
        None
    };
    Ok(FontRebuildReport {
        mappings: map.entries.len(),
        native_glyphs,
        changed_slots,
        changed_bytes,
        output: output.to_path_buf(),
        atlas,
    })
}

/// Encode a literal string as a low-level renderer probe. Formal game text
/// injection must additionally validate immutable source metadata.
pub fn encode_probe_text(
    map_path: &Path,
    value: &str,
    output: &Path,
    overwrite: bool,
) -> Result<FontEncodeReport> {
    let map = FontMap::load(map_path)?;
    let encoded = map.encode_plain(value, "probe text")?;
    write_output_file(output, &encoded, overwrite)?;
    Ok(FontEncodeReport {
        characters: value.chars().count(),
        bytes: encoded.len(),
        output: output.to_path_buf(),
    })
}

fn migrate_to_current_policy(map: &mut FontMap) -> Result<usize> {
    if map.is_current_policy() {
        return Ok(0);
    }
    let mut legacy_entries = std::mem::take(&mut map.entries);
    let non_ascii_entries = legacy_entries
        .iter()
        .filter(|entry| {
            one_char(&entry.character, "mapping character")
                .ok()
                .and_then(direct_ascii_slot)
                .is_none()
        })
        .count();
    ensure!(
        non_ascii_entries <= GENERAL_CARRIER_CAPACITY,
        "legacy font map has {non_ascii_entries} non-ASCII entries but the runtime-safe general carrier capacity is {GENERAL_CARRIER_CAPACITY}"
    );
    let mut remapped = 0usize;

    let mut entries = Vec::with_capacity(non_ascii_entries + DIRECT_ASCII_COUNT);
    for byte in b'!'..=b'~' {
        let character = fullwidth_ascii(byte);
        let slot = direct_ascii_slot(character).expect("direct ASCII slot must exist");
        let mut entry = if let Some(index) = legacy_entries
            .iter()
            .position(|entry| entry.character == character.to_string())
        {
            legacy_entries.remove(index)
        } else {
            FontMapEntry {
                character: character.to_string(),
                carrier: String::new(),
                slot,
                rom_offset: slot * KANJI1_GLYPH_BYTES,
                kind: "fullwidth_ascii".to_owned(),
                usage_count: 0,
            }
        };
        let carrier = carrier_for_slot(slot);
        if entry.slot != slot || decode_carrier(&entry.carrier).ok() != Some(carrier) {
            remapped += usize::from(!entry.carrier.is_empty());
        }
        entry.carrier = hex_encode(&carrier);
        entry.slot = slot;
        entry.rom_offset = slot * KANJI1_GLYPH_BYTES;
        entry.kind = "fullwidth_ascii".to_owned();
        entries.push(entry);
    }

    for (mut entry, slot) in legacy_entries.into_iter().zip(general_slots()) {
        let carrier = carrier_for_slot(slot);
        if entry.slot != slot || decode_carrier(&entry.carrier)? != carrier {
            remapped += 1;
        }
        entry.carrier = hex_encode(&carrier);
        entry.slot = slot;
        entry.rom_offset = slot * KANJI1_GLYPH_BYTES;
        entries.push(entry);
    }
    map.entries = entries;
    map.version = FONT_MAP_VERSION;
    map.mapping_id = MAPPING_ID.to_owned();
    map.carrier_policy = CARRIER_POLICY.to_owned();
    Ok(remapped)
}

fn printable_slots() -> impl Iterator<Item = usize> {
    (0..SAFE_LEADS.len()).flat_map(|page| {
        (PRINTABLE_TRAIL_START..=PRINTABLE_TRAIL_END)
            .filter(|trail| *trail != BASIC_STRING_QUOTE)
            .map(move |trail| (page << 8) | usize::from(trail))
            .filter(|slot| *slot != NATIVE_ASCII_SENTINEL_SLOT && *slot != PAGE_WAIT_SENTINEL_SLOT)
    })
}

fn general_slots() -> impl Iterator<Item = usize> {
    printable_slots().filter(|slot| slot >> 8 != 0)
}

fn carrier_candidates(existing: &HashSet<usize>) -> Vec<usize> {
    let mut candidates = general_slots()
        .filter(|slot| !existing.contains(slot))
        .collect::<Vec<_>>();
    // Allocation pops from the end; reverse to retain ascending physical slots.
    candidates.reverse();
    candidates
}

fn slot_is_assignable(slot: usize) -> bool {
    if slot >= KANJI1_SLOTS {
        return false;
    }
    let trail = (slot & 0xFF) as u8;
    (PRINTABLE_TRAIL_START..=PRINTABLE_TRAIL_END).contains(&trail)
        && trail != BASIC_STRING_QUOTE
        && slot != NATIVE_ASCII_SENTINEL_SLOT
        && slot != PAGE_WAIT_SENTINEL_SLOT
}

fn fullwidth_ascii(byte: u8) -> char {
    char::from_u32(u32::from(byte) + 0xFEE0).expect("ASCII fullwidth mapping is valid")
}

fn direct_ascii_slot(character: char) -> Option<usize> {
    let codepoint = u32::from(character).checked_sub(0xFEE0)?;
    let ascii = u8::try_from(codepoint).ok()?;
    if !(b'!'..=b'~').contains(&ascii) {
        return None;
    }
    let trail = if ascii <= b'"' { ascii - 1 } else { ascii };
    Some(usize::from(trail))
}

fn legacy_slot_is_assignable(slot: usize) -> bool {
    slot < KANJI1_SLOTS && slot & 0xFF != 0 && !(CGROM_SLOT_START..CGROM_SLOT_END).contains(&slot)
}

pub fn carrier_for_slot(slot: usize) -> [u8; 2] {
    [SAFE_LEADS[(slot >> 8) & 0x0F], (slot & 0xFF) as u8]
}

fn decode_carrier(value: &str) -> Result<[u8; 2]> {
    ensure!(
        value.len() == 4,
        "carrier must contain exactly four hex digits"
    );
    let bytes = value.as_bytes();
    Ok([decode_hex_pair(&bytes[..2])?, decode_hex_pair(&bytes[2..])?])
}

fn decode_hex_pair(value: &[u8]) -> Result<u8> {
    let text = std::str::from_utf8(value).context("carrier is not ASCII")?;
    u8::from_str_radix(text, 16).with_context(|| format!("invalid carrier hex {text:?}"))
}

fn read_translations(root: &Path) -> Result<(Vec<TranslationFile>, usize)> {
    let paths = collect_json_paths(root)?;
    let mut translations = Vec::new();
    for path in paths {
        let bytes = fs::read(&path)
            .with_context(|| format!("failed to read translation JSON {}", path.display()))?;
        let value: serde_json::Value = match serde_json::from_slice(&bytes) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if value.get("_format").and_then(serde_json::Value::as_str) != Some(TRANSLATION_FORMAT) {
            continue;
        }
        let translation: TranslationFile = serde_json::from_value(value)
            .with_context(|| format!("invalid translation JSON {}", path.display()))?;
        ensure!(
            translation.format == TRANSLATION_FORMAT,
            "translation format changed"
        );
        translations.push(translation);
    }
    if translations.is_empty() {
        bail!(
            "translation root contains no {TRANSLATION_FORMAT} files: {}",
            root.display()
        );
    }
    let count = translations.len();
    Ok((translations, count))
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
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
            {
                paths.push(entry.path());
            }
        }
    }
    paths.sort();
    Ok(paths)
}

fn scan_editable_chars(value: &str, counts: &mut BTreeMap<char, usize>) -> Result<()> {
    for (offset, source_character) in value.char_indices() {
        if source_character == '\0' || source_character.is_control() {
            bail!(
                "translation contains forbidden U+{:04X} at character offset {offset}",
                u32::from(source_character)
            );
        }
        let character = normalize_display_character(source_character);
        if requires_font_mapping(character) {
            *counts.entry(character).or_insert(0) += 1;
        }
    }
    Ok(())
}

fn requires_font_mapping(character: char) -> bool {
    let character = normalize_display_character(character);
    if native_byte_for_glyph(character).is_some() {
        return false;
    }
    let text = character.to_string();
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&text);
    had_errors || encoded.len() != 1
}

fn normalize_display_character(character: char) -> char {
    if ('!'..='~').contains(&character) {
        char::from_u32(u32::from(character) + 0xFEE0).expect("ASCII fullwidth mapping is valid")
    } else {
        character
    }
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

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:X}", Sha256::digest(bytes))
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}

fn write_output_file(path: &Path, bytes: &[u8], overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        bail!(
            "output already exists: {}; pass --overwrite",
            path.display()
        );
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    ensure!(
        parent.is_dir(),
        "output parent is not a directory: {}",
        parent.display()
    );
    fs::write(path, bytes).with_context(|| format!("failed to write {}", path.display()))
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
            ensure!(added > 0, "failed to load private font {}", path.display());
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
                unsafe { delete_dc(dc) };
                bail!("failed to create 16px GDI font for {family:?}");
            }
            let old_font = unsafe { select_object(dc, font) };
            if old_font.is_null() {
                unsafe {
                    delete_object(font);
                    delete_dc(dc);
                }
                bail!("failed to select GDI font {family:?}");
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
                "GDI font {family:?} produced {}px metrics for a 16px cell",
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
                "16x16 renderer only supports BMP characters: {character} (U+{:X})",
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
                "font has no glyph for {character} (U+{:04X})",
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
                "font glyph outline is missing for {character}"
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

            // Preserve the font's origin inside the 16x16 em square.
            let left = metrics.glyph_origin.x;
            let top = self.ascent - metrics.glyph_origin.y;
            let right = left + metrics.black_box_x as i32;
            let bottom = top + metrics.black_box_y as i32;
            ensure!(
                left >= 0 && top >= 0 && right <= 16 && bottom <= 16,
                "font glyph {character} exceeds 16x16: ({left},{top})..({right},{bottom})"
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
                "font glyph rendered blank for {character}"
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
        let face = Face::parse(ttf, 0).context("failed to parse font name table")?;
        for id in [name_id::TYPOGRAPHIC_FAMILY, name_id::FAMILY] {
            for english_only in [true, false] {
                if let Some(name) = face.names().into_iter().find_map(|name| {
                    (name.name_id == id
                        && name.platform_id == PlatformId::Windows
                        && (!english_only || name.language_id == 0x0409))
                        .then(|| name.to_string())
                        .flatten()
                }) {
                    ensure!(!name.contains('\0'), "font family name contains NUL");
                    return Ok(name);
                }
            }
        }
        bail!("font does not contain a Windows Unicode family name")
    }

    fn wide_null(value: &OsStr) -> Vec<u16> {
        value.encode_wide().chain(std::iter::once(0)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_map() -> FontMap {
        let mut entries = vec![FontMapEntry {
            character: "你".to_owned(),
            carrier: "E120".to_owned(),
            slot: 0x120,
            rom_offset: 0x120 * KANJI1_GLYPH_BYTES,
            kind: "substitute".to_owned(),
            usage_count: 1,
        }];
        entries.extend((b'!'..=b'~').map(|byte| {
            let character = fullwidth_ascii(byte);
            let slot = direct_ascii_slot(character).unwrap();
            FontMapEntry {
                character: character.to_string(),
                carrier: hex_encode(&carrier_for_slot(slot)),
                slot,
                rom_offset: slot * KANJI1_GLYPH_BYTES,
                kind: "fullwidth_ascii".to_owned(),
                usage_count: 0,
            }
        }));
        FontMap {
            format: FONT_MAP_FORMAT.to_owned(),
            version: FONT_MAP_VERSION,
            mapping_id: MAPPING_ID.to_owned(),
            source_rom_sha256: String::new(),
            ttf_sha256: String::new(),
            rom_size: KANJI1_SIZE,
            carrier_policy: CARRIER_POLICY.to_owned(),
            entries,
        }
    }

    #[test]
    fn printable_trail_mapping_matches_runtime_policy() {
        assert_eq!(carrier_for_slot(0x020), [0xE0, 0x20]);
        assert_eq!(carrier_for_slot(0x820), [0xEB, 0x20]);
        assert_eq!(carrier_for_slot(0xF7E), [0xF8, 0x7E]);
        assert!(!slot_is_assignable(0x000));
        assert!(slot_is_assignable(0x020));
        assert!(!slot_is_assignable(0x022));
        assert!(slot_is_assignable(0x07E));
        assert!(!slot_is_assignable(0x07F));
        assert!(!slot_is_assignable(0x110));
        assert!(!slot_is_assignable(NATIVE_ASCII_SENTINEL_SLOT));
        assert!(!slot_is_assignable(PAGE_WAIT_SENTINEL_SLOT));
        assert!(slot_is_assignable(0x120));
        assert_eq!(printable_slots().count(), PRINTABLE_CARRIER_CAPACITY);
        assert_eq!(general_slots().count(), GENERAL_CARRIER_CAPACITY);
        assert_eq!(DIRECT_ASCII_COUNT, 94);
        assert_eq!(direct_ascii_slot('！'), Some(0x020));
        assert_eq!(direct_ascii_slot('＂'), Some(0x021));
        assert_eq!(direct_ascii_slot('＃'), Some(0x023));
        assert_eq!(direct_ascii_slot('～'), Some(0x07E));
        assert_eq!(PRINTABLE_CARRIER_CAPACITY, 1502);
        assert_eq!(GENERAL_CARRIER_CAPACITY, 1408);
    }

    #[test]
    fn legacy_v1_maps_remain_loadable() {
        let mut map = sample_map();
        map.version = 1;
        map.mapping_id = LEGACY_MAPPING_ID.to_owned();
        map.carrier_policy = LEGACY_V1_CARRIER_POLICY.to_owned();
        map.validate().unwrap();
        assert!(map.ensure_runtime_safe().is_err());
    }

    #[test]
    fn legacy_v2_maps_remain_loadable_for_migration() {
        let mut map = sample_map();
        map.version = 2;
        map.mapping_id = LEGACY_MAPPING_ID.to_owned();
        map.carrier_policy = LEGACY_V2_CARRIER_POLICY.to_owned();
        map.validate().unwrap();
        assert!(map.ensure_runtime_safe().is_err());
    }

    #[test]
    fn legacy_v3_maps_remain_loadable_for_migration() {
        let mut map = sample_map();
        map.version = 3;
        map.mapping_id = LEGACY_V3_MAPPING_ID.to_owned();
        map.carrier_policy = LEGACY_V3_CARRIER_POLICY.to_owned();
        map.validate().unwrap();
        assert!(map.ensure_runtime_safe().is_err());
    }

    #[test]
    fn legacy_v4_maps_remain_loadable_for_migration() {
        let mut map = sample_map();
        map.version = 4;
        map.mapping_id = LEGACY_V4_MAPPING_ID.to_owned();
        map.carrier_policy = LEGACY_V4_CARRIER_POLICY.to_owned();
        map.validate().unwrap();
        assert!(map.ensure_runtime_safe().is_err());
        migrate_to_current_policy(&mut map).unwrap();
        map.validate().unwrap();
    }

    #[test]
    fn legacy_v5_maps_remain_loadable_for_migration() {
        let mut map = sample_map();
        map.version = 5;
        map.mapping_id = LEGACY_V5_MAPPING_ID.to_owned();
        map.carrier_policy = LEGACY_V5_CARRIER_POLICY.to_owned();
        map.validate().unwrap();
        assert!(map.ensure_runtime_safe().is_err());
        assert_eq!(migrate_to_current_policy(&mut map).unwrap(), 0);
        map.validate().unwrap();
    }

    #[test]
    fn legacy_maps_are_remapped_in_entry_order() {
        let mut map = sample_map();
        map.version = 2;
        map.mapping_id = LEGACY_MAPPING_ID.to_owned();
        map.carrier_policy = LEGACY_V2_CARRIER_POLICY.to_owned();
        map.entries[0].carrier = "E110".to_owned();
        map.entries[0].slot = 0x110;
        map.entries[0].rom_offset = 0x110 * KANJI1_GLYPH_BYTES;
        map.entries.push(FontMapEntry {
            character: "好".to_owned(),
            carrier: "E212".to_owned(),
            slot: 0x212,
            rom_offset: 0x212 * KANJI1_GLYPH_BYTES,
            kind: "substitute".to_owned(),
            usage_count: 1,
        });

        assert_eq!(migrate_to_current_policy(&mut map).unwrap(), 2);
        assert!(map.is_current_policy());
        assert_eq!(map.entries[0].carrier, "E020");
        assert_eq!(map.entries[1].carrier, "E021");
        map.validate().unwrap();
        assert_eq!(migrate_to_current_policy(&mut map).unwrap(), 0);
    }

    #[test]
    fn mapped_plain_uses_carrier_and_rejects_unmapped_multibyte() {
        let map = sample_map();
        map.validate().unwrap();
        assert_eq!(
            map.encode_plain("你A", "message").unwrap(),
            [0xE1, 0x20, b'A']
        );
        assert_eq!(map.mapped_carrier('Ａ'), Some([0xE0, 0x41]));
        assert!(map.encode_plain("好", "message").is_err());
    }

    #[test]
    fn all_multibyte_characters_require_mapping() {
        assert!(requires_font_mapping('汉'));
        assert!(requires_font_mapping('。'));
        assert!(requires_font_mapping('A'));
        assert!(!requires_font_mapping(' '));
        assert!(!requires_font_mapping('ｱ'));
        assert!(!requires_font_mapping('♥'));
        assert!(!requires_font_mapping('◆'));
    }

    #[test]
    fn translated_visible_ascii_normalizes_to_fullwidth() {
        assert_eq!(normalize_display_character('A'), 'Ａ');
        assert_eq!(normalize_display_character('1'), '１');
        assert_eq!(normalize_display_character('!'), '！');
        assert_eq!(normalize_display_character(' '), ' ');
        assert_eq!(normalize_display_character('中'), '中');
    }

    #[test]
    fn native_pc88_glyphs_override_legacy_font_map_entries() {
        let mut map = sample_map();
        map.version = 2;
        map.mapping_id = LEGACY_MAPPING_ID.to_owned();
        map.carrier_policy = LEGACY_V2_CARRIER_POLICY.to_owned();
        map.entries.push(FontMapEntry {
            character: "♥".to_owned(),
            carrier: "F6BF".to_owned(),
            slot: 0xDBF,
            rom_offset: 0xDBF * KANJI1_GLYPH_BYTES,
            kind: "substitute".to_owned(),
            usage_count: 6,
        });
        map.validate().unwrap();
        migrate_to_current_policy(&mut map).unwrap();
        assert_eq!(
            map.encode_plain("♥◆▄◣", "message").unwrap(),
            [0xE9, 0xEA, 0x83, 0xE5]
        );
    }
}
