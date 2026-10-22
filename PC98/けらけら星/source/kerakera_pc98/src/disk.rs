use crate::mes::Result;
use fivec_new::{Archive, FileSystem};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Disk {
    pub name: String,
    pub bytes: Vec<u8>,
    pub filesystem: FileSystem,
    pub files: BTreeMap<String, Vec<u8>>,
}

fn short_name(path: &Path) -> Result<String> {
    let name = path
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or("镜像文件名必须为 Unicode")?;
    if name.contains(['/', '\\']) || name == "." || name == ".." {
        return Err("非法镜像名称".into());
    }
    Ok(name.into())
}

pub fn load_paths(paths: &[PathBuf]) -> Result<Vec<Disk>> {
    let mut disks = Vec::new();
    let mut names = std::collections::BTreeSet::new();
    for path in paths {
        let name = short_name(path)?;
        if !names.insert(name.to_lowercase()) {
            return Err("镜像名称重复".into());
        }
        let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let archive = Archive::from_bytes(name.clone(), bytes.clone())?;
        if archive.inspection().format != "anex86-fdi-hdi" || archive.volumes().len() != 1 {
            return Err(format!("{name}: 仅支持本作已确认的整盘 FDI/FAT12"));
        }
        let volume = &archive.volumes()[0];
        if volume.id != "disk-000-whole" || volume.logical_byte_offset != 0 {
            return Err("不支持分区盘".into());
        }
        let filesystem = volume.filesystem.clone().ok_or("镜像没有可用 FAT12")?;
        if !filesystem.fat12.fat_copies_identical {
            return Err("FAT 副本不一致，拒绝写回".into());
        }
        let mut files = BTreeMap::new();
        for entry in &filesystem.files {
            files.insert(
                entry.export_path.clone(),
                archive.read_file(&volume.id, &entry.id)?,
            );
        }
        disks.push(Disk {
            name,
            bytes,
            filesystem,
            files,
        });
    }
    if disks.is_empty() {
        return Err("没有找到本作 FDI 镜像".into());
    }
    Ok(disks)
}

pub fn discover(path: &Path) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(path).map_err(|e| format!("{}: {e}", path.display()))? {
        let p = entry.map_err(|e| e.to_string())?.path();
        if p.is_file() && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("fdi")) {
            paths.push(p);
        }
    }
    paths.sort();
    Ok(paths)
}

pub fn key(disk: &Disk, path: &str) -> String {
    format!(
        "{}/{}",
        disk.name.trim_end_matches(".fdi").trim_end_matches(".FDI"),
        path
    )
}

fn fat_get(fat: &[u8], cluster: usize) -> Result<u16> {
    let p = cluster * 3 / 2;
    let pair = fat.get(p..p + 2).ok_or("FAT 索引越界")?;
    let value = u16::from_le_bytes([pair[0], pair[1]]);
    Ok(if cluster & 1 == 0 {
        value & 0xfff
    } else {
        value >> 4
    })
}

fn fat_set(fat: &mut [u8], cluster: usize, value: u16) -> Result<()> {
    let p = cluster * 3 / 2;
    let pair = fat.get_mut(p..p + 2).ok_or("FAT 索引越界")?;
    let old = u16::from_le_bytes([pair[0], pair[1]]);
    let value = if cluster & 1 == 0 {
        (old & 0xf000) | (value & 0xfff)
    } else {
        (old & 0xf) | ((value & 0xfff) << 4)
    };
    pair.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

pub fn rebuild(disk: &Disk, changes: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>> {
    let f = &disk.filesystem.fat12;
    let head = u32::from_le_bytes(
        disk.bytes
            .get(8..12)
            .ok_or("FDI 头截断")?
            .try_into()
            .map_err(|_| "FDI 头长度")?,
    ) as usize;
    let fat_start = head + f.reserved_sectors * f.bytes_per_sector;
    let fat_size = f.sectors_per_fat * f.bytes_per_sector;
    let mut fat = disk
        .bytes
        .get(fat_start..fat_start + fat_size)
        .ok_or("FAT 越界")?
        .to_vec();
    let changed = disk
        .filesystem
        .files
        .iter()
        .filter(|entry| {
            changes
                .get(&entry.export_path)
                .is_some_and(|data| disk.files[&entry.export_path] != *data)
        })
        .collect::<Vec<_>>();
    if changed.is_empty() {
        return Ok(disk.bytes.clone());
    }
    for entry in &changed {
        for cluster in &entry.clusters {
            fat_set(&mut fat, *cluster as usize, 0)?;
        }
    }
    let mut output = disk.bytes.clone();
    let cluster_size = f.sectors_per_cluster * f.bytes_per_sector;
    for entry in changed {
        let data = &changes[&entry.export_path];
        let count = data.len().div_ceil(cluster_size);
        let mut clusters = Vec::new();
        for cluster in &entry.clusters {
            if clusters.len() == count {
                break;
            }
            if fat_get(&fat, *cluster as usize)? == 0 {
                fat_set(&mut fat, *cluster as usize, 0xfff)?;
                clusters.push(*cluster as usize);
            }
        }
        for cluster in 2..f.data_clusters + 2 {
            if clusters.len() == count {
                break;
            }
            if fat_get(&fat, cluster)? == 0 {
                fat_set(&mut fat, cluster, 0xfff)?;
                clusters.push(cluster);
            }
        }
        if clusters.len() != count {
            return Err(format!(
                "{}: 空闲簇不足，无法回注 {}",
                disk.name, entry.display_path
            ));
        }
        for (i, cluster) in clusters.iter().enumerate() {
            fat_set(
                &mut fat,
                *cluster,
                clusters.get(i + 1).copied().map_or(0xfff, |v| v as u16),
            )?;
            let start = head
                + (f.first_data_sector + (cluster - 2) * f.sectors_per_cluster)
                    * f.bytes_per_sector;
            let slice = &data[i * cluster_size..data.len().min((i + 1) * cluster_size)];
            output
                .get_mut(start..start + slice.len())
                .ok_or("文件簇写入越界")?
                .copy_from_slice(slice);
        }
        let ranges = &entry.directory_entry_source_ranges;
        if ranges.len() != 1 || ranges[0].length != 32 {
            return Err("FDI 目录项不是连续的 32 字节".into());
        }
        let start = ranges[0].source_offset;
        let directory = output.get_mut(start..start + 32).ok_or("目录项越界")?;
        directory[26..28]
            .copy_from_slice(&(clusters.first().copied().unwrap_or(0) as u16).to_le_bytes());
        let size = u32::try_from(data.len()).map_err(|_| "文件超出 FAT12 长度字段")?;
        directory[28..32].copy_from_slice(&size.to_le_bytes());
    }
    for copy in 0..f.fat_copies {
        output
            .get_mut(fat_start + copy * fat_size..fat_start + (copy + 1) * fat_size)
            .ok_or("FAT 副本写入越界")?
            .copy_from_slice(&fat);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fat12_neighbors_keep_their_shared_nibble() {
        let mut fat = vec![0xff; 12];
        fat_set(&mut fat, 2, 0x123).unwrap();
        fat_set(&mut fat, 3, 0x456).unwrap();
        assert_eq!(fat_get(&fat, 2).unwrap(), 0x123);
        assert_eq!(fat_get(&fat, 3).unwrap(), 0x456);
        fat_set(&mut fat, 2, 0).unwrap();
        assert_eq!(fat_get(&fat, 3).unwrap(), 0x456);
    }
}
