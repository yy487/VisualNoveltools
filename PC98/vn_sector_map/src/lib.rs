#![forbid(unsafe_code)]

use std::error::Error as StdError;
use std::fmt;
use std::ops::Range;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidSourceRange {
        label: String,
        range: Range<usize>,
        source_len: usize,
    },
    InvalidLogicalRange {
        view: String,
        range: Range<usize>,
        view_len: usize,
    },
    EmptyViewName,
    EmptyView,
    EmptyExtent,
    InvalidRawGeometry(String),
    RawGeometryLengthMismatch {
        expected: usize,
        actual: usize,
    },
    WrongView {
        expected: String,
        actual: String,
    },
    OverlappingRegions {
        left: Range<usize>,
        right: Range<usize>,
    },
    InvalidPatchLength {
        range: Range<usize>,
        replacement_len: usize,
    },
    OverlappingPatches {
        left: Range<usize>,
        right: Range<usize>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSourceRange {
                label,
                range,
                source_len,
            } => write!(
                formatter,
                "{label} source range 0x{:X}..0x{:X} exceeds 0x{source_len:X} bytes",
                range.start, range.end
            ),
            Self::InvalidLogicalRange {
                view,
                range,
                view_len,
            } => write!(
                formatter,
                "logical range 0x{:X}..0x{:X} exceeds view {view:?} length 0x{view_len:X}",
                range.start, range.end
            ),
            Self::EmptyViewName => write!(formatter, "linear view name must not be empty"),
            Self::EmptyView => write!(formatter, "linear view must contain at least one segment"),
            Self::EmptyExtent => {
                write!(formatter, "extent must contain at least one logical range")
            }
            Self::InvalidRawGeometry(message) => {
                write!(formatter, "invalid raw-sector geometry: {message}")
            }
            Self::RawGeometryLengthMismatch { expected, actual } => write!(
                formatter,
                "raw-sector geometry expects 0x{expected:X} bytes, data range contains 0x{actual:X}"
            ),
            Self::WrongView { expected, actual } => write!(
                formatter,
                "extent belongs to view {expected:?}, not {actual:?}"
            ),
            Self::OverlappingRegions { left, right } => write!(
                formatter,
                "source regions overlap at 0x{:X}..0x{:X} and 0x{:X}..0x{:X}",
                left.start, left.end, right.start, right.end
            ),
            Self::InvalidPatchLength {
                range,
                replacement_len,
            } => write!(
                formatter,
                "fixed patch for 0x{:X}..0x{:X} has {replacement_len} replacement bytes",
                range.start, range.end
            ),
            Self::OverlappingPatches { left, right } => write!(
                formatter,
                "patches overlap at 0x{:X}..0x{:X} and 0x{:X}..0x{:X}",
                left.start, left.end, right.start, right.end
            ),
        }
    }
}

impl StdError for Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SectorId {
    pub cylinder: u16,
    pub head: u8,
    pub record: u16,
    pub size_code: u8,
}

impl SectorId {
    pub fn nominal_size(self) -> Option<usize> {
        128usize.checked_shl(u32::from(self.size_code))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SectorAddress {
    pub track_slot: usize,
    pub physical_ordinal: usize,
    pub id: SectorId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawGeometry {
    pub cylinders: u16,
    pub heads: u8,
    pub sectors_per_track: u16,
    pub bytes_per_sector: usize,
    pub first_record: u16,
    pub size_code: u8,
}

impl RawGeometry {
    pub fn sector_count(self) -> Option<usize> {
        usize::from(self.cylinders)
            .checked_mul(usize::from(self.heads))?
            .checked_mul(usize::from(self.sectors_per_track))
    }

    pub fn byte_len(self) -> Option<usize> {
        self.sector_count()?.checked_mul(self.bytes_per_sector)
    }

    pub fn physical_view(
        self,
        name: impl Into<String>,
        source_len: usize,
        data_range: Range<usize>,
    ) -> Result<LinearView> {
        validate_source_range("raw-sector data", &data_range, source_len)?;
        if self.cylinders == 0
            || self.heads == 0
            || self.sectors_per_track == 0
            || self.bytes_per_sector == 0
        {
            return Err(Error::InvalidRawGeometry(
                "cylinders, heads, sectors, and sector bytes must be nonzero".to_owned(),
            ));
        }
        if (SectorId {
            cylinder: 0,
            head: 0,
            record: self.first_record,
            size_code: self.size_code,
        })
        .nominal_size()
            != Some(self.bytes_per_sector)
        {
            return Err(Error::InvalidRawGeometry(format!(
                "size code {} does not describe {} bytes",
                self.size_code, self.bytes_per_sector
            )));
        }
        self.first_record
            .checked_add(self.sectors_per_track - 1)
            .ok_or_else(|| {
                Error::InvalidRawGeometry("sector record numbers overflow u16".to_owned())
            })?;
        let expected = self
            .byte_len()
            .ok_or_else(|| Error::InvalidRawGeometry("byte length overflows usize".to_owned()))?;
        if data_range.len() != expected {
            return Err(Error::RawGeometryLengthMismatch {
                expected,
                actual: data_range.len(),
            });
        }

        let mut segments = Vec::with_capacity(self.sector_count().unwrap_or(0));
        let mut cursor = data_range.start;
        for cylinder in 0..self.cylinders {
            for head in 0..self.heads {
                let track_slot =
                    usize::from(cylinder) * usize::from(self.heads) + usize::from(head);
                for ordinal in 0..self.sectors_per_track {
                    let end = cursor + self.bytes_per_sector;
                    segments.push(ViewSegment {
                        sector: SectorAddress {
                            track_slot,
                            physical_ordinal: usize::from(ordinal),
                            id: SectorId {
                                cylinder,
                                head,
                                record: self.first_record + ordinal,
                                size_code: self.size_code,
                            },
                        },
                        source_range: cursor..end,
                    });
                    cursor = end;
                }
            }
        }
        LinearView::new(name, source_len, segments)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewSegment {
    pub sector: SectorAddress,
    pub source_range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedSpan {
    pub sector: SectorAddress,
    pub logical_range: Range<usize>,
    pub source_range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinearView {
    name: String,
    source_len: usize,
    logical_len: usize,
    segments: Vec<ViewSegment>,
}

impl LinearView {
    pub fn new(
        name: impl Into<String>,
        source_len: usize,
        segments: Vec<ViewSegment>,
    ) -> Result<Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(Error::EmptyViewName);
        }
        if segments.is_empty() {
            return Err(Error::EmptyView);
        }
        let mut logical_len = 0usize;
        for (index, segment) in segments.iter().enumerate() {
            validate_source_range(
                &format!("view {name:?} segment {index}"),
                &segment.source_range,
                source_len,
            )?;
            logical_len = logical_len
                .checked_add(segment.source_range.len())
                .ok_or_else(|| Error::InvalidLogicalRange {
                    view: name.clone(),
                    range: 0..usize::MAX,
                    view_len: usize::MAX,
                })?;
        }
        Ok(Self {
            name,
            source_len,
            logical_len,
            segments,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source_len(&self) -> usize {
        self.source_len
    }

    pub fn len(&self) -> usize {
        self.logical_len
    }

    pub fn is_empty(&self) -> bool {
        self.logical_len == 0
    }

    pub fn segments(&self) -> &[ViewSegment] {
        &self.segments
    }

    pub fn map_range(&self, range: Range<usize>) -> Result<Vec<MappedSpan>> {
        if range.start > range.end || range.end > self.logical_len {
            return Err(Error::InvalidLogicalRange {
                view: self.name.clone(),
                range,
                view_len: self.logical_len,
            });
        }
        if range.is_empty() {
            return Ok(Vec::new());
        }

        let mut mapped = Vec::new();
        let mut logical_cursor = 0usize;
        for segment in &self.segments {
            let segment_logical = logical_cursor..logical_cursor + segment.source_range.len();
            logical_cursor = segment_logical.end;
            let start = range.start.max(segment_logical.start);
            let end = range.end.min(segment_logical.end);
            if start >= end {
                continue;
            }
            let within_segment = start - segment_logical.start;
            let source_start = segment.source_range.start + within_segment;
            mapped.push(MappedSpan {
                sector: segment.sector,
                logical_range: start..end,
                source_range: source_start..source_start + (end - start),
            });
        }
        Ok(mapped)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtentKind {
    Boot,
    Program,
    Directory,
    Data,
    Mixed,
    Unknown,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extent {
    pub name: Option<String>,
    pub kind: ExtentKind,
    pub view: String,
    pub logical_ranges: Vec<Range<usize>>,
    pub provenance: Option<String>,
}

impl Extent {
    pub fn resolve(&self, view: &LinearView) -> Result<Vec<MappedSpan>> {
        if self.view != view.name() {
            return Err(Error::WrongView {
                expected: self.view.clone(),
                actual: view.name().to_owned(),
            });
        }
        if self.logical_ranges.is_empty() {
            return Err(Error::EmptyExtent);
        }
        let mut mapped = Vec::new();
        for range in &self.logical_ranges {
            mapped.extend(view.map_range(range.clone())?);
        }
        Ok(mapped)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionKind {
    Header,
    TrackMetadata,
    SectorMetadata,
    SectorData,
    Comment,
    Padding,
    Custom(String),
    Opaque,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WritePolicy {
    Preserve,
    PatchFixed,
    RebuildByCodec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    pub name: Option<String>,
    pub source_range: Range<usize>,
    pub kind: RegionKind,
    pub write_policy: WritePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionMap {
    source_len: usize,
    regions: Vec<Region>,
}

impl RegionMap {
    pub fn from_claims(source_len: usize, mut claims: Vec<Region>) -> Result<Self> {
        for (index, claim) in claims.iter().enumerate() {
            validate_source_range(
                &format!("region claim {index}"),
                &claim.source_range,
                source_len,
            )?;
        }
        claims.sort_by_key(|claim| (claim.source_range.start, claim.source_range.end));
        for pair in claims.windows(2) {
            if pair[0].source_range.end > pair[1].source_range.start {
                return Err(Error::OverlappingRegions {
                    left: pair[0].source_range.clone(),
                    right: pair[1].source_range.clone(),
                });
            }
        }

        let mut regions = Vec::new();
        let mut cursor = 0usize;
        for claim in claims {
            if cursor < claim.source_range.start {
                regions.push(Region {
                    name: None,
                    source_range: cursor..claim.source_range.start,
                    kind: RegionKind::Opaque,
                    write_policy: WritePolicy::Preserve,
                });
            }
            cursor = claim.source_range.end;
            regions.push(claim);
        }
        if cursor < source_len {
            regions.push(Region {
                name: None,
                source_range: cursor..source_len,
                kind: RegionKind::Opaque,
                write_policy: WritePolicy::Preserve,
            });
        }
        Ok(Self {
            source_len,
            regions,
        })
    }

    pub fn source_len(&self) -> usize {
        self.source_len
    }

    pub fn regions(&self) -> &[Region] {
        &self.regions
    }

    pub fn region_containing(&self, range: &Range<usize>) -> Option<&Region> {
        self.regions.iter().find(|region| {
            region.source_range.start <= range.start && region.source_range.end >= range.end
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedPatch {
    pub source_range: Range<usize>,
    pub replacement: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PatchPlan {
    patches: Vec<FixedPatch>,
}

impl PatchPlan {
    pub fn new(mut patches: Vec<FixedPatch>, source_len: usize) -> Result<Self> {
        for (index, patch) in patches.iter().enumerate() {
            validate_source_range(&format!("patch {index}"), &patch.source_range, source_len)?;
            if patch.source_range.len() != patch.replacement.len() {
                return Err(Error::InvalidPatchLength {
                    range: patch.source_range.clone(),
                    replacement_len: patch.replacement.len(),
                });
            }
        }
        patches.sort_by_key(|patch| (patch.source_range.start, patch.source_range.end));
        for pair in patches.windows(2) {
            if pair[0].source_range.end > pair[1].source_range.start {
                return Err(Error::OverlappingPatches {
                    left: pair[0].source_range.clone(),
                    right: pair[1].source_range.clone(),
                });
            }
        }
        Ok(Self { patches })
    }

    pub fn patches(&self) -> &[FixedPatch] {
        &self.patches
    }

    pub fn apply(&self, source: &[u8]) -> Result<Vec<u8>> {
        for (index, patch) in self.patches.iter().enumerate() {
            validate_source_range(&format!("patch {index}"), &patch.source_range, source.len())?;
        }
        let mut output = source.to_vec();
        for patch in &self.patches {
            output[patch.source_range.clone()].copy_from_slice(&patch.replacement);
        }
        Ok(output)
    }
}

fn validate_source_range(label: &str, range: &Range<usize>, source_len: usize) -> Result<()> {
    if range.start > range.end || range.end > source_len {
        return Err(Error::InvalidSourceRange {
            label: label.to_owned(),
            range: range.clone(),
            source_len,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sector(slot: usize, start: usize, end: usize) -> ViewSegment {
        ViewSegment {
            sector: SectorAddress {
                track_slot: slot,
                physical_ordinal: 0,
                id: SectorId {
                    cylinder: slot as u16,
                    head: 0,
                    record: 1,
                    size_code: 1,
                },
            },
            source_range: start..end,
        }
    }

    #[test]
    fn maps_across_noncontiguous_sector_data() {
        let view = LinearView::new(
            "payload",
            0x500,
            vec![sector(0, 0x100, 0x180), sector(2, 0x300, 0x380)],
        )
        .expect("view");
        let mapped = view.map_range(0x70..0x90).expect("mapping");
        assert_eq!(mapped.len(), 2);
        assert_eq!(mapped[0].source_range, 0x170..0x180);
        assert_eq!(mapped[1].source_range, 0x300..0x310);
    }

    #[test]
    fn fills_unclaimed_bytes_as_preserved_opaque_regions() {
        let map = RegionMap::from_claims(
            100,
            vec![Region {
                name: Some("known".to_owned()),
                source_range: 20..40,
                kind: RegionKind::Header,
                write_policy: WritePolicy::RebuildByCodec,
            }],
        )
        .expect("region map");
        assert_eq!(map.regions().len(), 3);
        assert_eq!(map.regions()[0].source_range, 0..20);
        assert_eq!(map.regions()[0].kind, RegionKind::Opaque);
        assert_eq!(map.regions()[2].source_range, 40..100);
    }

    #[test]
    fn fixed_patch_preserves_every_other_byte() {
        let source = (0u8..16).collect::<Vec<_>>();
        let plan = PatchPlan::new(
            vec![FixedPatch {
                source_range: 4..7,
                replacement: vec![0xAA, 0xBB, 0xCC],
            }],
            source.len(),
        )
        .expect("plan");
        let output = plan.apply(&source).expect("apply");
        assert_eq!(&output[..4], &source[..4]);
        assert_eq!(&output[4..7], &[0xAA, 0xBB, 0xCC]);
        assert_eq!(&output[7..], &source[7..]);
    }

    #[test]
    fn raw_geometry_requires_an_explicit_exact_layout() {
        let geometry = RawGeometry {
            cylinders: 2,
            heads: 2,
            sectors_per_track: 2,
            bytes_per_sector: 256,
            first_record: 1,
            size_code: 1,
        };
        let view = geometry
            .physical_view("raw", 0x900, 0x100..0x900)
            .expect("raw view");
        assert_eq!(view.len(), 0x800);
        assert_eq!(view.segments().len(), 8);
        assert_eq!(view.segments()[4].sector.id.cylinder, 1);
        assert_eq!(view.segments()[4].source_range, 0x500..0x600);
    }
}
