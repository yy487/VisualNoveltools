use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::min;
use std::fs;
use std::path::{Component, Path, PathBuf};

const D88_HEADER_SIZE: usize = 0x2B0;
const D88_TRACK_SLOTS: usize = 164;
const D88_SECTOR_HEADER_SIZE: usize = 16;
const LOGICAL_SECTOR_SIZE: usize = 0x400;
const LOGICAL_TRACK_SIZE: u64 = 5 * LOGICAL_SECTOR_SIZE as u64;
const CATALOG_TRACK_INDEX: usize = 79;
const CATALOG_SECTOR_FIRST: u8 = 3;
const CATALOG_SECTOR_LAST: u8 = 5;
const MKDS_HEADER_SIZE: usize = 9;
const MKDS_ENTRY_SIZE: usize = 9;
const MANAGED_MARKER: &str = ".foxy_d88_managed";
const CONTAINER_FORMAT: &str = "concatenated D88 containing FOXY MKDS_3 volumes";
const CATALOG_FORMAT: &str = "FOXY MKDS_3 catalog";

pub type ArchiveResult<T> = Result<T, String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnpackSummary {
    pub volume_count: usize,
    pub entry_count: usize,
    pub compressed_entry_count: usize,
    pub output: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackSummary {
    pub volume_count: usize,
    pub changed_entry_count: usize,
    pub output_size: usize,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
struct Sector {
    c: u8,
    h: u8,
    r: u8,
    n: u8,
    density: u8,
    deleted: u8,
    status: u8,
    data_offset: usize,
    data_size: usize,
}

#[derive(Debug, Clone)]
struct Track {
    table_index: usize,
    relative_offset: u32,
    sectors: Vec<Sector>,
}

#[derive(Debug, Clone)]
struct D88Image {
    index: usize,
    name: String,
    container_offset: usize,
    image_size: usize,
    write_protect: u8,
    media_type: u8,
    tracks: Vec<Option<Track>>,
}

#[derive(Debug, Clone)]
struct CatalogEntry {
    index: usize,
    raw_name: String,
    display_name: String,
    packed_key: [u8; 7],
    stored_size: usize,
    compressed: bool,
    logical_offset: u64,
}

#[derive(Debug, Clone)]
struct Catalog {
    disk_id: u8,
    layout_flag: u8,
    entry_count: usize,
    initial_logical_offset: u64,
    entries: Vec<CatalogEntry>,
}

#[derive(Debug)]
struct DecodedStream {
    bytes: Vec<u8>,
    consumed_bits: usize,
    total_bits: usize,
}

#[derive(Debug, Clone, Copy)]
struct HuffmanCode {
    length: usize,
    bits: u16,
    symbol: u8,
}

struct BitReader<'a> {
    data: &'a [u8],
    bit_offset: usize,
}

struct BitWriter {
    bytes: Vec<u8>,
    bit_offset: usize,
}

#[derive(Debug, Clone)]
struct EncodeNode {
    frequency: u64,
    minimum_symbol: u8,
    left: Option<usize>,
    right: Option<usize>,
    symbol: Option<u8>,
}

#[derive(Serialize)]
struct ContainerManifest {
    format: &'static str,
    tool: &'static str,
    tool_version: &'static str,
    source_name: String,
    source_size: usize,
    source_sha256: String,
    volume_count: usize,
    split_rejoined_exact: bool,
    volumes: Vec<VolumeSummary>,
}

#[derive(Serialize)]
struct VolumeSummary {
    index: usize,
    directory: String,
    image_name: String,
    image_path: String,
    container_offset: usize,
    image_size: usize,
    image_sha256: String,
    write_protect_byte: u8,
    write_protected: bool,
    media_type: u8,
    catalog_entries: usize,
    compressed_entries: usize,
    stored_payload_size: usize,
    decoded_payload_size: usize,
    catalog_path: String,
}

#[derive(Serialize)]
struct CatalogManifest {
    format: &'static str,
    image_index: usize,
    image_name: String,
    disk_id: u8,
    layout_flag: u8,
    catalog_track_index: usize,
    catalog_sector_ids: [u8; 3],
    entry_count: usize,
    initial_logical_offset: u64,
    entries: Vec<EntryManifest>,
}

#[derive(Serialize)]
struct EntryManifest {
    index: usize,
    raw_name_6_3: String,
    name: String,
    packed_key_hex: String,
    stored_size: usize,
    compressed: bool,
    logical_offset: u64,
    start_track_index: u64,
    start_cylinder: u8,
    start_head: u8,
    start_sector_id: u8,
    start_sector_n: u8,
    start_sector_density: u8,
    start_sector_deleted: u8,
    start_sector_status: u8,
    start_sector_offset: usize,
    stored_path: String,
    stored_sha256: String,
    decoded_path: String,
    decoded_size: usize,
    decoded_sha256: String,
    compression_consumed_bits: Option<usize>,
    compression_trailing_bits: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct PackContainerManifest {
    format: String,
    volume_count: usize,
    volumes: Vec<PackVolumeSummary>,
}

#[derive(Debug, Deserialize)]
struct PackVolumeSummary {
    index: usize,
    directory: String,
    image_name: String,
    image_path: String,
    image_size: usize,
    image_sha256: String,
    catalog_path: String,
}

#[derive(Debug, Deserialize)]
struct PackCatalogManifest {
    format: String,
    image_index: usize,
    image_name: String,
    entry_count: usize,
    initial_logical_offset: u64,
    entries: Vec<PackEntryManifest>,
}

#[derive(Debug, Deserialize)]
struct PackEntryManifest {
    index: usize,
    name: String,
    stored_size: usize,
    compressed: bool,
    logical_offset: u64,
    stored_path: String,
    stored_sha256: String,
    decoded_path: String,
    decoded_sha256: String,
}

pub fn suggested_unpack_output(input: &Path) -> PathBuf {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("d88");
    parent.join(format!("{stem}_unpacked"))
}

pub fn suggested_pack_output(input: &Path) -> PathBuf {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let stem = input
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("foxy_unpacked")
        .trim_end_matches("_unpacked");
    parent.join(format!("{stem}_repacked.d88"))
}

pub fn unpack_d88(input: &Path, output: &Path, overwrite: bool) -> ArchiveResult<UnpackSummary> {
    if !input.is_file() {
        return Err(format!("input D88 is not a file: {}", input.display()));
    }
    refuse_output_containing_input(input, output)?;
    let source =
        fs::read(input).map_err(|error| format!("failed to read {}: {error}", input.display()))?;
    let images = parse_d88_images(&source)?;
    if images.is_empty() {
        return Err("no D88 images were found".to_string());
    }

    prepare_output(output, overwrite)?;
    fs::write(output.join(MANAGED_MARKER), b"foxy_disk_tool\n").map_err(|error| {
        format!(
            "failed to mark managed output {}: {error}",
            output.display()
        )
    })?;
    let images_dir = output.join("images");
    let volumes_dir = output.join("volumes");
    fs::create_dir_all(&images_dir)
        .map_err(|error| format!("failed to create {}: {error}", images_dir.display()))?;
    fs::create_dir_all(&volumes_dir)
        .map_err(|error| format!("failed to create {}: {error}", volumes_dir.display()))?;

    let mut summaries = Vec::with_capacity(images.len());
    let mut rejoined = Vec::with_capacity(source.len());
    let mut total_entries = 0usize;
    let mut total_compressed = 0usize;

    for image in &images {
        let catalog = parse_catalog(&source, image)?;
        let volume_label = format!("{:02}_{}", image.index, sanitize_component(&image.name));
        let volume_dir = volumes_dir.join(&volume_label);
        let stored_dir = volume_dir.join("files_stored");
        let decoded_dir = volume_dir.join("files_decoded");
        fs::create_dir_all(&stored_dir)
            .map_err(|error| format!("failed to create {}: {error}", stored_dir.display()))?;
        fs::create_dir_all(&decoded_dir)
            .map_err(|error| format!("failed to create {}: {error}", decoded_dir.display()))?;

        let image_filename = format!("{volume_label}.d88");
        let image_path = images_dir.join(&image_filename);
        let image_bytes =
            &source[image.container_offset..image.container_offset + image.image_size];
        fs::write(&image_path, image_bytes)
            .map_err(|error| format!("failed to write {}: {error}", image_path.display()))?;
        rejoined.extend_from_slice(image_bytes);

        let mut entry_manifests = Vec::with_capacity(catalog.entries.len());
        for entry in &catalog.entries {
            let stored_name = format!(
                "{:03}_{}",
                entry.index,
                sanitize_filename(&entry.display_name)
            );
            let stored_path = stored_dir.join(&stored_name);
            let stored =
                read_logical_file(&source, image, entry.logical_offset, entry.stored_size)?;
            fs::write(&stored_path, &stored)
                .map_err(|error| format!("failed to write {}: {error}", stored_path.display()))?;

            let decoded = if entry.compressed {
                decode_mkds_compressed(&stored).map_err(|error| {
                    format!(
                        "{} entry {} ({}) compression error: {error}",
                        image.name, entry.index, entry.display_name
                    )
                })?
            } else {
                DecodedStream {
                    bytes: stored.clone(),
                    consumed_bits: 0,
                    total_bits: 0,
                }
            };
            let decoded_path = decoded_dir.join(&stored_name);
            fs::write(&decoded_path, &decoded.bytes)
                .map_err(|error| format!("failed to write {}: {error}", decoded_path.display()))?;

            let logical = logical_position(entry.logical_offset);
            let start_track = image
                .tracks
                .get(logical.track_index as usize)
                .and_then(Option::as_ref)
                .ok_or_else(|| {
                    format!(
                        "{} entry {} starts on missing track {}",
                        image.name, entry.index, logical.track_index
                    )
                })?;
            let start_sector = start_track
                .sectors
                .iter()
                .find(|sector| sector.r == logical.sector_id)
                .ok_or_else(|| {
                    format!(
                        "{} entry {} starts on missing sector {} of track {}",
                        image.name, entry.index, logical.sector_id, logical.track_index
                    )
                })?;

            entry_manifests.push(EntryManifest {
                index: entry.index,
                raw_name_6_3: entry.raw_name.clone(),
                name: entry.display_name.clone(),
                packed_key_hex: hex_bytes(&entry.packed_key),
                stored_size: entry.stored_size,
                compressed: entry.compressed,
                logical_offset: entry.logical_offset,
                start_track_index: logical.track_index,
                start_cylinder: start_sector.c,
                start_head: start_sector.h,
                start_sector_id: logical.sector_id,
                start_sector_n: start_sector.n,
                start_sector_density: start_sector.density,
                start_sector_deleted: start_sector.deleted,
                start_sector_status: start_sector.status,
                start_sector_offset: logical.sector_offset,
                stored_path: slash_path(&Path::new("files_stored").join(&stored_name)),
                stored_sha256: sha256_hex(&stored),
                decoded_path: slash_path(&Path::new("files_decoded").join(&stored_name)),
                decoded_size: decoded.bytes.len(),
                decoded_sha256: sha256_hex(&decoded.bytes),
                compression_consumed_bits: entry.compressed.then_some(decoded.consumed_bits),
                compression_trailing_bits: entry
                    .compressed
                    .then_some(decoded.total_bits - decoded.consumed_bits),
            });
        }

        let compressed_entries = catalog
            .entries
            .iter()
            .filter(|entry| entry.compressed)
            .count();
        total_entries += catalog.entry_count;
        total_compressed += compressed_entries;
        let catalog_manifest = CatalogManifest {
            format: CATALOG_FORMAT,
            image_index: image.index,
            image_name: image.name.clone(),
            disk_id: catalog.disk_id,
            layout_flag: catalog.layout_flag,
            catalog_track_index: CATALOG_TRACK_INDEX,
            catalog_sector_ids: [3, 4, 5],
            entry_count: catalog.entry_count,
            initial_logical_offset: catalog.initial_logical_offset,
            entries: entry_manifests,
        };
        let catalog_path = volume_dir.join("catalog.json");
        write_json(&catalog_path, &catalog_manifest)?;

        summaries.push(VolumeSummary {
            index: image.index,
            directory: slash_path(&Path::new("volumes").join(&volume_label)),
            image_name: image.name.clone(),
            image_path: slash_path(&Path::new("images").join(&image_filename)),
            container_offset: image.container_offset,
            image_size: image.image_size,
            image_sha256: sha256_hex(image_bytes),
            write_protect_byte: image.write_protect,
            write_protected: image.write_protect != 0,
            media_type: image.media_type,
            catalog_entries: catalog.entry_count,
            compressed_entries,
            stored_payload_size: catalog.entries.iter().map(|entry| entry.stored_size).sum(),
            decoded_payload_size: catalog_manifest
                .entries
                .iter()
                .map(|entry| entry.decoded_size)
                .sum(),
            catalog_path: slash_path(
                &Path::new("volumes")
                    .join(&volume_label)
                    .join("catalog.json"),
            ),
        });
    }

    let split_rejoined_exact = rejoined == source;
    if !split_rejoined_exact {
        return Err("internal verification failed: split images do not rejoin exactly".to_string());
    }
    let manifest = ContainerManifest {
        format: CONTAINER_FORMAT,
        tool: "foxy_disk_tool",
        tool_version: env!("CARGO_PKG_VERSION"),
        source_name: input
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| "input.d88".to_string()),
        source_size: source.len(),
        source_sha256: sha256_hex(&source),
        volume_count: images.len(),
        split_rejoined_exact,
        volumes: summaries,
    };
    write_json(&output.join("manifest.json"), &manifest)?;
    fs::remove_file(output.join(MANAGED_MARKER)).map_err(|error| {
        format!(
            "failed to clear managed-output marker in {}: {error}",
            output.display()
        )
    })?;

    Ok(UnpackSummary {
        volume_count: images.len(),
        entry_count: total_entries,
        compressed_entry_count: total_compressed,
        output: output.to_path_buf(),
    })
}

pub fn pack_d88(input: &Path, output: &Path, overwrite: bool) -> ArchiveResult<PackSummary> {
    if !input.is_dir() {
        return Err(format!(
            "pack input is not a directory: {}",
            input.display()
        ));
    }
    refuse_output_inside_input(input, output)?;
    let manifest_path = input.join("manifest.json");
    let manifest: PackContainerManifest = read_json(&manifest_path)?;
    if manifest.format != CONTAINER_FORMAT {
        return Err(format!("unsupported workspace format: {}", manifest.format));
    }
    if manifest.volume_count != manifest.volumes.len() || manifest.volumes.is_empty() {
        return Err("container manifest has an invalid volume count".to_string());
    }
    let mut volumes = manifest.volumes;
    volumes.sort_by_key(|volume| volume.index);
    for (expected, volume) in volumes.iter().enumerate() {
        if volume.index != expected {
            return Err(format!("missing or duplicate volume index {expected}"));
        }
    }

    let mut container = Vec::new();
    let mut changed_entries = 0usize;
    for volume in &volumes {
        let volume_dir = input.join(safe_manifest_path(&volume.directory)?);
        let image_path = input.join(safe_manifest_path(&volume.image_path)?);
        let catalog_path = input.join(safe_manifest_path(&volume.catalog_path)?);
        if catalog_path.parent() != Some(volume_dir.as_path()) {
            return Err(format!(
                "catalog path does not match volume directory: {}",
                volume.catalog_path
            ));
        }
        let catalog_manifest: PackCatalogManifest = read_json(&catalog_path)?;
        validate_pack_catalog(volume, &catalog_manifest)?;
        let mut image_bytes = fs::read(&image_path)
            .map_err(|error| format!("failed to read {}: {error}", image_path.display()))?;
        if image_bytes.len() != volume.image_size || sha256_hex(&image_bytes) != volume.image_sha256
        {
            return Err(format!(
                "original split image changed: {}",
                image_path.display()
            ));
        }
        let parsed = parse_d88_images(&image_bytes)?;
        if parsed.len() != 1 {
            return Err(format!(
                "split image contains {} D88 images: {}",
                parsed.len(),
                image_path.display()
            ));
        }
        let image = &parsed[0];
        if image.name != volume.image_name {
            return Err(format!("image name changed in {}", image_path.display()));
        }
        let live_catalog = parse_catalog(&image_bytes, image)?;
        if live_catalog.entry_count != catalog_manifest.entry_count
            || live_catalog.initial_logical_offset != catalog_manifest.initial_logical_offset
        {
            return Err(format!(
                "catalog metadata changed in {}",
                catalog_path.display()
            ));
        }

        let mut stored_payloads = Vec::with_capacity(catalog_manifest.entries.len());
        let mut manifest_offset = catalog_manifest.initial_logical_offset;
        let mut packed_offset = catalog_manifest.initial_logical_offset;
        for (index, entry) in catalog_manifest.entries.iter().enumerate() {
            if entry.index != index || entry.logical_offset != manifest_offset {
                return Err(format!(
                    "catalog entry order/offset changed for {} index {index}",
                    volume.image_name
                ));
            }
            manifest_offset = manifest_offset
                .checked_add(entry.stored_size as u64)
                .ok_or_else(|| "manifest logical offset overflow".to_string())?;
            let decoded_path = volume_dir.join(safe_manifest_path(&entry.decoded_path)?);
            let decoded = fs::read(&decoded_path)
                .map_err(|error| format!("failed to read {}: {error}", decoded_path.display()))?;
            let decoded_hash = sha256_hex(&decoded);
            let stored = if decoded_hash == entry.decoded_sha256 {
                let stored_path = volume_dir.join(safe_manifest_path(&entry.stored_path)?);
                let original = fs::read(&stored_path).map_err(|error| {
                    format!("failed to read {}: {error}", stored_path.display())
                })?;
                if original.len() != entry.stored_size
                    || sha256_hex(&original) != entry.stored_sha256
                {
                    return Err(format!("stored source changed: {}", stored_path.display()));
                }
                original
            } else {
                changed_entries += 1;
                if entry.compressed {
                    let encoded = encode_mkds_compressed(&decoded)?;
                    let verified = decode_mkds_compressed(&encoded)?;
                    if verified.bytes != decoded {
                        return Err(format!(
                            "compression verification failed for {}",
                            decoded_path.display()
                        ));
                    }
                    encoded
                } else {
                    decoded
                }
            };
            if stored.len() > 0x7FFF {
                return Err(format!(
                    "{} entry {} ({}) is {} bytes; catalog limit is 32767",
                    volume.image_name,
                    index,
                    entry.name,
                    stored.len()
                ));
            }
            packed_offset = packed_offset
                .checked_add(stored.len() as u64)
                .ok_or_else(|| "packed logical offset overflow".to_string())?;
            stored_payloads.push(stored);
        }
        let catalog_tail_size = MKDS_ENTRY_SIZE * (catalog_manifest.entry_count + 1);
        let catalog_start =
            (CATALOG_TRACK_INDEX as u64 + 1) * LOGICAL_TRACK_SIZE - catalog_tail_size as u64;
        if packed_offset > catalog_start {
            return Err(format!(
                "{} payloads end at {packed_offset:#x}, crossing catalog start {catalog_start:#x}",
                volume.image_name
            ));
        }

        let sizes: Vec<_> = catalog_manifest
            .entries
            .iter()
            .zip(&stored_payloads)
            .map(|(entry, stored)| (stored.len(), entry.compressed))
            .collect();
        let mut write_offset = catalog_manifest.initial_logical_offset;
        for payload in &stored_payloads {
            write_logical_file(&mut image_bytes, image, write_offset, payload)?;
            write_offset += payload.len() as u64;
        }
        update_catalog_sizes(&mut image_bytes, image, &sizes)?;
        let verified_catalog = parse_catalog(&image_bytes, image)?;
        if verified_catalog.entries.len() != sizes.len()
            || verified_catalog
                .entries
                .iter()
                .zip(&sizes)
                .any(|(entry, (size, compressed))| {
                    entry.stored_size != *size || entry.compressed != *compressed
                })
        {
            return Err(format!(
                "rebuilt catalog verification failed for {}",
                volume.image_name
            ));
        }
        container.extend_from_slice(&image_bytes);
    }

    write_output_file(output, &container, overwrite)?;
    let reparsed = parse_d88_images(&container)?;
    if reparsed.len() != volumes.len() {
        return Err("final D88 verification found the wrong volume count".to_string());
    }
    Ok(PackSummary {
        volume_count: reparsed.len(),
        changed_entry_count: changed_entries,
        output_size: container.len(),
        output: output.to_path_buf(),
    })
}

fn validate_pack_catalog(
    volume: &PackVolumeSummary,
    catalog: &PackCatalogManifest,
) -> ArchiveResult<()> {
    if catalog.format != CATALOG_FORMAT
        || catalog.image_index != volume.index
        || catalog.image_name != volume.image_name
        || catalog.entry_count != catalog.entries.len()
    {
        return Err(format!(
            "invalid catalog manifest for {}",
            volume.image_name
        ));
    }
    Ok(())
}

fn parse_d88_images(data: &[u8]) -> ArchiveResult<Vec<D88Image>> {
    let mut images = Vec::new();
    let mut cursor = 0usize;
    while cursor < data.len() {
        require_range(data, cursor, D88_HEADER_SIZE, "D88 header")?;
        let image_size = read_u32_le(data, cursor + 0x1C)? as usize;
        if image_size < D88_HEADER_SIZE {
            return Err(format!(
                "invalid D88 image size {image_size:#x} at container offset {cursor:#x}"
            ));
        }
        require_range(data, cursor, image_size, "D88 image")?;

        let name_slice = &data[cursor..cursor + 17];
        let name_end = name_slice
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(name_slice.len());
        let name = String::from_utf8_lossy(&name_slice[..name_end]).to_string();
        let mut tracks = vec![None; D88_TRACK_SLOTS];
        for (track_index, slot) in tracks.iter_mut().enumerate() {
            let relative_offset = read_u32_le(data, cursor + 0x20 + track_index * 4)?;
            if relative_offset == 0 {
                continue;
            }
            if relative_offset as usize >= image_size {
                return Err(format!(
                    "{} track {} points outside its D88 image: {relative_offset:#x}",
                    name, track_index
                ));
            }
            let mut sector_position = cursor + relative_offset as usize;
            require_range(
                data,
                sector_position,
                D88_SECTOR_HEADER_SIZE,
                "sector header",
            )?;
            let sector_count = read_u16_le(data, sector_position + 4)? as usize;
            if sector_count == 0 {
                return Err(format!("{} track {} has zero sectors", name, track_index));
            }
            let mut sectors = Vec::with_capacity(sector_count);
            for _ in 0..sector_count {
                require_range(
                    data,
                    sector_position,
                    D88_SECTOR_HEADER_SIZE,
                    "sector header",
                )?;
                let data_size = read_u16_le(data, sector_position + 14)? as usize;
                let declared_sector_count = read_u16_le(data, sector_position + 4)? as usize;
                if declared_sector_count != sector_count {
                    return Err(format!(
                        "{} track {} has inconsistent sector count {} (expected {})",
                        name, track_index, declared_sector_count, sector_count
                    ));
                }
                require_range(
                    data,
                    sector_position + D88_SECTOR_HEADER_SIZE,
                    data_size,
                    "sector data",
                )?;
                let sector_end = sector_position + D88_SECTOR_HEADER_SIZE + data_size;
                if sector_end > cursor + image_size {
                    return Err(format!(
                        "{} track {} sector data crosses the D88 image boundary",
                        name, track_index
                    ));
                }
                sectors.push(Sector {
                    c: data[sector_position],
                    h: data[sector_position + 1],
                    r: data[sector_position + 2],
                    n: data[sector_position + 3],
                    density: data[sector_position + 6],
                    deleted: data[sector_position + 7],
                    status: data[sector_position + 8],
                    data_offset: sector_position + D88_SECTOR_HEADER_SIZE,
                    data_size,
                });
                sector_position += D88_SECTOR_HEADER_SIZE + data_size;
            }
            *slot = Some(Track {
                table_index: track_index,
                relative_offset,
                sectors,
            });
        }

        images.push(D88Image {
            index: images.len(),
            name: if name.is_empty() {
                format!("DISK_{}", images.len())
            } else {
                name
            },
            container_offset: cursor,
            image_size,
            write_protect: data[cursor + 0x1A],
            media_type: data[cursor + 0x1B],
            tracks,
        });
        cursor += image_size;
    }
    if cursor != data.len() {
        return Err(format!(
            "D88 image sizes stop at {cursor:#x}, but the container is {:#x} bytes",
            data.len()
        ));
    }
    Ok(images)
}

fn parse_catalog(data: &[u8], image: &D88Image) -> ArchiveResult<Catalog> {
    let track = image
        .tracks
        .get(CATALOG_TRACK_INDEX)
        .and_then(Option::as_ref)
        .ok_or_else(|| {
            format!(
                "{} is missing catalog track {}",
                image.name, CATALOG_TRACK_INDEX
            )
        })?;
    let mut catalog_block = Vec::with_capacity(3 * LOGICAL_SECTOR_SIZE);
    for sector_id in CATALOG_SECTOR_FIRST..=CATALOG_SECTOR_LAST {
        let sector = find_sector(track, sector_id)?;
        if sector.data_size != LOGICAL_SECTOR_SIZE {
            return Err(format!(
                "{} catalog track {} sector {} is {} bytes, expected {}",
                image.name, CATALOG_TRACK_INDEX, sector_id, sector.data_size, LOGICAL_SECTOR_SIZE
            ));
        }
        catalog_block
            .extend_from_slice(&data[sector.data_offset..sector.data_offset + sector.data_size]);
    }

    if catalog_block.len() < MKDS_HEADER_SIZE {
        return Err(format!("{} catalog block is too short", image.name));
    }
    let end = catalog_block.len();
    if &catalog_block[end - 6..] != b"3_SDKM" {
        return Err(format!(
            "{} catalog marker is missing at track {} sectors {}-{}",
            image.name, CATALOG_TRACK_INDEX, CATALOG_SECTOR_FIRST, CATALOG_SECTOR_LAST
        ));
    }
    let disk_id = catalog_block[end - 7];
    let layout_flag = catalog_block[end - 8];
    let entry_count = catalog_block[end - 9] as usize;
    let reverse_length = MKDS_ENTRY_SIZE
        .checked_mul(entry_count + 1)
        .ok_or_else(|| "catalog size overflow".to_string())?;
    if reverse_length > catalog_block.len() {
        return Err(format!(
            "{} catalog declares {} entries, exceeding its catalog block",
            image.name, entry_count
        ));
    }
    let decoded: Vec<u8> = catalog_block[end - reverse_length..end]
        .iter()
        .rev()
        .copied()
        .collect();
    if &decoded[..6] != b"MKDS_3" {
        return Err(format!("{} reversed catalog header is invalid", image.name));
    }

    let initial_logical_offset = if layout_flag == 0 { 0x1400 } else { 0 };
    let mut logical_offset = initial_logical_offset;
    let mut entries = Vec::with_capacity(entry_count);
    for index in 0..entry_count {
        let entry_offset = MKDS_HEADER_SIZE + index * MKDS_ENTRY_SIZE;
        let packed_key: [u8; 7] = decoded[entry_offset..entry_offset + 7]
            .try_into()
            .map_err(|_| "internal catalog key conversion failed".to_string())?;
        let raw_name = decode_packed_name(&packed_key);
        let display_name = display_name_6_3(&raw_name);
        let size_high = decoded[entry_offset + 7];
        let stored_size = (((size_high & 0x7F) as usize) << 8) | decoded[entry_offset + 8] as usize;
        let compressed = size_high & 0x80 != 0;
        entries.push(CatalogEntry {
            index,
            raw_name,
            display_name,
            packed_key,
            stored_size,
            compressed,
            logical_offset,
        });
        logical_offset = logical_offset
            .checked_add(stored_size as u64)
            .ok_or_else(|| "logical file offset overflow".to_string())?;
    }

    Ok(Catalog {
        disk_id,
        layout_flag,
        entry_count,
        initial_logical_offset,
        entries,
    })
}

fn read_logical_file(
    data: &[u8],
    image: &D88Image,
    offset: u64,
    size: usize,
) -> ArchiveResult<Vec<u8>> {
    let mut output = Vec::with_capacity(size);
    let mut logical_offset = offset;
    let mut remaining = size;
    while remaining > 0 {
        let position = logical_position(logical_offset);
        let track = image
            .tracks
            .get(position.track_index as usize)
            .and_then(Option::as_ref)
            .ok_or_else(|| {
                format!(
                    "{} logical offset {logical_offset:#x} maps to missing track {}",
                    image.name, position.track_index
                )
            })?;
        let sector = find_sector(track, position.sector_id)?;
        if sector.data_size < LOGICAL_SECTOR_SIZE {
            return Err(format!(
                "{} track {} sector {} is {} bytes, too short for logical 1 KiB access",
                image.name, position.track_index, position.sector_id, sector.data_size
            ));
        }
        let available = LOGICAL_SECTOR_SIZE - position.sector_offset;
        let take = min(remaining, available);
        let source_start = sector.data_offset + position.sector_offset;
        output.extend_from_slice(&data[source_start..source_start + take]);
        remaining -= take;
        logical_offset += take as u64;
    }
    Ok(output)
}

fn write_logical_file(
    data: &mut [u8],
    image: &D88Image,
    offset: u64,
    bytes: &[u8],
) -> ArchiveResult<()> {
    let mut source_offset = 0usize;
    let mut logical_offset = offset;
    while source_offset < bytes.len() {
        let position = logical_position(logical_offset);
        let track = image
            .tracks
            .get(position.track_index as usize)
            .and_then(Option::as_ref)
            .ok_or_else(|| {
                format!(
                    "logical write maps to missing track {}",
                    position.track_index
                )
            })?;
        let sector = find_sector(track, position.sector_id)?;
        if sector.data_size < LOGICAL_SECTOR_SIZE {
            return Err(format!(
                "logical write sector {} is too short",
                position.sector_id
            ));
        }
        let take = min(
            bytes.len() - source_offset,
            LOGICAL_SECTOR_SIZE - position.sector_offset,
        );
        let destination = sector.data_offset + position.sector_offset;
        data[destination..destination + take]
            .copy_from_slice(&bytes[source_offset..source_offset + take]);
        source_offset += take;
        logical_offset += take as u64;
    }
    Ok(())
}

fn update_catalog_sizes(
    data: &mut [u8],
    image: &D88Image,
    sizes: &[(usize, bool)],
) -> ArchiveResult<()> {
    let track = image
        .tracks
        .get(CATALOG_TRACK_INDEX)
        .and_then(Option::as_ref)
        .ok_or_else(|| "catalog track is missing".to_string())?;
    let mut block = Vec::with_capacity(3 * LOGICAL_SECTOR_SIZE);
    for sector_id in CATALOG_SECTOR_FIRST..=CATALOG_SECTOR_LAST {
        let sector = find_sector(track, sector_id)?;
        block
            .extend_from_slice(&data[sector.data_offset..sector.data_offset + LOGICAL_SECTOR_SIZE]);
    }
    let reverse_length = MKDS_ENTRY_SIZE
        .checked_mul(sizes.len() + 1)
        .ok_or_else(|| "catalog size overflow".to_string())?;
    if reverse_length > block.len() {
        return Err("rebuilt catalog exceeds catalog sectors".to_string());
    }
    let start = block.len() - reverse_length;
    let mut decoded: Vec<u8> = block[start..].iter().rev().copied().collect();
    if &decoded[..6] != b"MKDS_3" || decoded[8] as usize != sizes.len() {
        return Err("catalog header changed before size update".to_string());
    }
    for (index, (size, compressed)) in sizes.iter().copied().enumerate() {
        let offset = MKDS_HEADER_SIZE + index * MKDS_ENTRY_SIZE + 7;
        decoded[offset] = ((size >> 8) as u8) | if compressed { 0x80 } else { 0 };
        decoded[offset + 1] = size as u8;
    }
    for (destination, source) in block[start..].iter_mut().zip(decoded.iter().rev()) {
        *destination = *source;
    }
    for (index, sector_id) in (CATALOG_SECTOR_FIRST..=CATALOG_SECTOR_LAST).enumerate() {
        let sector = find_sector(track, sector_id)?;
        let source = index * LOGICAL_SECTOR_SIZE;
        data[sector.data_offset..sector.data_offset + LOGICAL_SECTOR_SIZE]
            .copy_from_slice(&block[source..source + LOGICAL_SECTOR_SIZE]);
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct LogicalPosition {
    track_index: u64,
    sector_id: u8,
    sector_offset: usize,
}

fn logical_position(offset: u64) -> LogicalPosition {
    let track_index = offset / LOGICAL_TRACK_SIZE;
    let within_track = offset % LOGICAL_TRACK_SIZE;
    LogicalPosition {
        track_index,
        sector_id: (within_track / LOGICAL_SECTOR_SIZE as u64) as u8 + 1,
        sector_offset: (within_track % LOGICAL_SECTOR_SIZE as u64) as usize,
    }
}

fn find_sector(track: &Track, sector_id: u8) -> ArchiveResult<&Sector> {
    track
        .sectors
        .iter()
        .find(|sector| sector.r == sector_id)
        .ok_or_else(|| {
            format!(
                "track {} (D88 offset {:#x}) is missing sector {}",
                track.table_index, track.relative_offset, sector_id
            )
        })
}

fn prepare_output(output: &Path, overwrite: bool) -> ArchiveResult<()> {
    validate_output_path(output)?;
    if output.exists() {
        if !output.is_dir() {
            return Err(format!(
                "output path is not a directory: {}",
                output.display()
            ));
        }
        let mut entries = fs::read_dir(output)
            .map_err(|error| format!("failed to inspect {}: {error}", output.display()))?;
        let nonempty = entries
            .next()
            .transpose()
            .map_err(|error| {
                format!(
                    "failed to inspect output directory {}: {error}",
                    output.display()
                )
            })?
            .is_some();
        if nonempty {
            if !overwrite {
                return Err(format!(
                    "output directory is not empty: {} (use --overwrite for a previous managed output)",
                    output.display()
                ));
            }
            if !is_managed_output(output)? {
                return Err(format!(
                    "refusing to replace an unrecognized directory: {}",
                    output.display()
                ));
            }
            fs::remove_dir_all(output).map_err(|error| {
                format!(
                    "failed to replace managed output {}: {error}",
                    output.display()
                )
            })?;
        }
    }
    fs::create_dir_all(output)
        .map_err(|error| format!("failed to create {}: {error}", output.display()))
}

fn is_managed_output(output: &Path) -> ArchiveResult<bool> {
    let marker = output.join(MANAGED_MARKER);
    if marker.is_file() {
        let value = fs::read(&marker)
            .map_err(|error| format!("failed to read {}: {error}", marker.display()))?;
        if value == b"foxy_disk_tool\n" || value == b"foxy_d88_tool\n" {
            return Ok(true);
        }
    }
    let manifest = output.join("manifest.json");
    if !manifest.is_file() {
        return Ok(false);
    }
    let value: serde_json::Value = read_json(&manifest)?;
    Ok(value.get("format").and_then(serde_json::Value::as_str) == Some(CONTAINER_FORMAT))
}

fn write_output_file(output: &Path, bytes: &[u8], overwrite: bool) -> ArchiveResult<()> {
    validate_output_path(output)?;
    if output.exists() && !overwrite {
        return Err(format!("output file exists: {}", output.display()));
    }
    if output.exists() && !output.is_file() {
        return Err(format!("output path is not a file: {}", output.display()));
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    let file_name = output
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output.d88".to_string());
    let temporary = output.with_file_name(format!(".{file_name}.foxy-{}.tmp", std::process::id()));
    if temporary.exists() {
        return Err(format!(
            "temporary output already exists: {}",
            temporary.display()
        ));
    }
    fs::write(&temporary, bytes)
        .map_err(|error| format!("failed to write {}: {error}", temporary.display()))?;
    let verified = fs::read(&temporary)
        .map_err(|error| format!("failed to verify {}: {error}", temporary.display()))?;
    if verified != bytes {
        let _ = fs::remove_file(&temporary);
        return Err("temporary output verification failed".to_string());
    }
    if output.exists() {
        fs::remove_file(output)
            .map_err(|error| format!("failed to replace {}: {error}", output.display()))?;
    }
    fs::rename(&temporary, output)
        .map_err(|error| format!("failed to finalize {}: {error}", output.display()))
}

fn validate_output_path(output: &Path) -> ArchiveResult<()> {
    if output.as_os_str().is_empty() {
        return Err("output path cannot be empty".to_string());
    }
    let absolute = absolute_normalized(output)?;
    if absolute.parent().is_none() {
        return Err("refusing to use a filesystem root as output".to_string());
    }
    let current = fs::canonicalize(".")
        .map_err(|error| format!("failed to resolve current directory: {error}"))?;
    if absolute == current {
        return Err("refusing to use the current working directory as output".to_string());
    }
    Ok(())
}

fn refuse_output_containing_input(input: &Path, output: &Path) -> ArchiveResult<()> {
    let input = fs::canonicalize(input)
        .map_err(|error| format!("failed to resolve {}: {error}", input.display()))?;
    let output = absolute_normalized(output)?;
    if input.starts_with(&output) {
        return Err(format!(
            "refusing output directory that contains the input D88: {}",
            output.display()
        ));
    }
    Ok(())
}

fn refuse_output_inside_input(input: &Path, output: &Path) -> ArchiveResult<()> {
    let input = fs::canonicalize(input)
        .map_err(|error| format!("failed to resolve {}: {error}", input.display()))?;
    let output = absolute_normalized(output)?;
    if output.starts_with(&input) {
        return Err(format!(
            "refusing D88 output inside the unpacked input workspace: {}",
            output.display()
        ));
    }
    Ok(())
}

fn absolute_normalized(path: &Path) -> ArchiveResult<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path)
            .map_err(|error| format!("failed to resolve {}: {error}", path.display()));
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| format!("failed to resolve current directory: {error}"))?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(format!(
                        "path escapes its filesystem root: {}",
                        path.display()
                    ));
                }
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str())
            }
        }
    }
    Ok(normalized)
}

fn safe_manifest_path(value: &str) -> ArchiveResult<PathBuf> {
    let path = Path::new(value);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!("unsafe path in manifest: {value}"));
    }
    Ok(path.to_path_buf())
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            bit_offset: 0,
        }
    }

    fn read_bits(&mut self, count: usize) -> ArchiveResult<u16> {
        if count > 16 {
            return Err(format!("cannot read a {count}-bit value into u16"));
        }
        let end = self
            .bit_offset
            .checked_add(count)
            .ok_or_else(|| "compressed bit offset overflow".to_string())?;
        if end > self.data.len() * 8 {
            return Err(format!(
                "compressed stream ended at bit {}, while reading {} bits at bit {}",
                self.data.len() * 8,
                count,
                self.bit_offset
            ));
        }
        let mut value = 0u16;
        for _ in 0..count {
            let byte = self.data[self.bit_offset / 8];
            let shift = 7 - self.bit_offset % 8;
            value = (value << 1) | ((byte >> shift) & 1) as u16;
            self.bit_offset += 1;
        }
        Ok(value)
    }
}

fn decode_mkds_compressed(stored: &[u8]) -> ArchiveResult<DecodedStream> {
    if stored.is_empty() {
        return Err("compressed stream is empty".to_string());
    }
    let mut reader = BitReader::new(stored);
    let explicit_count = reader.read_bits(4)? as usize;
    if explicit_count == 0 || explicit_count > 15 {
        return Err(format!(
            "invalid explicit Huffman symbol count {explicit_count}"
        ));
    }
    let mut current_length = reader.read_bits(4)? as usize;
    if current_length == 0 || current_length > 15 {
        return Err(format!(
            "invalid initial Huffman code length {current_length}"
        ));
    }

    let mut codes = Vec::with_capacity(explicit_count);
    let mut remaining = explicit_count;
    while remaining > 0 {
        let group_count = reader.read_bits(4)? as usize;
        if group_count == 0 {
            return Err("zero-sized Huffman code group is not supported by the loader".to_string());
        }
        for _ in 0..min(group_count, remaining) {
            let bits = reader.read_bits(current_length)?;
            let symbol = reader.read_bits(4)? as u8;
            codes.push(HuffmanCode {
                length: current_length,
                bits,
                symbol,
            });
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }
        if remaining > 0 {
            let length_delta = reader.read_bits(4)? as usize;
            if length_delta == 0 {
                return Err(
                    "zero Huffman code-length delta would trigger a 256-step loader loop"
                        .to_string(),
                );
            }
            current_length = current_length
                .checked_add(length_delta)
                .ok_or_else(|| "Huffman code length overflow".to_string())?;
            if current_length > 15 {
                return Err(format!(
                    "Huffman code length {current_length} exceeds the loader's 15-bit limit"
                ));
            }
        }
    }

    let _final_code_bits = if current_length >= 9 {
        let high = reader.read_bits(current_length - 8)?;
        (high << 8) | reader.read_bits(8)?
    } else {
        reader.read_bits(current_length)?
    };
    let final_symbol = reader.read_bits(4)? as u8;
    let decode_nibble = |reader: &mut BitReader<'_>| -> ArchiveResult<u8> {
        let mut bits = 0u16;
        for length in 1..=current_length {
            bits = (bits << 1) | reader.read_bits(1)?;
            if let Some(code) = codes
                .iter()
                .find(|code| code.length == length && code.bits == bits)
            {
                return Ok(code.symbol);
            }
        }
        Ok(final_symbol)
    };
    let decode_byte = |reader: &mut BitReader<'_>| -> ArchiveResult<u8> {
        let high = decode_nibble(reader)?;
        let low = decode_nibble(reader)?;
        Ok((high << 4) | low)
    };

    let size_high = decode_byte(&mut reader)?;
    let size_low = decode_byte(&mut reader)?;
    let decoded_with_header = u16::from_be_bytes([size_high, size_low]) as usize;
    if decoded_with_header < 2 {
        return Err(format!(
            "invalid decoded size header {decoded_with_header:#x}"
        ));
    }
    let decoded_size = decoded_with_header - 2;
    let mut bytes = Vec::with_capacity(decoded_size);
    for _ in 0..decoded_size {
        bytes.push(decode_byte(&mut reader)?);
    }
    Ok(DecodedStream {
        bytes,
        consumed_bits: reader.bit_offset,
        total_bits: stored.len() * 8,
    })
}

impl BitWriter {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            bit_offset: 0,
        }
    }

    fn write_bits(&mut self, value: u16, count: usize) -> ArchiveResult<()> {
        if count > 16 || (count < 16 && value >= (1u16 << count)) {
            return Err(format!("value {value:#x} does not fit in {count} bits"));
        }
        for shift in (0..count).rev() {
            if self.bit_offset.is_multiple_of(8) {
                self.bytes.push(0);
            }
            if (value >> shift) & 1 != 0 {
                let last = self.bytes.len() - 1;
                self.bytes[last] |= 1 << (7 - self.bit_offset % 8);
            }
            self.bit_offset += 1;
        }
        Ok(())
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

fn encode_mkds_compressed(decoded: &[u8]) -> ArchiveResult<Vec<u8>> {
    let decoded_with_header = decoded
        .len()
        .checked_add(2)
        .ok_or_else(|| "decoded size overflow".to_string())?;
    if decoded_with_header > u16::MAX as usize {
        return Err(format!(
            "decoded file is {} bytes; compressed format limit is {}",
            decoded.len(),
            u16::MAX as usize - 2
        ));
    }
    let size_bytes = (decoded_with_header as u16).to_be_bytes();
    let mut frequencies = [1u64; 16];
    for byte in size_bytes.iter().chain(decoded) {
        frequencies[(byte >> 4) as usize] += 1;
        frequencies[(byte & 0x0F) as usize] += 1;
    }
    let codes = build_huffman_codes(&frequencies)?;
    let maximum_length = codes.iter().map(|code| code.length).max().unwrap_or(0);
    let final_code = codes
        .iter()
        .filter(|code| code.length == maximum_length)
        .max_by_key(|code| code.symbol)
        .copied()
        .ok_or_else(|| "failed to select Huffman fallback symbol".to_string())?;
    let mut explicit: Vec<_> = codes
        .iter()
        .copied()
        .filter(|code| code.symbol != final_code.symbol)
        .collect();
    explicit.sort_by_key(|code| (code.length, code.symbol));
    if explicit.len() != 15 || explicit.last().map(|code| code.length) != Some(maximum_length) {
        return Err("Huffman tree does not leave another maximum-depth explicit code".to_string());
    }

    let mut writer = BitWriter::new();
    writer.write_bits(15, 4)?;
    let mut current_length = explicit[0].length;
    writer.write_bits(current_length as u16, 4)?;
    let mut index = 0usize;
    while index < explicit.len() {
        let group_end = explicit[index..]
            .iter()
            .position(|code| code.length != current_length)
            .map(|relative| index + relative)
            .unwrap_or(explicit.len());
        let count = group_end - index;
        writer.write_bits(count as u16, 4)?;
        for code in &explicit[index..group_end] {
            writer.write_bits(code.bits, code.length)?;
            writer.write_bits(code.symbol as u16, 4)?;
        }
        index = group_end;
        if index < explicit.len() {
            let next_length = explicit[index].length;
            writer.write_bits((next_length - current_length) as u16, 4)?;
            current_length = next_length;
        }
    }
    if current_length != final_code.length {
        return Err(
            "Huffman fallback code length differs from the final explicit group".to_string(),
        );
    }
    writer.write_bits(final_code.bits, final_code.length)?;
    writer.write_bits(final_code.symbol as u16, 4)?;
    for byte in size_bytes.iter().chain(decoded) {
        for symbol in [byte >> 4, byte & 0x0F] {
            let code = &codes[symbol as usize];
            writer.write_bits(code.bits, code.length)?;
        }
    }
    Ok(writer.finish())
}

fn build_huffman_codes(frequencies: &[u64; 16]) -> ArchiveResult<[HuffmanCode; 16]> {
    let mut nodes = Vec::with_capacity(31);
    let mut queue = Vec::with_capacity(16);
    for (symbol, frequency) in frequencies.iter().copied().enumerate() {
        nodes.push(EncodeNode {
            frequency,
            minimum_symbol: symbol as u8,
            left: None,
            right: None,
            symbol: Some(symbol as u8),
        });
        queue.push(symbol);
    }
    while queue.len() > 1 {
        queue.sort_by_key(|index| {
            let node = &nodes[*index];
            (node.frequency, node.minimum_symbol, *index)
        });
        let left = queue.remove(0);
        let right = queue.remove(0);
        let index = nodes.len();
        nodes.push(EncodeNode {
            frequency: nodes[left].frequency + nodes[right].frequency,
            minimum_symbol: min(nodes[left].minimum_symbol, nodes[right].minimum_symbol),
            left: Some(left),
            right: Some(right),
            symbol: None,
        });
        queue.push(index);
    }
    let root = queue[0];
    let mut lengths = [0usize; 16];
    assign_huffman_depths(&nodes, root, 0, &mut lengths)?;
    if lengths.iter().any(|length| *length == 0 || *length > 15) {
        return Err(format!("unsupported Huffman code lengths: {lengths:?}"));
    }
    let mut ordered: Vec<_> = (0u8..16)
        .map(|symbol| (lengths[symbol as usize], symbol))
        .collect();
    ordered.sort();
    let mut result = [HuffmanCode {
        length: 0,
        bits: 0,
        symbol: 0,
    }; 16];
    let mut code = 0u16;
    let mut previous_length = ordered[0].0;
    for (index, (length, symbol)) in ordered.iter().copied().enumerate() {
        if index > 0 {
            code = code
                .checked_add(1)
                .ok_or_else(|| "canonical Huffman code overflow".to_string())?;
            code <<= length - previous_length;
        }
        result[symbol as usize] = HuffmanCode {
            length,
            bits: code,
            symbol,
        };
        previous_length = length;
    }
    Ok(result)
}

fn assign_huffman_depths(
    nodes: &[EncodeNode],
    index: usize,
    depth: usize,
    lengths: &mut [usize; 16],
) -> ArchiveResult<()> {
    let node = nodes
        .get(index)
        .ok_or_else(|| "invalid Huffman node".to_string())?;
    if let Some(symbol) = node.symbol {
        lengths[symbol as usize] = depth;
        return Ok(());
    }
    assign_huffman_depths(
        nodes,
        node.left.ok_or("missing Huffman left child")?,
        depth + 1,
        lengths,
    )?;
    assign_huffman_depths(
        nodes,
        node.right.ok_or("missing Huffman right child")?,
        depth + 1,
        lengths,
    )
}

fn decode_packed_name(packed: &[u8; 7]) -> String {
    let mut chars = String::with_capacity(9);
    for index in 0..9 {
        let bit_offset = index * 6;
        let mut value = 0u8;
        for bit in 0..6 {
            let absolute = bit_offset + bit;
            let source = packed[absolute / 8];
            let source_bit = 7 - (absolute % 8);
            value = (value << 1) | ((source >> source_bit) & 1);
        }
        chars.push((value + 0x20) as char);
    }
    chars
}

fn display_name_6_3(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let base = String::from_utf8_lossy(&bytes[..min(6, bytes.len())])
        .trim_end()
        .to_string();
    let extension = if bytes.len() > 6 {
        String::from_utf8_lossy(&bytes[6..]).trim_end().to_string()
    } else {
        String::new()
    };
    let base = if base.is_empty() {
        "UNNAMED".to_string()
    } else {
        base
    };
    if extension.is_empty() {
        base
    } else {
        format!("{base}.{extension}")
    }
}

fn sanitize_component(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|character| {
            if character.is_control() || "<>:\"/\\|?*".contains(character) {
                '_'
            } else {
                character
            }
        })
        .collect();
    let trimmed = sanitized.trim().trim_end_matches(['.', ' ']);
    if trimmed.is_empty() {
        "UNNAMED".to_string()
    } else {
        trimmed.to_string()
    }
}

fn sanitize_filename(value: &str) -> String {
    sanitize_component(value)
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join("")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> ArchiveResult<()> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> ArchiveResult<T> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn read_u16_le(data: &[u8], offset: usize) -> ArchiveResult<u16> {
    require_range(data, offset, 2, "16-bit integer")?;
    Ok(u16::from_le_bytes([data[offset], data[offset + 1]]))
}

fn read_u32_le(data: &[u8], offset: usize) -> ArchiveResult<u32> {
    require_range(data, offset, 4, "32-bit integer")?;
    Ok(u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

fn require_range(data: &[u8], offset: usize, size: usize, label: &str) -> ArchiveResult<()> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| format!("{label} range overflow"))?;
    if end > data.len() {
        return Err(format!(
            "{label} at {offset:#x}..{end:#x} exceeds input size {:#x}",
            data.len()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_name(raw: &str) -> [u8; 7] {
        assert_eq!(raw.len(), 9);
        let mut bits = Vec::with_capacity(56);
        for byte in raw.bytes() {
            let value = byte - 0x20;
            for shift in (0..6).rev() {
                bits.push((value >> shift) & 1);
            }
        }
        bits.extend_from_slice(&[0, 0]);
        let mut output = [0u8; 7];
        for (index, bit) in bits.into_iter().enumerate() {
            output[index / 8] |= bit << (7 - index % 8);
        }
        output
    }

    #[test]
    fn packed_names_round_trip() {
        for raw in ["FLAG0    ", "21-1  ANI", "ENDINGMES", "AISUB COM"] {
            assert_eq!(decode_packed_name(&encode_name(raw)), raw);
        }
    }

    #[test]
    fn display_name_uses_six_plus_three_layout() {
        assert_eq!(display_name_6_3("21-1  ANI"), "21-1.ANI");
        assert_eq!(display_name_6_3("ENDINGMES"), "ENDING.MES");
        assert_eq!(display_name_6_3("FLAG0    "), "FLAG0");
    }

    #[test]
    fn logical_offsets_follow_five_one_kib_sectors_per_track() {
        assert_eq!(
            logical_position(0x1400),
            LogicalPosition {
                track_index: 1,
                sector_id: 1,
                sector_offset: 0,
            }
        );
        assert_eq!(
            logical_position(79 * 0x1400 + 0x0A55),
            LogicalPosition {
                track_index: 79,
                sector_id: 3,
                sector_offset: 0x255,
            }
        );
    }

    #[test]
    fn mkds_compression_round_trips_varied_nibbles() {
        let mut data = Vec::new();
        for value in 0u8..=255 {
            data.extend(std::iter::repeat_n(value, usize::from(value % 7) + 1));
        }
        let encoded = encode_mkds_compressed(&data).unwrap();
        let decoded = decode_mkds_compressed(&encoded).unwrap();
        assert_eq!(decoded.bytes, data);
        assert!(decoded.consumed_bits <= encoded.len() * 8);
    }

    #[test]
    fn manifest_paths_cannot_escape_workspace() {
        assert!(safe_manifest_path("volumes/00_FOXY_A/catalog.json").is_ok());
        assert!(safe_manifest_path("../manifest.json").is_err());
        assert!(safe_manifest_path("C:\\outside.d88").is_err());
        assert!(safe_manifest_path("").is_err());
    }
}
