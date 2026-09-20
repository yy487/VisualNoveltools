use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::ops::Range;
use std::path::{Component, Path, PathBuf};

pub mod mes;
pub mod workflow;

pub const WORKSPACE_FORMAT: &str = "shinjuku-d88-fat12-unpack-workspace-v1";
const D88_MIN_HEADER_SIZE: usize = 0x2B0;
const TRACK_TABLE_OFFSET: usize = 0x20;
const TRACK_TABLE_ENTRIES: usize = 164;
const SECTOR_HEADER_SIZE: usize = 16;

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D88Info {
    pub disk_name: Option<String>,
    pub disk_name_raw_hex: String,
    pub write_protected: bool,
    pub media_code: u8,
    pub declared_size: usize,
    pub header_size: usize,
    pub populated_tracks: usize,
    pub cylinders: usize,
    pub heads: usize,
    pub sectors_per_track: usize,
    pub sector_size: usize,
    pub logical_sectors: usize,
    pub logical_bytes: usize,
    pub container_roundtrip_exact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fat12Info {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub fat_copies: u8,
    pub root_entries: u16,
    pub total_sectors: u32,
    pub media_descriptor: u8,
    pub sectors_per_fat: u16,
    pub sectors_per_track: u16,
    pub heads: u16,
    pub root_directory_sectors: u32,
    pub first_data_sector: u32,
    pub data_clusters: u32,
    pub fat_copies_identical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryManifest {
    pub path: String,
    pub raw_short_name_hex: String,
    pub attributes: u8,
    pub directory_entry_logical_offset: u64,
    pub start_cluster: u16,
    pub cluster_chain: Vec<u16>,
    pub dos_time: u16,
    pub dos_date: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManifest {
    pub path: String,
    pub raw_short_name_hex: String,
    pub attributes: u8,
    pub directory_entry_logical_offset: u64,
    pub start_cluster: u16,
    pub size: u32,
    pub cluster_chain: Vec<u16>,
    pub dos_time: u16,
    pub dos_date: u16,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskManifest {
    pub source_file: String,
    pub source_sha256: String,
    pub logical_sha256: String,
    pub output_dir: String,
    pub d88: D88Info,
    pub fat12: Fat12Info,
    pub volume_labels: Vec<String>,
    pub directories: Vec<DirectoryManifest>,
    pub files: Vec<FileManifest>,
    pub skipped_deleted_entries: u64,
    pub skipped_lfn_entries: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RolePaths {
    unpacked_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    _format: String,
    tool_version: String,
    role_paths: RolePaths,
    pub disks: Vec<DiskManifest>,
}

#[derive(Debug, Clone)]
struct ExtractedFile {
    relative_path: String,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
struct ParsedDisk {
    source_file: String,
    source_sha256: String,
    logical_sha256: String,
    d88: D88Info,
    fat12: Fat12Info,
    volume_labels: Vec<String>,
    directories: Vec<DirectoryManifest>,
    files: Vec<FileManifest>,
    extracted_files: Vec<ExtractedFile>,
    skipped_deleted_entries: u64,
    skipped_lfn_entries: u64,
}

#[derive(Debug, Clone)]
struct PreparedDisk {
    output_dir: String,
    parsed: ParsedDisk,
}

#[derive(Debug, Clone, Copy)]
pub struct PreparationSummary {
    pub images: usize,
    pub files: usize,
    pub directories: usize,
    pub file_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct PreparedBatch {
    input_paths: Vec<PathBuf>,
    disks: Vec<PreparedDisk>,
    summary: PreparationSummary,
}

#[derive(Debug, Clone)]
pub struct DiskSummary {
    pub source_file: String,
    pub files: usize,
    pub directories: usize,
    pub file_bytes: u64,
    pub tracks: usize,
    pub sectors: usize,
}

impl PreparedBatch {
    pub fn input_paths(&self) -> &[PathBuf] {
        &self.input_paths
    }

    pub fn summary(&self) -> PreparationSummary {
        self.summary
    }

    pub fn disk_summaries(&self) -> Vec<DiskSummary> {
        self.disks
            .iter()
            .map(|disk| DiskSummary {
                source_file: disk.parsed.source_file.clone(),
                files: disk.parsed.files.len(),
                directories: disk.parsed.directories.len(),
                file_bytes: disk
                    .parsed
                    .files
                    .iter()
                    .map(|file| u64::from(file.size))
                    .sum(),
                tracks: disk.parsed.d88.populated_tracks,
                sectors: disk.parsed.d88.logical_sectors,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct UnpackReport {
    pub extracted_files: usize,
    pub extracted_directories: usize,
    pub extracted_bytes: u64,
    pub warnings: Vec<String>,
    pub output_root: PathBuf,
}

#[derive(Debug, Clone)]
struct RawDirEntry {
    bytes: [u8; 32],
    logical_offset: usize,
}

struct FatParser<'a> {
    logical: &'a [u8],
    fat: &'a [u8],
    info: Fat12Info,
    max_cluster: u16,
    cluster_bytes: usize,
    owners: HashMap<u16, String>,
    seen_paths: HashSet<String>,
    volume_labels: Vec<String>,
    directories: Vec<DirectoryManifest>,
    files: Vec<FileManifest>,
    extracted_files: Vec<ExtractedFile>,
    skipped_deleted_entries: u64,
    skipped_lfn_entries: u64,
}

#[derive(Debug)]
struct D88Parsed {
    info: D88Info,
    logical: Vec<u8>,
    data_ranges: Vec<Range<usize>>,
}

pub fn prepare_batch(selected: &[PathBuf]) -> Result<PreparedBatch> {
    let inputs = discover_d88_inputs(selected)?;
    let mut output_names = HashSet::new();
    let mut disks = Vec::with_capacity(inputs.len());
    for input in &inputs {
        let stem = input
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("输入文件名无法表示为 Unicode: {}", input.display()))?;
        validate_output_segment(stem)?;
        if !output_names.insert(stem.to_uppercase()) {
            return Err(format!("多个 D88 会映射到同一个输出目录: {stem}"));
        }
        let bytes = fs::read(input).map_err(|e| format!("无法读取 {}: {e}", input.display()))?;
        let parsed = parse_disk(input, &bytes)?;
        disks.push(PreparedDisk {
            output_dir: stem.to_string(),
            parsed,
        });
    }
    let summary = PreparationSummary {
        images: disks.len(),
        files: disks.iter().map(|disk| disk.parsed.files.len()).sum(),
        directories: disks.iter().map(|disk| disk.parsed.directories.len()).sum(),
        file_bytes: disks
            .iter()
            .flat_map(|disk| &disk.parsed.files)
            .map(|file| u64::from(file.size))
            .sum(),
    };
    Ok(PreparedBatch {
        input_paths: inputs,
        disks,
        summary,
    })
}

fn discover_d88_inputs(selected: &[PathBuf]) -> Result<Vec<PathBuf>> {
    if selected.is_empty() {
        return Err("至少需要一个 D88 文件或目录".into());
    }
    let mut found = Vec::new();
    for path in selected {
        let canonical =
            fs::canonicalize(path).map_err(|e| format!("无法解析输入 {}: {e}", path.display()))?;
        if canonical.is_file() {
            if !has_d88_extension(&canonical) {
                return Err(format!("输入文件不是 D88: {}", canonical.display()));
            }
            found.push(canonical);
        } else if canonical.is_dir() {
            let mut children = fs::read_dir(&canonical)
                .map_err(|e| format!("无法读取输入目录 {}: {e}", canonical.display()))?
                .map(|entry| entry.map(|item| item.path()))
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| format!("无法枚举输入目录 {}: {e}", canonical.display()))?;
            children.sort_by_key(|path| path.file_name().map(|name| name.to_os_string()));
            found.extend(
                children
                    .into_iter()
                    .filter(|path| path.is_file() && has_d88_extension(path)),
            );
        } else {
            return Err(format!("输入既不是文件也不是目录: {}", canonical.display()));
        }
    }
    if found.is_empty() {
        return Err("所选路径中没有 D88 文件".into());
    }
    let mut seen = HashSet::new();
    found.retain(|path| seen.insert(path.to_string_lossy().to_uppercase()));
    Ok(found)
}

fn has_d88_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("d88"))
}

fn parse_disk(path: &Path, bytes: &[u8]) -> Result<ParsedDisk> {
    let d88 = parse_d88(path, bytes)?;
    let logical_sha256 = sha256_hex(&d88.logical);
    let (fat12, parser) = parse_fat12(path, &d88.logical)?;
    let source_file = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("输入文件名无法表示为 Unicode: {}", path.display()))?
        .to_string();
    Ok(ParsedDisk {
        source_file,
        source_sha256: sha256_hex(bytes),
        logical_sha256,
        d88: d88.info,
        fat12,
        volume_labels: parser.volume_labels,
        directories: parser.directories,
        files: parser.files,
        extracted_files: parser.extracted_files,
        skipped_deleted_entries: parser.skipped_deleted_entries,
        skipped_lfn_entries: parser.skipped_lfn_entries,
    })
}

fn parse_d88(path: &Path, bytes: &[u8]) -> Result<D88Parsed> {
    if bytes.len() < D88_MIN_HEADER_SIZE {
        return Err(format!("{}: D88 头被截断", path.display()));
    }
    let declared_size =
        usize::try_from(read_u32(bytes, 0x1C)?).map_err(|_| "D88 声明大小过大".to_string())?;
    if declared_size != bytes.len() {
        return Err(format!(
            "{}: D88 声明大小 0x{declared_size:X} 与实际 0x{:X} 不一致",
            path.display(),
            bytes.len()
        ));
    }
    let mut track_offsets = Vec::new();
    for slot in 0..TRACK_TABLE_ENTRIES {
        let offset = usize::try_from(read_u32(bytes, TRACK_TABLE_OFFSET + slot * 4)?)
            .map_err(|_| "D88 轨道偏移过大".to_string())?;
        if offset != 0 {
            track_offsets.push((slot, offset));
        }
    }
    if track_offsets.is_empty() {
        return Err(format!("{}: D88 没有活动轨道", path.display()));
    }
    let header_size = track_offsets[0].1;
    if header_size < D88_MIN_HEADER_SIZE {
        return Err(format!("{}: D88 首轨位于头部之内", path.display()));
    }
    for pair in track_offsets.windows(2) {
        if pair[0].1 >= pair[1].1 || pair[1].1 >= bytes.len() {
            return Err(format!("{}: D88 轨道偏移无效或未递增", path.display()));
        }
    }

    let mut logical = Vec::new();
    let mut data_ranges: Vec<Range<usize>> = Vec::new();
    let mut common_count = None;
    let mut common_size = None;
    let mut max_cylinder = 0usize;
    let mut heads = HashSet::new();
    for (track_index, (slot, start)) in track_offsets.iter().copied().enumerate() {
        let end = track_offsets
            .get(track_index + 1)
            .map(|(_, offset)| *offset)
            .unwrap_or(bytes.len());
        if start + SECTOR_HEADER_SIZE > end {
            return Err(format!("{}: D88 轨道 {slot} 被截断", path.display()));
        }
        let expected_count = usize::from(read_u16(bytes, start + 4)?);
        if expected_count == 0 || expected_count > 64 {
            return Err(format!("{}: D88 轨道 {slot} 扇区数无效", path.display()));
        }
        if common_count
            .replace(expected_count)
            .is_some_and(|old| old != expected_count)
        {
            return Err(format!("{}: D88 每轨扇区数不一致", path.display()));
        }
        let mut cursor = start;
        for ordinal in 0..expected_count {
            if cursor + SECTOR_HEADER_SIZE > end {
                return Err(format!("{}: D88 轨道 {slot} 扇区头被截断", path.display()));
            }
            let cylinder = bytes[cursor];
            let head = bytes[cursor + 1];
            let sector_id = bytes[cursor + 2];
            let size_code = bytes[cursor + 3];
            let count = usize::from(read_u16(bytes, cursor + 4)?);
            let data_size = usize::from(read_u16(bytes, cursor + 14)?);
            let nominal = 128usize
                .checked_shl(u32::from(size_code))
                .ok_or_else(|| format!("{}: D88 扇区大小码无效", path.display()))?;
            if count != expected_count || data_size != nominal {
                return Err(format!(
                    "{}: D88 轨道 {slot} 扇区 {} 的计数或大小不一致",
                    path.display(),
                    ordinal + 1
                ));
            }
            if cylinder != (slot / 2) as u8
                || head != (slot % 2) as u8
                || sector_id != (ordinal + 1) as u8
            {
                return Err(format!(
                    "{}: D88 轨道 {slot} 的 CHR 顺序不符合线性 FAT 布局",
                    path.display()
                ));
            }
            if common_size
                .replace(data_size)
                .is_some_and(|old| old != data_size)
            {
                return Err(format!("{}: D88 扇区大小不一致", path.display()));
            }
            let data_start = cursor + SECTOR_HEADER_SIZE;
            let data_end = data_start
                .checked_add(data_size)
                .ok_or_else(|| "D88 扇区范围溢出".to_string())?;
            if data_end > end {
                return Err(format!("{}: D88 扇区越过轨道边界", path.display()));
            }
            logical.extend_from_slice(&bytes[data_start..data_end]);
            data_ranges.push(data_start..data_end);
            cursor = data_end;
            max_cylinder = max_cylinder.max(usize::from(cylinder));
            heads.insert(head);
        }
        if cursor != end {
            return Err(format!(
                "{}: D88 轨道 {slot} 尾部含未解释字节",
                path.display()
            ));
        }
    }
    let mut rebuilt = bytes.to_vec();
    let mut logical_cursor = 0usize;
    for range in &data_ranges {
        let next = logical_cursor + range.len();
        rebuilt[range.clone()].copy_from_slice(&logical[logical_cursor..next]);
        logical_cursor = next;
    }
    let raw_name = &bytes[..17];
    let name_end = raw_name.iter().position(|byte| *byte == 0).unwrap_or(17);
    let disk_name = if name_end == 0 {
        None
    } else {
        Some(decode_cp932_exact(&raw_name[..name_end], "D88 磁盘名")?)
    };
    Ok(D88Parsed {
        info: D88Info {
            disk_name,
            disk_name_raw_hex: hex_upper(raw_name),
            write_protected: bytes[0x1A] != 0,
            media_code: bytes[0x1B],
            declared_size,
            header_size,
            populated_tracks: track_offsets.len(),
            cylinders: max_cylinder + 1,
            heads: heads.len(),
            sectors_per_track: common_count.unwrap_or(0),
            sector_size: common_size.unwrap_or(0),
            logical_sectors: data_ranges.len(),
            logical_bytes: logical.len(),
            container_roundtrip_exact: rebuilt == bytes,
        },
        logical,
        data_ranges,
    })
}

fn parse_fat12<'a>(path: &Path, logical: &'a [u8]) -> Result<(Fat12Info, FatParser<'a>)> {
    if logical.len() < 64 {
        return Err(format!("{}: D88 逻辑数据不足以容纳 BPB", path.display()));
    }
    let bytes_per_sector = read_u16(logical, 11)?;
    let sectors_per_cluster = logical[13];
    let reserved_sectors = read_u16(logical, 14)?;
    let fat_copies = logical[16];
    let root_entries = read_u16(logical, 17)?;
    let total_sectors_16 = read_u16(logical, 19)?;
    let media_descriptor = logical[21];
    let sectors_per_fat = read_u16(logical, 22)?;
    let sectors_per_track = read_u16(logical, 24)?;
    let heads = read_u16(logical, 26)?;
    let total_sectors = if total_sectors_16 != 0 {
        u32::from(total_sectors_16)
    } else {
        read_u32(logical, 32)?
    };
    if !(128..=4096).contains(&bytes_per_sector) || !bytes_per_sector.is_power_of_two() {
        return Err(format!("{}: FAT BPB 扇区大小无效", path.display()));
    }
    if sectors_per_cluster == 0 || !sectors_per_cluster.is_power_of_two() {
        return Err(format!("{}: FAT BPB 每簇扇区数无效", path.display()));
    }
    if reserved_sectors == 0 || fat_copies == 0 || root_entries == 0 || sectors_per_fat == 0 {
        return Err(format!("{}: FAT12 BPB 必需字段为零", path.display()));
    }
    let expected_bytes = u64::from(total_sectors) * u64::from(bytes_per_sector);
    if expected_bytes != logical.len() as u64 {
        return Err(format!(
            "{}: FAT BPB 容量 0x{expected_bytes:X} 与 D88 逻辑容量 0x{:X} 不一致",
            path.display(),
            logical.len()
        ));
    }
    let root_directory_sectors =
        (u32::from(root_entries) * 32).div_ceil(u32::from(bytes_per_sector));
    let first_root_sector = u32::from(reserved_sectors)
        .checked_add(u32::from(fat_copies) * u32::from(sectors_per_fat))
        .ok_or_else(|| "FAT 根目录偏移溢出".to_string())?;
    let first_data_sector = first_root_sector
        .checked_add(root_directory_sectors)
        .ok_or_else(|| "FAT 数据区偏移溢出".to_string())?;
    if first_data_sector >= total_sectors {
        return Err(format!("{}: FAT 数据区超出磁盘", path.display()));
    }
    let data_clusters = (total_sectors - first_data_sector) / u32::from(sectors_per_cluster);
    if data_clusters == 0 || data_clusters >= 4085 {
        return Err(format!("{}: 数据簇数量不属于 FAT12", path.display()));
    }
    let max_cluster =
        u16::try_from(data_clusters + 1).map_err(|_| "FAT12 最大簇号溢出".to_string())?;
    let bps = usize::from(bytes_per_sector);
    let fat_bytes = usize::from(sectors_per_fat)
        .checked_mul(bps)
        .ok_or_else(|| "FAT 字节数溢出".to_string())?;
    let fat_offset = usize::from(reserved_sectors)
        .checked_mul(bps)
        .ok_or_else(|| "FAT 偏移溢出".to_string())?;
    let fat_end = fat_offset
        .checked_add(fat_bytes)
        .ok_or_else(|| "FAT 范围溢出".to_string())?;
    let fat = logical
        .get(fat_offset..fat_end)
        .ok_or_else(|| format!("{}: 第一份 FAT 被截断", path.display()))?;
    if fat.len() < 3 || fat[0] != media_descriptor || fat[1] != 0xFF || fat[2] & 0x0F != 0x0F {
        return Err(format!("{}: FAT12 保留项或介质字节无效", path.display()));
    }
    if (fat_bytes * 2) / 3 <= usize::from(max_cluster) {
        return Err(format!("{}: FAT 表容量不足", path.display()));
    }
    let mut fat_copies_identical = true;
    for copy in 1..usize::from(fat_copies) {
        let start = fat_offset + copy * fat_bytes;
        let end = start + fat_bytes;
        let candidate = logical
            .get(start..end)
            .ok_or_else(|| format!("{}: FAT 副本被截断", path.display()))?;
        if candidate != fat {
            fat_copies_identical = false;
        }
    }
    let info = Fat12Info {
        bytes_per_sector,
        sectors_per_cluster,
        reserved_sectors,
        fat_copies,
        root_entries,
        total_sectors,
        media_descriptor,
        sectors_per_fat,
        sectors_per_track,
        heads,
        root_directory_sectors,
        first_data_sector,
        data_clusters,
        fat_copies_identical,
    };
    let cluster_bytes = bps * usize::from(sectors_per_cluster);
    let mut parser = FatParser {
        logical,
        fat,
        info: info.clone(),
        max_cluster,
        cluster_bytes,
        owners: HashMap::new(),
        seen_paths: HashSet::new(),
        volume_labels: Vec::new(),
        directories: Vec::new(),
        files: Vec::new(),
        extracted_files: Vec::new(),
        skipped_deleted_entries: 0,
        skipped_lfn_entries: 0,
    };
    let root_offset =
        usize::try_from(first_root_sector).map_err(|_| "根目录偏移过大".to_string())? * bps;
    let mut root = Vec::with_capacity(usize::from(root_entries));
    for index in 0..usize::from(root_entries) {
        root.push(read_raw_dir_entry(logical, root_offset + index * 32)?);
    }
    parser.parse_directory(root, "", 0)?;
    Ok((info, parser))
}

impl FatParser<'_> {
    fn parse_directory(
        &mut self,
        entries: Vec<RawDirEntry>,
        prefix: &str,
        depth: usize,
    ) -> Result<()> {
        if depth > 64 {
            return Err(format!("目录嵌套超过限制: {prefix}"));
        }
        for raw in entries {
            let entry = &raw.bytes;
            match entry[0] {
                0x00 => break,
                0xE5 => {
                    self.skipped_deleted_entries += 1;
                    continue;
                }
                _ => {}
            }
            let attributes = entry[11];
            if attributes & 0x0F == 0x0F {
                self.skipped_lfn_entries += 1;
                continue;
            }
            let (name, raw_short_name_hex) = decode_short_name(entry)?;
            if name == "." || name == ".." {
                continue;
            }
            if attributes & 0x08 != 0 {
                self.volume_labels.push(name);
                continue;
            }
            validate_output_segment(&name)?;
            let relative_path = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            if !self.seen_paths.insert(relative_path.to_uppercase()) {
                return Err(format!("输出路径发生大小写不敏感冲突: {relative_path}"));
            }
            let start_cluster = read_u16(entry, 26)?;
            let size = read_u32(entry, 28)?;
            let dos_time = read_u16(entry, 22)?;
            let dos_date = read_u16(entry, 24)?;
            if attributes & 0x10 != 0 {
                if start_cluster < 2 {
                    return Err(format!("目录 {relative_path} 起始簇无效"));
                }
                let chain = self.read_chain(start_cluster, &format!("{relative_path} [dir]"))?;
                let children = self.directory_entries_from_chain(&chain)?;
                self.directories.push(DirectoryManifest {
                    path: relative_path.clone(),
                    raw_short_name_hex,
                    attributes,
                    directory_entry_logical_offset: raw.logical_offset as u64,
                    start_cluster,
                    cluster_chain: chain,
                    dos_time,
                    dos_date,
                });
                self.parse_directory(children, &relative_path, depth + 1)?;
            } else {
                let chain = if size == 0 {
                    if start_cluster != 0 {
                        return Err(format!("空文件 {relative_path} 使用非零起始簇"));
                    }
                    Vec::new()
                } else {
                    if start_cluster < 2 {
                        return Err(format!("文件 {relative_path} 起始簇无效"));
                    }
                    self.read_chain(start_cluster, &format!("{relative_path} [file]"))?
                };
                let needed = if size == 0 {
                    0
                } else {
                    usize::try_from(u64::from(size).div_ceil(self.cluster_bytes as u64))
                        .map_err(|_| format!("文件 {relative_path} 大小过大"))?
                };
                if chain.len() != needed {
                    return Err(format!(
                        "文件 {relative_path} 需要 {needed} 簇，FAT 链实际为 {} 簇",
                        chain.len()
                    ));
                }
                let data = self.read_file_data(&chain, size, &relative_path)?;
                self.files.push(FileManifest {
                    path: relative_path.clone(),
                    raw_short_name_hex,
                    attributes,
                    directory_entry_logical_offset: raw.logical_offset as u64,
                    start_cluster,
                    size,
                    cluster_chain: chain,
                    dos_time,
                    dos_date,
                    sha256: sha256_hex(&data),
                });
                self.extracted_files.push(ExtractedFile {
                    relative_path,
                    data,
                });
            }
        }
        Ok(())
    }

    fn read_chain(&mut self, start: u16, owner: &str) -> Result<Vec<u16>> {
        let mut chain = Vec::new();
        let mut local_seen = HashSet::new();
        let mut cluster = start;
        loop {
            if cluster < 2 || cluster > self.max_cluster {
                return Err(format!("{owner}: 簇 {cluster} 超出范围"));
            }
            if !local_seen.insert(cluster) {
                return Err(format!("{owner}: FAT 链在簇 {cluster} 形成循环"));
            }
            if let Some(previous) = self.owners.insert(cluster, owner.to_string()) {
                return Err(format!("{owner}: 簇 {cluster} 与 {previous} 交叉链接"));
            }
            chain.push(cluster);
            let next = fat12_next(self.fat, cluster)?;
            match next {
                0xFF8..=0xFFF => break,
                0xFF7 => return Err(format!("{owner}: FAT 链遇到坏簇")),
                0xFF0..=0xFF6 => return Err(format!("{owner}: FAT 链遇到保留值")),
                0 | 1 => return Err(format!("{owner}: FAT 链意外终止")),
                _ => cluster = next,
            }
        }
        Ok(chain)
    }

    fn directory_entries_from_chain(&self, chain: &[u16]) -> Result<Vec<RawDirEntry>> {
        let mut entries = Vec::new();
        for &cluster in chain {
            let offset = self.cluster_offset(cluster)?;
            for local in (0..self.cluster_bytes).step_by(32) {
                entries.push(read_raw_dir_entry(self.logical, offset + local)?);
            }
        }
        Ok(entries)
    }

    fn read_file_data(&self, chain: &[u16], size: u32, owner: &str) -> Result<Vec<u8>> {
        let mut data = Vec::with_capacity(chain.len() * self.cluster_bytes);
        for &cluster in chain {
            let offset = self.cluster_offset(cluster)?;
            data.extend_from_slice(&self.logical[offset..offset + self.cluster_bytes]);
        }
        let size = usize::try_from(size).map_err(|_| format!("{owner}: 文件大小过大"))?;
        if size > data.len() {
            return Err(format!("{owner}: FAT 链容量小于文件大小"));
        }
        data.truncate(size);
        Ok(data)
    }

    fn cluster_offset(&self, cluster: u16) -> Result<usize> {
        if cluster < 2 || cluster > self.max_cluster {
            return Err(format!("簇 {cluster} 超出范围"));
        }
        let sector = u32::from(cluster - 2) * u32::from(self.info.sectors_per_cluster)
            + self.info.first_data_sector;
        let offset = usize::try_from(sector).map_err(|_| "簇扇区偏移过大".to_string())?
            * usize::from(self.info.bytes_per_sector);
        if offset + self.cluster_bytes > self.logical.len() {
            return Err(format!("簇 {cluster} 超出逻辑磁盘"));
        }
        Ok(offset)
    }
}

/// Rebuild one D88 image after replacing concrete FAT12 files. Unchanged files
/// and all non-filesystem container bytes are preserved byte-for-byte.
pub fn rebuild_d88(
    source_path: &Path,
    source_bytes: &[u8],
    replacements: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<u8>> {
    let d88 = parse_d88(source_path, source_bytes)?;
    let (info, parser) = parse_fat12(source_path, &d88.logical)?;
    let files = parser.files.clone();
    drop(parser);

    let mut by_path = HashMap::new();
    for file in &files {
        by_path.insert(file.path.to_uppercase(), file);
    }
    let mut changed = Vec::new();
    for (path, data) in replacements {
        let file = by_path
            .get(&path.to_uppercase())
            .copied()
            .ok_or_else(|| format!("{} 中没有 FAT12 文件 {path}", source_path.display()))?;
        if sha256_hex(data) != file.sha256 {
            changed.push((path.as_str(), file, data.as_slice()));
        }
    }
    if changed.is_empty() {
        return Ok(source_bytes.to_vec());
    }

    let bps = usize::from(info.bytes_per_sector);
    let cluster_bytes = bps * usize::from(info.sectors_per_cluster);
    let fat_bytes = usize::from(info.sectors_per_fat) * bps;
    let fat_offset = usize::from(info.reserved_sectors) * bps;
    let mut fat = d88.logical[fat_offset..fat_offset + fat_bytes].to_vec();
    let mut logical = d88.logical;

    for (_, file, _) in &changed {
        for &cluster in &file.cluster_chain {
            fat12_set(&mut fat, cluster, 0)?;
        }
    }
    let max_cluster =
        u16::try_from(info.data_clusters + 1).map_err(|_| "FAT12 最大簇号溢出".to_string())?;
    let mut free = (2..=max_cluster)
        .filter(|cluster| fat12_next(&fat, *cluster).is_ok_and(|value| value == 0))
        .collect::<Vec<_>>();
    free.sort_unstable();
    let required = changed
        .iter()
        .map(|(_, _, data)| data.len().div_ceil(cluster_bytes))
        .sum::<usize>();
    if required > free.len() {
        return Err(format!(
            "{} FAT12 空间不足：回注需要 {required} 簇，可用 {} 簇",
            source_path.display(),
            free.len()
        ));
    }

    let mut free_cursor = 0;
    for (path, file, data) in changed {
        let needed = data.len().div_ceil(cluster_bytes);
        let chain = free[free_cursor..free_cursor + needed].to_vec();
        free_cursor += needed;
        for (index, cluster) in chain.iter().copied().enumerate() {
            let next = chain.get(index + 1).copied().unwrap_or(0xfff);
            fat12_set(&mut fat, cluster, next)?;
            let sector = u32::from(cluster - 2) * u32::from(info.sectors_per_cluster)
                + info.first_data_sector;
            let offset =
                usize::try_from(sector).map_err(|_| format!("{path}: 簇扇区偏移过大"))? * bps;
            let target = logical
                .get_mut(offset..offset + cluster_bytes)
                .ok_or_else(|| format!("{path}: 簇 {cluster} 超出逻辑磁盘"))?;
            target.fill(0);
            let source_start = index * cluster_bytes;
            let source_end = (source_start + cluster_bytes).min(data.len());
            if source_start < source_end {
                target[..source_end - source_start]
                    .copy_from_slice(&data[source_start..source_end]);
            }
        }
        let entry_offset = usize::try_from(file.directory_entry_logical_offset)
            .map_err(|_| format!("{path}: 目录项偏移过大"))?;
        let entry = logical
            .get_mut(entry_offset..entry_offset + 32)
            .ok_or_else(|| format!("{path}: 目录项越界"))?;
        let old_cluster = u16::from_le_bytes([entry[26], entry[27]]);
        let old_size = u32::from_le_bytes([entry[28], entry[29], entry[30], entry[31]]);
        if old_cluster != file.start_cluster || old_size != file.size {
            return Err(format!("{path}: 目录项与解析快照不一致"));
        }
        let start_cluster = chain.first().copied().unwrap_or(0);
        entry[26..28].copy_from_slice(&start_cluster.to_le_bytes());
        let new_size = u32::try_from(data.len()).map_err(|_| format!("{path}: 文件超过 4 GiB"))?;
        entry[28..32].copy_from_slice(&new_size.to_le_bytes());
    }

    for copy in 0..usize::from(info.fat_copies) {
        let start = fat_offset + copy * fat_bytes;
        logical[start..start + fat_bytes].copy_from_slice(&fat);
    }
    let mut output = source_bytes.to_vec();
    let mut logical_offset = 0;
    for range in &d88.data_ranges {
        let end = logical_offset + range.len();
        output[range.clone()].copy_from_slice(&logical[logical_offset..end]);
        logical_offset = end;
    }
    if logical_offset != logical.len() {
        return Err("D88 扇区映射未覆盖完整逻辑磁盘".to_owned());
    }

    let verified = parse_disk(source_path, &output)?;
    for (path, data) in replacements {
        let expected = sha256_hex(data);
        let actual = verified
            .files
            .iter()
            .find(|file| file.path.eq_ignore_ascii_case(path))
            .ok_or_else(|| format!("回包验证找不到 {path}"))?;
        if actual.size as usize != data.len() || actual.sha256 != expected {
            return Err(format!("回包验证失败: {path}"));
        }
    }
    Ok(output)
}

fn fat12_set(fat: &mut [u8], cluster: u16, value: u16) -> Result<()> {
    if value > 0x0fff {
        return Err(format!("FAT12 值超出 12 位: 0x{value:X}"));
    }
    let offset = usize::from(cluster) * 3 / 2;
    let pair = fat
        .get_mut(offset..offset + 2)
        .ok_or_else(|| format!("FAT12 项 {cluster} 越界"))?;
    if cluster & 1 == 0 {
        pair[0] = value as u8;
        pair[1] = (pair[1] & 0xf0) | ((value >> 8) as u8 & 0x0f);
    } else {
        pair[0] = (pair[0] & 0x0f) | ((value << 4) as u8 & 0xf0);
        pair[1] = (value >> 4) as u8;
    }
    Ok(())
}

pub fn validate_output_target(output: &Path, overwrite: bool) -> Result<()> {
    if output.as_os_str().is_empty() {
        return Err("输出目录不能为空".into());
    }
    let mut normal = 0usize;
    for component in output.components() {
        match component {
            Component::ParentDir => return Err("输出目录不能包含 ..".into()),
            Component::Normal(_) => normal += 1,
            _ => {}
        }
    }
    if normal == 0 {
        return Err("拒绝把文件系统根目录作为输出".into());
    }
    if !output.exists() {
        return Ok(());
    }
    let metadata =
        fs::symlink_metadata(output).map_err(|e| format!("无法读取输出目录元数据: {e}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("输出路径必须是普通目录".into());
    }
    if !overwrite {
        return Err(format!("输出目录已存在；默认不覆盖: {}", output.display()));
    }
    if fs::read_dir(output)
        .map_err(|e| format!("无法读取输出目录: {e}"))?
        .next()
        .is_none()
    {
        return Ok(());
    }
    let manifest = fs::read(output.join("workspace.json"))
        .map_err(|_| "拒绝覆盖不属于本工具的非空目录".to_string())?;
    let value: serde_json::Value = serde_json::from_slice(&manifest)
        .map_err(|_| "现有 workspace.json 无效，拒绝覆盖".to_string())?;
    if value.get("_format").and_then(|item| item.as_str()) != Some(WORKSPACE_FORMAT) {
        return Err("现有输出不是本工具工作区，拒绝覆盖".into());
    }
    Ok(())
}

pub fn write_prepared(
    prepared: PreparedBatch,
    output_root: &Path,
    overwrite: bool,
) -> Result<UnpackReport> {
    validate_output_target(output_root, overwrite)?;
    let parent = output_root
        .parent()
        .ok_or_else(|| "输出目录缺少父目录".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("无法创建输出父目录 {}: {e}", parent.display()))?;
    let name = output_root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "输出目录名无法表示为 Unicode".to_string())?;
    let staging = unique_sibling(parent, &format!(".{name}.tmp-{}", std::process::id()))?;
    fs::create_dir(&staging).map_err(|e| format!("无法创建临时输出: {e}"))?;
    let result = (|| -> Result<UnpackReport> {
        let mut manifests = Vec::with_capacity(prepared.disks.len());
        let mut files = 0usize;
        let mut directories = 0usize;
        let mut bytes = 0u64;
        let mut warnings = Vec::new();
        for disk in &prepared.disks {
            let disk_root = staging.join(&disk.output_dir);
            fs::create_dir(&disk_root).map_err(|e| format!("无法创建磁盘目录: {e}"))?;
            let mut dirs: Vec<_> = disk.parsed.directories.iter().collect();
            dirs.sort_by_key(|item| item.path.matches('/').count());
            for directory in dirs {
                fs::create_dir_all(join_manifest_path(&disk_root, &directory.path)?)
                    .map_err(|e| format!("无法创建资源目录: {e}"))?;
            }
            for file in &disk.parsed.extracted_files {
                let target = join_manifest_path(&disk_root, &file.relative_path)?;
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent).map_err(|e| format!("无法创建文件父目录: {e}"))?;
                }
                let mut handle = fs::File::create(&target)
                    .map_err(|e| format!("无法创建 {}: {e}", target.display()))?;
                handle
                    .write_all(&file.data)
                    .map_err(|e| format!("无法写入 {}: {e}", target.display()))?;
                files += 1;
                bytes += file.data.len() as u64;
            }
            directories += disk.parsed.directories.len();
            if !disk.parsed.fat12.fat_copies_identical {
                warnings.push(format!(
                    "{}: FAT 副本不一致；文件按第一份 FAT 提取",
                    disk.parsed.source_file
                ));
            }
            manifests.push(DiskManifest {
                source_file: disk.parsed.source_file.clone(),
                source_sha256: disk.parsed.source_sha256.clone(),
                logical_sha256: disk.parsed.logical_sha256.clone(),
                output_dir: disk.output_dir.clone(),
                d88: disk.parsed.d88.clone(),
                fat12: disk.parsed.fat12.clone(),
                volume_labels: disk.parsed.volume_labels.clone(),
                directories: disk.parsed.directories.clone(),
                files: disk.parsed.files.clone(),
                skipped_deleted_entries: disk.parsed.skipped_deleted_entries,
                skipped_lfn_entries: disk.parsed.skipped_lfn_entries,
            });
        }
        let manifest = WorkspaceManifest {
            _format: WORKSPACE_FORMAT.into(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            role_paths: RolePaths {
                unpacked_root: ".".into(),
            },
            disks: manifests,
        };
        let mut json = serde_json::to_vec_pretty(&manifest)
            .map_err(|e| format!("无法序列化 workspace.json: {e}"))?;
        json.push(b'\n');
        fs::write(staging.join("workspace.json"), json)
            .map_err(|e| format!("无法写入 workspace.json: {e}"))?;
        commit_staging(&staging, output_root, overwrite)?;
        Ok(UnpackReport {
            extracted_files: files,
            extracted_directories: directories,
            extracted_bytes: bytes,
            warnings,
            output_root: output_root.to_path_buf(),
        })
    })();
    if result.is_err() && staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn commit_staging(staging: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if !output.exists() {
        return fs::rename(staging, output).map_err(|e| format!("无法提交输出: {e}"));
    }
    if !overwrite {
        return Err("输出已存在，默认不覆盖".into());
    }
    let parent = output
        .parent()
        .ok_or_else(|| "输出缺少父目录".to_string())?;
    let backup = unique_sibling(parent, &format!(".shinjuku-backup-{}", std::process::id()))?;
    fs::rename(output, &backup).map_err(|e| format!("无法暂存旧输出: {e}"))?;
    if let Err(error) = fs::rename(staging, output) {
        let rollback = fs::rename(&backup, output);
        return match rollback {
            Ok(()) => Err(format!("提交失败，已恢复旧输出: {error}")),
            Err(rollback_error) => Err(format!(
                "提交失败且旧输出恢复失败；备份在 {}: {error}; {rollback_error}",
                backup.display()
            )),
        };
    }
    fs::remove_dir_all(&backup).map_err(|e| format!("新输出已提交，但旧备份清理失败: {e}"))
}

fn decode_short_name(entry: &[u8; 32]) -> Result<(String, String)> {
    let raw_hex = hex_upper(&entry[..11]);
    let mut stem = entry[..8].to_vec();
    if stem.first() == Some(&0x05) {
        stem[0] = 0xE5;
    }
    while stem.last() == Some(&b' ') {
        stem.pop();
    }
    let mut extension = entry[8..11].to_vec();
    while extension.last() == Some(&b' ') {
        extension.pop();
    }
    let stem_text = decode_cp932_exact(&stem, &format!("目录短名 {raw_hex}"))?;
    let extension_text = decode_cp932_exact(&extension, &format!("目录扩展名 {raw_hex}"))?;
    let name = if extension_text.is_empty() {
        stem_text
    } else {
        format!("{stem_text}.{extension_text}")
    };
    Ok((name, raw_hex))
}

fn decode_cp932_exact(bytes: &[u8], context: &str) -> Result<String> {
    let text = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(bytes)
        .ok_or_else(|| format!("{context} 不是有效 CP932"))?;
    let (encoded, _, errors) = SHIFT_JIS.encode(&text);
    if errors || encoded.as_ref() != bytes {
        return Err(format!("{context} 无法按 CP932 字节往返"));
    }
    Ok(text.into_owned())
}

fn read_raw_dir_entry(bytes: &[u8], offset: usize) -> Result<RawDirEntry> {
    let slice = bytes
        .get(offset..offset + 32)
        .ok_or_else(|| format!("目录项在逻辑偏移 0x{offset:X} 被截断"))?;
    let mut entry = [0u8; 32];
    entry.copy_from_slice(slice);
    Ok(RawDirEntry {
        bytes: entry,
        logical_offset: offset,
    })
}

fn fat12_next(fat: &[u8], cluster: u16) -> Result<u16> {
    let offset = usize::from(cluster) * 3 / 2;
    let pair = fat
        .get(offset..offset + 2)
        .ok_or_else(|| format!("FAT12 项 {cluster} 越界"))?;
    let word = u16::from(pair[0]) | (u16::from(pair[1]) << 8);
    Ok(if cluster & 1 == 0 {
        word & 0x0FFF
    } else {
        word >> 4
    })
}

fn validate_output_segment(segment: &str) -> Result<()> {
    if segment.is_empty() || segment == "." || segment == ".." {
        return Err(format!("不安全的输出路径段: {segment:?}"));
    }
    if segment.ends_with(' ') || segment.ends_with('.') {
        return Err(format!("Windows 文件名不能以空格或点结尾: {segment}"));
    }
    if segment
        .chars()
        .any(|ch| ch < ' ' || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'))
    {
        return Err(format!("Windows 文件名含非法字符: {segment}"));
    }
    let stem = segment
        .split('.')
        .next()
        .unwrap_or(segment)
        .to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.as_bytes()[3].is_ascii_digit()
            && stem.as_bytes()[3] != b'0');
    if reserved {
        return Err(format!("Windows 保留设备名不能输出: {segment}"));
    }
    Ok(())
}

fn join_manifest_path(root: &Path, relative: &str) -> Result<PathBuf> {
    let mut path = root.to_path_buf();
    for segment in relative.split('/') {
        validate_output_segment(segment)?;
        path.push(segment);
    }
    Ok(path)
}

fn unique_sibling(parent: &Path, base: &str) -> Result<PathBuf> {
    for suffix in 0..1000u32 {
        let name = if suffix == 0 {
            base.to_string()
        } else {
            format!("{base}-{suffix}")
        };
        let candidate = parent.join(name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(format!("无法在 {} 分配临时目录", parent.display()))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let raw = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| format!("读取 u16 越界: 0x{offset:X}"))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| format!("读取 u32 越界: 0x{offset:X}"))?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex_upper(&Sha256::digest(bytes))
}

fn hex_upper(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut output = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0F)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn set_fat12(fat: &mut [u8], cluster: u16, value: u16) {
        let offset = usize::from(cluster) * 3 / 2;
        if cluster & 1 == 0 {
            fat[offset] = value as u8;
            fat[offset + 1] = (fat[offset + 1] & 0xF0) | ((value >> 8) as u8 & 0x0F);
        } else {
            fat[offset] = (fat[offset] & 0x0F) | ((value << 4) as u8 & 0xF0);
            fat[offset + 1] = (value >> 4) as u8;
        }
    }

    fn sample_d88() -> Vec<u8> {
        const BPS: usize = 1024;
        const SPT: usize = 4;
        const TRACKS: usize = 4;
        let mut logical = vec![0u8; BPS * SPT * TRACKS];
        logical[..3].copy_from_slice(&[0xEB, 0x1C, 0x90]);
        put_u16(&mut logical, 11, BPS as u16);
        logical[13] = 1;
        put_u16(&mut logical, 14, 1);
        logical[16] = 2;
        put_u16(&mut logical, 17, 32);
        put_u16(&mut logical, 19, (SPT * TRACKS) as u16);
        logical[21] = 0xFE;
        put_u16(&mut logical, 22, 1);
        put_u16(&mut logical, 24, SPT as u16);
        put_u16(&mut logical, 26, 2);
        let mut fat = vec![0u8; BPS];
        fat[..3].copy_from_slice(&[0xFE, 0xFF, 0xFF]);
        set_fat12(&mut fat, 2, 0xFFF);
        logical[BPS..BPS * 2].copy_from_slice(&fat);
        logical[BPS * 2..BPS * 3].copy_from_slice(&fat);
        let root = BPS * 3;
        logical[root..root + 11].copy_from_slice(b"HELLO   TXT");
        logical[root + 11] = 0x20;
        put_u16(&mut logical, root + 26, 2);
        put_u32(&mut logical, root + 28, 5);
        logical[BPS * 4..BPS * 4 + 5].copy_from_slice(b"hello");

        let track_bytes = SPT * (SECTOR_HEADER_SIZE + BPS);
        let mut d88 = vec![0u8; D88_MIN_HEADER_SIZE + TRACKS * track_bytes];
        for track in 0..TRACKS {
            let track_start = D88_MIN_HEADER_SIZE + track * track_bytes;
            put_u32(&mut d88, TRACK_TABLE_OFFSET + track * 4, track_start as u32);
            for sector in 0..SPT {
                let header = track_start + sector * (SECTOR_HEADER_SIZE + BPS);
                d88[header] = (track / 2) as u8;
                d88[header + 1] = (track % 2) as u8;
                d88[header + 2] = (sector + 1) as u8;
                d88[header + 3] = 3;
                put_u16(&mut d88, header + 4, SPT as u16);
                put_u16(&mut d88, header + 14, BPS as u16);
                let logical_start = (track * SPT + sector) * BPS;
                d88[header + SECTOR_HEADER_SIZE..header + SECTOR_HEADER_SIZE + BPS]
                    .copy_from_slice(&logical[logical_start..logical_start + BPS]);
            }
        }
        let size = d88.len() as u32;
        put_u32(&mut d88, 0x1C, size);
        d88
    }

    #[test]
    fn parses_d88_and_extracts_fat_file() {
        let bytes = sample_d88();
        let parsed = parse_disk(Path::new("sample.d88"), &bytes).expect("parse sample");
        assert_eq!(parsed.d88.logical_bytes, 16 * 1024);
        assert!(parsed.d88.container_roundtrip_exact);
        assert!(parsed.fat12.fat_copies_identical);
        assert_eq!(parsed.files.len(), 1);
        assert_eq!(parsed.files[0].path, "HELLO.TXT");
        assert_eq!(parsed.extracted_files[0].data, b"hello");
    }

    #[test]
    fn rejects_a_sector_slice_as_input() {
        let error = parse_disk(Path::new("slice.d88"), &[0u8; 1024]).unwrap_err();
        assert!(error.contains("D88 声明大小"));
    }

    #[test]
    fn rebuilds_a_growing_fat12_file_and_reparses_it() {
        let source = sample_d88();
        let replacement = vec![0x5a; 1500];
        let mut replacements = BTreeMap::new();
        replacements.insert("HELLO.TXT".to_owned(), replacement.clone());
        let rebuilt = rebuild_d88(Path::new("sample.d88"), &source, &replacements).unwrap();
        let parsed = parse_disk(Path::new("sample.d88"), &rebuilt).unwrap();
        let file = parsed
            .extracted_files
            .iter()
            .find(|file| file.relative_path == "HELLO.TXT")
            .unwrap();
        assert_eq!(file.data, replacement);
        assert_eq!(parsed.files[0].cluster_chain.len(), 2);
    }
}
