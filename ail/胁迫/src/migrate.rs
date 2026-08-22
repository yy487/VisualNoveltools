use crate::controls::{normalize_text, unwrap_name};
use crate::extract::{
    collect_bin_files, extract_entries_from_offsets, file_name, prepare_output_dir, write_json,
};
use crate::format::{decode_target, parse_layout, read_u16, write_u16, Layout};
use crate::json::JsonEntry;
use crate::Result;
use encoding_rs::SHIFT_JIS;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct PoolString {
    start: u16,
    raw_len: usize,
}

pub fn migrate_path(
    source_dir: &Path,
    legacy_dir: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<(usize, usize, usize)> {
    if !source_dir.is_dir() || !legacy_dir.is_dir() {
        return Err("migrate requires original and legacy BIN directories".to_string());
    }
    prepare_output_dir(output)?;
    let files = collect_bin_files(source_dir, false)?;
    let mut entry_count = 0usize;
    let mut ref_count = 0usize;
    for source_path in &files {
        let name = file_name(source_path)?;
        let legacy_path = legacy_dir.join(&name);
        let source = fs::read(source_path)
            .map_err(|err| format!("failed to read {}: {err}", source_path.display()))?;
        let legacy = fs::read(&legacy_path)
            .map_err(|err| format!("failed to read {}: {err}", legacy_path.display()))?;
        let offsets = discover_changed_refs(&name, &source, &legacy)?;
        let mut entries = extract_entries_from_offsets(&name, &source, &offsets)?;
        apply_legacy_translations(&name, &source, &legacy, &mut entries)?;
        let output_path = output.join(format!(
            "{}.json",
            source_path
                .file_stem()
                .expect("BIN path has stem")
                .to_string_lossy()
        ));
        write_json(&output_path, &entries, overwrite)?;
        entry_count += entries.len();
        ref_count += offsets.len();
    }
    Ok((files.len(), entry_count, ref_count))
}

pub fn discover_changed_refs(file_name: &str, source: &[u8], legacy: &[u8]) -> Result<Vec<usize>> {
    let layout = parse_layout(source, file_name)?;
    if legacy.len() < source.len() {
        return Err(format!(
            "{file_name}: legacy file is shorter than the original"
        ));
    }
    let original_pool = parse_pool(source, layout.text_start, source.len(), file_name)?;
    let legacy_pool = parse_pool(legacy, layout.text_start, legacy.len(), file_name)?;
    let original_starts: BTreeSet<u16> = original_pool.iter().map(|item| item.start).collect();
    let legacy_starts: BTreeSet<u16> = legacy_pool.iter().map(|item| item.start).collect();
    let changed: BTreeSet<usize> = (layout.bytecode_start..source.len())
        .filter(|&offset| source[offset] != legacy[offset])
        .collect();
    let mut refs = Vec::new();
    for offset in layout.bytecode_start..layout.text_start.saturating_sub(1) {
        let old_target = read_u16(source, offset)?;
        let new_target = read_u16(legacy, offset)?;
        if old_target != new_target
            && original_starts.contains(&old_target)
            && legacy_starts.contains(&new_target)
            && new_target as usize >= layout.text_len
        {
            refs.push(offset);
        }
    }
    refs.sort_unstable();
    refs.dedup();
    for pair in refs.windows(2) {
        if pair[0] + 1 >= pair[1] {
            return Err(format!(
                "{file_name}: overlapping recovered references at 0x{:X} and 0x{:X}",
                pair[0], pair[1]
            ));
        }
    }
    let covered: BTreeSet<usize> = refs
        .iter()
        .flat_map(|&offset| [offset, offset + 1])
        .filter(|offset| changed.contains(offset))
        .collect();
    let remaining: Vec<usize> = changed.difference(&covered).copied().collect();
    let groups = consecutive_groups(&remaining);
    for group in groups {
        let mut candidates = Vec::new();
        let begin = group[0].saturating_sub(1).max(layout.bytecode_start);
        let end = (group[group.len() - 1] + 1).min(layout.text_start.saturating_sub(2));
        for offset in begin..=end {
            let old_target = read_u16(source, offset)?;
            let new_target = read_u16(legacy, offset)?;
            if old_target == new_target
                || !legacy_starts.contains(&new_target)
                || (new_target as usize) < layout.text_len
                || containing_string(&original_pool, old_target).is_none()
            {
                continue;
            }
            let field_changed: BTreeSet<usize> = [offset, offset + 1]
                .into_iter()
                .filter(|&position| source[position] != legacy[position])
                .collect();
            if field_changed != group.iter().copied().collect() {
                continue;
            }
            if decode_target(source, layout, old_target).is_err() {
                continue;
            }
            let container = containing_string(&original_pool, old_target).expect("checked above");
            let prefix_start = layout.text_start + container.start as usize;
            let prefix_end = layout.text_start + old_target as usize;
            let (_, _, prefix_error) = SHIFT_JIS.decode(&source[prefix_start..prefix_end]);
            if !prefix_error {
                candidates.push(offset);
            }
        }
        if candidates.len() != 1 {
            return Err(format!(
                "{file_name}: changed byte group at 0x{:X} has {} substring candidates",
                group[0],
                candidates.len()
            ));
        }
        refs.push(candidates[0]);
    }
    refs.sort_unstable();
    refs.dedup();

    let mut replay = source.to_vec();
    for &offset in &refs {
        write_u16(&mut replay, offset, read_u16(legacy, offset)?)?;
    }
    if replay != legacy[..source.len()] {
        let mismatch = replay
            .iter()
            .zip(&legacy[..source.len()])
            .position(|(left, right)| left != right)
            .unwrap_or(0);
        return Err(format!(
            "{file_name}: recovered reference replay differs from legacy at 0x{mismatch:X}"
        ));
    }
    Ok(refs)
}

fn apply_legacy_translations(
    file_name: &str,
    source: &[u8],
    legacy: &[u8],
    entries: &mut [JsonEntry],
) -> Result<()> {
    let layout = parse_layout(source, file_name)?;
    let mut translated = HashMap::new();
    for entry in entries.iter() {
        translated.insert(
            entry.ref_offset,
            decode_legacy_ref(legacy, layout, entry.ref_offset)?,
        );
    }
    for entry in entries {
        if entry.scr_name.is_some() {
            let raw_name = translated
                .get(&entry.ref_offset)
                .ok_or_else(|| format!("{file_name}: missing translated name reference"))?;
            let cleaned = unwrap_name(raw_name).unwrap_or_else(|| normalize_text(raw_name).clean);
            if cleaned.contains('%') || cleaned.contains('％') {
                return Err(format!(
                    "{file_name} entry {} translated name contains a format token",
                    entry.index
                ));
            }
            entry.name = Some(cleaned);
            continue;
        }
        let translated_text = translated
            .get(&entry.ref_offset)
            .ok_or_else(|| format!("{file_name}: missing translated message reference"))?;
        let normalized = normalize_text(translated_text);
        if normalized.format_controls != entry.format_controls {
            return Err(format!(
                "{file_name} entry {} format controls differ: source {:?}, translation {:?}",
                entry.index, entry.format_controls, normalized.format_controls
            ));
        }
        entry.message = normalized.clean;
        if entry.format_controls.is_empty() {
            entry.message_parts = None;
        } else {
            entry.message_parts = Some(normalized.parts);
        }
    }
    Ok(())
}

fn decode_legacy_ref(legacy: &[u8], layout: Layout, ref_offset: usize) -> Result<String> {
    let target = read_u16(legacy, ref_offset)? as usize;
    let start = layout
        .text_start
        .checked_add(target)
        .ok_or_else(|| format!("legacy target 0x{target:X} overflows"))?;
    if start >= legacy.len() {
        return Err(format!("legacy target 0x{target:X} is out of range"));
    }
    let stop = (start..legacy.len().saturating_sub(1))
        .find(|&offset| legacy[offset] == 0 && legacy[offset + 1] == 0)
        .ok_or_else(|| format!("legacy target 0x{target:X} is unterminated"))?;
    let (text, _, had_errors) = SHIFT_JIS.decode(&legacy[start..stop]);
    if had_errors {
        return Err(format!("legacy target 0x{target:X} is not valid CP932"));
    }
    Ok(text.into_owned())
}

fn parse_pool(data: &[u8], base: usize, end: usize, file_name: &str) -> Result<Vec<PoolString>> {
    let mut result = Vec::new();
    let mut cursor = base;
    while cursor < end {
        let stop = (cursor..end.saturating_sub(1))
            .find(|&offset| data[offset] == 0 && data[offset + 1] == 0)
            .ok_or_else(|| format!("{file_name}: unterminated pool string at 0x{cursor:X}"))?;
        let start = u16::try_from(cursor - base)
            .map_err(|_| format!("{file_name}: pool offset exceeds 0xFFFF"))?;
        let (_, _, had_errors) = SHIFT_JIS.decode(&data[cursor..stop]);
        if had_errors {
            return Err(format!(
                "{file_name}: invalid CP932 pool string at 0x{cursor:X}"
            ));
        }
        result.push(PoolString {
            start,
            raw_len: stop - cursor,
        });
        cursor = stop + 2;
    }
    if cursor != end {
        return Err(format!("{file_name}: pool boundary mismatch"));
    }
    Ok(result)
}

fn containing_string(pool: &[PoolString], target: u16) -> Option<&PoolString> {
    pool.iter()
        .rev()
        .find(|item| item.start <= target && target as usize <= item.start as usize + item.raw_len)
}

fn consecutive_groups(values: &[usize]) -> Vec<Vec<usize>> {
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for &value in values {
        if groups
            .last()
            .and_then(|group| group.last())
            .is_some_and(|last| *last + 1 == value)
        {
            groups.last_mut().expect("group exists").push(value);
        } else {
            groups.push(vec![value]);
        }
    }
    groups
}
