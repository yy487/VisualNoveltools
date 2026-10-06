use encoding_rs::SHIFT_JIS;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

pub mod text;
pub use text::{prepare_text_export, prepare_text_import, PreparedTextExport, PreparedTextImport};

pub const MANIFEST_FILENAME: &str = ".hdi_manifest.json";
pub const MANIFEST_FORMAT: &str = "platinum-star-anex86-hdi-fat12-unpack-v1";

pub type Result<T> = std::result::Result<T, String>;

const PARTITION_ENTRY_BYTES: usize = 0x20;
const DIRECTORY_ENTRY_BYTES: usize = 0x20;
const FAT12_MAX_CLUSTERS: u32 = 4_084;

#[derive(Debug, Clone, Serialize)]
pub struct HdiHeader {
    pub header_size: u32,
    pub disk_size: u32,
    pub physical_sector_size: u32,
    pub sectors_per_track: u32,
    pub heads: u32,
    pub cylinders: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PartitionInfo {
    pub table_offset: u64,
    pub entry_index: u32,
    pub entry_offset: u64,
    pub raw_entry_hex: String,
    pub start_sector: u8,
    pub start_head: u8,
    pub start_cylinder: u16,
    pub start_lba: u32,
    pub byte_offset: u64,
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
    pub fat_media_descriptor: u8,
    pub media_descriptor_matches_fat: bool,
    pub sectors_per_fat: u16,
    pub sectors_per_track: u16,
    pub heads: u16,
    pub hidden_sectors: u32,
    pub root_directory_sectors: u32,
    pub first_data_sector: u32,
    pub data_clusters: u32,
    pub cluster_bytes: u32,
    pub fat_copies_identical: bool,
    pub fat_mismatch_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DirectoryManifest {
    pub path: String,
    pub decoded_name: Option<String>,
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
    pub decoded_name: Option<String>,
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
pub struct HdiManifest {
    pub _format: String,
    pub tool_version: String,
    pub source_file: String,
    pub source_sha256: String,
    pub source_bytes: u64,
    pub role_paths: RolePaths,
    pub hdi: HdiHeader,
    pub partition: PartitionInfo,
    pub fat12: Fat12Info,
    pub volume_labels: Vec<String>,
    pub directories: Vec<DirectoryManifest>,
    pub files: Vec<FileManifest>,
    pub skipped_deleted_entries: u64,
    pub skipped_lfn_entries: u64,
    pub orphan_clusters: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RolePaths {
    pub source_hdi: String,
    pub unpacked_root: String,
}

#[derive(Debug, Clone)]
pub struct Inspection {
    pub source_bytes: u64,
    pub source_sha256: String,
    pub partition_offset: u64,
    pub files: usize,
    pub directories: usize,
    pub extracted_bytes: u64,
    pub orphan_clusters: u64,
    pub fat_mismatch_bytes: u64,
    pub media_descriptor_matches_fat: bool,
    pub bytes_per_sector: u16,
    pub cluster_bytes: u32,
}

#[derive(Debug, Clone)]
pub struct UnpackReport {
    pub files: usize,
    pub directories: usize,
    pub extracted_bytes: u64,
    pub orphan_clusters: u64,
    pub fat_mismatch_bytes: u64,
    pub output_root: PathBuf,
    pub manifest: PathBuf,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
struct ExtractedFile {
    relative_path: String,
    data: Vec<u8>,
}

#[derive(Debug)]
struct ParsedImage {
    source_path: PathBuf,
    manifest: HdiManifest,
    extracted_files: Vec<ExtractedFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OutputState {
    Missing,
    Existing(String),
}

#[derive(Debug)]
pub struct PreparedUnpack {
    parsed: ParsedImage,
    output: PathBuf,
    overwrite: bool,
    output_state: OutputState,
}

#[derive(Debug, Clone)]
pub struct HdiPackInspection {
    pub source_bytes: u64,
    pub source_files: usize,
    pub changed_files: usize,
    pub changed_bytes: u64,
    pub free_clusters: usize,
}

#[derive(Debug, Clone)]
pub struct HdiPackReport {
    pub output: PathBuf,
    pub source_files: usize,
    pub changed_files: usize,
    pub changed_bytes: u64,
    pub output_sha256: String,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub struct PreparedHdiPack {
    source_hdi: PathBuf,
    input_root: PathBuf,
    output: PathBuf,
    overwrite: bool,
    image: Vec<u8>,
    manifest: HdiManifest,
    updates: Vec<PackUpdate>,
    warnings: Vec<String>,
    inspection: HdiPackInspection,
}

#[derive(Debug)]
struct PackUpdate {
    manifest_path: String,
    directory_entry_offset: usize,
    old_chain: Vec<u16>,
    data: Vec<u8>,
}

impl PreparedUnpack {
    pub fn source(&self) -> &Path {
        &self.parsed.source_path
    }

    pub fn output(&self) -> &Path {
        &self.output
    }

    pub fn inspection(&self) -> Inspection {
        inspection_from_manifest(&self.parsed.manifest)
    }

    pub fn execute(self) -> Result<UnpackReport> {
        write_prepared(self)
    }
}

impl PreparedHdiPack {
    pub fn source(&self) -> &Path {
        &self.source_hdi
    }

    pub fn input_root(&self) -> &Path {
        &self.input_root
    }

    pub fn output(&self) -> &Path {
        &self.output
    }

    pub fn inspection(&self) -> &HdiPackInspection {
        &self.inspection
    }

    pub fn execute(self) -> Result<HdiPackReport> {
        write_prepared_hdi_pack(self)
    }
}

#[derive(Debug, Clone)]
struct RawDirEntry {
    bytes: [u8; DIRECTORY_ENTRY_BYTES],
    image_offset: usize,
}

struct FatParser<'a> {
    image: &'a [u8],
    partition_offset: usize,
    partition_end: usize,
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

pub fn inspect_hdi(path: &Path) -> Result<Inspection> {
    let bytes = fs::read(path).map_err(|error| format!("无法读取 {}: {error}", path.display()))?;
    let parsed = parse_hdi(path, &bytes)?;
    Ok(inspection_from_manifest(&parsed.manifest))
}

pub fn prepare_unpack(source: &Path, output: &Path, overwrite: bool) -> Result<PreparedUnpack> {
    validate_output_root(output)?;
    let metadata = fs::metadata(source)
        .map_err(|error| format!("无法读取输入元数据 {}: {error}", source.display()))?;
    if !metadata.is_file() {
        return Err(format!("输入不是普通文件: {}", source.display()));
    }
    let source_path = fs::canonicalize(source)
        .map_err(|error| format!("无法解析输入路径 {}: {error}", source.display()))?;
    let output = absolute_lexical(output)?;
    if source_path.starts_with(&output) {
        return Err("输出目录包含输入 HDI，覆盖时可能删除源盘".to_string());
    }
    let bytes = fs::read(&source_path)
        .map_err(|error| format!("无法读取 {}: {error}", source_path.display()))?;
    let parsed = parse_hdi(&source_path, &bytes)?;
    let output_state = capture_output_state(&output, overwrite)?;
    Ok(PreparedUnpack {
        parsed,
        output,
        overwrite,
        output_state,
    })
}

pub fn prepare_hdi_pack(
    source_hdi: &Path,
    input_root: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<PreparedHdiPack> {
    let source_hdi = fs::canonicalize(source_hdi)
        .map_err(|error| format!("无法解析源 HDI {}: {error}", source_hdi.display()))?;
    let source_metadata = fs::metadata(&source_hdi)
        .map_err(|error| format!("无法读取源 HDI {}: {error}", source_hdi.display()))?;
    if !source_metadata.is_file() {
        return Err(format!("源 HDI 不是普通文件: {}", source_hdi.display()));
    }
    let input_root = fs::canonicalize(input_root)
        .map_err(|error| format!("无法解析回注目录 {}: {error}", input_root.display()))?;
    if !input_root.is_dir() {
        return Err(format!("回注目录不是目录: {}", input_root.display()));
    }
    let output = absolute_lexical(output)?;
    if output == source_hdi {
        return Err("输出 HDI 不能覆盖源 HDI".into());
    }
    if output.starts_with(&input_root) || input_root.starts_with(&output) {
        return Err("输出 HDI 不能与回注目录互相包含".into());
    }
    if output.exists() {
        if !overwrite {
            return Err(format!("输出 HDI 已存在；默认不覆盖: {}", output.display()));
        }
        let metadata = fs::symlink_metadata(&output)
            .map_err(|error| format!("无法读取已有输出 HDI {}: {error}", output.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!("已有输出 HDI 不是普通文件: {}", output.display()));
        }
    }

    let image = fs::read(&source_hdi)
        .map_err(|error| format!("无法读取源 HDI {}: {error}", source_hdi.display()))?;
    let parsed = parse_hdi(&source_hdi, &image)?;
    let input_files = snapshot_pack_input(&input_root)?;
    let source_data = parsed
        .extracted_files
        .iter()
        .map(|file| (file.relative_path.clone(), file.data.as_slice()))
        .collect::<BTreeMap<_, _>>();
    let mut updates = Vec::new();
    let mut matched = HashSet::new();
    let mut warnings = Vec::new();
    let prefixes = manifest_prefixes(&parsed.manifest);
    let input_keys = input_files
        .keys()
        .filter(|relative| !is_pack_sidecar(relative))
        .collect::<Vec<_>>();
    let mut scored_prefixes = prefixes
        .into_iter()
        .map(|prefix| {
            let score = input_keys
                .iter()
                .filter(|relative| {
                    let target = if prefix.is_empty() {
                        (*relative).to_string()
                    } else {
                        format!("{prefix}/{relative}")
                    };
                    parsed.manifest.files.iter().any(|file| file.path == target)
                })
                .count();
            (prefix, score)
        })
        .collect::<Vec<_>>();
    scored_prefixes.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    let (prefix, prefix_score) = scored_prefixes
        .first()
        .cloned()
        .ok_or_else(|| "源 HDI 没有可回封的活动文件".to_string())?;
    if prefix_score != input_keys.len() {
        return Err(format!(
            "回注目录中有 {} 个文件无法映射到同一 HDI 目录前缀",
            input_keys.len().saturating_sub(prefix_score)
        ));
    }
    if scored_prefixes
        .get(1)
        .is_some_and(|candidate| candidate.1 == prefix_score)
    {
        return Err("回注目录对应多个 HDI 目录前缀，拒绝猜测".into());
    }
    if !prefix.is_empty() {
        warnings.push(format!("回封活动文件目录前缀: {prefix}"));
    }
    for (relative, data) in input_files {
        let target = if prefix.is_empty() {
            relative.clone()
        } else {
            format!("{prefix}/{relative}")
        };
        let Some(file) = parsed
            .manifest
            .files
            .iter()
            .find(|file| file.path == target)
        else {
            if is_pack_sidecar(&relative) {
                warnings.push(format!("忽略回注目录附加文件: {relative}"));
                continue;
            }
            return Err(format!("回注目录文件无法映射到源 HDI 活动文件: {relative}"));
        };
        if !matched.insert(file.path.clone()) {
            return Err(format!("同一个 HDI 文件被回注目录重复匹配: {}", file.path));
        }
        let original = source_data
            .get(&file.path)
            .ok_or_else(|| format!("缺少源文件数据: {}", file.path))?;
        if *original != data.as_slice() {
            updates.push(PackUpdate {
                manifest_path: file.path.clone(),
                directory_entry_offset: usize::try_from(file.directory_entry_offset)
                    .map_err(|_| format!("目录项偏移过大: {}", file.path))?,
                old_chain: file.cluster_chain.clone(),
                data,
            });
        }
    }
    updates.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
    let free_clusters = free_clusters_after_release(&image, &parsed.manifest, &updates)?;
    let changed_bytes = updates.iter().map(|update| update.data.len() as u64).sum();
    let inspection = HdiPackInspection {
        source_bytes: image.len() as u64,
        source_files: parsed.manifest.files.len(),
        changed_files: updates.len(),
        changed_bytes,
        free_clusters,
    };
    Ok(PreparedHdiPack {
        source_hdi,
        input_root,
        output,
        overwrite,
        image,
        manifest: parsed.manifest,
        updates,
        warnings,
        inspection,
    })
}

fn inspection_from_manifest(manifest: &HdiManifest) -> Inspection {
    Inspection {
        source_bytes: manifest.source_bytes,
        source_sha256: manifest.source_sha256.clone(),
        partition_offset: manifest.partition.byte_offset,
        files: manifest.files.len(),
        directories: manifest.directories.len(),
        extracted_bytes: manifest.files.iter().map(|file| u64::from(file.size)).sum(),
        orphan_clusters: manifest.orphan_clusters,
        fat_mismatch_bytes: manifest.fat12.fat_mismatch_bytes,
        media_descriptor_matches_fat: manifest.fat12.media_descriptor_matches_fat,
        bytes_per_sector: manifest.fat12.bytes_per_sector,
        cluster_bytes: manifest.fat12.cluster_bytes,
    }
}

fn parse_hdi(path: &Path, bytes: &[u8]) -> Result<ParsedImage> {
    if bytes.len() < 0x20 {
        return Err(format!("{}: HDI 头被截断", path.display()));
    }
    let hdi = HdiHeader {
        header_size: read_u32(bytes, 0x08)?,
        disk_size: read_u32(bytes, 0x0C)?,
        physical_sector_size: read_u32(bytes, 0x10)?,
        sectors_per_track: read_u32(bytes, 0x14)?,
        heads: read_u32(bytes, 0x18)?,
        cylinders: read_u32(bytes, 0x1C)?,
    };
    validate_hdi_header(path, bytes, &hdi)?;

    let header_size = usize_from_u32(hdi.header_size, "HDI 头大小")?;
    let physical_sector = usize_from_u32(hdi.physical_sector_size, "HDI 物理扇区大小")?;
    let table_offset = header_size
        .checked_add(physical_sector)
        .ok_or_else(|| "PC-98 分区表偏移溢出".to_string())?;
    let table_end = table_offset
        .checked_add(physical_sector)
        .ok_or_else(|| "PC-98 分区表范围溢出".to_string())?;
    if table_end > bytes.len() {
        return Err(format!("{}: PC-98 分区表被截断", path.display()));
    }

    let mut candidates = Vec::new();
    let mut candidate_errors = Vec::new();
    for entry_index in 0..physical_sector / PARTITION_ENTRY_BYTES {
        let entry_offset = table_offset + entry_index * PARTITION_ENTRY_BYTES;
        let entry = &bytes[entry_offset..entry_offset + PARTITION_ENTRY_BYTES];
        if entry[0] == 0 && entry[1] == 0 {
            continue;
        }
        let start_sector = entry[8];
        let start_head = entry[9];
        let start_cylinder = read_u16(entry, 10)?;
        if u32::from(start_sector) >= hdi.sectors_per_track
            || u32::from(start_head) >= hdi.heads
            || u32::from(start_cylinder) >= hdi.cylinders
        {
            continue;
        }
        let start_lba = u32::from(start_cylinder)
            .checked_mul(hdi.heads)
            .and_then(|value| value.checked_add(u32::from(start_head)))
            .and_then(|value| value.checked_mul(hdi.sectors_per_track))
            .and_then(|value| value.checked_add(u32::from(start_sector)))
            .ok_or_else(|| "PC-98 分区 CHS 换算溢出".to_string())?;
        let byte_offset = header_size
            .checked_add(
                usize::try_from(start_lba)
                    .map_err(|_| "分区 LBA 过大".to_string())?
                    .checked_mul(physical_sector)
                    .ok_or_else(|| "分区字节偏移溢出".to_string())?,
            )
            .ok_or_else(|| "分区字节偏移溢出".to_string())?;
        match parse_fat12_info(bytes, byte_offset, &hdi) {
            Ok(info) => candidates.push((
                entry_index,
                entry_offset,
                entry.to_vec(),
                start_sector,
                start_head,
                start_cylinder,
                start_lba,
                byte_offset,
                info,
            )),
            Err(error) => candidate_errors.push(format!(
                "分区项 {entry_index}（偏移 0x{byte_offset:X}）: {error}"
            )),
        }
    }
    if candidates.is_empty() {
        let details = if candidate_errors.is_empty() {
            String::new()
        } else {
            format!("；{}", candidate_errors.join("；"))
        };
        return Err(format!(
            "{}: 未找到可用的 PC-98 FAT12 分区{details}",
            path.display()
        ));
    }
    if candidates.len() > 1 {
        return Err(format!(
            "{}: 找到 {} 个 FAT12 分区，当前版本拒绝猜测目标分区",
            path.display(),
            candidates.len()
        ));
    }
    let (
        entry_index,
        entry_offset,
        raw_entry,
        start_sector,
        start_head,
        start_cylinder,
        start_lba,
        partition_offset,
        info,
    ) = candidates.pop().expect("one partition candidate");
    let partition_bytes = usize::try_from(info.total_sectors)
        .map_err(|_| "分区扇区数过大".to_string())?
        .checked_mul(usize::from(info.bytes_per_sector))
        .ok_or_else(|| "FAT12 分区大小溢出".to_string())?;
    let partition_end = partition_offset
        .checked_add(partition_bytes)
        .ok_or_else(|| "FAT12 分区范围溢出".to_string())?;

    let fat_offset = partition_offset
        .checked_add(usize::from(info.reserved_sectors) * usize::from(info.bytes_per_sector))
        .ok_or_else(|| "FAT 偏移溢出".to_string())?;
    let fat_bytes = usize::from(info.sectors_per_fat)
        .checked_mul(usize::from(info.bytes_per_sector))
        .ok_or_else(|| "FAT 大小溢出".to_string())?;
    let fat = bytes
        .get(fat_offset..fat_offset + fat_bytes)
        .ok_or_else(|| "第一份 FAT12 被截断".to_string())?;
    let max_cluster =
        u16::try_from(info.data_clusters + 1).map_err(|_| "FAT12 最大簇号溢出".to_string())?;
    let cluster_bytes =
        usize::try_from(info.cluster_bytes).map_err(|_| "FAT12 簇大小过大".to_string())?;
    let mut parser = FatParser {
        image: bytes,
        partition_offset,
        partition_end,
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
    let root_sector = u32::from(info.reserved_sectors)
        .checked_add(u32::from(info.fat_copies) * u32::from(info.sectors_per_fat))
        .ok_or_else(|| "根目录扇区偏移溢出".to_string())?;
    let root_offset = partition_offset
        .checked_add(
            usize::try_from(root_sector).map_err(|_| "根目录扇区偏移过大".to_string())?
                * usize::from(info.bytes_per_sector),
        )
        .ok_or_else(|| "根目录字节偏移溢出".to_string())?;
    let mut root_entries = Vec::with_capacity(usize::from(info.root_entries));
    for index in 0..usize::from(info.root_entries) {
        root_entries.push(read_raw_dir_entry(
            bytes,
            root_offset + index * DIRECTORY_ENTRY_BYTES,
        )?);
    }
    parser.parse_directory(root_entries, "", 0)?;
    let orphan_clusters = (2..=max_cluster)
        .filter(|cluster| {
            fat12_next(fat, *cluster)
                .is_ok_and(|next| next != 0 && !parser.owners.contains_key(cluster))
        })
        .count() as u64;

    let source_file = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("输入文件名无法表示为 Unicode: {}", path.display()))?
        .to_string();
    let manifest = HdiManifest {
        _format: MANIFEST_FORMAT.to_string(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        source_file: source_file.clone(),
        source_sha256: sha256_hex(bytes),
        source_bytes: bytes.len() as u64,
        role_paths: RolePaths {
            source_hdi: source_file,
            unpacked_root: ".".to_string(),
        },
        hdi,
        partition: PartitionInfo {
            table_offset: table_offset as u64,
            entry_index: entry_index as u32,
            entry_offset: entry_offset as u64,
            raw_entry_hex: hex_upper(&raw_entry),
            start_sector,
            start_head,
            start_cylinder,
            start_lba,
            byte_offset: partition_offset as u64,
        },
        fat12: info,
        volume_labels: parser.volume_labels,
        directories: parser.directories,
        files: parser.files,
        skipped_deleted_entries: parser.skipped_deleted_entries,
        skipped_lfn_entries: parser.skipped_lfn_entries,
        orphan_clusters,
    };
    Ok(ParsedImage {
        source_path: path.to_path_buf(),
        manifest,
        extracted_files: parser.extracted_files,
    })
}

fn validate_hdi_header(path: &Path, bytes: &[u8], hdi: &HdiHeader) -> Result<()> {
    if hdi.header_size < 0x20
        || hdi.physical_sector_size == 0
        || hdi.sectors_per_track == 0
        || hdi.heads == 0
        || hdi.cylinders == 0
    {
        return Err(format!("{}: HDI 几何字段为零或头大小无效", path.display()));
    }
    if !hdi.physical_sector_size.is_power_of_two()
        || !(128..=4096).contains(&hdi.physical_sector_size)
    {
        return Err(format!(
            "{}: HDI 物理扇区大小无效: {}",
            path.display(),
            hdi.physical_sector_size
        ));
    }
    let geometry = [
        hdi.physical_sector_size,
        hdi.sectors_per_track,
        hdi.heads,
        hdi.cylinders,
    ]
    .into_iter()
    .try_fold(1u64, |product, value| {
        product
            .checked_mul(u64::from(value))
            .ok_or_else(|| "HDI 几何容量溢出".to_string())
    })?;
    if geometry != u64::from(hdi.disk_size) {
        return Err(format!(
            "{}: HDI 几何容量 0x{geometry:X} 与头部磁盘大小 0x{:X} 不一致",
            path.display(),
            hdi.disk_size
        ));
    }
    let declared = u64::from(hdi.header_size) + u64::from(hdi.disk_size);
    if declared != bytes.len() as u64 {
        return Err(format!(
            "{}: HDI 声明长度 0x{declared:X} 与实际长度 0x{:X} 不一致",
            path.display(),
            bytes.len()
        ));
    }
    Ok(())
}

fn parse_fat12_info(bytes: &[u8], partition_offset: usize, hdi: &HdiHeader) -> Result<Fat12Info> {
    let boot = bytes
        .get(partition_offset..partition_offset + 64)
        .ok_or_else(|| "分区引导扇区被截断".to_string())?;
    let bytes_per_sector = read_u16(boot, 11)?;
    let sectors_per_cluster = boot[13];
    let reserved_sectors = read_u16(boot, 14)?;
    let fat_copies = boot[16];
    let root_entries = read_u16(boot, 17)?;
    let total_sectors_16 = read_u16(boot, 19)?;
    let media_descriptor = boot[21];
    let sectors_per_fat = read_u16(boot, 22)?;
    let sectors_per_track = read_u16(boot, 24)?;
    let heads = read_u16(boot, 26)?;
    let hidden_sectors = read_u32(boot, 28)?;
    let total_sectors = if total_sectors_16 == 0 {
        read_u32(boot, 32)?
    } else {
        u32::from(total_sectors_16)
    };
    if !(128..=4096).contains(&bytes_per_sector) || !bytes_per_sector.is_power_of_two() {
        return Err(format!("非法 BPB 扇区大小: {bytes_per_sector}"));
    }
    if sectors_per_cluster == 0 || !sectors_per_cluster.is_power_of_two() {
        return Err(format!("非法 BPB 每簇扇区数: {sectors_per_cluster}"));
    }
    if reserved_sectors == 0
        || fat_copies == 0
        || root_entries == 0
        || sectors_per_fat == 0
        || total_sectors == 0
    {
        return Err("FAT12 BPB 必需字段为零".to_string());
    }
    if u32::from(sectors_per_track) != hdi.sectors_per_track || u32::from(heads) != hdi.heads {
        return Err("HDI 几何与 FAT BPB 的磁头/每磁道扇区数不一致".to_string());
    }
    let partition_bytes = u64::from(total_sectors)
        .checked_mul(u64::from(bytes_per_sector))
        .ok_or_else(|| "FAT12 分区容量溢出".to_string())?;
    if partition_offset as u64 + partition_bytes > bytes.len() as u64 {
        return Err("FAT12 分区超出 HDI".to_string());
    }
    let root_directory_sectors = (u32::from(root_entries) * DIRECTORY_ENTRY_BYTES as u32)
        .div_ceil(u32::from(bytes_per_sector));
    let first_data_sector = u32::from(reserved_sectors)
        .checked_add(u32::from(fat_copies) * u32::from(sectors_per_fat))
        .and_then(|value| value.checked_add(root_directory_sectors))
        .ok_or_else(|| "FAT12 元数据布局溢出".to_string())?;
    if first_data_sector >= total_sectors {
        return Err("FAT12 元数据占满整个分区".to_string());
    }
    let data_clusters = (total_sectors - first_data_sector) / u32::from(sectors_per_cluster);
    if data_clusters == 0 || data_clusters > FAT12_MAX_CLUSTERS {
        return Err(format!("数据簇数量 {data_clusters} 不属于 FAT12"));
    }
    let cluster_bytes = u32::from(bytes_per_sector)
        .checked_mul(u32::from(sectors_per_cluster))
        .ok_or_else(|| "FAT12 簇大小溢出".to_string())?;
    let fat_bytes = usize::from(sectors_per_fat)
        .checked_mul(usize::from(bytes_per_sector))
        .ok_or_else(|| "FAT12 表大小溢出".to_string())?;
    let fat_offset = partition_offset
        .checked_add(usize::from(reserved_sectors) * usize::from(bytes_per_sector))
        .ok_or_else(|| "FAT12 表偏移溢出".to_string())?;
    let fat = bytes
        .get(fat_offset..fat_offset + fat_bytes)
        .ok_or_else(|| "第一份 FAT12 被截断".to_string())?;
    if fat.len() < 3 || fat[0] < 0xF0 || fat[1] != 0xFF || (fat[2] & 0x0F) != 0x0F {
        return Err("FAT12 保留项无效".to_string());
    }
    let fat_capacity = fat_bytes * 2 / 3;
    if fat_capacity
        <= usize::try_from(data_clusters + 1).map_err(|_| "FAT12 簇数过大".to_string())?
    {
        return Err("FAT12 表容量不足".to_string());
    }
    let mut fat_mismatch_bytes = 0u64;
    for copy_index in 1..usize::from(fat_copies) {
        let copy_offset = fat_offset
            .checked_add(copy_index * fat_bytes)
            .ok_or_else(|| "FAT12 副本偏移溢出".to_string())?;
        let copy = bytes
            .get(copy_offset..copy_offset + fat_bytes)
            .ok_or_else(|| "FAT12 副本被截断".to_string())?;
        fat_mismatch_bytes += fat
            .iter()
            .zip(copy)
            .filter(|(left, right)| left != right)
            .count() as u64;
    }
    Ok(Fat12Info {
        bytes_per_sector,
        sectors_per_cluster,
        reserved_sectors,
        fat_copies,
        root_entries,
        total_sectors,
        media_descriptor,
        fat_media_descriptor: fat[0],
        media_descriptor_matches_fat: fat[0] == media_descriptor,
        sectors_per_fat,
        sectors_per_track,
        heads,
        hidden_sectors,
        root_directory_sectors,
        first_data_sector,
        data_clusters,
        cluster_bytes,
        fat_copies_identical: fat_mismatch_bytes == 0,
        fat_mismatch_bytes,
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
            return Err(format!("目录嵌套超过 64 层: {prefix}"));
        }
        let mut local_raw_names = HashSet::new();
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
            let mut raw_name = [0u8; 11];
            raw_name.copy_from_slice(&entry[..11]);
            if !local_raw_names.insert(raw_name) {
                return Err(format!(
                    "目录 {prefix:?} 中存在重复短名 {}",
                    hex_upper(&raw_name)
                ));
            }
            let decoded_name = decode_short_name(&raw_name);
            if matches!(decoded_name.as_deref(), Some("." | "..")) {
                continue;
            }
            if attributes & 0x08 != 0 {
                self.volume_labels.push(
                    decoded_name.unwrap_or_else(|| format!("__raw_{}", hex_upper(&raw_name))),
                );
                continue;
            }
            let (host_name, decoded_name) = safe_host_name(&raw_name, decoded_name);
            let relative_path = if prefix.is_empty() {
                host_name
            } else {
                format!("{prefix}/{host_name}")
            };
            let collision_key = relative_path.to_uppercase();
            if !self.seen_paths.insert(collision_key) {
                return Err(format!("输出路径发生大小写不敏感冲突: {relative_path}"));
            }
            let start_cluster = read_u16(entry, 26)?;
            let size = read_u32(entry, 28)?;
            let dos_time = read_u16(entry, 22)?;
            let dos_date = read_u16(entry, 24)?;
            let raw_short_name_hex = hex_upper(&raw_name);

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
                    decoded_name,
                    raw_short_name_hex,
                    attributes,
                    directory_entry_offset: raw.image_offset as u64,
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
                let sha256 = sha256_hex(&data);
                self.files.push(FileManifest {
                    path: relative_path.clone(),
                    decoded_name,
                    raw_short_name_hex,
                    attributes,
                    directory_entry_offset: raw.image_offset as u64,
                    start_cluster,
                    size,
                    cluster_chain: chain,
                    dos_time,
                    dos_date,
                    sha256,
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
                return Err(format!("{owner}: FAT 链长度超过分区容量"));
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
            for local_offset in (0..self.cluster_bytes).step_by(DIRECTORY_ENTRY_BYTES) {
                entries.push(read_raw_dir_entry(self.image, offset + local_offset)?);
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
            data.extend_from_slice(&self.image[offset..end]);
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
        let sector = u32::from(cluster - 2)
            .checked_mul(u32::from(self.info.sectors_per_cluster))
            .and_then(|value| value.checked_add(self.info.first_data_sector))
            .ok_or_else(|| format!("簇 {cluster} 的扇区偏移溢出"))?;
        let offset = usize::try_from(sector)
            .map_err(|_| format!("簇 {cluster} 的扇区偏移过大"))?
            .checked_mul(usize::from(self.info.bytes_per_sector))
            .and_then(|value| value.checked_add(self.partition_offset))
            .ok_or_else(|| format!("簇 {cluster} 的字节偏移溢出"))?;
        let end = offset
            .checked_add(self.cluster_bytes)
            .ok_or_else(|| format!("簇 {cluster} 的字节范围溢出"))?;
        if end > self.partition_end {
            return Err(format!("簇 {cluster} 超出 FAT12 分区"));
        }
        Ok(offset)
    }
}

fn write_prepared(prepared: PreparedUnpack) -> Result<UnpackReport> {
    let current_state = capture_output_state(&prepared.output, prepared.overwrite)?;
    if current_state != prepared.output_state {
        return Err("输出目录在预检后发生变化，请重新执行预检".to_string());
    }
    let output_parent = prepared
        .output
        .parent()
        .ok_or_else(|| format!("输出目录没有父目录: {}", prepared.output.display()))?;
    fs::create_dir_all(output_parent)
        .map_err(|error| format!("无法创建输出父目录 {}: {error}", output_parent.display()))?;
    let output_name = prepared
        .output
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            format!(
                "输出目录名无法表示为 Unicode: {}",
                prepared.output.display()
            )
        })?;
    let staging = unique_sibling(
        output_parent,
        &format!(".{output_name}.tmp-{}", std::process::id()),
    )?;
    fs::create_dir(&staging)
        .map_err(|error| format!("无法创建临时输出 {}: {error}", staging.display()))?;

    let write_result = (|| -> Result<()> {
        let mut directories: Vec<&DirectoryManifest> =
            prepared.parsed.manifest.directories.iter().collect();
        directories.sort_by_key(|entry| entry.path.matches('/').count());
        for directory in directories {
            let output_path = join_manifest_path(&staging, &directory.path)?;
            fs::create_dir_all(&output_path)
                .map_err(|error| format!("无法创建目录 {}: {error}", output_path.display()))?;
        }
        for file in &prepared.parsed.extracted_files {
            let output_path = join_manifest_path(&staging, &file.relative_path)?;
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("无法创建目录 {}: {error}", parent.display()))?;
            }
            let mut handle = fs::File::create(&output_path)
                .map_err(|error| format!("无法创建文件 {}: {error}", output_path.display()))?;
            handle
                .write_all(&file.data)
                .map_err(|error| format!("无法写入文件 {}: {error}", output_path.display()))?;
        }
        let mut manifest_json = serde_json::to_vec_pretty(&prepared.parsed.manifest)
            .map_err(|error| format!("无法序列化 {MANIFEST_FILENAME}: {error}"))?;
        manifest_json.push(b'\n');
        let manifest_path = staging.join(MANIFEST_FILENAME);
        fs::write(&manifest_path, manifest_json)
            .map_err(|error| format!("无法写入 {}: {error}", manifest_path.display()))?;
        verify_staging(&staging, &prepared.parsed.manifest)?;
        commit_staging(&staging, &prepared.output, prepared.overwrite)
    })();
    if write_result.is_err() && staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    write_result?;

    let manifest = prepared.output.join(MANIFEST_FILENAME);
    let mut warnings = Vec::new();
    if !prepared.parsed.manifest.fat12.media_descriptor_matches_fat {
        warnings.push(format!(
            "BPB 介质字节为 0x{:02X}，FAT 保留项为 0x{:02X}；已记录该源盘差异",
            prepared.parsed.manifest.fat12.media_descriptor,
            prepared.parsed.manifest.fat12.fat_media_descriptor
        ));
    }
    if prepared.parsed.manifest.fat12.fat_mismatch_bytes != 0 {
        warnings.push(format!(
            "FAT 副本有 {} 个不一致字节；文件读取使用第一份 FAT",
            prepared.parsed.manifest.fat12.fat_mismatch_bytes
        ));
    }
    if prepared.parsed.manifest.orphan_clusters != 0 {
        warnings.push(format!(
            "发现 {} 个已分配但未被活动目录树引用的簇，未把它们伪装成文件",
            prepared.parsed.manifest.orphan_clusters
        ));
    }
    Ok(UnpackReport {
        files: prepared.parsed.manifest.files.len(),
        directories: prepared.parsed.manifest.directories.len(),
        extracted_bytes: prepared
            .parsed
            .manifest
            .files
            .iter()
            .map(|file| u64::from(file.size))
            .sum(),
        orphan_clusters: prepared.parsed.manifest.orphan_clusters,
        fat_mismatch_bytes: prepared.parsed.manifest.fat12.fat_mismatch_bytes,
        output_root: prepared.output,
        manifest,
        warnings,
    })
}

fn write_prepared_hdi_pack(mut prepared: PreparedHdiPack) -> Result<HdiPackReport> {
    let output_parent = prepared
        .output
        .parent()
        .ok_or_else(|| format!("输出 HDI 没有父目录: {}", prepared.output.display()))?;
    fs::create_dir_all(output_parent).map_err(|error| {
        format!(
            "无法创建输出 HDI 父目录 {}: {error}",
            output_parent.display()
        )
    })?;
    apply_hdi_pack_updates(&mut prepared.image, &prepared.manifest, &prepared.updates)?;
    let parsed = parse_hdi(&prepared.output, &prepared.image)
        .map_err(|error| format!("回封后 HDI 复核失败: {error}"))?;
    for update in &prepared.updates {
        let file = parsed
            .manifest
            .files
            .iter()
            .find(|file| file.path == update.manifest_path)
            .ok_or_else(|| format!("回封后缺少文件: {}", update.manifest_path))?;
        let data = parsed
            .extracted_files
            .iter()
            .find(|file| file.relative_path == update.manifest_path)
            .map(|file| file.data.as_slice())
            .ok_or_else(|| format!("回封后无法读取文件: {}", update.manifest_path))?;
        if data != update.data.as_slice() || file.size as usize != update.data.len() {
            return Err(format!("回封后文件内容复核失败: {}", update.manifest_path));
        }
    }
    let output_name = prepared
        .output
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            format!(
                "输出 HDI 文件名无法表示为 Unicode: {}",
                prepared.output.display()
            )
        })?;
    let staging = unique_sibling(
        output_parent,
        &format!(
            ".{output_name}.platinum-star-hdi.tmp-{}",
            std::process::id()
        ),
    )?;
    let backup = unique_sibling(
        output_parent,
        &format!(
            ".{output_name}.platinum-star-hdi.backup-{}",
            std::process::id()
        ),
    )?;
    let write_result = (|| -> Result<()> {
        let mut file = fs::File::create(&staging)
            .map_err(|error| format!("无法创建 HDI 临时文件 {}: {error}", staging.display()))?;
        file.write_all(&prepared.image)
            .and_then(|_| file.sync_all())
            .map_err(|error| format!("无法写入 HDI 临时文件 {}: {error}", staging.display()))?;
        let written = fs::read(&staging)
            .map_err(|error| format!("无法复核 HDI 临时文件 {}: {error}", staging.display()))?;
        if written != prepared.image {
            return Err("HDI 临时文件逐字节复核失败".into());
        }
        if prepared.output.exists() {
            if !prepared.overwrite {
                return Err(format!("输出 HDI 已存在: {}", prepared.output.display()));
            }
            fs::rename(&prepared.output, &backup).map_err(|error| {
                format!("无法把旧 HDI 移到事务备份 {}: {error}", backup.display())
            })?;
        }
        if let Err(error) = fs::rename(&staging, &prepared.output) {
            if backup.exists() {
                let _ = fs::rename(&backup, &prepared.output);
            }
            return Err(format!("无法提交新 HDI: {error}"));
        }
        if backup.exists() {
            fs::remove_file(&backup)
                .map_err(|error| format!("无法清理旧 HDI 备份 {}: {error}", backup.display()))?;
        }
        Ok(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&staging);
    }
    write_result?;
    Ok(HdiPackReport {
        output: prepared.output,
        source_files: prepared.inspection.source_files,
        changed_files: prepared.inspection.changed_files,
        changed_bytes: prepared.inspection.changed_bytes,
        output_sha256: sha256_hex(&prepared.image),
        warnings: prepared.warnings,
    })
}

fn snapshot_pack_input(root: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    fn visit(root: &Path, relative: &Path, files: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
        let directory = root.join(relative);
        let mut children = fs::read_dir(&directory)
            .map_err(|error| format!("无法读取回注目录 {}: {error}", directory.display()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| format!("无法枚举回注目录 {}: {error}", directory.display()))?;
        children.sort_by_key(|entry| entry.file_name());
        for child in children {
            let child_relative = relative.join(child.file_name());
            let child_path = root.join(&child_relative);
            let metadata = fs::symlink_metadata(&child_path).map_err(|error| {
                format!("无法读取回注成员元数据 {}: {error}", child_path.display())
            })?;
            if metadata.file_type().is_symlink() {
                return Err(format!(
                    "回注目录包含符号链接，拒绝跟随: {}",
                    child_path.display()
                ));
            }
            if metadata.is_dir() {
                visit(root, &child_relative, files)?;
            } else if metadata.is_file() {
                let key = child_relative.to_string_lossy().replace('\\', "/");
                if key == MANIFEST_FILENAME
                    || key == ".platinum_star_import_manifest.json"
                    || key == "font_mapping.json"
                {
                    continue;
                }
                files.insert(
                    key,
                    fs::read(&child_path).map_err(|error| {
                        format!("无法读取回注文件 {}: {error}", child_path.display())
                    })?,
                );
            } else {
                return Err(format!(
                    "回注目录包含不支持的成员类型: {}",
                    child_path.display()
                ));
            }
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    visit(root, Path::new(""), &mut files)?;
    Ok(files)
}

fn manifest_prefixes(manifest: &HdiManifest) -> BTreeSet<String> {
    let mut prefixes = BTreeSet::from([String::new()]);
    for file in &manifest.files {
        let components = file.path.split('/').collect::<Vec<_>>();
        for length in 1..components.len() {
            prefixes.insert(components[..length].join("/"));
        }
    }
    prefixes
}

fn is_pack_sidecar(relative: &str) -> bool {
    relative == "font.bmp"
        || relative == "font.tmp"
        || relative == "font_mapping.json"
        || relative == ".platinum_star_import_manifest.json"
        || relative == MANIFEST_FILENAME
}

fn free_clusters_after_release(
    image: &[u8],
    manifest: &HdiManifest,
    updates: &[PackUpdate],
) -> Result<usize> {
    let fat_offset = fat_offset(manifest)?;
    let fat_bytes = fat_bytes(manifest)?;
    let mut fat = image
        .get(fat_offset..fat_offset + fat_bytes)
        .ok_or_else(|| "FAT12 表被截断".to_string())?
        .to_vec();
    for update in updates {
        for &cluster in &update.old_chain {
            set_fat12(&mut fat, cluster, 0)?;
        }
    }
    let max_cluster = u16::try_from(manifest.fat12.data_clusters + 1)
        .map_err(|_| "FAT12 最大簇号溢出".to_string())?;
    (2..=max_cluster)
        .map(|cluster| fat12_next(&fat, cluster))
        .collect::<Result<Vec<_>>>()
        .map(|values| values.into_iter().filter(|value| *value == 0).count())
}

fn apply_hdi_pack_updates(
    image: &mut [u8],
    manifest: &HdiManifest,
    updates: &[PackUpdate],
) -> Result<()> {
    if updates.is_empty() {
        return Ok(());
    }
    let fat_offset = fat_offset(manifest)?;
    let fat_bytes = fat_bytes(manifest)?;
    let mut fat = image
        .get(fat_offset..fat_offset + fat_bytes)
        .ok_or_else(|| "FAT12 表被截断".to_string())?
        .to_vec();
    for update in updates {
        for &cluster in &update.old_chain {
            set_fat12(&mut fat, cluster, 0)?;
        }
    }
    let max_cluster = u16::try_from(manifest.fat12.data_clusters + 1)
        .map_err(|_| "FAT12 最大簇号溢出".to_string())?;
    let cluster_bytes = usize::try_from(manifest.fat12.cluster_bytes)
        .map_err(|_| "FAT12 簇大小过大".to_string())?;
    let available = (2..=max_cluster)
        .filter(|cluster| fat12_next(&fat, *cluster).is_ok_and(|value| value == 0))
        .collect::<Vec<_>>();
    let mut cursor = 0usize;
    for update in updates {
        let count = update.data.len().div_ceil(cluster_bytes);
        if cursor + count > available.len() {
            return Err(format!(
                "回封后的文件需要 {} 个新簇，但只剩 {} 个可用簇: {}",
                count,
                available.len().saturating_sub(cursor),
                update.manifest_path
            ));
        }
        let chain = &available[cursor..cursor + count];
        cursor += count;
        for (index, &cluster) in chain.iter().enumerate() {
            let next = chain.get(index + 1).copied().unwrap_or(0xFFF);
            set_fat12(&mut fat, cluster, next)?;
            let offset = cluster_offset(manifest, cluster)?;
            let end = offset
                .checked_add(cluster_bytes)
                .ok_or_else(|| "簇写入范围溢出".to_string())?;
            let destination = image
                .get_mut(offset..end)
                .ok_or_else(|| format!("簇 {cluster} 超出 HDI 分区"))?;
            destination.fill(0);
            let start = index * cluster_bytes;
            let length = update.data.len().saturating_sub(start).min(cluster_bytes);
            destination[..length].copy_from_slice(&update.data[start..start + length]);
        }
        let entry = image
            .get_mut(
                update.directory_entry_offset
                    ..update.directory_entry_offset + DIRECTORY_ENTRY_BYTES,
            )
            .ok_or_else(|| format!("目录项被截断: {}", update.manifest_path))?;
        let start_cluster = chain.first().copied().unwrap_or(0);
        entry[26..28].copy_from_slice(&start_cluster.to_le_bytes());
        let size = u32::try_from(update.data.len())
            .map_err(|_| format!("文件过大，无法写入 FAT12 目录项: {}", update.manifest_path))?;
        entry[28..32].copy_from_slice(&size.to_le_bytes());
    }
    for copy_index in 0..usize::from(manifest.fat12.fat_copies) {
        let offset = fat_offset
            .checked_add(copy_index * fat_bytes)
            .ok_or_else(|| "FAT 副本偏移溢出".to_string())?;
        let destination = image
            .get_mut(offset..offset + fat_bytes)
            .ok_or_else(|| "FAT 副本被截断".to_string())?;
        destination.copy_from_slice(&fat);
    }
    Ok(())
}

fn fat_offset(manifest: &HdiManifest) -> Result<usize> {
    usize::try_from(manifest.partition.byte_offset)
        .ok()
        .and_then(|offset| {
            offset.checked_add(
                usize::from(manifest.fat12.reserved_sectors)
                    * usize::from(manifest.fat12.bytes_per_sector),
            )
        })
        .ok_or_else(|| "FAT 偏移溢出".to_string())
}

fn fat_bytes(manifest: &HdiManifest) -> Result<usize> {
    usize::from(manifest.fat12.sectors_per_fat)
        .checked_mul(usize::from(manifest.fat12.bytes_per_sector))
        .ok_or_else(|| "FAT 大小溢出".to_string())
}

fn cluster_offset(manifest: &HdiManifest, cluster: u16) -> Result<usize> {
    if cluster < 2 || u32::from(cluster) > manifest.fat12.data_clusters + 1 {
        return Err(format!("簇号无效: {cluster}"));
    }
    let sector = manifest
        .fat12
        .first_data_sector
        .checked_add(u32::from(cluster - 2) * u32::from(manifest.fat12.sectors_per_cluster))
        .ok_or_else(|| "簇扇区偏移溢出".to_string())?;
    usize::try_from(manifest.partition.byte_offset)
        .ok()
        .and_then(|offset| {
            offset.checked_add(
                usize::try_from(sector).ok()? * usize::from(manifest.fat12.bytes_per_sector),
            )
        })
        .ok_or_else(|| "簇字节偏移溢出".to_string())
}

fn set_fat12(fat: &mut [u8], cluster: u16, value: u16) -> Result<()> {
    if value > 0xFFF {
        return Err(format!("FAT12 值超出范围: {value:X}"));
    }
    let offset = usize::from(cluster) * 3 / 2;
    let pair = fat
        .get_mut(offset..offset + 2)
        .ok_or_else(|| format!("FAT12 项 {cluster} 超出表范围"))?;
    if cluster & 1 == 0 {
        pair[0] = value as u8;
        pair[1] = (pair[1] & 0xF0) | ((value >> 8) as u8 & 0x0F);
    } else {
        pair[0] = (pair[0] & 0x0F) | ((value << 4) as u8 & 0xF0);
        pair[1] = (value >> 4) as u8;
    }
    Ok(())
}

fn verify_staging(staging: &Path, manifest: &HdiManifest) -> Result<()> {
    for file in &manifest.files {
        let path = join_manifest_path(staging, &file.path)?;
        let data = fs::read(&path)
            .map_err(|error| format!("写出复核无法读取 {}: {error}", path.display()))?;
        if data.len() != file.size as usize || sha256_hex(&data) != file.sha256 {
            return Err(format!("写出复核失败: {}", file.path));
        }
    }
    Ok(())
}

fn commit_staging(staging: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if !output.exists() {
        return fs::rename(staging, output).map_err(|error| {
            format!(
                "无法把临时输出 {} 提交为 {}: {error}",
                staging.display(),
                output.display()
            )
        });
    }
    if !overwrite {
        return Err(format!("输出目录已存在: {}", output.display()));
    }
    let parent = output
        .parent()
        .ok_or_else(|| "输出目录没有父目录".to_string())?;
    let backup = unique_sibling(
        parent,
        &format!(".hdi-unpack-backup-{}", std::process::id()),
    )?;
    fs::rename(output, &backup).map_err(|error| {
        format!(
            "无法把旧输出 {} 移到备份 {}: {error}",
            output.display(),
            backup.display()
        )
    })?;
    if let Err(error) = fs::rename(staging, output) {
        let rollback = fs::rename(&backup, output);
        return match rollback {
            Ok(()) => Err(format!("提交新输出失败，已恢复旧输出: {error}")),
            Err(rollback_error) => Err(format!(
                "提交新输出失败且旧输出恢复失败；备份位于 {}: {error}; {rollback_error}",
                backup.display()
            )),
        };
    }
    fs::remove_dir_all(&backup).map_err(|error| {
        format!(
            "新输出已提交，但无法清理旧输出备份 {}: {error}",
            backup.display()
        )
    })?;
    Ok(())
}

fn capture_output_state(output: &Path, overwrite: bool) -> Result<OutputState> {
    if !output.exists() {
        return Ok(OutputState::Missing);
    }
    let metadata = fs::symlink_metadata(output)
        .map_err(|error| format!("无法读取输出元数据 {}: {error}", output.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("输出必须是非符号链接目录: {}", output.display()));
    }
    if !overwrite {
        return Err(format!(
            "输出目录已存在；请改用新目录或显式指定 --overwrite: {}",
            output.display()
        ));
    }
    let mut entries = fs::read_dir(output)
        .map_err(|error| format!("无法读取输出目录 {}: {error}", output.display()))?;
    if entries.next().is_some() {
        let manifest_path = output.join(MANIFEST_FILENAME);
        let manifest_bytes = fs::read(&manifest_path).map_err(|_| {
            format!(
                "拒绝覆盖非空且不含有效 {MANIFEST_FILENAME} 的目录: {}",
                output.display()
            )
        })?;
        let value: serde_json::Value = serde_json::from_slice(&manifest_bytes)
            .map_err(|error| format!("现有 {MANIFEST_FILENAME} 无效，拒绝覆盖: {error}"))?;
        if value.get("_format").and_then(|item| item.as_str()) != Some(MANIFEST_FORMAT) {
            return Err(format!(
                "现有 {MANIFEST_FILENAME} 不属于本工具，拒绝覆盖: {}",
                output.display()
            ));
        }
    }
    Ok(OutputState::Existing(directory_fingerprint(output)?))
}

fn directory_fingerprint(root: &Path) -> Result<String> {
    fn visit(root: &Path, relative: &Path, records: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
        let directory = root.join(relative);
        let mut children = fs::read_dir(&directory)
            .map_err(|error| format!("无法读取目录 {}: {error}", directory.display()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| format!("无法枚举目录 {}: {error}", directory.display()))?;
        children.sort_by_key(|entry| entry.file_name());
        for child in children {
            let child_relative = relative.join(child.file_name());
            let child_path = root.join(&child_relative);
            let metadata = fs::symlink_metadata(&child_path)
                .map_err(|error| format!("无法读取 {} 的元数据: {error}", child_path.display()))?;
            if metadata.file_type().is_symlink() {
                return Err(format!(
                    "输出工作区不能含符号链接: {}",
                    child_path.display()
                ));
            }
            let key = child_relative.to_string_lossy().replace('\\', "/");
            if metadata.is_dir() {
                records.insert(format!("D:{key}"), Vec::new());
                visit(root, &child_relative, records)?;
            } else if metadata.is_file() {
                let data = fs::read(&child_path)
                    .map_err(|error| format!("无法读取 {}: {error}", child_path.display()))?;
                records.insert(format!("F:{key}"), data);
            } else {
                return Err(format!(
                    "输出工作区含不支持的文件类型: {}",
                    child_path.display()
                ));
            }
        }
        Ok(())
    }

    let mut records = BTreeMap::new();
    visit(root, Path::new(""), &mut records)?;
    let mut hasher = Sha256::new();
    for (key, data) in records {
        hasher.update((key.len() as u64).to_le_bytes());
        hasher.update(key.as_bytes());
        hasher.update((data.len() as u64).to_le_bytes());
        hasher.update(&data);
    }
    Ok(hex_upper(&hasher.finalize()))
}

fn validate_output_root(output: &Path) -> Result<()> {
    if output.as_os_str().is_empty() {
        return Err("输出目录不能为空".to_string());
    }
    let mut normal_components = 0usize;
    for component in output.components() {
        match component {
            Component::ParentDir => return Err("输出目录不能包含 ..".to_string()),
            Component::Normal(_) => normal_components += 1,
            _ => {}
        }
    }
    if normal_components == 0 {
        return Err(format!(
            "拒绝把文件系统根目录作为输出: {}",
            output.display()
        ));
    }
    Ok(())
}

fn absolute_lexical(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path)
            .map_err(|error| format!("无法解析路径 {}: {error}", path.display()));
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("路径没有父目录: {}", path.display()))?;
    let parent_absolute = if parent.exists() {
        fs::canonicalize(parent)
            .map_err(|error| format!("无法解析父目录 {}: {error}", parent.display()))?
    } else if parent.is_absolute() {
        parent.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| format!("无法读取当前目录: {error}"))?
            .join(parent)
    };
    let name = path
        .file_name()
        .ok_or_else(|| format!("输出路径缺少目录名: {}", path.display()))?;
    Ok(parent_absolute.join(name))
}

fn safe_host_name(raw_name: &[u8; 11], decoded: Option<String>) -> (String, Option<String>) {
    let decoded_for_manifest = decoded.clone();
    let host = decoded
        .filter(|name| validate_output_segment(name).is_ok())
        .unwrap_or_else(|| format!("__raw_{}", hex_upper(raw_name)));
    (host, decoded_for_manifest)
}

fn validate_output_segment(segment: &str) -> Result<()> {
    if segment.is_empty() || segment == "." || segment == ".." {
        return Err(format!("不安全的输出路径段: {segment:?}"));
    }
    if segment.ends_with(' ') || segment.ends_with('.') {
        return Err(format!("Windows 输出路径不能以空格或点结尾: {segment}"));
    }
    if segment.chars().any(|character| {
        character < ' '
            || matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            )
    }) {
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

fn decode_short_name(raw_name: &[u8; 11]) -> Option<String> {
    let mut stem = raw_name[..8].to_vec();
    if stem.first() == Some(&0x05) {
        stem[0] = 0xE5;
    }
    while stem.last() == Some(&b' ') {
        stem.pop();
    }
    let mut extension = raw_name[8..].to_vec();
    while extension.last() == Some(&b' ') {
        extension.pop();
    }
    let stem_raw = stem.clone();
    let extension_raw = extension.clone();
    let stem = SHIFT_JIS.decode_without_bom_handling_and_without_replacement(&stem)?;
    let extension = SHIFT_JIS.decode_without_bom_handling_and_without_replacement(&extension)?;
    let (encoded_stem, _, stem_errors) = SHIFT_JIS.encode(&stem);
    let (encoded_extension, _, extension_errors) = SHIFT_JIS.encode(&extension);
    if stem_errors
        || extension_errors
        || encoded_stem.as_ref() != stem_raw
        || encoded_extension.as_ref() != extension_raw
    {
        return None;
    }
    Some(if extension.is_empty() {
        stem.into_owned()
    } else {
        format!("{stem}.{extension}")
    })
}

fn fat12_next(fat: &[u8], cluster: u16) -> Result<u16> {
    let offset = usize::from(cluster) * 3 / 2;
    let pair = fat
        .get(offset..offset + 2)
        .ok_or_else(|| format!("FAT12 项 {cluster} 超出表范围"))?;
    let word = u16::from(pair[0]) | (u16::from(pair[1]) << 8);
    Ok(if cluster & 1 == 0 {
        word & 0x0FFF
    } else {
        (word >> 4) & 0x0FFF
    })
}

fn read_raw_dir_entry(bytes: &[u8], offset: usize) -> Result<RawDirEntry> {
    let slice = bytes
        .get(offset..offset + DIRECTORY_ENTRY_BYTES)
        .ok_or_else(|| format!("目录项在 0x{offset:X} 被截断"))?;
    let mut entry = [0u8; DIRECTORY_ENTRY_BYTES];
    entry.copy_from_slice(slice);
    Ok(RawDirEntry {
        bytes: entry,
        image_offset: offset,
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

fn usize_from_u32(value: u32, label: &str) -> Result<usize> {
    usize::try_from(value).map_err(|_| format!("{label} 超出当前平台范围: {value}"))
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

    const HEADER_SIZE: usize = 0x1000;
    const SECTOR_SIZE: usize = 512;
    const SECTORS_PER_TRACK: usize = 4;
    const HEADS: usize = 2;
    const CYLINDERS: usize = 10;
    const PARTITION_OFFSET: usize = HEADER_SIZE + SECTOR_SIZE * SECTORS_PER_TRACK * HEADS;

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

    fn put_entry(
        bytes: &mut [u8],
        offset: usize,
        name: &[u8; 11],
        attributes: u8,
        cluster: u16,
        size: u32,
    ) {
        bytes[offset..offset + 11].copy_from_slice(name);
        bytes[offset + 11] = attributes;
        put_u16(bytes, offset + 26, cluster);
        put_u32(bytes, offset + 28, size);
    }

    fn sample_hdi() -> Vec<u8> {
        let disk_size = SECTOR_SIZE * SECTORS_PER_TRACK * HEADS * CYLINDERS;
        let mut bytes = vec![0u8; HEADER_SIZE + disk_size];
        put_u32(&mut bytes, 0x08, HEADER_SIZE as u32);
        put_u32(&mut bytes, 0x0C, disk_size as u32);
        put_u32(&mut bytes, 0x10, SECTOR_SIZE as u32);
        put_u32(&mut bytes, 0x14, SECTORS_PER_TRACK as u32);
        put_u32(&mut bytes, 0x18, HEADS as u32);
        put_u32(&mut bytes, 0x1C, CYLINDERS as u32);

        let partition_entry = HEADER_SIZE + SECTOR_SIZE;
        bytes[partition_entry] = 0xA1;
        bytes[partition_entry + 1] = 0x81;
        put_u16(&mut bytes, partition_entry + 10, 1);

        let total_sectors = 64u16;
        let boot = PARTITION_OFFSET;
        bytes[boot..boot + 3].copy_from_slice(&[0xEB, 0x3C, 0x90]);
        bytes[boot + 3..boot + 11].copy_from_slice(b"NEC  5.0");
        put_u16(&mut bytes, boot + 11, SECTOR_SIZE as u16);
        bytes[boot + 13] = 1;
        put_u16(&mut bytes, boot + 14, 1);
        bytes[boot + 16] = 2;
        put_u16(&mut bytes, boot + 17, 32);
        put_u16(&mut bytes, boot + 19, total_sectors);
        bytes[boot + 21] = 0xF8;
        put_u16(&mut bytes, boot + 22, 1);
        put_u16(&mut bytes, boot + 24, SECTORS_PER_TRACK as u16);
        put_u16(&mut bytes, boot + 26, HEADS as u16);
        put_u32(&mut bytes, boot + 28, (SECTORS_PER_TRACK * HEADS) as u32);

        let fat1 = boot + SECTOR_SIZE;
        let mut fat = vec![0u8; SECTOR_SIZE];
        fat[..3].copy_from_slice(&[0xF8, 0xFF, 0xFF]);
        set_fat12(&mut fat, 2, 0xFFF);
        set_fat12(&mut fat, 3, 0xFFF);
        bytes[fat1..fat1 + SECTOR_SIZE].copy_from_slice(&fat);
        bytes[fat1 + SECTOR_SIZE..fat1 + SECTOR_SIZE * 2].copy_from_slice(&fat);

        let root = boot + SECTOR_SIZE * 3;
        put_entry(&mut bytes, root, b"GAME       ", 0x10, 2, 0);
        let data = boot + SECTOR_SIZE * 5;
        put_entry(&mut bytes, data, b".          ", 0x10, 2, 0);
        put_entry(
            &mut bytes,
            data + DIRECTORY_ENTRY_BYTES,
            b"..         ",
            0x10,
            0,
            0,
        );
        put_entry(
            &mut bytes,
            data + DIRECTORY_ENTRY_BYTES * 2,
            b"SCRIPT  DAT",
            0x20,
            3,
            5,
        );
        bytes[data + SECTOR_SIZE..data + SECTOR_SIZE + 5].copy_from_slice(b"hello");
        bytes
    }

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "platinum-star-hdi-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    #[test]
    fn parses_nested_fat12_file() {
        let parsed = parse_hdi(Path::new("star_pt.hdi"), &sample_hdi()).expect("valid HDI");
        assert_eq!(
            parsed.manifest.partition.byte_offset,
            PARTITION_OFFSET as u64
        );
        assert_eq!(parsed.manifest.files.len(), 1);
        assert_eq!(parsed.manifest.files[0].path, "GAME/SCRIPT.DAT");
        assert_eq!(parsed.manifest.files[0].cluster_chain, vec![3]);
        assert_eq!(parsed.extracted_files[0].data, b"hello");
    }

    #[test]
    fn rejects_cluster_cycle() {
        let mut bytes = sample_hdi();
        for fat_index in 0..2 {
            let fat = PARTITION_OFFSET + SECTOR_SIZE * (1 + fat_index);
            set_fat12(&mut bytes[fat..fat + SECTOR_SIZE], 3, 3);
        }
        let error = parse_hdi(Path::new("cycle.hdi"), &bytes).expect_err("cycle must fail");
        assert!(error.contains("形成循环"));
    }

    #[test]
    fn invalid_windows_name_uses_reversible_raw_name() {
        let mut bytes = sample_hdi();
        let directory = PARTITION_OFFSET + SECTOR_SIZE * 5;
        bytes[directory + DIRECTORY_ENTRY_BYTES * 2..directory + DIRECTORY_ENTRY_BYTES * 2 + 8]
            .copy_from_slice(b"BAD\\NAME");
        let parsed = parse_hdi(Path::new("raw.hdi"), &bytes).expect("safe fallback");
        assert!(parsed.manifest.files[0].path.contains("__raw_"));
        assert_eq!(parsed.extracted_files[0].data, b"hello");
    }

    #[test]
    fn writes_transactional_workspace_and_refuses_unmanaged_overwrite() {
        let root = temp_root("write");
        fs::create_dir(&root).expect("create root");
        let source = root.join("star_pt.hdi");
        let output = root.join("unpack");
        fs::write(&source, sample_hdi()).expect("write image");
        let prepared = prepare_unpack(&source, &output, false).expect("prepare");
        let report = prepared.execute().expect("execute");
        assert_eq!(report.files, 1);
        assert_eq!(
            fs::read(output.join("GAME/SCRIPT.DAT")).expect("file"),
            b"hello"
        );
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join(MANIFEST_FILENAME)).expect("manifest"))
                .expect("json");
        assert_eq!(manifest["_format"], MANIFEST_FORMAT);

        let unmanaged = root.join("unmanaged");
        fs::create_dir(&unmanaged).expect("create unmanaged");
        fs::write(unmanaged.join("keep.txt"), b"keep").expect("write unmanaged");
        let error = prepare_unpack(&source, &unmanaged, true).expect_err("must refuse");
        assert!(error.contains("拒绝覆盖"));
        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn repacks_changed_file_into_new_hdi() {
        let root = temp_root("pack");
        fs::create_dir(&root).expect("create root");
        let source = root.join("star_pt.hdi");
        let input = root.join("re");
        let output = root.join("translated.hdi");
        fs::write(&source, sample_hdi()).expect("write image");
        fs::create_dir(&input).expect("create input");
        fs::write(input.join("SCRIPT.DAT"), b"translated text").expect("write input");
        let prepared = prepare_hdi_pack(&source, &input, &output, false).expect("prepare pack");
        assert_eq!(prepared.inspection().changed_files, 1);
        let report = prepared.execute().expect("execute pack");
        assert_eq!(report.changed_files, 1);
        let bytes = fs::read(&output).expect("read output");
        let parsed = parse_hdi(&output, &bytes).expect("parse output");
        assert_eq!(parsed.extracted_files[0].data, b"translated text");
        fs::remove_dir_all(&root).expect("cleanup");
    }
}
