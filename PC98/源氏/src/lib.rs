pub mod font;
pub mod localization;

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::HashSet, fs, path::Path};

pub type Result<T> = std::result::Result<T, String>;

fn word(b: &[u8], p: usize) -> usize {
    u16::from_le_bytes([b[p], b[p + 1]]) as usize
}
fn dword(b: &[u8], p: usize) -> usize {
    u32::from_le_bytes(b[p..p + 4].try_into().unwrap()) as usize
}
pub fn sha256_hex(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}

#[derive(Serialize)]
pub struct FileRecord {
    pub name: String,
    pub size: usize,
    pub attributes: u8,
    pub clusters: Vec<usize>,
    pub sha256: String,
    #[serde(skip)]
    data: Vec<u8>,
}

#[derive(Serialize)]
pub struct Resource {
    pub id: usize,
    pub start_block: usize,
    pub block_count: usize,
    pub reserved_byte: u8,
    pub offset: usize,
    pub size: usize,
    pub status: String,
    pub output: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Serialize)]
pub struct Archive {
    pub name: String,
    pub status: String,
    pub entries: Vec<Resource>,
}

#[derive(Serialize)]
pub struct Manifest {
    pub format: String,
    pub source_sha256: String,
    pub source_size: usize,
    pub files: Vec<FileRecord>,
    pub archives: Vec<Archive>,
    pub warnings: Vec<String>,
    pub resource_count: usize,
}

fn safe_name(raw: &[u8]) -> Result<String> {
    let a = std::str::from_utf8(&raw[..8])
        .map_err(|_| "Non-ASCII FAT filename")?
        .trim_end();
    let e = std::str::from_utf8(&raw[8..11])
        .map_err(|_| "Non-ASCII FAT extension")?
        .trim_end();
    if a.is_empty()
        || !a
            .bytes()
            .chain(e.bytes())
            .all(|c| c.is_ascii_alphanumeric() || c == b'_')
    {
        return Err("Unsafe/unsupported FAT filename".into());
    }
    Ok(if e.is_empty() {
        a.into()
    } else {
        format!("{a}.{e}")
    })
}

pub fn parse(b: &[u8]) -> Result<Manifest> {
    if b.len() < 32 {
        return Err("FDI header truncated at offset 0".into());
    }
    let header = dword(b, 8);
    let size = dword(b, 12);
    if dword(b, 0) != 0
        || dword(b, 4) != 0x90
        || header < 32
        || size != 77 * 2 * 8 * 1024
        || [dword(b, 16), dword(b, 20), dword(b, 24), dword(b, 28)] != [1024, 8, 2, 77]
        || header.checked_add(size) != Some(b.len())
    {
        return Err("Unsupported or inconsistent FDI header/geometry/length".into());
    }
    let disk = &b[header..];
    if disk.get(2..31) != Some(b"IPL loader for PC-9801 series".as_slice()) {
        return Err("Unsupported boot sector: requires Genji/Kamejima IPL profile".into());
    }
    let fat = &disk[1024..3072];
    if fat != &disk[3072..5120] || fat[..3] != [0xfe, 0xff, 0xff] {
        return Err("FAT copies disagree or invalid media marker at disk offset 0x400".into());
    }
    let mut files = Vec::new();
    let mut used = HashSet::new();
    let mut names = HashSet::new();
    for entry in disk[5120..11264].chunks_exact(32) {
        if entry[0] == 0 {
            break;
        }
        if entry[0] == 0xe5 {
            continue;
        }
        if entry[11] & 0x18 != 0 {
            return Err("Unsupported directory/volume entry in Genji root".into());
        }
        let name = safe_name(entry)?;
        if !names.insert(name.clone()) {
            return Err(format!("Duplicate FAT filename: {name}"));
        }
        let size = dword(entry, 28);
        let mut c = word(entry, 26);
        let mut data = Vec::new();
        let mut clusters = Vec::new();
        let count = size.div_ceil(1024);
        if size == 0 && c != 0 {
            return Err(format!("{name}: empty file has a cluster"));
        }
        for _ in 0..count {
            if !(2..=1222).contains(&c) || !used.insert(c) {
                return Err(format!(
                    "{name}: invalid/cyclic/cross-linked FAT cluster {c}"
                ));
            }
            let pos = 11264 + (c - 2) * 1024;
            data.extend_from_slice(&disk[pos..pos + 1024]);
            clusters.push(c);
            let v = word(fat, c * 3 / 2);
            c = if c & 1 == 1 { v >> 4 } else { v & 0xfff };
        }
        if count > 0 && c < 0xff8 {
            return Err(format!(
                "{name}: FAT chain does not terminate at declared file size"
            ));
        }
        data.truncate(size);
        files.push(FileRecord {
            name,
            size,
            attributes: entry[11],
            clusters,
            sha256: sha256_hex(&data),
            data,
        });
    }
    let mut m = Manifest {
        format: "Genji FDI / Kamejima FAT12 / indexed DAT v1".into(),
        source_sha256: sha256_hex(b),
        source_size: b.len(),
        files,
        archives: Vec::new(),
        warnings: Vec::new(),
        resource_count: 0,
    };
    for file in &m.files {
        if !file.name.ends_with(".DAT") {
            continue;
        }
        let g = &file.data;
        if g.len() < 1024 {
            return Err(format!("{}: DAT index truncated", file.name));
        }
        // These exact 1 KiB tables contain no payload in the supplied disk.
        let placeholder = g.len() == 1024
            && (0..256)
                .all(|i| word(g, i * 4) == 5 + i * 229 && g[i * 4 + 2] == 229 && g[i * 4 + 3] == 0);
        let mut archive = Archive {
            name: file.name.clone(),
            status: if placeholder {
                "index_only_payload_absent"
            } else {
                "complete"
            }
            .into(),
            entries: Vec::new(),
        };
        let mut end = 1024;
        for id in 0..256 {
            let start = word(g, id * 4);
            let count = g[id * 4 + 2] as usize;
            let reserved = g[id * 4 + 3];
            if start < 5 || reserved != 0 {
                return Err(format!(
                    "{}: unsupported index entry {id} at {:#x}",
                    file.name,
                    id * 4
                ));
            }
            let offset = (start - 1) * 256;
            let size = count * 256;
            if !placeholder && (offset != end || offset + size > g.len()) {
                return Err(format!("{}: entry {id} non-contiguous/out-of-range resource at {offset:#x}, size {size}",file.name));
            }
            let present = !placeholder && size > 0;
            let output = present.then(|| {
                format!(
                    "resources/{}/{id:04}.res",
                    file.name.trim_end_matches(".DAT")
                )
            });
            let digest = present.then(|| sha256_hex(&g[offset..offset + size]));
            if present {
                m.resource_count += 1;
            }
            archive.entries.push(Resource {
                id,
                start_block: start,
                block_count: count,
                reserved_byte: reserved,
                offset,
                size,
                status: if placeholder {
                    "payload_absent"
                } else if size == 0 {
                    "empty"
                } else {
                    "extracted"
                }
                .into(),
                output,
                sha256: digest,
            });
            end = offset + size;
        }
        if !placeholder && end != g.len() {
            return Err(format!(
                "{}: unindexed trailing bytes at {end:#x}",
                file.name
            ));
        }
        if placeholder {
            m.warnings.push(format!("{} contains an index only; its 256 referenced payloads are absent from this image. No synthetic resources were emitted.",file.name));
        }
        m.archives.push(archive);
    }
    if m.archives.is_empty() || m.resource_count == 0 {
        return Err("No complete Genji resource archive found".into());
    }
    Ok(m)
}

pub fn unpack(input: &Path, output: &Path, overwrite: bool) -> Result<Manifest> {
    let b = fs::read(input).map_err(|e| format!("{}: {e}", input.display()))?;
    let m = parse(&b)?;
    let mut prepared: Vec<(String, Vec<u8>)> = Vec::new();
    for f in &m.files {
        prepared.push((format!("files/{}", f.name), f.data.clone()));
    }
    for a in &m.archives {
        let f = m.files.iter().find(|f| f.name == a.name).unwrap();
        for r in &a.entries {
            if let Some(path) = &r.output {
                prepared.push((path.clone(), f.data[r.offset..r.offset + r.size].to_vec()));
            }
        }
    }
    let mut json = serde_json::to_vec_pretty(&m).map_err(|e| e.to_string())?;
    json.push(b'\n');
    prepared.push(("manifest.json".into(), json));
    if output.exists() && (!overwrite || !output.is_dir()) {
        return Err(format!(
            "Output exists: {} (use --overwrite for a directory)",
            output.display()
        ));
    }
    // Preflight every destination; never overwrite the source or traverse links.
    let source = fs::canonicalize(input).map_err(|e| e.to_string())?;
    for (name, _) in &prepared {
        let path = output.join(name);
        for ancestor in path.ancestors() {
            if let Ok(meta) = fs::symlink_metadata(ancestor) {
                if meta.file_type().is_symlink() {
                    return Err(format!("Symlink destination: {}", ancestor.display()));
                }
            }
        }
        if path.exists()
            && (!path.is_file() || fs::canonicalize(&path).map_err(|e| e.to_string())? == source)
        {
            return Err(format!("Unsafe destination: {}", path.display()));
        }
    }
    for (name, data) in prepared {
        let path = output.join(name);
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::write(&path, data).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(m)
}

#[derive(Debug, Clone, Serialize)]
pub struct RebuildFdiReport {
    pub output: String,
    pub source_size: usize,
    pub output_size: usize,
    pub resources: usize,
    pub changed_resources: usize,
    pub g1_size: usize,
    pub g1_clusters: usize,
}

pub(crate) fn split_dat_resources(data: &[u8]) -> Result<Vec<Vec<u8>>> {
    if data.len() < 1024 {
        return Err("G1.DAT index is truncated".into());
    }
    let mut resources = Vec::with_capacity(256);
    let mut end = 1024usize;
    for id in 0..256 {
        let start = word(data, id * 4);
        let count = data[id * 4 + 2] as usize;
        if start < 5 || data[id * 4 + 3] != 0 {
            return Err(format!("G1.DAT has an invalid index entry {id}"));
        }
        let offset = (start - 1) * 256;
        let size = count * 256;
        if offset != end || offset + size > data.len() {
            return Err(format!(
                "G1.DAT resource {id} is non-contiguous or outside the archive"
            ));
        }
        resources.push(data[offset..offset + size].to_vec());
        end = offset + size;
    }
    if end != data.len() {
        return Err(format!("G1.DAT has unindexed data at 0x{end:X}"));
    }
    Ok(resources)
}

pub(crate) fn build_dat(resources: &[Vec<u8>]) -> Result<Vec<u8>> {
    if resources.len() != 256 {
        return Err("G1.DAT rebuild requires exactly 256 resource slots".into());
    }
    let mut output = vec![0u8; 1024];
    for (id, resource) in resources.iter().enumerate() {
        if resource.len() % 256 != 0 {
            return Err(format!(
                "resource {id:04} has {} bytes; DAT resources must be 256-byte aligned",
                resource.len()
            ));
        }
        let blocks = resource.len() / 256;
        if blocks > u8::MAX as usize {
            return Err(format!(
                "resource {id:04} needs {blocks} blocks; the DAT index limit is 255"
            ));
        }
        let start = output.len() / 256 + 1;
        if start > u16::MAX as usize {
            return Err("G1.DAT start-block field overflow".into());
        }
        output[id * 4..id * 4 + 2].copy_from_slice(&(start as u16).to_le_bytes());
        output[id * 4 + 2] = blocks as u8;
        output.extend_from_slice(resource);
    }
    Ok(output)
}

fn fat12_set(fat: &mut [u8], cluster: usize, value: usize) -> Result<()> {
    if cluster > 1222 || value > 0x0FFF {
        return Err("FAT12 value is outside the Genji disk profile".into());
    }
    let offset = cluster * 3 / 2;
    let current = u16::from_le_bytes([fat[offset], fat[offset + 1]]);
    let updated = if cluster & 1 == 1 {
        (current & 0x000F) | ((value as u16) << 4)
    } else {
        (current & 0xF000) | value as u16
    };
    fat[offset..offset + 2].copy_from_slice(&updated.to_le_bytes());
    Ok(())
}

pub(crate) fn replace_fat_file(source: &[u8], filename: &str, data: &[u8]) -> Result<Vec<u8>> {
    let manifest = parse(source)?;
    let target = manifest
        .files
        .iter()
        .find(|file| file.name.eq_ignore_ascii_case(filename))
        .ok_or_else(|| format!("{filename} is not present in the FDI root directory"))?;
    let header = dword(source, 8);
    let mut output = source.to_vec();
    let disk = &source[header..];
    let fat = &disk[1024..3072];

    let mut unavailable = HashSet::new();
    for file in &manifest.files {
        if !file.name.eq_ignore_ascii_case(filename) {
            unavailable.extend(file.clusters.iter().copied());
        }
    }
    let required = data.len().div_ceil(1024);
    let mut allocation = Vec::with_capacity(required);
    for cluster in target.clusters.iter().copied().chain(2..=1222) {
        if allocation.len() == required {
            break;
        }
        if !unavailable.contains(&cluster) && !allocation.contains(&cluster) {
            allocation.push(cluster);
        }
    }
    if allocation.len() != required {
        return Err(format!(
            "rebuilt {filename} needs {required} clusters, but only {} are available",
            allocation.len()
        ));
    }

    let mut new_fat = fat.to_vec();
    for cluster in &target.clusters {
        fat12_set(&mut new_fat, *cluster, 0)?;
    }
    for (index, cluster) in allocation.iter().enumerate() {
        let next = allocation.get(index + 1).copied().unwrap_or(0x0FFF);
        fat12_set(&mut new_fat, *cluster, next)?;
    }
    output[header + 1024..header + 3072].copy_from_slice(&new_fat);
    output[header + 3072..header + 5120].copy_from_slice(&new_fat);

    for (index, cluster) in allocation.iter().enumerate() {
        let source_start = index * 1024;
        let source_end = (source_start + 1024).min(data.len());
        let disk_start = header + 11264 + (cluster - 2) * 1024;
        output[disk_start..disk_start + 1024].fill(0);
        output[disk_start..disk_start + source_end - source_start]
            .copy_from_slice(&data[source_start..source_end]);
    }

    let mut root_entry = None;
    for offset in (header + 5120..header + 11264).step_by(32) {
        let entry = &source[offset..offset + 32];
        if entry[0] == 0 {
            break;
        }
        if entry[0] != 0xE5 && safe_name(entry)?.eq_ignore_ascii_case(filename) {
            root_entry = Some(offset);
            break;
        }
    }
    let root_entry = root_entry.ok_or_else(|| format!("{filename} root entry disappeared"))?;
    let first = allocation.first().copied().unwrap_or(0);
    output[root_entry + 26..root_entry + 28].copy_from_slice(&(first as u16).to_le_bytes());
    output[root_entry + 28..root_entry + 32].copy_from_slice(&(data.len() as u32).to_le_bytes());
    Ok(output)
}

pub(crate) fn g1_data(manifest: &Manifest) -> Result<&[u8]> {
    manifest
        .files
        .iter()
        .find(|file| file.name.eq_ignore_ascii_case("G1.DAT"))
        .map(|file| file.data.as_slice())
        .ok_or_else(|| "G1.DAT is missing from the FDI".into())
}

pub(crate) fn write_file_checked(path: &Path, data: &[u8], overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        return Err(format!("Output exists: {}", path.display()));
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(path, data).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn rebuild_fdi(
    source_fdi: &Path,
    unpacked: &Path,
    output_fdi: &Path,
    overwrite: bool,
) -> Result<RebuildFdiReport> {
    if output_fdi.exists()
        && fs::canonicalize(source_fdi).map_err(|e| e.to_string())?
            == fs::canonicalize(output_fdi).map_err(|e| e.to_string())?
    {
        return Err("output FDI must be separate from the source FDI".into());
    }
    let source = fs::read(source_fdi).map_err(|e| format!("{}: {e}", source_fdi.display()))?;
    let manifest = parse(&source)?;
    let original = split_dat_resources(g1_data(&manifest)?)?;
    let resource_dir = unpacked.join("resources/G1");
    let mut resources = Vec::with_capacity(256);
    let mut changed = 0usize;
    for (id, old) in original.iter().enumerate() {
        let path = resource_dir.join(format!("{id:04}.res"));
        let value = if path.is_file() {
            fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?
        } else if old.is_empty() {
            Vec::new()
        } else {
            return Err(format!("Missing extracted resource: {}", path.display()));
        };
        changed += usize::from(value != *old);
        resources.push(value);
    }
    let rebuilt_g1 = build_dat(&resources)?;
    let rebuilt_fdi = if rebuilt_g1 == g1_data(&manifest)? {
        source.clone()
    } else {
        replace_fat_file(&source, "G1.DAT", &rebuilt_g1)?
    };
    write_file_checked(output_fdi, &rebuilt_fdi, overwrite)?;
    Ok(RebuildFdiReport {
        output: output_fdi.display().to_string(),
        source_size: source.len(),
        output_size: rebuilt_fdi.len(),
        resources: resources.iter().filter(|item| !item.is_empty()).count(),
        changed_resources: changed,
        g1_size: rebuilt_g1.len(),
        g1_clusters: rebuilt_g1.len().div_ceil(1024),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reject_short_header() {
        assert!(parse(&[0; 31]).is_err());
    }
    #[test]
    fn reject_unsafe_names() {
        assert!(safe_name(b"..      DAT").is_err());
        assert!(safe_name(b"ABC/    DAT").is_err());
    }
}
