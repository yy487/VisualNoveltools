use crate::controls::{normalize_text, unwrap_name};
use crate::format::{find_text_refs_at, parse_layout, TextRef};
use crate::json::{bytes_to_hex, JsonEntry};
use crate::Result;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub fn extract_entries_from_offsets(
    file_name: &str,
    data: &[u8],
    ref_offsets: &[usize],
) -> Result<Vec<JsonEntry>> {
    let layout = parse_layout(data, file_name)?;
    let wanted = ref_offsets.iter().copied().collect();
    let mut refs = find_text_refs_at(data, layout, file_name, &wanted)?;
    refs.sort_by_key(|text_ref| (text_ref.inst_offset, text_ref.ref_offset));
    let mut entries = Vec::with_capacity(refs.len());
    for text_ref in &refs {
        entries.push(make_entry(file_name, entries.len(), text_ref));
    }
    Ok(entries)
}

fn make_entry(file_name: &str, index: usize, message_ref: &TextRef) -> JsonEntry {
    let source_name = unwrap_name(&message_ref.text);
    let normalized = normalize_text(&message_ref.text);
    let has_format_controls = !normalized.format_controls.is_empty();
    let is_name = source_name.is_some();
    JsonEntry {
        file: file_name.to_string(),
        index,
        inst_offset: message_ref.inst_offset,
        ref_offset: message_ref.ref_offset,
        target: message_ref.target,
        opcode: format!("{:02X}", message_ref.opcode),
        entry_type: role_for(message_ref.opcode, is_name).to_string(),
        raw_hex: bytes_to_hex(&message_ref.raw),
        encoding: "cp932".to_string(),
        name_inst_offset: None,
        name_ref_offset: None,
        name_target: None,
        name_opcode: None,
        name_raw_hex: None,
        scr_name: source_name.clone(),
        name: source_name,
        scr_msg: if is_name {
            String::new()
        } else {
            normalized.clean.clone()
        },
        message: if is_name {
            String::new()
        } else {
            normalized.clean
        },
        scr_msg_parts: (!is_name && has_format_controls).then(|| normalized.parts.clone()),
        message_parts: (!is_name && has_format_controls).then_some(normalized.parts),
        format_controls: if is_name {
            Vec::new()
        } else {
            normalized.format_controls
        },
    }
}

fn role_for(opcode: u8, is_name: bool) -> &'static str {
    if is_name {
        return "name";
    }
    match opcode {
        0x00 | 0x01 => "message",
        0x09 => "choice_prompt",
        0x6E => "route_label",
        0xAA | 0xC9 | 0xD1 | 0xEC => "system",
        0xD7 => "choice",
        _ => "text",
    }
}

pub fn extract_path(
    input: &Path,
    references: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<(usize, usize)> {
    prepare_output_dir(output)?;
    let files = collect_bin_files(input, false)?;
    let mut entry_count = 0usize;
    for path in &files {
        let file_name = file_name(path)?;
        let data =
            fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
        let reference_path = references.join(format!(
            "{}.json",
            path.file_stem()
                .expect("BIN path has stem")
                .to_string_lossy()
        ));
        let reference_text = fs::read_to_string(&reference_path).map_err(|err| {
            format!(
                "failed to read reference map {}: {err}",
                reference_path.display()
            )
        })?;
        let reference_entries: Vec<JsonEntry> =
            serde_json::from_str(&reference_text).map_err(|err| {
                format!(
                    "failed to parse reference map {}: {err}",
                    reference_path.display()
                )
            })?;
        let offsets = reference_offsets(&reference_entries);
        let entries = extract_entries_from_offsets(&file_name, &data, &offsets)?;
        let output_path = output.join(format!(
            "{}.json",
            Path::new(&file_name)
                .file_stem()
                .expect("BIN name has a stem")
                .to_string_lossy()
        ));
        write_json(&output_path, &entries, overwrite)?;
        entry_count += entries.len();
    }
    Ok((files.len(), entry_count))
}

pub fn reference_offsets(entries: &[JsonEntry]) -> Vec<usize> {
    let mut offsets = Vec::with_capacity(entries.len() * 2);
    for entry in entries {
        if let Some(name_ref_offset) = entry.name_ref_offset {
            offsets.push(name_ref_offset);
        }
        offsets.push(entry.ref_offset);
    }
    offsets.sort_unstable();
    offsets.dedup();
    offsets
}

pub fn write_json(path: &Path, entries: &[JsonEntry], overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        return Err(format!(
            "{} already exists; pass --overwrite",
            path.display()
        ));
    }
    let text = serde_json::to_string_pretty(entries)
        .map_err(|err| format!("failed to serialize {}: {err}", path.display()))?;
    fs::write(path, format!("{text}\n"))
        .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

pub fn prepare_output_dir(path: &Path) -> Result<()> {
    if path.is_file() {
        return Err(format!(
            "{} is a file, expected a directory",
            path.display()
        ));
    }
    fs::create_dir_all(path).map_err(|err| format!("failed to create {}: {err}", path.display()))
}

pub fn collect_bin_files(input: &Path, include_0047: bool) -> Result<Vec<PathBuf>> {
    if input.is_file() {
        if !is_bin(input) {
            return Err(format!("{} is not a BIN file", input.display()));
        }
        if !include_0047 && is_0047(input) {
            return Ok(Vec::new());
        }
        return Ok(vec![input.to_path_buf()]);
    }
    let mut files = Vec::new();
    for entry in
        fs::read_dir(input).map_err(|err| format!("failed to read {}: {err}", input.display()))?
    {
        let path = entry
            .map_err(|err| format!("failed to read directory entry: {err}"))?
            .path();
        if path.is_file() && is_bin(&path) && (include_0047 || !is_0047(&path)) {
            files.push(path);
        }
    }
    files.sort_by_key(|path| path.file_name().map(|name| name.to_os_string()));
    Ok(files)
}

pub fn file_name(path: &Path) -> Result<String> {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| format!("invalid path: {}", path.display()))
}

pub fn is_0047(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case(OsStr::new("0047.bin")))
}

fn is_bin(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("bin"))
}
