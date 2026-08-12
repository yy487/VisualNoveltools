use serde::{Deserialize, Serialize};

pub const FORMAT_ID: &str = "farthest2015-cd-json-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationFile {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_scenario_id")]
    pub scenario_id: u32,
    #[serde(rename = "_title")]
    pub title: String,
    #[serde(rename = "_source_md5")]
    pub source_md5: String,
    pub entries: Vec<TranslationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: u32,
    #[serde(rename = "_offset")]
    pub offset: u64,
    #[serde(rename = "_inst_offset")]
    pub inst_offset: u64,
    #[serde(rename = "_size")]
    pub size: u32,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_opcode")]
    pub opcode: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_policy")]
    pub policy: String,
    #[serde(rename = "_flags")]
    pub flags: u16,
    #[serde(rename = "_choice_index", skip_serializing_if = "Option::is_none")]
    pub choice_index: Option<u16>,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    pub scr_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub scr_msg: String,
    pub message: String,
    #[serde(rename = "_links", default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<LinkMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinkMeta {
    #[serde(rename = "_id")]
    pub id: u32,
    #[serde(rename = "_inst_offset")]
    pub inst_offset: u64,
    #[serde(rename = "_target_scenario")]
    pub target_scenario: u32,
    #[serde(rename = "_target_entry")]
    pub target_entry: u32,
    #[serde(rename = "_target_code_offset")]
    pub target_code_offset: u32,
    #[serde(rename = "_source_line")]
    pub source_line: u16,
    #[serde(rename = "_source_first")]
    pub source_first: u16,
    #[serde(rename = "_source_len")]
    pub source_len: u16,
    #[serde(rename = "_inline")]
    pub inline: bool,
}
