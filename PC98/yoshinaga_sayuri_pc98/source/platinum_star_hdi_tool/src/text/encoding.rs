use encoding_rs::SHIFT_JIS;
use std::collections::BTreeSet;
use vn_font::font_98::{self, EncodingPlan};

pub fn encode_game_text(value: &str, plan: Option<&EncodingPlan>) -> Result<Vec<u8>, String> {
    let mut output = Vec::with_capacity(value.len());
    for character in value.chars() {
        let text = character.to_string();
        // The font plan may redirect even a CP932-encodable character when its
        // native code points to a page that NP2 does not load (e.g. 赶/黑).
        if let Some(carrier) = plan.and_then(|plan| plan.carrier_for(character).ok()) {
            output.extend_from_slice(&font_98::cp932_for_carrier(carrier)?);
            continue;
        }
        let (encoded, _, had_errors) = SHIFT_JIS.encode(&text);
        if had_errors {
            let plan = plan.ok_or_else(|| {
                format!(
                    "文本含无法编码为 CP932 的字符: {character} (U+{:04X})；需要 PC-98 字库映射",
                    character as u32
                )
            })?;
            output.extend_from_slice(&plan.encode_cp932(&text)?);
        } else {
            output.extend_from_slice(&encoded);
        }
    }
    Ok(output)
}

pub fn characters_needing_carriers<'a>(texts: impl IntoIterator<Item = &'a str>) -> BTreeSet<char> {
    texts
        .into_iter()
        .flat_map(str::chars)
        .filter(|character| {
            let text = character.to_string();
            let (encoded, _, had_errors) = SHIFT_JIS.encode(&text);
            had_errors || (encoded.len() == 2 && !font_98::has_loaded_np2_slot(*character))
        })
        .collect()
}

pub fn native_double_byte_slots<'a>(texts: impl IntoIterator<Item = &'a str>) -> BTreeSet<u16> {
    let mut slots = BTreeSet::new();
    for character in texts.into_iter().flat_map(str::chars) {
        let text = character.to_string();
        let (encoded, _, had_errors) = SHIFT_JIS.encode(&text);
        if !had_errors && encoded.len() == 2 && font_98::has_loaded_np2_slot(character) {
            slots.insert(u16::from_be_bytes([encoded[0], encoded[1]]));
        }
    }
    slots
}

#[cfg(test)]
mod tests {
    use super::{characters_needing_carriers, encode_game_text, native_double_byte_slots};
    use vn_font::font_98::{EncodingPlan, SubstitutionMap};

    #[test]
    fn distinguishes_native_and_unencodable_characters() {
        assert_eq!(
            characters_needing_carriers(["あ咦！"].iter().copied()).len(),
            1
        );
        assert!(native_double_byte_slots(["あ咦！"].iter().copied()).contains(&0x82A0));
        assert_eq!(encode_game_text("あ", None).expect("CP932"), [0x82, 0xA0]);
        assert!(encode_game_text("咦", None).is_err());
    }

    #[test]
    fn uses_font_carriers_for_native_cp932_characters_on_unloaded_pages() {
        let substitutions = SubstitutionMap::embedded().expect("substitutions");
        let plan = EncodingPlan::build(&substitutions, [], ["赶黑"]).expect("font plan");
        assert_eq!(
            encode_game_text("赶黑", None).expect("CP932"),
            [0xFB, 0xB0, 0xFC, 0x4B]
        );
        let encoded = encode_game_text("赶黑", Some(&plan)).expect("carriers");
        assert_eq!(
            encoded,
            [
                vn_font::font_98::cp932_for_carrier(plan.carrier_for('赶').expect("赶"))
                    .expect("赶 carrier")[0],
                vn_font::font_98::cp932_for_carrier(plan.carrier_for('赶').expect("赶"))
                    .expect("赶 carrier")[1],
                vn_font::font_98::cp932_for_carrier(plan.carrier_for('黑').expect("黑"))
                    .expect("黑 carrier")[0],
                vn_font::font_98::cp932_for_carrier(plan.carrier_for('黑').expect("黑"))
                    .expect("黑 carrier")[1],
            ]
        );
        assert_ne!(encoded, [0xFB, 0xB0, 0xFC, 0x4B]);
        assert_eq!(
            characters_needing_carriers(["赶黑あA"].iter().copied()),
            ['赶', '黑'].into()
        );
        assert_eq!(
            native_double_byte_slots(["赶黑あ"].iter().copied()),
            [0x82A0].into()
        );
    }
}
