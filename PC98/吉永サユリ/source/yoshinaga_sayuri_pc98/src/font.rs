use crate::{decode_text, encode_text, json, Result};
use encoding_rs::SHIFT_JIS;
use std::collections::{BTreeMap, BTreeSet};
use vn_font::font_98::{self, EncodingPlan, SubstitutionMap};
use vn_text::Entry;

pub struct FontArtifacts {
    pub plan: Option<EncodingPlan>,
    pub files: BTreeMap<String, Vec<u8>>,
    pub patched: usize,
}

fn native_slot(ch: char) -> Option<u16> {
    let text = ch.to_string();
    let (bytes, _, failed) = SHIFT_JIS.encode(&text);
    if !failed && bytes.len() == 2 && font_98::has_loaded_np2_slot(ch) {
        Some(u16::from_be_bytes([bytes[0], bytes[1]]))
    } else {
        None
    }
}

pub fn prepare(
    text: &[Entry],
    ui: &[Entry],
    original_font: &[u8],
    face: &str,
) -> Result<FontArtifacts> {
    font_98::validate_font(original_font)?;
    let mut display = BTreeSet::new();
    let mut reserved = BTreeSet::new();
    let mut needs_carriers = false;
    for entry in text.iter().chain(ui) {
        for ch in entry.message.chars() {
            if ch.is_control() {
                if ch == '\n' {
                    continue;
                }
                return Err(format!(
                    "control U+{:04X} is not allowed in display text",
                    ch as u32
                ));
            }
            let character = ch.to_string();
            let (bytes, _, failed) = SHIFT_JIS.encode(&character);
            if !failed && bytes.len() == 1 {
                continue;
            }
            if let Some(slot) = native_slot(ch) {
                reserved.insert(slot);
            } else {
                needs_carriers = true;
            }
            display.insert(ch);
        }
    }
    if !needs_carriers {
        return Ok(FontArtifacts {
            plan: None,
            files: BTreeMap::new(),
            patched: 0,
        });
    }
    let characters = display.iter().collect::<String>();
    let plan = EncodingPlan::build(
        &SubstitutionMap::embedded()?,
        reserved.iter().copied(),
        [characters.as_str()],
    )?;
    let requests = plan.requests();
    let font = font_98::prepare_font(original_font, &requests, &reserved, face)?;
    let actual_face = if let Some(request) = requests.first() {
        font_98::render_glyph_16x16_with_face(request.replacement, face)?.1
    } else {
        face.to_string()
    };
    let request = serde_json::json!({"schema_version":1,"backend":"pc98-np2", "final_texts":[characters],"reserved_cp932":reserved});
    let manifest = serde_json::json!({"backend":"vn-font::font_98", "source_sha256":fivec_new::sha256(original_font), "output_sha256":fivec_new::sha256(&font.bytes), "face_requested":face,"face_actual":actual_face,"patched_glyphs":font.patched_glyphs,"output":"font.bmp","runtime":"Configure the NP2 font input to use this BMP and restart NP2. Y2.EXE reads glyphs through the PC98 font ports; no executable hook is required."});
    let files = BTreeMap::from([
        ("font/font.bmp".into(), font.bytes),
        (
            "font/font_mapping.json".into(),
            json(&plan.manifest_entries()?)?,
        ),
        ("font/font_request.json".into(), json(&request)?),
        ("font/font_manifest.json".into(), json(&manifest)?),
    ]);
    Ok(FontArtifacts {
        plan: Some(plan),
        files,
        patched: font.patched_glyphs,
    })
}

pub fn encode(text: &str, newline: bool, plan: Option<&EncodingPlan>) -> Result<Vec<u8>> {
    let Some(plan) = plan else {
        return encode_text(text, newline);
    };
    let mut output = Vec::new();
    for ch in text.chars() {
        if ch.is_control() {
            if newline && ch == '\n' {
                output.push(0x1f);
                continue;
            }
            return Err(format!("control U+{:04X} is not allowed", ch as u32));
        }
        let character = ch.to_string();
        let (native, _, failed) = SHIFT_JIS.encode(&character);
        if !failed && native.len() == 1 {
            output.extend_from_slice(&native);
        } else {
            output.extend(font_98::cp932_for_carrier(plan.carrier_for(ch)?)?);
        }
    }
    if plan.decode_carriers(&decode_text(&output, newline)?) != text {
        return Err(
            "font carrier encoding does not round-trip to the requested display text".into(),
        );
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_native_protection_and_chinese_carriers() {
        let entries = vec![Entry {
            name: None,
            message: "あA\n咦赶黑".into(),
        }];
        let font = prepare(&entries, &[], font_98::EMBEDDED_FONT, font_98::FONT_FACE).unwrap();
        let plan = font.plan.as_ref().unwrap();
        assert_eq!(font.patched, 3);
        assert_eq!(
            font_98::read_glyph(&font.files["font/font.bmp"], 'あ').unwrap(),
            font_98::read_glyph(font_98::EMBEDDED_FONT, 'あ').unwrap()
        );
        assert_eq!(
            &encode("あA\n", true, Some(plan)).unwrap(),
            &[0x82, 0xa0, 0x41, 0x1f]
        );
        assert_eq!(
            plan.decode_carriers(
                &decode_text(
                    &encode(&entries[0].message, true, Some(plan)).unwrap(),
                    true
                )
                .unwrap()
            ),
            entries[0].message
        );
    }
}
