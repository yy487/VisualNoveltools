pub mod ab;
pub mod ab_text;
pub mod ab_workflow;

use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const FL2_MAGIC: &[u8; 6] = b"FL2.0\0";
pub const KNOWN_HEADER_SIZE: usize = 32;
pub const MANIFEST_FILE: &str = "fl2_manifest.json";

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
pub struct Fl2Header {
    pub raw: Vec<u8>,
    pub header_size: u16,
    pub file_count: u32,
    pub index_size: u32,
    pub index_offset: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fl2Entry {
    pub index: usize,
    pub name: String,
    pub name_bytes: Vec<u8>,
    pub size: u32,
    pub data_offset: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fl2Archive {
    pub file_len: u64,
    pub header: Fl2Header,
    pub entries: Vec<Fl2Entry>,
    pub index_tail: Vec<u8>,
}

impl Fl2Archive {
    pub fn payload<'a>(&self, bytes: &'a [u8], entry: &Fl2Entry) -> ToolResult<&'a [u8]> {
        let start = usize::try_from(entry.data_offset)
            .map_err(|_| ToolError(format!("entry {} offset does not fit usize", entry.index)))?;
        let end = start
            .checked_add(
                usize::try_from(entry.size).map_err(|_| {
                    ToolError(format!("entry {} size does not fit usize", entry.index))
                })?,
            )
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
    pub source: String,
    pub file_size: u64,
    pub magic: String,
    pub header_size: u16,
    pub file_count: u32,
    pub index_offset: u32,
    pub index_size: u32,
    pub data_offset: u16,
    pub data_size: u64,
    pub header_hex: String,
    pub index_tail_hex: String,
    pub entries: Vec<ManifestEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestEntry {
    pub index: usize,
    pub name: String,
    pub name_bytes_hex: String,
    pub size: u32,
    pub data_offset: u64,
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
pub struct PackReport {
    pub input: PathBuf,
    pub output: PathBuf,
    pub packed_files: usize,
    pub output_bytes: u64,
}

fn read_u16(data: &[u8], offset: usize, label: &str) -> ToolResult<u16> {
    let end = offset
        .checked_add(2)
        .ok_or_else(|| ToolError(format!("{label} offset overflow")))?;
    let bytes = data
        .get(offset..end)
        .ok_or_else(|| ToolError(format!("{label} is truncated at 0x{offset:x}")))?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_u32(data: &[u8], offset: usize, label: &str) -> ToolResult<u32> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| ToolError(format!("{label} offset overflow")))?;
    let bytes = data
        .get(offset..end)
        .ok_or_else(|| ToolError(format!("{label} is truncated at 0x{offset:x}")))?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
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
            "entry {index} name failed CP932 round-trip: {}",
            hex(raw)
        )));
    }
    Ok(name)
}

/// Parse the FL2.0 header, entry table, and payload boundaries without modifying input bytes.
pub fn parse_archive(bytes: &[u8]) -> ToolResult<Fl2Archive> {
    if bytes.len() < KNOWN_HEADER_SIZE {
        return Err(ToolError(format!(
            "archive is too short for the 32-byte FL2 header: {} bytes",
            bytes.len()
        )));
    }
    if &bytes[..FL2_MAGIC.len()] != FL2_MAGIC {
        return Err(ToolError(format!(
            "unsupported archive magic: {}",
            hex(&bytes[..FL2_MAGIC.len()])
        )));
    }

    let header_size = read_u16(bytes, 6, "header_size")?;
    if usize::from(header_size) < KNOWN_HEADER_SIZE {
        return Err(ToolError(format!(
            "header_size {header_size} is smaller than the known 32-byte header"
        )));
    }
    if usize::from(header_size) > bytes.len() {
        return Err(ToolError(format!(
            "header_size {header_size} exceeds archive length {}",
            bytes.len()
        )));
    }

    let file_count = read_u32(bytes, 8, "file_count")?;
    let index_size = read_u32(bytes, 12, "index_size")?;
    let index_offset = read_u32(bytes, 16, "index_offset")?;
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

    let index_start_usize = usize::try_from(index_start)
        .map_err(|_| ToolError("index offset does not fit usize".to_string()))?;
    let index_end_usize = usize::try_from(index_end)
        .map_err(|_| ToolError("index end does not fit usize".to_string()))?;
    let file_count_usize = usize::try_from(file_count)
        .map_err(|_| ToolError("file_count does not fit usize".to_string()))?;
    if file_count_usize > usize::try_from(index_size).unwrap_or(usize::MAX) / 5 {
        return Err(ToolError(format!(
            "file_count {file_count} cannot fit in index_size {index_size}"
        )));
    }
    let header_end = u64::from(header_size);
    let mut cursor = index_start_usize;
    let mut data_cursor = header_end;
    let mut entries = Vec::with_capacity(file_count_usize);

    for index in 0..file_count_usize {
        let size = read_u32(bytes, cursor, &format!("entry {index} size"))?;
        cursor = cursor
            .checked_add(4)
            .ok_or_else(|| ToolError(format!("entry {index} index offset overflow")))?;
        let name_len = *bytes
            .get(cursor)
            .ok_or_else(|| ToolError(format!("entry {index} name length is truncated")))?
            as usize;
        cursor += 1;
        let name_end = cursor
            .checked_add(name_len)
            .ok_or_else(|| ToolError(format!("entry {index} name length overflows")))?;
        if name_end > index_end_usize {
            return Err(ToolError(format!(
                "entry {index} name [0x{cursor:x}, 0x{name_end:x}) exceeds index end 0x{index_end_usize:x}"
            )));
        }
        let name_bytes = bytes[cursor..name_end].to_vec();
        let name = decode_name(&name_bytes, index)?;
        cursor = name_end;

        let next_data_cursor = checked_end(
            data_cursor,
            u64::from(size),
            &format!("entry {index} payload"),
        )?;
        if next_data_cursor > index_start {
            return Err(ToolError(format!(
                "entry {index} payload ends at 0x{next_data_cursor:x}, beyond index start 0x{index_offset:x}"
            )));
        }
        entries.push(Fl2Entry {
            index,
            name,
            name_bytes,
            size,
            data_offset: data_cursor,
        });
        data_cursor = next_data_cursor;
    }

    let tail = bytes[cursor..index_end_usize].to_vec();
    if tail != [0xff, 0xff, 0xff, 0xff] {
        return Err(ToolError(format!(
            "index terminator is not four 0xff bytes at 0x{cursor:x}: {}",
            hex(&tail)
        )));
    }
    if data_cursor != index_start {
        return Err(ToolError(format!(
            "payload size sum ends at 0x{data_cursor:x}, expected index start 0x{index_offset:x}"
        )));
    }

    Ok(Fl2Archive {
        file_len: bytes.len() as u64,
        header: Fl2Header {
            raw: bytes[..usize::from(header_size)].to_vec(),
            header_size,
            file_count,
            index_size,
            index_offset,
        },
        entries,
        index_tail: tail,
    })
}

fn reserved_windows_name(component: &str) -> bool {
    let base = component
        .split('.')
        .next()
        .unwrap_or(component)
        .to_ascii_uppercase();
    matches!(
        base.as_str(),
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
    let parts = name.split(['/', '\\']);
    let mut path = PathBuf::new();
    let mut key_parts = Vec::new();
    for component in parts {
        if component.is_empty() || component == "." || component == ".." {
            return Err(ToolError(format!("unsafe archive path: {name:?}")));
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
        key_parts.push(component.to_ascii_uppercase());
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

pub fn default_packed_output_path(input: &Path) -> PathBuf {
    let name = input
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "archive_unpacked".to_string());
    let stem = name
        .strip_suffix("_unpacked")
        .filter(|value| !value.is_empty())
        .unwrap_or(&name);
    input.with_file_name(format!("{stem}_packed.FL2"))
}

pub fn build_manifest(
    input: &Path,
    archive: &Fl2Archive,
    paths: &[PathBuf],
) -> ToolResult<Manifest> {
    if paths.len() != archive.entries.len() {
        return Err(ToolError(
            "manifest path count does not match entry count".to_string(),
        ));
    }
    let data_offset = archive.header.header_size;
    let data_size = u64::from(archive.header.index_offset) - u64::from(data_offset);
    let entries = archive
        .entries
        .iter()
        .zip(paths)
        .map(|(entry, path)| ManifestEntry {
            index: entry.index,
            name: entry.name.clone(),
            name_bytes_hex: hex(&entry.name_bytes),
            size: entry.size,
            data_offset: entry.data_offset,
            output_path: path_to_manifest(path),
        })
        .collect();
    Ok(Manifest {
        format: "KOKOROV FL2.0".to_string(),
        source: input.display().to_string(),
        file_size: archive.file_len,
        magic: String::from("FL2.0\\0"),
        header_size: archive.header.header_size,
        file_count: archive.header.file_count,
        index_offset: archive.header.index_offset,
        index_size: archive.header.index_size,
        data_offset,
        data_size,
        header_hex: hex(&archive.header.raw),
        index_tail_hex: hex(&archive.index_tail),
        entries,
    })
}

pub fn unpack_archive(
    input: &Path,
    output_arg: Option<&Path>,
    overwrite: bool,
) -> ToolResult<UnpackReport> {
    let bytes = fs::read(input)
        .map_err(|error| ToolError(format!("cannot read '{}': {error}", input.display())))?;
    let archive = parse_archive(&bytes)?;
    let output = output_arg
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_path(input));
    if output.exists() && !output.is_dir() {
        return Err(ToolError(format!(
            "output path exists and is not a directory: '{}'",
            output.display()
        )));
    }
    if output.exists() && !overwrite {
        return Err(ToolError(format!(
            "refusing to overwrite existing output directory '{}'; pass --overwrite",
            output.display()
        )));
    }

    let mut destinations = Vec::with_capacity(archive.entries.len());
    let mut destination_keys = HashSet::new();
    for entry in &archive.entries {
        let (relative, key) = safe_relative_path(&entry.name)?;
        if !destination_keys.insert(key) {
            return Err(ToolError(format!(
                "duplicate output path for entry {}: {}",
                entry.index, entry.name
            )));
        }
        archive.payload(&bytes, entry)?;
        destinations.push(relative);
    }

    fs::create_dir_all(&output)
        .map_err(|error| ToolError(format!("cannot create '{}': {error}", output.display())))?;
    let mut extracted_bytes = 0u64;
    for (entry, relative) in archive.entries.iter().zip(&destinations) {
        let target = output.join(relative);
        if target.exists() && !overwrite {
            return Err(ToolError(format!(
                "refusing to overwrite existing file '{}'",
                target.display()
            )));
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                ToolError(format!("cannot create '{}': {error}", parent.display()))
            })?;
        }
        let payload = archive.payload(&bytes, entry)?;
        fs::write(&target, payload)
            .map_err(|error| ToolError(format!("cannot write '{}': {error}", target.display())))?;
        extracted_bytes += u64::from(entry.size);
    }

    let manifest_path = output.join(MANIFEST_FILE);
    let manifest = build_manifest(input, &archive, &destinations)?;
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| ToolError(format!("cannot serialize manifest: {error}")))?;
    fs::write(&manifest_path, manifest_bytes).map_err(|error| {
        ToolError(format!(
            "cannot write '{}': {error}",
            manifest_path.display()
        ))
    })?;

    Ok(UnpackReport {
        input: input.to_path_buf(),
        output,
        manifest: manifest_path,
        extracted_files: archive.entries.len(),
        extracted_bytes,
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
        let high = hex_digit(bytes[offset]).ok_or_else(|| {
            ToolError(format!("{label} has a non-hex digit at position {offset}"))
        })?;
        let low = hex_digit(bytes[offset + 1]).ok_or_else(|| {
            ToolError(format!(
                "{label} has a non-hex digit at position {}",
                offset + 1
            ))
        })?;
        output.push((high << 4) | low);
    }
    Ok(output)
}

fn hex_digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn validate_pack_manifest(manifest: &Manifest) -> ToolResult<Vec<u8>> {
    if manifest.format != "KOKOROV FL2.0" {
        return Err(ToolError(format!(
            "unsupported manifest format: {:?}",
            manifest.format
        )));
    }
    if manifest.magic != "FL2.0\\0" {
        return Err(ToolError(format!(
            "unsupported manifest magic: {:?}",
            manifest.magic
        )));
    }
    if usize::from(manifest.header_size) < KNOWN_HEADER_SIZE {
        return Err(ToolError(format!(
            "manifest header_size {} is smaller than {KNOWN_HEADER_SIZE}",
            manifest.header_size
        )));
    }
    let header = decode_hex(&manifest.header_hex, "manifest header_hex")?;
    if header.len() != usize::from(manifest.header_size) {
        return Err(ToolError(format!(
            "manifest header_hex is {} bytes, expected header_size {}",
            header.len(),
            manifest.header_size
        )));
    }
    if header.get(..FL2_MAGIC.len()) != Some(FL2_MAGIC) {
        return Err(ToolError(
            "manifest header_hex does not contain FL2.0 magic".to_string(),
        ));
    }
    if read_u16(&header, 6, "manifest header_size")? != manifest.header_size
        || read_u32(&header, 8, "manifest file_count")? != manifest.file_count
        || read_u32(&header, 12, "manifest index_size")? != manifest.index_size
        || read_u32(&header, 16, "manifest index_offset")? != manifest.index_offset
    {
        return Err(ToolError(
            "manifest fields do not match the preserved header bytes".to_string(),
        ));
    }
    if manifest.data_offset != manifest.header_size {
        return Err(ToolError(format!(
            "manifest data_offset {} does not equal header_size {}",
            manifest.data_offset, manifest.header_size
        )));
    }
    let expected_file_size = u64::from(manifest.index_offset)
        .checked_add(u64::from(manifest.index_size))
        .ok_or_else(|| ToolError("manifest file size overflows u64".to_string()))?;
    if manifest.file_size != expected_file_size {
        return Err(ToolError(format!(
            "manifest file_size {} does not equal index_offset + index_size {}",
            manifest.file_size, expected_file_size
        )));
    }
    let entry_count = usize::try_from(manifest.file_count)
        .map_err(|_| ToolError("manifest file_count does not fit usize".to_string()))?;
    if manifest.entries.len() != entry_count {
        return Err(ToolError(format!(
            "manifest has {} entries, expected file_count {}",
            manifest.entries.len(),
            manifest.file_count
        )));
    }
    let index_tail = decode_hex(&manifest.index_tail_hex, "manifest index_tail_hex")?;
    if index_tail != [0xff, 0xff, 0xff, 0xff] {
        return Err(ToolError(format!(
            "manifest index terminator is not four 0xff bytes: {}",
            manifest.index_tail_hex
        )));
    }

    let mut expected_data_offset = u64::from(manifest.header_size);
    let mut expected_index_size = u64::try_from(index_tail.len())
        .map_err(|_| ToolError("manifest index tail length does not fit u64".to_string()))?;
    let mut paths = HashSet::new();
    for (index, entry) in manifest.entries.iter().enumerate() {
        if entry.index != index {
            return Err(ToolError(format!(
                "manifest entry order mismatch: position {index} stores index {}",
                entry.index
            )));
        }
        let name_bytes = decode_hex(
            &entry.name_bytes_hex,
            &format!("manifest entry {index} name_bytes_hex"),
        )?;
        if name_bytes.len() > usize::from(u8::MAX) {
            return Err(ToolError(format!(
                "manifest entry {index} name is {} bytes, exceeds u8 length",
                name_bytes.len()
            )));
        }
        let decoded_name = decode_name(&name_bytes, index)?;
        if decoded_name != entry.name {
            return Err(ToolError(format!(
                "manifest entry {index} name does not match name_bytes_hex"
            )));
        }
        let (relative, path_key) = safe_relative_path(&entry.name)?;
        if !paths.insert(path_key) {
            return Err(ToolError(format!(
                "manifest has duplicate output path for entry {index}: {}",
                entry.name
            )));
        }
        if entry.output_path != path_to_manifest(&relative) {
            return Err(ToolError(format!(
                "manifest entry {index} output_path {:?} does not match archive name {:?}",
                entry.output_path, entry.name
            )));
        }
        if entry.data_offset != expected_data_offset {
            return Err(ToolError(format!(
                "manifest entry {index} data_offset {} does not match expected {}",
                entry.data_offset, expected_data_offset
            )));
        }
        expected_data_offset = expected_data_offset
            .checked_add(u64::from(entry.size))
            .ok_or_else(|| ToolError(format!("manifest entry {index} size overflows u64")))?;
        expected_index_size = expected_index_size
            .checked_add(5)
            .and_then(|value| value.checked_add(name_bytes.len() as u64))
            .ok_or_else(|| ToolError(format!("manifest entry {index} index size overflows u64")))?;
    }
    if expected_data_offset != u64::from(manifest.index_offset) {
        return Err(ToolError(format!(
            "manifest payloads end at {}, expected index_offset {}",
            expected_data_offset, manifest.index_offset
        )));
    }
    if manifest.data_size != u64::from(manifest.index_offset) - u64::from(manifest.data_offset) {
        return Err(ToolError(
            "manifest data_size does not match data range".to_string(),
        ));
    }
    if expected_index_size != u64::from(manifest.index_size) {
        return Err(ToolError(format!(
            "manifest index_size {} does not match entry table size {}",
            manifest.index_size, expected_index_size
        )));
    }
    Ok(header)
}

fn pack_staging_path(output: &Path) -> ToolResult<PathBuf> {
    let file_name = output.file_name().ok_or_else(|| {
        ToolError(format!(
            "output path has no file name: '{}'",
            output.display()
        ))
    })?;
    let mut staging_name = std::ffi::OsString::from(".");
    staging_name.push(file_name);
    staging_name.push(format!(".fl2pack-{}.tmp", std::process::id()));
    Ok(output.with_file_name(staging_name))
}

fn normalized_absolute_path(path: &Path) -> ToolResult<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| ToolError(format!("cannot get current directory: {error}")))?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(ToolError(format!(
                        "cannot normalize path '{}'",
                        path.display()
                    )));
                }
            }
        }
    }
    Ok(normalized)
}

fn ensure_pack_output_outside_input(input: &Path, output: &Path) -> ToolResult<()> {
    let input = normalized_absolute_path(input)?;
    let output = normalized_absolute_path(output)?;
    let input_key = input
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_uppercase();
    let output_key = output
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_uppercase();
    if output_key == input_key || output_key.starts_with(&format!("{input_key}/")) {
        return Err(ToolError(format!(
            "packed output '{}' must be outside unpacked directory '{}'",
            output.display(),
            input.display()
        )));
    }
    Ok(())
}

fn write_packed_archive(output: &Path, bytes: &[u8], overwrite: bool) -> ToolResult<()> {
    if output.exists() && output.is_dir() {
        return Err(ToolError(format!(
            "output path is a directory: '{}'",
            output.display()
        )));
    }
    if output.exists() && !overwrite {
        return Err(ToolError(format!(
            "refusing to overwrite existing output '{}'; pass --overwrite",
            output.display()
        )));
    }
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|error| ToolError(format!("cannot create '{}': {error}", parent.display())))?;
    }
    let staging = pack_staging_path(output)?;
    if staging.exists() {
        return Err(ToolError(format!(
            "staging path already exists; remove it before retrying: '{}'",
            staging.display()
        )));
    }
    if let Err(error) = fs::write(&staging, bytes) {
        return Err(ToolError(format!(
            "cannot write '{}': {error}",
            staging.display()
        )));
    }
    if output.exists() {
        if !overwrite {
            let _ = fs::remove_file(&staging);
            return Err(ToolError(format!(
                "output appeared while processing: '{}'",
                output.display()
            )));
        }
        if let Err(error) = fs::remove_file(output) {
            let _ = fs::remove_file(&staging);
            return Err(ToolError(format!(
                "cannot overwrite '{}': {error}",
                output.display()
            )));
        }
    }
    if let Err(error) = fs::rename(&staging, output) {
        let _ = fs::remove_file(&staging);
        return Err(ToolError(format!(
            "cannot move completed archive '{}' to '{}': {error}",
            staging.display(),
            output.display()
        )));
    }
    Ok(())
}

/// Rebuild a FL2.0 archive from a directory previously written by `unpack_archive`.
pub fn pack_archive(
    input: &Path,
    output_arg: Option<&Path>,
    overwrite: bool,
) -> ToolResult<PackReport> {
    if !input.is_dir() {
        return Err(ToolError(format!(
            "input is not an unpacked directory: '{}'",
            input.display()
        )));
    }
    let output = output_arg
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_packed_output_path(input));
    ensure_pack_output_outside_input(input, &output)?;
    let manifest_path = input.join(MANIFEST_FILE);
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
    let mut header = validate_pack_manifest(&manifest)?;

    let mut payloads = Vec::with_capacity(manifest.entries.len());
    let mut index = Vec::new();
    let mut data_size = 0u64;
    for (index_number, entry) in manifest.entries.iter().enumerate() {
        let (relative, _) = safe_relative_path(&entry.name)?;
        let payload_path = input.join(relative);
        if !payload_path.is_file() {
            return Err(ToolError(format!(
                "manifest entry {index_number} payload is missing or not a file: '{}'",
                payload_path.display()
            )));
        }
        let payload = fs::read(&payload_path).map_err(|error| {
            ToolError(format!("cannot read '{}': {error}", payload_path.display()))
        })?;
        let size = u32::try_from(payload.len()).map_err(|_| {
            ToolError(format!(
                "manifest entry {index_number} payload is {} bytes, exceeds u32",
                payload.len()
            ))
        })?;
        let name_bytes = decode_hex(
            &entry.name_bytes_hex,
            &format!("manifest entry {index_number} name_bytes_hex"),
        )?;
        index.extend_from_slice(&size.to_le_bytes());
        index.push(u8::try_from(name_bytes.len()).map_err(|_| {
            ToolError(format!(
                "manifest entry {index_number} name length exceeds u8"
            ))
        })?);
        index.extend_from_slice(&name_bytes);
        data_size = data_size
            .checked_add(u64::from(size))
            .ok_or_else(|| ToolError("packed payload size overflows u64".to_string()))?;
        payloads.push(payload);
    }
    index.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]);
    let packed_index_size = u32::try_from(index.len())
        .map_err(|_| ToolError("packed index size exceeds u32".to_string()))?;
    let packed_index_offset = u32::try_from(
        u64::from(manifest.header_size)
            .checked_add(data_size)
            .ok_or_else(|| ToolError("packed index offset overflows u64".to_string()))?,
    )
    .map_err(|_| ToolError("packed index offset exceeds u32".to_string()))?;

    header[8..12].copy_from_slice(&manifest.file_count.to_le_bytes());
    header[12..16].copy_from_slice(&packed_index_size.to_le_bytes());
    header[16..20].copy_from_slice(&packed_index_offset.to_le_bytes());

    let header_len = u64::try_from(header.len())
        .map_err(|_| ToolError("packed header size does not fit u64".to_string()))?;
    let index_len = u64::try_from(index.len())
        .map_err(|_| ToolError("packed index size does not fit u64".to_string()))?;
    let output_bytes = header_len
        .checked_add(data_size)
        .and_then(|value| value.checked_add(index_len))
        .ok_or_else(|| ToolError("packed archive size overflows u64".to_string()))?;
    let capacity = usize::try_from(output_bytes)
        .map_err(|_| ToolError("packed archive size does not fit usize".to_string()))?;
    let mut archive = Vec::with_capacity(capacity);
    archive.extend_from_slice(&header);
    for payload in &payloads {
        archive.extend_from_slice(payload);
    }
    archive.extend_from_slice(&index);
    if archive.len() != capacity {
        return Err(ToolError(format!(
            "packed archive size mismatch: calculated {capacity}, wrote {}",
            archive.len()
        )));
    }
    let reparsed = parse_archive(&archive)?;
    if reparsed.entries.len() != manifest.entries.len()
        || reparsed
            .entries
            .iter()
            .zip(&manifest.entries)
            .any(|(entry, manifest_entry)| entry.name != manifest_entry.name)
    {
        return Err(ToolError(
            "packed archive changed manifest entry order or names".to_string(),
        ));
    }

    write_packed_archive(&output, &archive, overwrite)?;
    Ok(PackReport {
        input: input.to_path_buf(),
        output,
        packed_files: manifest.entries.len(),
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

fn path_to_manifest(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
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
                "kokorov-fl2-test-{}-{}",
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
        let header_size = 32u32;
        let payload_size: usize = entries.iter().map(|(_, data)| data.len()).sum();
        let index_offset = header_size as usize + payload_size;
        let mut index = Vec::new();
        for (name, data) in entries {
            index.extend_from_slice(&(data.len() as u32).to_le_bytes());
            index.push(name.len() as u8);
            index.extend_from_slice(name);
        }
        index.extend_from_slice(&[0xff; 4]);

        let mut archive = vec![0u8; header_size as usize];
        archive[..6].copy_from_slice(FL2_MAGIC);
        archive[6..8].copy_from_slice(&(header_size as u16).to_le_bytes());
        archive[8..12].copy_from_slice(&(entries.len() as u32).to_le_bytes());
        archive[12..16].copy_from_slice(&(index.len() as u32).to_le_bytes());
        archive[16..20].copy_from_slice(&(index_offset as u32).to_le_bytes());
        archive[20..28].copy_from_slice(b"FMT_TEST");
        for (_, data) in entries {
            archive.extend_from_slice(data);
        }
        archive.extend_from_slice(&index);
        archive
    }

    #[test]
    fn parses_payload_offsets_and_cp932_names() {
        let archive = synthetic_archive(&[(b"ONE.AB", b"abc"), (b"TWO.TXT", b"defgh")]);
        let parsed = parse_archive(&archive).expect("valid archive");
        assert_eq!(parsed.entries.len(), 2);
        assert_eq!(parsed.entries[0].data_offset, 32);
        assert_eq!(parsed.entries[1].data_offset, 35);
        assert_eq!(
            parsed.payload(&archive, &parsed.entries[1]).unwrap(),
            b"defgh"
        );
        assert_eq!(parsed.index_tail, vec![0xff; 4]);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut archive = synthetic_archive(&[(b"ONE.AB", b"abc")]);
        archive[0] = b'X';
        let error = parse_archive(&archive).unwrap_err().to_string();
        assert!(error.contains("unsupported archive magic"));
    }

    #[test]
    fn rejects_payload_size_mismatch() {
        let mut archive = synthetic_archive(&[(b"ONE.AB", b"abc")]);
        let index_offset = u32::from_le_bytes(archive[16..20].try_into().unwrap()) as usize;
        archive[index_offset..index_offset + 4].copy_from_slice(&4u32.to_le_bytes());
        let error = parse_archive(&archive).unwrap_err().to_string();
        assert!(error.contains("beyond index start"));
    }

    #[test]
    fn rejects_unsafe_paths() {
        assert!(safe_relative_path("../evil.AB").is_err());
        assert!(safe_relative_path("C:/evil.AB").is_err());
        assert!(safe_relative_path("NUL.TXT").is_err());
    }

    #[test]
    fn rejects_bad_index_terminator() {
        let mut archive = synthetic_archive(&[(b"ONE.AB", b"abc")]);
        let index_offset = u32::from_le_bytes(archive[16..20].try_into().unwrap()) as usize;
        let index_size = u32::from_le_bytes(archive[12..16].try_into().unwrap()) as usize;
        archive[index_offset + index_size - 1] = 0;
        let error = parse_archive(&archive).unwrap_err().to_string();
        assert!(error.contains("index terminator"));
    }

    #[test]
    fn unpack_then_pack_is_byte_exact() {
        let temp = TempDirectory::new();
        let source = temp.0.join("A.FL2");
        let expected = synthetic_archive(&[(b"ONE.AB", b"abc"), (b"TWO.TXT", b"defgh")]);
        fs::write(&source, &expected).unwrap();

        let unpacked = temp.0.join("A_unpacked");
        unpack_archive(&source, Some(&unpacked), false).unwrap();
        let packed = temp.0.join("A_packed.FL2");
        let report = pack_archive(&unpacked, Some(&packed), false).unwrap();

        assert_eq!(report.packed_files, 2);
        assert_eq!(report.output_bytes, expected.len() as u64);
        assert_eq!(fs::read(&packed).unwrap(), expected);
    }

    #[test]
    fn pack_rebuilds_index_after_payload_size_change() {
        let temp = TempDirectory::new();
        let source = temp.0.join("A.FL2");
        let original = synthetic_archive(&[(b"ONE.AB", b"abc"), (b"TWO.TXT", b"defgh")]);
        fs::write(&source, original).unwrap();

        let unpacked = temp.0.join("A_unpacked");
        unpack_archive(&source, Some(&unpacked), false).unwrap();
        fs::write(unpacked.join("ONE.AB"), b"a longer replacement").unwrap();
        let packed = temp.0.join("A_packed.FL2");
        pack_archive(&unpacked, Some(&packed), false).unwrap();

        let rebuilt = fs::read(&packed).unwrap();
        let parsed = parse_archive(&rebuilt).unwrap();
        assert_eq!(parsed.entries[0].size, 20);
        assert_eq!(parsed.entries[1].data_offset, 32 + 20);
        assert_eq!(
            parsed.payload(&rebuilt, &parsed.entries[0]).unwrap(),
            b"a longer replacement"
        );
        assert_eq!(
            parsed.payload(&rebuilt, &parsed.entries[1]).unwrap(),
            b"defgh"
        );
    }

    #[test]
    fn pack_rejects_output_inside_unpacked_directory() {
        let temp = TempDirectory::new();
        let source = temp.0.join("A.FL2");
        fs::write(&source, synthetic_archive(&[(b"ONE.AB", b"abc")])).unwrap();
        let unpacked = temp.0.join("A_unpacked");
        unpack_archive(&source, Some(&unpacked), false).unwrap();

        let error = pack_archive(&unpacked, Some(&unpacked.join("bad.FL2")), false)
            .unwrap_err()
            .to_string();
        assert!(error.contains("must be outside unpacked directory"));
    }
}
