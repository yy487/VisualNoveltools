use crate::catalog::{self, Catalog, Unit, BANK, MARKER, WINDOWS};
use crate::Result;

const ID_WRAPPER: u16 = 0xE2A6;
const MACRO_WRAPPER: u16 = 0xE2D0;
const FETCH: u16 = 0xE330;
const CMP20: u16 = 0xE370;
const DELAY: u16 = 0xE376;
const STA16: u16 = 0xE380;
const ANDDF: u16 = 0xE386;
const CMP28: u16 = 0xE38C;
const NEXT_BOUNDARY: u16 = 0xE398;
const ID_LOW: u16 = 0xE3E0;
const ID_HIGH: u16 = 0xE3EA;
const EMPTY_ID: u16 = 0xE400;
const MENU_WRAPPER: u16 = 0xE420;

fn put_word(data: &mut [u8], offset: usize, word: u16) {
    data[offset..offset + 2].copy_from_slice(&word.to_le_bytes());
}
fn records(unit: &Unit) -> Result<Vec<Vec<u8>>> {
    unit.records
        .iter()
        .map(|r| catalog::unhex(&r.raw_hex))
        .collect()
}

fn write_pool(
    prg: &mut [u8],
    bank: usize,
    start: u16,
    end: u16,
    raw: &[Vec<u8>],
    fixed: bool,
) -> Result<Vec<u16>> {
    let capacity = (end - start) as usize;
    let size: usize = raw.iter().map(Vec::len).sum();
    if size > capacity {
        return Err(format!("Bank {bank:02X} ${start:04X}-${end:04X} 文本需要{size}字节，容量{capacity}；请启用扩容或缩短文本").into());
    }
    let offset = catalog::offset(bank, start, fixed)?;
    prg[offset..offset + capacity].fill(0xFF);
    let mut pointers = vec![start];
    let mut pos = offset;
    for record in raw {
        if record.len() > 255 {
            return Err("记录超过引擎255字节上限".into());
        }
        prg[pos..pos + record.len()].copy_from_slice(record);
        pos += record.len();
        pointers.push(start + (pos - offset) as u16);
    }
    Ok(pointers)
}
fn put_table(prg: &mut [u8], bank: usize, cpu: u16, pointers: &[u16], fixed: bool) -> Result<()> {
    let offset = catalog::offset(bank, cpu, fixed)?;
    if offset + pointers.len() * 2 > (bank + 1) * BANK {
        return Err("指针表跨Bank".into());
    }
    for (i, p) in pointers.iter().enumerate() {
        put_word(prg, offset + i * 2, *p);
    }
    Ok(())
}

fn put_guards(
    prg: &mut [u8],
    bank: usize,
    pointers: &[u16],
    unit: &Unit,
    fixed: bool,
    owned_end: u16,
) -> Result<()> {
    for (i, record) in unit.records.iter().enumerate() {
        let guard = catalog::unhex(&record.overrun_hex)?;
        if guard.is_empty() {
            continue;
        }
        let cpu = pointers[i + 1];
        let offset = catalog::offset(bank, cpu, fixed)?;
        if offset + guard.len() > (bank + 1) * BANK {
            return Err("越界控制参数跨Bank".into());
        }
        if i + 1 < unit.records.len() || cpu as usize + guard.len() > owned_end as usize {
            if prg[offset..offset + guard.len()] != guard {
                return Err(format!(
                    "{} record{}的参数延伸至相邻记录/非文本区，不能改写其字节",
                    unit.key, record.index
                )
                .into());
            }
        } else {
            prg[offset..offset + guard.len()].copy_from_slice(&guard);
        }
    }
    Ok(())
}

pub fn rebuild(source: &[u8], catalog: &Catalog, expand: bool) -> Result<Vec<u8>> {
    let original = catalog::parse(source)?;
    if !expand && catalog::check_payloads(&original, catalog).is_ok() {
        return Ok(source.to_vec());
    }
    if expand || catalog.expanded {
        return expanded(source, catalog);
    }
    let mut output = source.to_vec();
    let prg = &mut output[16..16 + 8 * BANK];
    for (key, bank, table, start, end) in [
        ("fixed", 7, 0xEB99, 0xECDB, 0xEF9E),
        ("macros", 7, 0xE1D0, 0xE2A6, 0xEB88),
        ("intro", 6, 0x85ED, 0x85FB, 0x875C),
        ("resources_bank01", 1, 0xA000, 0xA16A, 0xBAC2),
        ("resources_bank02", 2, 0x8000, 0x831C, 0xB334),
        ("resources_bank03", 3, 0x8000, 0x8374, 0xB5B2),
    ] {
        let mut raw = records(catalog.unit(key))?;
        let end = if key == "macros" {
            raw.truncate(52);
            catalog.unit(key).records[53].locations[0].start
        } else {
            end
        };
        let pointers = write_pool(prg, bank, start, end, &raw, bank == 7)?;
        put_table(prg, bank, table, &pointers, bank == 7)?;
        put_guards(prg, bank, &pointers, catalog.unit(key), bank == 7, end)?;
    }
    Ok(output)
}

fn jump(address: u16) -> [u8; 3] {
    let [lo, hi] = address.to_le_bytes();
    [0x4C, lo, hi]
}
fn call(address: u16) -> [u8; 3] {
    let [lo, hi] = address.to_le_bytes();
    [0x20, lo, hi]
}

fn patch(
    prg: &mut [u8],
    bank: usize,
    cpu: u16,
    expected: &[u8],
    replacement: &[u8],
    fixed: bool,
) -> Result<()> {
    if expected.len() != replacement.len() {
        return Err("补丁未覆盖完整等长指令区间".into());
    }
    let offset = catalog::offset(bank, cpu, fixed)?;
    let old = &prg[offset..offset + expected.len()];
    if old != expected && old != replacement {
        return Err(format!("Bank {bank:02X} ${cpu:04X} 待替换指令不匹配").into());
    }
    prg[offset..offset + replacement.len()].copy_from_slice(replacement);
    Ok(())
}
fn put_code(prg: &mut [u8], cpu: u16, code: &[u8], limit: u16) -> Result<()> {
    if cpu as usize + code.len() > limit as usize {
        return Err("固定读取钩子互相覆盖".into());
    }
    let offset = catalog::offset(15, cpu, true)?;
    prg[offset..offset + code.len()].copy_from_slice(code);
    Ok(())
}

fn expanded(source: &[u8], catalog: &Catalog) -> Result<Vec<u8>> {
    let mut header = source[..16].to_vec();
    header[4] = 16;
    let mut prg = vec![0xFF; 16 * BANK];
    // Retain the old Bank 07, so pre-existing switchable-bank references stay valid.
    prg[..8 * BANK].copy_from_slice(&source[16..16 + 8 * BANK]);
    let original_fixed = prg[7 * BANK..8 * BANK].to_vec();
    prg[15 * BANK..16 * BANK].copy_from_slice(&original_fixed);
    let reset = prg[BANK - 14..BANK].to_vec();
    for bank in 8..15 {
        prg[(bank + 1) * BANK - 14..(bank + 1) * BANK].copy_from_slice(&reset);
    }
    for (selector, (master, index, count)) in WINDOWS.iter().copied().enumerate() {
        let bank = 8 + selector;
        let unit = catalog.unit(&format!("resources_bank{master:02}"));
        let raw = unit.records[index..index + count - 1]
            .iter()
            .map(|r| catalog::unhex(&r.raw_hex))
            .collect::<Result<Vec<_>>>()?;
        let start = 0x8000 + (count * 2) as u16;
        let subset = Unit {
            key: unit.key.clone(),
            records: unit.records[index..index + count - 1].to_vec(),
        };
        let tail_guard =
            catalog::unhex(&subset.records.last().ok_or("空资源窗口")?.overrun_hex)?.len();
        let pointers = write_pool(
            &mut prg,
            bank,
            start,
            0xBFF2 - tail_guard as u16,
            &raw,
            false,
        )?;
        put_table(&mut prg, bank, 0x8000, &pointers, false)?;
        put_guards(&mut prg, bank, &pointers, &subset, false, 0xBFF2)?;
        put_word(&mut prg, 15 * BANK + 0x1BFC + selector * 2, 0x8000);
        prg[15 * BANK + 0x1C08 + selector] = bank as u8;
    }
    let mut cpu = 0x8000;
    for (key, table, bank, fixed_table) in [
        ("fixed", 0xEB99, 15, true),
        ("macros", 0xE1D0, 15, true),
        ("intro", 0x85ED, 6, false),
    ] {
        let mut raw = records(catalog.unit(key))?;
        if key == "macros" {
            // E23A is also a scene-title descriptor table. Its fixed-bank
            // auxiliary entries must remain at their original addresses.
            raw.truncate(52);
        }
        let pointers = write_pool(&mut prg, 14, cpu, 0xBFF2, &raw, false)?;
        cpu = *pointers.last().ok_or("空指针表")?;
        put_table(&mut prg, bank, table, &pointers, fixed_table)?;
    }
    hooks(&mut prg, &original_fixed)?;
    let marker_offset = catalog::offset(15, 0xECDB, true)?;
    prg[marker_offset..marker_offset + MARKER.len()].copy_from_slice(MARKER);
    let mut output = header;
    output.extend_from_slice(&prg);
    output.extend_from_slice(&source[source.len() - 0x20000..]);
    Ok(output)
}

fn hooks(prg: &mut [u8], original_fixed: &[u8]) -> Result<()> {
    let mut code = vec![
        0xA5, 0x10, 0x85, 0x12, 0xA5, 0x85, 0x48, 0xA9, 0x0E, 0x85, 0x85,
    ];
    code.extend(call(0xF521));
    code.extend(call(0xD619));
    code.extend([0x68, 0x85, 0x85]);
    code.extend(call(0xF521));
    code.push(0x60);
    put_code(prg, ID_WRAPPER, &code, MACRO_WRAPPER)?;
    let mut replacement = jump(ID_WRAPPER).to_vec();
    replacement.push(0xEA);
    patch(
        prg,
        15,
        0xD615,
        &[0xA5, 0x10, 0x85, 0x12],
        &replacement,
        true,
    )?;

    // Read $22's selector operands from the caller's bank before selecting the macro bank.
    let expected = &original_fixed[0x167E..0x1692];
    if expected != catalog::unhex("E6 19 A4 19 B1 10 0A 0A 0A 0A 85 00 C8 84 19 B1 10 18 65 00")? {
        return Err("宏参数读取代码不匹配".into());
    }
    let mut code = vec![0xA5, 0x85, 0x48];
    code.extend_from_slice(expected);
    code.push(0x48);
    code.extend([0xA9, 0x0E, 0x85, 0x85]);
    code.extend(call(0xF521));
    code.push(0x68);
    code.extend(call(0xD692));
    code.extend([0x68, 0x85, 0x85]);
    code.extend(call(0xF521));
    code.push(0x60);
    put_code(prg, MACRO_WRAPPER, &code, FETCH)?;
    let mut replacement = jump(MACRO_WRAPPER).to_vec();
    replacement.push(0xEA);
    patch(
        prg,
        15,
        0xD67E,
        &[0xE6, 0x19, 0xA4, 0x19],
        &replacement,
        true,
    )?;

    // Store the fetched byte in a dummy stack slot, then restore the bank, X/Y and flags.
    let mut code = catalog::unhex("48 08 8A 48 98 48 A5 85 48 A9 0E 85 85")?;
    code.extend(call(0xF521));
    code.extend(catalog::unhex("B1 10 BA 9D 05 01 68 85 85")?);
    code.extend(call(0xF521));
    code.extend(catalog::unhex("68 A8 68 AA 28 68 60")?);
    put_code(prg, FETCH, &code, CMP20)?;
    for (address, next, original) in [
        (CMP20, DELAY, vec![0xC9, 0x20]),
        (DELAY, STA16, call(0x88A7).to_vec()),
        (STA16, ANDDF, vec![0x85, 0x16]),
        (ANDDF, CMP28, vec![0x29, 0xDF]),
        (CMP28, NEXT_BOUNDARY, vec![0xC9, 0x28]),
    ] {
        let mut wrapper = call(FETCH).to_vec();
        wrapper.extend(original);
        wrapper.push(0x60);
        put_code(prg, address, &wrapper, next)?;
    }
    for (address, expected, target) in [
        (0x8489, "B1 10 C9 20", CMP20),
        (0x84D8, "B1 10 20 A7 88", DELAY),
        (0x8762, "B1 10 85 16", STA16),
        (0x877A, "B1 10 29 DF", ANDDF),
    ] {
        let expected = catalog::unhex(expected)?;
        let mut replacement = call(target).to_vec();
        replacement.resize(expected.len(), 0xEA);
        patch(prg, 6, address, &expected, &replacement, false)?;
    }
    // Credits use the same secondary renderer but read bank-6 tables 9799/97B1.
    // Restore the older expansion's overly broad hooks when rebuilding it.
    for (address, original, target) in [
        (0x91AB, "B1 10 C9 28", CMP28),
        (0x9207, "B1 10 85 16", STA16),
        (0x921F, "B1 10 29 DF", ANDDF),
    ] {
        let original = catalog::unhex(original)?;
        let mut old_hook = call(target).to_vec();
        old_hook.resize(original.len(), 0xEA);
        patch(prg, 6, address, &old_hook, &original, false)?;
    }
    // $5D=$FF needs boundary 256. The old ADC/ASL sequence loses the carry at wraparound.
    let mut code = catalog::unhex("68 C9 FF F0 08 18 69 01 0A A8")?;
    code.extend(jump(0xD7F0));
    code.extend(catalog::unhex(
        "A0 00 E6 01 E6 01 B1 00 85 02 C8 B1 00 C6 01 C6 01",
    )?);
    code.extend(jump(0xD805));
    put_code(prg, NEXT_BOUNDARY, &code, ID_LOW)?;
    let mut replacement = jump(NEXT_BOUNDARY).to_vec();
    replacement.push(0xEA);
    patch(
        prg,
        15,
        0xD7EA,
        &[0x68, 0x18, 0x69, 0x01],
        &replacement,
        true,
    )?;
    // ASL's carry selects the second table page, including boundary 128 for ID 127.
    for (address, low) in [(ID_LOW, 0x99), (ID_HIGH, 0x9A)] {
        let code = [0x90, 0x04, 0xB9, low, 0xEC, 0x60, 0xB9, low, 0xEB, 0x60];
        put_code(prg, address, &code, 0xE420)?;
    }
    for (address, low, target) in [
        (0xD622, 0x99, ID_LOW),
        (0xD627, 0x9A, ID_HIGH),
        (0xD632, 0x99, ID_LOW),
        (0xD637, 0x9A, ID_HIGH),
    ] {
        patch(prg, 15, address, &[0xB9, low, 0xEB], &call(target), true)?;
    }
    let code = catalog::unhex("85 2D D0 03 4C 6E D6 A9 22 4C 45 D6")?;
    put_code(prg, EMPTY_ID, &code, 0xE420)?;
    let mut replacement = jump(EMPTY_ID).to_vec();
    replacement.push(0xEA);
    patch(
        prg,
        15,
        0xD641,
        &[0x85, 0x2D, 0xA9, 0x22],
        &replacement,
        true,
    )?;
    // C7CA draws command names via the EB99 table at offset EBE3, bypassing D615.
    let mut code = catalog::unhex("A5 85 48 A9 0E 85 85")?;
    code.extend(call(0xF521));
    code.extend(call(0xD82A));
    code.extend(catalog::unhex("68 85 85")?);
    code.extend(call(0xF521));
    code.push(0x60);
    put_code(prg, MENU_WRAPPER, &code, 0xE450)?;
    patch(prg, 15, 0xC824, &call(0xD82A), &call(MENU_WRAPPER), true)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cycle_plan, export_plan, import_plan};
    use std::path::Path;
    fn fixture() -> Vec<u8> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../workspaces/sekiryuuou/01_unpacked");
        ["ines_header.bin", "prg_rom.bin", "chr_rom.bin"]
            .into_iter()
            .flat_map(|name| std::fs::read(root.join(name)).expect("unpacked Sekiryuuou fixture"))
            .collect()
    }
    #[test]
    fn complete_real_rom_cycle_is_lossless() {
        let source = fixture();
        let catalog = catalog::parse(&source).unwrap();
        assert_eq!(catalog.record_count(), 1290);
        assert!(catalog.unit("macros").records[53].spans.is_empty());
        assert_eq!(catalog.unit("macros").records[53].raw_hex, "22 CF DF EF FF");
        let result = cycle_plan(&source).unwrap();
        assert_eq!(result.files["rebuilt/original_roundtrip.nes"], source);
        let expanded = &result.files["rebuilt/expanded.nes"];
        assert_eq!(expanded.len(), 393232);
        assert_eq!(
            &expanded[expanded.len() - 0x20000..],
            &source[source.len() - 0x20000..]
        );
        assert_eq!(&expanded[16..16 + 6 * BANK], &source[16..16 + 6 * BANK]);
        assert_eq!(
            &expanded[16 + 7 * BANK..16 + 8 * BANK],
            &source[16 + 7 * BANK..16 + 8 * BANK]
        );
    }
    #[test]
    fn translated_growth_roundtrips_and_aliases_stay_equal() {
        let source = fixture();
        let mut bundle = export_plan(&source).unwrap();
        let key = "translations/resources_bank03.json";
        let mut entries = vn_text::read_template(&bundle.files[key]).unwrap();
        let i = entries
            .iter()
            .position(|e| !e.message.contains('⟦') && e.message.chars().count() < 60)
            .unwrap();
        entries[i].message.push_str("あいうえお");
        bundle
            .files
            .insert(key.into(), vn_text::write_template(&entries).unwrap());
        let edited = import_plan(&bundle.files, true).unwrap();
        assert_eq!(edited.report["changed_fragments"], 1);
        let exported = export_plan(&edited.rom).unwrap();
        let view = vn_text::read_template(&exported.files[key]).unwrap();
        assert_eq!(view[i].message, entries[i].message);
        assert_eq!(import_plan(&exported.files, false).unwrap().rom, edited.rom);
    }
    #[test]
    fn expansion_keeps_scene_title_descriptors_fixed_and_credits_local() {
        let source = fixture();
        let result = cycle_plan(&source).unwrap();
        let expanded = &result.files["rebuilt/expanded.nes"];
        let before = catalog::parse(&source).unwrap();
        let after = catalog::parse(expanded).unwrap();
        for record in &after.unit("macros").records[53..] {
            assert_eq!(record.locations[0].bank, 15);
            assert!(record.locations[0].start >= 0xC000);
            assert_eq!(
                record.raw_hex,
                before.unit("macros").records[record.index].raw_hex
            );
        }
        for (address, size) in [(0x91AB, 4), (0x9207, 4), (0x921F, 4)] {
            let at = 16 + catalog::offset(6, address, false).unwrap();
            assert_eq!(&expanded[at..at + size], &source[at..at + size]);
        }
        let at = 16 + catalog::offset(15, 0xC824, true).unwrap();
        assert_eq!(&expanded[at..at + 3], call(MENU_WRAPPER));
    }
    #[test]
    fn shorter_native_macros_do_not_move_fixed_scene_titles() {
        let source = fixture();
        let mut bundle = export_plan(&source).unwrap();
        let key = "translations/macros.json";
        let mut entries = vn_text::read_template(&bundle.files[key]).unwrap();
        entries[0].message = "あ".into();
        bundle
            .files
            .insert(key.into(), vn_text::write_template(&entries).unwrap());
        let edited = import_plan(&bundle.files, false).unwrap();
        let table = 16 + 7 * BANK + 0x23A;
        assert_eq!(&edited.rom[table..table + 108], &source[table..table + 108]);
        let aux = 16 + 7 * BANK + 0x29BF;
        assert_eq!(&edited.rom[aux..aux + 474], &source[aux..aux + 474]);
    }
    #[test]
    fn source_metadata_and_capacity_fail_before_writing() {
        let source = fixture();
        let mut bundle = export_plan(&source).unwrap();
        bundle.files.get_mut("source.nes").unwrap()[16] ^= 1;
        assert!(import_plan(&bundle.files, true).is_err());
        let mut bundle = export_plan(&source).unwrap();
        let key = "translations/fixed.json";
        let mut entries = vn_text::read_template(&bundle.files[key]).unwrap();
        entries[0].message = "あ".repeat(256);
        bundle
            .files
            .insert(key.into(), vn_text::write_template(&entries).unwrap());
        assert!(import_plan(&bundle.files, true).is_err());
    }
}
