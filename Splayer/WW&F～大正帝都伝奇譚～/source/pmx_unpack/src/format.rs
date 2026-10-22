use std::collections::HashSet;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use encoding_rs::SHIFT_JIS;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

use crate::error::{PmxError, Result};

pub const XOR_KEY: u8 = 0x21;
pub(crate) const ARCHIVE_HEADER_SIZE: usize = 4;
pub(crate) const ARCHIVE_RECORD_SIZE: usize = 36;
pub(crate) const ARCHIVE_NAME_SIZE: usize = 32;

#[derive(Debug)]
pub(crate) enum ParsedPayload {
    Archive(Archive),
    Raw,
}

#[derive(Debug)]
pub(crate) struct Archive {
    pub table_size: usize,
    pub entries: Vec<ArchiveEntry>,
}

#[derive(Debug)]
pub(crate) struct ArchiveEntry {
    pub index: usize,
    pub relative_path: PathBuf,
    pub manifest_path: String,
    pub size: usize,
    pub record_offset: usize,
    pub data_offset: usize,
    pub name_slot: [u8; ARCHIVE_NAME_SIZE],
}

pub(crate) fn decode_envelope(encoded: &[u8]) -> Result<Vec<u8>> {
    if encoded.len() < 6 {
        return Err(PmxError::new(format!(
            "PMX 文件过短，至少需要 6 字节，实际为 {} 字节",
            encoded.len()
        )));
    }

    let zlib: Vec<u8> = encoded.iter().map(|byte| byte ^ XOR_KEY).collect();
    let header = u16::from(zlib[0]) << 8 | u16::from(zlib[1]);
    if zlib[0] & 0x0f != 8 || header % 31 != 0 {
        return Err(PmxError::new(format!(
            "XOR 0x21 后不是有效 zlib 数据（头部 {:02X} {:02X}）",
            zlib[0], zlib[1]
        )));
    }

    let mut decoder = ZlibDecoder::new(zlib.as_slice());
    let mut decoded = Vec::new();
    decoder
        .read_to_end(&mut decoded)
        .map_err(|error| PmxError::context("zlib 解压失败", error))?;

    let consumed = usize::try_from(decoder.total_in())
        .map_err(|_| PmxError::new("压缩数据长度超过当前平台可表示范围"))?;
    if consumed != zlib.len() {
        return Err(PmxError::new(format!(
            "zlib 流后存在未消费数据：总长 {}，已消费 {}",
            zlib.len(),
            consumed
        )));
    }

    Ok(decoded)
}

pub(crate) fn encode_envelope(decoded: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder
        .write_all(decoded)
        .map_err(|error| PmxError::context("zlib 压缩失败", error))?;
    let zlib = encoder
        .finish()
        .map_err(|error| PmxError::context("结束 zlib 压缩流失败", error))?;
    Ok(zlib.into_iter().map(|byte| byte ^ XOR_KEY).collect())
}

pub(crate) fn parse_payload(decoded: &[u8]) -> Result<ParsedPayload> {
    if decoded.len() < ARCHIVE_HEADER_SIZE + ARCHIVE_RECORD_SIZE {
        return Ok(ParsedPayload::Raw);
    }

    let first_slot: [u8; ARCHIVE_NAME_SIZE] = decoded
        [ARCHIVE_HEADER_SIZE..ARCHIVE_HEADER_SIZE + ARCHIVE_NAME_SIZE]
        .try_into()
        .expect("fixed archive name slice");
    let first_name = match decode_name_slot(&first_slot) {
        Ok(name) => name,
        Err(_) => return Ok(ParsedPayload::Raw),
    };

    // Global.Pmx and other standalone PMX payloads have their own structures.
    // Data*.Pmx archives are identified by the first fixed-width name slot.
    if !first_name.to_ascii_lowercase().ends_with(".pmx") {
        return Ok(ParsedPayload::Raw);
    }

    let count = read_u32_le(decoded, 0)? as usize;
    if count == 0 {
        return Err(PmxError::new(
            "Data PMX 文件表以 .PMX 名称开头，但文件数量为 0",
        ));
    }

    let table_size = count
        .checked_mul(ARCHIVE_RECORD_SIZE)
        .and_then(|size| size.checked_add(ARCHIVE_HEADER_SIZE))
        .ok_or_else(|| PmxError::new("Data PMX 文件表大小溢出"))?;
    if table_size > decoded.len() {
        return Err(PmxError::new(format!(
            "Data PMX 文件表越界：count={count}，表尾={table_size}，解压大小={}",
            decoded.len()
        )));
    }

    let mut entries = Vec::with_capacity(count);
    let mut seen_paths = HashSet::with_capacity(count);
    let mut cumulative_size = 0usize;

    for index in 0..count {
        let record_offset = ARCHIVE_HEADER_SIZE + index * ARCHIVE_RECORD_SIZE;
        let name_slot: [u8; ARCHIVE_NAME_SIZE] = decoded
            [record_offset..record_offset + ARCHIVE_NAME_SIZE]
            .try_into()
            .expect("validated archive record slice");
        let name = decode_name_slot(&name_slot).map_err(|error| {
            PmxError::new(format!("Data PMX 记录 {index} 的文件名无效：{error}"))
        })?;
        let relative_path = validate_relative_path(&name)
            .map_err(|error| PmxError::new(format!("Data PMX 记录 {index} 的路径无效：{error}")))?;
        let manifest_path = name.replace('\\', "/");
        let collision_key = manifest_path.to_lowercase();
        if !seen_paths.insert(collision_key) {
            return Err(PmxError::new(format!(
                "Data PMX 存在大小写不敏感的重复路径：{manifest_path}"
            )));
        }

        let size = read_u32_le(decoded, record_offset + ARCHIVE_NAME_SIZE)? as usize;
        let data_offset = table_size
            .checked_add(cumulative_size)
            .ok_or_else(|| PmxError::new("Data PMX 载荷偏移溢出"))?;
        cumulative_size = cumulative_size
            .checked_add(size)
            .ok_or_else(|| PmxError::new("Data PMX 记录大小总和溢出"))?;

        entries.push(ArchiveEntry {
            index,
            relative_path,
            manifest_path,
            size,
            record_offset,
            data_offset,
            name_slot,
        });
    }

    let payload_size = decoded.len() - table_size;
    if cumulative_size != payload_size {
        return Err(PmxError::new(format!(
            "Data PMX 记录大小总和不匹配：记录总和={cumulative_size}，实际载荷={payload_size}"
        )));
    }

    Ok(ParsedPayload::Archive(Archive {
        table_size,
        entries,
    }))
}

pub(crate) fn decode_name_slot(slot: &[u8; ARCHIVE_NAME_SIZE]) -> Result<String> {
    let end = slot
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| PmxError::new("32 字节名称槽缺少 NUL 终止符"))?;
    if end == 0 {
        return Err(PmxError::new("文件名为空"));
    }

    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(&slot[..end]);
    if had_errors {
        return Err(PmxError::new("文件名不能按 CP932/Shift-JIS 解码"));
    }
    Ok(decoded.into_owned())
}

pub(crate) fn validate_relative_path(name: &str) -> Result<PathBuf> {
    let path = Path::new(name);
    if path.is_absolute() {
        return Err(PmxError::new(format!("不允许绝对路径：{name}")));
    }

    let mut has_normal_component = false;
    for component in path.components() {
        match component {
            Component::Normal(_) => has_normal_component = true,
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                return Err(PmxError::new(format!("不允许越界路径：{name}")));
            }
            Component::CurDir => {
                return Err(PmxError::new(format!("不允许当前目录路径段：{name}")));
            }
        }
    }

    if !has_normal_component {
        return Err(PmxError::new("文件名没有有效路径段"));
    }
    Ok(path.to_path_buf())
}

fn read_u32_le(data: &[u8], offset: usize) -> Result<u32> {
    let bytes: [u8; 4] = data
        .get(offset..offset + 4)
        .ok_or_else(|| PmxError::new(format!("读取 u32 越界：offset={offset}")))?
        .try_into()
        .expect("four-byte slice");
    Ok(u32::from_le_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_xor_zlib_envelope() {
        let expected = b"Splayer PMX payload";
        let encoded = encode_envelope(expected).unwrap();

        assert_eq!(&encoded[..2], &[0x59, 0x20]);
        assert_eq!(decode_envelope(&encoded).unwrap(), expected);
    }

    #[test]
    fn parses_data_archive_table() {
        let decoded = make_archive(&[("4.PMX", b"abc"), ("6.PMX", b"defgh")]);
        let ParsedPayload::Archive(archive) = parse_payload(&decoded).unwrap() else {
            panic!("expected archive");
        };

        assert_eq!(archive.table_size, 76);
        assert_eq!(archive.entries.len(), 2);
        assert_eq!(archive.entries[0].manifest_path, "4.PMX");
        assert_eq!(archive.entries[0].data_offset, 76);
        assert_eq!(archive.entries[1].data_offset, 79);
        assert_eq!(archive.entries[1].size, 5);
    }

    #[test]
    fn treats_global_style_payload_as_raw() {
        let mut decoded = vec![0u8; 80];
        decoded[0..4].copy_from_slice(&4u32.to_le_bytes());
        decoded[4..8].copy_from_slice(b"Ww&f");

        assert!(matches!(
            parse_payload(&decoded).unwrap(),
            ParsedPayload::Raw
        ));
    }

    #[test]
    fn rejects_archive_size_mismatch() {
        let mut decoded = make_archive(&[("4.PMX", b"abc")]);
        decoded.pop();
        let error = parse_payload(&decoded).unwrap_err().to_string();
        assert!(error.contains("大小总和不匹配"));
    }

    #[test]
    fn rejects_archive_path_traversal() {
        let decoded = make_archive(&[("..\\evil.PMX", b"abc")]);
        let error = parse_payload(&decoded).unwrap_err().to_string();
        assert!(error.contains("路径无效"));
    }

    fn make_archive(files: &[(&str, &[u8])]) -> Vec<u8> {
        let table_size = ARCHIVE_HEADER_SIZE + ARCHIVE_RECORD_SIZE * files.len();
        let payload_size: usize = files.iter().map(|(_, data)| data.len()).sum();
        let mut decoded = vec![0u8; table_size + payload_size];
        decoded[0..4].copy_from_slice(&(files.len() as u32).to_le_bytes());

        let mut data_offset = table_size;
        for (index, (name, data)) in files.iter().enumerate() {
            let record_offset = ARCHIVE_HEADER_SIZE + index * ARCHIVE_RECORD_SIZE;
            decoded[record_offset..record_offset + name.len()].copy_from_slice(name.as_bytes());
            decoded[record_offset + ARCHIVE_NAME_SIZE..record_offset + ARCHIVE_RECORD_SIZE]
                .copy_from_slice(&(data.len() as u32).to_le_bytes());
            decoded[data_offset..data_offset + data.len()].copy_from_slice(data);
            data_offset += data.len();
        }
        decoded
    }
}
