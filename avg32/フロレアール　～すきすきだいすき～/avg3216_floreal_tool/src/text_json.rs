use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const FORMAT_ID: &str = "avg3216-floreal-seen-text-v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectProfile {
    pub name_field: bool,
    pub source_encoding: String,
    pub target_encoding: String,
    pub message_rule: String,
    pub extracted_types: Vec<String>,
    pub excluded_types: Vec<String>,
}

impl Default for ProjectProfile {
    fn default() -> Self {
        Self {
            name_field: false,
            source_encoding: "CP932".to_owned(),
            target_encoding: "CP932".to_owned(),
            message_rule: "本作没有 name 标签；台词、旁白和选项均完整写入 message".to_owned(),
            extracted_types: vec!["message".to_owned(), "choice".to_owned()],
            excluded_types: vec![
                "FE 单字节内部字符串".to_owned(),
                "0x60 动态拼接的调试/场景标签".to_owned(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationDocument {
    pub format: String,
    pub source_file: String,
    pub source_size: u64,
    pub source_sha256: String,
    pub profile: ProjectProfile,
    pub entries: Vec<TextEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_file_index")]
    pub file_index: usize,
    #[serde(rename = "_subscript")]
    pub subscript: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_subscript_offset")]
    pub subscript_offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_opcode")]
    pub opcode: String,
    #[serde(rename = "_choice_index", skip_serializing_if = "Option::is_none")]
    pub choice_index: Option<usize>,
    pub scr_msg: String,
    pub message: String,
}

pub fn write_document(path: &Path, document: &TranslationDocument) -> Result<()> {
    let mut json = serde_json::to_string_pretty(document).context("序列化翻译 JSON 失败")?;
    json.push('\n');
    fs::write(path, json.as_bytes()).with_context(|| format!("写入 JSON 失败：{}", path.display()))
}

pub fn read_document(path: &Path) -> Result<TranslationDocument> {
    let data = fs::read(path).with_context(|| format!("读取 JSON 失败：{}", path.display()))?;
    serde_json::from_slice(&data).with_context(|| format!("解析 JSON 失败：{}", path.display()))
}
