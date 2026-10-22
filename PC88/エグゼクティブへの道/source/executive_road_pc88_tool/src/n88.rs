use encoding_rs::SHIFT_JIS;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use vn_d88::{Decoder, Disk, Sector, StandardCodec};

const SYSTEM_TRACK_SLOT: usize = 37;
const DIRECTORY_FIRST_RECORD: u16 = 1;
const DIRECTORY_LAST_RECORD: u16 = 6;
const STARTUP_RECORD: u16 = 7;
const ALLOCATION_RECORD: u16 = 8;
const CLUSTER_COUNT: usize = 160;
const SECTORS_PER_CLUSTER: usize = 8;
const SECTOR_SIZE: usize = 256;

#[derive(Debug, Clone, Serialize)]
pub struct SectorSource {
    pub track_slot: usize,
    pub sector_id: u16,
    pub source_offset: usize,
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct ResourceFile {
    pub directory_index: usize,
    pub raw_name: [u8; 9],
    pub raw_entry: [u8; 16],
    pub directory_entry_source_offset: usize,
    pub display_name: String,
    pub output_name: String,
    pub attribute: u8,
    pub start_cluster: u8,
    pub clusters: Vec<u8>,
    pub final_sectors: u8,
    pub data: Vec<u8>,
    pub sha256: String,
    pub source_sectors: Vec<SectorSource>,
}

#[derive(Debug, Clone, Serialize)]
pub struct N88DiskSummary {
    pub source_length: usize,
    pub source_sha256: String,
    pub d88_name_hex: String,
    pub directory_track_slot: usize,
    pub directory_entries: usize,
    pub directory_terminator_entry: usize,
    pub stale_entries_after_terminator: usize,
    pub directory_table_sha256: String,
    pub startup_sector_sha256: String,
    pub allocation_track_slot: usize,
    pub allocation_sector: u16,
    pub allocation_entries: usize,
    pub allocation_table_sha256: String,
    pub referenced_clusters: usize,
    pub reserved_clusters: usize,
    pub free_clusters: usize,
    pub unreferenced_allocated_clusters: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ParsedN88Disk {
    pub summary: N88DiskSummary,
    pub files: Vec<ResourceFile>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FileReplacement {
    pub directory_index: usize,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
struct DirectoryEntry {
    index: usize,
    raw_name: [u8; 9],
    raw_entry: [u8; 16],
    source_offset: usize,
    attribute: u8,
    start_cluster: u8,
}

/// Parse this game's N88-like disk layout from one validated D88 image.
/// Directory records occupy system-track R1-R6; R7 carries startup metadata,
/// and R8 stores one 160-entry allocation chain table. Each cluster is eight
/// 256-byte sectors, mapped to R1-R8 or R9-R16 by cluster parity.
pub fn parse_disk(source: &[u8]) -> Result<ParsedN88Disk, String> {
    let image = StandardCodec
        .decode(source)
        .map_err(|error| format!("D88 解析失败：{error}"))?;
    if image.disks.len() != 1 {
        return Err(format!(
            "资源提取要求单盘 D88，实际解析到 {} 张",
            image.disks.len()
        ));
    }
    if image.trailing_range.is_some() || !image.diagnostics.is_empty() {
        return Err("D88 含有未解析尾部或轨道诊断，拒绝声称资源完整".into());
    }
    let disk = &image.disks[0];
    validate_physical_layout(source, disk)?;

    let directory_track = track(disk, SYSTEM_TRACK_SLOT)?;
    let directory_sectors = (DIRECTORY_FIRST_RECORD..=DIRECTORY_LAST_RECORD)
        .map(|record| sector(directory_track, SYSTEM_TRACK_SLOT, record))
        .collect::<Result<Vec<_>, _>>()?;
    let mut directory_bytes = Vec::with_capacity(directory_sectors.len() * SECTOR_SIZE);
    for item in &directory_sectors {
        directory_bytes.extend_from_slice(
            source
                .get(item.data_range.clone())
                .ok_or("目录扇区来源范围越界")?,
        );
    }
    let (entries, terminator, stale_entries) =
        parse_directory_bytes(&directory_bytes, &directory_sectors)?;
    if entries.is_empty() {
        return Err("系统轨目录没有活动文件项".into());
    }

    // The first 160 bytes of R8 are the game's per-disk allocation chain table.
    // The rest of the sector is opaque and remains in the source D88.
    let allocation_sector = sector(directory_track, SYSTEM_TRACK_SLOT, ALLOCATION_RECORD)?;
    let startup_sector = sector(directory_track, SYSTEM_TRACK_SLOT, STARTUP_RECORD)?;
    let allocation_bytes = &source[allocation_sector.data_range.clone()];
    if allocation_bytes.len() != SECTOR_SIZE {
        return Err(format!(
            "分配表扇区长度为 {}，预期 {SECTOR_SIZE}",
            allocation_bytes.len()
        ));
    }
    let allocation = &allocation_bytes[..CLUSTER_COUNT];
    validate_allocation_values(allocation)?;

    let mut files = Vec::with_capacity(entries.len());
    let mut cluster_owner = HashMap::<u8, usize>::new();
    let mut output_names = HashSet::<String>::new();
    for entry in &entries {
        let (clusters, final_sectors) = trace_chain(allocation, entry.start_cluster)?;
        for cluster in &clusters {
            if let Some(previous) = cluster_owner.insert(*cluster, entry.index) {
                return Err(format!(
                    "簇 {cluster:02X} 被目录项 {} 和 {} 交叉引用",
                    previous, entry.index
                ));
            }
        }
        let display_name = decode_name(&entry.raw_name)?;
        let mut output_name = safe_filename(&display_name, entry.index);
        if !output_names.insert(output_name.to_lowercase()) {
            output_name = add_collision_suffix(&output_name, entry.index);
            if !output_names.insert(output_name.to_lowercase()) {
                return Err(format!("无法为目录项 {} 生成唯一输出名", entry.index));
            }
        }

        let mut data = Vec::new();
        let mut source_sectors = Vec::new();
        for (chain_index, cluster) in clusters.iter().copied().enumerate() {
            let track_slot = usize::from(cluster) / 2;
            let first_record = if cluster % 2 == 0 { 1 } else { 9 };
            let sector_count = if chain_index + 1 == clusters.len() {
                usize::from(final_sectors)
            } else {
                SECTORS_PER_CLUSTER
            };
            let data_track = track(disk, track_slot)?;
            for sector_id in first_record..first_record + sector_count as u16 {
                let item = sector(data_track, track_slot, sector_id)?;
                if item.fdc_status != 0 || item.deleted_data != 0 {
                    return Err(format!(
                        "文件 {} 的簇 {cluster:02X} 扇区 R{sector_id} 状态非零",
                        display_name
                    ));
                }
                let bytes = source
                    .get(item.data_range.clone())
                    .ok_or_else(|| format!("文件 {} 的扇区范围越界", display_name))?;
                if bytes.len() != SECTOR_SIZE {
                    return Err(format!(
                        "文件 {} 的簇 {cluster:02X} 扇区 R{sector_id} 不是 256 字节",
                        display_name
                    ));
                }
                data.extend_from_slice(bytes);
                source_sectors.push(SectorSource {
                    track_slot,
                    sector_id,
                    source_offset: item.data_range.start,
                    length: item.data_range.len(),
                });
            }
        }
        let expected_length =
            ((clusters.len() - 1) * SECTORS_PER_CLUSTER + usize::from(final_sectors)) * SECTOR_SIZE;
        if data.len() != expected_length {
            return Err(format!("文件 {display_name} 长度与簇链不符"));
        }
        files.push(ResourceFile {
            directory_index: entry.index,
            raw_name: entry.raw_name,
            raw_entry: entry.raw_entry,
            directory_entry_source_offset: entry.source_offset,
            display_name,
            output_name,
            attribute: entry.attribute,
            start_cluster: entry.start_cluster,
            clusters,
            final_sectors,
            sha256: super::sha256_hex(&data),
            data,
            source_sectors,
        });
    }

    let mut referenced = cluster_owner.keys().copied().collect::<HashSet<_>>();
    let unreferenced_allocated_clusters = allocation
        .iter()
        .enumerate()
        .filter_map(|(cluster, value)| {
            let allocated = usize::from(*value) < CLUSTER_COUNT || is_end_marker(*value);
            (allocated && !referenced.remove(&(cluster as u8))).then_some(cluster as u8)
        })
        .collect::<Vec<_>>();
    let free_clusters = allocation.iter().filter(|value| **value == 0xFF).count();
    let reserved_clusters = allocation.iter().filter(|value| **value == 0xFE).count();
    let referenced_clusters = cluster_owner.len();
    let end_markers = allocation
        .iter()
        .filter(|value| is_end_marker(**value))
        .count();
    if end_markers < files.len() {
        return Err(format!(
            "分配表有 {end_markers} 个链尾标记，但目录与孤立簇无法解释全部链尾"
        ));
    }

    let mut warnings = Vec::new();
    if stale_entries != 0 {
        warnings.push(format!(
            "目录终止符后保留了 {stale_entries} 条非空旧记录，未作为活动文件导出"
        ));
    }
    if !unreferenced_allocated_clusters.is_empty() {
        warnings.push(format!(
            "分配表有 {} 个未被活动目录引用的已分配簇",
            unreferenced_allocated_clusters.len()
        ));
    }
    let summary = N88DiskSummary {
        source_length: source.len(),
        source_sha256: super::sha256_hex(source),
        d88_name_hex: hex(&disk.header.raw_name),
        directory_track_slot: SYSTEM_TRACK_SLOT,
        directory_entries: files.len(),
        directory_terminator_entry: terminator,
        stale_entries_after_terminator: stale_entries,
        directory_table_sha256: super::sha256_hex(&directory_bytes),
        startup_sector_sha256: super::sha256_hex(
            source
                .get(startup_sector.data_range.clone())
                .ok_or("启动元数据扇区范围越界")?,
        ),
        allocation_track_slot: SYSTEM_TRACK_SLOT,
        allocation_sector: ALLOCATION_RECORD,
        allocation_entries: CLUSTER_COUNT,
        allocation_table_sha256: super::sha256_hex(allocation),
        referenced_clusters,
        reserved_clusters,
        free_clusters,
        unreferenced_allocated_clusters,
    };
    Ok(ParsedN88Disk {
        summary,
        files,
        warnings,
    })
}

/// Rebuild selected file data while preserving the D88 track layout. Replaced
/// files may grow or shrink in 256-byte sectors; their FAT chains are reused and
/// extended from free clusters as needed. The disk image itself stays the same
/// physical size.
pub fn rebuild_file_data(
    source: &[u8],
    replacements: &[FileReplacement],
) -> Result<Vec<u8>, String> {
    let parsed = parse_disk(source)?;
    if replacements.is_empty() {
        return Ok(source.to_vec());
    }

    let mut replacement_by_index = BTreeMap::<usize, &FileReplacement>::new();
    let mut files_by_index = BTreeMap::<usize, &ResourceFile>::new();
    for file in &parsed.files {
        files_by_index.insert(file.directory_index, file);
    }
    for replacement in replacements {
        if replacement.data.is_empty() || replacement.data.len() % SECTOR_SIZE != 0 {
            return Err(format!(
                "目录项 {} 的替换文件长度 {} 必须是 256 字节的正整数倍",
                replacement.directory_index,
                replacement.data.len()
            ));
        }
        if !files_by_index.contains_key(&replacement.directory_index) {
            return Err(format!(
                "找不到要替换的目录项 {}",
                replacement.directory_index
            ));
        }
        if replacement_by_index
            .insert(replacement.directory_index, replacement)
            .is_some()
        {
            return Err(format!(
                "目录项 {} 被要求替换多次",
                replacement.directory_index
            ));
        }
    }

    let image = StandardCodec
        .decode(source)
        .map_err(|error| format!("D88 解析失败：{error}"))?;
    let disk = image
        .disks
        .first()
        .ok_or_else(|| "D88 中没有磁盘".to_owned())?;
    let system_track = track(disk, SYSTEM_TRACK_SLOT)?;
    let allocation_sector = sector(system_track, SYSTEM_TRACK_SLOT, ALLOCATION_RECORD)?;
    let allocation_range = allocation_sector.data_range.clone();
    let allocation_end = allocation_range
        .start
        .checked_add(CLUSTER_COUNT)
        .ok_or_else(|| "分配表来源范围溢出".to_owned())?;
    let mut allocation = source
        .get(allocation_range.start..allocation_end)
        .ok_or_else(|| "分配表来源范围越界".to_owned())?
        .to_vec();
    validate_allocation_values(&allocation)?;

    let mut needed_clusters = BTreeMap::<usize, usize>::new();
    let mut chains = BTreeMap::<usize, Vec<u8>>::new();
    let mut reserved = HashSet::<u8>::new();
    for (directory_index, replacement) in &replacement_by_index {
        let file = files_by_index[directory_index];
        let sectors_needed = replacement.data.len() / SECTOR_SIZE;
        let clusters_needed = sectors_needed.div_ceil(SECTORS_PER_CLUSTER);
        if clusters_needed == 0 || clusters_needed > CLUSTER_COUNT {
            return Err(format!(
                "目录项 {directory_index} 需要 {clusters_needed} 个簇，超出磁盘容量"
            ));
        }
        needed_clusters.insert(*directory_index, clusters_needed);
        for cluster in &file.clusters {
            allocation[usize::from(*cluster)] = 0xFF;
        }
        let preferred = file
            .clusters
            .iter()
            .copied()
            .take(clusters_needed.min(file.clusters.len()))
            .collect::<Vec<_>>();
        for cluster in &preferred {
            if !reserved.insert(*cluster) {
                return Err(format!("簇 {cluster:02X} 在文件替换计划中重复占用"));
            }
        }
        chains.insert(*directory_index, preferred);
    }

    for (directory_index, needed) in &needed_clusters {
        let chain = chains
            .get_mut(directory_index)
            .ok_or_else(|| "替换文件缺少簇链".to_owned())?;
        while chain.len() < *needed {
            let candidate = allocation
                .iter()
                .enumerate()
                .find_map(|(index, value)| {
                    let cluster = index as u8;
                    (*value == 0xFF && !reserved.contains(&cluster)).then_some(cluster)
                })
                .ok_or_else(|| {
                    format!("目录项 {directory_index} 扩展后需要 {needed} 个簇，但磁盘空闲簇不足")
                })?;
            reserved.insert(candidate);
            chain.push(candidate);
        }
    }

    for (directory_index, chain) in &chains {
        let replacement = replacement_by_index[directory_index];
        let sectors_needed = replacement.data.len() / SECTOR_SIZE;
        for (position, cluster) in chain.iter().copied().enumerate() {
            allocation[usize::from(cluster)] = if position + 1 == chain.len() {
                let final_sectors = sectors_needed - position * SECTORS_PER_CLUSTER;
                if !(1..=SECTORS_PER_CLUSTER).contains(&final_sectors) {
                    return Err(format!(
                        "目录项 {directory_index} 末簇扇区数 {final_sectors} 无效"
                    ));
                }
                0xC0 | final_sectors as u8
            } else {
                chain[position + 1]
            };
        }
    }

    let mut output = source.to_vec();
    for (directory_index, chain) in &chains {
        let file = files_by_index[directory_index];
        let replacement = replacement_by_index[directory_index];
        let start_cluster_offset = file
            .directory_entry_source_offset
            .checked_add(10)
            .ok_or_else(|| "目录项起始簇偏移溢出".to_owned())?;
        *output
            .get_mut(start_cluster_offset)
            .ok_or_else(|| "目录项起始簇字段越界".to_owned())? =
            *chain.first().ok_or_else(|| "替换文件簇链为空".to_owned())?;

        let sector_count = replacement.data.len() / SECTOR_SIZE;
        for sector_index in 0..sector_count {
            let cluster = chain[sector_index / SECTORS_PER_CLUSTER];
            let in_cluster = (sector_index % SECTORS_PER_CLUSTER) as u16;
            let first_record = if cluster % 2 == 0 { 1 } else { 9 };
            let record = first_record + in_cluster;
            let data_track = track(disk, usize::from(cluster) / 2)?;
            let target_sector = sector(data_track, usize::from(cluster) / 2, record)?;
            if target_sector.data_range.len() != SECTOR_SIZE {
                return Err(format!(
                    "目录项 {directory_index} 簇 {cluster:02X} R{record} 不是 256 字节"
                ));
            }
            let source_start = sector_index * SECTOR_SIZE;
            let source_end = source_start + SECTOR_SIZE;
            let destination = output
                .get_mut(target_sector.data_range.clone())
                .ok_or_else(|| "替换目标扇区越界".to_owned())?;
            destination.copy_from_slice(&replacement.data[source_start..source_end]);
        }
    }

    output[allocation_range.start..allocation_end].copy_from_slice(&allocation);
    let rebuilt = parse_disk(&output)?;
    for (directory_index, replacement) in &replacement_by_index {
        let file = rebuilt
            .files
            .iter()
            .find(|file| file.directory_index == *directory_index)
            .ok_or_else(|| format!("重解析后目录项 {directory_index} 不存在"))?;
        if file.data != replacement.data {
            return Err(format!("目录项 {directory_index} 写回后字节校验失败"));
        }
    }
    Ok(output)
}

fn validate_physical_layout(source: &[u8], disk: &Disk) -> Result<(), String> {
    if disk.source_range.start != 0 || disk.source_range.end != source.len() {
        return Err("D88 数据盘范围没有覆盖完整输入".into());
    }
    if disk.tracks.len() != 80 {
        return Err(format!(
            "预期 80 条 40 柱面双面轨道，实际解析到 {} 条",
            disk.tracks.len()
        ));
    }
    for slot in 0..80 {
        let current = track(disk, slot)?;
        if current.sectors.len() != 16 || current.opaque_tail.is_some() {
            return Err(format!("轨槽 {slot} 不是完整的 16×256 字节轨道"));
        }
        for record in 1..=16 {
            let item = sector(current, slot, record)?;
            if item.address.id.cylinder != (slot / 2) as u16
                || usize::from(item.address.id.head) != slot % 2
                || item.address.id.size_code != 1
                || item.sectors_in_track != 16
                || item.data_range.len() != SECTOR_SIZE
                || item.fdc_status != 0
                || item.deleted_data != 0
            {
                return Err(format!(
                    "轨槽 {slot} 的 CHRN/长度/状态不符合本作磁盘布局 (R{record})"
                ));
            }
        }
    }
    Ok(())
}

fn track(disk: &Disk, slot: usize) -> Result<&vn_d88::Track, String> {
    let matches = disk
        .tracks
        .iter()
        .filter(|candidate| candidate.slot == slot)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [only] => Ok(*only),
        [] => Err(format!("D88 缺少轨槽 {slot}")),
        _ => Err(format!("D88 轨槽 {slot} 重复")),
    }
}

fn sector(track: &vn_d88::Track, slot: usize, record: u16) -> Result<&Sector, String> {
    let matches = track
        .sectors
        .iter()
        .filter(|candidate| candidate.address.id.record == record)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [only] => Ok(*only),
        [] => Err(format!("轨槽 {slot} 缺少 R{record}")),
        _ => Err(format!("轨槽 {slot} 的 R{record} ID 重复")),
    }
}

fn parse_directory_bytes(
    directory: &[u8],
    sectors: &[&Sector],
) -> Result<(Vec<DirectoryEntry>, usize, usize), String> {
    if directory.len() != sectors.len() * SECTOR_SIZE {
        return Err("目录逻辑视图长度不符".into());
    }
    let entries = directory.chunks_exact(16).collect::<Vec<_>>();
    let Some(terminator) = entries.iter().position(|entry| entry[0] == 0xFF) else {
        return Err("系统轨目录没有 0xFF 终止项".into());
    };
    let mut files = Vec::new();
    for (index, entry) in entries[..terminator].iter().enumerate() {
        if entry[0] == 0x00 || entry[0] == 0xE5 {
            continue;
        }
        let raw_entry: [u8; 16] = (*entry).try_into().map_err(|_| "目录项长度错误")?;
        let raw_name: [u8; 9] = raw_entry[..9]
            .try_into()
            .map_err(|_| "目录文件名字段长度错误")?;
        let _ = decode_name(&raw_name)?;
        files.push(DirectoryEntry {
            index,
            raw_name,
            raw_entry,
            source_offset: sectors[index / 16].data_range.start + (index % 16) * 16,
            attribute: raw_entry[9],
            start_cluster: raw_entry[10],
        });
    }
    let stale_entries = entries[terminator + 1..]
        .iter()
        .filter(|entry| entry.iter().any(|byte| *byte != 0x00 && *byte != 0xFF))
        .count();
    Ok((files, terminator, stale_entries))
}

fn validate_allocation_values(allocation: &[u8]) -> Result<(), String> {
    if allocation.len() != CLUSTER_COUNT {
        return Err(format!(
            "分配表有 {} 项，预期 {CLUSTER_COUNT}",
            allocation.len()
        ));
    }
    for (cluster, value) in allocation.iter().copied().enumerate() {
        if usize::from(value) < CLUSTER_COUNT
            || value == 0xFE
            || value == 0xFF
            || is_end_marker(value)
        {
            continue;
        }
        return Err(format!("分配表簇 {cluster:02X} 含有未知值 {value:02X}"));
    }
    Ok(())
}

fn trace_chain(allocation: &[u8], start: u8) -> Result<(Vec<u8>, u8), String> {
    let mut current = start;
    let mut chain = Vec::new();
    let mut visited = HashSet::new();
    for _ in 0..=CLUSTER_COUNT {
        if usize::from(current) >= CLUSTER_COUNT {
            return Err(format!("目录起始簇 {start:02X} 或后继簇越界"));
        }
        if !visited.insert(current) {
            return Err(format!("目录起始簇 {start:02X} 的分配链存在环"));
        }
        chain.push(current);
        let next = allocation[usize::from(current)];
        if is_end_marker(next) {
            let final_sectors = next & 0x0F;
            if !(1..=8).contains(&final_sectors) {
                return Err(format!(
                    "簇 {current:02X} 的末簇扇区数 {final_sectors} 无效"
                ));
            }
            return Ok((chain, final_sectors));
        }
        if next == 0xFE || next == 0xFF {
            return Err(format!(
                "目录起始簇 {start:02X} 的分配链在簇 {current:02X} 遇到 {next:02X}"
            ));
        }
        if usize::from(next) >= CLUSTER_COUNT {
            return Err(format!("簇 {current:02X} 的后继 {next:02X} 越界"));
        }
        current = next;
    }
    Err(format!("目录起始簇 {start:02X} 的分配链过长"))
}

fn is_end_marker(value: u8) -> bool {
    (0xC1..=0xC8).contains(&value)
}

fn decode_name(raw: &[u8; 9]) -> Result<String, String> {
    let base = trim_field(&raw[..6]);
    let extension = trim_field(&raw[6..9]);
    if base.is_empty() {
        return Err(format!("目录文件名为空：{}", hex(raw)));
    }
    let (base, base_errors) = SHIFT_JIS.decode_without_bom_handling(base);
    let (extension, extension_errors) = SHIFT_JIS.decode_without_bom_handling(extension);
    if base_errors || extension_errors {
        return Err(format!("目录文件名不是有效 CP932：{}", hex(raw)));
    }
    let base = base.trim_end_matches([' ', '\0']);
    let extension = extension.trim_end_matches([' ', '\0']);
    if base.is_empty() {
        return Err(format!("目录文件名主体为空：{}", hex(raw)));
    }
    Ok(if extension.is_empty() {
        base.to_owned()
    } else {
        format!("{base}.{extension}")
    })
}

fn trim_field(bytes: &[u8]) -> &[u8] {
    let end = bytes
        .iter()
        .rposition(|byte| !matches!(*byte, b' ' | 0))
        .map_or(0, |index| index + 1);
    &bytes[..end]
}

fn safe_filename(name: &str, directory_index: usize) -> String {
    let mut output = name
        .chars()
        .map(|character| {
            if character.is_control() || "<>:\"/\\|?*".contains(character) {
                '_'
            } else {
                character
            }
        })
        .collect::<String>();
    while output.ends_with([' ', '.']) {
        output.pop();
    }
    let stem = output
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    let numbered_device = stem
        .strip_prefix("COM")
        .or_else(|| stem.strip_prefix("LPT"))
        .is_some_and(|suffix| suffix.len() == 1 && matches!(suffix.as_bytes()[0], b'1'..=b'9'));
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL") || numbered_device;
    if output.is_empty() || output == "." || output == ".." || reserved {
        format!("entry_{directory_index:03}")
    } else {
        output
    }
}

fn add_collision_suffix(name: &str, directory_index: usize) -> String {
    match name.rsplit_once('.') {
        Some((base, extension)) => format!("{base}__entry_{directory_index:03}.{extension}"),
        None => format!("{name}__entry_{directory_index:03}"),
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[usize::from(byte >> 4)] as char);
        output.push(DIGITS[usize::from(byte & 0x0f)] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_disk() -> Vec<u8> {
        let header_len = 0x2A0usize;
        let track_len = 0x1100usize;
        let mut bytes = vec![0u8; header_len + 80 * track_len];
        bytes[..7].copy_from_slice(b"DISK A\0");
        for slot in 0..80usize {
            let track_start = header_len + slot * track_len;
            bytes[0x20 + slot * 4..0x24 + slot * 4]
                .copy_from_slice(&(track_start as u32).to_le_bytes());
            for record in 1..=16u8 {
                let sector_start = track_start + usize::from(record - 1) * 0x110;
                bytes[sector_start] = (slot / 2) as u8;
                bytes[sector_start + 1] = (slot % 2) as u8;
                bytes[sector_start + 2] = record;
                bytes[sector_start + 3] = 1;
                bytes[sector_start + 4..sector_start + 6].copy_from_slice(&16u16.to_le_bytes());
                bytes[sector_start + 14..sector_start + 16].copy_from_slice(&256u16.to_le_bytes());
                let data_start = sector_start + 16;
                bytes[data_start..data_start + SECTOR_SIZE].fill(0xFF);
            }
        }
        let system_start = header_len + SYSTEM_TRACK_SLOT * track_len;
        let directory_start = system_start + 16;
        let mut entry = [0u8; 16];
        entry[..6].copy_from_slice(b"FILE  ");
        entry[6..9].copy_from_slice(b"TXT");
        entry[9] = 0x01;
        entry[10] = 10;
        bytes[directory_start..directory_start + 16].copy_from_slice(&entry);
        bytes[directory_start + 16] = 0xFF;

        let allocation_start = system_start + 7 * 0x110 + 16;
        bytes[allocation_start..allocation_start + CLUSTER_COUNT].fill(0xFF);
        bytes[allocation_start + 10] = 0xC2;

        let data_track = header_len + (10 / 2) * track_len;
        let file_start = data_track + 16;
        bytes[file_start..file_start + SECTOR_SIZE].fill(b'X');
        bytes[file_start + 0x110..file_start + 0x110 + SECTOR_SIZE].fill(b'Y');
        let disk_len = bytes.len() as u32;
        bytes[0x1C..0x20].copy_from_slice(&disk_len.to_le_bytes());
        bytes
    }

    #[test]
    fn extracts_sector_exact_files_from_the_game_specific_allocation_table() {
        let source = synthetic_disk();
        let parsed = parse_disk(&source).expect("parse synthetic resource disk");
        assert_eq!(parsed.files.len(), 1);
        let file = &parsed.files[0];
        assert_eq!(file.display_name, "FILE.TXT");
        assert_eq!(file.directory_entry_source_offset, 0x277B0);
        assert_eq!(&file.raw_entry[..9], b"FILE  TXT");
        assert_eq!(file.clusters, [10]);
        assert_eq!(file.final_sectors, 2);
        assert_eq!(file.data, [vec![b'X'; 256], vec![b'Y'; 256]].concat());
        assert_eq!(file.source_sectors.len(), 2);
        assert_eq!(parsed.summary.referenced_clusters, 1);
        assert!(parsed.warnings.is_empty());
    }

    #[test]
    fn rejects_cross_linked_or_unterminated_file_chains() {
        let mut cross_linked = synthetic_disk();
        let header_len = 0x2A0usize;
        let track_len = 0x1100usize;
        let directory_start = header_len + SYSTEM_TRACK_SLOT * track_len + 16;
        cross_linked[directory_start + 16..directory_start + 32].fill(0);
        cross_linked[directory_start + 16..directory_start + 22].copy_from_slice(b"OTHER ");
        cross_linked[directory_start + 16 + 9] = 0;
        cross_linked[directory_start + 16 + 10] = 10;
        cross_linked[directory_start + 32] = 0xFF;
        assert!(parse_disk(&cross_linked).unwrap_err().contains("交叉引用"));

        let mut unterminated = synthetic_disk();
        let allocation_start = header_len + SYSTEM_TRACK_SLOT * track_len + 7 * 0x110 + 16;
        unterminated[allocation_start + 10] = 0xFF;
        assert!(parse_disk(&unterminated).unwrap_err().contains("遇到 FF"));
    }
}
