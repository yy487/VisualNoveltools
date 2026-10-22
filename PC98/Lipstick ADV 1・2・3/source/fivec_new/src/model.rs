use serde::{Deserialize, Serialize};
use vn_sector_map::LinearView;

#[derive(Debug, Clone, Serialize)]
pub struct Geometry {
    pub cylinders: u32,
    pub heads: u32,
    pub sectors_per_track: u32,
    pub bytes_per_sector: u32,
}

pub(crate) struct Container {
    pub format: String,
    pub disks: Vec<Disk>,
    pub diagnostics: Vec<String>,
}

pub(crate) struct Disk {
    pub index: usize,
    pub name: String,
    pub geometry: Option<Geometry>,
    pub sector_count: usize,
    pub payload_bytes: usize,
    pub view: Option<LinearView>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileSpan {
    pub file_offset: usize,
    pub source_offset: usize,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    /// Stable within the source snapshot; based on the directory entry position.
    pub id: String,
    pub display_path: String,
    /// Safe relative path within the selected volume's export directory.
    pub export_path: String,
    pub raw_name_components_hex: Vec<String>,
    pub attributes: u8,
    pub directory_entry_offset: usize,
    pub directory_entry_source_ranges: Vec<FileSpan>,
    pub size: usize,
    pub sha256: String,
    pub clusters: Vec<u32>,
    pub source_ranges: Vec<FileSpan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DirectoryEntry {
    pub display_path: String,
    pub export_path: String,
    pub raw_name_components_hex: Vec<String>,
    pub directory_entry_offset: usize,
    pub clusters: Vec<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Fat12Info {
    pub bytes_per_sector: usize,
    pub sectors_per_cluster: usize,
    pub reserved_sectors: usize,
    pub fat_copies: usize,
    pub sectors_per_fat: usize,
    pub root_entries: usize,
    pub total_sectors: usize,
    pub first_data_sector: usize,
    pub data_clusters: usize,
    pub fat_copies_identical: bool,
    pub deleted_entries_skipped: usize,
    pub long_name_entries_skipped: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileSystem {
    pub kind: String,
    pub fat12: Fat12Info,
    pub files: Vec<FileEntry>,
    pub directories: Vec<DirectoryEntry>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Volume {
    pub id: String,
    pub disk_index: usize,
    pub partition_index: Option<usize>,
    pub kind: String,
    pub logical_byte_offset: usize,
    pub logical_byte_length: usize,
    pub partition_entry_hex: Option<String>,
    pub filesystem: Option<FileSystem>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskReport {
    pub index: usize,
    pub name: String,
    pub geometry: Option<Geometry>,
    pub sector_count: usize,
    pub payload_bytes: usize,
    pub logical_view_available: bool,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Inspection {
    pub schema: String,
    pub parser_version: String,
    pub source_name: String,
    pub source_size: usize,
    pub source_sha256: String,
    pub format: String,
    pub disks: Vec<DiskReport>,
    pub volumes: Vec<Volume>,
    pub diagnostics: Vec<String>,
}
