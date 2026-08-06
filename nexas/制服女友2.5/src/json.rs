use crate::ToolResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranslationEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub scr_msg: String,
    pub message: String,

    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    pub scr_name: Option<String>,
    #[serde(rename = "_scr_raw")]
    pub scr_raw: String,
    #[serde(
        rename = "_control_prefix",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub control_prefix: String,
    #[serde(
        rename = "_control_suffix",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub control_suffix: String,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_body_size")]
    pub body_size: usize,
    #[serde(rename = "_scope")]
    pub scope: String,
    #[serde(rename = "_function_index", skip_serializing_if = "Option::is_none")]
    pub function_index: Option<usize>,
    #[serde(rename = "_function_id", skip_serializing_if = "Option::is_none")]
    pub function_id: Option<i32>,
    #[serde(rename = "_string_index")]
    pub string_index: usize,
    #[serde(rename = "_name_scope", skip_serializing_if = "Option::is_none")]
    pub name_scope: Option<String>,
    #[serde(
        rename = "_name_function_index",
        skip_serializing_if = "Option::is_none"
    )]
    pub name_function_index: Option<usize>,
    #[serde(rename = "_name_function_id", skip_serializing_if = "Option::is_none")]
    pub name_function_id: Option<i32>,
    #[serde(rename = "_name_string_index", skip_serializing_if = "Option::is_none")]
    pub name_string_index: Option<usize>,
    #[serde(rename = "_name_offset", skip_serializing_if = "Option::is_none")]
    pub name_offset: Option<usize>,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_opcode")]
    pub opcode: String,
    #[serde(rename = "_rule")]
    pub rule: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_policy")]
    pub policy: String,
    #[serde(rename = "_choice_group", skip_serializing_if = "Option::is_none")]
    pub choice_group: Option<usize>,
    #[serde(rename = "_choice_index", skip_serializing_if = "Option::is_none")]
    pub choice_index: Option<usize>,
}

pub fn read_entries(path: &Path) -> ToolResult<Vec<TranslationEntry>> {
    let text = fs::read_to_string(path)
        .map_err(|error| crate::error(format!("{}: cannot read JSON: {error}", path.display())))?;
    serde_json::from_str(&text).map_err(|error| {
        crate::error(format!(
            "{}: invalid UTF-8 JSON array: {error}",
            path.display()
        ))
    })
}

pub fn write_entries(path: &Path, entries: &[TranslationEntry]) -> ToolResult<()> {
    let text = serde_json::to_string_pretty(entries)? + "\n";
    fs::write(path, text.as_bytes()).map_err(|io_error| {
        crate::error(format!(
            "{}: cannot write UTF-8 JSON: {io_error}",
            path.display()
        ))
    })
}
