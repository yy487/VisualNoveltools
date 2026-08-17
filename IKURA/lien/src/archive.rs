use crate::{Result, fail, hex_decode, hex_encode, sha256_hex};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const NAME_BYTES: usize = 12;
const ENTRY_BYTES: usize = 16;

#[derive(Clone, Debug)]
pub struct DrsEntry {
    pub name: String,
    pub name_raw: [u8; NAME_BYTES],
    pub offset: u32,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct DrsArchive {
    pub entries: Vec<DrsEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArchiveManifest {
    pub schema_version: u32,
    pub format: String,
    pub source_sha256: String,
    pub entries: Vec<ManifestEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ManifestEntry {
    pub index: usize,
    pub name: String,
    pub name_raw_hex: String,
    pub original_offset: u32,
    pub original_size: usize,
    pub original_sha256: String,
}

impl DrsArchive {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 2 + ENTRY_BYTES {
            return fail("DRS is too small to contain a directory and sentinel");
        }
        let directory_size = u16::from_le_bytes([data[0], data[1]]) as usize;
        if directory_size < ENTRY_BYTES || !directory_size.is_multiple_of(ENTRY_BYTES) {
            return fail(format!(
                "invalid DRS directory size 0x{directory_size:X}; expected a non-zero multiple of 16"
            ));
        }
        let directory_end = 2usize
            .checked_add(directory_size)
            .ok_or("DRS directory size overflow")?;
        if directory_end > data.len() {
            return fail(format!(
                "DRS directory ends at 0x{directory_end:X}, beyond file size 0x{:X}",
                data.len()
            ));
        }

        let slot_count = directory_size / ENTRY_BYTES;
        let file_count = slot_count - 1;
        let mut slots = Vec::with_capacity(slot_count);
        for index in 0..slot_count {
            let start = 2 + index * ENTRY_BYTES;
            let mut name_raw = [0u8; NAME_BYTES];
            name_raw.copy_from_slice(&data[start..start + NAME_BYTES]);
            let offset = u32::from_le_bytes(
                data[start + NAME_BYTES..start + ENTRY_BYTES]
                    .try_into()
                    .expect("fixed four-byte slice"),
            );
            slots.push((name_raw, offset));
        }

        if slots[file_count].0.iter().any(|&byte| byte != 0) {
            return fail("DRS terminal offset record must have an empty name");
        }
        let sentinel = slots[file_count].1 as usize;
        if sentinel != data.len() {
            return fail(format!(
                "DRS terminal offset 0x{sentinel:X} does not equal file size 0x{:X}",
                data.len()
            ));
        }
        if file_count > 0 && slots[0].1 as usize != directory_end {
            return fail(format!(
                "first DRS payload starts at 0x{:X}, expected directory end 0x{directory_end:X}",
                slots[0].1
            ));
        }

        let mut names = HashSet::with_capacity(file_count);
        let mut entries = Vec::with_capacity(file_count);
        for index in 0..file_count {
            let (name_raw, offset_u32) = slots[index];
            let next_offset = slots[index + 1].1 as usize;
            let offset = offset_u32 as usize;
            if offset < directory_end || offset >= next_offset || next_offset > data.len() {
                return fail(format!(
                    "invalid DRS payload bounds for entry {index}: 0x{offset:X}..0x{next_offset:X}"
                ));
            }
            let name = decode_name(&name_raw, index)?;
            validate_safe_member_name(&name)?;
            if !names.insert(name.to_uppercase()) {
                return fail(format!("duplicate DRS member name: {name}"));
            }
            entries.push(DrsEntry {
                name,
                name_raw,
                offset: offset_u32,
                data: data[offset..next_offset].to_vec(),
            });
        }
        Ok(Self { entries })
    }

    pub fn build(&self) -> Result<Vec<u8>> {
        let slot_count = self
            .entries
            .len()
            .checked_add(1)
            .ok_or("DRS entry count overflow")?;
        let directory_size = slot_count
            .checked_mul(ENTRY_BYTES)
            .ok_or("DRS directory size overflow")?;
        let directory_size_u16 = u16::try_from(directory_size)
            .map_err(|_| format!("DRS directory is too large: {directory_size} bytes"))?;
        let mut next_offset = 2usize
            .checked_add(directory_size)
            .ok_or("DRS output offset overflow")?;
        let mut seen = HashSet::with_capacity(self.entries.len());
        let mut offsets = Vec::with_capacity(self.entries.len());
        for (index, entry) in self.entries.iter().enumerate() {
            validate_safe_member_name(&entry.name)?;
            if !seen.insert(entry.name.to_uppercase()) {
                return fail(format!("duplicate DRS member name: {}", entry.name));
            }
            let decoded = decode_name(&entry.name_raw, index)?;
            if decoded != entry.name {
                return fail(format!(
                    "DRS name metadata mismatch at entry {index}: manifest={:?}, bytes={:?}",
                    entry.name, decoded
                ));
            }
            offsets.push(u32::try_from(next_offset).map_err(|_| "DRS output exceeds 4 GiB")?);
            next_offset = next_offset
                .checked_add(entry.data.len())
                .ok_or("DRS output size overflow")?;
        }
        let sentinel = u32::try_from(next_offset).map_err(|_| "DRS output exceeds 4 GiB")?;

        let mut output = Vec::with_capacity(next_offset);
        output.extend_from_slice(&directory_size_u16.to_le_bytes());
        for (entry, offset) in self.entries.iter().zip(offsets) {
            output.extend_from_slice(&entry.name_raw);
            output.extend_from_slice(&offset.to_le_bytes());
        }
        output.extend_from_slice(&[0u8; NAME_BYTES]);
        output.extend_from_slice(&sentinel.to_le_bytes());
        for entry in &self.entries {
            output.extend_from_slice(&entry.data);
        }
        Ok(output)
    }

    pub fn manifest(&self, source_data: &[u8]) -> ArchiveManifest {
        ArchiveManifest {
            schema_version: 1,
            format: "lien-drs".to_owned(),
            source_sha256: sha256_hex(source_data),
            entries: self
                .entries
                .iter()
                .enumerate()
                .map(|(index, entry)| ManifestEntry {
                    index,
                    name: entry.name.clone(),
                    name_raw_hex: hex_encode(&entry.name_raw),
                    original_offset: entry.offset,
                    original_size: entry.data.len(),
                    original_sha256: sha256_hex(&entry.data),
                })
                .collect(),
        }
    }

    pub fn unpack_to(&self, source_data: &[u8], output: &Path, overwrite: bool) -> Result<()> {
        prepare_directory(output, overwrite)?;
        for entry in &self.entries {
            let target = output.join(&entry.name);
            write_new_or_overwrite(&target, &entry.data, overwrite)?;
        }
        let mut manifest = serde_json::to_vec_pretty(&self.manifest(source_data))?;
        manifest.push(b'\n');
        write_new_or_overwrite(&output.join("lien-drs-manifest.json"), &manifest, overwrite)?;
        Ok(())
    }

    pub fn from_unpacked(input: &Path, manifest_path: &Path) -> Result<Self> {
        let manifest_data = fs::read(manifest_path)?;
        let manifest: ArchiveManifest = serde_json::from_slice(&manifest_data)?;
        if manifest.schema_version != 1 || manifest.format != "lien-drs" {
            return fail("unsupported or invalid DRS manifest");
        }
        let mut entries = Vec::with_capacity(manifest.entries.len());
        for (expected_index, record) in manifest.entries.iter().enumerate() {
            if record.index != expected_index {
                return fail(format!(
                    "DRS manifest index mismatch: expected {expected_index}, got {}",
                    record.index
                ));
            }
            validate_safe_member_name(&record.name)?;
            let raw = hex_decode(&record.name_raw_hex)?;
            let name_raw: [u8; NAME_BYTES] = raw.try_into().map_err(|value: Vec<u8>| {
                format!(
                    "DRS manifest name bytes for {} have length {}, expected 12",
                    record.name,
                    value.len()
                )
            })?;
            let decoded = decode_name(&name_raw, expected_index)?;
            if decoded != record.name {
                return fail(format!(
                    "DRS manifest name mismatch at entry {expected_index}: {:?} != {:?}",
                    decoded, record.name
                ));
            }
            let data = fs::read(input.join(&record.name))
                .map_err(|error| format!("failed to read DRS member {}: {error}", record.name))?;
            entries.push(DrsEntry {
                name: record.name.clone(),
                name_raw,
                offset: record.original_offset,
                data,
            });
        }
        Ok(Self { entries })
    }
}

fn decode_name(raw: &[u8; NAME_BYTES], index: usize) -> Result<String> {
    let nul = raw.iter().position(|&byte| byte == 0).unwrap_or(NAME_BYTES);
    if raw[nul..].iter().any(|&byte| byte != 0) {
        return fail(format!(
            "DRS entry {index} has non-zero bytes after its name terminator"
        ));
    }
    if nul == 0 {
        return fail(format!("DRS entry {index} has an empty name"));
    }
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(&raw[..nul]);
    if had_errors {
        return fail(format!("DRS entry {index} name is not valid CP932"));
    }
    Ok(decoded.into_owned())
}

fn validate_safe_member_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains(['/', '\\', ':'])
        || name.chars().any(char::is_control)
    {
        return fail(format!("unsafe DRS member name: {name:?}"));
    }
    Ok(())
}

pub fn prepare_directory(path: &Path, overwrite: bool) -> Result<()> {
    if path.exists() {
        if !path.is_dir() {
            return fail(format!(
                "output exists and is not a directory: {}",
                path.display()
            ));
        }
        if !overwrite && fs::read_dir(path)?.next().is_some() {
            return fail(format!(
                "output directory already exists and is not empty: {} (use --overwrite)",
                path.display()
            ));
        }
    } else {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn write_new_or_overwrite(path: &Path, data: &[u8], overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        return fail(format!(
            "output already exists: {} (use --overwrite)",
            path.display()
        ));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, data)?;
    Ok(())
}

pub fn read_archive(path: &Path) -> Result<(Vec<u8>, DrsArchive)> {
    let data = fs::read(path)?;
    let archive = DrsArchive::parse(&data)?;
    Ok((data, archive))
}

pub fn default_unpack_output(input: &Path) -> PathBuf {
    let stem = input
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "archive".to_owned());
    input.with_file_name(format!("{stem}_unpacked"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_archive() -> Vec<u8> {
        let entries = vec![
            DrsEntry {
                name: "A.ISF".to_owned(),
                name_raw: *b"A.ISF\0\0\0\0\0\0\0",
                offset: 0,
                data: vec![1, 2, 3],
            },
            DrsEntry {
                name: "B.ISF".to_owned(),
                name_raw: *b"B.ISF\0\0\0\0\0\0\0",
                offset: 0,
                data: vec![4, 5],
            },
        ];
        DrsArchive { entries }.build().unwrap()
    }

    #[test]
    fn drs_roundtrip_is_exact() {
        let bytes = sample_archive();
        let parsed = DrsArchive::parse(&bytes).unwrap();
        assert_eq!(parsed.build().unwrap(), bytes);
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(validate_safe_member_name("../A.ISF").is_err());
        assert!(validate_safe_member_name("A\\B.ISF").is_err());
    }

    #[test]
    fn rejects_bad_sentinel() {
        let mut bytes = sample_archive();
        let sentinel_name = 2 + 2 * ENTRY_BYTES;
        bytes[sentinel_name] = b'X';
        assert!(DrsArchive::parse(&bytes).is_err());
    }
}
