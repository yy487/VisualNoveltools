use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

pub const WORKSPACE_FORMAT: &str = "pc98-fdi-fat12-unpack-workspace-v1";

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize)]
pub struct FdiHeader {
    pub unknown_0x00: u32,
    pub disk_type: u32,
    pub data_offset: u32,
    pub data_size: u32,
    pub sector_size: u32,
    pub sectors_per_track: u32,
    pub heads: u32,
    pub cylinders: u32,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct DirectoryManifest {
    pub path: String,
    pub raw_short_name_hex: String,
    pub attributes: u8,
    pub directory_entry_offset: u64,
    pub start_cluster: u16,
    pub cluster_chain: Vec<u16>,
    pub dos_time: u16,
    pub dos_date: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileManifest {
    pub path: String,
    pub raw_short_name_hex: String,
    pub attributes: u8,
    pub directory_entry_offset: u64,
    pub start_cluster: u16,
    pub size: u32,
    pub cluster_chain: Vec<u16>,
    pub dos_time: u16,
    pub dos_date: u16,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArchiveManifest {
    pub source_file: String,
    pub source_sha256: String,
    pub output_dir: String,
    pub archive_bytes: u64,
    pub fdi: FdiHeader,
    pub fat12: Fat12Info,
    pub volume_labels: Vec<String>,
    pub directories: Vec<DirectoryManifest>,
    pub files: Vec<FileManifest>,
    pub skipped_deleted_entries: u64,
    pub skipped_lfn_entries: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePaths {
    pub unpacked_root: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceManifest {
    pub _format: String,
    pub tool_version: String,
    pub role_paths: RolePaths,
    pub archives: Vec<ArchiveManifest>,
}

#[derive(Debug)]
struct ExtractedFile {
    relative_path: String,
    data: Vec<u8>,
}

#[derive(Debug)]
struct ParsedDisk {
    source_file: String,
    source_sha256: String,
    archive_bytes: u64,
    fdi: FdiHeader,
    fat12: Fat12Info,
    volume_labels: Vec<String>,
    directories: Vec<DirectoryManifest>,
    files: Vec<FileManifest>,
    extracted_files: Vec<ExtractedFile>,
    skipped_deleted_entries: u64,
    skipped_lfn_entries: u64,
}

#[derive(Debug)]
struct PreparedDisk {
    output_dir: String,
    parsed: ParsedDisk,
}

#[derive(Debug, Clone)]
pub struct UnpackReport {
    pub images: usize,
    pub extracted_files: usize,
    pub extracted_directories: usize,
    pub extracted_bytes: u64,
    pub output_root: PathBuf,
}

#[derive(Debug, Clone)]
struct RawDirEntry {
    bytes: [u8; 32],
    archive_offset: usize,
}

struct FatParser<'a> {
    archive: &'a [u8],
    data_offset: usize,
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

pub fn inspect_fdi(path: &Path) -> Result<(FdiHeader, Fat12Info, usize, usize, u64)> {
    let bytes = fs::read(path).map_err(|e| format!("无法读取 {}: {e}", path.display()))?;
    let parsed = parse_fdi(path, &bytes)?;
    let total_bytes = parsed.files.iter().map(|entry| u64::from(entry.size)).sum();
    Ok((
        parsed.fdi,
        parsed.fat12,
        parsed.files.len(),
        parsed.directories.len(),
        total_bytes,
    ))
}

pub fn preview_mappings(inputs: &[PathBuf]) -> Result<Vec<(PathBuf, String)>> {
    if inputs.is_empty() {
        return Err("至少需要一个 FDI 输入文件".to_string());
    }
    let mut seen = HashSet::new();
    let mut mappings = Vec::with_capacity(inputs.len());
    for input in inputs {
        let file_stem = input
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("输入文件名无法表示为 Unicode: {}", input.display()))?;
        validate_output_segment(file_stem)?;
        let key = file_stem.to_uppercase();
        if !seen.insert(key) {
            return Err(format!("多个输入会映射到同一个输出目录: {file_stem}"));
        }
        mappings.push((input.clone(), file_stem.to_string()));
    }
    Ok(mappings)
}

pub fn unpack_batch(
    inputs: &[PathBuf],
    output_root: &Path,
    overwrite: bool,
) -> Result<UnpackReport> {
    validate_output_root(output_root)?;
    let mappings = preview_mappings(inputs)?;
    validate_output_does_not_contain_inputs(inputs, output_root)?;

    let mut prepared = Vec::with_capacity(inputs.len());
    for (input, output_dir) in mappings {
        let metadata = fs::metadata(&input)
            .map_err(|e| format!("无法读取输入元数据 {}: {e}", input.display()))?;
        if !metadata.is_file() {
            return Err(format!("输入不是普通文件: {}", input.display()));
        }
        let bytes = fs::read(&input).map_err(|e| format!("无法读取 {}: {e}", input.display()))?;
        let parsed = parse_fdi(&input, &bytes)?;
        prepared.push(PreparedDisk { output_dir, parsed });
    }

    validate_existing_output(output_root, overwrite)?;
    write_workspace(&prepared, output_root, overwrite)
}

fn parse_fdi(path: &Path, bytes: &[u8]) -> Result<ParsedDisk> {
    if bytes.len() < 36 {
        return Err(format!("{}: FDI 头被截断", path.display()));
    }
    let fdi = FdiHeader {
        unknown_0x00: read_u32(bytes, 0)?,
        disk_type: read_u32(bytes, 4)?,
        data_offset: read_u32(bytes, 8)?,
        data_size: read_u32(bytes, 12)?,
        sector_size: read_u32(bytes, 16)?,
        sectors_per_track: read_u32(bytes, 20)?,
        heads: read_u32(bytes, 24)?,
        cylinders: read_u32(bytes, 28)?,
    };

    let data_offset =
        usize::try_from(fdi.data_offset).map_err(|_| "FDI 数据偏移过大".to_string())?;
    let data_size = usize::try_from(fdi.data_size).map_err(|_| "FDI 数据大小过大".to_string())?;
    if data_offset < 32 {
        return Err(format!("{}: FDI 数据偏移小于头部大小", path.display()));
    }
    let data_end = data_offset
        .checked_add(data_size)
        .ok_or_else(|| format!("{}: FDI 数据范围溢出", path.display()))?;
    if data_end != bytes.len() {
        return Err(format!(
            "{}: FDI 长度不一致，头部声明结束于 0x{data_end:X}，实际为 0x{:X}",
            path.display(),
            bytes.len()
        ));
    }
    let geometry_size = checked_product(&[
        fdi.sector_size,
        fdi.sectors_per_track,
        fdi.heads,
        fdi.cylinders,
    ])?;
    if geometry_size != u64::from(fdi.data_size) {
        return Err(format!(
            "{}: FDI 几何容量 0x{geometry_size:X} 与数据大小 0x{:X} 不一致",
            path.display(),
            fdi.data_size
        ));
    }
    if data_size < 64 {
        return Err(format!("{}: FDI 数据区过小", path.display()));
    }

    let boot = &bytes[data_offset..data_end];
    let bytes_per_sector = read_u16(boot, 11)?;
    let sectors_per_cluster = boot[13];
    let reserved_sectors = read_u16(boot, 14)?;
    let fat_copies = boot[16];
    let root_entries = read_u16(boot, 17)?;
    let total_sectors_16 = read_u16(boot, 19)?;
    let media_descriptor = boot[21];
    let sectors_per_fat = read_u16(boot, 22)?;
    let bpb_sectors_per_track = read_u16(boot, 24)?;
    let bpb_heads = read_u16(boot, 26)?;
    let total_sectors = if total_sectors_16 != 0 {
        u32::from(total_sectors_16)
    } else {
        read_u32(boot, 32)?
    };

    if !(128..=4096).contains(&bytes_per_sector) || !bytes_per_sector.is_power_of_two() {
        return Err(format!(
            "{}: 非法 BPB 扇区大小 {bytes_per_sector}",
            path.display()
        ));
    }
    if sectors_per_cluster == 0 || !sectors_per_cluster.is_power_of_two() {
        return Err(format!(
            "{}: 非法 BPB 每簇扇区数 {sectors_per_cluster}",
            path.display()
        ));
    }
    if reserved_sectors == 0 || fat_copies == 0 || root_entries == 0 || sectors_per_fat == 0 {
        return Err(format!("{}: FAT12 BPB 必需字段为零", path.display()));
    }
    if u32::from(bytes_per_sector) != fdi.sector_size
        || u32::from(bpb_sectors_per_track) != fdi.sectors_per_track
        || u32::from(bpb_heads) != fdi.heads
    {
        return Err(format!("{}: FDI 几何参数与 FAT BPB 不一致", path.display()));
    }
    let bpb_bytes = u64::from(total_sectors)
        .checked_mul(u64::from(bytes_per_sector))
        .ok_or_else(|| "BPB 容量溢出".to_string())?;
    if bpb_bytes != u64::from(fdi.data_size) {
        return Err(format!(
            "{}: BPB 容量 0x{bpb_bytes:X} 与 FDI 数据大小 0x{:X} 不一致",
            path.display(),
            fdi.data_size
        ));
    }

    let root_directory_bytes = u32::from(root_entries)
        .checked_mul(32)
        .ok_or_else(|| "根目录大小溢出".to_string())?;
    let root_directory_sectors = root_directory_bytes.div_ceil(u32::from(bytes_per_sector));
    let first_root_sector = u32::from(reserved_sectors)
        .checked_add(u32::from(fat_copies) * u32::from(sectors_per_fat))
        .ok_or_else(|| "FAT 布局溢出".to_string())?;
    let first_data_sector = first_root_sector
        .checked_add(root_directory_sectors)
        .ok_or_else(|| "FAT 数据区偏移溢出".to_string())?;
    if first_data_sector >= total_sectors {
        return Err(format!("{}: FAT 数据区超出磁盘", path.display()));
    }
    let data_clusters = (total_sectors - first_data_sector) / u32::from(sectors_per_cluster);
    if data_clusters == 0 || data_clusters >= 4085 {
        return Err(format!(
            "{}: 数据簇数量 {data_clusters} 不是 FAT12 范围",
            path.display()
        ));
    }
    let max_cluster_u32 = data_clusters + 1;
    let max_cluster = u16::try_from(max_cluster_u32).map_err(|_| "FAT12 簇号溢出".to_string())?;

    let bps = usize::from(bytes_per_sector);
    let fat_bytes = usize::from(sectors_per_fat)
        .checked_mul(bps)
        .ok_or_else(|| "FAT 字节数溢出".to_string())?;
    let fat_offset = data_offset
        .checked_add(usize::from(reserved_sectors) * bps)
        .ok_or_else(|| "FAT 偏移溢出".to_string())?;
    let fat_end = fat_offset
        .checked_add(fat_bytes)
        .ok_or_else(|| "FAT 范围溢出".to_string())?;
    if fat_end > data_end {
        return Err(format!("{}: 第一份 FAT 被截断", path.display()));
    }
    let fat = &bytes[fat_offset..fat_end];
    if fat.len() < 3 || fat[0] != media_descriptor || fat[1] != 0xFF || (fat[2] & 0x0F) != 0x0F {
        return Err(format!("{}: FAT12 保留项或介质字节无效", path.display()));
    }
    let fat_capacity = (fat_bytes * 2) / 3;
    if fat_capacity <= usize::from(max_cluster) {
        return Err(format!("{}: FAT 表容量不足", path.display()));
    }
    for copy_index in 1..usize::from(fat_copies) {
        let copy_offset = fat_offset
            .checked_add(copy_index * fat_bytes)
            .ok_or_else(|| "FAT 副本偏移溢出".to_string())?;
        let copy_end = copy_offset
            .checked_add(fat_bytes)
            .ok_or_else(|| "FAT 副本范围溢出".to_string())?;
        if copy_end > data_end || bytes[copy_offset..copy_end] != *fat {
            return Err(format!(
                "{}: 第 {} 份 FAT 与第一份不一致",
                path.display(),
                copy_index + 1
            ));
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
        sectors_per_track: bpb_sectors_per_track,
        heads: bpb_heads,
        root_directory_sectors,
        first_data_sector,
        data_clusters,
        fat_copies_identical: true,
    };
    let cluster_bytes = bps
        .checked_mul(usize::from(sectors_per_cluster))
        .ok_or_else(|| "簇大小溢出".to_string())?;
    let mut parser = FatParser {
        archive: bytes,
        data_offset,
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

    let root_offset = data_offset
        .checked_add(
            usize::try_from(first_root_sector).map_err(|_| "根目录偏移过大".to_string())? * bps,
        )
        .ok_or_else(|| "根目录偏移溢出".to_string())?;
    let mut root = Vec::with_capacity(usize::from(root_entries));
    for index in 0..usize::from(root_entries) {
        let offset = root_offset
            .checked_add(index * 32)
            .ok_or_else(|| "根目录项偏移溢出".to_string())?;
        root.push(read_raw_dir_entry(bytes, offset)?);
    }
    parser.parse_directory(root, "", 0)?;

    let source_file = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("输入文件名无法表示为 Unicode: {}", path.display()))?
        .to_string();
    Ok(ParsedDisk {
        source_file,
        source_sha256: sha256_hex(bytes),
        archive_bytes: bytes.len() as u64,
        fdi,
        fat12: info,
        volume_labels: parser.volume_labels,
        directories: parser.directories,
        files: parser.files,
        extracted_files: parser.extracted_files,
        skipped_deleted_entries: parser.skipped_deleted_entries,
        skipped_lfn_entries: parser.skipped_lfn_entries,
    })
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
            let collision_key = relative_path.to_uppercase();
            if !self.seen_paths.insert(collision_key) {
                return Err(format!("输出路径发生大小写不敏感冲突: {relative_path}"));
            }
            let start_cluster = read_u16(entry, 26)?;
            let size = read_u32(entry, 28)?;
            let dos_time = read_u16(entry, 22)?;
            let dos_date = read_u16(entry, 24)?;

            if attributes & 0x10 != 0 {
                if start_cluster < 2 {
                    return Err(format!(
                        "目录 {relative_path} 的起始簇无效: {start_cluster}"
                    ));
                }
                let chain = self.read_chain(start_cluster, &format!("{relative_path} [dir]"))?;
                let child_entries = self.directory_entries_from_chain(&chain)?;
                self.directories.push(DirectoryManifest {
                    path: relative_path.clone(),
                    raw_short_name_hex,
                    attributes,
                    directory_entry_offset: raw.archive_offset as u64,
                    start_cluster,
                    cluster_chain: chain,
                    dos_time,
                    dos_date,
                });
                self.parse_directory(child_entries, &relative_path, depth + 1)?;
            } else {
                let chain = if size == 0 {
                    if start_cluster != 0 {
                        return Err(format!(
                            "空文件 {relative_path} 使用了非零起始簇 {start_cluster}"
                        ));
                    }
                    Vec::new()
                } else {
                    if start_cluster < 2 {
                        return Err(format!(
                            "文件 {relative_path} 的起始簇无效: {start_cluster}"
                        ));
                    }
                    self.read_chain(start_cluster, &format!("{relative_path} [file]"))?
                };
                let needed_clusters = if size == 0 {
                    0
                } else {
                    usize::try_from(u64::from(size).div_ceil(self.cluster_bytes as u64))
                        .map_err(|_| format!("文件 {relative_path} 大小过大"))?
                };
                if chain.len() != needed_clusters {
                    return Err(format!(
                        "文件 {relative_path} 大小为 {size}，需要 {needed_clusters} 簇，FAT 链实际为 {} 簇",
                        chain.len()
                    ));
                }
                let data = self.read_file_data(&chain, size, &relative_path)?;
                let file_hash = sha256_hex(&data);
                self.files.push(FileManifest {
                    path: relative_path.clone(),
                    raw_short_name_hex,
                    attributes,
                    directory_entry_offset: raw.archive_offset as u64,
                    start_cluster,
                    size,
                    cluster_chain: chain,
                    dos_time,
                    dos_date,
                    sha256: file_hash,
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
                return Err(format!("{owner}: 簇 {cluster} 超出有效范围"));
            }
            if !local_seen.insert(cluster) {
                return Err(format!("{owner}: FAT 链在簇 {cluster} 形成循环"));
            }
            if let Some(previous_owner) = self.owners.get(&cluster) {
                return Err(format!(
                    "{owner}: 簇 {cluster} 与 {previous_owner} 交叉链接"
                ));
            }
            self.owners.insert(cluster, owner.to_string());
            chain.push(cluster);
            if chain.len() > usize::from(self.max_cluster) {
                return Err(format!("{owner}: FAT 链长度超过磁盘容量"));
            }
            let next = fat12_next(self.fat, cluster)?;
            match next {
                0xFF8..=0xFFF => break,
                0xFF7 => return Err(format!("{owner}: FAT 链遇到坏簇标记")),
                0xFF0..=0xFF6 => return Err(format!("{owner}: FAT 链遇到保留值 0x{next:03X}")),
                0 => return Err(format!("{owner}: FAT 链意外指向空闲簇")),
                1 => return Err(format!("{owner}: FAT 链意外指向保留簇 1")),
                _ => cluster = next,
            }
        }
        Ok(chain)
    }

    fn directory_entries_from_chain(&self, chain: &[u16]) -> Result<Vec<RawDirEntry>> {
        let mut entries = Vec::with_capacity(chain.len() * (self.cluster_bytes / 32));
        for &cluster in chain {
            let offset = self.cluster_offset(cluster)?;
            for local_offset in (0..self.cluster_bytes).step_by(32) {
                entries.push(read_raw_dir_entry(self.archive, offset + local_offset)?);
            }
        }
        Ok(entries)
    }

    fn read_file_data(&self, chain: &[u16], size: u32, owner: &str) -> Result<Vec<u8>> {
        let capacity = chain
            .len()
            .checked_mul(self.cluster_bytes)
            .ok_or_else(|| format!("{owner}: 文件缓冲区大小溢出"))?;
        let mut data = Vec::with_capacity(capacity);
        for &cluster in chain {
            let offset = self.cluster_offset(cluster)?;
            let end = offset
                .checked_add(self.cluster_bytes)
                .ok_or_else(|| format!("{owner}: 簇范围溢出"))?;
            data.extend_from_slice(&self.archive[offset..end]);
        }
        let size = usize::try_from(size).map_err(|_| format!("{owner}: 文件大小过大"))?;
        if size > data.len() {
            return Err(format!("{owner}: FAT 链容量小于目录项大小"));
        }
        data.truncate(size);
        Ok(data)
    }

    fn cluster_offset(&self, cluster: u16) -> Result<usize> {
        if cluster < 2 || cluster > self.max_cluster {
            return Err(format!("簇 {cluster} 超出有效范围"));
        }
        let cluster_sector = u32::from(cluster - 2)
            .checked_mul(u32::from(self.info.sectors_per_cluster))
            .and_then(|value| value.checked_add(self.info.first_data_sector))
            .ok_or_else(|| format!("簇 {cluster} 的扇区偏移溢出"))?;
        let byte_offset = usize::try_from(cluster_sector)
            .map_err(|_| format!("簇 {cluster} 的扇区偏移过大"))?
            .checked_mul(usize::from(self.info.bytes_per_sector))
            .and_then(|value| value.checked_add(self.data_offset))
            .ok_or_else(|| format!("簇 {cluster} 的字节偏移溢出"))?;
        let end = byte_offset
            .checked_add(self.cluster_bytes)
            .ok_or_else(|| format!("簇 {cluster} 的字节范围溢出"))?;
        if end > self.archive.len() {
            return Err(format!("簇 {cluster} 超出 FDI 数据区"));
        }
        Ok(byte_offset)
    }
}

fn write_workspace(
    prepared: &[PreparedDisk],
    output_root: &Path,
    overwrite: bool,
) -> Result<UnpackReport> {
    let parent = output_root
        .parent()
        .ok_or_else(|| format!("输出目录没有父目录: {}", output_root.display()))?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("无法创建输出父目录 {}: {e}", parent.display()))?;
    let root_name = output_root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("输出目录名无法表示为 Unicode: {}", output_root.display()))?;
    let staging = unique_sibling(parent, &format!(".{root_name}.tmp-{}", std::process::id()))?;
    fs::create_dir(&staging).map_err(|e| format!("无法创建临时输出 {}: {e}", staging.display()))?;

    let write_result = (|| -> Result<UnpackReport> {
        let mut archive_manifests = Vec::with_capacity(prepared.len());
        let mut extracted_files = 0usize;
        let mut extracted_directories = 0usize;
        let mut extracted_bytes = 0u64;

        for disk in prepared {
            let disk_root = staging.join(&disk.output_dir);
            fs::create_dir(&disk_root)
                .map_err(|e| format!("无法创建磁盘输出目录 {}: {e}", disk_root.display()))?;
            let mut directories: Vec<&DirectoryManifest> = disk.parsed.directories.iter().collect();
            directories.sort_by_key(|entry| entry.path.matches('/').count());
            for directory in directories {
                let output_path = join_manifest_path(&disk_root, &directory.path)?;
                fs::create_dir_all(&output_path)
                    .map_err(|e| format!("无法创建目录 {}: {e}", output_path.display()))?;
            }
            for file in &disk.parsed.extracted_files {
                let output_path = join_manifest_path(&disk_root, &file.relative_path)?;
                if let Some(file_parent) = output_path.parent() {
                    fs::create_dir_all(file_parent).map_err(|e| {
                        format!("无法创建文件父目录 {}: {e}", file_parent.display())
                    })?;
                }
                let mut handle = fs::File::create(&output_path)
                    .map_err(|e| format!("无法创建文件 {}: {e}", output_path.display()))?;
                handle
                    .write_all(&file.data)
                    .map_err(|e| format!("无法写入文件 {}: {e}", output_path.display()))?;
                extracted_files += 1;
                extracted_bytes += file.data.len() as u64;
            }
            extracted_directories += disk.parsed.directories.len();
            archive_manifests.push(ArchiveManifest {
                source_file: disk.parsed.source_file.clone(),
                source_sha256: disk.parsed.source_sha256.clone(),
                output_dir: disk.output_dir.clone(),
                archive_bytes: disk.parsed.archive_bytes,
                fdi: disk.parsed.fdi.clone(),
                fat12: disk.parsed.fat12.clone(),
                volume_labels: disk.parsed.volume_labels.clone(),
                directories: disk.parsed.directories.clone(),
                files: disk.parsed.files.clone(),
                skipped_deleted_entries: disk.parsed.skipped_deleted_entries,
                skipped_lfn_entries: disk.parsed.skipped_lfn_entries,
            });
        }
        let workspace = WorkspaceManifest {
            _format: WORKSPACE_FORMAT.to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            role_paths: RolePaths {
                unpacked_root: ".".to_string(),
            },
            archives: archive_manifests,
        };
        let mut json = serde_json::to_vec_pretty(&workspace)
            .map_err(|e| format!("无法序列化 workspace.json: {e}"))?;
        json.push(b'\n');
        fs::write(staging.join("workspace.json"), json)
            .map_err(|e| format!("无法写入 workspace.json: {e}"))?;

        commit_staging(&staging, output_root, overwrite)?;
        Ok(UnpackReport {
            images: prepared.len(),
            extracted_files,
            extracted_directories,
            extracted_bytes,
            output_root: output_root.to_path_buf(),
        })
    })();

    if write_result.is_err() && staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    write_result
}

fn commit_staging(staging: &Path, output_root: &Path, overwrite: bool) -> Result<()> {
    if !output_root.exists() {
        return fs::rename(staging, output_root).map_err(|e| {
            format!(
                "无法把临时输出 {} 提交为 {}: {e}",
                staging.display(),
                output_root.display()
            )
        });
    }
    if !overwrite {
        return Err(format!("输出目录已存在: {}", output_root.display()));
    }
    let parent = output_root
        .parent()
        .ok_or_else(|| "输出目录没有父目录".to_string())?;
    let backup = unique_sibling(
        parent,
        &format!(".fdi-unpack-backup-{}", std::process::id()),
    )?;
    fs::rename(output_root, &backup).map_err(|e| {
        format!(
            "无法把现有输出 {} 移到备份 {}: {e}",
            output_root.display(),
            backup.display()
        )
    })?;
    if let Err(error) = fs::rename(staging, output_root) {
        let rollback = fs::rename(&backup, output_root);
        return match rollback {
            Ok(()) => Err(format!("提交新输出失败，已恢复旧输出: {error}")),
            Err(rollback_error) => Err(format!(
                "提交新输出失败且旧输出恢复失败；备份位于 {}: {error}; {rollback_error}",
                backup.display()
            )),
        };
    }
    fs::remove_dir_all(&backup).map_err(|e| {
        format!(
            "新输出已提交，但无法清理旧输出备份 {}: {e}",
            backup.display()
        )
    })?;
    Ok(())
}

fn validate_existing_output(output_root: &Path, overwrite: bool) -> Result<()> {
    if !output_root.exists() {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(output_root)
        .map_err(|e| format!("无法读取输出目录元数据 {}: {e}", output_root.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "输出路径必须是非符号链接目录: {}",
            output_root.display()
        ));
    }
    if !overwrite {
        return Err(format!(
            "输出目录已存在；请改用新目录或显式指定 --overwrite: {}",
            output_root.display()
        ));
    }
    let mut entries = fs::read_dir(output_root)
        .map_err(|e| format!("无法读取输出目录 {}: {e}", output_root.display()))?;
    if entries.next().is_none() {
        return Ok(());
    }
    let manifest_path = output_root.join("workspace.json");
    let manifest_bytes = fs::read(&manifest_path).map_err(|_| {
        format!(
            "拒绝覆盖非空且不含有效 workspace.json 的目录: {}",
            output_root.display()
        )
    })?;
    let value: serde_json::Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("现有 workspace.json 无效，拒绝覆盖: {e}"))?;
    if value.get("_format").and_then(|item| item.as_str()) != Some(WORKSPACE_FORMAT) {
        return Err(format!(
            "现有 workspace.json 格式不属于本工具，拒绝覆盖: {}",
            output_root.display()
        ));
    }
    Ok(())
}

fn validate_output_root(output_root: &Path) -> Result<()> {
    if output_root.as_os_str().is_empty() {
        return Err("输出目录不能为空".to_string());
    }
    let mut normal_components = 0usize;
    for component in output_root.components() {
        match component {
            Component::ParentDir => return Err("输出目录不能包含 ..".to_string()),
            Component::Normal(_) => normal_components += 1,
            _ => {}
        }
    }
    if normal_components == 0 {
        return Err(format!(
            "拒绝把文件系统根目录作为输出: {}",
            output_root.display()
        ));
    }
    Ok(())
}

fn validate_output_does_not_contain_inputs(inputs: &[PathBuf], output_root: &Path) -> Result<()> {
    let output_absolute = absolute_lexical(output_root)?;
    for input in inputs {
        let input_absolute = fs::canonicalize(input)
            .map_err(|e| format!("无法解析输入路径 {}: {e}", input.display()))?;
        if input_absolute.starts_with(&output_absolute) {
            return Err(format!(
                "输出目录包含输入文件，覆盖时可能删除源盘，已拒绝: {}",
                input.display()
            ));
        }
    }
    Ok(())
}

fn absolute_lexical(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path).map_err(|e| format!("无法解析路径 {}: {e}", path.display()));
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("路径没有父目录: {}", path.display()))?;
    let parent_absolute = if parent.exists() {
        fs::canonicalize(parent).map_err(|e| format!("无法解析父目录 {}: {e}", parent.display()))?
    } else if parent.is_absolute() {
        parent.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("无法读取当前目录: {e}"))?
            .join(parent)
    };
    let name = path
        .file_name()
        .ok_or_else(|| format!("路径缺少目录名: {}", path.display()))?;
    Ok(parent_absolute.join(name))
}

fn validate_output_segment(segment: &str) -> Result<()> {
    if segment.is_empty() || segment == "." || segment == ".." {
        return Err(format!("不安全的输出路径段: {segment:?}"));
    }
    if segment.ends_with(' ') || segment.ends_with('.') {
        return Err(format!("Windows 输出路径不能以空格或点结尾: {segment}"));
    }
    if segment
        .chars()
        .any(|ch| ch < ' ' || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'))
    {
        return Err(format!("Windows 输出路径含非法字符: {segment}"));
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
        return Err(format!("Windows 保留设备名不能作为输出路径: {segment}"));
    }
    Ok(())
}

fn join_manifest_path(root: &Path, relative: &str) -> Result<PathBuf> {
    let mut output = root.to_path_buf();
    for segment in relative.split('/') {
        validate_output_segment(segment)?;
        output.push(segment);
    }
    Ok(output)
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
    let stem = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(&stem)
        .ok_or_else(|| format!("目录项短文件名不是有效 CP932: {raw_hex}"))?;
    let extension = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(&extension)
        .ok_or_else(|| format!("目录项扩展名不是有效 CP932: {raw_hex}"))?;
    let name = if extension.is_empty() {
        stem.into_owned()
    } else {
        format!("{stem}.{extension}")
    };
    Ok((name, raw_hex))
}

fn fat12_next(fat: &[u8], cluster: u16) -> Result<u16> {
    let offset = (usize::from(cluster) * 3) / 2;
    let end = offset
        .checked_add(2)
        .ok_or_else(|| "FAT12 项偏移溢出".to_string())?;
    if end > fat.len() {
        return Err(format!("FAT12 项 {cluster} 超出表范围"));
    }
    let word = u16::from(fat[offset]) | (u16::from(fat[offset + 1]) << 8);
    Ok(if cluster & 1 == 0 {
        word & 0x0FFF
    } else {
        (word >> 4) & 0x0FFF
    })
}

fn read_raw_dir_entry(bytes: &[u8], offset: usize) -> Result<RawDirEntry> {
    let end = offset
        .checked_add(32)
        .ok_or_else(|| "目录项范围溢出".to_string())?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| format!("目录项在 0x{offset:X} 被截断"))?;
    let mut entry = [0u8; 32];
    entry.copy_from_slice(slice);
    Ok(RawDirEntry {
        bytes: entry,
        archive_offset: offset,
    })
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let slice = bytes
        .get(offset..offset.saturating_add(2))
        .ok_or_else(|| format!("读取 0x{offset:X} 处 u16 时越界"))?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let slice = bytes
        .get(offset..offset.saturating_add(4))
        .ok_or_else(|| format!("读取 0x{offset:X} 处 u32 时越界"))?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn checked_product(values: &[u32]) -> Result<u64> {
    values.iter().try_fold(1u64, |accumulator, value| {
        accumulator
            .checked_mul(u64::from(*value))
            .ok_or_else(|| "FDI 几何容量溢出".to_string())
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex_upper(&digest)
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

fn unique_sibling(parent: &Path, base_name: &str) -> Result<PathBuf> {
    for suffix in 0..1000u32 {
        let name = if suffix == 0 {
            base_name.to_string()
        } else {
            format!("{base_name}-{suffix}")
        };
        let candidate = parent.join(name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(format!("无法在 {} 创建唯一临时目录名", parent.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    const HEADER_SIZE: usize = 0x100;
    const SECTOR_SIZE: usize = 1024;
    const TOTAL_SECTORS: usize = 16;

    fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn set_fat12(fat: &mut [u8], cluster: u16, value: u16) {
        let offset = (usize::from(cluster) * 3) / 2;
        if cluster & 1 == 0 {
            fat[offset] = value as u8;
            fat[offset + 1] = (fat[offset + 1] & 0xF0) | ((value >> 8) as u8 & 0x0F);
        } else {
            fat[offset] = (fat[offset] & 0x0F) | ((value << 4) as u8 & 0xF0);
            fat[offset + 1] = (value >> 4) as u8;
        }
    }

    fn sample_fdi() -> Vec<u8> {
        let disk_size = SECTOR_SIZE * TOTAL_SECTORS;
        let mut bytes = vec![0u8; HEADER_SIZE + disk_size];
        put_u32(&mut bytes, 4, 0x90);
        put_u32(&mut bytes, 8, HEADER_SIZE as u32);
        put_u32(&mut bytes, 12, disk_size as u32);
        put_u32(&mut bytes, 16, SECTOR_SIZE as u32);
        put_u32(&mut bytes, 20, 4);
        put_u32(&mut bytes, 24, 1);
        put_u32(&mut bytes, 28, 4);

        let boot = HEADER_SIZE;
        bytes[boot..boot + 3].copy_from_slice(&[0xEB, 0x1C, 0x90]);
        put_u16(&mut bytes, boot + 11, SECTOR_SIZE as u16);
        bytes[boot + 13] = 1;
        put_u16(&mut bytes, boot + 14, 1);
        bytes[boot + 16] = 2;
        put_u16(&mut bytes, boot + 17, 32);
        put_u16(&mut bytes, boot + 19, TOTAL_SECTORS as u16);
        bytes[boot + 21] = 0xFE;
        put_u16(&mut bytes, boot + 22, 1);
        put_u16(&mut bytes, boot + 24, 4);
        put_u16(&mut bytes, boot + 26, 1);

        let fat1_offset = HEADER_SIZE + SECTOR_SIZE;
        let mut fat = vec![0u8; SECTOR_SIZE];
        fat[..3].copy_from_slice(&[0xFE, 0xFF, 0xFF]);
        set_fat12(&mut fat, 2, 0xFFF);
        bytes[fat1_offset..fat1_offset + SECTOR_SIZE].copy_from_slice(&fat);
        bytes[fat1_offset + SECTOR_SIZE..fat1_offset + SECTOR_SIZE * 2].copy_from_slice(&fat);

        let root = HEADER_SIZE + SECTOR_SIZE * 3;
        bytes[root..root + 11].copy_from_slice(b"HELLO   TXT");
        bytes[root + 11] = 0x20;
        put_u16(&mut bytes, root + 26, 2);
        put_u32(&mut bytes, root + 28, 5);
        let cluster2 = HEADER_SIZE + SECTOR_SIZE * 4;
        bytes[cluster2..cluster2 + 5].copy_from_slice(b"hello");
        bytes
    }

    #[test]
    fn parses_valid_fdi_and_extracts_exact_bytes() {
        let bytes = sample_fdi();
        let parsed = parse_fdi(Path::new("disk.fdi"), &bytes).expect("valid image");
        assert_eq!(parsed.files.len(), 1);
        assert_eq!(parsed.files[0].path, "HELLO.TXT");
        assert_eq!(parsed.files[0].cluster_chain, vec![2]);
        assert_eq!(parsed.extracted_files[0].data, b"hello");
    }

    #[test]
    fn rejects_truncated_fdi() {
        let mut bytes = sample_fdi();
        bytes.pop();
        let error = parse_fdi(Path::new("disk.fdi"), &bytes).unwrap_err();
        assert!(error.contains("长度不一致"));
    }

    #[test]
    fn rejects_mismatched_fat_copies() {
        let mut bytes = sample_fdi();
        bytes[HEADER_SIZE + SECTOR_SIZE * 2 + 10] ^= 1;
        let error = parse_fdi(Path::new("disk.fdi"), &bytes).unwrap_err();
        assert!(error.contains("FAT 与第一份不一致"));
    }

    #[test]
    fn rejects_cluster_loop() {
        let mut bytes = sample_fdi();
        for fat_index in 0..2 {
            let start = HEADER_SIZE + SECTOR_SIZE * (1 + fat_index);
            set_fat12(&mut bytes[start..start + SECTOR_SIZE], 2, 2);
        }
        let error = parse_fdi(Path::new("disk.fdi"), &bytes).unwrap_err();
        assert!(error.contains("形成循环"));
    }

    #[test]
    fn rejects_file_size_chain_mismatch() {
        let mut bytes = sample_fdi();
        let root = HEADER_SIZE + SECTOR_SIZE * 3;
        put_u32(&mut bytes, root + 28, 1025);
        let error = parse_fdi(Path::new("disk.fdi"), &bytes).unwrap_err();
        assert!(error.contains("需要 2 簇"));
    }

    #[test]
    fn rejects_cross_linked_files() {
        let mut bytes = sample_fdi();
        let root = HEADER_SIZE + SECTOR_SIZE * 3;
        let second = root + 32;
        bytes[second..second + 11].copy_from_slice(b"COPY    TXT");
        bytes[second + 11] = 0x20;
        put_u16(&mut bytes, second + 26, 2);
        put_u32(&mut bytes, second + 28, 5);
        let error = parse_fdi(Path::new("disk.fdi"), &bytes).unwrap_err();
        assert!(error.contains("交叉链接"));
    }

    #[test]
    fn rejects_unsafe_output_name() {
        let mut bytes = sample_fdi();
        let root = HEADER_SIZE + SECTOR_SIZE * 3;
        bytes[root..root + 8].copy_from_slice(b"BAD\\NAME");
        let error = parse_fdi(Path::new("disk.fdi"), &bytes).unwrap_err();
        assert!(error.contains("非法字符"));
    }

    #[test]
    fn writes_workspace_manifest_and_file() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let base =
            std::env::temp_dir().join(format!("fdi-unpack-test-{}-{nonce}", std::process::id()));
        fs::create_dir(&base).expect("create test dir");
        let input = base.join("disk.fdi");
        let output = base.join("result");
        fs::write(&input, sample_fdi()).expect("write input");
        let report = unpack_batch(std::slice::from_ref(&input), &output, false).expect("unpack");
        assert_eq!(report.images, 1);
        assert_eq!(report.extracted_files, 1);
        assert_eq!(
            fs::read(output.join("disk/HELLO.TXT")).expect("read output"),
            b"hello"
        );
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("workspace.json")).expect("manifest"))
                .expect("valid json");
        assert_eq!(manifest["_format"], WORKSPACE_FORMAT);
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn batch_preflight_failure_writes_nothing() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let base = std::env::temp_dir().join(format!(
            "fdi-unpack-preflight-test-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&base).expect("create test dir");
        let valid = base.join("valid.fdi");
        let invalid = base.join("invalid.fdi");
        let output = base.join("result");
        fs::write(&valid, sample_fdi()).expect("write valid input");
        fs::write(&invalid, b"truncated").expect("write invalid input");
        let error = unpack_batch(&[valid, invalid], &output, false).unwrap_err();
        assert!(error.contains("FDI 头被截断"));
        assert!(!output.exists());
        fs::remove_dir_all(&base).expect("cleanup");
    }
}
