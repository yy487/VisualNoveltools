//! Verified text tables stored directly in the game executables.
use crate::Result;
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SPEAKER_COUNT: usize = 24;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeakerTable {
    pub pointer_offset: usize,
    pub strings_offset: usize,
    /// Four CP932 character cells, excluding the renderer's `「` suffix.
    pub labels: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuString {
    pub id: String,
    pub offset: usize,
    /// Maximum source slot size, including the trailing `$` terminator.
    pub capacity: usize,
    pub message: String,
}

const SPEAKER_ZERO: [u8; 11] = [
    0x81, 0x40, 0x81, 0x40, 0x81, 0x40, 0x81, 0x40, 0x81, 0x75, 0x24,
];

pub fn find_speaker_table(bytes: &[u8]) -> Result<SpeakerTable> {
    let mut matches = Vec::new();
    for pool in 0..=bytes.len().saturating_sub(SPEAKER_COUNT * 11) {
        if bytes.get(pool..pool + SPEAKER_ZERO.len()) != Some(SPEAKER_ZERO.as_slice()) {
            continue;
        }
        let Some(pointer_offset) = pool.checked_sub(SPEAKER_COUNT * 2) else {
            continue;
        };
        let Some(expected_base) = pool.checked_add(0x100) else {
            continue;
        };
        let mut labels = Vec::with_capacity(SPEAKER_COUNT);
        let mut valid = true;
        for index in 0..SPEAKER_COUNT {
            let pointer_at = pointer_offset + index * 2;
            let Some(pointer_bytes) = bytes.get(pointer_at..pointer_at + 2) else {
                valid = false;
                break;
            };
            let pointer = usize::from(u16::from_le_bytes([pointer_bytes[0], pointer_bytes[1]]));
            let expected = expected_base + index * 11;
            let start = pool + index * 11;
            if pointer != expected || bytes.get(start + 10) != Some(&0x24) {
                valid = false;
                break;
            }
            let row = bytes.get(start..start + 10).ok_or("说话人字符串表被截断")?;
            let quote = SHIFT_JIS
                .decode_without_bom_handling_and_without_replacement(&row[8..10])
                .ok_or("说话人表中的开引号不是有效 CP932")?;
            if quote != "「" {
                valid = false;
                break;
            }
            let label = SHIFT_JIS
                .decode_without_bom_handling_and_without_replacement(&row[..8])
                .ok_or("说话人姓名不是有效 CP932")?
                .into_owned();
            if label.chars().count() != 4 {
                valid = false;
                break;
            }
            labels.push(label);
        }
        if valid {
            matches.push(SpeakerTable {
                pointer_offset,
                strings_offset: pool,
                labels,
            });
        }
    }
    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => Err("可执行文件中找不到经过校验的 24 项说话人表".into()),
        count => Err(format!("可执行文件中发现 {count} 个候选说话人表")),
    }
}

/// Return the index and character-cell names used by the runtime `#<index>` token.
pub fn speaker_messages(table: &SpeakerTable) -> Vec<String> {
    table
        .labels
        .iter()
        .map(|label| label.trim_end_matches('　').to_owned())
        .collect()
}

pub fn patch_speaker_table<F>(
    bytes: &[u8],
    table: &SpeakerTable,
    labels: &[String],
    encode: F,
) -> Result<Vec<u8>>
where
    F: Fn(&str) -> Result<Vec<u8>>,
{
    if labels.len() != SPEAKER_COUNT {
        return Err(format!(
            "说话人表必须有 {SPEAKER_COUNT} 项，收到 {} 项",
            labels.len()
        ));
    }
    let mut out = bytes.to_vec();
    for (index, label) in labels.iter().enumerate() {
        validate_cell_text(label, 4, "说话人姓名")?;
        let mut cells = label.clone();
        let count = cells.chars().count();
        cells.extend(std::iter::repeat_n('　', 4 - count));
        cells.push('「');
        let encoded = encode(&cells)?;
        if encoded.len() != 10 {
            return Err(format!("说话人 #{} 编码后不是 5 个双字节字形", index));
        }
        let start = table.strings_offset + index * 11;
        let mut replacement = encoded;
        replacement.push(0x24);
        let target = out
            .get_mut(start..start + 11)
            .ok_or("说话人字符串写入超出可执行文件")?;
        target.copy_from_slice(&replacement);
    }
    Ok(out)
}

/// Locate the visible strings in JESUS2.COM by their verified CP932 content.
pub fn find_boot_menu(bytes: &[u8]) -> Result<Vec<MenuString>> {
    const HEAD: [(&str, &str); 4] = [
        ("select", "選択して下さい。"),
        ("new_game", "最初から始める"),
        ("load_game", "以前のデータを読み込む"),
        ("load_slot_prompt", "何番のデータを読み込みますか？"),
    ];
    let mut fields = Vec::new();
    for (id, text) in HEAD {
        let encoded = encode_source_cp932(text)?;
        let start = find_unique(bytes, &encoded, text)?;
        let end = bytes[start..]
            .iter()
            .position(|byte| *byte == 0x24)
            .map(|offset| start + offset + 1)
            .ok_or_else(|| format!("JESUS2.COM 中的菜单字符串没有 `$` 终止符: {text}"))?;
        fields.push((id.to_owned(), start, end, text.to_owned()));
    }
    fields.sort_by_key(|field| field.1);
    if fields.windows(2).any(|window| window[0].2 > window[1].1) {
        return Err("JESUS2.COM 菜单字符串区域重叠".into());
    }

    let prompt_index = fields
        .iter()
        .position(|field| field.0 == "load_slot_prompt")
        .ok_or("缺少读档提示")?;
    let mut start = fields[prompt_index].2;
    for (id, expected) in [
        ("load_slot_1", "１"),
        ("load_slot_2", "２"),
        ("load_slot_3", "３"),
        ("start_first_half", "前半から始める"),
        ("start_second_half", "後半から始める"),
    ] {
        let end = bytes[start..]
            .iter()
            .position(|byte| *byte == 0x24)
            .map(|offset| start + offset + 1)
            .ok_or_else(|| format!("读档/分段菜单中缺少 `$` 终止符: {expected}"))?;
        let decoded = decode_cp932(&bytes[start..end - 1])?;
        if decoded != expected {
            return Err(format!(
                "JESUS2.COM 菜单顺序不匹配: 期望 {expected:?}，实际 {decoded:?}"
            ));
        }
        fields.push((id.to_owned(), start, end, expected.to_owned()));
        start = end;
    }
    fields.sort_by_key(|field| field.1);
    Ok(fields
        .iter()
        .enumerate()
        .map(|(index, (id, offset, end, text))| MenuString {
            id: id.clone(),
            offset: *offset,
            capacity: fields
                .get(index + 1)
                .map(|next| next.1 - offset)
                .unwrap_or(end - offset),
            message: text.clone(),
        })
        .collect())
}

pub fn patch_boot_menu<F>(
    bytes: &[u8],
    fields: &[MenuString],
    replacements: &BTreeMap<String, String>,
    encode: F,
) -> Result<Vec<u8>>
where
    F: Fn(&str) -> Result<Vec<u8>>,
{
    let mut out = bytes.to_vec();
    for field in fields {
        let Some(message) = replacements.get(&field.id) else {
            continue;
        };
        if message.is_empty() || message.contains('$') || message.contains('\n') {
            return Err(format!(
                "菜单 {} 是固定 CP932 字符串，不能含空文本、`$` 或换行",
                field.id
            ));
        }
        let encoded = encode(message)?;
        let total = encoded
            .len()
            .checked_add(1)
            .ok_or_else(|| "菜单字符串长度溢出".to_string())?;
        if total > field.capacity {
            return Err(format!(
                "菜单 {} 需要 {total} 字节，固定槽仅有 {} 字节",
                field.id, field.capacity
            ));
        }
        let target = out
            .get_mut(field.offset..field.offset + field.capacity)
            .ok_or_else(|| format!("菜单 {} 超出 JESUS2.COM", field.id))?;
        target.fill(0);
        target[..encoded.len()].copy_from_slice(&encoded);
        target[encoded.len()] = 0x24;
    }
    Ok(out)
}

pub fn encode_source_cp932(text: &str) -> Result<Vec<u8>> {
    let (bytes, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        return Err(format!("源菜单不是可往返的 CP932: {text:?}"));
    }
    Ok(bytes.into_owned())
}

fn decode_cp932(bytes: &[u8]) -> Result<String> {
    SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(bytes)
        .map(|text| text.into_owned())
        .ok_or_else(|| format!("菜单字节不是有效 CP932: {}", crate::hex(bytes)))
}

fn find_unique(bytes: &[u8], needle: &[u8], label: &str) -> Result<usize> {
    let matches = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(offset, candidate)| (candidate == needle).then_some(offset))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [offset] => Ok(*offset),
        [] => Err(format!("JESUS2.COM 缺少菜单字符串: {label}")),
        _ => Err(format!("JESUS2.COM 菜单字符串重复: {label}")),
    }
}

fn validate_cell_text(text: &str, max_cells: usize, label: &str) -> Result<()> {
    if text.chars().count() > max_cells
        || text.chars().any(|character| {
            character.is_control() || matches!(character, '$' | '#' | '@' | '*' | '^')
        })
    {
        return Err(format!("{label} 超出固定字槽或包含控制字符"));
    }
    Ok(())
}
