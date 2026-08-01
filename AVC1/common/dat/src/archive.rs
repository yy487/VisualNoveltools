use std::fmt;
use std::io::{Read, Write};

use anyhow::{Context, Result, bail};
use encoding_rs::SHIFT_JIS;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use serde::{Deserialize, Serialize};

pub const MAGIC_ACV1: [u8; 4] = *b"ACV1";
pub const COUNT_XOR_ACV1: u32 = 0x8B6A_4E5F;
pub const COUNT_XOR_LEGACY: u32 = 0x26AC_A46E;
pub const ENTRY_SIZE: usize = 21;
pub const CRC64_ECMA_POLY: u64 = 0x42F0_E1EB_A9EA_3693;
pub const MAX_ENTRIES: usize = 100_000;
pub const DEFAULT_ZLIB_LEVEL: u32 = 9;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveBranch {
    Acv1,
    Legacy,
}

impl ArchiveBranch {
    pub fn header_size(self) -> usize {
        match self {
            Self::Acv1 => 8,
            Self::Legacy => 4,
        }
    }

    fn count_xor(self) -> u32 {
        match self {
            Self::Acv1 => COUNT_XOR_ACV1,
            Self::Legacy => COUNT_XOR_LEGACY,
        }
    }

    fn offset_extra_xor(self) -> u32 {
        match self {
            Self::Acv1 => COUNT_XOR_ACV1,
            Self::Legacy => 0,
        }
    }
}

impl fmt::Display for ArchiveBranch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Acv1 => f.write_str("ACV1"),
            Self::Legacy => f.write_str("legacy"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveEntry {
    pub index: usize,
    pub key_lo: u32,
    pub key_hi: u32,
    pub flag: u8,
    pub offset: u32,
    pub packed_size: u32,
    pub out_capacity: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutItem {
    pub entry_index: usize,
    pub gap_before: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedArchive {
    pub branch: ArchiveBranch,
    pub index_end: usize,
    pub entries: Vec<ArchiveEntry>,
    pub layout: Vec<LayoutItem>,
    pub trailing_data: Vec<u8>,
}

pub fn crc64_ecma_msb(data: &[u8]) -> u64 {
    let mut crc = u64::MAX;
    for &byte in data {
        let index = ((crc >> 56) as u8) ^ byte;
        let mut current = u64::from(index) << 56;
        for _ in 0..8 {
            current = if current & 0x8000_0000_0000_0000 != 0 {
                (current << 1) ^ CRC64_ECMA_POLY
            } else {
                current << 1
            };
        }
        crc = (crc << 8) ^ current;
    }
    !crc
}

pub fn title_key(game_title: &str) -> Result<(u64, u32)> {
    let (encoded, _, had_errors) = SHIFT_JIS.encode(game_title);
    if had_errors {
        bail!("游戏名包含 CP932 无法编码的字符: {game_title:?}");
    }
    let crc = crc64_ecma_msb(&encoded);
    Ok((crc, crc as u32))
}

pub fn parse_archive(data: &[u8]) -> Result<ParsedArchive> {
    if data.len() < 4 {
        bail!("文件过小: 至少需要 4 字节，实际 {} 字节", data.len());
    }

    let branch = if data.starts_with(&MAGIC_ACV1) {
        if data.len() < 8 {
            bail!("ACV1 文件头被截断: 实际 {} 字节", data.len());
        }
        ArchiveBranch::Acv1
    } else {
        ArchiveBranch::Legacy
    };

    let encoded_count = match branch {
        ArchiveBranch::Acv1 => u32_at(data, 4)?,
        ArchiveBranch::Legacy => u32_at(data, 0)?,
    };
    let count = (encoded_count ^ branch.count_xor()) as usize;
    if count > MAX_ENTRIES {
        bail!("条目数异常: {count}，上限为 {MAX_ENTRIES}");
    }

    let table_bytes = count.checked_mul(ENTRY_SIZE).context("索引长度整数溢出")?;
    let index_end = branch
        .header_size()
        .checked_add(table_bytes)
        .context("索引末尾整数溢出")?;
    if index_end > data.len() {
        bail!(
            "索引被截断: 需要 0x{index_end:X} 字节，文件只有 0x{:X} 字节",
            data.len()
        );
    }

    let mut entries = Vec::with_capacity(count);
    let mut pos = branch.header_size();
    for index in 0..count {
        let key_lo = u32_at(data, pos)?;
        let key_hi = u32_at(data, pos + 4)?;
        let flag = data[pos + 8] ^ (key_lo as u8);
        let offset = u32_at(data, pos + 9)? ^ key_lo ^ branch.offset_extra_xor();
        let packed_size = u32_at(data, pos + 13)? ^ key_lo;
        let out_capacity = u32_at(data, pos + 17)? ^ key_lo;
        pos += ENTRY_SIZE;

        let start = offset as usize;
        let end = start
            .checked_add(packed_size as usize)
            .with_context(|| format!("entry {index} payload 末尾整数溢出"))?;
        if start < index_end {
            bail!(
                "entry {index} payload 与索引重叠: offset=0x{start:X}, index_end=0x{index_end:X}"
            );
        }
        if end > data.len() {
            bail!(
                "entry {index} payload 越界: offset=0x{start:X}, size=0x{packed_size:X}, file=0x{:X}",
                data.len()
            );
        }

        entries.push(ArchiveEntry {
            index,
            key_lo,
            key_hi,
            flag,
            offset,
            packed_size,
            out_capacity,
        });
    }

    let mut physical: Vec<&ArchiveEntry> = entries.iter().collect();
    physical.sort_by_key(|entry| (entry.offset, entry.packed_size, entry.index));

    let mut layout = Vec::with_capacity(count);
    let mut cursor = index_end;
    for entry in physical {
        let start = entry.offset as usize;
        if start < cursor {
            bail!(
                "entry {} payload 与前一 payload 重叠: offset=0x{start:X}, previous_end=0x{cursor:X}",
                entry.index
            );
        }
        layout.push(LayoutItem {
            entry_index: entry.index,
            gap_before: data[cursor..start].to_vec(),
        });
        cursor = start + entry.packed_size as usize;
    }

    Ok(ParsedArchive {
        branch,
        index_end,
        entries,
        layout,
        trailing_data: data[cursor..].to_vec(),
    })
}

pub fn decode_all(
    data: &[u8],
    archive: &ParsedArchive,
    title_key_low32: u32,
) -> Result<Vec<Vec<u8>>> {
    archive
        .entries
        .iter()
        .map(|entry| decode_entry(data, entry, title_key_low32))
        .collect()
}

pub fn decode_entry(data: &[u8], entry: &ArchiveEntry, title_key_low32: u32) -> Result<Vec<u8>> {
    let start = entry.offset as usize;
    let end = start
        .checked_add(entry.packed_size as usize)
        .with_context(|| format!("entry {} payload 末尾整数溢出", entry.index))?;
    let encrypted = data
        .get(start..end)
        .with_context(|| format!("entry {} payload 越界", entry.index))?;
    let compressed = xor_payload_dwords(encrypted, title_key_low32 ^ entry.key_lo);

    let mut decoder = ZlibDecoder::new(compressed.as_slice());
    let mut plain = Vec::with_capacity((entry.out_capacity as usize).min(16 * 1024 * 1024));
    {
        let mut limited = decoder.by_ref().take(u64::from(entry.out_capacity) + 1);
        limited.read_to_end(&mut plain).with_context(|| {
            format!(
                "entry {} zlib 解压失败；游戏名可能不正确 (xor=0x{:08X})",
                entry.index,
                title_key_low32 ^ entry.key_lo
            )
        })?;
    }
    if plain.len() > entry.out_capacity as usize {
        bail!(
            "entry {} 解压结果超过 out_capacity: result={}, capacity={}",
            entry.index,
            plain.len(),
            entry.out_capacity
        );
    }
    if decoder.total_in() as usize != compressed.len() {
        bail!(
            "entry {} zlib 流未消费完整 payload: consumed={}, packed={}",
            entry.index,
            decoder.total_in(),
            compressed.len()
        );
    }
    Ok(plain)
}

pub fn xor_payload_dwords(payload: &[u8], key: u32) -> Vec<u8> {
    let mut output = payload.to_vec();
    let full_len = output.len() / 4 * 4;
    for chunk in output[..full_len].chunks_exact_mut(4) {
        let value = u32::from_le_bytes(chunk.try_into().expect("DWORD chunk")) ^ key;
        chunk.copy_from_slice(&value.to_le_bytes());
    }
    output
}

pub fn rebuild_archive(
    template: &ParsedArchive,
    plain_entries: &[Vec<u8>],
    title_key_low32: u32,
    preserve_capacity: bool,
) -> Result<(Vec<u8>, Vec<ArchiveEntry>)> {
    if plain_entries.len() != template.entries.len() {
        bail!(
            "明文条目数不一致: expected={}, actual={}",
            template.entries.len(),
            plain_entries.len()
        );
    }
    validate_layout(template)?;

    let mut payloads = Vec::with_capacity(template.entries.len());
    for entry in &template.entries {
        let plain = &plain_entries[entry.index];
        // flag 的运行时语义尚未确认；真实样本 flag=2，但原包压缩比对应 zlib 9。
        // 回封保留 flag 字节，压缩级别使用统一的可复现默认值。
        payloads.push(encode_entry(
            plain,
            entry,
            title_key_low32,
            DEFAULT_ZLIB_LEVEL,
        )?);
    }

    let expected_index_end = template
        .branch
        .header_size()
        .checked_add(template.entries.len() * ENTRY_SIZE)
        .context("索引末尾整数溢出")?;
    if template.index_end != expected_index_end {
        bail!(
            "template index_end 不一致: expected=0x{expected_index_end:X}, actual=0x{:X}",
            template.index_end
        );
    }

    let mut rebuilt_entries = template.entries.clone();
    let mut body = Vec::new();
    for item in &template.layout {
        body.extend_from_slice(&item.gap_before);
        let entry = &mut rebuilt_entries[item.entry_index];
        let payload = &payloads[item.entry_index];
        let absolute_offset = expected_index_end
            .checked_add(body.len())
            .context("payload offset 整数溢出")?;
        entry.offset = u32::try_from(absolute_offset).context("payload offset 超过 u32")?;
        entry.packed_size = u32::try_from(payload.len()).context("packed_size 超过 u32")?;
        let plain_len = u32::try_from(plain_entries[item.entry_index].len())
            .context("unpacked_size 超过 u32")?;
        entry.out_capacity = if preserve_capacity {
            entry.out_capacity.max(plain_len)
        } else {
            plain_len
        };
        body.extend_from_slice(payload);
    }
    body.extend_from_slice(&template.trailing_data);

    let mut output = build_index(template.branch, &rebuilt_entries)?;
    output.extend_from_slice(&body);
    Ok((output, rebuilt_entries))
}

fn encode_entry(
    plain: &[u8],
    entry: &ArchiveEntry,
    title_key_low32: u32,
    level: u32,
) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(level));
    encoder
        .write_all(plain)
        .with_context(|| format!("entry {} zlib 压缩失败", entry.index))?;
    let compressed = encoder
        .finish()
        .with_context(|| format!("entry {} zlib 压缩收尾失败", entry.index))?;
    Ok(xor_payload_dwords(
        &compressed,
        title_key_low32 ^ entry.key_lo,
    ))
}

fn build_index(branch: ArchiveBranch, entries: &[ArchiveEntry]) -> Result<Vec<u8>> {
    let count = u32::try_from(entries.len()).context("条目数超过 u32")?;
    let mut output = Vec::with_capacity(branch.header_size() + entries.len() * ENTRY_SIZE);
    match branch {
        ArchiveBranch::Acv1 => {
            output.extend_from_slice(&MAGIC_ACV1);
            output.extend_from_slice(&(count ^ COUNT_XOR_ACV1).to_le_bytes());
        }
        ArchiveBranch::Legacy => {
            output.extend_from_slice(&(count ^ COUNT_XOR_LEGACY).to_le_bytes());
        }
    }
    for entry in entries {
        output.extend_from_slice(&entry.key_lo.to_le_bytes());
        output.extend_from_slice(&entry.key_hi.to_le_bytes());
        output.push(entry.flag ^ (entry.key_lo as u8));
        output.extend_from_slice(
            &(entry.offset ^ entry.key_lo ^ branch.offset_extra_xor()).to_le_bytes(),
        );
        output.extend_from_slice(&(entry.packed_size ^ entry.key_lo).to_le_bytes());
        output.extend_from_slice(&(entry.out_capacity ^ entry.key_lo).to_le_bytes());
    }
    Ok(output)
}

fn validate_layout(archive: &ParsedArchive) -> Result<()> {
    if archive.layout.len() != archive.entries.len() {
        bail!(
            "layout 条目数不一致: expected={}, actual={}",
            archive.entries.len(),
            archive.layout.len()
        );
    }
    let mut seen = vec![false; archive.entries.len()];
    for item in &archive.layout {
        let slot = seen
            .get_mut(item.entry_index)
            .with_context(|| format!("layout entry_index 越界: {}", item.entry_index))?;
        if *slot {
            bail!("layout entry_index 重复: {}", item.entry_index);
        }
        *slot = true;
    }
    Ok(())
}

fn u32_at(data: &[u8], offset: usize) -> Result<u32> {
    let bytes = data
        .get(offset..offset + 4)
        .with_context(|| format!("读取 u32 越界: offset=0x{offset:X}"))?;
    Ok(u32::from_le_bytes(bytes.try_into().expect("four bytes")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_template(branch: ArchiveBranch) -> ParsedArchive {
        let index_end = branch.header_size() + 2 * ENTRY_SIZE;
        ParsedArchive {
            branch,
            index_end,
            entries: vec![
                ArchiveEntry {
                    index: 0,
                    key_lo: 0x1122_3344,
                    key_hi: 0x5566_7788,
                    flag: 2,
                    offset: index_end as u32,
                    packed_size: 0,
                    out_capacity: 128,
                },
                ArchiveEntry {
                    index: 1,
                    key_lo: 0x89AB_CDEF,
                    key_hi: 0x1020_3040,
                    flag: 9,
                    offset: index_end as u32,
                    packed_size: 0,
                    out_capacity: 128,
                },
            ],
            layout: vec![
                LayoutItem {
                    entry_index: 1,
                    gap_before: b"HEADER-GAP".to_vec(),
                },
                LayoutItem {
                    entry_index: 0,
                    gap_before: b"MID".to_vec(),
                },
            ],
            trailing_data: b"TAIL".to_vec(),
        }
    }

    #[test]
    fn known_title_crc_matches_real_sample() {
        let (crc, low) = title_key("姉小路直子と銀色の死神").unwrap();
        assert_eq!(crc, 0xF9EF_88FE_7D5D_C8F6);
        assert_eq!(low, 0x7D5D_C8F6);
    }

    #[test]
    fn title_must_be_cp932() {
        assert!(title_key("title-😀").is_err());
    }

    #[test]
    fn dword_xor_preserves_tail_and_is_symmetric() {
        let source = b"1234567890";
        let encrypted = xor_payload_dwords(source, 0xDEAD_BEEF);
        assert_eq!(&encrypted[8..], &source[8..]);
        assert_eq!(xor_payload_dwords(&encrypted, 0xDEAD_BEEF), source);
    }

    #[test]
    fn both_branches_roundtrip_and_preserve_opaque_bytes() {
        let (_, key) = title_key("姉小路直子と銀色の死神").unwrap();
        let plains = vec![b"*first\r\ntext".to_vec(), b"*second\r\nmore text".to_vec()];
        for branch in [ArchiveBranch::Acv1, ArchiveBranch::Legacy] {
            let template = synthetic_template(branch);
            let (archive_bytes, _) = rebuild_archive(&template, &plains, key, true).unwrap();
            let parsed = parse_archive(&archive_bytes).unwrap();
            assert_eq!(parsed.branch, branch);
            assert_eq!(parsed.layout[0].gap_before, b"HEADER-GAP");
            assert_eq!(parsed.layout[1].gap_before, b"MID");
            assert_eq!(parsed.trailing_data, b"TAIL");
            assert_eq!(decode_all(&archive_bytes, &parsed, key).unwrap(), plains);
        }
    }

    #[test]
    fn truncated_index_is_rejected() {
        let mut data = Vec::new();
        data.extend_from_slice(&MAGIC_ACV1);
        data.extend_from_slice(&(1_u32 ^ COUNT_XOR_ACV1).to_le_bytes());
        assert!(parse_archive(&data).is_err());
    }

    #[test]
    fn absurd_legacy_count_is_rejected() {
        let data = ((MAX_ENTRIES as u32 + 1) ^ COUNT_XOR_LEGACY).to_le_bytes();
        assert!(parse_archive(&data).is_err());
    }
}
