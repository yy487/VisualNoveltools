//! Game-adapter composition for PC-88 KANJI1 font planning and rebuilding.
//!
//! The caller supplies display strings parsed from the game's script and every
//! original double-byte CP932 code that must keep its existing glyph. One
//! mapping plan is shared by text encoding and ROM patching so that translations
//! cannot be encoded with a different carrier assignment than the ROM uses.

use std::collections::BTreeSet;

use serde::Serialize;
use vn_font::font_88::{DynamicFontPlan, EncodedText, FontBuild, FontManifest, FontResources};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Pc88FontReport {
    pub plan: DynamicFontPlan,
    pub manifest: FontManifest,
}

#[derive(Debug, Clone)]
pub struct Pc88FontOutput {
    /// Encoded strings are in the same order as the input `display_texts`.
    pub encoded_texts: Vec<EncodedText>,
    pub rom: Vec<u8>,
    pub generated_glyph_table: Option<Vec<u8>>,
    pub resources: FontResources,
    pub report: Pc88FontReport,
}

/// Builds a KANJI1 ROM and encodes its display strings with one shared plan.
///
/// `original_double_byte_codes` must come from every source string that will
/// remain present in the rebuilt scripts (including untranslated records).
/// The adapter's script parser is responsible for supplying display text only,
/// without bytecode and structural controls.
pub fn build_pc88_kanji1(
    source_rom: &[u8],
    original_double_byte_codes: &[u16],
    display_texts: &[String],
) -> Result<Pc88FontOutput> {
    build_pc88_kanji1_with_font_face(
        source_rom,
        original_double_byte_codes,
        display_texts,
        vn_font::font_88::DEFAULT_GLYPH_FONT_FACE,
    )
}

pub fn build_pc88_kanji1_with_font_face(
    source_rom: &[u8],
    original_double_byte_codes: &[u16],
    display_texts: &[String],
    font_face: &str,
) -> Result<Pc88FontOutput> {
    build_pc88_kanji1_with_font_face_and_glyph_table(
        source_rom,
        original_double_byte_codes,
        display_texts,
        font_face,
        None,
    )
}

/// Builds a KANJI1 ROM, optionally extending the embedded FCG1 with a reusable
/// generated-glyph sidecar before rendering any remaining missing characters.
pub fn build_pc88_kanji1_with_font_face_and_glyph_table(
    source_rom: &[u8],
    original_double_byte_codes: &[u16],
    display_texts: &[String],
    font_face: &str,
    additional_glyph_table: Option<&[u8]>,
) -> Result<Pc88FontOutput> {
    build_pc88_kanji1_with_font_face_and_glyph_table_and_reserved_jis_codes(
        source_rom,
        original_double_byte_codes,
        display_texts,
        font_face,
        additional_glyph_table,
        &[],
    )
}

/// Like `build_pc88_kanji1_with_font_face_and_glyph_table`, but keeps the
/// supplied JIS glyph slots out of the dynamic carrier pool. This is needed
/// when other live text paths still reach those slots through a shared table.
pub fn build_pc88_kanji1_with_font_face_and_glyph_table_and_reserved_jis_codes(
    source_rom: &[u8],
    original_double_byte_codes: &[u16],
    display_texts: &[String],
    font_face: &str,
    additional_glyph_table: Option<&[u8]>,
    reserved_jis_codes: &[u16],
) -> Result<Pc88FontOutput> {
    let mut resources = FontResources::load_embedded()
        .map_err(|error| format!("载入 PC-88 字库映射资源失败：{error}"))?;
    if let Some(glyph_table) = additional_glyph_table {
        resources
            .extend_glyphs_from_fcg1(glyph_table)
            .map_err(|error| format!("合并已有 FCG1 点阵表失败：{error}"))?;
    }
    resources
        .generate_missing_glyphs(display_texts.iter(), font_face)
        .map_err(|error| format!("现场绘制 PC-88 缺失字形失败：{error}"))?;
    let reserved_jis_codes = reserved_jis_codes.iter().copied().collect::<BTreeSet<_>>();
    let generated_glyph_table = resources
        .generated_glyph_table()
        .map_err(|error| format!("序列化现场绘制的 PC-88 字形失败：{error}"))?;
    let plan = resources
        .plan_dynamic_mapping_with_carrier_filter(
            original_double_byte_codes.iter().copied(),
            display_texts.iter(),
            |jis| {
                let row = jis >> 8;
                let cell = jis & 0x00ff;
                ((row == 0x21 && (0x21..=0x7e).contains(&cell))
                    || ((0x30..=0x4f).contains(&row)
                        && (0x21..=0x7e).contains(&cell)
                        && (row != 0x4f || cell <= 0x53)))
                    && !reserved_jis_codes.contains(&jis)
            },
        )
        .map_err(|error| format!("规划 PC-88 字库映射失败：{error}"))?;
    let encoded_texts = display_texts
        .iter()
        .enumerate()
        .map(|(index, text)| {
            resources
                .encode_text(text, &plan)
                .map_err(|error| format!("编码第 {} 条显示文本失败：{error}", index + 1))
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let FontBuild { rom, manifest } = resources
        .build_rom(source_rom, &plan)
        .map_err(|error| format!("重建 PC-88 KANJI1.ROM 失败：{error}"))?;

    Ok(Pc88FontOutput {
        encoded_texts,
        rom,
        generated_glyph_table,
        resources,
        report: Pc88FontReport { plan, manifest },
    })
}
