use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const HEADER_SIZE: usize = 4;
const ENTRY_SIZE: usize = 0x28;
const NAME_SIZE: usize = 0x20;
const NAME_XOR: u8 = 0x5f;
const SIZE_XOR: u32 = 0x4683_1582;
const OFFSET_XOR: u32 = 0x1752_8913;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    pub index: usize,
    pub name: String,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archive {
    pub archive_size: usize,
    pub table_end: usize,
    pub entries: Vec<ArchiveEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnpackReport {
    pub extracted_files: usize,
    pub payload_bytes: u64,
    pub archive_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackReport {
    pub packed_files: usize,
    pub payload_bytes: u64,
    pub output_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveError(String);

impl ArchiveError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ArchiveError {}

impl Archive {
    pub fn parse(bytes: &[u8]) -> Result<Self, ArchiveError> {
        if bytes.len() < HEADER_SIZE {
            return Err(ArchiveError::new(format!(
                "archive is too small: expected at least {HEADER_SIZE} bytes, got {}",
                bytes.len()
            )));
        }

        let count = read_u32(bytes, 0)? as usize;
        let table_size = count
            .checked_mul(ENTRY_SIZE)
            .and_then(|size| size.checked_add(HEADER_SIZE))
            .ok_or_else(|| ArchiveError::new(format!("entry count is too large: {count}")))?;
        if table_size > bytes.len() {
            return Err(ArchiveError::new(format!(
                "archive table is truncated: count={count}, table_end=0x{table_size:X}, archive_size=0x{:X}",
                bytes.len()
            )));
        }

        let mut entries = Vec::with_capacity(count);
        let mut names = HashSet::with_capacity(count);
        for index in 0..count {
            let record_offset = HEADER_SIZE + index * ENTRY_SIZE;
            let name = decode_name(&bytes[record_offset..record_offset + NAME_SIZE], index)?;
            let folded_name = name.to_ascii_uppercase();
            if !names.insert(folded_name) {
                return Err(ArchiveError::new(format!(
                    "entry[{index}] has a duplicate Windows filename: {name:?}"
                )));
            }

            let size = read_u32(bytes, record_offset + NAME_SIZE)? ^ SIZE_XOR;
            let offset = read_u32(bytes, record_offset + NAME_SIZE + 4)? ^ OFFSET_XOR;
            let end = (offset as usize)
                .checked_add(size as usize)
                .ok_or_else(|| {
                    ArchiveError::new(format!(
                    "entry[{index}] {name:?} range overflows: offset=0x{offset:X}, size=0x{size:X}"
                ))
                })?;
            if (offset as usize) < table_size {
                return Err(ArchiveError::new(format!(
                    "entry[{index}] {name:?} starts inside the archive table: offset=0x{offset:X}, table_end=0x{table_size:X}"
                )));
            }
            if end > bytes.len() {
                return Err(ArchiveError::new(format!(
                    "entry[{index}] {name:?} is out of bounds: offset=0x{offset:X}, size=0x{size:X}, archive_size=0x{:X}",
                    bytes.len()
                )));
            }

            entries.push(ArchiveEntry {
                index,
                name,
                offset,
                size,
            });
        }

        let mut ranges: Vec<_> = entries
            .iter()
            .map(|entry| {
                (
                    entry.offset as usize,
                    entry.offset as usize + entry.size as usize,
                    entry,
                )
            })
            .collect();
        ranges.sort_unstable_by_key(|(start, _, _)| *start);
        for pair in ranges.windows(2) {
            let (_, previous_end, previous) = pair[0];
            let (next_start, _, next) = pair[1];
            if next_start < previous_end {
                return Err(ArchiveError::new(format!(
                    "entry[{}] {:?} overlaps entry[{}] {:?}: 0x{next_start:X} < 0x{previous_end:X}",
                    previous.index, previous.name, next.index, next.name
                )));
            }
        }

        Ok(Self {
            archive_size: bytes.len(),
            table_end: table_size,
            entries,
        })
    }

    pub fn data<'a>(&self, bytes: &'a [u8], entry: &ArchiveEntry) -> &'a [u8] {
        let start = entry.offset as usize;
        let end = start + entry.size as usize;
        &bytes[start..end]
    }

    pub fn rebuild(&self, payloads: &[Vec<u8>]) -> Result<Vec<u8>, ArchiveError> {
        if payloads.len() != self.entries.len() {
            return Err(ArchiveError::new(format!(
                "payload count mismatch: expected {}, got {}",
                self.entries.len(),
                payloads.len()
            )));
        }
        let payload_size = payloads
            .iter()
            .try_fold(0usize, |total, payload| total.checked_add(payload.len()));
        let output_size = payload_size
            .and_then(|size| size.checked_add(self.table_end))
            .ok_or_else(|| ArchiveError::new("rebuilt archive size overflows usize"))?;
        if output_size > u32::MAX as usize {
            return Err(ArchiveError::new(format!(
                "rebuilt archive is too large: {output_size} bytes"
            )));
        }

        let mut output = vec![0u8; self.table_end];
        output[..4].copy_from_slice(&(self.entries.len() as u32).to_le_bytes());
        let mut data_offset = self.table_end;
        for (entry, payload) in self.entries.iter().zip(payloads) {
            let size = u32::try_from(payload.len()).map_err(|_| {
                ArchiveError::new(format!("payload is too large for {:?}", entry.name))
            })?;
            let offset = u32::try_from(data_offset)
                .map_err(|_| ArchiveError::new("archive data offset exceeds u32"))?;
            let record_offset = HEADER_SIZE + entry.index * ENTRY_SIZE;
            encode_name(
                &entry.name,
                &mut output[record_offset..record_offset + NAME_SIZE],
            )?;
            output[record_offset + NAME_SIZE..record_offset + NAME_SIZE + 4]
                .copy_from_slice(&(size ^ SIZE_XOR).to_le_bytes());
            output[record_offset + NAME_SIZE + 4..record_offset + ENTRY_SIZE]
                .copy_from_slice(&(offset ^ OFFSET_XOR).to_le_bytes());
            output.extend_from_slice(payload);
            data_offset += payload.len();
        }
        Ok(output)
    }
}

pub fn unpack_file(input: &Path, output: &Path) -> Result<UnpackReport, ArchiveError> {
    let bytes = fs::read(input).map_err(|error| io_error("read archive", input, error))?;
    let archive = Archive::parse(&bytes)?;
    if output.exists() {
        return Err(ArchiveError::new(format!(
            "output already exists: {}",
            output.display()
        )));
    }

    let staging = staging_path(output)?;
    if staging.exists() {
        return Err(ArchiveError::new(format!(
            "staging output already exists: {}",
            staging.display()
        )));
    }
    fs::create_dir(&staging)
        .map_err(|error| io_error("create staging directory", &staging, error))?;

    let write_result = (|| {
        for entry in &archive.entries {
            let path = staging.join(&entry.name);
            fs::write(&path, archive.data(&bytes, entry))
                .map_err(|error| io_error("write extracted file", &path, error))?;
        }
        fs::rename(&staging, output)
            .map_err(|error| io_error("commit extracted directory", output, error))?;
        Ok(())
    })();

    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    Ok(UnpackReport {
        extracted_files: archive.entries.len(),
        payload_bytes: archive
            .entries
            .iter()
            .map(|entry| u64::from(entry.size))
            .sum(),
        archive_bytes: archive.archive_size,
    })
}

pub fn pack_directory(
    template: &Path,
    input: &Path,
    output: &Path,
) -> Result<PackReport, ArchiveError> {
    let template_bytes =
        fs::read(template).map_err(|error| io_error("read template archive", template, error))?;
    let archive = Archive::parse(&template_bytes)?;
    if output.exists() {
        return Err(ArchiveError::new(format!(
            "output already exists: {}",
            output.display()
        )));
    }
    let metadata =
        fs::metadata(input).map_err(|error| io_error("inspect input directory", input, error))?;
    if !metadata.is_dir() {
        return Err(ArchiveError::new(format!(
            "pack input is not a directory: {}",
            input.display()
        )));
    }

    let expected: HashSet<_> = archive
        .entries
        .iter()
        .map(|entry| entry.name.to_ascii_uppercase())
        .collect();
    let mut supplied = HashSet::new();
    for item in
        fs::read_dir(input).map_err(|error| io_error("read input directory", input, error))?
    {
        let item = item.map_err(|error| io_error("enumerate input directory", input, error))?;
        let file_type = item
            .file_type()
            .map_err(|error| io_error("inspect input item", &item.path(), error))?;
        if !file_type.is_file() {
            return Err(ArchiveError::new(format!(
                "input directory must be flat and contain only files: {}",
                item.path().display()
            )));
        }
        let name = item.file_name().to_string_lossy().into_owned();
        let folded = name.to_ascii_uppercase();
        if !supplied.insert(folded.clone()) {
            return Err(ArchiveError::new(format!(
                "input has duplicate Windows filename: {name:?}"
            )));
        }
        if !expected.contains(&folded) {
            return Err(ArchiveError::new(format!(
                "input contains a file not present in the template: {name:?}"
            )));
        }
    }
    if supplied != expected {
        let mut missing: Vec<_> = archive
            .entries
            .iter()
            .filter(|entry| !supplied.contains(&entry.name.to_ascii_uppercase()))
            .map(|entry| entry.name.as_str())
            .collect();
        missing.sort_unstable();
        return Err(ArchiveError::new(format!(
            "input is missing {} template file(s): {}",
            missing.len(),
            missing.join(", ")
        )));
    }

    let mut payloads = Vec::with_capacity(archive.entries.len());
    for entry in &archive.entries {
        let path = input.join(&entry.name);
        payloads.push(fs::read(&path).map_err(|error| io_error("read payload", &path, error))?);
    }
    let payload_bytes = payloads.iter().map(|payload| payload.len() as u64).sum();
    let rebuilt = archive.rebuild(&payloads)?;
    let staging = staging_path(output)?;
    if staging.exists() {
        return Err(ArchiveError::new(format!(
            "staging output already exists: {}",
            staging.display()
        )));
    }
    fs::write(&staging, &rebuilt)
        .map_err(|error| io_error("write staging archive", &staging, error))?;
    if let Err(error) = fs::rename(&staging, output) {
        let _ = fs::remove_file(&staging);
        return Err(io_error("commit packed archive", output, error));
    }
    Ok(PackReport {
        packed_files: archive.entries.len(),
        payload_bytes,
        output_bytes: rebuilt.len(),
    })
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, ArchiveError> {
    let raw = bytes.get(offset..offset + 4).ok_or_else(|| {
        ArchiveError::new(format!("cannot read u32 at archive offset 0x{offset:X}"))
    })?;
    Ok(u32::from_le_bytes(raw.try_into().expect("four-byte slice")))
}

fn decode_name(raw: &[u8], index: usize) -> Result<String, ArchiveError> {
    let decoded: Vec<u8> = raw.iter().map(|byte| byte ^ NAME_XOR).collect();
    let nul = decoded.iter().position(|byte| *byte == 0).ok_or_else(|| {
        ArchiveError::new(format!(
            "entry[{index}] filename is not NUL-terminated within {NAME_SIZE} bytes"
        ))
    })?;
    if nul == 0 {
        return Err(ArchiveError::new(format!(
            "entry[{index}] filename is empty"
        )));
    }
    if decoded[nul + 1..].iter().any(|byte| *byte != 0) {
        return Err(ArchiveError::new(format!(
            "entry[{index}] filename has nonzero bytes after its terminator"
        )));
    }
    let name_bytes = &decoded[..nul];
    if name_bytes
        .iter()
        .any(|byte| !byte.is_ascii() || byte.is_ascii_control())
    {
        return Err(ArchiveError::new(format!(
            "entry[{index}] filename is not printable ASCII"
        )));
    }
    let name =
        String::from_utf8(name_bytes.to_vec()).expect("an ASCII filename must be valid UTF-8");
    validate_name(&name, index)?;
    Ok(name)
}

fn encode_name(name: &str, output: &mut [u8]) -> Result<(), ArchiveError> {
    if output.len() != NAME_SIZE {
        return Err(ArchiveError::new("internal filename field size mismatch"));
    }
    if !name.is_ascii() || name.len() >= NAME_SIZE {
        return Err(ArchiveError::new(format!(
            "filename cannot be encoded in the archive table: {name:?}"
        )));
    }
    output.fill(0);
    output[..name.len()].copy_from_slice(name.as_bytes());
    for byte in output {
        *byte ^= NAME_XOR;
    }
    Ok(())
}

fn validate_name(name: &str, index: usize) -> Result<(), ArchiveError> {
    let path = Path::new(name);
    let mut components = path.components();
    let is_one_normal_component =
        matches!(components.next(), Some(std::path::Component::Normal(_)))
            && components.next().is_none();
    if !is_one_normal_component || name == "." || name == ".." || name.contains(['/', '\\', ':']) {
        return Err(ArchiveError::new(format!(
            "entry[{index}] has an unsafe filename: {name:?}"
        )));
    }
    Ok(())
}

fn staging_path(output: &Path) -> Result<PathBuf, ArchiveError> {
    let file_name = output.file_name().ok_or_else(|| {
        ArchiveError::new(format!(
            "output must name a directory below a parent: {}",
            output.display()
        ))
    })?;
    let mut staging_name = file_name.to_os_string();
    staging_name.push(".partial");
    Ok(output.with_file_name(staging_name))
}

fn io_error(action: &str, path: &Path, error: io::Error) -> ArchiveError {
    ArchiveError::new(format!("failed to {action} {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn archive_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let table_end = HEADER_SIZE + entries.len() * ENTRY_SIZE;
        let mut output = vec![0; table_end];
        output[..4].copy_from_slice(&(entries.len() as u32).to_le_bytes());
        let mut data_offset = table_end;
        for (index, (name, data)) in entries.iter().enumerate() {
            let record_offset = HEADER_SIZE + index * ENTRY_SIZE;
            let mut raw_name = [0u8; NAME_SIZE];
            raw_name[..name.len()].copy_from_slice(name.as_bytes());
            for byte in &mut raw_name {
                *byte ^= NAME_XOR;
            }
            output[record_offset..record_offset + NAME_SIZE].copy_from_slice(&raw_name);
            output[record_offset + NAME_SIZE..record_offset + NAME_SIZE + 4]
                .copy_from_slice(&((data.len() as u32) ^ SIZE_XOR).to_le_bytes());
            output[record_offset + NAME_SIZE + 4..record_offset + ENTRY_SIZE]
                .copy_from_slice(&((data_offset as u32) ^ OFFSET_XOR).to_le_bytes());
            output.extend_from_slice(data);
            data_offset += data.len();
        }
        output
    }

    #[test]
    fn parses_entries_and_payloads() {
        let bytes = archive_bytes(&[("A.MES", b"abc"), ("MAIN0.MES", b"xyz123")]);
        let archive = Archive::parse(&bytes).unwrap();
        assert_eq!(archive.table_end, 84);
        assert_eq!(archive.entries.len(), 2);
        assert_eq!(archive.entries[0].name, "A.MES");
        assert_eq!(archive.entries[0].offset, 84);
        assert_eq!(archive.data(&bytes, &archive.entries[0]), b"abc");
        assert_eq!(archive.data(&bytes, &archive.entries[1]), b"xyz123");
    }

    #[test]
    fn rejects_truncated_header() {
        let error = Archive::parse(&[0, 0, 0]).unwrap_err();
        assert!(error.to_string().contains("too small"));
    }

    #[test]
    fn rejects_truncated_table() {
        let error = Archive::parse(&1u32.to_le_bytes()).unwrap_err();
        assert!(error.to_string().contains("table is truncated"));
    }

    #[test]
    fn rejects_path_traversal() {
        let bytes = archive_bytes(&[("..\\A.MES", b"abc")]);
        let error = Archive::parse(&bytes).unwrap_err();
        assert!(error.to_string().contains("unsafe filename"));
    }

    #[test]
    fn rejects_case_insensitive_duplicates() {
        let bytes = archive_bytes(&[("A.MES", b"a"), ("a.mes", b"b")]);
        let error = Archive::parse(&bytes).unwrap_err();
        assert!(error.to_string().contains("duplicate Windows filename"));
    }

    #[test]
    fn rejects_entry_inside_table() {
        let mut bytes = archive_bytes(&[("A.MES", b"abc")]);
        bytes[HEADER_SIZE + NAME_SIZE + 4..HEADER_SIZE + ENTRY_SIZE]
            .copy_from_slice(&OFFSET_XOR.to_le_bytes());
        let error = Archive::parse(&bytes).unwrap_err();
        assert!(error.to_string().contains("inside the archive table"));
    }

    #[test]
    fn rejects_out_of_bounds_entry() {
        let mut bytes = archive_bytes(&[("A.MES", b"abc")]);
        bytes[HEADER_SIZE + NAME_SIZE..HEADER_SIZE + NAME_SIZE + 4]
            .copy_from_slice(&(100u32 ^ SIZE_XOR).to_le_bytes());
        let error = Archive::parse(&bytes).unwrap_err();
        assert!(error.to_string().contains("out of bounds"));
    }

    #[test]
    fn rejects_overlapping_entries() {
        let mut bytes = archive_bytes(&[("A.MES", b"abc"), ("B.MES", b"def")]);
        let first_offset = (HEADER_SIZE + 2 * ENTRY_SIZE) as u32;
        let second_record = HEADER_SIZE + ENTRY_SIZE;
        bytes[second_record + NAME_SIZE + 4..second_record + ENTRY_SIZE]
            .copy_from_slice(&(first_offset ^ OFFSET_XOR).to_le_bytes());
        let error = Archive::parse(&bytes).unwrap_err();
        assert!(error.to_string().contains("overlaps"));
    }

    #[test]
    fn rebuilds_archive_with_new_payload_sizes() {
        let original = archive_bytes(&[("A.MES", b"abc"), ("B.MES", b"def")]);
        let archive = Archive::parse(&original).unwrap();
        let rebuilt = archive
            .rebuild(&[b"longer".to_vec(), b"x".to_vec()])
            .unwrap();
        let reparsed = Archive::parse(&rebuilt).unwrap();
        assert_eq!(reparsed.data(&rebuilt, &reparsed.entries[0]), b"longer");
        assert_eq!(reparsed.data(&rebuilt, &reparsed.entries[1]), b"x");
        assert_eq!(reparsed.entries[1].offset, reparsed.table_end as u32 + 6);
    }

    #[test]
    fn unchanged_rebuild_is_byte_exact_for_contiguous_archive() {
        let original = archive_bytes(&[("A.MES", b"abc"), ("B.MES", b"def")]);
        let archive = Archive::parse(&original).unwrap();
        let payloads: Vec<_> = archive
            .entries
            .iter()
            .map(|entry| archive.data(&original, entry).to_vec())
            .collect();
        assert_eq!(archive.rebuild(&payloads).unwrap(), original);
    }
}
