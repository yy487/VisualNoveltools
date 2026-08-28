use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

const DIRECTORY_OFFSET: usize = 0xA0000;
const DIRECTORY_ENTRY_SIZE: usize = 24;
const LOGICAL_TRACK_SIZE: usize = 0x2000;
const D88_HEADER_SIZE: usize = 0x2B0;
const TRACK_TABLE_OFFSET: usize = 0x20;
const TRACK_TABLE_SLOTS: usize = 164;
const SECTOR_HEADER_SIZE: usize = 16;
const TOOL_NAME: &str = "foxy2_d88_splitter 0.3.0";

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceBatchManifest {
    pub format: String,
    pub tool: String,
    pub directory_offset: usize,
    pub disks: Vec<ResourceDiskSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceDiskSummary {
    pub source_name: String,
    pub source_directory: String,
    pub output_directory: String,
    pub logical_sha256: String,
    pub logical_size: usize,
    pub directory_entry_count: usize,
    pub extracted_file_count: usize,
    pub warning_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResourceDiskManifest {
    format: String,
    tool: String,
    source_name: String,
    original_d88_sha256: String,
    original_d88_size: usize,
    source_directory: String,
    logical_sha256: String,
    logical_size: usize,
    directory_offset: usize,
    directory_entry_size: usize,
    directory_entry_count: usize,
    extracted_file_count: usize,
    entries: Vec<ResourceEntry>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResourceEntry {
    index: usize,
    name: String,
    output_path: String,
    start_track: u8,
    start_offset: u16,
    logical_offset: usize,
    size: usize,
    span: usize,
    sha256: String,
    extension: String,
}

#[derive(Debug)]
struct ParsedEntry {
    index: usize,
    name: String,
    start_track: u8,
    start_offset: u16,
    logical_offset: usize,
    size: usize,
    span: usize,
}

#[derive(Debug)]
struct DiskInput {
    path: PathBuf,
    source_name: String,
    source_kind: DiskSourceKind,
}

#[derive(Debug, Clone, Copy)]
enum DiskSourceKind {
    D88,
    SplitWorkspace,
}

#[derive(Debug)]
struct D88Sector {
    data_offset: usize,
    data_size: usize,
}

#[derive(Debug)]
struct D88Layout {
    sectors: Vec<D88Sector>,
    logical_size: usize,
}

#[derive(Debug)]
struct PackedDisk {
    source_name: String,
    bytes: Vec<u8>,
    changed_count: usize,
}

#[derive(Debug, Serialize)]
struct PackManifest {
    format: &'static str,
    tool: &'static str,
    disks: Vec<PackSummary>,
}

#[derive(Debug, Serialize)]
struct PackSummary {
    source_name: String,
    output_name: String,
    changed_file_count: usize,
    output_sha256: String,
    output_size: usize,
}

pub fn is_resource_workspace(input: &Path) -> bool {
    discover_resource_manifests(input)
        .map(|manifests| !manifests.is_empty())
        .unwrap_or(false)
}

pub fn extract_inputs(input: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if !input.exists() {
        return Err(format!("input does not exist: {}", input.display()));
    }
    if output.exists() && !overwrite {
        return Err(format!(
            "output already exists; pass --overwrite: {}",
            output.display()
        ));
    }
    ensure_output_outside_input(input, output)?;
    let disks = discover_disks(input)?;
    if disks.is_empty() {
        return Err(format!(
            "no D88 image or compatible disk workspace found in {}",
            input.display()
        ));
    }

    let mut prepared = Vec::with_capacity(disks.len());
    for disk in disks {
        let (logical, original_d88) = load_disk_source(&disk)?;
        let (entries, warnings) = parse_directory(&logical, Path::new(&disk.source_name))?;
        prepared.push((disk, logical, original_d88, entries, warnings));
    }

    if output.exists() {
        remove_output(output)?;
    }
    fs::create_dir_all(output).map_err(|e| format!("create output: {e}"))?;
    let mut summaries = Vec::with_capacity(prepared.len());
    for (index, (disk, logical, original_d88, entries, mut warnings)) in
        prepared.into_iter().enumerate()
    {
        let stem = Path::new(&disk.source_name)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("disk");
        let dir_name = format!("{index:02}_{}", sanitize_name(stem));
        let disk_output = output.join(&dir_name);
        if disk_output.exists() && !overwrite {
            return Err(format!(
                "resource output already exists; pass --overwrite: {}",
                disk_output.display()
            ));
        }
        let files_output = disk_output.join("files");
        fs::create_dir_all(&files_output)
            .map_err(|e| format!("create {}: {e}", files_output.display()))?;
        let table_end = DIRECTORY_OFFSET
            .checked_add(entries.len() * DIRECTORY_ENTRY_SIZE)
            .ok_or("directory table range overflow")?;
        let table = logical
            .get(DIRECTORY_OFFSET..table_end)
            .ok_or("directory table exceeds logical image")?;
        fs::write(disk_output.join("directory_table.bin"), table)
            .map_err(|e| format!("write directory table: {e}"))?;
        fs::write(disk_output.join("original.d88"), &original_d88)
            .map_err(|e| format!("write preserved D88: {e}"))?;

        let mut output_entries = Vec::with_capacity(entries.len());
        let mut used_names = HashSet::with_capacity(entries.len());
        for entry in entries {
            let data = read_resource_payload(&logical, entry.logical_offset, entry.size)
                .map_err(|error| format!("{}: {error}", entry.name))?;
            let safe_name = sanitize_name(&entry.name);
            if safe_name.is_empty() || !used_names.insert(safe_name.clone()) {
                return Err(format!(
                    "entry {} has an invalid or duplicate output filename",
                    entry.index
                ));
            }
            let output_path = files_output.join(&safe_name);
            if output_path.exists() && !overwrite {
                return Err(format!(
                    "resource file already exists; pass --overwrite: {}",
                    output_path.display()
                ));
            }
            fs::write(&output_path, &data)
                .map_err(|e| format!("write {}: {e}", output_path.display()))?;
            if entry.size != entry.span {
                warnings.push(format!(
                    "entry {} {} has primary size {} and directory span {}",
                    entry.index, entry.name, entry.size, entry.span
                ));
            }
            output_entries.push(ResourceEntry {
                index: entry.index,
                name: entry.name.clone(),
                output_path: format!("files/{safe_name}"),
                start_track: entry.start_track,
                start_offset: entry.start_offset,
                logical_offset: entry.logical_offset,
                size: entry.size,
                span: entry.span,
                sha256: sha256_hex(&data),
                extension: entry
                    .name
                    .rsplit_once('.')
                    .map(|(_, extension)| extension.to_ascii_lowercase())
                    .unwrap_or_default(),
            });
        }
        // Keep manifests portable: the source path may point outside the managed workspace.
        let source_directory = format!("<INPUT>/{}", disk.source_name);
        let manifest = ResourceDiskManifest {
            format: "ELF-DOS fixed directory resources extracted from logical D88 sectors"
                .to_string(),
            tool: TOOL_NAME.to_string(),
            source_name: disk.source_name.clone(),
            original_d88_sha256: sha256_hex(&original_d88),
            original_d88_size: original_d88.len(),
            source_directory: source_directory.clone(),
            logical_sha256: sha256_hex(&logical),
            logical_size: logical.len(),
            directory_offset: DIRECTORY_OFFSET,
            directory_entry_size: DIRECTORY_ENTRY_SIZE,
            directory_entry_count: output_entries.len(),
            extracted_file_count: output_entries.len(),
            entries: output_entries,
            warnings,
        };
        write_json(&disk_output.join("manifest.json"), &manifest)?;
        summaries.push(ResourceDiskSummary {
            source_name: disk.source_name,
            source_directory,
            output_directory: dir_name,
            logical_sha256: manifest.logical_sha256.clone(),
            logical_size: manifest.logical_size,
            directory_entry_count: manifest.directory_entry_count,
            extracted_file_count: manifest.extracted_file_count,
            warning_count: manifest.warnings.len(),
        });
    }
    let disk_count = summaries.len();
    write_json(
        &output.join("manifest.json"),
        &ResourceBatchManifest {
            format: "ELF-DOS fixed directory resources extracted from logical D88 sectors"
                .to_string(),
            tool: TOOL_NAME.to_string(),
            directory_offset: DIRECTORY_OFFSET,
            disks: summaries,
        },
    )?;
    println!(
        "extracted {disk_count} disk resource set(s) into {}",
        output.display()
    );
    Ok(())
}

pub fn pack_inputs(input: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if !input.exists() {
        return Err(format!("input does not exist: {}", input.display()));
    }
    if output.exists() && !overwrite {
        return Err(format!(
            "output already exists; pass --overwrite: {}",
            output.display()
        ));
    }
    ensure_output_outside_input(input, output)?;
    let manifests = discover_resource_manifests(input)?;
    if manifests.is_empty() {
        return Err(format!(
            "no resource manifests found in {}",
            input.display()
        ));
    }

    let mut packed = Vec::with_capacity(manifests.len());
    let mut output_names = HashSet::with_capacity(manifests.len());
    for (directory, manifest) in manifests {
        validate_leaf_name(&manifest.source_name)?;
        if !output_names.insert(manifest.source_name.clone()) {
            return Err(format!(
                "duplicate output D88 name: {}",
                manifest.source_name
            ));
        }
        if !manifest.original_d88_sha256.is_empty() {
            let original = fs::read(directory.join("original.d88"))
                .map_err(|e| format!("read {}: {e}", directory.join("original.d88").display()))?;
            if manifest.original_d88_size != original.len() {
                return Err(format!(
                    "{} original.d88 size does not match manifest",
                    directory.display()
                ));
            }
            if sha256_hex(&original) != manifest.original_d88_sha256 {
                return Err(format!(
                    "{} original.d88 hash does not match manifest",
                    directory.display()
                ));
            }
        }
        packed.push(pack_one(&directory, &manifest)?);
    }

    for disk in &packed {
        let target = output.join(&disk.source_name);
        if target.exists() && !overwrite {
            return Err(format!(
                "output D88 already exists; pass --overwrite: {}",
                target.display()
            ));
        }
    }
    if output.exists() {
        remove_output(output)?;
    }
    fs::create_dir_all(output).map_err(|e| format!("create output: {e}"))?;
    let mut summaries = Vec::with_capacity(packed.len());
    for disk in packed {
        let target = output.join(&disk.source_name);
        fs::write(&target, &disk.bytes).map_err(|e| format!("write {}: {e}", target.display()))?;
        summaries.push(PackSummary {
            source_name: disk.source_name.clone(),
            output_name: disk.source_name,
            changed_file_count: disk.changed_count,
            output_sha256: sha256_hex(&disk.bytes),
            output_size: disk.bytes.len(),
        });
    }
    write_json(
        &output.join("pack_manifest.json"),
        &PackManifest {
            format: "Foxy 2 D88 resources repacked in place",
            tool: TOOL_NAME,
            disks: summaries,
        },
    )?;
    println!(
        "packed {} D88 image(s) into {}",
        output_names.len(),
        output.display()
    );
    Ok(())
}

fn pack_one(directory: &Path, manifest: &ResourceDiskManifest) -> Result<PackedDisk> {
    if manifest.directory_offset != DIRECTORY_OFFSET
        || manifest.directory_entry_size != DIRECTORY_ENTRY_SIZE
    {
        return Err(format!(
            "{} uses an unsupported resource directory layout",
            directory.display()
        ));
    }
    let original_path = directory.join("original.d88");
    let original =
        fs::read(&original_path).map_err(|e| format!("read {}: {e}", original_path.display()))?;
    let mut logical = d88_to_logical(&original)?;
    if manifest.logical_size != logical.len() || manifest.logical_sha256 != sha256_hex(&logical) {
        return Err(format!(
            "{} logical image does not match manifest",
            directory.display()
        ));
    }
    let (actual, _) = parse_directory(&logical, directory)?;
    if actual.len() != manifest.entries.len() || actual.len() != manifest.directory_entry_count {
        return Err(format!(
            "{} directory entry count does not match manifest",
            directory.display()
        ));
    }
    let mut changed_count = 0;
    let mut used_paths = HashSet::with_capacity(manifest.entries.len());
    let mut payloads = Vec::with_capacity(manifest.entries.len());
    for entry in &manifest.entries {
        let actual_entry = actual.get(entry.index).ok_or_else(|| {
            format!(
                "{} entry index {} is out of range",
                directory.display(),
                entry.index
            )
        })?;
        if actual_entry.index != entry.index
            || actual_entry.name != entry.name
            || actual_entry.start_track != entry.start_track
            || actual_entry.start_offset != entry.start_offset
            || actual_entry.logical_offset != entry.logical_offset
            || actual_entry.size != entry.size
            || actual_entry.span != entry.span
        {
            return Err(format!(
                "{} entry {} metadata does not match original D88",
                directory.display(),
                entry.index
            ));
        }
        let path = safe_join(directory, &entry.output_path)?;
        if !used_paths.insert(path.clone()) {
            return Err(format!(
                "{} contains duplicate resource path",
                directory.display()
            ));
        }
        let data = fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        if sha256_hex(&data) != entry.sha256 {
            changed_count += 1;
        }
        payloads.push(data);
    }

    let needs_reflow = manifest
        .entries
        .iter()
        .zip(&payloads)
        .any(|(entry, data)| data.len() > entry.span);
    if needs_reflow {
        let capacity = data_capacity(logical.len())?;
        let mut cursor = 0usize;
        for (entry, data) in manifest.entries.iter().zip(&payloads) {
            let original_rank = logical_to_data_rank(entry.logical_offset)?;
            let start_rank = cursor.max(original_rank);
            let allocation = entry.span.max(data.len());
            let end_rank = start_rank
                .checked_add(allocation)
                .ok_or("resource allocation overflow")?;
            if end_rank > capacity {
                return Err(format!(
                    "{} cannot fit translated resources: need {} bytes in a {}-byte data area",
                    directory.display(),
                    end_rank,
                    capacity
                ));
            }
            let logical_offset = data_rank_to_logical(start_rank)?;
            write_resource_payload(&mut logical, logical_offset, data)?;
            write_directory_record(
                &mut logical,
                entry.index,
                logical_offset,
                data.len(),
                allocation,
            )?;
            cursor = end_rank;
        }
    } else {
        for (entry, data) in manifest.entries.iter().zip(&payloads) {
            write_resource_payload(&mut logical, entry.logical_offset, data)?;
            write_directory_record(
                &mut logical,
                entry.index,
                entry.logical_offset,
                data.len(),
                entry.span,
            )?;
        }
    }
    let bytes = logical_to_d88(&original, &logical)?;
    Ok(PackedDisk {
        source_name: manifest.source_name.clone(),
        bytes,
        changed_count,
    })
}

fn discover_resource_manifests(input: &Path) -> Result<Vec<(PathBuf, ResourceDiskManifest)>> {
    if !input.is_dir() {
        return Ok(Vec::new());
    }
    let root_manifest = input.join("manifest.json");
    if root_manifest.is_file() {
        if let Ok(batch) = read_json::<ResourceBatchManifest>(&root_manifest) {
            let mut result = Vec::with_capacity(batch.disks.len());
            for summary in batch.disks {
                let directory = safe_join(input, &summary.output_directory)?;
                result.push((
                    directory.clone(),
                    read_json(&directory.join("manifest.json"))?,
                ));
            }
            if !result.is_empty() {
                return Ok(result);
            }
        }
        if let Ok(manifest) = read_json::<ResourceDiskManifest>(&root_manifest) {
            return Ok(vec![(input.to_path_buf(), manifest)]);
        }
    }
    let mut result = Vec::new();
    for entry in fs::read_dir(input).map_err(|e| format!("read resource directory: {e}"))? {
        let directory = entry
            .map_err(|e| format!("read resource entry: {e}"))?
            .path();
        if directory.is_dir() {
            let path = directory.join("manifest.json");
            if path.is_file() {
                if let Ok(manifest) = read_json::<ResourceDiskManifest>(&path) {
                    result.push((directory, manifest));
                }
            }
        }
    }
    result.sort_by_key(|(path, _)| path.file_name().map(|value| value.to_os_string()));
    Ok(result)
}

fn discover_disks(input: &Path) -> Result<Vec<DiskInput>> {
    if input.is_file() {
        if input
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("d88"))
        {
            return Ok(vec![DiskInput {
                path: input.to_path_buf(),
                source_name: input
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("input.d88")
                    .to_string(),
                source_kind: DiskSourceKind::D88,
            }]);
        }
        return Err(format!(
            "extract input is not a D88 image: {}",
            input.display()
        ));
    }
    if input.is_dir() && input.join("logical_1024.bin").is_file() {
        return Ok(vec![DiskInput {
            path: input.to_path_buf(),
            source_name: input
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("split-disk")
                .to_string(),
            source_kind: DiskSourceKind::SplitWorkspace,
        }]);
    }
    let mut disks = Vec::new();
    for entry in fs::read_dir(input).map_err(|e| format!("read input directory: {e}"))? {
        let path = entry.map_err(|e| format!("read input entry: {e}"))?.path();
        if path.is_file()
            && path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("d88"))
        {
            let source_name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("input.d88")
                .to_string();
            disks.push(DiskInput {
                path,
                source_name,
                source_kind: DiskSourceKind::D88,
            });
        } else if path.is_dir() && path.join("logical_1024.bin").is_file() {
            let source_name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("split-disk")
                .to_string();
            disks.push(DiskInput {
                path,
                source_name,
                source_kind: DiskSourceKind::SplitWorkspace,
            });
        }
    }
    disks.sort_by_key(|disk| disk.source_name.clone());
    Ok(disks)
}

fn load_disk_source(disk: &DiskInput) -> Result<(Vec<u8>, Vec<u8>)> {
    match disk.source_kind {
        DiskSourceKind::D88 => {
            let original =
                fs::read(&disk.path).map_err(|e| format!("read {}: {e}", disk.path.display()))?;
            Ok((d88_to_logical(&original)?, original))
        }
        DiskSourceKind::SplitWorkspace => {
            let logical_path = disk.path.join("logical_1024.bin");
            let logical = fs::read(&logical_path)
                .map_err(|e| format!("read {}: {e}", logical_path.display()))?;
            let original_path = disk.path.join("original.d88");
            let original = fs::read(&original_path)
                .map_err(|e| format!("read {}: {e}", original_path.display()))?;
            if d88_to_logical(&original)? != logical {
                return Err(format!(
                    "{} logical_1024.bin does not match original.d88",
                    disk.path.display()
                ));
            }
            Ok((logical, original))
        }
    }
}

fn parse_directory(logical: &[u8], disk: &Path) -> Result<(Vec<ParsedEntry>, Vec<String>)> {
    if logical.len() < DIRECTORY_OFFSET + DIRECTORY_ENTRY_SIZE {
        return Err(format!(
            "{} logical image is too short for the ELF-DOS directory",
            disk.display()
        ));
    }
    let mut entries: Vec<ParsedEntry> = Vec::new();
    let warnings = Vec::new();
    for index in 0..1024usize {
        let offset = DIRECTORY_OFFSET
            .checked_add(index * DIRECTORY_ENTRY_SIZE)
            .ok_or("directory offset overflow")?;
        let record = logical
            .get(offset..offset + DIRECTORY_ENTRY_SIZE)
            .ok_or_else(|| format!("directory entry {index} exceeds logical image"))?;
        if record.iter().all(|byte| *byte == 0) {
            break;
        }
        let name_end = record[..13]
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(13);
        let name_bytes = &record[..name_end];
        if name_bytes.is_empty() {
            return Err(format!("directory entry {index} has an empty filename"));
        }
        if name_bytes.iter().any(|byte| *byte < 0x20 || *byte >= 0x80) {
            return Err(format!("directory entry {index} has a non-ASCII filename"));
        }
        let name = String::from_utf8(name_bytes.to_vec())
            .map_err(|_| format!("directory entry {index} filename is not UTF-8"))?;
        let start_track = record[13];
        let start_offset = u16::from_le_bytes([record[14], record[15]]);
        if usize::from(start_offset) >= LOGICAL_TRACK_SIZE {
            return Err(format!("directory entry {index} {name} has track offset {start_offset:#x} outside a logical track"));
        }
        let logical_offset = usize::from(start_track)
            .checked_mul(LOGICAL_TRACK_SIZE)
            .and_then(|value| value.checked_add(usize::from(start_offset)))
            .ok_or_else(|| format!("directory entry {index} offset overflows"))?;
        let size = read_u32(record, 16)? as usize;
        let span = read_u32(record, 20)? as usize;
        let start_rank = logical_to_data_rank(logical_offset).map_err(|error| {
            format!("directory entry {index} {name} has invalid start: {error}")
        })?;
        let actual_end = start_rank
            .checked_add(size)
            .ok_or_else(|| format!("directory entry {index} actual size overflows"))?;
        let span_end = start_rank
            .checked_add(span)
            .ok_or_else(|| format!("directory entry {index} span overflows"))?;
        let capacity = data_capacity(logical.len())?;
        if actual_end > capacity {
            return Err(format!(
                "directory entry {index} {name} actual range exceeds logical image"
            ));
        }
        if span_end > capacity {
            return Err(format!(
                "directory entry {index} {name} span exceeds logical image"
            ));
        }
        if let Some(previous) = entries.last() {
            let previous_start = logical_to_data_rank(previous.logical_offset)?;
            let previous_actual_end = previous_start
                .checked_add(previous.size)
                .ok_or("previous resource size overflows")?;
            let previous_span_end = previous_start
                .checked_add(previous.span)
                .ok_or("previous resource span overflows")?;
            if start_rank < previous_actual_end || start_rank < previous_span_end {
                return Err(format!(
                    "directory entry {index} {name} overlaps previous resource span"
                ));
            }
        }
        entries.push(ParsedEntry {
            index,
            name,
            start_track,
            start_offset,
            logical_offset,
            size,
            span,
        });
    }
    if entries.is_empty() {
        return Err(format!(
            "{} contains no ELF-DOS directory entries",
            disk.display()
        ));
    }
    Ok((entries, warnings))
}

fn data_capacity(logical_size: usize) -> Result<usize> {
    logical_size
        .checked_sub(LOGICAL_TRACK_SIZE)
        .ok_or_else(|| "logical image is too short for its reserved directory track".to_string())
}

fn logical_to_data_rank(logical_offset: usize) -> Result<usize> {
    let directory_end = DIRECTORY_OFFSET + LOGICAL_TRACK_SIZE;
    if (DIRECTORY_OFFSET..directory_end).contains(&logical_offset) {
        return Err(format!(
            "logical offset {logical_offset:#x} points into the reserved directory track"
        ));
    }
    if logical_offset < DIRECTORY_OFFSET {
        Ok(logical_offset)
    } else {
        Ok(logical_offset - LOGICAL_TRACK_SIZE)
    }
}

fn data_rank_to_logical(rank: usize) -> Result<usize> {
    if rank < DIRECTORY_OFFSET {
        Ok(rank)
    } else {
        rank.checked_add(LOGICAL_TRACK_SIZE)
            .ok_or_else(|| "logical resource offset overflow".to_string())
    }
}

fn read_resource_payload(logical: &[u8], start: usize, size: usize) -> Result<Vec<u8>> {
    let start_rank = logical_to_data_rank(start)?;
    let end_rank = start_rank
        .checked_add(size)
        .ok_or("resource range overflow")?;
    if end_rank > data_capacity(logical.len())? {
        return Err("resource range exceeds logical data area".to_string());
    }
    let mut output = Vec::with_capacity(size);
    let mut rank = start_rank;
    while rank < end_rank {
        let logical_offset = data_rank_to_logical(rank)?;
        let track_remaining = LOGICAL_TRACK_SIZE - logical_offset % LOGICAL_TRACK_SIZE;
        let count = track_remaining.min(end_rank - rank);
        output.extend_from_slice(&logical[logical_offset..logical_offset + count]);
        rank += count;
    }
    Ok(output)
}

fn write_resource_payload(logical: &mut [u8], start: usize, data: &[u8]) -> Result<()> {
    let start_rank = logical_to_data_rank(start)?;
    let end_rank = start_rank
        .checked_add(data.len())
        .ok_or("resource range overflow")?;
    if end_rank > data_capacity(logical.len())? {
        return Err("resource range exceeds logical data area".to_string());
    }
    let mut rank = start_rank;
    let mut source = 0usize;
    while rank < end_rank {
        let logical_offset = data_rank_to_logical(rank)?;
        let track_remaining = LOGICAL_TRACK_SIZE - logical_offset % LOGICAL_TRACK_SIZE;
        let count = track_remaining.min(end_rank - rank);
        logical[logical_offset..logical_offset + count]
            .copy_from_slice(&data[source..source + count]);
        rank += count;
        source += count;
    }
    Ok(())
}

fn write_directory_record(
    logical: &mut [u8],
    index: usize,
    logical_offset: usize,
    size: usize,
    span: usize,
) -> Result<()> {
    let track = logical_offset / LOGICAL_TRACK_SIZE;
    let offset = logical_offset % LOGICAL_TRACK_SIZE;
    if track > u8::MAX as usize || offset > u16::MAX as usize {
        return Err("resource address cannot be represented by the ELF-DOS directory".to_string());
    }
    let record_offset = DIRECTORY_OFFSET
        .checked_add(
            index
                .checked_mul(DIRECTORY_ENTRY_SIZE)
                .ok_or("directory entry offset overflow")?,
        )
        .ok_or("directory entry offset overflow")?;
    *logical
        .get_mut(record_offset + 13)
        .ok_or("directory track field exceeds logical image")? = track as u8;
    logical
        .get_mut(record_offset + 14..record_offset + 16)
        .ok_or("directory offset field exceeds logical image")?
        .copy_from_slice(&(offset as u16).to_le_bytes());
    write_u32(logical, record_offset + 16, size as u32)?;
    write_u32(logical, record_offset + 20, span as u32)
}

fn d88_to_logical(bytes: &[u8]) -> Result<Vec<u8>> {
    let layout = parse_d88_layout(bytes)?;
    let mut logical = Vec::with_capacity(layout.logical_size);
    for sector in layout.sectors {
        logical
            .extend_from_slice(&bytes[sector.data_offset..sector.data_offset + sector.data_size]);
    }
    Ok(logical)
}

fn logical_to_d88(original: &[u8], logical: &[u8]) -> Result<Vec<u8>> {
    let layout = parse_d88_layout(original)?;
    if logical.len() != layout.logical_size {
        return Err(format!(
            "logical payload size {} does not match D88 layout {}",
            logical.len(),
            layout.logical_size
        ));
    }
    let mut output = original.to_vec();
    let mut cursor = 0;
    for sector in layout.sectors {
        let end = cursor + sector.data_size;
        output[sector.data_offset..sector.data_offset + sector.data_size]
            .copy_from_slice(&logical[cursor..end]);
        cursor = end;
    }
    Ok(output)
}

fn parse_d88_layout(bytes: &[u8]) -> Result<D88Layout> {
    if bytes.len() < D88_HEADER_SIZE {
        return Err("D88 image is shorter than the header".to_string());
    }
    let declared_size = read_u32(bytes, 0x1C)? as usize;
    if declared_size != bytes.len() {
        return Err(format!(
            "D88 declared size {declared_size:#x} does not match file size {:#x}",
            bytes.len()
        ));
    }
    let mut track_offsets = Vec::new();
    for slot in 0..TRACK_TABLE_SLOTS {
        let offset = read_u32(bytes, TRACK_TABLE_OFFSET + slot * 4)? as usize;
        if offset != 0 {
            track_offsets.push((slot, offset));
        }
    }
    if track_offsets.is_empty() {
        return Err("D88 image has no populated tracks".to_string());
    }
    for pair in track_offsets.windows(2) {
        if pair[0].1 < D88_HEADER_SIZE || pair[0].1 >= pair[1].1 || pair[1].1 >= bytes.len() {
            return Err("D88 track offsets are invalid or not increasing".to_string());
        }
    }
    if track_offsets[0].1 < D88_HEADER_SIZE
        || track_offsets
            .last()
            .is_some_and(|(_, offset)| *offset >= bytes.len())
    {
        return Err("D88 track offset is outside image".to_string());
    }
    let mut sectors = Vec::new();
    for (index, (slot, start)) in track_offsets.iter().copied().enumerate() {
        let end = track_offsets
            .get(index + 1)
            .map(|(_, offset)| *offset)
            .unwrap_or(bytes.len());
        if end <= start || end > bytes.len() {
            return Err(format!("D88 track {slot} range is invalid"));
        }
        if end - start < SECTOR_HEADER_SIZE {
            return Err(format!("D88 track {slot} is shorter than a sector header"));
        }
        let first = &bytes[start..start + SECTOR_HEADER_SIZE];
        let sector_count = first[4] as usize;
        if sector_count == 0 || sector_count > 32 {
            return Err(format!(
                "D88 track {slot} has invalid sector count {}",
                first[4]
            ));
        }
        let mut cursor = start;
        for ordinal in 0..sector_count {
            if cursor + SECTOR_HEADER_SIZE > end {
                return Err(format!("D88 track {slot} ends inside sector header"));
            }
            let header = &bytes[cursor..cursor + SECTOR_HEADER_SIZE];
            let data_size = 128usize
                .checked_shl(u32::from(header[3]))
                .ok_or_else(|| format!("D88 track {slot} has invalid size code {}", header[3]))?;
            if data_size != 1024 {
                return Err(format!("D88 track {slot} sector {ordinal} is {data_size}-byte, expected 1024-byte Foxy sector"));
            }
            let data_offset = cursor + SECTOR_HEADER_SIZE;
            let data_end = data_offset
                .checked_add(data_size)
                .ok_or("D88 sector size overflow")?;
            if data_end > end {
                return Err(format!(
                    "D88 track {slot} sector {ordinal} exceeds track boundary"
                ));
            }
            if header[4] as usize != sector_count {
                return Err(format!(
                    "D88 track {slot} sector {ordinal} count differs from track count"
                ));
            }
            if header[2] != (ordinal + 1) as u8 {
                return Err(format!("D88 track {slot} sector order is not 1-based"));
            }
            sectors.push(D88Sector {
                data_offset,
                data_size,
            });
            cursor = data_end;
        }
    }
    let logical_size = sectors.iter().map(|sector| sector.data_size).sum();
    Ok(D88Layout {
        sectors,
        logical_size,
    })
}

fn safe_join(base: &Path, relative: &str) -> Result<PathBuf> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe relative path in manifest: {relative}"));
    }
    Ok(base.join(path))
}

fn ensure_output_outside_input(input: &Path, output: &Path) -> Result<()> {
    let input =
        fs::canonicalize(input).map_err(|e| format!("resolve input {}: {e}", input.display()))?;
    let output_resolved = if output.exists() {
        fs::canonicalize(output).map_err(|e| format!("resolve output {}: {e}", output.display()))?
    } else if let (Some(parent), Some(name)) = (output.parent(), output.file_name()) {
        fs::canonicalize(parent)
            .map(|parent| parent.join(name))
            .unwrap_or_else(|_| output.to_path_buf())
    } else {
        output.to_path_buf()
    };
    if output_resolved.starts_with(&input) {
        return Err("output directory must not be the input or one of its descendants".to_string());
    }
    Ok(())
}

fn remove_output(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|e| format!("remove old output {}: {e}", path.display()))
    } else {
        fs::remove_file(path).map_err(|e| format!("remove old output {}: {e}", path.display()))
    }
}

fn validate_leaf_name(name: &str) -> Result<()> {
    if name.is_empty()
        || Path::new(name)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe output name: {name}"));
    }
    Ok(())
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("parse {}: {e}", path.display()))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| format!("u32 at record offset {offset} exceeds input"))?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) -> Result<()> {
    let target = bytes
        .get_mut(offset..offset + 4)
        .ok_or_else(|| format!("u32 at offset {offset:#x} exceeds logical image"))?;
    target.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn sanitize_name(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| format!("serialize JSON: {e}"))?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(|e| format!("write {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_track_pointer_and_primary_size() {
        let mut logical = vec![0u8; DIRECTORY_OFFSET + DIRECTORY_ENTRY_SIZE * 2];
        let record = &mut logical[DIRECTORY_OFFSET..DIRECTORY_OFFSET + DIRECTORY_ENTRY_SIZE];
        record[..8].copy_from_slice(b"TEST.MES");
        record[13] = 2;
        record[14..16].copy_from_slice(&0x0753u16.to_le_bytes());
        record[16..20].copy_from_slice(&123u32.to_le_bytes());
        record[20..24].copy_from_slice(&128u32.to_le_bytes());
        let (entries, warnings) = parse_directory(&logical, Path::new("fixture"))
            .expect("fixture directory should parse");
        assert!(warnings.is_empty());
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].logical_offset, 2 * LOGICAL_TRACK_SIZE + 0x0753);
        assert_eq!(entries[0].size, 123);
        assert_eq!(entries[0].span, 128);
    }

    #[test]
    fn d88_logical_round_trip_replaces_only_sector_data() {
        let track_offset = D88_HEADER_SIZE;
        let track_size = SECTOR_HEADER_SIZE + 1024;
        let mut bytes = vec![0u8; track_offset + track_size];
        bytes[TRACK_TABLE_OFFSET..TRACK_TABLE_OFFSET + 4]
            .copy_from_slice(&(track_offset as u32).to_le_bytes());
        let declared_size = bytes.len() as u32;
        bytes[0x1C..0x20].copy_from_slice(&declared_size.to_le_bytes());
        bytes[track_offset] = 0;
        bytes[track_offset + 1] = 0;
        bytes[track_offset + 2] = 1;
        bytes[track_offset + 3] = 3;
        bytes[track_offset + 4] = 1;
        bytes[track_offset + SECTOR_HEADER_SIZE..].fill(0x5A);
        let logical = d88_to_logical(&bytes).expect("logical payload");
        assert_eq!(logical.len(), 1024);
        let mut changed = logical.clone();
        changed[0] = 0xA5;
        let rebuilt = logical_to_d88(&bytes, &changed).expect("rebuilt D88");
        assert_eq!(rebuilt[track_offset + SECTOR_HEADER_SIZE], 0xA5);
        assert_eq!(
            &rebuilt[..track_offset + SECTOR_HEADER_SIZE],
            &bytes[..track_offset + SECTOR_HEADER_SIZE]
        );
    }

    #[test]
    fn rejects_parent_manifest_paths() {
        assert!(safe_join(Path::new("base"), "../x").is_err());
        assert!(safe_join(Path::new("base"), "files/x").is_ok());
    }

    #[test]
    fn resource_payload_skips_reserved_directory_track() {
        let mut logical = vec![0u8; DIRECTORY_OFFSET + LOGICAL_TRACK_SIZE * 2];
        logical[DIRECTORY_OFFSET - 4..DIRECTORY_OFFSET].copy_from_slice(&[1, 2, 3, 4]);
        let after = DIRECTORY_OFFSET + LOGICAL_TRACK_SIZE;
        logical[after..after + 4].copy_from_slice(&[5, 6, 7, 8]);
        logical[DIRECTORY_OFFSET..after].fill(0xCC);

        let payload = read_resource_payload(&logical, DIRECTORY_OFFSET - 4, 8)
            .expect("cross-track resource should read");
        assert_eq!(payload, [1, 2, 3, 4, 5, 6, 7, 8]);

        write_resource_payload(
            &mut logical,
            DIRECTORY_OFFSET - 4,
            &[8, 7, 6, 5, 4, 3, 2, 1],
        )
        .expect("cross-track resource should write");
        assert!(logical[DIRECTORY_OFFSET..after]
            .iter()
            .all(|byte| *byte == 0xCC));
    }
}
