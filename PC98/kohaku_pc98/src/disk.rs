//! Conservative FAT12 writer for the validated, whole-disk FDI layout.
//! Unchanged files, free sectors, directory fields and container headers survive.
use crate::Result;
use std::collections::BTreeMap;

fn word(b: &[u8], at: usize) -> usize {
    usize::from(u16::from_le_bytes([b[at], b[at + 1]]))
}
fn fat_get(b: &[u8], cluster: usize) -> u16 {
    let value = u16::from_le_bytes([b[cluster * 3 / 2], b[cluster * 3 / 2 + 1]]);
    if cluster & 1 == 0 {
        value & 0xfff
    } else {
        value >> 4
    }
}
fn fat_set(b: &mut [u8], cluster: usize, value: u16) {
    let at = cluster * 3 / 2;
    let old = u16::from_le_bytes([b[at], b[at + 1]]);
    let new = if cluster & 1 == 0 {
        (old & 0xf000) | (value & 0xfff)
    } else {
        (old & 0x000f) | (value << 4)
    };
    b[at..at + 2].copy_from_slice(&new.to_le_bytes());
}

pub fn rebuild(original: &[u8], replacements: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>> {
    let image = fivec_new::Archive::from_bytes("source.fdi", original.to_vec())?;
    if image.inspection().format != "anex86-fdi-hdi"
        || image.volumes().len() != 1
        || image.volumes()[0].id != "disk-000-whole"
    {
        return Err("writer requires a whole-disk FDI FAT12 image".into());
    }
    let volume = &image.volumes()[0];
    let fs = volume.filesystem.as_ref().ok_or("writer requires FAT12")?;
    if !fs.directories.is_empty() || !fs.fat12.fat_copies_identical {
        return Err("writer requires matching FAT copies and root-only files".into());
    }
    for name in replacements.keys() {
        if !fs.files.iter().any(|f| &f.display_path == name) {
            return Err(format!("no source file {name}"));
        }
    }
    let header = u32::from_le_bytes(original[8..12].try_into().map_err(|_| "FDI header")?) as usize;
    let sector = fs.fat12.bytes_per_sector;
    let cluster_bytes = sector * fs.fat12.sectors_per_cluster;
    let fat_start = header + fs.fat12.reserved_sectors * sector;
    let fat_len = fs.fat12.sectors_per_fat * sector;
    let mut fat = original
        .get(fat_start..fat_start + fat_len)
        .ok_or("FAT outside FDI")?
        .to_vec();
    let mut out = original.to_vec();
    for file in &fs.files {
        let Some(bytes) = replacements.get(&file.display_path) else {
            continue;
        };
        let before = image.read_file(&volume.id, &file.id)?;
        if before == *bytes {
            continue;
        }
        if bytes.is_empty() || bytes.len() > u32::MAX as usize {
            return Err("replacement has invalid size".into());
        }
        let required = bytes.len().div_ceil(cluster_bytes);
        let mut chain: Vec<usize> = file.clusters.iter().map(|&c| c as usize).collect();
        while chain.len() < required {
            let next = (2..fs.fat12.data_clusters + 2)
                .find(|&c| fat_get(&fat, c) == 0)
                .ok_or("FDI free space exhausted")?;
            fat_set(&mut fat, next, 0xfff);
            chain.push(next);
        }
        for &c in &chain[required..] {
            fat_set(&mut fat, c, 0);
        }
        chain.truncate(required);
        for (i, &c) in chain.iter().enumerate() {
            fat_set(
                &mut fat,
                c,
                chain.get(i + 1).copied().map(|v| v as u16).unwrap_or(0xfff),
            );
            let start = header + fs.fat12.first_data_sector * sector + (c - 2) * cluster_bytes;
            let data_start = i * cluster_bytes;
            let length = (bytes.len() - data_start).min(cluster_bytes);
            out.get_mut(start..start + length)
                .ok_or("cluster outside image")?
                .copy_from_slice(&bytes[data_start..data_start + length]);
        }
        let spans = &file.directory_entry_source_ranges;
        if spans.len() != 1 || spans[0].length != 32 {
            return Err("unsupported split directory entry".into());
        }
        let at = spans[0].source_offset;
        if word(&out, at + 26) != file.clusters.first().copied().unwrap_or(0) as usize {
            return Err("directory cluster disagreement".into());
        }
        out[at + 26..at + 28].copy_from_slice(&(chain[0] as u16).to_le_bytes());
        out[at + 28..at + 32].copy_from_slice(&(bytes.len() as u32).to_le_bytes());
    }
    for copy in 0..fs.fat12.fat_copies {
        out[fat_start + copy * fat_len..fat_start + (copy + 1) * fat_len].copy_from_slice(&fat);
    }
    let rebuilt = fivec_new::Archive::from_bytes("rebuilt.fdi", out.clone())?;
    let after_fs = rebuilt.volumes()[0]
        .filesystem
        .as_ref()
        .ok_or("rebuilt FAT missing")?;
    if after_fs.files.len() != fs.files.len() {
        return Err("rebuilt file set changed".into());
    }
    for (before, after) in fs.files.iter().zip(&after_fs.files) {
        if before.display_path != after.display_path {
            return Err("rebuilt directory order changed".into());
        }
        let old = image.read_file(&volume.id, &before.id)?;
        let expected = replacements.get(&before.display_path).unwrap_or(&old);
        if rebuilt.read_file(&volume.id, &after.id)? != *expected {
            return Err(format!("FDI read-back differs for {}", before.display_path));
        }
    }
    Ok(out)
}
