use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const XFL_MAGIC: u16 = 0x424c;
pub const XFL_VERSION: u8 = 1;
pub const HEADER_SIZE: usize = 12;
pub const ENTRY_SIZE: usize = 40;
pub const ENTRY_NAME_SIZE: usize = 32;
pub const XFL_MANIFEST_NAME: &str = ".xfl-manifest.json";

const XFL_MANIFEST_FORMAT: &str = "railsoft-xfl-manifest-v1";

#[derive(Debug)]
pub enum XflError {
    InvalidFormat(String),
    Io {
        context: String,
        source: io::Error,
    },
    Json {
        context: String,
        source: serde_json::Error,
    },
}

impl XflError {
    fn invalid(message: impl Into<String>) -> Self {
        Self::InvalidFormat(message.into())
    }

    fn io(context: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    fn json(context: impl Into<String>, source: serde_json::Error) -> Self {
        Self::Json {
            context: context.into(),
            source,
        }
    }
}

impl fmt::Display for XflError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat(message) => write!(formatter, "invalid XFL archive: {message}"),
            Self::Io { context, source } => write!(formatter, "{context}: {source}"),
            Self::Json { context, source } => write!(formatter, "{context}: {source}"),
        }
    }
}

impl Error for XflError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidFormat(_) => None,
            Self::Io { source, .. } => Some(source),
            Self::Json { source, .. } => Some(source),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XflHeader {
    pub version: u8,
    pub reserved: u8,
    pub table_size: u32,
    pub entry_count: u32,
    pub data_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XflEntry {
    pub name: String,
    pub name_was_escaped: bool,
    pub raw_name: Vec<u8>,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug)]
pub struct XflArchive<'a> {
    pub header: XflHeader,
    pub entries: Vec<XflEntry>,
    data: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtractStats {
    pub extracted_files: usize,
    pub extracted_bytes: u64,
    pub escaped_names: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackStats {
    pub packed_files: usize,
    pub packed_bytes: u64,
    pub used_manifest: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct XflManifest {
    format: String,
    version: u8,
    reserved: u8,
    entries: Vec<XflManifestEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct XflManifestEntry {
    path: String,
    raw_name_hex: String,
}

#[derive(Debug)]
struct PackEntry {
    raw_name: Vec<u8>,
    path: PathBuf,
    data: Vec<u8>,
}

impl<'a> XflArchive<'a> {
    /// Parse and validate an XFL archive from its complete byte representation.
    ///
    /// # Errors
    ///
    /// Returns [`XflError::InvalidFormat`] when the header, table, filename, or
    /// payload ranges violate the supported version 1 format.
    pub fn parse(data: &'a [u8]) -> Result<Self, XflError> {
        if data.len() < HEADER_SIZE {
            return Err(XflError::invalid(format!(
                "file is {} bytes, shorter than the {HEADER_SIZE}-byte header",
                data.len()
            )));
        }

        let magic = read_u16(data, 0)?;
        if magic != XFL_MAGIC {
            return Err(XflError::invalid(format!(
                "unexpected magic 0x{magic:04x}, expected 0x{XFL_MAGIC:04x} (LB)"
            )));
        }

        let version = data[2];
        if version != XFL_VERSION {
            return Err(XflError::invalid(format!(
                "unsupported version {version}, expected {XFL_VERSION}"
            )));
        }

        let reserved = data[3];
        if reserved != 0 {
            return Err(XflError::invalid(format!(
                "reserved header byte is 0x{reserved:02x}, expected 0"
            )));
        }

        let table_size = read_u32(data, 4)?;
        let entry_count = read_u32(data, 8)?;
        let expected_table_size = usize::try_from(entry_count)
            .ok()
            .and_then(|count| count.checked_mul(ENTRY_SIZE))
            .ok_or_else(|| XflError::invalid("entry table size overflows this platform"))?;
        let table_size_usize = usize::try_from(table_size)
            .map_err(|_| XflError::invalid("entry table size does not fit this platform"))?;

        if table_size_usize != expected_table_size {
            return Err(XflError::invalid(format!(
                "table size is {table_size} bytes, but {entry_count} entries require {expected_table_size} bytes"
            )));
        }

        let data_offset = HEADER_SIZE
            .checked_add(table_size_usize)
            .ok_or_else(|| XflError::invalid("data offset overflows this platform"))?;
        if data_offset > data.len() {
            return Err(XflError::invalid(format!(
                "entry table ends at 0x{data_offset:x}, beyond file size 0x{:x}",
                data.len()
            )));
        }

        let mut entries = Vec::with_capacity(entry_count as usize);
        for index in 0..entry_count as usize {
            let record_offset = HEADER_SIZE + index * ENTRY_SIZE;
            let raw_name_field = &data[record_offset..record_offset + ENTRY_NAME_SIZE];
            let name_end = raw_name_field
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(ENTRY_NAME_SIZE);
            if name_end == 0 {
                return Err(XflError::invalid(format!(
                    "entry {index} has an empty filename"
                )));
            }
            if raw_name_field[name_end..].iter().any(|byte| *byte != 0) {
                return Err(XflError::invalid(format!(
                    "entry {index} has nonzero filename padding"
                )));
            }

            let raw_name = raw_name_field[..name_end].to_vec();
            let (name, name_was_escaped) = decode_name(&raw_name);
            let offset = read_u32(data, record_offset + 32)?;
            let size = read_u32(data, record_offset + 36)?;
            let relative_end = u64::from(offset) + u64::from(size);
            let absolute_end = u64::try_from(data_offset)
                .ok()
                .and_then(|base| base.checked_add(relative_end))
                .ok_or_else(|| XflError::invalid(format!("entry {index} range overflows")))?;
            if absolute_end > data.len() as u64 {
                return Err(XflError::invalid(format!(
                    "entry {index} ({name}) ends at 0x{absolute_end:x}, beyond file size 0x{:x}",
                    data.len()
                )));
            }

            entries.push(XflEntry {
                name,
                name_was_escaped,
                raw_name,
                offset,
                size,
            });
        }

        Ok(Self {
            header: XflHeader {
                version,
                reserved,
                table_size,
                entry_count,
                data_offset,
            },
            entries,
            data,
        })
    }

    #[must_use]
    pub fn payload_size(&self) -> usize {
        self.data.len() - self.header.data_offset
    }

    #[must_use]
    pub fn entry_data(&self, entry: &XflEntry) -> &'a [u8] {
        let start = self.header.data_offset + entry.offset as usize;
        let end = start + entry.size as usize;
        &self.data[start..end]
    }

    /// Extract every validated entry and a repacking manifest below `output_dir`.
    ///
    /// # Errors
    ///
    /// Returns an error for unsafe or colliding paths, existing files when
    /// `force` is false, or any directory, file, or manifest write failure.
    pub fn extract_to(&self, output_dir: &Path, force: bool) -> Result<ExtractStats, XflError> {
        let mut destinations = Vec::with_capacity(self.entries.len());
        let mut seen_paths = HashSet::with_capacity(self.entries.len());

        for (index, entry) in self.entries.iter().enumerate() {
            let relative = safe_relative_path(&entry.name).map_err(|message| {
                XflError::invalid(format!("unsafe filename in entry {index}: {message}"))
            })?;
            let collision_key = path_key(&relative);
            if !seen_paths.insert(collision_key) {
                return Err(XflError::invalid(format!(
                    "duplicate output path for entry {index}: {}",
                    relative.display()
                )));
            }

            let destination = output_dir.join(&relative);
            if destination.exists() && !force {
                return Err(XflError::invalid(format!(
                    "output already exists (use --force to overwrite): {}",
                    destination.display()
                )));
            }
            destinations.push((relative, destination));
        }

        let manifest_path = output_dir.join(XFL_MANIFEST_NAME);
        if manifest_path.exists() && !force {
            return Err(XflError::invalid(format!(
                "output manifest already exists (use --force to overwrite): {}",
                manifest_path.display()
            )));
        }

        fs::create_dir_all(output_dir).map_err(|error| {
            XflError::io(
                format!("failed to create output directory {}", output_dir.display()),
                error,
            )
        })?;

        let mut extracted_bytes = 0_u64;
        let mut escaped_names = 0_usize;
        for (entry, (_, destination)) in self.entries.iter().zip(&destinations) {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    XflError::io(
                        format!("failed to create directory {}", parent.display()),
                        error,
                    )
                })?;
            }
            fs::write(destination, self.entry_data(entry)).map_err(|error| {
                XflError::io(format!("failed to write {}", destination.display()), error)
            })?;
            extracted_bytes += u64::from(entry.size);
            escaped_names += usize::from(entry.name_was_escaped);
        }

        let manifest = XflManifest {
            format: XFL_MANIFEST_FORMAT.to_owned(),
            version: self.header.version,
            reserved: self.header.reserved,
            entries: self
                .entries
                .iter()
                .zip(destinations)
                .map(|(entry, (relative, _))| XflManifestEntry {
                    path: path_to_slashes(&relative),
                    raw_name_hex: encode_hex(&entry.raw_name),
                })
                .collect(),
        };
        write_json(&manifest_path, &manifest)?;

        Ok(ExtractStats {
            extracted_files: self.entries.len(),
            extracted_bytes,
            escaped_names,
        })
    }
}

/// Build a complete XFL archive from a directory.
///
/// A manifest created by [`XflArchive::extract_to`] preserves original entry
/// order and raw filename bytes. Without a manifest, files are packed in
/// case-insensitive relative-path order using UTF-8 archive names.
///
/// # Errors
///
/// Returns an error when a manifest is invalid, a filename cannot be encoded,
/// an entry is missing, or an archive size exceeds the version 1 limits.
pub fn pack_directory(input_dir: &Path) -> Result<(Vec<u8>, PackStats), XflError> {
    if !input_dir.is_dir() {
        return Err(XflError::invalid(format!(
            "pack input is not a directory: {}",
            input_dir.display()
        )));
    }

    let manifest_path = input_dir.join(XFL_MANIFEST_NAME);
    let (version, reserved, mut entries, used_manifest) = if manifest_path.is_file() {
        let manifest = read_manifest(&manifest_path)?;
        let entries = entries_from_manifest(input_dir, &manifest)?;
        (manifest.version, manifest.reserved, entries, true)
    } else {
        (XFL_VERSION, 0, Vec::new(), false)
    };

    let disk_files = collect_payload_files(input_dir)?;
    let mut used_paths: HashSet<String> =
        entries.iter().map(|entry| path_key(&entry.path)).collect();
    for relative in disk_files {
        if used_paths.insert(path_key(&relative)) {
            let raw_name = path_to_slashes(&relative).into_bytes();
            validate_raw_name(&raw_name)?;
            let full_path = input_dir.join(&relative);
            let data = read_file(&full_path)?;
            entries.push(PackEntry {
                raw_name,
                path: relative,
                data,
            });
        }
    }

    if entries.is_empty() {
        return Err(XflError::invalid(format!(
            "input directory contains no payload files: {}",
            input_dir.display()
        )));
    }
    if !used_manifest {
        entries.sort_by(|left, right| path_key(&left.path).cmp(&path_key(&right.path)));
    }

    let entry_count =
        u32::try_from(entries.len()).map_err(|_| XflError::invalid("entry count exceeds u32"))?;
    let table_size_usize = entries
        .len()
        .checked_mul(ENTRY_SIZE)
        .ok_or_else(|| XflError::invalid("entry table size overflows this platform"))?;
    let table_size = u32::try_from(table_size_usize)
        .map_err(|_| XflError::invalid("entry table size exceeds u32"))?;

    let payload_size = entries.iter().try_fold(0_u64, |total, entry| {
        total
            .checked_add(entry.data.len() as u64)
            .ok_or_else(|| XflError::invalid("payload size overflows u64"))
    })?;
    if payload_size > u64::from(u32::MAX) {
        return Err(XflError::invalid(
            "payload size exceeds the u32 offset range",
        ));
    }

    let payload_size_usize = usize::try_from(payload_size)
        .map_err(|_| XflError::invalid("payload size does not fit this platform"))?;
    let capacity = HEADER_SIZE
        .checked_add(table_size_usize)
        .and_then(|size| size.checked_add(payload_size_usize))
        .ok_or_else(|| XflError::invalid("archive size overflows this platform"))?;
    let mut output = Vec::with_capacity(capacity);
    output.extend_from_slice(&XFL_MAGIC.to_le_bytes());
    output.push(version);
    output.push(reserved);
    output.extend_from_slice(&table_size.to_le_bytes());
    output.extend_from_slice(&entry_count.to_le_bytes());

    let mut offset = 0_u32;
    for entry in &entries {
        validate_raw_name(&entry.raw_name)?;
        let mut name_field = [0_u8; ENTRY_NAME_SIZE];
        name_field[..entry.raw_name.len()].copy_from_slice(&entry.raw_name);
        output.extend_from_slice(&name_field);
        output.extend_from_slice(&offset.to_le_bytes());
        let size = u32::try_from(entry.data.len())
            .map_err(|_| XflError::invalid("individual payload size exceeds u32"))?;
        output.extend_from_slice(&size.to_le_bytes());
        offset = offset
            .checked_add(size)
            .ok_or_else(|| XflError::invalid("payload offsets exceed u32"))?;
    }
    for entry in &entries {
        output.extend_from_slice(&entry.data);
    }

    Ok((
        output,
        PackStats {
            packed_files: entries.len(),
            packed_bytes: payload_size,
            used_manifest,
        },
    ))
}

fn read_manifest(path: &Path) -> Result<XflManifest, XflError> {
    let data = read_file(path)?;
    let manifest: XflManifest = serde_json::from_slice(&data)
        .map_err(|error| XflError::json(format!("failed to parse {}", path.display()), error))?;
    if manifest.format != XFL_MANIFEST_FORMAT {
        return Err(XflError::invalid(format!(
            "unsupported manifest format {:?} in {}",
            manifest.format,
            path.display()
        )));
    }
    if manifest.version != XFL_VERSION || manifest.reserved != 0 {
        return Err(XflError::invalid(format!(
            "manifest requests unsupported XFL version/reserved values: {}/{}",
            manifest.version, manifest.reserved
        )));
    }
    Ok(manifest)
}

fn entries_from_manifest(
    input_dir: &Path,
    manifest: &XflManifest,
) -> Result<Vec<PackEntry>, XflError> {
    let mut entries = Vec::with_capacity(manifest.entries.len());
    let mut seen_paths = HashSet::with_capacity(manifest.entries.len());
    let mut seen_names = HashSet::with_capacity(manifest.entries.len());

    for (index, item) in manifest.entries.iter().enumerate() {
        let relative = safe_relative_path(&item.path).map_err(|message| {
            XflError::invalid(format!("unsafe manifest path at entry {index}: {message}"))
        })?;
        if !seen_paths.insert(path_key(&relative)) {
            return Err(XflError::invalid(format!(
                "duplicate manifest path: {}",
                relative.display()
            )));
        }

        let raw_name = decode_hex(&item.raw_name_hex).map_err(|message| {
            XflError::invalid(format!(
                "invalid raw name at manifest entry {index}: {message}"
            ))
        })?;
        validate_raw_name(&raw_name)?;
        if !seen_names.insert(raw_name.clone()) {
            return Err(XflError::invalid(format!(
                "duplicate raw archive name at manifest entry {index}"
            )));
        }

        let full_path = input_dir.join(&relative);
        if !full_path.is_file() {
            return Err(XflError::invalid(format!(
                "manifest payload is missing: {}",
                full_path.display()
            )));
        }
        entries.push(PackEntry {
            raw_name,
            path: relative,
            data: read_file(&full_path)?,
        });
    }
    Ok(entries)
}

fn collect_payload_files(root: &Path) -> Result<Vec<PathBuf>, XflError> {
    fn visit(root: &Path, current: &Path, output: &mut Vec<PathBuf>) -> Result<(), XflError> {
        let read_dir = fs::read_dir(current).map_err(|error| {
            XflError::io(
                format!("failed to read directory {}", current.display()),
                error,
            )
        })?;
        for item in read_dir {
            let item = item.map_err(|error| {
                XflError::io(
                    format!("failed to enumerate directory {}", current.display()),
                    error,
                )
            })?;
            let path = item.path();
            let file_type = item.file_type().map_err(|error| {
                XflError::io(format!("failed to inspect {}", path.display()), error)
            })?;
            if file_type.is_symlink() {
                return Err(XflError::invalid(format!(
                    "symbolic links are not supported while packing: {}",
                    path.display()
                )));
            }
            if file_type.is_dir() {
                visit(root, &path, output)?;
            } else if file_type.is_file() {
                let relative = path.strip_prefix(root).map_err(|_| {
                    XflError::invalid(format!("path escaped pack root: {}", path.display()))
                })?;
                if relative != Path::new(XFL_MANIFEST_NAME) {
                    output.push(relative.to_path_buf());
                }
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    visit(root, root, &mut files)?;
    files.sort_by_key(|path| path_key(path));
    Ok(files)
}

fn validate_raw_name(raw_name: &[u8]) -> Result<(), XflError> {
    if raw_name.is_empty() {
        return Err(XflError::invalid("archive filename is empty"));
    }
    if raw_name.len() > ENTRY_NAME_SIZE {
        return Err(XflError::invalid(format!(
            "archive filename is {} bytes, exceeding the {ENTRY_NAME_SIZE}-byte field",
            raw_name.len()
        )));
    }
    if raw_name.contains(&0) {
        return Err(XflError::invalid("archive filename contains NUL"));
    }
    Ok(())
}

fn read_u16(data: &[u8], offset: usize) -> Result<u16, XflError> {
    let bytes = data
        .get(offset..offset + 2)
        .ok_or_else(|| XflError::invalid(format!("missing u16 at 0x{offset:x}")))?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32, XflError> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or_else(|| XflError::invalid(format!("missing u32 at 0x{offset:x}")))?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn decode_name(raw_name: &[u8]) -> (String, bool) {
    if let Ok(name) = std::str::from_utf8(raw_name) {
        return (name.to_owned(), false);
    }

    let mut escaped = String::with_capacity(raw_name.len() * 3);
    for byte in raw_name {
        if byte.is_ascii_alphanumeric() || matches!(*byte, b'.' | b'_' | b'-' | b' ') {
            escaped.push(char::from(*byte));
        } else {
            use std::fmt::Write as _;
            let _ = write!(escaped, "%{byte:02X}");
        }
    }
    (escaped, true)
}

fn safe_relative_path(name: &str) -> Result<PathBuf, String> {
    let mut path = PathBuf::new();
    for component in name.split(['/', '\\']) {
        if component.is_empty() || component == "." || component == ".." {
            return Err(format!("invalid path component in {name:?}"));
        }
        if component.contains(':') || component.chars().any(char::is_control) {
            return Err(format!("unsupported path component {component:?}"));
        }
        if component.ends_with(['.', ' ']) {
            return Err(format!(
                "path component is not portable to Windows: {component:?}"
            ));
        }
        if is_windows_device_name(component) {
            return Err(format!("reserved Windows device name: {component:?}"));
        }
        path.push(component);
    }

    if path.as_os_str().is_empty() {
        return Err("empty output path".to_owned());
    }
    Ok(path)
}

fn is_windows_device_name(component: &str) -> bool {
    let stem = component.split('.').next().unwrap_or(component);
    let upper = stem.to_ascii_uppercase();
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || upper
            .strip_prefix("COM")
            .or_else(|| upper.strip_prefix("LPT"))
            .is_some_and(|suffix| {
                matches!(suffix, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            })
}

fn path_key(path: &Path) -> String {
    path_to_slashes(path).to_lowercase()
}

fn path_to_slashes(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(encoded, "{byte:02X}");
    }
    encoded
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if !value.len().is_multiple_of(2) {
        return Err("hex string has an odd length".to_owned());
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = decode_nibble(pair[0])?;
            let low = decode_nibble(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn decode_nibble(value: u8) -> Result<u8, String> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(format!("invalid hex digit {:?}", char::from(value))),
    }
}

fn read_file(path: &Path) -> Result<Vec<u8>, XflError> {
    fs::read(path)
        .map_err(|error| XflError::io(format!("failed to read {}", path.display()), error))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), XflError> {
    let mut data = serde_json::to_vec_pretty(value).map_err(|error| {
        XflError::json(format!("failed to serialize {}", path.display()), error)
    })?;
    data.push(b'\n');
    fs::write(path, data)
        .map_err(|error| XflError::io(format!("failed to write {}", path.display()), error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn build_archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let table_size = entries.len() * ENTRY_SIZE;
        let table_size_u32 = u32::try_from(table_size).expect("test table should fit in u32");
        let entry_count = u32::try_from(entries.len()).expect("test count should fit in u32");
        let mut archive = Vec::new();
        archive.extend_from_slice(&XFL_MAGIC.to_le_bytes());
        archive.push(XFL_VERSION);
        archive.push(0);
        archive.extend_from_slice(&table_size_u32.to_le_bytes());
        archive.extend_from_slice(&entry_count.to_le_bytes());

        let mut offset = 0_u32;
        for (name, payload) in entries {
            assert!(name.len() <= ENTRY_NAME_SIZE);
            let mut raw_name = [0_u8; ENTRY_NAME_SIZE];
            raw_name[..name.len()].copy_from_slice(name.as_bytes());
            archive.extend_from_slice(&raw_name);
            archive.extend_from_slice(&offset.to_le_bytes());
            let payload_size =
                u32::try_from(payload.len()).expect("test payload should fit in u32");
            archive.extend_from_slice(&payload_size.to_le_bytes());
            offset = offset
                .checked_add(payload_size)
                .expect("test archive offsets should fit in u32");
        }
        for (_, payload) in entries {
            archive.extend_from_slice(payload);
        }
        archive
    }

    fn temporary_directory(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "railsoft-xfl-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    #[test]
    fn parses_entries_and_payloads() {
        let bytes = build_archive(&[("0000.gsc", b"abc"), ("dir/0001.gsc", b"xy")]);
        let archive = XflArchive::parse(&bytes).expect("archive should parse");

        assert_eq!(archive.header.entry_count, 2);
        assert_eq!(archive.header.table_size, 80);
        assert_eq!(archive.header.data_offset, 92);
        assert_eq!(archive.entries[0].offset, 0);
        assert_eq!(archive.entries[1].offset, 3);
        assert_eq!(archive.entry_data(&archive.entries[0]), b"abc");
        assert_eq!(archive.entry_data(&archive.entries[1]), b"xy");
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = build_archive(&[("a.gsc", b"abc")]);
        bytes[0] = 0;
        let error = XflArchive::parse(&bytes).expect_err("bad magic must fail");
        assert!(error.to_string().contains("unexpected magic"));
    }

    #[test]
    fn rejects_mismatched_table_size() {
        let mut bytes = build_archive(&[("a.gsc", b"abc")]);
        bytes[4..8].copy_from_slice(&39_u32.to_le_bytes());
        let error = XflArchive::parse(&bytes).expect_err("bad table size must fail");
        assert!(error.to_string().contains("table size"));
    }

    #[test]
    fn rejects_out_of_bounds_entry() {
        let mut bytes = build_archive(&[("a.gsc", b"abc")]);
        bytes[48..52].copy_from_slice(&4_u32.to_le_bytes());
        let error = XflArchive::parse(&bytes).expect_err("bad entry range must fail");
        assert!(error.to_string().contains("beyond file size"));
    }

    #[test]
    fn rejects_unsafe_paths() {
        assert!(safe_relative_path("../escape.gsc").is_err());
        assert!(safe_relative_path("C:/escape.gsc").is_err());
        assert!(safe_relative_path("NUL.txt").is_err());
        assert_eq!(
            safe_relative_path("script\\0001.gsc").expect("path should be safe"),
            PathBuf::from("script").join("0001.gsc")
        );
    }

    #[test]
    fn manifest_roundtrip_is_byte_exact() {
        let bytes = build_archive(&[("0001.gsc", b"first"), ("0000.gsc", b"second")]);
        let archive = XflArchive::parse(&bytes).expect("archive should parse");
        let directory = temporary_directory("roundtrip");
        archive
            .extract_to(&directory, false)
            .expect("archive should extract");

        let (rebuilt, stats) = pack_directory(&directory).expect("directory should pack");
        assert!(stats.used_manifest);
        assert_eq!(rebuilt, bytes);

        fs::remove_dir_all(directory).expect("temporary directory should be removable");
    }

    #[test]
    fn pack_without_manifest_uses_sorted_paths() {
        let directory = temporary_directory("sorted");
        fs::create_dir_all(&directory).expect("temporary directory should be created");
        fs::write(directory.join("b.gsc"), b"b").expect("test payload should be written");
        fs::write(directory.join("a.gsc"), b"a").expect("test payload should be written");

        let (bytes, stats) = pack_directory(&directory).expect("directory should pack");
        let archive = XflArchive::parse(&bytes).expect("packed archive should parse");
        assert!(!stats.used_manifest);
        assert_eq!(archive.entries[0].name, "a.gsc");
        assert_eq!(archive.entries[1].name, "b.gsc");

        fs::remove_dir_all(directory).expect("temporary directory should be removable");
    }
}
