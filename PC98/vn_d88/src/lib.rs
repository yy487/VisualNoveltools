#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;
use std::ops::Range;

use vn_sector_map::{
    LinearView, PatchPlan, Region, RegionKind, RegionMap, SectorAddress, SectorId, ViewSegment,
    WritePolicy,
};

pub const LEGACY_HEADER_SIZE: usize = 0x2A0;
pub const EXTENDED_HEADER_SIZE: usize = 0x2B0;
pub const SECTOR_HEADER_SIZE: usize = 0x10;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidDiskIndex { index: usize, disk_count: usize },
    InvalidModel(String),
    Mapping(vn_sector_map::Error),
    Decode(String),
    Encode(String),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDiskIndex { index, disk_count } => {
                write!(
                    formatter,
                    "D88 disk index {index} exceeds disk count {disk_count}"
                )
            }
            Self::InvalidModel(message) => write!(formatter, "invalid D88 model: {message}"),
            Self::Mapping(error) => write!(formatter, "D88 sector mapping failed: {error}"),
            Self::Decode(message) => write!(formatter, "D88 decode failed: {message}"),
            Self::Encode(message) => write!(formatter, "D88 encode failed: {message}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Mapping(error) => Some(error),
            _ => None,
        }
    }
}

impl From<vn_sector_map::Error> for Error {
    fn from(value: vn_sector_map::Error) -> Self {
        Self::Mapping(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderLayout {
    Legacy160,
    Extended164,
}

impl HeaderLayout {
    pub const fn byte_len(self) -> usize {
        match self {
            Self::Legacy160 => LEGACY_HEADER_SIZE,
            Self::Extended164 => EXTENDED_HEADER_SIZE,
        }
    }

    pub const fn track_slots(self) -> usize {
        match self {
            Self::Legacy160 => 160,
            Self::Extended164 => 164,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub raw_name: [u8; 17],
    pub reserved: [u8; 9],
    pub write_protect: u8,
    pub media_type: u8,
    pub declared_disk_size: u32,
    pub layout: HeaderLayout,
    pub raw_track_offsets: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectorLengthBasis {
    ActualLengthField,
    NominalSizeCode,
    RecoveredTrackLayout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sector {
    pub address: SectorAddress,
    pub sectors_in_track: u16,
    pub density: u8,
    pub deleted_data: u8,
    pub fdc_status: u8,
    pub reserved: [u8; 5],
    pub actual_length_field: u16,
    pub nominal_data_len: Option<usize>,
    pub chosen_length_basis: SectorLengthBasis,
    pub header_range: Range<usize>,
    pub data_range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    pub slot: usize,
    pub source_range: Range<usize>,
    pub sectors: Vec<Sector>,
    pub opaque_tail: Option<Range<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disk {
    pub index: usize,
    pub source_range: Range<usize>,
    pub header_range: Range<usize>,
    pub header: Header,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub source_len: usize,
    pub disks: Vec<Disk>,
    pub regions: RegionMap,
    pub trailing_range: Option<Range<usize>>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub disk_index: Option<usize>,
    pub track_slot: Option<usize>,
    pub source_range: Range<usize>,
    pub message: String,
}

impl Image {
    pub fn physical_view(&self, disk_index: usize) -> Result<LinearView> {
        let disk = self.disks.get(disk_index).ok_or(Error::InvalidDiskIndex {
            index: disk_index,
            disk_count: self.disks.len(),
        })?;
        let segments = disk
            .tracks
            .iter()
            .flat_map(|track| &track.sectors)
            .map(|sector| ViewSegment {
                sector: sector.address,
                source_range: sector.data_range.clone(),
            })
            .collect::<Vec<_>>();
        Ok(LinearView::new(
            format!("d88-disk-{disk_index}-physical"),
            self.source_len,
            segments,
        )?)
    }
}

/// Decodes all concatenated disks in one D88-family container.
///
/// A decoder must preserve the raw track table, physical sector order, both
/// sector-length claims, and every unparsed byte through `RegionMap`.
pub trait Decoder {
    fn decode(&self, source: &[u8]) -> Result<Image>;
}

/// Rebuilds a D88-family container from its original bytes and validated edits.
///
/// Fixed sector edits should normally clone the source and apply `PatchPlan`.
/// Layout-changing writes must update every affected disk size and track offset.
pub trait Encoder {
    fn rebuild(&self, source: &[u8], image: &Image, patches: &PatchPlan) -> Result<Vec<u8>>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StandardCodec;

impl Decoder for StandardCodec {
    fn decode(&self, source: &[u8]) -> Result<Image> {
        let mut disks = Vec::new();
        let mut claims = Vec::new();
        let mut diagnostics = Vec::new();
        let mut cursor = 0usize;
        let mut trailing_range = None;

        while cursor < source.len() {
            match decode_disk(source, cursor, disks.len()) {
                Ok(decoded) => {
                    cursor = decoded.disk.source_range.end;
                    claims.extend(decoded.claims);
                    diagnostics.extend(decoded.diagnostics);
                    disks.push(decoded.disk);
                }
                Err(error) if !disks.is_empty() => {
                    trailing_range = Some(cursor..source.len());
                    diagnostics.push(Diagnostic {
                        disk_index: None,
                        track_slot: None,
                        source_range: cursor..source.len(),
                        message: format!(
                            "bytes after the last decoded disk remain opaque: {error}"
                        ),
                    });
                    break;
                }
                Err(error) => return Err(error),
            }
        }
        if disks.is_empty() {
            return Err(Error::Decode(
                "container has no D88 disk records".to_owned(),
            ));
        }
        let regions = RegionMap::from_claims(source.len(), claims)?;
        Ok(Image {
            source_len: source.len(),
            disks,
            regions,
            trailing_range,
            diagnostics,
        })
    }
}

impl Encoder for StandardCodec {
    fn rebuild(&self, source: &[u8], image: &Image, patches: &PatchPlan) -> Result<Vec<u8>> {
        if source.len() != image.source_len {
            return Err(Error::Encode(format!(
                "source length 0x{:X} differs from decoded model length 0x{:X}",
                source.len(),
                image.source_len
            )));
        }
        for patch in patches.patches() {
            if patch.source_range.is_empty() {
                continue;
            }
            let region = image
                .regions
                .region_containing(&patch.source_range)
                .ok_or_else(|| {
                    Error::Encode(format!(
                        "patch 0x{:X}..0x{:X} crosses region boundaries",
                        patch.source_range.start, patch.source_range.end
                    ))
                })?;
            if region.write_policy != WritePolicy::PatchFixed {
                return Err(Error::Encode(format!(
                    "region 0x{:X}..0x{:X} does not allow fixed patches",
                    region.source_range.start, region.source_range.end
                )));
            }
        }
        let output = patches.apply(source)?;
        self.decode(&output)
            .map_err(|error| Error::Encode(format!("patched image no longer decodes: {error}")))?;
        Ok(output)
    }
}

struct DecodedDisk {
    disk: Disk,
    claims: Vec<Region>,
    diagnostics: Vec<Diagnostic>,
}

fn decode_disk(source: &[u8], disk_start: usize, disk_index: usize) -> Result<DecodedDisk> {
    let minimum_end = disk_start
        .checked_add(0x20)
        .ok_or_else(|| Error::Decode("disk header offset overflow".to_owned()))?;
    if minimum_end > source.len() {
        return Err(Error::Decode(format!(
            "disk {disk_index} header at 0x{disk_start:X} is truncated"
        )));
    }
    let declared_disk_size = read_u32(source, disk_start + 0x1C)?;
    let disk_len = usize::try_from(declared_disk_size)
        .map_err(|_| Error::Decode(format!("disk {disk_index} size does not fit usize")))?;
    let disk_end = disk_start
        .checked_add(disk_len)
        .ok_or_else(|| Error::Decode(format!("disk {disk_index} end offset overflow")))?;
    if disk_len < LEGACY_HEADER_SIZE || disk_end > source.len() {
        return Err(Error::Decode(format!(
            "disk {disk_index} declares 0x{disk_len:X} bytes at 0x{disk_start:X}, outside the container"
        )));
    }

    let layout = detect_header_layout(source, disk_start, disk_len)?;
    let header_len = layout.byte_len();
    if disk_len < header_len {
        return Err(Error::Decode(format!(
            "disk {disk_index} is shorter than its 0x{header_len:X}-byte header"
        )));
    }
    let header_end = disk_start + header_len;
    let mut raw_track_offsets = Vec::with_capacity(layout.track_slots());
    let mut populated = Vec::new();
    for slot in 0..layout.track_slots() {
        let relative = read_u32(source, disk_start + 0x20 + slot * 4)?;
        raw_track_offsets.push(relative);
        let offset = usize::try_from(relative)
            .map_err(|_| Error::Decode(format!("track slot {slot} offset does not fit usize")))?;
        if offset == 0 || offset == disk_len {
            continue;
        }
        if offset < header_len || offset > disk_len {
            return Err(Error::Decode(format!(
                "disk {disk_index} track slot {slot} offset 0x{offset:X} is invalid"
            )));
        }
        populated.push((slot, disk_start + offset));
    }
    populated.sort_by_key(|(_, offset)| *offset);
    for pair in populated.windows(2) {
        if pair[0].1 == pair[1].1 {
            return Err(Error::Decode(format!(
                "disk {disk_index} track slots {} and {} alias offset 0x{:X}",
                pair[0].0, pair[1].0, pair[0].1
            )));
        }
    }

    let mut raw_name = [0u8; 17];
    raw_name.copy_from_slice(&source[disk_start..disk_start + 17]);
    let mut reserved = [0u8; 9];
    reserved.copy_from_slice(&source[disk_start + 0x11..disk_start + 0x1A]);
    let header = Header {
        raw_name,
        reserved,
        write_protect: source[disk_start + 0x1A],
        media_type: source[disk_start + 0x1B],
        declared_disk_size,
        layout,
        raw_track_offsets,
    };

    let mut claims = vec![Region {
        name: Some(format!("disk-{disk_index}-header")),
        source_range: disk_start..header_end,
        kind: RegionKind::Header,
        write_policy: WritePolicy::RebuildByCodec,
    }];
    let mut tracks = Vec::with_capacity(populated.len());
    let mut diagnostics = Vec::new();
    for (physical_index, (slot, start)) in populated.iter().copied().enumerate() {
        let end = populated
            .get(physical_index + 1)
            .map(|(_, offset)| *offset)
            .unwrap_or(disk_end);
        let (track, diagnostic) = decode_track(source, disk_index, slot, start, end);
        if let Some(diagnostic) = diagnostic {
            diagnostics.push(diagnostic);
        }
        if track.sectors.is_empty() {
            claims.push(Region {
                name: Some(format!("disk-{disk_index}-track-{slot}-opaque")),
                source_range: track.source_range.clone(),
                kind: RegionKind::Opaque,
                write_policy: WritePolicy::Preserve,
            });
        } else {
            for sector in &track.sectors {
                claims.push(Region {
                    name: Some(format!(
                        "disk-{disk_index}-track-{slot}-sector-{}-header",
                        sector.address.physical_ordinal
                    )),
                    source_range: sector.header_range.clone(),
                    kind: RegionKind::SectorMetadata,
                    write_policy: WritePolicy::Preserve,
                });
                if !sector.data_range.is_empty() {
                    claims.push(Region {
                        name: Some(format!(
                            "disk-{disk_index}-track-{slot}-sector-{}-data",
                            sector.address.physical_ordinal
                        )),
                        source_range: sector.data_range.clone(),
                        kind: RegionKind::SectorData,
                        write_policy: WritePolicy::PatchFixed,
                    });
                }
            }
            if let Some(tail) = &track.opaque_tail {
                claims.push(Region {
                    name: Some(format!("disk-{disk_index}-track-{slot}-tail")),
                    source_range: tail.clone(),
                    kind: RegionKind::Opaque,
                    write_policy: WritePolicy::Preserve,
                });
            }
        }
        tracks.push(track);
    }

    Ok(DecodedDisk {
        disk: Disk {
            index: disk_index,
            source_range: disk_start..disk_end,
            header_range: disk_start..header_end,
            header,
            tracks,
        },
        claims,
        diagnostics,
    })
}

fn detect_header_layout(source: &[u8], disk_start: usize, disk_len: usize) -> Result<HeaderLayout> {
    let first_offset = usize::try_from(read_u32(source, disk_start + 0x20)?)
        .map_err(|_| Error::Decode("first D88 track offset does not fit usize".to_owned()))?;
    match first_offset {
        LEGACY_HEADER_SIZE => return Ok(HeaderLayout::Legacy160),
        EXTENDED_HEADER_SIZE => return Ok(HeaderLayout::Extended164),
        _ => {}
    }
    if disk_len < EXTENDED_HEADER_SIZE {
        return Ok(HeaderLayout::Legacy160);
    }
    let mut minimum = None;
    for slot in 0..164usize {
        let value = usize::try_from(read_u32(source, disk_start + 0x20 + slot * 4)?)
            .map_err(|_| Error::Decode("D88 track offset does not fit usize".to_owned()))?;
        if value != 0 && value <= disk_len {
            minimum = Some(minimum.map_or(value, |old: usize| old.min(value)));
        }
    }
    match minimum {
        Some(value) if value < LEGACY_HEADER_SIZE => Err(Error::Decode(format!(
            "first D88 data offset 0x{value:X} is inside the minimum header"
        ))),
        Some(value) if value < EXTENDED_HEADER_SIZE => Ok(HeaderLayout::Legacy160),
        _ => Ok(HeaderLayout::Extended164),
    }
}

#[derive(Debug, Clone)]
struct LayoutResult {
    sectors: Vec<Sector>,
    cursor: usize,
    actual_length_choices: usize,
}

fn decode_track(
    source: &[u8],
    disk_index: usize,
    slot: usize,
    start: usize,
    end: usize,
) -> (Track, Option<Diagnostic>) {
    let opaque = |message: String| {
        (
            Track {
                slot,
                source_range: start..end,
                sectors: Vec::new(),
                opaque_tail: Some(start..end),
            },
            Some(Diagnostic {
                disk_index: Some(disk_index),
                track_slot: Some(slot),
                source_range: start..end,
                message,
            }),
        )
    };
    if start >= end || end > source.len() || end - start < SECTOR_HEADER_SIZE {
        return opaque("track is too short for a sector header".to_owned());
    }
    let expected = usize::from(u16::from_le_bytes([source[start + 4], source[start + 5]]));
    if expected == 0 || expected > (end - start) / SECTOR_HEADER_SIZE {
        return opaque(format!("track declares an invalid sector count {expected}"));
    }
    let mut memo = HashMap::new();
    let Some(layout) = choose_sector_layout(source, slot, start, end, 0, expected, &mut memo)
    else {
        return opaque(format!(
            "no consistent layout for {expected} sector headers; track preserved without interpretation"
        ));
    };
    let opaque_tail = (layout.cursor < end).then_some(layout.cursor..end);
    (
        Track {
            slot,
            source_range: start..end,
            sectors: layout.sectors,
            opaque_tail,
        },
        None,
    )
}

fn choose_sector_layout(
    source: &[u8],
    track_slot: usize,
    cursor: usize,
    track_end: usize,
    ordinal: usize,
    expected: usize,
    memo: &mut HashMap<(usize, usize), Option<LayoutResult>>,
) -> Option<LayoutResult> {
    if let Some(cached) = memo.get(&(ordinal, cursor)) {
        return cached.clone();
    }
    let result = if ordinal == expected {
        (cursor <= track_end).then_some(LayoutResult {
            sectors: Vec::new(),
            cursor,
            actual_length_choices: 0,
        })
    } else if cursor
        .checked_add(SECTOR_HEADER_SIZE)
        .is_none_or(|header_end| header_end > track_end)
    {
        None
    } else {
        let header_end = cursor + SECTOR_HEADER_SIZE;
        let count = u16::from_le_bytes([source[cursor + 4], source[cursor + 5]]);
        if usize::from(count) != expected {
            None
        } else {
            let id = SectorId {
                cylinder: u16::from(source[cursor]),
                head: source[cursor + 1],
                record: u16::from(source[cursor + 2]),
                size_code: source[cursor + 3],
            };
            let actual_length = u16::from_le_bytes([source[cursor + 14], source[cursor + 15]]);
            let nominal = id.nominal_size();
            let mut candidates = vec![(
                usize::from(actual_length),
                SectorLengthBasis::ActualLengthField,
            )];
            if nominal != Some(usize::from(actual_length)) {
                if let Some(nominal) = nominal {
                    candidates.push((nominal, SectorLengthBasis::NominalSizeCode));
                }
            }

            let mut best: Option<LayoutResult> = None;
            for (data_len, basis) in candidates {
                let Some(data_end) = header_end.checked_add(data_len) else {
                    continue;
                };
                if data_end > track_end {
                    continue;
                }
                let remaining = expected - ordinal - 1;
                if data_end
                    .checked_add(remaining.saturating_mul(SECTOR_HEADER_SIZE))
                    .is_none_or(|minimum_end| minimum_end > track_end)
                {
                    continue;
                }
                let Some(mut tail) = choose_sector_layout(
                    source,
                    track_slot,
                    data_end,
                    track_end,
                    ordinal + 1,
                    expected,
                    memo,
                ) else {
                    continue;
                };
                let sector = Sector {
                    address: SectorAddress {
                        track_slot,
                        physical_ordinal: ordinal,
                        id,
                    },
                    sectors_in_track: count,
                    density: source[cursor + 6],
                    deleted_data: source[cursor + 7],
                    fdc_status: source[cursor + 8],
                    reserved: source[cursor + 9..cursor + 14]
                        .try_into()
                        .expect("fixed D88 reserved field"),
                    actual_length_field: actual_length,
                    nominal_data_len: nominal,
                    chosen_length_basis: basis,
                    header_range: cursor..header_end,
                    data_range: header_end..data_end,
                };
                tail.sectors.insert(0, sector);
                if basis == SectorLengthBasis::ActualLengthField {
                    tail.actual_length_choices += 1;
                }
                let replace = best.as_ref().is_none_or(|current| {
                    tail.cursor > current.cursor
                        || (tail.cursor == current.cursor
                            && tail.actual_length_choices > current.actual_length_choices)
                });
                if replace {
                    best = Some(tail);
                }
            }
            best
        }
    };
    memo.insert((ordinal, cursor), result.clone());
    result
}

fn read_u32(source: &[u8], offset: usize) -> Result<u32> {
    let bytes = source
        .get(offset..offset + 4)
        .ok_or_else(|| Error::Decode(format!("u32 at 0x{offset:X} exceeds input")))?;
    Ok(u32::from_le_bytes(
        bytes.try_into().expect("four-byte D88 field"),
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectorKey {
    pub disk_index: usize,
    pub id: SectorId,
    pub physical_ordinal: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use vn_sector_map::{FixedPatch, PatchPlan, Region, RegionKind, WritePolicy};

    #[test]
    fn header_variants_remain_explicit() {
        assert_eq!(HeaderLayout::Legacy160.byte_len(), 0x2A0);
        assert_eq!(HeaderLayout::Legacy160.track_slots(), 160);
        assert_eq!(HeaderLayout::Extended164.byte_len(), 0x2B0);
        assert_eq!(HeaderLayout::Extended164.track_slots(), 164);
    }

    #[test]
    fn physical_view_preserves_sector_order_and_source_gaps() {
        let address = SectorAddress {
            track_slot: 0,
            physical_ordinal: 0,
            id: SectorId {
                cylinder: 0,
                head: 0,
                record: 1,
                size_code: 1,
            },
        };
        let source_len = 0x500;
        let regions = RegionMap::from_claims(
            source_len,
            vec![Region {
                name: Some("sector".to_owned()),
                source_range: 0x2B0..0x3C0,
                kind: RegionKind::SectorData,
                write_policy: WritePolicy::PatchFixed,
            }],
        )
        .expect("regions");
        let image = Image {
            source_len,
            disks: vec![Disk {
                index: 0,
                source_range: 0..source_len,
                header_range: 0..0x2B0,
                header: Header {
                    raw_name: [0; 17],
                    reserved: [0; 9],
                    write_protect: 0,
                    media_type: 0,
                    declared_disk_size: source_len as u32,
                    layout: HeaderLayout::Extended164,
                    raw_track_offsets: vec![0; 164],
                },
                tracks: vec![Track {
                    slot: 0,
                    source_range: 0x2B0..0x3C0,
                    sectors: vec![Sector {
                        address,
                        sectors_in_track: 1,
                        density: 0,
                        deleted_data: 0,
                        fdc_status: 0,
                        reserved: [0; 5],
                        actual_length_field: 0x100,
                        nominal_data_len: Some(0x100),
                        chosen_length_basis: SectorLengthBasis::ActualLengthField,
                        header_range: 0x2B0..0x2C0,
                        data_range: 0x2C0..0x3C0,
                    }],
                    opaque_tail: None,
                }],
            }],
            regions,
            trailing_range: None,
            diagnostics: Vec::new(),
        };
        let view = image.physical_view(0).expect("view");
        assert_eq!(view.len(), 0x100);
        assert_eq!(view.segments()[0].source_range, 0x2C0..0x3C0);
    }

    #[test]
    fn standard_codec_decodes_and_patches_one_sector_losslessly() {
        let track_start = EXTENDED_HEADER_SIZE;
        let data_start = track_start + SECTOR_HEADER_SIZE;
        let mut source = vec![0u8; data_start + 0x100];
        let declared_size = source.len() as u32;
        source[0x1C..0x20].copy_from_slice(&declared_size.to_le_bytes());
        source[0x20..0x24].copy_from_slice(&(track_start as u32).to_le_bytes());
        source[track_start] = 0;
        source[track_start + 1] = 0;
        source[track_start + 2] = 1;
        source[track_start + 3] = 1;
        source[track_start + 4..track_start + 6].copy_from_slice(&1u16.to_le_bytes());
        source[track_start + 14..track_start + 16].copy_from_slice(&0x100u16.to_le_bytes());
        source[data_start..].fill(0x5A);

        let codec = StandardCodec;
        let image = codec.decode(&source).expect("decode");
        assert_eq!(image.disks.len(), 1);
        assert_eq!(image.disks[0].tracks.len(), 1);
        assert_eq!(image.disks[0].tracks[0].sectors.len(), 1);
        assert!(image.diagnostics.is_empty());

        let unchanged = codec
            .rebuild(&source, &image, &PatchPlan::default())
            .expect("zero-change rebuild");
        assert_eq!(unchanged, source);

        let patches = PatchPlan::new(
            vec![FixedPatch {
                source_range: data_start..data_start + 1,
                replacement: vec![0xA5],
            }],
            source.len(),
        )
        .expect("patch plan");
        let changed = codec
            .rebuild(&source, &image, &patches)
            .expect("patched rebuild");
        assert_eq!(changed[data_start], 0xA5);
        assert_eq!(&changed[..data_start], &source[..data_start]);
        assert_eq!(&changed[data_start + 1..], &source[data_start + 1..]);
    }
}
