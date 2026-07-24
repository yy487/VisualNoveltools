use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

pub const MANIFEST_FILENAME: &str = ".yuno_manifest.json";
const MANIFEST_FORMAT: &str = "yuno-pc98-ai5-archive-v1";
const HEADER_SIZE: usize = 4;
const ENTRY_SIZE: usize = 20;
const NAME_SIZE: usize = 14;
const ENGINE_TABLE_OVERREAD: usize = 4;

#[derive(Debug, Error)]
pub enum YunoError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("JSON error at {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid YU-NO archive: {0}")]
    Invalid(String),
    #[error("output already exists: {0}")]
    OutputExists(PathBuf),
}

pub type Result<T> = std::result::Result<T, YunoError>;

#[derive(Debug, Clone)]
pub struct UnpackRequest {
    pub source: PathBuf,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PackRequest {
    pub unpacked: PathBuf,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
pub struct UnpackStats {
    pub archive: String,
    pub extracted_files: usize,
    pub extracted_bytes: u64,
    pub manifest: PathBuf,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PackStats {
    pub archive: String,
    pub packed_files: usize,
    pub changed_files: usize,
    pub unchanged_files: usize,
    pub output_bytes: usize,
    pub byte_exact: bool,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
struct ArchiveEntry {
    name: String,
    raw_name: [u8; NAME_SIZE],
    offset: u32,
    size: u16,
    sha256: String,
}

#[derive(Debug)]
struct ParsedArchive {
    archive_name: String,
    header: [u8; HEADER_SIZE],
    count: u16,
    key: u16,
    data_offset: usize,
    bytes: Vec<u8>,
    entries: Vec<ArchiveEntry>,
    source_sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArchiveManifest {
    format: String,
    version: u32,
    archive_name: String,
    header_hex: String,
    count: u16,
    key: u16,
    rotate: u8,
    data_offset: u32,
    source_size: u64,
    source_sha256: String,
    entries: Vec<ManifestEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestEntry {
    index: u16,
    name: String,
    raw_name_hex: String,
    offset: u32,
    size: u16,
    resource_type: String,
    sha256: String,
}

#[derive(Debug)]
struct PreparedPack {
    output: PathBuf,
    bytes: Vec<u8>,
    stats: PackStats,
}

fn io_error(path: &Path, source: io::Error) -> YunoError {
    YunoError::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn json_error(path: &Path, source: serde_json::Error) -> YunoError {
    YunoError::Json {
        path: path.to_path_buf(),
        source,
    }
}

fn read_file(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|source| io_error(path, source))
}

fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn validate_sha256(value: &str, context: &str) -> Result<()> {
    if value.len() != 64 || !value.as_bytes().iter().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(YunoError::Invalid(format!(
            "{context} is not a 64-digit SHA-256"
        )));
    }
    Ok(())
}

fn validate_archive_name(name: &str) -> Result<()> {
    let bytes = name.as_bytes();
    if bytes.len() != 6 || &bytes[..5] != b"YUNO_" || !(b'A'..=b'Q').contains(&bytes[5]) {
        return Err(YunoError::Invalid(format!(
            "unsupported archive name {name:?}; expected YUNO_A through YUNO_Q"
        )));
    }
    Ok(())
}

fn archive_name_from_path(path: &Path) -> Result<String> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            YunoError::Invalid(format!(
                "archive path has no UTF-8 filename: {}",
                path.display()
            ))
        })?;
    validate_archive_name(name)?;
    Ok(name.to_owned())
}

fn data_offset(count: u16) -> Result<usize> {
    usize::from(count)
        .checked_mul(ENTRY_SIZE)
        .and_then(|value| value.checked_add(HEADER_SIZE))
        .ok_or_else(|| YunoError::Invalid("archive table size overflows usize".to_owned()))
}

fn decrypt_table(encoded: &[u8], key: u16) -> Vec<u8> {
    let rotate = u32::from((key & 0x00ff) as u8) & 7;
    let mut xor = (key >> 8) as u8;
    encoded
        .iter()
        .map(|byte| {
            let decoded = byte.rotate_right(rotate) ^ xor;
            xor = xor.wrapping_add(1);
            decoded
        })
        .collect()
}

fn encrypt_table(decoded: &[u8], key: u16) -> Vec<u8> {
    let rotate = u32::from((key & 0x00ff) as u8) & 7;
    let mut xor = (key >> 8) as u8;
    decoded
        .iter()
        .map(|byte| {
            let encoded = (byte ^ xor).rotate_left(rotate);
            xor = xor.wrapping_add(1);
            encoded
        })
        .collect()
}

fn is_windows_reserved(name: &str) -> bool {
    let stem = name
        .split('.')
        .next()
        .unwrap_or(name)
        .trim_end_matches([' ', '.'])
        .to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || stem
            .strip_prefix("COM")
            .or_else(|| stem.strip_prefix("LPT"))
            .is_some_and(|number| number.len() == 1 && matches!(number.as_bytes()[0], b'1'..=b'9'))
}

fn validate_host_name(name: &str) -> Result<()> {
    let path = Path::new(name);
    let mut components = path.components();
    if name.is_empty()
        || name.ends_with([' ', '.'])
        || is_windows_reserved(name)
        || !matches!(components.next(), Some(Component::Normal(_)))
        || components.next().is_some()
        || name.chars().any(|character| {
            matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            )
        })
    {
        return Err(YunoError::Invalid(format!(
            "unsafe archive entry name {name:?}"
        )));
    }
    Ok(())
}

fn decode_name(raw_name: &[u8; NAME_SIZE], index: usize) -> Result<String> {
    let nul = raw_name.iter().position(|byte| *byte == 0).ok_or_else(|| {
        YunoError::Invalid(format!(
            "entry {index} name has no NUL terminator in its 14-byte field"
        ))
    })?;
    if raw_name[nul..].iter().any(|byte| *byte != 0) {
        return Err(YunoError::Invalid(format!(
            "entry {index} name has nonzero bytes after its NUL terminator"
        )));
    }
    let decoded = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(&raw_name[..nul])
        .ok_or_else(|| {
            YunoError::Invalid(format!(
                "entry {index} name is not valid CP932/Shift-JIS: {}",
                hex::encode(&raw_name[..nul])
            ))
        })?
        .into_owned();
    validate_host_name(&decoded)?;
    Ok(decoded)
}

fn resource_type(name: &str) -> String {
    Path::new(name)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_uppercase())
        .unwrap_or_default()
}

fn parse_archive_bytes(
    source: PathBuf,
    archive_name: String,
    bytes: Vec<u8>,
) -> Result<ParsedArchive> {
    validate_archive_name(&archive_name)?;
    if bytes.len() < HEADER_SIZE {
        return Err(YunoError::Invalid(format!(
            "{} is shorter than the 4-byte header",
            source.display()
        )));
    }

    let header: [u8; HEADER_SIZE] = bytes[..HEADER_SIZE]
        .try_into()
        .expect("slice length was checked");
    let count = u16::from_le_bytes([header[0], header[1]]);
    let key = u16::from_le_bytes([header[2], header[3]]);
    if count == 0 {
        return Err(YunoError::Invalid(format!(
            "{} has an empty archive table",
            source.display()
        )));
    }

    let data_offset = data_offset(count)?;
    let engine_read_end = data_offset
        .checked_add(ENGINE_TABLE_OVERREAD)
        .ok_or_else(|| YunoError::Invalid("engine table read range overflows usize".to_owned()))?;
    if bytes.len() < engine_read_end {
        return Err(YunoError::Invalid(format!(
            "{} is truncated: AI5X reads through 0x{engine_read_end:X}, file length is 0x{:X}",
            source.display(),
            bytes.len()
        )));
    }

    let decoded = decrypt_table(&bytes[HEADER_SIZE..data_offset], key);
    let mut entries = Vec::with_capacity(usize::from(count));
    let mut expected_offset = 0u64;
    let mut names = HashSet::new();

    for index in 0..usize::from(count) {
        let start = index * ENTRY_SIZE;
        let record = &decoded[start..start + ENTRY_SIZE];
        let raw_name: [u8; NAME_SIZE] = record[..NAME_SIZE]
            .try_into()
            .expect("record size is fixed");
        let name = decode_name(&raw_name, index)?;
        if !names.insert(name.to_uppercase()) {
            return Err(YunoError::Invalid(format!(
                "{} contains a duplicate Windows filename: {name}",
                source.display()
            )));
        }
        let offset = u32::from_le_bytes(record[14..18].try_into().expect("record size is fixed"));
        let size = u16::from_le_bytes(record[18..20].try_into().expect("record size is fixed"));
        if u64::from(offset) != expected_offset {
            return Err(YunoError::Invalid(format!(
                "{} entry {index} ({name}) starts at 0x{offset:X}, expected contiguous offset 0x{expected_offset:X}",
                source.display()
            )));
        }
        let payload_start = data_offset
            .checked_add(usize::try_from(offset).map_err(|_| {
                YunoError::Invalid(format!("entry {index} offset does not fit usize"))
            })?)
            .ok_or_else(|| YunoError::Invalid(format!("entry {index} offset overflows usize")))?;
        let payload_end = payload_start
            .checked_add(usize::from(size))
            .ok_or_else(|| {
                YunoError::Invalid(format!("entry {index} payload range overflows usize"))
            })?;
        let payload = bytes.get(payload_start..payload_end).ok_or_else(|| {
            YunoError::Invalid(format!(
                "{} entry {index} ({name}) range 0x{payload_start:X}..0x{payload_end:X} exceeds file length 0x{:X}",
                source.display(),
                bytes.len()
            ))
        })?;
        expected_offset = u64::from(offset) + u64::from(size);
        entries.push(ArchiveEntry {
            name,
            raw_name,
            offset,
            size,
            sha256: sha256_hex(payload),
        });
    }

    let payload_size = bytes.len() - data_offset;
    if expected_offset != payload_size as u64 {
        return Err(YunoError::Invalid(format!(
            "{} directory covers 0x{expected_offset:X} payload bytes, but 0x{payload_size:X} bytes remain after the table",
            source.display()
        )));
    }

    let source_sha256 = sha256_hex(&bytes);
    Ok(ParsedArchive {
        archive_name,
        header,
        count,
        key,
        data_offset,
        bytes,
        entries,
        source_sha256,
    })
}

fn parse_archive(path: &Path) -> Result<ParsedArchive> {
    let archive_name = archive_name_from_path(path)?;
    parse_archive_bytes(path.to_path_buf(), archive_name, read_file(path)?)
}

fn manifest_for_archive(archive: &ParsedArchive) -> ArchiveManifest {
    ArchiveManifest {
        format: MANIFEST_FORMAT.to_owned(),
        version: 1,
        archive_name: archive.archive_name.clone(),
        header_hex: hex::encode(archive.header),
        count: archive.count,
        key: archive.key,
        rotate: (archive.key & 0x00ff) as u8 & 7,
        data_offset: archive.data_offset as u32,
        source_size: archive.bytes.len() as u64,
        source_sha256: archive.source_sha256.clone(),
        entries: archive
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| ManifestEntry {
                index: index as u16,
                name: entry.name.clone(),
                raw_name_hex: hex::encode(entry.raw_name),
                offset: entry.offset,
                size: entry.size,
                resource_type: resource_type(&entry.name),
                sha256: entry.sha256.clone(),
            })
            .collect(),
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| io_error(path, source))?;
    file.write_all(bytes)
        .map_err(|source| io_error(path, source))?;
    file.flush().map_err(|source| io_error(path, source))
}

fn write_unpacked(archive: &ParsedArchive, output: &Path) -> Result<UnpackStats> {
    if output.exists() {
        return Err(YunoError::OutputExists(output.to_path_buf()));
    }
    fs::create_dir(output).map_err(|source| io_error(output, source))?;

    let write_result = (|| {
        for entry in &archive.entries {
            let start = archive.data_offset + entry.offset as usize;
            let end = start + usize::from(entry.size);
            write_new(&output.join(&entry.name), &archive.bytes[start..end])?;
        }

        let manifest_path = output.join(MANIFEST_FILENAME);
        let mut json = serde_json::to_vec_pretty(&manifest_for_archive(archive))
            .map_err(|source| json_error(&manifest_path, source))?;
        json.push(b'\n');
        write_new(&manifest_path, &json)?;

        Ok(UnpackStats {
            archive: archive.archive_name.clone(),
            extracted_files: archive.entries.len(),
            extracted_bytes: (archive.bytes.len() - archive.data_offset) as u64,
            manifest: manifest_path,
            output: output.to_path_buf(),
        })
    })();

    if write_result.is_err() {
        let _ = fs::remove_dir_all(output);
    }
    write_result
}

pub fn unpack_archives(requests: &[UnpackRequest]) -> Result<Vec<UnpackStats>> {
    if requests.is_empty() {
        return Err(YunoError::Invalid("no input archives supplied".to_owned()));
    }

    let mut outputs = HashSet::new();
    for request in requests {
        if !outputs.insert(request.output.clone()) {
            return Err(YunoError::Invalid(format!(
                "multiple inputs target the same output directory: {}",
                request.output.display()
            )));
        }
        if request.output.exists() {
            return Err(YunoError::OutputExists(request.output.clone()));
        }
    }

    let parsed: Vec<_> = requests
        .iter()
        .map(|request| parse_archive(&request.source))
        .collect::<Result<_>>()?;

    let mut stats = Vec::with_capacity(requests.len());
    let mut created = Vec::new();
    for (archive, request) in parsed.iter().zip(requests) {
        match write_unpacked(archive, &request.output) {
            Ok(item) => {
                created.push(request.output.clone());
                stats.push(item);
            }
            Err(error) => {
                for path in created.iter().rev() {
                    let _ = fs::remove_dir_all(path);
                }
                return Err(error);
            }
        }
    }
    Ok(stats)
}

fn parse_hex_array<const N: usize>(value: &str, context: &str) -> Result<[u8; N]> {
    let decoded = hex::decode(value)
        .map_err(|error| YunoError::Invalid(format!("{context} is invalid hex: {error}")))?;
    decoded.try_into().map_err(|decoded: Vec<u8>| {
        YunoError::Invalid(format!(
            "{context} is {} bytes, expected {N}",
            decoded.len()
        ))
    })
}

fn read_manifest(directory: &Path) -> Result<ArchiveManifest> {
    let path = directory.join(MANIFEST_FILENAME);
    let metadata = fs::symlink_metadata(&path).map_err(|source| io_error(&path, source))?;
    if !metadata.file_type().is_file() {
        return Err(YunoError::Invalid(format!(
            "manifest is not a regular file: {}",
            path.display()
        )));
    }
    serde_json::from_slice(&read_file(&path)?).map_err(|source| json_error(&path, source))
}

fn validate_manifest(directory: &Path, manifest: &ArchiveManifest) -> Result<[u8; HEADER_SIZE]> {
    if manifest.format != MANIFEST_FORMAT || manifest.version != 1 {
        return Err(YunoError::Invalid(format!(
            "unsupported manifest format/version in {}: {:?}/{}",
            directory.display(),
            manifest.format,
            manifest.version
        )));
    }
    validate_archive_name(&manifest.archive_name)?;
    let directory_name = directory
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            YunoError::Invalid(format!(
                "unpacked directory has no UTF-8 filename: {}",
                directory.display()
            ))
        })?;
    if directory_name != manifest.archive_name {
        return Err(YunoError::Invalid(format!(
            "directory name {directory_name:?} does not match manifest archive_name {:?}",
            manifest.archive_name
        )));
    }

    validate_sha256(&manifest.source_sha256, "manifest source_sha256")?;
    let header = parse_hex_array::<HEADER_SIZE>(&manifest.header_hex, "manifest header_hex")?;
    let header_count = u16::from_le_bytes([header[0], header[1]]);
    let header_key = u16::from_le_bytes([header[2], header[3]]);
    if header_count != manifest.count || header_key != manifest.key {
        return Err(YunoError::Invalid(
            "manifest header_hex does not match count/key".to_owned(),
        ));
    }
    if manifest.rotate != (manifest.key & 0x00ff) as u8 & 7 {
        return Err(YunoError::Invalid(
            "manifest rotate does not match the low byte of key".to_owned(),
        ));
    }
    let expected_data_offset = data_offset(manifest.count)?;
    if manifest.data_offset as usize != expected_data_offset {
        return Err(YunoError::Invalid(format!(
            "manifest data_offset is 0x{:X}, expected 0x{expected_data_offset:X}",
            manifest.data_offset
        )));
    }
    if manifest.entries.len() != usize::from(manifest.count) {
        return Err(YunoError::Invalid(format!(
            "manifest has {} entries, header count is {}",
            manifest.entries.len(),
            manifest.count
        )));
    }

    let mut names = HashSet::new();
    let mut expected_offset = 0u64;
    for (index, entry) in manifest.entries.iter().enumerate() {
        if usize::from(entry.index) != index {
            return Err(YunoError::Invalid(format!(
                "manifest entry {index} has index {}",
                entry.index
            )));
        }
        let raw_name = parse_hex_array::<NAME_SIZE>(
            &entry.raw_name_hex,
            &format!("manifest entry {index} raw_name_hex"),
        )?;
        let decoded_name = decode_name(&raw_name, index)?;
        if decoded_name != entry.name {
            return Err(YunoError::Invalid(format!(
                "manifest entry {index} name {:?} does not match raw name {:?}",
                entry.name, decoded_name
            )));
        }
        if !names.insert(entry.name.to_uppercase()) {
            return Err(YunoError::Invalid(format!(
                "manifest contains duplicate Windows filename {:?}",
                entry.name
            )));
        }
        if entry.resource_type != resource_type(&entry.name) {
            return Err(YunoError::Invalid(format!(
                "manifest entry {index} resource_type does not match its extension"
            )));
        }
        if u64::from(entry.offset) != expected_offset {
            return Err(YunoError::Invalid(format!(
                "manifest entry {index} offset is 0x{:X}, expected 0x{expected_offset:X}",
                entry.offset
            )));
        }
        validate_sha256(&entry.sha256, &format!("manifest entry {index} sha256"))?;
        expected_offset = u64::from(entry.offset) + u64::from(entry.size);
    }
    let expected_source_size = expected_data_offset as u64 + expected_offset;
    if manifest.source_size != expected_source_size {
        return Err(YunoError::Invalid(format!(
            "manifest source_size is 0x{:X}, table and entries require 0x{expected_source_size:X}",
            manifest.source_size
        )));
    }
    Ok(header)
}

fn validate_directory_inventory(directory: &Path, manifest: &ArchiveManifest) -> Result<()> {
    let mut expected: HashSet<String> = manifest
        .entries
        .iter()
        .map(|entry| entry.name.to_uppercase())
        .collect();
    expected.insert(MANIFEST_FILENAME.to_uppercase());

    let mut actual = HashSet::new();
    let reader = fs::read_dir(directory).map_err(|source| io_error(directory, source))?;
    for item in reader {
        let item = item.map_err(|source| io_error(directory, source))?;
        let path = item.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| io_error(&path, source))?;
        if !metadata.file_type().is_file() {
            return Err(YunoError::Invalid(format!(
                "unpacked directory contains a non-regular entry: {}",
                path.display()
            )));
        }
        let name = item.file_name().into_string().map_err(|_| {
            YunoError::Invalid(format!(
                "unpacked directory contains a non-UTF-8 filename: {}",
                path.display()
            ))
        })?;
        if !actual.insert(name.to_uppercase()) {
            return Err(YunoError::Invalid(format!(
                "unpacked directory contains case-colliding names: {name}"
            )));
        }
    }

    if actual != expected {
        let mut missing: Vec<_> = expected.difference(&actual).cloned().collect();
        let mut extra: Vec<_> = actual.difference(&expected).cloned().collect();
        missing.sort();
        extra.sort();
        return Err(YunoError::Invalid(format!(
            "unpacked directory inventory changed; missing={missing:?}, extra={extra:?}"
        )));
    }
    Ok(())
}

fn prepare_pack(request: &PackRequest) -> Result<PreparedPack> {
    if request.output.exists() {
        return Err(YunoError::OutputExists(request.output.clone()));
    }
    let manifest = read_manifest(&request.unpacked)?;
    let header = validate_manifest(&request.unpacked, &manifest)?;
    validate_directory_inventory(&request.unpacked, &manifest)?;

    let mut decoded_table = vec![0u8; usize::from(manifest.count) * ENTRY_SIZE];
    let mut payload = Vec::new();
    let mut changed_files = 0usize;

    for (index, entry) in manifest.entries.iter().enumerate() {
        let path = request.unpacked.join(&entry.name);
        let metadata = fs::symlink_metadata(&path).map_err(|source| io_error(&path, source))?;
        if !metadata.file_type().is_file() {
            return Err(YunoError::Invalid(format!(
                "resource is not a regular file: {}",
                path.display()
            )));
        }
        let resource = read_file(&path)?;
        let size = u16::try_from(resource.len()).map_err(|_| {
            YunoError::Invalid(format!(
                "resource {} is {} bytes; archive length field allows at most 65535 bytes",
                path.display(),
                resource.len()
            ))
        })?;
        let offset = u32::try_from(payload.len()).map_err(|_| {
            YunoError::Invalid("rebuilt payload exceeds the 32-bit offset field".to_owned())
        })?;
        let raw_name = parse_hex_array::<NAME_SIZE>(
            &entry.raw_name_hex,
            &format!("manifest entry {index} raw_name_hex"),
        )?;
        let record_start = index * ENTRY_SIZE;
        decoded_table[record_start..record_start + NAME_SIZE].copy_from_slice(&raw_name);
        decoded_table[record_start + 14..record_start + 18].copy_from_slice(&offset.to_le_bytes());
        decoded_table[record_start + 18..record_start + 20].copy_from_slice(&size.to_le_bytes());

        if !sha256_hex(&resource).eq_ignore_ascii_case(&entry.sha256) {
            changed_files += 1;
        }
        payload.extend_from_slice(&resource);
    }
    if payload.len() < ENGINE_TABLE_OVERREAD {
        return Err(YunoError::Invalid(format!(
            "rebuilt payload is {} bytes; AI5X overreads 4 bytes after the table",
            payload.len()
        )));
    }

    let mut bytes = Vec::with_capacity(HEADER_SIZE + decoded_table.len() + payload.len());
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(&encrypt_table(&decoded_table, manifest.key));
    bytes.extend_from_slice(&payload);

    let reparsed = parse_archive_bytes(
        request.output.clone(),
        manifest.archive_name.clone(),
        bytes.clone(),
    )?;
    if reparsed.entries.len() != manifest.entries.len() {
        return Err(YunoError::Invalid(
            "rebuilt archive verification changed the entry count".to_owned(),
        ));
    }
    for (index, (entry, verified)) in manifest.entries.iter().zip(&reparsed.entries).enumerate() {
        let resource = read_file(&request.unpacked.join(&entry.name))?;
        if verified.name != entry.name || verified.sha256 != sha256_hex(&resource) {
            return Err(YunoError::Invalid(format!(
                "rebuilt archive verification failed at entry {index} ({})",
                entry.name
            )));
        }
    }

    let byte_exact = sha256_hex(&bytes).eq_ignore_ascii_case(&manifest.source_sha256);
    if changed_files == 0 && !byte_exact {
        return Err(YunoError::Invalid(
            "unchanged resources did not rebuild byte-exactly; manifest is inconsistent".to_owned(),
        ));
    }
    let packed_files = manifest.entries.len();
    let stats = PackStats {
        archive: manifest.archive_name,
        packed_files,
        changed_files,
        unchanged_files: packed_files - changed_files,
        output_bytes: bytes.len(),
        byte_exact,
        output: request.output.clone(),
    };
    Ok(PreparedPack {
        output: request.output.clone(),
        bytes,
        stats,
    })
}

pub fn pack_archives(requests: &[PackRequest]) -> Result<Vec<PackStats>> {
    if requests.is_empty() {
        return Err(YunoError::Invalid(
            "no unpacked archive directories supplied".to_owned(),
        ));
    }
    let mut outputs = HashSet::new();
    for request in requests {
        if !outputs.insert(request.output.clone()) {
            return Err(YunoError::Invalid(format!(
                "multiple inputs target the same output file: {}",
                request.output.display()
            )));
        }
        if request.output.exists() {
            return Err(YunoError::OutputExists(request.output.clone()));
        }
    }

    let prepared: Vec<_> = requests.iter().map(prepare_pack).collect::<Result<_>>()?;
    let mut written = Vec::new();
    let mut stats = Vec::with_capacity(prepared.len());
    for archive in prepared {
        if let Err(error) = write_new(&archive.output, &archive.bytes) {
            for path in written.iter().rev() {
                let _ = fs::remove_file(path);
            }
            return Err(error);
        }
        written.push(archive.output.clone());
        stats.push(archive.stats);
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_name(name: &str) -> [u8; NAME_SIZE] {
        let mut result = [0u8; NAME_SIZE];
        result[..name.len()].copy_from_slice(name.as_bytes());
        result
    }

    fn synthetic_archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let count = entries.len() as u16;
        let key = 0x5501u16;
        let mut table = vec![0u8; entries.len() * ENTRY_SIZE];
        let mut payload = Vec::new();
        for (index, (name, resource)) in entries.iter().enumerate() {
            let start = index * ENTRY_SIZE;
            table[start..start + NAME_SIZE].copy_from_slice(&raw_name(name));
            table[start + 14..start + 18].copy_from_slice(&(payload.len() as u32).to_le_bytes());
            table[start + 18..start + 20].copy_from_slice(&(resource.len() as u16).to_le_bytes());
            payload.extend_from_slice(resource);
        }
        let mut archive = Vec::new();
        archive.extend_from_slice(&count.to_le_bytes());
        archive.extend_from_slice(&key.to_le_bytes());
        archive.extend_from_slice(&encrypt_table(&table, key));
        archive.extend_from_slice(&payload);
        archive
    }

    #[test]
    fn table_cipher_round_trips() {
        let data: Vec<_> = (0..=255).collect();
        let encrypted = encrypt_table(&data, 0x5501);
        assert_ne!(encrypted, data);
        assert_eq!(decrypt_table(&encrypted, 0x5501), data);
    }

    #[test]
    fn parses_contiguous_archive() {
        let bytes = synthetic_archive(&[("A.MES", b"abcd"), ("B.GP4", b"payload")]);
        let parsed = parse_archive_bytes(PathBuf::from("YUNO_A"), "YUNO_A".into(), bytes)
            .expect("valid archive");
        assert_eq!(parsed.entries.len(), 2);
        assert_eq!(parsed.entries[0].offset, 0);
        assert_eq!(parsed.entries[0].size, 4);
        assert_eq!(parsed.entries[1].offset, 4);
        assert_eq!(parsed.entries[1].size, 7);
    }

    #[test]
    fn rejects_truncated_engine_overread() {
        let bytes = synthetic_archive(&[("A.MES", b"abc")]);
        let error = parse_archive_bytes(PathBuf::from("YUNO_A"), "YUNO_A".into(), bytes)
            .expect_err("payload shorter than engine overread must fail");
        assert!(error.to_string().contains("truncated"));
    }

    #[test]
    fn rejects_noncontiguous_offsets() {
        let mut bytes = synthetic_archive(&[("A.MES", b"abcd"), ("B.GP4", b"payload")]);
        let key = u16::from_le_bytes([bytes[2], bytes[3]]);
        let end = data_offset(2).expect("offset");
        let mut table = decrypt_table(&bytes[HEADER_SIZE..end], key);
        table[ENTRY_SIZE + 14..ENTRY_SIZE + 18].copy_from_slice(&5u32.to_le_bytes());
        bytes[HEADER_SIZE..end].copy_from_slice(&encrypt_table(&table, key));
        let error = parse_archive_bytes(PathBuf::from("YUNO_A"), "YUNO_A".into(), bytes)
            .expect_err("gap must fail");
        assert!(error.to_string().contains("expected contiguous offset"));
    }

    #[test]
    fn rejects_unsafe_name() {
        let bytes = synthetic_archive(&[("../A.MES", b"payload")]);
        let error = parse_archive_bytes(PathBuf::from("YUNO_A"), "YUNO_A".into(), bytes)
            .expect_err("path traversal must fail");
        assert!(error.to_string().contains("unsafe archive entry name"));
    }
}
