use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

fn table(body: &[u8]) -> Vec<u8> {
    let mut b = vec![0, 0];
    b.extend(body);
    b.extend([0, 0]);
    b
}

#[test]
fn jis_roundtrip_newline_and_non_ascii() {
    let s = "私は、琥珀館に戻った。\n「～－」";
    let b = text::encode(s).unwrap();
    assert_eq!(text::decode(&b).unwrap(), s);
    assert!(b.windows(2).any(|p| p == [0x21, 0x77]));
    assert!(text::encode("A").is_err());
    assert!(text::encode("\0").is_err());
    assert!(text::encode("\r").is_err());
    assert!(text::encode("＠").is_err());
    assert!(text::decode(&[0x21]).is_err());
    assert!(text::decode(&[0x7e, 0x7e]).is_err());
}

#[test]
fn records_keep_slot_identity_and_control_references() {
    let mut b = table(&text::encode("私").unwrap());
    b.extend([0, 1, 0, 0, 0x23, 0x41, 0, 0]);
    b.extend(text::encode("私").unwrap());
    b.extend([0, 0]);
    let t = text::parse("scene.bin", "DISK1.DAT", 40, &b, "scene-text", true).unwrap();
    assert_eq!(t.entries.len(), 2);
    assert_eq!(t.entries[0]._index, 1);
    assert_eq!(t.entries[1]._index, 4);
    assert_eq!(t.entries[0]._source_offset, 42);
    assert_eq!(t.slots[2].kind, "previous-message");
    assert_eq!(t.slots[3].target_message_id, Some(131));
    // Rebuild every byte independently from retained slots and terminators.
    let mut rebuilt = Vec::new();
    for slot in &t.slots {
        for i in (0..slot.raw_hex.len()).step_by(2) {
            rebuilt.push(u8::from_str_radix(&slot.raw_hex[i..i + 2], 16).unwrap());
        }
        rebuilt.extend([0, 0]);
    }
    assert_eq!(rebuilt, b);
}

#[test]
fn malformed_text_is_not_silently_skipped() {
    for b in [
        vec![],
        vec![0],
        vec![0, 0, 0x24, 0x22],
        vec![0x24, 0x22, 0, 0],
        table(&[0, 2]),
        table(&[0, 1]),
    ] {
        assert!(text::parse("bad", "DISK1.DAT", 0, &b, "text", true).is_err());
    }
}

fn synthetic_index() -> Vec<u8> {
    let mut exe = vec![0; 0xb546];
    for at in (0xaef2..0xb546).step_by(6) {
        exe[at + 4] = 2;
    }
    // Empty disk2 entry is an intentional sentinel, not a file.
    exe[0xb13c] = 0;
    exe
}

#[test]
fn indices_deduplicate_shared_ranges_but_preserve_all_references() {
    let c = archive::parse(&synthetic_index(), &[1, 2], &[3, 4]).unwrap();
    assert_eq!(c.references.len(), 270);
    assert_eq!(c.resources.len(), 2);
    assert_eq!(
        c.references.iter().filter(|r| r.resource.is_none()).count(),
        1
    );
    assert_eq!(c.resources[0].bytes, [1, 2]);
    assert_eq!(c.resources[1].bytes, [3, 4]);
}

#[test]
fn invalid_indices_and_incomplete_coverage_fail() {
    let base = synthetic_index();
    assert!(archive::parse(&base[..50], &[1, 2], &[3, 4]).is_err());
    assert!(archive::parse(&base, &[1], &[3, 4]).is_err());
    assert!(archive::parse(&base, &[1, 2, 3], &[3, 4]).is_err());
    let mut overlap = base.clone();
    overlap[0xaef6] = 1;
    assert!(archive::parse(&overlap, &[1, 2], &[3, 4]).is_err());
    let mut bad = base;
    bad[0xb13c] = 1; // illegal nonempty overlapping sentinel
    assert!(archive::parse(&bad, &[1, 2], &[3, 4]).is_err());
}

struct TestDir(PathBuf);
impl TestDir {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "kohaku-tests-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn source(&self) -> Source {
        let p = self.0.join("source.fdi");
        fs::write(&p, b"source snapshot").unwrap();
        Source {
            path: fs::canonicalize(p).unwrap(),
            hash: fivec_new::sha256(b"source snapshot"),
        }
    }
    fn job(&self) -> Prepared {
        Prepared::new(
            vec![self.source()],
            &self.0.join("output"),
            BTreeMap::from([("a.bin".into(), vec![1, 2])]),
            1,
            0,
            0,
        )
        .unwrap()
    }
    fn no_stage(&self) {
        assert!(!fs::read_dir(&self.0).unwrap().any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".kohaku-stage")));
    }
}
impl Drop for TestDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn library_commit_and_existing_output_protection() {
    let d = TestDir::new();
    let job = d.job();
    job.execute().unwrap();
    assert_eq!(fs::read(d.0.join("output/a.bin")).unwrap(), [1, 2]);
    assert_eq!(
        fs::read(d.0.join("source.fdi")).unwrap(),
        b"source snapshot"
    );
    assert!(Prepared::new(
        vec![d.source()],
        &d.0.join("output"),
        BTreeMap::new(),
        0,
        0,
        0
    )
    .is_err());
    d.no_stage();
}

#[test]
fn source_or_output_change_after_prepare_is_rejected() {
    let d = TestDir::new();
    let job = d.job();
    fs::write(d.0.join("source.fdi"), b"changed").unwrap();
    assert!(job.execute().unwrap_err().contains("changed"));
    assert!(!d.0.join("output").exists());
    let job = d.job();
    fs::create_dir(d.0.join("output")).unwrap();
    fs::write(d.0.join("output/keep"), b"user data").unwrap();
    assert!(job.execute().is_err());
    assert_eq!(fs::read(d.0.join("output/keep")).unwrap(), b"user data");
    d.no_stage();
}

#[test]
fn input_paths_and_traversal_are_protected_in_library() {
    let d = TestDir::new();
    let source = d.source();
    assert!(Prepared::new(vec![source.clone()], &source.path, BTreeMap::new(), 0, 0, 0).is_err());
    assert!(Prepared::new(
        vec![source.clone()],
        &d.0.join("output"),
        BTreeMap::from([("../escape".into(), vec![1])]),
        0,
        0,
        0
    )
    .is_err());
    assert!(Prepared::new(
        vec![source],
        &d.0.join("missing/output"),
        BTreeMap::new(),
        0,
        0,
        0
    )
    .is_err());
    d.no_stage();
}

#[test]
fn staging_failure_leaves_no_partial_output() {
    let d = TestDir::new();
    let job = Prepared::new(
        vec![d.source()],
        &d.0.join("output"),
        BTreeMap::from([("a".into(), vec![1]), ("a/b".into(), vec![2])]),
        0,
        0,
        0,
    )
    .unwrap();
    assert!(job.execute().is_err());
    assert!(!d.0.join("output").exists());
    d.no_stage();
}

fn plan(texts: &[&str], reserved: &[u16]) -> vn_font::font_98::EncodingPlan {
    use vn_font::font_98::{self, EncodingPlan, SubstitutionMap};
    let forbidden = [
        [0x21, 0x77],
        [0x23, 0x41],
        [0x23, 0x42],
        [0x23, 0x43],
        [0x23, 0x44],
        [0x23, 0x45],
        [0x23, 0x46],
    ]
    .map(|jis| u16::from_be_bytes(font_98::jis_to_cp932(jis).unwrap()));
    EncodingPlan::build_with_forbidden_cp932(
        &SubstitutionMap::embedded().unwrap(),
        reserved.iter().copied(),
        forbidden,
        texts.iter().copied(),
    )
    .unwrap()
}

#[test]
fn chinese_black_gan_mixed_variants_and_reserved_controls_roundtrip() {
    use vn_font::font_98;
    let sample = "黑赶黒趕黑赶ＡＢＣＤＥＦ＠A";
    let reserved = [u16::from_be_bytes(
        font_98::cp932_for_carrier('黒').unwrap(),
    )];
    let p = plan(&[sample], &reserved);
    let bytes = workflow::encode_message(&format!("{sample}\n赶黑"), &p).unwrap();
    assert_eq!(
        p.decode_carriers(&text::decode(&bytes).unwrap()),
        format!("{sample}\n赶黑")
    );
    let carriers = "黑赶黒趕"
        .chars()
        .map(|c| p.carrier_for(c).unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(carriers.len(), 4);
    assert_eq!(p.carrier_for('黒').unwrap(), '黒');
    assert!(!p.requests().iter().any(|r| r.carrier == '黒'));
    for c in "ＡＢＣＤＥＦ＠".chars() {
        let encoded = workflow::encode_message(&c.to_string(), &p).unwrap();
        let doc = text::parse(
            "scene",
            "DISK1.DAT",
            0,
            &table(&encoded),
            "scene-text",
            true,
        )
        .unwrap();
        assert_eq!(doc.entries.len(), 1);
        assert_eq!(p.decode_carriers(&doc.entries[0].scr_msg), c.to_string());
    }
    #[cfg(windows)]
    {
        let f = font_98::prepare_font(
            font_98::EMBEDDED_FONT,
            &p.requests(),
            &reserved.into_iter().collect(),
            font_98::FONT_FACE,
        )
        .unwrap();
        let glyphs = "黑赶黒趕"
            .chars()
            .map(|c| font_98::read_glyph(&f.bytes, p.carrier_for(c).unwrap()).unwrap())
            .collect::<Vec<_>>();
        for g in &glyphs {
            assert!(g.iter().any(|&b| b != 255));
        }
        assert_eq!(
            glyphs
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            4
        );
        assert_eq!(
            glyphs[2],
            font_98::read_glyph(font_98::EMBEDDED_FONT, '黒').unwrap()
        );
    }
}

#[test]
fn shortened_and_longer_messages_preserve_sentinels_and_references() {
    let mut raw = table(&text::encode("長い原文です").unwrap());
    raw.extend([0, 1, 0, 0, 0x23, 0x41, 0, 0]);
    raw.extend(text::encode("私").unwrap());
    raw.extend([0, 0, 0, 0]);
    let mut doc = text::parse("scene", "DISK1.DAT", 0, &raw, "scene-text", true).unwrap();
    assert_eq!(workflow::rebuild_text(&doc, &plan(&[], &[])).unwrap(), raw);
    doc.entries[0].message = "黑".into();
    doc.entries[1].message = "赶快调查\n黑夜将至".into();
    let p = plan(&["黑赶快调查夜将至"], &[]);
    let rebuilt = workflow::rebuild_text(&doc, &p).unwrap();
    let reread = text::parse("scene", "DISK1.DAT", 0, &rebuilt, "scene-text", true).unwrap();
    assert_eq!(reread.slots.len(), doc.slots.len());
    for i in [0, 2, 3, 5] {
        assert_eq!(reread.slots[i].raw_hex, doc.slots[i].raw_hex);
    }
    for (a, b) in doc.entries.iter().zip(&reread.entries) {
        assert_eq!(a._index, b._index);
        assert_eq!(a.message, p.decode_carriers(&b.scr_msg));
    }
}

#[test]
fn translation_validation_rejects_structural_and_layout_damage() {
    for s in ["", "黑\0赶", "黑\r\n赶", "\t", "😀"] {
        assert!(workflow::validate_message(s).is_err());
    }
    assert!(workflow::validate_dialogue_layout(&"黑".repeat(155)).is_ok());
    assert!(workflow::validate_dialogue_layout(&"黑".repeat(156)).is_err());
    assert!(workflow::validate_dialogue_layout("黑\n赶\n黑\n赶\n黑").is_ok());
    assert!(workflow::validate_dialogue_layout("黑\n赶\n黑\n赶\n黑\n赶").is_err());
}

fn merge_fixture() -> (BTreeMap<String, text::Translation>, serde_json::Value) {
    let mut raw = table(&text::encode("私").unwrap());
    raw.extend(text::encode("私").unwrap());
    raw.extend([0, 0]);
    let doc = text::parse("scene", "DISK1.DAT", 20, &raw, "scene-text", true).unwrap();
    let value = serde_json::json!({"source":doc.source,"source_offset":doc.source_offset,
        "source_sha256":doc.source_sha256,"entries":[{"_index":1,"scr_msg":"私","message":"黑赶"}]});
    (BTreeMap::from([("scene".into(), doc)]), value)
}

#[test]
fn partial_json_uses_slot_or_offset_and_rejects_ambiguity_and_metadata_edits() {
    use std::collections::BTreeSet;
    let (mut docs, value) = merge_fixture();
    let mut selected = BTreeSet::new();
    workflow::merge_document(&mut docs, &value, &mut selected).unwrap();
    assert_eq!(docs["scene"].entries[0].message, "黑赶");
    assert_eq!(docs["scene"].entries[1].message, "私");
    assert!(workflow::merge_document(&mut docs, &value, &mut selected).is_err());
    for field in ["scr_msg", "_file", "_offset", "_raw_hex"] {
        let (mut docs, mut bad) = merge_fixture();
        bad["entries"][0][field] = serde_json::json!("changed");
        assert!(workflow::merge_document(&mut docs, &bad, &mut BTreeSet::new()).is_err());
    }
    let (mut docs, mut by_offset) = merge_fixture();
    by_offset["entries"][0]
        .as_object_mut()
        .unwrap()
        .remove("_index");
    assert!(
        workflow::merge_document(&mut docs, &by_offset, &mut BTreeSet::new())
            .unwrap_err()
            .contains("ambiguous")
    );
    by_offset["entries"][0]["_offset"] = serde_json::json!(2);
    workflow::merge_document(&mut docs, &by_offset, &mut BTreeSet::new()).unwrap();
    let (mut docs, mut bad) = merge_fixture();
    bad["source_sha256"] = serde_json::json!("bad");
    assert!(workflow::merge_document(&mut docs, &bad, &mut BTreeSet::new()).is_err());
    let (mut docs, mut bad) = merge_fixture();
    bad["slots"] = serde_json::json!([]);
    assert!(workflow::merge_document(&mut docs, &bad, &mut BTreeSet::new()).is_err());
}

#[test]
fn moved_com_music_identifiers_are_still_excluded() {
    let mut raw = vec![0, 0];
    for _ in 1..186 {
        raw.extend(text::encode("私").unwrap());
        raw.extend([0, 0]);
    }
    let mut doc = text::parse(
        "KOHAKU.COM/labels",
        "KOHAKU.COM",
        0x888f,
        &raw,
        "ui-label",
        false,
    )
    .unwrap();
    doc.entries[0].message = "黑赶".into();
    let rebuilt = workflow::rebuild_text(&doc, &plan(&["黑赶"], &[])).unwrap();
    let actual = text::parse(
        "KOHAKU.COM/labels",
        "KOHAKU.COM",
        0x888f,
        &rebuilt,
        "ui-label",
        false,
    )
    .unwrap();
    assert_eq!(actual.entries.len(), doc.entries.len());
    assert_eq!(actual.slots[170].offset, doc.slots[170].offset + 2);
    for i in 170..184 {
        assert_eq!(actual.slots[i].kind, "resource-label");
        assert_eq!(actual.slots[i].raw_hex, doc.slots[i].raw_hex);
    }
}

#[test]
fn resized_dat_updates_all_aliases_and_later_offsets() {
    let mut exe = synthetic_index();
    // A separate final DISK1 resource forces its offset to move.
    exe[0xb132 + 2..0xb132 + 4].copy_from_slice(&2u16.to_le_bytes());
    let original = archive::parse(&exe, &[1, 2, 3, 4], &[5, 6]).unwrap();
    let key = original.resources[0].path.clone();
    let (patched, dat) = workflow::rebuild_archives(
        &original,
        &exe,
        &BTreeMap::from([(key.clone(), vec![7; 20])]),
    )
    .unwrap();
    let rebuilt = archive::parse(&patched, &dat["DISK1.DAT"], &dat["DISK2.DAT"]).unwrap();
    assert_eq!(rebuilt.resources[1].offset, 20);
    assert_eq!(rebuilt.resources[1].bytes, [3, 4]);
    assert_eq!(dat["DISK2.DAT"], [5, 6]);
    for r in rebuilt
        .references
        .iter()
        .filter(|r| r.resource.as_ref() == Some(&key))
    {
        assert_eq!(r.size, 20);
    }
    assert!(
        workflow::rebuild_archives(&original, &exe, &BTreeMap::from([(key, vec![0; 0x5002])]))
            .is_err()
    );
}

#[test]
fn translation_directory_snapshot_detects_add_remove_and_mutation() {
    for mode in 0..3 {
        let d = TestDir::new();
        let dir = d.0.join("translations");
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("a.json"), b"original").unwrap();
        let source = Source {
            path: fs::canonicalize(&dir).unwrap(),
            hash: workflow::source_hash(&dir).unwrap(),
        };
        assert!(Prepared::new(
            vec![source.clone()],
            &dir.join("output"),
            BTreeMap::new(),
            0,
            0,
            0
        )
        .is_err());
        let job =
            Prepared::new(vec![source], &d.0.join("output"), BTreeMap::new(), 0, 0, 0).unwrap();
        match mode {
            0 => fs::write(dir.join("new.json"), b"new").unwrap(),
            1 => fs::remove_file(dir.join("a.json")).unwrap(),
            _ => fs::write(dir.join("a.json"), b"changed").unwrap(),
        }
        assert!(job.execute().is_err());
        assert!(!d.0.join("output").exists());
        d.no_stage();
    }
}

fn fat_fixture() -> Vec<u8> {
    let mut b = vec![0; 4096 + 128 * 512];
    for (i, v) in [4096u32, 128 * 512, 512, 16, 2, 4].into_iter().enumerate() {
        b[8 + i * 4..12 + i * 4].copy_from_slice(&v.to_le_bytes());
    }
    let h = 4096;
    for (at, v) in [(11, 512u16), (14, 1), (17, 16), (19, 128), (22, 1)] {
        b[h + at..h + at + 2].copy_from_slice(&v.to_le_bytes());
    }
    b[h + 13] = 1;
    b[h + 16] = 2;
    b[h + 21] = 0xf0;
    for base in [h + 512, h + 1024] {
        b[base..base + 6].copy_from_slice(&[0xf0, 0xff, 0xff, 0xff, 0xff, 0xff]);
    }
    let root = h + 1536;
    for (i, name) in [b"TEXT    DAT", b"KEEP    BIN", b"EMPTY   BIN"]
        .into_iter()
        .enumerate()
    {
        let at = root + i * 32;
        b[at..at + 11].copy_from_slice(name);
        b[at + 11] = 0x20;
        if i < 2 {
            b[at + 26..at + 28].copy_from_slice(&(i as u16 + 2).to_le_bytes());
            b[at + 28..at + 32].copy_from_slice(&3u32.to_le_bytes());
        }
    }
    b[h + 2048..h + 2051].copy_from_slice(b"old");
    b[h + 2560..h + 2563].copy_from_slice(b"xyz");
    b
}

#[test]
fn fdi_noop_growth_shrink_and_empty_source_preserve_other_files() {
    let original = fat_fixture();
    assert_eq!(
        disk::rebuild(&original, &BTreeMap::new()).unwrap(),
        original
    );
    let grown = disk::rebuild(
        &original,
        &BTreeMap::from([
            ("TEXT.DAT".into(), vec![0x42; 1700]),
            ("EMPTY.BIN".into(), vec![0x43; 700]),
        ]),
    )
    .unwrap();
    assert_eq!(&grown[..4096], &original[..4096]);
    let shrunk = disk::rebuild(
        &grown,
        &BTreeMap::from([("TEXT.DAT".into(), vec![0x44; 2])]),
    )
    .unwrap();
    let image = fivec_new::Archive::from_bytes("sample.fdi", shrunk).unwrap();
    let v = &image.volumes()[0];
    let files = &v.filesystem.as_ref().unwrap().files;
    for (name, bytes) in [
        ("TEXT.DAT", vec![0x44; 2]),
        ("KEEP.BIN", b"xyz".to_vec()),
        ("EMPTY.BIN", vec![0x43; 700]),
    ] {
        let f = files.iter().find(|f| f.display_path == name).unwrap();
        assert_eq!(image.read_file(&v.id, &f.id).unwrap(), bytes);
    }
    assert!(disk::rebuild(
        &original,
        &BTreeMap::from([("TEXT.DAT".into(), vec![1; 128 * 512])])
    )
    .unwrap_err()
    .contains("space"));
    assert!(disk::rebuild(&original, &BTreeMap::from([("MISSING".into(), vec![1])])).is_err());
    let mut corrupt = original.clone();
    corrupt[4096 + 1024 + 5] ^= 1;
    assert!(disk::rebuild(&corrupt, &BTreeMap::new()).is_err());
    assert!(disk::rebuild(&original[..4097], &BTreeMap::new()).is_err());
}
