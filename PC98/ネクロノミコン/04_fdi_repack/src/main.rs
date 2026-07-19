use encoding_rs::SHIFT_JIS;
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

const FDI_HEADER_SIZE: usize = 0x1000;
const FAT_EOC: u16 = 0x0FFF;

#[derive(Clone, Debug)]
struct Layout {
    bytes_per_sector: usize,
    sectors_per_cluster: usize,
    reserved_sectors: usize,
    fat_count: usize,
    root_entries: usize,
    sectors_per_fat: usize,
    total_sectors: usize,
    root_start: usize,
    root_size: usize,
    data_start: usize,
    cluster_size: usize,
    max_cluster: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PathKey(Vec<[u8; 11]>);

#[derive(Clone, Debug)]
struct FileEntry {
    key: PathKey,
    disk_offset: usize,
    start_cluster: u16,
    size: u32,
}

fn le_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| format!("missing u16 at 0x{offset:X}"))?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn le_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| format!("missing u32 at 0x{offset:X}"))?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn parse_layout(image: &[u8]) -> Result<Layout, String> {
    if image.len() < FDI_HEADER_SIZE + 0x24 {
        return Err("FDI is smaller than header plus boot sector".to_owned());
    }
    let boot = FDI_HEADER_SIZE;
    let bytes_per_sector = usize::from(le_u16(image, boot + 0x0B)?);
    let sectors_per_cluster =
        usize::from(*image.get(boot + 0x0D).ok_or("missing sectors/cluster")?);
    let reserved_sectors = usize::from(le_u16(image, boot + 0x0E)?);
    let fat_count = usize::from(*image.get(boot + 0x10).ok_or("missing FAT count")?);
    let root_entries = usize::from(le_u16(image, boot + 0x11)?);
    let total_short = usize::from(le_u16(image, boot + 0x13)?);
    let sectors_per_fat = usize::from(le_u16(image, boot + 0x16)?);
    if bytes_per_sector == 0
        || sectors_per_cluster == 0
        || reserved_sectors == 0
        || fat_count == 0
        || sectors_per_fat == 0
    {
        return Err("unsupported zero-valued FAT12 geometry field".to_owned());
    }
    let total_sectors = if total_short != 0 {
        total_short
    } else {
        usize::try_from(le_u32(image, boot + 0x20)?)
            .map_err(|_| "total sectors overflow".to_owned())?
    };
    let disk_size = total_sectors
        .checked_mul(bytes_per_sector)
        .ok_or("disk size overflow")?;
    if image.len() != FDI_HEADER_SIZE + disk_size {
        return Err(format!(
            "FDI size {} does not match header + {} disk bytes",
            image.len(),
            disk_size
        ));
    }
    let root_start_sector = reserved_sectors
        .checked_add(
            fat_count
                .checked_mul(sectors_per_fat)
                .ok_or("FAT geometry overflow")?,
        )
        .ok_or("root geometry overflow")?;
    let root_size = root_entries
        .checked_mul(32)
        .ok_or("root directory overflow")?;
    let root_sectors = root_size.div_ceil(bytes_per_sector);
    let data_start_sector = root_start_sector
        .checked_add(root_sectors)
        .ok_or("data geometry overflow")?;
    if data_start_sector >= total_sectors {
        return Err("data area starts beyond disk".to_owned());
    }
    let data_sectors = total_sectors - data_start_sector;
    let cluster_count = data_sectors / sectors_per_cluster;
    let max_cluster =
        u16::try_from(cluster_count + 1).map_err(|_| "too many FAT12 clusters".to_owned())?;
    Ok(Layout {
        bytes_per_sector,
        sectors_per_cluster,
        reserved_sectors,
        fat_count,
        root_entries,
        sectors_per_fat,
        total_sectors,
        root_start: FDI_HEADER_SIZE + root_start_sector * bytes_per_sector,
        root_size,
        data_start: FDI_HEADER_SIZE + data_start_sector * bytes_per_sector,
        cluster_size: bytes_per_sector * sectors_per_cluster,
        max_cluster,
    })
}

fn fat_offset(layout: &Layout) -> usize {
    FDI_HEADER_SIZE + layout.reserved_sectors * layout.bytes_per_sector
}

fn read_fat(image: &[u8], layout: &Layout) -> Result<Vec<u8>, String> {
    let size = layout.sectors_per_fat * layout.bytes_per_sector;
    let first = fat_offset(layout);
    let fat = image
        .get(first..first + size)
        .ok_or("first FAT outside FDI")?
        .to_vec();
    for index in 1..layout.fat_count {
        let offset = first + index * size;
        if image.get(offset..offset + size) != Some(fat.as_slice()) {
            return Err(format!("FAT copy {index} differs from first FAT"));
        }
    }
    Ok(fat)
}

fn fat_get(fat: &[u8], cluster: u16) -> Result<u16, String> {
    let offset = usize::from(cluster) + usize::from(cluster / 2);
    let pair = fat
        .get(offset..offset + 2)
        .ok_or_else(|| format!("FAT entry {cluster} outside FAT"))?;
    let value = u16::from_le_bytes([pair[0], pair[1]]);
    Ok(if cluster & 1 == 0 {
        value & 0x0FFF
    } else {
        value >> 4
    })
}

fn fat_set(fat: &mut [u8], cluster: u16, value: u16) -> Result<(), String> {
    let offset = usize::from(cluster) + usize::from(cluster / 2);
    let pair = fat
        .get_mut(offset..offset + 2)
        .ok_or_else(|| format!("FAT entry {cluster} outside FAT"))?;
    let value = value & 0x0FFF;
    let old = u16::from_le_bytes([pair[0], pair[1]]);
    let next = if cluster & 1 == 0 {
        (old & 0xF000) | value
    } else {
        (old & 0x000F) | (value << 4)
    };
    pair.copy_from_slice(&next.to_le_bytes());
    Ok(())
}

fn cluster_chain(fat: &[u8], layout: &Layout, start: u16) -> Result<Vec<u16>, String> {
    if start == 0 {
        return Ok(Vec::new());
    }
    if start < 2 || start > layout.max_cluster {
        return Err(format!("invalid starting cluster {start}"));
    }
    let mut result = Vec::new();
    let mut current = start;
    loop {
        if result.contains(&current) {
            return Err(format!("FAT cluster loop at {current}"));
        }
        result.push(current);
        let next = fat_get(fat, current)?;
        if next >= 0xFF8 {
            return Ok(result);
        }
        if next < 2 || next > layout.max_cluster {
            return Err(format!("invalid FAT link {current} -> {next}"));
        }
        current = next;
    }
}

fn cluster_offset(layout: &Layout, cluster: u16) -> Result<usize, String> {
    if cluster < 2 || cluster > layout.max_cluster {
        return Err(format!("cluster {cluster} outside data area"));
    }
    Ok(layout.data_start + (usize::from(cluster) - 2) * layout.cluster_size)
}

#[allow(clippy::too_many_arguments)]
fn walk_directory(
    image: &[u8],
    layout: &Layout,
    fat: &[u8],
    start_cluster: u16,
    root: bool,
    prefix: &PathKey,
    visited: &mut Vec<u16>,
    output: &mut Vec<FileEntry>,
) -> Result<(), String> {
    let mut ranges = Vec::new();
    if root {
        ranges.push((layout.root_start, layout.root_size));
    } else {
        let chain = cluster_chain(fat, layout, start_cluster)?;
        if visited.contains(&start_cluster) {
            return Err(format!("directory cluster loop at {start_cluster}"));
        }
        visited.push(start_cluster);
        for cluster in chain {
            ranges.push((cluster_offset(layout, cluster)?, layout.cluster_size));
        }
    }
    for (range_start, range_size) in ranges {
        let directory = image
            .get(range_start..range_start + range_size)
            .ok_or("directory outside FDI")?;
        for (index, item) in directory.chunks_exact(32).enumerate() {
            let marker = item[0];
            if marker == 0x00 {
                return Ok(());
            }
            if marker == 0xE5 || item[11] == 0x0F {
                continue;
            }
            let mut name = [0u8; 11];
            name.copy_from_slice(&item[..11]);
            let attr = item[11];
            if name[0] == b'.' {
                continue;
            }
            let mut key = prefix.0.clone();
            key.push(name);
            let disk_offset = range_start + index * 32;
            let start = le_u16(item, 26)?;
            let size = le_u32(item, 28)?;
            if attr & 0x10 != 0 {
                walk_directory(
                    image,
                    layout,
                    fat,
                    start,
                    false,
                    &PathKey(key),
                    visited,
                    output,
                )?;
            } else {
                output.push(FileEntry {
                    key: PathKey(key),
                    disk_offset,
                    start_cluster: start,
                    size,
                });
            }
        }
    }
    Ok(())
}

fn list_files(image: &[u8], layout: &Layout, fat: &[u8]) -> Result<Vec<FileEntry>, String> {
    let mut output = Vec::new();
    walk_directory(
        image,
        layout,
        fat,
        0,
        true,
        &PathKey(Vec::new()),
        &mut Vec::new(),
        &mut output,
    )?;
    Ok(output)
}

fn encode_short_name(component: &str) -> Result<[u8; 11], String> {
    if component == "." || component == ".." || component.is_empty() {
        return Err(format!("invalid DOS path component {component:?}"));
    }
    let (encoded, _, had_errors) = SHIFT_JIS.encode(component);
    if had_errors {
        return Err(format!("path component is not CP932: {component:?}"));
    }
    let bytes = encoded.as_ref();
    let dot = bytes.iter().rposition(|byte| *byte == b'.');
    let (base, extension) = dot.map_or((bytes, &[][..]), |position| {
        (&bytes[..position], &bytes[position + 1..])
    });
    if base.is_empty()
        || base.len() > 8
        || extension.len() > 3
        || base.contains(&b' ')
        || extension.contains(&b' ')
    {
        return Err(format!("component is not an 8.3 name: {component:?}"));
    }
    let mut result = [b' '; 11];
    for (index, byte) in base.iter().enumerate() {
        result[index] = byte.to_ascii_uppercase();
    }
    for (index, byte) in extension.iter().enumerate() {
        result[8 + index] = byte.to_ascii_uppercase();
    }
    Ok(result)
}

fn collect_replacements(root: &Path) -> Result<HashMap<PathKey, Vec<u8>>, String> {
    if !root.is_dir() {
        return Err(format!(
            "replacement root is not a directory: {}",
            root.display()
        ));
    }
    let mut files = Vec::new();
    let mut pending = vec![root.to_owned()];
    while let Some(directory) = pending.pop() {
        for item in fs::read_dir(&directory)
            .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
        {
            let item =
                item.map_err(|error| format!("cannot enumerate {}: {error}", directory.display()))?;
            let path = item.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files.sort();
    let mut result = HashMap::new();
    for file in files {
        let relative = file
            .strip_prefix(root)
            .map_err(|_| "replacement path escaped root".to_owned())?;
        let mut components = Vec::new();
        for component in relative.components() {
            let Component::Normal(value) = component else {
                return Err(format!("unsafe replacement path {}", relative.display()));
            };
            components.push(encode_short_name(&value.to_string_lossy())?);
        }
        let key = PathKey(components);
        if result
            .insert(
                key,
                fs::read(&file)
                    .map_err(|error| format!("cannot read {}: {error}", file.display()))?,
            )
            .is_some()
        {
            return Err(format!(
                "duplicate replacement path after 8.3 normalization: {}",
                file.display()
            ));
        }
    }
    Ok(result)
}

fn read_file(
    image: &[u8],
    layout: &Layout,
    fat: &[u8],
    entry: &FileEntry,
) -> Result<Vec<u8>, String> {
    let chain = cluster_chain(fat, layout, entry.start_cluster)?;
    let needed = usize::try_from(entry.size).map_err(|_| "file size overflow".to_owned())?;
    if needed == 0 && !chain.is_empty() {
        return Err("zero-size file has a cluster chain".to_owned());
    }
    if needed > 0 && chain.len() < needed.div_ceil(layout.cluster_size) {
        return Err(format!("file {:?} cluster chain is too short", entry.key));
    }
    let mut output = Vec::with_capacity(needed);
    for cluster in chain {
        let offset = cluster_offset(layout, cluster)?;
        output.extend_from_slice(
            image
                .get(offset..offset + layout.cluster_size)
                .ok_or("file cluster outside FDI")?,
        );
        if output.len() >= needed {
            break;
        }
    }
    output.truncate(needed);
    Ok(output)
}

fn clusters_for_size(size: usize, cluster_size: usize) -> usize {
    size.div_ceil(cluster_size)
}

fn final_free_clusters(free_clusters: usize, deltas: &[isize]) -> Result<usize, String> {
    let net_delta = deltas.iter().sum::<isize>();
    let final_free = isize::try_from(free_clusters)
        .map_err(|_| "free cluster count overflow".to_owned())?
        - net_delta;
    if final_free < 0 {
        return Err(format!(
            "not enough free clusters for final image: need {} additional clusters, have {free_clusters}, short {}",
            net_delta.max(0),
            -final_free
        ));
    }
    usize::try_from(final_free).map_err(|_| "final free cluster count overflow".to_owned())
}

fn replace_file(
    image: &mut [u8],
    layout: &Layout,
    fat: &mut [u8],
    entry: &FileEntry,
    data: &[u8],
) -> Result<(), String> {
    if u32::try_from(data.len()).is_err() {
        return Err(format!("replacement file {:?} is too large", entry.key));
    }
    let old_chain = cluster_chain(fat, layout, entry.start_cluster)?;
    let needed = clusters_for_size(data.len(), layout.cluster_size);
    let mut chain = old_chain;
    if needed < chain.len() {
        for cluster in chain.drain(needed..) {
            fat_set(fat, cluster, 0)?;
        }
    }
    while chain.len() < needed {
        let cluster = (2..=layout.max_cluster)
            .find(|candidate| fat_get(fat, *candidate).is_ok_and(|value| value == 0))
            .ok_or_else(|| format!("not enough free clusters for {:?}", entry.key))?;
        fat_set(fat, cluster, FAT_EOC)?;
        chain.push(cluster);
    }
    for (index, cluster) in chain.iter().copied().enumerate() {
        fat_set(
            fat,
            cluster,
            if index + 1 < chain.len() {
                chain[index + 1]
            } else {
                FAT_EOC
            },
        )?;
        let offset = cluster_offset(layout, cluster)?;
        let target = image
            .get_mut(offset..offset + layout.cluster_size)
            .ok_or("replacement cluster outside FDI")?;
        target.fill(0);
        let data_start = index * layout.cluster_size;
        let data_end = (data_start + layout.cluster_size).min(data.len());
        if data_start < data_end {
            target[..data_end - data_start].copy_from_slice(&data[data_start..data_end]);
        }
    }
    let start_cluster = chain.first().copied().unwrap_or(0);
    image[entry.disk_offset + 26..entry.disk_offset + 28]
        .copy_from_slice(&start_cluster.to_le_bytes());
    image[entry.disk_offset + 28..entry.disk_offset + 32]
        .copy_from_slice(&(data.len() as u32).to_le_bytes());
    Ok(())
}

fn snapshot(
    image: &[u8],
    layout: &Layout,
    fat: &[u8],
) -> Result<HashMap<PathKey, Vec<u8>>, String> {
    let mut result = HashMap::new();
    for entry in list_files(image, layout, fat)? {
        result.insert(entry.key.clone(), read_file(image, layout, fat, &entry)?);
    }
    Ok(result)
}

fn write_fats(image: &mut [u8], layout: &Layout, fat: &[u8]) -> Result<(), String> {
    let size = layout.sectors_per_fat * layout.bytes_per_sector;
    let first = fat_offset(layout);
    for index in 0..layout.fat_count {
        let offset = first + index * size;
        image
            .get_mut(offset..offset + size)
            .ok_or("FAT outside output")?
            .copy_from_slice(fat);
    }
    Ok(())
}

fn temp_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("archive.FDI");
    path.with_file_name(format!(".{name}.partial-{}", std::process::id()))
}

fn print_help() {
    println!("fdi_repack - replace existing files in a PC-98 FAT12 FDI image");
    println!("Usage: fdi_repack <input.FDI> --replacements <root-tree> --output <output.FDI>");
    println!(
        "Replacement paths are relative to the disk root and must already exist in the image."
    );
    println!(
        "The 0x1000-byte FDI header is preserved; both FAT copies and all file chains are verified."
    );
}

fn run() -> Result<(), String> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return Ok(());
    }
    let input_path = PathBuf::from(&args[0]);
    let mut replacement_root = None;
    let mut output_path = None;
    let mut index = 1usize;
    while index < args.len() {
        match args[index].to_string_lossy().as_ref() {
            "--replacements" => {
                replacement_root = Some(PathBuf::from(
                    args.get(index + 1)
                        .ok_or("--replacements requires a path")?,
                ));
                index += 2;
            }
            "--output" => {
                output_path = Some(PathBuf::from(
                    args.get(index + 1).ok_or("--output requires a path")?,
                ));
                index += 2;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    let replacement_root = replacement_root.ok_or("--replacements is required")?;
    let output_path = output_path.ok_or("--output is required")?;
    if output_path.exists() {
        return Err(format!("refusing to overwrite {}", output_path.display()));
    }
    let original = fs::read(&input_path)
        .map_err(|error| format!("cannot read {}: {error}", input_path.display()))?;
    let layout = parse_layout(&original)?;
    let original_fat = read_fat(&original, &layout)?;
    let before = snapshot(&original, &layout, &original_fat)?;
    let replacements = collect_replacements(&replacement_root)?;
    let mut image = original.clone();
    let mut fat = original_fat.clone();
    let entries = list_files(&image, &layout, &fat)?;
    let mut by_key = HashMap::new();
    for entry in entries {
        if by_key.insert(entry.key.clone(), entry).is_some() {
            return Err("duplicate file path in FAT directory tree".to_owned());
        }
    }
    let free_clusters_before = (2..=layout.max_cluster)
        .filter(|cluster| fat_get(&fat, *cluster).is_ok_and(|value| value == 0))
        .count();
    let mut plan = Vec::with_capacity(replacements.len());
    for (key, data) in &replacements {
        let entry = by_key
            .get(key)
            .ok_or_else(|| format!("replacement path does not exist in FDI: {key:?}"))?;
        let old_clusters = cluster_chain(&fat, &layout, entry.start_cluster)?.len();
        let new_clusters = clusters_for_size(data.len(), layout.cluster_size);
        let delta = isize::try_from(new_clusters)
            .map_err(|_| "replacement cluster count overflow".to_owned())?
            - isize::try_from(old_clusters)
                .map_err(|_| "original cluster count overflow".to_owned())?;
        plan.push((delta, key, data, entry));
    }
    let deltas = plan.iter().map(|item| item.0).collect::<Vec<_>>();
    let free_clusters_after = final_free_clusters(free_clusters_before, &deltas)?;
    plan.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(right.1)));
    for (_, _, data, entry) in plan {
        replace_file(&mut image, &layout, &mut fat, entry, data)?;
    }
    write_fats(&mut image, &layout, &fat)?;
    if image[..FDI_HEADER_SIZE] != original[..FDI_HEADER_SIZE] {
        return Err("FDI header changed during repack".to_owned());
    }
    let output_fat = read_fat(&image, &layout)?;
    if output_fat != fat {
        return Err("written FAT cannot be read back".to_owned());
    }
    let after = snapshot(&image, &layout, &output_fat)?;
    if before.len() != after.len() {
        return Err("file count changed during repack".to_owned());
    }
    for (key, old_data) in &before {
        let new_data = after
            .get(key)
            .ok_or_else(|| format!("file disappeared after repack: {key:?}"))?;
        if let Some(expected) = replacements.get(key) {
            if new_data != expected {
                return Err(format!("replacement verification failed for {key:?}"));
            }
        } else if new_data != old_data {
            return Err(format!("unreplaced file changed: {key:?}"));
        }
    }
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    let temporary = temp_path(&output_path);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| format!("cannot create {}: {error}", temporary.display()))?;
    if let Err(error) = file.write_all(&image) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("cannot write {}: {error}", temporary.display()));
    }
    drop(file);
    fs::rename(&temporary, &output_path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("cannot finalize {}: {error}", output_path.display())
    })?;
    println!("[repack] input={}", input_path.display());
    println!("[repack] files_replaced={}", replacements.len());
    println!("[repack] output={}", output_path.display());
    println!(
        "[repack] free_clusters_before={free_clusters_before}, free_clusters_after={free_clusters_after}"
    );
    println!(
        "[repack] geometry={} bytes/sector, {} sectors/cluster, {} FATs, {} root entries, {} total sectors",
        layout.bytes_per_sector,
        layout.sectors_per_cluster,
        layout.fat_count,
        layout.root_entries,
        layout.total_sectors
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("[repack] error: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_cluster_capacity_is_accepted() {
        assert_eq!(final_free_clusters(9, &[4, -2, 7]).unwrap(), 0);
    }

    #[test]
    fn final_cluster_shortage_is_reported() {
        let error = final_free_clusters(9, &[4, -2, 8]).unwrap_err();
        assert!(error.contains("short 1"));
    }

    #[test]
    fn cluster_rounding_matches_fat_allocation() {
        assert_eq!(clusters_for_size(0, 1024), 0);
        assert_eq!(clusters_for_size(1024, 1024), 1);
        assert_eq!(clusters_for_size(1025, 1024), 2);
    }
}
