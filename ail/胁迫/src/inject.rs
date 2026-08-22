use crate::extract::{
    collect_bin_files, extract_entries_from_offsets, file_name, is_0047, prepare_output_dir,
    reference_offsets,
};
use crate::format::{parse_layout, write_u16};
use crate::json::JsonEntry;
use crate::Result;
use encoding_rs::SHIFT_JIS;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn inject_path(
    source: &Path,
    json_dir: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<(usize, usize, usize)> {
    if !source.is_dir() || !json_dir.is_dir() {
        return Err("inject requires source and JSON directories".to_string());
    }
    prepare_output_dir(output)?;
    let files = collect_bin_files(source, true)?;
    let mut translated_files = 0usize;
    let mut unchanged_files = 0usize;
    let mut entry_count = 0usize;
    for path in files {
        let name = file_name(&path)?;
        let output_path = output.join(&name);
        refuse_existing(&output_path, overwrite)?;
        let source_data =
            fs::read(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
        if is_0047(&path) {
            fs::write(&output_path, &source_data)
                .map_err(|err| format!("failed to copy {}: {err}", output_path.display()))?;
            unchanged_files += 1;
            continue;
        }
        let json_path = json_dir.join(format!(
            "{}.json",
            path.file_stem()
                .expect("BIN path has stem")
                .to_string_lossy()
        ));
        let entries = read_entries(&json_path)?;
        let rebuilt = inject_file(&name, &source_data, &entries)?;
        if rebuilt == source_data {
            unchanged_files += 1;
        } else {
            translated_files += 1;
        }
        entry_count += entries.len();
        fs::write(&output_path, rebuilt)
            .map_err(|err| format!("failed to write {}: {err}", output_path.display()))?;
    }
    Ok((translated_files, unchanged_files, entry_count))
}

pub fn inject_file(file_name: &str, source: &[u8], entries: &[JsonEntry]) -> Result<Vec<u8>> {
    let offsets = reference_offsets(entries);
    let source_entries = extract_entries_from_offsets(file_name, source, &offsets)?;
    validate_entries(file_name, &source_entries, entries)?;
    if entries.iter().all(JsonEntry::is_unchanged) {
        return Ok(source.to_vec());
    }
    let layout = parse_layout(source, file_name)?;
    let mut patches: Vec<(usize, Vec<u8>)> = Vec::new();
    for entry in entries {
        if let Some(name) = entry.translated_name()? {
            crate::controls::validate_translated_text(name)
                .map_err(|err| format!("{file_name} entry {} name: {err}", entry.index))?;
            if name.contains('【') || name.contains('】') {
                return Err(format!(
                    "{file_name} entry {} name must not contain structural brackets",
                    entry.index
                ));
            }
            let wrapped = format!("【{name}】");
            if !entry.is_unchanged() {
                patches.push((
                    entry.ref_offset,
                    encode_cp932(file_name, entry.index, "name", &wrapped)?,
                ));
            }
        } else if !entry.is_unchanged() {
            let rendered = entry.rendered_message()?;
            patches.push((
                entry.ref_offset,
                encode_cp932(file_name, entry.index, "message", &rendered)?,
            ));
        }
    }
    patches.sort_by_key(|(offset, _)| *offset);

    let mut pool = source[layout.text_start..].to_vec();
    let mut offsets: HashMap<Vec<u8>, u16> = HashMap::new();
    seed_pool_offsets(&pool, &mut offsets)?;
    let mut resolved = Vec::with_capacity(patches.len());
    for (ref_offset, raw) in patches {
        let target = if let Some(&target) = offsets.get(&raw) {
            target
        } else {
            let target = u16::try_from(pool.len())
                .map_err(|_| format!("{file_name}: rebuilt text pool offset exceeds 0xFFFF"))?;
            pool.extend_from_slice(&raw);
            pool.extend_from_slice(&[0, 0]);
            offsets.insert(raw, target);
            target
        };
        resolved.push((ref_offset, target));
    }
    let text_len = u16::try_from(pool.len())
        .map_err(|_| format!("{file_name}: rebuilt text pool exceeds 65535 bytes"))?;
    let mut rebuilt = source[..layout.text_start].to_vec();
    for (ref_offset, target) in resolved {
        write_u16(&mut rebuilt, ref_offset, target)?;
    }
    write_u16(&mut rebuilt, 8, text_len)?;
    rebuilt.extend_from_slice(&pool);
    parse_layout(&rebuilt, file_name)?;
    Ok(rebuilt)
}

fn validate_entries(file_name: &str, source: &[JsonEntry], translated: &[JsonEntry]) -> Result<()> {
    if source.len() != translated.len() {
        return Err(format!(
            "{file_name}: JSON has {} entries, source extraction has {}",
            translated.len(),
            source.len()
        ));
    }
    for (expected, actual) in source.iter().zip(translated) {
        let same_metadata = expected.file == actual.file
            && expected.index == actual.index
            && expected.inst_offset == actual.inst_offset
            && expected.ref_offset == actual.ref_offset
            && expected.target == actual.target
            && expected.opcode.eq_ignore_ascii_case(&actual.opcode)
            && expected.entry_type == actual.entry_type
            && expected.raw_hex.eq_ignore_ascii_case(&actual.raw_hex)
            && expected.name_inst_offset == actual.name_inst_offset
            && expected.name_ref_offset == actual.name_ref_offset
            && expected.name_target == actual.name_target
            && expected.name_opcode == actual.name_opcode
            && expected.name_raw_hex == actual.name_raw_hex;
        if !same_metadata {
            return Err(format!(
                "{file_name} entry {} metadata does not match the source BIN",
                actual.index
            ));
        }
        if expected.scr_msg != actual.scr_msg
            || expected.scr_name != actual.scr_name
            || expected.scr_msg_parts != actual.scr_msg_parts
            || expected.format_controls != actual.format_controls
        {
            return Err(format!(
                "{file_name} entry {} source fields were modified; edit only name/message/message_parts",
                actual.index
            ));
        }
    }
    Ok(())
}

fn seed_pool_offsets(pool: &[u8], offsets: &mut HashMap<Vec<u8>, u16>) -> Result<()> {
    let mut cursor = 0usize;
    while cursor < pool.len() {
        let stop = (cursor..pool.len().saturating_sub(1))
            .find(|&offset| pool[offset] == 0 && pool[offset + 1] == 0)
            .ok_or_else(|| format!("unterminated source pool string at 0x{cursor:X}"))?;
        let target =
            u16::try_from(cursor).map_err(|_| "source pool offset exceeds 0xFFFF".to_string())?;
        offsets.entry(pool[cursor..stop].to_vec()).or_insert(target);
        cursor = stop + 2;
    }
    Ok(())
}

fn encode_cp932(file_name: &str, index: usize, field: &str, text: &str) -> Result<Vec<u8>> {
    let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        return Err(format!(
            "{file_name} entry {index} {field} contains a character unavailable in CP932"
        ));
    }
    Ok(encoded.into_owned())
}

fn read_entries(path: &Path) -> Result<Vec<JsonEntry>> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn refuse_existing(path: &Path, overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        return Err(format!(
            "{} already exists; pass --overwrite",
            path.display()
        ));
    }
    Ok(())
}

pub fn json_path_for_bin(json_dir: &Path, bin_path: &Path) -> PathBuf {
    json_dir.join(format!(
        "{}.json",
        bin_path.file_stem().unwrap_or_default().to_string_lossy()
    ))
}
