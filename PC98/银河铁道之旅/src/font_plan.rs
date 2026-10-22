//! One NP2 encoding plan shared by text injection and `font.tmp` generation.

use serde::Serialize;
use std::collections::BTreeSet;
use vn_font::font_98::{self, EncodingPlan, EncodingPlanEntry, SubstitutionMap};

use crate::Result;

pub const SCHEMA: &str = "galaxy-railway-pc98-font-plan-v1";

#[derive(Clone, Debug)]
pub struct DisplayText {
    pub source_file: String,
    pub index: usize,
    pub original: String,
    pub display: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct EncodedText {
    pub source_file: String,
    pub index: usize,
    pub byte_length: usize,
    pub encoded_hex: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct FontPlanReport {
    pub schema: String,
    pub face: String,
    pub source_font_sha256: String,
    pub reserved_cp932: Vec<String>,
    pub mapping: Vec<EncodingPlanEntry>,
    pub encodings: Vec<EncodedText>,
    pub patched_glyphs: usize,
}

pub struct FontArtifacts {
    pub report: FontPlanReport,
    pub bytes: Vec<u8>,
    pub encoding: EncodingPlan,
}

pub fn build(original_font: &[u8], texts: &[DisplayText], face: &str) -> Result<FontArtifacts> {
    if texts.is_empty() {
        return Err("没有可用于 NP2 字库规划的文本".into());
    }
    let substitutions =
        SubstitutionMap::embedded().map_err(|e| format!("读取字库映射失败: {e}"))?;
    let mut reserved = BTreeSet::<u16>::new();
    let mut display_texts = Vec::<String>::new();
    for text in texts {
        if text.original == text.display {
            collect_double_byte_codes(&text.original, &mut reserved)?;
        }
        display_texts.push(double_byte_characters(&text.display)?);
    }
    let encoding = EncodingPlan::build(
        &substitutions,
        reserved.iter().copied(),
        display_texts.iter().map(String::as_str),
    )
    .map_err(|e| format!("构建 CP932 字槽计划失败: {e}"))?;

    let mut encodings = Vec::with_capacity(texts.len());
    for text in texts {
        let encoded = encode_display(&text.display, &encoding)?;
        encodings.push(EncodedText {
            source_file: text.source_file.clone(),
            index: text.index,
            byte_length: encoded.len(),
            encoded_hex: hex(&encoded),
        });
    }

    let font = font_98::prepare_font(original_font, &encoding.requests(), &reserved, face)
        .map_err(|e| format!("生成 NP2 font.tmp 失败: {e}"))?;
    let report = FontPlanReport {
        schema: SCHEMA.into(),
        face: face.into(),
        source_font_sha256: crate::sha256(original_font),
        reserved_cp932: reserved.iter().map(|code| format!("{code:04X}")).collect(),
        mapping: encoding
            .manifest_entries()
            .map_err(|e| format!("生成字库映射清单失败: {e}"))?,
        encodings,
        patched_glyphs: font.patched_glyphs,
    };
    Ok(FontArtifacts {
        report,
        bytes: font.bytes,
        encoding,
    })
}

pub fn encode_display(text: &str, plan: &EncodingPlan) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(text.len());
    for character in text.chars() {
        if character == '\n' {
            // MSG text uses NUL-terminated segments; the JSON workflow exposes
            // those native separators as ordinary line breaks.
            output.push(0);
            continue;
        }
        if character == '\0' || character == '\r' || character.is_control() {
            return Err(format!("文本包含不可注回控制符 U+{:04X}", character as u32));
        }
        let character_text = character.to_string();
        let (native, _, had_errors) = encoding_rs::SHIFT_JIS.encode(&character_text);
        if !had_errors && native.len() == 1 {
            // ASCII and half-width kana keep their native single-byte form.
            output.push(native[0]);
        } else {
            // Native CP932 characters keep their own slots; characters such as
            // Simplified Chinese use the shared font plan's CP932 carrier.
            output.extend(
                plan.encode_cp932(&character_text)
                    .map_err(|e| format!("字符 {character} 的字槽计划编码失败: {e}"))?,
            );
        }
    }
    Ok(output)
}

fn double_byte_characters(text: &str) -> Result<String> {
    let mut output = String::new();
    for character in text.chars() {
        let character_text = character.to_string();
        let (encoded, _, had_errors) = encoding_rs::SHIFT_JIS.encode(&character_text);
        if had_errors || encoded.len() == 2 {
            output.push(character);
        }
    }
    Ok(output)
}

fn collect_double_byte_codes(text: &str, output: &mut BTreeSet<u16>) -> Result<()> {
    let (encoded, _, had_errors) = encoding_rs::SHIFT_JIS.encode(text);
    if had_errors {
        return Err("未修改原文无法用 CP932 原样编码，不能安全保护对应字槽".into());
    }
    let mut position = 0usize;
    while position < encoded.len() {
        let first = encoded[position];
        if is_cp932_lead(first) {
            let second = *encoded
                .get(position + 1)
                .ok_or_else(|| "CP932 双字节原文被截断".to_string())?;
            output.insert(u16::from_be_bytes([first, second]));
            position += 2;
        } else {
            position += 1;
        }
    }
    Ok(())
}

fn is_cp932_lead(byte: u8) -> bool {
    matches!(byte, 0x81..=0x9f | 0xe0..=0xfc)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}
