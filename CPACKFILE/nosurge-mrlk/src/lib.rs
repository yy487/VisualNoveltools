use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub const MAGIC: [u8; 4] = *b"MRLK";
pub const HEADER_SIZE: u64 = 24;
pub const ENTRY_SIZE: u64 = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MrlkHeader {
    pub reserved: u32,
    pub table_end: u32,
    pub file_count: u32,
    pub names_offset: u32,
    pub names_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MrlkEntry {
    pub index: u32,
    pub name: String,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug)]
pub struct MrlkArchive {
    path: PathBuf,
    raw_names: Vec<u8>,
    pub header: MrlkHeader,
    pub entries: Vec<MrlkEntry>,
    pub data_offset: u64,
    pub file_size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackSummary {
    pub file_count: usize,
    pub payload_bytes: u64,
    pub output_bytes: u64,
}

impl MrlkArchive {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        let mut file = File::open(&path)
            .map_err(|error| format!("cannot open archive {}: {error}", path.display()))?;
        let file_size = file
            .metadata()
            .map_err(|error| format!("cannot read metadata for {}: {error}", path.display()))?
            .len();
        if file_size < HEADER_SIZE {
            return Err(format!(
                "archive {} is truncated: expected at least {HEADER_SIZE} bytes, found {file_size}",
                path.display()
            ));
        }

        let mut raw_header = [0u8; HEADER_SIZE as usize];
        file.read_exact(&mut raw_header)
            .map_err(|error| format!("cannot read header from {}: {error}", path.display()))?;
        if raw_header[0..4] != MAGIC {
            return Err(format!(
                "{} is not an MRLK archive (magic is {:02X?})",
                path.display(),
                &raw_header[0..4]
            ));
        }

        let header = MrlkHeader {
            reserved: le_u32(&raw_header[4..8]),
            table_end: le_u32(&raw_header[8..12]),
            file_count: le_u32(&raw_header[12..16]),
            names_offset: le_u32(&raw_header[16..20]),
            names_size: le_u32(&raw_header[20..24]),
        };
        if header.reserved != 0 {
            return Err(format!(
                "unsupported MRLK reserved field in {}: 0x{:08X} (the game requires zero)",
                path.display(),
                header.reserved
            ));
        }

        let table_bytes = u64::from(header.file_count)
            .checked_mul(ENTRY_SIZE)
            .ok_or_else(|| "MRLK table size overflow".to_owned())?;
        let expected_table_end = HEADER_SIZE
            .checked_add(table_bytes)
            .ok_or_else(|| "MRLK table end overflow".to_owned())?;
        if u64::from(header.table_end) != expected_table_end {
            return Err(format!(
                "inconsistent MRLK table end in {}: header=0x{:X}, expected=0x{expected_table_end:X}",
                path.display(),
                header.table_end
            ));
        }
        if header.names_offset != header.table_end {
            return Err(format!(
                "unsupported gap before the MRLK name table in {}: table ends at 0x{:X}, names start at 0x{:X}",
                path.display(),
                header.table_end,
                header.names_offset
            ));
        }

        let data_offset = u64::from(header.names_offset)
            .checked_add(u64::from(header.names_size))
            .ok_or_else(|| "MRLK name table range overflow".to_owned())?;
        if data_offset > file_size {
            return Err(format!(
                "truncated MRLK name table in {}: data would start at 0x{data_offset:X}, file ends at 0x{file_size:X}",
                path.display()
            ));
        }

        let entry_capacity = usize::try_from(header.file_count)
            .map_err(|_| "MRLK entry count does not fit memory".to_owned())?;
        let table_len = usize::try_from(table_bytes)
            .map_err(|_| "MRLK table size does not fit memory".to_owned())?;
        let mut raw_table = vec![0u8; table_len];
        file.read_exact(&mut raw_table).map_err(|error| {
            format!(
                "cannot read {}-entry table from {}: {error}",
                header.file_count,
                path.display()
            )
        })?;

        file.seek(SeekFrom::Start(u64::from(header.names_offset)))
            .map_err(|error| format!("cannot seek to MRLK names in {}: {error}", path.display()))?;
        let names_len = usize::try_from(header.names_size)
            .map_err(|_| "MRLK name table size does not fit memory".to_owned())?;
        let mut raw_names = vec![0u8; names_len];
        file.read_exact(&mut raw_names)
            .map_err(|error| format!("cannot read MRLK names from {}: {error}", path.display()))?;
        let names = parse_names(&raw_names, header.file_count, &path)?;

        let mut entries = Vec::with_capacity(entry_capacity);
        let mut casefolded_names = HashSet::with_capacity(entry_capacity);
        let mut expected_offset = data_offset;
        for (index, name) in names.into_iter().enumerate() {
            let record = index * ENTRY_SIZE as usize;
            let offset = le_u32(&raw_table[record..record + 4]);
            let size = le_u32(&raw_table[record + 4..record + 8]);
            if u64::from(offset) != expected_offset {
                return Err(format!(
                    "entry {index} ({name:?}) in {} is not contiguous: offset=0x{offset:X}, expected=0x{expected_offset:X}",
                    path.display()
                ));
            }
            let end = u64::from(offset)
                .checked_add(u64::from(size))
                .ok_or_else(|| format!("entry {index} ({name:?}) range overflows"))?;
            if end > file_size {
                return Err(format!(
                    "entry {index} ({name:?}) in {} ends at 0x{end:X}, past file end 0x{file_size:X}",
                    path.display()
                ));
            }

            let relative = safe_relative_path(index, &name)?;
            let collision_key = relative.to_string_lossy().replace('\\', "/").to_lowercase();
            if !casefolded_names.insert(collision_key) {
                return Err(format!(
                    "entry {index} collides with another output path on a case-insensitive filesystem: {name:?}"
                ));
            }

            entries.push(MrlkEntry {
                index: index as u32,
                name,
                offset,
                size,
            });
            expected_offset = end;
        }
        if expected_offset != file_size {
            return Err(format!(
                "MRLK payload coverage mismatch in {}: table ends at 0x{expected_offset:X}, file ends at 0x{file_size:X}",
                path.display()
            ));
        }

        Ok(Self {
            path,
            raw_names,
            header,
            entries,
            data_offset,
            file_size,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn payload_bytes(&self) -> u64 {
        self.file_size - self.data_offset
    }

    pub fn extract_to(&self, output: impl AsRef<Path>, overwrite: bool) -> Result<(), String> {
        let output = output.as_ref();
        let output_name = final_component(output, "output directory")?;
        if output.exists() && !overwrite {
            return Err(format!(
                "output already exists: {} (pass --overwrite or approve replacement interactively)",
                output.display()
            ));
        }

        let mut relative_paths = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            relative_paths.push(safe_relative_path(entry.index as usize, &entry.name)?);
        }

        let parent = usable_parent(output);
        let stage = parent.join(temporary_name(&output_name, "tmp"));
        let backup = parent.join(temporary_name(&output_name, "backup"));
        ensure_staging_paths_are_free(&stage, &backup)?;
        fs::create_dir_all(parent).map_err(|error| {
            format!("cannot create output parent {}: {error}", parent.display())
        })?;
        fs::create_dir(&stage).map_err(|error| {
            format!(
                "cannot create temporary output {}: {error}",
                stage.display()
            )
        })?;

        let result = self.extract_into_stage(&stage, &relative_paths);
        if let Err(error) = result {
            let _ = fs::remove_dir_all(&stage);
            return Err(error);
        }
        commit_staged_path(&stage, output, &backup, overwrite)
    }

    pub fn pack_from_directory(
        &self,
        input: impl AsRef<Path>,
        output: impl AsRef<Path>,
        overwrite: bool,
    ) -> Result<PackSummary, String> {
        let input = input.as_ref();
        let output = output.as_ref();
        if !input.is_dir() {
            return Err(format!("input is not a directory: {}", input.display()));
        }
        if output.exists() {
            if output.is_dir() {
                return Err(format!(
                    "output archive path is a directory: {}",
                    output.display()
                ));
            }
            if same_existing_path(&self.path, output)? {
                return Err(format!(
                    "refusing to overwrite the template archive: {}",
                    self.path.display()
                ));
            }
            if !overwrite {
                return Err(format!(
                    "output already exists: {} (pass --overwrite or approve replacement interactively)",
                    output.display()
                ));
            }
        }
        let output_name = final_component(output, "output archive")?;

        let mut inputs = Vec::with_capacity(self.entries.len());
        let mut rebuilt_records = Vec::with_capacity(self.entries.len());
        let mut next_offset = self.data_offset;
        let mut payload_bytes = 0u64;
        for entry in &self.entries {
            let relative = safe_relative_path(entry.index as usize, &entry.name)?;
            let source = input.join(relative);
            let metadata = fs::metadata(&source).map_err(|error| {
                format!(
                    "missing or unreadable input for entry {} ({:?}) at {}: {error}",
                    entry.index,
                    entry.name,
                    source.display()
                )
            })?;
            if !metadata.is_file() {
                return Err(format!(
                    "input for entry {} ({:?}) is not a file: {}",
                    entry.index,
                    entry.name,
                    source.display()
                ));
            }
            if output.exists() && same_existing_path(&source, output)? {
                return Err(format!(
                    "output archive would overwrite input entry {} ({:?})",
                    entry.index, entry.name
                ));
            }
            let size = u32::try_from(metadata.len()).map_err(|_| {
                format!(
                    "entry {} ({:?}) is too large for MRLK: {} bytes",
                    entry.index,
                    entry.name,
                    metadata.len()
                )
            })?;
            let offset = u32::try_from(next_offset).map_err(|_| {
                format!(
                    "rebuilt archive exceeds the MRLK 32-bit offset limit before entry {} ({:?})",
                    entry.index, entry.name
                )
            })?;
            next_offset = next_offset
                .checked_add(u64::from(size))
                .ok_or_else(|| "rebuilt MRLK size overflow".to_owned())?;
            if next_offset > u64::from(u32::MAX) {
                return Err(format!(
                    "rebuilt archive exceeds the MRLK 32-bit size limit at entry {} ({:?})",
                    entry.index, entry.name
                ));
            }
            payload_bytes = payload_bytes
                .checked_add(u64::from(size))
                .ok_or_else(|| "rebuilt payload size overflow".to_owned())?;
            inputs.push(PackInput { path: source, size });
            rebuilt_records.push((offset, size));
        }

        let parent = usable_parent(output);
        let stage = parent.join(temporary_name(&output_name, "tmp"));
        let backup = parent.join(temporary_name(&output_name, "backup"));
        ensure_staging_paths_are_free(&stage, &backup)?;
        fs::create_dir_all(parent).map_err(|error| {
            format!("cannot create output parent {}: {error}", parent.display())
        })?;

        let result = self.write_rebuilt_archive(&stage, &inputs, &rebuilt_records);
        if let Err(error) = result {
            let _ = fs::remove_file(&stage);
            return Err(error);
        }

        let rebuilt = match Self::open(&stage) {
            Ok(archive) => archive,
            Err(error) => {
                let _ = fs::remove_file(&stage);
                return Err(format!("rebuilt archive failed validation: {error}"));
            }
        };
        if rebuilt.entries.len() != self.entries.len() || rebuilt.file_size != next_offset {
            let _ = fs::remove_file(&stage);
            return Err(
                "rebuilt archive validation totals do not match the prepared output".to_owned(),
            );
        }

        commit_staged_path(&stage, output, &backup, overwrite)?;
        Ok(PackSummary {
            file_count: inputs.len(),
            payload_bytes,
            output_bytes: next_offset,
        })
    }

    fn extract_into_stage(&self, stage: &Path, paths: &[PathBuf]) -> Result<(), String> {
        let mut archive = File::open(&self.path)
            .map_err(|error| format!("cannot reopen archive {}: {error}", self.path.display()))?;
        let mut buffer = vec![0u8; 1024 * 1024];
        for (entry, relative) in self.entries.iter().zip(paths) {
            let destination = stage.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    format!(
                        "cannot create directory for entry {} ({:?}): {error}",
                        entry.index, entry.name
                    )
                })?;
            }
            let mut output = File::options()
                .write(true)
                .create_new(true)
                .open(&destination)
                .map_err(|error| {
                    format!(
                        "cannot create output for entry {} ({:?}) at {}: {error}",
                        entry.index,
                        entry.name,
                        destination.display()
                    )
                })?;
            archive
                .seek(SeekFrom::Start(u64::from(entry.offset)))
                .map_err(|error| {
                    format!("cannot seek to entry {} payload: {error}", entry.index)
                })?;
            copy_exact(
                &mut archive,
                &mut output,
                u64::from(entry.size),
                &mut buffer,
                &format!("entry {} ({:?})", entry.index, entry.name),
            )?;
            output.flush().map_err(|error| {
                format!(
                    "cannot flush entry {} at {}: {error}",
                    entry.index,
                    destination.display()
                )
            })?;
        }
        Ok(())
    }

    fn write_rebuilt_archive(
        &self,
        stage: &Path,
        inputs: &[PackInput],
        records: &[(u32, u32)],
    ) -> Result<(), String> {
        let mut output = File::options()
            .write(true)
            .create_new(true)
            .open(stage)
            .map_err(|error| {
                format!(
                    "cannot create temporary archive {}: {error}",
                    stage.display()
                )
            })?;
        output
            .write_all(&MAGIC)
            .and_then(|_| output.write_all(&self.header.reserved.to_le_bytes()))
            .and_then(|_| output.write_all(&self.header.table_end.to_le_bytes()))
            .and_then(|_| output.write_all(&self.header.file_count.to_le_bytes()))
            .and_then(|_| output.write_all(&self.header.names_offset.to_le_bytes()))
            .and_then(|_| output.write_all(&self.header.names_size.to_le_bytes()))
            .map_err(|error| format!("cannot write MRLK header: {error}"))?;
        for &(offset, size) in records {
            output
                .write_all(&offset.to_le_bytes())
                .and_then(|_| output.write_all(&size.to_le_bytes()))
                .map_err(|error| format!("cannot write MRLK file table: {error}"))?;
        }
        output
            .write_all(&self.raw_names)
            .map_err(|error| format!("cannot write MRLK name table: {error}"))?;

        let mut buffer = vec![0u8; 1024 * 1024];
        for (entry, source) in self.entries.iter().zip(inputs) {
            let mut input = File::open(&source.path).map_err(|error| {
                format!(
                    "cannot open input for entry {} ({:?}) at {}: {error}",
                    entry.index,
                    entry.name,
                    source.path.display()
                )
            })?;
            copy_exact(
                &mut input,
                &mut output,
                u64::from(source.size),
                &mut buffer,
                &format!("entry {} ({:?})", entry.index, entry.name),
            )?;
            let mut extra = [0u8; 1];
            if input.read(&mut extra).map_err(|error| {
                format!("cannot recheck input {}: {error}", source.path.display())
            })? != 0
            {
                return Err(format!(
                    "input changed size while packing entry {} ({:?}); no output was installed",
                    entry.index, entry.name
                ));
            }
        }
        output.flush().map_err(|error| {
            format!(
                "cannot flush temporary archive {}: {error}",
                stage.display()
            )
        })?;
        output.sync_all().map_err(|error| {
            format!(
                "cannot finish temporary archive {}: {error}",
                stage.display()
            )
        })?;
        Ok(())
    }
}

#[derive(Debug)]
struct PackInput {
    path: PathBuf,
    size: u32,
}

fn parse_names(raw: &[u8], expected_count: u32, archive: &Path) -> Result<Vec<String>, String> {
    if expected_count == 0 {
        if raw.is_empty() {
            return Ok(Vec::new());
        }
        return Err(format!(
            "{} has name bytes but declares zero entries",
            archive.display()
        ));
    }
    if raw.len() < 2 || !raw.ends_with(b"\r\n") {
        return Err(format!(
            "MRLK name table in {} does not end with CRLF",
            archive.display()
        ));
    }

    let expected = usize::try_from(expected_count)
        .map_err(|_| "MRLK name count does not fit memory".to_owned())?;
    let mut names = Vec::with_capacity(expected);
    let mut start = 0usize;
    while start < raw.len() {
        let relative_end = raw[start..]
            .windows(2)
            .position(|pair| pair == b"\r\n")
            .ok_or_else(|| format!("unterminated MRLK name in {}", archive.display()))?;
        let end = start + relative_end;
        let name_bytes = &raw[start..end];
        if name_bytes.is_empty() {
            return Err(format!("empty MRLK filename in {}", archive.display()));
        }
        if name_bytes.contains(&b'\r') || name_bytes.contains(&b'\n') {
            return Err(format!(
                "isolated CR or LF inside an MRLK filename in {}",
                archive.display()
            ));
        }
        let name = std::str::from_utf8(name_bytes).map_err(|error| {
            format!(
                "MRLK filename {} in {} is not UTF-8/ASCII: {error}",
                names.len(),
                archive.display()
            )
        })?;
        names.push(name.to_owned());
        start = end + 2;
    }
    if names.len() != expected {
        return Err(format!(
            "MRLK filename count mismatch in {}: header={}, names={}",
            archive.display(),
            expected_count,
            names.len()
        ));
    }
    Ok(names)
}

pub fn safe_relative_path(index: usize, archive_name: &str) -> Result<PathBuf, String> {
    if archive_name.is_empty() || archive_name.starts_with(['/', '\\']) {
        return Err(format!(
            "entry {index} has an empty or absolute archive path: {archive_name:?}"
        ));
    }
    let mut result = PathBuf::new();
    for component in archive_name.split(['/', '\\']) {
        if component.is_empty() || component == "." || component == ".." {
            return Err(format!(
                "entry {index} has an unsafe path component in {archive_name:?}"
            ));
        }
        if component.contains(':')
            || component.contains('\0')
            || component.chars().any(|character| {
                character < ' ' || matches!(character, '<' | '>' | '"' | '|' | '?' | '*')
            })
        {
            return Err(format!(
                "entry {index} contains characters unsafe on Windows: {archive_name:?}"
            ));
        }
        if component.trim_end_matches([' ', '.']) != component || is_windows_device_name(component)
        {
            return Err(format!(
                "entry {index} cannot be represented safely on Windows: {archive_name:?}"
            ));
        }
        result.push(component);
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

fn copy_exact(
    input: &mut File,
    output: &mut File,
    size: u64,
    buffer: &mut [u8],
    context: &str,
) -> Result<(), String> {
    let mut remaining = size;
    let mut position = 0u64;
    while remaining != 0 {
        let amount = usize::try_from(remaining.min(buffer.len() as u64))
            .map_err(|_| format!("{context} chunk size does not fit memory"))?;
        input.read_exact(&mut buffer[..amount]).map_err(|error| {
            format!("cannot read {context} at payload byte {position}: {error}")
        })?;
        output.write_all(&buffer[..amount]).map_err(|error| {
            format!("cannot write {context} at payload byte {position}: {error}")
        })?;
        remaining -= amount as u64;
        position += amount as u64;
    }
    Ok(())
}

fn le_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().expect("four-byte u32 slice"))
}

fn final_component(path: &Path, role: &str) -> Result<OsString, String> {
    path.file_name()
        .map(OsString::from)
        .ok_or_else(|| format!("refusing {role} without a final name: {}", path.display()))
}

fn usable_parent(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn temporary_name(final_name: &OsString, role: &str) -> OsString {
    let mut name = OsString::from(".");
    name.push(final_name);
    name.push(format!(".nosurge_mrlk_{role}_{}", std::process::id()));
    name
}

fn ensure_staging_paths_are_free(stage: &Path, backup: &Path) -> Result<(), String> {
    if stage.exists() {
        return Err(format!(
            "temporary output already exists; remove it before retrying: {}",
            stage.display()
        ));
    }
    if backup.exists() {
        return Err(format!(
            "temporary backup already exists; inspect it before retrying: {}",
            backup.display()
        ));
    }
    Ok(())
}

fn commit_staged_path(
    stage: &Path,
    destination: &Path,
    backup: &Path,
    overwrite: bool,
) -> Result<(), String> {
    if !destination.exists() {
        return fs::rename(stage, destination).map_err(|error| {
            format!(
                "cannot install completed output {} as {}: {error}",
                stage.display(),
                destination.display()
            )
        });
    }
    if !overwrite {
        return Err(format!("output already exists: {}", destination.display()));
    }

    fs::rename(destination, backup).map_err(|error| {
        format!(
            "cannot move existing output {} to temporary backup {}: {error}",
            destination.display(),
            backup.display()
        )
    })?;
    match fs::rename(stage, destination) {
        Ok(()) => remove_path(backup).map_err(|error| {
            format!(
                "new output is complete at {}, but the old backup could not be removed from {}: {error}",
                destination.display(),
                backup.display()
            )
        }),
        Err(install_error) => match fs::rename(backup, destination) {
            Ok(()) => Err(format!(
                "cannot install completed output at {}; the previous output was restored: {install_error}",
                destination.display()
            )),
            Err(restore_error) => Err(format!(
                "cannot install completed output at {} ({install_error}); previous output remains at {} because restoration also failed ({restore_error})",
                destination.display(),
                backup.display()
            )),
        },
    }
}

fn remove_path(path: &Path) -> std::io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn same_existing_path(left: &Path, right: &Path) -> Result<bool, String> {
    let left = fs::canonicalize(left)
        .map_err(|error| format!("cannot resolve {}: {error}", left.display()))?;
    let right = fs::canonicalize(right)
        .map_err(|error| format!("cannot resolve {}: {error}", right.display()))?;
    #[cfg(windows)]
    {
        Ok(left
            .to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy()))
    }
    #[cfg(not(windows))]
    {
        Ok(left == right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    const FIRST_NAME: &str = "dir/hello.g1t";
    const SECOND_NAME: &str = "plain.bin";
    const FIRST_PAYLOAD: &[u8] = b"GT1Gsample-one";
    const SECOND_PAYLOAD: &[u8] = b"opaque-two";

    #[test]
    fn rejects_unsafe_paths() {
        assert!(safe_relative_path(0, "dir/file.g1t").is_ok());
        assert!(safe_relative_path(0, "../escape.g1t").is_err());
        assert!(safe_relative_path(0, "C:\\absolute.g1t").is_err());
        assert!(safe_relative_path(0, "\\rooted.g1t").is_err());
        assert!(safe_relative_path(0, "dir/CON.txt").is_err());
        assert!(safe_relative_path(0, "dir//file.g1t").is_err());
    }

    #[test]
    fn parses_and_rejects_malformed_archives() {
        let root = unique_root("parse");
        fs::create_dir(&root).expect("create test root");
        let valid_path = root.join("valid.psarc");
        fs::write(&valid_path, sample_archive()).expect("write valid archive");
        let archive = MrlkArchive::open(&valid_path).expect("parse valid archive");
        assert_eq!(archive.entries.len(), 2);
        assert_eq!(archive.entries[0].name, FIRST_NAME);
        assert_eq!(archive.entries[1].name, SECOND_NAME);

        let mut reserved = sample_archive();
        reserved[4..8].copy_from_slice(&1u32.to_le_bytes());
        let reserved_path = root.join("reserved.psarc");
        fs::write(&reserved_path, reserved).expect("write reserved archive");
        assert!(MrlkArchive::open(&reserved_path).is_err());

        let mut discontinuous = sample_archive();
        let original_offset = le_u32(&discontinuous[24..28]);
        discontinuous[24..28].copy_from_slice(&(original_offset + 1).to_le_bytes());
        let discontinuous_path = root.join("discontinuous.psarc");
        fs::write(&discontinuous_path, discontinuous).expect("write discontinuous archive");
        assert!(MrlkArchive::open(&discontinuous_path).is_err());

        assert!(parse_names(b"one.g1t\none-more.g1t\n", 2, Path::new("bad")).is_err());
        assert!(parse_names(b"one.g1t\r\n\r\n", 2, Path::new("bad")).is_err());
        fs::remove_dir_all(root).expect("clean test root");
    }

    #[test]
    fn unchanged_unpack_pack_is_byte_exact() {
        let root = unique_root("roundtrip");
        fs::create_dir(&root).expect("create test root");
        let source = root.join("source.psarc");
        let unpacked = root.join("unpacked");
        let rebuilt = root.join("rebuilt.psarc");
        let original = sample_archive();
        fs::write(&source, &original).expect("write source archive");

        let archive = MrlkArchive::open(&source).expect("parse source archive");
        archive
            .extract_to(&unpacked, false)
            .expect("unpack source archive");
        let summary = archive
            .pack_from_directory(&unpacked, &rebuilt, false)
            .expect("pack unchanged directory");
        assert_eq!(summary.file_count, 2);
        assert_eq!(fs::read(&rebuilt).expect("read rebuilt archive"), original);
        fs::remove_dir_all(root).expect("clean test root");
    }

    #[test]
    fn modified_pack_rebuilds_offsets_and_preserves_other_files() {
        let root = unique_root("modified");
        fs::create_dir(&root).expect("create test root");
        let source = root.join("source.psarc");
        let unpacked = root.join("unpacked");
        let rebuilt = root.join("rebuilt.psarc");
        let checked = root.join("checked");
        fs::write(&source, sample_archive()).expect("write source archive");

        let archive = MrlkArchive::open(&source).expect("parse source archive");
        archive
            .extract_to(&unpacked, false)
            .expect("unpack source archive");
        let replacement = b"GT1Gthis replacement is longer than the original";
        fs::write(unpacked.join("dir").join("hello.g1t"), replacement).expect("write replacement");
        archive
            .pack_from_directory(&unpacked, &rebuilt, false)
            .expect("pack modified directory");

        let rebuilt_archive = MrlkArchive::open(&rebuilt).expect("parse rebuilt archive");
        assert_eq!(rebuilt_archive.entries[0].size, replacement.len() as u32);
        assert_eq!(
            rebuilt_archive.entries[1].offset,
            rebuilt_archive.entries[0].offset + rebuilt_archive.entries[0].size
        );
        rebuilt_archive
            .extract_to(&checked, false)
            .expect("unpack rebuilt archive");
        assert_eq!(
            fs::read(checked.join("dir").join("hello.g1t")).expect("read replacement"),
            replacement
        );
        assert_eq!(
            fs::read(checked.join("plain.bin")).expect("read preserved file"),
            SECOND_PAYLOAD
        );
        fs::remove_dir_all(root).expect("clean test root");
    }

    #[test]
    fn packing_requires_every_template_entry_and_never_overwrites_template() {
        let root = unique_root("missing");
        fs::create_dir(&root).expect("create test root");
        let source = root.join("source.psarc");
        let unpacked = root.join("unpacked");
        let output = root.join("output.psarc");
        fs::write(&source, sample_archive()).expect("write source archive");
        let archive = MrlkArchive::open(&source).expect("parse source archive");
        archive
            .extract_to(&unpacked, false)
            .expect("unpack source archive");
        fs::remove_file(unpacked.join("plain.bin")).expect("remove required input");
        assert!(archive
            .pack_from_directory(&unpacked, &output, false)
            .is_err());
        assert!(!output.exists());
        assert!(archive
            .pack_from_directory(&unpacked, &source, true)
            .is_err());
        fs::remove_dir_all(root).expect("clean test root");
    }

    fn sample_archive() -> Vec<u8> {
        let names = format!("{FIRST_NAME}\r\n{SECOND_NAME}\r\n").into_bytes();
        let count = 2u32;
        let table_end = HEADER_SIZE as u32 + count * ENTRY_SIZE as u32;
        let names_size = names.len() as u32;
        let first_offset = table_end + names_size;
        let second_offset = first_offset + FIRST_PAYLOAD.len() as u32;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC);
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&table_end.to_le_bytes());
        bytes.extend_from_slice(&count.to_le_bytes());
        bytes.extend_from_slice(&table_end.to_le_bytes());
        bytes.extend_from_slice(&names_size.to_le_bytes());
        bytes.extend_from_slice(&first_offset.to_le_bytes());
        bytes.extend_from_slice(&(FIRST_PAYLOAD.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&second_offset.to_le_bytes());
        bytes.extend_from_slice(&(SECOND_PAYLOAD.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&names);
        bytes.extend_from_slice(FIRST_PAYLOAD);
        bytes.extend_from_slice(SECOND_PAYLOAD);
        bytes
    }

    fn unique_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "nosurge_mrlk_{label}_{}_{}",
            std::process::id(),
            nonce
        ))
    }
}
