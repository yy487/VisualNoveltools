use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpeakerMap {
    pub format: String,
    pub context_only: bool,
    pub resource_pattern: String,
    pub entries: BTreeMap<String, SpeakerRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpeakerRecord {
    pub name: Option<String>,
    pub nameplate_text: String,
    pub resource: String,
}

impl SpeakerMap {
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "sbridge-speaker-map-v1" {
            return Err(format!("unsupported speaker map format {:?}", self.format));
        }
        if !self.context_only {
            return Err("speaker map must be marked context_only".to_owned());
        }
        for (id, record) in &self.entries {
            if id.len() != 3 || !id.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(format!("speaker map key {id:?} is not a three-digit ID"));
            }
            if record.resource != format!("gf{id}.wcg") {
                return Err(format!(
                    "speaker map ID {id} points to unexpected resource {:?}",
                    record.resource
                ));
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn name(&self, id: u16) -> Option<&str> {
        self.entries
            .get(&format!("{id:03}"))
            .and_then(|entry| entry.name.as_deref())
    }
}
