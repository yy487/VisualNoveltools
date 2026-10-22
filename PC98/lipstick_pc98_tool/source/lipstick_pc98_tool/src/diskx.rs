//! Lipstick 1's XDISK200 container. FAT parsing/reallocation stays in shared crates.
use crate::{Result, dword, word};
use std::collections::BTreeSet;

struct Layout {
    bps: usize,
    spc: usize,
    reserved: usize,
    spf: usize,
    total: usize,
    data: usize,
    extra: usize,
    table: usize,
    bitmap: usize,
    physical_data: usize,
    physical_count: usize,
}

fn layout(b: &[u8]) -> Result<Layout> {
    if b.get(3..11) != Some(b"XDISK200") {
        return Err("不是已确认的 XDISK200".into());
    }
    let bps = usize::from(word(b, 11)?);
    let spc = usize::from(*b.get(13).ok_or("DiskX BPB 截断")?);
    let reserved = usize::from(word(b, 14)?);
    let spf = usize::from(word(b, 22)?);
    let entries = usize::from(word(b, 17)?);
    if (bps, spc, reserved, b[16], entries, spf) != (1024, 8, 1, 1, 1280, 4) {
        return Err("不支持这个 DiskX BPB".into());
    }
    let total = usize::from(word(b, 19)?);
    let bitmap_sectors = usize::from(word(b, 0x1b2)?);
    let table_sectors = usize::from(word(b, 0x1b4)?);
    let extra = bitmap_sectors + table_sectors;
    let data = reserved + spf + (entries * 32).div_ceil(bps);
    let physical_data = data + extra;
    let physical_count = usize::from(word(b, 0x1ae)?);
    if bitmap_sectors != 1
        || table_sectors != 9
        || total < data
        || b.len() != (physical_data + physical_count) * bps
        || (total - data) / spc * 4 > table_sectors * bps
        || physical_count > bitmap_sectors * bps * 8
    {
        return Err("DiskX 元数据／容量不符合已确认布局".into());
    }
    Ok(Layout {
        bps,
        spc,
        reserved,
        spf,
        total,
        data,
        extra,
        table: (reserved + bitmap_sectors) * bps,
        bitmap: reserved * bps,
        physical_data,
        physical_count,
    })
}

fn fat_value(fat: &[u8], cluster: usize) -> Result<u16> {
    let value = word(fat, cluster * 3 / 2)?;
    Ok(if cluster & 1 == 0 {
        value & 0xfff
    } else {
        value >> 4
    })
}

struct Bits<'a> {
    data: &'a [u8],
    position: usize,
}
impl Bits<'_> {
    fn get(&mut self, count: usize) -> Result<usize> {
        let mut value = 0;
        for _ in 0..count {
            let byte = *self.data.get(self.position / 8).ok_or("DiskX 位流截断")?;
            value = value * 2 + usize::from((byte >> (7 - self.position % 8)) & 1);
            self.position += 1;
        }
        Ok(value)
    }
}

fn decompress(data: &[u8], capacity: usize) -> Result<Vec<u8>> {
    let mut bits = Bits { data, position: 0 };
    let mut output = Vec::new();
    loop {
        if bits.get(1)? != 0 {
            if output.len() == capacity {
                return Err("DiskX 字面量越界".into());
            }
            output.push(bits.get(8)? as u8);
            continue;
        }
        let distance = if bits.get(1)? != 0 {
            bits.get(11)?
        } else if bits.get(1)? != 0 {
            bits.get(8)?
        } else {
            bits.get(6)?
        };
        if distance == 0 {
            return Ok(output);
        }
        let (mut length, mut width) = (2, 1);
        while bits.get(1)? != 0 {
            length += 1 << width;
            width += 1;
            if width > 5 {
                return Err("DiskX 匹配长度非法".into());
            }
        }
        length += bits.get(width)?;
        if distance > output.len() || output.len() + length > capacity {
            return Err("DiskX 回溯距离／长度越界".into());
        }
        for _ in 0..length {
            output.push(output[output.len() - distance]);
        }
    }
}

#[derive(Default)]
struct Writer {
    bytes: Vec<u8>,
    bits: usize,
}
impl Writer {
    fn put(&mut self, value: usize, count: usize) {
        for shift in (0..count).rev() {
            if self.bits.is_multiple_of(8) {
                self.bytes.push(0);
            }
            let last = self.bytes.len() - 1;
            self.bytes[last] |= (((value >> shift) & 1) as u8) << (7 - self.bits % 8);
            self.bits += 1;
        }
    }
}

fn compress(data: &[u8]) -> Vec<u8> {
    let mut w = Writer::default();
    let mut head = vec![usize::MAX; 65536];
    let mut previous = vec![usize::MAX; data.len()];
    let mut p = 0;
    while p < data.len() {
        let mut best = (0, 0);
        if p + 1 < data.len() {
            let key = usize::from(u16::from_le_bytes([data[p], data[p + 1]]));
            let mut candidate = head[key];
            while candidate != usize::MAX && p - candidate < 2048 {
                let mut length = 2;
                while length < 63
                    && p + length < data.len()
                    && data[candidate + length] == data[p + length]
                {
                    length += 1;
                }
                if length > best.0 {
                    best = (length, p - candidate);
                }
                if length == 63 {
                    break;
                }
                candidate = previous[candidate];
            }
        }
        let used = if best.0 >= 2 {
            w.put(0, 1);
            let (length, distance) = best;
            if distance >= 256 {
                w.put(1, 1);
                w.put(distance, 11);
            } else if distance >= 64 {
                w.put(1, 2);
                w.put(distance, 8);
            } else {
                w.put(0, 2);
                w.put(distance, 6);
            }
            let mut base = 2;
            let mut width = 1;
            while length >= base + (1 << width) {
                w.put(1, 1);
                base += 1 << width;
                width += 1;
            }
            w.put(0, 1);
            w.put(length - base, width);
            length
        } else {
            w.put(1, 1);
            w.put(usize::from(data[p]), 8);
            1
        };
        for i in p..p + used {
            if i + 1 < data.len() {
                let key = usize::from(u16::from_le_bytes([data[i], data[i + 1]]));
                previous[i] = head[key];
                head[key] = i;
            }
        }
        p += used;
    }
    w.put(0, 9);
    w.bytes
}

fn mapped(b: &[u8], l: &Layout, index: usize) -> Result<(usize, usize, u8)> {
    let offset = l.table + index * 4;
    let sector = usize::from(word(b, offset)?) | (usize::from(b[offset + 2] & 15) << 16);
    let count = usize::from(b[offset + 3] & 63) + 1;
    if sector < l.physical_data || sector + count > l.physical_data + l.physical_count {
        return Err(format!("DiskX 簇 {} 物理范围越界", index + 2));
    }
    if b[offset + 2] & 0x40 != 0 {
        return Err("不支持 DiskX 字节型压缩器".into());
    }
    Ok((sector, count, b[offset + 3]))
}

pub fn logical_fdi(source: &[u8]) -> Result<Vec<u8>> {
    let l = layout(source)?;
    let mut raw = vec![0; l.total * l.bps];
    raw[..l.bps].copy_from_slice(&source[..l.bps]);
    // Scratch FDI geometry only: DiskX itself has no physical CHS here. The
    // shared FAT writer requires BPB/FDI agreement. Never copy this scratch BPB
    // back over the original DiskX boot sector.
    raw[24..26].copy_from_slice(&(l.total as u16).to_le_bytes());
    raw[26..28].copy_from_slice(&1u16.to_le_bytes());
    raw[l.reserved * l.bps..l.data * l.bps]
        .copy_from_slice(&source[(l.reserved + l.extra) * l.bps..(l.data + l.extra) * l.bps]);
    let fat = raw[l.reserved * l.bps..(l.reserved + l.spf) * l.bps].to_vec();
    for index in 0..(l.total - l.data) / l.spc {
        let value = fat_value(&fat, index + 2)?;
        if value == 0 || value == 0xff7 {
            continue;
        }
        let (sector, count, flags) = mapped(source, &l, index)?;
        let bytes = &source[sector * l.bps..(sector + count) * l.bps];
        let block = if flags & 64 != 0 {
            decompress(bytes, l.bps * l.spc)?
        } else {
            bytes.to_vec()
        };
        if block.len() > l.bps * l.spc {
            return Err("DiskX 解码簇过长".into());
        }
        let offset = (l.data + index * l.spc) * l.bps;
        raw[offset..offset + block.len()].copy_from_slice(&block);
    }
    let mut out = vec![0; 4096];
    for (i, v) in [
        0,
        0,
        4096,
        raw.len() as u32,
        l.bps as u32,
        l.total as u32,
        1,
        1,
    ]
    .iter()
    .enumerate()
    {
        out[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
    }
    out.extend(raw);
    Ok(out)
}

pub fn rebuild(source: &[u8], original_view: &[u8], new_view: &[u8]) -> Result<Vec<u8>> {
    if original_view == new_view {
        return Ok(source.to_vec());
    }
    let l = layout(source)?;
    let header = dword(new_view, 8)? as usize;
    let old_header = dword(original_view, 8)? as usize;
    let raw = new_view.get(header..).ok_or("DiskX 逻辑头越界")?;
    let old = original_view
        .get(old_header..)
        .ok_or("DiskX 原始逻辑头越界")?;
    if raw.len() != l.total * l.bps || old.len() != raw.len() || raw[..l.bps] != old[..l.bps] {
        return Err("DiskX 逻辑布局发生变化".into());
    }
    let old_fat = &old[l.reserved * l.bps..(l.reserved + l.spf) * l.bps];
    let fat = &raw[l.reserved * l.bps..(l.reserved + l.spf) * l.bps];
    let mut out = source.to_vec();
    let mut occupied = (0..l.physical_count)
        .map(|i| source[l.bitmap + i / 8] & (0x80 >> (i % 8)) != 0)
        .collect::<Vec<_>>();
    let mut owned = BTreeSet::new();
    let cluster_count = (l.total - l.data) / l.spc;
    for index in 0..cluster_count {
        let value = fat_value(old_fat, index + 2)?;
        if value == 0 || value == 0xff7 {
            continue;
        }
        let (sector, count, _) = mapped(source, &l, index)?;
        for (p, used) in occupied
            .iter()
            .enumerate()
            .take(sector + count - l.physical_data)
            .skip(sector - l.physical_data)
        {
            if !used || !owned.insert(p) {
                return Err("DiskX 位图与活动簇映射冲突".into());
            }
        }
    }
    for p in owned {
        occupied[p] = false;
    }
    for index in 0..cluster_count {
        let value = fat_value(fat, index + 2)?;
        let map = l.table + index * 4;
        if value == 0 || value == 0xff7 {
            if fat_value(old_fat, index + 2)? != 0 {
                out[map..map + 4].fill(0);
            }
            continue;
        }
        let logical = (l.data + index * l.spc) * l.bps;
        let plain = &raw[logical..logical + l.bps * l.spc];
        let (mut block, flags, high) = if plain == &old[logical..logical + plain.len()]
            && fat_value(old_fat, index + 2)? != 0
        {
            let (sector, count, flags) = mapped(source, &l, index)?;
            (
                source[sector * l.bps..(sector + count) * l.bps].to_vec(),
                flags,
                source[map + 2] & 0xf0,
            )
        } else {
            let packed = compress(plain);
            let packed_sectors = packed.len().div_ceil(l.bps);
            let raw_sectors = plain
                .iter()
                .rposition(|b| *b != 0)
                .map_or(1, |p| (p + 1).div_ceil(l.bps));
            if packed_sectors < raw_sectors {
                (packed, 0x40 | (packed_sectors - 1) as u8, 0)
            } else {
                (
                    plain[..raw_sectors * l.bps].to_vec(),
                    (raw_sectors - 1) as u8,
                    if raw_sectors < l.spc { 0x20 } else { 0 },
                )
            }
        };
        let count = usize::from(flags & 63) + 1;
        block.resize(count * l.bps, 0);
        let start = occupied
            .windows(count)
            .position(|run| run.iter().all(|v| !v))
            .ok_or_else(|| {
                format!(
                    "DiskX 物理空间不足，簇 {} 需要 {count} 个连续扇区",
                    index + 2
                )
            })?;
        occupied[start..start + count].fill(true);
        let sector = start + l.physical_data;
        out[sector * l.bps..(sector + count) * l.bps].copy_from_slice(&block);
        out[map..map + 2].copy_from_slice(&(sector as u16).to_le_bytes());
        out[map + 2] = high | ((sector >> 16) as u8 & 15);
        out[map + 3] = flags;
    }
    out[(l.reserved + l.extra) * l.bps..(l.data + l.extra) * l.bps]
        .copy_from_slice(&raw[l.reserved * l.bps..l.data * l.bps]);
    for (i, used) in occupied.into_iter().enumerate() {
        let mask = 0x80 >> (i % 8);
        if used {
            out[l.bitmap + i / 8] |= mask;
        } else {
            out[l.bitmap + i / 8] &= !mask;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diskx_overlap_distance_and_length_boundaries() {
        let mut bytes = (0..=255).cycle().take(8192).collect::<Vec<u8>>();
        bytes[2030..4100].fill(0);
        let encoded = compress(&bytes);
        assert_eq!(decompress(&encoded, 8192).unwrap(), bytes);
        assert!(decompress(&encoded[..3], 8192).is_err());
    }

    #[test]
    fn scratch_geometry_is_accepted_by_the_shared_writer_without_changing_diskx_boot() {
        let mut source = vec![0; 1037 * 1024];
        source[3..11].copy_from_slice(b"XDISK200");
        for (offset, value) in [
            (11, 1024u16),
            (14, 1),
            (17, 1280),
            (19, 15773),
            (22, 4),
            (0x1ae, 982),
            (0x1b2, 1),
            (0x1b4, 9),
        ] {
            source[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        }
        source[13] = 8;
        source[16] = 1;
        source[21] = 0xf8;
        source[11 * 1024..11 * 1024 + 3].copy_from_slice(&[0xf8, 0xff, 0xff]);
        let view = logical_fdi(&source).unwrap();
        assert_eq!(word(&source, 24).unwrap(), 0);
        assert_eq!(word(&view, 4096 + 24).unwrap(), 15773);
        pc98_fdi_unpack::rebuild_fdi(&view, "scratch.fdi", &std::collections::BTreeMap::new())
            .unwrap();
    }
}
