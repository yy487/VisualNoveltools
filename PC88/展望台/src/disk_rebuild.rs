//! Preserving D88 reconstruction with game-adapter controlled sector edits.
//!
//! `vn-d88` intentionally limits its shared codec to fixed-size byte patches.
//! This module handles length-changing sector payloads for this adapter while
//! preserving the original headers, physical sector order, opaque track data,
//! and concatenated-disk layout.

use std::collections::BTreeMap;
use std::ops::Range;

use serde::Serialize;
use vn_d88::{Decoder, Image, StandardCodec};

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SectorPatchKey {
    pub disk_index: usize,
    pub track_slot: usize,
    pub physical_ordinal: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RebuildReport {
    pub source_size: usize,
    pub rebuilt_size: usize,
    pub source_sha256: String,
    pub rebuilt_sha256: String,
    pub rebuilt_disks: usize,
    pub replacement_entries: usize,
    pub changed_sectors: usize,
}

/// An immutable, owning snapshot of one source container and its parsed model.
///
/// The bytes, parsed ranges, and source hash are captured together so a caller
/// cannot accidentally pair a stale sector map with a separately re-read disk.
#[derive(Debug)]
pub struct PreparedD88 {
    source: Vec<u8>,
    image: Image,
    source_sha256: String,
}

impl PreparedD88 {
    /// Reads the D88 structure and binds the parsed model to these exact bytes.
    pub fn new(source: Vec<u8>) -> Result<Self> {
        let image = StandardCodec
            .decode(&source)
            .map_err(|error| format!("D88 解析失败：{error}"))?;
        let source_sha256 = crate::sha256_hex(&source);
        Ok(Self {
            source,
            image,
            source_sha256,
        })
    }

    pub fn source(&self) -> &[u8] {
        &self.source
    }

    pub fn image(&self) -> &Image {
        &self.image
    }

    pub fn source_sha256(&self) -> &str {
        &self.source_sha256
    }

    /// Rebuilds from the captured source snapshot and decoded model.
    ///
    /// Payloads may grow or shrink within a sector record. This rewrites the
    /// record length, track offsets, EOF sentinels, and each disk's declared
    /// size while preserving all copied unowned ranges byte-for-byte.
    pub fn rebuild(
        &self,
        replacements: &BTreeMap<SectorPatchKey, Vec<u8>>,
    ) -> Result<(Vec<u8>, RebuildReport)> {
        let source = self.source.as_slice();
        let image = &self.image;
        if source.len() != image.source_len {
            return Err(format!(
                "来源长度 0x{:X} 与解析模型长度 0x{:X} 不一致",
                source.len(),
                image.source_len
            ));
        }

        let changed_sectors = validate_replacement_keys(source, image, replacements)?;
        let mut preserved_ranges = Vec::new();
        let mut rebuilt = Vec::with_capacity(source.len());
        for disk in &image.disks {
            let disk_start = disk.source_range.start;
            let disk_end = disk.source_range.end;
            let header_len = disk.header.layout.byte_len();
            if disk.header_range.start != disk_start
                || disk.header_range.end != disk_start + header_len
                || disk_end > source.len()
            {
                return Err(format!(
                    "磁盘 {} 的头部或来源范围与 D88 头布局不一致",
                    disk.index
                ));
            }

            let rebuilt_disk_start = rebuilt.len();
            let mut disk_bytes = source[disk.header_range.clone()].to_vec();
            let mut tracks = disk.tracks.iter().collect::<Vec<_>>();
            tracks.sort_by_key(|track| track.source_range.start);
            let mut source_cursor = disk_start + header_len;

            for track in tracks {
                if track.source_range.start < source_cursor
                    || track.source_range.end < track.source_range.start
                    || track.source_range.end > disk_end
                {
                    return Err(format!(
                        "磁盘 {} 轨槽 {} 来源范围重叠或越界",
                        disk.index, track.slot
                    ));
                }
                let gap = source_cursor..track.source_range.start;
                let rebuilt_gap_start = rebuilt_disk_start + disk_bytes.len();
                preserve_range(
                    &mut preserved_ranges,
                    gap.clone(),
                    rebuilt_gap_start,
                    format!("磁盘 {} 轨槽 {} 之前的空隙", disk.index, track.slot),
                )?;
                disk_bytes.extend_from_slice(&source[gap]);

                let relative_offset = disk_bytes.len();
                let offset = u32::try_from(relative_offset).map_err(|_| {
                    format!(
                        "磁盘 {} 轨槽 {} 新偏移超过 D88 32 位范围",
                        disk.index, track.slot
                    )
                })?;
                write_u32(&mut disk_bytes, 0x20 + track.slot * 4, offset, disk.index)?;

                let rebuilt_track_start = rebuilt_disk_start + disk_bytes.len();
                let track_bytes = rebuild_track(source, disk.index, track, replacements)?;
                if track.sectors.is_empty() {
                    preserve_range(
                        &mut preserved_ranges,
                        track.source_range.clone(),
                        rebuilt_track_start,
                        format!("磁盘 {} 轨槽 {} 不透明轨道", disk.index, track.slot),
                    )?;
                } else if let Some(tail) = &track.opaque_tail {
                    let tail_len = tail.end.checked_sub(tail.start).ok_or_else(|| {
                        format!("磁盘 {} 轨槽 {} 不透明尾部范围无效", disk.index, track.slot)
                    })?;
                    let rebuilt_tail_start = rebuilt_track_start
                        .checked_add(track_bytes.len().checked_sub(tail_len).ok_or_else(|| {
                            format!("磁盘 {} 轨槽 {} 不透明尾部长度无效", disk.index, track.slot)
                        })?)
                        .ok_or_else(|| "重建后不透明尾部偏移溢出".to_owned())?;
                    preserve_range(
                        &mut preserved_ranges,
                        tail.clone(),
                        rebuilt_tail_start,
                        format!("磁盘 {} 轨槽 {} 不透明尾部", disk.index, track.slot),
                    )?;
                }
                disk_bytes.extend_from_slice(&track_bytes);
                source_cursor = track.source_range.end;
            }

            // Preserve any bytes not claimed by a populated track, including
            // the empty body of a disk whose track table has no records.
            let disk_tail = source_cursor..disk_end;
            preserve_range(
                &mut preserved_ranges,
                disk_tail.clone(),
                rebuilt_disk_start + disk_bytes.len(),
                format!("磁盘 {} 末尾空隙", disk.index),
            )?;
            disk_bytes.extend_from_slice(&source[disk_tail]);
            let new_disk_size = u32::try_from(disk_bytes.len())
                .map_err(|_| format!("重建后磁盘 {} 超过 D88 32 位长度", disk.index))?;

            // Some writers use the declared disk end as an empty-track
            // sentinel. Keep those entries meaningful after size changes.
            for (slot, old_offset) in disk.header.raw_track_offsets.iter().enumerate() {
                if *old_offset == disk.header.declared_disk_size {
                    write_u32(&mut disk_bytes, 0x20 + slot * 4, new_disk_size, disk.index)?;
                }
            }
            write_u32(&mut disk_bytes, 0x1C, new_disk_size, disk.index)?;
            rebuilt.extend_from_slice(&disk_bytes);
        }

        if let Some(tail) = &image.trailing_range {
            if tail.start > tail.end || tail.end != source.len() || tail.start > source.len() {
                return Err("容器尾部来源范围无效".into());
            }
            preserve_range(
                &mut preserved_ranges,
                tail.clone(),
                rebuilt.len(),
                "容器尾部".to_owned(),
            )?;
            rebuilt.extend_from_slice(&source[tail.clone()]);
        }

        let decoded = StandardCodec
            .decode(&rebuilt)
            .map_err(|error| format!("重建结果无法重新解析：{error}"))?;
        validate_rebuilt_shape(source, image, &rebuilt, &decoded, replacements)?;
        validate_preserved_ranges(source, &rebuilt, &preserved_ranges)?;

        let report = RebuildReport {
            source_size: source.len(),
            rebuilt_size: rebuilt.len(),
            source_sha256: self.source_sha256.clone(),
            rebuilt_sha256: crate::sha256_hex(&rebuilt),
            rebuilt_disks: decoded.disks.len(),
            replacement_entries: replacements.len(),
            changed_sectors,
        };
        Ok((rebuilt, report))
    }
}

struct PreservedRange {
    source: Range<usize>,
    rebuilt: Range<usize>,
    label: String,
}

fn preserve_range(
    ranges: &mut Vec<PreservedRange>,
    source: Range<usize>,
    rebuilt_start: usize,
    label: String,
) -> Result<()> {
    if source.start > source.end {
        return Err(format!("{label}来源范围无效"));
    }
    let rebuilt_end = rebuilt_start
        .checked_add(source.end - source.start)
        .ok_or_else(|| format!("{label}重建偏移溢出"))?;
    ranges.push(PreservedRange {
        source,
        rebuilt: rebuilt_start..rebuilt_end,
        label,
    });
    Ok(())
}

fn validate_preserved_ranges(
    source: &[u8],
    rebuilt: &[u8],
    ranges: &[PreservedRange],
) -> Result<()> {
    for preserved in ranges {
        if preserved.source.start > preserved.source.end
            || preserved.source.end > source.len()
            || preserved.rebuilt.start > preserved.rebuilt.end
            || preserved.rebuilt.end > rebuilt.len()
            || preserved.source.len() != preserved.rebuilt.len()
        {
            return Err(format!("{}范围在重建结果中无效", preserved.label));
        }
        if source[preserved.source.clone()] != rebuilt[preserved.rebuilt.clone()] {
            return Err(format!("重建结果未逐字节保留{}", preserved.label));
        }
    }
    Ok(())
}

fn validate_replacement_keys(
    source: &[u8],
    image: &Image,
    replacements: &BTreeMap<SectorPatchKey, Vec<u8>>,
) -> Result<usize> {
    let mut changed_sectors = 0;
    for key in replacements.keys() {
        let disk = image
            .disks
            .get(key.disk_index)
            .ok_or_else(|| format!("替换目标磁盘 {} 不存在", key.disk_index))?;
        let track = disk
            .tracks
            .iter()
            .find(|track| track.slot == key.track_slot)
            .ok_or_else(|| {
                format!(
                    "替换目标磁盘 {} 轨槽 {} 不存在",
                    key.disk_index, key.track_slot
                )
            })?;
        let sector = track
            .sectors
            .iter()
            .find(|sector| sector.address.physical_ordinal == key.physical_ordinal)
            .ok_or_else(|| {
                format!(
                    "替换目标磁盘 {} 轨槽 {} 扇区序号 {} 不存在",
                    key.disk_index, key.track_slot, key.physical_ordinal
                )
            })?;
        let payload = &replacements[key];
        if payload.as_slice() != &source[sector.data_range.clone()] {
            changed_sectors += 1;
        }
        if payload.is_empty() {
            return Err(format!(
                "不支持将扇区数据重建为空长度：磁盘 {} 轨槽 {} 扇区序号 {}",
                key.disk_index, key.track_slot, key.physical_ordinal
            ));
        }
        if payload.len() > usize::from(u16::MAX) {
            return Err(format!(
                "替换数据超过 D88 扇区长度上限：磁盘 {} 轨槽 {} 扇区序号 {}",
                key.disk_index, key.track_slot, key.physical_ordinal
            ));
        }
    }
    Ok(changed_sectors)
}

fn rebuild_track(
    source: &[u8],
    disk_index: usize,
    track: &vn_d88::Track,
    replacements: &BTreeMap<SectorPatchKey, Vec<u8>>,
) -> Result<Vec<u8>> {
    if track.sectors.is_empty() {
        return Ok(source[track.source_range.clone()].to_vec());
    }

    let mut output = Vec::with_capacity(track.source_range.len());
    let mut cursor = track.source_range.start;
    for sector in &track.sectors {
        if sector.address.track_slot != track.slot
            || sector.header_range.start != cursor
            || sector.header_range.len() != vn_d88::SECTOR_HEADER_SIZE
            || sector.data_range.start != sector.header_range.end
            || sector.data_range.end > track.source_range.end
        {
            return Err(format!(
                "磁盘 {disk_index} 轨槽 {} 的扇区记录不连续或越界",
                track.slot
            ));
        }

        let key = SectorPatchKey {
            disk_index,
            track_slot: track.slot,
            physical_ordinal: sector.address.physical_ordinal,
        };
        let replacement = replacements.get(&key);
        let data = replacement
            .map(Vec::as_slice)
            .unwrap_or(&source[sector.data_range.clone()]);
        let mut header = source[sector.header_range.clone()].to_vec();
        if replacement.is_some() && data.len() != sector.data_range.len() {
            let encoded_len = u16::try_from(data.len()).map_err(|_| {
                format!(
                    "磁盘 {disk_index} 轨槽 {} 扇区序号 {} 长度无法写入 D88",
                    track.slot, sector.address.physical_ordinal
                )
            })?;
            header[14..16].copy_from_slice(&encoded_len.to_le_bytes());
        }
        output.extend_from_slice(&header);
        output.extend_from_slice(data);
        cursor = sector.data_range.end;
    }

    if let Some(tail) = &track.opaque_tail {
        if tail.start != cursor || tail.end != track.source_range.end {
            return Err(format!(
                "磁盘 {disk_index} 轨槽 {} 的不透明尾部不连续",
                track.slot
            ));
        }
        output.extend_from_slice(&source[tail.clone()]);
    } else if cursor != track.source_range.end {
        return Err(format!(
            "磁盘 {disk_index} 轨槽 {} 存在未声明的尾部数据",
            track.slot
        ));
    }
    Ok(output)
}

fn validate_rebuilt_shape(
    source: &[u8],
    before: &Image,
    rebuilt: &[u8],
    after: &Image,
    replacements: &BTreeMap<SectorPatchKey, Vec<u8>>,
) -> Result<()> {
    if before.disks.len() != after.disks.len() {
        return Err("重建结果中的磁盘数量发生变化".into());
    }
    if before
        .trailing_range
        .as_ref()
        .map(|range| &source[range.clone()])
        != after
            .trailing_range
            .as_ref()
            .map(|range| &rebuilt[range.clone()])
    {
        return Err("重建结果未保留容器尾部数据".into());
    }

    for (old_disk, new_disk) in before.disks.iter().zip(&after.disks) {
        if old_disk.header.raw_name != new_disk.header.raw_name
            || old_disk.header.reserved != new_disk.header.reserved
            || old_disk.header.write_protect != new_disk.header.write_protect
            || old_disk.header.media_type != new_disk.header.media_type
            || old_disk.header.layout != new_disk.header.layout
            || old_disk.header.raw_track_offsets.len() != new_disk.header.raw_track_offsets.len()
            || old_disk.tracks.len() != new_disk.tracks.len()
        {
            return Err(format!(
                "重建结果改变了磁盘 {} 的保留元数据",
                old_disk.index
            ));
        }

        let expected_disk_size = u32::try_from(new_disk.source_range.len())
            .map_err(|_| "重建后的磁盘长度超过 D88 上限".to_owned())?;
        if new_disk.header.declared_disk_size != expected_disk_size {
            return Err(format!(
                "重建结果中的磁盘 {} 长度字段与实际范围不匹配",
                old_disk.index
            ));
        }
        for (slot, old_offset) in old_disk.header.raw_track_offsets.iter().enumerate() {
            let expected_offset =
                if let Some(track) = old_disk.tracks.iter().find(|t| t.slot == slot) {
                    let relative = new_disk
                        .tracks
                        .iter()
                        .find(|new_track| new_track.slot == slot)
                        .and_then(|new_track| {
                            new_track
                                .source_range
                                .start
                                .checked_sub(new_disk.source_range.start)
                        })
                        .ok_or_else(|| {
                            format!(
                                "重建结果缺少磁盘 {} 轨槽 {} 的有效偏移",
                                old_disk.index, track.slot
                            )
                        })?;
                    u32::try_from(relative).map_err(|_| {
                        format!(
                            "重建结果中磁盘 {} 轨槽 {} 的偏移超过 D88 上限",
                            old_disk.index, slot
                        )
                    })?
                } else if *old_offset == old_disk.header.declared_disk_size {
                    expected_disk_size
                } else {
                    *old_offset
                };
            if new_disk.header.raw_track_offsets.get(slot) != Some(&expected_offset) {
                return Err(format!(
                    "重建结果中磁盘 {} 轨槽 {} 的偏移字段不匹配",
                    old_disk.index, slot
                ));
            }
        }

        for (old_track, new_track) in old_disk.tracks.iter().zip(&new_disk.tracks) {
            if old_track.slot != new_track.slot
                || old_track.sectors.len() != new_track.sectors.len()
                || old_track.sectors.is_empty() != new_track.sectors.is_empty()
            {
                return Err(format!(
                    "重建结果改变了磁盘 {} 轨槽 {} 的物理布局",
                    old_disk.index, old_track.slot
                ));
            }
            if old_track.sectors.is_empty() {
                if source[old_track.source_range.clone()] != rebuilt[new_track.source_range.clone()]
                {
                    return Err(format!(
                        "重建结果未保留磁盘 {} 轨槽 {} 的不透明轨道",
                        old_disk.index, old_track.slot
                    ));
                }
                continue;
            }

            for (old_sector, new_sector) in old_track.sectors.iter().zip(&new_track.sectors) {
                let key = SectorPatchKey {
                    disk_index: old_disk.index,
                    track_slot: old_track.slot,
                    physical_ordinal: old_sector.address.physical_ordinal,
                };
                if old_sector.address.id != new_sector.address.id
                    || old_sector.address.physical_ordinal != new_sector.address.physical_ordinal
                    || old_sector.sectors_in_track != new_sector.sectors_in_track
                    || old_sector.density != new_sector.density
                    || old_sector.deleted_data != new_sector.deleted_data
                    || old_sector.fdc_status != new_sector.fdc_status
                    || old_sector.reserved != new_sector.reserved
                {
                    return Err(format!(
                        "重建结果改变了磁盘 {} 轨槽 {} 扇区序号 {} 的元数据",
                        old_disk.index, old_track.slot, old_sector.address.physical_ordinal
                    ));
                }

                let expected = replacements
                    .get(&key)
                    .map(Vec::as_slice)
                    .unwrap_or_else(|| &source[old_sector.data_range.clone()]);
                if rebuilt[new_sector.data_range.clone()] != *expected {
                    return Err(format!(
                        "重建结果中磁盘 {} 轨槽 {} 扇区序号 {} 的数据不符合预期",
                        old_disk.index, old_track.slot, old_sector.address.physical_ordinal
                    ));
                }

                let expected_length = if replacements
                    .get(&key)
                    .is_some_and(|payload| payload.len() != old_sector.data_range.len())
                {
                    u16::try_from(expected.len())
                        .map_err(|_| "重建后的扇区长度超过 D88 上限".to_owned())?
                } else {
                    old_sector.actual_length_field
                };
                if new_sector.actual_length_field != expected_length {
                    return Err(format!(
                        "重建结果中磁盘 {} 轨槽 {} 扇区序号 {} 的长度字段不匹配",
                        old_disk.index, old_track.slot, old_sector.address.physical_ordinal
                    ));
                }
            }

            match (&old_track.opaque_tail, &new_track.opaque_tail) {
                (Some(old_tail), Some(new_tail))
                    if source[old_tail.clone()] == rebuilt[new_tail.clone()] => {}
                (None, None) => {}
                _ => {
                    return Err(format!(
                        "重建结果未保留磁盘 {} 轨槽 {} 的不透明尾部",
                        old_disk.index, old_track.slot
                    ));
                }
            }
        }
    }
    Ok(())
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32, disk_index: usize) -> Result<()> {
    let target = bytes
        .get_mut(offset..offset + 4)
        .ok_or_else(|| format!("磁盘 {disk_index} 头部字段 0x{offset:X} 不在声明头部内"))?;
    target.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::synthetic_loader_disk;

    #[test]
    fn prepared_snapshot_rebuilds_variable_length_sector_payloads() {
        let original = synthetic_loader_disk();
        let prepared = PreparedD88::new(original.clone()).expect("prepare D88 snapshot");
        let original_disk_size = prepared.image().disks[0].header.declared_disk_size;
        let track = prepared.image().disks[0]
            .tracks
            .iter()
            .find(|track| track.slot == 3)
            .expect("track slot 3");
        let following_sector = track
            .sectors
            .iter()
            .find(|sector| sector.address.physical_ordinal == 1)
            .expect("following physical sector");
        let following_payload = prepared.source()[following_sector.data_range.clone()].to_vec();
        let key = SectorPatchKey {
            disk_index: 0,
            track_slot: 3,
            physical_ordinal: 0,
        };
        let replacements = BTreeMap::from([(key, vec![0xA5; 0x123])]);

        let (rebuilt, report) = prepared.rebuild(&replacements).expect("rebuild disk");
        let decoded = StandardCodec.decode(&rebuilt).expect("decode rebuilt disk");
        let rebuilt_track = decoded.disks[0]
            .tracks
            .iter()
            .find(|track| track.slot == 3)
            .expect("rebuilt track slot 3");
        let rebuilt_sector = rebuilt_track
            .sectors
            .iter()
            .find(|sector| sector.address.physical_ordinal == 0)
            .expect("rebuilt sector");

        assert_eq!(rebuilt.len(), original.len() + 0x23);
        assert_eq!(report.replacement_entries, 1);
        assert_eq!(report.changed_sectors, 1);
        assert_eq!(rebuilt_sector.data_range.len(), 0x123);
        assert_eq!(
            &rebuilt[rebuilt_sector.data_range.clone()],
            vec![0xA5; 0x123]
        );
        assert_eq!(
            &rebuilt[rebuilt_track.sectors[1].data_range.clone()],
            &following_payload
        );
        assert_eq!(prepared.source(), original);
        assert_eq!(
            decoded.disks[0].header.declared_disk_size as usize,
            original_disk_size as usize + 0x23
        );
        let original_first_track = &prepared.image().disks[0].tracks[0];
        let rebuilt_first_track = &decoded.disks[0].tracks[0];
        let original_gap =
            prepared.image().disks[0].header_range.end..original_first_track.source_range.start;
        let rebuilt_gap = decoded.disks[0].header_range.end..rebuilt_first_track.source_range.start;
        assert_eq!(
            &rebuilt[rebuilt_gap.clone()],
            &original[original_gap.clone()]
        );
        assert_eq!(
            decoded
                .trailing_range
                .as_ref()
                .map(|range| &rebuilt[range.clone()]),
            prepared
                .image()
                .trailing_range
                .as_ref()
                .map(|range| &original[range.clone()])
        );
    }

    #[test]
    fn unchanged_rebuild_is_byte_identical() {
        let source = synthetic_loader_disk();
        let prepared = PreparedD88::new(source.clone()).expect("prepare D88 snapshot");
        let (rebuilt, report) = prepared
            .rebuild(&BTreeMap::new())
            .expect("rebuild unchanged");
        assert_eq!(rebuilt, source);
        assert_eq!(report.replacement_entries, 0);
        assert_eq!(report.changed_sectors, 0);
    }
}
