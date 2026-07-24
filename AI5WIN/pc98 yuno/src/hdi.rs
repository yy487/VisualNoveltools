use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const MANIFEST_FILENAME: &str = ".hdi_manifest.json";
const MANIFEST_FORMAT: &str = "yuno-pc98-anex86-fat16-v1";
const HDI_HEADER_MIN_SIZE: usize = 0x20;
const PARTITION_ENTRY_SIZE: usize = 0x20;
const DIRECTORY_ENTRY_SIZE: usize = 0x20;
const FAT16_MIN_CLUSTERS: usize = 4_085;
const FAT16_MAX_CLUSTERS: usize = 65_525;
const FAT16_BAD: u16 = 0xFFF7;
const FAT16_EOC_MIN: u16 = 0xFFF8;
const FAT16_EOC: u16 = 0xFFFF;

#[derive(Debug, Error)]
pub enum HdiError {
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
    #[error("invalid Anex86 HDI/FAT16 image: {0}")]
    Invalid(String),
    #[error("output already exists: {0}")]
    OutputExists(PathBuf),
}

pub type Result<T> = std::result::Result<T, HdiError>;

fn io_error(path: &Path, source: io::Error) -> HdiError {
    HdiError::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn json_error(path: &Path, source: serde_json::Error) -> HdiError {
    HdiError::Json {
        path: path.to_path_buf(),
        source,
    }
}

fn checked_range<'a>(
    data: &'a [u8],
    offset: usize,
    length: usize,
    context: &str,
) -> Result<&'a [u8]> {
    let end = offset
        .checked_add(length)
        .ok_or_else(|| HdiError::Invalid(format!("{context} range overflows usize")))?;
    data.get(offset..end).ok_or_else(|| {
        HdiError::Invalid(format!(
            "{context} range 0x{offset:X}..0x{end:X} is outside image length 0x{:X}",
            data.len()
        ))
    })
}

fn checked_range_mut<'a>(
    data: &'a mut [u8],
    offset: usize,
    length: usize,
    context: &str,
) -> Result<&'a mut [u8]> {
    let image_length = data.len();
    let end = offset
        .checked_add(length)
        .ok_or_else(|| HdiError::Invalid(format!("{context} range overflows usize")))?;
    data.get_mut(offset..end).ok_or_else(|| {
        HdiError::Invalid(format!(
            "{context} range 0x{offset:X}..0x{end:X} is outside image length 0x{image_length:X}"
        ))
    })
}

fn u16_at(data: &[u8], offset: usize, context: &str) -> Result<u16> {
    let bytes = checked_range(data, offset, 2, context)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn u32_at(data: &[u8], offset: usize, context: &str) -> Result<u32> {
    let bytes = checked_range(data, offset, 4, context)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn put_u16(data: &mut [u8], offset: usize, value: u16, context: &str) -> Result<()> {
    checked_range_mut(data, offset, 2, context)?.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn put_u32(data: &mut [u8], offset: usize, value: u32, context: &str) -> Result<()> {
    checked_range_mut(data, offset, 4, context)?.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn usize_from_u32(value: u32, context: &str) -> Result<usize> {
    usize::try_from(value)
        .map_err(|_| HdiError::Invalid(format!("{context} does not fit this platform: {value}")))
}

fn checked_product(values: &[usize], context: &str) -> Result<usize> {
    values.iter().try_fold(1usize, |product, value| {
        product
            .checked_mul(*value)
            .ok_or_else(|| HdiError::Invalid(format!("{context} overflows usize")))
    })
}

fn sha256_hex(data: &[u8]) -> String {
    hex::encode_upper(Sha256::digest(data))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HdiLayout {
    pub header_size: usize,
    pub disk_size: usize,
    pub physical_sector_size: usize,
    pub sectors_per_track: usize,
    pub heads: usize,
    pub cylinders: usize,
    pub partition_table_offset: usize,
    pub partition_entry_offset: usize,
    pub partition_offset: usize,
    pub bytes_per_sector: usize,
    pub sectors_per_cluster: usize,
    pub reserved_sectors: usize,
    pub fat_count: usize,
    pub fat_mismatch_entries: usize,
    pub sectors_per_fat: usize,
    pub root_entries: usize,
    pub total_sectors: usize,
    pub hidden_sectors: usize,
    pub fat_offset: usize,
    pub root_offset: usize,
    pub root_sectors: usize,
    pub data_offset: usize,
    pub cluster_size: usize,
    pub cluster_count: usize,
}

impl HdiLayout {
    fn max_cluster(&self) -> u16 {
        u16::try_from(self.cluster_count + 1).expect("FAT16 cluster count fits u16")
    }

    fn fat_bytes(&self) -> usize {
        self.sectors_per_fat * self.bytes_per_sector
    }

    fn cluster_offset(&self, cluster: u16) -> Result<usize> {
        if !(2..=self.max_cluster()).contains(&cluster) {
            return Err(HdiError::Invalid(format!(
                "cluster {cluster} is outside 2..={}",
                self.max_cluster()
            )));
        }
        self.data_offset
            .checked_add((usize::from(cluster) - 2) * self.cluster_size)
            .ok_or_else(|| HdiError::Invalid("cluster offset overflows usize".to_owned()))
    }
}

fn parse_layout(data: &[u8]) -> Result<HdiLayout> {
    if data.len() < HDI_HEADER_MIN_SIZE {
        return Err(HdiError::Invalid(format!(
            "file is shorter than the HDI header: {} bytes",
            data.len()
        )));
    }
    let header_size = usize_from_u32(u32_at(data, 0x08, "HDI header size")?, "header size")?;
    let disk_size = usize_from_u32(u32_at(data, 0x0C, "HDI disk size")?, "disk size")?;
    let physical_sector_size = usize_from_u32(
        u32_at(data, 0x10, "HDI physical sector size")?,
        "physical sector size",
    )?;
    let sectors_per_track = usize_from_u32(
        u32_at(data, 0x14, "HDI sectors per track")?,
        "sectors per track",
    )?;
    let heads = usize_from_u32(u32_at(data, 0x18, "HDI heads")?, "heads")?;
    let cylinders = usize_from_u32(u32_at(data, 0x1C, "HDI cylinders")?, "cylinders")?;
    if header_size < HDI_HEADER_MIN_SIZE
        || physical_sector_size == 0
        || sectors_per_track == 0
        || heads == 0
        || cylinders == 0
    {
        return Err(HdiError::Invalid("zero or invalid HDI geometry".to_owned()));
    }
    let geometry_size = checked_product(
        &[physical_sector_size, sectors_per_track, heads, cylinders],
        "HDI geometry",
    )?;
    if geometry_size != disk_size {
        return Err(HdiError::Invalid(format!(
            "HDI geometry is {geometry_size} bytes but header disk size is {disk_size}"
        )));
    }
    if header_size.checked_add(disk_size) != Some(data.len()) {
        return Err(HdiError::Invalid(format!(
            "HDI header + disk size is {}, file length is {}",
            header_size.saturating_add(disk_size),
            data.len()
        )));
    }

    let partition_table_offset = header_size
        .checked_add(physical_sector_size)
        .ok_or_else(|| HdiError::Invalid("partition table offset overflow".to_owned()))?;
    checked_range(
        data,
        partition_table_offset,
        physical_sector_size,
        "PC-98 partition table",
    )?;
    let mut selected = None;
    for index in 0..physical_sector_size / PARTITION_ENTRY_SIZE {
        let entry = partition_table_offset + index * PARTITION_ENTRY_SIZE;
        if data[entry] == 0 && data[entry + 1] == 0 {
            continue;
        }
        let start_sector = usize::from(data[entry + 8]);
        let start_head = usize::from(data[entry + 9]);
        let start_cylinder = usize::from(u16_at(data, entry + 10, "partition start cylinder")?);
        if start_sector >= sectors_per_track || start_head >= heads || start_cylinder >= cylinders {
            continue;
        }
        let lba = start_cylinder
            .checked_mul(heads)
            .and_then(|value| value.checked_add(start_head))
            .and_then(|value| value.checked_mul(sectors_per_track))
            .and_then(|value| value.checked_add(start_sector))
            .ok_or_else(|| HdiError::Invalid("partition CHS overflows usize".to_owned()))?;
        let candidate = header_size
            .checked_add(lba * physical_sector_size)
            .ok_or_else(|| HdiError::Invalid("partition offset overflow".to_owned()))?;
        if checked_range(data, candidate, 64, "partition boot sector").is_ok()
            && matches!(
                u16_at(data, candidate + 11, "BPB bytes per sector"),
                Ok(128..=4096)
            )
        {
            selected = Some((entry, candidate));
            break;
        }
    }
    let (partition_entry_offset, partition_offset) = selected
        .ok_or_else(|| HdiError::Invalid("no usable PC-98 FAT partition entry found".to_owned()))?;

    let bytes_per_sector =
        usize::from(u16_at(data, partition_offset + 11, "BPB bytes per sector")?);
    let sectors_per_cluster = usize::from(data[partition_offset + 13]);
    let reserved_sectors =
        usize::from(u16_at(data, partition_offset + 14, "BPB reserved sectors")?);
    let fat_count = usize::from(data[partition_offset + 16]);
    let root_entries = usize::from(u16_at(data, partition_offset + 17, "BPB root entries")?);
    let total_sectors_16 = usize::from(u16_at(data, partition_offset + 19, "BPB total sectors")?);
    let sectors_per_fat = usize::from(u16_at(data, partition_offset + 22, "BPB sectors per FAT")?);
    let hidden_sectors = usize_from_u32(
        u32_at(data, partition_offset + 28, "BPB hidden sectors")?,
        "hidden sectors",
    )?;
    let total_sectors = if total_sectors_16 != 0 {
        total_sectors_16
    } else {
        usize_from_u32(
            u32_at(data, partition_offset + 32, "BPB large total sectors")?,
            "large total sectors",
        )?
    };
    if !bytes_per_sector.is_power_of_two()
        || !(128..=4096).contains(&bytes_per_sector)
        || !sectors_per_cluster.is_power_of_two()
        || reserved_sectors == 0
        || fat_count == 0
        || sectors_per_fat == 0
        || root_entries == 0
        || total_sectors == 0
    {
        return Err(HdiError::Invalid("invalid FAT BPB values".to_owned()));
    }
    let root_bytes = root_entries
        .checked_mul(DIRECTORY_ENTRY_SIZE)
        .ok_or_else(|| HdiError::Invalid("root directory size overflow".to_owned()))?;
    let root_sectors = root_bytes.div_ceil(bytes_per_sector);
    let metadata_sectors = reserved_sectors
        .checked_add(fat_count * sectors_per_fat)
        .and_then(|value| value.checked_add(root_sectors))
        .ok_or_else(|| HdiError::Invalid("FAT metadata size overflow".to_owned()))?;
    if metadata_sectors >= total_sectors {
        return Err(HdiError::Invalid(
            "FAT metadata consumes the whole partition".to_owned(),
        ));
    }
    let data_sectors = total_sectors - metadata_sectors;
    let cluster_count = data_sectors / sectors_per_cluster;
    if !(FAT16_MIN_CLUSTERS..FAT16_MAX_CLUSTERS).contains(&cluster_count) {
        return Err(HdiError::Invalid(format!(
            "only FAT16 is supported, cluster count is {cluster_count}"
        )));
    }
    let partition_bytes = total_sectors
        .checked_mul(bytes_per_sector)
        .ok_or_else(|| HdiError::Invalid("partition size overflow".to_owned()))?;
    checked_range(data, partition_offset, partition_bytes, "FAT16 partition")?;
    let fat_offset = partition_offset + reserved_sectors * bytes_per_sector;
    let fat_bytes = sectors_per_fat * bytes_per_sector;
    let root_offset = fat_offset + fat_count * fat_bytes;
    let data_offset = root_offset + root_sectors * bytes_per_sector;
    let cluster_size = sectors_per_cluster * bytes_per_sector;
    let required_fat_bytes = (cluster_count + 2) * 2;
    if required_fat_bytes > fat_bytes {
        return Err(HdiError::Invalid(format!(
            "FAT has {fat_bytes} bytes but {required_fat_bytes} are required"
        )));
    }
    let first_fat = checked_range(data, fat_offset, fat_bytes, "first FAT")?;
    let mut fat_mismatch_entries = 0usize;
    for copy in 1..fat_count {
        let other = checked_range(data, fat_offset + copy * fat_bytes, fat_bytes, "FAT copy")?;
        fat_mismatch_entries += first_fat
            .chunks_exact(2)
            .zip(other.chunks_exact(2))
            .filter(|(left, right)| left != right)
            .count();
    }
    if fat16_get(first_fat, 0)? < 0xFFF0 || fat16_get(first_fat, 1)? < FAT16_EOC_MIN {
        return Err(HdiError::Invalid(
            "FAT16 reserved entries 0 and 1 are invalid".to_owned(),
        ));
    }

    Ok(HdiLayout {
        header_size,
        disk_size,
        physical_sector_size,
        sectors_per_track,
        heads,
        cylinders,
        partition_table_offset,
        partition_entry_offset,
        partition_offset,
        bytes_per_sector,
        sectors_per_cluster,
        reserved_sectors,
        fat_count,
        fat_mismatch_entries,
        sectors_per_fat,
        root_entries,
        total_sectors,
        hidden_sectors,
        fat_offset,
        root_offset,
        root_sectors,
        data_offset,
        cluster_size,
        cluster_count,
    })
}

fn fat16_get(fat: &[u8], cluster: u16) -> Result<u16> {
    let offset = usize::from(cluster) * 2;
    let bytes = checked_range(fat, offset, 2, "FAT16 entry")?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn fat16_set(fat: &mut [u8], cluster: u16, value: u16) -> Result<()> {
    let offset = usize::from(cluster) * 2;
    checked_range_mut(fat, offset, 2, "FAT16 entry")?.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn cluster_chain(layout: &HdiLayout, fat: &[u8], first: u16) -> Result<Vec<u16>> {
    if first == 0 {
        return Ok(Vec::new());
    }
    if !(2..=layout.max_cluster()).contains(&first) {
        return Err(HdiError::Invalid(format!("invalid first cluster {first}")));
    }
    let mut chain = Vec::new();
    let mut seen = HashSet::new();
    let mut cluster = first;
    loop {
        if !seen.insert(cluster) {
            return Err(HdiError::Invalid(format!("FAT cycle at cluster {cluster}")));
        }
        chain.push(cluster);
        let next = fat16_get(fat, cluster)?;
        if next >= FAT16_EOC_MIN {
            break;
        }
        if next < 2 || next == FAT16_BAD || next > layout.max_cluster() {
            return Err(HdiError::Invalid(format!(
                "invalid FAT chain value 0x{next:04X} after cluster {cluster}"
            )));
        }
        cluster = next;
    }
    Ok(chain)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DirectoryEntry {
    offset: usize,
    name_raw: [u8; 11],
    attributes: u8,
    first_cluster: u16,
    size: u32,
}

impl DirectoryEntry {
    fn is_directory(&self) -> bool {
        self.attributes & 0x10 != 0
    }

    fn is_volume_label(&self) -> bool {
        self.attributes & 0x08 != 0
    }

    fn is_dot_entry(&self) -> bool {
        self.name_raw[0] == b'.'
    }
}

fn parse_directory_entry(data: &[u8], offset: usize) -> Result<Option<DirectoryEntry>> {
    let entry = checked_range(data, offset, DIRECTORY_ENTRY_SIZE, "directory entry")?;
    if entry[0] == 0 || entry[0] == 0xE5 || entry[11] == 0x0F {
        return Ok(None);
    }
    let mut name_raw = [0u8; 11];
    name_raw.copy_from_slice(&entry[..11]);
    Ok(Some(DirectoryEntry {
        offset,
        name_raw,
        attributes: entry[11],
        first_cluster: u16::from_le_bytes([entry[26], entry[27]]),
        size: u32::from_le_bytes([entry[28], entry[29], entry[30], entry[31]]),
    }))
}

fn directory_entries(
    data: &[u8],
    layout: &HdiLayout,
    fat: &[u8],
    first_cluster: Option<u16>,
) -> Result<Vec<DirectoryEntry>> {
    let mut offsets = Vec::new();
    if let Some(cluster) = first_cluster {
        for cluster in cluster_chain(layout, fat, cluster)? {
            let start = layout.cluster_offset(cluster)?;
            offsets.extend(
                (0..layout.cluster_size)
                    .step_by(DIRECTORY_ENTRY_SIZE)
                    .map(|delta| start + delta),
            );
        }
    } else {
        offsets.extend(
            (0..layout.root_entries).map(|index| layout.root_offset + index * DIRECTORY_ENTRY_SIZE),
        );
    }
    let mut entries = Vec::new();
    for offset in offsets {
        if data[offset] == 0 {
            break;
        }
        if let Some(entry) = parse_directory_entry(data, offset)? {
            entries.push(entry);
        }
    }
    Ok(entries)
}

fn decode_short_name(raw: &[u8; 11]) -> Option<String> {
    let mut normalized = *raw;
    if normalized[0] == 0x05 {
        normalized[0] = 0xE5;
    }
    let base_end = normalized[..8]
        .iter()
        .rposition(|byte| *byte != b' ')
        .map_or(0, |index| index + 1);
    let extension_end = normalized[8..]
        .iter()
        .rposition(|byte| *byte != b' ')
        .map_or(0, |index| index + 1);
    let base =
        SHIFT_JIS.decode_without_bom_handling_and_without_replacement(&normalized[..base_end])?;
    let extension = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(&normalized[8..8 + extension_end])?;
    if extension.is_empty() {
        Some(base.into_owned())
    } else {
        Some(format!("{base}.{extension}"))
    }
}

fn is_safe_windows_component(name: &str) -> bool {
    if name.is_empty() || name == "." || name == ".." || name.ends_with([' ', '.']) {
        return false;
    }
    if name
        .chars()
        .any(|character| character <= '\u{1F}' || r#"<>:"/\|?*"#.contains(character))
    {
        return false;
    }
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        && !(stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

fn host_component(raw: &[u8; 11], used: &mut HashSet<String>) -> String {
    let decoded = decode_short_name(raw);
    let mut candidate = decoded
        .filter(|name| is_safe_windows_component(name))
        .unwrap_or_else(|| format!("__raw_{}", hex::encode_upper(raw)));
    if candidate.eq_ignore_ascii_case(MANIFEST_FILENAME)
        || !used.insert(candidate.to_ascii_uppercase())
    {
        candidate = format!("__raw_{}", hex::encode_upper(raw));
        let base = candidate.clone();
        let mut suffix = 1usize;
        while !used.insert(candidate.to_ascii_uppercase()) {
            candidate = format!("{base}_{suffix}");
            suffix += 1;
        }
    }
    candidate
}

fn join_host_path(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        child.to_owned()
    } else {
        format!("{parent}/{child}")
    }
}

fn path_from_host(root: &Path, host_path: &str) -> PathBuf {
    host_path
        .split('/')
        .filter(|component| !component.is_empty())
        .fold(root.to_path_buf(), |path, component| path.join(component))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum EntryKind {
    File,
    Directory,
}

#[derive(Debug, Clone)]
struct TreeEntry {
    host_path: String,
    kind: EntryKind,
    directory: DirectoryEntry,
    chain: Vec<u16>,
}

#[derive(Debug)]
struct TreeScan {
    entries: Vec<TreeEntry>,
    orphan_clusters: usize,
}

fn scan_tree(data: &[u8], layout: &HdiLayout, fat: &[u8]) -> Result<TreeScan> {
    fn claim_chain(chain: &[u16], path: &str, owners: &mut HashMap<u16, String>) -> Result<()> {
        for cluster in chain {
            if let Some(previous) = owners.insert(*cluster, path.to_owned()) {
                return Err(HdiError::Invalid(format!(
                    "cluster {cluster} is cross-linked by {previous} and {path}"
                )));
            }
        }
        Ok(())
    }

    fn visit(
        data: &[u8],
        layout: &HdiLayout,
        fat: &[u8],
        directory_cluster: Option<u16>,
        host_parent: &str,
        owners: &mut HashMap<u16, String>,
        output: &mut Vec<TreeEntry>,
    ) -> Result<()> {
        let entries = directory_entries(data, layout, fat, directory_cluster)?;
        let mut raw_names = HashSet::new();
        let mut host_names = HashSet::new();
        for entry in entries {
            if entry.is_dot_entry() || entry.is_volume_label() {
                continue;
            }
            if !raw_names.insert(entry.name_raw) {
                return Err(HdiError::Invalid(format!(
                    "duplicate short-name entry {} in {host_parent}",
                    hex::encode_upper(entry.name_raw)
                )));
            }
            let component = host_component(&entry.name_raw, &mut host_names);
            let host_path = join_host_path(host_parent, &component);
            let chain = cluster_chain(layout, fat, entry.first_cluster)?;
            if entry.is_directory() {
                if entry.first_cluster == 0 {
                    return Err(HdiError::Invalid(format!(
                        "directory {host_path} has no first cluster"
                    )));
                }
                claim_chain(&chain, &host_path, owners)?;
                output.push(TreeEntry {
                    host_path: host_path.clone(),
                    kind: EntryKind::Directory,
                    directory: entry.clone(),
                    chain,
                });
                visit(
                    data,
                    layout,
                    fat,
                    Some(entry.first_cluster),
                    &host_path,
                    owners,
                    output,
                )?;
            } else {
                let size = usize_from_u32(entry.size, "directory file size")?;
                if size > 0 && entry.first_cluster == 0 {
                    return Err(HdiError::Invalid(format!(
                        "file {host_path} has size {size} but no first cluster"
                    )));
                }
                if chain.len().saturating_mul(layout.cluster_size) < size {
                    return Err(HdiError::Invalid(format!(
                        "file {host_path} has a {}-byte chain for size {size}",
                        chain.len() * layout.cluster_size
                    )));
                }
                claim_chain(&chain, &host_path, owners)?;
                output.push(TreeEntry {
                    host_path,
                    kind: EntryKind::File,
                    directory: entry,
                    chain,
                });
            }
        }
        Ok(())
    }

    let mut owners = HashMap::new();
    let mut entries = Vec::new();
    visit(data, layout, fat, None, "", &mut owners, &mut entries)?;
    let orphan_clusters = (2..=layout.max_cluster())
        .filter(|cluster| {
            fat16_get(fat, *cluster).is_ok_and(|value| value != 0 && value != FAT16_BAD)
                && !owners.contains_key(cluster)
        })
        .count();
    Ok(TreeScan {
        entries,
        orphan_clusters,
    })
}

fn file_bytes(data: &[u8], layout: &HdiLayout, entry: &TreeEntry) -> Result<Vec<u8>> {
    if entry.kind != EntryKind::File {
        return Err(HdiError::Invalid(format!(
            "not a regular file: {}",
            entry.host_path
        )));
    }
    let size = usize_from_u32(entry.directory.size, "directory file size")?;
    let mut output = Vec::with_capacity(entry.chain.len() * layout.cluster_size);
    for cluster in &entry.chain {
        let offset = layout.cluster_offset(*cluster)?;
        output.extend_from_slice(checked_range(
            data,
            offset,
            layout.cluster_size,
            "file cluster",
        )?);
    }
    output.truncate(size);
    Ok(output)
}

#[derive(Debug, Clone)]
pub struct HdiImage {
    bytes: Vec<u8>,
    layout: HdiLayout,
}

impl HdiImage {
    pub fn parse(bytes: Vec<u8>) -> Result<Self> {
        let layout = parse_layout(&bytes)?;
        let fat = checked_range(&bytes, layout.fat_offset, layout.fat_bytes(), "primary FAT")?;
        scan_tree(&bytes, &layout, fat)?;
        Ok(Self { bytes, layout })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn layout(&self) -> &HdiLayout {
        &self.layout
    }

    fn primary_fat(&self) -> Result<Vec<u8>> {
        Ok(checked_range(
            &self.bytes,
            self.layout.fat_offset,
            self.layout.fat_bytes(),
            "primary FAT",
        )?
        .to_vec())
    }

    fn scan(&self) -> Result<TreeScan> {
        let fat = checked_range(
            &self.bytes,
            self.layout.fat_offset,
            self.layout.fat_bytes(),
            "primary FAT",
        )?;
        scan_tree(&self.bytes, &self.layout, fat)
    }

    fn replace_contents(&mut self, replacements: &[(TreeEntry, Vec<u8>)]) -> Result<PackStats> {
        struct Job {
            entry: TreeEntry,
            data: Vec<u8>,
            needed_clusters: usize,
            new_chain: Vec<u16>,
            extra_clusters: Vec<u16>,
        }

        let original_bytes = self.bytes.clone();
        let mut fat = self.primary_fat()?;
        let mut touched_fat_clusters = HashSet::new();
        let mut jobs = Vec::new();
        let mut stats = PackStats {
            source_files: replacements.len(),
            ..PackStats::default()
        };
        for (entry, data) in replacements {
            let original = file_bytes(&self.bytes, &self.layout, entry)?;
            if original == *data {
                stats.unchanged_files += 1;
                continue;
            }
            if u32::try_from(data.len()).is_err() {
                return Err(HdiError::Invalid(format!(
                    "replacement is larger than 4 GiB: {}",
                    entry.host_path
                )));
            }
            jobs.push(Job {
                entry: entry.clone(),
                data: data.clone(),
                needed_clusters: data.len().div_ceil(self.layout.cluster_size),
                new_chain: Vec::new(),
                extra_clusters: Vec::new(),
            });
        }
        stats.changed_files = jobs.len();

        let free_before = (2..=self.layout.max_cluster())
            .filter(|cluster| fat16_get(&fat, *cluster).is_ok_and(|value| value == 0))
            .count();
        let old_changed_clusters: usize = jobs.iter().map(|job| job.entry.chain.len()).sum();
        let needed_changed_clusters: usize = jobs.iter().map(|job| job.needed_clusters).sum();
        if needed_changed_clusters > old_changed_clusters + free_before {
            return Err(HdiError::Invalid(format!(
                "not enough FAT16 space: changed files need {needed_changed_clusters} clusters, have {old_changed_clusters} old + {free_before} free"
            )));
        }

        for job in &mut jobs {
            let keep = job.entry.chain.len().min(job.needed_clusters);
            job.new_chain.extend_from_slice(&job.entry.chain[..keep]);
            for cluster in &job.entry.chain[keep..] {
                fat16_set(&mut fat, *cluster, 0)?;
                touched_fat_clusters.insert(*cluster);
                stats.freed_clusters += 1;
            }
        }
        let free_clusters: Vec<_> = (2..=self.layout.max_cluster())
            .filter(|cluster| fat16_get(&fat, *cluster).is_ok_and(|value| value == 0))
            .collect();
        let mut free_cursor = 0usize;
        for job in &mut jobs {
            let extra = job.needed_clusters - job.new_chain.len();
            let end = free_cursor + extra;
            if end > free_clusters.len() {
                return Err(HdiError::Invalid(
                    "free-cluster planning became inconsistent".to_owned(),
                ));
            }
            job.extra_clusters
                .extend_from_slice(&free_clusters[free_cursor..end]);
            job.new_chain
                .extend_from_slice(&free_clusters[free_cursor..end]);
            stats.allocated_clusters += extra;
            free_cursor = end;
        }

        for job in &jobs {
            for (index, cluster) in job.new_chain.iter().enumerate() {
                let next = job.new_chain.get(index + 1).copied().unwrap_or(FAT16_EOC);
                fat16_set(&mut fat, *cluster, next)?;
                touched_fat_clusters.insert(*cluster);
            }
            let first_cluster = job.new_chain.first().copied().unwrap_or(0);
            put_u16(
                &mut self.bytes,
                job.entry.directory.offset + 26,
                first_cluster,
                "directory first cluster",
            )?;
            put_u32(
                &mut self.bytes,
                job.entry.directory.offset + 28,
                u32::try_from(job.data.len()).expect("replacement size checked"),
                "directory file size",
            )?;

            let extra: HashSet<_> = job.extra_clusters.iter().copied().collect();
            for (index, cluster) in job.new_chain.iter().enumerate() {
                let cluster_offset = self.layout.cluster_offset(*cluster)?;
                let target = checked_range_mut(
                    &mut self.bytes,
                    cluster_offset,
                    self.layout.cluster_size,
                    "replacement cluster",
                )?;
                if extra.contains(cluster) {
                    target.fill(0);
                }
                let source_start = index * self.layout.cluster_size;
                let source_end = (source_start + self.layout.cluster_size).min(job.data.len());
                if source_start < source_end {
                    target[..source_end - source_start]
                        .copy_from_slice(&job.data[source_start..source_end]);
                }
            }
        }

        if !jobs.is_empty() {
            for copy in 0..self.layout.fat_count {
                let offset = self.layout.fat_offset + copy * self.layout.fat_bytes();
                let output_fat = checked_range_mut(
                    &mut self.bytes,
                    offset,
                    self.layout.fat_bytes(),
                    "FAT output copy",
                )?;
                for cluster in &touched_fat_clusters {
                    fat16_set(output_fat, *cluster, fat16_get(&fat, *cluster)?)?;
                }
            }
        }
        stats.free_clusters_after = (2..=self.layout.max_cluster())
            .filter(|cluster| fat16_get(&fat, *cluster).is_ok_and(|value| value == 0))
            .count();
        stats.byte_exact = self.bytes == original_bytes;

        let verified = HdiImage::parse(self.bytes.clone())?;
        let verified_by_path: HashMap<_, _> = verified
            .scan()?
            .entries
            .into_iter()
            .map(|entry| (entry.host_path.clone(), entry))
            .collect();
        for (source_entry, expected) in replacements {
            let actual_entry = verified_by_path
                .get(&source_entry.host_path)
                .ok_or_else(|| {
                    HdiError::Invalid(format!(
                        "file disappeared after rebuild: {}",
                        source_entry.host_path
                    ))
                })?;
            let actual = file_bytes(verified.bytes(), verified.layout(), actual_entry)?;
            if actual != *expected {
                return Err(HdiError::Invalid(format!(
                    "rebuilt file verification failed: {}",
                    source_entry.host_path
                )));
            }
        }
        Ok(stats)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Manifest {
    format: String,
    source_sha256: String,
    source_bytes: u64,
    layout: HdiLayout,
    entries: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ManifestEntry {
    host_path: String,
    kind: EntryKind,
    raw_short_name_hex: String,
    attributes: u8,
    directory_entry_offset: u64,
    first_cluster: u16,
    clusters: Vec<u16>,
    size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha256: Option<String>,
}

fn manifest_for_image(image: &HdiImage) -> Result<(Manifest, TreeScan)> {
    let scan = image.scan()?;
    let mut entries = Vec::with_capacity(scan.entries.len());
    for entry in &scan.entries {
        let sha256 = if entry.kind == EntryKind::File {
            Some(sha256_hex(&file_bytes(
                image.bytes(),
                image.layout(),
                entry,
            )?))
        } else {
            None
        };
        entries.push(ManifestEntry {
            host_path: entry.host_path.clone(),
            kind: entry.kind,
            raw_short_name_hex: hex::encode_upper(entry.directory.name_raw),
            attributes: entry.directory.attributes,
            directory_entry_offset: u64::try_from(entry.directory.offset)
                .expect("image offset fits u64"),
            first_cluster: entry.directory.first_cluster,
            clusters: entry.chain.clone(),
            size: u64::from(entry.directory.size),
            sha256,
        });
    }
    Ok((
        Manifest {
            format: MANIFEST_FORMAT.to_owned(),
            source_sha256: sha256_hex(image.bytes()),
            source_bytes: u64::try_from(image.bytes().len()).expect("image size fits u64"),
            layout: image.layout().clone(),
            entries,
        },
        scan,
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnpackStats {
    pub extracted_files: usize,
    pub extracted_dirs: usize,
    pub extracted_bytes: u64,
    pub orphan_clusters: usize,
    pub fat_mismatch_entries: usize,
    pub manifest: PathBuf,
}

pub fn unpack_hdi(source: &Path, output: &Path) -> Result<UnpackStats> {
    if output.exists() {
        return Err(HdiError::OutputExists(output.to_path_buf()));
    }
    let source_bytes = fs::read(source).map_err(|error| io_error(source, error))?;
    let image = HdiImage::parse(source_bytes)?;
    let (manifest, scan) = manifest_for_image(&image)?;
    let mut prepared_files = Vec::new();
    let mut extracted_bytes = 0u64;
    for entry in &scan.entries {
        if entry.kind == EntryKind::File {
            let data = file_bytes(image.bytes(), image.layout(), entry)?;
            extracted_bytes = extracted_bytes
                .checked_add(u64::try_from(data.len()).expect("file size fits u64"))
                .ok_or_else(|| {
                    HdiError::Invalid("extracted byte count overflows u64".to_owned())
                })?;
            prepared_files.push((entry.host_path.clone(), data));
        }
    }
    let mut manifest_text =
        serde_json::to_string_pretty(&manifest).map_err(|error| json_error(output, error))?;
    manifest_text.push('\n');

    fs::create_dir_all(output).map_err(|error| io_error(output, error))?;
    let result = (|| -> Result<()> {
        for entry in &scan.entries {
            if entry.kind == EntryKind::Directory {
                let path = path_from_host(output, &entry.host_path);
                fs::create_dir(&path).map_err(|error| io_error(&path, error))?;
            }
        }
        for (host_path, data) in &prepared_files {
            let path = path_from_host(output, host_path);
            fs::write(&path, data).map_err(|error| io_error(&path, error))?;
        }
        let manifest_path = output.join(MANIFEST_FILENAME);
        fs::write(&manifest_path, manifest_text.as_bytes())
            .map_err(|error| io_error(&manifest_path, error))?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_dir_all(output);
        return Err(error);
    }

    Ok(UnpackStats {
        extracted_files: prepared_files.len(),
        extracted_dirs: scan
            .entries
            .iter()
            .filter(|entry| entry.kind == EntryKind::Directory)
            .count(),
        extracted_bytes,
        orphan_clusters: scan.orphan_clusters,
        fat_mismatch_entries: image.layout().fat_mismatch_entries,
        manifest: output.join(MANIFEST_FILENAME),
    })
}

fn collect_host_entries(root: &Path) -> Result<HashMap<String, EntryKind>> {
    fn visit(root: &Path, relative: &str, output: &mut HashMap<String, EntryKind>) -> Result<()> {
        let directory = path_from_host(root, relative);
        for entry in fs::read_dir(&directory).map_err(|error| io_error(&directory, error))? {
            let entry = entry.map_err(|error| io_error(&directory, error))?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|error| io_error(&path, error))?;
            let name = entry.file_name().into_string().map_err(|_| {
                HdiError::Invalid(format!(
                    "non-Unicode path in unpacked directory: {}",
                    path.display()
                ))
            })?;
            let host_path = join_host_path(relative, &name);
            if relative.is_empty() && name == MANIFEST_FILENAME {
                if !file_type.is_file() || file_type.is_symlink() {
                    return Err(HdiError::Invalid(format!(
                        "manifest is not a regular file: {}",
                        path.display()
                    )));
                }
                continue;
            }
            if file_type.is_symlink() {
                return Err(HdiError::Invalid(format!(
                    "symbolic links are not supported in unpacked trees: {}",
                    path.display()
                )));
            }
            let kind = if file_type.is_file() {
                EntryKind::File
            } else if file_type.is_dir() {
                EntryKind::Directory
            } else {
                return Err(HdiError::Invalid(format!(
                    "unsupported filesystem entry: {}",
                    path.display()
                )));
            };
            if output.insert(host_path.clone(), kind).is_some() {
                return Err(HdiError::Invalid(format!(
                    "duplicate host path in unpacked tree: {host_path}"
                )));
            }
            if kind == EntryKind::Directory {
                visit(root, &host_path, output)?;
            }
        }
        Ok(())
    }

    if !root.is_dir() {
        return Err(HdiError::Invalid(format!(
            "unpacked path is not a directory: {}",
            root.display()
        )));
    }
    let mut entries = HashMap::new();
    visit(root, "", &mut entries)?;
    Ok(entries)
}

fn validate_unpacked_tree(unpacked: &Path, manifest: &Manifest) -> Result<()> {
    let actual = collect_host_entries(unpacked)?;
    let expected: HashMap<_, _> = manifest
        .entries
        .iter()
        .map(|entry| (entry.host_path.clone(), entry.kind))
        .collect();
    for (path, kind) in &expected {
        match actual.get(path) {
            Some(actual_kind) if actual_kind == kind => {}
            Some(_) => {
                return Err(HdiError::Invalid(format!(
                    "entry type changed in unpacked tree: {path}"
                )))
            }
            None => {
                return Err(HdiError::Invalid(format!(
                    "entry is missing from unpacked tree: {path}"
                )))
            }
        }
    }
    if let Some(extra) = actual.keys().find(|path| !expected.contains_key(*path)) {
        return Err(HdiError::Invalid(format!(
            "unexpected entry in unpacked tree (add/delete/rename is unsupported): {extra}"
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackStats {
    pub source_files: usize,
    pub changed_files: usize,
    pub unchanged_files: usize,
    pub allocated_clusters: usize,
    pub freed_clusters: usize,
    pub free_clusters_after: usize,
    pub byte_exact: bool,
    pub source_fat_mismatch_entries: usize,
}

#[derive(Debug)]
pub struct PreparedPack {
    pub source: PathBuf,
    pub unpacked: PathBuf,
    pub image: HdiImage,
    pub stats: PackStats,
}

impl PreparedPack {
    pub fn write_to(&self, output: &Path) -> Result<()> {
        if output.exists() {
            return Err(HdiError::OutputExists(output.to_path_buf()));
        }
        if let Some(parent) = output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
        }
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)
            .map_err(|error| io_error(output, error))?;
        if let Err(error) = file.write_all(self.image.bytes()) {
            drop(file);
            let _ = fs::remove_file(output);
            return Err(io_error(output, error));
        }
        Ok(())
    }
}

pub fn prepare_pack(source: &Path, unpacked: &Path) -> Result<PreparedPack> {
    let source_bytes = fs::read(source).map_err(|error| io_error(source, error))?;
    let manifest_path = unpacked.join(MANIFEST_FILENAME);
    let manifest_bytes =
        fs::read(&manifest_path).map_err(|error| io_error(&manifest_path, error))?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| json_error(&manifest_path, error))?;
    if manifest.format != MANIFEST_FORMAT {
        return Err(HdiError::Invalid(format!(
            "unsupported manifest format {:?}; expected {MANIFEST_FORMAT:?}",
            manifest.format
        )));
    }
    if manifest.source_bytes != u64::try_from(source_bytes.len()).expect("source size fits u64")
        || manifest.source_sha256 != sha256_hex(&source_bytes)
    {
        return Err(HdiError::Invalid(
            "source HDI does not match the manifest size and SHA-256".to_owned(),
        ));
    }

    let mut image = HdiImage::parse(source_bytes)?;
    let source_fat_mismatch_entries = image.layout().fat_mismatch_entries;
    let (expected_manifest, scan) = manifest_for_image(&image)?;
    if manifest != expected_manifest {
        return Err(HdiError::Invalid(
            "manifest metadata does not match the source HDI".to_owned(),
        ));
    }
    validate_unpacked_tree(unpacked, &manifest)?;

    let mut replacements = Vec::new();
    for entry in scan.entries {
        if entry.kind != EntryKind::File {
            continue;
        }
        let path = path_from_host(unpacked, &entry.host_path);
        let data = fs::read(&path).map_err(|error| io_error(&path, error))?;
        replacements.push((entry, data));
    }
    let mut stats = image.replace_contents(&replacements)?;
    stats.source_fat_mismatch_entries = source_fat_mismatch_entries;
    Ok(PreparedPack {
        source: source.to_path_buf(),
        unpacked: unpacked.to_path_buf(),
        image,
        stats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn put_entry(
        image: &mut [u8],
        offset: usize,
        raw_name: &[u8; 11],
        attributes: u8,
        cluster: u16,
        size: u32,
    ) {
        image[offset..offset + 11].copy_from_slice(raw_name);
        image[offset + 11] = attributes;
        image[offset + 26..offset + 28].copy_from_slice(&cluster.to_le_bytes());
        image[offset + 28..offset + 32].copy_from_slice(&size.to_le_bytes());
    }

    fn synthetic_image() -> Vec<u8> {
        let header_size = 0x1000usize;
        let physical_sector = 512usize;
        let sectors_per_track = 17usize;
        let heads = 8usize;
        let cylinders = 64usize;
        let disk_size = physical_sector * sectors_per_track * heads * cylinders;
        let mut image = vec![0u8; header_size + disk_size];
        image[0x08..0x0C].copy_from_slice(&(header_size as u32).to_le_bytes());
        image[0x0C..0x10].copy_from_slice(&(disk_size as u32).to_le_bytes());
        image[0x10..0x14].copy_from_slice(&(physical_sector as u32).to_le_bytes());
        image[0x14..0x18].copy_from_slice(&(sectors_per_track as u32).to_le_bytes());
        image[0x18..0x1C].copy_from_slice(&(heads as u32).to_le_bytes());
        image[0x1C..0x20].copy_from_slice(&(cylinders as u32).to_le_bytes());

        let partition_entry = header_size + physical_sector;
        image[partition_entry] = 0xA1;
        image[partition_entry + 1] = 0x91;
        image[partition_entry + 10..partition_entry + 12].copy_from_slice(&1u16.to_le_bytes());

        let partition = header_size + physical_sector * sectors_per_track * heads;
        let bytes_per_sector = 512usize;
        let total_sectors = 8_000usize;
        let sectors_per_fat = 32usize;
        let root_entries = 128usize;
        image[partition..partition + 3].copy_from_slice(&[0xEB, 0x3C, 0x90]);
        image[partition + 3..partition + 11].copy_from_slice(b"NEC  6.2");
        image[partition + 11..partition + 13]
            .copy_from_slice(&(bytes_per_sector as u16).to_le_bytes());
        image[partition + 13] = 1;
        image[partition + 14..partition + 16].copy_from_slice(&1u16.to_le_bytes());
        image[partition + 16] = 2;
        image[partition + 17..partition + 19].copy_from_slice(&(root_entries as u16).to_le_bytes());
        image[partition + 19..partition + 21]
            .copy_from_slice(&(total_sectors as u16).to_le_bytes());
        image[partition + 21] = 0xF8;
        image[partition + 22..partition + 24]
            .copy_from_slice(&(sectors_per_fat as u16).to_le_bytes());
        image[partition + 24..partition + 26]
            .copy_from_slice(&(sectors_per_track as u16).to_le_bytes());
        image[partition + 26..partition + 28].copy_from_slice(&(heads as u16).to_le_bytes());
        image[partition + 28..partition + 32]
            .copy_from_slice(&((sectors_per_track * heads) as u32).to_le_bytes());
        image[partition + 54..partition + 62].copy_from_slice(b"FAT16   ");

        let fat_offset = partition + bytes_per_sector;
        let fat_bytes = sectors_per_fat * bytes_per_sector;
        let mut fat = vec![0u8; fat_bytes];
        fat16_set(&mut fat, 0, 0xFFF8).unwrap();
        fat16_set(&mut fat, 1, FAT16_EOC).unwrap();
        for cluster in 2..=4 {
            fat16_set(&mut fat, cluster, FAT16_EOC).unwrap();
        }
        image[fat_offset..fat_offset + fat_bytes].copy_from_slice(&fat);
        image[fat_offset + fat_bytes..fat_offset + 2 * fat_bytes].copy_from_slice(&fat);

        let root = fat_offset + 2 * fat_bytes;
        put_entry(&mut image, root, b"GAME       ", 0x10, 2, 0);
        put_entry(
            &mut image,
            root + DIRECTORY_ENTRY_SIZE,
            b"AUTOEXECBAT",
            0x20,
            3,
            4,
        );
        let data = root + root_entries * DIRECTORY_ENTRY_SIZE;
        put_entry(&mut image, data, b".          ", 0x10, 2, 0);
        put_entry(
            &mut image,
            data + DIRECTORY_ENTRY_SIZE,
            b"..         ",
            0x10,
            0,
            0,
        );
        put_entry(
            &mut image,
            data + 2 * DIRECTORY_ENTRY_SIZE,
            b"SCRIPT  DAT",
            0x20,
            4,
            3,
        );
        put_entry(
            &mut image,
            data + 3 * DIRECTORY_ENTRY_SIZE,
            b"EMPTY   BIN",
            0x20,
            0,
            0,
        );
        image[data + bytes_per_sector..data + bytes_per_sector + 4].copy_from_slice(b"boot");
        image[data + 2 * bytes_per_sector..data + 2 * bytes_per_sector + 3].copy_from_slice(b"old");
        image
    }

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "yuno_hdi_tools_{label}_{}_{}",
            std::process::id(),
            nonce
        ))
    }

    #[test]
    fn parses_fat16_and_reads_nested_files() {
        let image = HdiImage::parse(synthetic_image()).unwrap();
        assert_eq!(image.layout.header_size, 0x1000);
        assert_eq!(image.layout.partition_offset, 0x12000);
        assert_eq!(image.layout.cluster_count, 7_927);
        let entries: HashMap<_, _> = image
            .scan()
            .unwrap()
            .entries
            .into_iter()
            .map(|entry| (entry.host_path.clone(), entry))
            .collect();
        assert_eq!(
            file_bytes(image.bytes(), image.layout(), &entries["AUTOEXEC.BAT"]).unwrap(),
            b"boot"
        );
        assert_eq!(
            file_bytes(image.bytes(), image.layout(), &entries["GAME/SCRIPT.DAT"]).unwrap(),
            b"old"
        );
        assert_eq!(
            file_bytes(image.bytes(), image.layout(), &entries["GAME/EMPTY.BIN"]).unwrap(),
            b""
        );
    }

    #[test]
    fn unchanged_unpack_pack_is_byte_exact_and_growth_round_trips() {
        let root = temp_root("roundtrip");
        fs::create_dir(&root).unwrap();
        let source_path = root.join("source.hdi");
        let unpacked = root.join("source_unpacked");
        let source = synthetic_image();
        fs::write(&source_path, &source).unwrap();
        let unpack_stats = unpack_hdi(&source_path, &unpacked).unwrap();
        assert_eq!(unpack_stats.extracted_files, 3);
        assert_eq!(unpack_stats.extracted_dirs, 1);

        let unchanged = prepare_pack(&source_path, &unpacked).unwrap();
        assert_eq!(unchanged.stats.unchanged_files, 3);
        assert!(unchanged.stats.byte_exact);
        assert_eq!(unchanged.image.bytes(), source);

        let replacement = vec![0xA5; 1_500];
        fs::write(unpacked.join("GAME").join("SCRIPT.DAT"), &replacement).unwrap();
        let changed = prepare_pack(&source_path, &unpacked).unwrap();
        assert_eq!(changed.stats.changed_files, 1);
        assert_eq!(changed.stats.allocated_clusters, 2);
        assert!(!changed.stats.byte_exact);
        let rebuilt = HdiImage::parse(changed.image.bytes().to_vec()).unwrap();
        let script = rebuilt
            .scan()
            .unwrap()
            .entries
            .into_iter()
            .find(|entry| entry.host_path == "GAME/SCRIPT.DAT")
            .unwrap();
        assert_eq!(
            file_bytes(rebuilt.bytes(), rebuilt.layout(), &script).unwrap(),
            replacement
        );

        let mut shrunk = rebuilt;
        let script = shrunk
            .scan()
            .unwrap()
            .entries
            .into_iter()
            .find(|entry| entry.host_path == "GAME/SCRIPT.DAT")
            .unwrap();
        let stats = shrunk.replace_contents(&[(script, b"x".to_vec())]).unwrap();
        assert_eq!(stats.changed_files, 1);
        assert_eq!(stats.freed_clusters, 2);
        let shrunk_script = shrunk
            .scan()
            .unwrap()
            .entries
            .into_iter()
            .find(|entry| entry.host_path == "GAME/SCRIPT.DAT")
            .unwrap();
        assert_eq!(
            file_bytes(shrunk.bytes(), shrunk.layout(), &shrunk_script).unwrap(),
            b"x"
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn extra_file_and_wrong_source_are_rejected() {
        let root = temp_root("reject");
        fs::create_dir(&root).unwrap();
        let source_path = root.join("source.hdi");
        let unpacked = root.join("source_unpacked");
        fs::write(&source_path, synthetic_image()).unwrap();
        unpack_hdi(&source_path, &unpacked).unwrap();
        fs::write(unpacked.join("EXTRA.DAT"), b"extra").unwrap();
        assert!(matches!(
            prepare_pack(&source_path, &unpacked),
            Err(HdiError::Invalid(message)) if message.contains("unexpected entry")
        ));
        fs::remove_file(unpacked.join("EXTRA.DAT")).unwrap();
        let mut wrong = synthetic_image();
        wrong[0x20] = 1;
        let wrong_path = root.join("wrong.hdi");
        fs::write(&wrong_path, wrong).unwrap();
        assert!(matches!(
            prepare_pack(&wrong_path, &unpacked),
            Err(HdiError::Invalid(message)) if message.contains("SHA-256")
        ));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn mismatched_fat_copy_is_reported_and_preserved() {
        let mut bytes = synthetic_image();
        let layout = parse_layout(&bytes).unwrap();
        bytes[layout.fat_offset + layout.fat_bytes() + 4] ^= 1;
        let parsed = HdiImage::parse(bytes.clone()).unwrap();
        assert_eq!(parsed.layout.fat_mismatch_entries, 1);
        assert_eq!(parsed.bytes(), bytes);
    }
}
