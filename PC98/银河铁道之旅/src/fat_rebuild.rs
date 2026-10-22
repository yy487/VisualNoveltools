//! In-memory FAT12 file replacement for Galaxy Railway Disk 1.
//!
//! The caller supplies paths relative to the FAT volume (for example
//! `HAK1_MSG.MSG` or `GRAPH/END_PRS.CHN`). The optional `disk-000-whole/`
//! prefix used by fivec-new exports is accepted as well. This module never
//! writes the source image; it returns a separately rebuilt D88 byte vector.

use crate::{sha256, Result};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use vn_d88::{Decoder, StandardCodec};
use vn_sector_map::LinearView;

const EXPECTED_D88_SIZE: usize = 1_281_968;
const EXPECTED_LOGICAL_SIZE: usize = 1_261_568;
const EXPECTED_TRACKS: usize = 154;
const EXPECTED_SECTORS: usize = 1_232;
const EXPECTED_SECTORS_PER_TRACK: usize = 8;
const EXPECTED_BYTES_PER_SECTOR: usize = 1_024;

#[derive(Debug, Clone)]
pub struct Disk1RebuildReport {
    /// Rebuilt image. The input slice is never modified.
    pub bytes: Vec<u8>,
    pub source_sha256: String,
    pub output_sha256: String,
    pub files: usize,
    pub changed_files: usize,
    pub changes: Vec<FileChange>,
    pub allocated_clusters: usize,
    pub released_clusters: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: String,
    pub old_size: usize,
    pub new_size: usize,
    pub old_first_cluster: u16,
    pub new_first_cluster: u16,
}

#[derive(Debug, Clone)]
struct Layout {
    bytes_per_sector: usize,
    fat_copies: usize,
    total_sectors: usize,
    first_data_sector: usize,
    fat_offset: usize,
    fat_bytes: usize,
    root_offset: usize,
    root_bytes: usize,
    cluster_bytes: usize,
    max_cluster: u16,
    media_descriptor: u8,
}

#[derive(Debug, Clone)]
struct FilePlan {
    path: String,
    entry_offset: usize,
    old_chain: Vec<u16>,
    desired_chain: Vec<u16>,
    original: Vec<u8>,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
struct DirectoryTask {
    path: String,
    first_cluster: u16,
}

type InspectedSource = (LinearView, Layout, Vec<u8>, Vec<FilePlan>, usize);

/// Rebuild Disk 1 by replacing existing FAT12 files.
///
/// Replacement keys are case-insensitive DOS 8.3 paths relative to the FAT12
/// root, using `/` or `\\` between directory components. Keys may include
/// fivec-new's `disk-000-whole/` prefix. New files and directory entries are
/// not created. All directory/file chains and both FAT copies are validated
/// before a result image is assembled.
pub fn rebuild_disk1_d88(
    source_d88: &[u8],
    replacements: &BTreeMap<String, Vec<u8>>,
) -> Result<Disk1RebuildReport> {
    let source_sha256 = sha256(source_d88);
    let (view, layout, fat, mut files, file_count) = inspect_source(source_d88)?;
    let replacements = validate_replacement_paths(replacements, &files)?;

    for plan in &mut files {
        if let Some(data) = replacements.get(&plan.path) {
            if data.len() > u32::MAX as usize {
                return Err(format!(
                    "{} exceeds the FAT12 directory file-size field",
                    plan.path
                ));
            }
            plan.data.clone_from(data);
        }
    }
    let changed_paths: BTreeSet<String> = files
        .iter()
        .filter(|plan| plan.data != plan.original)
        .map(|plan| plan.path.clone())
        .collect();

    if changed_paths.is_empty() {
        return Ok(Disk1RebuildReport {
            bytes: source_d88.to_vec(),
            source_sha256: source_sha256.clone(),
            output_sha256: source_sha256,
            files: file_count,
            changed_files: 0,
            changes: Vec::new(),
            allocated_clusters: 0,
            released_clusters: 0,
        });
    }

    let mut available = BTreeSet::new();
    let mut reserved = HashSet::new();
    for plan in &files {
        if !changed_paths.contains(&plan.path) {
            reserved.extend(plan.old_chain.iter().copied());
        }
    }
    for cluster in 2..=layout.max_cluster {
        if !reserved.contains(&cluster) && fat12_next(&fat, cluster)? == 0 {
            available.insert(cluster);
        }
    }

    let mut released_clusters = 0usize;
    for plan in &mut files {
        if !changed_paths.contains(&plan.path) {
            plan.desired_chain.clone_from(&plan.old_chain);
            continue;
        }
        let needed = cluster_count(plan.data.len(), layout.cluster_bytes);
        let retained = needed.min(plan.old_chain.len());
        plan.desired_chain
            .extend_from_slice(&plan.old_chain[..retained]);
        for &cluster in &plan.old_chain[retained..] {
            if !available.insert(cluster) {
                return Err(format!(
                    "{} releases cluster {cluster} which is already available",
                    plan.path
                ));
            }
            released_clusters = released_clusters
                .checked_add(1)
                .ok_or_else(|| "released cluster count overflow".to_string())?;
        }
    }

    let additional = files
        .iter()
        .filter(|plan| changed_paths.contains(&plan.path))
        .try_fold(0usize, |total, plan| {
            total
                .checked_add(
                    cluster_count(plan.data.len(), layout.cluster_bytes)
                        .saturating_sub(plan.desired_chain.len()),
                )
                .ok_or_else(|| "additional cluster count overflow".to_string())
        })?;
    if additional > available.len() {
        return Err(format!(
            "insufficient FAT12 space: need {additional} clusters, have {}",
            available.len()
        ));
    }

    let mut allocated_clusters = 0usize;
    for plan in &mut files {
        if !changed_paths.contains(&plan.path) {
            continue;
        }
        let needed = cluster_count(plan.data.len(), layout.cluster_bytes);
        while plan.desired_chain.len() < needed {
            let cluster = available
                .pop_first()
                .ok_or_else(|| "FAT12 free-cluster accounting underflow".to_string())?;
            plan.desired_chain.push(cluster);
            allocated_clusters = allocated_clusters
                .checked_add(1)
                .ok_or_else(|| "allocated cluster count overflow".to_string())?;
        }
    }

    let mut rebuilt = source_d88.to_vec();
    let mut rebuilt_fat = fat;
    rewrite_changed_chains(&mut rebuilt_fat, &files, &changed_paths)?;
    for copy in 0..layout.fat_copies {
        let offset = layout
            .fat_offset
            .checked_add(
                copy.checked_mul(layout.fat_bytes)
                    .ok_or_else(|| "FAT copy offset overflow".to_string())?,
            )
            .ok_or_else(|| "FAT offset overflow".to_string())?;
        patch_logical(&view, &mut rebuilt, offset, &rebuilt_fat)?;
    }

    let mut changes = Vec::new();
    for plan in &files {
        if !changed_paths.contains(&plan.path) {
            continue;
        }
        for (index, &cluster) in plan.desired_chain.iter().enumerate() {
            let logical = cluster_offset(&layout, cluster)?;
            let start = index
                .checked_mul(layout.cluster_bytes)
                .ok_or_else(|| "file data offset overflow".to_string())?;
            let end = start
                .checked_add(layout.cluster_bytes)
                .ok_or_else(|| "file data range overflow".to_string())?
                .min(plan.data.len());
            if start < end {
                patch_logical(&view, &mut rebuilt, logical, &plan.data[start..end])?;
            }
        }
        let start_cluster = plan.desired_chain.first().copied().unwrap_or(0);
        patch_logical(
            &view,
            &mut rebuilt,
            plan.entry_offset + 26,
            &start_cluster.to_le_bytes(),
        )?;
        let size = u32::try_from(plan.data.len())
            .map_err(|_| format!("{} exceeds FAT12 directory file-size field", plan.path))?;
        patch_logical(
            &view,
            &mut rebuilt,
            plan.entry_offset + 28,
            &size.to_le_bytes(),
        )?;
        changes.push(FileChange {
            path: plan.path.clone(),
            old_size: plan.original.len(),
            new_size: plan.data.len(),
            old_first_cluster: plan.old_chain.first().copied().unwrap_or(0),
            new_first_cluster: start_cluster,
        });
    }

    validate_rebuilt(&rebuilt, &files, &changed_paths)?;
    let output_sha256 = sha256(&rebuilt);
    Ok(Disk1RebuildReport {
        bytes: rebuilt,
        source_sha256,
        output_sha256,
        files: file_count,
        changed_files: changes.len(),
        changes,
        allocated_clusters,
        released_clusters,
    })
}

fn inspect_source(source: &[u8]) -> Result<InspectedSource> {
    if source.len() != EXPECTED_D88_SIZE {
        return Err(format!(
            "Disk 1 D88 size mismatch: expected 0x{EXPECTED_D88_SIZE:X}, got 0x{:X}",
            source.len()
        ));
    }
    let image = StandardCodec
        .decode(source)
        .map_err(|error| format!("D88 decode failed: {error}"))?;
    if image.disks.len() != 1 || image.trailing_range.is_some() || !image.diagnostics.is_empty() {
        return Err("Disk 1 must be one complete, diagnostic-free D88 image".into());
    }
    let disk = &image.disks[0];
    if disk.tracks.len() != EXPECTED_TRACKS
        || disk
            .tracks
            .iter()
            .map(|track| track.sectors.len())
            .sum::<usize>()
            != EXPECTED_SECTORS
    {
        return Err("Disk 1 geometry is not 77 cylinders x 2 heads x 8 sectors".into());
    }
    for (slot, track) in disk.tracks.iter().enumerate() {
        if track.sectors.len() != EXPECTED_SECTORS_PER_TRACK {
            return Err(format!("D88 track slot {slot} does not contain 8 sectors"));
        }
        for (ordinal, sector) in track.sectors.iter().enumerate() {
            let id = sector.address.id;
            if sector.data_range.len() != EXPECTED_BYTES_PER_SECTOR
                || id.size_code != 3
                || id.record as usize != ordinal + 1
                || id.cylinder as usize != slot / 2
                || id.head as usize != slot % 2
                || sector.fdc_status != 0
                || sector.deleted_data != 0
            {
                return Err(format!(
                    "D88 track slot {slot} has unsupported CHRN or sector flags"
                ));
            }
        }
    }
    let view = image
        .physical_view(0)
        .map_err(|error| format!("D88 physical view failed: {error}"))?;
    if view.len() != EXPECTED_LOGICAL_SIZE {
        return Err(format!(
            "Disk 1 logical size mismatch: expected 0x{EXPECTED_LOGICAL_SIZE:X}, got 0x{:X}",
            view.len()
        ));
    }

    let boot = read_logical(source, &view, 0, EXPECTED_BYTES_PER_SECTOR)?;
    let layout = parse_layout(&boot, view.len())?;
    let first_fat = read_logical(source, &view, layout.fat_offset, layout.fat_bytes)?;
    for copy in 1..layout.fat_copies {
        let offset = layout
            .fat_offset
            .checked_add(
                copy.checked_mul(layout.fat_bytes)
                    .ok_or_else(|| "FAT copy offset overflow".to_string())?,
            )
            .ok_or_else(|| "FAT offset overflow".to_string())?;
        let bytes = read_logical(source, &view, offset, layout.fat_bytes)?;
        if bytes != first_fat {
            return Err(format!("FAT copy {copy} differs from the primary FAT"));
        }
    }
    validate_fat(&first_fat, &layout)?;
    let (files, file_count) = scan_files(source, &view, &layout, &first_fat)?;
    Ok((view, layout, first_fat, files, file_count))
}

fn parse_layout(boot: &[u8], logical_size: usize) -> Result<Layout> {
    let bytes_per_sector = usize::from(read_u16(boot, 11)?);
    let sectors_per_cluster = usize::from(*boot.get(13).ok_or("BPB is truncated")?);
    let reserved_sectors = usize::from(read_u16(boot, 14)?);
    let fat_copies = usize::from(*boot.get(16).ok_or("BPB is truncated")?);
    let root_entries = usize::from(read_u16(boot, 17)?);
    let total16 = usize::from(read_u16(boot, 19)?);
    let media_descriptor = *boot.get(21).ok_or("BPB is truncated")?;
    let sectors_per_fat = usize::from(read_u16(boot, 22)?);
    let total32 = usize::try_from(u32::from_le_bytes(
        boot.get(32..36)
            .ok_or("BPB is truncated")?
            .try_into()
            .expect("four bytes"),
    ))
    .map_err(|_| "BPB total sector count does not fit usize".to_string())?;
    let total_sectors = if total16 != 0 { total16 } else { total32 };

    if bytes_per_sector != EXPECTED_BYTES_PER_SECTOR
        || sectors_per_cluster != 1
        || reserved_sectors != 1
        || fat_copies != 2
        || sectors_per_fat != 2
        || root_entries != 192
        || total_sectors != EXPECTED_SECTORS
    {
        return Err(format!(
            "Disk 1 BPB does not match the verified FAT12 layout: bps={bytes_per_sector}, spc={sectors_per_cluster}, reserved={reserved_sectors}, fats={fat_copies}, root_entries={root_entries}, sectors_per_fat={sectors_per_fat}, total16={total16}, total32={total32}"
        ));
    }
    let root_bytes = root_entries
        .checked_mul(32)
        .ok_or_else(|| "root directory size overflow".to_string())?;
    let root_sectors = root_bytes.div_ceil(bytes_per_sector);
    let first_data_sector = reserved_sectors
        .checked_add(
            fat_copies
                .checked_mul(sectors_per_fat)
                .ok_or_else(|| "FAT region size overflow".to_string())?,
        )
        .and_then(|value| value.checked_add(root_sectors))
        .ok_or_else(|| "first data sector overflow".to_string())?;
    if first_data_sector >= total_sectors {
        return Err("FAT12 data area is empty".into());
    }
    let data_clusters = (total_sectors - first_data_sector) / sectors_per_cluster;
    if data_clusters == 0 || data_clusters >= 4085 {
        return Err(format!("FAT cluster count {data_clusters} is not FAT12"));
    }
    let max_cluster = u16::try_from(data_clusters + 1)
        .map_err(|_| "FAT12 maximum cluster exceeds u16".to_string())?;
    let fat_bytes = sectors_per_fat
        .checked_mul(bytes_per_sector)
        .ok_or_else(|| "FAT byte size overflow".to_string())?;
    let volume_bytes = total_sectors
        .checked_mul(bytes_per_sector)
        .ok_or_else(|| "volume byte size overflow".to_string())?;
    if volume_bytes != logical_size || volume_bytes != EXPECTED_LOGICAL_SIZE {
        return Err(format!(
            "BPB volume length 0x{volume_bytes:X} does not cover the logical disk"
        ));
    }
    let fat_offset = reserved_sectors * bytes_per_sector;
    let root_offset = (reserved_sectors + fat_copies * sectors_per_fat) * bytes_per_sector;
    let cluster_bytes = bytes_per_sector
        .checked_mul(sectors_per_cluster)
        .ok_or_else(|| "cluster byte size overflow".to_string())?;
    let fat_capacity = fat_bytes
        .checked_mul(2)
        .ok_or_else(|| "FAT capacity overflow".to_string())?
        / 3;
    if usize::from(max_cluster) >= fat_capacity {
        return Err("FAT12 table is too short for the BPB data area".into());
    }
    Ok(Layout {
        bytes_per_sector,
        fat_copies,
        total_sectors,
        first_data_sector,
        fat_offset,
        fat_bytes,
        root_offset,
        root_bytes,
        cluster_bytes,
        max_cluster,
        media_descriptor,
    })
}

fn validate_fat(fat: &[u8], layout: &Layout) -> Result<()> {
    if fat.len() != layout.fat_bytes {
        return Err("FAT byte size differs from BPB".into());
    }
    if fat.first().copied() != Some(layout.media_descriptor)
        || fat12_next(fat, 0)? != (0x0f00 | u16::from(layout.media_descriptor))
        || fat12_next(fat, 1)? < 0x0ff8
    {
        return Err("FAT12 reserved entries are invalid".into());
    }
    fat12_next(fat, layout.max_cluster)?;
    Ok(())
}

fn scan_files(
    source: &[u8],
    view: &LinearView,
    layout: &Layout,
    fat: &[u8],
) -> Result<(Vec<FilePlan>, usize)> {
    let root = read_logical(source, view, layout.root_offset, layout.root_bytes)?;
    let mut owners = HashMap::<u16, String>::new();
    let mut files = Vec::new();
    let mut seen_paths = HashSet::new();
    let mut pending = VecDeque::new();
    scan_directory_entries(
        source,
        view,
        layout,
        fat,
        &root,
        layout.root_offset,
        "".into(),
        &mut owners,
        &mut files,
        &mut seen_paths,
        &mut pending,
    )?;

    let mut seen_directories = HashSet::new();
    while let Some(task) = pending.pop_front() {
        if !seen_directories.insert(task.first_cluster) {
            return Err(format!(
                "directory {} loops or is referenced more than once",
                task.path
            ));
        }
        let chain = read_chain(
            fat,
            layout.max_cluster,
            task.first_cluster,
            &task.path,
            &mut owners,
        )?;
        if chain.is_empty() {
            return Err(format!(
                "directory {} has an empty cluster chain",
                task.path
            ));
        }
        let mut ended = false;
        for &cluster in &chain {
            if ended {
                break;
            }
            let offset = cluster_offset(layout, cluster)?;
            let bytes = read_logical(source, view, offset, layout.cluster_bytes)?;
            ended = scan_directory_entries(
                source,
                view,
                layout,
                fat,
                &bytes,
                offset,
                task.path.clone(),
                &mut owners,
                &mut files,
                &mut seen_paths,
                &mut pending,
            )?;
        }
    }

    for cluster in 2..=layout.max_cluster {
        let next = fat12_next(fat, cluster)?;
        if next != 0 && !(0x0ff0..=0x0ff7).contains(&next) && !owners.contains_key(&cluster) {
            return Err(format!(
                "allocated orphan FAT12 cluster {cluster} is not owned by a file or directory"
            ));
        }
    }
    let count = files.len();
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok((files, count))
}

#[allow(clippy::too_many_arguments)]
fn scan_directory_entries(
    source: &[u8],
    view: &LinearView,
    layout: &Layout,
    fat: &[u8],
    bytes: &[u8],
    logical_base: usize,
    parent_path: String,
    owners: &mut HashMap<u16, String>,
    files: &mut Vec<FilePlan>,
    seen_paths: &mut HashSet<String>,
    pending: &mut VecDeque<DirectoryTask>,
) -> Result<bool> {
    if !bytes.len().is_multiple_of(32) {
        return Err(format!(
            "directory {parent_path:?} has a byte length not divisible by 32"
        ));
    }
    for (index, entry) in bytes.chunks_exact(32).enumerate() {
        match entry[0] {
            0x00 => return Ok(true),
            0xe5 => continue,
            _ => {}
        }
        let attributes = entry[11];
        if attributes == 0x0f {
            // Long-file-name records are metadata for the short entry that
            // follows. Paths are intentionally addressed by verified 8.3 name.
            continue;
        }
        if attributes & 0x08 != 0 {
            // Volume labels do not own a file cluster chain.
            continue;
        }
        let name = short_name(entry)?;
        if name == "." || name == ".." {
            continue;
        }
        if name.is_empty() {
            return Err(format!(
                "directory {parent_path:?} has an empty live entry at index {index}"
            ));
        }
        let path = if parent_path.is_empty() {
            name
        } else {
            format!("{parent_path}/{name}")
        };
        let key = normalize_path(&path)?;
        if !seen_paths.insert(key) {
            return Err(format!(
                "duplicate DOS path in FAT12 directory tree: {path}"
            ));
        }
        let entry_offset = logical_base
            .checked_add(
                index
                    .checked_mul(32)
                    .ok_or_else(|| "directory entry offset overflow".to_string())?,
            )
            .ok_or_else(|| "directory entry offset overflow".to_string())?;
        let first_cluster = read_u16(entry, 26)?;
        let size = usize::try_from(u32::from_le_bytes(
            entry[28..32].try_into().expect("four bytes"),
        ))
        .map_err(|_| format!("{path} file size does not fit usize"))?;
        if attributes & 0x10 != 0 {
            if size != 0 {
                return Err(format!("directory {path} has a nonzero file size"));
            }
            if first_cluster < 2 || first_cluster > layout.max_cluster {
                return Err(format!("directory {path} has invalid first cluster"));
            }
            pending.push_back(DirectoryTask {
                path,
                first_cluster,
            });
            continue;
        }

        let path_key = normalize_path(&path)?;
        let old_chain = read_chain(fat, layout.max_cluster, first_cluster, &path, owners)?;
        let expected = cluster_count(size, layout.cluster_bytes);
        if old_chain.len() != expected {
            return Err(format!(
                "{path} size requires {expected} clusters but its FAT12 chain has {}",
                old_chain.len()
            ));
        }
        let original = read_file_data(source, view, layout, &old_chain, size, &path)?;
        files.push(FilePlan {
            path: path_key,
            entry_offset,
            old_chain,
            desired_chain: Vec::new(),
            original: original.clone(),
            data: original,
        });
    }
    Ok(false)
}

fn validate_replacement_paths(
    replacements: &BTreeMap<String, Vec<u8>>,
    files: &[FilePlan],
) -> Result<BTreeMap<String, Vec<u8>>> {
    let known: HashSet<&str> = files.iter().map(|plan| plan.path.as_str()).collect();
    let mut supplied = HashSet::new();
    let mut normalized = BTreeMap::new();
    for path in replacements.keys() {
        let key = normalize_path(path)?;
        if !supplied.insert(key.clone()) {
            return Err(format!(
                "duplicate replacement path after normalization: {path}"
            ));
        }
        if !known.contains(key.as_str()) {
            return Err(format!(
                "replacement does not name an existing 8.3 file: {path}"
            ));
        }
        normalized.insert(key, replacements[path].clone());
    }
    Ok(normalized)
}

fn normalize_path(value: &str) -> Result<String> {
    if value.is_empty() {
        return Err("replacement path is empty".into());
    }
    let slashed = value.replace('\\', "/");
    let mut parts = slashed.split('/').collect::<Vec<_>>();
    if parts
        .first()
        .is_some_and(|part| part.eq_ignore_ascii_case("disk-000-whole"))
    {
        parts.remove(0);
    }
    if parts.is_empty()
        || parts
            .iter()
            .any(|part| part.is_empty() || *part == "." || *part == ".." || part.contains(':'))
    {
        return Err(format!("unsafe FAT12 replacement path: {value}"));
    }
    if parts[0].starts_with('/') || slashed.starts_with('/') {
        return Err(format!(
            "absolute FAT12 replacement path is not allowed: {value}"
        ));
    }
    Ok(parts.join("/").to_ascii_uppercase())
}

fn short_name(entry: &[u8]) -> Result<String> {
    let raw = entry
        .get(..11)
        .ok_or_else(|| "truncated FAT12 directory entry".to_string())?;
    let mut base = raw[..8].to_vec();
    let mut extension = raw[8..11].to_vec();
    while base.last() == Some(&b' ') {
        base.pop();
    }
    while extension.last() == Some(&b' ') {
        extension.pop();
    }
    if base.first() == Some(&0x05) {
        base[0] = 0xe5;
    }
    if base.is_empty()
        || base
            .iter()
            .chain(&extension)
            .any(|byte| *byte == 0 || *byte == b'/' || *byte == b'\\' || *byte == b':')
    {
        return Err("invalid live FAT12 short name".into());
    }
    let base = decode_dos_component(&base);
    let extension = decode_dos_component(&extension);
    if extension.is_empty() {
        Ok(base)
    } else {
        Ok(format!("{base}.{extension}"))
    }
}

fn decode_dos_component(bytes: &[u8]) -> String {
    if bytes.is_ascii() {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    encoding_rs::SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(bytes)
        .map(|text| text.into_owned())
        .unwrap_or_else(|| format!("~{}", hex(bytes)))
}

fn read_file_data(
    source: &[u8],
    view: &LinearView,
    layout: &Layout,
    chain: &[u16],
    size: usize,
    path: &str,
) -> Result<Vec<u8>> {
    let capacity = chain
        .len()
        .checked_mul(layout.cluster_bytes)
        .ok_or_else(|| format!("{path} chain capacity overflow"))?;
    if size > capacity {
        return Err(format!("{path} size exceeds its cluster chain"));
    }
    let mut data = Vec::with_capacity(size);
    for &cluster in chain {
        if data.len() == size {
            break;
        }
        let count = (size - data.len()).min(layout.cluster_bytes);
        let offset = cluster_offset(layout, cluster)?;
        data.extend_from_slice(&read_logical(source, view, offset, count)?);
    }
    if data.len() != size {
        return Err(format!("{path} data chain is shorter than its file size"));
    }
    Ok(data)
}

fn read_chain(
    fat: &[u8],
    max_cluster: u16,
    start: u16,
    path: &str,
    owners: &mut HashMap<u16, String>,
) -> Result<Vec<u16>> {
    if start == 0 {
        return Ok(Vec::new());
    }
    let mut chain = Vec::new();
    let mut seen = HashSet::new();
    let mut cluster = start;
    loop {
        if cluster < 2 || cluster > max_cluster || cluster >= 0x0ff0 {
            return Err(format!(
                "{path} has out-of-range FAT12 cluster {cluster:#x}"
            ));
        }
        if !seen.insert(cluster) {
            return Err(format!("{path} FAT12 chain loops at cluster {cluster}"));
        }
        if let Some(previous) = owners.insert(cluster, path.to_string()) {
            return Err(format!("{path} is cross-linked with {previous}"));
        }
        chain.push(cluster);
        match fat12_next(fat, cluster)? {
            0x0ff8..=0x0fff => return Ok(chain),
            0x0ff7 => return Err(format!("{path} chain reaches a bad cluster")),
            0x0ff0..=0x0ff6 => return Err(format!("{path} chain reaches a reserved marker")),
            0 | 1 => return Err(format!("{path} chain reaches a free/reserved cluster")),
            next => cluster = next,
        }
    }
}

fn rewrite_changed_chains(
    fat: &mut [u8],
    files: &[FilePlan],
    changed_paths: &BTreeSet<String>,
) -> Result<()> {
    for plan in files {
        if changed_paths.contains(&plan.path) {
            for &cluster in &plan.old_chain {
                set_fat12(fat, cluster, 0)?;
            }
        }
    }
    for plan in files {
        if !changed_paths.contains(&plan.path) {
            continue;
        }
        for (index, &cluster) in plan.desired_chain.iter().enumerate() {
            let next = plan.desired_chain.get(index + 1).copied().unwrap_or(0x0fff);
            set_fat12(fat, cluster, next)?;
        }
    }
    Ok(())
}

fn validate_rebuilt(
    rebuilt: &[u8],
    expected_files: &[FilePlan],
    changed_paths: &BTreeSet<String>,
) -> Result<()> {
    let (view, layout, _fat, files, file_count) = inspect_source(rebuilt)?;
    if file_count != expected_files.len() || files.len() != expected_files.len() {
        return Err("rebuilt FAT12 file count changed".into());
    }
    let actual: HashMap<&str, &FilePlan> = files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect();
    for expected in expected_files {
        let rebuilt_file = actual
            .get(expected.path.as_str())
            .ok_or_else(|| format!("rebuilt file {} is missing", expected.path))?;
        if rebuilt_file.old_chain != expected.desired_chain {
            return Err(format!(
                "rebuilt file {} FAT12 chain differs from the allocation plan",
                expected.path
            ));
        }
        let desired = if changed_paths.contains(&expected.path) {
            &expected.data
        } else {
            &expected.original
        };
        if &rebuilt_file.original != desired {
            return Err(format!("rebuilt file {} content mismatch", expected.path));
        }
        let expected_size = if changed_paths.contains(&expected.path) {
            expected.data.len()
        } else {
            expected.original.len()
        };
        let expected_first = expected.desired_chain.first().copied().unwrap_or(0);
        let entry = read_logical(rebuilt, &view, rebuilt_file.entry_offset, 32)?;
        if rebuilt_file.original.len() != expected_size
            || read_u16(&entry, 26)? != expected_first
            || usize::try_from(u32::from_le_bytes(
                entry[28..32].try_into().expect("four bytes"),
            ))
            .map_err(|_| format!("{} size does not fit usize", expected.path))?
                != expected_size
        {
            return Err(format!(
                "rebuilt file {} directory entry or size mismatch",
                expected.path
            ));
        }
    }
    // Rechecking the FAT copies and all directory chains is performed by
    // inspect_source above. Keep these values explicit so the postflight also
    // confirms the expected geometry is addressable through its logical view.
    if layout.total_sectors * layout.bytes_per_sector != view.len() {
        return Err("rebuilt logical volume length mismatch".into());
    }
    Ok(())
}

fn cluster_count(size: usize, cluster_bytes: usize) -> usize {
    size.div_ceil(cluster_bytes)
}

fn cluster_offset(layout: &Layout, cluster: u16) -> Result<usize> {
    if cluster < 2 || cluster > layout.max_cluster {
        return Err(format!("FAT12 cluster {cluster} is outside the data area"));
    }
    let data_base = layout
        .first_data_sector
        .checked_mul(layout.bytes_per_sector)
        .ok_or_else(|| "FAT12 data-area offset overflow".to_string())?;
    let delta = usize::from(cluster - 2)
        .checked_mul(layout.cluster_bytes)
        .ok_or_else(|| "FAT12 cluster offset overflow".to_string())?;
    data_base
        .checked_add(delta)
        .ok_or_else(|| "FAT12 cluster offset overflow".to_string())
}

fn fat12_next(fat: &[u8], cluster: u16) -> Result<u16> {
    let offset = usize::from(cluster) * 3 / 2;
    let pair = fat
        .get(offset..offset + 2)
        .ok_or_else(|| format!("FAT12 table has no slot for cluster {cluster}"))?;
    let packed = u16::from_le_bytes([pair[0], pair[1]]);
    Ok(if cluster & 1 == 0 {
        packed & 0x0fff
    } else {
        packed >> 4
    })
}

fn set_fat12(fat: &mut [u8], cluster: u16, value: u16) -> Result<()> {
    if value > 0x0fff {
        return Err(format!("FAT12 value {value:#x} exceeds 12 bits"));
    }
    let offset = usize::from(cluster) * 3 / 2;
    let pair = fat
        .get_mut(offset..offset + 2)
        .ok_or_else(|| format!("FAT12 table has no slot for cluster {cluster}"))?;
    let old = u16::from_le_bytes([pair[0], pair[1]]);
    let packed = if cluster & 1 == 0 {
        (old & 0xf000) | value
    } else {
        (old & 0x000f) | (value << 4)
    };
    pair.copy_from_slice(&packed.to_le_bytes());
    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let pair = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| format!("u16 read at {offset:#x} is out of bounds"))?;
    Ok(u16::from_le_bytes([pair[0], pair[1]]))
}

fn read_logical(source: &[u8], view: &LinearView, offset: usize, length: usize) -> Result<Vec<u8>> {
    let end = offset
        .checked_add(length)
        .ok_or_else(|| "logical read range overflow".to_string())?;
    let mapped = view
        .map_range(offset..end)
        .map_err(|error| format!("logical read mapping failed: {error}"))?;
    let mut data = Vec::with_capacity(length);
    for span in mapped {
        if span.source_range.end > source.len() {
            return Err("logical read maps beyond source D88".into());
        }
        data.extend_from_slice(&source[span.source_range]);
    }
    if data.len() != length {
        return Err("logical view does not cover the complete read".into());
    }
    Ok(data)
}

fn patch_logical(view: &LinearView, output: &mut [u8], offset: usize, data: &[u8]) -> Result<()> {
    let end = offset
        .checked_add(data.len())
        .ok_or_else(|| "logical patch range overflow".to_string())?;
    let mapped = view
        .map_range(offset..end)
        .map_err(|error| format!("logical patch mapping failed: {error}"))?;
    let mut cursor = 0usize;
    for span in mapped {
        let len = span.source_range.len();
        let next = cursor
            .checked_add(len)
            .ok_or_else(|| "logical patch cursor overflow".to_string())?;
        if next > data.len() || span.source_range.end > output.len() {
            return Err("logical patch maps outside its source image".into());
        }
        output[span.source_range].copy_from_slice(&data[cursor..next]);
        cursor = next;
    }
    if cursor != data.len() {
        return Err("logical view does not cover the complete patch".into());
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn released_cluster_reused_by_earlier_file_is_not_cleared_later() {
        let mut fat = vec![0; 32];
        set_fat12(&mut fat, 2, 0x0fff).unwrap();
        set_fat12(&mut fat, 3, 4).unwrap();
        set_fat12(&mut fat, 4, 0x0fff).unwrap();
        let files = vec![
            FilePlan {
                path: "CREATE.CRS".into(),
                entry_offset: 0,
                old_chain: vec![2],
                desired_chain: vec![2, 4],
                original: Vec::new(),
                data: Vec::new(),
            },
            FilePlan {
                path: "LATER.CRS".into(),
                entry_offset: 0,
                old_chain: vec![3, 4],
                desired_chain: vec![3],
                original: Vec::new(),
                data: Vec::new(),
            },
        ];
        let changed_paths = ["CREATE.CRS".into(), "LATER.CRS".into()]
            .into_iter()
            .collect();

        rewrite_changed_chains(&mut fat, &files, &changed_paths).unwrap();

        assert_eq!(fat12_next(&fat, 2).unwrap(), 4);
        assert_eq!(fat12_next(&fat, 4).unwrap(), 0x0fff);
        assert_eq!(fat12_next(&fat, 3).unwrap(), 0x0fff);
    }
}
