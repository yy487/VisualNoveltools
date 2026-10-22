use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub const FL4_MAGIC: &[u8; 8] = b"FL4.0\0\0\0";
pub const MIN_HEADER_SIZE: usize = 28;
pub const BUCKET_COUNT: usize = 512;
pub const HASH_MODULUS: u32 = 0x1fd;
pub const BUCKET_TABLE_SIZE: usize = BUCKET_COUNT * 6;
pub const MANIFEST_FILE: &str = "fl4_manifest.json";

pub type ToolResult<T> = Result<T, ToolError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolError(pub String);

impl fmt::Display for ToolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ToolError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fl4Header {
    pub raw: Vec<u8>,
    pub header_size: u16,
    pub index_size: u32,
    pub index_offset: u32,
    pub file_count: u32,
    pub key_length: u16,
    pub flags: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fl4Bucket {
    pub index: usize,
    pub records_offset: u32,
    pub entry_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fl4Entry {
    pub index: usize,
    pub bucket: usize,
    pub bucket_index: usize,
    pub record_offset: u32,
    pub name: String,
    pub name_bytes: Vec<u8>,
    pub relative_data_offset: u32,
    pub data_offset: u64,
    pub size: u32,
    pub data_order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fl4Archive {
    pub file_len: u64,
    pub header: Fl4Header,
    pub buckets: Vec<Fl4Bucket>,
    pub entries: Vec<Fl4Entry>,
    pub index_tail: Vec<u8>,
}

impl Fl4Archive {
    pub fn payload<'a>(&self, bytes: &'a [u8], entry: &Fl4Entry) -> ToolResult<&'a [u8]> {
        let start = usize::try_from(entry.data_offset)
            .map_err(|_| ToolError(format!("entry {} offset does not fit usize", entry.index)))?;
        let size = usize::try_from(entry.size)
            .map_err(|_| ToolError(format!("entry {} size does not fit usize", entry.index)))?;
        let end = start
            .checked_add(size)
            .ok_or_else(|| ToolError(format!("entry {} payload offset overflow", entry.index)))?;
        bytes.get(start..end).ok_or_else(|| {
            ToolError(format!(
                "entry {} payload [0x{start:x}, 0x{end:x}) is outside archive",
                entry.index
            ))
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: String,
    pub source_archive: String,
    pub file_size: u64,
    pub magic: String,
    pub header_size: u16,
    pub index_size: u32,
    pub index_offset: u32,
    pub file_count: u32,
    pub data_offset: u16,
    pub data_size: u64,
    pub bucket_count: usize,
    pub hash_modulus: u32,
    pub key_length: u16,
    pub flags: u16,
    pub header_hex: String,
    pub index_tail_hex: String,
    pub buckets: Vec<ManifestBucket>,
    pub entries: Vec<ManifestEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestBucket {
    pub index: usize,
    pub records_offset: u32,
    pub entry_count: u16,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestEntry {
    pub index: usize,
    pub bucket: usize,
    pub bucket_index: usize,
    pub record_offset: u32,
    pub data_order: usize,
    pub name: String,
    pub name_bytes_hex: String,
    pub relative_data_offset: u32,
    pub data_offset: u64,
    pub size: u32,
    pub output_path: String,
}

#[derive(Debug)]
pub struct UnpackReport {
    pub input: PathBuf,
    pub output: PathBuf,
    pub manifest: PathBuf,
    pub extracted_files: usize,
    pub extracted_bytes: u64,
}

#[derive(Debug)]
pub struct PackInputInfo {
    pub root: PathBuf,
    pub manifest: PathBuf,
    pub entries: usize,
    pub original_file_size: u64,
}

#[derive(Debug)]
pub struct PackReport {
    pub input: PathBuf,
    pub manifest: PathBuf,
    pub output: PathBuf,
    pub packed_files: usize,
    pub output_bytes: u64,
}

#[derive(Debug)]
struct ValidatedManifestEntry {
    name_bytes: Vec<u8>,
    relative_path: PathBuf,
}

#[derive(Debug)]
struct ValidatedManifest {
    header: Vec<u8>,
    index_tail: Vec<u8>,
    entries: Vec<ValidatedManifestEntry>,
    bucket_entries: Vec<Vec<usize>>,
    data_order: Vec<usize>,
}

fn read_u16(data: &[u8], offset: usize, label: &str) -> ToolResult<u16> {
    let end = offset
        .checked_add(2)
        .ok_or_else(|| ToolError(format!("{label} offset overflow")))?;
    let raw = data
        .get(offset..end)
        .ok_or_else(|| ToolError(format!("{label} is truncated at 0x{offset:x}")))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(data: &[u8], offset: usize, label: &str) -> ToolResult<u32> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| ToolError(format!("{label} offset overflow")))?;
    let raw = data
        .get(offset..end)
        .ok_or_else(|| ToolError(format!("{label} is truncated at 0x{offset:x}")))?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn checked_end(start: u64, size: u64, label: &str) -> ToolResult<u64> {
    start
        .checked_add(size)
        .ok_or_else(|| ToolError(format!("{label} range overflows u64")))
}

fn decode_name(raw: &[u8], index: usize) -> ToolResult<String> {
    if raw.is_empty() {
        return Err(ToolError(format!("entry {index} has an empty name")));
    }
    if raw.contains(&0) {
        return Err(ToolError(format!("entry {index} name contains NUL")));
    }
    let (decoded, _, had_errors) = SHIFT_JIS.decode(raw);
    if had_errors {
        return Err(ToolError(format!(
            "entry {index} name is not valid CP932/Shift-JIS: {}",
            hex(raw)
        )));
    }
    let name = decoded.into_owned();
    let (encoded, _, encode_errors) = SHIFT_JIS.encode(&name);
    if encode_errors || encoded.as_ref() != raw {
        return Err(ToolError(format!(
            "entry {index} name failed CP932 byte round-trip: {}",
            hex(raw)
        )));
    }
    Ok(name)
}

/// Return the bucket selected by the original runtime for an archive name.
pub fn runtime_name_hash(name_bytes: &[u8]) -> u32 {
    let basename = name_bytes
        .iter()
        .rposition(|byte| *byte == b'\\')
        .map(|index| &name_bytes[index + 1..])
        .unwrap_or(name_bytes);
    basename.iter().fold(0u32, |hash, byte| {
        let normalized = if byte.is_ascii_lowercase() {
            byte.to_ascii_uppercase()
        } else {
            *byte
        };
        (u32::from(normalized) + (hash << 8)) % HASH_MODULUS
    })
}

/// Parse and validate the FL4.0 header, hash index, names, and payload boundaries.
pub fn parse_archive(bytes: &[u8]) -> ToolResult<Fl4Archive> {
    if bytes.len() < MIN_HEADER_SIZE {
        return Err(ToolError(format!(
            "archive is too short for the 28-byte FL4 header: {} bytes",
            bytes.len()
        )));
    }
    if &bytes[..FL4_MAGIC.len()] != FL4_MAGIC {
        return Err(ToolError(format!(
            "unsupported archive magic: {}",
            hex(&bytes[..FL4_MAGIC.len()])
        )));
    }

    let header_size = read_u16(bytes, 8, "header_size")?;
    let index_size = read_u32(bytes, 10, "index_size")?;
    let index_offset = read_u32(bytes, 14, "index_offset")?;
    let file_count = read_u32(bytes, 18, "file_count")?;
    let key_length = read_u16(bytes, 22, "key_length")?;
    let flags = read_u16(bytes, 24, "flags")?;

    if usize::from(header_size) < MIN_HEADER_SIZE {
        return Err(ToolError(format!(
            "header_size {header_size} is smaller than the known 28-byte header"
        )));
    }
    if usize::from(header_size) > bytes.len() {
        return Err(ToolError(format!(
            "header_size {header_size} exceeds archive length {}",
            bytes.len()
        )));
    }
    if flags & 1 != 0 {
        return Err(ToolError(format!(
            "encrypted FL4 index is not supported (flags=0x{flags:04x}, key_length={key_length})"
        )));
    }

    let index_start = u64::from(index_offset);
    let index_end = checked_end(index_start, u64::from(index_size), "index")?;
    if index_start < u64::from(header_size) {
        return Err(ToolError(format!(
            "index_offset 0x{index_offset:x} is before header end 0x{header_size:x}"
        )));
    }
    if index_end > bytes.len() as u64 {
        return Err(ToolError(format!(
            "index [0x{index_offset:x}, 0x{index_end:x}) exceeds archive length {}",
            bytes.len()
        )));
    }
    if index_end != bytes.len() as u64 {
        return Err(ToolError(format!(
            "index ends at 0x{index_end:x}, but archive ends at 0x{:x}; trailing bytes are not understood",
            bytes.len()
        )));
    }

    let index_size_usize = usize::try_from(index_size)
        .map_err(|_| ToolError("index_size does not fit usize".to_string()))?;
    let minimum_index_size = BUCKET_TABLE_SIZE
        .checked_add(4)
        .ok_or_else(|| ToolError("minimum index size overflow".to_string()))?;
    if index_size_usize < minimum_index_size {
        return Err(ToolError(format!(
            "index_size {index_size} is smaller than the bucket table and terminator"
        )));
    }
    let index_start_usize = usize::try_from(index_start)
        .map_err(|_| ToolError("index_offset does not fit usize".to_string()))?;
    let records_end = index_size_usize - 4;
    let tail_start = index_start_usize + records_end;
    let index_tail = bytes[tail_start..tail_start + 4].to_vec();
    if index_tail != [0xff; 4] {
        return Err(ToolError(format!(
            "unsupported FL4 index terminator: {}",
            hex(&index_tail)
        )));
    }

    let mut buckets = Vec::with_capacity(BUCKET_COUNT);
    for bucket_index in 0..BUCKET_COUNT {
        let descriptor = index_start_usize + bucket_index * 6;
        let records_offset = read_u32(
            bytes,
            descriptor,
            &format!("bucket {bucket_index} records_offset"),
        )?;
        let entry_count = read_u16(
            bytes,
            descriptor + 4,
            &format!("bucket {bucket_index} entry_count"),
        )?;
        let records_offset_usize = usize::try_from(records_offset)
            .map_err(|_| ToolError(format!("bucket {bucket_index} offset does not fit usize")))?;
        if !(BUCKET_TABLE_SIZE..=records_end).contains(&records_offset_usize) {
            return Err(ToolError(format!(
                "bucket {bucket_index} records_offset 0x{records_offset:x} is outside the record area"
            )));
        }
        buckets.push(Fl4Bucket {
            index: bucket_index,
            records_offset,
            entry_count,
        });
    }
    if buckets[0].records_offset != BUCKET_TABLE_SIZE as u32 {
        return Err(ToolError(format!(
            "first bucket starts at 0x{:x}, expected 0x{BUCKET_TABLE_SIZE:x}",
            buckets[0].records_offset
        )));
    }

    let expected_count = usize::try_from(file_count)
        .map_err(|_| ToolError("file_count does not fit usize".to_string()))?;
    let bucket_entry_count = buckets.iter().try_fold(0usize, |total, bucket| {
        total
            .checked_add(usize::from(bucket.entry_count))
            .ok_or_else(|| ToolError("bucket entry count overflow".to_string()))
    })?;
    if bucket_entry_count != expected_count {
        return Err(ToolError(format!(
            "bucket counts total {bucket_entry_count}, header declares {file_count} files"
        )));
    }

    let mut entries = Vec::with_capacity(expected_count);
    let mut destination_keys = HashSet::with_capacity(expected_count);
    for bucket_index in 0..BUCKET_COUNT {
        let bucket = &buckets[bucket_index];
        let mut cursor = usize::try_from(bucket.records_offset)
            .map_err(|_| ToolError(format!("bucket {bucket_index} offset does not fit usize")))?;
        let expected_end = if bucket_index + 1 < BUCKET_COUNT {
            usize::try_from(buckets[bucket_index + 1].records_offset).map_err(|_| {
                ToolError(format!(
                    "bucket {} offset does not fit usize",
                    bucket_index + 1
                ))
            })?
        } else {
            records_end
        };

        for bucket_entry_index in 0..usize::from(bucket.entry_count) {
            let record_offset = cursor;
            let absolute_record = index_start_usize.checked_add(cursor).ok_or_else(|| {
                ToolError(format!("bucket {bucket_index} record offset overflow"))
            })?;
            let relative_data_offset = read_u32(
                bytes,
                absolute_record,
                &format!("entry {} relative_data_offset", entries.len()),
            )?;
            let size = read_u32(
                bytes,
                absolute_record + 4,
                &format!("entry {} size", entries.len()),
            )?;
            let name_len = *bytes.get(absolute_record + 8).ok_or_else(|| {
                ToolError(format!("entry {} name length is truncated", entries.len()))
            })? as usize;
            let name_start = absolute_record + 9;
            let name_end = name_start.checked_add(name_len).ok_or_else(|| {
                ToolError(format!("entry {} name length overflows", entries.len()))
            })?;
            let absolute_expected_end = index_start_usize
                .checked_add(expected_end)
                .ok_or_else(|| ToolError(format!("bucket {bucket_index} end offset overflow")))?;
            if name_end > absolute_expected_end {
                return Err(ToolError(format!(
                    "entry {} name crosses bucket {bucket_index} boundary",
                    entries.len()
                )));
            }
            let name_bytes = bytes[name_start..name_end].to_vec();
            let name = decode_name(&name_bytes, entries.len())?;
            let (_, destination_key) = safe_relative_path(&name)?;
            if destination_key == MANIFEST_FILE.to_ascii_uppercase() {
                return Err(ToolError(format!(
                    "entry {} conflicts with reserved manifest name {MANIFEST_FILE}",
                    entries.len()
                )));
            }
            if !destination_keys.insert(destination_key) {
                return Err(ToolError(format!(
                    "duplicate Windows output path for entry {}: {name}",
                    entries.len()
                )));
            }
            let actual_bucket = runtime_name_hash(&name_bytes);
            if actual_bucket != bucket_index as u32 {
                return Err(ToolError(format!(
                    "entry {} name {name:?} hashes to bucket {actual_bucket}, stored in bucket {bucket_index}",
                    entries.len()
                )));
            }
            cursor = cursor.checked_add(9 + name_len).ok_or_else(|| {
                ToolError(format!("entry {} record length overflows", entries.len()))
            })?;
            entries.push(Fl4Entry {
                index: entries.len(),
                bucket: bucket_index,
                bucket_index: bucket_entry_index,
                record_offset: u32::try_from(record_offset)
                    .map_err(|_| ToolError("record offset exceeds u32".to_string()))?,
                name,
                name_bytes,
                relative_data_offset,
                data_offset: 0,
                size,
                data_order: 0,
            });
        }
        if cursor != expected_end {
            return Err(ToolError(format!(
                "bucket {bucket_index} records end at 0x{cursor:x}, next boundary is 0x{expected_end:x}"
            )));
        }
    }
    if entries.len() != expected_count {
        return Err(ToolError(format!(
            "parsed {} entries, header declares {file_count}",
            entries.len()
        )));
    }

    let data_size = index_start
        .checked_sub(u64::from(header_size))
        .ok_or_else(|| ToolError("data range underflow".to_string()))?;
    let mut data_order = (0..entries.len()).collect::<Vec<_>>();
    data_order.sort_by_key(|entry_index| {
        (
            entries[*entry_index].relative_data_offset,
            entries[*entry_index].index,
        )
    });
    let mut data_cursor = 0u64;
    for (order, entry_index) in data_order.into_iter().enumerate() {
        let entry = &mut entries[entry_index];
        let relative_offset = u64::from(entry.relative_data_offset);
        if relative_offset != data_cursor {
            let relation = if relative_offset < data_cursor {
                "overlaps the preceding payload"
            } else {
                "leaves an unexplained gap before the payload"
            };
            return Err(ToolError(format!(
                "entry {} ({:?}) starts at relative 0x{relative_offset:x}, expected 0x{data_cursor:x}; {relation}",
                entry.index, entry.name
            )));
        }
        entry.data_offset = u64::from(header_size)
            .checked_add(relative_offset)
            .ok_or_else(|| ToolError(format!("entry {} absolute offset overflows", entry.index)))?;
        entry.data_order = order;
        data_cursor = checked_end(data_cursor, u64::from(entry.size), "payload data")?;
        if data_cursor > data_size {
            return Err(ToolError(format!(
                "entry {} ({:?}) payload extends beyond index start",
                entry.index, entry.name
            )));
        }
    }
    if data_cursor != data_size {
        return Err(ToolError(format!(
            "payloads end at relative 0x{data_cursor:x}, data area ends at 0x{data_size:x}"
        )));
    }

    Ok(Fl4Archive {
        file_len: bytes.len() as u64,
        header: Fl4Header {
            raw: bytes[..usize::from(header_size)].to_vec(),
            header_size,
            index_size,
            index_offset,
            file_count,
            key_length,
            flags,
        },
        buckets,
        entries,
        index_tail,
    })
}

fn reserved_windows_name(component: &str) -> bool {
    let stem = component
        .split('.')
        .next()
        .unwrap_or(component)
        .trim_end_matches([' ', '.'])
        .to_ascii_uppercase();
    matches!(
        stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

pub fn safe_relative_path(name: &str) -> ToolResult<(PathBuf, String)> {
    let mut path = PathBuf::new();
    let mut key_parts = Vec::new();
    for component in name.split(['/', '\\']) {
        if component.is_empty() || component == "." || component == ".." {
            return Err(ToolError(format!("unsafe archive path: {name:?}")));
        }
        if component.ends_with([' ', '.']) {
            return Err(ToolError(format!(
                "archive path has a trailing dot or space: {name:?}"
            )));
        }
        if component.chars().any(|character| {
            character == ':'
                || character.is_control()
                || matches!(character, '<' | '>' | '"' | '|' | '?' | '*')
        }) {
            return Err(ToolError(format!(
                "archive path has unsafe characters: {name:?}"
            )));
        }
        if reserved_windows_name(component) {
            return Err(ToolError(format!(
                "archive path uses a reserved Windows name: {name:?}"
            )));
        }
        path.push(component);
        key_parts.push(component.to_uppercase());
    }
    if key_parts.is_empty() {
        return Err(ToolError(format!("archive path is empty: {name:?}")));
    }
    Ok((path, key_parts.join("/")))
}

pub fn default_output_path(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "archive".to_string());
    input.with_file_name(format!("{stem}_unpacked"))
}

pub fn default_packed_output_path(input_root: &Path) -> PathBuf {
    let name = input_root
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "workspace".to_string());
    input_root.with_file_name(format!("{name}_packed.FL4"))
}

fn path_to_manifest(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn build_manifest(
    input: &Path,
    archive: &Fl4Archive,
    paths: &[PathBuf],
) -> ToolResult<Manifest> {
    if paths.len() != archive.entries.len() {
        return Err(ToolError(
            "manifest path count does not match entry count".to_string(),
        ));
    }
    let data_size = u64::from(archive.header.index_offset)
        .checked_sub(u64::from(archive.header.header_size))
        .ok_or_else(|| ToolError("manifest data size underflow".to_string()))?;
    Ok(Manifest {
        format: "KOKOROV FL4.0".to_string(),
        source_archive: input
            .file_name()
            .unwrap_or(input.as_os_str())
            .to_string_lossy()
            .into_owned(),
        file_size: archive.file_len,
        magic: "FL4.0\\0\\0\\0".to_string(),
        header_size: archive.header.header_size,
        index_size: archive.header.index_size,
        index_offset: archive.header.index_offset,
        file_count: archive.header.file_count,
        data_offset: archive.header.header_size,
        data_size,
        bucket_count: BUCKET_COUNT,
        hash_modulus: HASH_MODULUS,
        key_length: archive.header.key_length,
        flags: archive.header.flags,
        header_hex: hex(&archive.header.raw),
        index_tail_hex: hex(&archive.index_tail),
        buckets: archive
            .buckets
            .iter()
            .map(|bucket| ManifestBucket {
                index: bucket.index,
                records_offset: bucket.records_offset,
                entry_count: bucket.entry_count,
            })
            .collect(),
        entries: archive
            .entries
            .iter()
            .zip(paths)
            .map(|(entry, path)| ManifestEntry {
                index: entry.index,
                bucket: entry.bucket,
                bucket_index: entry.bucket_index,
                record_offset: entry.record_offset,
                data_order: entry.data_order,
                name: entry.name.clone(),
                name_bytes_hex: hex(&entry.name_bytes),
                relative_data_offset: entry.relative_data_offset,
                data_offset: entry.data_offset,
                size: entry.size,
                output_path: path_to_manifest(path),
            })
            .collect(),
    })
}

fn decode_hex(text: &str, label: &str) -> ToolResult<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return Err(ToolError(format!(
            "{label} has an odd number of hex digits"
        )));
    }
    let mut output = Vec::with_capacity(text.len() / 2);
    let bytes = text.as_bytes();
    for offset in (0..bytes.len()).step_by(2) {
        let high = char::from(bytes[offset])
            .to_digit(16)
            .ok_or_else(|| ToolError(format!("{label} has invalid hex at character {offset}")))?;
        let low = char::from(bytes[offset + 1]).to_digit(16).ok_or_else(|| {
            ToolError(format!(
                "{label} has invalid hex at character {}",
                offset + 1
            ))
        })?;
        output.push(((high << 4) | low) as u8);
    }
    Ok(output)
}

fn validate_pack_manifest(manifest: &Manifest) -> ToolResult<ValidatedManifest> {
    if manifest.format != "KOKOROV FL4.0" {
        return Err(ToolError(format!(
            "unsupported manifest format {:?}",
            manifest.format
        )));
    }
    if manifest.magic != "FL4.0\\0\\0\\0" {
        return Err(ToolError(format!(
            "unsupported manifest magic {:?}",
            manifest.magic
        )));
    }
    if manifest.source_archive.is_empty()
        || Path::new(&manifest.source_archive).file_name()
            != Some(Path::new(&manifest.source_archive).as_os_str())
    {
        return Err(ToolError(format!(
            "manifest source_archive must be a single file name: {:?}",
            manifest.source_archive
        )));
    }
    if usize::from(manifest.header_size) < MIN_HEADER_SIZE {
        return Err(ToolError(format!(
            "manifest header_size {} is smaller than {MIN_HEADER_SIZE}",
            manifest.header_size
        )));
    }
    if manifest.data_offset != manifest.header_size {
        return Err(ToolError(format!(
            "manifest data_offset {} does not match header_size {}",
            manifest.data_offset, manifest.header_size
        )));
    }
    if manifest.bucket_count != BUCKET_COUNT || manifest.buckets.len() != BUCKET_COUNT {
        return Err(ToolError(format!(
            "manifest must contain {BUCKET_COUNT} buckets, declares {} and stores {}",
            manifest.bucket_count,
            manifest.buckets.len()
        )));
    }
    if manifest.hash_modulus != HASH_MODULUS {
        return Err(ToolError(format!(
            "manifest hash_modulus 0x{:x} is unsupported",
            manifest.hash_modulus
        )));
    }
    if manifest.flags & 1 != 0 {
        return Err(ToolError(format!(
            "cannot pack an encrypted FL4 index (flags=0x{:04x})",
            manifest.flags
        )));
    }

    let header = decode_hex(&manifest.header_hex, "header_hex")?;
    if header.len() != usize::from(manifest.header_size) {
        return Err(ToolError(format!(
            "header_hex is {} bytes, header_size is {}",
            header.len(),
            manifest.header_size
        )));
    }
    if header.get(..FL4_MAGIC.len()) != Some(FL4_MAGIC.as_slice()) {
        return Err(ToolError(
            "header_hex does not contain FL4.0 magic".to_string(),
        ));
    }
    let header_fields = [
        (
            u64::from(read_u16(&header, 8, "header header_size")?),
            u64::from(manifest.header_size),
            "header_size",
        ),
        (
            u64::from(read_u32(&header, 10, "header index_size")?),
            u64::from(manifest.index_size),
            "index_size",
        ),
        (
            u64::from(read_u32(&header, 14, "header index_offset")?),
            u64::from(manifest.index_offset),
            "index_offset",
        ),
        (
            u64::from(read_u32(&header, 18, "header file_count")?),
            u64::from(manifest.file_count),
            "file_count",
        ),
        (
            u64::from(read_u16(&header, 22, "header key_length")?),
            u64::from(manifest.key_length),
            "key_length",
        ),
        (
            u64::from(read_u16(&header, 24, "header flags")?),
            u64::from(manifest.flags),
            "flags",
        ),
    ];
    for (header_value, manifest_value, label) in header_fields {
        if header_value != manifest_value {
            return Err(ToolError(format!(
                "header_hex {label} {header_value} does not match manifest value {manifest_value}"
            )));
        }
    }

    let index_tail = decode_hex(&manifest.index_tail_hex, "index_tail_hex")?;
    if index_tail != [0xff; 4] {
        return Err(ToolError(format!(
            "unsupported manifest index terminator: {}",
            manifest.index_tail_hex
        )));
    }
    let expected_count = usize::try_from(manifest.file_count)
        .map_err(|_| ToolError("manifest file_count does not fit usize".to_string()))?;
    if manifest.entries.len() != expected_count {
        return Err(ToolError(format!(
            "manifest stores {} entries, file_count is {}",
            manifest.entries.len(),
            manifest.file_count
        )));
    }

    let mut entries = Vec::with_capacity(expected_count);
    let mut bucket_entries = vec![Vec::new(); BUCKET_COUNT];
    let mut data_order = vec![usize::MAX; expected_count];
    let mut destination_keys = HashSet::with_capacity(expected_count);
    for (entry_index, entry) in manifest.entries.iter().enumerate() {
        if entry.index != entry_index {
            return Err(ToolError(format!(
                "manifest entry array position {entry_index} has index {}",
                entry.index
            )));
        }
        if entry.bucket >= BUCKET_COUNT {
            return Err(ToolError(format!(
                "manifest entry {entry_index} bucket {} is outside 0..{BUCKET_COUNT}",
                entry.bucket
            )));
        }
        if entry.data_order >= expected_count || data_order[entry.data_order] != usize::MAX {
            return Err(ToolError(format!(
                "manifest entry {entry_index} has invalid or duplicate data_order {}",
                entry.data_order
            )));
        }
        data_order[entry.data_order] = entry_index;
        if entry.bucket_index != bucket_entries[entry.bucket].len() {
            return Err(ToolError(format!(
                "manifest entry {entry_index} has bucket_index {}, expected {} in bucket {}",
                entry.bucket_index,
                bucket_entries[entry.bucket].len(),
                entry.bucket
            )));
        }

        let name_bytes = decode_hex(
            &entry.name_bytes_hex,
            &format!("manifest entry {entry_index} name_bytes_hex"),
        )?;
        if name_bytes.len() > u8::MAX as usize {
            return Err(ToolError(format!(
                "manifest entry {entry_index} name is {} bytes, exceeds u8",
                name_bytes.len()
            )));
        }
        let decoded_name = decode_name(&name_bytes, entry_index)?;
        if decoded_name != entry.name {
            return Err(ToolError(format!(
                "manifest entry {entry_index} name {:?} does not match original bytes {:?}",
                entry.name, decoded_name
            )));
        }
        let actual_bucket = runtime_name_hash(&name_bytes) as usize;
        if actual_bucket != entry.bucket {
            return Err(ToolError(format!(
                "manifest entry {entry_index} name {:?} hashes to bucket {actual_bucket}, records bucket {}",
                entry.name, entry.bucket
            )));
        }
        let (relative_path, destination_key) = safe_relative_path(&entry.name)?;
        if !destination_keys.insert(destination_key) {
            return Err(ToolError(format!(
                "duplicate Windows output path in manifest entry {entry_index}: {:?}",
                entry.name
            )));
        }
        let expected_output_path = path_to_manifest(&relative_path);
        if entry.output_path != expected_output_path {
            return Err(ToolError(format!(
                "manifest entry {entry_index} output_path {:?} does not match name-derived path {:?}",
                entry.output_path, expected_output_path
            )));
        }
        bucket_entries[entry.bucket].push(entry_index);
        entries.push(ValidatedManifestEntry {
            name_bytes,
            relative_path,
        });
    }
    if data_order.contains(&usize::MAX) {
        return Err(ToolError(
            "manifest data_order is not a complete permutation".to_string(),
        ));
    }

    let mut record_cursor = BUCKET_TABLE_SIZE;
    for (bucket_index, (bucket, bucket_entry_indices)) in
        manifest.buckets.iter().zip(&bucket_entries).enumerate()
    {
        if bucket.index != bucket_index {
            return Err(ToolError(format!(
                "manifest bucket array position {bucket_index} has index {}",
                bucket.index
            )));
        }
        if usize::try_from(bucket.records_offset).ok() != Some(record_cursor) {
            return Err(ToolError(format!(
                "manifest bucket {bucket_index} records_offset 0x{:x}, expected 0x{record_cursor:x}",
                bucket.records_offset
            )));
        }
        if usize::from(bucket.entry_count) != bucket_entry_indices.len() {
            return Err(ToolError(format!(
                "manifest bucket {bucket_index} declares {} entries, records {}",
                bucket.entry_count,
                bucket_entry_indices.len()
            )));
        }
        for entry_index in bucket_entry_indices {
            let entry = &manifest.entries[*entry_index];
            if usize::try_from(entry.record_offset).ok() != Some(record_cursor) {
                return Err(ToolError(format!(
                    "manifest entry {entry_index} record_offset 0x{:x}, expected 0x{record_cursor:x}",
                    entry.record_offset
                )));
            }
            record_cursor = record_cursor
                .checked_add(9 + entries[*entry_index].name_bytes.len())
                .ok_or_else(|| ToolError("manifest record area size overflow".to_string()))?;
        }
    }
    let rebuilt_index_size = record_cursor
        .checked_add(index_tail.len())
        .ok_or_else(|| ToolError("manifest index size overflow".to_string()))?;
    if u32::try_from(rebuilt_index_size).ok() != Some(manifest.index_size) {
        return Err(ToolError(format!(
            "manifest index_size {} does not match records size {rebuilt_index_size}",
            manifest.index_size
        )));
    }

    let mut data_cursor = 0u64;
    for entry_index in &data_order {
        let entry = &manifest.entries[*entry_index];
        if u64::from(entry.relative_data_offset) != data_cursor {
            return Err(ToolError(format!(
                "manifest entry {entry_index} relative_data_offset 0x{:x}, expected 0x{data_cursor:x}",
                entry.relative_data_offset
            )));
        }
        let expected_absolute = u64::from(manifest.header_size)
            .checked_add(data_cursor)
            .ok_or_else(|| ToolError("manifest absolute payload offset overflow".to_string()))?;
        if entry.data_offset != expected_absolute {
            return Err(ToolError(format!(
                "manifest entry {entry_index} data_offset 0x{:x}, expected 0x{expected_absolute:x}",
                entry.data_offset
            )));
        }
        data_cursor = checked_end(data_cursor, u64::from(entry.size), "manifest payload data")?;
    }
    if data_cursor != manifest.data_size {
        return Err(ToolError(format!(
            "manifest payload sizes total {data_cursor}, data_size is {}",
            manifest.data_size
        )));
    }
    let expected_index_offset = u64::from(manifest.header_size)
        .checked_add(data_cursor)
        .ok_or_else(|| ToolError("manifest index offset overflow".to_string()))?;
    if expected_index_offset != u64::from(manifest.index_offset) {
        return Err(ToolError(format!(
            "manifest index_offset {} does not match header plus data size {expected_index_offset}",
            manifest.index_offset
        )));
    }
    let expected_file_size = expected_index_offset
        .checked_add(u64::from(manifest.index_size))
        .ok_or_else(|| ToolError("manifest file size overflow".to_string()))?;
    if expected_file_size != manifest.file_size {
        return Err(ToolError(format!(
            "manifest file_size {} does not match calculated size {expected_file_size}",
            manifest.file_size
        )));
    }

    Ok(ValidatedManifest {
        header,
        index_tail,
        entries,
        bucket_entries,
        data_order,
    })
}

fn resolve_pack_input(input: &Path) -> ToolResult<(PathBuf, PathBuf)> {
    if input.is_dir() {
        return Ok((input.to_path_buf(), input.join(MANIFEST_FILE)));
    }
    if input.is_file() {
        let root = input
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        return Ok((root, input.to_path_buf()));
    }
    Err(ToolError(format!(
        "pack input is neither a directory nor a manifest file: '{}'",
        input.display()
    )))
}

fn read_validated_pack_input(
    input: &Path,
) -> ToolResult<(PathBuf, PathBuf, Manifest, ValidatedManifest)> {
    let (root, manifest_path) = resolve_pack_input(input)?;
    let manifest_bytes = fs::read(&manifest_path).map_err(|error| {
        ToolError(format!(
            "cannot read manifest '{}': {error}",
            manifest_path.display()
        ))
    })?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes).map_err(|error| {
        ToolError(format!(
            "cannot parse UTF-8 manifest '{}': {error}",
            manifest_path.display()
        ))
    })?;
    let validated = validate_pack_manifest(&manifest)?;
    Ok((root, manifest_path, manifest, validated))
}

fn validate_payload_files(root: &Path, validated: &ValidatedManifest) -> ToolResult<()> {
    for (entry_index, entry) in validated.entries.iter().enumerate() {
        let payload_path = root.join(&entry.relative_path);
        let metadata = fs::symlink_metadata(&payload_path).map_err(|error| {
            ToolError(format!(
                "manifest entry {entry_index} payload is missing '{}': {error}",
                payload_path.display()
            ))
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(ToolError(format!(
                "manifest entry {entry_index} payload is not a regular file: '{}'",
                payload_path.display()
            )));
        }
    }
    Ok(())
}

pub fn inspect_pack_input(input: &Path) -> ToolResult<PackInputInfo> {
    let (root, manifest_path, manifest, validated) = read_validated_pack_input(input)?;
    validate_payload_files(&root, &validated)?;
    Ok(PackInputInfo {
        root,
        manifest: manifest_path,
        entries: validated.entries.len(),
        original_file_size: manifest.file_size,
    })
}

fn unique_sibling(path: &Path, role: &str) -> ToolResult<PathBuf> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .ok_or_else(|| {
            ToolError(format!(
                "output must have a file or directory name: '{}'",
                path.display()
            ))
        })?
        .to_string_lossy();
    for attempt in 0..1000u32 {
        let candidate = parent.join(format!(
            ".{name}.fl4-{role}-{}-{attempt}",
            std::process::id()
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(ToolError(format!(
        "cannot allocate a temporary sibling for '{}'",
        path.display()
    )))
}

fn validate_output_target(input: &Path, output: &Path) -> ToolResult<()> {
    if output.file_name().is_none() {
        return Err(ToolError(format!(
            "output must be a named directory, not a filesystem root: '{}'",
            output.display()
        )));
    }
    if output.exists() {
        let metadata = fs::symlink_metadata(output).map_err(|error| {
            ToolError(format!(
                "cannot inspect output '{}': {error}",
                output.display()
            ))
        })?;
        if metadata.file_type().is_symlink() {
            return Err(ToolError(format!(
                "refusing to replace symlinked output directory '{}'",
                output.display()
            )));
        }
        if !metadata.is_dir() {
            return Err(ToolError(format!(
                "output path exists and is not a directory: '{}'",
                output.display()
            )));
        }
        let canonical_input = fs::canonicalize(input).map_err(|error| {
            ToolError(format!(
                "cannot resolve input '{}': {error}",
                input.display()
            ))
        })?;
        let canonical_output = fs::canonicalize(output).map_err(|error| {
            ToolError(format!(
                "cannot resolve output '{}': {error}",
                output.display()
            ))
        })?;
        if canonical_input.starts_with(&canonical_output) {
            return Err(ToolError(format!(
                "output directory '{}' contains the input archive and cannot be replaced",
                output.display()
            )));
        }
    }
    Ok(())
}

fn finalize_staging(staging: &Path, output: &Path, overwrite: bool) -> ToolResult<()> {
    if !output.exists() {
        return fs::rename(staging, output).map_err(|error| {
            ToolError(format!(
                "cannot move completed output '{}' to '{}': {error}",
                staging.display(),
                output.display()
            ))
        });
    }
    if !overwrite {
        return Err(ToolError(format!(
            "refusing to overwrite existing output directory '{}'; pass --overwrite",
            output.display()
        )));
    }
    let backup = unique_sibling(output, "backup")?;
    fs::rename(output, &backup).map_err(|error| {
        ToolError(format!(
            "cannot move existing output '{}' aside: {error}",
            output.display()
        ))
    })?;
    if let Err(error) = fs::rename(staging, output) {
        let restore_error = fs::rename(&backup, output).err();
        return Err(ToolError(match restore_error {
            Some(restore) => format!(
                "cannot install completed output '{}': {error}; restoring '{}' also failed: {restore}",
                output.display(),
                backup.display()
            ),
            None => format!(
                "cannot install completed output '{}': {error}; original output was restored",
                output.display()
            ),
        }));
    }
    fs::remove_dir_all(&backup).map_err(|error| {
        ToolError(format!(
            "output was installed, but old backup '{}' could not be removed: {error}",
            backup.display()
        ))
    })
}

/// Extract a validated FL4 archive into a transactionally prepared directory.
pub fn unpack_archive(
    input: &Path,
    output_arg: Option<&Path>,
    overwrite: bool,
) -> ToolResult<UnpackReport> {
    if !input.is_file() {
        return Err(ToolError(format!(
            "input is not an archive file: '{}'",
            input.display()
        )));
    }
    let bytes = fs::read(input)
        .map_err(|error| ToolError(format!("cannot read '{}': {error}", input.display())))?;
    let archive = parse_archive(&bytes)?;
    let output = output_arg
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_path(input));
    validate_output_target(input, &output)?;
    if output.exists() && !overwrite {
        return Err(ToolError(format!(
            "refusing to overwrite existing output directory '{}'; pass --overwrite",
            output.display()
        )));
    }

    let mut destinations = Vec::with_capacity(archive.entries.len());
    for entry in &archive.entries {
        let (relative, _) = safe_relative_path(&entry.name)?;
        archive.payload(&bytes, entry)?;
        destinations.push(relative);
    }
    let manifest = build_manifest(input, &archive, &destinations)?;
    let mut manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| ToolError(format!("cannot serialize manifest: {error}")))?;
    manifest_bytes.push(b'\n');

    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| ToolError(format!("cannot create '{}': {error}", parent.display())))?;
    let staging = unique_sibling(&output, "staging")?;
    fs::create_dir(&staging)
        .map_err(|error| ToolError(format!("cannot create '{}': {error}", staging.display())))?;

    let write_result = (|| -> ToolResult<()> {
        for (entry, relative) in archive.entries.iter().zip(&destinations) {
            let target = staging.join(relative);
            if let Some(target_parent) = target.parent() {
                fs::create_dir_all(target_parent).map_err(|error| {
                    ToolError(format!(
                        "cannot create '{}': {error}",
                        target_parent.display()
                    ))
                })?;
            }
            fs::write(&target, archive.payload(&bytes, entry)?).map_err(|error| {
                ToolError(format!("cannot write '{}': {error}", target.display()))
            })?;
        }
        let manifest_path = staging.join(MANIFEST_FILE);
        fs::write(&manifest_path, &manifest_bytes).map_err(|error| {
            ToolError(format!(
                "cannot write '{}': {error}",
                manifest_path.display()
            ))
        })?;
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    if let Err(error) = finalize_staging(&staging, &output, overwrite) {
        if staging.exists() {
            let _ = fs::remove_dir_all(&staging);
        }
        return Err(error);
    }

    Ok(UnpackReport {
        input: input.to_path_buf(),
        manifest: output.join(MANIFEST_FILE),
        output,
        extracted_files: archive.entries.len(),
        extracted_bytes: u64::from(archive.header.index_offset)
            - u64::from(archive.header.header_size),
    })
}

fn validate_pack_output_target(
    input_root: &Path,
    manifest: &Manifest,
    output: &Path,
    overwrite: bool,
) -> ToolResult<()> {
    let output_name = output.file_name().ok_or_else(|| {
        ToolError(format!(
            "packed output must name a file, not a filesystem root: '{}'",
            output.display()
        ))
    })?;
    if output.exists() {
        let metadata = fs::symlink_metadata(output).map_err(|error| {
            ToolError(format!(
                "cannot inspect packed output '{}': {error}",
                output.display()
            ))
        })?;
        if metadata.file_type().is_symlink() {
            return Err(ToolError(format!(
                "refusing to replace symlinked packed output '{}'",
                output.display()
            )));
        }
        if !metadata.is_file() {
            return Err(ToolError(format!(
                "packed output exists and is not a regular file: '{}'",
                output.display()
            )));
        }
        if !overwrite {
            return Err(ToolError(format!(
                "refusing to overwrite existing packed output '{}'; pass --overwrite",
                output.display()
            )));
        }
    }

    let output_parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !output_parent.is_dir() {
        return Err(ToolError(format!(
            "packed output parent does not exist or is not a directory: '{}'",
            output_parent.display()
        )));
    }
    let canonical_root = fs::canonicalize(input_root).map_err(|error| {
        ToolError(format!(
            "cannot resolve pack input root '{}': {error}",
            input_root.display()
        ))
    })?;
    let canonical_parent = fs::canonicalize(output_parent).map_err(|error| {
        ToolError(format!(
            "cannot resolve packed output parent '{}': {error}",
            output_parent.display()
        ))
    })?;
    let canonical_output = if output.exists() {
        fs::canonicalize(output).map_err(|error| {
            ToolError(format!(
                "cannot resolve packed output '{}': {error}",
                output.display()
            ))
        })?
    } else {
        canonical_parent.join(output_name)
    };
    if canonical_output.starts_with(&canonical_root) {
        return Err(ToolError(format!(
            "packed output '{}' must be outside the unpacked input directory",
            output.display()
        )));
    }

    if let Some(root_parent) = canonical_root.parent() {
        let source_candidate = root_parent.join(&manifest.source_archive);
        if source_candidate.is_file() {
            let canonical_source = fs::canonicalize(&source_candidate).map_err(|error| {
                ToolError(format!(
                    "cannot resolve source archive candidate '{}': {error}",
                    source_candidate.display()
                ))
            })?;
            if canonical_output == canonical_source {
                return Err(ToolError(format!(
                    "refusing to overwrite the source archive recorded by the manifest: '{}'",
                    output.display()
                )));
            }
        }
    }
    Ok(())
}

fn write_packed_archive(output: &Path, bytes: &[u8], overwrite: bool) -> ToolResult<()> {
    let staging = unique_sibling(output, "staging")?;
    fs::write(&staging, bytes)
        .map_err(|error| ToolError(format!("cannot write '{}': {error}", staging.display())))?;
    if !output.exists() {
        return fs::rename(&staging, output).map_err(|error| {
            let _ = fs::remove_file(&staging);
            ToolError(format!(
                "cannot move completed archive '{}' to '{}': {error}",
                staging.display(),
                output.display()
            ))
        });
    }
    if !overwrite {
        let _ = fs::remove_file(&staging);
        return Err(ToolError(format!(
            "refusing to overwrite existing packed output '{}'; pass --overwrite",
            output.display()
        )));
    }

    let backup = unique_sibling(output, "backup")?;
    if let Err(error) = fs::rename(output, &backup) {
        let _ = fs::remove_file(&staging);
        return Err(ToolError(format!(
            "cannot move existing packed output '{}' aside: {error}",
            output.display()
        )));
    }
    if let Err(error) = fs::rename(&staging, output) {
        let restore_error = fs::rename(&backup, output).err();
        let _ = fs::remove_file(&staging);
        return Err(ToolError(match restore_error {
            Some(restore) => format!(
                "cannot install completed archive '{}': {error}; restoring '{}' also failed: {restore}",
                output.display(),
                backup.display()
            ),
            None => format!(
                "cannot install completed archive '{}': {error}; original output was restored",
                output.display()
            ),
        }));
    }
    fs::remove_file(&backup).map_err(|error| {
        ToolError(format!(
            "packed output was installed, but old backup '{}' could not be removed: {error}",
            backup.display()
        ))
    })
}

/// Rebuild an FL4 archive from a validated unpack manifest and its listed payload files.
pub fn pack_archive(
    input: &Path,
    output_arg: Option<&Path>,
    overwrite: bool,
) -> ToolResult<PackReport> {
    let (root, manifest_path, manifest, validated) = read_validated_pack_input(input)?;
    validate_payload_files(&root, &validated)?;
    let output = output_arg
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_packed_output_path(&root));
    validate_pack_output_target(&root, &manifest, &output, overwrite)?;

    let mut payloads = Vec::with_capacity(validated.entries.len());
    let mut payload_sizes = Vec::with_capacity(validated.entries.len());
    for (entry_index, entry) in validated.entries.iter().enumerate() {
        let payload_path = root.join(&entry.relative_path);
        let payload = fs::read(&payload_path).map_err(|error| {
            ToolError(format!(
                "cannot read manifest entry {entry_index} payload '{}': {error}",
                payload_path.display()
            ))
        })?;
        let size = u32::try_from(payload.len()).map_err(|_| {
            ToolError(format!(
                "manifest entry {entry_index} payload is {} bytes, exceeds u32: '{}'",
                payload.len(),
                payload_path.display()
            ))
        })?;
        payloads.push(payload);
        payload_sizes.push(size);
    }

    let mut relative_offsets = vec![0u32; validated.entries.len()];
    let mut data_size = 0u64;
    for entry_index in &validated.data_order {
        relative_offsets[*entry_index] = u32::try_from(data_size).map_err(|_| {
            ToolError(format!(
                "payload data before manifest entry {entry_index} exceeds u32"
            ))
        })?;
        data_size = checked_end(
            data_size,
            u64::from(payload_sizes[*entry_index]),
            "packed payload data",
        )?;
    }
    let packed_index_offset_u64 = u64::from(manifest.header_size)
        .checked_add(data_size)
        .ok_or_else(|| ToolError("packed index offset overflow".to_string()))?;
    let packed_index_offset = u32::try_from(packed_index_offset_u64)
        .map_err(|_| ToolError("packed index offset exceeds u32".to_string()))?;

    let mut bucket_table = Vec::with_capacity(BUCKET_TABLE_SIZE);
    let mut records = Vec::new();
    for bucket_index in 0..BUCKET_COUNT {
        let records_offset = BUCKET_TABLE_SIZE
            .checked_add(records.len())
            .ok_or_else(|| ToolError("packed record offset overflow".to_string()))?;
        bucket_table.extend_from_slice(
            &u32::try_from(records_offset)
                .map_err(|_| ToolError("packed record offset exceeds u32".to_string()))?
                .to_le_bytes(),
        );
        bucket_table.extend_from_slice(
            &u16::try_from(validated.bucket_entries[bucket_index].len())
                .map_err(|_| ToolError(format!("bucket {bucket_index} entry count exceeds u16")))?
                .to_le_bytes(),
        );
        for entry_index in &validated.bucket_entries[bucket_index] {
            let name_bytes = &validated.entries[*entry_index].name_bytes;
            records.extend_from_slice(&relative_offsets[*entry_index].to_le_bytes());
            records.extend_from_slice(&payload_sizes[*entry_index].to_le_bytes());
            records.push(u8::try_from(name_bytes.len()).map_err(|_| {
                ToolError(format!(
                    "manifest entry {entry_index} name length exceeds u8"
                ))
            })?);
            records.extend_from_slice(name_bytes);
        }
    }
    if bucket_table.len() != BUCKET_TABLE_SIZE {
        return Err(ToolError(format!(
            "packed bucket table is {} bytes, expected {BUCKET_TABLE_SIZE}",
            bucket_table.len()
        )));
    }
    let mut index = bucket_table;
    index.extend_from_slice(&records);
    index.extend_from_slice(&validated.index_tail);
    let packed_index_size = u32::try_from(index.len())
        .map_err(|_| ToolError("packed index size exceeds u32".to_string()))?;

    let mut header = validated.header;
    header[10..14].copy_from_slice(&packed_index_size.to_le_bytes());
    header[14..18].copy_from_slice(&packed_index_offset.to_le_bytes());
    header[18..22].copy_from_slice(&manifest.file_count.to_le_bytes());

    let output_bytes = u64::try_from(header.len())
        .ok()
        .and_then(|header_len| header_len.checked_add(data_size))
        .and_then(|value| value.checked_add(index.len() as u64))
        .ok_or_else(|| ToolError("packed archive size overflow".to_string()))?;
    let capacity = usize::try_from(output_bytes)
        .map_err(|_| ToolError("packed archive size does not fit usize".to_string()))?;
    let mut archive = Vec::with_capacity(capacity);
    archive.extend_from_slice(&header);
    for entry_index in &validated.data_order {
        archive.extend_from_slice(&payloads[*entry_index]);
    }
    archive.extend_from_slice(&index);
    if archive.len() != capacity {
        return Err(ToolError(format!(
            "packed archive size mismatch: calculated {capacity}, wrote {}",
            archive.len()
        )));
    }

    let reparsed = parse_archive(&archive)?;
    if reparsed.entries.len() != validated.entries.len() {
        return Err(ToolError(format!(
            "packed archive reparsed {} entries, expected {}",
            reparsed.entries.len(),
            validated.entries.len()
        )));
    }
    for (entry_index, entry) in reparsed.entries.iter().enumerate() {
        let expected = &validated.entries[entry_index];
        if entry.name_bytes != expected.name_bytes
            || entry.bucket != manifest.entries[entry_index].bucket
            || entry.bucket_index != manifest.entries[entry_index].bucket_index
            || entry.relative_data_offset != relative_offsets[entry_index]
            || entry.size != payload_sizes[entry_index]
        {
            return Err(ToolError(format!(
                "packed archive changed structural metadata for manifest entry {entry_index}"
            )));
        }
        if reparsed.payload(&archive, entry)? != payloads[entry_index] {
            return Err(ToolError(format!(
                "packed archive payload verification failed for manifest entry {entry_index}"
            )));
        }
    }

    write_packed_archive(&output, &archive, overwrite)?;
    Ok(PackReport {
        input: root,
        manifest: manifest_path,
        output,
        packed_files: payloads.len(),
        output_bytes,
    })
}

pub fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

    struct TempDirectory(PathBuf);

    impl TempDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "kokorov-fl4-test-{}-{}",
                std::process::id(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn synthetic_archive(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
        let header_size = MIN_HEADER_SIZE;
        let mut relative_offsets = Vec::with_capacity(entries.len());
        let mut data_size = 0usize;
        for (_, payload) in entries {
            relative_offsets.push(data_size);
            data_size += payload.len();
        }

        let mut grouped = vec![Vec::new(); BUCKET_COUNT];
        for (entry_index, (name, _)) in entries.iter().enumerate() {
            grouped[runtime_name_hash(name) as usize].push(entry_index);
        }
        let mut records = Vec::new();
        let mut bucket_table = Vec::with_capacity(BUCKET_TABLE_SIZE);
        for bucket_entries in &grouped {
            bucket_table
                .extend_from_slice(&((BUCKET_TABLE_SIZE + records.len()) as u32).to_le_bytes());
            bucket_table.extend_from_slice(&(bucket_entries.len() as u16).to_le_bytes());
            for entry_index in bucket_entries {
                let (name, payload) = entries[*entry_index];
                records.extend_from_slice(&(relative_offsets[*entry_index] as u32).to_le_bytes());
                records.extend_from_slice(&(payload.len() as u32).to_le_bytes());
                records.push(name.len() as u8);
                records.extend_from_slice(name);
            }
        }
        let index_size = bucket_table.len() + records.len() + 4;
        let index_offset = header_size + data_size;
        let mut archive = vec![0u8; header_size];
        archive[..FL4_MAGIC.len()].copy_from_slice(FL4_MAGIC);
        archive[8..10].copy_from_slice(&(header_size as u16).to_le_bytes());
        archive[10..14].copy_from_slice(&(index_size as u32).to_le_bytes());
        archive[14..18].copy_from_slice(&(index_offset as u32).to_le_bytes());
        archive[18..22].copy_from_slice(&(entries.len() as u32).to_le_bytes());
        for (_, payload) in entries {
            archive.extend_from_slice(payload);
        }
        archive.extend_from_slice(&bucket_table);
        archive.extend_from_slice(&records);
        archive.extend_from_slice(&[0xff; 4]);
        archive
    }

    #[test]
    fn parses_hash_buckets_and_data_order() {
        let archive = synthetic_archive(&[(b"ONE.AB", b"abc"), (b"two.bmp", b"defgh")]);
        let parsed = parse_archive(&archive).expect("valid archive");
        assert_eq!(parsed.entries.len(), 2);
        assert_eq!(parsed.header.header_size, 28);
        let mut entries = parsed.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.data_order);
        assert_eq!(entries[0].name, "ONE.AB");
        assert_eq!(entries[0].data_offset, 28);
        assert_eq!(parsed.payload(&archive, entries[1]).unwrap(), b"defgh");
        assert_eq!(entries[0].bucket as u32, runtime_name_hash(b"ONE.AB"));
    }

    #[test]
    fn runtime_hash_uses_basename_and_ascii_uppercase() {
        assert_eq!(
            runtime_name_hash(b"DIR\\name.ab"),
            runtime_name_hash(b"NAME.AB")
        );
        assert!(runtime_name_hash(b"DIR/name.ab") != runtime_name_hash(b"NAME.AB"));
    }

    #[test]
    fn rejects_bad_magic() {
        let mut archive = synthetic_archive(&[(b"ONE.AB", b"abc")]);
        archive[0] = b'X';
        assert!(parse_archive(&archive)
            .unwrap_err()
            .to_string()
            .contains("unsupported archive magic"));
    }

    #[test]
    fn rejects_encrypted_index() {
        let mut archive = synthetic_archive(&[(b"ONE.AB", b"abc")]);
        archive[24..26].copy_from_slice(&1u16.to_le_bytes());
        assert!(parse_archive(&archive)
            .unwrap_err()
            .to_string()
            .contains("encrypted FL4 index"));
    }

    #[test]
    fn rejects_bucket_hash_mismatch() {
        let mut archive = synthetic_archive(&[(b"ONE.AB", b"abc")]);
        let index_offset = u32::from_le_bytes(archive[14..18].try_into().unwrap()) as usize;
        let bucket = runtime_name_hash(b"ONE.AB") as usize;
        let descriptor = index_offset + bucket * 6;
        let records_offset =
            u32::from_le_bytes(archive[descriptor..descriptor + 4].try_into().unwrap());
        let next_bucket = (bucket + 1) % HASH_MODULUS as usize;
        let next_descriptor = index_offset + next_bucket * 6;
        archive[next_descriptor..next_descriptor + 4]
            .copy_from_slice(&records_offset.to_le_bytes());
        archive[next_descriptor + 4..next_descriptor + 6].copy_from_slice(&1u16.to_le_bytes());
        archive[descriptor + 4..descriptor + 6].copy_from_slice(&0u16.to_le_bytes());
        assert!(parse_archive(&archive).is_err());
    }

    #[test]
    fn rejects_payload_gap() {
        let mut archive = synthetic_archive(&[(b"ONE.AB", b"abc")]);
        let index_offset = u32::from_le_bytes(archive[14..18].try_into().unwrap()) as usize;
        let bucket = runtime_name_hash(b"ONE.AB") as usize;
        let descriptor = index_offset + bucket * 6;
        let records_offset =
            u32::from_le_bytes(archive[descriptor..descriptor + 4].try_into().unwrap()) as usize;
        archive[index_offset + records_offset..index_offset + records_offset + 4]
            .copy_from_slice(&1u32.to_le_bytes());
        assert!(parse_archive(&archive)
            .unwrap_err()
            .to_string()
            .contains("unexplained gap"));
    }

    #[test]
    fn rejects_unsafe_paths() {
        assert!(safe_relative_path("../evil.AB").is_err());
        assert!(safe_relative_path("C:/evil.AB").is_err());
        assert!(safe_relative_path("NUL.TXT").is_err());
        assert!(safe_relative_path("bad. ").is_err());
    }

    #[test]
    fn rejects_bad_index_terminator() {
        let mut archive = synthetic_archive(&[(b"ONE.AB", b"abc")]);
        *archive.last_mut().unwrap() = 0;
        assert!(parse_archive(&archive)
            .unwrap_err()
            .to_string()
            .contains("index terminator"));
    }

    #[test]
    fn unpacks_payloads_and_utf8_manifest() {
        let temp = TempDirectory::new();
        let source = temp.0.join("sample arbitrary name.FL4");
        let archive = synthetic_archive(&[(b"ONE.AB", b"abc"), (b"DIR\\TWO.BMP", b"defgh")]);
        fs::write(&source, archive).unwrap();
        let output = temp.0.join("custom result");
        let report = unpack_archive(&source, Some(&output), false).unwrap();
        assert_eq!(report.extracted_files, 2);
        assert_eq!(fs::read(output.join("ONE.AB")).unwrap(), b"abc");
        assert_eq!(
            fs::read(output.join("DIR").join("TWO.BMP")).unwrap(),
            b"defgh"
        );
        let manifest: Manifest =
            serde_json::from_slice(&fs::read(output.join(MANIFEST_FILE)).unwrap()).unwrap();
        assert_eq!(manifest.source_archive, "sample arbitrary name.FL4");
        assert_eq!(manifest.entries.len(), 2);
    }

    #[test]
    fn refuses_output_directory_that_contains_input() {
        let temp = TempDirectory::new();
        let source = temp.0.join("A.FL4");
        fs::write(&source, synthetic_archive(&[(b"ONE.AB", b"abc")])).unwrap();
        let error = unpack_archive(&source, Some(&temp.0), true)
            .unwrap_err()
            .to_string();
        assert!(error.contains("contains the input archive"));
    }

    #[test]
    fn unpack_then_pack_is_byte_exact() {
        let temp = TempDirectory::new();
        let source = temp.0.join("sample.FL4");
        let expected = synthetic_archive(&[
            (b"ONE.AB", b"abc"),
            (b"DIR\\TWO.BMP", b"defgh"),
            (b"THREE.WV5", b"voice payload"),
        ]);
        fs::write(&source, &expected).unwrap();
        let unpacked = temp.0.join("arbitrary workspace");
        unpack_archive(&source, Some(&unpacked), false).unwrap();
        let packed = temp.0.join("rebuilt.FL4");
        let report = pack_archive(&unpacked, Some(&packed), false).unwrap();
        assert_eq!(report.packed_files, 3);
        assert_eq!(report.output_bytes, expected.len() as u64);
        assert_eq!(fs::read(&packed).unwrap(), expected);
    }

    #[test]
    fn pack_accepts_manifest_file_as_input() {
        let temp = TempDirectory::new();
        let source = temp.0.join("source.FL4");
        let expected = synthetic_archive(&[(b"ONE.AB", b"abc")]);
        fs::write(&source, &expected).unwrap();
        let unpacked = temp.0.join("renamed workspace");
        unpack_archive(&source, Some(&unpacked), false).unwrap();
        let packed = temp.0.join("from manifest.FL4");
        pack_archive(&unpacked.join(MANIFEST_FILE), Some(&packed), false).unwrap();
        assert_eq!(fs::read(&packed).unwrap(), expected);
    }

    #[test]
    fn pack_rebuilds_changed_payload_offsets_and_sizes() {
        let temp = TempDirectory::new();
        let source = temp.0.join("source.FL4");
        fs::write(
            &source,
            synthetic_archive(&[(b"ONE.AB", b"abc"), (b"DIR\\TWO.BMP", b"defgh")]),
        )
        .unwrap();
        let unpacked = temp.0.join("files");
        unpack_archive(&source, Some(&unpacked), false).unwrap();
        fs::write(unpacked.join("ONE.AB"), b"a much longer replacement").unwrap();
        fs::write(unpacked.join("DIR").join("TWO.BMP"), b"x").unwrap();

        let packed = temp.0.join("changed.FL4");
        pack_archive(&unpacked, Some(&packed), false).unwrap();
        let rebuilt = fs::read(&packed).unwrap();
        let parsed = parse_archive(&rebuilt).unwrap();
        let one = parsed
            .entries
            .iter()
            .find(|entry| entry.name == "ONE.AB")
            .unwrap();
        let two = parsed
            .entries
            .iter()
            .find(|entry| entry.name == "DIR\\TWO.BMP")
            .unwrap();
        assert_eq!(
            parsed.payload(&rebuilt, one).unwrap(),
            b"a much longer replacement"
        );
        assert_eq!(parsed.payload(&rebuilt, two).unwrap(), b"x");
        assert_eq!(one.relative_data_offset, 0);
        assert_eq!(two.relative_data_offset, one.size);
        assert_eq!(parsed.header.index_offset as u64, 28 + one.size as u64 + 1);
    }

    #[test]
    fn pack_rejects_manifest_name_tampering() {
        let temp = TempDirectory::new();
        let source = temp.0.join("source.FL4");
        fs::write(&source, synthetic_archive(&[(b"ONE.AB", b"abc")])).unwrap();
        let unpacked = temp.0.join("files");
        unpack_archive(&source, Some(&unpacked), false).unwrap();
        let manifest_path = unpacked.join(MANIFEST_FILE);
        let mut manifest: Manifest =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest.entries[0].name = "CHANGED.AB".to_string();
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        let error = pack_archive(&unpacked, Some(&temp.0.join("bad.FL4")), false)
            .unwrap_err()
            .to_string();
        assert!(error.contains("does not match original bytes"));
    }

    #[test]
    fn pack_rejects_duplicate_data_order() {
        let temp = TempDirectory::new();
        let source = temp.0.join("source.FL4");
        fs::write(
            &source,
            synthetic_archive(&[(b"ONE.AB", b"abc"), (b"TWO.AB", b"def")]),
        )
        .unwrap();
        let unpacked = temp.0.join("files");
        unpack_archive(&source, Some(&unpacked), false).unwrap();
        let manifest_path = unpacked.join(MANIFEST_FILE);
        let mut manifest: Manifest =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest.entries[1].data_order = manifest.entries[0].data_order;
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        let error = pack_archive(&unpacked, Some(&temp.0.join("bad.FL4")), false)
            .unwrap_err()
            .to_string();
        assert!(error.contains("invalid or duplicate data_order"));
    }

    #[test]
    fn pack_rejects_missing_payload_and_output_inside_input() {
        let temp = TempDirectory::new();
        let source = temp.0.join("source.FL4");
        fs::write(&source, synthetic_archive(&[(b"ONE.AB", b"abc")])).unwrap();
        let unpacked = temp.0.join("files");
        unpack_archive(&source, Some(&unpacked), false).unwrap();
        let inside_error = pack_archive(&unpacked, Some(&unpacked.join("bad.FL4")), false)
            .unwrap_err()
            .to_string();
        assert!(inside_error.contains("outside the unpacked input directory"));

        fs::remove_file(unpacked.join("ONE.AB")).unwrap();
        let missing_error = pack_archive(&unpacked, Some(&temp.0.join("bad.FL4")), false)
            .unwrap_err()
            .to_string();
        assert!(missing_error.contains("payload is missing"));
    }

    #[test]
    fn pack_refuses_to_overwrite_recorded_source_archive() {
        let temp = TempDirectory::new();
        let source = temp.0.join("source.FL4");
        fs::write(&source, synthetic_archive(&[(b"ONE.AB", b"abc")])).unwrap();
        let unpacked = temp.0.join("files");
        unpack_archive(&source, Some(&unpacked), false).unwrap();
        let error = pack_archive(&unpacked, Some(&source), true)
            .unwrap_err()
            .to_string();
        assert!(error.contains("refusing to overwrite the source archive"));
    }

    #[test]
    fn pack_requires_explicit_overwrite_and_replaces_transactionally() {
        let temp = TempDirectory::new();
        let source = temp.0.join("source.FL4");
        let expected = synthetic_archive(&[(b"ONE.AB", b"abc")]);
        fs::write(&source, &expected).unwrap();
        let unpacked = temp.0.join("files");
        unpack_archive(&source, Some(&unpacked), false).unwrap();
        let output = temp.0.join("existing packed.FL4");
        fs::write(&output, b"old output").unwrap();

        let error = pack_archive(&unpacked, Some(&output), false)
            .unwrap_err()
            .to_string();
        assert!(error.contains("refusing to overwrite"));
        assert_eq!(fs::read(&output).unwrap(), b"old output");

        pack_archive(&unpacked, Some(&output), true).unwrap();
        assert_eq!(fs::read(&output).unwrap(), expected);
    }
}
