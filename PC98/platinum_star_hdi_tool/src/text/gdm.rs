use super::controls::{collect_explicit_speakers, extract_records, render_edited_record};
use super::encoding::encode_game_text;
use super::model::{TextDocument, TextEntry};
use encoding_rs::SHIFT_JIS;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use vn_font::font_98::EncodingPlan;

const INSTRUCTION_SIZES: [u8; 76] = [
    1, 1, 2, 3, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 3, 3, 3, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 0xFF, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 1, 1, 3, 3, 3, 1, 2, 2, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3,
];

pub struct ScenarioResult {
    pub document: TextDocument,
    pub blocks: usize,
    pub warnings: Vec<String>,
}

#[derive(Clone)]
struct DecodedBlock {
    number: usize,
    data: Vec<u8>,
}

struct GdmArchive {
    index: Vec<u8>,
    blocks: Vec<Vec<u8>>,
    raw_blocks: Vec<Vec<u8>>,
    tail: Vec<u8>,
}

pub fn extract_scenario(data: &[u8], jobs: usize) -> Result<ScenarioResult, String> {
    let blocks = unpack_gdm(data)?;
    let block_count = blocks.len();
    let speakers = Arc::new(collect_speakers(&blocks)?);
    let blocks = Arc::new(
        blocks
            .into_iter()
            .enumerate()
            .map(|(number, data)| DecodedBlock { number, data })
            .collect::<Vec<_>>(),
    );
    let next = Arc::new(AtomicUsize::new(0));
    let (sender, receiver) = mpsc::channel();
    let workers = jobs.max(1).min(blocks.len().max(1));
    thread::scope(|scope| {
        for _ in 0..workers {
            let blocks = Arc::clone(&blocks);
            let next = Arc::clone(&next);
            let speakers = Arc::clone(&speakers);
            let sender = sender.clone();
            scope.spawn(move || loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                let Some(block) = blocks.get(index) else {
                    break;
                };
                let result = extract_block(block, &speakers);
                if sender.send((block.number, result)).is_err() {
                    break;
                }
            });
        }
    });
    drop(sender);

    let mut results = receiver.into_iter().collect::<Vec<_>>();
    results.sort_by_key(|(number, _)| *number);
    let mut entries = Vec::new();
    let mut warnings = Vec::new();
    for (number, result) in results {
        let (mut block_entries, mut block_warnings) =
            result.map_err(|error| format!("SCENARIO.GDM 块 {number}: {error}"))?;
        entries.append(&mut block_entries);
        warnings.append(&mut block_warnings);
    }
    for (index, entry) in entries.iter_mut().enumerate() {
        entry._index = index as u64;
    }

    Ok(ScenarioResult {
        document: TextDocument {
            _format: "platinum-star-text-v1".into(),
            _file: "SCENARIO.GDM".into(),
            _source_sha256: sha256(data),
            _source_bytes: data.len() as u64,
            _decoded_sha256: None,
            _decoded_bytes: None,
            _encoding: "CP932".into(),
            entries,
        },
        blocks: block_count,
        warnings,
    })
}

pub fn inject_scenario(
    source: &[u8],
    baseline: &TextDocument,
    translated: &TextDocument,
    plan: Option<&EncodingPlan>,
) -> Result<(Vec<u8>, usize), String> {
    let archive = parse_archive(source)?;
    let mut groups: BTreeMap<(u32, u32), Vec<(&TextEntry, &TextEntry)>> = BTreeMap::new();
    let mut changed_entries = 0usize;
    for (base, edit) in baseline.entries.iter().zip(&translated.entries) {
        if base.name != edit.name || base.message != edit.message {
            changed_entries += 1;
            let key = (
                base._block
                    .ok_or_else(|| "场景记录缺少 _block".to_string())?,
                base._string_index
                    .ok_or_else(|| "场景记录缺少 _string_index".to_string())?,
            );
            groups.entry(key).or_default().push((base, edit));
        }
    }
    if changed_entries == 0 {
        return Ok((source.to_vec(), 0));
    }

    let mut rebuilt_blocks = archive.blocks.clone();
    for (block_number, block) in rebuilt_blocks.iter_mut().enumerate() {
        let relevant = groups
            .iter()
            .filter(|((number, _), _)| *number as usize == block_number)
            .map(|((_, string), records)| (*string, records.as_slice()))
            .collect::<BTreeMap<_, _>>();
        if !relevant.is_empty() {
            *block = patch_block(block, &relevant, plan)
                .map_err(|error| format!("SCENARIO.GDM 块 {block_number}: {error}"))?;
        }
    }

    let mut packed_blocks = Vec::with_capacity(rebuilt_blocks.len());
    for (number, block) in rebuilt_blocks.iter().enumerate() {
        if block == &archive.blocks[number] {
            packed_blocks.push(archive.raw_blocks[number].clone());
        } else {
            packed_blocks.push(encode_literal_block(block)?);
        }
    }
    let mut offsets = Vec::with_capacity(packed_blocks.len() + 1);
    let mut relative = 0u32;
    offsets.push(relative);
    for block in &packed_blocks {
        relative = relative
            .checked_add(u32::try_from(block.len()).map_err(|_| "GDM 块过大".to_string())?)
            .ok_or_else(|| "GDM 相对偏移溢出".to_string())?;
        offsets.push(relative);
    }
    if archive.index.len() < offsets.len() * 4 {
        return Err("GDM 索引没有足够空间保存重建偏移".into());
    }
    let mut index = archive.index.clone();
    for (slot, value) in offsets.iter().enumerate() {
        index[slot * 4..slot * 4 + 4].copy_from_slice(&value.to_le_bytes());
    }
    let mut output = encode_literal_block(&index)?;
    for block in packed_blocks {
        output.extend_from_slice(&block);
    }
    output.extend_from_slice(&archive.tail);
    let reparsed = parse_archive(&output)?;
    if reparsed.blocks != rebuilt_blocks {
        return Err("SCENARIO.GDM 重建后解码内容不一致".into());
    }
    Ok((output, changed_entries))
}

fn patch_block(
    block: &[u8],
    groups: &BTreeMap<u32, &[(&TextEntry, &TextEntry)]>,
    plan: Option<&EncodingPlan>,
) -> Result<Vec<u8>, String> {
    let (pool, _) = parse_program(block)?;
    let strings_start = pool + 2;
    if strings_start > block.len() {
        return Err("字符串池运行时字段越界".into());
    }
    let mut output = block[..strings_start].to_vec();
    let mut pos = strings_start;
    let mut string_index = 0u32;
    while pos < block.len() {
        let terminator = block[pos..]
            .iter()
            .position(|byte| *byte == 0)
            .map(|relative| pos + relative);
        let end = terminator.unwrap_or(block.len());
        if let Some(records) = groups.get(&string_index) {
            let decoded = decode_cp932(&block[pos..end])?;
            let patched = patch_string(&decoded, records)?;
            output.extend_from_slice(&encode_game_text(&patched, plan)?);
        } else {
            output.extend_from_slice(&block[pos..end]);
        }
        if terminator.is_some() {
            output.push(0);
            pos = end + 1;
        } else {
            break;
        }
        string_index += 1;
    }
    Ok(output)
}

fn patch_string(source: &str, records: &[(&TextEntry, &TextEntry)]) -> Result<String, String> {
    let mut cursor = 0usize;
    let mut output = String::new();
    for (base, edit) in records {
        let raw = base
            ._source_raw
            .as_deref()
            .ok_or_else(|| format!("记录 {} 缺少 _source_raw", base._index))?;
        let relative = source[cursor..].find(raw).ok_or_else(|| {
            format!(
                "记录 {} 的 _source_raw 无法在源字符串中顺序定位",
                base._index
            )
        })?;
        let start = cursor + relative;
        output.push_str(&source[cursor..start]);
        output.push_str(&render_edited_record(
            raw,
            base._scr_name.as_deref(),
            edit.name.as_deref(),
            &base.scr_msg,
            &edit.message,
        )?);
        cursor = start + raw.len();
    }
    output.push_str(&source[cursor..]);
    Ok(output)
}

fn extract_block(
    block: &DecodedBlock,
    speakers: &HashSet<String>,
) -> Result<(Vec<TextEntry>, Vec<String>), String> {
    let (pool, file_only) = parse_program(&block.data)?;
    if pool + 2 > block.data.len() {
        return Err("字符串池缺少两字节运行时字段".into());
    }
    let mut entries = Vec::new();
    let mut warnings = Vec::new();
    let mut pos = pool + 2;
    let mut string_index = 0usize;
    while pos < block.data.len() {
        let end = block.data[pos..]
            .iter()
            .position(|byte| *byte == 0)
            .map(|relative| pos + relative)
            .unwrap_or(block.data.len());
        let raw = &block.data[pos..end];
        if !raw.is_empty() && !file_only.contains(&string_index) {
            let decoded = decode_cp932(raw).map_err(|error| {
                format!("字符串 {string_index} @0x{pos:X} 不是可往返 CP932: {error}")
            })?;
            let (mut records, mut record_warnings) = extract_records(
                "SCENARIO.GDM",
                block.number as u32,
                string_index as u32,
                pos as u64,
                &decoded,
                speakers,
            );
            entries.append(&mut records);
            warnings.extend(record_warnings.drain(..).map(|warning| {
                format!(
                    "SCENARIO.GDM 块 {} 字符串 {} @0x{:X}: {}",
                    block.number, string_index, pos, warning
                )
            }));
        }
        string_index += 1;
        if end == block.data.len() {
            break;
        }
        pos = end + 1;
    }
    Ok((entries, warnings))
}

fn collect_speakers(blocks: &[Vec<u8>]) -> Result<HashSet<String>, String> {
    let mut speakers = HashSet::new();
    for (block_number, block) in blocks.iter().enumerate() {
        let (pool, file_only) = parse_program(block)
            .map_err(|error| format!("SCENARIO.GDM 块 {block_number}: {error}"))?;
        let mut pos = pool + 2;
        let mut string_index = 0usize;
        while pos < block.len() {
            let end = block[pos..]
                .iter()
                .position(|byte| *byte == 0)
                .map(|relative| pos + relative)
                .unwrap_or(block.len());
            if end > pos && !file_only.contains(&string_index) {
                let decoded = decode_cp932(&block[pos..end]).map_err(|error| {
                    format!(
                        "SCENARIO.GDM 块 {block_number} 字符串 {string_index} @0x{pos:X}: {error}"
                    )
                })?;
                speakers.extend(collect_explicit_speakers(&decoded));
            }
            string_index += 1;
            if end == block.len() {
                break;
            }
            pos = end + 1;
        }
    }
    // These speakers are written directly before an opening quote throughout
    // the confirmed sample and never occur in its tab-separated form.
    speakers.extend(
        [
            "リュシィ",
            "リュシイ",
            "眼鏡っ娘",
            "ジェミー",
            "エミニー",
            "カプリア",
            "サジタリス",
            "ピスケス",
        ]
        .into_iter()
        .map(str::to_string),
    );
    Ok(speakers)
}

fn parse_program(block: &[u8]) -> Result<(usize, HashSet<usize>), String> {
    if block.len() < 2 {
        return Err("场景块短于程序头".into());
    }
    let pool = u16_at(block, 0)? as usize;
    if !(2..=block.len()).contains(&pool) {
        return Err(format!("字符串池偏移越界: 0x{pool:X}"));
    }
    let mut pos = 2usize;
    let mut text_refs = HashSet::new();
    let mut file_refs = HashSet::new();
    while pos < pool {
        let opcode = block[pos] as usize;
        let Some(&fixed) = INSTRUCTION_SIZES.get(opcode) else {
            return Err(format!("未知操作码 0x{opcode:02X} @0x{pos:X}"));
        };
        let size = if opcode == 0x24 {
            if pos + 12 > pool {
                return Err(format!("变长指令被截断 @0x{pos:X}"));
            }
            block[pos + 11] as usize * 6 + 12
        } else {
            fixed as usize
        };
        if size == 0 || pos + size > pool {
            return Err(format!("非法指令长度 {size} @0x{pos:X}"));
        }
        if matches!(opcode, 0x48..=0x4A) && size == 3 {
            let index = u16_at(block, pos + 1)? as usize;
            if opcode == 0x48 {
                text_refs.insert(index);
            } else {
                file_refs.insert(index);
            }
        }
        pos += size;
    }
    if pos != pool {
        return Err("指令流没有精确结束在字符串池边界".into());
    }
    Ok((
        pool,
        file_refs
            .difference(&text_refs)
            .copied()
            .collect::<HashSet<_>>(),
    ))
}

fn unpack_gdm(data: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    Ok(parse_archive(data)?.blocks)
}

fn parse_archive(data: &[u8]) -> Result<GdmArchive, String> {
    let (index, base) = decode_block(data, 0)?;
    if index.len() % 4 != 0 {
        return Err(format!("GDM 索引长度 {} 不是四字节对齐", index.len()));
    }
    let mut offsets = Vec::new();
    for raw in index.chunks_exact(4) {
        let value = u32::from_le_bytes(raw.try_into().expect("four bytes")) as usize;
        if !offsets.is_empty() && value == 0 {
            break;
        }
        offsets.push(value);
    }
    if offsets.first() != Some(&0) || offsets.len() < 2 {
        return Err("GDM 索引没有有效的首尾偏移".into());
    }
    if offsets.windows(2).any(|pair| pair[1] <= pair[0]) {
        return Err("GDM 块偏移不是严格递增".into());
    }
    let mut blocks = Vec::with_capacity(offsets.len() - 1);
    let mut raw_blocks = Vec::with_capacity(offsets.len() - 1);
    for pair in offsets.windows(2) {
        let start = base
            .checked_add(pair[0])
            .ok_or_else(|| "GDM 块偏移溢出".to_string())?;
        let (decoded, end) = decode_block(data, start)?;
        let expected = base + pair[1];
        if end != expected {
            return Err(format!("GDM 块结束于 0x{end:X}，索引要求 0x{expected:X}"));
        }
        blocks.push(decoded);
        raw_blocks.push(data[start..end].to_vec());
    }
    let final_end = base + offsets[offsets.len() - 1];
    Ok(GdmArchive {
        index,
        blocks,
        raw_blocks,
        tail: data[final_end..].to_vec(),
    })
}

fn encode_literal_block(data: &[u8]) -> Result<Vec<u8>, String> {
    let flag_bytes = data.len().div_ceil(8);
    let encoded_size = flag_bytes
        .checked_add(data.len())
        .ok_or_else(|| "GDM 字面量块长度溢出".to_string())?;
    let encoded_size = u16::try_from(encoded_size)
        .map_err(|_| format!("GDM 字面量块超过 65535 字节: {encoded_size}"))?;
    let flag_bytes_u16 = u16::try_from(flag_bytes).map_err(|_| "GDM 标志流过长".to_string())?;
    let mut output = Vec::with_capacity(4 + encoded_size as usize);
    output.extend_from_slice(&encoded_size.to_le_bytes());
    output.extend_from_slice(&flag_bytes_u16.to_le_bytes());
    output.resize(4 + flag_bytes, 0);
    output.extend_from_slice(data);
    Ok(output)
}

fn decode_block(data: &[u8], pos: usize) -> Result<(Vec<u8>, usize), String> {
    let encoded_size = u16_at(data, pos)? as usize;
    let byte_stream_offset = u16_at(data, pos + 2)? as usize;
    let flag_start = pos + 4;
    let mut byte_pos = flag_start
        .checked_add(byte_stream_offset)
        .ok_or_else(|| "GDM 字节流偏移溢出".to_string())?;
    let end = flag_start
        .checked_add(encoded_size)
        .ok_or_else(|| "GDM 块长度溢出".to_string())?;
    if byte_stream_offset > encoded_size || end > data.len() {
        return Err(format!("非法 GDM 压缩块 @0x{pos:X}"));
    }
    let mut output = Vec::new();
    let mut flag_index = 0usize;
    while byte_pos < end {
        let flag_pos = flag_start + flag_index / 8;
        if flag_pos >= flag_start + byte_stream_offset {
            return Err(format!("GDM 块标志流耗尽 @0x{pos:X}"));
        }
        let duplicate = data[flag_pos] & (1 << (flag_index & 7)) != 0;
        flag_index += 1;
        if !duplicate {
            output.push(data[byte_pos]);
            byte_pos += 1;
            continue;
        }
        if byte_pos + 2 > end {
            return Err(format!("GDM 回溯项被截断 @0x{pos:X}"));
        }
        let distance = data[byte_pos] as usize + 1;
        let length = data[byte_pos + 1] as usize + 3;
        byte_pos += 2;
        if distance > output.len() {
            return Err(format!("GDM 回溯距离越过输出开头 @0x{pos:X}"));
        }
        for _ in 0..length {
            let value = output[output.len() - distance];
            output.push(value);
        }
    }
    Ok((output, end))
}

fn decode_cp932(bytes: &[u8]) -> Result<String, String> {
    let Some(decoded) = SHIFT_JIS.decode_without_bom_handling_and_without_replacement(bytes) else {
        return Err("解码失败".into());
    };
    let decoded = decoded.into_owned();
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&decoded);
    if had_errors || encoded.as_ref() != bytes {
        return Err("解码后无法编码回相同字节".into());
    }
    Ok(decoded)
}

fn u16_at(data: &[u8], pos: usize) -> Result<u16, String> {
    let raw = data
        .get(pos..pos + 2)
        .ok_or_else(|| format!("16 位读取越界 @0x{pos:X}"))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn sha256(data: &[u8]) -> String {
    format!("{:X}", Sha256::digest(data))
}

#[cfg(test)]
mod tests {
    use super::decode_cp932;

    #[test]
    fn cp932_roundtrip_is_strict() {
        assert_eq!(decode_cp932(&[0x82, 0xA0]).expect("decode"), "あ");
        assert!(decode_cp932(&[0x81]).is_err());
    }
}
