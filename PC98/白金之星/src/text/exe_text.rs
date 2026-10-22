use super::controls::{extract_records, render_edited_record};
use super::encoding::encode_game_text;
use super::model::{TextDocument, TextEntry};
use super::pklite::unpack_mz_file;
use encoding_rs::SHIFT_JIS;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use vn_font::font_98::EncodingPlan;

const SPD_DECODED_SHA256: &str = "C52AE8DEB0F25DB4572A36BE699B0A27CEBB224CE7101913C88891D75BA6BF12";
const SPG_DECODED_SHA256: &str = "44BF279B39083F7B0E2E01504D07B5B46B4633972C26EFD8C6F99E7D4767CEDD";

const SPD_SLOTS: [usize; 15] = [
    0x1787E, 0x178C4, 0x178F9, 0x17953, 0x17991, 0x179D9, 0x17A67, 0x17AF8, 0x17B3A, 0x17BBF,
    0x17BE6, 0x17C84, 0x17CE9, 0x17D56, 0x17DC7,
];

const MAIN_MENU: [&str; 7] = [
    "ゲームに戻る",
    "ゲーム終了",
    "カード移動",
    "音楽",
    "効果音",
    "Ｖｅｒｂｏｓｅ",
    "ノートモード",
];

const EXIT_MENU: [&str; 8] = [
    "ゲームに戻る",
    "セーブして終了",
    "セーブしないで終了",
    "カード移動",
    "音楽",
    "効果音",
    "Ｖｅｒｂｏｓｅ",
    "ノートモード",
];

pub fn extract_spd(packed: &[u8]) -> Result<TextDocument, String> {
    let decoded = unpack_mz_file(packed).map_err(|error| format!("SPD.BIN: {error}"))?;
    verify_decoded("SPD.BIN", &decoded, SPD_DECODED_SHA256)?;
    let mut entries = Vec::new();
    for (index, &offset) in SPD_SLOTS.iter().enumerate() {
        let raw = nul_slot(&decoded, offset)?;
        let decoded_text = decode_cp932(raw, "SPD.BIN", offset)?;
        let (records, warnings) = extract_records(
            "SPD.BIN",
            0,
            index as u32,
            offset as u64,
            &decoded_text,
            &HashSet::new(),
        );
        if !warnings.is_empty() {
            return Err(format!(
                "SPD.BIN 文本 @0x{offset:X} 含未确认控制符: {}",
                warnings.join("；")
            ));
        }
        if records.len() != 1 {
            return Err(format!(
                "SPD.BIN 文本 @0x{offset:X} 意外拆成 {} 条",
                records.len()
            ));
        }
        let clean = &records[0];
        entries.push(TextEntry {
            _file: "SPD.BIN".into(),
            _index: index as u64,
            _type: "help".into(),
            _block: None,
            _string_index: None,
            _string_offset: None,
            _record_index: None,
            _offset: Some(offset as u64),
            _size: Some(raw.len() as u64),
            _container_offset: None,
            _group: Some("card-rules".into()),
            _item_index: Some(index as u32),
            _page: None,
            _boundary_before: None,
            _boundary_after: None,
            _source_raw: Some(decoded_text),
            _controls: clean._controls.clone(),
            _scr_name: None,
            name: None,
            scr_msg: clean.scr_msg.clone(),
            message: clean.message.clone(),
        });
    }
    Ok(document("SPD.BIN", packed, &decoded, entries))
}

pub fn extract_spg(packed: &[u8]) -> Result<TextDocument, String> {
    let decoded = unpack_mz_file(packed).map_err(|error| format!("SPG.BIN: {error}"))?;
    verify_decoded("SPG.BIN", &decoded, SPG_DECODED_SHA256)?;
    let mut entries = Vec::new();
    append_menu(&decoded, 0x1CA09, "main-menu", &MAIN_MENU, &mut entries)?;
    append_menu(&decoded, 0x1CA5E, "exit-menu", &EXIT_MENU, &mut entries)?;
    for (index, entry) in entries.iter_mut().enumerate() {
        entry._index = index as u64;
    }
    Ok(document("SPG.BIN", packed, &decoded, entries))
}

pub fn inject_spd(
    packed: &[u8],
    baseline: &TextDocument,
    translated: &TextDocument,
    plan: Option<&EncodingPlan>,
) -> Result<(Vec<u8>, usize), String> {
    let changed = baseline
        .entries
        .iter()
        .zip(&translated.entries)
        .filter(|(base, edit)| base.message != edit.message)
        .count();
    if changed == 0 {
        return Ok((packed.to_vec(), 0));
    }
    let mut decoded = unpack_mz_file(packed).map_err(|error| format!("SPD.BIN: {error}"))?;
    verify_decoded("SPD.BIN", &decoded, SPD_DECODED_SHA256)?;
    for (base, edit) in baseline.entries.iter().zip(&translated.entries) {
        if base.message == edit.message {
            continue;
        }
        let offset = usize::try_from(
            base._offset
                .ok_or_else(|| "SPD 记录缺少 _offset".to_string())?,
        )
        .map_err(|_| "SPD 偏移过大".to_string())?;
        let capacity = usize::try_from(base._size.ok_or_else(|| "SPD 记录缺少 _size".to_string())?)
            .map_err(|_| "SPD 槽长度过大".to_string())?;
        let raw = base
            ._source_raw
            .as_deref()
            .ok_or_else(|| format!("SPD 记录 {} 缺少 _source_raw", base._index))?;
        let rendered = render_edited_record(raw, None, None, &base.scr_msg, &edit.message)
            .map_err(|error| format!("SPD 记录 {}: {error}", base._index))?;
        let encoded = encode_game_text(&rendered, plan)?;
        if encoded.len() > capacity {
            return Err(format!(
                "SPD 记录 {} 编码后 {} 字节，固定槽只有 {} 字节",
                base._index,
                encoded.len(),
                capacity
            ));
        }
        let slot = decoded
            .get_mut(offset..offset + capacity)
            .ok_or_else(|| format!("SPD 记录 {} 槽越界", base._index))?;
        slot.fill(0);
        slot[..encoded.len()].copy_from_slice(&encoded);
    }
    Ok((decoded, changed))
}

pub fn inject_spg(
    packed: &[u8],
    baseline: &TextDocument,
    translated: &TextDocument,
    plan: Option<&EncodingPlan>,
) -> Result<(Vec<u8>, usize), String> {
    let changed = baseline
        .entries
        .iter()
        .zip(&translated.entries)
        .filter(|(base, edit)| base.message != edit.message)
        .count();
    if changed == 0 {
        return Ok((packed.to_vec(), 0));
    }
    let mut decoded = unpack_mz_file(packed).map_err(|error| format!("SPG.BIN: {error}"))?;
    verify_decoded("SPG.BIN", &decoded, SPG_DECODED_SHA256)?;
    for (group, container_offset) in [("main-menu", 0x1CA09usize), ("exit-menu", 0x1CA5Eusize)] {
        let base_items = baseline
            .entries
            .iter()
            .filter(|entry| entry._group.as_deref() == Some(group))
            .collect::<Vec<_>>();
        let edit_items = translated
            .entries
            .iter()
            .filter(|entry| entry._group.as_deref() == Some(group))
            .collect::<Vec<_>>();
        if base_items.len() != edit_items.len() {
            return Err(format!("SPG 菜单组 {group} 条目数量不一致"));
        }
        if !base_items
            .iter()
            .zip(&edit_items)
            .any(|(base, edit)| base.message != edit.message)
        {
            continue;
        }
        let original = nul_slot(&decoded, container_offset)?.to_vec();
        let original_text = decode_cp932(&original, "SPG.BIN", container_offset)?;
        let mut rebuilt = String::new();
        for (base, edit) in base_items.into_iter().zip(edit_items) {
            let raw = base
                ._source_raw
                .as_deref()
                .ok_or_else(|| format!("SPG 记录 {} 缺少 _source_raw", base._index))?;
            let prefix = raw.strip_suffix(&base.scr_msg).ok_or_else(|| {
                format!("SPG 记录 {} 的 _source_raw 不以 scr_msg 结尾", base._index)
            })?;
            let validated =
                render_edited_record(&base.scr_msg, None, None, &base.scr_msg, &edit.message)
                    .map_err(|error| format!("SPG 记录 {}: {error}", base._index))?;
            rebuilt.push_str(prefix);
            rebuilt.push_str(&validated);
        }
        let baseline_joined = baseline
            .entries
            .iter()
            .filter(|entry| entry._group.as_deref() == Some(group))
            .map(|entry| entry._source_raw.as_deref().unwrap_or(""))
            .collect::<String>();
        if baseline_joined != original_text {
            return Err(format!("SPG 菜单组 {group} 的物理模板校验失败"));
        }
        let encoded = encode_game_text(&rebuilt, plan)?;
        if encoded.len() > original.len() {
            return Err(format!(
                "SPG 菜单组 {group} 编码后 {} 字节，固定容器只有 {} 字节",
                encoded.len(),
                original.len()
            ));
        }
        let slot = decoded
            .get_mut(container_offset..container_offset + original.len())
            .ok_or_else(|| format!("SPG 菜单组 {group} 容器越界"))?;
        slot.fill(0);
        slot[..encoded.len()].copy_from_slice(&encoded);
    }
    Ok((decoded, changed))
}

fn append_menu(
    decoded: &[u8],
    container_offset: usize,
    group: &str,
    items: &[&str],
    output: &mut Vec<TextEntry>,
) -> Result<(), String> {
    let container = nul_slot(decoded, container_offset)?;
    let _ = decode_cp932(container, "SPG.BIN", container_offset)?;
    let mut cursor = 0usize;
    for (item_index, item) in items.iter().enumerate() {
        let encoded = encode_cp932(item)?;
        let relative = container[cursor..]
            .windows(encoded.len())
            .position(|window| window == encoded.as_slice())
            .map(|value| cursor + value)
            .ok_or_else(|| {
                format!("SPG.BIN 菜单 {group} @0x{container_offset:X} 缺少项目 {item}")
            })?;
        let raw_start = cursor;
        let end = relative + encoded.len();
        let source_raw = decode_cp932(
            &container[raw_start..end],
            "SPG.BIN",
            container_offset + raw_start,
        )?;
        output.push(TextEntry {
            _file: "SPG.BIN".into(),
            _index: 0,
            _type: "menu_item".into(),
            _block: None,
            _string_index: None,
            _string_offset: None,
            _record_index: None,
            _offset: Some((container_offset + relative) as u64),
            _size: Some(encoded.len() as u64),
            _container_offset: Some(container_offset as u64),
            _group: Some(group.into()),
            _item_index: Some(item_index as u32),
            _page: None,
            _boundary_before: None,
            _boundary_after: None,
            _source_raw: Some(source_raw),
            _controls: Vec::new(),
            _scr_name: None,
            name: None,
            scr_msg: (*item).into(),
            message: (*item).into(),
        });
        cursor = end;
    }
    Ok(())
}

fn document(file: &str, packed: &[u8], decoded: &[u8], entries: Vec<TextEntry>) -> TextDocument {
    TextDocument {
        _format: "platinum-star-text-v1".into(),
        _file: file.into(),
        _source_sha256: sha256(packed),
        _source_bytes: packed.len() as u64,
        _decoded_sha256: Some(sha256(decoded)),
        _decoded_bytes: Some(decoded.len() as u64),
        _encoding: "CP932".into(),
        entries,
    }
}

fn verify_decoded(file: &str, decoded: &[u8], expected: &str) -> Result<(), String> {
    let actual = sha256(decoded);
    if actual != expected {
        return Err(format!(
            "{file} 解壳结果不属于已确认版本：SHA-256 {actual}，预期 {expected}"
        ));
    }
    Ok(())
}

fn nul_slot(data: &[u8], offset: usize) -> Result<&[u8], String> {
    let tail = data
        .get(offset..)
        .ok_or_else(|| format!("字符串偏移越界 @0x{offset:X}"))?;
    let length = tail
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| format!("字符串没有 NUL 终止符 @0x{offset:X}"))?;
    Ok(&tail[..length])
}

fn decode_cp932(bytes: &[u8], file: &str, offset: usize) -> Result<String, String> {
    SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(bytes)
        .map(|value| value.into_owned())
        .ok_or_else(|| format!("{file} @0x{offset:X} 不是有效 CP932"))
}

fn encode_cp932(value: &str) -> Result<Vec<u8>, String> {
    let (encoded, _, had_errors) = SHIFT_JIS.encode(value);
    if had_errors {
        return Err(format!("菜单原文无法编码为 CP932: {value}"));
    }
    Ok(encoded.into_owned())
}

fn sha256(data: &[u8]) -> String {
    format!("{:X}", Sha256::digest(data))
}
