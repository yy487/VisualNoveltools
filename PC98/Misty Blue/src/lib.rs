#![forbid(unsafe_code)]

use encoding_rs::SHIFT_JIS;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use vn_d88::{Decoder, HeaderLayout, StandardCodec};

pub mod bun;
pub mod mes;
pub mod workflow;

pub const WORKSPACE_FORMAT: &str = "misty-blue-enix-dos-unpack-v1";
const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");
const LOGICAL_SIZE: usize = 0x134000;
const BYTES_PER_SECTOR: usize = 0x400;
const SECTORS_PER_TRACK: usize = 8;
const TRACKS: usize = 154;
const FAT1_OFFSET: usize = 0x400;
const FAT_BYTES: usize = 0x800;
const FAT2_OFFSET: usize = FAT1_OFFSET + FAT_BYTES;
const ROOT_OFFSET: usize = 0x1400;
const ROOT_ENTRIES: usize = 192;
const DIRECTORY_ENTRY_SIZE: usize = 32;
const DATA_OFFSET: usize = 0x2C00;
const CLUSTER_BYTES: usize = BYTES_PER_SECTOR;

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Copy)]
pub struct PreparationSummary {
    pub images: usize,
    pub files: usize,
    pub directories: usize,
    pub file_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct DiskSummary {
    pub source_file: String,
    pub files: usize,
    pub directories: usize,
    pub file_bytes: u64,
}

#[derive(Debug)]
pub struct PreparedBatch {
    input_paths: Vec<PathBuf>,
    disks: Vec<PreparedDisk>,
    summary: PreparationSummary,
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
                source_file: disk.manifest.source_file.clone(),
                files: disk.manifest.files.len(),
                directories: disk.manifest.directories.len(),
                file_bytes: disk.manifest.files.iter().map(|file| file.size).sum(),
            })
            .collect()
    }
}

#[derive(Debug)]
struct PreparedDisk {
    output_directory: String,
    manifest: DiskManifest,
    artifacts: Vec<Artifact>,
}

#[derive(Debug)]
struct Artifact {
    relative_path: PathBuf,
    bytes: Vec<u8>,
}

#[derive(Debug, Serialize)]
struct WorkspaceManifest {
    #[serde(rename = "_format")]
    format: &'static str,
    tool_version: &'static str,
    profile: String,
    role_paths: RolePaths,
    disks: Vec<DiskWorkspaceEntry>,
}

#[derive(Debug, Serialize)]
struct RolePaths {
    resources: &'static str,
}

#[derive(Debug, Serialize)]
struct DiskWorkspaceEntry {
    source_file: String,
    source_role: String,
    output_directory: String,
    files: usize,
    directories: usize,
    file_bytes: u64,
}

#[derive(Debug, Serialize)]
struct DiskManifest {
    #[serde(rename = "_format")]
    format: &'static str,
    source_file: String,
    source_role: String,
    source_sha256: String,
    d88_name_raw_hex: String,
    d88_media_code: u8,
    d88_tracks: usize,
    d88_sectors: usize,
    sector_size: usize,
    logical_size: usize,
    filesystem: FileSystemManifest,
    volume_labels: Vec<String>,
    directories: Vec<DirectoryManifest>,
    files: Vec<FileManifest>,
}

#[derive(Debug, Serialize)]
struct FileSystemManifest {
    kind: &'static str,
    bytes_per_sector: usize,
    sectors_per_cluster: usize,
    fat_copies: usize,
    sectors_per_fat: usize,
    root_entries: usize,
    first_data_sector: usize,
}

#[derive(Debug, Serialize)]
struct DirectoryManifest {
    path: String,
    attributes: u8,
    start_cluster: u16,
    cluster_chain: Vec<u16>,
}

#[derive(Debug, Clone, Serialize)]
struct FileManifest {
    path: String,
    output_path: String,
    attributes: u8,
    start_cluster: u16,
    cluster_chain: Vec<u16>,
    size: u64,
    sha256: String,
}

#[derive(Debug)]
struct ParsedFile {
    manifest: FileManifest,
    data: Vec<u8>,
}

#[derive(Debug)]
struct ParsedFileSystem {
    volume_labels: Vec<String>,
    directories: Vec<DirectoryManifest>,
    files: Vec<ParsedFile>,
}

#[derive(Debug)]
struct RawEntry {
    name: String,
    attributes: u8,
    start_cluster: u16,
    size: usize,
}

struct EnixDos<'a> {
    logical: &'a [u8],
    fat: &'a [u8],
    visited_directories: HashSet<u16>,
    seen_paths: HashSet<String>,
    volume_labels: Vec<String>,
    directories: Vec<DirectoryManifest>,
    files: Vec<ParsedFile>,
}

pub fn prepare_batch(selected: &[PathBuf]) -> Result<PreparedBatch> {
    let inputs = discover_inputs(selected)?;
    let mut output_names = BTreeSet::new();
    let mut disks = Vec::with_capacity(inputs.len());
    for input in &inputs {
        let bytes =
            fs::read(input).map_err(|error| format!("读取 {} 失败: {error}", input.display()))?;
        let codec = StandardCodec;
        let image = codec
            .decode(&bytes)
            .map_err(|error| format!("{}: {error}", input.display()))?;
        if image.disks.len() != 1 {
            return Err(format!(
                "{}: 当前 ENIX-DOS profile 要求每个文件恰有一张 D88 盘，实际 {}",
                input.display(),
                image.disks.len()
            ));
        }
        if !image.diagnostics.is_empty() {
            return Err(format!(
                "{}: D88 容器存在 {} 条未解释诊断",
                input.display(),
                image.diagnostics.len()
            ));
        }
        let disk = &image.disks[0];
        let logical = materialize_logical(&bytes, disk)?;
        let parsed = parse_enix_dos(&logical, input)?;
        let source_file = input
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("输入文件名无法表示为 Unicode: {}", input.display()))?
            .to_owned();
        let stem = input
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("输入文件名无法表示为 Unicode: {}", input.display()))?;
        let output_directory = safe_component(stem);
        if !output_names.insert(output_directory.to_ascii_uppercase()) {
            return Err(format!(
                "多个输入会映射到同一个输出目录: {output_directory}"
            ));
        }
        let files = parsed
            .files
            .iter()
            .map(|file| FileManifest {
                path: file.manifest.path.clone(),
                output_path: file.manifest.output_path.clone(),
                attributes: file.manifest.attributes,
                start_cluster: file.manifest.start_cluster,
                cluster_chain: file.manifest.cluster_chain.clone(),
                size: file.manifest.size,
                sha256: file.manifest.sha256.clone(),
            })
            .collect::<Vec<_>>();
        let artifacts = parsed
            .files
            .into_iter()
            .map(|file| Artifact {
                relative_path: PathBuf::from(&file.manifest.output_path),
                bytes: file.data,
            })
            .collect();
        disks.push(PreparedDisk {
            output_directory,
            manifest: DiskManifest {
                format: WORKSPACE_FORMAT,
                source_role: format!("<INPUT>/{source_file}"),
                source_file,
                source_sha256: sha256_hex(&bytes),
                d88_name_raw_hex: hex_upper(&disk.header.raw_name),
                d88_media_code: disk.header.media_type,
                d88_tracks: disk.tracks.len(),
                d88_sectors: disk.tracks.iter().map(|track| track.sectors.len()).sum(),
                sector_size: BYTES_PER_SECTOR,
                logical_size: logical.len(),
                filesystem: FileSystemManifest {
                    kind: "ENIX-DOS fixed-layout FAT12 (no BPB)",
                    bytes_per_sector: BYTES_PER_SECTOR,
                    sectors_per_cluster: 1,
                    fat_copies: 2,
                    sectors_per_fat: FAT_BYTES / BYTES_PER_SECTOR,
                    root_entries: ROOT_ENTRIES,
                    first_data_sector: DATA_OFFSET / BYTES_PER_SECTOR,
                },
                volume_labels: parsed.volume_labels,
                directories: parsed.directories,
                files,
            },
            artifacts,
        });
    }
    let summary = PreparationSummary {
        images: disks.len(),
        files: disks.iter().map(|disk| disk.manifest.files.len()).sum(),
        directories: disks
            .iter()
            .map(|disk| disk.manifest.directories.len())
            .sum(),
        file_bytes: disks
            .iter()
            .flat_map(|disk| &disk.manifest.files)
            .map(|file| file.size)
            .sum(),
    };
    Ok(PreparedBatch {
        input_paths: inputs,
        disks,
        summary,
    })
}

pub fn validate_output_target(output: &Path, overwrite: bool) -> Result<()> {
    if !output.exists() {
        return Ok(());
    }
    if !output.is_dir() {
        return Err(format!("输出已存在且不是目录: {}", output.display()));
    }
    if !overwrite {
        return Err(format!(
            "输出目录已存在；如需替换，请显式使用 --overwrite: {}",
            output.display()
        ));
    }
    let manifest = output.join("workspace.json");
    let bytes = fs::read(&manifest).map_err(|_| {
        format!(
            "拒绝覆盖没有有效 workspace.json 的目录: {}",
            output.display()
        )
    })?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|_| format!("拒绝覆盖清单无效的目录: {}", output.display()))?;
    if value.get("_format").and_then(|item| item.as_str()) != Some(WORKSPACE_FORMAT) {
        return Err(format!("拒绝覆盖其他工具管理的目录: {}", output.display()));
    }
    Ok(())
}

pub fn write_prepared(prepared: PreparedBatch, output: &Path, overwrite: bool) -> Result<PathBuf> {
    write_prepared_with_profile(
        prepared,
        output,
        overwrite,
        "Misty Blue / ENIX-DOS fixed-layout FAT12",
        "Misty Blue D88 游戏资源\r\n\r\n每张盘的实际 ENIX-DOS 文件位于 <盘名>/files/。\r\nmanifest.json 记录原目录路径、FAT12 簇链、大小和 SHA-256。\r\n本工作区不包含轨道切片、扇区切片或 logical.bin。\r\n",
    )
}

/// Commit a prepared ENIX-DOS extraction while allowing a game adapter to brand
/// the shared workspace with its own profile and user-facing notes.
pub fn write_prepared_with_profile(
    prepared: PreparedBatch,
    output: &Path,
    overwrite: bool,
    profile: &str,
    readme: &str,
) -> Result<PathBuf> {
    if profile.trim().is_empty() || profile.contains('\0') || readme.contains('\0') {
        return Err("解包工作区说明不能为空或包含 NUL".to_owned());
    }
    validate_output_target(output, overwrite)?;
    let parent = output
        .parent()
        .ok_or_else(|| format!("输出目录没有父目录: {}", output.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("创建输出父目录 {} 失败: {error}", parent.display()))?;
    let staging = unique_sibling(output, "staging")?;
    fs::create_dir(&staging)
        .map_err(|error| format!("创建临时目录 {} 失败: {error}", staging.display()))?;
    let write_result = write_staging(&prepared, &staging, profile, readme);
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    commit_staging(&staging, output, overwrite)?;
    Ok(output.to_path_buf())
}

fn write_staging(
    prepared: &PreparedBatch,
    staging: &Path,
    profile: &str,
    readme: &str,
) -> Result<()> {
    let mut workspace_disks = Vec::with_capacity(prepared.disks.len());
    for disk in &prepared.disks {
        let disk_root = staging.join(&disk.output_directory);
        let files_root = disk_root.join("files");
        fs::create_dir_all(&files_root)
            .map_err(|error| format!("创建 {} 失败: {error}", files_root.display()))?;
        for artifact in &disk.artifacts {
            let target = safe_join(&disk_root, &artifact.relative_path)?;
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("创建 {} 失败: {error}", parent.display()))?;
            }
            fs::write(&target, &artifact.bytes)
                .map_err(|error| format!("写入 {} 失败: {error}", target.display()))?;
        }
        write_json(&disk_root.join("manifest.json"), &disk.manifest)?;
        workspace_disks.push(DiskWorkspaceEntry {
            source_file: disk.manifest.source_file.clone(),
            source_role: disk.manifest.source_role.clone(),
            output_directory: disk.output_directory.clone(),
            files: disk.manifest.files.len(),
            directories: disk.manifest.directories.len(),
            file_bytes: disk.manifest.files.iter().map(|file| file.size).sum(),
        });
    }
    write_json(
        &staging.join("workspace.json"),
        &WorkspaceManifest {
            format: WORKSPACE_FORMAT,
            tool_version: TOOL_VERSION,
            profile: profile.to_owned(),
            role_paths: RolePaths {
                resources: "<disk>/files",
            },
            disks: workspace_disks,
        },
    )?;
    fs::write(staging.join("README.txt"), readme)
        .map_err(|error| format!("写入 README.txt 失败: {error}"))?;
    Ok(())
}

fn materialize_logical(source: &[u8], disk: &vn_d88::Disk) -> Result<Vec<u8>> {
    if disk.header.layout != HeaderLayout::Extended164 {
        return Err("Misty Blue profile 要求 164 槽 D88 头".into());
    }
    if disk.tracks.len() != TRACKS {
        return Err(format!(
            "Misty Blue profile 要求 {TRACKS} 个盘面轨，实际 {}",
            disk.tracks.len()
        ));
    }
    let mut logical = Vec::with_capacity(LOGICAL_SIZE);
    for (expected_slot, track) in disk.tracks.iter().enumerate() {
        if track.slot != expected_slot || track.sectors.len() != SECTORS_PER_TRACK {
            return Err(format!(
                "盘面轨 {expected_slot} 几何不匹配: slot={} sectors={}",
                track.slot,
                track.sectors.len()
            ));
        }
        for (ordinal, sector) in track.sectors.iter().enumerate() {
            let id = sector.address.id;
            if sector.data_range.len() != BYTES_PER_SECTOR
                || id.cylinder != (expected_slot / 2) as u16
                || id.head != (expected_slot % 2) as u8
                || id.record != (ordinal + 1) as u16
            {
                return Err(format!(
                    "盘面轨 {expected_slot} 扇区 {} 的 CHRN/长度不符合 ENIX-DOS 线性布局",
                    ordinal + 1
                ));
            }
            logical.extend_from_slice(&source[sector.data_range.clone()]);
        }
    }
    if logical.len() != LOGICAL_SIZE {
        return Err(format!(
            "Misty Blue 逻辑盘大小应为 0x{LOGICAL_SIZE:X}，实际 0x{:X}",
            logical.len()
        ));
    }
    Ok(logical)
}

fn parse_enix_dos(logical: &[u8], source: &Path) -> Result<ParsedFileSystem> {
    let fat1 = logical
        .get(FAT1_OFFSET..FAT1_OFFSET + FAT_BYTES)
        .ok_or_else(|| format!("{}: 第一份 FAT 被截断", source.display()))?;
    let fat2 = logical
        .get(FAT2_OFFSET..FAT2_OFFSET + FAT_BYTES)
        .ok_or_else(|| format!("{}: 第二份 FAT 被截断", source.display()))?;
    if fat1 != fat2 {
        return Err(format!("{}: 两份 ENIX-DOS FAT 不一致", source.display()));
    }
    if fat12_entry(fat1, 0)? != 0xFFE || fat12_entry(fat1, 1)? < 0xFF8 {
        return Err(format!("{}: ENIX-DOS FAT 保留项无效", source.display()));
    }
    let root_end = ROOT_OFFSET + ROOT_ENTRIES * DIRECTORY_ENTRY_SIZE;
    let root = logical
        .get(ROOT_OFFSET..root_end)
        .ok_or_else(|| format!("{}: 根目录被截断", source.display()))?;
    let mut parser = EnixDos {
        logical,
        fat: fat1,
        visited_directories: HashSet::new(),
        seen_paths: HashSet::new(),
        volume_labels: Vec::new(),
        directories: Vec::new(),
        files: Vec::new(),
    };
    parser.parse_directory_bytes(root, "")?;
    Ok(ParsedFileSystem {
        volume_labels: parser.volume_labels,
        directories: parser.directories,
        files: parser.files,
    })
}

impl EnixDos<'_> {
    fn parse_directory_bytes(&mut self, bytes: &[u8], parent: &str) -> Result<()> {
        for record in bytes.chunks_exact(DIRECTORY_ENTRY_SIZE) {
            match record[0] {
                0x00 => break,
                0xE5 => continue,
                _ => {}
            }
            let attributes = record[11];
            if attributes == 0x0F {
                continue;
            }
            let name = decode_short_name(record)?;
            if name == "." || name == ".." {
                continue;
            }
            if attributes & 0x08 != 0 {
                self.volume_labels.push(name);
                continue;
            }
            let entry = RawEntry {
                name,
                attributes,
                start_cluster: u16::from_le_bytes([record[26], record[27]]),
                size: u32::from_le_bytes([record[28], record[29], record[30], record[31]]) as usize,
            };
            let component = safe_component(&entry.name);
            let path = if parent.is_empty() {
                component
            } else {
                format!("{parent}/{component}")
            };
            let folded = path.to_ascii_uppercase();
            if !self.seen_paths.insert(folded) {
                return Err(format!("ENIX-DOS 目录含重复路径: {path}"));
            }
            if entry.attributes & 0x10 != 0 {
                self.parse_subdirectory(&entry, &path)?;
            } else {
                self.parse_file(&entry, &path)?;
            }
        }
        Ok(())
    }

    fn parse_subdirectory(&mut self, entry: &RawEntry, path: &str) -> Result<()> {
        if entry.start_cluster < 2 {
            return Err(format!("目录 {path} 的起始簇无效"));
        }
        if !self.visited_directories.insert(entry.start_cluster) {
            return Err(format!("目录 {path} 重复引用簇 {}", entry.start_cluster));
        }
        let chain = self.cluster_chain(entry.start_cluster)?;
        let bytes = self.read_chain(&chain)?;
        self.directories.push(DirectoryManifest {
            path: path.to_owned(),
            attributes: entry.attributes,
            start_cluster: entry.start_cluster,
            cluster_chain: chain,
        });
        self.parse_directory_bytes(&bytes, path)
    }

    fn parse_file(&mut self, entry: &RawEntry, path: &str) -> Result<()> {
        let (chain, data) = if entry.size == 0 {
            (Vec::new(), Vec::new())
        } else {
            if entry.start_cluster < 2 {
                return Err(format!("文件 {path} 的起始簇无效"));
            }
            let chain = self.cluster_chain(entry.start_cluster)?;
            let mut data = self.read_chain(&chain)?;
            if entry.size > data.len() {
                return Err(format!(
                    "文件 {path} 声明 {} 字节，簇链只能提供 {} 字节",
                    entry.size,
                    data.len()
                ));
            }
            data.truncate(entry.size);
            (chain, data)
        };
        self.files.push(ParsedFile {
            manifest: FileManifest {
                path: path.to_owned(),
                output_path: format!("files/{path}"),
                attributes: entry.attributes,
                start_cluster: entry.start_cluster,
                cluster_chain: chain,
                size: data.len() as u64,
                sha256: sha256_hex(&data),
            },
            data,
        });
        Ok(())
    }

    fn cluster_chain(&self, start: u16) -> Result<Vec<u16>> {
        let max_cluster = ((self.logical.len() - DATA_OFFSET) / CLUSTER_BYTES + 1) as u16;
        let mut chain = Vec::new();
        let mut seen = HashSet::new();
        let mut current = start;
        loop {
            if current < 2 || current > max_cluster {
                return Err(format!("FAT12 簇 {current} 超出数据区"));
            }
            if !seen.insert(current) {
                return Err(format!("FAT12 簇链在 {current} 形成循环"));
            }
            chain.push(current);
            let next = fat12_entry(self.fat, current)?;
            match next {
                0xFF8..=0xFFF => break,
                0xFF7 => return Err(format!("FAT12 簇链遇到坏簇 {current}")),
                0xFF0..=0xFF6 => return Err(format!("FAT12 簇链遇到保留值 0x{next:03X}")),
                0x000 => return Err(format!("FAT12 簇链从 {current} 指向空闲簇")),
                value => current = value,
            }
        }
        Ok(chain)
    }

    fn read_chain(&self, chain: &[u16]) -> Result<Vec<u8>> {
        let mut output = Vec::with_capacity(chain.len() * CLUSTER_BYTES);
        for &cluster in chain {
            let start = DATA_OFFSET + (usize::from(cluster) - 2) * CLUSTER_BYTES;
            let end = start + CLUSTER_BYTES;
            let bytes = self
                .logical
                .get(start..end)
                .ok_or_else(|| format!("簇 {cluster} 超出逻辑盘"))?;
            output.extend_from_slice(bytes);
        }
        Ok(output)
    }
}

/// Rebuild one complete D88 after replacing root-level ENIX-DOS files.
/// Container headers, sector headers, and every untouched sector byte are preserved.
pub fn rebuild_d88(
    source_path: &Path,
    source_bytes: &[u8],
    replacements: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<u8>> {
    let codec = StandardCodec;
    let image = codec
        .decode(source_bytes)
        .map_err(|error| format!("{}: {error}", source_path.display()))?;
    if image.disks.len() != 1 || !image.diagnostics.is_empty() {
        return Err(format!(
            "{}: 重建要求一张且无容器诊断的 D88",
            source_path.display()
        ));
    }
    let disk = &image.disks[0];
    let mut logical = materialize_logical(source_bytes, disk)?;
    let parsed = parse_enix_dos(&logical, source_path)?;
    let by_path = parsed
        .files
        .iter()
        .map(|file| (file.manifest.path.to_ascii_uppercase(), file))
        .collect::<HashMap<_, _>>();

    struct Change<'a> {
        path: &'a str,
        start_cluster: u16,
        old_size: usize,
        old_chain: Vec<u16>,
        data: &'a [u8],
    }
    let mut changed = Vec::new();
    for (path, data) in replacements {
        if path.contains('/') || path.contains('\\') {
            return Err(format!(
                "{path}: 当前 Misty Blue 文本资源应位于 ENIX-DOS 根目录"
            ));
        }
        let file = by_path
            .get(&path.to_ascii_uppercase())
            .copied()
            .ok_or_else(|| format!("{} 中没有 ENIX-DOS 文件 {path}", source_path.display()))?;
        if !sha256_hex(data).eq_ignore_ascii_case(&file.manifest.sha256) {
            changed.push(Change {
                path,
                start_cluster: file.manifest.start_cluster,
                old_size: file.data.len(),
                old_chain: file.manifest.cluster_chain.clone(),
                data,
            });
        }
    }
    if changed.is_empty() {
        return Ok(source_bytes.to_vec());
    }

    let mut fat = logical[FAT1_OFFSET..FAT1_OFFSET + FAT_BYTES].to_vec();
    for change in &changed {
        for &cluster in &change.old_chain {
            fat12_set(&mut fat, cluster, 0)?;
        }
    }
    let max_cluster = ((logical.len() - DATA_OFFSET) / CLUSTER_BYTES + 1) as u16;
    let free = (2..=max_cluster)
        .filter(|cluster| fat12_entry(&fat, *cluster).is_ok_and(|value| value == 0))
        .collect::<Vec<_>>();
    let required = changed
        .iter()
        .map(|change| change.data.len().div_ceil(CLUSTER_BYTES))
        .sum::<usize>();
    if required > free.len() {
        return Err(format!(
            "{} ENIX-DOS 空间不足：回注需要 {required} 簇，可用 {} 簇",
            source_path.display(),
            free.len()
        ));
    }

    let mut free_cursor = 0usize;
    for change in changed {
        let needed = change.data.len().div_ceil(CLUSTER_BYTES);
        let chain = &free[free_cursor..free_cursor + needed];
        free_cursor += needed;
        for (index, &cluster) in chain.iter().enumerate() {
            let next = chain.get(index + 1).copied().unwrap_or(0xfff);
            fat12_set(&mut fat, cluster, next)?;
            let start = DATA_OFFSET + (usize::from(cluster) - 2) * CLUSTER_BYTES;
            let target = logical
                .get_mut(start..start + CLUSTER_BYTES)
                .ok_or_else(|| format!("{}: 簇 {cluster} 超出逻辑盘", change.path))?;
            target.fill(0);
            let source_start = index * CLUSTER_BYTES;
            let source_end = (source_start + CLUSTER_BYTES).min(change.data.len());
            if source_start < source_end {
                target[..source_end - source_start]
                    .copy_from_slice(&change.data[source_start..source_end]);
            }
        }

        let entry_offset = find_root_entry_offset(&logical, change.path)?;
        let entry = logical
            .get_mut(entry_offset..entry_offset + DIRECTORY_ENTRY_SIZE)
            .ok_or_else(|| format!("{}: 根目录项越界", change.path))?;
        let actual_cluster = u16::from_le_bytes([entry[26], entry[27]]);
        let actual_size = u32::from_le_bytes([entry[28], entry[29], entry[30], entry[31]]) as usize;
        if actual_cluster != change.start_cluster || actual_size != change.old_size {
            return Err(format!("{}: 根目录项与解析快照不一致", change.path));
        }
        let start_cluster = chain.first().copied().unwrap_or(0);
        entry[26..28].copy_from_slice(&start_cluster.to_le_bytes());
        let size = u32::try_from(change.data.len())
            .map_err(|_| format!("{}: 文件超过 4 GiB", change.path))?;
        entry[28..32].copy_from_slice(&size.to_le_bytes());
    }
    logical[FAT1_OFFSET..FAT1_OFFSET + FAT_BYTES].copy_from_slice(&fat);
    logical[FAT2_OFFSET..FAT2_OFFSET + FAT_BYTES].copy_from_slice(&fat);

    let mut output = source_bytes.to_vec();
    let mut logical_offset = 0usize;
    for track in &disk.tracks {
        for sector in &track.sectors {
            let end = logical_offset + sector.data_range.len();
            output[sector.data_range.clone()].copy_from_slice(&logical[logical_offset..end]);
            logical_offset = end;
        }
    }
    if logical_offset != logical.len() {
        return Err("D88 扇区映射没有覆盖完整 ENIX-DOS 逻辑盘".to_owned());
    }

    // One bounded postcondition pass: ensure the finished container exposes every
    // requested file with exactly the rebuilt bytes.
    let verified_image = codec
        .decode(&output)
        .map_err(|error| format!("重建 D88 无法重新解析: {error}"))?;
    let verified_logical = materialize_logical(&output, &verified_image.disks[0])?;
    let verified = parse_enix_dos(&verified_logical, source_path)?;
    for (path, expected) in replacements {
        let actual = verified
            .files
            .iter()
            .find(|file| file.manifest.path.eq_ignore_ascii_case(path))
            .ok_or_else(|| format!("回包后找不到 {path}"))?;
        if actual.data != *expected {
            return Err(format!("回包后 {path} 内容不匹配"));
        }
    }
    Ok(output)
}

fn find_root_entry_offset(logical: &[u8], path: &str) -> Result<usize> {
    for index in 0..ROOT_ENTRIES {
        let offset = ROOT_OFFSET + index * DIRECTORY_ENTRY_SIZE;
        let record = &logical[offset..offset + DIRECTORY_ENTRY_SIZE];
        match record[0] {
            0x00 => break,
            0xe5 => continue,
            _ => {}
        }
        if record[11] == 0x0f || record[11] & 0x18 != 0 {
            continue;
        }
        if decode_short_name(record).is_ok_and(|name| name.eq_ignore_ascii_case(path)) {
            return Ok(offset);
        }
    }
    Err(format!("根目录中找不到文件 {path}"))
}

fn fat12_set(fat: &mut [u8], cluster: u16, value: u16) -> Result<()> {
    if value > 0x0fff {
        return Err(format!("FAT12 值超出 12 位: 0x{value:X}"));
    }
    let cluster = usize::from(cluster);
    let offset = cluster + cluster / 2;
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

fn fat12_entry(fat: &[u8], cluster: u16) -> Result<u16> {
    let cluster = usize::from(cluster);
    let offset = cluster + cluster / 2;
    let bytes = fat
        .get(offset..offset + 2)
        .ok_or_else(|| format!("FAT12 项 {cluster} 越界"))?;
    let pair = u16::from_le_bytes([bytes[0], bytes[1]]);
    Ok(if cluster & 1 == 0 {
        pair & 0x0FFF
    } else {
        pair >> 4
    })
}

fn decode_short_name(record: &[u8]) -> Result<String> {
    let base = decode_name_part(&record[..8])?;
    let extension = decode_name_part(&record[8..11])?;
    if base.is_empty() {
        return Err("ENIX-DOS 目录项文件名为空".into());
    }
    if extension.is_empty() {
        Ok(base)
    } else {
        Ok(format!("{base}.{extension}"))
    }
}

fn decode_name_part(bytes: &[u8]) -> Result<String> {
    let end = bytes
        .iter()
        .rposition(|byte| *byte != b' ' && *byte != 0)
        .map_or(0, |index| index + 1);
    let bytes = &bytes[..end];
    if bytes.contains(&0) {
        return Err(format!("ENIX-DOS 短名称内部含 NUL: {}", hex_upper(bytes)));
    }
    let decoded = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(bytes)
        .ok_or_else(|| format!("ENIX-DOS 短名称不是有效 CP932: {}", hex_upper(bytes)))?;
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&decoded);
    if had_errors || encoded.as_ref() != bytes {
        return Err(format!(
            "ENIX-DOS 短名称不能按 CP932 原样往返: {}",
            hex_upper(bytes)
        ));
    }
    if decoded
        .chars()
        .any(|ch| ch.is_control() || matches!(ch, '/' | '\\' | ':'))
    {
        return Err(format!("ENIX-DOS 短名称含不安全字符: {}", hex_upper(bytes)));
    }
    Ok(decoded.into_owned())
}

fn discover_inputs(selected: &[PathBuf]) -> Result<Vec<PathBuf>> {
    if selected.is_empty() {
        return Err("至少需要一个 D88 文件或目录".into());
    }
    let mut inputs = Vec::new();
    for selected_path in selected {
        let path = fs::canonicalize(selected_path)
            .map_err(|error| format!("无法访问 {}: {error}", selected_path.display()))?;
        if path.is_file() {
            if !is_d88(&path) {
                return Err(format!("输入文件不是 D88: {}", path.display()));
            }
            inputs.push(path);
        } else if path.is_dir() {
            let mut children = fs::read_dir(&path)
                .map_err(|error| format!("无法读取 {}: {error}", path.display()))?
                .map(|entry| entry.map(|item| item.path()))
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|error| format!("枚举 {} 失败: {error}", path.display()))?;
            children.sort();
            inputs.extend(
                children
                    .into_iter()
                    .filter(|child| child.is_file() && is_d88(child)),
            );
        }
    }
    inputs.sort();
    inputs.dedup_by(|left, right| {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    });
    if inputs.is_empty() {
        return Err("所选路径中没有 D88 文件".into());
    }
    Ok(inputs)
}

fn is_d88(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("d88"))
}

fn safe_component(name: &str) -> String {
    let mut safe = name
        .chars()
        .map(|ch| {
            if ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
            {
                '_'
            } else {
                ch
            }
        })
        .collect::<String>();
    while safe.ends_with([' ', '.']) {
        safe.pop();
    }
    let stem = safe.split('.').next().unwrap_or("");
    if matches!(
        stem.to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    ) {
        safe.push('_');
    }
    if safe.is_empty() {
        "_".into()
    } else {
        safe
    }
}

fn safe_join(root: &Path, relative: &Path) -> Result<PathBuf> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(format!("不安全的输出相对路径: {}", relative.display()));
    }
    Ok(root.join(relative))
}

fn commit_staging(staging: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if !output.exists() {
        return fs::rename(staging, output)
            .map_err(|error| format!("提交输出 {} 失败: {error}", output.display()));
    }
    if !overwrite {
        return Err(format!("输出已存在，默认不覆盖: {}", output.display()));
    }
    let backup = unique_sibling(output, "backup")?;
    fs::rename(output, &backup)
        .map_err(|error| format!("暂存旧输出 {} 失败: {error}", output.display()))?;
    if let Err(error) = fs::rename(staging, output) {
        let rollback = fs::rename(&backup, output);
        return match rollback {
            Ok(()) => Err(format!("提交失败，已恢复旧输出: {error}")),
            Err(rollback_error) => Err(format!(
                "提交失败且旧输出恢复失败；备份位于 {}: {error}; {rollback_error}",
                backup.display()
            )),
        };
    }
    fs::remove_dir_all(&backup).map_err(|error| format!("新输出已提交，但清理旧备份失败: {error}"))
}

fn unique_sibling(path: &Path, label: &str) -> Result<PathBuf> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("路径没有父目录: {}", path.display()))?;
    let leaf = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("路径名无法表示为 Unicode: {}", path.display()))?;
    for index in 0..1000usize {
        let candidate = parent.join(format!(".{leaf}.{label}-{}-{index}", std::process::id()));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(format!("无法为 {} 分配临时目录", path.display()))
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("序列化 {} 失败: {error}", path.display()))?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(|error| format!("写入 {} 失败: {error}", path.display()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex_upper(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
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

    fn sample_enix_d88() -> Vec<u8> {
        let mut logical = vec![0u8; LOGICAL_SIZE];
        let mut fat = vec![0u8; FAT_BYTES];
        fat[..3].copy_from_slice(&[0xfe, 0xff, 0xff]);
        fat12_set(&mut fat, 2, 0xfff).unwrap();
        logical[FAT1_OFFSET..FAT1_OFFSET + FAT_BYTES].copy_from_slice(&fat);
        logical[FAT2_OFFSET..FAT2_OFFSET + FAT_BYTES].copy_from_slice(&fat);
        logical[ROOT_OFFSET..ROOT_OFFSET + 11].copy_from_slice(b"HELLO   BUN");
        logical[ROOT_OFFSET + 11] = 0x20;
        put_u16(&mut logical, ROOT_OFFSET + 26, 2);
        put_u32(&mut logical, ROOT_OFFSET + 28, 4);
        logical[DATA_OFFSET..DATA_OFFSET + 4].copy_from_slice(b"test");

        let track_bytes = SECTORS_PER_TRACK * (16 + BYTES_PER_SECTOR);
        let mut d88 = vec![0u8; 0x2b0 + TRACKS * track_bytes];
        d88[0x1b] = 0x20;
        for track in 0..TRACKS {
            let track_start = 0x2b0 + track * track_bytes;
            put_u32(&mut d88, 0x20 + track * 4, track_start as u32);
            for sector in 0..SECTORS_PER_TRACK {
                let header = track_start + sector * (16 + BYTES_PER_SECTOR);
                d88[header] = (track / 2) as u8;
                d88[header + 1] = (track % 2) as u8;
                d88[header + 2] = (sector + 1) as u8;
                d88[header + 3] = 3;
                put_u16(&mut d88, header + 4, SECTORS_PER_TRACK as u16);
                put_u16(&mut d88, header + 14, BYTES_PER_SECTOR as u16);
                let source = (track * SECTORS_PER_TRACK + sector) * BYTES_PER_SECTOR;
                d88[header + 16..header + 16 + BYTES_PER_SECTOR]
                    .copy_from_slice(&logical[source..source + BYTES_PER_SECTOR]);
            }
        }
        let size = d88.len() as u32;
        put_u32(&mut d88, 0x1c, size);
        d88
    }

    #[test]
    fn fat12_pairs_decode_even_and_odd_entries() {
        let fat = [0xFE, 0xFF, 0xFF, 0x03, 0x40, 0x00];
        assert_eq!(fat12_entry(&fat, 0).unwrap(), 0xFFE);
        assert_eq!(fat12_entry(&fat, 1).unwrap(), 0xFFF);
        assert_eq!(fat12_entry(&fat, 2).unwrap(), 0x003);
        assert_eq!(fat12_entry(&fat, 3).unwrap(), 0x004);
    }

    #[test]
    fn short_names_preserve_real_resource_names() {
        let mut record = [b' '; 32];
        record[..8].copy_from_slice(b"MAIN    ");
        record[8..11].copy_from_slice(b"EXE");
        assert_eq!(decode_short_name(&record).unwrap(), "MAIN.EXE");
    }

    #[test]
    fn growing_file_is_reallocated_and_written_back_to_complete_d88() {
        let source = sample_enix_d88();
        let replacement = vec![0x5a; 1500];
        let mut replacements = BTreeMap::new();
        replacements.insert("HELLO.BUN".to_owned(), replacement.clone());
        let rebuilt = rebuild_d88(Path::new("sample.d88"), &source, &replacements).unwrap();
        assert_eq!(rebuilt.len(), source.len());
        let image = StandardCodec.decode(&rebuilt).unwrap();
        let logical = materialize_logical(&rebuilt, &image.disks[0]).unwrap();
        let parsed = parse_enix_dos(&logical, Path::new("sample.d88")).unwrap();
        assert_eq!(parsed.files[0].data, replacement);
        assert_eq!(parsed.files[0].manifest.cluster_chain.len(), 2);
    }
}
