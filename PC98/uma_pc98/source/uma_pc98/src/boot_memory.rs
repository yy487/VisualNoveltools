use crate::Result;
use fivec_new::Archive;

pub const CONFIG: &[u8] = b"DEVICE=\\HIMEM.SYS\r\nDOS=HIGH\r\nFILES=20\r\nBUFFERS=1\r\n";
#[derive(Clone, Copy)]
pub enum ConfigPolicy {
    IfMissing,
    Ensure,
}

fn read_entry(source: &[u8], file: &fivec_new::FileEntry) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(file.size);
    for span in &file.source_ranges {
        if span.file_offset != out.len() {
            return Err("HIMEM.SYS 来源范围不连续".into());
        }
        let end = span
            .source_offset
            .checked_add(span.length)
            .ok_or("来源范围溢出")?;
        out.extend_from_slice(
            source
                .get(span.source_offset..end)
                .ok_or("HIMEM.SYS 来源范围越界")?,
        );
    }
    if out.len() != file.size {
        return Err("HIMEM.SYS 来源范围与文件大小不符".into());
    }
    Ok(out)
}

fn fat_get(fat: &[u8], cluster: usize) -> Result<usize> {
    let i = cluster.checked_mul(3).ok_or("FAT12 偏移溢出")? / 2;
    let pair = fat.get(i..i + 2).ok_or("FAT12 条目越界")?;
    Ok(if cluster.is_multiple_of(2) {
        usize::from(u16::from_le_bytes([pair[0], pair[1]]) & 0x0fff)
    } else {
        usize::from(u16::from_le_bytes([pair[0], pair[1]]) >> 4)
    })
}

fn fat_set(fat: &mut [u8], cluster: usize, value: usize) -> Result<()> {
    if value > 0xfff {
        return Err("FAT12 值越界".into());
    }
    let i = cluster.checked_mul(3).ok_or("FAT12 偏移溢出")? / 2;
    let pair = fat.get_mut(i..i + 2).ok_or("FAT12 条目越界")?;
    let old = u16::from_le_bytes([pair[0], pair[1]]);
    let new = if cluster.is_multiple_of(2) {
        (old & 0xf000) | value as u16
    } else {
        (old & 0x000f) | ((value as u16) << 4)
    };
    pair.copy_from_slice(&new.to_le_bytes());
    Ok(())
}

/// Add root HIMEM.SYS and a simple CONFIG.SYS to an Anex86 single-volume HDI.
pub fn add_boot_files(base: &[u8], himem: &[u8], config_policy: ConfigPolicy) -> Result<Vec<u8>> {
    if himem.is_empty() {
        return Err("HIMEM.SYS 为空".into());
    }
    let archive = Archive::from_bytes("base.hdi", base.to_vec())?;
    if archive.inspection().format != "anex86-fdi-hdi" {
        return Err("不是受支持的 Anex86 HDI".into());
    }
    let report = archive.inspection().disks.first().ok_or("HDI 缺少磁盘")?;
    if archive.inspection().disks.len() != 1 || report.payload_bytes > base.len() {
        return Err("HDI 不是单盘连续载荷".into());
    }
    let geometry = report.geometry.as_ref().ok_or("HDI 缺少可验证几何")?;
    let capacity = (geometry.cylinders as usize)
        .checked_mul(geometry.heads as usize)
        .and_then(|n| n.checked_mul(geometry.sectors_per_track as usize))
        .and_then(|n| n.checked_mul(geometry.bytes_per_sector as usize))
        .ok_or("HDI 几何容量溢出")?;
    if capacity != report.payload_bytes {
        return Err("HDI 几何容量与物理载荷长度不符".into());
    }
    let header = base.len() - report.payload_bytes;
    let volumes: Vec<_> = archive
        .volumes()
        .iter()
        .filter(|v| v.filesystem.is_some())
        .collect();
    if volumes.len() != 1 {
        return Err("HDI 必须恰有一个 FAT 卷".into());
    }
    let vol = volumes[0];
    let fs = vol.filesystem.as_ref().ok_or("缺少 FAT 文件系统")?;
    if fs.kind != "FAT12" || !fs.fat12.fat_copies_identical {
        return Err("仅支持 FAT12 且 FAT 副本一致的 HDI".into());
    }
    for name in ["IO.SYS", "MSDOS.SYS", "COMMAND.COM"] {
        if !fs
            .files
            .iter()
            .any(|file| file.export_path.eq_ignore_ascii_case(name) && file.size != 0)
        {
            return Err(format!(
                "基础 HDI 缺少 DOS 系统文件 {name}，不能补充启动配置"
            ));
        }
    }
    let info = &fs.fat12;
    let bps = info.bytes_per_sector;
    let spc = info.sectors_per_cluster;
    let fat_bytes = info
        .sectors_per_fat
        .checked_mul(bps)
        .ok_or("FAT 大小溢出")?;
    let fat_region = info
        .fat_copies
        .checked_mul(info.sectors_per_fat)
        .ok_or("FAT 区大小溢出")?;
    let root_sector = info
        .reserved_sectors
        .checked_add(fat_region)
        .ok_or("根目录扇区溢出")?;
    let root_start = root_sector.checked_mul(bps).ok_or("根目录偏移溢出")?;
    let root_bytes = info.root_entries.checked_mul(32).ok_or("根目录大小溢出")?;
    let data_start = info
        .first_data_sector
        .checked_mul(bps)
        .ok_or("数据偏移溢出")?;
    let volume_start = header
        .checked_add(vol.logical_byte_offset)
        .ok_or("卷偏移溢出")?;
    let volume_bytes = info.total_sectors.checked_mul(bps).ok_or("卷长度溢出")?;
    let root_end = root_start
        .checked_add(root_bytes)
        .ok_or("根目录结束位置溢出")?;
    let volume_end = volume_start.checked_add(volume_bytes).ok_or("卷边界溢出")?;
    if volume_end > base.len() || root_end > data_start || data_start > volume_bytes {
        return Err("FAT12 元数据边界不一致".into());
    }
    for file in &fs.files {
        let _ = read_entry(base, file)?;
        if file.source_ranges.iter().any(|r| {
            r.source_offset < volume_start
                || r.source_offset
                    .checked_add(r.length)
                    .is_none_or(|end| end > volume_end)
        }) {
            return Err(format!("HDI 文件来源范围越出 FAT 卷: {}", file.export_path));
        }
    }
    let himems: Vec<_> = fs
        .files
        .iter()
        .filter(|f| f.export_path.eq_ignore_ascii_case("HIMEM.SYS"))
        .collect();
    let configs: Vec<_> = fs
        .files
        .iter()
        .filter(|f| f.export_path.eq_ignore_ascii_case("CONFIG.SYS"))
        .collect();
    if himems.len() > 1 || configs.len() > 1 {
        return Err("根目录启动文件重复".into());
    }
    if !configs.is_empty() && matches!(config_policy, ConfigPolicy::IfMissing) {
        // A configured DOS installation owns its driver selection and paths.
        return Ok(base.to_vec());
    }
    let mut image = base.to_vec();
    let mut additions: Vec<(&str, &[u8])> = Vec::new();
    if let Some(existing) = himems.first() {
        if read_entry(base, existing)? != himem {
            return Err("现有根 HIMEM.SYS 与原版驱动不同，拒绝覆盖".into());
        }
    } else {
        additions.push(("HIMEM.SYS", himem));
    }
    if let Some(existing) = configs.first() {
        if matches!(config_policy, ConfigPolicy::Ensure) && read_entry(base, existing)? != CONFIG {
            return Err("HDI 已有不同 CONFIG.SYS，拒绝覆盖".into());
        }
    } else {
        additions.push(("CONFIG.SYS", CONFIG));
    }
    let root_abs = volume_start + root_start;
    let mut slots = Vec::new();
    let mut end_marker = None;
    for i in 0..info.root_entries {
        let off = root_abs + i * 32;
        let first = *image.get(off).ok_or("根目录项越界")?;
        if first == 0 {
            end_marker = Some(off);
            slots.push(off);
            for j in i + 1..info.root_entries {
                slots.push(root_abs + j * 32);
            }
            break;
        } else if first == 0xe5 {
            slots.push(off);
        }
    }
    if additions.len() > slots.len() {
        return Err("根目录没有足够空目录项".into());
    }
    let fat_abs = volume_start
        .checked_add(
            info.reserved_sectors
                .checked_mul(bps)
                .ok_or("FAT 偏移溢出")?,
        )
        .ok_or("FAT 物理偏移溢出")?;
    if fat_abs
        .checked_add(
            info.fat_copies
                .checked_mul(fat_bytes)
                .ok_or("FAT 副本总长溢出")?,
        )
        .is_none_or(|end| end > volume_start + data_start)
    {
        return Err("FAT 区越过数据区".into());
    }
    let first_fat = image
        .get(fat_abs..fat_abs + fat_bytes)
        .ok_or("FAT1 越界")?
        .to_vec();
    for copy in 1..info.fat_copies {
        let off = fat_abs + copy * fat_bytes;
        if image.get(off..off + fat_bytes) != Some(first_fat.as_slice()) {
            return Err("FAT 副本不同".into());
        }
    }
    let cluster_bytes = spc.checked_mul(bps).ok_or("簇大小溢出")?;
    if cluster_bytes == 0 {
        return Err("FAT12 簇大小为零".into());
    }
    let mut fat = first_fat;
    for (index, (name, data)) in additions.iter().enumerate() {
        let count = data.len().div_ceil(cluster_bytes);
        let mut chain = Vec::new();
        for cluster in 2..=info.data_clusters + 1 {
            if fat_get(&fat, cluster)? == 0 {
                chain.push(cluster);
                if chain.len() == count {
                    break;
                }
            }
        }
        if chain.len() != count {
            return Err("FAT12 没有足够空闲簇".into());
        }
        for (i, &c) in chain.iter().enumerate() {
            fat_set(
                &mut fat,
                c,
                if i + 1 == chain.len() {
                    0xfff
                } else {
                    chain[i + 1]
                },
            )?;
        }
        let mut padded = vec![0u8; count * cluster_bytes];
        padded[..data.len()].copy_from_slice(data);
        for (i, c) in chain.iter().enumerate() {
            let off = volume_start + data_start + (c - 2) * cluster_bytes;
            let end = off.checked_add(cluster_bytes).ok_or("簇写入溢出")?;
            if end > volume_end {
                return Err("簇数据越过卷边界".into());
            }
            image[off..end].copy_from_slice(&padded[i * cluster_bytes..(i + 1) * cluster_bytes]);
        }
        let e = slots[index];
        let mut entry = [0u8; 32];
        let short = match *name {
            "HIMEM.SYS" => *b"HIMEM   SYS",
            "CONFIG.SYS" => *b"CONFIG  SYS",
            _ => return Err("内部 DOS 文件名错误".into()),
        };
        entry[..11].copy_from_slice(&short);
        entry[11] = 0x20;
        entry[26..28].copy_from_slice(&(chain[0] as u16).to_le_bytes());
        entry[28..32].copy_from_slice(&(data.len() as u32).to_le_bytes());
        image[e..e + 32].copy_from_slice(&entry);
    }
    for copy in 0..info.fat_copies {
        let off = fat_abs + copy * fat_bytes;
        image[off..off + fat_bytes].copy_from_slice(&fat);
    }
    // Preserve the end-of-directory marker after filling an empty slot.
    if let Some(marker) = end_marker {
        if slots
            .iter()
            .take(additions.len())
            .any(|slot| *slot >= marker)
        {
            let next = slots[additions.len() - 1] + 32;
            if next + 32 <= root_abs + root_bytes {
                image[next..next + 32].fill(0);
            }
        }
    }
    Ok(image)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fat12_updates_preserve_the_neighboring_half_entry() {
        let mut fat = vec![0xa5, 0x5a, 0xc3, 0x7e, 0x91, 0x42];
        let odd_before = fat_get(&fat, 1).unwrap();
        fat_set(&mut fat, 2, 0xabc).unwrap();
        assert_eq!(fat_get(&fat, 2).unwrap(), 0xabc);
        assert_eq!(fat_get(&fat, 1).unwrap(), odd_before);
        let even_before = fat_get(&fat, 2).unwrap();
        fat_set(&mut fat, 3, 0xdef).unwrap();
        assert_eq!(fat_get(&fat, 3).unwrap(), 0xdef);
        assert_eq!(fat_get(&fat, 2).unwrap(), even_before);
    }
}
