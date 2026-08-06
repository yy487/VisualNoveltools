use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub mod extract;
pub mod font;
pub mod glyph;
pub mod inject;
pub mod script;
pub mod text_json;
pub mod workflow;
pub mod workspace;

pub const HEADER_SIZE: u64 = 0x40;
pub const RECORD_SIZE: u64 = 0x100;
pub const PAYLOAD_ALIGNMENT: u64 = 0x800;
pub const MANIFEST_FILE: &str = ".mpk-manifest.json";
const COPY_BUFFER_SIZE: usize = 1024 * 1024;
const ZERO_BUFFER_SIZE: usize = 64 * 1024;

#[derive(Debug, Clone)]
pub struct ToolError(pub String);

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for ToolError {}

impl From<io::Error> for ToolError {
    fn from(value: io::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<serde_json::Error> for ToolError {
    fn from(value: serde_json::Error) -> Self {
        Self(value.to_string())
    }
}

pub type ToolResult<T> = Result<T, ToolError>;

#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub index: usize,
    pub status: u8,
    pub ordinal: u32,
    pub offset: u64,
    pub stored_size: u64,
    pub original_size: u64,
    pub name: String,
    pub name_bytes: Vec<u8>,
    pub raw_record: [u8; RECORD_SIZE as usize],
}

#[derive(Debug)]
pub struct Archive {
    pub header: [u8; HEADER_SIZE as usize],
    pub version: u32,
    pub data_start: u64,
    pub file_len: u64,
    pub entries: Vec<ArchiveEntry>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    schema: String,
    source_name: String,
    alignment: u64,
    header_hex: String,
    warnings: Vec<String>,
    entries: Vec<ManifestEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestEntry {
    index: u64,
    status: u8,
    ordinal: u32,
    offset: u64,
    stored_size: u64,
    original_size: u64,
    name: String,
    name_bytes_hex: String,
    record_hex: String,
}

#[derive(Debug, Clone)]
pub struct UnpackReport {
    pub input: PathBuf,
    pub output: PathBuf,
    pub extracted_files: usize,
    pub warnings: usize,
    pub warning_messages: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PackReport {
    pub input: PathBuf,
    pub output: PathBuf,
    pub packed_files: usize,
    pub output_bytes: u64,
    pub warnings: usize,
}

#[derive(Debug)]
struct PackEntry {
    name: String,
    path: PathBuf,
    record: [u8; RECORD_SIZE as usize],
    data_len: u64,
    offset: u64,
}

pub fn default_unpack_output(input: &Path) -> ToolResult<PathBuf> {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ToolError(format!(
                "cannot derive output name from '{}'",
                input.display()
            ))
        })?;
    Ok(parent.join(format!("{stem}_unpacked")))
}

pub fn default_pack_output(input_dir: &Path) -> ToolResult<PathBuf> {
    let parent = input_dir.parent().unwrap_or_else(|| Path::new("."));
    let stem = input_dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ToolError(format!(
                "cannot derive output name from '{}'",
                input_dir.display()
            ))
        })?;
    let base = stem.strip_suffix("_unpacked").unwrap_or(stem);
    Ok(parent.join(format!("{base}_packed.mpk")))
}

pub fn parse_archive(path: &Path) -> ToolResult<Archive> {
    let mut file = File::open(path)
        .map_err(|error| ToolError(format!("cannot open '{}': {error}", path.display())))?;
    let file_len = file
        .metadata()
        .map_err(|error| ToolError(format!("cannot stat '{}': {error}", path.display())))?
        .len();
    if file_len < HEADER_SIZE {
        return Err(ToolError(format!(
            "'{}' is truncated before the 0x40-byte MPK header",
            path.display()
        )));
    }

    let mut header = [0u8; HEADER_SIZE as usize];
    file.read_exact(&mut header).map_err(|error| {
        ToolError(format!(
            "cannot read header from '{}': {error}",
            path.display()
        ))
    })?;
    if &header[..4] != b"MPK\0" {
        return Err(ToolError(format!(
            "'{}' has unsupported magic {}; expected MPK\\0",
            path.display(),
            hex_encode(&header[..4])
        )));
    }

    let version = read_u32(&header, 4);
    let count = read_u64(&header, 8);
    let table_bytes = count.checked_mul(RECORD_SIZE).ok_or_else(|| {
        ToolError(format!(
            "'{}' has an overflowing entry table",
            path.display()
        ))
    })?;
    let table_end = HEADER_SIZE.checked_add(table_bytes).ok_or_else(|| {
        ToolError(format!(
            "'{}' has an overflowing entry table",
            path.display()
        ))
    })?;
    if table_end > file_len {
        return Err(ToolError(format!(
            "'{}' entry table ends at 0x{table_end:X}, beyond file size 0x{file_len:X}",
            path.display()
        )));
    }
    if count > usize::MAX as u64 {
        return Err(ToolError(format!(
            "'{}' entry count {count} does not fit this process",
            path.display()
        )));
    }
    let data_start = round_up(table_end, PAYLOAD_ALIGNMENT)?;
    if data_start > file_len {
        return Err(ToolError(format!(
            "'{}' data area starts at 0x{data_start:X}, beyond file size 0x{file_len:X}",
            path.display()
        )));
    }

    let table_len: usize = table_bytes
        .try_into()
        .map_err(|_| ToolError(format!("'{}' entry table is too large", path.display())))?;
    let mut table = vec![0u8; table_len];
    file.seek(SeekFrom::Start(HEADER_SIZE)).map_err(|error| {
        ToolError(format!(
            "cannot seek table in '{}': {error}",
            path.display()
        ))
    })?;
    file.read_exact(&mut table).map_err(|error| {
        ToolError(format!(
            "cannot read table from '{}': {error}",
            path.display()
        ))
    })?;
    if table_end < data_start {
        ensure_zero_range(&mut file, table_end, data_start - table_end, path, 0)?;
    }

    let mut entries = Vec::with_capacity(count as usize);
    let mut warnings = Vec::new();
    let mut previous_end = data_start;
    for index in 0..count as usize {
        let base = index * RECORD_SIZE as usize;
        let mut raw_record = [0u8; RECORD_SIZE as usize];
        raw_record.copy_from_slice(&table[base..base + RECORD_SIZE as usize]);
        let status = raw_record[0];
        let ordinal = read_u32(&raw_record, 4);
        let offset = read_u64(&raw_record, 8);
        let stored_size = read_u64(&raw_record, 16);
        let original_size = read_u64(&raw_record, 24);
        let name_end = raw_record[32..]
            .iter()
            .position(|byte| *byte == 0)
            .ok_or_else(|| {
                ToolError(format!(
                    "'{}' entry {index} has no NUL terminator in its 224-byte name field",
                    path.display()
                ))
            })?;
        let name_bytes = raw_record[32..32 + name_end].to_vec();
        let (decoded_name, _, had_errors) = SHIFT_JIS.decode(&name_bytes);
        if had_errors {
            return Err(ToolError(format!(
                "'{}' entry {index} name is not valid CP932 bytes",
                path.display()
            )));
        }
        let name = decoded_name.into_owned();
        validate_member_name(&name).map_err(|error| {
            ToolError(format!(
                "'{}' entry {index} name '{}': {error}",
                path.display(),
                name
            ))
        })?;
        if name == MANIFEST_FILE {
            return Err(ToolError(format!(
                "'{}' entry {index} reserves the manifest filename '{MANIFEST_FILE}'",
                path.display()
            )));
        }
        if ordinal != index as u32 {
            return Err(ToolError(format!(
                "'{}' entry {index} has ordinal {ordinal}, expected {index}",
                path.display()
            )));
        }
        if !offset.is_multiple_of(PAYLOAD_ALIGNMENT) {
            return Err(ToolError(format!(
                "'{}' entry {index} offset 0x{offset:X} is not 0x{PAYLOAD_ALIGNMENT:X}-aligned",
                path.display()
            )));
        }
        if offset < data_start {
            return Err(ToolError(format!(
                "'{}' entry {index} offset 0x{offset:X} lies inside the header/table",
                path.display()
            )));
        }
        if offset < previous_end {
            return Err(ToolError(format!(
                "'{}' entry {index} overlaps the previous payload",
                path.display()
            )));
        }
        let end = offset.checked_add(stored_size).ok_or_else(|| {
            ToolError(format!(
                "'{}' entry {index} payload end overflows",
                path.display()
            ))
        })?;
        if end > file_len {
            return Err(ToolError(format!(
                "'{}' entry {index} payload ends at 0x{end:X}, beyond file size 0x{file_len:X}",
                path.display()
            )));
        }
        if offset > previous_end {
            ensure_zero_range(&mut file, previous_end, offset - previous_end, path, index)?;
        }
        if status != 0 {
            warnings.push(format!(
                "entry {index} ({name}) has opaque status byte 0x{status:02X}; payload copied as stored bytes"
            ));
        }
        if stored_size != original_size {
            warnings.push(format!(
                "entry {index} ({name}) has stored_size={stored_size} and original_size={original_size}; no decompression is applied"
            ));
        }
        previous_end = end;
        entries.push(ArchiveEntry {
            index,
            status,
            ordinal,
            offset,
            stored_size,
            original_size,
            name,
            name_bytes,
            raw_record,
        });
    }
    if previous_end != file_len {
        if previous_end < file_len {
            ensure_zero_range(
                &mut file,
                previous_end,
                file_len - previous_end,
                path,
                entries.len(),
            )?;
        }
        return Err(ToolError(format!(
            "'{}' has unsupported trailing padding after the last payload (end 0x{previous_end:X}, file size 0x{file_len:X})",
            path.display()
        )));
    }

    Ok(Archive {
        header,
        version,
        data_start,
        file_len,
        entries,
        warnings,
    })
}

pub fn unpack_archive(input: &Path, output: Option<&Path>) -> ToolResult<UnpackReport> {
    let archive = parse_archive(input)?;
    let output_path = match output {
        Some(path) => path.to_path_buf(),
        None => default_unpack_output(input)?,
    };
    refuse_existing(&output_path)?;
    let parent = output_path.parent().unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(ToolError(format!(
            "output parent '{}' does not exist",
            parent.display()
        )));
    }
    let temp_path = temp_sibling(&output_path)?;
    refuse_existing(&temp_path)?;
    fs::create_dir(&temp_path).map_err(|error| {
        ToolError(format!(
            "cannot create temporary output '{}': {error}",
            temp_path.display()
        ))
    })?;

    let operation = (|| -> ToolResult<()> {
        let mut source = File::open(input)
            .map_err(|error| ToolError(format!("cannot reopen '{}': {error}", input.display())))?;
        let mut buffer = vec![0u8; COPY_BUFFER_SIZE];
        for entry in &archive.entries {
            let destination = temp_path.join(&entry.name);
            let mut target = File::create(&destination).map_err(|error| {
                ToolError(format!(
                    "cannot create extracted file '{}' (entry {}): {error}",
                    destination.display(),
                    entry.index
                ))
            })?;
            source
                .seek(SeekFrom::Start(entry.offset))
                .map_err(|error| {
                    ToolError(format!(
                        "cannot seek '{}' to entry {} offset 0x{:X}: {error}",
                        input.display(),
                        entry.index,
                        entry.offset
                    ))
                })?;
            copy_exact(&mut source, &mut target, entry.stored_size, &mut buffer).map_err(
                |error| {
                    ToolError(format!(
                        "cannot extract '{}' entry {}: {error}",
                        input.display(),
                        entry.index
                    ))
                },
            )?;
            target.flush().map_err(|error| {
                ToolError(format!("cannot flush '{}': {error}", destination.display()))
            })?;
        }

        let manifest = Manifest {
            schema: "merry-mpk-manifest-v1".to_string(),
            source_name: input
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string(),
            alignment: PAYLOAD_ALIGNMENT,
            header_hex: hex_encode(&archive.header),
            warnings: archive.warnings.clone(),
            entries: archive
                .entries
                .iter()
                .map(|entry| ManifestEntry {
                    index: entry.index as u64,
                    status: entry.status,
                    ordinal: entry.ordinal,
                    offset: entry.offset,
                    stored_size: entry.stored_size,
                    original_size: entry.original_size,
                    name: entry.name.clone(),
                    name_bytes_hex: hex_encode(&entry.name_bytes),
                    record_hex: hex_encode(&entry.raw_record),
                })
                .collect(),
        };
        let mut manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
        manifest_bytes.push(b'\n');
        fs::write(temp_path.join(MANIFEST_FILE), manifest_bytes).map_err(|error| {
            ToolError(format!(
                "cannot write manifest '{}': {error}",
                temp_path.join(MANIFEST_FILE).display()
            ))
        })?;
        Ok(())
    })();
    if let Err(error) = operation {
        let _ = fs::remove_dir_all(&temp_path);
        return Err(error);
    }
    fs::rename(&temp_path, &output_path).map_err(|error| {
        let _ = fs::remove_dir_all(&temp_path);
        ToolError(format!(
            "cannot finalize output '{}' from '{}': {error}",
            output_path.display(),
            temp_path.display()
        ))
    })?;

    Ok(UnpackReport {
        input: input.to_path_buf(),
        output: output_path,
        extracted_files: archive.entries.len(),
        warnings: archive.warnings.len(),
        warning_messages: archive.warnings,
    })
}

pub fn pack_archive(input_dir: &Path, output: Option<&Path>) -> ToolResult<PackReport> {
    if !input_dir.is_dir() {
        return Err(ToolError(format!(
            "pack input '{}' is not a directory",
            input_dir.display()
        )));
    }
    let manifest_path = input_dir.join(MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path).map_err(|error| {
        ToolError(format!(
            "cannot read manifest '{}': {error}",
            manifest_path.display()
        ))
    })?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes).map_err(|error| {
        ToolError(format!(
            "cannot parse manifest '{}': {error}",
            manifest_path.display()
        ))
    })?;
    if manifest.schema != "merry-mpk-manifest-v1" {
        return Err(ToolError(format!(
            "manifest '{}' has unsupported schema '{}', expected merry-mpk-manifest-v1",
            manifest_path.display(),
            manifest.schema
        )));
    }
    if manifest.alignment != PAYLOAD_ALIGNMENT {
        return Err(ToolError(format!(
            "manifest '{}' uses unsupported alignment 0x{:X}",
            manifest_path.display(),
            manifest.alignment
        )));
    }
    let header_vec = hex_decode(&manifest.header_hex).map_err(|error| {
        ToolError(format!(
            "manifest '{}' header_hex: {error}",
            manifest_path.display()
        ))
    })?;
    if header_vec.len() != HEADER_SIZE as usize || &header_vec[..4] != b"MPK\0" {
        return Err(ToolError(format!(
            "manifest '{}' does not contain a valid 0x40-byte MPK header",
            manifest_path.display()
        )));
    }
    let mut header = [0u8; HEADER_SIZE as usize];
    header.copy_from_slice(&header_vec);
    let count = read_u64(&header, 8);
    if count != manifest.entries.len() as u64 {
        return Err(ToolError(format!(
            "manifest '{}' header count {count} does not match {} entries",
            manifest_path.display(),
            manifest.entries.len()
        )));
    }
    if count > u32::MAX as u64 {
        return Err(ToolError(format!(
            "entry count {count} exceeds MPK engine limit"
        )));
    }

    let table_end = HEADER_SIZE
        .checked_add(count.checked_mul(RECORD_SIZE).ok_or_else(|| {
            ToolError(format!(
                "manifest '{}' table size overflows",
                manifest_path.display()
            ))
        })?)
        .ok_or_else(|| {
            ToolError(format!(
                "manifest '{}' table size overflows",
                manifest_path.display()
            ))
        })?;
    let data_start = round_up(table_end, PAYLOAD_ALIGNMENT)?;
    let mut entries: Vec<PackEntry> = Vec::with_capacity(manifest.entries.len());
    let mut seen_names = Vec::<String>::new();
    let mut previous_offset = data_start;
    for (position, manifest_entry) in manifest.entries.iter().enumerate() {
        if manifest_entry.index != position as u64 || manifest_entry.ordinal != position as u32 {
            return Err(ToolError(format!(
                "manifest entry {position} has unstable index/ordinal ({}/{})",
                manifest_entry.index, manifest_entry.ordinal
            )));
        }
        validate_member_name(&manifest_entry.name).map_err(|error| {
            ToolError(format!(
                "manifest entry {position} name '{}': {error}",
                manifest_entry.name
            ))
        })?;
        if manifest_entry.name == MANIFEST_FILE {
            return Err(ToolError(format!(
                "manifest entry {position} collides with {MANIFEST_FILE}"
            )));
        }
        if seen_names
            .iter()
            .any(|name| name.to_lowercase() == manifest_entry.name.to_lowercase())
        {
            return Err(ToolError(format!(
                "manifest contains duplicate member name '{}'",
                manifest_entry.name
            )));
        }
        seen_names.push(manifest_entry.name.clone());

        let raw_vec = hex_decode(&manifest_entry.record_hex)
            .map_err(|error| ToolError(format!("manifest entry {position} record_hex: {error}")))?;
        if raw_vec.len() != RECORD_SIZE as usize {
            return Err(ToolError(format!(
                "manifest entry {position} record_hex has {} bytes, expected 256",
                raw_vec.len()
            )));
        }
        let mut record = [0u8; RECORD_SIZE as usize];
        record.copy_from_slice(&raw_vec);
        let status = record[0];
        let ordinal = read_u32(&record, 4);
        let old_offset = read_u64(&record, 8);
        let old_stored_size = read_u64(&record, 16);
        let old_original_size = read_u64(&record, 24);
        if status != manifest_entry.status
            || ordinal != manifest_entry.ordinal
            || old_offset != manifest_entry.offset
            || old_stored_size != manifest_entry.stored_size
            || old_original_size != manifest_entry.original_size
        {
            return Err(ToolError(format!(
                "manifest entry {position} metadata disagrees with its raw record"
            )));
        }
        if status != 0 || old_stored_size != old_original_size {
            return Err(ToolError(format!(
                "manifest entry {position} ({}) uses an opaque/compressed payload (status=0x{status:02X}, stored_size={old_stored_size}, original_size={old_original_size}); pack is currently limited to uncompressed MPK",
                manifest_entry.name
            )));
        }
        let raw_name_end = record[32..]
            .iter()
            .position(|byte| *byte == 0)
            .ok_or_else(|| {
                ToolError(format!(
                    "manifest entry {position} has no NUL name terminator"
                ))
            })?;
        let raw_name = &record[32..32 + raw_name_end];
        let (decoded_name, _, had_errors) = SHIFT_JIS.decode(raw_name);
        if had_errors || decoded_name != manifest_entry.name {
            return Err(ToolError(format!(
                "manifest entry {position} name does not match its raw CP932 name field"
            )));
        }
        let expected_name_hex = hex_encode(raw_name);
        if expected_name_hex != manifest_entry.name_bytes_hex {
            return Err(ToolError(format!(
                "manifest entry {position} name_bytes_hex does not match its raw name field"
            )));
        }

        let member_path = input_dir.join(&manifest_entry.name);
        let metadata = fs::metadata(&member_path).map_err(|error| {
            ToolError(format!(
                "cannot stat member '{}' (entry {}): {error}",
                member_path.display(),
                position
            ))
        })?;
        if !metadata.is_file() {
            return Err(ToolError(format!(
                "member '{}' (entry {}) is not a regular file",
                member_path.display(),
                position
            )));
        }
        let data_len = metadata.len();
        let offset = if position == 0 {
            data_start
        } else {
            round_up(
                previous_offset
                    .checked_add(entries[position - 1].data_len)
                    .ok_or_else(|| {
                        ToolError(format!("entry {position} payload layout overflows"))
                    })?,
                PAYLOAD_ALIGNMENT,
            )?
        };
        if position == 0 && old_offset != data_start {
            return Err(ToolError(format!(
                "manifest entry 0 starts at 0x{old_offset:X}, expected 0x{data_start:X}"
            )));
        }
        let end = offset
            .checked_add(data_len)
            .ok_or_else(|| ToolError(format!("entry {position} payload layout overflows")))?;
        let mut patched_record = record;
        write_u64(&mut patched_record, 8, offset);
        write_u64(&mut patched_record, 16, data_len);
        write_u64(&mut patched_record, 24, data_len);
        entries.push(PackEntry {
            name: manifest_entry.name.clone(),
            path: member_path,
            record: patched_record,
            data_len,
            offset,
        });
        previous_offset = offset;
        if end < offset {
            return Err(ToolError(format!(
                "entry {position} payload layout wrapped"
            )));
        }
    }

    let output_path = match output {
        Some(path) => path.to_path_buf(),
        None => default_pack_output(input_dir)?,
    };
    refuse_existing(&output_path)?;
    let parent = output_path.parent().unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(ToolError(format!(
            "output parent '{}' does not exist",
            parent.display()
        )));
    }
    let temp_path = temp_sibling(&output_path)?;
    refuse_existing(&temp_path)?;

    let operation = (|| -> ToolResult<u64> {
        let mut output_file = File::create(&temp_path).map_err(|error| {
            ToolError(format!(
                "cannot create temporary package '{}': {error}",
                temp_path.display()
            ))
        })?;
        output_file
            .write_all(&header)
            .map_err(|error| ToolError(format!("cannot write package header: {error}")))?;
        for entry in &entries {
            output_file.write_all(&entry.record).map_err(|error| {
                ToolError(format!(
                    "cannot write package record '{}': {error}",
                    entry.name
                ))
            })?;
        }
        let table_end = HEADER_SIZE + count * RECORD_SIZE;
        write_zeros(&mut output_file, data_start - table_end)?;
        let mut current = data_start;
        let mut source_buffer = vec![0u8; COPY_BUFFER_SIZE];
        for entry in &entries {
            if entry.offset < current {
                return Err(ToolError(format!(
                    "entry '{}' has overlapping rebuilt offset 0x{:X}",
                    entry.name, entry.offset
                )));
            }
            write_zeros(&mut output_file, entry.offset - current)?;
            let mut source = File::open(&entry.path).map_err(|error| {
                ToolError(format!(
                    "cannot open member '{}': {error}",
                    entry.path.display()
                ))
            })?;
            copy_exact(
                &mut source,
                &mut output_file,
                entry.data_len,
                &mut source_buffer,
            )
            .map_err(|error| {
                ToolError(format!(
                    "cannot copy member '{}': {error}",
                    entry.path.display()
                ))
            })?;
            current = entry.offset + entry.data_len;
        }
        output_file
            .flush()
            .map_err(|error| ToolError(format!("cannot flush package: {error}")))?;
        output_file
            .sync_all()
            .map_err(|error| ToolError(format!("cannot sync package: {error}")))?;
        Ok(current)
    })();
    let output_bytes = match operation {
        Ok(value) => value,
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            return Err(error);
        }
    };
    fs::rename(&temp_path, &output_path).map_err(|error| {
        let _ = fs::remove_file(&temp_path);
        ToolError(format!(
            "cannot finalize package '{}' from '{}': {error}",
            output_path.display(),
            temp_path.display()
        ))
    })?;

    Ok(PackReport {
        input: input_dir.to_path_buf(),
        output: output_path,
        packed_files: entries.len(),
        output_bytes,
        warnings: 0,
    })
}

pub fn validate_member_name(name: &str) -> ToolResult<()> {
    if name.is_empty() || name == "." || name == ".." {
        return Err(ToolError("empty or special path component".to_string()));
    }
    if name.chars().any(|character| {
        character.is_control()
            || matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            )
    }) {
        return Err(ToolError(
            "contains an unsafe Windows path character".to_string(),
        ));
    }
    if name.ends_with('.') || name.ends_with(' ') {
        return Err(ToolError(
            "ends with a Windows-trimmed dot or space".to_string(),
        ));
    }
    if Path::new(name).is_absolute() {
        return Err(ToolError("is an absolute path".to_string()));
    }
    Ok(())
}

pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 0x0F) as usize] as char);
    }
    result
}

pub fn hex_decode(value: &str) -> ToolResult<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return Err(ToolError("hex string has odd length".to_string()));
    }
    let bytes = value.as_bytes();
    let mut result = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0;
    while index < bytes.len() {
        let high = hex_nibble(bytes[index])
            .ok_or_else(|| ToolError(format!("invalid hex digit at position {index}")))?;
        let low = hex_nibble(bytes[index + 1])
            .ok_or_else(|| ToolError(format!("invalid hex digit at position {}", index + 1)))?;
        result.push((high << 4) | low);
        index += 2;
    }
    Ok(result)
}

fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("fixed-width u32"),
    )
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        bytes[offset..offset + 8]
            .try_into()
            .expect("fixed-width u64"),
    )
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn round_up(value: u64, alignment: u64) -> ToolResult<u64> {
    if alignment == 0 {
        return Err(ToolError("alignment cannot be zero".to_string()));
    }
    let remainder = value % alignment;
    if remainder == 0 {
        Ok(value)
    } else {
        value
            .checked_add(alignment - remainder)
            .ok_or_else(|| ToolError("alignment calculation overflows".to_string()))
    }
}

fn ensure_zero_range(
    file: &mut File,
    start: u64,
    length: u64,
    path: &Path,
    entry: usize,
) -> ToolResult<()> {
    file.seek(SeekFrom::Start(start)).map_err(|error| {
        ToolError(format!(
            "cannot seek '{}' to alignment padding before entry {entry}: {error}",
            path.display()
        ))
    })?;
    let mut remaining = length;
    let mut buffer = [0u8; ZERO_BUFFER_SIZE];
    while remaining > 0 {
        let requested = remaining.min(buffer.len() as u64) as usize;
        file.read_exact(&mut buffer[..requested]).map_err(|error| {
            ToolError(format!(
                "cannot read alignment padding in '{}' before entry {entry}: {error}",
                path.display()
            ))
        })?;
        if buffer[..requested].iter().any(|byte| *byte != 0) {
            return Err(ToolError(format!(
                "'{}' contains non-zero alignment padding before entry {entry}; refusing lossy unpack",
                path.display()
            )));
        }
        remaining -= requested as u64;
    }
    Ok(())
}

fn copy_exact<R: Read, W: Write>(
    source: &mut R,
    destination: &mut W,
    mut length: u64,
    buffer: &mut [u8],
) -> ToolResult<()> {
    while length > 0 {
        let requested = length.min(buffer.len() as u64) as usize;
        let read = source.read(&mut buffer[..requested])?;
        if read == 0 {
            return Err(ToolError(
                "unexpected end of file while copying payload".to_string(),
            ));
        }
        destination.write_all(&buffer[..read])?;
        length -= read as u64;
    }
    Ok(())
}

fn write_zeros<W: Write>(destination: &mut W, mut length: u64) -> ToolResult<()> {
    let buffer = [0u8; ZERO_BUFFER_SIZE];
    while length > 0 {
        let requested = length.min(buffer.len() as u64) as usize;
        destination.write_all(&buffer[..requested])?;
        length -= requested as u64;
    }
    Ok(())
}

fn refuse_existing(path: &Path) -> ToolResult<()> {
    if path.exists() {
        return Err(ToolError(format!(
            "output already exists: '{}'",
            path.display()
        )));
    }
    Ok(())
}

fn temp_sibling(path: &Path) -> ToolResult<PathBuf> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ToolError(format!(
                "cannot derive temporary name from '{}': invalid filename",
                path.display()
            ))
        })?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| ToolError(format!("system clock is before UNIX epoch: {error}")))?
        .as_nanos();
    Ok(parent.join(format!(".{name}.tmp-{}-{stamp}", std::process::id())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn round_up_is_checked() {
        assert_eq!(round_up(0x40, 0x800).unwrap(), 0x800);
        assert_eq!(round_up(0x800, 0x800).unwrap(), 0x800);
        assert_eq!(round_up(0x801, 0x800).unwrap(), 0x1000);
    }

    #[test]
    fn hex_round_trip() {
        let source = [0x00, 0x01, 0xAB, 0xFF];
        assert_eq!(hex_decode(&hex_encode(&source)).unwrap(), source);
    }

    #[test]
    fn member_names_are_path_safe() {
        assert!(validate_member_name("main00.msb").is_ok());
        assert!(validate_member_name("../main00.msb").is_err());
        assert!(validate_member_name("a\\b").is_err());
        assert!(validate_member_name("a?.msb").is_err());
    }

    #[test]
    fn parses_a_minimal_archive() {
        let path = std::env::temp_dir().join(format!("merry-mpk-test-{}.mpk", std::process::id()));
        let mut bytes = vec![0u8; 0x800 + 3];
        bytes[..4].copy_from_slice(b"MPK\0");
        bytes[4..8].copy_from_slice(&0x0002_0000u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&1u64.to_le_bytes());
        bytes[0x40 + 4..0x40 + 8].copy_from_slice(&0u32.to_le_bytes());
        bytes[0x40 + 8..0x40 + 16].copy_from_slice(&0x800u64.to_le_bytes());
        bytes[0x40 + 16..0x40 + 24].copy_from_slice(&3u64.to_le_bytes());
        bytes[0x40 + 24..0x40 + 32].copy_from_slice(&3u64.to_le_bytes());
        bytes[0x40 + 32..0x40 + 38].copy_from_slice(b"a.msb\0");
        bytes[0x800..].copy_from_slice(b"abc");
        fs::write(&path, bytes).unwrap();
        let archive = parse_archive(&path).unwrap();
        assert_eq!(archive.entries.len(), 1);
        assert_eq!(archive.entries[0].name, "a.msb");
        assert_eq!(archive.entries[0].stored_size, 3);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_bad_magic() {
        let path =
            std::env::temp_dir().join(format!("merry-mpk-bad-magic-{}.mpk", std::process::id()));
        fs::write(&path, vec![0u8; 0x800]).unwrap();
        assert!(parse_archive(&path).is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_nonzero_table_padding() {
        let path =
            std::env::temp_dir().join(format!("merry-mpk-bad-padding-{}.mpk", std::process::id()));
        let mut bytes = vec![0u8; 0x800 + 1];
        bytes[..4].copy_from_slice(b"MPK\0");
        bytes[8..16].copy_from_slice(&1u64.to_le_bytes());
        bytes[0x40 + 4..0x40 + 8].copy_from_slice(&0u32.to_le_bytes());
        bytes[0x40 + 8..0x40 + 16].copy_from_slice(&0x800u64.to_le_bytes());
        bytes[0x40 + 16..0x40 + 24].copy_from_slice(&1u64.to_le_bytes());
        bytes[0x40 + 24..0x40 + 32].copy_from_slice(&1u64.to_le_bytes());
        bytes[0x40 + 32..0x40 + 38].copy_from_slice(b"a.msb\0");
        bytes[0x400] = 1;
        fs::write(&path, bytes).unwrap();
        assert!(parse_archive(&path).is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_truncated_payload() {
        let path =
            std::env::temp_dir().join(format!("merry-mpk-truncated-{}.mpk", std::process::id()));
        let mut bytes = vec![0u8; 0x802];
        bytes[..4].copy_from_slice(b"MPK\0");
        bytes[8..16].copy_from_slice(&1u64.to_le_bytes());
        bytes[0x40 + 4..0x40 + 8].copy_from_slice(&0u32.to_le_bytes());
        bytes[0x40 + 8..0x40 + 16].copy_from_slice(&0x800u64.to_le_bytes());
        bytes[0x40 + 16..0x40 + 24].copy_from_slice(&3u64.to_le_bytes());
        bytes[0x40 + 24..0x40 + 32].copy_from_slice(&3u64.to_le_bytes());
        bytes[0x40 + 32..0x40 + 38].copy_from_slice(b"a.msb\0");
        fs::write(&path, bytes).unwrap();
        assert!(parse_archive(&path).is_err());
        fs::remove_file(path).unwrap();
    }
}
