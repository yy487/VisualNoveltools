//! Game-side composition for the shared PC-88 dynamic KANJI1 font module.
//!
//! This adapter keeps the original JIS glyphs used by the game and allocates
//! carriers dynamically for this build. No PC-98 substitution table or
//! tenboudai-specific carrier range is used here.

use serde::Serialize;
use vn_font::font_88::{
    self, DynamicFontPlan, FontManifest, FontResources, GLYPH_BYTES, KANJI1_ROM_SIZE,
};

use crate::Result;

pub const BUILTIN_KANJI1_ROM: &[u8; KANJI1_ROM_SIZE] = include_bytes!("../assets/pc88/KANJI1.ROM");

/// Full-width JIS codes which RIO53 treats as inline control operators.
/// They must never be selected as carriers for a translated glyph.
pub const RESERVED_TEXT_OP_JIS: [u16; 5] = [0x2363, 0x2365, 0x2369, 0x236E, 0x2372];

/// PC-88/N88 input-help strings seen outside the extracted game text files use
/// the ordinary full-width ASCII/punctuation JIS slots. Keep those slots out
/// of the dynamic carrier pool so system/program UI remains readable.
fn is_unavailable_literal_jis(jis: u16) -> bool {
    RESERVED_TEXT_OP_JIS.contains(&jis)
        || (0x2121..=0x217E).contains(&jis)
        || (0x2330..=0x2339).contains(&jis)
        || (0x2341..=0x235A).contains(&jis)
        || (0x2361..=0x237A).contains(&jis)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Pc88FontReport {
    pub source_rom_sha256: String,
    pub plan: DynamicFontPlan,
    pub manifest: FontManifest,
    pub generated_glyph_table: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct Pc88FontBuild {
    pub resources: FontResources,
    pub plan: DynamicFontPlan,
    pub rom: Vec<u8>,
    pub report: Pc88FontReport,
}

/// Build a dynamic PC-88 plan from every original double-byte code and every
/// final display string. The same plan drives JIS text encoding and ROM slots.
pub fn build_pc88_font(
    original_double_byte_codes: &[u16],
    display_texts: &[String],
) -> Result<Pc88FontBuild> {
    let source_rom = BUILTIN_KANJI1_ROM.as_slice();
    font_88::validate_rom(source_rom)?;

    let mut resources = FontResources::load_embedded()
        .map_err(|error| format!("载入公共 PC-88 字库资源失败：{error}"))?;
    resources
        .generate_missing_glyphs(display_texts.iter(), font_88::DEFAULT_GLYPH_FONT_FACE)
        .map_err(|error| format!("绘制缺失 PC-88 字形失败：{error}"))?;
    let generated_glyph_table = resources
        .generated_glyph_table()
        .map_err(|error| format!("序列化生成的 PC-88 字形失败：{error}"))?;
    // RIO53 controls and PC-88 input-help characters cannot safely be used as
    // carriers; remap translated instances of those native glyphs elsewhere.
    let plan = resources
        .plan_dynamic_mapping_with_filters(
            original_double_byte_codes.iter().copied(),
            display_texts.iter(),
            |jis| !is_unavailable_literal_jis(jis),
            |jis| !is_unavailable_literal_jis(jis),
        )
        .map_err(|error| format!("规划本作 PC-88 动态字库映射失败：{error}"))?;
    let font_88::FontBuild { rom, manifest } = resources
        .build_rom(source_rom, &plan)
        .map_err(|error| format!("重建内置 PC-88 KANJI1.ROM 失败：{error}"))?;
    let report = Pc88FontReport {
        source_rom_sha256: manifest.source_sha256.clone(),
        plan: plan.clone(),
        manifest,
        generated_glyph_table,
    };

    Ok(Pc88FontBuild {
        resources,
        plan,
        rom,
        report,
    })
}

/// Encode one display run with the build's fixed carrier plan, then convert its
/// CP932 pairs back to 7-bit JIS row/cell bytes for the PC-88 ESC K resources.
pub fn encode_display_run(
    resources: &FontResources,
    plan: &DynamicFontPlan,
    text: &str,
) -> Result<Vec<u8>> {
    let encoded = resources
        .encode_text(text, plan)
        .map_err(|error| format!("PC-88 动态映射编码失败：{error}"))?;
    if encoded.bytes.len() % 2 != 0 {
        return Err("本作 ESC K 显示文本必须由完整的 CP932 双字节字形组成".into());
    }
    let mut jis_bytes = Vec::with_capacity(encoded.bytes.len());
    for pair in encoded.bytes.chunks_exact(2) {
        let cp932 = u16::from_be_bytes([pair[0], pair[1]]);
        let jis = font_88::sjis_to_jis(cp932)
            .ok_or_else(|| format!("载体 CP932 {cp932:04X} 无法转换为标准 JIS"))?;
        if is_unavailable_literal_jis(jis) {
            return Err(format!(
                "JIS {jis:04X} 被保留给 PC-88/RIO53 系统或控制文本，拒绝写入显示字"
            ));
        }
        jis_bytes.extend_from_slice(&jis.to_be_bytes());
    }
    Ok(jis_bytes)
}

/// Convert one JIS X 0208 row/cell code to CP932 using the standard reversible
/// arithmetic mapping, then let encoding_rs resolve the Unicode character.
pub fn jis_pair_to_char(jis: u16) -> Result<char> {
    let cp932 = jis_to_cp932(jis)?;
    let bytes = cp932.to_be_bytes();
    let (decoded, _, had_errors) = encoding_rs::SHIFT_JIS.decode(&bytes);
    if had_errors || decoded.chars().count() != 1 {
        return Err(format!(
            "JIS {jis:04X}/CP932 {cp932:04X} 无法解码为单个字符"
        ));
    }
    decoded
        .chars()
        .next()
        .ok_or_else(|| format!("JIS {jis:04X} 解码为空"))
}

/// Convert a standard 7-bit JIS X 0208 row/cell to a CP932 double-byte code.
pub fn jis_to_cp932(jis: u16) -> Result<u16> {
    let row = u8::try_from(jis >> 8).map_err(|_| format!("JIS {jis:04X} 行码越界"))?;
    let cell = u8::try_from(jis & 0xFF).map_err(|_| format!("JIS {jis:04X} 列码越界"))?;
    if !(0x21..=0x7E).contains(&row) || !(0x21..=0x7E).contains(&cell) {
        return Err(format!("JIS {jis:04X} 不是有效的 7-bit row/cell"));
    }
    let row_index = u16::from(row - 0x21);
    let mut lead = (row_index >> 1) + 0x81;
    if lead > 0x9F {
        lead += 0x40;
    }
    let trail = if row_index & 1 == 0 {
        let trail = u16::from(cell) + 0x1F;
        if trail >= 0x7F {
            trail + 1
        } else {
            trail
        }
    } else {
        u16::from(cell) + 0x7E
    };
    let cp932 = (lead << 8) | trail;
    if font_88::sjis_to_jis(cp932) != Some(jis) {
        return Err(format!("JIS {jis:04X} 不能无损往返为 CP932"));
    }
    Ok(cp932)
}

pub fn jis_pair_to_cp932(jis: u16) -> Result<u16> {
    jis_to_cp932(jis)
}

pub fn strict_cp932_pair(character: char) -> Option<u16> {
    font_88::strict_cp932_pair(character).ok()
}

pub fn is_reserved_text_op_jis(jis: u16) -> bool {
    RESERVED_TEXT_OP_JIS.contains(&jis)
}

pub const fn glyph_bytes() -> usize {
    GLYPH_BYTES
}
