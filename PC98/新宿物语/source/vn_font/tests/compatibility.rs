//! Frozen original implementations are compiled only for differential tests.
#![allow(dead_code)]
#[rustfmt::skip]
#[allow(clippy::all)]
#[path = "reference/pc88/src/font.rs"]
mod original_88;
#[rustfmt::skip]
#[allow(clippy::all)]
#[path = "reference/pc98/src/font.rs"]
mod original_98;

use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use vn_font::{font_88, font_98};

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn compare_88(source: &[u8]) {
    let old = original_88::FontResources::load_embedded().unwrap();
    let new = font_88::FontResources::load_embedded().unwrap();
    let texts = ["你好汉语翻译测试", "日本語", "测试文本"];
    let reserved = [
        original_88::strict_cp932_pair('日').unwrap(),
        original_88::strict_cp932_pair('語').unwrap(),
    ];
    let old_plan = old.plan_dynamic_mapping(reserved, texts).unwrap();
    let new_plan = new.plan_dynamic_mapping(reserved, texts).unwrap();
    assert_eq!(
        serde_json::to_value(&old_plan).unwrap(),
        serde_json::to_value(&new_plan).unwrap()
    );
    for text in texts {
        assert_eq!(
            old.encode_ai1_text(text, &old_plan).unwrap().bytes,
            new.encode_text(text, &new_plan).unwrap().bytes
        );
    }
    let before = source.to_vec();
    let old_built = old.build_rom(source, &old_plan).unwrap();
    let new_built = new.build_rom(source, &new_plan).unwrap();
    assert_eq!(old_built.rom, new_built.rom);
    assert_eq!(source, before);
    assert!(new_built.manifest.untouched_bytes_preserved);
    let old_previews = old
        .render_preview_pages(
            &old_built.rom,
            &old_plan,
            &original_88::PreviewOptions::default(),
        )
        .unwrap();
    let new_previews = new
        .render_preview_pages(
            &new_built.rom,
            &new_plan,
            &font_88::PreviewOptions::default(),
        )
        .unwrap();
    assert_eq!(old_previews.len(), new_previews.len());
    for (old, new) in old_previews.iter().zip(new_previews) {
        assert_eq!(old.bmp, new.bmp);
    }
    let replay = new
        .plan_from_bytes(&serde_json::to_vec(&new_plan).unwrap())
        .unwrap();
    assert_eq!(new.build_rom(source, &replay).unwrap().rom, new_built.rom);
}

#[test]
fn pc88_new_and_original_outputs_match() {
    compare_88(&vec![0xa5; font_88::KANJI1_ROM_SIZE]);
}

#[test]
#[ignore = "Set VN_FONT_TEST_ROM to an original KANJI1.ROM, then run --ignored"]
fn pc88_real_rom_outputs_match() {
    let path = std::env::var_os("VN_FONT_TEST_ROM").expect("VN_FONT_TEST_ROM is required");
    let before = std::fs::read(&path).unwrap();
    compare_88(&before);
    assert_eq!(std::fs::read(path).unwrap(), before);
}

#[cfg(windows)]
#[test]
fn pc98_new_and_original_outputs_match_with_game_normalization_outside_library() {
    let texts = ["你好，日本語!", "ABC 123", "测试文本。"];
    let normalized = texts
        .iter()
        .map(|s| {
            s.chars()
                .map(original_98::normalize_character)
                .collect::<Result<String, _>>()
                .unwrap()
        })
        .collect::<Vec<_>>();
    let old_plan = original_98::EncodingPlan::build(texts).unwrap();
    let new_plan = font_98::EncodingPlan::build(
        &font_98::SubstitutionMap::embedded().unwrap(),
        [],
        normalized.iter().map(String::as_str),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(old_plan.manifest_entries().unwrap()).unwrap(),
        serde_json::to_value(new_plan.manifest_entries().unwrap()).unwrap()
    );
    let old = original_98::prepare_font(&old_plan.requests(), &BTreeSet::new()).unwrap();
    let new = font_98::prepare_font(
        font_98::EMBEDDED_FONT,
        &new_plan.requests(),
        &BTreeSet::new(),
        font_98::FONT_FACE,
    )
    .unwrap();
    assert_eq!(old.bytes, new.bytes);
    assert_eq!(old.patched_glyphs, new.patched_glyphs);
}

#[test]
fn pc88_loaded_mapping_cannot_silently_overwrite_other_texts_native_character() {
    let resources = font_88::FontResources::load_embedded().unwrap();
    let plan = resources.plan_dynamic_mapping([], ["你"]).unwrap();
    let replay = resources
        .mapping_from_bytes(&font_88::mapping_used_json_bytes(&plan).unwrap())
        .unwrap();
    let native_text = replay.mapping_used[0].carrier.to_string();
    assert!(resources.encode_text(&native_text, &replay).is_err());
}

#[test]
fn pc88_rejects_invalid_jis_and_forged_full_plan_metadata() {
    assert_eq!(font_88::jis_to_rom_address(0x307f), None);
    let resources = font_88::FontResources::load_embedded().unwrap();
    let mut plan = resources.plan_dynamic_mapping([], ["日本語你"]).unwrap();
    plan.native_double_byte[0].rom_byte_offset += 32;
    assert!(resources
        .plan_from_bytes(&serde_json::to_vec(&plan).unwrap())
        .is_err());
    assert!(resources
        .build_rom(&vec![0; font_88::KANJI1_ROM_SIZE], &plan)
        .is_err());
}

#[test]
fn resources_can_be_supplied_externally_and_must_match() {
    let custom =
        font_88::FontResources::from_bytes(font_88::EMBEDDED_MAPPING, font_88::EMBEDDED_GLYPHS)
            .unwrap();
    assert!(custom.plan_dynamic_mapping([], ["你"]).is_ok());
    assert!(font_88::FontResources::from_bytes(b"{}", font_88::EMBEDDED_GLYPHS).is_err());
    assert!(font_98::SubstitutionMap::from_json(br#"{"a":"x"}"#).is_err());
    assert!(
        font_98::SubstitutionMap::from_json(font_98::EMBEDDED_SUBSTITUTIONS.as_bytes()).is_ok()
    );
}

#[test]
fn pc98_reserved_native_slots_remain_out_of_patch_requests() {
    let resources = font_98::SubstitutionMap::embedded().unwrap();
    let reserved_code = u16::from_be_bytes(font_98::cp932_for_carrier('凜').unwrap());
    let plan = font_98::EncodingPlan::build(&resources, [reserved_code], ["你凜"]).unwrap();
    assert_ne!(plan.carrier_for('你').unwrap(), '凜');
    assert_eq!(plan.carrier_for('凜').unwrap(), '凜');
    assert!(plan.requests().iter().all(|r| r.carrier != '凜'));
    assert!(font_98::prepare_font(
        font_98::EMBEDDED_FONT,
        &[font_98::FontPatchRequest {
            carrier: '凜',
            replacement: '你'
        }],
        &BTreeSet::from([reserved_code]),
        font_98::FONT_FACE
    )
    .is_err());
}

#[test]
fn pc98_does_not_apply_genji_fullwidth_conversion() {
    let plan =
        font_98::EncodingPlan::build(&font_98::SubstitutionMap::embedded().unwrap(), [], ["AＡ"])
            .unwrap();
    let entries = plan.manifest_entries().unwrap();
    assert!(entries.iter().any(|e| e.character == "A"));
    assert!(entries.iter().any(|e| e.character == "Ａ"));
    assert_ne!(
        plan.carrier_for('A').unwrap(),
        plan.carrier_for('Ａ').unwrap()
    );
}

#[test]
fn pc98_physical_slot_aliases_cannot_hold_distinct_glyphs() {
    assert_eq!(
        font_98::cp932_for_carrier('−').unwrap(),
        font_98::cp932_for_carrier('－').unwrap()
    );
    let resources = font_98::SubstitutionMap::embedded().unwrap();
    assert!(font_98::EncodingPlan::build(&resources, [], ["−－"]).is_err());
    let requests = [
        font_98::FontPatchRequest {
            carrier: '−',
            replacement: '你',
        },
        font_98::FontPatchRequest {
            carrier: '－',
            replacement: '好',
        },
    ];
    assert!(font_98::prepare_font(
        font_98::EMBEDDED_FONT,
        &requests,
        &BTreeSet::new(),
        font_98::FONT_FACE
    )
    .is_err());
}

#[test]
fn pc98_fullwidth_space_is_always_reserved() {
    let resources = font_98::SubstitutionMap::from_json("{\"你\":\"　\"}".as_bytes()).unwrap();
    assert!(font_98::EncodingPlan::build(&resources, [], ["你　"]).is_err());
    assert!(font_98::prepare_font(
        font_98::EMBEDDED_FONT,
        &[font_98::FontPatchRequest {
            carrier: '　',
            replacement: '你',
        }],
        &BTreeSet::new(),
        font_98::FONT_FACE,
    )
    .is_err());
}

#[cfg(windows)]
#[test]
fn pc98_same_glyph_slot_aliases_are_patched_once() {
    let requests = [
        font_98::FontPatchRequest {
            carrier: '−',
            replacement: '你',
        },
        font_98::FontPatchRequest {
            carrier: '－',
            replacement: '你',
        },
    ];
    let once = font_98::prepare_font(
        font_98::EMBEDDED_FONT,
        &requests[..1],
        &BTreeSet::new(),
        font_98::FONT_FACE,
    )
    .unwrap();
    let twice = font_98::prepare_font(
        font_98::EMBEDDED_FONT,
        &requests,
        &BTreeSet::new(),
        font_98::FONT_FACE,
    )
    .unwrap();
    assert_eq!(twice.patched_glyphs, 1);
    assert_eq!(once.bytes, twice.bytes);
}

#[cfg(windows)]
#[test]
fn pc98_preserves_external_baseline_and_rejects_missing_font() {
    let mut source = font_98::EMBEDDED_FONT.to_vec();
    source[100] ^= 1;
    let request = [font_98::FontPatchRequest {
        carrier: '凜',
        replacement: '你',
    }];
    let built =
        font_98::prepare_font(&source, &request, &BTreeSet::new(), font_98::FONT_FACE).unwrap();
    assert_eq!(built.bytes[100], source[100]);
    assert_eq!(source[100], font_98::EMBEDDED_FONT[100] ^ 1);
    assert!(font_98::prepare_font(
        &source,
        &request,
        &BTreeSet::new(),
        "vn-font-nonexistent-face-000"
    )
    .is_err());
    assert!(font_98::prepare_font(&source, &request, &BTreeSet::new(), "").is_err());
    assert!(font_98::prepare_font(&source, &request, &BTreeSet::new(), "新宋体\0other").is_err());
}
