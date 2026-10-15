mod catalog;
mod chinese;
mod rebuild;
pub mod workflow;

use std::collections::BTreeMap;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub const INPUT_FILES: [&str; 8] = [
    "source.nes",
    "native.json",
    "translations/fixed.json",
    "translations/resources_bank01.json",
    "translations/resources_bank02.json",
    "translations/resources_bank03.json",
    "translations/macros.json",
    "translations/intro.json",
];

pub struct ExportPlan {
    pub files: BTreeMap<String, Vec<u8>>,
    pub summary: String,
}

pub struct ImportPlan {
    pub rom: Vec<u8>,
    pub report: serde_json::Value,
}

pub fn chinese_import_plan(
    original_files: &BTreeMap<String, Vec<u8>>,
    translations: &BTreeMap<String, Vec<u8>>,
    font_manifest: &[u8],
    font_data: &[u8],
) -> Result<ExportPlan> {
    let plan = chinese::integrate_runtime(chinese::compile(
        original_files,
        translations,
        font_manifest,
        font_data,
    )?)?;
    let mut files = plan.files;
    files.insert("rebuilt.nes".into(), plan.rom);
    files.insert("source.nes".into(), original_files["source.nes"].clone());
    files.insert("native.json".into(), original_files["native.json"].clone());
    Ok(ExportPlan {
        files,
        summary: format!(
            "回注{}个中文片段、{}个字形，512 KiB PRG / MMC3 / 32 KiB CHR-RAM 运行时已接入。",
            plan.report["editable_fragments"], plan.config.font_count
        ),
    })
}

pub fn chinese_export_plan(
    rom: &[u8],
    metadata: &[u8],
    font_manifest: &[u8],
) -> Result<ExportPlan> {
    chinese::reextract(rom, metadata, font_manifest)
}

pub fn export_plan(rom: &[u8]) -> Result<ExportPlan> {
    let parsed = catalog::parse(rom)?;
    let mut files = BTreeMap::from([
        ("source.nes".into(), rom.to_vec()),
        ("native.json".into(), json_bytes(&parsed)?),
    ]);
    let mut review = String::from("unit\trecord\tfragment\tmessage\n");
    for unit in &parsed.units {
        files.insert(
            format!("translations/{}.json", unit.key),
            vn_text::write_template(&unit.view())?,
        );
        for record in &unit.records {
            for (i, span) in record.spans.iter().enumerate() {
                review.push_str(&format!(
                    "{}\t{}\t{}\t{}\n",
                    unit.key,
                    record.index,
                    i,
                    span.message.replace('\t', "\\t").replace('\n', "\\n")
                ));
            }
        }
    }
    files.insert("review.tsv".into(), review.into_bytes());
    Ok(ExportPlan {
        files,
        summary: format!(
            "导出 {} 个源记录、{} 个可编辑文本片段",
            parsed.record_count(),
            parsed.fragment_count()
        ),
    })
}

pub fn import_plan(files: &BTreeMap<String, Vec<u8>>, expand: bool) -> Result<ImportPlan> {
    let source = files.get("source.nes").ok_or("缺少 source.nes")?;
    let mut current = catalog::parse(source)?;
    let saved: catalog::Catalog =
        serde_json::from_slice(files.get("native.json").ok_or("缺少 native.json")?)?;
    if saved != current {
        return Err("一次提取记录与当前 source.nes 不一致；请重新导出，勿修改 native.json".into());
    }
    let mut changed = 0;
    for unit in &mut current.units {
        let key = format!("translations/{}.json", unit.key);
        let supplied =
            vn_text::read_template(files.get(&key).ok_or_else(|| format!("缺少 {key}"))?)?;
        let expected = unit.view();
        vn_text::validate_shape(&expected, &supplied)?;
        changed += unit.apply(&supplied)?;
    }
    let rom = rebuild::rebuild(source, &current, expand)?;
    let reparsed = catalog::parse(&rom)?;
    catalog::check_payloads(&current, &reparsed)?;
    let report = serde_json::json!({
        "source_sha256": catalog::digest(source), "output_sha256": catalog::digest(&rom),
        "source_records": current.record_count(), "editable_fragments": current.fragment_count(),
        "changed_fragments": changed, "prg_kib": if reparsed.expanded {256} else {128},
        "chr_kib": 128, "mapper": 1, "reextracted_payloads_equal": true,
        "chr_unchanged": source[source.len()-0x20000..] == rom[rom.len()-0x20000..],
        "byte_identical": source == &rom,
        "scope": "All located text records; control bytes preserved. No new Chinese font or CHR-RAM conversion."
    });
    Ok(ImportPlan { rom, report })
}

pub fn cycle_plan(rom: &[u8]) -> Result<ExportPlan> {
    let first = export_plan(rom)?;
    let ordinary = import_plan(&first.files, false)?;
    if ordinary.rom != rom {
        return Err("未修改译文的原容量回注未逐字节还原".into());
    }
    let enlarged = import_plan(&first.files, true)?;
    let second = export_plan(&enlarged.rom)?;
    let again = import_plan(&second.files, false)?;
    if again.rom != enlarged.rom {
        return Err("扩容 ROM 的第二次往返未逐字节还原".into());
    }
    let mut files = BTreeMap::new();
    for (name, bytes) in first.files {
        files.insert(format!("export/{name}"), bytes);
    }
    for (name, bytes) in second.files {
        files.insert(format!("reexport/{name}"), bytes);
    }
    files.insert("rebuilt/original_roundtrip.nes".into(), ordinary.rom);
    files.insert("rebuilt/expanded.nes".into(), enlarged.rom);
    files.insert("verification.json".into(), json_bytes(&serde_json::json!({
        "original_roundtrip_byte_identical": true, "expanded_second_roundtrip_byte_identical": true,
        "expanded": enlarged.report, "reexport": again.report
    }))?);
    Ok(ExportPlan {
        files,
        summary: "完整循环通过：原容量逐字节往返、扩容后重提取、扩容版本再次逐字节往返".into(),
    })
}

pub(crate) fn json_bytes(value: &impl serde::Serialize) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}
