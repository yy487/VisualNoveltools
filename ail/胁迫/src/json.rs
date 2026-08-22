use crate::controls::{render_parts, validate_translated_text};
use crate::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_inst_offset")]
    pub inst_offset: usize,
    #[serde(rename = "_ref_offset")]
    pub ref_offset: usize,
    #[serde(rename = "_target")]
    pub target: u16,
    #[serde(rename = "_opcode")]
    pub opcode: String,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_raw_hex")]
    pub raw_hex: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,

    #[serde(
        rename = "_name_inst_offset",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub name_inst_offset: Option<usize>,
    #[serde(
        rename = "_name_ref_offset",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub name_ref_offset: Option<usize>,
    #[serde(
        rename = "_name_target",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub name_target: Option<u16>,
    #[serde(
        rename = "_name_opcode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub name_opcode: Option<String>,
    #[serde(
        rename = "_name_raw_hex",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub name_raw_hex: Option<String>,
    #[serde(rename = "_scr_name", default, skip_serializing_if = "Option::is_none")]
    pub scr_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "scr_msg", alias = "scr-msg")]
    pub scr_msg: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scr_msg_parts: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_parts: Option<Vec<String>>,
    #[serde(
        rename = "_format_controls",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub format_controls: Vec<String>,
}

impl JsonEntry {
    pub fn translated_name(&self) -> Result<Option<&str>> {
        match (&self.scr_name, &self.name) {
            (None, None) => Ok(None),
            (Some(_), Some(name)) => Ok(Some(name.as_str())),
            (Some(_), None) => Err(format!(
                "{} entry {} has _scr_name but no name",
                self.file, self.index
            )),
            (None, Some(_)) => Err(format!(
                "{} entry {} has name but no _scr_name",
                self.file, self.index
            )),
        }
    }

    pub fn rendered_message(&self) -> Result<String> {
        if self.scr_name.is_some() {
            if !self.message.is_empty() || !self.scr_msg.is_empty() {
                return Err(format!(
                    "{} entry {} is a name entry and must have empty message fields",
                    self.file, self.index
                ));
            }
            return Ok(String::new());
        }
        if self.format_controls.is_empty() {
            if self.message_parts.is_some() {
                return Err(format!(
                    "{} entry {} has message_parts without hidden controls",
                    self.file, self.index
                ));
            }
            validate_translated_text(&self.message)
                .map_err(|err| format!("{} entry {} message: {err}", self.file, self.index))?;
            return Ok(self.message.clone());
        }
        let parts = self.message_parts.as_ref().ok_or_else(|| {
            format!(
                "{} entry {} requires message_parts for hidden controls",
                self.file, self.index
            )
        })?;
        if parts.concat() != self.message {
            return Err(format!(
                "{} entry {} message must equal the concatenation of message_parts",
                self.file, self.index
            ));
        }
        for part in parts {
            validate_translated_text(part).map_err(|err| {
                format!("{} entry {} message_parts: {err}", self.file, self.index)
            })?;
        }
        render_parts(parts, &self.format_controls)
            .map_err(|err| format!("{} entry {}: {err}", self.file, self.index))
    }

    pub fn is_unchanged(&self) -> bool {
        let name_unchanged = match (&self.scr_name, &self.name) {
            (None, None) => true,
            (Some(source), Some(translated)) => source == translated,
            _ => false,
        };
        let message_unchanged = self.message == self.scr_msg
            && match (&self.scr_msg_parts, &self.message_parts) {
                (None, None) => true,
                (Some(source), Some(translated)) => source == translated,
                _ => false,
            };
        name_unchanged && message_unchanged
    }
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut out, "{byte:02X}").expect("writing to String cannot fail");
    }
    out
}

pub fn hex_to_bytes(text: &str) -> Result<Vec<u8>> {
    let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    if !compact.len().is_multiple_of(2) {
        return Err("hex string has odd length".to_string());
    }
    let mut out = Vec::with_capacity(compact.len() / 2);
    for index in (0..compact.len()).step_by(2) {
        out.push(
            u8::from_str_radix(&compact[index..index + 2], 16)
                .map_err(|err| format!("invalid hex byte at {index}: {err}"))?,
        );
    }
    Ok(out)
}
