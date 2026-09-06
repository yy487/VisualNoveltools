use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::ops::Range;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod ag00;
pub mod font;
pub mod localize;
pub mod script;

const NFD_R0_SIGNATURE: &[u8; 14] = b"T98FDDIMAGE.R0";
const NFD_R1_SIGNATURE: &[u8; 14] = b"T98FDDIMAGE.R1";
const NFD_BASE_HEADER_SIZE: usize = 0x120;
const NFD_HEADER_SIZE_OFFSET: usize = 0x110;
const NFD_WRITE_PROTECT_OFFSET: usize = 0x114;
const NFD_HEADS_OFFSET: usize = 0x115;
const MAP_ENTRY_SIZE: usize = 0x10;
const MAP_ENTRIES_PER_TRACK_SIDE: usize = 26;
const DIRECTORY_ENTRY_SIZE: usize = 16;
const DIRECTORY_SECTORS: u8 = 23;
const FAT_FIRST_SECTOR: u8 = 24;
const FAT_COPIES: usize = 3;
const SECTORS_PER_CLUSTER: u8 = 26;
const DATA_SECTOR_SIZE: usize = 256;
const FAT_RESERVED: u8 = 0xFE;
const FAT_FREE: u8 = 0xFF;
const WORKSPACE_FORMAT: &str = "tauhido-nfd-n88-unpack-workspace-v1";

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NfdManifest {
    pub revision: u8,
    pub header_size: u32,
    pub write_protected: bool,
    pub heads: u8,
    pub map_track_slots: u16,
    pub formatted_track_sides: u16,
    pub sectors: u32,
    pub pda_values: Vec<u8>,
    pub deleted_data_sectors: u32,
    pub sectors_with_read_errors: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemManifest {
    pub kind: String,
    pub directory_track_side: u8,
    pub directory_cylinder: u8,
    pub directory_head: u8,
    pub directory_sectors: u8,
    pub fat_sectors: [u8; FAT_COPIES],
    pub fat_copies_identical: bool,
    pub sectors_per_cluster: u8,
    pub sector_size: u16,
    pub fat_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManifest {
    #[serde(rename = "_index")]
    pub index: usize,
    pub name: String,
    pub path: String,
    pub raw_name_hex: String,
    pub attribute: u8,
    #[serde(rename = "_directory_entry_offset")]
    pub directory_entry_offset: u32,
    #[serde(rename = "_start_cluster")]
    pub start_cluster: u8,
    #[serde(rename = "_clusters")]
    pub clusters: Vec<u8>,
    #[serde(rename = "_final_sectors")]
    pub final_sectors: u8,
    #[serde(rename = "_size")]
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageManifest {
    pub source_file: String,
    pub source_sha256: String,
    pub output_dir: String,
    pub nfd: NfdManifest,
    pub filesystem: FilesystemManifest,
    pub files: Vec<FileManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryManifest {
    pub images: usize,
    pub files: usize,
    pub extracted_bytes: u64,
    pub warnings: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    #[serde(rename = "_format")]
    pub format: String,
    pub images: Vec<ImageManifest>,
    pub summary: SummaryManifest,
}

#[derive(Debug, Clone)]
pub struct UnpackReport {
    pub images: usize,
    pub extracted_files: usize,
    pub extracted_bytes: u64,
    pub warnings: Vec<String>,
    pub output_root: PathBuf,
}

#[derive(Debug, Clone)]
struct Sector {
    c: u8,
    h: u8,
    r: u8,
    n: u8,
    data: Range<usize>,
}

#[derive(Debug, Clone)]
struct TrackSide {
    slot: usize,
    sectors: Vec<Sector>,
}

#[derive(Debug, Clone)]
struct DiskFile {
    index: usize,
    name: String,
    raw_name: [u8; 9],
    attribute: u8,
    directory_entry_offset: usize,
    start_cluster: u8,
    clusters: Vec<u8>,
    final_sectors: u8,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
struct ParsedImage {
    source: Vec<u8>,
    source_name: String,
    nfd: NfdManifest,
    directory_slot: usize,
    directory_c: u8,
    directory_h: u8,
    fat: Vec<u8>,
    tracks: Vec<TrackSide>,
    files: Vec<DiskFile>,
    warnings: Vec<String>,
}

#[derive(Debug)]
struct PreparedImage {
    output_dir: String,
    parsed: ParsedImage,
}

#[derive(Debug, Deserialize)]
struct WorkspaceMarker {
    #[serde(rename = "_format")]
    format: String,
}

pub fn preview_mappings(inputs: &[PathBuf]) -> Result<Vec<(PathBuf, String)>> {
    let resolved = resolve_inputs(inputs)?;
    Ok(resolved
        .into_iter()
        .enumerate()
        .map(|(index, path)| {
            let label = safe_input_label(&path);
            (path, format!("{index:02}_{label}"))
        })
        .collect())
}

pub fn unpack_batch(
    inputs: &[PathBuf],
    output_root: &Path,
    overwrite: bool,
) -> Result<UnpackReport> {
    let resolved = resolve_inputs(inputs)?;
    validate_output_does_not_contain_inputs(&resolved, output_root)?;
    validate_existing_output(output_root, overwrite)?;

    let mut prepared = Vec::with_capacity(resolved.len());
    for (index, path) in resolved.iter().enumerate() {
        let source =
            fs::read(path).map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
        let source_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or_else(|| format!("输入路径没有文件名: {}", path.display()))?;
        let parsed = parse_nfd(source, source_name)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        prepared.push(PreparedImage {
            output_dir: format!("{index:02}_{}", safe_input_label(path)),
            parsed,
        });
    }

    write_workspace(&prepared, output_root, overwrite)
}

fn resolve_inputs(inputs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    if inputs.is_empty() {
        return Err("至少需要一个 NFD 文件或目录".to_string());
    }

    let mut resolved = Vec::new();
    for input in inputs {
        let metadata = fs::metadata(input)
            .map_err(|error| format!("无法访问 {}: {error}", input.display()))?;
        if metadata.is_file() {
            resolved.push(input.clone());
            continue;
        }
        if !metadata.is_dir() {
            return Err(format!("输入不是普通文件或目录: {}", input.display()));
        }

        let mut found = Vec::new();
        for entry in fs::read_dir(input)
            .map_err(|error| format!("无法读取目录 {}: {error}", input.display()))?
        {
            let entry = entry.map_err(|error| format!("读取目录项失败: {error}"))?;
            if !entry
                .file_type()
                .map_err(|error| format!("读取 {} 类型失败: {error}", entry.path().display()))?
                .is_file()
            {
                continue;
            }
            if has_nfd_signature(&entry.path())? {
                found.push(entry.path());
            }
        }
        found.sort_by_key(|path| path.to_string_lossy().to_lowercase());
        if found.is_empty() {
            return Err(format!("目录中没有检测到 NFD 镜像: {}", input.display()));
        }
        resolved.extend(found);
    }

    let mut seen = HashSet::new();
    let mut unique = Vec::with_capacity(resolved.len());
    for path in resolved {
        let canonical = fs::canonicalize(&path)
            .map_err(|error| format!("无法规范化 {}: {error}", path.display()))?;
        let key = canonical.to_string_lossy().to_lowercase();
        if seen.insert(key) {
            unique.push(canonical);
        }
    }
    if unique.is_empty() {
        return Err("没有可处理的 NFD 镜像".to_string());
    }
    Ok(unique)
}

fn has_nfd_signature(path: &Path) -> Result<bool> {
    let mut file =
        fs::File::open(path).map_err(|error| format!("打开 {} 失败: {error}", path.display()))?;
    let mut signature = [0u8; 14];
    match file.read_exact(&mut signature) {
        Ok(()) => Ok(&signature == NFD_R0_SIGNATURE || &signature == NFD_R1_SIGNATURE),
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => Ok(false),
        Err(error) => Err(format!("读取 {} 文件头失败: {error}", path.display())),
    }
}

fn parse_nfd(source: Vec<u8>, source_name: String) -> Result<ParsedImage> {
    if source.len() < NFD_BASE_HEADER_SIZE {
        return Err(format!(
            "文件短于 NFD R0 基础头 0x{NFD_BASE_HEADER_SIZE:X} 字节"
        ));
    }
    if source.starts_with(NFD_R1_SIGNATURE) {
        return Err("检测到 NFD R1；本工具当前只支持样本所用的 NFD R0".to_string());
    }
    if !source.starts_with(NFD_R0_SIGNATURE) {
        return Err("不是 T98FDDIMAGE.R0 镜像".to_string());
    }

    let header_size = usize::try_from(read_u32(&source, NFD_HEADER_SIZE_OFFSET)?)
        .map_err(|_| "NFD 头长度无法表示为本机地址".to_string())?;
    if header_size < NFD_BASE_HEADER_SIZE + MAP_ENTRY_SIZE || header_size > source.len() {
        return Err(format!("NFD 头长度 0x{header_size:X} 越界"));
    }
    let map_bytes = header_size - NFD_BASE_HEADER_SIZE;
    if !map_bytes.is_multiple_of(MAP_ENTRY_SIZE) {
        return Err("NFD R0 扇区表长度不是 0x10 的整数倍".to_string());
    }
    let map_entries = map_bytes / MAP_ENTRY_SIZE;
    if map_entries < 2 || !(map_entries - 1).is_multiple_of(MAP_ENTRIES_PER_TRACK_SIDE) {
        return Err("NFD R0 扇区表不是 26 项 track-side 加终止项的结构".to_string());
    }
    let track_slots = (map_entries - 1) / MAP_ENTRIES_PER_TRACK_SIDE;
    if track_slots > 256 {
        return Err(format!(
            "NFD track-side 数 {track_slots} 超过一字节 FAT 可表示范围"
        ));
    }
    let sentinel_start = header_size - MAP_ENTRY_SIZE;
    if source[sentinel_start..header_size]
        .iter()
        .any(|byte| *byte != 0)
    {
        return Err("NFD R0 扇区表终止项不是全零".to_string());
    }

    let write_protected = source[NFD_WRITE_PROTECT_OFFSET] != 0;
    let heads = source[NFD_HEADS_OFFSET];
    if heads != 2 {
        return Err(format!("样本文件系统要求 2 个磁头，镜像声明为 {heads}"));
    }

    let mut tracks = Vec::with_capacity(track_slots);
    let mut data_cursor = header_size;
    let mut all_chs = HashSet::new();
    let mut pda_values = HashSet::new();
    let mut deleted_data_sectors = 0u32;
    let mut sectors_with_read_errors = 0u32;
    let mut total_sectors = 0u32;

    for slot in 0..track_slots {
        let mut sectors = Vec::new();
        let base = NFD_BASE_HEADER_SIZE + slot * MAP_ENTRIES_PER_TRACK_SIDE * MAP_ENTRY_SIZE;
        for entry_index in 0..MAP_ENTRIES_PER_TRACK_SIDE {
            let offset = base + entry_index * MAP_ENTRY_SIZE;
            let entry = &source[offset..offset + MAP_ENTRY_SIZE];
            if entry[0] == 0xFF {
                continue;
            }
            if entry[3] > 7 {
                return Err(format!(
                    "扇区表 0x{offset:X} 的尺寸代码 N={} 不受支持",
                    entry[3]
                ));
            }
            let size = 128usize
                .checked_shl(u32::from(entry[3]))
                .ok_or_else(|| format!("扇区表 0x{offset:X} 的尺寸溢出"))?;
            let end = data_cursor
                .checked_add(size)
                .ok_or_else(|| "NFD 数据偏移溢出".to_string())?;
            if end > source.len() {
                return Err(format!(
                    "C{}/H{}/R{} 数据越过文件末尾",
                    entry[0], entry[1], entry[2]
                ));
            }
            if !all_chs.insert((entry[0], entry[1], entry[2])) {
                return Err(format!(
                    "NFD 存在重复的 C{}/H{}/R{}",
                    entry[0], entry[1], entry[2]
                ));
            }
            if entry[5] != 0 {
                deleted_data_sectors += 1;
            }
            if entry[6] != 0 || entry[7] & 0xF8 != 0 || entry[8] != 0 || entry[9] != 0 {
                sectors_with_read_errors += 1;
            }
            pda_values.insert(entry[10]);
            sectors.push(Sector {
                c: entry[0],
                h: entry[1],
                r: entry[2],
                n: entry[3],
                data: data_cursor..end,
            });
            data_cursor = end;
            total_sectors += 1;
        }

        if !sectors.is_empty() {
            let c = sectors[0].c;
            let h = sectors[0].h;
            if sectors.iter().any(|sector| sector.c != c || sector.h != h) {
                return Err(format!("track-side 槽 {slot} 混有多个柱面或磁头"));
            }
            let expected_slot = usize::from(c) * usize::from(heads) + usize::from(h);
            if expected_slot != slot {
                return Err(format!(
                    "track-side 槽 {slot} 实为 C{c}/H{h}，线性编号应为 {expected_slot}"
                ));
            }
            sectors.sort_by_key(|sector| sector.r);
        }
        tracks.push(TrackSide { slot, sectors });
    }
    if data_cursor != source.len() {
        return Err(format!(
            "NFD 扇区表覆盖到 0x{data_cursor:X}，文件实际长度为 0x{:X}",
            source.len()
        ));
    }

    let (directory_slot, fat, files) = find_filesystem(&source, &tracks)?;
    let directory_track = &tracks[directory_slot];
    let directory_c = directory_track.sectors[0].c;
    let directory_h = directory_track.sectors[0].h;
    let formatted_track_sides = tracks
        .iter()
        .filter(|track| !track.sectors.is_empty())
        .count();

    let mut warnings = Vec::new();
    if deleted_data_sectors != 0 {
        warnings.push(format!(
            "{source_name}: {deleted_data_sectors} 个扇区带 DDAM 标记，数据已原样提取"
        ));
    }
    if sectors_with_read_errors != 0 {
        warnings.push(format!(
            "{source_name}: {sectors_with_read_errors} 个扇区的 FDD BIOS 状态含错误位，数据已原样提取"
        ));
    }
    let mut pda_values: Vec<u8> = pda_values.into_iter().collect();
    pda_values.sort_unstable();

    Ok(ParsedImage {
        source,
        source_name,
        nfd: NfdManifest {
            revision: 0,
            header_size: u32::try_from(header_size)
                .map_err(|_| "NFD 头长度超过 u32".to_string())?,
            write_protected,
            heads,
            map_track_slots: u16::try_from(track_slots)
                .map_err(|_| "track-side 数超过 u16".to_string())?,
            formatted_track_sides: u16::try_from(formatted_track_sides)
                .map_err(|_| "已格式化 track-side 数超过 u16".to_string())?,
            sectors: total_sectors,
            pda_values,
            deleted_data_sectors,
            sectors_with_read_errors,
        },
        directory_slot,
        directory_c,
        directory_h,
        fat,
        tracks,
        files,
        warnings,
    })
}

fn rebuild_nfd(parsed: &ParsedImage, replacements: &HashMap<String, Vec<u8>>) -> Result<Vec<u8>> {
    if replacements.is_empty() {
        return Ok(parsed.source.clone());
    }
    for name in replacements.keys() {
        if !parsed
            .files
            .iter()
            .any(|file| file.name.eq_ignore_ascii_case(name))
        {
            return Err(format!("{}: NFD 中没有成员 {name:?}", parsed.source_name));
        }
    }
    for (name, bytes) in replacements {
        if bytes.is_empty() || !bytes.len().is_multiple_of(DATA_SECTOR_SIZE) {
            return Err(format!(
                "{} / {name}: 替换成员必须是非空的 256 字节整数倍，实际 {} 字节",
                parsed.source_name,
                bytes.len()
            ));
        }
    }

    let mut output = parsed.source.clone();
    let mut fat = parsed.fat.clone();
    let mut changed_files = parsed
        .files
        .iter()
        .filter(|file| {
            replacements
                .iter()
                .any(|(name, bytes)| name.eq_ignore_ascii_case(&file.name) && bytes != &file.data)
        })
        .collect::<Vec<_>>();
    if changed_files.is_empty() {
        return Ok(output);
    }
    changed_files.sort_by_key(|file| file.index);

    // 先释放全部待替换成员的旧簇，再按目录顺序确定性重分配。
    for file in &changed_files {
        for cluster in &file.clusters {
            fat[usize::from(*cluster)] = FAT_FREE;
        }
    }

    for file in changed_files {
        let bytes = replacements
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(&file.name))
            .map(|(_, bytes)| bytes)
            .ok_or_else(|| format!("{}: 缺少替换数据", file.name))?;
        let sectors = bytes.len() / DATA_SECTOR_SIZE;
        let needed_clusters = sectors.div_ceil(usize::from(SECTORS_PER_CLUSTER));
        let mut clusters = Vec::with_capacity(needed_clusters);
        for cluster in file.clusters.iter().copied() {
            if clusters.len() == needed_clusters {
                break;
            }
            if fat[usize::from(cluster)] == FAT_FREE
                && is_standard_data_track(&parsed.tracks[usize::from(cluster)])
            {
                fat[usize::from(cluster)] = FAT_RESERVED;
                clusters.push(cluster);
            }
        }
        if clusters.len() < needed_clusters {
            for slot in 0..parsed.tracks.len().min(fat.len()) {
                if clusters.len() == needed_clusters {
                    break;
                }
                if slot == parsed.directory_slot
                    || fat[slot] != FAT_FREE
                    || !is_standard_data_track(&parsed.tracks[slot])
                {
                    continue;
                }
                fat[slot] = FAT_RESERVED;
                clusters.push(slot as u8);
            }
        }
        if clusters.len() != needed_clusters {
            return Err(format!(
                "{} / {}: 空闲簇不足，需要 {} 个，只分配到 {} 个",
                parsed.source_name,
                file.name,
                needed_clusters,
                clusters.len()
            ));
        }
        let final_sectors = sectors - (needed_clusters - 1) * usize::from(SECTORS_PER_CLUSTER);
        for pair in clusters.windows(2) {
            fat[usize::from(pair[0])] = pair[1];
        }
        let last = *clusters.last().ok_or_else(|| "空簇链".to_string())?;
        fat[usize::from(last)] =
            0xC0 + u8::try_from(final_sectors).map_err(|_| "末簇扇区数超过 u8".to_string())?;

        let mut cursor = 0usize;
        for (cluster_index, cluster) in clusters.iter().copied().enumerate() {
            let take = if cluster_index + 1 == clusters.len() {
                final_sectors
            } else {
                usize::from(SECTORS_PER_CLUSTER)
            };
            for sector in parsed.tracks[usize::from(cluster)]
                .sectors
                .iter()
                .take(take)
            {
                output[sector.data.clone()]
                    .copy_from_slice(&bytes[cursor..cursor + DATA_SECTOR_SIZE]);
                cursor += DATA_SECTOR_SIZE;
            }
        }
        if cursor != bytes.len() {
            return Err(format!("{}: 写入成员时没有消耗全部数据", file.name));
        }

        let directory_sector_id = u8::try_from(file.directory_entry_offset / DATA_SECTOR_SIZE + 1)
            .map_err(|_| "目录项扇区号超过 u8".to_string())?;
        let directory_byte = file.directory_entry_offset % DATA_SECTOR_SIZE + 10;
        let directory_sector = parsed.tracks[parsed.directory_slot]
            .sectors
            .iter()
            .find(|sector| sector.r == directory_sector_id)
            .ok_or_else(|| format!("{}: 找不到目录项扇区", file.name))?;
        output[directory_sector.data.start + directory_byte] = clusters[0];
    }

    for sector_id in FAT_FIRST_SECTOR..=SECTORS_PER_CLUSTER {
        let sector = parsed.tracks[parsed.directory_slot]
            .sectors
            .iter()
            .find(|sector| sector.r == sector_id)
            .ok_or_else(|| format!("FAT 缺少 R{sector_id}"))?;
        output[sector.data.clone()].copy_from_slice(&fat);
    }

    let verified = parse_nfd(output.clone(), parsed.source_name.clone())?;
    for file in &parsed.files {
        let rebuilt = verified
            .files
            .iter()
            .find(|candidate| candidate.name.eq_ignore_ascii_case(&file.name))
            .ok_or_else(|| format!("重建后缺少成员 {}", file.name))?;
        let expected = replacements
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(&file.name))
            .map_or(&file.data, |(_, bytes)| bytes);
        if &rebuilt.data != expected {
            return Err(format!("{}: NFD 重建后成员复查不一致", file.name));
        }
    }
    Ok(output)
}

fn find_filesystem(source: &[u8], tracks: &[TrackSide]) -> Result<(usize, Vec<u8>, Vec<DiskFile>)> {
    let mut candidates = Vec::new();
    for track in tracks {
        if !is_standard_data_track(track) {
            continue;
        }
        let Some(fat) = identical_fat_copies(source, track) else {
            continue;
        };
        if track.slot >= fat.len() || fat[track.slot] != FAT_RESERVED {
            continue;
        }
        if let Ok(files) = parse_directory_candidate(source, tracks, track, &fat) {
            candidates.push((track.slot, fat, files));
        }
    }

    match candidates.len() {
        0 => Err("未找到通过目录项、三份 FAT 和全部文件簇链校验的 N88 2HD 文件系统".to_string()),
        1 => Ok(candidates.remove(0)),
        count => Err(format!("检测到 {count} 个可能的目录 track-side，拒绝猜测")),
    }
}

fn is_standard_data_track(track: &TrackSide) -> bool {
    track.sectors.len() == usize::from(SECTORS_PER_CLUSTER)
        && track.sectors.iter().enumerate().all(|(index, sector)| {
            sector.r == (index + 1) as u8 && sector.n == 1 && sector.data.len() == DATA_SECTOR_SIZE
        })
}

fn identical_fat_copies(source: &[u8], track: &TrackSide) -> Option<Vec<u8>> {
    let mut copies = Vec::with_capacity(FAT_COPIES);
    for sector_id in FAT_FIRST_SECTOR..=SECTORS_PER_CLUSTER {
        let sector = track.sectors.iter().find(|sector| sector.r == sector_id)?;
        copies.push(source[sector.data.clone()].to_vec());
    }
    if copies.len() == FAT_COPIES && copies[1..].iter().all(|copy| *copy == copies[0]) {
        Some(copies.remove(0))
    } else {
        None
    }
}

fn parse_directory_candidate(
    source: &[u8],
    tracks: &[TrackSide],
    directory_track: &TrackSide,
    fat: &[u8],
) -> Result<Vec<DiskFile>> {
    let mut directory = Vec::with_capacity(usize::from(DIRECTORY_SECTORS) * DATA_SECTOR_SIZE);
    for sector_id in 1..=DIRECTORY_SECTORS {
        let sector = directory_track
            .sectors
            .iter()
            .find(|sector| sector.r == sector_id)
            .ok_or_else(|| format!("目录缺少 R{sector_id}"))?;
        directory.extend_from_slice(&source[sector.data.clone()]);
    }

    let mut entries = Vec::new();
    let mut terminated = false;
    for (entry_index, entry) in directory.chunks_exact(DIRECTORY_ENTRY_SIZE).enumerate() {
        if entry[0] == 0xFF {
            terminated = true;
            break;
        }
        if entry[0] == 0x00 {
            continue;
        }
        if entry[11..].iter().any(|byte| *byte != 0xFF) {
            return Err(format!("目录项 {entry_index} 的保留字节不是 FF"));
        }
        let mut raw_name = [0u8; 9];
        raw_name.copy_from_slice(&entry[..9]);
        let name = decode_filename(&raw_name)?;
        validate_output_segment(&name)?;
        entries.push((entry_index, name, raw_name, entry[9], entry[10]));
    }
    if !terminated || entries.is_empty() {
        return Err("目录没有有效终止项或活动文件".to_string());
    }

    let mut names = HashSet::new();
    let mut used_clusters = HashSet::new();
    let mut files = Vec::with_capacity(entries.len());
    for (entry_index, name, raw_name, attribute, start_cluster) in entries {
        if !names.insert(name.to_lowercase()) {
            return Err(format!("目录含有 Windows 下重名的成员 {name:?}"));
        }
        let (clusters, final_sectors) =
            follow_chain(&name, start_cluster, fat, tracks, &mut used_clusters)?;
        let data = gather_file_data(source, tracks, &clusters, final_sectors, &name)?;
        files.push(DiskFile {
            index: files.len(),
            name,
            raw_name,
            attribute,
            directory_entry_offset: entry_index * DIRECTORY_ENTRY_SIZE,
            start_cluster,
            clusters,
            final_sectors,
            data,
        });
    }
    Ok(files)
}

fn follow_chain(
    name: &str,
    start: u8,
    fat: &[u8],
    tracks: &[TrackSide],
    used_clusters: &mut HashSet<u8>,
) -> Result<(Vec<u8>, u8)> {
    let mut current = start;
    let mut local = HashSet::new();
    let mut chain = Vec::new();
    loop {
        let index = usize::from(current);
        if index >= tracks.len() || index >= fat.len() {
            return Err(format!("{name}: FAT 簇 0x{current:02X} 越界"));
        }
        if !local.insert(current) {
            return Err(format!("{name}: FAT 簇链在 0x{current:02X} 成环"));
        }
        if !used_clusters.insert(current) {
            return Err(format!("{name}: FAT 簇 0x{current:02X} 与其他文件交叉"));
        }
        if !is_standard_data_track(&tracks[index]) {
            return Err(format!(
                "{name}: 簇 0x{current:02X} 不是 26x256 字节的完整 track-side"
            ));
        }
        chain.push(current);
        let next = fat[index];
        if (0xC1..=0xC0 + SECTORS_PER_CLUSTER).contains(&next) {
            return Ok((chain, next - 0xC0));
        }
        if next == FAT_FREE || next == FAT_RESERVED {
            return Err(format!(
                "{name}: FAT 簇 0x{current:02X} 指向保留值 0x{next:02X}"
            ));
        }
        current = next;
    }
}

fn gather_file_data(
    source: &[u8],
    tracks: &[TrackSide],
    clusters: &[u8],
    final_sectors: u8,
    name: &str,
) -> Result<Vec<u8>> {
    let sector_count = (clusters.len() - 1)
        .checked_mul(usize::from(SECTORS_PER_CLUSTER))
        .and_then(|count| count.checked_add(usize::from(final_sectors)))
        .ok_or_else(|| format!("{name}: 文件扇区数溢出"))?;
    let mut data = Vec::with_capacity(
        sector_count
            .checked_mul(DATA_SECTOR_SIZE)
            .ok_or_else(|| format!("{name}: 文件大小溢出"))?,
    );
    for (cluster_index, cluster) in clusters.iter().copied().enumerate() {
        let take = if cluster_index + 1 == clusters.len() {
            final_sectors
        } else {
            SECTORS_PER_CLUSTER
        };
        for sector in tracks[usize::from(cluster)]
            .sectors
            .iter()
            .take(usize::from(take))
        {
            data.extend_from_slice(&source[sector.data.clone()]);
        }
    }
    Ok(data)
}

fn decode_filename(raw: &[u8; 9]) -> Result<String> {
    let base_end = raw[..6]
        .iter()
        .rposition(|byte| *byte != b' ')
        .map_or(0, |index| index + 1);
    let extension_end = raw[6..]
        .iter()
        .rposition(|byte| *byte != b' ')
        .map_or(0, |index| index + 1);
    if base_end == 0 {
        return Err("目录项文件名为空".to_string());
    }
    let base = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(&raw[..base_end])
        .ok_or_else(|| format!("目录项 basename 不是有效 CP932: {}", hex_upper(raw)))?;
    let extension = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(&raw[6..6 + extension_end])
        .ok_or_else(|| format!("目录项扩展名不是有效 CP932: {}", hex_upper(raw)))?;
    if extension.is_empty() {
        Ok(base.into_owned())
    } else {
        Ok(format!("{base}.{extension}"))
    }
}

fn write_workspace(
    prepared: &[PreparedImage],
    output_root: &Path,
    overwrite: bool,
) -> Result<UnpackReport> {
    let parent = output_root
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("创建输出父目录 {} 失败: {error}", parent.display()))?;
    let staging = create_unique_sibling(output_root, "staging")?;

    let write_result = (|| -> Result<(WorkspaceManifest, Vec<String>)> {
        let mut images = Vec::with_capacity(prepared.len());
        let mut warnings = Vec::new();
        let mut total_files = 0usize;
        let mut total_bytes = 0u64;

        for item in prepared {
            let image_root = staging.join(&item.output_dir);
            fs::create_dir(&image_root)
                .map_err(|error| format!("创建成员目录 {} 失败: {error}", image_root.display()))?;
            let mut file_manifests = Vec::with_capacity(item.parsed.files.len());
            for file in &item.parsed.files {
                let output_path = image_root.join(&file.name);
                fs::write(&output_path, &file.data)
                    .map_err(|error| format!("写入 {} 失败: {error}", output_path.display()))?;
                let relative_path = format!("{}/{}", item.output_dir, file.name);
                file_manifests.push(FileManifest {
                    index: file.index,
                    name: file.name.clone(),
                    path: relative_path,
                    raw_name_hex: hex_upper(&file.raw_name),
                    attribute: file.attribute,
                    directory_entry_offset: u32::try_from(file.directory_entry_offset)
                        .map_err(|_| "目录项偏移超过 u32".to_string())?,
                    start_cluster: file.start_cluster,
                    clusters: file.clusters.clone(),
                    final_sectors: file.final_sectors,
                    size: u64::try_from(file.data.len())
                        .map_err(|_| "文件大小超过 u64".to_string())?,
                    sha256: sha256_hex(&file.data),
                });
                total_files += 1;
                total_bytes = total_bytes
                    .checked_add(file.data.len() as u64)
                    .ok_or_else(|| "解包总字节数溢出".to_string())?;
            }
            warnings.extend(item.parsed.warnings.iter().cloned());
            images.push(ImageManifest {
                source_file: item.parsed.source_name.clone(),
                source_sha256: sha256_hex(&item.parsed.source),
                output_dir: item.output_dir.clone(),
                nfd: item.parsed.nfd.clone(),
                filesystem: FilesystemManifest {
                    kind: "N88 Disk BASIC 2HD track-side FAT".to_string(),
                    directory_track_side: u8::try_from(item.parsed.directory_slot)
                        .map_err(|_| "目录 track-side 超过 u8".to_string())?,
                    directory_cylinder: item.parsed.directory_c,
                    directory_head: item.parsed.directory_h,
                    directory_sectors: DIRECTORY_SECTORS,
                    fat_sectors: [24, 25, 26],
                    fat_copies_identical: true,
                    sectors_per_cluster: SECTORS_PER_CLUSTER,
                    sector_size: DATA_SECTOR_SIZE as u16,
                    fat_sha256: sha256_hex(&item.parsed.fat),
                },
                files: file_manifests,
            });
        }

        let manifest = WorkspaceManifest {
            format: WORKSPACE_FORMAT.to_string(),
            images,
            summary: SummaryManifest {
                images: prepared.len(),
                files: total_files,
                extracted_bytes: total_bytes,
                warnings: warnings.len(),
            },
        };
        let mut json = serde_json::to_string_pretty(&manifest)
            .map_err(|error| format!("序列化 workspace.json 失败: {error}"))?;
        json.push('\n');
        fs::write(staging.join("workspace.json"), json.as_bytes())
            .map_err(|error| format!("写入 workspace.json 失败: {error}"))?;
        Ok((manifest, warnings))
    })();

    let (manifest, warnings) = match write_result {
        Ok(result) => result,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
    };

    if let Err(error) = commit_staging(&staging, output_root, overwrite) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    Ok(UnpackReport {
        images: manifest.summary.images,
        extracted_files: manifest.summary.files,
        extracted_bytes: manifest.summary.extracted_bytes,
        warnings,
        output_root: output_root.to_path_buf(),
    })
}

fn validate_existing_output(output_root: &Path, overwrite: bool) -> Result<()> {
    if !output_root.exists() {
        return Ok(());
    }
    if !output_root.is_dir() {
        return Err(format!("输出已存在且不是目录: {}", output_root.display()));
    }
    if !overwrite {
        return Err(format!(
            "输出目录已存在；需要显式 --overwrite: {}",
            output_root.display()
        ));
    }
    let mut entries =
        fs::read_dir(output_root).map_err(|error| format!("读取输出目录失败: {error}"))?;
    if entries.next().is_none() {
        return Ok(());
    }
    let manifest_path = output_root.join("workspace.json");
    let bytes = fs::read(&manifest_path).map_err(|_| {
        format!(
            "拒绝覆盖非本工具工作区（缺少有效 workspace.json）: {}",
            output_root.display()
        )
    })?;
    let marker: WorkspaceMarker = serde_json::from_slice(&bytes).map_err(|_| {
        format!(
            "拒绝覆盖非本工具工作区（workspace.json 无效）: {}",
            output_root.display()
        )
    })?;
    if marker.format != WORKSPACE_FORMAT {
        return Err(format!(
            "拒绝覆盖其他格式的工作区: {}",
            output_root.display()
        ));
    }
    Ok(())
}

fn commit_staging(staging: &Path, output_root: &Path, overwrite: bool) -> Result<()> {
    if !output_root.exists() {
        return fs::rename(staging, output_root)
            .map_err(|error| format!("提交输出目录失败: {error}"));
    }
    if !overwrite {
        return Err("输出在写入期间被其他进程创建".to_string());
    }

    let backup = unique_unused_sibling(output_root, "backup")?;
    fs::rename(output_root, &backup).map_err(|error| format!("备份旧输出目录失败: {error}"))?;
    if let Err(error) = fs::rename(staging, output_root) {
        let restore = fs::rename(&backup, output_root);
        return match restore {
            Ok(()) => Err(format!("提交新输出失败，旧输出已恢复: {error}")),
            Err(restore_error) => Err(format!(
                "提交新输出失败且旧输出恢复失败: {error}; {restore_error}"
            )),
        };
    }
    fs::remove_dir_all(&backup)
        .map_err(|error| format!("新输出已提交，但清理旧输出备份失败: {error}"))?;
    Ok(())
}

fn validate_output_does_not_contain_inputs(inputs: &[PathBuf], output_root: &Path) -> Result<()> {
    let output = absolute_lexical(output_root)?;
    let output_key = normalized_windows_path(&output);
    for input in inputs {
        let canonical = fs::canonicalize(input)
            .map_err(|error| format!("无法规范化输入 {}: {error}", input.display()))?;
        let input_key = normalized_windows_path(&canonical);
        if input_key == output_key || input_key.starts_with(&(output_key.clone() + "\\")) {
            return Err(format!(
                "输出目录不能包含输入镜像，否则覆盖时可能删除源文件: {}",
                output_root.display()
            ));
        }
    }
    Ok(())
}

fn absolute_lexical(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| format!("读取当前目录失败: {error}"))?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(format!("输出路径越过根目录: {}", path.display()));
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    Ok(normalized)
}

fn normalized_windows_path(path: &Path) -> String {
    path.to_string_lossy().replace('/', "\\").to_lowercase()
}

fn create_unique_sibling(output_root: &Path, purpose: &str) -> Result<PathBuf> {
    let parent = output_root
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("创建输出父目录 {} 失败: {error}", parent.display()))?;
    let candidate = unique_unused_sibling(output_root, purpose)?;
    fs::create_dir(&candidate)
        .map_err(|error| format!("创建临时输出目录 {} 失败: {error}", candidate.display()))?;
    Ok(candidate)
}

fn unique_unused_sibling(output_root: &Path, purpose: &str) -> Result<PathBuf> {
    let parent = output_root
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = output_root
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "nfd-output".to_string());
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("系统时间异常: {error}"))?
        .as_nanos();
    for attempt in 0..1000u32 {
        let candidate = parent.join(format!(
            ".{name}.{purpose}.{}.{nonce}.{attempt}",
            std::process::id()
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(format!("无法为 {} 分配临时目录", output_root.display()))
}

fn safe_input_label(path: &Path) -> String {
    let source = path
        .file_stem()
        .or_else(|| path.file_name())
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "disk".to_string());
    let mut output = String::new();
    for ch in source.chars().take(80) {
        if ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') {
            output.push('_');
        } else {
            output.push(ch);
        }
    }
    let trimmed = output.trim().trim_end_matches('.').to_string();
    let mut output = if trimmed.is_empty() {
        "disk".to_string()
    } else {
        trimmed
    };
    if is_windows_reserved_name(&output) {
        output.insert(0, '_');
    }
    output
}

fn validate_output_segment(segment: &str) -> Result<()> {
    if segment.is_empty() || segment == "." || segment == ".." {
        return Err("磁盘成员名为空或为相对路径标记".to_string());
    }
    if segment.ends_with(' ') || segment.ends_with('.') {
        return Err(format!("磁盘成员名末尾含空格或句点: {segment:?}"));
    }
    if segment.chars().any(|ch| {
        ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
    }) {
        return Err(format!("磁盘成员名含不安全字符: {segment:?}"));
    }
    if is_windows_reserved_name(segment) {
        return Err(format!("磁盘成员名是 Windows 保留设备名: {segment:?}"));
    }
    Ok(())
}

fn is_windows_reserved_name(segment: &str) -> bool {
    let stem = segment
        .split('.')
        .next()
        .unwrap_or(segment)
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| format!("在 0x{offset:X} 读取 u32 越界"))?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex_upper(&digest).to_lowercase()
}

fn hex_upper(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        let _ = write!(output, "{byte:02X}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TRACK_SLOTS: usize = 163;
    const SAMPLE_FORMATTED_TRACKS: usize = 154;
    const SAMPLE_DIRECTORY_SLOT: usize = 70;

    #[test]
    fn parses_nfd_r0_and_reorders_interleaved_sectors() {
        let sample = build_sample();
        let parsed = parse_nfd(sample, "sample.nfd".to_string()).expect("sample must parse");
        assert_eq!(parsed.directory_slot, SAMPLE_DIRECTORY_SLOT);
        assert_eq!(parsed.nfd.pda_values, vec![0x90]);
        assert_eq!(parsed.nfd.sectors_with_read_errors, 0);
        assert_eq!(parsed.files.len(), 1);
        let file = &parsed.files[0];
        assert_eq!(file.name, "TEST.BIN");
        assert_eq!(file.clusters, vec![71]);
        assert_eq!(file.final_sectors, 2);
        assert_eq!(file.data.len(), 512);
        assert!(file.data[..256].iter().all(|byte| *byte == 1));
        assert!(file.data[256..].iter().all(|byte| *byte == 2));
    }

    #[test]
    fn rejects_truncated_payload() {
        let mut sample = build_sample();
        sample.pop();
        let error = parse_nfd(sample, "truncated.nfd".to_string()).unwrap_err();
        assert!(error.contains("越过文件末尾") || error.contains("覆盖到"));
    }

    #[test]
    fn reports_real_fdd_status_error_bits() {
        let mut sample = build_sample();
        sample[NFD_BASE_HEADER_SIZE + 8] = 0x20;
        let parsed = parse_nfd(sample, "status.nfd".to_string()).expect("sample must parse");
        assert_eq!(parsed.nfd.sectors_with_read_errors, 1);
        assert_eq!(parsed.warnings.len(), 1);
    }

    #[test]
    fn rejects_duplicate_chs() {
        let mut sample = build_sample();
        let second_entry = NFD_BASE_HEADER_SIZE + MAP_ENTRY_SIZE;
        sample[second_entry + 2] = sample[NFD_BASE_HEADER_SIZE + 2];
        let error = parse_nfd(sample, "duplicate.nfd".to_string()).unwrap_err();
        assert!(error.contains("重复"));
    }

    #[test]
    fn rejects_inconsistent_fat_copies() {
        let mut sample = build_sample();
        let offset = sector_payload_offset(SAMPLE_DIRECTORY_SLOT, 25);
        sample[offset + 71] ^= 1;
        let error = parse_nfd(sample, "fat.nfd".to_string()).unwrap_err();
        assert!(error.contains("未找到"));
    }

    #[test]
    fn rejects_unsafe_member_name() {
        let mut sample = build_sample();
        let offset = sector_payload_offset(SAMPLE_DIRECTORY_SLOT, 1);
        sample[offset..offset + 6].copy_from_slice(b"BAD/  ");
        let error = parse_nfd(sample, "unsafe.nfd".to_string()).unwrap_err();
        assert!(error.contains("未找到"));
    }

    #[test]
    fn writes_unicode_workspace_and_requires_overwrite() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "tauhido_nfd_test_{}_{}_路径 & 日本語",
            std::process::id(),
            nonce
        ));
        let input_dir = root.join("输入");
        let output = root.join("输出");
        fs::create_dir_all(&input_dir).expect("create input");
        let input = input_dir.join("样本.bin");
        fs::write(&input, build_sample()).expect("write sample");

        let first =
            unpack_batch(std::slice::from_ref(&input_dir), &output, false).expect("first unpack");
        assert_eq!(first.extracted_files, 1);
        assert!(output.join("00_样本").join("TEST.BIN").is_file());
        assert!(unpack_batch(std::slice::from_ref(&input), &output, false).is_err());
        unpack_batch(std::slice::from_ref(&input), &output, true).expect("overwrite");

        fs::remove_dir_all(&root).expect("cleanup test workspace");
    }

    fn build_sample() -> Vec<u8> {
        let header_size = NFD_BASE_HEADER_SIZE
            + (SAMPLE_TRACK_SLOTS * MAP_ENTRIES_PER_TRACK_SIDE + 1) * MAP_ENTRY_SIZE;
        let mut header = vec![0u8; header_size];
        header[..14].copy_from_slice(NFD_R0_SIGNATURE);
        header[NFD_HEADER_SIZE_OFFSET..NFD_HEADER_SIZE_OFFSET + 4]
            .copy_from_slice(&(header_size as u32).to_le_bytes());
        header[NFD_HEADS_OFFSET] = 2;

        let order = physical_sector_order();
        for slot in 0..SAMPLE_TRACK_SLOTS {
            for (physical_index, sector_id) in order.iter().copied().enumerate() {
                let offset = NFD_BASE_HEADER_SIZE
                    + (slot * MAP_ENTRIES_PER_TRACK_SIDE + physical_index) * MAP_ENTRY_SIZE;
                if slot >= SAMPLE_FORMATTED_TRACKS {
                    header[offset] = 0xFF;
                    continue;
                }
                header[offset] = (slot / 2) as u8;
                header[offset + 1] = (slot % 2) as u8;
                header[offset + 2] = sector_id;
                header[offset + 3] = if slot == 0 { 0 } else { 1 };
                header[offset + 4] = if slot == 0 { 0 } else { 1 };
                header[offset + 10] = 0x90;
            }
        }

        let mut fat = vec![FAT_FREE; 256];
        fat[..12].fill(FAT_RESERVED);
        fat[SAMPLE_DIRECTORY_SLOT] = FAT_RESERVED;
        fat[71] = 0xC2;

        let mut output = header;
        for slot in 0..SAMPLE_FORMATTED_TRACKS {
            for sector_id in order.iter().copied() {
                let size = if slot == 0 { 128 } else { DATA_SECTOR_SIZE };
                let mut data = vec![0x40; size];
                if slot == SAMPLE_DIRECTORY_SLOT {
                    if sector_id <= DIRECTORY_SECTORS {
                        data.fill(0xFF);
                    }
                    if sector_id == 1 {
                        data[..9].copy_from_slice(b"TEST  BIN");
                        data[9] = 1;
                        data[10] = 71;
                        data[11..16].fill(0xFF);
                    }
                    if (FAT_FIRST_SECTOR..=SECTORS_PER_CLUSTER).contains(&sector_id) {
                        data.copy_from_slice(&fat);
                    }
                } else if slot == 71 {
                    data.fill(sector_id);
                }
                output.extend_from_slice(&data);
            }
        }
        output
    }

    fn physical_sector_order() -> Vec<u8> {
        (1..=13).flat_map(|sector| [sector, sector + 13]).collect()
    }

    fn sector_payload_offset(slot: usize, sector_id: u8) -> usize {
        let header_size = NFD_BASE_HEADER_SIZE
            + (SAMPLE_TRACK_SLOTS * MAP_ENTRIES_PER_TRACK_SIDE + 1) * MAP_ENTRY_SIZE;
        let preceding = if slot == 0 {
            0
        } else {
            MAP_ENTRIES_PER_TRACK_SIDE * 128
                + (slot - 1) * MAP_ENTRIES_PER_TRACK_SIDE * DATA_SECTOR_SIZE
        };
        let physical_index = if sector_id <= 13 {
            usize::from(sector_id - 1) * 2
        } else {
            usize::from(sector_id - 14) * 2 + 1
        };
        header_size + preceding + physical_index * DATA_SECTOR_SIZE
    }
}
