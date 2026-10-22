#![forbid(unsafe_code)]

use std::error::Error as StdError;
use std::fmt;
use std::ops::Range;

use vn_sector_map::{LinearView, PatchPlan, RegionMap, SectorAddress, SectorId, ViewSegment};

pub const R0_SIGNATURE: &[u8; 14] = b"T98FDDIMAGE.R0";
pub const R1_SIGNATURE: &[u8; 14] = b"T98FDDIMAGE.R1";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidModel(String),
    Mapping(vn_sector_map::Error),
    Decode(String),
    Encode(String),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidModel(message) => write!(formatter, "invalid NFD model: {message}"),
            Self::Mapping(error) => write!(formatter, "NFD sector mapping failed: {error}"),
            Self::Decode(message) => write!(formatter, "NFD decode failed: {message}"),
            Self::Encode(message) => write!(formatter, "NFD encode failed: {message}"),
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
pub enum Revision {
    R0,
    R1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub revision: Revision,
    pub raw_comment: Vec<u8>,
    pub header_size: u32,
    pub write_protect: u8,
    pub heads: u8,
    pub raw_track_offsets: Vec<u32>,
    pub additional_info_offset: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadStatus {
    pub bios_status: u8,
    pub st0: u8,
    pub st1: u8,
    pub st2: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sector {
    pub address: SectorAddress,
    pub mfm: u8,
    pub deleted_data: u8,
    pub status: ReadStatus,
    pub pda: u8,
    pub metadata_range: Range<usize>,
    pub primary_data_range: Range<usize>,
    pub retry_data_ranges: Vec<Range<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticRead {
    pub command: u8,
    pub id: SectorId,
    pub status: ReadStatus,
    pub pda: u8,
    pub metadata_range: Range<usize>,
    pub primary_data_range: Range<usize>,
    pub retry_data_ranges: Vec<Range<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    pub slot: usize,
    pub metadata_range: Range<usize>,
    pub sectors: Vec<Sector>,
    pub diagnostic_reads: Vec<DiagnosticRead>,
    pub opaque_metadata: Vec<Range<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub source_len: usize,
    pub header: Header,
    pub header_range: Range<usize>,
    pub data_range: Range<usize>,
    pub tracks: Vec<Track>,
    pub regions: RegionMap,
}

impl Image {
    /// Returns one primary payload per ordinary sector in stored track order.
    /// Retry payloads and diagnostic reads remain available only through their
    /// NFD-specific records and are never silently flattened into this view.
    pub fn primary_view(&self) -> Result<LinearView> {
        let segments = self
            .tracks
            .iter()
            .flat_map(|track| &track.sectors)
            .map(|sector| ViewSegment {
                sector: sector.address,
                source_range: sector.primary_data_range.clone(),
            })
            .collect();
        Ok(LinearView::new(
            "nfd-primary-physical",
            self.source_len,
            segments,
        )?)
    }
}

/// Decodes either NFD revision without discarding retries, diagnostic reads,
/// reserved fields, empty track slots, or the original storage order.
pub trait Decoder {
    fn decode(&self, source: &[u8]) -> Result<Image>;
}

/// Applies fixed edits or a revision-aware rebuild. Converting R0 to R1 is a
/// separate explicit operation and must never happen as a side effect of save.
pub trait Encoder {
    fn rebuild(&self, source: &[u8], image: &Image, patches: &PatchPlan) -> Result<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use vn_sector_map::{RegionMap, SectorId};

    #[test]
    fn primary_view_excludes_retry_and_diagnostic_payloads() {
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
        let image = Image {
            source_len: 0x600,
            header: Header {
                revision: Revision::R1,
                raw_comment: Vec::new(),
                header_size: 0x200,
                write_protect: 0,
                heads: 2,
                raw_track_offsets: vec![0; 164],
                additional_info_offset: None,
            },
            header_range: 0..0x200,
            data_range: 0x200..0x600,
            tracks: vec![Track {
                slot: 0,
                metadata_range: 0x120..0x150,
                sectors: vec![Sector {
                    address,
                    mfm: 1,
                    deleted_data: 0,
                    status: ReadStatus {
                        bios_status: 0,
                        st0: 0,
                        st1: 0,
                        st2: 0,
                    },
                    pda: 0x90,
                    metadata_range: 0x130..0x140,
                    primary_data_range: 0x200..0x300,
                    retry_data_ranges: std::iter::once(0x300..0x400).collect(),
                }],
                diagnostic_reads: vec![DiagnosticRead {
                    command: 6,
                    id: address.id,
                    status: ReadStatus {
                        bios_status: 0,
                        st0: 0,
                        st1: 0,
                        st2: 0,
                    },
                    pda: 0x90,
                    metadata_range: 0x140..0x150,
                    primary_data_range: 0x400..0x500,
                    retry_data_ranges: std::iter::once(0x500..0x600).collect(),
                }],
                opaque_metadata: Vec::new(),
            }],
            regions: RegionMap::from_claims(0x600, Vec::new()).expect("regions"),
        };
        let view = image.primary_view().expect("view");
        assert_eq!(view.len(), 0x100);
        assert_eq!(view.segments()[0].source_range, 0x200..0x300);
    }
}
