use serde::{Deserialize, Serialize};

pub const FORMAT_NAME: &str = "yuyake-mes-text-v2";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptJson {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_file")]
    pub file: String,
    pub entries: Vec<TextEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_inst_offset")]
    pub inst_offset: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_payload_offset")]
    pub payload_offset: usize,
    #[serde(rename = "_payload_size")]
    pub payload_size: usize,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_opcode")]
    pub opcode: String,
    #[serde(rename = "_page")]
    pub page: u16,
    #[serde(rename = "_part_index")]
    pub part_index: usize,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_policy")]
    pub policy: String,
    #[serde(rename = "_control_before")]
    pub control_before: String,
    #[serde(rename = "_control_after")]
    pub control_after: String,
    #[serde(rename = "_payload_sha256")]
    pub payload_sha256: String,
    pub scr_msg: String,
    pub message: String,
}
