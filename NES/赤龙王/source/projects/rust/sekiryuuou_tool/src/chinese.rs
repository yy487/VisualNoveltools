//! Chinese script/font layout and the bound MMC3 runtime integration.
use crate::catalog::{self, Catalog, Location, Span, BANK, WINDOWS};
use crate::{json_bytes, rebuild, ExportPlan, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const FIXED_BANK: usize = 31;
const FONT_BANK: usize = 16;
const ORIGINAL_CHR_BANK: usize = 20;
const REGISTRY_BANK: usize = 30;
const REGISTRY_BASE: u16 = 0x8000;
const REGISTRY_ENTRIES: u16 = REGISTRY_BASE + 2;
const DICTIONARY_BANKS: [usize; 3] = [19, 28, 29];
const DATA_END: usize = 0x3FF2;
const LOCAL_BASE: u8 = 0x30;
const LOCAL_COUNT: usize = 0xE0 - LOCAL_BASE as usize;

pub(crate) struct ChinesePlan {
    pub rom: Vec<u8>,
    pub files: BTreeMap<String, Vec<u8>>,
    pub config: ChineseConfig,
    pub report: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ChineseGlyph {
    pub id: u16,
    pub character: String,
    pub width_tiles: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChineseDictionary {
    pub bank: u8,
    pub pointer: u16,
    pub glyph_ids: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChineseSpan {
    pub offset: usize,
    pub length: usize,
    pub fragment: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChineseRecord {
    pub unit: String,
    pub index: usize,
    pub engine: String,
    pub raw_hex: String,
    pub overrun_hex: String,
    pub locations: Vec<Location>,
    pub dict_bank: u8,
    pub dict_pointer: u16,
    pub glyph_ids: Vec<u16>,
    pub spans: Vec<ChineseSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChineseRegistryEntry {
    pub source_bank: u8,
    pub source_start: u16,
    pub dict_bank: u8,
    pub dict_pointer: u16,
    pub unit: String,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChineseTable {
    pub unit: String,
    pub bank: usize,
    pub table: u16,
    pub data_bank: usize,
    pub pointers: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChineseResourceWindow {
    pub selector: usize,
    pub master_unit: String,
    pub index_base: usize,
    pub bank: usize,
    pub table: u16,
    pub pointers: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChineseConfig {
    pub format: String,
    pub state: String,
    pub source_sha256: String,
    pub output_sha256: String,
    pub font_count: usize,
    pub font_bank_base: usize,
    pub font_bytes: usize,
    pub font_data_sha256: String,
    pub original_chr_bank_base: usize,
    pub original_chr_bytes: usize,
    pub registry_bank: usize,
    pub registry_base: u16,
    pub registry_entries: u16,
    pub registry_count: usize,
    pub registry_entry_bytes: usize,
    pub fixed_bank: usize,
    pub local_code_base: u8,
    pub local_code_limit: u8,
    pub empty_dictionary_bank: u8,
    pub dictionaries: Vec<ChineseDictionary>,
    pub records: Vec<ChineseRecord>,
    pub registry: Vec<ChineseRegistryEntry>,
    pub text_tables: Vec<ChineseTable>,
    pub resource_windows: Vec<ChineseResourceWindow>,
}

#[derive(Debug, Clone)]
enum Atom {
    Glyph(char),
    Raw(u8),
    Newline,
}

fn tokenize(text: &str, allow_newline: bool) -> Result<Vec<Atom>> {
    let mut remaining = text;
    let mut atoms = Vec::new();
    while !remaining.is_empty() {
        if let Some(tail) = remaining.strip_prefix("⟦$") {
            let close = tail.find('⟧').ok_or("原始字节占位缺少右括号")?;
            if close != 2 {
                return Err("原始字节占位必须为 ⟦$XX⟧".into());
            }
            let code = u8::from_str_radix(&tail[..close], 16)?;
            if !matches!(code, 0x0B | 0x0C | 0x0D | 0x0F) {
                return Err(format!("中文回注未确认原始字节占位 ${code:02X}").into());
            }
            atoms.push(Atom::Raw(code));
            remaining = &tail[close + '⟧'.len_utf8()..];
            continue;
        }
        let ch = remaining.chars().next().ok_or("空字符")?;
        remaining = &remaining[ch.len_utf8()..];
        match ch {
            '\n' if allow_newline => atoms.push(Atom::Newline),
            '\0' | '\r' | '\t' | '\n' | '⟦' | '⟧' | '<' | '>' => {
                return Err(format!("不合法的文本字符 {ch:?}").into());
            }
            value if value.is_ascii() && value != ' ' && !value.is_ascii_digit() => {
                return Err(format!("暂未支持半角字符 {value:?}").into());
            }
            value => atoms.push(Atom::Glyph(value)),
        }
    }
    Ok(atoms)
}

fn raw_tokens(atoms: &[Atom]) -> Vec<u8> {
    atoms
        .iter()
        .filter_map(|a| match a {
            Atom::Raw(code) => Some(*code),
            _ => None,
        })
        .collect()
}

fn is_half(ch: char) -> bool {
    ch == ' ' || ch.is_ascii_digit()
}

fn font_manifest(
    bytes: &[u8],
    font_data: &[u8],
) -> Result<(Vec<ChineseGlyph>, BTreeMap<char, u16>)> {
    let manifest: Vec<ChineseGlyph> = serde_json::from_slice(bytes)?;
    if manifest.is_empty() || manifest.len() > 3 * BANK / 32 {
        return Err("字库必须包含1到1536个16×16字形".into());
    }
    if font_data.len() != manifest.len() * 32 {
        return Err("1bpp字库大小不是字形数量×32字节".into());
    }
    let mut map = BTreeMap::new();
    let mut previous = None;
    for (index, glyph) in manifest.iter().enumerate() {
        let mut chars = glyph.character.chars();
        let character = chars.next().ok_or("字形字符为空")?;
        if chars.next().is_some()
            || glyph.id as usize != index
            || glyph.width_tiles != 2
            || character.is_ascii()
            || matches!(character, '\n' | '\r' | '\t' | '\0')
            || previous.is_some_and(|old| old >= character)
        {
            return Err("字形表须按Unicode顺序排列单个全角字符，ID连续从0开始，宽度2列".into());
        }
        if character != '\u{3000}'
            && font_data[index * 32..(index + 1) * 32]
                .iter()
                .all(|b| *b == 0)
        {
            return Err(format!("字形 {character:?} 的点阵完全空白").into());
        }
        map.insert(character, glyph.id);
        previous = Some(character);
    }
    Ok((manifest, map))
}

fn encode_atoms(atoms: &[Atom], local: &BTreeMap<char, u8>) -> Result<Vec<u8>> {
    atoms
        .iter()
        .map(|atom| match atom {
            Atom::Raw(code) => Ok(*code),
            Atom::Newline => Ok(0x28),
            Atom::Glyph(' ') => Ok(0x2D),
            Atom::Glyph(ch) if ch.is_ascii_digit() => Ok(*ch as u8 - b'0'),
            Atom::Glyph(ch) => local
                .get(ch)
                .copied()
                .ok_or_else(|| format!("字形 {ch:?} 未进入记录局部字典").into()),
        })
        .collect()
}

fn decode_span(bytes: &[u8], globals: &[u16], manifest: &[ChineseGlyph]) -> Result<String> {
    let mut text = String::new();
    for &code in bytes {
        match code {
            0..=9 => text.push((b'0' + code) as char),
            0x0B | 0x0C | 0x0D | 0x0F => text.push_str(&format!("⟦${code:02X}⟧")),
            0x28 => text.push('\n'),
            0x2D => text.push(' '),
            LOCAL_BASE..=0xDF => {
                let id = *globals
                    .get((code - LOCAL_BASE) as usize)
                    .ok_or("局部字符码超出记录字典")?;
                text.push_str(&manifest.get(id as usize).ok_or("字典字形ID越界")?.character);
            }
            _ => return Err(format!("编译后文本字节 ${code:02X} 没有解码规则").into()),
        }
    }
    Ok(text)
}

fn translation_file<'a>(files: &'a BTreeMap<String, Vec<u8>>, unit: &str) -> Result<&'a [u8]> {
    let canonical = format!("translations/{unit}.json");
    let plain = format!("{unit}.json");
    match (files.get(&canonical), files.get(&plain)) {
        (Some(a), Some(b)) if a != b => Err(format!("{canonical}与{plain}给出了不同译文").into()),
        (Some(bytes), _) | (_, Some(bytes)) => Ok(bytes),
        _ => Err(format!("缺少译文 {unit}.json").into()),
    }
}

fn compile_records(
    original: &Catalog,
    translations: &BTreeMap<String, Vec<u8>>,
    font_ids: &BTreeMap<char, u16>,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(Catalog, Vec<ChineseRecord>, usize, usize)> {
    let mut compiled = original.clone();
    let mut metadata = Vec::new();
    let mut changed = 0;
    let mut br_count = 0;
    for unit in &mut compiled.units {
        let input = translation_file(translations, &unit.key)?;
        let mut supplied = vn_text::read_template(input)?;
        vn_text::validate_shape(&unit.view(), &supplied)?;
        files.insert(
            format!("translations_snapshot/{}.json", unit.key),
            input.to_vec(),
        );
        for entry in &mut supplied {
            br_count += entry.message.matches("<br>").count();
            entry.message = entry.message.replace("<br>", "\n");
        }
        files.insert(
            format!("translations_normalized/{}.json", unit.key),
            vn_text::write_template(&supplied)?,
        );
        let mut cursor = 0;
        for record in &mut unit.records {
            let original_raw = catalog::unhex(&record.raw_hex)?;
            let mut full = BTreeSet::new();
            let mut fragments = Vec::new();
            for span in &record.spans {
                let translated = &supplied[cursor].message;
                let atoms = tokenize(translated, record.engine != "fixed").map_err(|e| {
                    format!("{} record{} fragment{cursor}: {e}", unit.key, record.index)
                })?;
                let old_atoms = tokenize(&span.message, record.engine != "fixed")?;
                if raw_tokens(&old_atoms) != raw_tokens(&atoms) {
                    return Err(format!(
                        "{} record{} fragment{cursor}: 原始字节占位顺序/数量改变",
                        unit.key, record.index
                    )
                    .into());
                }
                if translated != &span.message {
                    changed += 1;
                }
                for atom in &atoms {
                    if let Atom::Glyph(ch) = atom {
                        if !is_half(*ch) {
                            if !font_ids.contains_key(ch) {
                                return Err(format!(
                                    "{} record{} fragment{cursor}: 字库缺少 {ch:?}",
                                    unit.key, record.index
                                )
                                .into());
                            }
                            full.insert(*ch);
                        }
                    }
                }
                fragments.push((cursor, translated.clone(), atoms));
                cursor += 1;
            }
            if full.len() > LOCAL_COUNT {
                return Err(format!(
                    "{} record{}含{}种全角字，局部字典容量{LOCAL_COUNT}",
                    unit.key,
                    record.index,
                    full.len()
                )
                .into());
            }
            let local: BTreeMap<char, u8> = full
                .iter()
                .enumerate()
                .map(|(i, ch)| (*ch, LOCAL_BASE + i as u8))
                .collect();
            let globals = full.iter().map(|ch| font_ids[ch]).collect::<Vec<_>>();
            let mut raw = Vec::new();
            let mut previous = 0;
            let mut spans = Vec::new();
            for (span, (fragment, message, atoms)) in record.spans.iter().zip(fragments) {
                raw.extend_from_slice(&original_raw[previous..span.offset]);
                let offset = raw.len();
                let bytes = encode_atoms(&atoms, &local)?;
                raw.extend_from_slice(&bytes);
                spans.push(ChineseSpan {
                    offset,
                    length: bytes.len(),
                    fragment,
                    message,
                });
                previous = span.offset + span.length;
            }
            raw.extend_from_slice(&original_raw[previous..]);
            if (!original_raw.is_empty() && raw.is_empty()) || raw.len() > 255 {
                return Err(format!(
                    "{} record{}编译后{}字节；须为非空且不超过255字节",
                    unit.key,
                    record.index,
                    raw.len()
                )
                .into());
            }
            record.raw_hex = catalog::hex(&raw);
            record.spans = spans
                .iter()
                .map(|s| Span {
                    offset: s.offset,
                    length: s.length,
                    message: s.message.clone(),
                })
                .collect();
            metadata.push(ChineseRecord {
                unit: unit.key.clone(),
                index: record.index,
                engine: record.engine.clone(),
                raw_hex: record.raw_hex.clone(),
                overrun_hex: record.overrun_hex.clone(),
                locations: Vec::new(),
                dict_bank: 0xFF,
                dict_pointer: 0,
                glyph_ids: globals,
                spans,
            });
        }
        if cursor != supplied.len() {
            return Err("译文片段没有恰好绑定到全部原生记录".into());
        }
    }
    Ok((compiled, metadata, changed, br_count))
}

fn put_dictionary(
    prg: &mut [u8],
    ids: &[u16],
    bank_slot: &mut usize,
    position: &mut usize,
) -> Result<(u8, u16)> {
    if ids.is_empty() {
        return Ok((0xFF, 0));
    }
    let size = ids.len() * 2;
    if *position + size > DATA_END {
        *bank_slot += 1;
        *position = 0;
    }
    let bank = *DICTIONARY_BANKS
        .get(*bank_slot)
        .ok_or("局部字典超过三个16KiB Bank容量")?;
    let pointer = 0x8000 + *position as u16;
    let offset = bank * BANK + *position;
    for (i, id) in ids.iter().enumerate() {
        prg[offset + i * 2..offset + i * 2 + 2].copy_from_slice(&id.to_le_bytes());
    }
    *position += size;
    Ok((bank as u8, pointer))
}

fn table(prg: &[u8], bank: usize, cpu: u16, count: usize, fixed: bool) -> Result<Vec<u16>> {
    let offset = catalog::offset(bank, cpu, fixed)?;
    (0..count)
        .map(|i| catalog::word(prg, offset + i * 2))
        .collect()
}

fn verify_records(prg: &[u8], records: &[ChineseRecord], manifest: &[ChineseGlyph]) -> Result<()> {
    for record in records {
        let expected = catalog::unhex(&record.raw_hex)?;
        let guard = catalog::unhex(&record.overrun_hex)?;
        if expected.len() > 255 || guard.len() > 16 || record.locations.is_empty() {
            return Err("中文记录长度、边界参数或定位无效".into());
        }
        if record.glyph_ids.len() > LOCAL_COUNT
            || record.locations.iter().any(|location| location.bank >= 32)
            || (!record.glyph_ids.is_empty()
                && (!DICTIONARY_BANKS.contains(&(record.dict_bank as usize))
                    || !(0x8000..=0xBFF2).contains(&record.dict_pointer)
                    || record.glyph_ids.len() * 2 > 0xBFF2 - record.dict_pointer as usize))
        {
            return Err("中文记录Bank或局部字典范围无效".into());
        }
        for span in &record.spans {
            if span.offset > expected.len() || span.length > expected.len() - span.offset {
                return Err("中文片段越出其记录".into());
            }
        }
        for location in &record.locations {
            let offset =
                catalog::offset(location.bank, location.start, location.bank == FIXED_BANK)?;
            let end = offset + expected.len();
            if prg.get(offset..end) != Some(expected.as_slice())
                || prg.get(end..end + guard.len()) != Some(guard.as_slice())
            {
                return Err(format!(
                    "重提取{} record{}载荷/边界外参数不一致",
                    record.unit, record.index
                )
                .into());
            }
            for span in &record.spans {
                let decoded = decode_span(
                    &prg[offset + span.offset..offset + span.offset + span.length],
                    &record.glyph_ids,
                    manifest,
                )?;
                if decoded != span.message {
                    return Err(format!(
                        "重提取{} record{} fragment{}中文不一致",
                        record.unit, record.index, span.fragment
                    )
                    .into());
                }
            }
        }
        if !record.glyph_ids.is_empty() {
            let offset = catalog::offset(record.dict_bank as usize, record.dict_pointer, false)?;
            let actual = (0..record.glyph_ids.len())
                .map(|i| catalog::word(prg, offset + i * 2))
                .collect::<Result<Vec<_>>>()?;
            if actual != record.glyph_ids {
                return Err("重提取局部字典不一致".into());
            }
        }
    }
    Ok(())
}

pub(crate) fn compile(
    original_files: &BTreeMap<String, Vec<u8>>,
    translations: &BTreeMap<String, Vec<u8>>,
    font_manifest_bytes: &[u8],
    font_data: &[u8],
) -> Result<ChinesePlan> {
    let source = original_files.get("source.nes").ok_or("缺少source.nes")?;
    let original = catalog::parse(source)?;
    let saved: Catalog =
        serde_json::from_slice(original_files.get("native.json").ok_or("缺少native.json")?)?;
    if saved != original {
        return Err("一次提取与source.nes不一致；请重新导出，勿修改native.json".into());
    }
    if original.expanded {
        return Err("完整中文转换须使用128KiB原版ROM的一次提取".into());
    }
    let (manifest, font_ids) = font_manifest(font_manifest_bytes, font_data)?;
    let mut files = BTreeMap::new();
    let (compiled, mut records, changed, br_count) =
        compile_records(&original, translations, &font_ids, &mut files)?;
    // Reuse the verified table/guard packer and corrected ID/menu/macro hooks.
    let base = rebuild::rebuild(source, &compiled, true)?;
    let expanded = catalog::parse(&base)?;
    catalog::check_payloads(&compiled, &expanded)?;
    let mut prg = vec![0xFF; 32 * BANK];
    prg[..16 * BANK].copy_from_slice(&base[16..16 + 16 * BANK]);
    prg[FIXED_BANK * BANK..(FIXED_BANK + 1) * BANK]
        .copy_from_slice(&base[16 + 15 * BANK..16 + 16 * BANK]);
    let reset = source[16 + BANK - 14..16 + BANK].to_vec();
    for bank in [19, 28, 29, 30] {
        prg[(bank + 1) * BANK - 14..(bank + 1) * BANK].copy_from_slice(&reset);
    }
    // Asset bank tails are image/font bytes, not executable MMC1 reset stubs.
    prg[FONT_BANK * BANK..FONT_BANK * BANK + font_data.len()].copy_from_slice(font_data);
    let source_chr = &source[source.len() - 0x20000..];
    prg[ORIGINAL_CHR_BANK * BANK..ORIGINAL_CHR_BANK * BANK + source_chr.len()]
        .copy_from_slice(source_chr);
    let mut dictionaries = Vec::new();
    let mut dictionary_locations: BTreeMap<Vec<u16>, (u8, u16)> = BTreeMap::new();
    let mut bank_slot = 0;
    let mut position = 0;
    let mut registry = Vec::new();
    for record in &mut records {
        let actual = &expanded.unit(&record.unit).records[record.index];
        if record.index != actual.index {
            return Err("原生记录顺序索引不连续".into());
        }
        record.locations = actual
            .locations
            .iter()
            .map(|location| {
                let mut location = location.clone();
                if location.bank == 15 {
                    location.bank = FIXED_BANK;
                }
                location
            })
            .collect();
        if record.engine == "opaque_auxiliary" {
            continue;
        }
        let (bank, pointer) = if let Some(location) = dictionary_locations.get(&record.glyph_ids) {
            *location
        } else {
            let location =
                put_dictionary(&mut prg, &record.glyph_ids, &mut bank_slot, &mut position)?;
            dictionary_locations.insert(record.glyph_ids.clone(), location);
            if !record.glyph_ids.is_empty() {
                dictionaries.push(ChineseDictionary {
                    bank: location.0,
                    pointer: location.1,
                    glyph_ids: record.glyph_ids.clone(),
                });
            }
            location
        };
        record.dict_bank = bank;
        record.dict_pointer = pointer;
        if !record.raw_hex.is_empty() {
            for location in &record.locations {
                registry.push(ChineseRegistryEntry {
                    source_bank: location.bank as u8,
                    source_start: location.start,
                    dict_bank: bank,
                    dict_pointer: pointer,
                    unit: record.unit.clone(),
                    index: record.index,
                });
            }
        }
    }
    registry.sort_by_key(|entry| (entry.source_bank, entry.source_start));
    for pair in registry.windows(2) {
        if (pair[0].source_bank, pair[0].source_start)
            == (pair[1].source_bank, pair[1].source_start)
        {
            return Err("字典context registry存在重复源地址".into());
        }
    }
    let registry_bytes = 2 + registry.len() * 6;
    if registry_bytes > DATA_END || registry.len() > u16::MAX as usize {
        return Err("记录context registry超过Bank30容量".into());
    }
    let registry_offset = REGISTRY_BANK * BANK;
    prg[registry_offset..registry_offset + 2]
        .copy_from_slice(&(registry.len() as u16).to_le_bytes());
    for (i, entry) in registry.iter().enumerate() {
        let offset = registry_offset + 2 + i * 6;
        prg[offset] = entry.source_bank;
        prg[offset + 1..offset + 3].copy_from_slice(&entry.source_start.to_le_bytes());
        prg[offset + 3] = entry.dict_bank;
        prg[offset + 4..offset + 6].copy_from_slice(&entry.dict_pointer.to_le_bytes());
    }
    verify_records(&prg, &records, &manifest)?;
    let aux_table_offset = 0xE23A - 0xC000;
    let aux_table_end = 0xE2A6 - 0xC000;
    if prg[FIXED_BANK * BANK + aux_table_offset..FIXED_BANK * BANK + aux_table_end]
        != source[16 + 7 * BANK + aux_table_offset..16 + 7 * BANK + aux_table_end]
    {
        return Err("固定Bank地点标题图块指针表发生改变".into());
    }
    let mut text_tables = Vec::new();
    for (unit, bank, cpu, count) in [
        ("fixed", FIXED_BANK, 0xEB99, 161),
        ("macros", FIXED_BANK, 0xE1D0, 53),
        ("intro", 6, 0x85ED, 7),
    ] {
        text_tables.push(ChineseTable {
            unit: unit.into(),
            bank,
            table: cpu,
            data_bank: 14,
            pointers: table(&prg, bank, cpu, count, bank == FIXED_BANK)?,
        });
    }
    let mut resource_windows = Vec::new();
    for (selector, (master, index_base, count)) in WINDOWS.iter().copied().enumerate() {
        resource_windows.push(ChineseResourceWindow {
            selector,
            master_unit: format!("resources_bank{master:02}"),
            index_base,
            bank: 8 + selector,
            table: 0x8000,
            pointers: table(&prg, 8 + selector, 0x8000, count, false)?,
        });
    }
    let mut header = [0u8; 16];
    header[..4].copy_from_slice(b"NES\x1A");
    header[4] = 32;
    header[6] = 0x40 | (source[6] & 1);
    header[7] = 0x08;
    header[10] = 7; // 64 << 7 = 8 KiB PRG-RAM, no battery-backed memory.
    header[11] = 9; // 64 << 9 = 32 KiB CHR-RAM, no CHR-ROM.
    let mut rom = header.to_vec();
    rom.extend_from_slice(&prg);
    let config = ChineseConfig {
        format: "Sekiryuuou Chinese MMC3 layout v1".into(),
        state: "runtime_assembly_patch_required".into(),
        source_sha256: catalog::digest(source),
        output_sha256: catalog::digest(&rom),
        font_count: manifest.len(),
        font_bank_base: FONT_BANK,
        font_bytes: font_data.len(),
        font_data_sha256: catalog::digest(font_data),
        original_chr_bank_base: ORIGINAL_CHR_BANK,
        original_chr_bytes: source_chr.len(),
        registry_bank: REGISTRY_BANK,
        registry_base: REGISTRY_BASE,
        registry_entries: REGISTRY_ENTRIES,
        registry_count: registry.len(),
        registry_entry_bytes: 6,
        fixed_bank: FIXED_BANK,
        local_code_base: LOCAL_BASE,
        local_code_limit: 0xDF,
        empty_dictionary_bank: 0xFF,
        dictionaries,
        records,
        registry,
        text_tables,
        resource_windows,
    };
    let report = serde_json::json!({
        "source_sha256": config.source_sha256, "output_sha256": config.output_sha256,
        "source_records": compiled.record_count(), "editable_fragments": compiled.fragment_count(),
        "changed_fragments": changed, "br_normalized_count": br_count,
        "font_count": manifest.len(), "dictionary_count": config.dictionaries.len(),
        "registry_count": config.registry_count,
        "max_record_bytes": config.records.iter().map(|r| r.raw_hex.split_whitespace().count()).max().unwrap_or(0),
        "max_local_glyphs": config.records.iter().map(|r| r.glyph_ids.len()).max().unwrap_or(0),
        "immutable_controls_and_headers_preserved": true, "raw_tokens_preserved": true,
        "normalized_chinese_reextracted_equal": true, "record_alias_payloads_equal": true,
        "overrun_parameters_preserved": true, "aux_title_table_and_payloads_preserved": true,
        "original_chr_asset_unchanged": true, "font_asset_unchanged": true,
        "prg_kib": 512, "chr_rom_kib": 0, "chr_ram_kib": 32, "prg_ram_kib": 8,
        "mapper": 4, "runtime_ready": false,
        "scope": "Compiled script/font/record-context data only. Runtime rendering, Mapper writes, cache and pagination assembly must be patched before playing."
    });
    files.insert("config.json".into(), json_bytes(&config)?);
    files.insert("verification.json".into(), json_bytes(&report)?);
    files.insert("font_manifest.json".into(), font_manifest_bytes.to_vec());
    files.insert("font_data_1bpp.bin".into(), font_data.to_vec());
    Ok(ChinesePlan {
        rom,
        files,
        config,
        report,
    })
}

#[derive(Deserialize)]
struct RuntimePatch {
    bank: usize,
    cpu: u16,
    expected: String,
    replacement: String,
}

#[derive(Deserialize)]
struct Runtime {
    source_sha256: String,
    patches: Vec<RuntimePatch>,
}

pub(crate) fn integrate_runtime(mut plan: ChinesePlan) -> Result<ChinesePlan> {
    let runtime_bytes = include_bytes!("../assets/chinese_runtime.json");
    let runtime: Runtime = serde_json::from_slice(runtime_bytes)?;
    if runtime.source_sha256 != plan.config.source_sha256 {
        return Err("完整中文字库运行时仅支持已核对的 Sekiryuuou 日本版原始ROM".into());
    }
    let original_chr_at = 16 + ORIGINAL_CHR_BANK * BANK;
    let font_at = 16 + FONT_BANK * BANK;
    let original_chr = plan.rom[original_chr_at..original_chr_at + 0x20000].to_vec();
    let font = plan.rom[font_at..font_at + plan.config.font_bytes].to_vec();
    let mut touched = BTreeSet::new();
    for patch in runtime.patches {
        if !matches!(patch.bank, 6 | FIXED_BANK) {
            return Err("运行时补丁不在允许的代码Bank中".into());
        }
        let expected = catalog::unhex(&patch.expected)?;
        let replacement = catalog::unhex(&patch.replacement)?;
        if expected.is_empty() || expected.len() != replacement.len() {
            return Err("运行时补丁大小无效".into());
        }
        let at = 16 + catalog::offset(patch.bank, patch.cpu, patch.bank == FIXED_BANK)?;
        if at + expected.len() > 16 + (patch.bank + 1) * BANK
            || plan.rom.get(at..at + expected.len()) != Some(expected.as_slice())
            || !(at..at + expected.len()).all(|position| touched.insert(position))
        {
            return Err(format!(
                "运行时补丁不匹配或重叠：Bank{:02X} ${:04X}",
                patch.bank, patch.cpu
            )
            .into());
        }
        plan.rom[at..at + replacement.len()].copy_from_slice(&replacement);
    }
    let manifest: Vec<ChineseGlyph> = serde_json::from_slice(&plan.files["font_manifest.json"])?;
    verify_records(&plan.rom[16..], &plan.config.records, &manifest)?;
    if plan.rom[original_chr_at..original_chr_at + original_chr.len()] != original_chr
        || plan.rom[font_at..font_at + font.len()] != font
    {
        return Err("运行时补丁改动了字模或原场景图形".into());
    }
    plan.config.state = "runtime_integrated".into();
    plan.config.output_sha256 = catalog::digest(&plan.rom);
    plan.report["runtime_ready"] = true.into();
    plan.report["output_sha256"] = plan.config.output_sha256.clone().into();
    plan.report["scope"] = "Chinese script/font data and MMC3/32KiB CHR-RAM runtime integrated; gameplay checks are separate evidence.".into();
    plan.files
        .insert("config.json".into(), json_bytes(&plan.config)?);
    plan.files
        .insert("verification.json".into(), json_bytes(&plan.report)?);
    plan.files
        .insert("runtime.json".into(), runtime_bytes.to_vec());
    Ok(plan)
}

pub(crate) fn reextract(rom: &[u8], metadata: &[u8], manifest_bytes: &[u8]) -> Result<ExportPlan> {
    let config: ChineseConfig = serde_json::from_slice(metadata)?;
    if rom.len() != 16 + 32 * BANK || config.output_sha256 != catalog::digest(rom) {
        return Err("中文镜像与其config.json不一致".into());
    }
    if config.format != "Sekiryuuou Chinese MMC3 layout v1"
        || config.state != "runtime_integrated"
        || rom[..8] != *b"NES\x1A\x20\x00\x40\x08"
        || (rom[10], rom[11]) != (7, 9)
        || config.font_bank_base != FONT_BANK
        || config.font_bytes > 3 * BANK
        || config.font_count.checked_mul(32) != Some(config.font_bytes)
    {
        return Err("中文镜像布局或字体元数据无效".into());
    }
    let font_at = 16 + FONT_BANK * BANK;
    let font_data = &rom[font_at..font_at + config.font_bytes];
    if config.font_data_sha256 != catalog::digest(font_data) {
        return Err("中文镜像字模与元数据不一致".into());
    }
    let (manifest, _) = font_manifest(manifest_bytes, font_data)?;
    verify_records(&rom[16..], &config.records, &manifest)?;
    let mut grouped: BTreeMap<String, Vec<vn_text::Entry>> = BTreeMap::new();
    for record in &config.records {
        let entries = grouped.entry(record.unit.clone()).or_default();
        let Some(location) = record.locations.first() else {
            return Err("中文记录缺少镜像定位".into());
        };
        let at = 16 + catalog::offset(location.bank, location.start, location.bank == FIXED_BANK)?;
        for span in &record.spans {
            if span.fragment != entries.len() {
                return Err("中文片段顺序不连续".into());
            }
            let text = decode_span(
                &rom[at + span.offset..at + span.offset + span.length],
                &record.glyph_ids,
                &manifest,
            )?;
            entries.push(vn_text::Entry {
                message: text,
                name: None,
            });
        }
    }
    let files = grouped
        .into_iter()
        .map(|(unit, entries)| {
            Ok((
                format!("translations/{unit}.json"),
                vn_text::write_template(&entries)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    Ok(ExportPlan {
        files,
        summary: "从中文镜像重新读取字节、字典并导出六份中文译文".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (BTreeMap<String, Vec<u8>>, BTreeMap<String, Vec<u8>>) {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../workspaces/sekiryuuou/01_unpacked");
        let mut rom = std::fs::read(root.join("ines_header.bin")).unwrap();
        rom.extend(std::fs::read(root.join("prg_rom.bin")).unwrap());
        rom.extend(std::fs::read(root.join("chr_rom.bin")).unwrap());
        let source = crate::export_plan(&rom).unwrap().files;
        let mut translations = BTreeMap::new();
        for (key, value) in &source {
            if !key.starts_with("translations/") {
                continue;
            }
            let mut entries = vn_text::read_template(value).unwrap();
            for entry in &mut entries {
                let values = tokenize(&entry.message, !key.ends_with("fixed.json")).unwrap();
                entry.message = values
                    .iter()
                    .map(|atom| match atom {
                        Atom::Raw(code) => format!("⟦${code:02X}⟧"),
                        Atom::Newline => "\n".into(),
                        Atom::Glyph(ch) if is_half(*ch) => ch.to_string(),
                        Atom::Glyph(_) => "中".into(),
                    })
                    .collect();
            }
            translations.insert(key.clone(), vn_text::write_template(&entries).unwrap());
        }
        (source, translations)
    }

    #[test]
    fn real_catalog_compiles_with_native_controls_aliases_and_assets_intact() {
        let (source, translations) = fixture();
        let manifest = br#"[{"id":0,"character":"\u4e2d","width_tiles":2}]"#;
        let font = vec![0xFF; 32];
        let plan = compile(&source, &translations, manifest, &font).unwrap();
        assert_eq!(plan.rom.len(), 16 + 32 * BANK);
        assert_eq!(&plan.rom[..8], b"NES\x1A\x20\x00\x40\x08");
        assert_eq!((plan.rom[10], plan.rom[11]), (7, 9));
        assert_eq!(plan.config.font_count, 1);
        assert_eq!(
            &plan.rom[16 + FONT_BANK * BANK..16 + FONT_BANK * BANK + 32],
            font
        );
        assert_eq!(
            &plan.rom[16 + ORIGINAL_CHR_BANK * BANK..16 + (ORIGINAL_CHR_BANK + 8) * BANK],
            &source["source.nes"][source["source.nes"].len() - 0x20000..]
        );
        assert_eq!(plan.config.records.len(), 1290);
        assert!(!plan
            .config
            .registry
            .iter()
            .any(|r| r.unit == "macros" && r.index >= 52));
        assert_eq!(plan.report["runtime_ready"], false);
        let registry = 16 + REGISTRY_BANK * BANK;
        assert_eq!(
            u16::from_le_bytes([plan.rom[registry], plan.rom[registry + 1]]) as usize,
            plan.config.registry_count
        );
        assert_eq!(
            plan.config
                .records
                .iter()
                .filter(|r| r.engine == "opaque_auxiliary")
                .count(),
            54
        );
    }

    #[test]
    fn malformed_native_font_shape_and_unknown_tokens_are_rejected() {
        let (mut source, mut translations) = fixture();
        let manifest = br#"[{"id":0,"character":"\u4e2d","width_tiles":2}]"#;
        assert!(compile(&source, &translations, manifest, &[0xFF; 31]).is_err());
        let key = "translations/resources_bank03.json";
        let mut entries = vn_text::read_template(&translations[key]).unwrap();
        let entry = entries
            .iter_mut()
            .find(|e| e.message.contains("⟦$0F⟧"))
            .unwrap();
        entry.message = "中".into();
        translations.insert(key.into(), vn_text::write_template(&entries).unwrap());
        assert!(compile(&source, &translations, manifest, &[0xFF; 32]).is_err());
        source.insert("native.json".into(), b"{}".to_vec());
        assert!(compile(&source, &translations, manifest, &[0xFF; 32]).is_err());
        assert!(tokenize("<br>", true).is_err());
        assert!(tokenize("⟦$29⟧", true).is_err());
        assert!(tokenize("中\n中", false).is_err());
    }

    #[test]
    fn runtime_integrates_and_chinese_bytes_reextract_losslessly() {
        let (source, translations) = fixture();
        let manifest = br#"[{"id":0,"character":"\u4e2d","width_tiles":2}]"#;
        let plan =
            integrate_runtime(compile(&source, &translations, manifest, &[0xFF; 32]).unwrap())
                .unwrap();
        assert_eq!(plan.report["runtime_ready"], true);
        let exported = reextract(&plan.rom, &plan.files["config.json"], manifest).unwrap();
        for (key, actual) in exported.files {
            let normalized = key.replace("translations/", "translations_normalized/");
            assert_eq!(actual, plan.files[&normalized]);
        }
        let mut corrupted = plan.rom.clone();
        corrupted[16 + 14 * BANK] ^= 1;
        assert!(reextract(&corrupted, &plan.files["config.json"], manifest).is_err());
        let mut metadata = plan.config.clone();
        metadata.records[0].locations[0].bank = usize::MAX;
        assert!(reextract(&plan.rom, &json_bytes(&metadata).unwrap(), manifest).is_err());
        metadata = plan.config.clone();
        metadata.font_bytes = usize::MAX;
        assert!(reextract(&plan.rom, &json_bytes(&metadata).unwrap(), manifest).is_err());
    }
}
