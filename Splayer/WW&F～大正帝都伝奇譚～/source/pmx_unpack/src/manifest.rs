use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{PmxError, Result};
use crate::format::ARCHIVE_NAME_SIZE;

pub(crate) const MANIFEST_NAME: &str = "_pmx_manifest.json";
pub(crate) const RAW_PAYLOAD_NAME: &str = "_payload.bin";
pub(crate) const MANIFEST_FORMAT: &str = "splayer_pmx";
pub(crate) const MANIFEST_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Manifest {
    pub format: String,
    pub manifest_version: u32,
    pub source_file: String,
    pub source_size: usize,
    pub decoded_size: usize,
    pub xor_key: u8,
    pub compression: String,
    pub payload_kind: String,
    pub payload_file: Option<String>,
    pub table_size: Option<usize>,
    pub entries: Vec<ManifestEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ManifestEntry {
    pub index: usize,
    pub path: String,
    pub record_offset: usize,
    pub data_offset: usize,
    pub size: usize,
    pub name_slot_hex: String,
}

pub(crate) fn read_manifest(directory: &Path) -> Result<Manifest> {
    let path = directory.join(MANIFEST_NAME);
    let file = File::open(&path)
        .map_err(|error| PmxError::context(format!("读取清单失败：{}", path.display()), error))?;
    serde_json::from_reader(BufReader::new(file))
        .map_err(|error| PmxError::context(format!("解析清单失败：{}", path.display()), error))
}

pub(crate) fn write_manifest(directory: &Path, manifest: &Manifest) -> Result<()> {
    let path = directory.join(MANIFEST_NAME);
    let file = File::create(&path)
        .map_err(|error| PmxError::context(format!("创建清单失败：{}", path.display()), error))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, manifest)
        .map_err(|error| PmxError::context(format!("写入清单失败：{}", path.display()), error))?;
    writer
        .write_all(b"\n")
        .and_then(|_| writer.flush())
        .map_err(|error| PmxError::context(format!("刷新清单失败：{}", path.display()), error))
}

pub(crate) fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

pub(crate) fn decode_name_slot_hex(value: &str) -> Result<[u8; ARCHIVE_NAME_SIZE]> {
    if value.len() != ARCHIVE_NAME_SIZE * 2 || !value.is_ascii() {
        return Err(PmxError::new(format!(
            "name_slot_hex 必须是 {} 个 ASCII 十六进制字符",
            ARCHIVE_NAME_SIZE * 2
        )));
    }

    let mut output = [0u8; ARCHIVE_NAME_SIZE];
    for (index, byte) in output.iter_mut().enumerate() {
        let start = index * 2;
        *byte = u8::from_str_radix(&value[start..start + 2], 16).map_err(|_| {
            PmxError::new(format!("name_slot_hex 在字符 {start} 处不是有效十六进制"))
        })?;
    }
    Ok(output)
}
