use super::*;
use std::{collections::BTreeMap, fs, path::PathBuf};
fn demo() -> Vec<u8> {
    fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("work/suishou/first_sentence_nes/suishou_first_sentence_mmc5.nes"),
    )
    .expect("verified demo fixture")
}
fn edit(game: &nes::Nes, message: &str) -> BTreeMap<String, Vec<vn_text::Entry>> {
    let mut view = game.native.resources[0].view();
    view[0].message = message.into();
    BTreeMap::from([("DATA0".into(), view)])
}
#[test]
fn exact_noop_and_full_inventory() -> Result<()> {
    let raw = demo();
    let game = nes::Nes::parse(&raw)?;
    assert_eq!(
        game.native
            .resources
            .iter()
            .filter(|r| r.name.starts_with("DATA"))
            .map(|r| r.texts.len())
            .sum::<usize>(),
        237
    );
    assert_eq!(
        game.native
            .resources
            .iter()
            .flat_map(|r| &r.scenes)
            .map(|s| s.events.len())
            .sum::<usize>(),
        471
    );
    let views = game
        .native
        .resources
        .iter()
        .map(|r| (r.name.clone(), r.view()))
        .collect();
    assert_eq!(game.import(&views, "SimHei")?.image, raw);
    Ok(())
}
#[test]
fn short_long_newlines_empty_and_incremental() -> Result<()> {
    let game = nes::Nes::parse(&demo())?;
    for text in [
        "龙",
        "",
        "水晶之龙\n向我们扑了过来！\n我们",
        &"水晶之龙向我们扑了过来！".repeat(160),
    ] {
        let rebuilt = game.import(&edit(&game, text), "SimHei")?;
        let new = nes::Nes::parse(&rebuilt.image)?;
        assert_eq!(new.native.resources[0].texts[0].original, text);
        assert_eq!(
            new.native.resources[0].scenes,
            game.native.resources[0].scenes
        );
        let again = new.import(&edit(&new, "我们向龙扑了过来！"), "SimHei")?;
        assert_eq!(
            nes::Nes::parse(&again.image)?.native.resources[0].texts[0].original,
            "我们向龙扑了过来！"
        );
        assert_eq!(
            new.import(&BTreeMap::new(), "nonexistent")?.image,
            rebuilt.image
        );
    }
    Ok(())
}
#[test]
fn one_mb_and_bank_boundary() -> Result<()> {
    let game = nes::Nes::parse(&demo())?;
    let text = "水".repeat(39 * 2800);
    let rebuilt = game.import(&edit(&game, &text), "SimHei")?;
    let new = nes::Nes::parse(&rebuilt.image)?;
    assert_eq!(new.banks, 128);
    assert_eq!(new.native.resources[0].texts[0].original, text);
    Ok(())
}
#[test]
fn rejects_corruption_and_invalid_controls() -> Result<()> {
    let game = nes::Nes::parse(&demo())?;
    for text in ["水\0龙", "水\r\n龙", "😀"] {
        assert!(game.import(&edit(&game, text), "SimHei").is_err());
    }
    let mut view = edit(&game, "水");
    view.get_mut("DATA0").unwrap().pop();
    assert!(game.import(&view, "SimHei").is_err());
    let mut raw = game.import(&edit(&game, "水"), "SimHei")?.image;
    raw[16 + 4 * nes::BANK + 1] ^= 1;
    assert!(nes::Nes::parse(&raw).is_err());
    Ok(())
}
#[test]
fn safe_publish_and_changed_input() -> Result<()> {
    let root = std::env::temp_dir().join(format!("suishou-test-{}", std::process::id()));
    fs::create_dir(&root)?;
    let outcome = (|| -> Result<()> {
        let source = root.join("source.bin");
        fs::write(&source, b"source")?;
        let mut inputs = io::Inputs::default();
        inputs.read(&source)?;
        let files = BTreeMap::from([("a/b.bin".into(), b"new".to_vec())]);
        let output = root.join("output");
        io::publish(&inputs, &output, &files)?;
        assert!(io::publish(&inputs, &output, &files).is_err());
        assert!(io::publish(&inputs, &source, &files).is_err());
        fs::write(&source, b"changed")?;
        assert!(io::publish(&inputs, &root.join("changed"), &files).is_err());
        assert!(!root.join("changed").exists());
        assert!(io::publish(
            &io::Inputs::default(),
            &root.join("escape"),
            &BTreeMap::from([("../escape".into(), vec![])])
        )
        .is_err());
        Ok(())
    })();
    fs::remove_dir_all(&root)?;
    outcome
}
#[test]
fn original_fds_noop() -> Result<()> {
    let raw = fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("work/suishou/first_sentence_nes/source.fds"),
    )?;
    let game = game::Game::parse(&raw)?;
    let views = game
        .native
        .resources
        .iter()
        .map(|r| (r.name.clone(), r.view()))
        .collect();
    assert_eq!(game.import(&views)?.image, raw);
    Ok(())
}

#[test]
fn interface_inventory_and_state_preserving_roundtrip() -> Result<()> {
    let game = nes::Nes::parse(&demo())?;
    let items = game
        .native
        .resources
        .iter()
        .find(|r| r.name == "ITEMS")
        .unwrap();
    let system = game
        .native
        .resources
        .iter()
        .find(|r| r.name == "SYSTEM")
        .unwrap();
    assert_eq!(items.texts.len(), 35);
    assert_eq!(system.texts.len(), 6);
    let engine = game
        .native
        .resources
        .iter()
        .find(|r| r.name == "ENGINE")
        .unwrap();
    assert_eq!(engine.texts.len(), 10);
    assert!(engine.texts[8].original.contains("{error_code}"));
    assert_eq!(engine.texts[9].original, "OK.");
    let mut translations = items.view();
    let mut target = None;
    for (i, t) in items.texts.iter().enumerate() {
        if matches!(
            t.ui,
            Some(ui::Locator::ItemVariant {
                item: 1,
                state: 3,
                ..
            })
        ) {
            assert_eq!(t.original, "おかね.100クレジットもっている.");
            translations[i].message = "金钱：持有100信用点。".into();
            target = Some(i);
        }
    }
    let mut pendant = false;
    for t in &items.texts {
        if matches!(
            t.ui,
            Some(ui::Locator::ItemVariant {
                item: 0,
                state: 0,
                ..
            })
        ) {
            assert!(t.original.contains("ひからない"));
            pendant = true;
        }
    }
    assert!(pendant);
    let result = game.import(
        &BTreeMap::from([("ITEMS".into(), translations.clone())]),
        "SimHei",
    )?;
    let new = nes::Nes::parse(&result.image)?;
    let rebuilt = new
        .native
        .resources
        .iter()
        .find(|r| r.name == "ITEMS")
        .unwrap();
    assert_eq!(rebuilt.view(), translations);
    assert!(rebuilt.texts[target.unwrap()].ui_reference.is_some());
    let fix_old = game
        .native
        .files
        .iter()
        .find(|f| f.name == "FIXDAT")
        .unwrap();
    let fix_new = new
        .native
        .files
        .iter()
        .find(|f| f.name == "FIXDAT")
        .unwrap();
    assert_eq!(
        &game.payload(fix_old)[..0x780],
        &new.payload(fix_new)[..0x780]
    );
    let mut controls = system.view();
    controls[2].message = "读取".into();
    assert!(game
        .import(&BTreeMap::from([("SYSTEM".into(), controls)]), "SimHei")
        .is_err());
    Ok(())
}

#[test]
fn previous_engine_read_and_upgrade_on_edit() -> Result<()> {
    let raw = fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("work/suishou/user_translation_20261014/rebuilt/source.nes"),
    )?;
    let old = nes::Nes::parse(&raw)?;
    let repaired = old.repair_runtime()?;
    let new = nes::Nes::parse(&repaired.image)?;
    assert!(new.current_runtime);
    let code_banks = old
        .native
        .files
        .iter()
        .filter(|f| matches!(f.name.as_str(), "ADVS" | "FIXDAT"))
        .flat_map(|f| {
            (f.data_offset - 16) / nes::BANK
                ..(f.data_offset - 16) / nes::BANK + f.size.div_ceil(nes::BANK)
        })
        .collect::<std::collections::BTreeSet<_>>();
    for bank in 0..old.banks - 1 {
        if !code_banks.contains(&bank) {
            let at = 16 + bank * nes::BANK;
            assert_eq!(
                &raw[at..at + nes::BANK],
                &repaired.image[at..at + nes::BANK]
            );
        }
    }
    assert_eq!(old.glyphs, new.glyphs);
    assert!(!old.current_runtime);
    assert_eq!(old.import(&BTreeMap::new(), "SimHei")?.image, raw);
    let upgraded = old.import(&edit(&old, "水晶之龙"), "SimHei")?;
    assert!(nes::Nes::parse(&upgraded.image)?.current_runtime);
    Ok(())
}

#[test]
fn conversion_preserves_full_scripts_and_rejects_wrong_bios() -> Result<()> {
    let raw = fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("work/suishou/first_sentence_nes/source.fds"),
    )?;
    let seed = demo();
    let bios = &seed[seed.len() - nes::BANK..];
    let new = nes::Nes::convert(&raw, bios)?;
    let old = game::Game::parse(&raw)?;
    for (a, b) in old.native.resources.iter().zip(&new.native.resources) {
        assert_eq!(a.view(), b.view());
        assert_eq!(a.scenes, b.scenes);
    }
    let patched = new.import(&edit(&new, "你好，水晶之龙！"), "SimHei")?;
    assert_eq!(
        nes::Nes::parse(&patched.image)?.native.resources[0].texts[0].original,
        "你好，水晶之龙！"
    );
    assert!(nes::Nes::convert(&raw, &vec![0; nes::BANK]).is_err());
    Ok(())
}
