use std::collections::HashSet;
use std::path::{Component, Path};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::archive::{
    ArchiveBranch, ArchiveEntry, ENTRY_SIZE, LayoutItem, ParsedArchive, title_key,
};

pub const MANIFEST_VERSION: u32 = 1;
pub const MANIFEST_FORMAT: &str = "acv1-script-dat-unified";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub manifest_version: u32,
    pub format: String,
    pub source_file: String,
    pub branch: ArchiveBranch,
    pub game_title: String,
    pub crc64_ecma: String,
    pub key_low32: String,
    pub entry_count: usize,
    pub index_end: usize,
    pub data_base: usize,
    pub entries: Vec<ManifestEntry>,
    pub layout: Vec<ManifestLayoutItem>,
    pub trailing_data_hex: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ManifestEntry {
    pub index: usize,
    pub key_lo: u32,
    pub key_hi: u32,
    pub flag: u8,
    pub offset: u32,
    pub packed_size: u32,
    pub out_capacity: u32,
    pub unpacked_size: usize,
    pub file: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ManifestLayoutItem {
    pub entry_index: usize,
    pub gap_before_hex: String,
}

impl Manifest {
    pub fn new(
        source_file: String,
        archive: &ParsedArchive,
        game_title: String,
        crc64: u64,
        key_low32: u32,
        files: &[String],
        plain_entries: &[Vec<u8>],
    ) -> Result<Self> {
        if files.len() != archive.entries.len() || plain_entries.len() != archive.entries.len() {
            bail!("生成 manifest 时条目数量不一致");
        }
        let entries = archive
            .entries
            .iter()
            .map(|entry| ManifestEntry {
                index: entry.index,
                key_lo: entry.key_lo,
                key_hi: entry.key_hi,
                flag: entry.flag,
                offset: entry.offset,
                packed_size: entry.packed_size,
                out_capacity: entry.out_capacity,
                unpacked_size: plain_entries[entry.index].len(),
                file: files[entry.index].clone(),
            })
            .collect();
        let layout = archive
            .layout
            .iter()
            .map(|item| ManifestLayoutItem {
                entry_index: item.entry_index,
                gap_before_hex: encode_hex(&item.gap_before),
            })
            .collect();
        let data_base = archive
            .entries
            .iter()
            .map(|entry| entry.offset as usize)
            .min()
            .unwrap_or(archive.index_end);
        Ok(Self {
            manifest_version: MANIFEST_VERSION,
            format: MANIFEST_FORMAT.to_owned(),
            source_file,
            branch: archive.branch,
            game_title,
            crc64_ecma: format!("0x{crc64:016X}"),
            key_low32: format!("0x{key_low32:08X}"),
            entry_count: archive.entries.len(),
            index_end: archive.index_end,
            data_base,
            entries,
            layout,
            trailing_data_hex: encode_hex(&archive.trailing_data),
        })
    }

    pub fn validate_and_template(&self) -> Result<(ParsedArchive, u32)> {
        if self.manifest_version != MANIFEST_VERSION {
            bail!(
                "不支持的 manifest_version: {}，当前仅支持 {}",
                self.manifest_version,
                MANIFEST_VERSION
            );
        }
        if self.format != MANIFEST_FORMAT {
            bail!("manifest format 不匹配: {:?}", self.format);
        }
        if self.entry_count != self.entries.len() {
            bail!(
                "manifest entry_count 不一致: declared={}, actual={}",
                self.entry_count,
                self.entries.len()
            );
        }
        let expected_index_end = self
            .branch
            .header_size()
            .checked_add(self.entry_count * ENTRY_SIZE)
            .context("manifest index_end 整数溢出")?;
        if self.index_end != expected_index_end {
            bail!(
                "manifest index_end 不一致: expected=0x{expected_index_end:X}, actual=0x{:X}",
                self.index_end
            );
        }

        let (derived_crc, derived_key) = title_key(&self.game_title)?;
        let stored_crc = parse_prefixed_hex(&self.crc64_ecma, 64)?;
        let stored_key = parse_prefixed_hex(&self.key_low32, 32)? as u32;
        if stored_crc != derived_crc || stored_key != derived_key {
            bail!(
                "manifest 游戏名与密钥不一致: derived_crc=0x{derived_crc:016X}, stored_crc=0x{stored_crc:016X}, derived_key=0x{derived_key:08X}, stored_key=0x{stored_key:08X}"
            );
        }

        let mut entries = Vec::with_capacity(self.entries.len());
        let mut seen_indices = HashSet::new();
        for entry in &self.entries {
            if entry.index >= self.entry_count || !seen_indices.insert(entry.index) {
                bail!("manifest entry index 非法或重复: {}", entry.index);
            }
            validate_file_name(&entry.file)?;
            entries.push(ArchiveEntry {
                index: entry.index,
                key_lo: entry.key_lo,
                key_hi: entry.key_hi,
                flag: entry.flag,
                offset: entry.offset,
                packed_size: entry.packed_size,
                out_capacity: entry.out_capacity,
            });
        }
        entries.sort_by_key(|entry| entry.index);
        if entries
            .iter()
            .enumerate()
            .any(|(expected, entry)| expected != entry.index)
        {
            bail!("manifest entry index 必须连续覆盖 0..entry_count");
        }

        let mut layout = Vec::with_capacity(self.layout.len());
        let mut seen_layout = HashSet::new();
        for item in &self.layout {
            if item.entry_index >= self.entry_count || !seen_layout.insert(item.entry_index) {
                bail!(
                    "manifest layout entry_index 非法或重复: {}",
                    item.entry_index
                );
            }
            layout.push(LayoutItem {
                entry_index: item.entry_index,
                gap_before: decode_hex(&item.gap_before_hex).with_context(|| {
                    format!("layout entry {} gap_before_hex 非法", item.entry_index)
                })?,
            });
        }
        if layout.len() != self.entry_count {
            bail!(
                "manifest layout 条目数不一致: expected={}, actual={}",
                self.entry_count,
                layout.len()
            );
        }

        Ok((
            ParsedArchive {
                branch: self.branch,
                index_end: self.index_end,
                entries,
                layout,
                trailing_data: decode_hex(&self.trailing_data_hex)
                    .context("trailing_data_hex 非法")?,
            },
            derived_key,
        ))
    }

    pub fn to_pretty_json(&self) -> Result<String> {
        let mut text = serde_json::to_string_pretty(self).context("序列化 manifest.json 失败")?;
        text.push('\n');
        Ok(text)
    }
}

pub fn encode_hex(data: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(data.len() * 2);
    for &byte in data {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0F) as usize] as char);
    }
    output
}

pub fn decode_hex(text: &str) -> Result<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        bail!("hex 字符数必须为偶数");
    }
    let bytes = text.as_bytes();
    let mut output = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let high = hex_nibble(pair[0])?;
        let low = hex_nibble(pair[1])?;
        output.push((high << 4) | low);
    }
    Ok(output)
}

fn parse_prefixed_hex(text: &str, bits: u32) -> Result<u64> {
    let digits = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .unwrap_or(text);
    let value =
        u64::from_str_radix(digits, 16).with_context(|| format!("十六进制数非法: {text:?}"))?;
    if bits < 64 && value >= (1_u64 << bits) {
        bail!("十六进制数超过 {bits} 位: {text:?}");
    }
    Ok(value)
}

fn hex_nibble(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => bail!("非法 hex 字符: {:?}", byte as char),
    }
}

fn validate_file_name(name: &str) -> Result<()> {
    let path = Path::new(name);
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
        || path.is_absolute()
        || path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
    {
        bail!("manifest file 不是安全的单一文件名: {name:?}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        let data = b"\x00\x01\xAB\xFF";
        assert_eq!(decode_hex(&encode_hex(data)).unwrap(), data);
    }

    #[test]
    fn unsafe_file_names_are_rejected() {
        for name in ["", "../x", "a/b", "a\\b", "C:x"] {
            assert!(validate_file_name(name).is_err(), "{name:?}");
        }
        assert!(validate_file_name("0001_scene.txt").is_ok());
    }
}
