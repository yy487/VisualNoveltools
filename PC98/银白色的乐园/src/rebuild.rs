//! Fixed-geometry D88/FAT12 rebuilding for the Chrome Paradise workspace.
//!
//! The unpacker deliberately keeps the original D88 byte stream immutable.  This
//! module accepts one `fivec-new` export directory, validates its manifest and
//! source hashes, then produces a new byte stream.  Only FAT12 metadata, root
//! directory entries and bytes belonging to changed files are patched.  D88
//! sector headers and opaque bytes are copied from the source unchanged.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use vn_d88::{Decoder, StandardCodec};
use vn_sector_map::LinearView;

use crate::Result;

const EXPORT_SCHEMA: &str = "fivec-new-export-v1";
const INSPECTION_SCHEMA: &str = "fivec-new-inspection-v1";
const EXPECTED_SOURCE_SIZE: usize = 1_281_968;
const EXPECTED_LOGICAL_SIZE: usize = 1_261_568;
const EXPECTED_SECTORS: usize = 1_232;
const EXPECTED_TRACKS: usize = 154;
const EXPECTED_SECTORS_PER_TRACK: usize = 8;
const EXPECTED_BYTES_PER_SECTOR: usize = 1_024;

#[derive(Debug, Clone)]
pub struct RebuildReport {
    pub bytes: Vec<u8>,
    pub files: usize,
    pub changed_files: usize,
    pub allocated_clusters: usize,
    pub released_clusters: usize,
    pub source_sha256: String,
    pub output_sha256: String,
}

/// Summary for one member of a multi-disk rebuild.
///
/// The `disk_number` is the one-based position in the caller's source slice;
/// it is deliberately kept in the report so a caller cannot accidentally
/// reorder the rebuilt images after the atomic commit.
#[derive(Debug, Clone, Serialize)]
pub struct BatchDiskRebuildReport {
    pub disk_number: usize,
    pub source_name: String,
    pub output_name: String,
    pub source_sha256: String,
    pub output_sha256: String,
    pub files: usize,
    pub changed_files: usize,
    pub allocated_clusters: usize,
    pub released_clusters: usize,
    pub bytes: usize,
}

/// Result of an all-disks D88 rebuild.
///
/// `output_directory` is committed only after every disk has passed the
/// single-disk verifier.  A failed disk therefore leaves no partially
/// populated output directory behind.
#[derive(Debug, Clone, Serialize)]
pub struct BatchRebuildReport {
    pub output_directory: PathBuf,
    pub disks: usize,
    pub files: usize,
    pub changed_files: usize,
    pub allocated_clusters: usize,
    pub released_clusters: usize,
    pub bytes: usize,
    pub disk_reports: Vec<BatchDiskRebuildReport>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct WorkspaceRootManifest {
    format: String,
    disk_count: usize,
    disks: Vec<WorkspaceRootDisk>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct WorkspaceRootDisk {
    disk_number: usize,
    source_name: String,
    source_size: usize,
    source_sha256: String,
    output_directory: String,
}

#[derive(Debug, Clone)]
struct BatchSource {
    path: PathBuf,
    name: OsString,
    display_name: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
struct ExportManifest {
    schema: String,
    inspection: InspectionManifest,
    files: Vec<ExportFileManifest>,
}

#[derive(Debug, Deserialize)]
struct InspectionManifest {
    schema: String,
    source_name: String,
    source_size: usize,
    source_sha256: String,
    format: String,
    volumes: Vec<VolumeManifest>,
}

#[derive(Debug, Deserialize)]
struct VolumeManifest {
    id: String,
    disk_index: usize,
    partition_index: Option<usize>,
    logical_byte_offset: usize,
    logical_byte_length: usize,
    filesystem: Option<FileSystemManifest>,
}

#[derive(Debug, Deserialize)]
struct FileSystemManifest {
    kind: String,
    fat12: Fat12Manifest,
    files: Vec<FsFileManifest>,
    #[serde(default)]
    directories: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Fat12Manifest {
    bytes_per_sector: usize,
    sectors_per_cluster: usize,
    reserved_sectors: usize,
    fat_copies: usize,
    sectors_per_fat: usize,
    root_entries: usize,
    total_sectors: usize,
    first_data_sector: usize,
    data_clusters: usize,
    fat_copies_identical: bool,
}

#[derive(Debug, Deserialize)]
struct FsFileManifest {
    id: String,
    export_path: String,
    raw_name_components_hex: Vec<String>,
    attributes: u8,
    directory_entry_offset: usize,
    size: usize,
    sha256: String,
    clusters: Vec<u32>,
}

#[derive(Debug, Deserialize)]
struct ExportFileManifest {
    path: String,
    volume_id: String,
    entry_id: String,
    size: usize,
    sha256: String,
}

#[derive(Debug, Clone)]
struct Layout {
    volume_base: usize,
    volume_bytes: usize,
    bytes_per_sector: usize,
    fat_copies: usize,
    first_data_sector: usize,
    fat_offset: usize,
    fat_bytes: usize,
    root_offset: usize,
    root_bytes: usize,
    cluster_bytes: usize,
    max_cluster: u16,
}

#[derive(Debug, Clone)]
struct FilePlan {
    path: String,
    raw_name: Vec<u8>,
    directory_entry_offset: usize,
    old_chain: Vec<u16>,
    desired_chain: Vec<u16>,
    original: Vec<u8>,
    data: Vec<u8>,
}

/// Rebuild one fixed-geometry Chrome Paradise D88 from a `fivec-new` export
/// directory.  Replacement keys are the exact manifest paths, for example
/// `disk-000-whole/PRO.SCR`.  The source and every workspace member must still
/// match the export manifest before replacements are applied.
pub fn rebuild_d88_fat12(
    source: &[u8],
    workspace: &Path,
    replacements: &BTreeMap<String, Vec<u8>>,
) -> Result<RebuildReport> {
    let manifest = read_manifest(workspace)?;
    let source_sha256 = sha256_hex(source);
    validate_source_manifest(source, &manifest, &source_sha256)?;

    let (_image, view) = decode_fixed_d88(source)?;
    let volume = select_volume(&manifest)?;
    let filesystem = volume
        .filesystem
        .as_ref()
        .ok_or_else(|| "manifest volume has no FAT12 filesystem".to_string())?;
    if filesystem.kind != "FAT12" {
        return Err(format!("unsupported filesystem kind: {}", filesystem.kind));
    }
    if !filesystem.directories.is_empty() {
        return Err("Chrome Paradise rebuild currently supports root files only".into());
    }
    let layout = validate_layout(source, &view, volume, &filesystem.fat12)?;
    let fat = read_logical(source, &view, layout.fat_offset, layout.fat_bytes)?;
    validate_fat(&fat, &filesystem.fat12, layout.max_cluster)?;

    let top_files = index_export_files(&manifest.files, &volume.id)?;
    let mut files = validate_workspace_files(workspace, volume, &filesystem.files, &top_files)?;
    validate_replacement_keys(replacements, &top_files)?;
    for plan in &mut files {
        if let Some(data) = replacements.get(&plan.path) {
            plan.data = data.clone();
        }
        if plan.data.len() > u32::MAX as usize {
            return Err(format!("{} exceeds FAT12 directory size", plan.path));
        }
    }

    let mut owners = HashMap::<u16, String>::new();
    for plan in &mut files {
        let entry = layout
            .volume_base
            .checked_add(plan.directory_entry_offset)
            .ok_or_else(|| "directory entry offset overflow".to_string())?;
        if plan.directory_entry_offset < layout.root_offset - layout.volume_base
            || plan.directory_entry_offset + 32
                > layout.root_offset - layout.volume_base + layout.root_bytes
            || plan.directory_entry_offset % 32 != 0
        {
            return Err(format!(
                "{} directory entry is outside the root directory",
                plan.path
            ));
        }
        let raw = read_logical(source, &view, entry, 32)?;
        if raw[..11] != plan.raw_name[..]
            || raw[11] != 0x20
            || usize::try_from(u32::from_le_bytes(
                raw[28..32].try_into().expect("four bytes"),
            ))
            .map_err(|_| "file size does not fit usize".to_string())?
                != plan.original.len()
        {
            return Err(format!(
                "{} directory entry does not match manifest",
                plan.path
            ));
        }
        let start = plan.old_chain.first().copied().unwrap_or(0);
        if read_u16(&raw, 26)? != start {
            return Err(format!(
                "{} directory start cluster differs from manifest",
                plan.path
            ));
        }
        let actual = read_chain(&fat, layout.max_cluster, start, &plan.path, &mut owners)?;
        if actual != plan.old_chain {
            return Err(format!("{} cluster chain differs from manifest", plan.path));
        }
        let expected = cluster_count(plan.original.len(), layout.cluster_bytes);
        if expected != actual.len() {
            return Err(format!("{} size does not match FAT12 chain", plan.path));
        }
    }
    reject_orphan_clusters(&fat, layout.max_cluster, &owners)?;

    let changed = files
        .iter()
        .filter(|plan| plan.data != plan.original)
        .count();
    if changed == 0 {
        return Ok(RebuildReport {
            bytes: source.to_vec(),
            files: files.len(),
            changed_files: 0,
            allocated_clusters: 0,
            released_clusters: 0,
            source_sha256: source_sha256.clone(),
            output_sha256: source_sha256,
        });
    }

    let mut reserved = HashSet::new();
    for plan in &files {
        if plan.data == plan.original {
            reserved.extend(plan.old_chain.iter().copied());
        }
    }
    let mut available = BTreeSet::new();
    for cluster in 2..=layout.max_cluster {
        if !reserved.contains(&cluster) && fat12_next(&fat, cluster)? == 0 {
            available.insert(cluster);
        }
    }

    let mut released_clusters = 0usize;
    for plan in &mut files {
        if plan.data == plan.original {
            plan.desired_chain = plan.old_chain.clone();
            continue;
        }
        let needed = cluster_count(plan.data.len(), layout.cluster_bytes);
        let retained = needed.min(plan.old_chain.len());
        plan.desired_chain
            .extend_from_slice(&plan.old_chain[..retained]);
        for &cluster in &plan.old_chain[retained..] {
            if !available.insert(cluster) {
                return Err(format!("released cluster {cluster} is already reserved"));
            }
            released_clusters += 1;
        }
    }
    let mut additional = 0usize;
    for plan in &files {
        if plan.data != plan.original {
            additional += cluster_count(plan.data.len(), layout.cluster_bytes)
                .saturating_sub(plan.desired_chain.len());
        }
    }
    if additional > available.len() {
        return Err(format!(
            "insufficient FAT12 space: need {additional} clusters, have {}",
            available.len()
        ));
    }
    let mut allocated_clusters = 0usize;
    for plan in &mut files {
        if plan.data == plan.original {
            continue;
        }
        let needed = cluster_count(plan.data.len(), layout.cluster_bytes);
        while plan.desired_chain.len() < needed {
            let cluster = available
                .pop_first()
                .ok_or_else(|| "FAT12 free-cluster accounting underflow".to_string())?;
            plan.desired_chain.push(cluster);
            allocated_clusters += 1;
        }
    }

    let mut rebuilt = source.to_vec();
    let mut rebuilt_fat = fat.clone();
    for plan in &files {
        if plan.data == plan.original {
            continue;
        }
        for &cluster in &plan.old_chain {
            set_fat12(&mut rebuilt_fat, cluster, 0)?;
        }
        for (index, &cluster) in plan.desired_chain.iter().enumerate() {
            let next = plan.desired_chain.get(index + 1).copied().unwrap_or(0x0fff);
            set_fat12(&mut rebuilt_fat, cluster, next)?;
        }
    }
    for copy in 0..layout.fat_copies {
        let offset = layout
            .fat_offset
            .checked_add(copy * layout.fat_bytes)
            .ok_or_else(|| "FAT offset overflow".to_string())?;
        patch_logical(&view, &mut rebuilt, offset, &rebuilt_fat)?;
    }

    for plan in &files {
        if plan.data == plan.original {
            continue;
        }
        for (index, &cluster) in plan.desired_chain.iter().enumerate() {
            let logical = cluster_offset(&layout, cluster)?;
            let start = index * layout.cluster_bytes;
            let end = (start + layout.cluster_bytes).min(plan.data.len());
            if start < end {
                patch_logical(&view, &mut rebuilt, logical, &plan.data[start..end])?;
            }
        }
        let entry = layout
            .volume_base
            .checked_add(plan.directory_entry_offset)
            .ok_or_else(|| "directory entry offset overflow".to_string())?;
        let start_cluster = plan.desired_chain.first().copied().unwrap_or(0);
        patch_logical(
            &view,
            &mut rebuilt,
            entry + 26,
            &start_cluster.to_le_bytes(),
        )?;
        let size =
            u32::try_from(plan.data.len()).map_err(|_| "file size exceeds u32".to_string())?;
        patch_logical(&view, &mut rebuilt, entry + 28, &size.to_le_bytes())?;
    }

    let (_, rebuilt_view) = decode_fixed_d88(&rebuilt)?;
    validate_rebuilt(
        &rebuilt,
        &rebuilt_view,
        &layout,
        &files,
        changed,
        &source_sha256,
    )?;
    let output_sha256 = sha256_hex(&rebuilt);
    Ok(RebuildReport {
        bytes: rebuilt,
        files: files.len(),
        changed_files: changed,
        allocated_clusters,
        released_clusters,
        source_sha256,
        output_sha256,
    })
}

/// File wrapper for `rebuild_d88_fat12`.  It refuses an existing output and
/// commits through a sibling temporary file, leaving the source untouched.
pub fn rebuild_d88_file(
    source_path: &Path,
    workspace: &Path,
    output_path: &Path,
    replacements: &BTreeMap<String, Vec<u8>>,
) -> Result<RebuildReport> {
    let source_path = fs::canonicalize(source_path)
        .map_err(|e| format!("failed to resolve source {}: {e}", source_path.display()))?;
    let workspace = fs::canonicalize(workspace)
        .map_err(|e| format!("failed to resolve workspace {}: {e}", workspace.display()))?;
    if !workspace.is_dir() {
        return Err(format!(
            "workspace is not a directory: {}",
            workspace.display()
        ));
    }
    if output_path.exists() {
        return Err(format!("output already exists: {}", output_path.display()));
    }
    let parent = output_path
        .parent()
        .ok_or_else(|| "output has no parent directory".to_string())?;
    let parent =
        fs::canonicalize(parent).map_err(|e| format!("failed to resolve output parent: {e}"))?;
    let output = parent.join(
        output_path
            .file_name()
            .ok_or_else(|| "output has no filename".to_string())?,
    );
    if source_path == output || source_path.starts_with(&output) {
        return Err("output overlaps source image".into());
    }
    if workspace == output || output.starts_with(&workspace) {
        return Err("output overlaps workspace".into());
    }
    let source = fs::read(&source_path)
        .map_err(|e| format!("failed to read source {}: {e}", source_path.display()))?;
    let report = rebuild_d88_fat12(&source, &workspace, replacements)?;
    let temporary = parent.join(format!(
        ".{}.chrome-paradise-rebuild-{}",
        output
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("output.d88"),
        std::process::id()
    ));
    if temporary.exists() {
        return Err(format!(
            "temporary output already exists: {}",
            temporary.display()
        ));
    }
    let write_result = fs::write(&temporary, &report.bytes)
        .map_err(|e| format!("failed to write temporary D88: {e}"))
        .and_then(|_| {
            fs::rename(&temporary, &output).map_err(|e| format!("failed to commit D88: {e}"))
        });
    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    write_result.map(|_| report)
}

/// Rebuild one D88 by taking all file bytes from a modified export directory
/// while validating the original export directory against the source image.
/// The two directories must contain the same manifest-shaped file set; the
/// modified directory may change file sizes, including FAT12 cluster growth.
pub fn rebuild_d88_from_workspaces(
    source_path: &Path,
    original_workspace: &Path,
    modified_workspace: &Path,
    output_path: &Path,
) -> Result<RebuildReport> {
    let original_workspace = fs::canonicalize(original_workspace)
        .map_err(|e| format!("failed to resolve original workspace: {e}"))?;
    let modified_workspace = fs::canonicalize(modified_workspace)
        .map_err(|e| format!("failed to resolve modified workspace: {e}"))?;
    if !original_workspace.is_dir() || !modified_workspace.is_dir() {
        return Err("original and modified workspaces must be directories".into());
    }
    let original_files = collect_workspace_files(&original_workspace)?;
    let modified_files = collect_workspace_files(&modified_workspace)?;
    if original_files != modified_files {
        return Err("modified workspace file set differs from the original export".into());
    }
    let mut replacements = BTreeMap::new();
    for relative in modified_files {
        let path = modified_workspace.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        let data =
            fs::read(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        replacements.insert(relative, data);
    }
    rebuild_d88_file(source_path, &original_workspace, output_path, &replacements)
}

/// Rebuild every disk in a Chrome Paradise workspace in input order.
///
/// `sources[i]` must correspond to `original_root/disk-{i + 1:02}` and
/// `modified_root/disk-{i + 1:02}`.  Both roots must contain the unpacker's
/// `workspace.json`; its disk order, source names, sizes and SHA-256 values
/// are checked against the current source files before any output is written.
/// Each modified disk is then staged by calling [`rebuild_d88_file`] with the
/// original disk directory as the manifest authority.  The destination must
/// not exist.  Once all disks and the batch manifest have been written, the
/// staging directory is renamed into `output_directory` as one commit.
///
/// The output files retain the source basenames.  Duplicate basenames are
/// rejected because they would make disk order ambiguous on a Windows host.
pub fn rebuild_d88_batch_from_workspaces(
    sources: &[PathBuf],
    original_root: &Path,
    modified_root: &Path,
    output_directory: &Path,
) -> Result<BatchRebuildReport> {
    if sources.is_empty() {
        return Err("至少需要一张 D88 输入镜像".into());
    }

    let original_root = fs::canonicalize(original_root)
        .map_err(|e| format!("failed to resolve original workspace root: {e}"))?;
    let modified_root = fs::canonicalize(modified_root)
        .map_err(|e| format!("failed to resolve modified workspace root: {e}"))?;
    if !original_root.is_dir() || !modified_root.is_dir() {
        return Err("original and modified workspace roots must be directories".into());
    }
    if original_root == modified_root {
        return Err("original and modified workspace roots must be different".into());
    }

    let original_manifest = read_workspace_root_manifest(&original_root, "original")?;
    let modified_manifest = read_workspace_root_manifest(&modified_root, "modified")?;
    validate_workspace_root_shape(
        &original_root,
        &original_manifest,
        sources.len(),
        "original",
    )?;
    validate_workspace_root_shape(
        &modified_root,
        &modified_manifest,
        sources.len(),
        "modified",
    )?;
    if original_manifest != modified_manifest {
        return Err("original and modified workspace.json metadata differ".into());
    }

    let sources = validate_batch_sources(sources, &original_manifest)?;
    let output_directory =
        normalize_batch_output(output_directory, &original_root, &modified_root, &sources)?;
    let parent = output_directory
        .parent()
        .ok_or_else(|| "output directory has no parent".to_string())?;
    let stage = allocate_batch_stage(parent, output_directory.file_name())?;

    let result = (|| {
        let mut disk_reports = Vec::with_capacity(sources.len());
        let mut files = 0usize;
        let mut changed_files = 0usize;
        let mut allocated_clusters = 0usize;
        let mut released_clusters = 0usize;
        let mut bytes = 0usize;

        for (index, source) in sources.iter().enumerate() {
            let disk_name = disk_directory_name(index + 1);
            let original_disk = original_root.join(&disk_name);
            let modified_disk = modified_root.join(&disk_name);
            let replacements = replacements_from_workspaces(&original_disk, &modified_disk)?;
            let output_path = stage.join(&source.name);
            let report =
                rebuild_d88_file(&source.path, &original_disk, &output_path, &replacements)?;
            if !report.source_sha256.eq_ignore_ascii_case(&source.sha256) {
                return Err(format!(
                    "source changed while rebuilding disk {} ({})",
                    index + 1,
                    source.display_name
                ));
            }
            let disk_report = BatchDiskRebuildReport {
                disk_number: index + 1,
                source_name: source.display_name.clone(),
                output_name: source.display_name.clone(),
                source_sha256: report.source_sha256,
                output_sha256: report.output_sha256,
                files: report.files,
                changed_files: report.changed_files,
                allocated_clusters: report.allocated_clusters,
                released_clusters: report.released_clusters,
                bytes: report.bytes.len(),
            };
            files = files
                .checked_add(disk_report.files)
                .ok_or_else(|| "file count overflow".to_string())?;
            changed_files = changed_files
                .checked_add(disk_report.changed_files)
                .ok_or_else(|| "changed-file count overflow".to_string())?;
            allocated_clusters = allocated_clusters
                .checked_add(disk_report.allocated_clusters)
                .ok_or_else(|| "allocated-cluster count overflow".to_string())?;
            released_clusters = released_clusters
                .checked_add(disk_report.released_clusters)
                .ok_or_else(|| "released-cluster count overflow".to_string())?;
            bytes = bytes
                .checked_add(disk_report.bytes)
                .ok_or_else(|| "output byte count overflow".to_string())?;
            disk_reports.push(disk_report);
        }

        let batch = BatchRebuildReport {
            output_directory: output_directory.clone(),
            disks: disk_reports.len(),
            files,
            changed_files,
            allocated_clusters,
            released_clusters,
            bytes,
            disk_reports,
        };
        let manifest = serde_json::to_vec_pretty(&batch).map_err(|e| e.to_string())?;
        let manifest_path = stage.join("batch_manifest.json");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&manifest_path)
            .map_err(|e| format!("failed to create {}: {e}", manifest_path.display()))?;
        use std::io::Write;
        file.write_all(&manifest).map_err(|e| e.to_string())?;
        file.write_all(b"\n").map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);

        if output_directory.exists() {
            return Err("output directory appeared during rebuild".into());
        }
        fs::rename(&stage, &output_directory)
            .map_err(|e| format!("failed to commit batch rebuild directory: {e}"))?;
        Ok(batch)
    })();

    if result.is_err() {
        let _ = fs::remove_dir_all(&stage);
    }
    result
}

fn read_workspace_root_manifest(root: &Path, label: &str) -> Result<WorkspaceRootManifest> {
    let path = root.join("workspace.json");
    let bytes = fs::read(&path).map_err(|e| {
        format!(
            "failed to read {label} workspace manifest {}: {e}",
            path.display()
        )
    })?;
    serde_json::from_slice(&bytes)
        .map_err(|e| format!("invalid {label} workspace manifest {}: {e}", path.display()))
}

fn validate_workspace_root_shape(
    root: &Path,
    manifest: &WorkspaceRootManifest,
    disk_count: usize,
    label: &str,
) -> Result<()> {
    if manifest.format != crate::WORKSPACE_FORMAT {
        return Err(format!(
            "unsupported {label} workspace format: {}",
            manifest.format
        ));
    }
    if manifest.disk_count != disk_count || manifest.disks.len() != disk_count {
        return Err(format!(
            "{label} workspace disk count does not match source list"
        ));
    }
    let expected_names: BTreeSet<String> = (1..=disk_count).map(disk_directory_name).collect();
    let mut actual_names = BTreeSet::new();
    for entry in
        fs::read_dir(root).map_err(|e| format!("failed to read {label} workspace root: {e}"))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        if metadata.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            actual_names.insert(name);
        }
    }
    if actual_names != expected_names {
        return Err(format!(
            "{label} workspace disk directories differ from expected order"
        ));
    }
    for (index, disk) in manifest.disks.iter().enumerate() {
        let expected_directory = disk_directory_name(index + 1);
        if disk.disk_number != index + 1 || disk.output_directory != expected_directory {
            return Err(format!(
                "{label} workspace.json disk order does not match disk directories"
            ));
        }
        if !root.join(&expected_directory).is_dir() {
            return Err(format!(
                "missing {label} disk directory: {}",
                root.join(&expected_directory).display()
            ));
        }
    }
    Ok(())
}

fn validate_batch_sources(
    sources: &[PathBuf],
    manifest: &WorkspaceRootManifest,
) -> Result<Vec<BatchSource>> {
    let mut result = Vec::with_capacity(sources.len());
    let mut paths = HashSet::new();
    let mut names = HashSet::new();
    for (index, input) in sources.iter().enumerate() {
        let path = fs::canonicalize(input)
            .map_err(|e| format!("failed to resolve source {}: {e}", input.display()))?;
        if !path.is_file() {
            return Err(format!("source is not a file: {}", path.display()));
        }
        let path_key = path.to_string_lossy().to_lowercase();
        if !paths.insert(path_key) {
            return Err(format!("duplicate source image: {}", path.display()));
        }
        let name = path
            .file_name()
            .ok_or_else(|| format!("source has no filename: {}", path.display()))?
            .to_os_string();
        let display_name = name.to_string_lossy().into_owned();
        if !names.insert(display_name.to_lowercase()) {
            return Err(format!("duplicate source basename: {display_name}"));
        }
        let bytes = fs::read(&path)
            .map_err(|e| format!("failed to read source {}: {e}", path.display()))?;
        let sha256 = sha256_hex(&bytes);
        let expected = &manifest.disks[index];
        if expected.source_name != display_name
            || expected.source_size != bytes.len()
            || !expected.source_sha256.eq_ignore_ascii_case(&sha256)
        {
            return Err(format!(
                "source order/name/size/SHA-256 mismatch for disk {} ({display_name})",
                index + 1
            ));
        }
        result.push(BatchSource {
            path,
            name,
            display_name,
            sha256,
        });
    }
    Ok(result)
}

fn normalize_batch_output(
    output_directory: &Path,
    original_root: &Path,
    modified_root: &Path,
    sources: &[BatchSource],
) -> Result<PathBuf> {
    if output_directory.exists() {
        return Err(format!(
            "output directory already exists: {}",
            output_directory.display()
        ));
    }
    let file_name = output_directory
        .file_name()
        .ok_or_else(|| "output directory has no filename".to_string())?;
    let parent = output_directory
        .parent()
        .ok_or_else(|| "output directory has no parent".to_string())?;
    let parent =
        fs::canonicalize(parent).map_err(|e| format!("failed to resolve output parent: {e}"))?;
    let output = parent.join(file_name);
    if output.starts_with(original_root) || output.starts_with(modified_root) {
        return Err("output directory cannot be inside a workspace root".into());
    }
    for source in sources {
        if source.path.starts_with(&output) {
            return Err("output directory contains a source image".into());
        }
    }
    Ok(output)
}

fn disk_directory_name(number: usize) -> String {
    format!("disk-{number:02}")
}

fn replacements_from_workspaces(
    original_workspace: &Path,
    modified_workspace: &Path,
) -> Result<BTreeMap<String, Vec<u8>>> {
    if !original_workspace.is_dir() || !modified_workspace.is_dir() {
        return Err("per-disk original and modified workspaces must be directories".into());
    }
    let original_files = collect_workspace_files(original_workspace)?;
    let modified_files = collect_workspace_files(modified_workspace)?;
    if original_files != modified_files {
        return Err(format!(
            "modified workspace file set differs from original for {}",
            original_workspace.display()
        ));
    }
    let mut replacements = BTreeMap::new();
    for relative in modified_files {
        let path = modified_workspace.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        let data =
            fs::read(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        replacements.insert(relative, data);
    }
    Ok(replacements)
}

fn allocate_batch_stage(parent: &Path, output_name: Option<&std::ffi::OsStr>) -> Result<PathBuf> {
    let output_name = output_name
        .map(|value| value.to_string_lossy())
        .filter(|value| !value.is_empty())
        .unwrap_or(std::borrow::Cow::Borrowed("d88"));
    for attempt in 0..100u32 {
        let candidate = parent.join(format!(
            ".{output_name}.chrome-paradise-batch-{}-{attempt}",
            std::process::id()
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "failed to create batch staging directory {}: {error}",
                    candidate.display()
                ));
            }
        }
    }
    Err("unable to allocate a unique batch staging directory".into())
}

fn read_manifest(workspace: &Path) -> Result<ExportManifest> {
    let path = workspace.join(".fivec_manifest.json");
    let bytes = fs::read(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("invalid fivec manifest: {e}"))
}

fn validate_source_manifest(
    source: &[u8],
    manifest: &ExportManifest,
    source_sha256: &str,
) -> Result<()> {
    if manifest.schema != EXPORT_SCHEMA {
        return Err(format!(
            "unsupported export manifest schema: {}",
            manifest.schema
        ));
    }
    if manifest.inspection.schema != INSPECTION_SCHEMA {
        return Err(format!(
            "unsupported inspection schema: {}",
            manifest.inspection.schema
        ));
    }
    if manifest.inspection.format != "d88" {
        return Err(format!(
            "manifest source format is not D88: {}",
            manifest.inspection.format
        ));
    }
    if manifest.inspection.source_size != source.len()
        || !manifest
            .inspection
            .source_sha256
            .eq_ignore_ascii_case(source_sha256)
    {
        return Err("source image does not match manifest size or SHA-256".into());
    }
    if source.len() != EXPECTED_SOURCE_SIZE {
        return Err(format!("unsupported D88 source size: {}", source.len()));
    }
    if manifest.inspection.source_name.is_empty() {
        return Err("manifest source_name is empty".into());
    }
    Ok(())
}

fn decode_fixed_d88(source: &[u8]) -> Result<(vn_d88::Image, LinearView)> {
    let image = StandardCodec
        .decode(source)
        .map_err(|e| format!("D88 decode failed: {e}"))?;
    if image.disks.len() != 1 || image.trailing_range.is_some() {
        return Err("rebuild requires one complete D88 disk without trailing bytes".into());
    }
    let disk = &image.disks[0];
    if disk.tracks.len() != EXPECTED_TRACKS
        || disk
            .tracks
            .iter()
            .map(|track| track.sectors.len())
            .sum::<usize>()
            != EXPECTED_SECTORS
    {
        return Err("D88 geometry is not 77 cylinders x 2 heads x 8 sectors".into());
    }
    for (slot, track) in disk.tracks.iter().enumerate() {
        if track.sectors.len() != EXPECTED_SECTORS_PER_TRACK {
            return Err(format!("D88 track slot {slot} does not contain 8 sectors"));
        }
        for (ordinal, sector) in track.sectors.iter().enumerate() {
            let id = sector.address.id;
            if sector.data_range.len() != EXPECTED_BYTES_PER_SECTOR
                || id.record as usize != ordinal + 1
                || id.cylinder as usize != slot / 2
                || id.head as usize != slot % 2
            {
                return Err(format!(
                    "D88 track slot {slot} has nonstandard CHRN geometry"
                ));
            }
            if sector.fdc_status != 0 || sector.deleted_data != 0 {
                return Err(format!("D88 track slot {slot} has FDC/deleted-data flags"));
            }
        }
    }
    let view = image
        .physical_view(0)
        .map_err(|e| format!("D88 physical view failed: {e}"))?;
    if view.len() != EXPECTED_LOGICAL_SIZE {
        return Err(format!("unexpected D88 logical size: {}", view.len()));
    }
    Ok((image, view))
}

fn select_volume(manifest: &ExportManifest) -> Result<&VolumeManifest> {
    let supported: Vec<_> = manifest
        .inspection
        .volumes
        .iter()
        .filter(|volume| volume.filesystem.is_some())
        .collect();
    if supported.len() != 1 {
        return Err(format!(
            "manifest must contain exactly one supported volume, found {}",
            supported.len()
        ));
    }
    let volume = supported[0];
    if volume.disk_index != 0 || volume.partition_index.is_some() {
        return Err("manifest volume must be the whole first D88 disk".into());
    }
    Ok(volume)
}

fn validate_layout(
    source: &[u8],
    view: &LinearView,
    volume: &VolumeManifest,
    fat12: &Fat12Manifest,
) -> Result<Layout> {
    if volume.logical_byte_offset + volume.logical_byte_length > view.len()
        || volume.logical_byte_length != EXPECTED_LOGICAL_SIZE
    {
        return Err("manifest volume range does not match the D88 logical view".into());
    }
    if fat12.bytes_per_sector != EXPECTED_BYTES_PER_SECTOR
        || fat12.sectors_per_cluster != 1
        || fat12.reserved_sectors != 1
        || fat12.fat_copies != 2
        || fat12.sectors_per_fat != 2
        || fat12.root_entries != 192
        || fat12.total_sectors != EXPECTED_SECTORS
        || fat12.first_data_sector != 11
        || fat12.data_clusters != 1221
        || !fat12.fat_copies_identical
    {
        return Err("manifest FAT12 layout is not the supported Chrome Paradise geometry".into());
    }
    let bpb = read_logical(source, view, volume.logical_byte_offset, 64)?;
    let bps = usize::from(read_u16(&bpb, 11)?);
    let spc = usize::from(bpb[13]);
    let reserved = usize::from(read_u16(&bpb, 14)?);
    let copies = usize::from(bpb[16]);
    let root_entries = usize::from(read_u16(&bpb, 17)?);
    let total = usize::from(read_u16(&bpb, 19)?);
    let spf = usize::from(read_u16(&bpb, 22)?);
    if (bps, spc, reserved, copies, root_entries, total, spf)
        != (EXPECTED_BYTES_PER_SECTOR, 1, 1, 2, 192, EXPECTED_SECTORS, 2)
    {
        return Err("source BPB does not match the strict FAT12 manifest".into());
    }
    let root_sectors = (root_entries * 32).div_ceil(bps);
    let first_data_sector = reserved + copies * spf + root_sectors;
    let data_clusters = (total - first_data_sector) / spc;
    let volume_base = volume.logical_byte_offset;
    let fat_offset = volume_base + reserved * bps;
    let fat_bytes = spf * bps;
    let root_offset = volume_base + (reserved + copies * spf) * bps;
    let root_bytes = root_entries * 32;
    let cluster_bytes = spc * bps;
    let max_cluster =
        u16::try_from(data_clusters + 1).map_err(|_| "FAT12 cluster overflow".to_string())?;
    if volume.logical_byte_length != total * bps {
        return Err("source BPB total sectors do not match volume length".into());
    }
    Ok(Layout {
        volume_base,
        volume_bytes: volume.logical_byte_length,
        bytes_per_sector: bps,
        fat_copies: copies,
        first_data_sector,
        fat_offset,
        fat_bytes,
        root_offset,
        root_bytes,
        cluster_bytes,
        max_cluster,
    })
}

fn index_export_files<'a>(
    files: &'a [ExportFileManifest],
    volume_id: &str,
) -> Result<HashMap<String, &'a ExportFileManifest>> {
    let mut result = HashMap::new();
    for file in files {
        if file.volume_id != volume_id || !file.path.starts_with(&(volume_id.to_owned() + "/")) {
            return Err(format!(
                "manifest file {} belongs to the wrong volume",
                file.path
            ));
        }
        safe_relative(&file.path)?;
        if result.insert(file.entry_id.clone(), file).is_some() {
            return Err(format!("duplicate manifest entry id: {}", file.entry_id));
        }
    }
    Ok(result)
}

fn validate_workspace_files(
    workspace: &Path,
    volume: &VolumeManifest,
    fs_manifest: &[FsFileManifest],
    top_files: &HashMap<String, &ExportFileManifest>,
) -> Result<Vec<FilePlan>> {
    let actual = collect_workspace_files(workspace)?;
    let expected: BTreeSet<String> = fs_manifest
        .iter()
        .map(|file| format!("{}/{}", volume.id, file.export_path))
        .collect();
    let actual_set: BTreeSet<String> = actual.iter().cloned().collect();
    if expected != actual_set {
        let extra: Vec<_> = actual_set.difference(&expected).cloned().collect();
        let missing: Vec<_> = expected.difference(&actual_set).cloned().collect();
        return Err(format!(
            "workspace files differ from manifest (extra={extra:?}, missing={missing:?})"
        ));
    }
    let mut plans = Vec::with_capacity(fs_manifest.len());
    let mut ids = HashSet::new();
    let mut paths = HashSet::new();
    for file in fs_manifest {
        if file.attributes != 0x20 {
            return Err(format!("{} is not an ordinary file", file.export_path));
        }
        if !ids.insert(file.id.clone()) || !paths.insert(file.export_path.to_uppercase()) {
            return Err("manifest contains duplicate file entries".into());
        }
        let path = format!("{}/{}", volume.id, file.export_path);
        safe_relative(&path)?;
        let top = top_files
            .get(&file.id)
            .ok_or_else(|| format!("manifest export file missing entry {}", file.id))?;
        if top.path != path
            || top.size != file.size
            || !top.sha256.eq_ignore_ascii_case(&file.sha256)
        {
            return Err(format!("manifest metadata mismatch for {path}"));
        }
        let host = workspace.join(path.replace('/', std::path::MAIN_SEPARATOR_STR));
        let original =
            fs::read(&host).map_err(|e| format!("failed to read {}: {e}", host.display()))?;
        if original.len() != file.size || !sha256_hex(&original).eq_ignore_ascii_case(&file.sha256)
        {
            return Err(format!("workspace member hash mismatch: {path}"));
        }
        let raw_hex = file
            .raw_name_components_hex
            .last()
            .ok_or_else(|| format!("{path} has no raw short name"))?;
        if decode_hex(raw_hex)?.len() != 11 {
            return Err(format!("{path} raw short name is not 11 bytes"));
        }
        plans.push(FilePlan {
            path,
            raw_name: decode_hex(raw_hex)?,
            directory_entry_offset: file.directory_entry_offset,
            old_chain: file
                .clusters
                .iter()
                .map(|cluster| {
                    u16::try_from(*cluster).map_err(|_| "cluster number exceeds u16".to_string())
                })
                .collect::<Result<Vec<_>>>()?,
            desired_chain: Vec::new(),
            original: original.clone(),
            data: original,
        });
    }
    if top_files.len() != plans.len() {
        return Err("manifest export file list does not match FAT12 file list".into());
    }
    Ok(plans)
}

fn collect_workspace_files(workspace: &Path) -> Result<Vec<String>> {
    let mut result = Vec::new();
    let mut stack = vec![workspace.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|e| format!("failed to read {}: {e}", directory.display()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        for entry in entries {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if metadata.file_type().is_symlink() {
                return Err(format!(
                    "workspace links are not allowed: {}",
                    path.display()
                ));
            }
            if metadata.is_dir() {
                stack.push(path);
            } else if metadata.is_file() {
                let relative = path
                    .strip_prefix(workspace)
                    .map_err(|_| "workspace path escape".to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                if relative != ".fivec_manifest.json" {
                    result.push(relative);
                }
            } else {
                return Err(format!(
                    "workspace contains a special file: {}",
                    path.display()
                ));
            }
        }
    }
    result.sort();
    Ok(result)
}

fn validate_replacement_keys(
    replacements: &BTreeMap<String, Vec<u8>>,
    files: &HashMap<String, &ExportFileManifest>,
) -> Result<()> {
    let expected: HashSet<_> = files.values().map(|file| file.path.as_str()).collect();
    for path in replacements.keys() {
        safe_relative(path)?;
        if !expected.contains(path.as_str()) {
            return Err(format!("replacement path is not in manifest: {path}"));
        }
    }
    Ok(())
}

fn validate_fat(fat: &[u8], info: &Fat12Manifest, max_cluster: u16) -> Result<()> {
    if fat.len() != info.sectors_per_fat * info.bytes_per_sector {
        return Err("FAT size mismatch".into());
    }
    if fat[0] < 0xf0
        || fat12_next(fat, 0)? != (0xf00 | u16::from(fat[0]))
        || fat12_next(fat, 1)? < 0xff8
    {
        return Err("FAT12 reserved entries are invalid".into());
    }
    fat12_next(fat, max_cluster)?;
    Ok(())
}

fn read_chain(
    fat: &[u8],
    max_cluster: u16,
    start: u16,
    path: &str,
    owners: &mut HashMap<u16, String>,
) -> Result<Vec<u16>> {
    if start == 0 {
        return Ok(Vec::new());
    }
    let mut chain = Vec::new();
    let mut seen = HashSet::new();
    let mut cluster = start;
    loop {
        if cluster < 2 || cluster > max_cluster || cluster >= 0xff0 {
            return Err(format!("{path} has out-of-range cluster {cluster:#x}"));
        }
        if !seen.insert(cluster) {
            return Err(format!("{path} FAT12 chain loops at cluster {cluster}"));
        }
        if let Some(previous) = owners.insert(cluster, path.to_string()) {
            return Err(format!("{path} is cross-linked with {previous}"));
        }
        chain.push(cluster);
        match fat12_next(fat, cluster)? {
            0xff8..=0xfff => return Ok(chain),
            0xff7 => return Err(format!("{path} reaches a bad cluster")),
            0xff0..=0xff6 => return Err(format!("{path} reaches a reserved cluster marker")),
            0 | 1 => return Err(format!("{path} reaches a free cluster marker")),
            next => cluster = next,
        }
    }
}

fn reject_orphan_clusters(
    fat: &[u8],
    max_cluster: u16,
    owners: &HashMap<u16, String>,
) -> Result<()> {
    for cluster in 2..=max_cluster {
        let next = fat12_next(fat, cluster)?;
        if next != 0 && next != 0xff7 && !owners.contains_key(&cluster) {
            return Err(format!(
                "allocated orphan cluster {cluster} is not represented in manifest"
            ));
        }
    }
    Ok(())
}

fn validate_rebuilt(
    rebuilt: &[u8],
    view: &LinearView,
    layout: &Layout,
    files: &[FilePlan],
    changed: usize,
    source_sha256: &str,
) -> Result<()> {
    if rebuilt.len() != EXPECTED_SOURCE_SIZE {
        return Err("rebuilt D88 size changed".into());
    }
    let fat = read_logical(rebuilt, view, layout.fat_offset, layout.fat_bytes)?;
    let mut owners = HashMap::new();
    for plan in files {
        let start = read_u16(
            &read_logical(
                rebuilt,
                view,
                layout.volume_base + plan.directory_entry_offset + 26,
                2,
            )?,
            0,
        )?;
        let actual = read_chain(&fat, layout.max_cluster, start, &plan.path, &mut owners)?;
        let expected = if plan.data.is_empty() {
            0
        } else {
            cluster_count(plan.data.len(), layout.cluster_bytes)
        };
        if actual.len() != expected {
            return Err(format!("rebuilt {} chain length mismatch", plan.path));
        }
        let mut data = Vec::with_capacity(plan.data.len());
        for &cluster in &actual {
            let offset = cluster_offset(layout, cluster)?;
            let count = (plan.data.len() - data.len()).min(layout.cluster_bytes);
            data.extend_from_slice(&read_logical(rebuilt, view, offset, count)?);
        }
        if data != plan.data {
            return Err(format!("rebuilt {} content mismatch", plan.path));
        }
    }
    if changed == 0 {
        return Err("internal changed-file accounting mismatch".into());
    }
    let rebuilt_sha = sha256_hex(rebuilt);
    if rebuilt_sha == source_sha256 {
        return Err("changed rebuild produced byte-identical output".into());
    }
    Ok(())
}

fn patch_logical(view: &LinearView, output: &mut [u8], offset: usize, data: &[u8]) -> Result<()> {
    let mapped = view
        .map_range(offset..offset + data.len())
        .map_err(|e| e.to_string())?;
    let mut cursor = 0usize;
    for span in mapped {
        let len = span.source_range.len();
        let end = cursor + len;
        if end > data.len() || span.source_range.end > output.len() {
            return Err("logical patch range exceeds source image".into());
        }
        output[span.source_range].copy_from_slice(&data[cursor..end]);
        cursor = end;
    }
    if cursor != data.len() {
        return Err("logical view does not cover the complete patch".into());
    }
    Ok(())
}

fn read_logical(source: &[u8], view: &LinearView, offset: usize, length: usize) -> Result<Vec<u8>> {
    let mapped = view
        .map_range(offset..offset + length)
        .map_err(|e| e.to_string())?;
    let mut data = Vec::with_capacity(length);
    for span in mapped {
        if span.source_range.end > source.len() {
            return Err("logical view source range exceeds image".into());
        }
        data.extend_from_slice(&source[span.source_range]);
    }
    if data.len() != length {
        return Err("logical view does not cover requested bytes".into());
    }
    Ok(data)
}

fn cluster_offset(layout: &Layout, cluster: u16) -> Result<usize> {
    if cluster < 2 || cluster > layout.max_cluster {
        return Err(format!("cluster {cluster} outside FAT12 data range"));
    }
    let offset = layout
        .volume_base
        .checked_add(layout.first_data_sector * layout.bytes_per_sector)
        .and_then(|value| value.checked_add((usize::from(cluster) - 2) * layout.cluster_bytes))
        .ok_or_else(|| "cluster offset overflow".to_string())?;
    if offset + layout.cluster_bytes > layout.volume_base + layout.volume_bytes {
        return Err(format!("cluster {cluster} exceeds FAT12 volume"));
    }
    Ok(offset)
}

fn fat12_next(fat: &[u8], cluster: u16) -> Result<u16> {
    let offset = usize::from(cluster) * 3 / 2;
    let pair = fat
        .get(offset..offset + 2)
        .ok_or_else(|| format!("FAT has no slot for cluster {cluster}"))?;
    let packed = u16::from_le_bytes([pair[0], pair[1]]);
    Ok(if cluster & 1 == 0 {
        packed & 0x0fff
    } else {
        packed >> 4
    })
}

fn set_fat12(fat: &mut [u8], cluster: u16, value: u16) -> Result<()> {
    if value > 0x0fff {
        return Err(format!("FAT12 value {value:#x} is too large"));
    }
    let offset = usize::from(cluster) * 3 / 2;
    let pair = fat
        .get_mut(offset..offset + 2)
        .ok_or_else(|| format!("FAT has no slot for cluster {cluster}"))?;
    let old = u16::from_le_bytes([pair[0], pair[1]]);
    let packed = if cluster & 1 == 0 {
        (old & 0xf000) | value
    } else {
        (old & 0x000f) | (value << 4)
    };
    pair.copy_from_slice(&packed.to_le_bytes());
    Ok(())
}

fn cluster_count(size: usize, cluster_bytes: usize) -> usize {
    size.div_ceil(cluster_bytes)
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let raw = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| format!("u16 read at {offset:#x} is out of bounds"))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn decode_hex(value: &str) -> Result<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return Err(format!("odd-length hex value: {value}"));
    }
    (0..value.len())
        .step_by(2)
        .map(|offset| {
            u8::from_str_radix(&value[offset..offset + 2], 16)
                .map_err(|_| format!("invalid hex value: {value}"))
        })
        .collect()
}

fn safe_relative(value: &str) -> Result<()> {
    if value.is_empty() || value.contains('\\') {
        return Err(format!("unsafe manifest path: {value}"));
    }
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe manifest path: {value}"));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_rebuild_requires_at_least_one_source() {
        let result = rebuild_d88_batch_from_workspaces(
            &[],
            Path::new("missing-original"),
            Path::new("missing-modified"),
            Path::new("missing-output"),
        );
        assert!(matches!(
            result,
            Err(message) if message == "至少需要一张 D88 输入镜像"
        ));
    }

    #[test]
    fn disk_directory_names_are_one_based_and_zero_padded() {
        assert_eq!(disk_directory_name(1), "disk-01");
        assert_eq!(disk_directory_name(8), "disk-08");
        assert_eq!(disk_directory_name(12), "disk-12");
    }
}
