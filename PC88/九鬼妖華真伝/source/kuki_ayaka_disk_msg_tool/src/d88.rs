use crate::util::{
    commit_output_directory, hex_encode, prepare_output_directory, sha256_hex,
    validate_relative_path, write_output_file,
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

const D88_HEADER_SIZE: usize = 0x2B0;
const TRACK_TABLE_OFFSET: usize = 0x20;
const TRACK_TABLE_ENTRIES: usize = 164;
const PROJECT_TRACKS: usize = 80;
const SYSTEM_SECTORS: usize = 16;
const SYSTEM_SECTOR_SIZE: usize = 256;
const DATA_SECTORS: usize = 5;
const DATA_BLOCK_SIZE: usize = 1024;
const DATA_BLOCKS: usize = 390;
const DIRECTORY_ENTRY_SIZE: usize = 32;
const DIRECTORY_START_BLOCK_OFFSET: usize = 26;
const DIRECTORY_SIZE_OFFSET: usize = 28;
const FAT_FIRST_SECTOR: usize = 12;
const FAT_FREE: u16 = 0xFFFF;
const FAT_END: u16 = 0xFFF8;
const MANIFEST_FORMAT: &str = "kuki-ayaka-d88-workspace-v1";
pub const MANIFEST_NAME: &str = "disk_manifest.json";

#[derive(Debug, Clone)]
pub struct D88ProjectImage {
    bytes: Vec<u8>,
    directory_ranges: Vec<Range<usize>>,
    fat_ranges: Vec<Range<usize>>,
    data_ranges: Vec<Range<usize>>,
    pub files: Vec<D88File>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct D88File {
    pub index: usize,
    pub name: String,
    pub directory_offset: usize,
    pub start_block: u16,
    pub size: u32,
    pub block_chain: Vec<u16>,
    pub data: Vec<u8>,
    raw_entry: [u8; DIRECTORY_ENTRY_SIZE],
}

#[derive(Debug, Clone)]
pub struct RepackResult {
    pub bytes: Vec<u8>,
    pub changed_files: usize,
    pub used_blocks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskWorkspaceManifest {
    pub format: String,
    pub roles: DiskWorkspaceRoles,
    pub source: DiskSource,
    pub disk: DiskSummary,
    pub files: Vec<DiskManifestFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskWorkspaceRoles {
    pub template: PathBuf,
    pub files_root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskSource {
    pub original_path: PathBuf,
    pub original_name: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskSummary {
    pub declared_size: usize,
    pub media_type: String,
    pub track_side_count: usize,
    pub logical_data_blocks: usize,
    pub block_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskManifestFile {
    pub index: usize,
    pub path: PathBuf,
    pub name: String,
    pub directory_offset: usize,
    pub start_block: u16,
    pub size: u32,
    pub sha256: String,
    pub raw_entry_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskUnpackReport {
    pub extracted_files: usize,
    pub output_bytes: usize,
    pub output: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskPackReport {
    pub packed_files: usize,
    pub changed_files: usize,
    pub used_blocks: usize,
    pub output_bytes: usize,
    pub byte_exact_template: bool,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
struct Track {
    sectors: Vec<Sector>,
}

#[derive(Debug, Clone)]
struct Sector {
    c: u8,
    h: u8,
    r: u8,
    n: u8,
    data: Range<usize>,
}

impl D88ProjectImage {
    pub fn parse(bytes: Vec<u8>) -> Result<Self> {
        if bytes.len() < D88_HEADER_SIZE {
            bail!("D88 image is shorter than the 0x{D88_HEADER_SIZE:X}-byte header");
        }
        let declared_size = usize::try_from(read_u32(&bytes, 0x1C)?)
            .context("D88 declared size does not fit usize")?;
        if declared_size != bytes.len() {
            bail!(
                "D88 declared size {declared_size} does not match file size {}",
                bytes.len()
            );
        }
        if bytes[0x1B] != 0x00 {
            bail!(
                "unsupported D88 media type 0x{:02X}; expected 2D (0x00)",
                bytes[0x1B]
            );
        }

        let offsets = parse_track_offsets(&bytes)?;
        if offsets.len() != PROJECT_TRACKS {
            bail!(
                "unsupported D88 track-side count {}; expected {PROJECT_TRACKS}",
                offsets.len()
            );
        }
        if offsets[0] != D88_HEADER_SIZE {
            bail!(
                "first D88 track starts at 0x{:X}, expected 0x{D88_HEADER_SIZE:X}",
                offsets[0]
            );
        }

        let mut tracks = Vec::with_capacity(PROJECT_TRACKS);
        for (index, start) in offsets.iter().copied().enumerate() {
            let end = offsets.get(index + 1).copied().unwrap_or(bytes.len());
            tracks.push(parse_track(&bytes, index, start, end)?);
        }
        validate_project_geometry(&tracks)?;

        let directory_ranges = tracks[1]
            .sectors
            .iter()
            .map(|sector| sector.data.clone())
            .collect::<Vec<_>>();
        let fat_ranges = tracks[0].sectors[FAT_FIRST_SECTOR..]
            .iter()
            .map(|sector| sector.data.clone())
            .collect::<Vec<_>>();
        let data_ranges = tracks[2..]
            .iter()
            .flat_map(|track| track.sectors.iter().map(|sector| sector.data.clone()))
            .collect::<Vec<_>>();
        if data_ranges.len() != DATA_BLOCKS {
            bail!(
                "D88 has {} logical data blocks, expected {DATA_BLOCKS}",
                data_ranges.len()
            );
        }

        let directory = gather_ranges(&bytes, &directory_ranges);
        let fat = gather_ranges(&bytes, &fat_ranges);
        if fat.len() < DATA_BLOCKS * 2 {
            bail!("D88 FAT is too short for {DATA_BLOCKS} data blocks");
        }
        let files = parse_directory(&bytes, &directory, &fat, &data_ranges)?;

        Ok(Self {
            bytes,
            directory_ranges,
            fat_ranges,
            data_ranges,
            files,
        })
    }

    pub fn source_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn rebuild_with_files(&self, replacements: &[Vec<u8>]) -> Result<RepackResult> {
        if replacements.len() != self.files.len() {
            bail!(
                "replacement count {} does not match disk file count {}",
                replacements.len(),
                self.files.len()
            );
        }
        let changed_files = replacements
            .iter()
            .zip(&self.files)
            .filter(|(replacement, file)| replacement.as_slice() != file.data)
            .count();
        if changed_files == 0 {
            return Ok(RepackResult {
                bytes: self.bytes.clone(),
                changed_files: 0,
                used_blocks: self.files.iter().map(|file| file.block_chain.len()).sum(),
            });
        }

        let mut block_counts = Vec::with_capacity(replacements.len());
        let mut used_blocks = 0usize;
        for (index, replacement) in replacements.iter().enumerate() {
            if replacement.is_empty() {
                bail!(
                    "replacement for {} is empty; zero-length files are not supported",
                    self.files[index].name
                );
            }
            u32::try_from(replacement.len()).with_context(|| {
                format!(
                    "replacement for {} exceeds u32 size",
                    self.files[index].name
                )
            })?;
            let blocks = replacement.len().div_ceil(DATA_BLOCK_SIZE);
            used_blocks = used_blocks
                .checked_add(blocks)
                .context("D88 block count overflow")?;
            block_counts.push(blocks);
        }
        if used_blocks > DATA_BLOCKS {
            bail!(
                "repacked files require {used_blocks} blocks, but disk capacity is {DATA_BLOCKS}"
            );
        }

        let mut output = self.bytes.clone();
        for block in 0..DATA_BLOCKS {
            write_logical_u16(&mut output, &self.fat_ranges, block * 2, FAT_FREE)?;
        }

        let mut next_block = 0usize;
        for ((file, replacement), block_count) in
            self.files.iter().zip(replacements).zip(block_counts)
        {
            let start_block = u16::try_from(next_block).context("D88 start block exceeds u16")?;
            for local_index in 0..block_count {
                let block_index = next_block + local_index;
                let range = &self.data_ranges[block_index];
                output[range.clone()].fill(0);
                let source_start = local_index * DATA_BLOCK_SIZE;
                let source_end = (source_start + DATA_BLOCK_SIZE).min(replacement.len());
                output[range.start..range.start + source_end - source_start]
                    .copy_from_slice(&replacement[source_start..source_end]);
                let fat_value = if local_index + 1 == block_count {
                    FAT_END
                } else {
                    u16::try_from(block_index + 1).context("D88 FAT block exceeds u16")?
                };
                write_logical_u16(&mut output, &self.fat_ranges, block_index * 2, fat_value)?;
            }
            write_logical_u16(
                &mut output,
                &self.directory_ranges,
                file.directory_offset + DIRECTORY_START_BLOCK_OFFSET,
                start_block,
            )?;
            write_logical_u32(
                &mut output,
                &self.directory_ranges,
                file.directory_offset + DIRECTORY_SIZE_OFFSET,
                u32::try_from(replacement.len()).context("replacement size exceeds u32")?,
            )?;
            next_block += block_count;
        }

        let verified = Self::parse(output.clone()).context("rebuilt D88 failed verification")?;
        if verified.files.len() != self.files.len() {
            bail!("rebuilt D88 file count changed during verification");
        }
        for ((expected_file, expected_data), actual_file) in
            self.files.iter().zip(replacements).zip(&verified.files)
        {
            if actual_file.name != expected_file.name || actual_file.data != *expected_data {
                bail!(
                    "rebuilt D88 verification mismatch for {}",
                    expected_file.name
                );
            }
        }

        Ok(RepackResult {
            bytes: output,
            changed_files,
            used_blocks,
        })
    }
}

pub fn unpack_disk(source: &Path, output: &Path, overwrite: bool) -> Result<DiskUnpackReport> {
    let source_bytes = fs::read(source)
        .with_context(|| format!("failed to read D88 image {}", source.display()))?;
    let image = D88ProjectImage::parse(source_bytes)
        .with_context(|| format!("failed to parse D88 image {}", source.display()))?;
    let temporary = prepare_output_directory(output, overwrite)?;
    let result = (|| -> Result<()> {
        let files_root = temporary.join("files");
        fs::create_dir(&files_root)
            .with_context(|| format!("failed to create {}", files_root.display()))?;
        let template_path = temporary.join("template.d88");
        fs::write(&template_path, image.source_bytes())
            .with_context(|| format!("failed to write {}", template_path.display()))?;

        let mut manifest_files = Vec::with_capacity(image.files.len());
        let mut windows_names = HashSet::new();
        for file in &image.files {
            if !windows_names.insert(file.name.to_ascii_uppercase()) {
                bail!("duplicate case-insensitive disk filename: {}", file.name);
            }
            let relative = PathBuf::from(&file.name);
            validate_relative_path(&relative)?;
            let destination = files_root.join(&relative);
            fs::write(&destination, &file.data)
                .with_context(|| format!("failed to write {}", destination.display()))?;
            manifest_files.push(DiskManifestFile {
                index: file.index,
                path: relative,
                name: file.name.clone(),
                directory_offset: file.directory_offset,
                start_block: file.start_block,
                size: file.size,
                sha256: sha256_hex(&file.data),
                raw_entry_hex: hex_encode(&file.raw_entry),
            });
        }

        let manifest = DiskWorkspaceManifest {
            format: MANIFEST_FORMAT.to_owned(),
            roles: DiskWorkspaceRoles {
                template: PathBuf::from("template.d88"),
                files_root: PathBuf::from("files"),
            },
            source: DiskSource {
                original_path: source.to_path_buf(),
                original_name: source
                    .file_name()
                    .context("D88 source path has no filename")?
                    .to_string_lossy()
                    .into_owned(),
                sha256: sha256_hex(image.source_bytes()),
            },
            disk: DiskSummary {
                declared_size: image.source_bytes().len(),
                media_type: "2D".to_owned(),
                track_side_count: PROJECT_TRACKS,
                logical_data_blocks: DATA_BLOCKS,
                block_size: DATA_BLOCK_SIZE,
            },
            files: manifest_files,
        };
        let mut json = serde_json::to_vec_pretty(&manifest)
            .context("failed to serialize D88 workspace manifest")?;
        json.push(b'\n');
        let manifest_path = temporary.join(MANIFEST_NAME);
        fs::write(&manifest_path, json)
            .with_context(|| format!("failed to write {}", manifest_path.display()))?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&temporary);
        return Err(error);
    }
    commit_output_directory(&temporary, output, overwrite)?;
    Ok(DiskUnpackReport {
        extracted_files: image.files.len(),
        output_bytes: image.files.iter().map(|file| file.data.len()).sum(),
        output: output.to_path_buf(),
    })
}

pub fn pack_disk(workspace: &Path, output: &Path, overwrite: bool) -> Result<DiskPackReport> {
    let (workspace_root, manifest_path) = if workspace.is_dir() {
        (workspace.to_path_buf(), workspace.join(MANIFEST_NAME))
    } else {
        let parent = workspace
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .context("manifest path has no parent directory")?;
        (parent.to_path_buf(), workspace.to_path_buf())
    };
    let manifest_bytes = fs::read(&manifest_path)
        .with_context(|| format!("failed to read manifest {}", manifest_path.display()))?;
    let manifest: DiskWorkspaceManifest = serde_json::from_slice(&manifest_bytes)
        .with_context(|| format!("invalid manifest JSON {}", manifest_path.display()))?;
    if manifest.format != MANIFEST_FORMAT {
        bail!(
            "unsupported disk workspace format {:?}; expected {:?}",
            manifest.format,
            MANIFEST_FORMAT
        );
    }
    validate_relative_path(&manifest.roles.template)?;
    validate_relative_path(&manifest.roles.files_root)?;
    let template_path = workspace_root.join(&manifest.roles.template);
    let files_root = workspace_root.join(&manifest.roles.files_root);
    if !files_root.is_dir() {
        bail!(
            "workspace files root is not a directory: {}",
            files_root.display()
        );
    }
    reject_source_overwrite(output, &template_path, &manifest.source.original_path)?;

    let template_bytes = fs::read(&template_path)
        .with_context(|| format!("failed to read D88 template {}", template_path.display()))?;
    let template_hash = sha256_hex(&template_bytes);
    if template_hash != manifest.source.sha256 {
        bail!(
            "D88 template hash mismatch: manifest {}, actual {}",
            manifest.source.sha256,
            template_hash
        );
    }
    let image = D88ProjectImage::parse(template_bytes)
        .with_context(|| format!("failed to parse D88 template {}", template_path.display()))?;
    validate_manifest_against_image(&manifest, &image)?;

    let mut replacements = Vec::with_capacity(manifest.files.len());
    for file in &manifest.files {
        validate_relative_path(&file.path)?;
        let path = files_root.join(&file.path);
        let bytes = fs::read(&path)
            .with_context(|| format!("failed to read workspace file {}", path.display()))?;
        replacements.push(bytes);
    }
    let repacked = image.rebuild_with_files(&replacements)?;
    let byte_exact_template = repacked.bytes == image.source_bytes();
    write_output_file(output, &repacked.bytes, overwrite)?;
    Ok(DiskPackReport {
        packed_files: replacements.len(),
        changed_files: repacked.changed_files,
        used_blocks: repacked.used_blocks,
        output_bytes: repacked.bytes.len(),
        byte_exact_template,
        output: output.to_path_buf(),
    })
}

fn validate_manifest_against_image(
    manifest: &DiskWorkspaceManifest,
    image: &D88ProjectImage,
) -> Result<()> {
    if manifest.disk.declared_size != image.source_bytes().len()
        || manifest.disk.media_type != "2D"
        || manifest.disk.track_side_count != PROJECT_TRACKS
        || manifest.disk.logical_data_blocks != DATA_BLOCKS
        || manifest.disk.block_size != DATA_BLOCK_SIZE
    {
        bail!("disk manifest geometry does not match the template");
    }
    if manifest.files.len() != image.files.len() {
        bail!(
            "manifest file count {} does not match template file count {}",
            manifest.files.len(),
            image.files.len()
        );
    }
    for (index, (record, file)) in manifest.files.iter().zip(&image.files).enumerate() {
        if record.index != index
            || record.name != file.name
            || record.directory_offset != file.directory_offset
            || record.start_block != file.start_block
            || record.size != file.size
            || record.sha256 != sha256_hex(&file.data)
            || record.raw_entry_hex != hex_encode(&file.raw_entry)
        {
            bail!("manifest metadata mismatch for template file index {index}");
        }
    }
    Ok(())
}

fn reject_source_overwrite(output: &Path, template: &Path, original: &Path) -> Result<()> {
    let normalized_output = normalize_for_compare(output)?;
    let normalized_template = normalize_for_compare(template)?;
    if normalized_output == normalized_template {
        bail!("output must not overwrite the workspace D88 template");
    }
    if !original.as_os_str().is_empty() && normalized_output == normalize_for_compare(original)? {
        bail!("output must not overwrite the original D88 source");
    }
    Ok(())
}

fn normalize_for_compare(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return path
            .canonicalize()
            .with_context(|| format!("failed to resolve {}", path.display()));
    }
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = parent
        .canonicalize()
        .with_context(|| format!("failed to resolve output parent {}", parent.display()))?;
    let name = path.file_name().context("path has no final component")?;
    Ok(parent.join(name))
}

fn parse_track_offsets(bytes: &[u8]) -> Result<Vec<usize>> {
    let mut offsets = Vec::new();
    let mut found_empty = false;
    for index in 0..TRACK_TABLE_ENTRIES {
        let value = usize::try_from(read_u32(bytes, TRACK_TABLE_OFFSET + index * 4)?)
            .context("D88 track offset does not fit usize")?;
        if value == 0 {
            found_empty = true;
            continue;
        }
        if found_empty {
            bail!("D88 track table has a nonzero entry after an empty entry");
        }
        if value < D88_HEADER_SIZE || value >= bytes.len() {
            bail!("D88 track {index} offset 0x{value:X} is out of bounds");
        }
        if let Some(previous) = offsets.last() {
            if value <= *previous {
                bail!("D88 track offsets are not strictly increasing at index {index}");
            }
        }
        offsets.push(value);
    }
    Ok(offsets)
}

fn parse_track(bytes: &[u8], track_index: usize, start: usize, end: usize) -> Result<Track> {
    if start >= end || end > bytes.len() {
        bail!("invalid D88 bounds for track {track_index}: 0x{start:X}..0x{end:X}");
    }
    if start + 16 > end {
        bail!("D88 track {track_index} has no complete sector header");
    }
    let sector_count = usize::from(read_u16(bytes, start + 4)?);
    if sector_count == 0 {
        bail!("D88 track {track_index} declares zero sectors");
    }
    let mut sectors = Vec::with_capacity(sector_count);
    let mut cursor = start;
    for sector_index in 0..sector_count {
        if cursor + 16 > end {
            bail!("truncated sector header in D88 track {track_index}");
        }
        let count = usize::from(read_u16(bytes, cursor + 4)?);
        if count != sector_count {
            bail!("inconsistent sector count in D88 track {track_index}, sector {sector_index}");
        }
        let data_size = usize::from(read_u16(bytes, cursor + 14)?);
        let data_start = cursor + 16;
        let data_end = data_start
            .checked_add(data_size)
            .context("D88 sector size overflow")?;
        if data_end > end {
            bail!("sector data exceeds D88 track {track_index} bounds");
        }
        let n = bytes[cursor + 3];
        let expected_size = 128usize
            .checked_shl(u32::from(n))
            .context("invalid D88 sector size exponent")?;
        if data_size != expected_size {
            bail!(
                "D88 track {track_index} sector {sector_index} size {data_size} does not match N={n}"
            );
        }
        sectors.push(Sector {
            c: bytes[cursor],
            h: bytes[cursor + 1],
            r: bytes[cursor + 2],
            n,
            data: data_start..data_end,
        });
        cursor = data_end;
    }
    if cursor != end {
        bail!(
            "D88 track {track_index} has {} trailing bytes after its declared sectors",
            end - cursor
        );
    }
    Ok(Track { sectors })
}

fn validate_project_geometry(tracks: &[Track]) -> Result<()> {
    for (track_index, track) in tracks.iter().enumerate() {
        let (expected_count, expected_size, expected_n) = if track_index < 2 {
            (SYSTEM_SECTORS, SYSTEM_SECTOR_SIZE, 1)
        } else {
            (DATA_SECTORS, DATA_BLOCK_SIZE, 3)
        };
        if track.sectors.len() != expected_count {
            bail!(
                "D88 track {track_index} has {} sectors, expected {expected_count}",
                track.sectors.len()
            );
        }
        for (sector_index, sector) in track.sectors.iter().enumerate() {
            if sector.c != (track_index / 2) as u8
                || sector.h != (track_index % 2) as u8
                || sector.r != (sector_index + 1) as u8
                || sector.n != expected_n
                || sector.data.len() != expected_size
            {
                bail!("unexpected geometry at D88 track {track_index}, sector {sector_index}");
            }
        }
    }
    Ok(())
}

fn parse_directory(
    bytes: &[u8],
    directory: &[u8],
    fat: &[u8],
    data_ranges: &[Range<usize>],
) -> Result<Vec<D88File>> {
    if !directory.len().is_multiple_of(DIRECTORY_ENTRY_SIZE) {
        bail!("D88 directory payload is not aligned to 32-byte entries");
    }
    let mut files = Vec::new();
    let mut reached_free = false;
    let mut used_blocks = HashSet::new();
    let mut names = HashSet::new();
    for (index, raw) in directory.chunks_exact(DIRECTORY_ENTRY_SIZE).enumerate() {
        if raw.iter().all(|byte| *byte == 0xFF) {
            reached_free = true;
            continue;
        }
        if reached_free {
            bail!("active D88 directory entry {index} appears after a free entry");
        }
        let raw_entry: [u8; DIRECTORY_ENTRY_SIZE] = raw
            .try_into()
            .expect("chunks_exact always yields a 32-byte directory entry");
        let name = decode_filename(&raw_entry[..11])?;
        if !names.insert(name.to_ascii_uppercase()) {
            bail!("duplicate case-insensitive D88 filename: {name}");
        }
        let start_block = u16::from_le_bytes([
            raw_entry[DIRECTORY_START_BLOCK_OFFSET],
            raw_entry[DIRECTORY_START_BLOCK_OFFSET + 1],
        ]);
        let size = u32::from_le_bytes([
            raw_entry[DIRECTORY_SIZE_OFFSET],
            raw_entry[DIRECTORY_SIZE_OFFSET + 1],
            raw_entry[DIRECTORY_SIZE_OFFSET + 2],
            raw_entry[DIRECTORY_SIZE_OFFSET + 3],
        ]);
        if size == 0 {
            bail!("D88 file {name} is empty; zero-length files are not supported");
        }
        let block_count = usize::try_from(size)
            .context("D88 file size does not fit usize")?
            .div_ceil(DATA_BLOCK_SIZE);
        let mut block_chain = Vec::with_capacity(block_count);
        let mut current = usize::from(start_block);
        for chain_index in 0..block_count {
            if current >= DATA_BLOCKS {
                bail!("D88 FAT block {current} for {name} is out of range");
            }
            if !used_blocks.insert(current) {
                bail!("D88 data block {current} is shared or repeated by {name}");
            }
            block_chain.push(u16::try_from(current).expect("data block count fits u16"));
            let next = read_u16(fat, current * 2)?;
            if chain_index + 1 == block_count {
                if next != FAT_END {
                    bail!("D88 FAT chain for {name} does not end with 0x{FAT_END:04X}");
                }
            } else {
                if next == FAT_END || next == FAT_FREE {
                    bail!("D88 FAT chain for {name} ends before its declared size");
                }
                current = usize::from(next);
            }
        }

        let mut data = Vec::with_capacity(block_count * DATA_BLOCK_SIZE);
        for block in &block_chain {
            data.extend_from_slice(&bytes[data_ranges[usize::from(*block)].clone()]);
        }
        data.truncate(usize::try_from(size).context("D88 file size does not fit usize")?);
        files.push(D88File {
            index: files.len(),
            name,
            directory_offset: index * DIRECTORY_ENTRY_SIZE,
            start_block,
            size,
            block_chain,
            data,
            raw_entry,
        });
    }
    if files.is_empty() {
        bail!("D88 directory contains no files");
    }
    Ok(files)
}

fn decode_filename(raw: &[u8]) -> Result<String> {
    let stem = trim_spaces(&raw[..8]);
    let extension = trim_spaces(&raw[8..11]);
    if stem.is_empty() {
        bail!("D88 directory entry has an empty filename");
    }
    for byte in stem.iter().chain(extension) {
        if !(0x21..=0x7E).contains(byte) || matches!(*byte, b'/' | b'\\' | b':') {
            bail!("unsupported or unsafe byte 0x{byte:02X} in D88 filename");
        }
    }
    let stem = std::str::from_utf8(stem).context("D88 filename is not ASCII")?;
    let extension = std::str::from_utf8(extension).context("D88 extension is not ASCII")?;
    if extension.is_empty() {
        Ok(stem.to_owned())
    } else {
        Ok(format!("{stem}.{extension}"))
    }
}

fn trim_spaces(raw: &[u8]) -> &[u8] {
    let end = raw
        .iter()
        .rposition(|byte| *byte != b' ')
        .map_or(0, |index| index + 1);
    &raw[..end]
}

fn gather_ranges(bytes: &[u8], ranges: &[Range<usize>]) -> Vec<u8> {
    let mut output = Vec::with_capacity(ranges.iter().map(Range::len).sum());
    for range in ranges {
        output.extend_from_slice(&bytes[range.clone()]);
    }
    output
}

fn write_logical(
    bytes: &mut [u8],
    ranges: &[Range<usize>],
    logical_offset: usize,
    value: &[u8],
) -> Result<()> {
    let total: usize = ranges.iter().map(Range::len).sum();
    let end = logical_offset
        .checked_add(value.len())
        .context("logical write offset overflow")?;
    if end > total {
        bail!("logical write 0x{logical_offset:X}..0x{end:X} exceeds payload size 0x{total:X}");
    }
    let mut source_cursor = 0usize;
    let mut logical_cursor = 0usize;
    for range in ranges {
        let range_end = logical_cursor + range.len();
        if logical_offset < range_end && end > logical_cursor {
            let local_start = logical_offset.saturating_sub(logical_cursor);
            let local_end = (end - logical_cursor).min(range.len());
            let count = local_end - local_start;
            bytes[range.start + local_start..range.start + local_end]
                .copy_from_slice(&value[source_cursor..source_cursor + count]);
            source_cursor += count;
        }
        logical_cursor = range_end;
        if source_cursor == value.len() {
            break;
        }
    }
    if source_cursor != value.len() {
        bail!("logical write did not consume the complete value");
    }
    Ok(())
}

fn write_logical_u16(
    bytes: &mut [u8],
    ranges: &[Range<usize>],
    offset: usize,
    value: u16,
) -> Result<()> {
    write_logical(bytes, ranges, offset, &value.to_le_bytes())
}

fn write_logical_u32(
    bytes: &mut [u8],
    ranges: &[Range<usize>],
    offset: usize,
    value: u32,
) -> Result<()> {
    write_logical(bytes, ranges, offset, &value.to_le_bytes())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let raw = bytes
        .get(offset..offset + 2)
        .with_context(|| format!("truncated u16 at 0x{offset:X}"))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let raw = bytes
        .get(offset..offset + 4)
        .with_context(|| format!("truncated u32 at 0x{offset:X}"))?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_sparse_track_table() {
        let mut bytes = vec![0u8; D88_HEADER_SIZE + 1];
        let size = bytes.len() as u32;
        bytes[0x1C..0x20].copy_from_slice(&size.to_le_bytes());
        bytes[TRACK_TABLE_OFFSET..TRACK_TABLE_OFFSET + 4]
            .copy_from_slice(&(D88_HEADER_SIZE as u32).to_le_bytes());
        bytes[TRACK_TABLE_OFFSET + 8..TRACK_TABLE_OFFSET + 12]
            .copy_from_slice(&(D88_HEADER_SIZE as u32).to_le_bytes());
        let error = D88ProjectImage::parse(bytes).unwrap_err().to_string();
        assert!(error.contains("nonzero entry after an empty entry"));
    }

    #[test]
    fn logical_write_crosses_sector_ranges() {
        let mut bytes = vec![0u8; 10];
        let ranges = vec![1..4, 6..9];
        write_logical(&mut bytes, &ranges, 2, &[1, 2, 3]).unwrap();
        assert_eq!(bytes, [0, 0, 0, 1, 0, 0, 2, 3, 0, 0]);
    }

    #[test]
    fn filename_decoding_preserves_83_layout() {
        assert_eq!(decode_filename(b"MSG1    ZIL").unwrap(), "MSG1.ZIL");
        assert_eq!(decode_filename(b"README     ").unwrap(), "README");
    }
}
