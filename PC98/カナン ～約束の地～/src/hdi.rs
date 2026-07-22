use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

const HDI_HEADER_MIN_SIZE: usize = 0x20;
const PARTITION_ENTRY_SIZE: usize = 0x20;
const DIRECTORY_ENTRY_SIZE: usize = 0x20;
const FAT12_MAX_CLUSTERS: usize = 4_085;
const FAT12_EOC_MIN: u16 = 0xFF8;
const FAT12_EOC: u16 = 0xFFF;

#[derive(Debug, Error)]
pub enum HdiError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("invalid HDI/FAT12 image: {0}")]
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

fn checked_range<'a>(
    data: &'a [u8],
    offset: usize,
    length: usize,
    context: &str,
) -> Result<&'a [u8]> {
    data.get(offset..offset.saturating_add(length))
        .ok_or_else(|| {
            HdiError::Invalid(format!(
                "{context} range 0x{offset:X}..0x{:X} is outside image length 0x{:X}",
                offset.saturating_add(length),
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
    data.get_mut(offset..offset.saturating_add(length))
        .ok_or_else(|| {
            HdiError::Invalid(format!(
                "{context} range 0x{offset:X}..0x{:X} is outside image length 0x{image_length:X}",
                offset.saturating_add(length)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HdiLayout {
    pub header_size: usize,
    pub disk_size: usize,
    pub physical_sector_size: usize,
    pub sectors_per_track: usize,
    pub heads: usize,
    pub cylinders: usize,
    pub partition_offset: usize,
    pub bytes_per_sector: usize,
    pub sectors_per_cluster: usize,
    pub reserved_sectors: usize,
    pub fat_count: usize,
    pub sectors_per_fat: usize,
    pub root_entries: usize,
    pub total_sectors: usize,
    pub fat_offset: usize,
    pub root_offset: usize,
    pub root_sectors: usize,
    pub data_offset: usize,
    pub cluster_size: usize,
    pub cluster_count: usize,
}

impl HdiLayout {
    fn max_cluster(&self) -> u16 {
        u16::try_from(self.cluster_count + 1).expect("FAT12 cluster count fits u16")
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
        Ok(self.data_offset + (usize::from(cluster) - 2) * self.cluster_size)
    }
}

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
pub struct Replacement {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HdiPatchStats {
    pub requested_files: usize,
    pub changed_files: usize,
    pub unchanged_files: usize,
    pub allocated_clusters: usize,
    pub freed_clusters: usize,
    pub free_clusters_after: usize,
    pub byte_exact: bool,
}

#[derive(Debug, Clone)]
pub struct HdiImage {
    bytes: Vec<u8>,
    layout: HdiLayout,
}

fn parse_layout(data: &[u8]) -> Result<HdiLayout> {
    if data.len() < HDI_HEADER_MIN_SIZE {
        return Err(HdiError::Invalid(format!(
            "file is shorter than HDI header: {} bytes",
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
    let geometry_size = physical_sector_size
        .checked_mul(sectors_per_track)
        .and_then(|value| value.checked_mul(heads))
        .and_then(|value| value.checked_mul(cylinders))
        .ok_or_else(|| HdiError::Invalid("HDI geometry overflows usize".to_owned()))?;
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

    let partition_table = header_size
        .checked_add(physical_sector_size)
        .ok_or_else(|| HdiError::Invalid("partition table offset overflow".to_owned()))?;
    checked_range(
        data,
        partition_table,
        physical_sector_size,
        "PC-98 partition table",
    )?;
    let mut partition_offset = None;
    for index in 0..physical_sector_size / PARTITION_ENTRY_SIZE {
        let entry = partition_table + index * PARTITION_ENTRY_SIZE;
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
            partition_offset = Some(candidate);
            break;
        }
    }
    let partition_offset = partition_offset
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
    if cluster_count == 0 || cluster_count >= FAT12_MAX_CLUSTERS {
        return Err(HdiError::Invalid(format!(
            "only FAT12 is supported, cluster count is {cluster_count}"
        )));
    }
    let partition_bytes = total_sectors
        .checked_mul(bytes_per_sector)
        .ok_or_else(|| HdiError::Invalid("partition size overflow".to_owned()))?;
    checked_range(data, partition_offset, partition_bytes, "FAT partition")?;
    let fat_offset = partition_offset + reserved_sectors * bytes_per_sector;
    let fat_bytes = sectors_per_fat * bytes_per_sector;
    let root_offset = fat_offset + fat_count * fat_bytes;
    let data_offset = root_offset + root_sectors * bytes_per_sector;
    let cluster_size = sectors_per_cluster * bytes_per_sector;
    let required_fat_bytes = ((cluster_count + 2) * 3).div_ceil(2);
    if required_fat_bytes > fat_bytes {
        return Err(HdiError::Invalid(format!(
            "FAT has {fat_bytes} bytes but {required_fat_bytes} are required"
        )));
    }
    let first_fat = checked_range(data, fat_offset, fat_bytes, "first FAT")?;
    for copy in 1..fat_count {
        let other = checked_range(data, fat_offset + copy * fat_bytes, fat_bytes, "FAT copy")?;
        if other != first_fat {
            return Err(HdiError::Invalid(format!(
                "FAT copy {copy} differs from the first FAT"
            )));
        }
    }
    if fat12_get(first_fat, 0)? < 0xFF0 || fat12_get(first_fat, 1)? < FAT12_EOC_MIN {
        return Err(HdiError::Invalid(
            "FAT12 reserved entries 0 and 1 are invalid".to_owned(),
        ));
    }

    Ok(HdiLayout {
        header_size,
        disk_size,
        physical_sector_size,
        sectors_per_track,
        heads,
        cylinders,
        partition_offset,
        bytes_per_sector,
        sectors_per_cluster,
        reserved_sectors,
        fat_count,
        sectors_per_fat,
        root_entries,
        total_sectors,
        fat_offset,
        root_offset,
        root_sectors,
        data_offset,
        cluster_size,
        cluster_count,
    })
}

fn fat12_get(fat: &[u8], cluster: u16) -> Result<u16> {
    let cluster = usize::from(cluster);
    let offset = cluster + cluster / 2;
    let bytes = checked_range(fat, offset, 2, "FAT12 entry")?;
    let word = u16::from_le_bytes([bytes[0], bytes[1]]);
    Ok(if cluster & 1 == 0 {
        word & 0x0FFF
    } else {
        (word >> 4) & 0x0FFF
    })
}

fn fat12_set(fat: &mut [u8], cluster: u16, value: u16) -> Result<()> {
    if value > 0x0FFF {
        return Err(HdiError::Invalid(format!(
            "FAT12 value 0x{value:04X} is too large"
        )));
    }
    let cluster_usize = usize::from(cluster);
    let offset = cluster_usize + cluster_usize / 2;
    let bytes = checked_range_mut(fat, offset, 2, "FAT12 entry")?;
    if cluster_usize & 1 == 0 {
        bytes[0] = value as u8;
        bytes[1] = (bytes[1] & 0xF0) | ((value >> 8) as u8 & 0x0F);
    } else {
        bytes[0] = (bytes[0] & 0x0F) | ((value << 4) as u8 & 0xF0);
        bytes[1] = (value >> 4) as u8;
    }
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
        let next = fat12_get(fat, cluster)?;
        if next >= FAT12_EOC_MIN {
            break;
        }
        if next == 0 || next == 1 || next == 0xFF7 || next > layout.max_cluster() {
            return Err(HdiError::Invalid(format!(
                "invalid FAT chain value 0x{next:03X} after cluster {cluster}"
            )));
        }
        cluster = next;
    }
    Ok(chain)
}

fn encode_short_name(name: &str) -> Result<[u8; 11]> {
    if name.is_empty() || name == "." || name == ".." {
        return Err(HdiError::Invalid(format!("invalid 8.3 name: {name:?}")));
    }
    let mut pieces = name.split('.');
    let base = pieces.next().expect("one piece");
    let extension = pieces.next().unwrap_or("");
    if pieces.next().is_some()
        || base.is_empty()
        || base.len() > 8
        || extension.len() > 3
        || !base.is_ascii()
        || !extension.is_ascii()
    {
        return Err(HdiError::Invalid(format!(
            "name is not a supported ASCII 8.3 name: {name}"
        )));
    }
    fn allowed(byte: u8) -> bool {
        byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b'$' | b'%'
                    | b'\''
                    | b'-'
                    | b'_'
                    | b'@'
                    | b'~'
                    | b'`'
                    | b'!'
                    | b'('
                    | b')'
                    | b'{'
                    | b'}'
                    | b'^'
                    | b'#'
                    | b'&'
            )
    }
    let mut raw = [b' '; 11];
    for (index, byte) in base
        .bytes()
        .map(|byte| byte.to_ascii_uppercase())
        .enumerate()
    {
        if !allowed(byte) {
            return Err(HdiError::Invalid(format!(
                "unsupported character in 8.3 name: {name}"
            )));
        }
        raw[index] = byte;
    }
    for (index, byte) in extension
        .bytes()
        .map(|byte| byte.to_ascii_uppercase())
        .enumerate()
    {
        if !allowed(byte) {
            return Err(HdiError::Invalid(format!(
                "unsupported character in 8.3 name: {name}"
            )));
        }
        raw[8 + index] = byte;
    }
    Ok(raw)
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

fn find_entry(
    data: &[u8],
    layout: &HdiLayout,
    fat: &[u8],
    directory_cluster: Option<u16>,
    name: &[u8; 11],
) -> Result<DirectoryEntry> {
    let matches: Vec<_> = directory_entries(data, layout, fat, directory_cluster)?
        .into_iter()
        .filter(|entry| &entry.name_raw == name)
        .collect();
    match matches.as_slice() {
        [entry] => Ok(entry.clone()),
        [] => Err(HdiError::Invalid(format!(
            "directory entry not found: {}",
            String::from_utf8_lossy(name)
        ))),
        _ => Err(HdiError::Invalid(format!(
            "duplicate directory entry: {}",
            String::from_utf8_lossy(name)
        ))),
    }
}

fn locate_directory(
    data: &[u8],
    layout: &HdiLayout,
    fat: &[u8],
    path: &str,
) -> Result<Option<u16>> {
    let normalized = path.replace('\\', "/");
    let mut current = None;
    for component in normalized
        .split('/')
        .filter(|component| !component.is_empty())
    {
        let raw = encode_short_name(component)?;
        let entry = find_entry(data, layout, fat, current, &raw)?;
        if !entry.is_directory() || entry.is_volume_label() {
            return Err(HdiError::Invalid(format!(
                "destination component is not a directory: {component}"
            )));
        }
        current = Some(entry.first_cluster);
    }
    Ok(current)
}

fn file_bytes(
    data: &[u8],
    layout: &HdiLayout,
    fat: &[u8],
    entry: &DirectoryEntry,
) -> Result<Vec<u8>> {
    let size = usize_from_u32(entry.size, "directory file size")?;
    let chain = cluster_chain(layout, fat, entry.first_cluster)?;
    if chain.len().saturating_mul(layout.cluster_size) < size {
        return Err(HdiError::Invalid(format!(
            "file chain at directory offset 0x{:X} has {} bytes for file size {size}",
            entry.offset,
            chain.len() * layout.cluster_size
        )));
    }
    let mut output = Vec::with_capacity(chain.len() * layout.cluster_size);
    for cluster in chain {
        let offset = layout.cluster_offset(cluster)?;
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

fn raw_name_display(raw: &[u8; 11]) -> String {
    let base = String::from_utf8_lossy(&raw[..8]).trim_end().to_owned();
    let extension = String::from_utf8_lossy(&raw[8..]).trim_end().to_owned();
    if extension.is_empty() {
        base
    } else {
        format!("{base}.{extension}")
    }
}

fn validate_filesystem(data: &[u8], layout: &HdiLayout, fat: &[u8]) -> Result<()> {
    fn visit_directory(
        data: &[u8],
        layout: &HdiLayout,
        fat: &[u8],
        directory_cluster: Option<u16>,
        path: &str,
        owners: &mut HashMap<u16, String>,
    ) -> Result<()> {
        let entries = directory_entries(data, layout, fat, directory_cluster)?;
        let mut names = HashSet::new();
        for entry in entries {
            if entry.name_raw[0] == b'.' || entry.is_volume_label() {
                continue;
            }
            if !names.insert(entry.name_raw) {
                return Err(HdiError::Invalid(format!(
                    "duplicate directory entry in {path}: {}",
                    raw_name_display(&entry.name_raw)
                )));
            }
            let name = raw_name_display(&entry.name_raw);
            let child_path = if path.is_empty() {
                name
            } else {
                format!("{path}/{name}")
            };
            if entry.first_cluster == 0 {
                if entry.is_directory() || entry.size != 0 {
                    return Err(HdiError::Invalid(format!(
                        "{child_path} has no first cluster"
                    )));
                }
                continue;
            }
            let chain = cluster_chain(layout, fat, entry.first_cluster)?;
            for cluster in &chain {
                if let Some(previous) = owners.insert(*cluster, child_path.clone()) {
                    return Err(HdiError::Invalid(format!(
                        "cluster {cluster} is cross-linked by {previous} and {child_path}"
                    )));
                }
            }
            if entry.is_directory() {
                visit_directory(
                    data,
                    layout,
                    fat,
                    Some(entry.first_cluster),
                    &child_path,
                    owners,
                )?;
            } else {
                let size = usize_from_u32(entry.size, "directory file size")?;
                if chain.len().saturating_mul(layout.cluster_size) < size {
                    return Err(HdiError::Invalid(format!(
                        "{child_path} has a {}-byte chain for size {size}",
                        chain.len() * layout.cluster_size
                    )));
                }
            }
        }
        Ok(())
    }

    let mut owners = HashMap::new();
    visit_directory(data, layout, fat, None, "", &mut owners)?;
    for cluster in 2..=layout.max_cluster() {
        let value = fat12_get(fat, cluster)?;
        if value == 0 || value == 0xFF7 {
            continue;
        }
        if !owners.contains_key(&cluster) {
            return Err(HdiError::Invalid(format!(
                "allocated FAT cluster {cluster} is not referenced by any directory entry"
            )));
        }
    }
    Ok(())
}

impl HdiImage {
    pub fn parse(bytes: Vec<u8>) -> Result<Self> {
        let layout = parse_layout(&bytes)?;
        let fat = checked_range(&bytes, layout.fat_offset, layout.fat_bytes(), "primary FAT")?;
        validate_filesystem(&bytes, &layout, fat)?;
        Ok(Self { bytes, layout })
    }

    pub fn layout(&self) -> &HdiLayout {
        &self.layout
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
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

    pub fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let normalized = path.replace('\\', "/");
        let mut components: Vec<_> = normalized
            .split('/')
            .filter(|component| !component.is_empty())
            .collect();
        let file_name = components
            .pop()
            .ok_or_else(|| HdiError::Invalid("empty file path".to_owned()))?;
        let fat = self.primary_fat()?;
        let directory = locate_directory(&self.bytes, &self.layout, &fat, &components.join("/"))?;
        let entry = find_entry(
            &self.bytes,
            &self.layout,
            &fat,
            directory,
            &encode_short_name(file_name)?,
        )?;
        if entry.is_directory() || entry.is_volume_label() {
            return Err(HdiError::Invalid(format!("not a regular file: {path}")));
        }
        file_bytes(&self.bytes, &self.layout, &fat, &entry)
    }

    pub fn replace_files(
        &mut self,
        destination: &str,
        replacements: &[Replacement],
    ) -> Result<HdiPatchStats> {
        if replacements.is_empty() {
            return Err(HdiError::Invalid(
                "no replacement files supplied".to_owned(),
            ));
        }
        let original_bytes = self.bytes.clone();
        let mut fat = self.primary_fat()?;
        let destination_cluster = locate_directory(&self.bytes, &self.layout, &fat, destination)?;
        let mut names = HashSet::new();

        struct Job<'a> {
            replacement: &'a Replacement,
            entry: DirectoryEntry,
            old_chain: Vec<u16>,
            needed_clusters: usize,
            new_chain: Vec<u16>,
        }

        let mut jobs = Vec::new();
        let mut stats = HdiPatchStats {
            requested_files: replacements.len(),
            ..HdiPatchStats::default()
        };
        for replacement in replacements {
            let name_raw = encode_short_name(&replacement.name)?;
            if !names.insert(name_raw) {
                return Err(HdiError::Invalid(format!(
                    "duplicate replacement name: {}",
                    replacement.name
                )));
            }
            let entry = find_entry(
                &self.bytes,
                &self.layout,
                &fat,
                destination_cluster,
                &name_raw,
            )?;
            if entry.is_directory() || entry.is_volume_label() {
                return Err(HdiError::Invalid(format!(
                    "replacement target is not a regular file: {}",
                    replacement.name
                )));
            }
            let old_bytes = file_bytes(&self.bytes, &self.layout, &fat, &entry)?;
            if old_bytes == replacement.data {
                stats.unchanged_files += 1;
                continue;
            }
            let old_chain = cluster_chain(&self.layout, &fat, entry.first_cluster)?;
            let needed_clusters = replacement.data.len().div_ceil(self.layout.cluster_size);
            jobs.push(Job {
                replacement,
                entry,
                old_chain,
                needed_clusters,
                new_chain: Vec::new(),
            });
        }
        stats.changed_files = jobs.len();

        let free_before = (2..=self.layout.max_cluster())
            .filter(|cluster| fat12_get(&fat, *cluster).is_ok_and(|value| value == 0))
            .count();
        let old_changed_clusters: usize = jobs.iter().map(|job| job.old_chain.len()).sum();
        let needed_changed_clusters: usize = jobs.iter().map(|job| job.needed_clusters).sum();
        if needed_changed_clusters > old_changed_clusters + free_before {
            self.bytes = original_bytes;
            return Err(HdiError::Invalid(format!(
                "not enough FAT12 space: need {needed_changed_clusters} clusters for changed files, have {} old + {free_before} free",
                old_changed_clusters
            )));
        }

        for job in &mut jobs {
            let keep = job.old_chain.len().min(job.needed_clusters);
            job.new_chain.extend_from_slice(&job.old_chain[..keep]);
            for cluster in &job.old_chain[keep..] {
                fat12_set(&mut fat, *cluster, 0)?;
                stats.freed_clusters += 1;
            }
        }
        let mut free_clusters: Vec<_> = (2..=self.layout.max_cluster())
            .filter(|cluster| fat12_get(&fat, *cluster).is_ok_and(|value| value == 0))
            .collect();
        free_clusters.sort_unstable();
        let mut free_cursor = 0usize;
        for job in &mut jobs {
            let extra = job.needed_clusters - job.new_chain.len();
            let end = free_cursor + extra;
            if end > free_clusters.len() {
                self.bytes = original_bytes;
                return Err(HdiError::Invalid(
                    "free-cluster planning became inconsistent".to_owned(),
                ));
            }
            job.new_chain
                .extend_from_slice(&free_clusters[free_cursor..end]);
            stats.allocated_clusters += extra;
            free_cursor = end;
        }

        for job in &jobs {
            for (index, cluster) in job.new_chain.iter().enumerate() {
                let next = job.new_chain.get(index + 1).copied().unwrap_or(FAT12_EOC);
                fat12_set(&mut fat, *cluster, next)?;
            }
            let first_cluster = job.new_chain.first().copied().unwrap_or(0);
            put_u16(
                &mut self.bytes,
                job.entry.offset + 26,
                first_cluster,
                "directory first cluster",
            )?;
            let new_size = u32::try_from(job.replacement.data.len()).map_err(|_| {
                HdiError::Invalid(format!(
                    "replacement is larger than 4 GiB: {}",
                    job.replacement.name
                ))
            })?;
            put_u32(
                &mut self.bytes,
                job.entry.offset + 28,
                new_size,
                "directory file size",
            )?;
            for (index, cluster) in job.new_chain.iter().enumerate() {
                let cluster_offset = self.layout.cluster_offset(*cluster)?;
                let target = checked_range_mut(
                    &mut self.bytes,
                    cluster_offset,
                    self.layout.cluster_size,
                    "replacement cluster",
                )?;
                target.fill(0);
                let source_start = index * self.layout.cluster_size;
                let source_end =
                    (source_start + self.layout.cluster_size).min(job.replacement.data.len());
                if source_start < source_end {
                    target[..source_end - source_start]
                        .copy_from_slice(&job.replacement.data[source_start..source_end]);
                }
            }
        }

        if !jobs.is_empty() {
            for copy in 0..self.layout.fat_count {
                let offset = self.layout.fat_offset + copy * self.layout.fat_bytes();
                checked_range_mut(
                    &mut self.bytes,
                    offset,
                    self.layout.fat_bytes(),
                    "FAT output copy",
                )?
                .copy_from_slice(&fat);
            }
        }
        stats.free_clusters_after = (2..=self.layout.max_cluster())
            .filter(|cluster| fat12_get(&fat, *cluster).is_ok_and(|value| value == 0))
            .count();
        stats.byte_exact = self.bytes == original_bytes;
        Ok(stats)
    }
}

#[derive(Debug)]
pub struct PreparedHdi {
    pub source: PathBuf,
    pub replacement_directory: PathBuf,
    pub destination: String,
    pub image: HdiImage,
    pub stats: HdiPatchStats,
}

impl PreparedHdi {
    pub fn write_to(&self, output: &Path) -> Result<()> {
        if output.exists() {
            return Err(HdiError::OutputExists(output.to_path_buf()));
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|source| io_error(parent, source))?;
        }
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)
            .map_err(|source| io_error(output, source))?;
        if let Err(source) = file.write_all(self.image.bytes()) {
            drop(file);
            let _ = fs::remove_file(output);
            return Err(io_error(output, source));
        }
        Ok(())
    }
}

pub fn prepare_hdi(
    source: &Path,
    replacement_directory: &Path,
    destination: &str,
) -> Result<PreparedHdi> {
    let source_bytes = fs::read(source).map_err(|error| io_error(source, error))?;
    if !replacement_directory.is_dir() {
        return Err(HdiError::Invalid(format!(
            "replacement path is not a directory: {}",
            replacement_directory.display()
        )));
    }
    let mut replacements = Vec::new();
    for entry in fs::read_dir(replacement_directory)
        .map_err(|error| io_error(replacement_directory, error))?
    {
        let entry = entry.map_err(|error| io_error(replacement_directory, error))?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| io_error(&path, error))?;
        if !file_type.is_file() {
            return Err(HdiError::Invalid(format!(
                "replacement directory may contain files only: {}",
                path.display()
            )));
        }
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| {
                HdiError::Invalid(format!("invalid replacement filename: {}", path.display()))
            })?
            .to_owned();
        let data = fs::read(&path).map_err(|error| io_error(&path, error))?;
        replacements.push(Replacement { name, data });
    }
    replacements.sort_by(|left, right| {
        left.name
            .to_ascii_uppercase()
            .cmp(&right.name.to_ascii_uppercase())
    });
    let mut image = HdiImage::parse(source_bytes)?;
    let stats = image.replace_files(destination, &replacements)?;
    Ok(PreparedHdi {
        source: source.to_path_buf(),
        replacement_directory: replacement_directory.to_path_buf(),
        destination: destination.to_owned(),
        image,
        stats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put_entry(
        image: &mut [u8],
        offset: usize,
        name: &str,
        attributes: u8,
        cluster: u16,
        size: u32,
    ) {
        image[offset..offset + 11].copy_from_slice(&encode_short_name(name).unwrap());
        image[offset + 11] = attributes;
        image[offset + 26..offset + 28].copy_from_slice(&cluster.to_le_bytes());
        image[offset + 28..offset + 32].copy_from_slice(&size.to_le_bytes());
    }

    fn synthetic_image() -> Vec<u8> {
        let header_size = 0x1000usize;
        let physical_sector = 256usize;
        let sectors_per_track = 8usize;
        let heads = 2usize;
        let cylinders = 20usize;
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
        image[partition_entry + 1] = 0x81;
        image[partition_entry + 10..partition_entry + 12].copy_from_slice(&1u16.to_le_bytes());
        image[partition_entry + 14..partition_entry + 16].copy_from_slice(&19u16.to_le_bytes());

        let partition = header_size + physical_sector * sectors_per_track * heads;
        let bytes_per_sector = 512usize;
        let total_sectors =
            (disk_size - physical_sector * sectors_per_track * heads) / bytes_per_sector;
        image[partition..partition + 3].copy_from_slice(&[0xEB, 0x3C, 0x90]);
        image[partition + 11..partition + 13]
            .copy_from_slice(&(bytes_per_sector as u16).to_le_bytes());
        image[partition + 13] = 1;
        image[partition + 14..partition + 16].copy_from_slice(&1u16.to_le_bytes());
        image[partition + 16] = 2;
        image[partition + 17..partition + 19].copy_from_slice(&32u16.to_le_bytes());
        image[partition + 19..partition + 21]
            .copy_from_slice(&(total_sectors as u16).to_le_bytes());
        image[partition + 21] = 0xF8;
        image[partition + 22..partition + 24].copy_from_slice(&1u16.to_le_bytes());

        let fat_offset = partition + bytes_per_sector;
        let mut fat = vec![0u8; bytes_per_sector];
        fat[0..3].copy_from_slice(&[0xF8, 0xFF, 0xFF]);
        fat12_set(&mut fat, 2, FAT12_EOC).unwrap();
        fat12_set(&mut fat, 3, FAT12_EOC).unwrap();
        fat12_set(&mut fat, 4, FAT12_EOC).unwrap();
        image[fat_offset..fat_offset + bytes_per_sector].copy_from_slice(&fat);
        image[fat_offset + bytes_per_sector..fat_offset + 2 * bytes_per_sector]
            .copy_from_slice(&fat);

        let root = partition + 3 * bytes_per_sector;
        put_entry(&mut image, root, "CANAAN", 0x10, 2, 0);
        let data_offset = root + 2 * bytes_per_sector;
        put_entry(&mut image, data_offset, "DISK_J.CAT", 0x20, 3, 3);
        put_entry(
            &mut image,
            data_offset + DIRECTORY_ENTRY_SIZE,
            "KEEP.DAT",
            0x20,
            4,
            4,
        );
        image[data_offset + bytes_per_sector..data_offset + bytes_per_sector + 3]
            .copy_from_slice(b"old");
        image[data_offset + 2 * bytes_per_sector..data_offset + 2 * bytes_per_sector + 4]
            .copy_from_slice(b"keep");
        image
    }

    #[test]
    fn fat12_even_and_odd_entries_round_trip() {
        let mut fat = vec![0u8; 32];
        fat12_set(&mut fat, 2, 0xABC).unwrap();
        fat12_set(&mut fat, 3, 0xDEF).unwrap();
        assert_eq!(fat12_get(&fat, 2).unwrap(), 0xABC);
        assert_eq!(fat12_get(&fat, 3).unwrap(), 0xDEF);
    }

    #[test]
    fn no_change_is_byte_exact_and_growth_preserves_other_file() {
        let source = synthetic_image();
        let mut unchanged = HdiImage::parse(source.clone()).unwrap();
        let stats = unchanged
            .replace_files(
                "CANAAN",
                &[Replacement {
                    name: "disk_j.cat".to_owned(),
                    data: b"old".to_vec(),
                }],
            )
            .unwrap();
        assert!(stats.byte_exact);
        assert_eq!(unchanged.bytes(), source);

        let mut changed = HdiImage::parse(source).unwrap();
        let replacement = vec![0x5A; 700];
        let stats = changed
            .replace_files(
                "CANAAN",
                &[Replacement {
                    name: "DISK_J.CAT".to_owned(),
                    data: replacement.clone(),
                }],
            )
            .unwrap();
        assert_eq!(stats.changed_files, 1);
        assert_eq!(stats.allocated_clusters, 1);
        assert_eq!(changed.read_file("CANAAN/DISK_J.CAT").unwrap(), replacement);
        assert_eq!(changed.read_file("CANAAN/KEEP.DAT").unwrap(), b"keep");
        let fat_bytes = changed.layout.fat_bytes();
        assert_eq!(
            &changed.bytes[changed.layout.fat_offset..changed.layout.fat_offset + fat_bytes],
            &changed.bytes
                [changed.layout.fat_offset + fat_bytes..changed.layout.fat_offset + 2 * fat_bytes]
        );
    }

    #[test]
    fn malformed_second_fat_is_rejected() {
        let mut image = synthetic_image();
        let parsed = HdiImage::parse(image.clone()).unwrap();
        image[parsed.layout.fat_offset + parsed.layout.fat_bytes()] ^= 1;
        assert!(matches!(HdiImage::parse(image), Err(HdiError::Invalid(_))));
    }

    #[test]
    #[ignore = "requires CANAAN_HDI and CANAAN_DUMP for the real image"]
    fn real_image_round_trip_and_growth() {
        let image_path = PathBuf::from(
            std::env::var_os("CANAAN_HDI").expect("set CANAAN_HDI to the original image"),
        );
        let dump = PathBuf::from(
            std::env::var_os("CANAAN_DUMP").expect("set CANAAN_DUMP to the extracted root"),
        );
        let source = fs::read(&image_path).unwrap();
        let mut image = HdiImage::parse(source.clone()).unwrap();
        assert_eq!(image.layout.header_size, 0x1000);
        assert_eq!(image.layout.disk_size, 20_951_040);
        assert_eq!(image.layout.physical_sector_size, 256);
        assert_eq!(image.layout.sectors_per_track, 33);
        assert_eq!(image.layout.heads, 8);
        assert_eq!(image.layout.cylinders, 310);
        assert_eq!(image.layout.partition_offset, 0x11800);
        assert_eq!(image.layout.bytes_per_sector, 1024);
        assert_eq!(image.layout.sectors_per_cluster, 8);
        assert_eq!(image.layout.cluster_size, 8192);
        assert_eq!(image.layout.cluster_count, 2468);

        let mut replacements = Vec::new();
        for letter in b'A'..=b'L' {
            for extension in ["CAT", "LIB"] {
                let name = format!("DISK_{}.{extension}", char::from(letter));
                let data = fs::read(dump.join("CANAAN").join(&name)).unwrap();
                assert_eq!(image.read_file(&format!("CANAAN/{name}")).unwrap(), data);
                replacements.push(Replacement { name, data });
            }
        }
        let stats = image.replace_files("CANAAN", &replacements).unwrap();
        assert_eq!(stats.unchanged_files, 24);
        assert!(stats.byte_exact);
        assert_eq!(image.bytes(), source);

        let keep_before = image.read_file("CANAAN/KEEP.FNT").unwrap();
        let grown = vec![0xA5; 9000];
        let stats = image
            .replace_files(
                "CANAAN",
                &[Replacement {
                    name: "CANAME.TXT".to_owned(),
                    data: grown.clone(),
                }],
            )
            .unwrap();
        assert_eq!(stats.changed_files, 1);
        assert_eq!(image.read_file("CANAAN/CANAME.TXT").unwrap(), grown);
        assert_eq!(image.read_file("CANAAN/KEEP.FNT").unwrap(), keep_before);
    }

    #[test]
    #[ignore = "requires CANAAN_PATCHED_HDI and CANAAN_REPLACEMENTS"]
    fn real_patched_image_contains_replacements() {
        let image_path =
            PathBuf::from(std::env::var_os("CANAAN_PATCHED_HDI").expect("set CANAAN_PATCHED_HDI"));
        let replacements = PathBuf::from(
            std::env::var_os("CANAAN_REPLACEMENTS").expect("set CANAAN_REPLACEMENTS"),
        );
        let image = HdiImage::parse(fs::read(image_path).unwrap()).unwrap();
        let mut checked = 0usize;
        for entry in fs::read_dir(replacements).unwrap() {
            let path = entry.unwrap().path();
            if !path.is_file() {
                continue;
            }
            let name = path.file_name().unwrap().to_str().unwrap();
            assert_eq!(
                image.read_file(&format!("CANAAN/{name}")).unwrap(),
                fs::read(&path).unwrap(),
                "{name}"
            );
            checked += 1;
        }
        assert!(checked > 0);
    }
}
