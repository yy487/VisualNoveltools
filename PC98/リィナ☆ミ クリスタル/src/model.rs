use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationFile {
    pub schema: String,
    pub source_file: String,
    pub entries: Vec<TranslationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub kind: EntryKind,
    #[serde(rename = "_opcode")]
    pub opcode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    pub scr_name: Option<String>,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
    Message,
    Choice,
}
