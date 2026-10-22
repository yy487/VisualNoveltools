use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub type FontResult<T> = Result<T, String>;

pub const KANJI1_ROM_SIZE: usize = 0x20_000;
pub const GLYPH_SIDE: usize = 16;
pub const GLYPH_BYTES: usize = 32;

const FCG1_MAGIC: &[u8; 4] = b"FCG1";
const FCG1_RECORD_BYTES: usize = 4 + GLYPH_BYTES;
const EMBEDDED_MAPPING: &[u8] = include_bytes!("../assets/subs_cn_jp.json");
const EMBEDDED_GLYPHS: &[u8] = include_bytes!("../assets/glyphs_16_mono.bin");

pub type GlyphBitmap = [u8; GLYPH_BYTES];

/// A JSON key is a supported target/glyph. Its JSON value is only this
/// target's preferred carrier; it is never treated as a fixed assignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CharacterSupport {
    pub target: char,
    pub json_candidate_carrier: char,
    pub preferred_carrier: Option<char>,
    pub preferred_cp932: Option<u16>,
    pub preferred_jis: Option<u16>,
    pub preferred_rom_address: Option<u16>,
    pub preferred_rom_byte_offset: Option<usize>,
}

/// The concrete dynamic assignment shared by text encoding and ROM patching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappingUse {
    pub target: char,
    pub carrier: char,
    pub carrier_cp932: u16,
    pub jis: u16,
    pub rom_address: u16,
    pub rom_byte_offset: usize,
    pub used_preferred_carrier: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeDoubleByteUse {
    pub character: char,
    pub cp932: u16,
    pub jis: u16,
    pub rom_address: u16,
    pub rom_byte_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicFontPlan {
    pub mapping_used: Vec<MappingUse>,
    pub original_double_byte_codes: Vec<u16>,
    pub native_double_byte: Vec<NativeDoubleByteUse>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedText {
    pub bytes: Vec<u8>,
    pub mapping_used: Vec<MappingUse>,
    pub native_double_byte: Vec<NativeDoubleByteUse>,
}

#[derive(Debug, Clone)]
struct CandidateCarrier {
    character: char,
    cp932: u16,
    jis: u16,
    rom_address: u16,
    rom_byte_offset: usize,
}

#[derive(Debug, Clone)]
pub struct FontResources {
    supports: BTreeMap<char, CharacterSupport>,
    glyphs: BTreeMap<char, GlyphBitmap>,
    candidate_pool: Vec<CandidateCarrier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PatchedSlotManifest {
    pub target: char,
    pub target_unicode: String,
    pub carrier: char,
    pub carrier_cp932: String,
    pub jis: String,
    pub rom_address: String,
    pub rom_byte_offset: usize,
    pub used_preferred_carrier: bool,
    pub before_black_pixels: usize,
    pub after_black_pixels: usize,
    pub changed_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FontManifest {
    pub format: String,
    pub layout: String,
    pub source_size: usize,
    pub source_sha256: String,
    pub output_sha256: String,
    pub supported_targets: usize,
    pub source_double_byte_codes_preserved: usize,
    pub native_translation_slots_preserved: usize,
    pub mapping_used: Vec<MappingUse>,
    pub patched_slots: usize,
    pub changed_bytes: usize,
    pub untouched_bytes_preserved: bool,
    pub slots: Vec<PatchedSlotManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontBuild {
    pub rom: Vec<u8>,
    pub manifest: FontManifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewOptions {
    pub scale: usize,
    pub columns: usize,
    pub rows_per_page: usize,
    pub gap: usize,
    pub margin: usize,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        Self {
            scale: 8,
            columns: 5,
            rows_per_page: 4,
            gap: 16,
            margin: 24,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewPage {
    pub file_name: String,
    pub first_entry: usize,
    pub entries: usize,
    pub width: usize,
    pub height: usize,
    pub bmp: Vec<u8>,
}

impl FontResources {
    pub fn load_embedded() -> FontResult<Self> {
        let (supports, candidate_pool) = parse_support_and_candidates(EMBEDDED_MAPPING)?;
        let glyphs = parse_glyph_table(EMBEDDED_GLYPHS)?;
        let support_targets = supports.keys().copied().collect::<BTreeSet<_>>();
        let glyph_targets = glyphs.keys().copied().collect::<BTreeSet<_>>();
        if support_targets != glyph_targets {
            let missing = support_targets
                .difference(&glyph_targets)
                .copied()
                .collect::<Vec<_>>();
            let extra = glyph_targets
                .difference(&support_targets)
                .copied()
                .collect::<Vec<_>>();
            return Err(format!(
                "embedded FCG1 targets do not match supported JSON targets: missing={missing:?}, extra={extra:?}"
            ));
        }
        Ok(Self {
            supports,
            glyphs,
            candidate_pool,
        })
    }

    /// Plan one collision-free mapping for all 28 source MES files and all
    /// final translated messages. `original_double_byte_codes` must be the
    /// union of every original MES two-byte CP932 code encountered by the
    /// structure-aware parser.
    pub fn plan_dynamic_mapping<C, I, S>(
        &self,
        original_double_byte_codes: C,
        translated_texts: I,
    ) -> FontResult<DynamicFontPlan>
    where
        C: IntoIterator<Item = u16>,
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut original_codes = BTreeSet::new();
        let mut reserved_cp932 = BTreeSet::new();
        let mut reserved_addresses = BTreeSet::new();
        for cp932 in original_double_byte_codes {
            let (jis, address) = code_to_address(cp932).map_err(|error| {
                format!("original MES double-byte code {cp932:04X} is invalid: {error}")
            })?;
            let _ = jis;
            original_codes.insert(cp932);
            reserved_cp932.insert(cp932);
            reserved_addresses.insert(address);
        }

        let mut native_by_address = BTreeMap::<u16, NativeDoubleByteUse>::new();
        let mut unsupported_order = Vec::new();
        let mut unsupported_seen = BTreeSet::new();
        for text in translated_texts {
            for character in text.as_ref().chars() {
                if character == '\0' {
                    return Err("translated message contains NUL".to_string());
                }
                match native_cp932(character)? {
                    Some(NativeEncoding::Single(_)) => {}
                    Some(NativeEncoding::Double(cp932)) => {
                        let (jis, rom_address) = code_to_address(cp932).map_err(|error| {
                            format!(
                                "native U+{:04X} {character:?} CP932 {cp932:04X} cannot use KANJI1.ROM: {error}",
                                character as u32
                            )
                        })?;
                        reserved_cp932.insert(cp932);
                        reserved_addresses.insert(rom_address);
                        native_by_address
                            .entry(rom_address)
                            .or_insert(NativeDoubleByteUse {
                                character,
                                cp932,
                                jis,
                                rom_address,
                                rom_byte_offset: usize::from(rom_address) * 2,
                            });
                    }
                    None => {
                        if !self.supports.contains_key(&character) {
                            return Err(format!(
                                "U+{:04X} {character:?} is not native CP932 and has no embedded FCG1 glyph",
                                character as u32
                            ));
                        }
                        if unsupported_seen.insert(character) {
                            unsupported_order.push(character);
                        }
                    }
                }
            }
        }

        let mut assignments = BTreeMap::<char, CandidateCarrier>::new();
        let mut assigned_cp932 = BTreeSet::new();
        let mut assigned_addresses = BTreeSet::new();

        // First preserve every available per-target preference. A target whose
        // preferred slot is occupied must not steal another pending target's
        // free preferred slot during fallback allocation.
        for target in &unsupported_order {
            let support = &self.supports[target];
            if let (Some(character), Some(cp932), Some(jis), Some(rom_address), Some(byte_offset)) = (
                support.preferred_carrier,
                support.preferred_cp932,
                support.preferred_jis,
                support.preferred_rom_address,
                support.preferred_rom_byte_offset,
            ) {
                if carrier_is_available(
                    cp932,
                    rom_address,
                    &reserved_cp932,
                    &reserved_addresses,
                    &assigned_cp932,
                    &assigned_addresses,
                ) {
                    let candidate = CandidateCarrier {
                        character,
                        cp932,
                        jis,
                        rom_address,
                        rom_byte_offset: byte_offset,
                    };
                    assigned_cp932.insert(candidate.cp932);
                    assigned_addresses.insert(candidate.rom_address);
                    assignments.insert(*target, candidate);
                }
            }
        }

        for target in &unsupported_order {
            if assignments.contains_key(target) {
                continue;
            }
            let candidate = self
                .candidate_pool
                .iter()
                .find(|candidate| {
                    carrier_is_available(
                        candidate.cp932,
                        candidate.rom_address,
                        &reserved_cp932,
                        &reserved_addresses,
                        &assigned_cp932,
                        &assigned_addresses,
                    )
                })
                .cloned()
                .ok_or_else(|| {
                    format!(
                        "no collision-free KANJI1.ROM carrier remains for U+{:04X} {target:?}; required={}, candidate_pool={}, source_reserved={}, native_reserved={}",
                        *target as u32,
                        unsupported_order.len(),
                        self.candidate_pool.len(),
                        original_codes.len(),
                        native_by_address.len()
                    )
                })?;
            assigned_cp932.insert(candidate.cp932);
            assigned_addresses.insert(candidate.rom_address);
            assignments.insert(*target, candidate);
        }

        let mapping_used = unsupported_order
            .iter()
            .map(|target| {
                let candidate = &assignments[target];
                let support = &self.supports[target];
                MappingUse {
                    target: *target,
                    carrier: candidate.character,
                    carrier_cp932: candidate.cp932,
                    jis: candidate.jis,
                    rom_address: candidate.rom_address,
                    rom_byte_offset: candidate.rom_byte_offset,
                    used_preferred_carrier: support.preferred_cp932 == Some(candidate.cp932)
                        && support.preferred_rom_address == Some(candidate.rom_address),
                }
            })
            .collect::<Vec<_>>();
        let plan = DynamicFontPlan {
            mapping_used,
            original_double_byte_codes: original_codes.into_iter().collect(),
            native_double_byte: native_by_address.into_values().collect(),
        };
        check_plan_conflicts(&plan)?;
        Ok(plan)
    }

    /// Encode with the already-approved plan. No assignment is performed here,
    /// so the exact same mapping necessarily drives both scripts and the ROM.
    pub fn encode_ai1_text(&self, text: &str, plan: &DynamicFontPlan) -> FontResult<EncodedText> {
        check_plan_conflicts(plan)?;
        let mapping_by_target = plan
            .mapping_used
            .iter()
            .map(|entry| (entry.target, entry))
            .collect::<BTreeMap<_, _>>();
        let mut bytes = Vec::with_capacity(text.len());
        let mut mapped = BTreeMap::<u16, MappingUse>::new();
        let mut native = BTreeMap::<u16, NativeDoubleByteUse>::new();

        for character in text.chars() {
            if character == '\0' {
                return Err("translated message contains NUL".to_string());
            }
            match native_cp932(character)? {
                Some(NativeEncoding::Single(byte)) => bytes.push(byte),
                Some(NativeEncoding::Double(cp932)) => {
                    let (jis, rom_address) = code_to_address(cp932).map_err(|error| {
                        format!(
                            "native U+{:04X} {character:?} CP932 {cp932:04X} cannot use KANJI1.ROM: {error}",
                            character as u32
                        )
                    })?;
                    bytes.extend_from_slice(&cp932.to_be_bytes());
                    native.entry(rom_address).or_insert(NativeDoubleByteUse {
                        character,
                        cp932,
                        jis,
                        rom_address,
                        rom_byte_offset: usize::from(rom_address) * 2,
                    });
                }
                None => {
                    if !self.supports.contains_key(&character) {
                        return Err(format!(
                            "U+{:04X} {character:?} is not native CP932 and has no embedded glyph",
                            character as u32
                        ));
                    }
                    let assignment = mapping_by_target.get(&character).ok_or_else(|| {
                        format!(
                            "dynamic font plan has no assignment for non-CP932 U+{:04X} {character:?}",
                            character as u32
                        )
                    })?;
                    bytes.extend_from_slice(&assignment.carrier_cp932.to_be_bytes());
                    mapped
                        .entry(assignment.rom_address)
                        .or_insert_with(|| (*assignment).clone());
                }
            }
        }

        let mapping_used = mapped.into_values().collect::<Vec<_>>();
        let native_double_byte = native.into_values().collect::<Vec<_>>();
        check_carrier_native_conflicts(
            &mapping_used,
            &plan.original_double_byte_codes,
            &native_double_byte,
        )?;
        Ok(EncodedText {
            bytes,
            mapping_used,
            native_double_byte,
        })
    }

    pub fn build_rom(&self, source_rom: &[u8], plan: &DynamicFontPlan) -> FontResult<FontBuild> {
        validate_rom(source_rom)?;
        check_plan_conflicts(plan)?;
        let mut output = source_rom.to_vec();
        let mut touched = vec![false; source_rom.len()];
        let mut slots = Vec::with_capacity(plan.mapping_used.len());
        let mut total_changed_bytes = 0usize;

        for assignment in &plan.mapping_used {
            if !self.supports.contains_key(&assignment.target) {
                return Err(format!(
                    "dynamic mapping contains unsupported U+{:04X} {:?}",
                    assignment.target as u32, assignment.target
                ));
            }
            let candidate = self
                .candidate_pool
                .iter()
                .find(|candidate| {
                    candidate.character == assignment.carrier
                        && candidate.cp932 == assignment.carrier_cp932
                        && candidate.jis == assignment.jis
                        && candidate.rom_address == assignment.rom_address
                        && candidate.rom_byte_offset == assignment.rom_byte_offset
                })
                .ok_or_else(|| {
                    format!(
                        "dynamic mapping for U+{:04X} {:?} does not use an embedded candidate carrier",
                        assignment.target as u32, assignment.target
                    )
                })?;
            let _ = candidate;
            let glyph = self.glyphs.get(&assignment.target).ok_or_else(|| {
                format!(
                    "embedded FCG1 table has no U+{:04X} {:?}",
                    assignment.target as u32, assignment.target
                )
            })?;
            let range = glyph_range(assignment.rom_address)?;
            if touched[range.clone()].iter().any(|value| *value) {
                return Err(format!(
                    "dynamic mapping repeats ROM address {:04X}",
                    assignment.rom_address
                ));
            }
            let before = &source_rom[range.clone()];
            let changed_bytes = before
                .iter()
                .zip(glyph.iter())
                .filter(|(left, right)| left != right)
                .count();
            output[range.clone()].copy_from_slice(glyph);
            touched[range.clone()].fill(true);
            total_changed_bytes = total_changed_bytes
                .checked_add(changed_bytes)
                .ok_or_else(|| "changed-byte count overflow".to_string())?;
            slots.push(PatchedSlotManifest {
                target: assignment.target,
                target_unicode: format!("U+{:04X}", assignment.target as u32),
                carrier: assignment.carrier,
                carrier_cp932: format!("{:04X}", assignment.carrier_cp932),
                jis: format!("{:04X}", assignment.jis),
                rom_address: format!("{:04X}", assignment.rom_address),
                rom_byte_offset: assignment.rom_byte_offset,
                used_preferred_carrier: assignment.used_preferred_carrier,
                before_black_pixels: black_pixel_count(before)?,
                after_black_pixels: black_pixel_count(glyph)?,
                changed_bytes,
            });
        }

        for (index, (before, after)) in source_rom.iter().zip(output.iter()).enumerate() {
            if !touched[index] && before != after {
                return Err(format!(
                    "internal rebuild error: untouched KANJI1.ROM byte {index:#x} changed"
                ));
            }
        }
        for assignment in &plan.mapping_used {
            let range = glyph_range(assignment.rom_address)?;
            if output[range] != self.glyphs[&assignment.target] {
                return Err(format!(
                    "internal rebuild error: U+{:04X} {:?} did not verify pixel-exactly",
                    assignment.target as u32, assignment.target
                ));
            }
        }

        let manifest = FontManifest {
            format: "FOXY PC-8801 KANJI1.ROM dynamic font patch".to_string(),
            layout: "131072-byte ROM; JIS-derived base; 16 rows x two adjacent bytes; MSB-left"
                .to_string(),
            source_size: source_rom.len(),
            source_sha256: sha256_hex(source_rom),
            output_sha256: sha256_hex(&output),
            supported_targets: self.supports.len(),
            source_double_byte_codes_preserved: plan.original_double_byte_codes.len(),
            native_translation_slots_preserved: plan.native_double_byte.len(),
            mapping_used: plan.mapping_used.clone(),
            patched_slots: slots.len(),
            changed_bytes: total_changed_bytes,
            untouched_bytes_preserved: true,
            slots,
        };
        Ok(FontBuild {
            rom: output,
            manifest,
        })
    }

    pub fn render_preview_pages(
        &self,
        rom: &[u8],
        plan: &DynamicFontPlan,
        options: &PreviewOptions,
    ) -> FontResult<Vec<PreviewPage>> {
        validate_rom(rom)?;
        check_plan_conflicts(plan)?;
        validate_preview_options(options)?;
        if plan.mapping_used.is_empty() {
            return Ok(Vec::new());
        }
        let per_page = options
            .columns
            .checked_mul(options.rows_per_page)
            .ok_or_else(|| "preview page capacity overflow".to_string())?;
        let mut ordered = plan.mapping_used.clone();
        ordered.sort_unstable_by_key(|entry| (entry.rom_address, entry.target));
        ordered
            .chunks(per_page)
            .enumerate()
            .map(|(page_index, entries)| {
                render_preview_page(rom, entries, options, page_index, "font_preview")
            })
            .collect()
    }
}

/// A stable set of native Japanese glyphs suitable for confirming the raw ROM
/// address formula and left/right row-byte order before any dynamic patch.
#[allow(dead_code)]
pub const DEFAULT_ROM_PROBE_TEXT: &str = "日本語漢字復習俺登録星画面文章少女会話選択表示行列確認";

/// Render readable nearest-neighbour pages directly from an unmodified ROM.
/// This path does not load the CN carrier mapping or FCG1 glyphs.
#[allow(dead_code)]
pub fn render_rom_probe_pages(
    rom: &[u8],
    native_characters: &str,
    options: &PreviewOptions,
) -> FontResult<Vec<PreviewPage>> {
    validate_rom(rom)?;
    validate_preview_options(options)?;
    let mut seen_addresses = BTreeSet::new();
    let mut entries = Vec::new();
    for character in native_characters.chars() {
        let cp932 = strict_cp932_pair(character).map_err(|error| {
            format!(
                "ROM probe U+{:04X} {character:?}: {error}",
                character as u32
            )
        })?;
        let (jis, rom_address) = code_to_address(cp932).map_err(|error| {
            format!(
                "ROM probe U+{:04X} {character:?} CP932 {cp932:04X}: {error}",
                character as u32
            )
        })?;
        if seen_addresses.insert(rom_address) {
            entries.push(MappingUse {
                target: character,
                carrier: character,
                carrier_cp932: cp932,
                jis,
                rom_address,
                rom_byte_offset: usize::from(rom_address) * 2,
                used_preferred_carrier: false,
            });
        }
    }
    if entries.is_empty() {
        return Err("ROM probe text contains no addressable two-byte CP932 glyphs".to_string());
    }
    let per_page = options
        .columns
        .checked_mul(options.rows_per_page)
        .ok_or_else(|| "ROM probe page capacity overflow".to_string())?;
    entries
        .chunks(per_page)
        .enumerate()
        .map(|(page_index, chunk)| {
            render_preview_page(rom, chunk, options, page_index, "rom_probe")
        })
        .collect()
}

pub fn check_plan_conflicts(plan: &DynamicFontPlan) -> FontResult<()> {
    check_carrier_native_conflicts(
        &plan.mapping_used,
        &plan.original_double_byte_codes,
        &plan.native_double_byte,
    )?;
    let mut targets = BTreeSet::new();
    let mut cp932_codes = BTreeSet::new();
    let mut addresses = BTreeSet::new();
    for mapping in &plan.mapping_used {
        if !targets.insert(mapping.target) {
            return Err(format!(
                "dynamic mapping repeats U+{:04X} {:?}",
                mapping.target as u32, mapping.target
            ));
        }
        if !cp932_codes.insert(mapping.carrier_cp932) {
            return Err(format!(
                "dynamic mapping repeats carrier CP932 {:04X}",
                mapping.carrier_cp932
            ));
        }
        if !addresses.insert(mapping.rom_address) {
            return Err(format!(
                "dynamic mapping repeats KANJI1.ROM address {:04X}",
                mapping.rom_address
            ));
        }
        let (jis, address) = code_to_address(mapping.carrier_cp932)?;
        if jis != mapping.jis
            || address != mapping.rom_address
            || usize::from(address) * 2 != mapping.rom_byte_offset
        {
            return Err(format!(
                "dynamic mapping metadata for U+{:04X} {:?} is inconsistent",
                mapping.target as u32, mapping.target
            ));
        }
    }
    Ok(())
}

pub fn check_carrier_native_conflicts(
    mapped: &[MappingUse],
    original_codes: &[u16],
    native: &[NativeDoubleByteUse],
) -> FontResult<()> {
    let original_addresses = original_codes
        .iter()
        .map(|cp932| code_to_address(*cp932).map(|(_, address)| (*cp932, address)))
        .collect::<FontResult<Vec<_>>>()?;
    let mut errors = Vec::new();
    for mapping in mapped {
        for (cp932, address) in &original_addresses {
            if *cp932 == mapping.carrier_cp932 || *address == mapping.rom_address {
                errors.push(format!(
                    "U+{:04X} {:?} carrier CP932 {:04X}/ROM {:04X} is still used by original MES code {cp932:04X}",
                    mapping.target as u32,
                    mapping.target,
                    mapping.carrier_cp932,
                    mapping.rom_address
                ));
            }
        }
        for entry in native {
            if entry.cp932 == mapping.carrier_cp932 || entry.rom_address == mapping.rom_address {
                errors.push(format!(
                    "U+{:04X} {:?} carrier {:?} CP932 {:04X} conflicts with native U+{:04X} {:?}",
                    mapping.target as u32,
                    mapping.target,
                    mapping.carrier,
                    mapping.carrier_cp932,
                    entry.character as u32,
                    entry.character
                ));
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("font slot conflict(s): {}", errors.join("; ")))
    }
}

pub fn mapping_used_json_bytes(plan: &DynamicFontPlan) -> FontResult<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(&plan.mapping_used)
        .map_err(|error| format!("failed to serialize mapping_used: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Load the standalone font tool's `mapping_used.json`. The returned plan has
/// no source/native reservation lists because allocation is already complete;
/// it is intended for MES encoding with exactly the same target-to-carrier
/// assignments. Every target, carrier, code, address, offset, uniqueness rule,
/// and preferred-carrier flag is revalidated against the embedded assets.
#[allow(dead_code)]
pub fn load_mapping_used(path: &Path) -> FontResult<DynamicFontPlan> {
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read mapping_used {}: {error}", path.display()))?;
    let mapping_used: Vec<MappingUse> = serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse mapping_used {}: {error}", path.display()))?;
    let resources = FontResources::load_embedded()?;
    let plan = DynamicFontPlan {
        mapping_used,
        original_double_byte_codes: Vec::new(),
        native_double_byte: Vec::new(),
    };
    check_plan_conflicts(&plan)?;
    for assignment in &plan.mapping_used {
        let support = resources.supports.get(&assignment.target).ok_or_else(|| {
            format!(
                "mapping_used contains unsupported U+{:04X} {:?}",
                assignment.target as u32, assignment.target
            )
        })?;
        if native_cp932(assignment.target)?.is_some() {
            return Err(format!(
                "mapping_used must not remap native CP932 U+{:04X} {:?}",
                assignment.target as u32, assignment.target
            ));
        }
        let candidate_is_valid = resources.candidate_pool.iter().any(|candidate| {
            candidate.character == assignment.carrier
                && candidate.cp932 == assignment.carrier_cp932
                && candidate.jis == assignment.jis
                && candidate.rom_address == assignment.rom_address
                && candidate.rom_byte_offset == assignment.rom_byte_offset
        });
        if !candidate_is_valid {
            return Err(format!(
                "mapping_used U+{:04X} {:?} does not use an embedded AI1-addressable candidate",
                assignment.target as u32, assignment.target
            ));
        }
        let expected_preferred = support.preferred_cp932 == Some(assignment.carrier_cp932)
            && support.preferred_rom_address == Some(assignment.rom_address);
        if assignment.used_preferred_carrier != expected_preferred {
            return Err(format!(
                "mapping_used U+{:04X} {:?} has an incorrect used_preferred_carrier flag",
                assignment.target as u32, assignment.target
            ));
        }
    }
    Ok(plan)
}

pub fn manifest_json_bytes(manifest: &FontManifest) -> FontResult<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|error| format!("failed to serialize font manifest: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn validate_rom(rom: &[u8]) -> FontResult<()> {
    if rom.len() != KANJI1_ROM_SIZE {
        return Err(format!(
            "KANJI1.ROM must be exactly {KANJI1_ROM_SIZE} bytes, got {}",
            rom.len()
        ));
    }
    Ok(())
}

pub fn strict_cp932_pair(character: char) -> FontResult<u16> {
    match native_cp932(character)? {
        Some(NativeEncoding::Double(cp932)) => Ok(cp932),
        _ => Err(format!(
            "U+{:04X} {character:?} is not one strict two-byte CP932 character",
            character as u32
        )),
    }
}

pub fn sjis_to_jis(sjis: u16) -> Option<u16> {
    let high = (sjis >> 8) as u8;
    let low = sjis as u8;
    if !((0x81..=0x9f).contains(&high) || (0xe0..=0xef).contains(&high))
        || !((0x40..=0x7e).contains(&low) || (0x80..=0xfc).contains(&low))
    {
        return None;
    }
    let mut code = i32::from(sjis);
    if code >= 0xe000 {
        code -= 0x4000;
    }
    code = ((((code & 0xff00) - 0x8100) << 1) | (code & 0x00ff)) & 0xffff;
    if code & 0xff >= 0x80 {
        code -= 1;
    }
    if code & 0xff >= 0x9e {
        code += 0x100 - 0x9e;
    } else {
        code -= 0x40;
    }
    Some((code + 0x2121) as u16)
}

pub fn jis_to_rom_address(code: u16) -> Option<u16> {
    let non_kanji = (0x2121..=0x217e).contains(&code)
        || (0x2221..=0x222e).contains(&code)
        || (0x2330..=0x2339).contains(&code)
        || (0x2341..=0x235a).contains(&code)
        || (0x2361..=0x237a).contains(&code)
        || (0x2421..=0x2473).contains(&code)
        || (0x2521..=0x2576).contains(&code)
        || (0x2621..=0x2638).contains(&code)
        || (0x2641..=0x2658).contains(&code)
        || (0x2721..=0x2741).contains(&code)
        || (0x2751..=0x2771).contains(&code);
    if non_kanji {
        return Some(((code & 0x0060) << 7) | ((code & 0x0700) << 1) | ((code & 0x001f) << 4));
    }
    if (0x3021..=0x4f53).contains(&code) {
        return Some(((code & 0x0060) << 9) | ((code & 0x1f00) << 1) | ((code & 0x001f) << 4));
    }
    None
}

pub fn read_rom_glyph(rom: &[u8], rom_address: u16) -> FontResult<GlyphBitmap> {
    validate_rom(rom)?;
    let range = glyph_range(rom_address)?;
    let mut glyph = [0u8; GLYPH_BYTES];
    glyph.copy_from_slice(&rom[range]);
    Ok(glyph)
}

pub fn glyph_rows(glyph: &GlyphBitmap) -> [u16; GLYPH_SIDE] {
    let mut rows = [0u16; GLYPH_SIDE];
    for (row, target) in rows.iter_mut().enumerate() {
        *target = u16::from_be_bytes([glyph[row * 2], glyph[row * 2 + 1]]);
    }
    rows
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeEncoding {
    Single(u8),
    Double(u16),
}

fn native_cp932(character: char) -> FontResult<Option<NativeEncoding>> {
    let text = character.to_string();
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&text);
    if had_errors {
        return Ok(None);
    }
    match encoded.len() {
        1 => Ok(Some(NativeEncoding::Single(encoded[0]))),
        2 => Ok(Some(NativeEncoding::Double(u16::from_be_bytes([
            encoded[0], encoded[1],
        ])))),
        length => Err(format!(
            "U+{:04X} {character:?} encoded to unexpected CP932 length {length}",
            character as u32
        )),
    }
}

fn code_to_address(cp932: u16) -> FontResult<(u16, u16)> {
    let jis = sjis_to_jis(cp932)
        .ok_or_else(|| format!("CP932 {cp932:04X} is outside standard two-byte JIS"))?;
    let address = jis_to_rom_address(jis)
        .ok_or_else(|| format!("CP932 {cp932:04X}/JIS {jis:04X} is outside KANJI1.ROM"))?;
    glyph_range(address)?;
    Ok((jis, address))
}

fn carrier_is_available(
    cp932: u16,
    address: u16,
    reserved_cp932: &BTreeSet<u16>,
    reserved_addresses: &BTreeSet<u16>,
    assigned_cp932: &BTreeSet<u16>,
    assigned_addresses: &BTreeSet<u16>,
) -> bool {
    !reserved_cp932.contains(&cp932)
        && !reserved_addresses.contains(&address)
        && !assigned_cp932.contains(&cp932)
        && !assigned_addresses.contains(&address)
}

fn parse_support_and_candidates(
    bytes: &[u8],
) -> FontResult<(BTreeMap<char, CharacterSupport>, Vec<CandidateCarrier>)> {
    let raw: BTreeMap<String, String> = serde_json::from_slice(bytes)
        .map_err(|error| format!("embedded support mapping is invalid UTF-8 JSON: {error}"))?;
    let mut supports = BTreeMap::new();
    let mut candidate_pool = Vec::with_capacity(raw.len());
    let mut carrier_owners = BTreeMap::<char, char>::new();
    let mut address_owners = BTreeMap::<u16, char>::new();
    for (target_text, carrier_text) in raw {
        let target = one_scalar("support target", &target_text)?;
        let carrier = one_scalar("candidate carrier", &carrier_text)?;
        let usable_candidate = strict_cp932_pair(carrier).ok().and_then(|cp932| {
            code_to_address(cp932)
                .ok()
                .map(|(jis, address)| (cp932, jis, address))
        });
        let (
            preferred_carrier,
            preferred_cp932,
            preferred_jis,
            preferred_rom_address,
            preferred_rom_byte_offset,
        ) = if let Some((cp932, jis, rom_address)) = usable_candidate {
            if let Some(previous) = carrier_owners.insert(carrier, target) {
                return Err(format!(
                        "support targets {previous:?} and {target:?} reuse candidate carrier {carrier:?}"
                    ));
            }
            if let Some(previous) = address_owners.insert(rom_address, target) {
                return Err(format!(
                        "support targets {previous:?} and {target:?} reuse ROM address {rom_address:04X}"
                    ));
            }
            let rom_byte_offset = usize::from(rom_address) * 2;
            candidate_pool.push(CandidateCarrier {
                character: carrier,
                cp932,
                jis,
                rom_address,
                rom_byte_offset,
            });
            (
                Some(carrier),
                Some(cp932),
                Some(jis),
                Some(rom_address),
                Some(rom_byte_offset),
            )
        } else {
            (None, None, None, None, None)
        };
        supports.insert(
            target,
            CharacterSupport {
                target,
                json_candidate_carrier: carrier,
                preferred_carrier,
                preferred_cp932,
                preferred_jis,
                preferred_rom_address,
                preferred_rom_byte_offset,
            },
        );
    }
    if supports.is_empty() {
        return Err("embedded support mapping is empty".to_string());
    }
    if candidate_pool.is_empty() {
        return Err("embedded mapping contains no AI1-addressable carrier candidates".to_string());
    }
    Ok((supports, candidate_pool))
}

fn parse_glyph_table(bytes: &[u8]) -> FontResult<BTreeMap<char, GlyphBitmap>> {
    if bytes.len() < 8 || &bytes[..4] != FCG1_MAGIC {
        return Err("embedded glyph table has invalid FCG1 magic".to_string());
    }
    let count = read_u32_le(bytes, 4)? as usize;
    let expected = 8usize
        .checked_add(
            count
                .checked_mul(FCG1_RECORD_BYTES)
                .ok_or_else(|| "FCG1 size overflow".to_string())?,
        )
        .ok_or_else(|| "FCG1 size overflow".to_string())?;
    if expected != bytes.len() {
        return Err(format!(
            "FCG1 declares {count} records ({expected} bytes), got {} bytes",
            bytes.len()
        ));
    }
    let mut glyphs = BTreeMap::new();
    for index in 0..count {
        let offset = 8 + index * FCG1_RECORD_BYTES;
        let codepoint = read_u32_le(bytes, offset)?;
        let character = char::from_u32(codepoint).ok_or_else(|| {
            format!("FCG1 record {index} contains invalid Unicode U+{codepoint:04X}")
        })?;
        let mut glyph = [0u8; GLYPH_BYTES];
        glyph.copy_from_slice(&bytes[offset + 4..offset + FCG1_RECORD_BYTES]);
        if glyph.iter().all(|byte| *byte == 0) {
            return Err(format!(
                "FCG1 record {index} U+{codepoint:04X} {character:?} is empty"
            ));
        }
        if glyphs.insert(character, glyph).is_some() {
            return Err(format!(
                "FCG1 record {index} duplicates U+{codepoint:04X} {character:?}"
            ));
        }
    }
    Ok(glyphs)
}

fn glyph_range(rom_address: u16) -> FontResult<std::ops::Range<usize>> {
    let start = usize::from(rom_address)
        .checked_mul(2)
        .ok_or_else(|| "glyph byte offset overflow".to_string())?;
    let end = start
        .checked_add(GLYPH_BYTES)
        .ok_or_else(|| "glyph extent overflow".to_string())?;
    if end > KANJI1_ROM_SIZE {
        return Err(format!(
            "ROM address {rom_address:04X} maps to {start:#x}..{end:#x}, outside KANJI1.ROM"
        ));
    }
    Ok(start..end)
}

fn black_pixel_count(glyph: &[u8]) -> FontResult<usize> {
    if glyph.len() != GLYPH_BYTES {
        return Err(format!(
            "16x16 monochrome glyph must be {GLYPH_BYTES} bytes, got {}",
            glyph.len()
        ));
    }
    Ok(glyph.iter().map(|byte| byte.count_ones() as usize).sum())
}

fn one_scalar(role: &str, text: &str) -> FontResult<char> {
    let mut characters = text.chars();
    let character = characters
        .next()
        .ok_or_else(|| format!("{role} is empty"))?;
    if characters.next().is_some() {
        return Err(format!("{role} {text:?} is not one Unicode scalar"));
    }
    Ok(character)
}

fn read_u32_le(bytes: &[u8], offset: usize) -> FontResult<u32> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| format!("truncated 32-bit value at {offset:#x}"))?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn validate_preview_options(options: &PreviewOptions) -> FontResult<()> {
    if !(4..=32).contains(&options.scale) {
        return Err(format!(
            "preview scale must be 4..=32 for readable nearest-neighbour output, got {}",
            options.scale
        ));
    }
    if options.columns == 0 || options.rows_per_page == 0 {
        return Err("preview columns and rows_per_page must be nonzero".to_string());
    }
    if options.gap < 8 {
        return Err(format!(
            "preview gap must be at least 8 pixels, got {}",
            options.gap
        ));
    }
    Ok(())
}

fn render_preview_page(
    rom: &[u8],
    entries: &[MappingUse],
    options: &PreviewOptions,
    page_index: usize,
    file_prefix: &str,
) -> FontResult<PreviewPage> {
    let label_scale = (options.scale / 4).max(1);
    let glyph_pixels = GLYPH_SIDE
        .checked_mul(options.scale)
        .ok_or_else(|| "preview glyph size overflow".to_string())?;
    let label_line = 7usize
        .checked_mul(label_scale)
        .and_then(|value| value.checked_add(5))
        .ok_or_else(|| "preview label size overflow".to_string())?;
    let label_height = label_line
        .checked_mul(4)
        .ok_or_else(|| "preview label height overflow".to_string())?;
    let cell_width = glyph_pixels
        .checked_add(options.gap * 2)
        .ok_or_else(|| "preview cell width overflow".to_string())?;
    let cell_height = glyph_pixels
        .checked_add(label_height)
        .and_then(|value| value.checked_add(options.gap * 2))
        .ok_or_else(|| "preview cell height overflow".to_string())?;
    let rows = entries.len().div_ceil(options.columns);
    let grid_width = options
        .columns
        .checked_mul(cell_width)
        .ok_or_else(|| "preview grid width overflow".to_string())?;
    let grid_height = rows
        .checked_mul(cell_height)
        .ok_or_else(|| "preview grid height overflow".to_string())?;
    let width = options
        .margin
        .checked_mul(2)
        .and_then(|value| value.checked_add(grid_width))
        .ok_or_else(|| "preview page width overflow".to_string())?;
    let height = options
        .margin
        .checked_mul(2)
        .and_then(|value| value.checked_add(grid_height))
        .ok_or_else(|| "preview page height overflow".to_string())?;
    let mut canvas = Canvas::new(width, height, [238, 241, 245])?;
    for (index, entry) in entries.iter().enumerate() {
        let glyph = read_rom_glyph(rom, entry.rom_address)?;
        let cell_x = options.margin + index % options.columns * cell_width;
        let cell_y = options.margin + index / options.columns * cell_height;
        let glyph_x = cell_x + options.gap;
        let glyph_y = cell_y + options.gap;
        canvas.draw_glyph(&glyph, glyph_x, glyph_y, options.scale);
        let label_y = glyph_y + glyph_pixels + 5;
        let labels = [
            format!("U+{:04X}", entry.target as u32),
            format!("S:{:04X}", entry.carrier_cp932),
            format!("J:{:04X}", entry.jis),
            format!("O:{:05X}", entry.rom_byte_offset),
        ];
        for (line, label) in labels.iter().enumerate() {
            canvas.draw_text(
                label,
                glyph_x,
                label_y + line * label_line,
                label_scale,
                [24, 28, 34],
            );
        }
    }
    let bmp = canvas.to_bmp()?;
    Ok(PreviewPage {
        file_name: format!("{file_prefix}_{:03}.bmp", page_index + 1),
        first_entry: page_index * options.columns * options.rows_per_page,
        entries: entries.len(),
        width,
        height,
        bmp,
    })
}

struct Canvas {
    width: usize,
    height: usize,
    rgb: Vec<u8>,
}

impl Canvas {
    fn new(width: usize, height: usize, color: [u8; 3]) -> FontResult<Self> {
        if width == 0 || height == 0 || width > i32::MAX as usize || height > i32::MAX as usize {
            return Err(format!("invalid preview canvas size {width}x{height}"));
        }
        let byte_count = width
            .checked_mul(height)
            .and_then(|value| value.checked_mul(3))
            .ok_or_else(|| "preview canvas allocation overflow".to_string())?;
        let mut rgb = vec![0u8; byte_count];
        for pixel in rgb.chunks_exact_mut(3) {
            pixel.copy_from_slice(&color);
        }
        Ok(Self { width, height, rgb })
    }

    fn set(&mut self, x: usize, y: usize, color: [u8; 3]) {
        if x < self.width && y < self.height {
            let offset = (y * self.width + x) * 3;
            self.rgb[offset..offset + 3].copy_from_slice(&color);
        }
    }

    fn fill_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: [u8; 3]) {
        for pixel_y in y..y.saturating_add(height).min(self.height) {
            for pixel_x in x..x.saturating_add(width).min(self.width) {
                self.set(pixel_x, pixel_y, color);
            }
        }
    }

    fn frame(&mut self, x: usize, y: usize, width: usize, height: usize, color: [u8; 3]) {
        if width == 0 || height == 0 {
            return;
        }
        self.fill_rect(x, y, width, 1, color);
        self.fill_rect(x, y + height - 1, width, 1, color);
        self.fill_rect(x, y, 1, height, color);
        self.fill_rect(x + width - 1, y, 1, height, color);
    }

    fn draw_glyph(&mut self, glyph: &GlyphBitmap, x: usize, y: usize, scale: usize) {
        let extent = GLYPH_SIDE * scale;
        self.fill_rect(x, y, extent, extent, [252, 252, 252]);
        for (row, bits) in glyph_rows(glyph).iter().enumerate() {
            for column in 0..GLYPH_SIDE {
                if bits & (0x8000 >> column) != 0 {
                    self.fill_rect(
                        x + column * scale,
                        y + row * scale,
                        scale,
                        scale,
                        [12, 12, 12],
                    );
                }
            }
        }
        if x > 0 && y > 0 {
            self.frame(x - 1, y - 1, extent + 2, extent + 2, [126, 132, 142]);
        }
    }

    fn draw_text(&mut self, text: &str, x: usize, y: usize, scale: usize, color: [u8; 3]) {
        let mut cursor = x;
        for character in text.chars() {
            for (row, bits) in tiny_font(character).iter().enumerate() {
                for column in 0..5 {
                    if bits & (0x10 >> column) != 0 {
                        self.fill_rect(
                            cursor + column * scale,
                            y + row * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
            cursor += 6 * scale;
        }
    }

    fn to_bmp(&self) -> FontResult<Vec<u8>> {
        let row_bytes = self
            .width
            .checked_mul(3)
            .ok_or_else(|| "BMP row size overflow".to_string())?;
        let stride = row_bytes
            .checked_add(3)
            .map(|value| value & !3)
            .ok_or_else(|| "BMP stride overflow".to_string())?;
        let pixel_bytes = stride
            .checked_mul(self.height)
            .ok_or_else(|| "BMP pixel size overflow".to_string())?;
        let file_size = 54usize
            .checked_add(pixel_bytes)
            .ok_or_else(|| "BMP file size overflow".to_string())?;
        if file_size > u32::MAX as usize {
            return Err(format!("BMP is too large: {file_size} bytes"));
        }
        let mut output = vec![0u8; file_size];
        output[0..2].copy_from_slice(b"BM");
        output[2..6].copy_from_slice(&(file_size as u32).to_le_bytes());
        output[10..14].copy_from_slice(&54u32.to_le_bytes());
        output[14..18].copy_from_slice(&40u32.to_le_bytes());
        output[18..22].copy_from_slice(&(self.width as i32).to_le_bytes());
        output[22..26].copy_from_slice(&(self.height as i32).to_le_bytes());
        output[26..28].copy_from_slice(&1u16.to_le_bytes());
        output[28..30].copy_from_slice(&24u16.to_le_bytes());
        output[34..38].copy_from_slice(&(pixel_bytes as u32).to_le_bytes());
        for file_y in 0..self.height {
            let source_y = self.height - 1 - file_y;
            let destination = 54 + file_y * stride;
            for x in 0..self.width {
                let source = (source_y * self.width + x) * 3;
                let target = destination + x * 3;
                output[target] = self.rgb[source + 2];
                output[target + 1] = self.rgb[source + 1];
                output[target + 2] = self.rgb[source];
            }
        }
        Ok(output)
    }
}

fn tiny_font(character: char) -> [u8; 7] {
    match character.to_ascii_uppercase() {
        '0' => [0x0e, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0e],
        '1' => [0x04, 0x0c, 0x04, 0x04, 0x04, 0x04, 0x0e],
        '2' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1f],
        '3' => [0x1e, 0x01, 0x01, 0x0e, 0x01, 0x01, 0x1e],
        '4' => [0x02, 0x06, 0x0a, 0x12, 0x1f, 0x02, 0x02],
        '5' => [0x1f, 0x10, 0x10, 0x1e, 0x01, 0x01, 0x1e],
        '6' => [0x06, 0x08, 0x10, 0x1e, 0x11, 0x11, 0x0e],
        '7' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0e, 0x11, 0x11, 0x0e, 0x11, 0x11, 0x0e],
        '9' => [0x0e, 0x11, 0x11, 0x0f, 0x01, 0x02, 0x0c],
        'A' => [0x0e, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'B' => [0x1e, 0x11, 0x11, 0x1e, 0x11, 0x11, 0x1e],
        'C' => [0x0f, 0x10, 0x10, 0x10, 0x10, 0x10, 0x0f],
        'D' => [0x1e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1e],
        'E' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x1f],
        'F' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x10],
        'G' => [0x0f, 0x10, 0x10, 0x13, 0x11, 0x11, 0x0f],
        'H' => [0x11, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'I' => [0x0e, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0e],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x12, 0x12, 0x0c],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1f],
        'M' => [0x11, 0x1b, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'P' => [0x1e, 0x11, 0x11, 0x1e, 0x10, 0x10, 0x10],
        'Q' => [0x0e, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0d],
        'R' => [0x1e, 0x11, 0x11, 0x1e, 0x14, 0x12, 0x11],
        'S' => [0x0f, 0x10, 0x10, 0x0e, 0x01, 0x01, 0x1e],
        'T' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0a, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0a],
        'X' => [0x11, 0x11, 0x0a, 0x04, 0x0a, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0a, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1f],
        '+' => [0x00, 0x04, 0x04, 0x1f, 0x04, 0x04, 0x00],
        ':' => [0x00, 0x04, 0x04, 0x00, 0x04, 0x04, 0x00],
        '-' => [0x00, 0x00, 0x00, 0x1f, 0x00, 0x00, 0x00],
        ' ' => [0; 7],
        _ => [0x1f, 0x01, 0x02, 0x04, 0x00, 0x04, 0x00],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_non_cp932_target(resources: &FontResources) -> char {
        resources
            .supports
            .values()
            .find(|entry| {
                entry.preferred_cp932.is_some() && native_cp932(entry.target).unwrap().is_none()
            })
            .map(|entry| entry.target)
            .expect("embedded support must contain non-CP932 targets")
    }

    #[test]
    fn address_formula_matches_confirmed_slots() {
        assert_eq!(sjis_to_jis(0x82a0), Some(0x2422));
        assert_eq!(sjis_to_jis(0x889f), Some(0x3021));
        assert_eq!(jis_to_rom_address(0x2121), Some(0x1210));
        assert_eq!(jis_to_rom_address(0x2422), Some(0x1820));
        assert_eq!(jis_to_rom_address(0x3021), Some(0x6010));
        assert_eq!(jis_to_rom_address(0x4f53), Some(0x9f30));
        assert_eq!(glyph_range(0x6010).unwrap(), 0xc020..0xc040);
    }

    #[test]
    fn embedded_fcg1_targets_match_support_keys() {
        let resources = FontResources::load_embedded().unwrap();
        assert_eq!(resources.supports.len(), 3025);
        assert_eq!(resources.glyphs.len(), 3025);
        let target = first_non_cp932_target(&resources);
        assert!(resources.glyphs[&target].iter().any(|byte| *byte != 0));
    }

    #[test]
    fn occupied_static_preference_falls_back_to_another_free_slot() {
        let resources = FontResources::load_embedded().unwrap();
        let target = first_non_cp932_target(&resources);
        let preferred = &resources.supports[&target];
        let preferred_cp932 = preferred.preferred_cp932.unwrap();
        let preferred_address = preferred.preferred_rom_address.unwrap();
        let plan = resources
            .plan_dynamic_mapping([preferred_cp932], [target.to_string()])
            .unwrap();
        let assigned = plan
            .mapping_used
            .iter()
            .find(|entry| entry.target == target)
            .unwrap();
        assert_ne!(assigned.carrier_cp932, preferred_cp932);
        assert_ne!(assigned.rom_address, preferred_address);
        assert!(!assigned.used_preferred_carrier);
    }

    #[test]
    fn native_translation_slot_is_reserved_before_dynamic_assignment() {
        let resources = FontResources::load_embedded().unwrap();
        let target = first_non_cp932_target(&resources);
        let preferred = &resources.supports[&target];
        let text = format!("{}{}", preferred.preferred_carrier.unwrap(), target);
        let plan = resources
            .plan_dynamic_mapping(Vec::<u16>::new(), [text])
            .unwrap();
        assert_ne!(
            plan.mapping_used
                .iter()
                .find(|entry| entry.target == target)
                .unwrap()
                .rom_address,
            preferred.preferred_rom_address.unwrap()
        );
    }

    #[test]
    fn one_plan_drives_text_bytes_and_one_32_byte_rom_patch() {
        let resources = FontResources::load_embedded().unwrap();
        let target = first_non_cp932_target(&resources);
        let text = target.to_string();
        let plan = resources
            .plan_dynamic_mapping(Vec::<u16>::new(), [&text])
            .unwrap();
        let assignment = plan
            .mapping_used
            .iter()
            .find(|entry| entry.target == target)
            .unwrap();
        let encoded = resources.encode_ai1_text(&text, &plan).unwrap();
        assert_eq!(encoded.bytes, assignment.carrier_cp932.to_be_bytes());

        let source = vec![0xa5; KANJI1_ROM_SIZE];
        let built = resources.build_rom(&source, &plan).unwrap();
        let range = glyph_range(assignment.rom_address).unwrap();
        assert_eq!(built.rom[range.clone()], resources.glyphs[&target]);
        for (index, (output_byte, source_byte)) in built.rom.iter().zip(&source).enumerate() {
            if !range.contains(&index) {
                assert_eq!(output_byte, source_byte, "byte {index:#x}");
            }
        }
        assert_eq!(built.manifest.patched_slots, 1);
        assert_eq!(built.manifest.mapping_used, plan.mapping_used);
    }

    #[test]
    fn malformed_rom_and_fcg1_are_rejected() {
        assert!(validate_rom(&vec![0; KANJI1_ROM_SIZE - 1]).is_err());
        assert!(parse_glyph_table(b"bad").is_err());
        let mut truncated = EMBEDDED_GLYPHS.to_vec();
        truncated.pop();
        assert!(parse_glyph_table(&truncated).is_err());
    }

    #[test]
    fn preview_is_clear_paged_bmp_with_encoding_labels() {
        let resources = FontResources::load_embedded().unwrap();
        let targets = resources
            .supports
            .values()
            .map(|entry| entry.target)
            .filter(|character| native_cp932(*character).unwrap().is_none())
            .take(3)
            .collect::<String>();
        let plan = resources
            .plan_dynamic_mapping(Vec::<u16>::new(), [&targets])
            .unwrap();
        let built = resources
            .build_rom(&vec![0; KANJI1_ROM_SIZE], &plan)
            .unwrap();
        let pages = resources
            .render_preview_pages(&built.rom, &plan, &PreviewOptions::default())
            .unwrap();
        assert_eq!(pages.len(), 1);
        assert_eq!(&pages[0].bmp[..2], b"BM");
        assert_eq!(pages[0].entries, 3);
        assert!(pages[0].width >= 800);
        assert!(pages[0].height >= 200);
    }

    #[test]
    fn raw_rom_probe_does_not_require_dynamic_mapping() {
        let pages = render_rom_probe_pages(
            &vec![0; KANJI1_ROM_SIZE],
            DEFAULT_ROM_PROBE_TEXT,
            &PreviewOptions::default(),
        )
        .unwrap();
        assert!(!pages.is_empty());
        assert_eq!(&pages[0].bmp[..2], b"BM");
        assert!(pages[0].file_name.starts_with("rom_probe_"));
    }

    #[test]
    fn mapping_and_manifest_json_are_utf8_with_lf() {
        let resources = FontResources::load_embedded().unwrap();
        let target = first_non_cp932_target(&resources);
        let text = target.to_string();
        let plan = resources
            .plan_dynamic_mapping(Vec::<u16>::new(), [&text])
            .unwrap();
        assert_eq!(mapping_used_json_bytes(&plan).unwrap().last(), Some(&b'\n'));
        let built = resources
            .build_rom(&vec![0; KANJI1_ROM_SIZE], &plan)
            .unwrap();
        let manifest = manifest_json_bytes(&built.manifest).unwrap();
        assert_eq!(manifest.last(), Some(&b'\n'));
        let value: serde_json::Value = serde_json::from_slice(&manifest).unwrap();
        assert_eq!(value["patched_slots"], 1);
        assert!(value.get("mapping_used").is_some());
    }

    #[test]
    fn mapping_used_file_loads_back_as_the_same_encoding_plan() {
        let resources = FontResources::load_embedded().unwrap();
        let target = first_non_cp932_target(&resources);
        let text = target.to_string();
        let plan = resources
            .plan_dynamic_mapping(Vec::<u16>::new(), [&text])
            .unwrap();
        let path = std::env::temp_dir().join(format!(
            "foxy-mapping-used-test-{}-{}.json",
            std::process::id(),
            target as u32
        ));
        fs::write(&path, mapping_used_json_bytes(&plan).unwrap()).unwrap();
        let loaded = load_mapping_used(&path).unwrap();
        let _ = fs::remove_file(&path);
        assert_eq!(loaded.mapping_used, plan.mapping_used);
        assert!(loaded.original_double_byte_codes.is_empty());
        assert!(loaded.native_double_byte.is_empty());
        assert_eq!(
            resources.encode_ai1_text(&text, &loaded).unwrap().bytes,
            resources.encode_ai1_text(&text, &plan).unwrap().bytes
        );
    }
}
