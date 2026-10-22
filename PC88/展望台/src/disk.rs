use serde::Serialize;
use vn_d88::{Decoder, StandardCodec};

use crate::Result;

#[derive(Debug, Serialize)]
pub struct ImageReport {
    pub schema: &'static str,
    pub source_size: usize,
    pub source_sha256: String,
    pub disk_count: usize,
    pub disks: Vec<DiskReport>,
}

#[derive(Debug, Serialize)]
pub struct DiskReport {
    pub index: usize,
    pub name: String,
    pub source_offset: usize,
    pub source_size: usize,
    pub track_slots: usize,
    pub sectors: usize,
    pub write_protect: u8,
    pub track_profiles: Vec<TrackProfile>,
    pub sector_sizes: Vec<usize>,
    pub identifier_ranges: Vec<String>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TrackProfile {
    pub track_slots: Vec<usize>,
    pub sectors_in_track: usize,
    pub physical_record_order: Vec<u16>,
    pub record_id_ranges: Vec<String>,
}

pub fn inspect(source: &[u8]) -> Result<ImageReport> {
    let image = StandardCodec
        .decode(source)
        .map_err(|error| format!("D88 解析失败：{error}"))?;
    let mut disks = Vec::with_capacity(image.disks.len());
    for disk in &image.disks {
        let mut ids = Vec::new();
        let mut sizes = Vec::new();
        let mut track_slots = 0usize;
        let mut sectors = 0usize;
        let mut diagnostics = Vec::new();
        let mut profiles = std::collections::BTreeMap::<Vec<u16>, Vec<usize>>::new();
        for track in &disk.tracks {
            if track.sectors.is_empty() && track.opaque_tail.is_none() {
                continue;
            }
            track_slots += 1;
            sectors += track.sectors.len();
            for sector in &track.sectors {
                ids.push(sector.address.id.record);
                sizes.push(sector.data_range.len());
            }
            let record_order = track
                .sectors
                .iter()
                .map(|sector| sector.address.id.record)
                .collect::<Vec<_>>();
            profiles.entry(record_order).or_default().push(track.slot);
            if let Some(tail) = &track.opaque_tail {
                diagnostics.push(format!(
                    "track-slot-{}: opaque bytes 0x{:X}..0x{:X}",
                    track.slot, tail.start, tail.end
                ));
            }
        }
        ids.sort_unstable();
        ids.dedup();
        sizes.sort_unstable();
        sizes.dedup();
        for diagnostic in image
            .diagnostics
            .iter()
            .filter(|item| item.disk_index == Some(disk.index))
        {
            diagnostics.push(format!(
                "0x{:X}..0x{:X}: {}",
                diagnostic.source_range.start, diagnostic.source_range.end, diagnostic.message
            ));
        }
        disks.push(DiskReport {
            index: disk.index,
            name: String::from_utf8_lossy(
                &disk.header.raw_name[..disk
                    .header
                    .raw_name
                    .iter()
                    .position(|b| *b == 0)
                    .unwrap_or(17)],
            )
            .trim()
            .to_owned(),
            source_offset: disk.source_range.start,
            source_size: disk.source_range.len(),
            track_slots,
            sectors,
            write_protect: disk.header.write_protect,
            track_profiles: profiles
                .into_iter()
                .map(|(physical_record_order, track_slots)| TrackProfile {
                    track_slots,
                    sectors_in_track: physical_record_order.len(),
                    record_id_ranges: format_ranges(&{
                        let mut sorted = physical_record_order.clone();
                        sorted.sort_unstable();
                        sorted.dedup();
                        sorted
                    }),
                    physical_record_order,
                })
                .collect(),
            sector_sizes: sizes,
            identifier_ranges: format_ranges(&ids),
            diagnostics,
        });
    }
    Ok(ImageReport {
        schema: "tenboudai-pc88-d88-inspection-v1",
        source_size: source.len(),
        source_sha256: crate::sha256_hex(source),
        disk_count: disks.len(),
        disks,
    })
}

fn format_ranges(values: &[u16]) -> Vec<String> {
    if values.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut start = values[0];
    let mut previous = start;
    for &value in &values[1..] {
        if value == previous + 1 {
            previous = value;
            continue;
        }
        result.push(if start == previous {
            start.to_string()
        } else {
            format!("{start}-{previous}")
        });
        start = value;
        previous = value;
    }
    result.push(if start == previous {
        start.to_string()
    } else {
        format!("{start}-{previous}")
    });
    result
}
