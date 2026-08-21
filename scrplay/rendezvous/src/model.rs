use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: u64,
    #[serde(rename = "_string_offset")]
    pub string_offset: u32,
    #[serde(rename = "_inst_offset")]
    pub instruction_offset: u64,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub entry_type: &'static str,
    #[serde(rename = "_opcode")]
    pub opcode: u8,
    #[serde(rename = "_encoding")]
    pub encoding: &'static str,
    #[serde(rename = "_policy")]
    pub policy: &'static str,
    #[serde(rename = "_terminator", skip_serializing_if = "Option::is_none")]
    pub terminator: Option<&'static str>,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    pub source_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Summary {
    pub files_scanned: usize,
    pub scripts_parsed: usize,
    pub json_files: usize,
    pub commands: usize,
    pub strings: usize,
    pub entries: usize,
    pub messages: usize,
    pub named_messages: usize,
    pub choices: usize,
    pub internal_newlines: usize,
    pub ruby_controls: usize,
    pub backslashes: usize,
    pub warnings: usize,
}

#[derive(Debug)]
pub struct ExtractedFile {
    pub relative_path: PathBuf,
    pub entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct Extraction {
    pub files: Vec<ExtractedFile>,
    pub summary: Summary,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: u64,
    #[serde(rename = "_string_offset")]
    pub string_offset: u32,
    #[serde(rename = "_inst_offset")]
    pub instruction_offset: u64,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_opcode")]
    pub opcode: u8,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_policy")]
    pub policy: String,
    #[serde(rename = "_terminator")]
    pub terminator: Option<String>,
    #[serde(rename = "_scr_name")]
    pub source_name: Option<String>,
    pub name: Option<String>,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InjectionSummary {
    pub source_files: usize,
    pub scripts: usize,
    pub translation_files: usize,
    pub entries: usize,
    pub changed_entries: usize,
    pub changed_messages: usize,
    pub changed_names: usize,
    pub rebuilt_scripts: usize,
    pub byte_exact_scripts: usize,
}
