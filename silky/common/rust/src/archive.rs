use crate::codec::{decode_text, encode_text};
use crate::lzss::{compress as lzss_compress, decompress_exact as lzss_decompress_exact};
use crate::util::{
    pretty_json_bytes, read_utf8, validate_relative_path, write_new_file, write_new_tree,
};
use anyhow::{anyhow, bail, Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const MANIFEST_NAME: &str = ".silky_arc_manifest.json";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ArchiveFormat {
    #[serde(rename = "silky-lzss")]
    SilkyLzss,
    #[serde(rename = "garbro-fixed")]
    GarbroFixed,
}

impl ArchiveFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SilkyLzss => "silky-lzss",
            Self::GarbroFixed => "garbro-fixed",
        }
    }

    pub fn parse_explicit(value: &str) -> Result<Self> {
        match value {
            "silky-lzss" => Ok(Self::SilkyLzss),
            "garbro-fixed" => Ok(Self::GarbroFixed),
            _ => bail!("unknown ARC format: {value}"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArcEntry {
    pub name: String,
    pub offset: u32,
    pub size: u32,
    pub unpacked_size: u32,
    #[serde(default)]
    pub packed: bool,
    #[serde(default)]
    pub index: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArcManifest {
    pub format: ArchiveFormat,
    #[serde(default = "default_encoding")]
    pub encoding: String,
    #[serde(default)]
    pub entries: Vec<ArcEntry>,
}

fn default_encoding() -> String {
    crate::DEFAULT_ENCODING.to_owned()
}

#[derive(Debug)]
pub struct ArchiveContents {
    pub manifest: ArcManifest,
    pub files: Vec<(PathBuf, Vec<u8>)>,
}

#[derive(Clone, Copy, Debug)]
pub struct RepackOptions<'a> {
    pub format: &'a str,
    pub encoding: &'a str,
    pub compress_new: bool,
    pub preserve_packed: bool,
    pub jobs: usize,
}

pub fn decrypt_name(data: &[u8], encoding: &str) -> Result<String> {
    let mut buffer = data.to_vec();
    let mut key = 0u8;
    for byte in buffer.iter_mut().rev() {
        key = key.wrapping_add(1);
        *byte = byte.wrapping_add(key);
    }
    decode_text(&buffer, encoding).context("failed to decode encrypted ARC file name")
}

pub fn encrypt_name(name: &str, encoding: &str) -> Result<Vec<u8>> {
    let mut raw = encode_text(name, encoding)
        .with_context(|| format!("ARC file name is not encodable: {name:?}"))?;
    if raw.len() > u8::MAX as usize {
        bail!("file name is too long for silky-lzss ARC: {name:?}");
    }
    let mut key = 0u8;
    for byte in raw.iter_mut().rev() {
        key = key.wrapping_add(1);
        *byte = byte.wrapping_sub(key);
    }
    Ok(raw)
}

pub fn parse_archive(data: &[u8], format: &str, encoding: &str) -> Result<ArcManifest> {
    match format {
        "auto" => {
            let silky = parse_silky_lzss(data, encoding);
            if let Ok(manifest) = silky {
                return Ok(manifest);
            }
            let garbro = parse_garbro_fixed(data, encoding);
            if let Ok(manifest) = garbro {
                return Ok(manifest);
            }
            bail!(
                "unsupported ARC format\n  silky-lzss: {}\n  garbro-fixed: {}",
                parse_silky_lzss(data, encoding).unwrap_err(),
                parse_garbro_fixed(data, encoding).unwrap_err()
            )
        }
        "silky-lzss" => parse_silky_lzss(data, encoding),
        "garbro-fixed" => parse_garbro_fixed(data, encoding),
        _ => bail!("unknown ARC format: {format}"),
    }
}

pub fn parse_silky_lzss(data: &[u8], encoding: &str) -> Result<ArcManifest> {
    if data.len() < 4 {
        bail!("ARC file is smaller than its header");
    }
    let header_size = read_u32_le(data, 0)? as usize;
    let header_end = 4usize
        .checked_add(header_size)
        .ok_or_else(|| anyhow!("silky-lzss header size overflow"))?;
    if header_size == 0 || header_end > data.len() {
        bail!("invalid silky-lzss header size: {header_size}");
    }

    let mut entries = Vec::new();
    let mut pos = 4usize;
    while pos < header_end {
        let name_len = *data
            .get(pos)
            .ok_or_else(|| anyhow!("truncated silky-lzss name length"))?
            as usize;
        pos += 1;
        if name_len == 0 || pos + name_len + 12 > header_end {
            bail!("invalid silky-lzss name/record length at header offset 0x{pos:08X}");
        }
        let name = decrypt_name(&data[pos..pos + name_len], encoding)?;
        let _ = archive_relative_path(&name)?;
        pos += name_len;
        let size = read_u32_be(data, pos)?;
        let unpacked_size = read_u32_be(data, pos + 4)?;
        let offset = read_u32_be(data, pos + 8)?;
        pos += 12;
        validate_placement(data.len(), header_end, offset, size, &name)?;
        entries.push(ArcEntry {
            name,
            offset,
            size,
            unpacked_size,
            packed: size != unpacked_size,
            index: entries.len(),
        });
    }
    if pos != header_end {
        bail!("silky-lzss header did not end exactly");
    }
    validate_entry_layout(&entries)?;
    Ok(ArcManifest {
        format: ArchiveFormat::SilkyLzss,
        encoding: encoding.to_owned(),
        entries,
    })
}

pub fn parse_garbro_fixed(data: &[u8], encoding: &str) -> Result<ArcManifest> {
    if data.len() < 4 {
        bail!("ARC file is smaller than its header");
    }
    let count = read_u32_le(data, 0)? as usize;
    if count == 0 || count > 200_000 {
        bail!("invalid garbro-fixed entry count: {count}");
    }
    let index_end = 4usize
        .checked_add(
            count
                .checked_mul(0x28)
                .ok_or_else(|| anyhow!("garbro-fixed index size overflow"))?,
        )
        .ok_or_else(|| anyhow!("garbro-fixed index size overflow"))?;
    if index_end > data.len() {
        bail!("garbro-fixed index exceeds file size");
    }

    let mut entries = Vec::with_capacity(count);
    let mut pos = 4usize;
    for index in 0..count {
        let raw_name = &data[pos..pos + 0x20];
        pos += 0x20;
        let name_end = raw_name.iter().position(|byte| *byte == 0).unwrap_or(0x20);
        if name_end == 0 {
            bail!("empty garbro-fixed name at entry {index}");
        }
        let name = decode_text(&raw_name[..name_end], encoding)
            .with_context(|| format!("invalid garbro-fixed name at entry {index}"))?;
        let _ = archive_relative_path(&name)?;
        let offset = read_u32_le(data, pos)?;
        let size = read_u32_le(data, pos + 4)?;
        pos += 8;
        validate_placement(data.len(), index_end, offset, size, &name)?;
        entries.push(ArcEntry {
            name,
            offset,
            size,
            unpacked_size: size,
            packed: false,
            index,
        });
    }
    validate_entry_layout(&entries)?;
    Ok(ArcManifest {
        format: ArchiveFormat::GarbroFixed,
        encoding: encoding.to_owned(),
        entries,
    })
}

pub fn decode_archive(data: &[u8], format: &str, encoding: &str) -> Result<ArchiveContents> {
    let manifest = parse_archive(data, format, encoding)?;
    let mut files = Vec::with_capacity(manifest.entries.len());
    for entry in &manifest.entries {
        let start = entry.offset as usize;
        let end = start + entry.size as usize;
        let stored = &data[start..end];
        let payload = if manifest.format == ArchiveFormat::SilkyLzss && entry.packed {
            lzss_decompress_exact(stored, entry.unpacked_size as usize)
                .with_context(|| format!("failed to decompress {}", entry.name))?
        } else {
            stored.to_vec()
        };
        if payload.len() != entry.unpacked_size as usize {
            bail!(
                "unpacked size mismatch for {}: {} != {}",
                entry.name,
                payload.len(),
                entry.unpacked_size
            );
        }
        files.push((archive_relative_path(&entry.name)?, payload));
    }
    Ok(ArchiveContents { manifest, files })
}

pub fn unpack_archive(
    archive_path: &Path,
    output_dir: &Path,
    format: &str,
    encoding: &str,
    write_manifest: bool,
) -> Result<ArcManifest> {
    let data = fs::read(archive_path)
        .with_context(|| format!("failed to read {}", archive_path.display()))?;
    let mut contents = decode_archive(&data, format, encoding)?;
    if write_manifest {
        contents.files.push((
            PathBuf::from(MANIFEST_NAME),
            pretty_json_bytes(&contents.manifest)?,
        ));
    }
    write_new_tree(output_dir, &contents.files)?;
    Ok(contents.manifest)
}

pub fn read_manifest(path: &Path) -> Result<ArcManifest> {
    let text = read_utf8(path)?;
    let manifest: ArcManifest = serde_json::from_str(&text)
        .with_context(|| format!("invalid ARC manifest: {}", path.display()))?;
    if manifest.entries.is_empty() {
        bail!("ARC manifest has no entries: {}", path.display());
    }
    let mut names = HashSet::new();
    for (index, entry) in manifest.entries.iter().enumerate() {
        if entry.index != index {
            bail!(
                "ARC manifest index mismatch at position {index}: {}",
                entry.index
            );
        }
        let _ = archive_relative_path(&entry.name)?;
        if !names.insert(entry.name.to_ascii_lowercase()) {
            bail!("duplicate ARC manifest name: {}", entry.name);
        }
    }
    Ok(manifest)
}

pub fn repack_archive(
    input_dir: &Path,
    output_arc: &Path,
    manifest_path: Option<&Path>,
    options: RepackOptions<'_>,
) -> Result<ArcManifest> {
    if !input_dir.is_dir() {
        bail!("input is not a directory: {}", input_dir.display());
    }
    let default_manifest = input_dir.join(MANIFEST_NAME);
    let manifest_path = manifest_path.or_else(|| {
        default_manifest
            .is_file()
            .then_some(default_manifest.as_path())
    });
    let manifest = manifest_path.map(read_manifest).transpose()?;

    let format = if options.format == "auto" {
        manifest
            .as_ref()
            .map_or(ArchiveFormat::SilkyLzss, |manifest| manifest.format)
    } else {
        ArchiveFormat::parse_explicit(options.format)?
    };
    if let Some(manifest) = &manifest {
        if manifest.format != format {
            bail!(
                "requested format {} conflicts with manifest format {}",
                format.as_str(),
                manifest.format.as_str()
            );
        }
    }
    let encoding = manifest
        .as_ref()
        .map(|manifest| manifest.encoding.as_str())
        .filter(|encoding| !encoding.is_empty())
        .unwrap_or(options.encoding);
    let records = ordered_input_files(input_dir, manifest.as_ref())?;
    if records.is_empty() {
        bail!("no files to repack: {}", input_dir.display());
    }

    let (bytes, rebuilt) = match format {
        ArchiveFormat::SilkyLzss => build_silky_lzss(
            records,
            encoding,
            options.compress_new,
            options.preserve_packed,
            options.jobs,
        )?,
        ArchiveFormat::GarbroFixed => build_garbro_fixed(records, encoding)?,
    };
    write_new_file(output_arc, &bytes)?;
    Ok(rebuilt)
}

type InputRecord = (String, Vec<u8>, Option<ArcEntry>);

fn ordered_input_files(
    input_dir: &Path,
    manifest: Option<&ArcManifest>,
) -> Result<Vec<InputRecord>> {
    let files = collect_tree(input_dir)?;
    let mut by_name: HashMap<String, (String, Vec<u8>)> = files
        .into_iter()
        .map(|(name, bytes)| (name.to_ascii_lowercase(), (name, bytes)))
        .collect();
    let mut result = Vec::new();

    if let Some(manifest) = manifest {
        for entry in &manifest.entries {
            let key = entry.name.to_ascii_lowercase();
            let (actual_name, bytes) = by_name
                .remove(&key)
                .ok_or_else(|| anyhow!("missing file for manifest entry: {}", entry.name))?;
            if actual_name != entry.name {
                bail!(
                    "archive path case/name changed: manifest={:?}, directory={actual_name:?}",
                    entry.name
                );
            }
            result.push((entry.name.clone(), bytes, Some(entry.clone())));
        }
    }

    let mut extras: Vec<_> = by_name.into_values().collect();
    extras.sort_by(|a, b| a.0.cmp(&b.0));
    result.extend(extras.into_iter().map(|(name, bytes)| (name, bytes, None)));
    Ok(result)
}

fn collect_tree(root: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    fn visit(root: &Path, current: &Path, output: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
        for entry in fs::read_dir(current)
            .with_context(|| format!("failed to list {}", current.display()))?
        {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                bail!(
                    "symbolic links are not allowed in repack input: {}",
                    entry.path().display()
                );
            }
            if file_type.is_dir() {
                visit(root, &entry.path(), output)?;
            } else if file_type.is_file() && entry.file_name() != MANIFEST_NAME {
                let relative = entry.path().strip_prefix(root)?.to_path_buf();
                validate_relative_path(&relative)?;
                let name = relative
                    .components()
                    .map(|component| component.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                let _ = archive_relative_path(&name)?;
                let bytes = fs::read(entry.path())?;
                output.push((name, bytes));
            }
        }
        Ok(())
    }

    let mut output = Vec::new();
    visit(root, root, &mut output)?;
    output.sort_by(|a, b| a.0.cmp(&b.0));
    let mut seen = HashSet::new();
    for (name, _) in &output {
        if !seen.insert(name.to_ascii_lowercase()) {
            bail!("case-insensitive duplicate input path: {name}");
        }
    }
    Ok(output)
}

fn build_silky_lzss(
    records: Vec<InputRecord>,
    encoding: &str,
    compress_new: bool,
    preserve_packed: bool,
    jobs: usize,
) -> Result<(Vec<u8>, ArcManifest)> {
    struct StoredRecord {
        name: String,
        encrypted_name: Vec<u8>,
        stored: Vec<u8>,
        raw_size: usize,
    }

    let workers = resolve_jobs(jobs, records.len());
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .build()
        .context("failed to create LZSS worker pool")?;
    let stored: Vec<StoredRecord> = pool.install(|| {
        records
            .into_par_iter()
            .map(|(name, raw, old)| -> Result<StoredRecord> {
                let should_pack = if preserve_packed {
                    old.as_ref().map_or(compress_new, |entry| entry.packed)
                } else {
                    compress_new
                };
                let raw_size = raw.len();
                let compressed = should_pack.then(|| lzss_compress(&raw));
                let stored = match compressed {
                    Some(compressed) if compressed.len() != raw.len() => compressed,
                    _ => raw,
                };
                Ok(StoredRecord {
                    encrypted_name: encrypt_name(&name, encoding)?,
                    name,
                    stored,
                    raw_size,
                })
            })
            .collect::<Result<Vec<_>>>()
    })?;

    let header_size: usize = stored
        .iter()
        .map(|record| 1 + record.encrypted_name.len() + 12)
        .sum();
    let mut offset = 4usize
        .checked_add(header_size)
        .ok_or_else(|| anyhow!("silky-lzss header size overflow"))?;
    let mut entries = Vec::with_capacity(stored.len());
    for (index, record) in stored.iter().enumerate() {
        let size = u32::try_from(record.stored.len()).context("ARC entry exceeds 4 GiB")?;
        let unpacked_size = u32::try_from(record.raw_size).context("ARC entry exceeds 4 GiB")?;
        entries.push(ArcEntry {
            name: record.name.clone(),
            offset: u32::try_from(offset).context("ARC offset exceeds 4 GiB")?,
            size,
            unpacked_size,
            packed: size != unpacked_size,
            index,
        });
        offset = offset
            .checked_add(record.stored.len())
            .ok_or_else(|| anyhow!("ARC size overflow"))?;
    }

    let mut output = Vec::with_capacity(offset);
    output.extend_from_slice(
        &u32::try_from(header_size)
            .context("silky-lzss header exceeds 4 GiB")?
            .to_le_bytes(),
    );
    for (entry, record) in entries.iter().zip(&stored) {
        output.push(record.encrypted_name.len() as u8);
        output.extend_from_slice(&record.encrypted_name);
        output.extend_from_slice(&entry.size.to_be_bytes());
        output.extend_from_slice(&entry.unpacked_size.to_be_bytes());
        output.extend_from_slice(&entry.offset.to_be_bytes());
    }
    for record in &stored {
        output.extend_from_slice(&record.stored);
    }
    Ok((
        output,
        ArcManifest {
            format: ArchiveFormat::SilkyLzss,
            encoding: encoding.to_owned(),
            entries,
        },
    ))
}

fn build_garbro_fixed(records: Vec<InputRecord>, encoding: &str) -> Result<(Vec<u8>, ArcManifest)> {
    let count = records.len();
    let mut offset = 4usize
        .checked_add(
            count
                .checked_mul(0x28)
                .ok_or_else(|| anyhow!("garbro-fixed index size overflow"))?,
        )
        .ok_or_else(|| anyhow!("garbro-fixed index size overflow"))?;
    let mut names = Vec::with_capacity(count);
    let mut entries = Vec::with_capacity(count);
    for (index, (name, bytes, _)) in records.iter().enumerate() {
        let name_bytes = encode_text(name, encoding)?;
        if name_bytes.len() > 0x1f {
            bail!("name is too long for garbro-fixed ARC: {name:?}");
        }
        let size = u32::try_from(bytes.len()).context("ARC entry exceeds 4 GiB")?;
        entries.push(ArcEntry {
            name: name.clone(),
            offset: u32::try_from(offset).context("ARC offset exceeds 4 GiB")?,
            size,
            unpacked_size: size,
            packed: false,
            index,
        });
        names.push(name_bytes);
        offset = offset
            .checked_add(bytes.len())
            .ok_or_else(|| anyhow!("ARC size overflow"))?;
    }

    let mut output = Vec::with_capacity(offset);
    output.extend_from_slice(&u32::try_from(count)?.to_le_bytes());
    for (entry, name) in entries.iter().zip(&names) {
        output.extend_from_slice(name);
        output.resize(output.len() + (0x20 - name.len()), 0);
        output.extend_from_slice(&entry.offset.to_le_bytes());
        output.extend_from_slice(&entry.size.to_le_bytes());
    }
    for (_, bytes, _) in &records {
        output.extend_from_slice(bytes);
    }
    Ok((
        output,
        ArcManifest {
            format: ArchiveFormat::GarbroFixed,
            encoding: encoding.to_owned(),
            entries,
        },
    ))
}

fn archive_relative_path(name: &str) -> Result<PathBuf> {
    if name.is_empty() || name.contains('\0') {
        bail!("unsafe empty/NUL archive path: {name:?}");
    }
    let normalized = name.replace('\\', "/");
    if normalized.starts_with('/') {
        bail!("absolute archive path is not allowed: {name:?}");
    }
    let mut path = PathBuf::new();
    for part in normalized.split('/') {
        if part.is_empty() || matches!(part, "." | "..") || part.contains(':') {
            bail!("unsafe archive path: {name:?}");
        }
        path.push(part);
    }
    validate_relative_path(&path)?;
    Ok(path)
}

fn validate_placement(
    file_len: usize,
    index_end: usize,
    offset: u32,
    size: u32,
    name: &str,
) -> Result<()> {
    let start = offset as usize;
    let end = start
        .checked_add(size as usize)
        .ok_or_else(|| anyhow!("ARC placement overflow for {name:?}"))?;
    if start < index_end || end > file_len {
        bail!("invalid ARC placement for {name:?}: offset={offset}, size={size}, file={file_len}");
    }
    Ok(())
}

fn validate_entry_layout(entries: &[ArcEntry]) -> Result<()> {
    let mut names = HashSet::new();
    let mut offsets = HashSet::new();
    for entry in entries {
        if !names.insert(entry.name.to_ascii_lowercase()) {
            bail!("case-insensitive duplicate ARC name: {}", entry.name);
        }
        if !offsets.insert(entry.offset) {
            bail!("duplicate ARC entry offset: 0x{:08X}", entry.offset);
        }
    }
    let mut ranges: Vec<_> = entries
        .iter()
        .map(|entry| {
            (
                entry.offset as u64,
                entry.offset as u64 + entry.size as u64,
                entry.name.as_str(),
            )
        })
        .collect();
    ranges.sort_by_key(|range| range.0);
    for pair in ranges.windows(2) {
        if pair[0].1 > pair[1].0 {
            bail!(
                "overlapping ARC entries: {:?} and {:?}",
                pair[0].2,
                pair[1].2
            );
        }
    }
    Ok(())
}

fn resolve_jobs(requested: usize, tasks: usize) -> usize {
    let available = std::thread::available_parallelism().map_or(1, usize::from);
    let requested = if requested == 0 { available } else { requested };
    requested.clamp(1, tasks.max(1))
}

fn read_u32_le(data: &[u8], pos: usize) -> Result<u32> {
    let bytes = data
        .get(pos..pos + 4)
        .ok_or_else(|| anyhow!("truncated little-endian u32 at 0x{pos:08X}"))?;
    Ok(u32::from_le_bytes(
        bytes.try_into().expect("slice length checked"),
    ))
}

fn read_u32_be(data: &[u8], pos: usize) -> Result<u32> {
    let bytes = data
        .get(pos..pos + 4)
        .ok_or_else(|| anyhow!("truncated big-endian u32 at 0x{pos:08X}"))?;
    Ok(u32::from_be_bytes(
        bytes.try_into().expect("slice length checked"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypted_names_roundtrip() {
        let name = "サブ/TEST.MES";
        let encrypted = encrypt_name(name, "cp932").unwrap();
        assert_eq!(decrypt_name(&encrypted, "cp932").unwrap(), name);
    }

    #[test]
    fn rejects_path_traversal_and_windows_prefixes() {
        for name in ["../evil", "a/../evil", "/root", "C:/evil", "a//b"] {
            assert!(archive_relative_path(name).is_err(), "accepted {name}");
        }
    }

    #[test]
    fn both_archive_formats_roundtrip_contents() {
        let records = vec![
            ("A.MES".to_owned(), b"abcabcabcabc".to_vec(), None),
            ("sub/B.bin".to_owned(), (0..=255).collect(), None),
        ];
        let (silky, silky_manifest) =
            build_silky_lzss(records.clone(), "cp932", true, true, 2).unwrap();
        let decoded = decode_archive(&silky, "auto", "cp932").unwrap();
        assert_eq!(decoded.manifest, silky_manifest);
        assert_eq!(decoded.files[0].1, b"abcabcabcabc");
        assert_eq!(decoded.files[1].1, (0..=255).collect::<Vec<u8>>());

        let (garbro, garbro_manifest) = build_garbro_fixed(records, "cp932").unwrap();
        let decoded = decode_archive(&garbro, "auto", "cp932").unwrap();
        assert_eq!(decoded.manifest, garbro_manifest);
        assert_eq!(decoded.files[0].1, b"abcabcabcabc");
        assert_eq!(decoded.files[1].1, (0..=255).collect::<Vec<u8>>());
    }

    #[test]
    fn rejects_overlapping_entries() {
        let entries = vec![
            ArcEntry {
                name: "a".to_owned(),
                offset: 10,
                size: 10,
                unpacked_size: 10,
                packed: false,
                index: 0,
            },
            ArcEntry {
                name: "b".to_owned(),
                offset: 15,
                size: 2,
                unpacked_size: 2,
                packed: false,
                index: 1,
            },
        ];
        assert!(validate_entry_layout(&entries).is_err());
    }
}
