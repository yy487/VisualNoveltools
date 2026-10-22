use crate::{patch_file, FileIdentity, Result};
use fivec_new::Archive;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

fn key(path: &str) -> Result<String> {
    if !path.is_ascii() {
        return Err(format!("HDI 游戏路径必须是 ASCII DOS 路径: {path}"));
    }
    Ok(path.replace('\\', "/").to_ascii_lowercase())
}

/// Patch the installed game files into a fresh in-memory HDI snapshot.
pub fn patch_installed(
    base: &[u8],
    installed: &BTreeMap<std::path::PathBuf, Vec<u8>>,
    start_batch: &[u8],
    himem: &[u8],
) -> Result<Vec<u8>> {
    let archive = Archive::from_bytes("base.hdi", base.to_vec())?;
    if archive.inspection().format != "anex86-fdi-hdi" {
        return Err("HDI 基础镜像不是已支持的 Anex86 HDI".into());
    }
    let volumes = archive
        .volumes()
        .iter()
        .filter(|volume| volume.filesystem.is_some())
        .collect::<Vec<_>>();
    if volumes.len() != 1 {
        return Err(format!(
            "HDI 必须恰有一个受支持的 FAT 卷，实际 {} 个",
            volumes.len()
        ));
    }
    let volume = volumes[0];
    let filesystem = volume
        .filesystem
        .as_ref()
        .ok_or("HDI 卷没有受支持的文件系统")?;
    if filesystem.kind != "FAT12" {
        return Err(format!("HDI 文件系统应为 FAT12，实际 {}", filesystem.kind));
    }

    let targets = installed
        .iter()
        .filter_map(|(path, data)| {
            let relative = path.strip_prefix(Path::new("installed/MV/UMA")).ok()?;
            (relative.components().count() == 1).then_some((relative, data))
        })
        .collect::<Vec<_>>();
    if targets.len() != 268 {
        return Err(format!(
            "installed/MV/UMA 应有 268 个文件，实际 {}",
            targets.len()
        ));
    }

    let expected = targets
        .iter()
        .map(|(relative, _)| {
            key(&format!(
                "MV/UMA/{}",
                relative.to_string_lossy().replace('\\', "/")
            ))
        })
        .collect::<Result<BTreeSet<_>>>()?;
    let mut matches = BTreeMap::new();
    for file in &filesystem.files {
        if !file.export_path.is_ascii() {
            continue;
        }
        let file_key = key(&file.export_path)?;
        if expected.contains(&file_key) && matches.insert(file_key, file).is_some() {
            return Err(format!(
                "HDI 有重复的大小写不敏感路径: {}",
                file.export_path
            ));
        }
    }

    let mut writes = Vec::with_capacity(targets.len() + 1);
    for (relative, data) in targets {
        let game_path = format!("MV/UMA/{}", relative.to_string_lossy().replace('\\', "/"));
        let file = matches
            .remove(&key(&game_path)?)
            .ok_or_else(|| format!("HDI 缺少安装文件: {game_path}"))?;
        if file.size != data.len() {
            return Err(format!(
                "HDI 文件 {game_path} 长度为 {}，新文件长度为 {}，不支持分配 FAT 簇",
                file.size,
                data.len()
            ));
        }
        writes.push((
            FileIdentity {
                path: file.export_path.clone(),
                size: file.size,
                ranges: file.source_ranges.clone(),
            },
            data.as_slice(),
        ));
    }
    let start_files = filesystem
        .files
        .iter()
        .filter(|file| key(&file.export_path).is_ok_and(|path| path == "start.bat"))
        .collect::<Vec<_>>();
    if start_files.len() > 1 {
        return Err("HDI 根目录有重复的大小写不敏感 START.BAT".into());
    }
    if let Some(file) = start_files.first() {
        if file.size != start_batch.len() {
            return Err(format!(
                "HDI 根目录 START.BAT 长度为 {}，新 START.BAT 长度为 {}，不支持扩展文件",
                file.size,
                start_batch.len()
            ));
        }
        writes.push((
            FileIdentity {
                path: file.export_path.clone(),
                size: file.size,
                ranges: file.source_ranges.clone(),
            },
            start_batch,
        ));
    }

    let mut image = base.to_vec();
    for (identity, data) in writes {
        patch_file(&mut image, &identity, data)?;
    }
    crate::boot_memory::add_boot_files(&image, himem, crate::boot_memory::ConfigPolicy::IfMissing)
}
