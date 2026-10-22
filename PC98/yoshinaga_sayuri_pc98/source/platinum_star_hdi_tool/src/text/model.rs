use serde::{Deserialize, Serialize};
use vn_font::font_98::EncodingPlanEntry;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextDocument {
    pub _format: String,
    pub _file: String,
    pub _source_sha256: String,
    pub _source_bytes: u64,
    pub _decoded_sha256: Option<String>,
    pub _decoded_bytes: Option<u64>,
    pub _encoding: String,
    pub entries: Vec<TextEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextEntry {
    pub _file: String,
    pub _index: u64,
    pub _type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _block: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _string_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _string_offset: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _record_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _offset: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _container_offset: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _item_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _page: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _boundary_before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _boundary_after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _source_raw: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub _controls: Vec<TextControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _scr_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextControl {
    /// Unicode scalar position in the cleaned physical source line.
    pub line: u32,
    pub at: u32,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportManifest {
    pub _format: String,
    pub tool_version: String,
    pub source_root: String,
    pub files: Vec<ExportedFile>,
    pub totals: ExportTotals,
    pub policy: ExportPolicy,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportedFile {
    pub source: String,
    pub output: String,
    pub source_sha256: String,
    pub decoded_sha256: Option<String>,
    pub entries: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportTotals {
    pub source_files: u64,
    pub entries: u64,
    pub scenario_blocks: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportPolicy {
    pub editable_fields: Vec<String>,
    pub immutable_source_fields: Vec<String>,
    pub controls: String,
    pub record_granularity: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportManifest {
    pub _format: String,
    pub tool_version: String,
    pub source_root: String,
    pub translation_root: String,
    pub output_files: Vec<ImportedFile>,
    pub changed_entries: u64,
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<FontManifest>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FontManifest {
    pub base_font: String,
    pub base_sha256: String,
    pub output: String,
    pub output_sha256: String,
    pub mapping: String,
    pub patched_glyphs: u64,
    pub reserved_cp932_slots: u64,
    pub face: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FontMappingDocument {
    pub _format: String,
    pub tool_version: String,
    pub base_font_sha256: String,
    pub output_font_sha256: String,
    pub entries: Vec<EncodingPlanEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportedFile {
    pub file: String,
    pub source_sha256: String,
    pub output_sha256: String,
    pub changed_entries: u64,
    pub output_form: String,
}
