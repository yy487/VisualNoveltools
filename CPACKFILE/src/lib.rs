use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub const HEADER_SIZE: u64 = 16;
pub const ENTRY_SIZE: u64 = 168;
pub const NAME_SIZE: usize = 128;
pub const KEY_SIZE: usize = 20;

#[derive(Debug, Clone)]
pub struct PakHeader {
    pub field_00: u32,
    pub file_count: u32,
    pub field_08: u32,
    pub flags: u32,
}

#[derive(Debug, Clone)]
pub struct PakEntry {
    pub index: u32,
    pub name: String,
    pub size: u32,
    pub offset: u64,
    key: [u8; KEY_SIZE],
}

#[derive(Debug)]
pub struct PakArchive {
    path: PathBuf,
    pub header: PakHeader,
    pub entries: Vec<PakEntry>,
    pub data_base: u64,
    pub file_size: u64,
}

impl PakArchive {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        let mut file = File::open(&path)
            .map_err(|e| format!("cannot open archive {}: {e}", path.display()))?;
        let file_size = file
            .metadata()
            .map_err(|e| format!("cannot read metadata for {}: {e}", path.display()))?
            .len();

        let mut raw_header = [0u8; HEADER_SIZE as usize];
        file.read_exact(&mut raw_header)
            .map_err(|e| format!("cannot read 16-byte header from {}: {e}", path.display()))?;
        let header = PakHeader {
            field_00: le_u32(&raw_header[0..4]),
            file_count: le_u32(&raw_header[4..8]),
            field_08: le_u32(&raw_header[8..12]),
            flags: le_u32(&raw_header[12..16]),
        };

        if header.field_00 != 0x0002_0000 || header.field_08 != 16 || header.flags != 13 {
            return Err(format!(
                "unsupported PAK header in {}: field_00=0x{:08X}, field_08={}, flags=0x{:08X} (expected 0x00020000, 16, 13)",
                path.display(),
                header.field_00,
                header.field_08,
                header.flags
            ));
        }

        let table_size = u64::from(header.file_count)
            .checked_mul(ENTRY_SIZE)
            .ok_or_else(|| "file table size overflow".to_owned())?;
        let data_base = HEADER_SIZE
            .checked_add(table_size)
            .ok_or_else(|| "archive data offset overflow".to_owned())?;
        if data_base > file_size {
            return Err(format!(
                "truncated file table in {}: data starts at 0x{data_base:X}, file is only {file_size} bytes",
                path.display()
            ));
        }

        let mut entries = Vec::with_capacity(header.file_count as usize);
        let mut expected_offset = 0u64;
        let mut casefolded_names = HashSet::with_capacity(header.file_count as usize);
        for index in 0..header.file_count {
            let mut raw = [0u8; ENTRY_SIZE as usize];
            file.read_exact(&mut raw).map_err(|e| {
                format!(
                    "cannot read entry {index} at archive offset 0x{:X}: {e}",
                    HEADER_SIZE + u64::from(index) * ENTRY_SIZE
                )
            })?;

            let mut key = [0u8; KEY_SIZE];
            key.copy_from_slice(&raw[132..152]);
            let name = decrypt_name(index, &raw[..NAME_SIZE], &key)?;
            let relative = safe_relative_path(index, &name)?;
            let collision_key = relative.to_string_lossy().replace('\\', "/").to_lowercase();
            if !casefolded_names.insert(collision_key) {
                return Err(format!(
                    "entry {index} collides with another output path on a case-insensitive filesystem: {name:?}"
                ));
            }

            let size = le_u32(&raw[128..132]);
            let offset = le_u64(&raw[152..160]);
            let state = le_u32(&raw[160..164]);
            let reserved = le_u32(&raw[164..168]);
            if state != 0 || reserved != 0 {
                return Err(format!(
                    "entry {index} has unsupported on-disk state/reserved values: state=0x{state:08X}, reserved=0x{reserved:08X}"
                ));
            }
            if offset != expected_offset {
                return Err(format!(
                    "entry {index} is not contiguous: offset=0x{offset:X}, expected=0x{expected_offset:X}"
                ));
            }
            expected_offset = offset
                .checked_add(u64::from(size))
                .ok_or_else(|| format!("entry {index} data range overflows"))?;

            entries.push(PakEntry {
                index,
                name,
                size,
                offset,
                key,
            });
        }

        let data_size = file_size - data_base;
        if expected_offset != data_size {
            return Err(format!(
                "archive payload size mismatch in {}: table describes {} bytes, file contains {} bytes",
                path.display(),
                expected_offset,
                data_size
            ));
        }

        Ok(Self {
            path,
            header,
            entries,
            data_base,
            file_size,
        })
    }

    pub fn extract_to(&self, output: impl AsRef<Path>, overwrite: bool) -> Result<(), String> {
        let output = output.as_ref();
        let output_name = output.file_name().ok_or_else(|| {
            format!(
                "refusing output path without a final directory name: {}",
                output.display()
            )
        })?;
        if output.exists() && !overwrite {
            return Err(format!(
                "output already exists: {} (pass --overwrite or approve it interactively)",
                output.display()
            ));
        }

        // Validate every archive path before the first output write.
        let mut relative_paths = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            relative_paths.push(safe_relative_path(entry.index, &entry.name)?);
        }

        let parent = output
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create output parent {}: {e}", parent.display()))?;
        let stage_name = format!(
            ".{}.nosurge_tmp_{}",
            output_name.to_string_lossy(),
            std::process::id()
        );
        let stage = parent.join(stage_name);
        if stage.exists() {
            return Err(format!(
                "temporary output already exists; remove it before retrying: {}",
                stage.display()
            ));
        }
        fs::create_dir(&stage)
            .map_err(|e| format!("cannot create temporary output {}: {e}", stage.display()))?;

        let result = self.extract_into_stage(&stage, &relative_paths);
        if let Err(error) = result {
            let _ = fs::remove_dir_all(&stage);
            return Err(error);
        }

        if output.exists() {
            let removal = if output.is_dir() {
                fs::remove_dir_all(output)
            } else {
                fs::remove_file(output)
            };
            if let Err(e) = removal {
                let _ = fs::remove_dir_all(&stage);
                return Err(format!(
                    "cannot replace existing output {}: {e}",
                    output.display()
                ));
            }
        }
        fs::rename(&stage, output).map_err(|e| {
            let _ = fs::remove_dir_all(&stage);
            format!(
                "cannot move completed temporary output {} to {}: {e}",
                stage.display(),
                output.display()
            )
        })?;
        Ok(())
    }

    fn extract_into_stage(&self, stage: &Path, paths: &[PathBuf]) -> Result<(), String> {
        let mut archive = File::open(&self.path)
            .map_err(|e| format!("cannot reopen archive {}: {e}", self.path.display()))?;
        let mut buffer = vec![0u8; 1024 * 1024];

        for (entry, relative) in self.entries.iter().zip(paths) {
            let destination = stage.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!(
                        "cannot create directory for entry {} ({:?}): {e}",
                        entry.index, entry.name
                    )
                })?;
            }
            let mut output = File::options()
                .write(true)
                .create_new(true)
                .open(&destination)
                .map_err(|e| {
                    format!(
                        "cannot create output for entry {} at {}: {e}",
                        entry.index,
                        destination.display()
                    )
                })?;
            let absolute_offset = self
                .data_base
                .checked_add(entry.offset)
                .ok_or_else(|| format!("entry {} absolute offset overflows", entry.index))?;
            archive
                .seek(SeekFrom::Start(absolute_offset))
                .map_err(|e| format!("cannot seek to entry {} payload: {e}", entry.index))?;

            let mut remaining = u64::from(entry.size);
            let mut payload_position = 0usize;
            while remaining != 0 {
                let amount = usize::try_from(remaining.min(buffer.len() as u64))
                    .map_err(|_| format!("entry {} chunk size does not fit memory", entry.index))?;
                archive.read_exact(&mut buffer[..amount]).map_err(|e| {
                    format!(
                        "cannot read entry {} ({:?}) at payload byte {}: {e}",
                        entry.index, entry.name, payload_position
                    )
                })?;
                for byte in &mut buffer[..amount] {
                    if self.header.flags & 1 != 0 {
                        *byte ^= entry.key[payload_position % KEY_SIZE];
                    }
                    payload_position += 1;
                }
                output.write_all(&buffer[..amount]).map_err(|e| {
                    format!(
                        "cannot write entry {} to {}: {e}",
                        entry.index,
                        destination.display()
                    )
                })?;
                remaining -= amount as u64;
            }
            output.flush().map_err(|e| {
                format!(
                    "cannot flush entry {} at {}: {e}",
                    entry.index,
                    destination.display()
                )
            })?;
        }
        Ok(())
    }
}

fn le_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().expect("u32 slice length"))
}

fn le_u64(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes.try_into().expect("u64 slice length"))
}

fn decrypt_name(index: u32, encrypted: &[u8], key: &[u8; KEY_SIZE]) -> Result<String, String> {
    let mut plain = [0u8; NAME_SIZE];
    for (position, byte) in encrypted.iter().enumerate() {
        plain[position] = *byte ^ key[position % KEY_SIZE];
    }
    let end = plain.iter().position(|b| *b == 0).ok_or_else(|| {
        format!("entry {index} filename has no NUL terminator in its 128-byte field")
    })?;
    if end == 0 {
        return Err(format!("entry {index} has an empty filename"));
    }
    String::from_utf8(plain[..end].to_vec())
        .map_err(|e| format!("entry {index} filename is not UTF-8/ASCII: {e}"))
}

pub fn safe_relative_path(index: u32, archive_name: &str) -> Result<PathBuf, String> {
    let mut result = PathBuf::new();
    for component in archive_name.split(['/', '\\']) {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." || component.contains(':') || component.contains('\0') {
            return Err(format!(
                "entry {index} has an unsafe path component in {archive_name:?}"
            ));
        }
        let trimmed = component.trim_end_matches([' ', '.']);
        if trimmed.is_empty() || is_windows_device_name(trimmed) {
            return Err(format!(
                "entry {index} cannot be represented safely on Windows: {archive_name:?}"
            ));
        }
        result.push(component);
    }
    if result.as_os_str().is_empty() {
        return Err(format!("entry {index} has no usable output path"));
    }
    Ok(result)
}

fn is_windows_device_name(component: &str) -> bool {
    let stem = component
        .split('.')
        .next()
        .unwrap_or(component)
        .to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    const TEST_KEY: [u8; KEY_SIZE] = *b"0123456789ABCDEFGHIJ";
    const TEST_NAME: &[u8] = b"\\dir\\hello.txt";
    const TEST_PAYLOAD: &[u8] = b"hello, nosurge\n";

    #[test]
    fn rejects_traversal_and_windows_devices() {
        assert!(safe_relative_path(0, "\\dir\\file.txt").is_ok());
        assert!(safe_relative_path(0, "..\\escape.txt").is_err());
        assert!(safe_relative_path(0, "C:\\absolute.txt").is_err());
        assert!(safe_relative_path(0, "dir\\CON.txt").is_err());
    }

    #[test]
    fn parses_and_extracts_a_synthetic_archive() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("nosurge_pak_test_{}_{}", std::process::id(), nonce));
        fs::create_dir(&root).expect("create test root");
        let pak = root.join("sample.pak");
        let output = root.join("out");
        fs::write(&pak, sample_archive_bytes(0)).expect("write sample archive");

        let archive = PakArchive::open(&pak).expect("parse sample archive");
        assert_eq!(archive.entries[0].name, "\\dir\\hello.txt");
        archive
            .extract_to(&output, false)
            .expect("extract sample archive");
        assert_eq!(
            fs::read(output.join("dir").join("hello.txt")).expect("read output"),
            TEST_PAYLOAD
        );
        fs::remove_dir_all(root).expect("clean test root");
    }

    #[test]
    fn rejects_truncated_and_inconsistent_archives() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "nosurge_pak_invalid_test_{}_{}",
            std::process::id(),
            nonce
        ));
        fs::create_dir(&root).expect("create test root");

        let truncated = root.join("truncated.pak");
        let valid = sample_archive_bytes(0);
        fs::write(&truncated, &valid[..HEADER_SIZE as usize + 4]).expect("write truncated PAK");
        assert!(PakArchive::open(&truncated).is_err());

        let inconsistent = root.join("inconsistent.pak");
        fs::write(&inconsistent, sample_archive_bytes(1)).expect("write inconsistent PAK");
        assert!(PakArchive::open(&inconsistent).is_err());

        fs::remove_dir_all(root).expect("clean test root");
    }

    fn sample_archive_bytes(offset: u64) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0x0002_0000u32.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&13u32.to_le_bytes());
        let mut entry = [0u8; ENTRY_SIZE as usize];
        let mut padded_name = [0u8; NAME_SIZE];
        padded_name[..TEST_NAME.len()].copy_from_slice(TEST_NAME);
        for position in 0..NAME_SIZE {
            entry[position] = padded_name[position] ^ TEST_KEY[position % KEY_SIZE];
        }
        entry[128..132].copy_from_slice(&(TEST_PAYLOAD.len() as u32).to_le_bytes());
        entry[132..152].copy_from_slice(&TEST_KEY);
        entry[152..160].copy_from_slice(&offset.to_le_bytes());
        bytes.extend_from_slice(&entry);
        bytes.extend(
            TEST_PAYLOAD
                .iter()
                .enumerate()
                .map(|(position, byte)| *byte ^ TEST_KEY[position % KEY_SIZE]),
        );
        bytes
    }
}
