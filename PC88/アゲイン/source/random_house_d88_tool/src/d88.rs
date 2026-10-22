use anyhow::{bail, Context, Result};
use std::collections::{HashMap, HashSet};
use std::ops::Range;

const D88_HEADER_SIZE: usize = 0x2A0;
const TRACK_TABLE_OFFSET: usize = 0x20;
const TRACK_TABLE_ENTRIES: usize = 160;
const TRACK_SIDE_COUNT: usize = 80;
const SECTORS_PER_TRACK_SIDE: usize = 16;
const SECTOR_SIZE: usize = 256;
const DIRECTORY_TRACK_SIDE: usize = 37;
const DIRECTORY_SECTORS: usize = 13;
const DIRECTORY_ENTRY_SIZE: usize = 16;
const FAT_COPIES: usize = 3;
const FAT_ENTRIES: usize = 160;
const CLUSTER_SECTORS: usize = 8;
const FAT_RESERVED: u8 = 0xFE;

#[derive(Debug, Clone)]
struct Sector {
    c: u8,
    h: u8,
    r: u8,
    n: u8,
    data: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct DiskFile {
    pub index: usize,
    pub name: String,
    pub attribute: u8,
    pub start_cluster: u8,
    pub clusters: Vec<u8>,
    pub final_sectors: u8,
    pub data: Vec<u8>,
    pub directory_offset: usize,
}

#[derive(Debug, Clone)]
pub struct D88Image {
    source: Vec<u8>,
    logical_sectors: Vec<Range<usize>>,
    directory_ranges: Vec<Range<usize>>,
    fat_ranges: Vec<Range<usize>>,
    fat: Vec<u8>,
    pub files: Vec<DiskFile>,
}

#[derive(Debug, Clone)]
pub struct RebuildResult {
    pub bytes: Vec<u8>,
    pub changed_files: usize,
}

impl D88Image {
    pub fn parse(source: Vec<u8>) -> Result<Self> {
        if source.len() < D88_HEADER_SIZE {
            bail!("D88 is shorter than the 0x{D88_HEADER_SIZE:X}-byte header");
        }
        let declared_size = usize::try_from(read_u32(&source, 0x1C)?)
            .context("D88 declared size does not fit usize")?;
        if declared_size != source.len() {
            bail!(
                "D88 declared size {declared_size} does not match actual size {}",
                source.len()
            );
        }

        let offsets = parse_track_offsets(&source)?;
        if offsets.len() != TRACK_SIDE_COUNT {
            bail!(
                "D88 has {} populated track-sides, expected {TRACK_SIDE_COUNT}",
                offsets.len()
            );
        }
        if offsets[0] != D88_HEADER_SIZE {
            bail!(
                "first track starts at 0x{:X}, expected 0x{D88_HEADER_SIZE:X}",
                offsets[0]
            );
        }

        let mut tracks = Vec::with_capacity(offsets.len());
        for (index, start) in offsets.iter().copied().enumerate() {
            let end = offsets.get(index + 1).copied().unwrap_or(source.len());
            tracks.push(parse_track(&source, index, start, end)?);
        }
        validate_geometry(&tracks)?;

        let logical_sectors = tracks
            .iter()
            .flat_map(|track| track.iter().map(|sector| sector.data.clone()))
            .collect::<Vec<_>>();
        if logical_sectors.len() != FAT_ENTRIES * CLUSTER_SECTORS {
            bail!(
                "logical sector count {} does not match {} FAT clusters",
                logical_sectors.len(),
                FAT_ENTRIES
            );
        }

        let system = &tracks[DIRECTORY_TRACK_SIDE];
        let directory_ranges = system[..DIRECTORY_SECTORS]
            .iter()
            .map(|sector| sector.data.clone())
            .collect::<Vec<_>>();
        let fat_ranges = system[DIRECTORY_SECTORS..]
            .iter()
            .map(|sector| sector.data.clone())
            .collect::<Vec<_>>();
        if fat_ranges.len() != FAT_COPIES {
            bail!(
                "D88 has {} FAT copies, expected {FAT_COPIES}",
                fat_ranges.len()
            );
        }

        let directory = gather_ranges(&source, &directory_ranges);
        let fat_copies = fat_ranges
            .iter()
            .map(|range| source[range.start..range.start + FAT_ENTRIES].to_vec())
            .collect::<Vec<_>>();
        if fat_copies[1..].iter().any(|copy| *copy != fat_copies[0]) {
            bail!("the three N88 FAT copies are not byte-identical");
        }
        let files = parse_directory(&source, &directory, &fat_copies[0], &logical_sectors)?;

        Ok(Self {
            source,
            logical_sectors,
            directory_ranges,
            fat_ranges,
            fat: fat_copies[0].clone(),
            files,
        })
    }

    pub fn source_bytes(&self) -> &[u8] {
        &self.source
    }

    pub fn file(&self, name: &str) -> Result<&DiskFile> {
        self.files
            .iter()
            .find(|file| file.name.eq_ignore_ascii_case(name))
            .with_context(|| format!("D88 file {name:?} was not found"))
    }

    pub fn free_cluster_count(&self) -> usize {
        self.fat.iter().filter(|value| **value == 0xFF).count()
    }

    pub fn rebuild(&self, replacements: &HashMap<String, Vec<u8>>) -> Result<RebuildResult> {
        let known_names = self
            .files
            .iter()
            .map(|file| file.name.to_ascii_uppercase())
            .collect::<HashSet<_>>();
        for name in replacements.keys() {
            if !known_names.contains(&name.to_ascii_uppercase()) {
                bail!("replacement refers to unknown D88 file {name:?}");
            }
        }

        let mut desired = Vec::with_capacity(self.files.len());
        let mut released = Vec::new();
        let mut retained = HashSet::new();
        let mut changed_files = 0usize;
        for file in &self.files {
            let data = replacements
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&file.name))
                .map_or(file.data.as_slice(), |(_, data)| data.as_slice());
            if data != file.data {
                changed_files += 1;
            }
            if data.is_empty() || data.len() % SECTOR_SIZE != 0 {
                bail!(
                    "replacement for {} must contain a nonzero whole number of 256-byte sectors",
                    file.name
                );
            }
            let sector_count = data.len() / SECTOR_SIZE;
            let required_clusters = sector_count.div_ceil(CLUSTER_SECTORS);
            let keep = required_clusters.min(file.clusters.len());
            let chain = file.clusters[..keep].to_vec();
            retained.extend(chain.iter().copied());
            released.extend(file.clusters[keep..].iter().copied());
            desired.push((data, sector_count, required_clusters, chain));
        }

        let mut free = (0..FAT_ENTRIES)
            .filter_map(|index| {
                (self.fat[index] == 0xFF && !retained.contains(&(index as u8)))
                    .then_some(index as u8)
            })
            .chain(released.iter().copied())
            .collect::<Vec<_>>();
        free.sort_unstable();
        free.dedup();
        let needed = desired
            .iter()
            .map(|(_, _, required, chain)| required.saturating_sub(chain.len()))
            .sum::<usize>();
        if needed > free.len() {
            let details = self
                .files
                .iter()
                .zip(desired.iter())
                .filter_map(|(file, (_, sector_count, required, chain))| {
                    let extra = required.saturating_sub(chain.len());
                    (extra != 0).then(|| {
                        format!(
                            "{}:+{} cluster(s), {} sector(s), {} byte(s)",
                            file.name,
                            extra,
                            sector_count,
                            sector_count * SECTOR_SIZE
                        )
                    })
                })
                .collect::<Vec<_>>()
                .join("; ");
            bail!(
                "D88 needs {needed} additional FAT cluster(s), but only {} are free; growth: {details}",
                free.len(),
            );
        }
        let mut free = free.into_iter();
        for (_, _, required_clusters, chain) in &mut desired {
            while chain.len() < *required_clusters {
                chain.push(free.next().context("free-cluster accounting mismatch")?);
            }
        }

        let mut output = self.source.clone();
        let planned = desired
            .iter()
            .flat_map(|(_, _, _, chain)| chain.iter().copied())
            .collect::<HashSet<_>>();
        for cluster in self
            .files
            .iter()
            .flat_map(|file| file.clusters.iter().copied())
        {
            if !planned.contains(&cluster) {
                write_fat_value(&mut output, &self.fat_ranges, cluster, 0xFF);
            }
        }

        for (file_index, file) in self.files.iter().enumerate() {
            let (replacement, sector_count, _, chain) = &desired[file_index];
            if replacement.is_empty() || replacement.len() % SECTOR_SIZE != 0 {
                bail!(
                    "replacement for {} must contain a nonzero whole number of 256-byte sectors",
                    file.name
                );
            }
            let final_sectors = sector_count - (chain.len() - 1) * CLUSTER_SECTORS;
            let final_sectors = u8::try_from(final_sectors)
                .context("final sector count does not fit in one byte")?;

            for (source_sector, target_range) in replacement
                .chunks_exact(SECTOR_SIZE)
                .zip(chain_sector_ranges(chain, &self.logical_sectors)?.into_iter())
            {
                output[target_range].copy_from_slice(source_sector);
            }
            for (chain_index, cluster) in chain.iter().copied().enumerate() {
                let value = if chain_index + 1 == chain.len() {
                    0xC0 + final_sectors
                } else {
                    chain[chain_index + 1]
                };
                write_fat_value(&mut output, &self.fat_ranges, cluster, value);
            }
            write_gathered_byte(
                &mut output,
                &self.directory_ranges,
                file.directory_offset + 10,
                chain[0],
            )?;
        }

        let verified =
            Self::parse(output.clone()).context("rebuilt D88 failed structural parse")?;
        if verified.files.len() != self.files.len() {
            bail!("rebuilt D88 file count changed");
        }
        for (file_index, expected) in self.files.iter().enumerate() {
            let expected_data = replacements
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&expected.name))
                .map_or(expected.data.as_slice(), |(_, data)| data.as_slice());
            let actual = verified.file(&expected.name)?;
            let expected_chain = &desired[file_index].3;
            if actual.attribute != expected.attribute
                || actual.start_cluster != expected_chain[0]
                || actual.clusters != *expected_chain
                || actual.data != expected_data
            {
                bail!("rebuilt D88 verification mismatch for {}", expected.name);
            }
        }

        Ok(RebuildResult {
            bytes: output,
            changed_files,
        })
    }
}

fn parse_track_offsets(bytes: &[u8]) -> Result<Vec<usize>> {
    let mut offsets = Vec::new();
    let mut reached_empty = false;
    for index in 0..TRACK_TABLE_ENTRIES {
        let value = usize::try_from(read_u32(bytes, TRACK_TABLE_OFFSET + index * 4)?)
            .context("track offset does not fit usize")?;
        if value == 0 {
            reached_empty = true;
            continue;
        }
        if reached_empty {
            bail!("D88 track table has data after its first empty entry");
        }
        if value < D88_HEADER_SIZE || value >= bytes.len() {
            bail!("D88 track offset 0x{value:X} at index {index} is out of bounds");
        }
        if offsets.last().is_some_and(|previous| value <= *previous) {
            bail!("D88 track offsets are not strictly increasing at index {index}");
        }
        offsets.push(value);
    }
    Ok(offsets)
}

fn parse_track(bytes: &[u8], track_index: usize, start: usize, end: usize) -> Result<Vec<Sector>> {
    if start >= end || end > bytes.len() {
        bail!("invalid D88 bounds for track-side {track_index}");
    }
    let mut sectors = Vec::new();
    let mut cursor = start;
    while cursor < end {
        if cursor + 16 > end {
            bail!("truncated sector header in track-side {track_index}");
        }
        let count = usize::from(read_u16(bytes, cursor + 4)?);
        let data_size = usize::from(read_u16(bytes, cursor + 14)?);
        let data_start = cursor + 16;
        let data_end = data_start
            .checked_add(data_size)
            .context("D88 sector size overflow")?;
        if data_end > end {
            bail!("sector crosses track-side {track_index} boundary");
        }
        if count != SECTORS_PER_TRACK_SIDE || data_size != SECTOR_SIZE {
            bail!(
                "unexpected geometry at track-side {track_index}: count={count}, size={data_size}"
            );
        }
        sectors.push(Sector {
            c: bytes[cursor],
            h: bytes[cursor + 1],
            r: bytes[cursor + 2],
            n: bytes[cursor + 3],
            data: data_start..data_end,
        });
        cursor = data_end;
    }
    if cursor != end || sectors.len() != SECTORS_PER_TRACK_SIDE {
        bail!("track-side {track_index} does not contain exactly 16 sectors");
    }
    Ok(sectors)
}

fn validate_geometry(tracks: &[Vec<Sector>]) -> Result<()> {
    for (track_index, sectors) in tracks.iter().enumerate() {
        for (sector_index, sector) in sectors.iter().enumerate() {
            if sector.c != (track_index / 2) as u8
                || sector.h != (track_index % 2) as u8
                || sector.r != (sector_index + 1) as u8
                || sector.n != 1
                || sector.data.len() != SECTOR_SIZE
            {
                bail!("unexpected CHRN or size at track-side {track_index}, sector {sector_index}");
            }
        }
    }
    Ok(())
}

fn parse_directory(
    bytes: &[u8],
    directory: &[u8],
    fat: &[u8],
    logical_sectors: &[Range<usize>],
) -> Result<Vec<DiskFile>> {
    if !directory.len().is_multiple_of(DIRECTORY_ENTRY_SIZE) || fat.len() != FAT_ENTRIES {
        bail!("invalid N88 directory or FAT size");
    }
    let mut files = Vec::new();
    let mut used_clusters = HashSet::new();
    let mut names = HashSet::new();
    for (entry_index, entry) in directory.chunks_exact(DIRECTORY_ENTRY_SIZE).enumerate() {
        if entry[0] == 0xFF {
            break;
        }
        if entry[0] == 0x00 {
            continue;
        }
        let name = decode_filename(entry)?;
        if !names.insert(name.to_ascii_uppercase()) {
            bail!("duplicate case-insensitive N88 filename {name:?}");
        }
        let start_cluster = entry[10];
        let mut current = start_cluster;
        let mut clusters = Vec::new();
        let final_sectors = loop {
            if usize::from(current) >= FAT_ENTRIES {
                bail!("{name}: FAT cluster {current:02X} is out of range");
            }
            if !used_clusters.insert(current) {
                bail!("{name}: FAT cluster {current:02X} is shared or repeated");
            }
            clusters.push(current);
            let next = fat[usize::from(current)];
            if (0xC1..=0xC8).contains(&next) {
                break next - 0xC0;
            }
            if next == FAT_RESERVED || usize::from(next) >= FAT_ENTRIES {
                bail!("{name}: invalid FAT value {next:02X} at cluster {current:02X}");
            }
            current = next;
        };
        let mut data = Vec::new();
        for (index, cluster) in clusters.iter().copied().enumerate() {
            let count = if index + 1 == clusters.len() {
                usize::from(final_sectors)
            } else {
                CLUSTER_SECTORS
            };
            let start = usize::from(cluster) * CLUSTER_SECTORS;
            for range in logical_sectors
                .get(start..start + count)
                .with_context(|| format!("{name}: cluster sector range is out of bounds"))?
            {
                data.extend_from_slice(&bytes[range.clone()]);
            }
        }
        files.push(DiskFile {
            index: files.len(),
            name,
            attribute: entry[9],
            start_cluster,
            clusters,
            final_sectors,
            data,
            directory_offset: entry_index * DIRECTORY_ENTRY_SIZE,
        });
    }
    if files.is_empty() {
        bail!("N88 directory contains no active files");
    }
    Ok(files)
}

fn decode_filename(entry: &[u8]) -> Result<String> {
    let base = trim_spaces(&entry[..6]);
    let extension = trim_spaces(&entry[6..9]);
    if base.is_empty() {
        bail!("N88 directory entry has an empty basename");
    }
    for byte in base.iter().chain(extension) {
        if !(0x20..=0x7E).contains(byte) || matches!(*byte, b'/' | b'\\' | b':') {
            bail!("unsupported byte 0x{byte:02X} in N88 filename");
        }
    }
    let base = std::str::from_utf8(base).context("N88 basename is not ASCII")?;
    let extension = std::str::from_utf8(extension).context("N88 extension is not ASCII")?;
    if extension.is_empty() {
        Ok(base.to_owned())
    } else {
        Ok(format!("{base}.{extension}"))
    }
}

fn trim_spaces(raw: &[u8]) -> &[u8] {
    let end = raw
        .iter()
        .rposition(|byte| *byte != b' ')
        .map_or(0, |index| index + 1);
    &raw[..end]
}

fn chain_sector_ranges(chain: &[u8], logical: &[Range<usize>]) -> Result<Vec<Range<usize>>> {
    let mut ranges = Vec::with_capacity(chain.len() * CLUSTER_SECTORS);
    for cluster in chain {
        let start = usize::from(*cluster) * CLUSTER_SECTORS;
        ranges.extend_from_slice(
            logical
                .get(start..start + CLUSTER_SECTORS)
                .context("FAT cluster range is out of bounds")?,
        );
    }
    Ok(ranges)
}

fn write_fat_value(output: &mut [u8], ranges: &[Range<usize>], cluster: u8, value: u8) {
    for range in ranges {
        output[range.start + usize::from(cluster)] = value;
    }
}

fn write_gathered_byte(
    output: &mut [u8],
    ranges: &[Range<usize>],
    logical_offset: usize,
    value: u8,
) -> Result<()> {
    let mut remaining = logical_offset;
    for range in ranges {
        if remaining < range.len() {
            output[range.start + remaining] = value;
            return Ok(());
        }
        remaining -= range.len();
    }
    bail!("gathered directory write at 0x{logical_offset:X} is out of bounds")
}

fn gather_ranges(bytes: &[u8], ranges: &[Range<usize>]) -> Vec<u8> {
    let mut output = Vec::with_capacity(ranges.iter().map(Range::len).sum());
    for range in ranges {
        output.extend_from_slice(&bytes[range.clone()]);
    }
    output
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let raw = bytes
        .get(offset..offset + 2)
        .with_context(|| format!("u16 read at 0x{offset:X} is out of bounds"))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let raw = bytes
        .get(offset..offset + 4)
        .with_context(|| format!("u32 read at 0x{offset:X} is out of bounds"))?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}
