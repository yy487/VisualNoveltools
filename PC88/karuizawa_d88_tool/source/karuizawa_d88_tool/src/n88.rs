use crate::Result;
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use vn_d88::{Decoder, StandardCodec};

const DIRECTORY_TRACK_SLOT: usize = 37;
const DIRECTORY_SECTOR_FIRST: u16 = 1;
const DIRECTORY_SECTOR_LAST: u16 = 13;
const FAT_SECTOR_FIRST: u16 = 14;
const FAT_SECTOR_LAST: u16 = 16;
const FAT_ENTRIES: usize = 160;
const SECTORS_PER_CLUSTER: usize = 8;
const SECTOR_SIZE: usize = 256;
const CLUSTER_BYTES: usize = SECTORS_PER_CLUSTER * SECTOR_SIZE;
const PROTECTION_TRACK_SLOT: usize = 5;

#[derive(Debug, Clone)]
pub struct DiskFile {
    pub directory_index: usize,
    pub name: String,
    pub attribute: u8,
    pub start_cluster: u8,
    pub clusters: Vec<u8>,
    pub final_sectors: u8,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct RebuildReport {
    pub bytes: Vec<u8>,
    pub changed_files: usize,
    pub allocated_new_clusters: usize,
    pub remaining_free_clusters: usize,
}

#[derive(Debug, Clone)]
pub struct N88Disk {
    source: Vec<u8>,
    cluster_ranges: Vec<Vec<Range<usize>>>,
    directory_ranges: Vec<Range<usize>>,
    fat_ranges: Vec<Range<usize>>,
    fat: Vec<u8>,
    pub files: Vec<DiskFile>,
}

#[derive(Debug, Clone)]
struct PlannedFile {
    name: String,
    data: Vec<u8>,
    chain: Vec<u8>,
    final_sectors: u8,
    changed: bool,
}

#[derive(Debug, Clone)]
struct PlannedAddition {
    name: String,
    data: Vec<u8>,
    chain: Vec<u8>,
    final_sectors: u8,
    directory_index: usize,
}

impl N88Disk {
    pub fn parse(source: Vec<u8>) -> Result<Self> {
        let image = StandardCodec
            .decode(&source)
            .map_err(|error| error.to_string())?;
        if image.disks.len() != 1 {
            return Err(format!(
                "本作应为单盘 D88，实际解析出 {} 个盘记录",
                image.disks.len()
            ));
        }
        let disk = &image.disks[0];
        let directory_ranges = sector_ranges(
            disk,
            DIRECTORY_TRACK_SLOT,
            DIRECTORY_SECTOR_FIRST..=DIRECTORY_SECTOR_LAST,
        )?;
        let fat_sector_ranges = sector_ranges(
            disk,
            DIRECTORY_TRACK_SLOT,
            FAT_SECTOR_FIRST..=FAT_SECTOR_LAST,
        )?;
        let fat_ranges = fat_sector_ranges
            .iter()
            .map(|range| range.start..range.start + FAT_ENTRIES)
            .collect::<Vec<_>>();
        let fat_copies = fat_ranges
            .iter()
            .map(|range| source[range.clone()].to_vec())
            .collect::<Vec<_>>();
        if fat_copies.len() != 3 || fat_copies[1..].iter().any(|copy| *copy != fat_copies[0]) {
            return Err("N88 的三份 FAT 不一致".into());
        }
        let fat = fat_copies[0].clone();

        let mut cluster_ranges = Vec::with_capacity(FAT_ENTRIES);
        for cluster in 0..FAT_ENTRIES {
            cluster_ranges.push(map_cluster(disk, cluster)?);
        }
        let directory = gather(&source, &directory_ranges);
        let files = parse_directory(&source, &directory, &fat, &cluster_ranges)?;
        Ok(Self {
            source,
            cluster_ranges,
            directory_ranges,
            fat_ranges,
            fat,
            files,
        })
    }

    pub fn file(&self, name: &str) -> Result<&DiskFile> {
        if let Some(exact) = self.files.iter().find(|file| file.name == name) {
            return Ok(exact);
        }
        let matches = self
            .files
            .iter()
            .filter(|file| file.name.eq_ignore_ascii_case(name))
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [only] => Ok(*only),
            [] => Err(format!("原盘中没有 N88 文件 {name:?}")),
            _ => Err(format!("N88 文件名 {name:?} 的大小写匹配不唯一")),
        }
    }

    pub fn source(&self) -> &[u8] {
        &self.source
    }

    pub fn free_clusters(&self) -> usize {
        self.fat.iter().filter(|value| **value == 0xFF).count()
    }

    pub fn rebuild(&self, replacements: &HashMap<String, Vec<u8>>) -> Result<RebuildReport> {
        self.rebuild_with_additions(replacements, &HashMap::new())
    }

    pub fn rebuild_with_additions(
        &self,
        replacements: &HashMap<String, Vec<u8>>,
        additions: &HashMap<String, Vec<u8>>,
    ) -> Result<RebuildReport> {
        let known = self
            .files
            .iter()
            .map(|file| file.name.as_str())
            .collect::<HashSet<_>>();
        for name in replacements.keys() {
            if !known.contains(name.as_str()) {
                return Err(format!("替换表引用了不存在的 N88 文件 {name:?}"));
            }
        }
        for name in additions.keys() {
            if known.contains(name.as_str()) {
                return Err(format!("新增 N88 文件与已有文件重名：{name:?}"));
            }
            validate_new_name(name)?;
        }

        let mut plans = Vec::with_capacity(self.files.len());
        let mut needed = 0usize;
        let mut released = Vec::new();
        for file in &self.files {
            let replacement = replacements.get(&file.name).map(Vec::as_slice);
            let Some(replacement) = replacement else {
                plans.push(PlannedFile {
                    name: file.name.clone(),
                    data: file.data.clone(),
                    chain: file.clusters.clone(),
                    final_sectors: file.final_sectors,
                    changed: false,
                });
                continue;
            };
            if replacement != file.data.as_slice()
                && file
                    .clusters
                    .iter()
                    .any(|cluster| usize::from(*cluster) / 2 == PROTECTION_TRACK_SLOT)
            {
                return Err(format!(
                    "{} 的簇链穿过特殊保护轨，禁止按普通 N88 文件改写",
                    file.name
                ));
            }
            if replacement.is_empty() {
                return Err(format!("{} 的替换内容为空", file.name));
            }
            let replacement_sectors = replacement.len().div_ceil(SECTOR_SIZE);
            let target_clusters = replacement_sectors.div_ceil(SECTORS_PER_CLUSTER);
            needed += target_clusters.saturating_sub(file.clusters.len());
            let mut chain = file.clusters.clone();
            if target_clusters < chain.len() {
                let tail = &chain[target_clusters..];
                if tail
                    .iter()
                    .all(|cluster| usize::from(*cluster) / 2 != PROTECTION_TRACK_SLOT)
                {
                    released.extend_from_slice(tail);
                    chain.truncate(target_clusters);
                }
            }
            let storage_clusters = target_clusters.max(chain.len());
            let original_sectors =
                (file.clusters.len() - 1) * SECTORS_PER_CLUSTER + usize::from(file.final_sectors);
            let storage_sectors = if storage_clusters == target_clusters {
                replacement_sectors
            } else {
                original_sectors
            };
            let mut data = replacement.to_vec();
            data.resize(storage_sectors * SECTOR_SIZE, 0);
            let final_sectors = storage_sectors - (storage_clusters - 1) * SECTORS_PER_CLUSTER;
            plans.push(PlannedFile {
                name: file.name.clone(),
                changed: data != file.data,
                data,
                chain,
                final_sectors: u8::try_from(final_sectors)
                    .map_err(|_| format!("{} 的尾簇扇区数溢出", file.name))?,
            });
        }

        let mut addition_names = additions.keys().cloned().collect::<Vec<_>>();
        addition_names.sort();
        let original_terminator = directory_terminator(&self.source, &self.directory_ranges)?;
        let directory_slots =
            find_directory_slots(&self.source, &self.directory_ranges, addition_names.len())?;
        let mut additions_plan = Vec::with_capacity(addition_names.len());
        for (name, directory_index) in addition_names.into_iter().zip(directory_slots) {
            let source = additions
                .get(&name)
                .ok_or_else(|| format!("新增文件数据丢失：{name:?}"))?;
            if source.is_empty() {
                return Err(format!("新增文件 {name:?} 的内容为空"));
            }
            let sectors = source.len().div_ceil(SECTOR_SIZE);
            let clusters = sectors.div_ceil(SECTORS_PER_CLUSTER);
            needed += clusters;
            let storage_sectors = sectors;
            let mut data = source.clone();
            data.resize(storage_sectors * SECTOR_SIZE, 0);
            additions_plan.push(PlannedAddition {
                name,
                data,
                chain: Vec::new(),
                final_sectors: u8::try_from(sectors - (clusters - 1) * SECTORS_PER_CLUSTER)
                    .map_err(|_| "新增文件尾簇扇区数溢出")?,
                directory_index,
            });
        }

        let mut free = self
            .fat
            .iter()
            .enumerate()
            .filter_map(|(cluster, value)| (*value == 0xFF).then_some(cluster as u8))
            .collect::<Vec<_>>();
        free.extend(released.iter().copied());
        free.sort_unstable();
        if needed > free.len() {
            let growth = plans
                .iter()
                .filter_map(|plan| {
                    let required = plan.data.len().div_ceil(CLUSTER_BYTES);
                    let extra = required.saturating_sub(plan.chain.len());
                    (extra != 0).then(|| {
                        format!(
                            "{}:{}字节/{}簇(+{extra})",
                            plan.name,
                            plan.data.len(),
                            required
                        )
                    })
                })
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "回注需要 {needed} 个新簇，但原盘只有 {} 个空闲簇；{growth}",
                free.len()
            ));
        }
        let mut free_iter = free.iter().copied();
        for plan in &mut plans {
            let required = plan.data.len().div_ceil(CLUSTER_BYTES);
            while plan.chain.len() < required {
                plan.chain
                    .push(free_iter.next().ok_or("空闲簇规划内部不一致")?);
            }
        }
        for plan in &mut additions_plan {
            let required = plan.data.len().div_ceil(CLUSTER_BYTES);
            while plan.chain.len() < required {
                plan.chain
                    .push(free_iter.next().ok_or("新增文件空闲簇规划内部不一致")?);
            }
        }

        let mut output = self.source.clone();
        for cluster in released {
            write_fat(&mut output, &self.fat_ranges, cluster, 0xFF);
        }
        for (plan, original) in plans.iter().zip(&self.files) {
            if plan.changed || plan.chain != original.clusters {
                let ranges = file_ranges(&self.cluster_ranges, &plan.chain, plan.final_sectors)?;
                write_ranges(&mut output, &ranges, &plan.data)?;
            }
            for (index, cluster) in plan.chain.iter().copied().enumerate() {
                let value = if index + 1 == plan.chain.len() {
                    0xC0 + plan.final_sectors
                } else {
                    plan.chain[index + 1]
                };
                write_fat(&mut output, &self.fat_ranges, cluster, value);
            }
            write_gathered_byte(
                &mut output,
                &self.directory_ranges,
                original.directory_index * 16 + 10,
                plan.chain[0],
            )?;
        }
        for plan in &additions_plan {
            let ranges = file_ranges(&self.cluster_ranges, &plan.chain, plan.final_sectors)?;
            write_ranges(&mut output, &ranges, &plan.data)?;
            for (index, cluster) in plan.chain.iter().copied().enumerate() {
                let value = if index + 1 == plan.chain.len() {
                    0xC0 + plan.final_sectors
                } else {
                    plan.chain[index + 1]
                };
                write_fat(&mut output, &self.fat_ranges, cluster, value);
            }
            write_directory_entry(
                &mut output,
                &self.directory_ranges,
                plan.directory_index,
                &plan.name,
                plan.chain[0],
            )?;
        }
        if let Some(last_index) = additions_plan
            .iter()
            .map(|plan| plan.directory_index)
            .filter(|index| *index >= original_terminator)
            .max()
        {
            write_gathered_byte(
                &mut output,
                &self.directory_ranges,
                (last_index + 1) * 16,
                0xFF,
            )?;
        }

        StandardCodec
            .decode(&output)
            .map_err(|error| format!("回封后的 D88 容器校验失败：{error}"))?;
        let verified = Self::parse(output.clone())?;
        if verified.files.len() != self.files.len() + additions_plan.len() {
            return Err("回封后的 N88 文件数与规划不一致".into());
        }
        for plan in &plans {
            let actual = verified.file(&plan.name)?;
            if actual.data != plan.data
                || actual.clusters != plan.chain
                || actual.final_sectors != plan.final_sectors
            {
                return Err(format!("回封读回校验失败：{}", plan.name));
            }
        }
        for plan in &additions_plan {
            let actual = verified.file(&plan.name)?;
            if actual.data != plan.data
                || actual.clusters != plan.chain
                || actual.final_sectors != plan.final_sectors
            {
                return Err(format!("新增文件回封读回校验失败：{}", plan.name));
            }
        }
        let changed_files = plans.iter().filter(|plan| plan.changed).count();
        Ok(RebuildReport {
            bytes: output,
            changed_files: changed_files + additions_plan.len(),
            allocated_new_clusters: needed,
            remaining_free_clusters: free.len() - needed,
        })
    }
}

fn sector_ranges(
    disk: &vn_d88::Disk,
    slot: usize,
    records: std::ops::RangeInclusive<u16>,
) -> Result<Vec<Range<usize>>> {
    let track = disk
        .tracks
        .iter()
        .find(|track| track.slot == slot)
        .ok_or_else(|| format!("D88 缺少轨道槽 {slot}"))?;
    let mut output = Vec::new();
    for record in records {
        let matches = track
            .sectors
            .iter()
            .filter(|sector| sector.address.id.record == record)
            .collect::<Vec<_>>();
        if matches.len() != 1 || matches[0].data_range.len() != SECTOR_SIZE {
            return Err(format!(
                "轨道槽 {slot} 的扇区 ID {record} 不能唯一映射为 256 字节"
            ));
        }
        output.push(matches[0].data_range.clone());
    }
    Ok(output)
}

fn map_cluster(disk: &vn_d88::Disk, cluster: usize) -> Result<Vec<Range<usize>>> {
    let slot = cluster / 2;
    let half = cluster % 2;
    let track = disk
        .tracks
        .iter()
        .find(|track| track.slot == slot)
        .ok_or_else(|| format!("N88 簇 {cluster:02X} 所需轨道槽 {slot} 不存在"))?;
    if slot == PROTECTION_TRACK_SLOT {
        let ranges = track
            .sectors
            .iter()
            .map(|sector| sector.data_range.clone())
            .collect::<Vec<_>>();
        let total = ranges.iter().map(Range::len).sum::<usize>();
        if total != 2 * CLUSTER_BYTES {
            return Err(format!("保护轨有效载荷为 {total} 字节，预期 4096"));
        }
        return slice_ranges(&ranges, half * CLUSTER_BYTES, CLUSTER_BYTES);
    }
    let first = if half == 0 { 1 } else { 9 };
    sector_ranges(disk, slot, first..=first + 7)
}

fn slice_ranges(ranges: &[Range<usize>], start: usize, len: usize) -> Result<Vec<Range<usize>>> {
    let end = start.checked_add(len).ok_or("逻辑范围长度溢出")?;
    let total = ranges.iter().map(Range::len).sum::<usize>();
    if end > total {
        return Err(format!("逻辑范围 {start}..{end} 超过 {total}"));
    }
    let mut output = Vec::new();
    let mut cursor = 0usize;
    for range in ranges {
        let logical = cursor..cursor + range.len();
        cursor = logical.end;
        let overlap_start = start.max(logical.start);
        let overlap_end = end.min(logical.end);
        if overlap_start < overlap_end {
            let within = overlap_start - logical.start;
            output.push(range.start + within..range.start + within + (overlap_end - overlap_start));
        }
    }
    Ok(output)
}

fn parse_directory(
    source: &[u8],
    directory: &[u8],
    fat: &[u8],
    clusters: &[Vec<Range<usize>>],
) -> Result<Vec<DiskFile>> {
    let mut files = Vec::new();
    let mut owned = HashSet::new();
    let mut names = HashSet::new();
    for (index, entry) in directory.chunks_exact(16).enumerate() {
        if entry[0] == 0xFF {
            break;
        }
        if entry[0] == 0 {
            continue;
        }
        if index == 0 && entry == hex_marker() {
            continue;
        }
        let name = decode_name(entry)?;
        if !names.insert(name.clone()) {
            return Err(format!("N88 目录存在重复文件名 {name:?}"));
        }
        let start_cluster = entry[10];
        let mut current = start_cluster;
        let mut chain = Vec::new();
        let final_sectors = loop {
            if usize::from(current) >= FAT_ENTRIES || !owned.insert(current) {
                return Err(format!("{name}: FAT 簇 {current:02X} 越界、共享或成环"));
            }
            chain.push(current);
            let next = fat[usize::from(current)];
            if (0xC1..=0xC8).contains(&next) {
                break next - 0xC0;
            }
            if next == 0xFE || next == 0xFF || usize::from(next) >= FAT_ENTRIES {
                return Err(format!("{name}: FAT 簇 {current:02X} 的值 {next:02X} 无效"));
            }
            current = next;
        };
        let ranges = file_ranges(clusters, &chain, final_sectors)?;
        files.push(DiskFile {
            directory_index: index,
            name,
            attribute: entry[9],
            start_cluster,
            clusters: chain,
            final_sectors,
            data: gather(source, &ranges),
        });
    }
    if files.is_empty() {
        return Err("N88 目录没有活动文件".into());
    }
    Ok(files)
}

fn file_ranges(
    clusters: &[Vec<Range<usize>>],
    chain: &[u8],
    final_sectors: u8,
) -> Result<Vec<Range<usize>>> {
    if chain.is_empty() || !(1..=8).contains(&final_sectors) {
        return Err("N88 文件簇链或尾簇扇区数无效".into());
    }
    let mut output = Vec::new();
    for (index, cluster) in chain.iter().copied().enumerate() {
        let source = clusters
            .get(usize::from(cluster))
            .ok_or_else(|| format!("FAT 簇 {cluster:02X} 越界"))?;
        if index + 1 == chain.len() {
            output.extend(slice_ranges(
                source,
                0,
                usize::from(final_sectors) * SECTOR_SIZE,
            )?);
        } else {
            output.extend(source.iter().cloned());
        }
    }
    Ok(output)
}

fn decode_name(entry: &[u8]) -> Result<String> {
    let raw = &entry[..9];
    let end = raw
        .iter()
        .rposition(|byte| *byte != b' ')
        .map_or(0, |position| position + 1);
    let raw = &raw[..end];
    if raw.is_empty()
        || raw
            .iter()
            .any(|byte| !(0x20..=0x7E).contains(byte) || matches!(*byte, b'/' | b'\\' | b':'))
    {
        return Err(format!("N88 文件名无效：{}", hex(raw)));
    }
    std::str::from_utf8(raw)
        .map(str::to_owned)
        .map_err(|error| format!("N88 文件名不是 ASCII：{error}"))
}

fn gather(source: &[u8], ranges: &[Range<usize>]) -> Vec<u8> {
    let mut output = Vec::with_capacity(ranges.iter().map(Range::len).sum());
    for range in ranges {
        output.extend_from_slice(&source[range.clone()]);
    }
    output
}

fn write_ranges(output: &mut [u8], ranges: &[Range<usize>], data: &[u8]) -> Result<()> {
    let expected = ranges.iter().map(Range::len).sum::<usize>();
    if expected != data.len() {
        return Err(format!(
            "写入范围共 {expected} 字节，但替换内容为 {} 字节",
            data.len()
        ));
    }
    let mut cursor = 0usize;
    for range in ranges {
        output[range.clone()].copy_from_slice(&data[cursor..cursor + range.len()]);
        cursor += range.len();
    }
    Ok(())
}

fn write_fat(output: &mut [u8], ranges: &[Range<usize>], cluster: u8, value: u8) {
    for range in ranges {
        output[range.start + usize::from(cluster)] = value;
    }
}

fn write_gathered_byte(
    output: &mut [u8],
    ranges: &[Range<usize>],
    logical_offset: usize,
    value: u8,
) -> Result<()> {
    let mut remaining = logical_offset;
    for range in ranges {
        if remaining < range.len() {
            output[range.start + remaining] = value;
            return Ok(());
        }
        remaining -= range.len();
    }
    Err(format!("目录逻辑偏移 0x{logical_offset:X} 越界"))
}

fn find_directory_slots(
    source: &[u8],
    ranges: &[Range<usize>],
    count: usize,
) -> Result<Vec<usize>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    let directory = gather(source, ranges);
    let entries = directory.chunks_exact(16).collect::<Vec<_>>();
    let mut slots = entries
        .iter()
        .enumerate()
        .take_while(|(_, entry)| entry[0] != 0xFF)
        .filter_map(|(index, entry)| (entry[0] == 0).then_some(index))
        .take(count)
        .collect::<Vec<_>>();
    if slots.len() == count {
        return Ok(slots);
    }
    let terminator = directory_terminator(source, ranges)?;
    let mut index = terminator;
    while slots.len() < count {
        if index + 1 >= entries.len() {
            return Err("N88 目录没有足够的新增文件项空间".into());
        }
        if entries[index][0] != 0 && entries[index][0] != 0xFF {
            return Err(format!("N88 目录结束标记后的项 {index} 非空"));
        }
        slots.push(index);
        index += 1;
    }
    Ok(slots)
}

fn directory_terminator(source: &[u8], ranges: &[Range<usize>]) -> Result<usize> {
    gather(source, ranges)
        .chunks_exact(16)
        .position(|entry| entry[0] == 0xFF)
        .ok_or_else(|| "N88 目录没有结束标记，无法新增文件".into())
}

fn write_directory_entry(
    output: &mut [u8],
    ranges: &[Range<usize>],
    index: usize,
    name: &str,
    start_cluster: u8,
) -> Result<()> {
    validate_new_name(name)?;
    let mut entry = [0u8; 16];
    entry[..9].fill(b' ');
    entry[..name.len()].copy_from_slice(name.as_bytes());
    entry[9] = 0x01;
    entry[10] = start_cluster;
    for (offset, value) in entry.into_iter().enumerate() {
        write_gathered_byte(output, ranges, index * 16 + offset, value)?;
    }
    Ok(())
}

fn validate_new_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 9 || name.ends_with(' ') {
        return Err(format!("新增 N88 文件名长度无效：{name:?}"));
    }
    if name
        .bytes()
        .any(|byte| !(0x20..=0x7E).contains(&byte) || matches!(byte, b'/' | b'\\' | b':'))
    {
        return Err(format!("新增 N88 文件名含非法字符：{name:?}"));
    }
    Ok(())
}

fn hex_marker() -> &'static [u8] {
    &[
        0x0C, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x0C, 0x00, 0x00, 0x12, 0x34, 0x56, 0x78,
        0x90,
    ]
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}
