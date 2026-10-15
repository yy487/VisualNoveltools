use crate::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const BANK: usize = 0x4000;
pub const MARKER: &[u8] = b"SKY256-SCRIPTS-v2";
pub const WINDOWS: [(usize, usize, usize); 6] = [
    (3, 0, 257),
    (2, 0, 257),
    (1, 0, 181),
    (2, 199, 199),
    (3, 150, 257),
    (3, 314, 128),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub format: String,
    pub source_sha256: String,
    pub expanded: bool,
    pub units: Vec<Unit>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub key: String,
    pub records: Vec<Record>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub index: usize,
    pub engine: String,
    pub raw_hex: String,
    pub overrun_hex: String,
    pub locations: Vec<Location>,
    pub spans: Vec<Span>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    pub bank: usize,
    pub start: u16,
    pub end: u16,
    pub selector: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Span {
    pub offset: usize,
    pub length: usize,
    pub message: String,
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|v| format!("{v:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}
pub fn unhex(text: &str) -> Result<Vec<u8>> {
    text.split_whitespace()
        .map(|v| Ok(u8::from_str_radix(v, 16)?))
        .collect()
}
pub fn charmap() -> BTreeMap<u8, char> {
    let values: BTreeMap<String, String> =
        serde_json::from_str(include_str!("../assets/character_map.json"))
            .expect("built-in character map");
    values
        .into_iter()
        .map(|(k, v)| {
            (
                u8::from_str_radix(&k, 16).expect("character code"),
                v.chars().next().expect("character"),
            )
        })
        .collect()
}

impl Catalog {
    pub fn record_count(&self) -> usize {
        self.units.iter().map(|v| v.records.len()).sum()
    }
    pub fn fragment_count(&self) -> usize {
        self.units
            .iter()
            .flat_map(|u| &u.records)
            .map(|r| r.spans.len())
            .sum()
    }
    pub fn unit(&self, key: &str) -> &Unit {
        self.units
            .iter()
            .find(|u| u.key == key)
            .expect("known catalog unit")
    }
}

impl Unit {
    pub fn view(&self) -> Vec<vn_text::Entry> {
        self.records
            .iter()
            .flat_map(|r| &r.spans)
            .map(|s| vn_text::Entry {
                name: None,
                message: s.message.clone(),
            })
            .collect()
    }
    pub fn apply(&mut self, entries: &[vn_text::Entry]) -> Result<usize> {
        let map = charmap();
        let mut cursor = 0;
        let mut changes = 0;
        for record in &mut self.records {
            let old = unhex(&record.raw_hex)?;
            let mut bytes = Vec::new();
            let mut previous = 0;
            let mut new_spans = Vec::new();
            for span in &record.spans {
                let entry = entries.get(cursor).ok_or("翻译片段数不足")?;
                cursor += 1;
                bytes.extend_from_slice(&old[previous..span.offset]);
                let offset = bytes.len();
                let original = &old[span.offset..span.offset + span.length];
                let encoded = if entry.message == span.message {
                    original.to_vec()
                } else {
                    changes += 1;
                    encode(&entry.message, original, record.engine != "fixed", &map).map_err(
                        |e| {
                            format!(
                                "{} record {} fragment {}: {e}",
                                self.key,
                                record.index,
                                cursor - 1
                            )
                        },
                    )?
                };
                bytes.extend_from_slice(&encoded);
                new_spans.push(Span {
                    offset,
                    length: encoded.len(),
                    message: entry.message.clone(),
                });
                previous = span.offset + span.length;
            }
            bytes.extend_from_slice(&old[previous..]);
            if !old.is_empty() && bytes.is_empty() {
                return Err(format!("{} record {} 不可变为空记录", self.key, record.index).into());
            }
            if bytes.len() > 255 {
                return Err(format!(
                    "{} record {} 长度 {} 超过引擎8位游标的255字节上限",
                    self.key,
                    record.index,
                    bytes.len()
                )
                .into());
            }
            record.raw_hex = hex(&bytes);
            record.spans = new_spans;
        }
        if cursor != entries.len() {
            return Err("翻译片段数多于源记录".into());
        }
        Ok(changes)
    }
}

pub fn encode(
    text: &str,
    original: &[u8],
    allow_newline: bool,
    map: &BTreeMap<u8, char>,
) -> Result<Vec<u8>> {
    let mut inverse = BTreeMap::new();
    for (&code, &ch) in map {
        inverse.entry(ch).or_insert(code);
    }
    for (&code, &ch) in map {
        if (0x70..=0x9D).contains(&code) {
            inverse.insert(ch, code);
        }
    }
    let unknown: Vec<u8> = original
        .iter()
        .copied()
        .filter(|v| !(map.contains_key(v) || *v == 0x28 && allow_newline))
        .collect();
    let mut supplied_unknown = Vec::new();
    let mut bytes = Vec::new();
    let mut remaining = text;
    while !remaining.is_empty() {
        if let Some(tail) = remaining.strip_prefix("⟦$") {
            let close = tail.find('⟧').ok_or("未知字节标记缺少右括号")?;
            if close != 2 {
                return Err("未知字节标记必须为 ⟦$XX⟧".into());
            }
            let code = u8::from_str_radix(&tail[..close], 16)?;
            supplied_unknown.push(code);
            bytes.push(code);
            remaining = &tail[close + '⟧'.len_utf8()..];
            continue;
        }
        let ch = remaining.chars().next().ok_or("空字符")?;
        remaining = &remaining[ch.len_utf8()..];
        if ch == '\n' && allow_newline {
            bytes.push(0x28);
            continue;
        }
        if matches!(ch, '\0' | '\r' | '\t') {
            return Err("文本包含非法NUL/CR/Tab".into());
        }
        bytes.push(
            *inverse
                .get(&ch)
                .ok_or_else(|| format!("字符 {ch:?} 尚无游戏编码/字形；本阶段不生成新中文字库"))?,
        );
    }
    if unknown != supplied_unknown {
        return Err("未知字节标记必须保持原有顺序和数量".into());
    }
    Ok(bytes)
}

fn decode(bytes: &[u8], newline: bool, map: &BTreeMap<u8, char>) -> String {
    let mut out = String::new();
    for &value in bytes {
        if value == 0x28 && newline {
            out.push('\n');
        } else if let Some(ch) = map.get(&value) {
            out.push(*ch);
        } else {
            out.push_str(&format!("⟦${value:02X}⟧"));
        }
    }
    out
}

fn spans(raw: &[u8], engine: &str, start: usize, notes: &mut Vec<String>) -> (Vec<Span>, usize) {
    let map = charmap();
    if engine == "opaque_auxiliary" {
        return (Vec::new(), 0);
    }
    if engine == "fixed" {
        return (
            vec![Span {
                offset: 0,
                length: raw.len(),
                message: decode(raw, false, &map),
            }],
            0,
        );
    }
    let mut output = Vec::new();
    let mut run = None;
    let mut cursor = start;
    let mut guard = 0;
    let flush = |output: &mut Vec<Span>, run: &mut Option<usize>, end: usize| {
        if let Some(offset) = run.take() {
            output.push(Span {
                offset,
                length: end - offset,
                message: decode(&raw[offset..end], true, &map),
            });
        }
    };
    while cursor < raw.len() {
        let value = raw[cursor];
        let operands = if engine == "intro_secondary" {
            None
        } else if engine == "intro" {
            match value {
                0x20 => Some(1),
                0x1C | 0x1D | 0x27 => Some(0),
                _ => None,
            }
        } else {
            match value {
                0x0A | 0x1D | 0x20 => Some(1),
                0x16 | 0x1C | 0x22 => Some(2),
                0x18 => Some(4),
                0x19 | 0x27 => Some(0),
                _ => None,
            }
        };
        if value == 0x0E && !engine.starts_with("intro") {
            flush(&mut output, &mut run, cursor);
            if engine.starts_with("macro") {
                cursor = (cursor + 2).min(raw.len());
                continue;
            }
            break;
        }
        if let Some(count) = operands {
            flush(&mut output, &mut run, cursor);
            if cursor + count + 1 > raw.len() {
                guard = cursor + count + 1 - raw.len();
                notes.push(format!(
                    "截断控制段 offset {cursor}: ${value:02X}; 保持原样"
                ));
                break;
            }
            cursor += 1 + count;
            if value == 0x18 && !engine.starts_with("intro") {
                while cursor < raw.len() && raw[cursor] == 0x18 {
                    if cursor + 3 > raw.len() {
                        guard = cursor + 3 - raw.len();
                        notes.push(format!("截断$18扩展 offset {cursor}; 保持原样"));
                        cursor = raw.len();
                        break;
                    }
                    cursor += 3;
                }
            }
            continue;
        }
        run.get_or_insert(cursor);
        cursor += 1;
    }
    flush(&mut output, &mut run, cursor);
    (output, guard)
}

fn guard_bytes(prg: &[u8], bank: usize, end: u16, count: usize, fixed: bool) -> Result<String> {
    if count == 0 {
        return Ok(String::new());
    }
    let start = offset(bank, end, fixed)?;
    if start + count > (bank + 1) * BANK {
        return Err("控制参数越过物理Bank边界，不能安全搬移".into());
    }
    Ok(hex(prg
        .get(start..start + count)
        .ok_or("越界控制参数超出PRG")?))
}

pub fn word(data: &[u8], offset: usize) -> Result<u16> {
    let pair = data.get(offset..offset + 2).ok_or("16位字段越界")?;
    Ok(u16::from_le_bytes([pair[0], pair[1]]))
}
pub fn offset(bank: usize, cpu: u16, fixed: bool) -> Result<usize> {
    let base = if fixed { 0xC000 } else { 0x8000 };
    if (cpu as usize) < base || (!fixed && cpu >= 0xC000) {
        return Err(format!("Bank {bank:X} CPU ${cpu:04X} 窗口不符").into());
    }
    Ok(bank * BANK + cpu as usize - base)
}
pub fn slice_record(prg: &[u8], bank: usize, start: u16, end: u16, fixed: bool) -> Result<Vec<u8>> {
    if end < start || end as usize - start as usize > 255 {
        return Err(format!("Bank {bank:X}记录边界/长度不合法: {start:04X}-{end:04X}").into());
    }
    let a = offset(bank, start, fixed)?;
    let b = a + (end - start) as usize;
    if b > (bank + 1) * BANK || (!fixed && end > 0xBFF2) {
        return Err("记录跨越Bank或占用复位桩".into());
    }
    Ok(prg.get(a..b).ok_or("记录超出PRG")?.to_vec())
}

pub fn parse(rom: &[u8]) -> Result<Catalog> {
    if rom.len() < 16 || &rom[..4] != b"NES\x1A" {
        return Err("不是iNES ROM".into());
    }
    let banks = rom[4] as usize;
    if !matches!(banks, 8 | 16)
        || rom[5] != 16
        || (rom[6] >> 4 | rom[7] & 0xF0) != 1
        || rom[6] & 4 != 0
        || rom[7] & 0x0C != 0
    {
        return Err("只支持本作Mapper1、128/256KiB PRG、128KiB CHR、无trainer的iNES1镜像".into());
    }
    if rom.len() != 16 + banks * BANK + 0x20000 {
        return Err("ROM实际长度与iNES头不一致".into());
    }
    let prg = &rom[16..16 + banks * BANK];
    let fixed = banks - 1;
    let fixed_base = fixed * BANK;
    let at = |cpu: usize| fixed_base + cpu - 0xC000;
    let expanded = banks == 16;
    if prg.get(at(0xD7B5)..at(0xD7BA)) != Some(&[0xA5, 0x5C, 0xAA, 0x0A, 0xA8][..]) {
        return Err("不匹配赤龙王资源解释器代码".into());
    }
    if expanded {
        if prg.get(at(0xECDB)..at(0xECDB) + MARKER.len()) != Some(MARKER) {
            return Err("256KiB镜像缺少本工具闭环标记".into());
        }
    } else if prg.get(at(0xD615)..at(0xD619)) != Some(&[0xA5, 0x10, 0x85, 0x12][..]) {
        return Err("固定ID绘字入口不匹配支持版本".into());
    }
    let mut units = Vec::new();
    let mut make_table =
        |key: &str, table: u16, count: usize, engine_fn: &dyn Fn(usize) -> String| -> Result<()> {
            let mut records = Vec::new();
            for i in 0..count {
                let start = word(prg, at(table as usize) + i * 2)?;
                let end = word(prg, at(table as usize) + (i + 1) * 2)?;
                let data_bank = if expanded { 14 } else { fixed };
                let raw = slice_record(prg, data_bank, start, end, !expanded)?;
                let engine = engine_fn(i);
                let mut notes = Vec::new();
                let start_offset = if engine == "macro_primary" {
                    2.min(raw.len())
                } else {
                    0
                };
                let parsed = spans(&raw, &engine, start_offset, &mut notes);
                records.push(Record {
                    index: i,
                    engine,
                    raw_hex: hex(&raw),
                    overrun_hex: guard_bytes(prg, data_bank, end, parsed.1, !expanded)?,
                    locations: vec![Location {
                        bank: data_bank,
                        start,
                        end,
                        selector: None,
                    }],
                    spans: parsed.0,
                    notes,
                });
            }
            units.push(Unit {
                key: key.into(),
                records,
            });
            Ok(())
        };
    make_table("fixed", 0xEB99, 160, &|_| "fixed".into())?;
    let mut masters: BTreeMap<(usize, usize), Record> = BTreeMap::new();
    for (selector, (master, index_base, boundaries)) in WINDOWS.iter().copied().enumerate() {
        let table = word(prg, at(0xDBFC) + selector * 2)?;
        let bank = prg[at(0xDC08) + selector] as usize;
        if bank >= fixed || (expanded && bank != 8 + selector) {
            return Err("资源Bank映射不匹配支持布局".into());
        }
        let table_offset = offset(bank, table, false)?;
        for i in 0..boundaries - 1 {
            let start = word(prg, table_offset + i * 2)?;
            let end = word(prg, table_offset + (i + 1) * 2)?;
            let raw = slice_record(prg, bank, start, end, false)?;
            let location = Location {
                bank,
                start,
                end,
                selector: Some(selector),
            };
            let slot = masters.entry((master, index_base + i));
            match slot {
                std::collections::btree_map::Entry::Vacant(e) => {
                    let mut notes = Vec::new();
                    let parsed = spans(&raw, "resource", 1.min(raw.len()), &mut notes);
                    e.insert(Record {
                        index: index_base + i,
                        engine: "resource".into(),
                        raw_hex: hex(&raw),
                        overrun_hex: guard_bytes(prg, bank, end, parsed.1, false)?,
                        locations: vec![location],
                        spans: parsed.0,
                        notes,
                    });
                }
                std::collections::btree_map::Entry::Occupied(mut e) => {
                    if e.get().raw_hex != hex(&raw) {
                        return Err(format!(
                            "共享资源记录bank{master} index{} 的副本不一致",
                            index_base + i
                        )
                        .into());
                    }
                    let count = unhex(&e.get().overrun_hex)?.len();
                    if guard_bytes(prg, bank, end, count, false)? != e.get().overrun_hex {
                        return Err("共享资源的越界参数副本不一致".into());
                    }
                    e.get_mut().locations.push(location);
                }
            }
        }
    }
    for bank in 1..=3 {
        let records = masters
            .iter()
            .filter(|((b, _), _)| *b == bank)
            .map(|(_, r)| r.clone())
            .collect::<Vec<_>>();
        let expected = [180, 397, 441][bank - 1];
        if records.len() != expected {
            return Err("资源主表覆盖不完整".into());
        }
        units.push(Unit {
            key: format!("resources_bank{bank:02}"),
            records,
        });
    }
    // The observed $22 selectors choose even records; adjacent odd records are alternate streams.
    let table = 0xE1D0;
    let mut records = Vec::new();
    for i in 0..106 {
        let start = word(prg, at(table) + i * 2)?;
        let end = if i == 52 {
            start // Empty separator/padding between macro text and the fixed title table.
        } else {
            word(prg, at(table) + (i + 1) * 2)?
        };
        let bank = if expanded && start < 0xC000 {
            14
        } else {
            fixed
        };
        let in_fixed = bank == fixed;
        let raw = slice_record(prg, bank, start, end, in_fixed)?;
        let engine = if i >= 52 {
            "opaque_auxiliary"
        } else if i % 2 == 0 {
            "macro_primary"
        } else {
            "macro_alternate"
        };
        let mut notes = Vec::new();
        if (50..52).contains(&i) {
            notes.push(
                "当前资源窗口中未找到直接引用；分支角色按相邻偶/奇记录推定，原始控制字节保持"
                    .into(),
            );
        }
        if i >= 52 {
            notes
                .push("空位或辅助数据（含2×2图块描述样式）；原样保留，不按宏文本导出为译文".into());
        }
        let parsed = spans(
            &raw,
            engine,
            if i % 2 == 0 { 2.min(raw.len()) } else { 0 },
            &mut notes,
        );
        records.push(Record {
            index: i,
            engine: engine.into(),
            raw_hex: hex(&raw),
            overrun_hex: guard_bytes(prg, bank, end, parsed.1, in_fixed)?,
            locations: vec![Location {
                bank,
                start,
                end,
                selector: None,
            }],
            spans: parsed.0,
            notes,
        });
    }
    units.push(Unit {
        key: "macros".into(),
        records,
    });
    let mut records = Vec::new();
    let table_offset = 6 * BANK + 0x5ED;
    for i in 0..6 {
        let start = word(prg, table_offset + i * 2)?;
        let end = word(prg, table_offset + (i + 1) * 2)?;
        let bank = if expanded { 14 } else { 6 };
        let raw = slice_record(prg, bank, start, end, false)?;
        let engine = if i == 4 { "intro_secondary" } else { "intro" };
        let mut notes = Vec::new();
        let parsed = spans(&raw, engine, 1.min(raw.len()), &mut notes);
        records.push(Record {
            index: i,
            engine: engine.into(),
            raw_hex: hex(&raw),
            overrun_hex: guard_bytes(prg, bank, end, parsed.1, false)?,
            locations: vec![Location {
                bank,
                start,
                end,
                selector: None,
            }],
            spans: parsed.0,
            notes,
        });
    }
    units.push(Unit {
        key: "intro".into(),
        records,
    });
    let notes=vec![
        "全部已定位表记录：160固定ID+1018唯一资源槽+106宏/辅助记录+6序章。宏表0–51按选择分支解析，52–105保持辅助原始数据；Bank4事件字节和其余ROM保存在source.nes。".into(),
        "$20的一个参数、$18的四基参数及18+两参数扩展，均为不可编辑控制段；未知字形保持⟦$XX⟧。".into(),
        "译文数组仅包含文本片段，顺序由native.json绑定；勿调换条目，勿修改一次提取元数据。".into(),
    ];
    Ok(Catalog {
        format: "Sekiryuuou script roundtrip v2".into(),
        source_sha256: digest(rom),
        expanded,
        units,
        notes,
    })
}

pub fn check_payloads(a: &Catalog, b: &Catalog) -> Result<()> {
    if a.units.len() != b.units.len() {
        return Err("重提取分组数变化".into());
    }
    for (left, right) in a.units.iter().zip(&b.units) {
        if left.key != right.key || left.records.len() != right.records.len() {
            return Err("重提取记录结构变化".into());
        }
        for (x, y) in left.records.iter().zip(&right.records) {
            if x.index != y.index || x.raw_hex != y.raw_hex || x.overrun_hex != y.overrun_hex {
                return Err(format!("重提取{} record{}原始载荷不一致", left.key, x.index).into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conditional_operands_are_not_exported_as_text() {
        let raw = [
            0, 0x70, 0x18, 1, 2, 3, 4, 0x18, 5, 6, 0x18, 7, 8, 0x71, 0x20, 0x09, 0x72, 0x0E, 0x73,
        ];
        let mut notes = Vec::new();
        let parsed = spans(&raw, "resource", 1, &mut notes);
        assert_eq!(
            parsed
                .0
                .iter()
                .map(|s| s.message.as_str())
                .collect::<Vec<_>>(),
            ["あ", "い", "う"]
        );
        assert!(notes.is_empty());
    }
    #[test]
    fn encoder_preserves_unknown_and_rejects_unallocated_hanzi() {
        let map = charmap();
        assert_eq!(
            encode("か⟦$B0⟧\nあ", &[0x75, 0xB0, 0x28, 0x70], true, &map).unwrap(),
            [0x75, 0xB0, 0x28, 0x70]
        );
        assert!(encode("测试", &[], true, &map).is_err());
        assert!(encode("あ", &[0xB0], true, &map).is_err());
    }
    #[test]
    fn alternate_macros_have_no_primary_header_skip() {
        let mut notes = Vec::new();
        let a = spans(
            &[0x70, 0x71, 0x72, 0x0E, 3, 0x73],
            "macro_alternate",
            0,
            &mut notes,
        );
        assert_eq!(
            a.0.iter().map(|s| s.message.as_str()).collect::<Vec<_>>(),
            ["あいう", "え"]
        );
    }
    #[test]
    fn truncated_control_records_keep_the_exact_outside_operands() {
        let mut notes = Vec::new();
        let parsed = spans(&[0, 0x16], "resource", 1, &mut notes);
        assert!(parsed.0.is_empty());
        assert_eq!(parsed.1, 2);
    }
}
