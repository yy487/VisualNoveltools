use serde::{Deserialize, Deserializer, Serialize};

pub fn editable_name(source: &str) -> String {
    source
        .strip_prefix('【')
        .and_then(|inner| inner.strip_suffix('】'))
        .unwrap_or(source)
        .to_string()
}

pub fn restore_name_wrappers(name: &str, source: &str) -> String {
    if source.starts_with('【') && source.ends_with('】') {
        format!("【{}】", name)
    } else {
        name.to_string()
    }
}

fn deserialize_editable_name<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.map(|name| editable_name(&name)))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRoles {
    pub source_root: String,
    pub translation_root: String,
    pub script_json_root: String,
    pub system_json_root: String,
    pub names_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalSourceFileManifest {
    pub file_id: usize,
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotFileManifest {
    pub relative_path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationFileManifest {
    pub relative_path: String,
    pub group: String,
    pub file_id: Option<usize>,
    pub file: Option<String>,
    pub entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceStats {
    pub source_files: usize,
    pub script_json_files: usize,
    pub system_json_files: usize,
    pub extracted_entries: usize,
    pub dialogue_entries: usize,
    pub choice_entries: usize,
    pub name_entries: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    pub format: String,
    pub version: u32,
    pub roles: WorkspaceRoles,
    pub source_manifest_format: String,
    pub logical_source_files: Vec<LogicalSourceFileManifest>,
    pub snapshot_files: Vec<SnapshotFileManifest>,
    pub translation_files: Vec<TranslationFileManifest>,
    pub stats: WorkspaceStats,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PuaInfo {
    pub character: String,
    pub codepoint: u32,
    pub cp932_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextReference {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_file_id")]
    pub file_id: usize,
    #[serde(rename = "_inst_offset")]
    pub inst_offset: String,
    #[serde(rename = "_line")]
    pub line: u32,
    #[serde(rename = "_dbg_record")]
    pub dbg_record: usize,
    #[serde(rename = "_type")]
    pub type_name: String,
    #[serde(rename = "_choice_group", skip_serializing_if = "Option::is_none")]
    pub choice_group: Option<usize>,
    #[serde(rename = "_choice_slot", skip_serializing_if = "Option::is_none")]
    pub choice_slot: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslationEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_file_id")]
    pub file_id: usize,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_type")]
    pub type_name: String,
    #[serde(
        default,
        alias = "_scr_name",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_editable_name"
    )]
    pub scr_name: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_editable_name"
    )]
    pub name: Option<String>,
    pub message: String,
    #[serde(alias = "scr_msge")]
    pub scr_msg: String,
    #[serde(rename = "_cstr_id")]
    pub cstr_id: usize,
    #[serde(rename = "_name_cstr_id", skip_serializing_if = "Option::is_none")]
    pub name_cstr_id: Option<usize>,
    #[serde(rename = "_talk_info_id", skip_serializing_if = "Option::is_none")]
    pub talk_info_id: Option<u32>,
    #[serde(rename = "_talk_style", skip_serializing_if = "Option::is_none")]
    pub talk_style: Option<[u32; 2]>,
    #[serde(rename = "_message_id", skip_serializing_if = "Option::is_none")]
    pub message_id: Option<u32>,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_cstr_size")]
    pub cstr_size: usize,
    #[serde(rename = "_inst_offset")]
    pub inst_offset: String,
    #[serde(rename = "_line")]
    pub line: u32,
    #[serde(rename = "_dbg_record")]
    pub dbg_record: usize,
    #[serde(rename = "_opcode")]
    pub opcode: String,
    #[serde(rename = "_target", skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_policy")]
    pub policy: String,
    #[serde(rename = "_choice_group", skip_serializing_if = "Option::is_none")]
    pub choice_group: Option<usize>,
    #[serde(rename = "_choice_slot", skip_serializing_if = "Option::is_none")]
    pub choice_slot: Option<usize>,
    #[serde(rename = "_body_origin", skip_serializing_if = "Option::is_none")]
    pub body_origin: Option<String>,
    #[serde(rename = "_pua", skip_serializing_if = "Vec::is_empty", default)]
    pub pua: Vec<PuaInfo>,
    #[serde(rename = "_refs", skip_serializing_if = "Vec::is_empty", default)]
    pub refs: Vec<TextReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationFile {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_version")]
    pub version: u32,
    #[serde(rename = "_group")]
    pub group: String,
    #[serde(rename = "_file_id", skip_serializing_if = "Option::is_none")]
    pub file_id: Option<usize>,
    #[serde(rename = "_file", skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    pub entries: Vec<TranslationEntry>,
}

pub const WORKSPACE_FORMAT: &str = "AGSI_TEXT_WORKSPACE_V1";
pub const TRANSLATION_FORMAT: &str = "AGSI_TRANSLATION_FILE_V1";
