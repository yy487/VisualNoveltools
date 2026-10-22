#![forbid(unsafe_code)]

use std::error::Error as StdError;
use std::fmt;
use std::ops::Range;

use vn_sector_map::{LinearView, PatchPlan, RegionMap, SectorAddress, ViewSegment};

pub const MIN_HEADER_SIZE: usize = 0x20;

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
            Self::InvalidModel(message) => write!(formatter, "invalid FDI model: {message}"),
            Self::Mapping(error) => write!(formatter, "FDI sector mapping failed: {error}"),
            Self::Decode(message) => write!(formatter, "FDI decode failed: {message}"),
            Self::Encode(message) => write!(formatter, "FDI encode failed: {message}"),
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
pub struct Geometry {
    pub bytes_per_sector: u32,
    pub sectors_per_track: u32,
    pub heads: u32,
    pub cylinders: u32,
}

impl Geometry {
    pub fn sector_count(self) -> Option<u64> {
        u64::from(self.sectors_per_track)
            .checked_mul(u64::from(self.heads))?
            .checked_mul(u64::from(self.cylinders))
    }

    pub fn data_len(self) -> Option<u64> {
        self.sector_count()?
            .checked_mul(u64::from(self.bytes_per_sector))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub reserved: u32,
    pub fdd_type: u32,
    pub header_size: u32,
    pub data_size: u32,
    pub geometry: Geometry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sector {
    pub address: SectorAddress,
    pub data_range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub source_len: usize,
    pub header: Header,
    pub fixed_header_range: Range<usize>,
    pub comment_range: Range<usize>,
    pub data_range: Range<usize>,
    pub sectors: Vec<Sector>,
    pub regions: RegionMap,
}

impl Image {
    pub fn physical_view(&self) -> Result<LinearView> {
        let segments = self
            .sectors
            .iter()
            .map(|sector| ViewSegment {
                sector: sector.address,
                source_range: sector.data_range.clone(),
            })
            .collect();
        Ok(LinearView::new("fdi-physical", self.source_len, segments)?)
    }
}

/// Decodes the fixed FDI header, arbitrary comment bytes, and flat CHS data.
///
/// A decoder must validate the checked geometry product and expose every sector
/// range without interpreting an optional filesystem or game layout.
pub trait Decoder {
    fn decode(&self, source: &[u8]) -> Result<Image>;
}

/// Applies proven edits while preserving the fixed header and comment area.
pub trait Encoder {
    fn rebuild(&self, source: &[u8], image: &Image, patches: &PatchPlan) -> Result<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_uses_checked_products() {
        let geometry = Geometry {
            bytes_per_sector: 1024,
            sectors_per_track: 8,
            heads: 2,
            cylinders: 77,
        };
        assert_eq!(geometry.sector_count(), Some(1232));
        assert_eq!(geometry.data_len(), Some(1_261_568));
    }
}
