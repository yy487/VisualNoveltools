//! Read-only FDI snapshots and per-disk FAT12 replacement/rebuild.
use crate::Result;
use pc98_fdi_unpack::{extract_fdi_files, rebuild_fdi, FdiFileData, PackReport};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileIdentity {
    pub path: String,
    pub size: usize,
}

#[derive(Clone, Debug)]
pub struct SourceDisk {
    pub index: usize,
    pub source_path: PathBuf,
    pub source_name: String,
    pub image: Vec<u8>,
    pub files: BTreeMap<String, Vec<u8>>,
    pub identities: Vec<FileIdentity>,
}

#[derive(Clone, Debug)]
pub struct RebuiltDisk {
    pub index: usize,
    pub source_name: String,
    pub bytes: Vec<u8>,
    pub report: PackReport,
}

type CollectedFiles = (BTreeMap<String, Vec<u8>>, Vec<FileIdentity>);

pub fn resolve_source(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("{label} 不可读取: {error}"))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("{label} 不接受符号链接: {}", path.display()));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(format!(
                "{label} 不接受目录联接或重解析点: {}",
                path.display()
            ));
        }
    }
    let resolved =
        fs::canonicalize(path).map_err(|error| format!("{label} 路径无法解析: {error}"))?;
    if !resolved.is_file() {
        return Err(format!("{label} 不是文件: {}", resolved.display()));
    }
    Ok(resolved)
}

/// Discovers the five game disks from their internal marker member paths.
/// The marker contents are deliberately ignored because DISK3_ID.DAT also
/// contains the text `No.2` in the original image set.
pub fn load_disks_from_directory(directory: &Path) -> Result<Vec<SourceDisk>> {
    const MARKERS: [&str; 5] = [
        "JESUS2/DISK1_ID.DAT",
        "J2_GRP/DISK2_ID.DAT",
        "J2_GRP/DISK3_ID.DAT",
        "J2_GRP/DISK4_ID.DAT",
        "J2_GRP/DISK5_ID.DAT",
    ];

    let directory_metadata = fs::symlink_metadata(directory)
        .map_err(|error| format!("FDI 目录不可读取 {}: {error}", directory.display()))?;
    reject_link_or_reparse(&directory_metadata, directory, "FDI 输入目录")?;
    if !directory_metadata.is_dir() {
        return Err(format!("FDI 输入路径不是目录: {}", directory.display()));
    }

    let entries = fs::read_dir(directory)
        .map_err(|error| format!("无法枚举 FDI 输入目录 {}: {error}", directory.display()))?;
    let mut fdi_paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "无法读取 FDI 输入目录 {} 的目录项: {error}",
                directory.display()
            )
        })?;
        let path = entry.path();
        let is_fdi = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("fdi"));
        if !is_fdi {
            continue;
        }

        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("无法读取 FDI 候选 {}: {error}", path.display()))?;
        reject_link_or_reparse(&metadata, &path, "FDI 候选")?;
        if !metadata.is_file() {
            return Err(format!("FDI 候选不是普通文件: {}", path.display()));
        }
        fdi_paths.push(path);
    }

    if fdi_paths.len() < MARKERS.len() {
        return Err(format!(
            "FDI 输入目录缺少磁盘：需要 5 个一级 .FDI 文件，当前找到 {} 个",
            fdi_paths.len()
        ));
    }
    if fdi_paths.len() > MARKERS.len() {
        return Err(format!(
            "FDI 输入目录有多余磁盘：需要 5 个一级 .FDI 文件，当前找到 {} 个",
            fdi_paths.len()
        ));
    }

    let mut ordered_paths: Vec<Option<PathBuf>> = vec![None; MARKERS.len()];
    for path in fdi_paths {
        let source_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("FDI 文件名不是有效 UTF-8: {}", path.display()))?;
        let image = fs::read(&path)
            .map_err(|error| format!("读取 FDI {} 失败: {error}", path.display()))?;
        let members = extract_fdi_files(&image, source_name)
            .map_err(|error| format!("解析盘号标记时读取 FDI {} 失败: {error}", path.display()))?;

        let mut marker_index = None;
        for member in &members {
            let member_path = normalize_member_path(&member.path)?;
            for (index, marker) in MARKERS.iter().enumerate() {
                if member_path.eq_ignore_ascii_case(marker) && marker_index.replace(index).is_some()
                {
                    return Err(format!("FDI {} 含有重复或多个盘号标记成员", path.display()));
                }
            }
        }

        let index = marker_index.ok_or_else(|| {
            format!(
                "FDI {} 缺少可识别的盘号标记成员（预期路径之一：{}）",
                path.display(),
                MARKERS.join(", ")
            )
        })?;
        if ordered_paths[index].replace(path.clone()).is_some() {
            return Err(format!("盘号标记 {} 在多个 FDI 中重复", MARKERS[index]));
        }
    }

    let paths = ordered_paths
        .into_iter()
        .enumerate()
        .map(|(index, path)| {
            path.ok_or_else(|| format!("FDI 输入目录缺少盘号标记 {}", MARKERS[index]))
        })
        .collect::<Result<Vec<_>>>()?;
    load_disks(&paths)
}

fn reject_link_or_reparse(metadata: &fs::Metadata, path: &Path, role: &str) -> Result<()> {
    if metadata.file_type().is_symlink() || metadata_is_reparse(metadata) {
        return Err(format!(
            "{role}不接受 symlink 或 reparse point: {}",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn metadata_is_reparse(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
fn metadata_is_reparse(_metadata: &fs::Metadata) -> bool {
    false
}

pub fn load_disks(paths: &[PathBuf]) -> Result<Vec<SourceDisk>> {
    if paths.len() != 5 {
        return Err(format!("必须提供 5 张 FDI，当前为 {} 张", paths.len()));
    }
    let mut seen_paths = HashSet::new();
    let mut disks = Vec::with_capacity(5);
    for (index, path) in paths.iter().enumerate() {
        let resolved = resolve_source(path, &format!("第 {} 张 FDI", index + 1))?;
        if !seen_paths.insert(resolved.clone()) {
            return Err(format!(
                "同一个文件被指定为多张 FDI: {}",
                resolved.display()
            ));
        }
        let image = fs::read(&resolved)
            .map_err(|error| format!("读取 FDI {} 失败: {error}", resolved.display()))?;
        let source_name = resolved
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("FDI 文件名不是有效 UTF-8: {}", resolved.display()))?
            .to_owned();
        let extracted = extract_fdi_files(&image, &source_name)
            .map_err(|error| format!("解析第 {} 张 FDI 失败: {error}", index + 1))?;
        let (files, identities) = collect_files(extracted)?;
        disks.push(SourceDisk {
            index,
            source_path: resolved,
            source_name,
            image,
            files,
            identities,
        });
    }
    validate_jesus2_set(&disks)?;
    Ok(disks)
}

fn collect_files(files: Vec<FdiFileData>) -> Result<CollectedFiles> {
    let mut by_key = BTreeMap::new();
    let mut identities = Vec::with_capacity(files.len());
    for file in files {
        let path = normalize_member_path(&file.path)?;
        let key = path.to_ascii_uppercase();
        if by_key
            .insert(key, (path.clone(), file.data.clone()))
            .is_some()
        {
            return Err(format!("FDI 中存在大小写不敏感重复路径: {path}"));
        }
        identities.push(FileIdentity {
            path,
            size: file.data.len(),
        });
    }
    identities.sort_by(|left, right| left.path.cmp(&right.path));
    let files = by_key.into_values().collect();
    Ok((files, identities))
}

pub fn normalize_member_path(raw: &str) -> Result<String> {
    let normalized = raw.replace('\\', "/");
    let path = Path::new(&normalized);
    if normalized.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(format!("FDI 成员路径不安全: {raw}"));
    }
    Ok(normalized)
}

fn key(path: &str) -> String {
    path.replace('\\', "/").to_ascii_uppercase()
}

fn validate_jesus2_set(disks: &[SourceDisk]) -> Result<()> {
    const EXPECTED_MEMBER_COUNTS: [usize; 5] = [47, 82, 76, 89, 74];
    if disks.len() != EXPECTED_MEMBER_COUNTS.len() {
        return Err(format!(
            "必须提供五张按盘序排列的原盘，当前为 {} 张",
            disks.len()
        ));
    }
    for (index, (disk, expected_count)) in disks.iter().zip(EXPECTED_MEMBER_COUNTS).enumerate() {
        if disk.files.len() != expected_count {
            return Err(format!(
                "第 {} 张盘含 {} 个活动成员，已确认盘序需要 {} 个",
                index + 1,
                disk.files.len(),
                expected_count
            ));
        }
        let id_path = if index == 0 {
            "JESUS2/DISK1_ID.DAT".to_owned()
        } else {
            format!("J2_GRP/DISK{}_ID.DAT", index + 1)
        };
        disk.file(&id_path).map_err(|_| {
            format!(
                "第 {} 张输入盘的盘标成员应为 {id_path}；请按游戏盘序指定 FDI",
                index + 1
            )
        })?;
    }

    let mut all_paths = HashSet::new();
    for disk in disks {
        for path in disk.files.keys() {
            all_paths.insert(key(path));
        }
    }
    for required in [
        "JESUS2.COM",
        "JESUS2/J2K.COM",
        "JESUS2/J2M.COM",
        "JESUS2/J2O.COM",
        "JESUS2/J2Z.COM",
        "J2_GRP/J2E.COM",
        "JESUS2/OPENNING.MES",
        "JESUS2/OPENNING.PRG",
    ] {
        if !all_paths.contains(&key(required)) {
            return Err(format!(
                "输入不是已支持的《ジーザス2》盘组：缺少 {required}"
            ));
        }
    }
    Ok(())
}

impl SourceDisk {
    pub fn file(&self, path: &str) -> Result<&[u8]> {
        let wanted = key(path);
        self.files
            .iter()
            .find(|(candidate, _)| key(candidate) == wanted)
            .map(|(_, bytes)| bytes.as_slice())
            .ok_or_else(|| format!("第 {} 张 FDI 缺少成员 {path}", self.index + 1))
    }

    pub fn files_with_extension(&self, extension: &str) -> impl Iterator<Item = (&str, &[u8])> {
        let wanted = extension.to_ascii_uppercase();
        self.files.iter().filter_map(move |(path, bytes)| {
            let matches = Path::new(path)
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case(&wanted));
            matches.then_some((path.as_str(), bytes.as_slice()))
        })
    }
}

pub fn rebuild_disks(
    source: &[SourceDisk],
    replacements: &[BTreeMap<String, Vec<u8>>],
) -> Result<Vec<RebuiltDisk>> {
    if source.len() != 5 || replacements.len() != source.len() {
        return Err("回封输入必须与五张来源 FDI 一一对应".into());
    }
    let mut rebuilt = Vec::with_capacity(source.len());
    for (disk, replacement) in source.iter().zip(replacements) {
        let report = rebuild_fdi(&disk.image, &disk.source_name, replacement)
            .map_err(|error| format!("重建第 {} 张 FDI 失败: {error}", disk.index + 1))?;
        let check = extract_fdi_files(&report.bytes, &disk.source_name)
            .map_err(|error| format!("第 {} 张重建 FDI 读回失败: {error}", disk.index + 1))?;
        let check_map = check
            .into_iter()
            .map(|file| (key(&file.path), file.data))
            .collect::<BTreeMap<_, _>>();
        if check_map.len() != disk.files.len() {
            return Err(format!("第 {} 张重建 FDI 的成员数量改变", disk.index + 1));
        }
        for (path, original) in &disk.files {
            let expected = replacement.get(path).unwrap_or(original);
            let actual = check_map
                .get(&key(path))
                .ok_or_else(|| format!("重建后缺少原成员: {path}"))?;
            if actual != expected {
                return Err(format!("重建后成员与预期不匹配: {path}"));
            }
        }
        for (path, expected) in replacement {
            let actual = check_map
                .get(&key(path))
                .ok_or_else(|| format!("重建后缺少 {}", path))?;
            if actual != expected {
                return Err(format!("重建后文件读回不匹配: {path}"));
            }
        }
        rebuilt.push(RebuiltDisk {
            index: disk.index,
            source_name: disk.source_name.clone(),
            bytes: report.bytes.clone(),
            report,
        });
    }
    Ok(rebuilt)
}
