//! Shared CPMAIN extension-code and NP2 font mapping plan.
//!
//! The SCR adapter owns the binary layout.  This module only consumes the
//! extracted JSON and keeps two namespaces separate:
//! * CPMAIN-confirmed extension pairs are emitted as raw CODE bytes.
//! * ordinary CP932 characters are passed to vn-font for carrier allocation.
//!
//! CPMAIN's disassembly has explicit handling for high byte 0x80 and for
//! high bytes at or above 0xA0.  Those high-byte ranges remain CODE markers
//! even if a generic CP932 decoder happens to accept a particular pair.

use crate::scr::{self, ScrDocument};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use vn_font::font_98::{self, EncodingPlan, EncodingPlanEntry, SubstitutionMap};

pub const SCHEMA: &str = "chrome-paradise-font-plan-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontPlan {
    pub schema: String,
    pub input: String,
    pub source_files: Vec<String>,
    pub documents: usize,
    pub strings: usize,
    pub reserved_cp932: Vec<String>,
    pub code_pairs: Vec<CodePairPlan>,
    pub mapping: Vec<EncodingPlanEntry>,
    pub font_patches: Vec<FontPatch>,
    pub encodings: Vec<EncodedString>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePairPlan {
    pub code: String,
    pub high: u8,
    pub low: u8,
    pub classification: String,
    pub cp932_valid: bool,
    pub source_count: usize,
    pub final_count: usize,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontPatch {
    pub character: String,
    pub carrier: String,
    pub cp932_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedString {
    pub source_file: String,
    pub index: usize,
    pub message_sha256: String,
    pub byte_length: usize,
    pub encoded_hex: String,
    pub code_pairs: Vec<String>,
}

#[derive(Default)]
struct PairState {
    source_count: usize,
    final_count: usize,
    files: BTreeSet<String>,
    cp932_valid: bool,
    cpmain_confirmed: bool,
}

/// Read one text-extract JSON or a complete flat text-extract directory and
/// build the immutable mapping/encoding plan used by later font and SCR
/// rebuild steps.
fn build_font_plan_internal(
    input: &Path,
) -> Result<(FontPlan, EncodingPlan, BTreeSet<u16>), String> {
    let input = input
        .canonicalize()
        .map_err(|error| format!("无法读取字体计划输入 {}: {error}", input.display()))?;
    let paths = collect_json_files(&input)?;
    if paths.is_empty() {
        return Err(format!("{} 中没有 SCR JSON", input.display()));
    }

    let mut documents = Vec::<(PathBuf, ScrDocument)>::new();
    for path in paths {
        let json = fs::read_to_string(&path)
            .map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
        let document: ScrDocument = serde_json::from_str(&json)
            .map_err(|error| format!("解析 {} 失败: {error}", path.display()))?;
        if document.schema != crate::scr::SCHEMA {
            return Err(format!(
                "{} 使用不支持的 SCR schema {}",
                path.display(),
                document.schema
            ));
        }
        let blocking = document
            .diagnostics
            .iter()
            .filter(|message| {
                !message.contains("含 CPMAIN 扩展码位") && !message.starts_with("已过滤 ")
            })
            .cloned()
            .collect::<Vec<_>>();
        if !blocking.is_empty() {
            return Err(format!(
                "{} 存在未确认的 SCR 结构诊断: {}",
                path.display(),
                blocking.join("; ")
            ));
        }
        documents.push((path, document));
    }

    let mut reserved_cp932 = BTreeSet::<u16>::new();
    let mut pairs = BTreeMap::<[u8; 2], PairState>::new();
    let mut final_texts = Vec::<String>::new();
    let mut encoding_inputs = Vec::<(String, usize, String, String, Vec<String>)>::new();
    let mut source_files = BTreeSet::<String>::new();
    let mut total_strings = 0usize;
    let mut diagnostics = Vec::new();

    for (_, document) in &documents {
        source_files.insert(document.source_file.clone());
        for item in &document.strings {
            total_strings += 1;
            let source_codes = scan_code_markers(&item.scr_msg, &document.source_file)?;
            let final_codes = scan_code_markers(&item.message, &document.source_file)?;
            for code in source_codes {
                let entry = pairs.entry(code).or_default();
                entry.source_count += 1;
                entry.files.insert(document.source_file.clone());
                update_pair_classification(entry, code);
            }
            for code in final_codes.iter().copied() {
                let entry = pairs.entry(code).or_default();
                entry.final_count += 1;
                entry.files.insert(document.source_file.clone());
                update_pair_classification(entry, code);
            }

            let styled_message = scr::restore_halfwidth_kana(&item.message, &item.halfwidth_kana)?;
            collect_reserved_cp932(&styled_message, &mut reserved_cp932)?;
            let planning_text = planning_text(&styled_message, &document.source_file)?;
            final_texts.push(planning_text);
            encoding_inputs.push((
                document.source_file.clone(),
                item.index,
                styled_message,
                fivec_new::sha256(item.message.as_bytes()),
                final_codes
                    .iter()
                    .map(|code| format!("{:02X}{:02X}", code[0], code[1]))
                    .collect(),
            ));
        }
    }

    let substitutions = SubstitutionMap::embedded()?;
    let plan = EncodingPlan::build_with_forbidden_cp932(
        &substitutions,
        reserved_cp932.iter().copied(),
        [],
        final_texts.iter().map(String::as_str),
    )?;
    let mapping = plan.manifest_entries()?;
    let font_patches = plan
        .requests()
        .into_iter()
        .map(|request| {
            let cp932 = font_98::cp932_for_carrier(request.carrier)?;
            Ok(FontPatch {
                character: request.replacement.to_string(),
                carrier: request.carrier.to_string(),
                cp932_hex: format!("{:02X}{:02X}", cp932[0], cp932[1]),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let mut encodings = Vec::with_capacity(encoding_inputs.len());
    for (source_file, index, message, message_sha256, code_pairs) in encoding_inputs {
        let encoded = encode_message_with_plan(&message, &plan, &source_file)?;
        encodings.push(EncodedString {
            source_file,
            index,
            message_sha256,
            byte_length: encoded.len(),
            encoded_hex: hex(&encoded),
            code_pairs,
        });
    }

    let code_pairs = pairs
        .into_iter()
        .map(|(code, state)| {
            let classification = if state.cpmain_confirmed {
                "cpmain-extension"
            } else if state.cp932_valid {
                "ordinary-cp932"
            } else {
                "unconfirmed"
            };
            if classification != "cpmain-extension" {
                diagnostics.push(format!(
                    "CODE:{:02X}{:02X} 分类为 {}，不会当作 CPMAIN 扩展字形",
                    code[0], code[1], classification
                ));
            }
            CodePairPlan {
                code: format!("{:02X}{:02X}", code[0], code[1]),
                high: code[0],
                low: code[1],
                classification: classification.into(),
                cp932_valid: state.cp932_valid,
                source_count: state.source_count,
                final_count: state.final_count,
                files: state.files.into_iter().collect(),
            }
        })
        .collect();

    let reserved_for_build = reserved_cp932.clone();
    let output = FontPlan {
        schema: SCHEMA.into(),
        input: input.display().to_string(),
        source_files: source_files.into_iter().collect(),
        documents: documents.len(),
        strings: total_strings,
        reserved_cp932: reserved_cp932
            .into_iter()
            .map(|code| format!("{code:04X}"))
            .collect(),
        code_pairs,
        mapping,
        font_patches,
        encodings,
        diagnostics,
    };
    Ok((output, plan, reserved_for_build))
}

pub fn build_font_plan(input: &Path) -> Result<FontPlan, String> {
    let (plan, _, _) = build_font_plan_internal(input)?;
    Ok(plan)
}

/// Write a plan as a new UTF-8 JSON file.  Existing files are never replaced.
pub fn write_font_plan(input: &Path, output: &Path) -> Result<FontPlan, String> {
    let plan = build_font_plan(input)?;
    if output.exists() {
        return Err(format!("字体计划输出已存在: {}", output.display()));
    }
    let parent = output
        .parent()
        .ok_or_else(|| "字体计划输出没有父目录".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("创建字体计划目录失败: {error}"))?;
    let bytes = serde_json::to_vec_pretty(&plan).map_err(|error| error.to_string())?;
    let temporary = output.with_extension("json.tmp");
    if temporary.exists() {
        return Err(format!("字体计划临时输出已存在: {}", temporary.display()));
    }
    fs::write(&temporary, bytes).map_err(|error| format!("写入字体计划失败: {error}"))?;
    fs::rename(&temporary, output).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("提交字体计划失败: {error}")
    })?;
    Ok(plan)
}

#[derive(Debug, Clone, Serialize)]
pub struct FontArtifactReport {
    pub output_directory: PathBuf,
    pub font_path: PathBuf,
    pub manifest_path: PathBuf,
    pub source_sha256: String,
    pub output_sha256: String,
    pub patched_glyphs: usize,
}

/// Rebuild an NP2 `font.tmp` from a previously generated plan and atomically
/// commit `font.tmp` plus a provenance manifest into a new directory.
pub fn build_font_artifact(
    source_font: &Path,
    plan: &FontPlan,
    output_directory: &Path,
    face: &str,
) -> Result<FontArtifactReport, String> {
    if plan.schema != SCHEMA {
        return Err(format!("不支持的字体计划 schema: {}", plan.schema));
    }
    if output_directory.exists() {
        return Err(format!(
            "字库输出目录已存在: {}",
            output_directory.display()
        ));
    }
    let source_font = source_font
        .canonicalize()
        .map_err(|error| format!("无法读取源 font.tmp: {error}"))?;
    let source = fs::read(&source_font)
        .map_err(|error| format!("读取源 font.tmp {} 失败: {error}", source_font.display()))?;
    font_98::validate_font(&source)?;
    let source_sha256 = fivec_new::sha256(&source);
    let reserved = plan
        .reserved_cp932
        .iter()
        .map(|code| {
            u16::from_str_radix(code, 16)
                .map_err(|error| format!("字体计划含非法 reserved_cp932 {code}: {error}"))
        })
        .collect::<Result<BTreeSet<_>, String>>()?;
    let requests = plan
        .font_patches
        .iter()
        .map(|patch| {
            let replacement = single_char(&patch.character, "font_patches.character")?;
            let carrier = single_char(&patch.carrier, "font_patches.carrier")?;
            Ok(font_98::FontPatchRequest {
                carrier,
                replacement,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let build = font_98::prepare_font(&source, &requests, &reserved, face)?;
    let output_directory = if output_directory.is_absolute() {
        output_directory.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| error.to_string())?
            .join(output_directory)
    };
    let parent = output_directory
        .parent()
        .ok_or_else(|| "字库输出目录没有父目录".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("创建字库输出父目录失败: {error}"))?;
    let stage = parent.join(format!(
        ".chrome-paradise-font-stage-{}",
        std::process::id()
    ));
    if stage.exists() {
        return Err(format!("字库暂存目录已存在: {}", stage.display()));
    }
    fs::create_dir(&stage).map_err(|error| format!("创建字库暂存目录失败: {error}"))?;
    let result = (|| {
        let font_path = stage.join("font.tmp");
        let manifest_path = stage.join("font_manifest.json");
        fs::write(&font_path, &build.bytes)
            .map_err(|error| format!("写入新 font.tmp 失败: {error}"))?;
        let output_sha256 = fivec_new::sha256(&build.bytes);
        let manifest = serde_json::json!({
            "schema": "chrome-paradise-font-artifact-v1",
            "plan_schema": plan.schema,
            "source_font": source_font.display().to_string(),
            "source_sha256": source_sha256,
            "output_sha256": output_sha256,
            "face": face,
            "patched_glyphs": build.patched_glyphs,
            "reserved_cp932": plan.reserved_cp932,
            "code_pairs": plan.code_pairs,
        });
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
        )
        .map_err(|error| format!("写入字库清单失败: {error}"))?;
        if output_directory.exists() {
            return Err("字库输出目录在执行期间出现，请重新预检".into());
        }
        fs::rename(&stage, &output_directory)
            .map_err(|error| format!("提交字库输出目录失败: {error}"))?;
        Ok(FontArtifactReport {
            output_directory: output_directory.clone(),
            font_path: output_directory.join("font.tmp"),
            manifest_path: output_directory.join("font_manifest.json"),
            source_sha256,
            output_sha256,
            patched_glyphs: build.patched_glyphs,
        })
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&stage);
    }
    result
}

/// Return planned encoded strings only after validating that their final
/// Unicode text still matches the exact plan inputs. Keys are the SCR source
/// path from JSON plus its zero-based string index.
pub fn planned_strings_for_document(
    plan: &FontPlan,
    document: &ScrDocument,
) -> Result<Vec<Vec<u8>>, String> {
    if plan.schema != SCHEMA {
        return Err(format!("不支持的字体计划 schema: {}", plan.schema));
    }
    let mut by_index = BTreeMap::new();
    for encoded in plan
        .encodings
        .iter()
        .filter(|entry| entry.source_file == document.source_file)
    {
        if by_index.insert(encoded.index, encoded).is_some() {
            return Err(format!(
                "字体计划中 {} 字符串索引重复",
                document.source_file
            ));
        }
    }
    let mut result = Vec::with_capacity(document.strings.len());
    for string in &document.strings {
        let encoded = by_index.remove(&string.index).ok_or_else(|| {
            format!(
                "字体计划缺少 {} 字符串 {}",
                document.source_file, string.index
            )
        })?;
        let actual_hash = fivec_new::sha256(string.message.as_bytes());
        if actual_hash != encoded.message_sha256 {
            return Err(format!(
                "字体计划与 {} 字符串 {} 的当前译文不匹配，请重新生成 font-plan",
                document.source_file, string.index
            ));
        }
        let bytes = decode_hex(&encoded.encoded_hex)?;
        if bytes.len() != encoded.byte_length {
            return Err(format!(
                "字体计划中 {} 字符串 {} 的编码长度不匹配",
                document.source_file, string.index
            ));
        }
        result.push(bytes);
    }
    if !by_index.is_empty() {
        return Err(format!("字体计划对 {} 含多余字符串", document.source_file));
    }
    Ok(result)
}

fn single_char(text: &str, field: &str) -> Result<char, String> {
    let mut chars = text.chars();
    let character = chars.next().ok_or_else(|| format!("{field} 不能为空"))?;
    if chars.next().is_some() {
        return Err(format!("{field} 必须恰好一个 Unicode 字符"));
    }
    Ok(character)
}

fn collect_json_files(input: &Path) -> Result<Vec<PathBuf>, String> {
    if input.is_file() {
        if !input
            .extension()
            .map(|value| value.eq_ignore_ascii_case("json"))
            .unwrap_or(false)
        {
            return Err(format!("字体计划输入不是 JSON: {}", input.display()));
        }
        return Ok(vec![input.to_path_buf()]);
    }
    if !input.is_dir() {
        return Err(format!("字体计划输入不是文件或目录: {}", input.display()));
    }
    let mut stack = vec![input.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = stack.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|error| format!("读取 {} 失败: {error}", directory.display()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        for entry in entries {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
            if metadata.is_dir() {
                stack.push(path);
            } else if metadata.is_file()
                && path
                    .extension()
                    .map(|value| value.eq_ignore_ascii_case("json"))
                    .unwrap_or(false)
                && path
                    .file_name()
                    .map(|value| value != "text_workspace.json")
                    .unwrap_or(false)
            {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn update_pair_classification(state: &mut PairState, code: [u8; 2]) {
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(&code);
    state.cp932_valid = !had_errors && !decoded.contains('\u{fffd}');
    // CPMAIN branches on the high byte before BIOS glyph lookup.  A pair
    // beginning with 0x80 or >=0xA0 is therefore a confirmed game code even
    // if a generic CP932 decoder happens to accept that byte pair.
    state.cpmain_confirmed = code[0] == 0x80 || code[0] >= 0xA0;
}

fn collect_reserved_cp932(text: &str, output: &mut BTreeSet<u16>) -> Result<(), String> {
    let mut cursor = 0usize;
    while cursor < text.len() {
        if let Some((marker, consumed)) = parse_marker(&text[cursor..])? {
            cursor += consumed;
            if let Marker::Unknown(token) = marker {
                return Err(format!("译文中含未知 SCR 标记 {token}"));
            }
            continue;
        }
        let character = text[cursor..]
            .chars()
            .next()
            .ok_or_else(|| "UTF-8 文本截断".to_string())?;
        cursor += character.len_utf8();
        let mut buffer = [0u8; 4];
        let (bytes, _, had_errors) = SHIFT_JIS.encode(character.encode_utf8(&mut buffer));
        if !had_errors && bytes.len() == 2 && font_98::has_loaded_np2_slot(character) {
            output.insert(u16::from_be_bytes([bytes[0], bytes[1]]));
        }
    }
    Ok(())
}

fn planning_text(text: &str, source_file: &str) -> Result<String, String> {
    let mut output = String::new();
    let mut cursor = 0usize;
    while cursor < text.len() {
        if let Some((marker, consumed)) = parse_marker(&text[cursor..])? {
            cursor += consumed;
            if let Marker::Unknown(token) = marker {
                return Err(format!("{source_file}: 未知 SCR 标记 {token}"));
            }
            continue;
        }
        let character = text[cursor..]
            .chars()
            .next()
            .ok_or_else(|| format!("{source_file}: UTF-8 文本截断"))?;
        cursor += character.len_utf8();
        if character == '\n' || character == '\r' || character == '\u{007F}' {
            continue;
        }
        if character.is_control() {
            return Err(format!(
                "{source_file}: 译文含未转义控制字符 U+{:04X}",
                character as u32
            ));
        }
        let mut buffer = [0u8; 4];
        let encoded = character.encode_utf8(&mut buffer);
        let (bytes, _, had_errors) = SHIFT_JIS.encode(encoded);
        if had_errors || bytes.len() != 1 {
            output.push(character);
        }
    }
    Ok(output)
}

fn encode_message_with_plan(
    text: &str,
    plan: &EncodingPlan,
    source_file: &str,
) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let mut cursor = 0usize;
    while cursor < text.len() {
        if let Some((marker, consumed)) = parse_marker(&text[cursor..])? {
            cursor += consumed;
            match marker {
                Marker::Byte(byte) | Marker::Ctrl(byte) => output.push(byte),
                Marker::Reference(index) => output.extend_from_slice(&[0x02, index]),
                Marker::Code(code) => {
                    output.extend_from_slice(&code);
                }
                Marker::Unknown(token) => {
                    return Err(format!("{source_file}: 未知 SCR 标记 {token}"));
                }
            }
            continue;
        }
        let character = text[cursor..]
            .chars()
            .next()
            .ok_or_else(|| format!("{source_file}: UTF-8 文本截断"))?;
        cursor += character.len_utf8();
        if character == '\n' {
            output.push(0x0A);
            continue;
        }
        if character == '\u{007F}' {
            output.push(0x7F);
            continue;
        }
        if character == '\r' || character == '\0' || character.is_control() {
            return Err(format!(
                "{source_file}: 译文含未转义控制字符 U+{:04X}",
                character as u32
            ));
        }
        let mut buffer = [0u8; 4];
        let encoded = character.encode_utf8(&mut buffer);
        let (bytes, _, had_errors) = SHIFT_JIS.encode(encoded);
        if !had_errors && bytes.len() == 1 {
            output.extend_from_slice(&bytes);
        } else {
            output.extend_from_slice(&plan.encode_cp932(&character.to_string())?);
        }
    }
    Ok(output)
}

#[derive(Debug)]
enum Marker {
    Byte(u8),
    Ctrl(u8),
    Reference(u8),
    Code([u8; 2]),
    Unknown(String),
}

fn parse_marker(text: &str) -> Result<Option<(Marker, usize)>, String> {
    if !text.starts_with("[[") {
        return Ok(None);
    }
    let Some(end) = text.find("]]") else {
        return Ok(None);
    };
    let token = &text[..end + 2];
    let (prefix, digits) = if token.starts_with("[[BYTE:") {
        ("[[BYTE:", 2)
    } else if token.starts_with("[[CTRL:") {
        ("[[CTRL:", 2)
    } else if token.starts_with("[[REF:") {
        ("[[REF:", 2)
    } else if token.starts_with("[[CODE:") {
        ("[[CODE:", 4)
    } else {
        return Ok(None);
    };
    if token.len() != prefix.len() + digits + 2
        || !token.ends_with("]]")
        || !(0..digits).all(|index| token.as_bytes()[prefix.len() + index].is_ascii_hexdigit())
    {
        return Ok(Some((Marker::Unknown(token.to_string()), token.len())));
    }
    let value = u16::from_str_radix(&token[prefix.len()..prefix.len() + digits], 16)
        .map_err(|error| format!("解析 SCR 标记 {token} 失败: {error}"))?;
    let marker = if prefix == "[[BYTE:" {
        Marker::Byte(value as u8)
    } else if prefix == "[[CTRL:" {
        Marker::Ctrl(value as u8)
    } else if prefix == "[[REF:" {
        Marker::Reference(value as u8)
    } else {
        Marker::Code([(value >> 8) as u8, value as u8])
    };
    Ok(Some((marker, token.len())))
}

fn scan_code_markers(text: &str, source_file: &str) -> Result<Vec<[u8; 2]>, String> {
    let mut output = Vec::new();
    let mut cursor = 0usize;
    while cursor < text.len() {
        if let Some((marker, consumed)) = parse_marker(&text[cursor..])? {
            cursor += consumed;
            if let Marker::Code(code) = marker {
                if code[0] != 0x80 && code[0] < 0xA0 {
                    return Err(format!(
                        "{source_file}: CODE:{:02X}{:02X} 不符合 CPMAIN 反汇编确认的高字节规则",
                        code[0], code[1]
                    ));
                }
                output.push(code);
            }
            continue;
        }
        let character = text[cursor..]
            .chars()
            .next()
            .ok_or_else(|| format!("{source_file}: UTF-8 文本截断"))?;
        cursor += character.len_utf8();
    }
    Ok(output)
}

fn decode_hex(text: &str) -> Result<Vec<u8>, String> {
    if !text.len().is_multiple_of(2) {
        return Err(format!("十六进制字段长度必须为偶数: {text}"));
    }
    (0..text.len())
        .step_by(2)
        .map(|offset| {
            u8::from_str_radix(&text[offset..offset + 2], 16)
                .map_err(|error| format!("非法十六进制字段 {text}: {error}"))
        })
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn code_pair_classification_distinguishes_cpmain_and_cp932() {
        let mut extension = PairState::default();
        update_pair_classification(&mut extension, [0xEB, 0xAF]);
        assert!(extension.cpmain_confirmed);
        assert!(!extension.cp932_valid);

        let mut cpmain_80 = PairState::default();
        update_pair_classification(&mut cpmain_80, [0x80, 0xA4]);
        assert!(cpmain_80.cpmain_confirmed);

        let mut ordinary = PairState::default();
        update_pair_classification(&mut ordinary, [0x82, 0xA0]);
        assert!(!ordinary.cpmain_confirmed);
        assert!(ordinary.cp932_valid);
    }

    #[test]
    fn planning_text_removes_controls_and_keeps_double_byte_chars() {
        let text = "A[[CODE:EBAF]]中\nＢ";
        let planned = planning_text(text, "test.SCR").unwrap();
        assert_eq!(planned, "中Ｂ");
    }

    #[test]
    fn source_cp932_reservation_only_keeps_valid_pairs() {
        let mut reserved = BTreeSet::new();
        let (unsupported, had_errors) = SHIFT_JIS.decode_without_bom_handling(&[0xFA, 0x89]);
        assert!(!had_errors);
        assert!(!font_98::has_loaded_np2_slot(
            unsupported.chars().next().unwrap()
        ));
        collect_reserved_cp932(
            &format!("あ{unsupported}嗯[[CODE:EBAF]][[REF:8A]]"),
            &mut reserved,
        )
        .unwrap();
        assert!(reserved.contains(&0x82A0));
        assert!(!reserved.contains(&0xEBAF));
        assert!(!reserved.contains(&0xFA89));
    }

    #[test]
    fn indexed_reference_is_a_marker_and_its_index_is_not_cp932() {
        let (marker, consumed) = parse_marker("[[REF:8A]]君").unwrap().unwrap();
        assert!(matches!(marker, Marker::Reference(0x8A)));
        assert_eq!(consumed, "[[REF:8A]]".len());

        let mut reserved = BTreeSet::new();
        collect_reserved_cp932("[[REF:8A]]あ", &mut reserved).unwrap();
        assert!(reserved.contains(&0x82A0));
        assert!(!reserved.contains(&0x8A8C));
    }
}
