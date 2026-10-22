use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{PmxError, Result};
use crate::format::{decode_envelope, parse_payload, Archive, ParsedPayload, XOR_KEY};
use crate::manifest::{
    encode_hex, write_manifest, Manifest, ManifestEntry, MANIFEST_FORMAT, MANIFEST_VERSION,
    RAW_PAYLOAD_NAME,
};

#[derive(Clone, Copy, Debug)]
pub enum PayloadKind {
    DataArchive,
    Raw,
}

impl PayloadKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DataArchive => "data_archive",
            Self::Raw => "raw",
        }
    }
}

#[derive(Debug)]
pub struct UnpackReport {
    pub input_bytes: usize,
    pub decoded_bytes: usize,
    pub extracted_files: usize,
    pub kind: PayloadKind,
    pub output_dir: PathBuf,
}

pub fn default_output_dir(input: &Path) -> Result<PathBuf> {
    let stem = input
        .file_stem()
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| PmxError::new(format!("输入路径缺少有效文件名：{}", input.display())))?;
    let output_name = format!("{}_unpacked", stem.to_string_lossy());
    Ok(input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(output_name))
}

pub fn unpack_file(input: &Path, output_dir: &Path, force_raw: bool) -> Result<UnpackReport> {
    if !input.is_file() {
        return Err(PmxError::new(format!(
            "输入不是普通文件：{}",
            input.display()
        )));
    }
    if output_dir.exists() {
        return Err(PmxError::new(format!(
            "输出目录已存在，为避免覆盖而停止：{}",
            output_dir.display()
        )));
    }

    let encoded = fs::read(input)
        .map_err(|error| PmxError::context(format!("读取失败：{}", input.display()), error))?;
    let decoded = decode_envelope(&encoded)?;
    let parsed = if force_raw {
        ParsedPayload::Raw
    } else {
        parse_payload(&decoded)?
    };

    fs::create_dir(output_dir).map_err(|error| {
        PmxError::context(format!("创建输出目录失败：{}", output_dir.display()), error)
    })?;

    let result = write_output(input, output_dir, &encoded, &decoded, parsed);
    if let Err(error) = result {
        let cleanup_error = fs::remove_dir_all(output_dir).err();
        return match cleanup_error {
            Some(cleanup_error) => Err(PmxError::new(format!(
                "{error}；清理未完成输出目录时又失败：{cleanup_error}"
            ))),
            None => Err(error),
        };
    }
    result
}

fn write_output(
    input: &Path,
    output_dir: &Path,
    encoded: &[u8],
    decoded: &[u8],
    parsed: ParsedPayload,
) -> Result<UnpackReport> {
    let source_file = input
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| input.display().to_string());

    match parsed {
        ParsedPayload::Archive(archive) => {
            write_archive_files(output_dir, decoded, &archive)?;
            let manifest = archive_manifest(source_file, encoded.len(), decoded.len(), &archive);
            write_manifest(output_dir, &manifest)?;
            Ok(UnpackReport {
                input_bytes: encoded.len(),
                decoded_bytes: decoded.len(),
                extracted_files: archive.entries.len(),
                kind: PayloadKind::DataArchive,
                output_dir: output_dir.to_path_buf(),
            })
        }
        ParsedPayload::Raw => {
            let payload_path = output_dir.join(RAW_PAYLOAD_NAME);
            fs::write(&payload_path, decoded).map_err(|error| {
                PmxError::context(format!("写入失败：{}", payload_path.display()), error)
            })?;
            let manifest = raw_manifest(source_file, encoded.len(), decoded.len());
            write_manifest(output_dir, &manifest)?;
            Ok(UnpackReport {
                input_bytes: encoded.len(),
                decoded_bytes: decoded.len(),
                extracted_files: 1,
                kind: PayloadKind::Raw,
                output_dir: output_dir.to_path_buf(),
            })
        }
    }
}

fn write_archive_files(output_dir: &Path, decoded: &[u8], archive: &Archive) -> Result<()> {
    for entry in &archive.entries {
        let end = entry
            .data_offset
            .checked_add(entry.size)
            .ok_or_else(|| PmxError::new(format!("记录 {} 的数据范围溢出", entry.index)))?;
        let data = decoded.get(entry.data_offset..end).ok_or_else(|| {
            PmxError::new(format!(
                "记录 {} 的数据越界：offset={} size={} decoded_size={}",
                entry.index,
                entry.data_offset,
                entry.size,
                decoded.len()
            ))
        })?;
        let target = output_dir.join(&entry.relative_path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                PmxError::context(format!("创建目录失败：{}", parent.display()), error)
            })?;
        }
        fs::write(&target, data)
            .map_err(|error| PmxError::context(format!("写入失败：{}", target.display()), error))?;
    }
    Ok(())
}

fn archive_manifest(
    source_file: String,
    source_size: usize,
    decoded_size: usize,
    archive: &Archive,
) -> Manifest {
    let entries = archive
        .entries
        .iter()
        .map(|entry| ManifestEntry {
            index: entry.index,
            path: entry.manifest_path.clone(),
            record_offset: entry.record_offset,
            data_offset: entry.data_offset,
            size: entry.size,
            name_slot_hex: encode_hex(&entry.name_slot),
        })
        .collect();

    Manifest {
        format: MANIFEST_FORMAT.to_owned(),
        manifest_version: MANIFEST_VERSION,
        source_file,
        source_size,
        decoded_size,
        xor_key: XOR_KEY,
        compression: "zlib".to_owned(),
        payload_kind: PayloadKind::DataArchive.as_str().to_owned(),
        payload_file: None,
        table_size: Some(archive.table_size),
        entries,
    }
}

fn raw_manifest(source_file: String, source_size: usize, decoded_size: usize) -> Manifest {
    Manifest {
        format: MANIFEST_FORMAT.to_owned(),
        manifest_version: MANIFEST_VERSION,
        source_file,
        source_size,
        decoded_size,
        xor_key: XOR_KEY,
        compression: "zlib".to_owned(),
        payload_kind: PayloadKind::Raw.as_str().to_owned(),
        payload_file: Some(RAW_PAYLOAD_NAME.to_owned()),
        table_size: None,
        entries: Vec::new(),
    }
}
