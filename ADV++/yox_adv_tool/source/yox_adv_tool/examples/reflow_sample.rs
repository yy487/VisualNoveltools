use serde::Deserialize;
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};
use yox_adv_tool::{font::YoxFont, script::Script, Archive};

#[derive(Deserialize)]
struct Mapping {
    source: String,
    cp932: String,
}
#[derive(Deserialize)]
struct MappingFile {
    mappings: Vec<Mapping>,
}
#[derive(Deserialize)]
struct PrimaryRecord {
    fid: usize,
    string_slot: usize,
    projected: bool,
    hidden_suffix: String,
}
#[derive(Deserialize)]
struct PrimaryManifest {
    projection_contract: u32,
    records: Vec<PrimaryRecord>,
}

fn copy_tree(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 9 {
        return Err("usage: reflow_sample <script.dat> <source-snapshot.dat> <primary.json> <translations.json> <font.dat> <carrier_map.json> <font-slot> <new-output-dir>".into());
    }
    let source_path = PathBuf::from(&args[1]);
    let snapshot = fs::read(&args[2])?;
    let source_bytes = fs::read(&source_path)?;
    if source_bytes != snapshot {
        return Err("source DAT differs from its bound export snapshot".into());
    }
    let archive = Archive::parse("script.dat".into(), source_bytes)?;
    let mut scripts = (0..archive.entries.len())
        .map(|fid| {
            let bytes = archive.entry_bytes(fid).map_err(std::io::Error::other)?;
            Script::parse(bytes).map_err(std::io::Error::other)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let original_scripts = scripts.clone();
    let primary: PrimaryManifest = serde_json::from_slice(&fs::read(&args[3])?)?;
    if primary.projection_contract != 2 {
        return Err("unsupported projection contract".into());
    }
    let projected_records = primary
        .records
        .iter()
        .filter(|record| record.projected)
        .collect::<Vec<_>>();
    let filtered_records = primary
        .records
        .iter()
        .filter(|record| !record.projected)
        .collect::<Vec<_>>();
    let supplied = vn_text::read_template(&fs::read(&args[4])?)?;
    if supplied.len() != projected_records.len() {
        return Err("translation projection and primary mapping count differ".into());
    }
    let font_archive = Archive::parse("font.dat".into(), fs::read(&args[5])?)?;
    let font_slot: usize = args[7].to_string_lossy().parse()?;
    let font = YoxFont::parse(font_archive.entry_bytes(font_slot)?.to_vec())
        .map_err(std::io::Error::other)?;
    let map: MappingFile = serde_json::from_slice(&fs::read(&args[6])?)?;
    let carriers: BTreeMap<char, u16> = map
        .mappings
        .into_iter()
        .map(|row| {
            let cp: u16 = u16::from_str_radix(&row.cp932, 16)?;
            Ok((row.source.chars().next().ok_or("empty mapping source")?, cp))
        })
        .collect::<Result<_, Box<dyn std::error::Error>>>()?;
    let mut cursor = 0usize;
    for script in &mut scripts {
        let count = script
            .translation_projection()
            .map_err(std::io::Error::other)?
            .messages
            .len();
        let end = cursor.checked_add(count).ok_or("message index overflow")?;
        let entries = supplied
            .get(cursor..end)
            .ok_or("translation/source record count mismatch")?;
        script
            .apply_projected_messages_reflow(entries, 320, &font, &carriers)
            .map_err(std::io::Error::other)?;
        cursor = end;
    }
    if cursor != supplied.len() {
        return Err("unused translation records".into());
    }
    let replacements = scripts
        .iter()
        .enumerate()
        .map(|(fid, script)| {
            Ok((
                fid,
                script.rebuild(&carriers).map_err(std::io::Error::other)?,
            ))
        })
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    let rebuilt_script = archive.rebuild(&replacements)?;
    let rebuilt_archive = Archive::parse("script.dat".into(), rebuilt_script.clone())?;
    let rebuilt_scripts = (0..rebuilt_archive.entries.len())
        .map(|fid| {
            let bytes = rebuilt_archive
                .entry_bytes(fid)
                .map_err(std::io::Error::other)?;
            Script::parse(bytes).map_err(std::io::Error::other)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for record in &filtered_records {
        if original_scripts[record.fid].strings[record.string_slot].bytes
            != rebuilt_scripts[record.fid].strings[record.string_slot].bytes
        {
            return Err(format!(
                "filtered source slot changed: FID {} slot {}",
                record.fid, record.string_slot
            )
            .into());
        }
    }
    let mut suffixes_restored = 0usize;
    for (record, entry) in projected_records.iter().zip(&supplied) {
        if entry.message == "短句：你好，世界。"
            || entry.message.starts_with("中文验收长句：")
            || entry.message == "更短的译文。"
        {
            let rebuilt_text = &rebuilt_scripts[record.fid].strings[record.string_slot].text;
            if record.hidden_suffix.is_empty() || !rebuilt_text.ends_with(&record.hidden_suffix) {
                return Err(format!(
                    "terminal control suffix was not restored for FID {} slot {}",
                    record.fid, record.string_slot
                )
                .into());
            }
            suffixes_restored += 1;
        }
    }
    if suffixes_restored != 3 {
        return Err(
            format!("restored suffixes for {suffixes_restored} of 3 translated records").into(),
        );
    }
    let old_output = Path::new(&args[5])
        .parent()
        .ok_or("font DAT has no parent")?;
    let requested = PathBuf::from(&args[8]);
    let parent = fs::canonicalize(requested.parent().ok_or("output has no parent")?)?;
    let output = parent.join(requested.file_name().ok_or("output has no name")?);
    if output.exists() {
        return Err("output already exists".into());
    }
    let stage = parent.join(format!(".reflow-{}.tmp", std::process::id()));
    if stage.exists() {
        return Err("temporary stage already exists".into());
    }
    copy_tree(old_output, &stage)?;
    fs::write(stage.join("script.dat"), rebuilt_script)?;
    fs::write(
        stage.join("projection_check.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "projection_contract": primary.projection_contract,
            "projected_records": projected_records.len(),
            "filtered_records": filtered_records.len(),
            "filtered_slot_bytes_unchanged": filtered_records.len(),
            "translated_suffixes_restored": suffixes_restored,
        }))?,
    )?;
    let fallback_path = stage.join("font_fallbacks.json");
    let mut fallbacks: Vec<serde_json::Value> = serde_json::from_slice(&fs::read(&fallback_path)?)?;
    if !fallbacks.len().is_multiple_of(6) {
        return Err("fallback metadata cannot be split evenly across six fonts".into());
    }
    let per_font = fallbacks.len() / 6;
    for font_id in 0..6 {
        for record in &mut fallbacks[font_id * per_font..(font_id + 1) * per_font] {
            record["font_id"] = serde_json::json!(font_id);
            record["font_size"] = serde_json::json!(
                [[15, 16], [20, 22], [24, 26], [27, 29], [53, 57], [40, 43]][font_id]
            );
        }
    }
    if per_font > 0 {
        for font_id in 1..6 {
            for index in 0..per_font {
                if fallbacks[index]["cp932"] != fallbacks[font_id * per_font + index]["cp932"] {
                    return Err(
                        "fallback groups differ by font; refusing to infer missing font IDs".into(),
                    );
                }
            }
        }
    }
    fs::write(fallback_path, serde_json::to_vec_pretty(&fallbacks)?)?;
    fs::rename(stage, output)?;
    Ok(())
}
