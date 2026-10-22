use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

use crate::error::{PmxError, Result};
use crate::script::ScriptText;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TextEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub scr_msg: String,
    pub message: String,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_inst_offset")]
    pub inst_offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_name_offset", skip_serializing_if = "Option::is_none")]
    pub name_offset: Option<usize>,
    #[serde(rename = "_name_size", skip_serializing_if = "Option::is_none")]
    pub name_size: Option<usize>,
    #[serde(rename = "_line")]
    pub line: usize,
    #[serde(rename = "_type")]
    pub kind: String,
    #[serde(rename = "_opcode")]
    pub opcode: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_policy")]
    pub policy: String,
}

impl TextEntry {
    pub fn from_script(file: String, source: &ScriptText) -> Self {
        Self {
            name: source.name.clone(),
            scr_msg: source.message.clone(),
            message: source.message.clone(),
            file,
            index: source.index,
            offset: source.message_offset,
            inst_offset: source.inst_offset,
            size: source.message_end - source.message_offset,
            name_offset: source.name_offset,
            name_size: source
                .name_offset
                .zip(source.name_end)
                .map(|(start, end)| end - start),
            line: source.line,
            kind: if source.name.is_some() {
                "dialogue".to_owned()
            } else {
                "monologue".to_owned()
            },
            opcode: "PutText".to_owned(),
            encoding: "cp932".to_owned(),
            policy: "relocate".to_owned(),
        }
    }
}

pub(crate) fn read_entries(path: &Path) -> Result<Vec<TextEntry>> {
    let file = File::open(path)
        .map_err(|error| PmxError::context(format!("读取 JSON 失败：{}", path.display()), error))?;
    serde_json::from_reader(BufReader::new(file))
        .map_err(|error| PmxError::context(format!("解析 JSON 失败：{}", path.display()), error))
}

pub(crate) fn write_entries(path: &Path, entries: &[TextEntry]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            PmxError::context(format!("创建 JSON 目录失败：{}", parent.display()), error)
        })?;
    }
    let file = File::create(path)
        .map_err(|error| PmxError::context(format!("创建 JSON 失败：{}", path.display()), error))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, entries)
        .map_err(|error| PmxError::context(format!("写入 JSON 失败：{}", path.display()), error))?;
    writer
        .write_all(b"\n")
        .and_then(|_| writer.flush())
        .map_err(|error| PmxError::context(format!("刷新 JSON 失败：{}", path.display()), error))
}

pub(crate) fn portable_relative_path(path: &Path) -> Result<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => parts.push(value.to_string_lossy().into_owned()),
            _ => {
                return Err(PmxError::new(format!(
                    "路径不是安全相对路径：{}",
                    path.display()
                )));
            }
        }
    }
    if parts.is_empty() {
        return Err(PmxError::new("相对路径为空"));
    }
    Ok(parts.join("/"))
}

pub(crate) fn is_pmx(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pmx"))
}

pub(crate) fn is_json(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
}
