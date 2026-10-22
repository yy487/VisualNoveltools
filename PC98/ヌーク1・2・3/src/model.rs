use serde::{Deserialize, Serialize};

pub const WORKSPACE_FORMAT: &str = "nooch-trilogy-localization-workspace-v2";
pub const LEGACY_WORKSPACE_FORMAT: &str = "nooch-trilogy-localization-workspace-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameId {
    Nooch,
    Nooch2,
    Nooch3,
}

impl GameId {
    pub const ALL: [Self; 3] = [Self::Nooch, Self::Nooch2, Self::Nooch3];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nooch => "nooch",
            Self::Nooch2 => "nooch2",
            Self::Nooch3 => "nooch3",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    pub _format: String,
    pub tool_version: String,
    pub input_root: String,
    pub games: Vec<GameManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameManifest {
    pub game: GameId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messages: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub translations: Vec<TranslationFileManifest>,
    pub disks: Vec<DiskManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationFileManifest {
    pub source_file: String,
    pub source_disk_sha256: String,
    pub member: String,
    pub json: String,
    pub entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskManifest {
    pub disk_index: usize,
    pub source_relative: String,
    pub source_file: String,
    pub source_sha256: String,
    pub extracted_directory: String,
    pub files: Vec<FileManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManifest {
    pub path: String,
    pub size: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationEntry {
    pub _file: String,
    pub _index: usize,
    pub _kind: String,
    pub _ref: usize,
    pub _code_word: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _scr_name: Option<String>,
    pub scr_msg: String,
    pub message: String,
    pub _disk_sha256: String,
    pub _member: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _span: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _scene: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _record: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _script: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _message: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _slot_offset: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _capacity: Option<usize>,
}

impl TranslationEntry {
    pub fn stable_key(&self) -> String {
        format!(
            "{}|{}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
            self._disk_sha256,
            self._member.to_ascii_uppercase(),
            self._line,
            self._span,
            self._record,
            self._script,
            self._message,
            self._slot_offset
        )
    }
}
