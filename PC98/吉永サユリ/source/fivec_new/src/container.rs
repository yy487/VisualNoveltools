use std::collections::{BTreeMap, BTreeSet};

use encoding_rs::SHIFT_JIS;
use vn_d88::Decoder;
use vn_sector_map::{LinearView, RawGeometry, RegionKind, ViewSegment};

use crate::{
    model::{Container, Disk, Geometry},
    Result,
};

/// Anex86 FDI and HDI share a header. The container header establishes geometry;
/// it does not establish either a filesystem or the presence of partitions.
pub(crate) fn decode(source: &[u8], source_name: &str) -> Result<Container> {
    let d88 = decode_d88(source);
    let anex86 = decode_anex86(source, source_name);
    match (d88, anex86) {
        (Ok(_), Ok(_)) => Err("ambiguous container: the input satisfies both D88 and Anex86 FDI/HDI structure".into()),
        (Ok(image), Err(_)) | (Err(_), Ok(image)) => Ok(image),
        (Err(d88), Err(anex86)) => Err(format!(
            "unsupported or invalid disk container (supported: D88 and Anex86 FDI/HDI); D88: {d88}; Anex86: {anex86}"
        )),
    }
}

fn decode_d88(source: &[u8]) -> Result<Container> {
    // The shared decoder retains physical ordinals, both length claims, embedded
    // disks and opaque regions. Its physical view is deliberately not used as a
    // filesystem view: physical order need not be logical CHR order.
    let image = vn_d88::StandardCodec
        .decode(source)
        .map_err(|e| e.to_string())?;
    if image.trailing_range.is_some() && image.disks.iter().all(|disk| disk.tracks.is_empty()) {
        return Err(
            "only an empty header followed by unrecognized bytes; insufficient D88 structure"
                .into(),
        );
    }
    let mut diagnostics = Vec::new();
    for item in image
        .diagnostics
        .iter()
        .filter(|item| item.disk_index.is_none())
    {
        diagnostics.push(format!(
            "source {:#x}..{:#x}: {}",
            item.source_range.start, item.source_range.end, item.message
        ));
    }
    let mut disks = Vec::with_capacity(image.disks.len());
    for original in &image.disks {
        let mut disk_diagnostics: Vec<String> = image
            .diagnostics
            .iter()
            .filter(|item| item.disk_index == Some(original.index))
            .map(|item| {
                format!(
                    "track {:?}, source {:#x}..{:#x}: {}",
                    item.track_slot, item.source_range.start, item.source_range.end, item.message
                )
            })
            .collect();
        for region in image.regions.regions().iter().filter(|region| {
            region.kind == RegionKind::Opaque
                && region.source_range.start >= original.source_range.start
                && region.source_range.end <= original.source_range.end
                && !region.source_range.is_empty()
        }) {
            disk_diagnostics.push(format!(
                "opaque bytes at source {:#x}..{:#x} ({:#x} bytes)",
                region.source_range.start,
                region.source_range.end,
                region.source_range.len()
            ));
        }
        if original.header.reserved.iter().any(|byte| *byte != 0) {
            disk_diagnostics.push(
                "D88 header has nonzero reserved bytes; retained without interpretation".into(),
            );
        }
        if !matches!(original.header.write_protect, 0 | 0x10) {
            disk_diagnostics.push(format!(
                "unrecognized D88 write-protect value {:#x}",
                original.header.write_protect
            ));
        }
        if !matches!(original.header.media_type, 0 | 0x10 | 0x20) {
            disk_diagnostics.push(format!(
                "unrecognized D88 media type {:#x}",
                original.header.media_type
            ));
        }
        let raw_name_end = original
            .header
            .raw_name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(original.header.raw_name.len());
        let (name, _, had_errors) = SHIFT_JIS.decode(&original.header.raw_name[..raw_name_end]);
        if had_errors {
            disk_diagnostics.push(format!(
                "D88 name contains invalid CP932 bytes; raw name: {}",
                crate::hex(&original.header.raw_name)
            ));
        }
        let name = if name.is_empty() {
            format!("disk-{}", original.index)
        } else {
            name.into_owned()
        };
        let sector_count = original
            .tracks
            .iter()
            .map(|track| track.sectors.len())
            .sum();
        let payload_bytes = original
            .tracks
            .iter()
            .flat_map(|track| &track.sectors)
            .try_fold(0usize, |sum, sector| {
                sum.checked_add(sector.data_range.len())
            })
            .ok_or("D88 payload size overflow")?;
        let (geometry, view) = logical_d88_view(source.len(), original, &mut disk_diagnostics)?;
        disks.push(Disk {
            index: original.index,
            name,
            geometry,
            sector_count,
            payload_bytes,
            view,
            diagnostics: disk_diagnostics,
        });
    }
    Ok(Container {
        format: "d88".into(),
        disks,
        diagnostics,
    })
}

fn logical_d88_view(
    source_len: usize,
    disk: &vn_d88::Disk,
    diagnostics: &mut Vec<String>,
) -> Result<(Option<Geometry>, Option<LinearView>)> {
    let mut sectors = Vec::new();
    let mut ids = BTreeSet::new();
    let mut logical_tracks: BTreeMap<(u16, u8), Vec<&vn_d88::Sector>> = BTreeMap::new();
    let mut usable = true;
    for track in &disk.tracks {
        if track.sectors.is_empty() || track.opaque_tail.is_some() {
            diagnostics.push(format!(
                "track {} is incomplete or has an opaque tail; no automatic logical view",
                track.slot
            ));
            usable = false;
        }
        let mut track_ch = None;
        for sector in &track.sectors {
            let id = sector.address.id;
            let ch = (id.cylinder, id.head);
            if track_ch.is_some_and(|previous| previous != ch) {
                diagnostics.push(format!(
                    "track {} contains mixed cylinder/head IDs",
                    track.slot
                ));
                usable = false;
            }
            track_ch = Some(ch);
            if !ids.insert((id.cylinder, id.head, id.record)) {
                diagnostics.push(format!(
                    "duplicate logical sector C{}/H{}/R{} at track {} physical ordinal {}; all records retained in payload totals, no automatic logical view",
                    id.cylinder, id.head, id.record, track.slot, sector.address.physical_ordinal
                ));
                usable = false;
            }
            if sector.data_range.is_empty() {
                diagnostics.push(format!(
                    "track {} physical ordinal {} has no sector payload",
                    track.slot, sector.address.physical_ordinal
                ));
                usable = false;
            }
            if sector.nominal_data_len != Some(sector.data_range.len()) {
                diagnostics.push(format!(
                    "track {} physical ordinal {} C{}/H{}/R{}: N={} claims {:?} bytes, actual length field={}, decoded payload={} bytes ({:?})",
                    track.slot, sector.address.physical_ordinal, id.cylinder, id.head, id.record,
                    id.size_code, sector.nominal_data_len, sector.actual_length_field,
                    sector.data_range.len(), sector.chosen_length_basis
                ));
            }
            if sector.chosen_length_basis != vn_d88::SectorLengthBasis::ActualLengthField {
                diagnostics.push(format!(
                    "track {} physical ordinal {} required recovery from the recorded length; automatic filesystem reading disabled",
                    track.slot, sector.address.physical_ordinal
                ));
                usable = false;
            }
            if sector.fdc_status != 0 || sector.deleted_data != 0 {
                diagnostics.push(format!(
                    "track {} physical ordinal {} has FDC status={:#x}, deleted-data={:#x}; automatic filesystem reading disabled",
                    track.slot, sector.address.physical_ordinal, sector.fdc_status, sector.deleted_data
                ));
                usable = false;
            }
            sectors.push(sector);
        }
        if let Some(ch) = track_ch {
            if logical_tracks
                .insert(ch, track.sectors.iter().collect())
                .is_some()
            {
                diagnostics.push(format!(
                    "multiple physical tracks claim C{}/H{}",
                    ch.0, ch.1
                ));
                usable = false;
            }
            // D88's track table is C * 2 + H, including single-sided images.
            if usize::from(ch.0) * 2 + usize::from(ch.1) != track.slot || ch.1 > 1 {
                diagnostics.push(format!(
                    "track-table slot {} disagrees with conventional D88 C{}/H{} addressing",
                    track.slot, ch.0, ch.1
                ));
                usable = false;
            }
        }
    }
    if sectors.is_empty() {
        diagnostics
            .push("no decoded sectors; disk retained for inspection without a logical view".into());
        return Ok((None, None));
    }
    sectors.sort_by_key(|sector| {
        let id = sector.address.id;
        (
            id.cylinder,
            id.head,
            id.record,
            sector.address.track_slot,
            sector.address.physical_ordinal,
        )
    });
    let bytes_per_sector = sectors[0].data_range.len();
    let cylinders = u32::from(
        sectors
            .iter()
            .map(|sector| sector.address.id.cylinder)
            .max()
            .unwrap(),
    ) + 1;
    let heads = u32::from(
        sectors
            .iter()
            .map(|sector| sector.address.id.head)
            .max()
            .unwrap(),
    ) + 1;
    let sectors_per_track = u32::from(
        sectors
            .iter()
            .map(|sector| sector.address.id.record)
            .max()
            .unwrap(),
    );
    let mut regular = true;
    if sectors_per_track == 0
        || sectors
            .iter()
            .any(|sector| sector.data_range.len() != bytes_per_sector)
    {
        diagnostics.push("zero-based sector IDs or mixed payload lengths require an explicit layout; no automatic logical view".into());
        regular = false;
    }
    let expected_tracks = usize::try_from(cylinders)
        .ok()
        .and_then(|cylinders| cylinders.checked_mul(usize::try_from(heads).ok()?));
    if expected_tracks != Some(logical_tracks.len()) || !logical_tracks.contains_key(&(0, 0)) {
        diagnostics.push(format!(
            "missing cylinder/head positions: observed {} tracks, dense C0..{}/H0..{} requires {}; no sectors are compacted across gaps",
            logical_tracks.len(), cylinders - 1, heads - 1, u64::from(cylinders) * u64::from(heads)
        ));
        regular = false;
    }
    for (&(cylinder, head), track_sectors) in &logical_tracks {
        let mut records: Vec<_> = track_sectors
            .iter()
            .map(|sector| sector.address.id.record)
            .collect();
        records.sort_unstable();
        if records.len() != sectors_per_track as usize
            || records
                .iter()
                .enumerate()
                .any(|(ordinal, record)| usize::from(*record) != ordinal + 1)
        {
            diagnostics.push(format!(
                "C{cylinder}/H{head} has missing, duplicate or nonstandard sector IDs; expected R1..R{sectors_per_track}, found {records:?}"
            ));
            regular = false;
        }
    }
    let geometry = regular.then_some(Geometry {
        cylinders,
        heads,
        sectors_per_track,
        bytes_per_sector: bytes_per_sector as u32,
    });
    let view = if usable && regular {
        Some(
            LinearView::new(
                format!("d88-disk-{}-chr", disk.index),
                source_len,
                sectors
                    .into_iter()
                    .map(|sector| ViewSegment {
                        sector: sector.address,
                        source_range: sector.data_range.clone(),
                    })
                    .collect(),
            )
            .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };
    Ok((geometry, view))
}

fn decode_anex86(source: &[u8], source_name: &str) -> Result<Container> {
    if source.len() < vn_fdi::MIN_HEADER_SIZE {
        return Err("header is shorter than 32 bytes".into());
    }
    let header = vn_fdi::Header {
        reserved: read_u32(source, 0)?,
        fdd_type: read_u32(source, 4)?,
        header_size: read_u32(source, 8)?,
        data_size: read_u32(source, 12)?,
        geometry: vn_fdi::Geometry {
            bytes_per_sector: read_u32(source, 16)?,
            sectors_per_track: read_u32(source, 20)?,
            heads: read_u32(source, 24)?,
            cylinders: read_u32(source, 28)?,
        },
    };
    let g = header.geometry;
    if !(128..=32768).contains(&g.bytes_per_sector)
        || !g.bytes_per_sector.is_power_of_two()
        || g.sectors_per_track == 0
        || g.heads == 0
        || g.cylinders == 0
    {
        return Err(format!("invalid Anex86 geometry {g:?}"));
    }
    let declared_geometry_bytes = g.data_len().ok_or("geometry byte product overflows u64")?;
    if declared_geometry_bytes != u64::from(header.data_size) {
        return Err(format!(
            "geometry declares {declared_geometry_bytes} bytes but header data_size is {}",
            header.data_size
        ));
    }
    let data_start =
        usize::try_from(header.header_size).map_err(|_| "header size does not fit usize")?;
    let data_size =
        usize::try_from(header.data_size).map_err(|_| "data size does not fit usize")?;
    let data_end = data_start
        .checked_add(data_size)
        .ok_or("Anex86 data range overflow")?;
    if data_start < vn_fdi::MIN_HEADER_SIZE || data_end != source.len() {
        return Err(format!(
            "declared data range {data_start:#x}..{data_end:#x} is inconsistent with {} source bytes", source.len()
        ));
    }
    let sector_count = usize::try_from(g.sector_count().ok_or("geometry sector product overflow")?)
        .map_err(|_| "sector count does not fit usize")?;
    let mut diagnostics = Vec::new();
    if header.reserved != 0 {
        diagnostics.push(format!(
            "Anex86 reserved header field is nonzero ({:#x}); retained without interpretation",
            header.reserved
        ));
    }
    let view = if let (Ok(cylinders), Ok(heads), Ok(sectors_per_track)) = (
        u16::try_from(g.cylinders),
        u8::try_from(g.heads),
        u16::try_from(g.sectors_per_track),
    ) {
        let raw = RawGeometry {
            cylinders,
            heads,
            sectors_per_track,
            bytes_per_sector: g.bytes_per_sector as usize,
            first_record: 1,
            size_code: (g.bytes_per_sector / 128).trailing_zeros() as u8,
        };
        Some(
            raw.physical_view("anex86-disk-0-chr", source.len(), data_start..data_end)
                .map_err(|e| e.to_string())?,
        )
    } else {
        diagnostics.push("geometry exceeds the shared CHR address model; container retained without a logical view".into());
        None
    };
    Ok(Container {
        format: "anex86-fdi-hdi".into(),
        disks: vec![Disk {
            index: 0,
            name: source_name.into(),
            geometry: Some(Geometry {
                cylinders: g.cylinders,
                heads: g.heads,
                sectors_per_track: g.sectors_per_track,
                bytes_per_sector: g.bytes_per_sector,
            }),
            sector_count,
            payload_bytes: data_size,
            view,
            diagnostics,
        }],
        diagnostics: Vec::new(),
    })
}

fn read_u32(source: &[u8], offset: usize) -> Result<u32> {
    let end = offset
        .checked_add(4)
        .ok_or("header field offset overflow")?;
    let bytes = source
        .get(offset..end)
        .ok_or_else(|| format!("truncated u32 at {offset:#x}"))?;
    Ok(u32::from_le_bytes(bytes.try_into().expect("four bytes")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d88(track_start: usize, records: &[(u8, u8, u8, u8, u16, u8)]) -> Vec<u8> {
        let mut bytes = vec![0; track_start];
        bytes[0x20..0x24].copy_from_slice(&(track_start as u32).to_le_bytes());
        for &(c, h, r, n, len, fill) in records {
            let mut sector = [0u8; 16];
            sector[..4].copy_from_slice(&[c, h, r, n]);
            sector[4..6].copy_from_slice(&(records.len() as u16).to_le_bytes());
            sector[14..16].copy_from_slice(&len.to_le_bytes());
            bytes.extend_from_slice(&sector);
            bytes.resize(bytes.len() + usize::from(len), fill);
        }
        let len = bytes.len() as u32;
        bytes[0x1c..0x20].copy_from_slice(&len.to_le_bytes());
        bytes
    }

    fn anex86(c: u32, h: u32, s: u32, b: u32) -> Vec<u8> {
        let mut source = vec![0; 0x1000 + (c * h * s * b) as usize];
        let fields = [0, 0x90, 0x1000, c * h * s * b, b, s, h, c];
        for (index, value) in fields.into_iter().enumerate() {
            source[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        source
    }

    #[test]
    fn concatenated_disks_keep_absolute_source_ranges_and_cp932_names() {
        let mut first = d88(0x2b0, &[(0, 0, 1, 1, 256, 0xa1)]);
        first[..4].copy_from_slice(&[0x83, 0x65, 0x83, 0x58]);
        let first_len = first.len();
        first.extend_from_slice(&d88(0x2b0, &[(0, 0, 1, 1, 256, 0xb2)]));
        let image = decode(&first, "renamed.bin").unwrap();
        assert_eq!(image.format, "d88");
        assert_eq!(image.disks.len(), 2);
        assert_eq!(image.disks[0].name, "テス");
        let second = image.disks[1].view.as_ref().unwrap();
        assert_eq!(
            second.segments()[0].source_range,
            first_len + 0x2c0..first_len + 0x3c0
        );
        assert_eq!(first[second.segments()[0].source_range.start], 0xb2);
    }

    #[test]
    fn actual_lengths_and_interleaving_preserve_record_identity() {
        let source = d88(0x2b0, &[(0, 0, 2, 1, 128, 0x22), (0, 0, 1, 1, 128, 0x11)]);
        let image = decode(&source, "disk.d88").unwrap();
        let disk = &image.disks[0];
        assert_eq!(disk.payload_bytes, 256);
        let segments = disk.view.as_ref().unwrap().segments();
        assert_eq!(segments[0].sector.id.record, 1);
        assert_eq!(segments[0].sector.physical_ordinal, 1);
        assert_eq!(segments[0].source_range.len(), 128);
        assert_eq!(source[segments[0].source_range.start], 0x11);
        assert!(disk
            .diagnostics
            .iter()
            .any(|message| message.contains("actual length field=128")));
    }

    #[test]
    fn full_track_offset_is_read_even_when_low_byte_is_zero() {
        let source = d88(0x300, &[(0, 0, 1, 1, 256, 0x6a)]);
        let image = decode(&source, "disk.d88").unwrap();
        let disk = &image.disks[0];
        assert_eq!(disk.sector_count, 1);
        assert_eq!(
            disk.view.as_ref().unwrap().segments()[0].source_range,
            0x310..0x410
        );
        assert!(disk
            .diagnostics
            .iter()
            .any(|message| message.contains("opaque bytes")));
    }

    #[test]
    fn duplicate_and_missing_record_ids_never_compact_into_a_view() {
        for records in [
            vec![(0, 0, 1, 1, 256, 1), (0, 0, 1, 1, 256, 2)],
            vec![(0, 0, 1, 1, 256, 1), (0, 0, 3, 1, 256, 2)],
        ] {
            let image = decode(&d88(0x2b0, &records), "disk.d88").unwrap();
            assert_eq!(image.disks[0].sector_count, 2);
            assert_eq!(image.disks[0].payload_bytes, 512);
            assert!(image.disks[0].view.is_none());
            assert!(!image.disks[0].diagnostics.is_empty());
        }
    }

    #[test]
    fn missing_track_and_opaque_sector_layout_remain_inspectable() {
        let mut source = d88(0x2b0, &[(1, 0, 1, 1, 256, 1)]);
        source[0x20..0x24].fill(0);
        source[0x28..0x2c].copy_from_slice(&0x2b0u32.to_le_bytes());
        let image = decode(&source, "gap.d88").unwrap();
        assert!(image.disks[0].view.is_none());
        assert!(image.disks[0]
            .diagnostics
            .iter()
            .any(|message| message.contains("missing cylinder/head")));

        let mut source = d88(0x2b0, &[(0, 0, 1, 1, 256, 1)]);
        source[0x2b4..0x2b6].fill(0);
        let image = decode(&source, "opaque.d88").unwrap();
        assert_eq!(image.disks[0].sector_count, 0);
        assert!(image.disks[0].view.is_none());
        assert!(image.disks[0]
            .diagnostics
            .iter()
            .any(|message| message.contains("opaque")));
    }

    #[test]
    fn out_of_bounds_d88_offsets_are_rejected() {
        let mut source = d88(0x2b0, &[(0, 0, 1, 1, 256, 1)]);
        source[0x20..0x24].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode(&source, "disk.d88").is_err());
        source[0x20..0x24].copy_from_slice(&0x2b0u32.to_le_bytes());
        source[0x1c..0x20].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode(&source, "disk.d88").is_err());
    }

    #[test]
    fn anex86_detection_uses_header_geometry_not_extension_or_head_count() {
        let source = anex86(2, 4, 3, 512);
        let image = decode(&source, "renamed.d88").unwrap();
        assert_eq!(image.format, "anex86-fdi-hdi");
        assert_eq!(image.disks[0].sector_count, 24);
        let view = image.disks[0].view.as_ref().unwrap();
        assert_eq!(view.len(), 24 * 512);
        assert_eq!(view.segments()[12].sector.id.cylinder, 1);
        assert_eq!(view.segments()[12].source_range.start, 0x1000 + 12 * 512);
        // A cylinder count that also looks like a D88 length must not identify a
        // spurious empty D88 disk followed by all Anex86 payload as opaque data.
        assert_eq!(
            decode(&anex86(688, 1, 1, 128), "renamed.bin")
                .unwrap()
                .format,
            "anex86-fdi-hdi"
        );
    }

    #[test]
    fn anex86_truncation_geometry_mismatch_and_overflow_fail() {
        let mut source = anex86(2, 2, 8, 1024);
        source.pop();
        assert!(decode(&source, "disk.fdi").is_err());
        let mut source = anex86(2, 2, 8, 1024);
        source[12..16].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode(&source, "disk.fdi").is_err());
        source[20..32].fill(0xff);
        assert!(decode(&source, "disk.hdi").is_err());
        assert!(decode(&[0; 31], "disk.hdi").is_err());
    }

    #[test]
    fn ambiguous_container_structure_fails_without_using_the_filename() {
        // Two empty D88 records can occupy a structurally valid Anex86 image.
        // This deliberately unusual fixture verifies that neither the source
        // suffix nor decoder order resolves an actual structural ambiguity.
        let mut source = vec![0u8; 32 + 688 * 128];
        for (index, value) in [0u32, 0, 32, 688 * 128, 128, 1, 1, 688]
            .into_iter()
            .enumerate()
        {
            source[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        let second_len = (source.len() - 688) as u32;
        source[688 + 28..688 + 32].copy_from_slice(&second_len.to_le_bytes());
        let error = decode(&source, "ambiguous.hdi").err().unwrap();
        assert!(error.contains("ambiguous container"), "{error}");
    }
}
