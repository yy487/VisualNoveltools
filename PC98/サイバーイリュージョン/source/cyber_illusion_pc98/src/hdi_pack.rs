//! Build a bootable PC-98 FAT12 HDI from the blank image and DOS tree.

use crate::{json_bytes, output, sha256_hex, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

const DOS_PARTITION_MIB: usize = 20;
const FAT_BYTES_PER_SECTOR: usize = 1024;
const FAT_SECTORS_PER_CLUSTER: usize = 8;
const FAT_RESERVED_SECTORS: usize = 1;
const FAT_COPIES: usize = 2;
const FAT_ROOT_ENTRIES: usize = 768;
const FAT_SECTORS_PER_FAT: usize = 4;
const FAT12_MAX_CLUSTERS: usize = 4084;
const DIRECTORY_ENTRY_BYTES: usize = 32;
const IMAGE_FILE: &str = "cyber_illusion.hdi";
const FONT_FILE: &str = "font.tmp";
const FONT_MAP_FILE: &str = "font98_mapping.json";

#[derive(Debug, Clone, Deserialize)]
struct SourceManifest {
    source_file: String,
    source_sha256: String,
    hdi: SourceHdi,
    partition: SourcePartition,
    fat12: SourceFat12,
    files: Vec<SourceFile>,
}

#[derive(Debug, Clone, Deserialize)]
struct SourceFile {
    path: String,
    size: u32,
    sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SourceHdi {
    header_size: u32,
    disk_size: u32,
    physical_sector_size: u32,
    sectors_per_track: u32,
    heads: u32,
    cylinders: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct SourcePartition {
    table_offset: u64,
    entry_offset: u64,
    raw_entry_hex: String,
    start_lba: u32,
    byte_offset: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct SourceFat12 {
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fat_copies: u8,
    root_entries: u16,
    total_sectors: u32,
    media_descriptor: u8,
    sectors_per_fat: u16,
    sectors_per_track: u16,
    heads: u16,
    hidden_sectors: u32,
}

#[derive(Debug, Clone, Serialize)]
struct HdiBuildManifest {
    schema: String,
    blank_hdi: String,
    blank_sha256: String,
    dos_template_hdi: String,
    dos_template_sha256: String,
    source_manifest_hdi_sha256: String,
    source_hdi_hash_matches_manifest: bool,
    dos_system_files_match_manifest: bool,
    ipl_sector_sha256: String,
    ipl_boot_area_bytes: usize,
    ipl_boot_area_source_sha256: String,
    game_directory: String,
    hdi_bytes: usize,
    disk_capacity_bytes: usize,
    partition_capacity_bytes: usize,
    partition_fat: String,
    bytes_per_sector: usize,
    sectors_per_cluster: usize,
    cluster_bytes: usize,
    file_count: usize,
    payload_bytes: u64,
    autoexec: String,
    font_file: String,
    font_sha256: String,
    font_mapping_sha256: String,
}

#[derive(Debug, Clone, Copy)]
struct HdiHeader {
    header_size: usize,
    disk_size: usize,
    physical_sector_size: usize,
    sectors_per_track: usize,
    heads: usize,
    cylinders: usize,
}

#[derive(Debug, Clone)]
struct DiskFile {
    name: [u8; 11],
    data: Vec<u8>,
    attributes: u8,
    cluster_chain: Vec<u16>,
}

#[derive(Debug, Clone)]
enum DiskItem {
    File(DiskFile),
    Directory(DiskDirectory),
}

#[derive(Debug, Clone)]
struct DiskDirectory {
    name: [u8; 11],
    items: Vec<DiskItem>,
    parent_cluster: u16,
    cluster_chain: Vec<u16>,
}

#[derive(Debug, Clone, Copy)]
struct FatLayout {
    partition_offset: usize,
    partition_bytes: usize,
    bytes_per_sector: usize,
    reserved_sectors: usize,
    fat_copies: usize,
    root_entries: usize,
    sectors_per_fat: usize,
    first_data_sector: usize,
    cluster_bytes: usize,
}

struct BuiltImage {
    bytes: Vec<u8>,
    files: usize,
    payload_bytes: u64,
}

/// Prepare an isolated package directory containing the HDI and NP2 font sidecars.
pub fn prepare(
    blank_hdi_path: &Path,
    dos_root_path: &Path,
    game_root_path: &Path,
    output_directory: &Path,
) -> Result<output::Prepared> {
    let blank_hdi_path = canonical_regular_path(blank_hdi_path, "空白 HDI")?;
    let dos_root = canonical_directory(dos_root_path, "DOS 解包目录")?;
    let game_root = canonical_directory(game_root_path, "Cyber 回注目录")?;

    let dos_manifest_bytes =
        read_regular_file(&dos_root.join(".hdi_manifest.json"), "DOS 解包清单")?;
    let dos_manifest: SourceManifest = serde_json::from_slice(&dos_manifest_bytes)
        .map_err(|error| format!("DOS 解包清单 JSON 无效：{error}"))?;
    let source_file = safe_basename(&dos_manifest.source_file)?;
    let template_path = dos_root
        .parent()
        .ok_or_else(|| "DOS 解包目录缺少父目录".to_owned())?
        .join(source_file);
    let template_hdi = canonical_regular_path(&template_path, "DOS 基线 HDI")?;
    let template_bytes = fs::read(&template_hdi)
        .map_err(|error| format!("读取 DOS 基线 HDI 失败 {}：{error}", template_hdi.display()))?;
    let template_sha256 = sha256_hex(&template_bytes);
    let source_hdi_hash_matches_manifest =
        template_sha256.eq_ignore_ascii_case(&dos_manifest.source_sha256);
    let template_header = parse_hdi_header(&template_hdi, &template_bytes)?;
    verify_source_manifest(&dos_manifest, &template_header, &template_bytes)?;
    validate_dos_system_files(&dos_root, &dos_manifest.files)?;
    let ipl_sector = template_ipl_sector(&template_bytes, &template_header)?;
    let ipl_boot_area = template_ipl_boot_area(&template_bytes, &template_header, &dos_manifest)?;

    let blank_bytes = fs::read(&blank_hdi_path)
        .map_err(|error| format!("读取空白 HDI 失败 {}：{error}", blank_hdi_path.display()))?;
    let blank_header = parse_hdi_header(&blank_hdi_path, &blank_bytes)?;
    let built = build_image(
        &blank_bytes,
        &blank_header,
        &template_bytes,
        &template_header,
        &dos_manifest,
        &dos_root,
        &game_root,
    )?;
    let font_bytes = read_regular_file(&game_root.join(FONT_FILE), "NP2 自定义字库")?;
    let font_map_bytes = read_regular_file(&game_root.join(FONT_MAP_FILE), "NP2 字库映射")?;
    let manifest = HdiBuildManifest {
        schema: "cyber-illusion-pc98-hdi-v1".into(),
        blank_hdi: blank_hdi_path.display().to_string(),
        blank_sha256: sha256_hex(&blank_bytes),
        dos_template_hdi: template_hdi.display().to_string(),
        dos_template_sha256: template_sha256,
        source_manifest_hdi_sha256: dos_manifest.source_sha256,
        source_hdi_hash_matches_manifest,
        dos_system_files_match_manifest: true,
        ipl_sector_sha256: sha256_hex(ipl_sector),
        ipl_boot_area_bytes: ipl_boot_area.len(),
        ipl_boot_area_source_sha256: sha256_hex(ipl_boot_area),
        game_directory: game_root.display().to_string(),
        hdi_bytes: built.bytes.len(),
        disk_capacity_bytes: blank_header.disk_size,
        partition_capacity_bytes: DOS_PARTITION_MIB * 1024 * 1024,
        partition_fat: "FAT12".into(),
        bytes_per_sector: FAT_BYTES_PER_SECTOR,
        sectors_per_cluster: FAT_SECTORS_PER_CLUSTER,
        cluster_bytes: FAT_BYTES_PER_SECTOR * FAT_SECTORS_PER_CLUSTER,
        file_count: built.files,
        payload_bytes: built.payload_bytes,
        autoexec: "PROMPT $P$G; PATH A:\\DOS; CD CYBER; CYBER".into(),
        font_file: FONT_FILE.into(),
        font_sha256: sha256_hex(&font_bytes),
        font_mapping_sha256: sha256_hex(&font_map_bytes),
    };

    let mut payloads = std::collections::BTreeMap::new();
    payloads.insert(IMAGE_FILE.into(), built.bytes);
    payloads.insert(FONT_FILE.into(), font_bytes);
    payloads.insert(FONT_MAP_FILE.into(), font_map_bytes);
    payloads.insert("README.txt".into(), package_readme().into_bytes());
    payloads.insert("hdi_build_manifest.json".into(), json_bytes(&manifest)?);
    let output_file_count = payloads.len();
    let warnings = if source_hdi_hash_matches_manifest {
        Vec::new()
    } else {
        vec![
            "DOS 解包绑定的原始 HDI 与当前同名文件总哈希不同；引导扇区、分区项/BPB 与 DOS 系统文件逐项通过清单校验".into(),
        ]
    };
    let details = vec![
        format!("空白 HDI：{}", blank_hdi_path.display()),
        format!("DOS 基线：{}", dos_root.display()),
        format!("HDI 容量：{} MiB", blank_header.disk_size / (1024 * 1024)),
        format!("PC-98 DOS 分区：{} MiB FAT12", DOS_PARTITION_MIB),
        format!("DOS 与游戏文件：{} 个", manifest.file_count),
        format!("DOS 与游戏文件逻辑字节：{}", manifest.payload_bytes),
        "font.tmp 与映射清单作为 NP2 外部配套文件，不写入 DOS 分区".into(),
    ];
    output::Prepared::new(
        vec![blank_hdi_path, dos_root, game_root, template_hdi],
        output_directory,
        payloads,
        format!(
            "已生成 {} MiB PC-98 DOS 硬盘镜像并附带 NP2 字库",
            DOS_PARTITION_MIB
        ),
        vec![
            "保留空白 HDI 的 Anex86 几何，复用 DOS 基线到分区起始前的完整 IPL 启动区".into(),
            "复用白金之星 DOS 基线的分区前 IPL 装载区、PC-98 分区表和 DOS 5 引导文件".into(),
            "建立 20 MiB FAT12 分区并写入 Cyber 游戏目录".into(),
            "在 DOS AUTOEXEC 中进入 CYBER 并启动 CYBER.BAT".into(),
            "单独输出 NP2 font.tmp、字库映射及构建清单".into(),
            "暂存输出、逐文件读回核对后提交".into(),
        ],
        details,
        output::Metrics {
            files: output_file_count,
            modules: 30,
            text_entries: 10_398,
            changed_entries: 0,
            warnings,
        },
    )
}

fn build_dos_root(dos_root: &Path, game_root: &Path) -> Result<DiskDirectory> {
    let mut root = DiskDirectory {
        name: [b' '; 11],
        items: Vec::new(),
        parent_cluster: 0,
        cluster_chain: Vec::new(),
    };
    root.items
        .push(DiskItem::File(load_named_file(dos_root, "IO.SYS", 0x27)?));
    root.items.push(DiskItem::File(load_named_file(
        dos_root,
        "MSDOS.SYS",
        0x27,
    )?));
    root.items.push(DiskItem::File(load_named_file(
        dos_root,
        "COMMAND.COM",
        0x20,
    )?));
    root.items.push(DiskItem::File(file_node(
        "CONFIG.SYS",
        read_regular_file(&dos_root.join("CONFIG.SYS"), "DOS CONFIG.SYS")?,
        0x20,
    )?));
    root.items.push(DiskItem::File(file_node(
        "AUTOEXEC.BAT",
        autoexec_bytes(),
        0x20,
    )?));
    root.items.push(DiskItem::Directory(load_directory(
        &dos_root.join("DOS"),
        "DOS",
        false,
    )?));
    root.items.push(DiskItem::Directory(load_directory(
        game_root, "CYBER", true,
    )?));
    ensure_unique_names(&root.items, "root")?;
    Ok(root)
}

fn autoexec_bytes() -> Vec<u8> {
    b"@ECHO OFF\r\nPROMPT $P$G\r\nPATH A:\\DOS\r\nCD CYBER\r\nCYBER\r\n\x1A".to_vec()
}

fn load_named_file(root: &Path, name: &str, attributes: u8) -> Result<DiskFile> {
    file_node(name, read_regular_file(&root.join(name), name)?, attributes)
}

fn load_directory(path: &Path, dos_name: &str, skip_host_sidecars: bool) -> Result<DiskDirectory> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("读取目录失败 {}：{error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("DOS 镜像输入必须是实际目录：{}", path.display()));
    }
    let mut entries = fs::read_dir(path)
        .map_err(|error| format!("遍历目录失败 {}：{error}", path.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| format!("读取目录项失败 {}：{error}", path.display()))?;
    entries.sort_by_key(|entry| entry.file_name());

    let mut items = Vec::new();
    for entry in entries {
        let child = entry.path();
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| format!("DOS 文件名不是 Unicode：{}", child.display()))?;
        if skip_host_sidecars && is_host_sidecar(&name) {
            continue;
        }
        let metadata = fs::symlink_metadata(&child)
            .map_err(|error| format!("读取文件元数据失败 {}：{error}", child.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("DOS 镜像输入不支持符号链接：{}", child.display()));
        }
        if metadata.is_dir() {
            items.push(DiskItem::Directory(load_directory(&child, &name, false)?));
        } else if metadata.is_file() {
            let data = fs::read(&child)
                .map_err(|error| format!("读取游戏文件失败 {}：{error}", child.display()))?;
            items.push(DiskItem::File(file_node(&name, data, 0x20)?));
        } else {
            return Err(format!("DOS 镜像输入包含特殊文件：{}", child.display()));
        }
    }
    ensure_unique_names(&items, &path.display().to_string())?;
    Ok(DiskDirectory {
        name: dos_short_name(dos_name)?,
        items,
        parent_cluster: 0,
        cluster_chain: Vec::new(),
    })
}

fn file_node(name: &str, data: Vec<u8>, attributes: u8) -> Result<DiskFile> {
    Ok(DiskFile {
        name: dos_short_name(name)?,
        data,
        attributes,
        cluster_chain: Vec::new(),
    })
}

fn is_host_sidecar(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    matches!(lower.as_str(), "font.tmp" | "font98_mapping.json")
        || [".i64", ".id0", ".id1", ".id2", ".nam", ".til"]
            .iter()
            .any(|extension| lower.ends_with(extension))
}

fn dos_short_name(name: &str) -> Result<[u8; 11]> {
    let mut parts = name.split('.');
    let base = parts.next().unwrap_or_default();
    let extension = parts.next().unwrap_or_default();
    if parts.next().is_some() || base.is_empty() || base.len() > 8 || extension.len() > 3 {
        return Err(format!("文件名不符合 DOS 8.3：{name}"));
    }
    let allowed = |byte: u8| byte.is_ascii_alphanumeric() || b"$%'-_@~!#&()^".contains(&byte);
    if !base.bytes().all(allowed) || !extension.bytes().all(allowed) {
        return Err(format!("文件名包含 DOS 8.3 不支持的字符：{name}"));
    }
    let mut output = [b' '; 11];
    for (index, byte) in base.bytes().enumerate() {
        output[index] = byte.to_ascii_uppercase();
    }
    for (index, byte) in extension.bytes().enumerate() {
        output[8 + index] = byte.to_ascii_uppercase();
    }
    Ok(output)
}

fn ensure_unique_names(items: &[DiskItem], parent: &str) -> Result<()> {
    let mut names = BTreeSet::new();
    for item in items {
        let name = match item {
            DiskItem::File(file) => file.name,
            DiskItem::Directory(directory) => directory.name,
        };
        if !names.insert(name) {
            return Err(format!(
                "{parent} 中有重复的 DOS 8.3 名称：{}",
                short_name_text(&name)
            ));
        }
    }
    Ok(())
}

fn assign_directory(
    directory: &mut DiskDirectory,
    is_root: bool,
    parent_cluster: u16,
    next_cluster: &mut u32,
    data_clusters: usize,
    cluster_bytes: usize,
) -> Result<()> {
    directory.parent_cluster = parent_cluster;
    if !is_root {
        let entry_count = directory
            .items
            .len()
            .checked_add(2)
            .ok_or_else(|| "子目录项数溢出".to_owned())?;
        let bytes = entry_count
            .checked_mul(DIRECTORY_ENTRY_BYTES)
            .ok_or_else(|| "子目录字节数溢出".to_owned())?;
        directory.cluster_chain =
            allocate_clusters(bytes.div_ceil(cluster_bytes), next_cluster, data_clusters)?;
    }
    let this_cluster = directory.cluster_chain.first().copied().unwrap_or(0);
    for item in &mut directory.items {
        match item {
            DiskItem::File(file) => {
                let clusters = file.data.len().div_ceil(cluster_bytes);
                file.cluster_chain = allocate_clusters(clusters, next_cluster, data_clusters)?;
            }
            DiskItem::Directory(child) => assign_directory(
                child,
                false,
                this_cluster,
                next_cluster,
                data_clusters,
                cluster_bytes,
            )?,
        }
    }
    Ok(())
}

fn allocate_clusters(
    count: usize,
    next_cluster: &mut u32,
    data_clusters: usize,
) -> Result<Vec<u16>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    let last = next_cluster
        .checked_add(count as u32 - 1)
        .ok_or_else(|| "FAT12 簇号溢出".to_owned())?;
    if last > (data_clusters + 1) as u32 || last > 0xFEF {
        return Err(format!(
            "DOS 文件需要更多 FAT12 簇：需要到簇 {last:04X}，分区上限为 {:04X}",
            data_clusters + 1
        ));
    }
    let chain = (*next_cluster..=last)
        .map(|cluster| u16::try_from(cluster).map_err(|_| "FAT12 簇号过大".to_owned()))
        .collect::<Result<Vec<_>>>()?;
    *next_cluster = last + 1;
    Ok(chain)
}

fn write_fat_and_directories(
    image: &mut [u8],
    root: &DiskDirectory,
    layout: &FatLayout,
) -> Result<()> {
    let fat_offset = layout.partition_offset + layout.reserved_sectors * layout.bytes_per_sector;
    let fat_bytes = layout.sectors_per_fat * layout.bytes_per_sector;
    let mut fat = vec![0u8; fat_bytes];
    fat[..3].copy_from_slice(&[0xF8, 0xFF, 0xFF]);
    write_directory_fat(root, &mut fat)?;
    for copy in 0..layout.fat_copies {
        let start = fat_offset + copy * fat_bytes;
        let end = start + fat_bytes;
        let target = image
            .get_mut(start..end)
            .ok_or_else(|| "FAT12 表超出 HDI".to_owned())?;
        target.copy_from_slice(&fat);
    }

    let root_offset = layout.partition_offset
        + (layout.reserved_sectors + layout.fat_copies * layout.sectors_per_fat)
            * layout.bytes_per_sector;
    let root_bytes = layout.root_entries * DIRECTORY_ENTRY_BYTES;
    let mut root_data = vec![0u8; root_bytes];
    write_directory_entries(root, true, &mut root_data)?;
    let root_end = root_offset + root_data.len();
    image
        .get_mut(root_offset..root_end)
        .ok_or_else(|| "FAT12 根目录超出 HDI".to_owned())?
        .copy_from_slice(&root_data);

    let data_offset = layout.partition_offset + layout.first_data_sector * layout.bytes_per_sector;
    write_item_data(image, root, true, data_offset, layout)?;
    let partition_end = layout.partition_offset + layout.partition_bytes;
    if partition_end > image.len() {
        return Err("FAT12 分区超出 HDI".into());
    }
    Ok(())
}

fn write_directory_fat(directory: &DiskDirectory, fat: &mut [u8]) -> Result<()> {
    set_fat_chain(fat, &directory.cluster_chain)?;
    for item in &directory.items {
        match item {
            DiskItem::File(file) => set_fat_chain(fat, &file.cluster_chain)?,
            DiskItem::Directory(child) => write_directory_fat(child, fat)?,
        }
    }
    Ok(())
}

fn set_fat_chain(fat: &mut [u8], chain: &[u16]) -> Result<()> {
    for pair in chain.windows(2) {
        set_fat12(fat, pair[0], pair[1])?;
    }
    if let Some(last) = chain.last() {
        set_fat12(fat, *last, 0xFFF)?;
    }
    Ok(())
}

fn set_fat12(fat: &mut [u8], cluster: u16, value: u16) -> Result<()> {
    let offset = usize::from(cluster) * 3 / 2;
    if offset + 1 >= fat.len() || value > 0x0FFF {
        return Err(format!("FAT12 项越界：簇 {cluster:04X}"));
    }
    if cluster & 1 == 0 {
        fat[offset] = value as u8;
        fat[offset + 1] = (fat[offset + 1] & 0xF0) | ((value >> 8) as u8 & 0x0F);
    } else {
        fat[offset] = (fat[offset] & 0x0F) | ((value << 4) as u8 & 0xF0);
        fat[offset + 1] = (value >> 4) as u8;
    }
    Ok(())
}

fn write_item_data(
    image: &mut [u8],
    directory: &DiskDirectory,
    is_root: bool,
    data_offset: usize,
    layout: &FatLayout,
) -> Result<()> {
    if !is_root {
        let buffer_size = directory.cluster_chain.len() * layout.cluster_bytes;
        let mut buffer = vec![0u8; buffer_size];
        write_directory_entries(directory, false, &mut buffer)?;
        write_clusters(
            image,
            data_offset,
            layout.cluster_bytes,
            &directory.cluster_chain,
            &buffer,
        )?;
    }
    for item in &directory.items {
        match item {
            DiskItem::File(file) => write_clusters(
                image,
                data_offset,
                layout.cluster_bytes,
                &file.cluster_chain,
                &file.data,
            )?,
            DiskItem::Directory(child) => {
                write_item_data(image, child, false, data_offset, layout)?;
            }
        }
    }
    Ok(())
}

fn write_directory_entries(
    directory: &DiskDirectory,
    is_root: bool,
    output: &mut [u8],
) -> Result<()> {
    let capacity = output.len() / DIRECTORY_ENTRY_BYTES;
    let mut index = 0usize;
    if !is_root {
        let current = directory.cluster_chain.first().copied().unwrap_or(0);
        write_directory_entry(output, index, &dot_name(false), 0x10, current, 0)?;
        index += 1;
        write_directory_entry(
            output,
            index,
            &dot_name(true),
            0x10,
            directory.parent_cluster,
            0,
        )?;
        index += 1;
    }
    for item in &directory.items {
        let (name, attributes, start_cluster, size) = match item {
            DiskItem::File(file) => (
                file.name,
                file.attributes,
                file.cluster_chain.first().copied().unwrap_or(0),
                u32::try_from(file.data.len())
                    .map_err(|_| format!("DOS 文件过大：{}", short_name_text(&file.name)))?,
            ),
            DiskItem::Directory(child) => (
                child.name,
                0x10,
                child.cluster_chain.first().copied().unwrap_or(0),
                0,
            ),
        };
        write_directory_entry(output, index, &name, attributes, start_cluster, size)?;
        index += 1;
    }
    if index > capacity {
        return Err(format!("目录项数 {index} 超过容量 {capacity}"));
    }
    Ok(())
}

fn dot_name(parent: bool) -> [u8; 11] {
    let mut name = [b' '; 11];
    name[0] = b'.';
    if parent {
        name[1] = b'.';
    }
    name
}

fn write_directory_entry(
    output: &mut [u8],
    index: usize,
    name: &[u8; 11],
    attributes: u8,
    start_cluster: u16,
    size: u32,
) -> Result<()> {
    let offset = index
        .checked_mul(DIRECTORY_ENTRY_BYTES)
        .ok_or_else(|| "目录项偏移溢出".to_owned())?;
    let entry = output
        .get_mut(offset..offset + DIRECTORY_ENTRY_BYTES)
        .ok_or_else(|| format!("目录项 {index} 超出目录缓冲区"))?;
    entry.fill(0);
    entry[..11].copy_from_slice(name);
    entry[11] = attributes;
    entry[22..24].copy_from_slice(&FAT_TIME.to_le_bytes());
    entry[24..26].copy_from_slice(&FAT_DATE.to_le_bytes());
    entry[26..28].copy_from_slice(&start_cluster.to_le_bytes());
    entry[28..32].copy_from_slice(&size.to_le_bytes());
    Ok(())
}

const FAT_TIME: u16 = 16 << 11; // 16:00:00
const FAT_DATE: u16 = ((1995 - 1980) << 9) | (10 << 5) | 11;

fn write_clusters(
    image: &mut [u8],
    data_offset: usize,
    cluster_bytes: usize,
    chain: &[u16],
    data: &[u8],
) -> Result<()> {
    if data.len() > chain.len() * cluster_bytes {
        return Err("文件数据超过已分配簇链容量".into());
    }
    for (index, cluster) in chain.iter().enumerate() {
        let start = data_offset
            .checked_add(usize::from(*cluster - 2) * cluster_bytes)
            .ok_or_else(|| "簇数据偏移溢出".to_owned())?;
        let end = start
            .checked_add(cluster_bytes)
            .ok_or_else(|| "簇数据范围溢出".to_owned())?;
        let target = image
            .get_mut(start..end)
            .ok_or_else(|| format!("簇 {cluster:04X} 超出 HDI"))?;
        let from = index * cluster_bytes;
        let count = data.len().saturating_sub(from).min(cluster_bytes);
        if count > 0 {
            target[..count].copy_from_slice(&data[from..from + count]);
        }
    }
    Ok(())
}

fn count_items(root: &DiskDirectory) -> (usize, u64) {
    fn recurse(directory: &DiskDirectory, files: &mut usize, bytes: &mut u64) {
        for item in &directory.items {
            match item {
                DiskItem::File(file) => {
                    *files += 1;
                    *bytes += file.data.len() as u64;
                }
                DiskItem::Directory(child) => {
                    recurse(child, files, bytes);
                }
            }
        }
    }
    let mut files = 0;
    let mut bytes = 0;
    recurse(root, &mut files, &mut bytes);
    (files, bytes)
}

fn verify_source_manifest(
    manifest: &SourceManifest,
    header: &HdiHeader,
    image: &[u8],
) -> Result<()> {
    if manifest.hdi.header_size as usize != header.header_size
        || manifest.hdi.disk_size as usize != header.disk_size
        || manifest.hdi.physical_sector_size as usize != header.physical_sector_size
        || manifest.hdi.sectors_per_track as usize != header.sectors_per_track
        || manifest.hdi.heads as usize != header.heads
        || manifest.hdi.cylinders as usize != header.cylinders
    {
        return Err("DOS 解包清单中的 HDI 几何与基线镜像不一致".into());
    }
    let table_offset = usize::try_from(manifest.partition.table_offset)
        .map_err(|_| "DOS 清单分区表偏移过大".to_owned())?;
    let entry_offset = usize::try_from(manifest.partition.entry_offset)
        .map_err(|_| "DOS 清单分区项偏移过大".to_owned())?;
    let raw_entry = parse_hex(&manifest.partition.raw_entry_hex)?;
    if image.get(entry_offset..entry_offset + raw_entry.len()) != Some(raw_entry.as_slice())
        || table_offset != header.header_size + header.physical_sector_size
    {
        return Err("DOS 解包清单分区表项与基线 HDI 不匹配".into());
    }
    let expected_partition_offset =
        header.header_size + manifest.partition.start_lba as usize * header.physical_sector_size;
    if expected_partition_offset != manifest.partition.byte_offset as usize {
        return Err("DOS 清单分区 CHS/LBA 字节偏移不一致".into());
    }
    let boot_offset = usize::try_from(manifest.partition.byte_offset)
        .map_err(|_| "DOS 清单分区引导扇区偏移过大".to_owned())?;
    let boot = image
        .get(boot_offset..boot_offset + 64)
        .ok_or_else(|| "DOS 基线分区引导扇区被截断".to_owned())?;
    let total_sectors_16 = read_u16(boot, 19)?;
    let total_sectors = if total_sectors_16 == 0 {
        read_u32(boot, 32)?
    } else {
        u32::from(total_sectors_16)
    };
    if read_u16(boot, 11)? != manifest.fat12.bytes_per_sector
        || boot[13] != manifest.fat12.sectors_per_cluster
        || read_u16(boot, 14)? != manifest.fat12.reserved_sectors
        || boot[16] != manifest.fat12.fat_copies
        || read_u16(boot, 17)? != manifest.fat12.root_entries
        || total_sectors != manifest.fat12.total_sectors
        || boot[21] != manifest.fat12.media_descriptor
        || read_u16(boot, 22)? != manifest.fat12.sectors_per_fat
        || read_u16(boot, 24)? != manifest.fat12.sectors_per_track
        || read_u16(boot, 26)? != manifest.fat12.heads
        || read_u32(boot, 28)? != manifest.fat12.hidden_sectors
        || boot.get(3..11) != Some(b"NEC  5.0")
    {
        return Err("DOS 基线卷引导扇区/BPB 与解包清单不一致".into());
    }
    Ok(())
}

fn template_ipl_sector<'a>(image: &'a [u8], header: &HdiHeader) -> Result<&'a [u8]> {
    let start = header.header_size;
    let end = start
        .checked_add(header.physical_sector_size)
        .ok_or_else(|| "DOS 基线 IPL1 扇区范围溢出".to_owned())?;
    let ipl = image
        .get(start..end)
        .ok_or_else(|| "DOS 基线 IPL1 扇区被截断".to_owned())?;
    if ipl.get(0..8) != Some(&[0xEB, 0x0A, 0x90, 0x90, b'I', b'P', b'L', b'1'][..])
        || ipl.get(ipl.len().saturating_sub(2)..) != Some(&[0x55, 0xAA][..])
    {
        return Err("DOS 基线硬盘首扇区不是带 55AA 签名的 PC-98 IPL1".into());
    }
    Ok(ipl)
}

fn template_ipl_boot_area<'a>(
    image: &'a [u8],
    header: &HdiHeader,
    manifest: &SourceManifest,
) -> Result<&'a [u8]> {
    template_ipl_sector(image, header)?;
    let start_lba = usize::try_from(manifest.partition.start_lba)
        .map_err(|_| "DOS 基线分区起始 LBA 过大".to_owned())?;
    if start_lba < 2 {
        return Err("DOS 基线分区前没有完整的 PC-98 IPL 保留区".into());
    }
    let end = header
        .header_size
        .checked_add(
            start_lba
                .checked_mul(header.physical_sector_size)
                .ok_or_else(|| "DOS 基线 IPL 保留区长度溢出".to_owned())?,
        )
        .ok_or_else(|| "DOS 基线 IPL 保留区范围溢出".to_owned())?;
    image
        .get(header.header_size..end)
        .ok_or_else(|| "DOS 基线 IPL 保留区被截断".to_owned())
}

fn validate_dos_system_files(dos_root: &Path, bindings: &[SourceFile]) -> Result<()> {
    for name in [
        "IO.SYS",
        "MSDOS.SYS",
        "COMMAND.COM",
        "CONFIG.SYS",
        "AUTOEXEC.BAT",
    ] {
        validate_bound_file(dos_root, name, bindings)?;
    }
    validate_bound_directory(dos_root, &dos_root.join("DOS"), "DOS", bindings)
}

fn validate_bound_directory(
    root: &Path,
    directory: &Path,
    relative: &str,
    bindings: &[SourceFile],
) -> Result<()> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("遍历 DOS 系统目录失败 {}：{error}", directory.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| format!("读取 DOS 系统目录项失败：{error}"))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let child = entry.path();
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| format!("DOS 系统文件名不是 Unicode：{}", child.display()))?;
        let child_relative = format!("{relative}/{name}");
        let metadata = fs::symlink_metadata(&child)
            .map_err(|error| format!("读取 DOS 系统文件元数据失败：{error}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("DOS 系统树不支持符号链接：{}", child.display()));
        }
        if metadata.is_dir() {
            validate_bound_directory(root, &child, &child_relative, bindings)?;
        } else if metadata.is_file() {
            validate_bound_file(root, &child_relative, bindings)?;
        } else {
            return Err(format!("DOS 系统树包含特殊文件：{}", child.display()));
        }
    }
    Ok(())
}

fn validate_bound_file(root: &Path, relative: &str, bindings: &[SourceFile]) -> Result<()> {
    let binding = bindings
        .iter()
        .find(|binding| binding.path.eq_ignore_ascii_case(relative))
        .ok_or_else(|| format!("DOS 解包清单缺少系统文件：{relative}"))?;
    let path = root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
    let bytes = read_regular_file(&path, "DOS 系统文件")?;
    if bytes.len() != binding.size as usize
        || !sha256_hex(&bytes).eq_ignore_ascii_case(&binding.sha256)
    {
        return Err(format!("DOS 系统文件与解包清单哈希不符：{relative}"));
    }
    Ok(())
}

fn parse_hdi_header(path: &Path, bytes: &[u8]) -> Result<HdiHeader> {
    if bytes.len() < 0x20 {
        return Err(format!("HDI 头被截断：{}", path.display()));
    }
    let header = HdiHeader {
        header_size: read_u32(bytes, 0x08)? as usize,
        disk_size: read_u32(bytes, 0x0C)? as usize,
        physical_sector_size: read_u32(bytes, 0x10)? as usize,
        sectors_per_track: read_u32(bytes, 0x14)? as usize,
        heads: read_u32(bytes, 0x18)? as usize,
        cylinders: read_u32(bytes, 0x1C)? as usize,
    };
    let geometry = header
        .physical_sector_size
        .checked_mul(header.sectors_per_track)
        .and_then(|value| value.checked_mul(header.heads))
        .and_then(|value| value.checked_mul(header.cylinders))
        .ok_or_else(|| "HDI 几何容量溢出".to_owned())?;
    if header.header_size < 0x20
        || header.disk_size != geometry
        || header.header_size + header.disk_size != bytes.len()
        || header.physical_sector_size != 512
        || header.sectors_per_track == 0
        || header.heads == 0
    {
        return Err(format!("HDI 头、长度或几何无效：{}", path.display()));
    }
    Ok(header)
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("读取{label}失败 {}：{error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("{label}必须是实际目录：{}", path.display()));
    }
    fs::canonicalize(path).map_err(|error| format!("解析{label}失败 {}：{error}", path.display()))
}

fn canonical_regular_path(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("读取{label}失败 {}：{error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("{label}必须是普通文件：{}", path.display()));
    }
    fs::canonicalize(path).map_err(|error| format!("解析{label}失败 {}：{error}", path.display()))
}

fn read_regular_file(path: &Path, label: &str) -> Result<Vec<u8>> {
    let path = canonical_regular_path(path, label)?;
    fs::read(&path).map_err(|error| format!("读取{label}失败 {}：{error}", path.display()))
}

fn safe_basename(name: &str) -> Result<&str> {
    let path = Path::new(name);
    if path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
        || name.contains('/')
        || name.contains('\\')
    {
        return Err(format!("DOS 清单源文件名不安全：{name}"));
    }
    Ok(name)
}

fn parse_hex(hex: &str) -> Result<Vec<u8>> {
    if hex.len() % 2 != 0 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("分区表项十六进制清单无效".into());
    }
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = hex_nibble(pair[0]).ok_or_else(|| "分区表项十六进制无效".to_owned())?;
            let low = hex_nibble(pair[1]).ok_or_else(|| "分区表项十六进制无效".to_owned())?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let part = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| format!("读取 u32 越界：0x{offset:X}"))?;
    Ok(u32::from_le_bytes([part[0], part[1], part[2], part[3]]))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let part = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| format!("读取 u16 越界：0x{offset:X}"))?;
    Ok(u16::from_le_bytes([part[0], part[1]]))
}

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) -> Result<()> {
    let target = bytes
        .get_mut(offset..offset + 2)
        .ok_or_else(|| format!("写入 u16 越界：0x{offset:X}"))?;
    target.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) -> Result<()> {
    let target = bytes
        .get_mut(offset..offset + 4)
        .ok_or_else(|| format!("写入 u32 越界：0x{offset:X}"))?;
    target.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn lba_to_chs(lba: usize, sectors_per_track: usize, heads: usize) -> Result<(u16, u8, u8)> {
    let track = sectors_per_track
        .checked_mul(heads)
        .ok_or_else(|| "CHS 轨道乘积溢出".to_owned())?;
    let cylinder = lba / track;
    let remainder = lba % track;
    let head = remainder / sectors_per_track;
    let sector = remainder % sectors_per_track;
    Ok((
        u16::try_from(cylinder).map_err(|_| "CHS 柱面超过 u16".to_owned())?,
        u8::try_from(head).map_err(|_| "CHS 磁头超过 u8".to_owned())?,
        u8::try_from(sector).map_err(|_| "CHS 扇区超过 u8".to_owned())?,
    ))
}

fn short_name_text(name: &[u8; 11]) -> String {
    let base = String::from_utf8_lossy(&name[..8]).trim_end().to_owned();
    let extension = String::from_utf8_lossy(&name[8..]).trim_end().to_owned();
    if extension.is_empty() {
        base
    } else {
        format!("{base}.{extension}")
    }
}

fn package_readme() -> String {
    format!(
        "Cyber Illusion PC-98 bootable HDI\r\n\r\nAttach {IMAGE_FILE} as HDD1 in NP2 and boot. The disk contains a 20 MiB PC-98 FAT12 partition with the reused DOS 5 system files and Cyber game files.\r\n\r\nThe host-side {FONT_FILE} is separate from the DOS filesystem. Load it through NP2's font directory/configuration so the simplified-Chinese carrier glyphs work. The BIOS ROM set remains in NP2's BIOS directory and is not included in this package.\r\n\r\nAfter the game exits, type CYBER at the DOS prompt to restart.\r\n"
    )
}

fn build_image(
    blank: &[u8],
    blank_header: &HdiHeader,
    template: &[u8],
    template_header: &HdiHeader,
    manifest: &SourceManifest,
    dos_root: &Path,
    game_root: &Path,
) -> Result<BuiltImage> {
    if blank_header.header_size != template_header.header_size
        || blank_header.physical_sector_size != template_header.physical_sector_size
        || blank_header.sectors_per_track != template_header.sectors_per_track
        || blank_header.heads != template_header.heads
    {
        return Err("空白 HDI 与 DOS 基线 HDI 的头部、扇区或 CHS 几何不兼容".into());
    }
    if blank_header.physical_sector_size != 512
        || blank_header.sectors_per_track != usize::from(manifest.fat12.sectors_per_track)
        || blank_header.heads != usize::from(manifest.fat12.heads)
        || usize::from(manifest.fat12.bytes_per_sector) != FAT_BYTES_PER_SECTOR
        || usize::from(manifest.fat12.fat_copies) != FAT_COPIES
        || usize::from(manifest.fat12.root_entries) != FAT_ROOT_ENTRIES
        || usize::from(manifest.fat12.sectors_per_fat) != FAT_SECTORS_PER_FAT
    {
        return Err("DOS 基线 FAT12 参数与计划的 20 MiB FAT12 分区不兼容".into());
    }

    let header_size = blank_header.header_size;
    let physical_sector = blank_header.physical_sector_size;
    let ipl_sector = template_ipl_sector(template, template_header)?;
    let ipl_boot_area = template_ipl_boot_area(template, template_header, manifest)?;
    let table_offset = header_size
        .checked_add(physical_sector)
        .ok_or_else(|| "分区表偏移溢出".to_owned())?;
    if blank
        .get(table_offset..)
        .ok_or_else(|| "空白 HDI 缺少 PC-98 分区表区".to_owned())?
        .iter()
        .any(|byte| *byte != 0)
    {
        return Err("newdisk.hdi 的分区表后含有非零数据；拒绝覆盖非空硬盘".into());
    }

    let start_lba = usize::try_from(manifest.partition.start_lba)
        .map_err(|_| "DOS 基线分区起始 LBA 过大".to_owned())?;
    let ipl_boot_area_end = header_size
        .checked_add(
            start_lba
                .checked_mul(physical_sector)
                .ok_or_else(|| "IPL 保留区长度溢出".to_owned())?,
        )
        .ok_or_else(|| "IPL 保留区范围溢出".to_owned())?;
    let partition_offset = header_size
        .checked_add(
            start_lba
                .checked_mul(physical_sector)
                .ok_or_else(|| "新 HDI 分区偏移溢出".to_owned())?,
        )
        .ok_or_else(|| "新 HDI 分区偏移溢出".to_owned())?;
    let bytes_per_sector = FAT_BYTES_PER_SECTOR;
    let total_sectors = DOS_PARTITION_MIB * 1024 * 1024 / bytes_per_sector;
    if total_sectors > u16::MAX as usize {
        return Err("分区扇区数不适合 DOS 5 FAT12 BPB".into());
    }
    let partition_bytes = total_sectors
        .checked_mul(bytes_per_sector)
        .ok_or_else(|| "FAT12 分区容量溢出".to_owned())?;
    let partition_end = partition_offset
        .checked_add(partition_bytes)
        .ok_or_else(|| "FAT12 分区范围溢出".to_owned())?;
    if partition_end > blank.len() {
        return Err(format!(
            "20 MiB FAT12 分区结束于 0x{partition_end:X}，超出空白 HDI 长度 0x{:X}",
            blank.len()
        ));
    }

    let source_table_offset = usize::try_from(manifest.partition.table_offset)
        .map_err(|_| "DOS 基线分区表偏移过大".to_owned())?;
    let source_entry_offset = usize::try_from(manifest.partition.entry_offset)
        .map_err(|_| "DOS 基线分区项偏移过大".to_owned())?;
    if source_table_offset != table_offset {
        return Err("DOS 基线 HDI 与空白 HDI 的 PC-98 分区表位置不同".into());
    }
    let table_sector_end = source_table_offset
        .checked_add(physical_sector)
        .ok_or_else(|| "PC-98 分区表范围溢出".to_owned())?;
    let mut table_sector = template
        .get(source_table_offset..table_sector_end)
        .ok_or_else(|| "DOS 基线 PC-98 分区表被截断".to_owned())?
        .to_vec();
    let entry_in_table = source_entry_offset
        .checked_sub(source_table_offset)
        .ok_or_else(|| "DOS 基线分区项不在分区表中".to_owned())?;
    let raw_entry = parse_hex(&manifest.partition.raw_entry_hex)?;
    let entry_end = entry_in_table
        .checked_add(raw_entry.len())
        .ok_or_else(|| "DOS 基线分区项范围溢出".to_owned())?;
    let entry = table_sector
        .get_mut(entry_in_table..entry_end)
        .ok_or_else(|| "DOS 基线分区项范围越界".to_owned())?;
    if entry != raw_entry.as_slice() || raw_entry.len() != 32 {
        return Err("DOS 解包清单的原始分区项与基线 HDI 不匹配".into());
    }

    let end_lba_exclusive = start_lba
        .checked_add(partition_bytes / physical_sector)
        .ok_or_else(|| "分区末端 LBA 溢出".to_owned())?;
    let disk_sectors = blank_header.disk_size / physical_sector;
    if end_lba_exclusive > disk_sectors {
        return Err("20 MiB FAT12 分区超出空白 HDI 几何范围".into());
    }
    let (start_cylinder, start_head, start_sector) = lba_to_chs(
        start_lba,
        blank_header.sectors_per_track,
        blank_header.heads,
    )?;
    let (end_cylinder, end_head, end_sector) = lba_to_chs(
        end_lba_exclusive - 1,
        blank_header.sectors_per_track,
        blank_header.heads,
    )?;
    entry[8] = start_sector;
    entry[9] = start_head;
    entry[10..12].copy_from_slice(&start_cylinder.to_le_bytes());
    entry[12] = end_sector;
    entry[13] = end_head;
    entry[14..16].copy_from_slice(&end_cylinder.to_le_bytes());
    let table_len = table_sector.len();
    table_sector[table_len - 2..].copy_from_slice(&[0x55, 0xAA]);

    let boot_offset = usize::try_from(manifest.partition.byte_offset)
        .map_err(|_| "DOS 基线分区引导扇区偏移过大".to_owned())?;
    let boot_end = boot_offset
        .checked_add(bytes_per_sector)
        .ok_or_else(|| "DOS 基线引导扇区范围溢出".to_owned())?;
    let mut boot_sector = template
        .get(boot_offset..boot_end)
        .ok_or_else(|| "DOS 基线分区引导扇区被截断".to_owned())?
        .to_vec();
    if boot_sector.get(..3) != Some(&[0xEB, 0x45, 0x90][..])
        || boot_sector.get(3..11) != Some(b"NEC  5.0")
    {
        return Err("DOS 基线不是已确认的 PC-98 MS-DOS 5.0 引导扇区".into());
    }

    let reserved = FAT_RESERVED_SECTORS;
    let fat_copies = FAT_COPIES;
    let root_entries = FAT_ROOT_ENTRIES;
    let sectors_per_fat = FAT_SECTORS_PER_FAT;
    let sectors_per_cluster = FAT_SECTORS_PER_CLUSTER;
    let root_directory_sectors = (root_entries * DIRECTORY_ENTRY_BYTES).div_ceil(bytes_per_sector);
    let first_data_sector = reserved + fat_copies * sectors_per_fat + root_directory_sectors;
    let data_clusters = (total_sectors - first_data_sector) / sectors_per_cluster;
    let cluster_bytes = bytes_per_sector * sectors_per_cluster;
    let fat_bytes = sectors_per_fat * bytes_per_sector;
    if data_clusters > FAT12_MAX_CLUSTERS || (data_clusters + 2) * 3 / 2 >= fat_bytes {
        return Err(format!(
            "20 MiB FAT12 参数无效：data_clusters={data_clusters}, FAT bytes={fat_bytes}"
        ));
    }

    put_u16(&mut boot_sector, 11, bytes_per_sector as u16)?;
    boot_sector[13] = sectors_per_cluster as u8;
    put_u16(&mut boot_sector, 14, reserved as u16)?;
    boot_sector[16] = fat_copies as u8;
    put_u16(&mut boot_sector, 17, root_entries as u16)?;
    put_u16(&mut boot_sector, 19, total_sectors as u16)?;
    boot_sector[21] = 0xF8;
    put_u16(&mut boot_sector, 22, sectors_per_fat as u16)?;
    put_u16(&mut boot_sector, 24, blank_header.sectors_per_track as u16)?;
    put_u16(&mut boot_sector, 26, blank_header.heads as u16)?;
    put_u32(&mut boot_sector, 28, start_lba as u32)?;
    put_u32(&mut boot_sector, 32, 0)?;
    boot_sector[43..54].copy_from_slice(b"CYBER98    ");
    boot_sector[54..62].copy_from_slice(b"FAT12   ");

    let layout = FatLayout {
        partition_offset,
        partition_bytes,
        bytes_per_sector,
        reserved_sectors: reserved,
        fat_copies,
        root_entries,
        sectors_per_fat,
        first_data_sector,
        cluster_bytes,
    };
    let mut root = build_dos_root(dos_root, game_root)?;
    if root.items.len() > root_entries {
        return Err(format!(
            "DOS 根目录条目数 {} 超过 {}",
            root.items.len(),
            root_entries
        ));
    }
    let mut next_cluster = 2u32;
    assign_directory(
        &mut root,
        true,
        0,
        &mut next_cluster,
        data_clusters,
        cluster_bytes,
    )?;
    let (file_count, payload_bytes) = count_items(&root);

    let mut image = blank.to_vec();
    image[header_size..ipl_boot_area_end].copy_from_slice(ipl_boot_area);
    image[header_size..header_size + physical_sector].copy_from_slice(ipl_sector);
    image[table_offset..table_offset + physical_sector].copy_from_slice(&table_sector);
    image[partition_offset..partition_end].fill(0);
    image[partition_offset..partition_offset + bytes_per_sector].copy_from_slice(&boot_sector);
    write_fat_and_directories(&mut image, &root, &layout)?;
    Ok(BuiltImage {
        bytes: image,
        files: file_count,
        payload_bytes,
    })
}
