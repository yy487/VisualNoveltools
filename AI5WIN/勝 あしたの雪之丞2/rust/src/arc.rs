use anyhow::{bail, ensure, Context, Result};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const ENTRY_SIZE: usize = 0x20;
const NAME_SIZE: usize = 24;
const ARC_KEY: [u8; 8] = [0x51, 0x92, 0xAB, 0xF1, 0x6E, 0x30, 0x7D, 0x48];
const XOR_SIZE: u32 = 0x7D30_6EF1;
const XOR_OFFSET: u32 = 0x6EF1_AB92;

#[derive(Clone, Debug, Serialize)]
pub struct ArcEntry {
    pub index: usize,
    pub name: String,
    pub size: u32,
    pub offset: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct VerifyResult {
    pub entries: usize,
    pub replaced: usize,
    pub unchanged_exact: usize,
}

pub fn read_index(path: &Path) -> Result<Vec<ArcEntry>> {
    let file = File::open(path).with_context(|| format!("open ARC: {}", path.display()))?;
    let file_len = file.metadata()?.len();
    let mut reader = BufReader::new(file);
    let count = read_u32(&mut reader)? as usize;
    let table_size = 4u64
        .checked_add(
            (count as u64)
                .checked_mul(ENTRY_SIZE as u64)
                .context("ARC table overflow")?,
        )
        .context("ARC table overflow")?;
    ensure!(
        table_size <= file_len,
        "ARC index exceeds file size: count={count}, table={table_size}, file={file_len}"
    );

    let mut entries = Vec::with_capacity(count);
    let mut names = HashSet::with_capacity(count);
    for index in 0..count {
        let mut encrypted_name = [0u8; NAME_SIZE];
        reader.read_exact(&mut encrypted_name)?;
        let name = decode_name(&encrypted_name)
            .with_context(|| format!("invalid ARC filename at entry {index}"))?;
        ensure!(
            names.insert(normalize_name(&name)),
            "duplicate ARC filename: {name}"
        );

        let size = decode_u32(read_u32(&mut reader)?, XOR_SIZE);
        let offset = decode_u32(read_u32(&mut reader)?, XOR_OFFSET);
        let end = u64::from(offset)
            .checked_add(u64::from(size))
            .context("ARC entry range overflow")?;
        ensure!(
            u64::from(offset) >= table_size && end <= file_len,
            "ARC entry out of range: index={index}, name={name}, offset={offset:#x}, size={size}"
        );
        entries.push(ArcEntry {
            index,
            name,
            size,
            offset,
        });
    }
    Ok(entries)
}

pub fn read_entry(path: &Path, wanted_name: &str) -> Result<Vec<u8>> {
    let wanted = normalize_name(wanted_name);
    let entry = read_index(path)?
        .into_iter()
        .find(|entry| normalize_name(&entry.name) == wanted)
        .with_context(|| format!("ARC entry not found: {wanted_name} in {}", path.display()))?;
    read_entry_data(path, &entry)
}

pub fn unpack(path: &Path, output_dir: &Path) -> Result<Vec<ArcEntry>> {
    let entries = read_index(path)?;
    fs::create_dir_all(output_dir)
        .with_context(|| format!("create output directory: {}", output_dir.display()))?;
    let mut input = BufReader::new(File::open(path)?);
    for entry in &entries {
        validate_flat_name(&entry.name)?;
        input.seek(SeekFrom::Start(u64::from(entry.offset)))?;
        let mut output = BufWriter::new(File::create(output_dir.join(&entry.name))?);
        copy_exact(&mut input, &mut output, u64::from(entry.size))?;
        output.flush()?;
    }
    let mut list = BufWriter::new(File::create(output_dir.join("__filelist.txt"))?);
    for entry in &entries {
        writeln!(list, "{}", entry.name)?;
    }
    list.flush()?;
    Ok(entries)
}

pub fn pack(input_dir: &Path, output_path: &Path) -> Result<Vec<ArcEntry>> {
    let names = read_pack_names(input_dir)?;
    let mut files = Vec::with_capacity(names.len());
    for name in names {
        validate_flat_name(&name)?;
        encode_name(&name)?;
        let path = input_dir.join(&name);
        ensure!(
            path.is_file(),
            "listed ARC input file is missing: {}",
            path.display()
        );
        let size = fs::metadata(&path)?.len();
        ensure!(
            size <= u32::MAX as u64,
            "ARC entry is too large: {}",
            path.display()
        );
        files.push((name, path, size as u32));
    }

    let start = checked_data_start(files.len())?;
    let entries = make_entries(
        files.iter().map(|(name, _, size)| (name.as_str(), *size)),
        start,
    )?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut output = BufWriter::new(File::create(output_path)?);
    write_index(&mut output, &entries)?;
    for (_, path, size) in &files {
        let mut input = BufReader::new(File::open(path)?);
        copy_exact(&mut input, &mut output, u64::from(*size))?;
    }
    output.flush()?;
    Ok(entries)
}

pub fn repack_with_replacements(
    source_arc: &Path,
    output_arc: &Path,
    replacements: &HashMap<String, Vec<u8>>,
) -> Result<Vec<ArcEntry>> {
    let source_entries = read_index(source_arc)?;
    let normalized: HashMap<String, &Vec<u8>> = replacements
        .iter()
        .map(|(name, data)| (normalize_name(name), data))
        .collect();
    let source_names: HashSet<String> = source_entries
        .iter()
        .map(|entry| normalize_name(&entry.name))
        .collect();
    for name in normalized.keys() {
        ensure!(
            source_names.contains(name),
            "replacement target is not in ARC: {name}"
        );
    }
    for (name, data) in &normalized {
        ensure!(
            data.len() <= u32::MAX as usize,
            "replacement is too large for ARC: {name} ({} bytes)",
            data.len()
        );
    }

    let start = checked_data_start(source_entries.len())?;
    let entries = make_entries(
        source_entries.iter().map(|entry| {
            let key = normalize_name(&entry.name);
            let size = normalized
                .get(&key)
                .map(|data| data.len() as u32)
                .unwrap_or(entry.size);
            (entry.name.as_str(), size)
        }),
        start,
    )?;

    if let Some(parent) = output_arc.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut source = BufReader::new(File::open(source_arc)?);
    let mut output = BufWriter::new(File::create(output_arc)?);
    write_index(&mut output, &entries)?;
    for source_entry in &source_entries {
        let key = normalize_name(&source_entry.name);
        if let Some(data) = normalized.get(&key) {
            output.write_all(data)?;
        } else {
            source.seek(SeekFrom::Start(u64::from(source_entry.offset)))?;
            copy_exact(&mut source, &mut output, u64::from(source_entry.size))?;
        }
    }
    output.flush()?;
    Ok(entries)
}

pub fn verify_repack(
    source_arc: &Path,
    rebuilt_arc: &Path,
    replacements: &HashMap<String, Vec<u8>>,
) -> Result<VerifyResult> {
    let source_entries = read_index(source_arc)?;
    let rebuilt_entries = read_index(rebuilt_arc)?;
    ensure!(
        source_entries.len() == rebuilt_entries.len(),
        "ARC entry count changed: {} -> {}",
        source_entries.len(),
        rebuilt_entries.len()
    );
    let normalized: HashMap<String, &Vec<u8>> = replacements
        .iter()
        .map(|(name, data)| (normalize_name(name), data))
        .collect();
    let mut source = BufReader::new(File::open(source_arc)?);
    let mut rebuilt = BufReader::new(File::open(rebuilt_arc)?);
    let mut replaced = 0usize;
    let mut unchanged_exact = 0usize;

    for (old, new) in source_entries.iter().zip(&rebuilt_entries) {
        ensure!(
            old.name == new.name,
            "ARC order/name changed at {}: {} -> {}",
            old.index,
            old.name,
            new.name
        );
        let key = normalize_name(&old.name);
        if let Some(expected) = normalized.get(&key) {
            ensure!(
                new.size as usize == expected.len(),
                "replacement size mismatch: {}",
                old.name
            );
            rebuilt.seek(SeekFrom::Start(u64::from(new.offset)))?;
            compare_reader_with_slice(&mut rebuilt, expected)
                .with_context(|| format!("replacement verification failed: {}", old.name))?;
            replaced += 1;
        } else {
            ensure!(
                old.size == new.size,
                "unchanged entry size changed: {}",
                old.name
            );
            source.seek(SeekFrom::Start(u64::from(old.offset)))?;
            rebuilt.seek(SeekFrom::Start(u64::from(new.offset)))?;
            compare_readers(&mut source, &mut rebuilt, u64::from(old.size))
                .with_context(|| format!("unchanged entry differs: {}", old.name))?;
            unchanged_exact += 1;
        }
    }

    Ok(VerifyResult {
        entries: source_entries.len(),
        replaced,
        unchanged_exact,
    })
}

fn make_entries<'a>(
    files: impl Iterator<Item = (&'a str, u32)>,
    start: u32,
) -> Result<Vec<ArcEntry>> {
    let mut offset = u64::from(start);
    let mut entries = Vec::new();
    for (index, (name, size)) in files.enumerate() {
        ensure!(
            offset <= u32::MAX as u64,
            "ARC offset exceeds u32 at entry {name}"
        );
        entries.push(ArcEntry {
            index,
            name: name.to_owned(),
            size,
            offset: offset as u32,
        });
        offset = offset
            .checked_add(u64::from(size))
            .context("ARC size overflow")?;
    }
    ensure!(
        offset <= u32::MAX as u64 + 1,
        "ARC output exceeds 4 GiB offset range"
    );
    Ok(entries)
}

fn write_index(output: &mut impl Write, entries: &[ArcEntry]) -> Result<()> {
    write_u32(
        output,
        u32::try_from(entries.len()).context("too many ARC entries")?,
    )?;
    for entry in entries {
        output.write_all(&encode_name(&entry.name)?)?;
        write_u32(output, encode_u32(entry.size, XOR_SIZE))?;
        write_u32(output, encode_u32(entry.offset, XOR_OFFSET))?;
    }
    Ok(())
}

fn read_pack_names(input_dir: &Path) -> Result<Vec<String>> {
    let filelist = input_dir.join("__filelist.txt");
    if filelist.is_file() {
        let text = fs::read_to_string(&filelist)
            .with_context(|| format!("read file list: {}", filelist.display()))?;
        let names: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect();
        ensure!(
            !names.is_empty(),
            "ARC file list is empty: {}",
            filelist.display()
        );
        return Ok(names);
    }

    let mut paths: Vec<PathBuf> = fs::read_dir(input_dir)?
        .filter_map(|entry| entry.ok().map(|item| item.path()))
        .filter(|path| path.is_file())
        .filter(|path| {
            !path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("__"))
        })
        .collect();
    paths.sort_by_key(|path| path.file_name().map(|name| name.to_ascii_lowercase()));
    paths
        .into_iter()
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned)
                .with_context(|| format!("non-Unicode ARC input filename: {}", path.display()))
        })
        .collect()
}

fn read_entry_data(path: &Path, entry: &ArcEntry) -> Result<Vec<u8>> {
    let mut input = File::open(path)?;
    input.seek(SeekFrom::Start(u64::from(entry.offset)))?;
    let mut data = vec![0u8; entry.size as usize];
    input.read_exact(&mut data)?;
    Ok(data)
}

fn checked_data_start(count: usize) -> Result<u32> {
    let start = 4usize
        .checked_add(
            count
                .checked_mul(ENTRY_SIZE)
                .context("ARC table overflow")?,
        )
        .context("ARC table overflow")?;
    u32::try_from(start).context("ARC table exceeds u32")
}

fn decode_name(raw: &[u8; NAME_SIZE]) -> Result<String> {
    let mut plain = [0u8; NAME_SIZE];
    for (index, byte) in raw.iter().enumerate() {
        plain[index] = *byte ^ ARC_KEY[index % ARC_KEY.len()];
    }
    let end = plain
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(NAME_SIZE);
    ensure!(plain[..end].is_ascii(), "ARC filename is not ASCII");
    ensure!(!plain[..end].is_empty(), "ARC filename is empty");
    Ok(String::from_utf8(plain[..end].to_vec())?)
}

fn encode_name(name: &str) -> Result<[u8; NAME_SIZE]> {
    ensure!(name.is_ascii(), "ARC filename must be ASCII: {name}");
    ensure!(
        name.len() <= NAME_SIZE,
        "ARC filename exceeds 24 bytes: {name}"
    );
    let mut encrypted = [0u8; NAME_SIZE];
    for index in 0..NAME_SIZE {
        let plain = name.as_bytes().get(index).copied().unwrap_or(0);
        encrypted[index] = plain ^ ARC_KEY[index % ARC_KEY.len()];
    }
    Ok(encrypted)
}

fn validate_flat_name(name: &str) -> Result<()> {
    ensure!(
        !name.contains('/') && !name.contains('\\') && name != "." && name != "..",
        "unsafe ARC filename: {name}"
    );
    Ok(())
}

fn normalize_name(name: &str) -> String {
    name.to_ascii_uppercase()
}

fn swap_bytes_in_u16_pairs(value: u32) -> u32 {
    ((value & 0x00FF_00FF) << 8) | ((value & 0xFF00_FF00) >> 8)
}

fn decode_u32(value: u32, key: u32) -> u32 {
    swap_bytes_in_u16_pairs(value ^ key)
}

fn encode_u32(value: u32, key: u32) -> u32 {
    swap_bytes_in_u16_pairs(value) ^ key
}

fn read_u32(input: &mut impl Read) -> Result<u32> {
    let mut raw = [0u8; 4];
    input.read_exact(&mut raw)?;
    Ok(u32::from_le_bytes(raw))
}

fn write_u32(output: &mut impl Write, value: u32) -> Result<()> {
    output.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn copy_exact(input: &mut impl Read, output: &mut impl Write, mut remaining: u64) -> Result<()> {
    let mut buffer = [0u8; 64 * 1024];
    while remaining != 0 {
        let amount = usize::try_from(remaining.min(buffer.len() as u64)).unwrap();
        input.read_exact(&mut buffer[..amount])?;
        output.write_all(&buffer[..amount])?;
        remaining -= amount as u64;
    }
    Ok(())
}

fn compare_readers(left: &mut impl Read, right: &mut impl Read, mut remaining: u64) -> Result<()> {
    let mut a = [0u8; 64 * 1024];
    let mut b = [0u8; 64 * 1024];
    while remaining != 0 {
        let amount = usize::try_from(remaining.min(a.len() as u64)).unwrap();
        left.read_exact(&mut a[..amount])?;
        right.read_exact(&mut b[..amount])?;
        if a[..amount] != b[..amount] {
            bail!("binary data differs");
        }
        remaining -= amount as u64;
    }
    Ok(())
}

fn compare_reader_with_slice(reader: &mut impl Read, expected: &[u8]) -> Result<()> {
    let mut offset = 0usize;
    let mut buffer = [0u8; 64 * 1024];
    while offset < expected.len() {
        let amount = (expected.len() - offset).min(buffer.len());
        reader.read_exact(&mut buffer[..amount])?;
        if buffer[..amount] != expected[offset..offset + amount] {
            bail!("binary data differs at replacement offset {offset:#x}");
        }
        offset += amount;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_swap_is_involutive_and_matches_documented_layout() {
        assert_eq!(swap_bytes_in_u16_pairs(0xAABB_CCDD), 0xBBAA_DDCC);
        assert_eq!(
            swap_bytes_in_u16_pairs(swap_bytes_in_u16_pairs(0x1234_5678)),
            0x1234_5678
        );
    }

    #[test]
    fn encrypted_u32_roundtrips() {
        for value in [0, 1, 0xE4E4, 0x1234_5678, u32::MAX] {
            assert_eq!(decode_u32(encode_u32(value, XOR_SIZE), XOR_SIZE), value);
            assert_eq!(decode_u32(encode_u32(value, XOR_OFFSET), XOR_OFFSET), value);
        }
    }
}
