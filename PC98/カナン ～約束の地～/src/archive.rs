use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

const CAT_HEADER_SIZE: usize = 6;
const LIB_HEADER_SIZE: usize = 6;
const RECORD_SIZE: usize = 0x16;
const NAME_SIZE: usize = 12;
const WINDOW_SIZE: usize = 0x1000;
const WINDOW_MASK: usize = WINDOW_SIZE - 1;
const MAX_MATCH: usize = 18;
const MIN_MATCH: usize = 3;
const MAX_DECODED_SIZE: usize = 64 * 1024 * 1024;
const MANIFEST_FORMAT: &str = "canaan-system98-catlib-v1";
const MANIFEST_NAME: &str = "_manifest.json";
const ORIGINAL_CAT_NAME: &str = "_original.cat";
const ORIGINAL_LIB_NAME: &str = "_original.lib";

#[derive(Debug, Error)]
pub enum ArchiveError {
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
    #[error("invalid archive: {0}")]
    Invalid(String),
    #[error("output already exists: {0}")]
    OutputExists(PathBuf),
}

pub type Result<T> = std::result::Result<T, ArchiveError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub format: String,
    pub catalog: String,
    pub library: String,
    pub catalog_marker: String,
    pub library_marker: String,
    pub entry_count: usize,
    pub catalog_sha256: String,
    pub library_sha256: String,
    pub catalog_trailing_bytes: usize,
    pub library_trailing_bytes: usize,
    pub entries: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_name_raw_hex")]
    pub name_raw_hex: String,
    #[serde(rename = "_offset")]
    pub offset: u32,
    #[serde(rename = "_size")]
    pub stored_size: u32,
    #[serde(rename = "_type")]
    pub storage_type: u16,
    #[serde(rename = "_decoded_size")]
    pub decoded_size: usize,
    #[serde(rename = "_decoded_size_marker")]
    pub decoded_size_marker: Option<u32>,
    #[serde(rename = "_stored_sha256")]
    pub stored_sha256: String,
    #[serde(rename = "_decoded_sha256")]
    pub decoded_sha256: String,
}

#[derive(Debug, Clone, Default)]
pub struct UnpackStats {
    pub entries: usize,
    pub raw: usize,
    pub compressed: usize,
    pub stored_bytes: usize,
    pub decoded_bytes: usize,
}

#[derive(Debug, Clone, Default)]
pub struct PackStats {
    pub entries: usize,
    pub unchanged_entries: usize,
    pub changed_entries: usize,
    pub reused_stored_entries: usize,
    pub output_cat_bytes: usize,
    pub output_lib_bytes: usize,
    pub byte_exact_pair: bool,
}

#[derive(Debug)]
pub struct PreparedUnpack {
    pub cat_path: PathBuf,
    pub lib_path: PathBuf,
    pub manifest: Manifest,
    pub entries: Vec<(String, Vec<u8>)>,
    pub original_cat: Vec<u8>,
    pub original_lib: Vec<u8>,
    pub stats: UnpackStats,
}

#[derive(Debug)]
pub struct PreparedPack {
    pub input_dir: PathBuf,
    pub catalog_name: String,
    pub library_name: String,
    pub cat_bytes: Vec<u8>,
    pub lib_bytes: Vec<u8>,
    pub stats: PackStats,
}

#[derive(Debug, Clone)]
struct Record {
    name_raw: [u8; NAME_SIZE],
    name: String,
    storage_type: u16,
    stored_size: u32,
    offset: u32,
}

#[derive(Debug)]
struct ParsedPair {
    catalog_marker: [u8; 4],
    library_marker: [u8; 4],
    records: Vec<Record>,
    catalog_trailing: Vec<u8>,
    library_trailing: Vec<u8>,
    stored_entries: Vec<Vec<u8>>,
    decoded_entries: Vec<Vec<u8>>,
    decoded_markers: Vec<Option<u32>>,
}

fn io_error(path: &Path, source: io::Error) -> ArchiveError {
    ArchiveError::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn read_file(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|source| io_error(path, source))
}

fn u16_at(data: &[u8], offset: usize, context: &str) -> Result<u16> {
    let bytes = data.get(offset..offset + 2).ok_or_else(|| {
        ArchiveError::Invalid(format!("{context}: truncated u16 at 0x{offset:X}"))
    })?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn u32_at(data: &[u8], offset: usize, context: &str) -> Result<u32> {
    let bytes = data.get(offset..offset + 4).ok_or_else(|| {
        ArchiveError::Invalid(format!("{context}: truncated u32 at 0x{offset:X}"))
    })?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn marker_string(marker: [u8; 4]) -> String {
    String::from_utf8_lossy(&marker).into_owned()
}

fn decode_name(raw: [u8; NAME_SIZE], index: usize) -> Result<String> {
    let end = raw
        .iter()
        .rposition(|byte| *byte != b' ')
        .map_or(0, |position| position + 1);
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(&raw[..end]);
    if had_errors {
        return Err(ArchiveError::Invalid(format!(
            "catalog entry {index}: filename is not valid CP932"
        )));
    }
    let name = decoded.into_owned();
    validate_flat_name(&name, index)?;
    Ok(name)
}

fn validate_flat_name(name: &str, index: usize) -> Result<()> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.chars().any(|ch| matches!(ch, '/' | '\\' | ':' | '\0'))
        || Path::new(name).file_name().and_then(|part| part.to_str()) != Some(name)
    {
        return Err(ArchiveError::Invalid(format!(
            "catalog entry {index}: unsafe filename {name:?}"
        )));
    }
    if matches!(
        name.to_ascii_lowercase().as_str(),
        MANIFEST_NAME | ORIGINAL_CAT_NAME | ORIGINAL_LIB_NAME
    ) {
        return Err(ArchiveError::Invalid(format!(
            "catalog entry {index}: filename conflicts with tool metadata: {name:?}"
        )));
    }
    Ok(())
}

fn decode_lzss(data: &[u8], start: usize, context: &str) -> Result<(Vec<u8>, usize)> {
    if start + 4 > data.len() {
        return Err(ArchiveError::Invalid(format!(
            "{context}: missing four-byte stream marker"
        )));
    }
    let mut source = start + 4;
    let mut window = [0u8; WINDOW_SIZE];
    let mut window_pos = 1usize;
    let mut output = Vec::new();
    let mut flags = 0u8;
    let mut mask = 0u8;

    loop {
        if mask == 0 {
            flags = *data
                .get(source)
                .ok_or_else(|| ArchiveError::Invalid(format!("{context}: truncated flag byte")))?;
            source += 1;
            mask = 1;
        }

        if flags & mask != 0 {
            let value = *data
                .get(source)
                .ok_or_else(|| ArchiveError::Invalid(format!("{context}: truncated literal")))?;
            source += 1;
            output.push(value);
            window[window_pos] = value;
            window_pos = (window_pos + 1) & WINDOW_MASK;
        } else {
            let token = u16_at(data, source, context)?;
            source += 2;
            if token == 0 {
                break;
            }
            let mut match_pos = usize::from(token >> 4);
            let length = usize::from(token & 0x0F) + MIN_MATCH;
            for _ in 0..length {
                let value = window[match_pos];
                match_pos = (match_pos + 1) & WINDOW_MASK;
                output.push(value);
                window[window_pos] = value;
                window_pos = (window_pos + 1) & WINDOW_MASK;
            }
        }

        if output.len() > MAX_DECODED_SIZE {
            return Err(ArchiveError::Invalid(format!(
                "{context}: decoded output exceeds {MAX_DECODED_SIZE} bytes"
            )));
        }
        mask = mask.wrapping_shl(1);
    }

    Ok((output, source))
}

fn insert_match_position(
    chains: &mut HashMap<[u8; 3], VecDeque<usize>>,
    input: &[u8],
    position: usize,
) {
    if position + MIN_MATCH > input.len() {
        return;
    }
    let key = [input[position], input[position + 1], input[position + 2]];
    let chain = chains.entry(key).or_default();
    chain.push_back(position);
    while chain
        .front()
        .is_some_and(|oldest| position.saturating_sub(*oldest) > WINDOW_MASK)
    {
        chain.pop_front();
    }
}

fn best_match(
    chains: &HashMap<[u8; 3], VecDeque<usize>>,
    input: &[u8],
    position: usize,
) -> Option<(usize, usize)> {
    if position + MIN_MATCH > input.len() {
        return None;
    }
    let key = [input[position], input[position + 1], input[position + 2]];
    let candidates = chains.get(&key)?;
    let maximum = MAX_MATCH.min(input.len() - position);
    let mut best: Option<(usize, usize)> = None;

    for &candidate in candidates.iter().rev() {
        let distance = position - candidate;
        if distance == 0 || distance > WINDOW_MASK {
            continue;
        }
        let mut length = MIN_MATCH;
        while length < maximum && input[position + length] == input[position + length - distance] {
            length += 1;
        }
        // Token 0x0000 is the end-of-stream marker, so this otherwise valid
        // three-byte match cannot be emitted.
        if ((1 + candidate) & WINDOW_MASK) == 0 && length == MIN_MATCH {
            continue;
        }
        if best.is_none_or(|(_, best_length)| length > best_length) {
            best = Some((candidate, length));
            if length == maximum {
                break;
            }
        }
    }
    best
}

fn encode_lzss_stream(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut chains: HashMap<[u8; 3], VecDeque<usize>> = HashMap::new();
    let mut position = 0usize;
    let mut finished = false;

    while !finished {
        let flags_offset = output.len();
        output.push(0);
        let mut flags = 0u8;

        for bit in 0..8u8 {
            if position >= input.len() {
                output.extend_from_slice(&0u16.to_le_bytes());
                finished = true;
                break;
            }

            if let Some((candidate, length)) = best_match(&chains, input, position) {
                let match_pos = (1 + candidate) & WINDOW_MASK;
                let token = ((match_pos as u16) << 4) | ((length - MIN_MATCH) as u16);
                output.extend_from_slice(&token.to_le_bytes());
                for consumed in 0..length {
                    insert_match_position(&mut chains, input, position + consumed);
                }
                position += length;
            } else {
                flags |= 1 << bit;
                output.push(input[position]);
                insert_match_position(&mut chains, input, position);
                position += 1;
            }
        }

        output[flags_offset] = flags;
    }

    output
}

fn encode_lzss_with_marker(marker: [u8; 4], input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len() + 8);
    output.extend_from_slice(&marker);
    output.extend_from_slice(&encode_lzss_stream(input));
    output
}

fn parse_catalog(cat_data: &[u8], context: &str) -> Result<([u8; 4], Vec<Record>, Vec<u8>)> {
    let marker: [u8; 4] = cat_data
        .get(..4)
        .ok_or_else(|| ArchiveError::Invalid(format!("{context}: truncated CAT header")))?
        .try_into()
        .expect("slice length checked");
    if marker != *b"Cat0" && marker != *b"Cat1" {
        return Err(ArchiveError::Invalid(format!(
            "{context}: unsupported CAT marker {:?}",
            marker_string(marker)
        )));
    }
    let count = usize::from(u16_at(cat_data, 4, context)?);
    let expected = count.checked_mul(RECORD_SIZE).ok_or_else(|| {
        ArchiveError::Invalid(format!("{context}: catalog record length overflow"))
    })?;

    let (record_bytes, consumed) = if marker == *b"Cat0" {
        let end = CAT_HEADER_SIZE
            .checked_add(expected)
            .ok_or_else(|| ArchiveError::Invalid(format!("{context}: catalog size overflow")))?;
        let bytes = cat_data.get(CAT_HEADER_SIZE..end).ok_or_else(|| {
            ArchiveError::Invalid(format!(
                "{context}: raw catalog is shorter than {expected} record bytes"
            ))
        })?;
        (bytes.to_vec(), end)
    } else {
        let (bytes, end) = decode_lzss(cat_data, 2, context)?;
        if bytes.len() != expected {
            return Err(ArchiveError::Invalid(format!(
                "{context}: decoded catalog length {} != {expected}",
                bytes.len()
            )));
        }
        (bytes, end)
    };

    let mut records = Vec::with_capacity(count);
    let mut names = HashSet::new();
    for index in 0..count {
        let start = index * RECORD_SIZE;
        let raw = &record_bytes[start..start + RECORD_SIZE];
        let name_raw: [u8; NAME_SIZE] = raw[..NAME_SIZE].try_into().expect("fixed record");
        let name = decode_name(name_raw, index)?;
        if !names.insert(name.to_lowercase()) {
            return Err(ArchiveError::Invalid(format!(
                "{context}: duplicate filename at entry {index}: {name:?}"
            )));
        }
        let storage_type = u16::from_le_bytes([raw[12], raw[13]]);
        if storage_type > 1 {
            return Err(ArchiveError::Invalid(format!(
                "{context}: unsupported storage type {storage_type} at entry {index}"
            )));
        }
        records.push(Record {
            name_raw,
            name,
            storage_type,
            stored_size: u32::from_le_bytes([raw[14], raw[15], raw[16], raw[17]]),
            offset: u32::from_le_bytes([raw[18], raw[19], raw[20], raw[21]]),
        });
    }

    Ok((marker, records, cat_data[consumed..].to_vec()))
}

fn parse_pair_bytes(cat_data: &[u8], lib_data: &[u8], context: &str) -> Result<ParsedPair> {
    let (catalog_marker, records, catalog_trailing) = parse_catalog(cat_data, context)?;
    let library_marker: [u8; 4] = lib_data
        .get(..4)
        .ok_or_else(|| ArchiveError::Invalid(format!("{context}: truncated LIB header")))?
        .try_into()
        .expect("slice length checked");
    if library_marker != *b"Lib0" {
        return Err(ArchiveError::Invalid(format!(
            "{context}: unsupported LIB marker {:?}",
            marker_string(library_marker)
        )));
    }
    let lib_count = usize::from(u16_at(lib_data, 4, context)?);
    if lib_count != records.len() {
        return Err(ArchiveError::Invalid(format!(
            "{context}: CAT count {} != LIB count {lib_count}",
            records.len()
        )));
    }

    let mut expected_offset = 0u32;
    let mut stored_entries = Vec::with_capacity(records.len());
    let mut decoded_entries = Vec::with_capacity(records.len());
    let mut decoded_markers = Vec::with_capacity(records.len());

    for (index, record) in records.iter().enumerate() {
        if record.offset != expected_offset {
            return Err(ArchiveError::Invalid(format!(
                "{context}: entry {index} offset 0x{:X} != expected 0x{expected_offset:X}",
                record.offset
            )));
        }
        let start = LIB_HEADER_SIZE
            .checked_add(record.offset as usize)
            .ok_or_else(|| ArchiveError::Invalid(format!("{context}: LIB offset overflow")))?;
        let end = start
            .checked_add(record.stored_size as usize)
            .ok_or_else(|| ArchiveError::Invalid(format!("{context}: LIB size overflow")))?;
        let stored = lib_data.get(start..end).ok_or_else(|| {
            ArchiveError::Invalid(format!("{context}: entry {index} exceeds LIB bounds"))
        })?;

        let (decoded, marker) = if record.storage_type == 0 {
            (stored.to_vec(), None)
        } else {
            let decoded_size = u32_at(stored, 0, context)?;
            let (decoded, consumed) = decode_lzss(stored, 0, context)?;
            if consumed != stored.len() {
                return Err(ArchiveError::Invalid(format!(
                    "{context}: entry {index} compressed stream has {} trailing bytes",
                    stored.len() - consumed
                )));
            }
            if decoded.len() != decoded_size as usize {
                return Err(ArchiveError::Invalid(format!(
                    "{context}: entry {index} decoded length {} != marker {decoded_size}",
                    decoded.len()
                )));
            }
            (decoded, Some(decoded_size))
        };
        expected_offset = expected_offset
            .checked_add(record.stored_size)
            .ok_or_else(|| ArchiveError::Invalid(format!("{context}: LIB payload overflow")))?;
        stored_entries.push(stored.to_vec());
        decoded_entries.push(decoded);
        decoded_markers.push(marker);
    }

    let payload_end = LIB_HEADER_SIZE + expected_offset as usize;
    if payload_end > lib_data.len() {
        return Err(ArchiveError::Invalid(format!(
            "{context}: LIB payload exceeds file"
        )));
    }

    Ok(ParsedPair {
        catalog_marker,
        library_marker,
        records,
        catalog_trailing,
        library_trailing: lib_data[payload_end..].to_vec(),
        stored_entries,
        decoded_entries,
        decoded_markers,
    })
}

fn paired_lib_path(cat_path: &Path) -> Result<PathBuf> {
    let parent = cat_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = cat_path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ArchiveError::Invalid(format!("invalid CAT filename: {}", cat_path.display()))
        })?;
    for extension in ["LIB", "lib", "Lib"] {
        let candidate = parent.join(format!("{stem}.{extension}"));
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(ArchiveError::Invalid(format!(
        "missing paired LIB for {}",
        cat_path.display()
    )))
}

pub fn prepare_unpack(cat_path: &Path) -> Result<PreparedUnpack> {
    if !cat_path.is_file() {
        return Err(ArchiveError::Invalid(format!(
            "CAT input is not a file: {}",
            cat_path.display()
        )));
    }
    let lib_path = paired_lib_path(cat_path)?;
    let cat_data = read_file(cat_path)?;
    let lib_data = read_file(&lib_path)?;
    let parsed = parse_pair_bytes(&cat_data, &lib_data, &cat_path.display().to_string())?;

    let catalog = cat_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ArchiveError::Invalid("CAT filename is not Unicode".to_string()))?
        .to_string();
    let library = lib_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ArchiveError::Invalid("LIB filename is not Unicode".to_string()))?
        .to_string();

    let mut stats = UnpackStats::default();
    let mut manifest_entries = Vec::with_capacity(parsed.records.len());
    let mut entries = Vec::with_capacity(parsed.records.len());
    for (index, record) in parsed.records.iter().enumerate() {
        let stored = &parsed.stored_entries[index];
        let decoded = &parsed.decoded_entries[index];
        stats.entries += 1;
        stats.stored_bytes += stored.len();
        stats.decoded_bytes += decoded.len();
        if record.storage_type == 0 {
            stats.raw += 1;
        } else {
            stats.compressed += 1;
        }
        manifest_entries.push(ManifestEntry {
            file: record.name.clone(),
            index,
            name_raw_hex: hex::encode(record.name_raw),
            offset: record.offset,
            stored_size: record.stored_size,
            storage_type: record.storage_type,
            decoded_size: decoded.len(),
            decoded_size_marker: parsed.decoded_markers[index],
            stored_sha256: sha256(stored),
            decoded_sha256: sha256(decoded),
        });
        entries.push((record.name.clone(), decoded.clone()));
    }

    let manifest = Manifest {
        format: MANIFEST_FORMAT.to_string(),
        catalog,
        library,
        catalog_marker: marker_string(parsed.catalog_marker),
        library_marker: marker_string(parsed.library_marker),
        entry_count: parsed.records.len(),
        catalog_sha256: sha256(&cat_data),
        library_sha256: sha256(&lib_data),
        catalog_trailing_bytes: parsed.catalog_trailing.len(),
        library_trailing_bytes: parsed.library_trailing.len(),
        entries: manifest_entries,
    };

    Ok(PreparedUnpack {
        cat_path: cat_path.to_path_buf(),
        lib_path,
        manifest,
        entries,
        original_cat: cat_data,
        original_lib: lib_data,
        stats,
    })
}

fn write_new(path: &Path, data: &[u8]) -> Result<()> {
    if path.exists() {
        return Err(ArchiveError::OutputExists(path.to_path_buf()));
    }
    fs::write(path, data).map_err(|source| io_error(path, source))
}

impl PreparedUnpack {
    pub fn write_to(&self, output_dir: &Path) -> Result<()> {
        if output_dir.exists() {
            return Err(ArchiveError::OutputExists(output_dir.to_path_buf()));
        }
        fs::create_dir(output_dir).map_err(|source| io_error(output_dir, source))?;
        let result = (|| {
            for (name, bytes) in &self.entries {
                write_new(&output_dir.join(name), bytes)?;
            }
            write_new(&output_dir.join(ORIGINAL_CAT_NAME), &self.original_cat)?;
            write_new(&output_dir.join(ORIGINAL_LIB_NAME), &self.original_lib)?;
            let mut manifest_json =
                serde_json::to_string_pretty(&self.manifest).map_err(|source| {
                    ArchiveError::Json {
                        path: output_dir.join(MANIFEST_NAME),
                        source,
                    }
                })?;
            manifest_json.push('\n');
            write_new(&output_dir.join(MANIFEST_NAME), manifest_json.as_bytes())?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(output_dir);
        }
        result
    }
}

fn read_manifest(input_dir: &Path) -> Result<Manifest> {
    let path = input_dir.join(MANIFEST_NAME);
    let data = read_file(&path)?;
    let manifest: Manifest =
        serde_json::from_slice(&data).map_err(|source| ArchiveError::Json {
            path: path.clone(),
            source,
        })?;
    if manifest.format != MANIFEST_FORMAT {
        return Err(ArchiveError::Invalid(format!(
            "{}: unsupported manifest format {:?}",
            path.display(),
            manifest.format
        )));
    }
    Ok(manifest)
}

fn validate_manifest(manifest: &Manifest, parsed: &ParsedPair) -> Result<()> {
    if manifest.entry_count != parsed.records.len()
        || manifest.entries.len() != parsed.records.len()
    {
        return Err(ArchiveError::Invalid(
            "manifest entry count does not match original archive".to_string(),
        ));
    }
    if manifest.catalog_marker != marker_string(parsed.catalog_marker)
        || manifest.library_marker != marker_string(parsed.library_marker)
        || manifest.catalog_trailing_bytes != parsed.catalog_trailing.len()
        || manifest.library_trailing_bytes != parsed.library_trailing.len()
    {
        return Err(ArchiveError::Invalid(
            "manifest archive metadata does not match original pair".to_string(),
        ));
    }
    let mut names = HashSet::new();
    for (index, entry) in manifest.entries.iter().enumerate() {
        let record = &parsed.records[index];
        let decoded = &parsed.decoded_entries[index];
        let stored = &parsed.stored_entries[index];
        validate_flat_name(&entry.file, index)?;
        if !names.insert(entry.file.to_lowercase()) {
            return Err(ArchiveError::Invalid(format!(
                "manifest has duplicate filename at entry {index}: {:?}",
                entry.file
            )));
        }
        let expected_raw = hex::decode(&entry.name_raw_hex).map_err(|error| {
            ArchiveError::Invalid(format!(
                "manifest entry {index}: invalid _name_raw_hex: {error}"
            ))
        })?;
        if entry.index != index
            || entry.file != record.name
            || expected_raw != record.name_raw
            || entry.offset != record.offset
            || entry.stored_size != record.stored_size
            || entry.storage_type != record.storage_type
            || entry.decoded_size != decoded.len()
            || entry.decoded_size_marker != parsed.decoded_markers[index]
            || entry.stored_sha256 != sha256(stored)
            || entry.decoded_sha256 != sha256(decoded)
        {
            return Err(ArchiveError::Invalid(format!(
                "manifest entry {index} does not match original archive"
            )));
        }
    }
    Ok(())
}

fn validate_unpack_directory(input_dir: &Path, manifest: &Manifest) -> Result<()> {
    let expected: HashSet<String> = manifest
        .entries
        .iter()
        .map(|entry| entry.file.to_lowercase())
        .chain(
            [MANIFEST_NAME, ORIGINAL_CAT_NAME, ORIGINAL_LIB_NAME]
                .into_iter()
                .map(str::to_string),
        )
        .collect();
    let mut actual = HashSet::new();
    for item in fs::read_dir(input_dir).map_err(|source| io_error(input_dir, source))? {
        let item = item.map_err(|source| io_error(input_dir, source))?;
        let file_type = item
            .file_type()
            .map_err(|source| io_error(&item.path(), source))?;
        if !file_type.is_file() {
            return Err(ArchiveError::Invalid(format!(
                "unexpected non-file in unpack directory: {}",
                item.path().display()
            )));
        }
        let name = item.file_name().to_string_lossy().to_lowercase();
        if !actual.insert(name.clone()) {
            return Err(ArchiveError::Invalid(format!(
                "duplicate path in unpack directory: {name}"
            )));
        }
        if !expected.contains(&name) {
            return Err(ArchiveError::Invalid(format!(
                "unexpected file in unpack directory: {}",
                item.path().display()
            )));
        }
    }
    if actual != expected {
        let missing: Vec<_> = expected.difference(&actual).cloned().collect();
        return Err(ArchiveError::Invalid(format!(
            "unpack directory is missing files: {}",
            missing.join(", ")
        )));
    }
    Ok(())
}

fn build_catalog(
    marker: [u8; 4],
    count: usize,
    record_bytes: &[u8],
    trailing: &[u8],
) -> Result<Vec<u8>> {
    let count = u16::try_from(count)
        .map_err(|_| ArchiveError::Invalid("too many catalog entries".to_string()))?;
    let mut output = Vec::new();
    output.extend_from_slice(&marker);
    output.extend_from_slice(&count.to_le_bytes());
    if marker == *b"Cat0" {
        output.extend_from_slice(record_bytes);
    } else if marker == *b"Cat1" {
        output.extend_from_slice(&encode_lzss_stream(record_bytes));
    } else {
        return Err(ArchiveError::Invalid(format!(
            "unsupported CAT marker {:?}",
            marker_string(marker)
        )));
    }
    output.extend_from_slice(trailing);
    Ok(output)
}

pub fn prepare_pack(input_dir: &Path) -> Result<PreparedPack> {
    if !input_dir.is_dir() {
        return Err(ArchiveError::Invalid(format!(
            "pack input is not a directory: {}",
            input_dir.display()
        )));
    }
    let manifest = read_manifest(input_dir)?;
    validate_unpack_directory(input_dir, &manifest)?;
    let original_cat = read_file(&input_dir.join(ORIGINAL_CAT_NAME))?;
    let original_lib = read_file(&input_dir.join(ORIGINAL_LIB_NAME))?;
    if sha256(&original_cat) != manifest.catalog_sha256
        || sha256(&original_lib) != manifest.library_sha256
    {
        return Err(ArchiveError::Invalid(
            "saved original CAT/LIB hash does not match manifest".to_string(),
        ));
    }
    let parsed = parse_pair_bytes(
        &original_cat,
        &original_lib,
        &input_dir.display().to_string(),
    )?;
    validate_manifest(&manifest, &parsed)?;

    let mut decoded_files = Vec::with_capacity(manifest.entries.len());
    let mut all_unchanged = true;
    for entry in &manifest.entries {
        let bytes = read_file(&input_dir.join(&entry.file))?;
        if sha256(&bytes) != entry.decoded_sha256 {
            all_unchanged = false;
        }
        decoded_files.push(bytes);
    }

    let mut stats = PackStats {
        entries: manifest.entries.len(),
        byte_exact_pair: all_unchanged,
        ..PackStats::default()
    };

    let (cat_bytes, lib_bytes) = if all_unchanged {
        stats.unchanged_entries = manifest.entries.len();
        stats.reused_stored_entries = manifest.entries.len();
        (original_cat, original_lib)
    } else {
        let mut records = Vec::with_capacity(manifest.entries.len() * RECORD_SIZE);
        let mut lib = Vec::new();
        lib.extend_from_slice(&parsed.library_marker);
        let count = u16::try_from(manifest.entries.len())
            .map_err(|_| ArchiveError::Invalid("too many library entries".to_string()))?;
        lib.extend_from_slice(&count.to_le_bytes());
        let mut offset = 0u32;

        for (index, entry) in manifest.entries.iter().enumerate() {
            let decoded = &decoded_files[index];
            let unchanged = sha256(decoded) == entry.decoded_sha256;
            let stored = if unchanged {
                stats.unchanged_entries += 1;
                stats.reused_stored_entries += 1;
                parsed.stored_entries[index].clone()
            } else if entry.storage_type == 0 {
                stats.changed_entries += 1;
                decoded.clone()
            } else {
                stats.changed_entries += 1;
                let decoded_size = u32::try_from(decoded.len()).map_err(|_| {
                    ArchiveError::Invalid(format!("entry {index} is too large to encode"))
                })?;
                encode_lzss_with_marker(decoded_size.to_le_bytes(), decoded)
            };
            let stored_size = u32::try_from(stored.len()).map_err(|_| {
                ArchiveError::Invalid(format!("entry {index} stored data is too large"))
            })?;
            let raw_name = hex::decode(&entry.name_raw_hex).map_err(|error| {
                ArchiveError::Invalid(format!(
                    "manifest entry {index}: invalid _name_raw_hex: {error}"
                ))
            })?;
            records.extend_from_slice(&raw_name);
            records.extend_from_slice(&entry.storage_type.to_le_bytes());
            records.extend_from_slice(&stored_size.to_le_bytes());
            records.extend_from_slice(&offset.to_le_bytes());
            lib.extend_from_slice(&stored);
            offset = offset.checked_add(stored_size).ok_or_else(|| {
                ArchiveError::Invalid("rebuilt LIB payload exceeds u32".to_string())
            })?;
        }
        lib.extend_from_slice(&parsed.library_trailing);
        let cat = build_catalog(
            parsed.catalog_marker,
            manifest.entries.len(),
            &records,
            &parsed.catalog_trailing,
        )?;
        // Validate the rebuilt pair before exposing it to the caller.
        let rebuilt = parse_pair_bytes(&cat, &lib, "rebuilt archive")?;
        if rebuilt.decoded_entries != decoded_files {
            return Err(ArchiveError::Invalid(
                "rebuilt archive failed decoded-content verification".to_string(),
            ));
        }
        (cat, lib)
    };

    stats.output_cat_bytes = cat_bytes.len();
    stats.output_lib_bytes = lib_bytes.len();
    Ok(PreparedPack {
        input_dir: input_dir.to_path_buf(),
        catalog_name: manifest.catalog,
        library_name: manifest.library,
        cat_bytes,
        lib_bytes,
        stats,
    })
}

impl PreparedPack {
    pub fn write_to(&self, output_cat: &Path, output_lib: &Path) -> Result<()> {
        if output_cat.exists() {
            return Err(ArchiveError::OutputExists(output_cat.to_path_buf()));
        }
        if output_lib.exists() {
            return Err(ArchiveError::OutputExists(output_lib.to_path_buf()));
        }
        if let Some(parent) = output_cat.parent() {
            fs::create_dir_all(parent).map_err(|source| io_error(parent, source))?;
        }
        if let Some(parent) = output_lib.parent() {
            fs::create_dir_all(parent).map_err(|source| io_error(parent, source))?;
        }
        write_new(output_cat, &self.cat_bytes)?;
        if let Err(error) = write_new(output_lib, &self.lib_bytes) {
            let _ = fs::remove_file(output_cat);
            return Err(error);
        }
        Ok(())
    }
}

pub fn list_catalogs(directory: &Path) -> Result<Vec<PathBuf>> {
    if !directory.is_dir() {
        return Err(ArchiveError::Invalid(format!(
            "input is not a directory: {}",
            directory.display()
        )));
    }
    let mut catalogs = Vec::new();
    for item in fs::read_dir(directory).map_err(|source| io_error(directory, source))? {
        let item = item.map_err(|source| io_error(directory, source))?;
        if item
            .file_type()
            .map_err(|source| io_error(&item.path(), source))?
            .is_file()
            && item
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("cat"))
        {
            catalogs.push(item.path());
        }
    }
    catalogs.sort_by_key(|path| path.file_name().map(|name| name.to_ascii_lowercase()));
    if catalogs.is_empty() {
        return Err(ArchiveError::Invalid(format!(
            "no CAT files found in {}",
            directory.display()
        )));
    }
    Ok(catalogs)
}

pub fn list_pack_directories(directory: &Path) -> Result<Vec<PathBuf>> {
    if directory.join(MANIFEST_NAME).is_file() {
        return Ok(vec![directory.to_path_buf()]);
    }
    let mut inputs = Vec::new();
    for item in fs::read_dir(directory).map_err(|source| io_error(directory, source))? {
        let item = item.map_err(|source| io_error(directory, source))?;
        if item
            .file_type()
            .map_err(|source| io_error(&item.path(), source))?
            .is_dir()
            && item.path().join(MANIFEST_NAME).is_file()
        {
            inputs.push(item.path());
        }
    }
    inputs.sort_by_key(|path| path.file_name().map(|name| name.to_ascii_lowercase()));
    if inputs.is_empty() {
        return Err(ArchiveError::Invalid(format!(
            "no unpack manifests found in {}",
            directory.display()
        )));
    }
    Ok(inputs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip_lzss(data: &[u8]) {
        let marker = u32::try_from(data.len()).unwrap().to_le_bytes();
        let encoded = encode_lzss_with_marker(marker, data);
        let (decoded, consumed) = decode_lzss(&encoded, 0, "test").unwrap();
        assert_eq!(decoded, data);
        assert_eq!(consumed, encoded.len());
    }

    #[test]
    fn lzss_round_trips_empty_and_literals() {
        round_trip_lzss(b"");
        round_trip_lzss(b"short literal payload");
    }

    #[test]
    fn lzss_round_trips_repetition_and_window_wrap() {
        round_trip_lzss(&vec![0; 20_000]);
        let data: Vec<u8> = (0..30_000).map(|index| (index % 251) as u8).collect();
        round_trip_lzss(&data);
    }

    #[test]
    fn lzss_never_emits_reserved_zero_match_token() {
        let mut data: Vec<u8> = (0..6_000)
            .map(|index| ((index * 73 + index / 17) % 251) as u8)
            .collect();
        let copied = [data[4_095], data[4_096], data[4_097]];
        data[5_000..5_003].copy_from_slice(&copied);
        data[5_003] = data[4_098].wrapping_add(1);
        round_trip_lzss(&data);
    }

    #[test]
    fn lzss_rejects_truncated_stream() {
        let error = decode_lzss(&[0, 0, 0, 0, 1], 0, "truncated").unwrap_err();
        assert!(error.to_string().contains("truncated literal"));
    }

    #[test]
    fn cat0_round_trip_preserves_record() {
        let mut record = [b' '; RECORD_SIZE];
        record[..6].copy_from_slice(b"a.s   ");
        record[12..14].copy_from_slice(&0u16.to_le_bytes());
        record[14..18].copy_from_slice(&3u32.to_le_bytes());
        record[18..22].copy_from_slice(&0u32.to_le_bytes());
        let cat = build_catalog(*b"Cat0", 1, &record, b"").unwrap();
        let mut lib = b"Lib0\x01\x00abc".to_vec();
        let parsed = parse_pair_bytes(&cat, &lib, "synthetic").unwrap();
        assert_eq!(parsed.records[0].name, "a.s");
        assert_eq!(parsed.decoded_entries[0], b"abc");
        lib.pop();
        assert!(parse_pair_bytes(&cat, &lib, "truncated").is_err());
    }

    #[test]
    fn cat1_and_compressed_entry_round_trip() {
        let decoded = b"compressed compressed compressed payload";
        let stored = encode_lzss_with_marker((decoded.len() as u32).to_le_bytes(), decoded);
        let mut record = [b' '; RECORD_SIZE];
        record[..6].copy_from_slice(b"a.s   ");
        record[12..14].copy_from_slice(&1u16.to_le_bytes());
        record[14..18].copy_from_slice(&(stored.len() as u32).to_le_bytes());
        record[18..22].copy_from_slice(&0u32.to_le_bytes());
        let cat = build_catalog(*b"Cat1", 1, &record, b"").unwrap();
        let mut lib = b"Lib0\x01\x00".to_vec();
        lib.extend_from_slice(&stored);
        let parsed = parse_pair_bytes(&cat, &lib, "synthetic compressed").unwrap();
        assert_eq!(parsed.catalog_marker, *b"Cat1");
        assert_eq!(parsed.records[0].storage_type, 1);
        assert_eq!(parsed.decoded_entries[0], decoded);
    }

    #[test]
    fn unsafe_names_are_rejected() {
        assert!(validate_flat_name("../bad", 0).is_err());
        assert!(validate_flat_name("a/b", 0).is_err());
        assert!(validate_flat_name("ok.s", 0).is_ok());
    }
}
