//! Reconstruct the main Z80 memory image from the table-driven C200 loader.
//!
//! The offsets and loop bounds mirror the verified IDA Z80 disassembly of the
//! R5-R7 loader: C200 is its base, its six three-byte track entries begin at
//! C2E3, C2A7 decodes each track byte into a D88-style physical track slot, and
//! the 0x0FFF byte request rounds up to sixteen 256-byte sectors per block.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use vn_d88::{Decoder, Image, Sector, StandardCodec, Track};

use crate::disk_rebuild::{PreparedD88, RebuildReport, SectorPatchKey};
use crate::Result;

pub const LOADER_BASE: u16 = 0xC200;
pub const LOADER_ENTRY_ADDRESS: u16 = 0xC200;
pub const MAIN_PROGRAM_BASE: u16 = 0x0100;
pub const MAIN_BLOCK_COUNT: usize = 6;
pub const MAIN_BLOCK_BYTES: usize = 0x1000;

const LOADER_BYTES: usize = 3 * 0x100;
const LOADER_TABLE_OFFSET: usize = 0x00E3;
const LOADER_TABLE_ENTRY_BYTES: usize = 3;
const LOADER_REQUESTED_BYTES: usize = 0x0FFF;
const SECTOR_BYTES: usize = 0x100;
const LOADER_SECTORS_PER_BLOCK: usize = LOADER_REQUESTED_BYTES.div_ceil(SECTOR_BYTES);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoaderSectorSource {
    pub record: u16,
    pub physical_ordinal: usize,
    pub source_range: std::ops::Range<usize>,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MainProgramBlock {
    pub index: usize,
    pub load_address: u16,
    pub loaded_end_exclusive: u32,
    pub cylinder: u16,
    pub head: u8,
    pub track_slot: usize,
    pub start_record: u16,
    pub raw_start_record_parameter: u8,
    pub unused_table_byte: u8,
    pub requested_bytes: usize,
    pub loaded_bytes: usize,
    pub source_sectors: Vec<LoaderSectorSource>,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MainProgramReport {
    pub schema: &'static str,
    pub source_size: usize,
    pub source_sha256: String,
    pub loader_base: u16,
    pub loader_entry_address: u16,
    pub loader_sha256: String,
    pub loader_table_address: u16,
    pub loader_sectors: Vec<LoaderSectorSource>,
    pub program_base: u16,
    pub program_size: usize,
    pub program_sha256: String,
    pub blocks: Vec<MainProgramBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainProgramExtraction {
    pub loader: Vec<u8>,
    pub program: Vec<u8>,
    pub report: MainProgramReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainProgramRebuild {
    pub disk: Vec<u8>,
    pub report: MainProgramRebuildReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MainProgramRebuildReport {
    pub schema: &'static str,
    pub source_program_sha256: String,
    pub rebuilt_program_sha256: String,
    pub program_size: usize,
    pub changed_sectors: usize,
    pub disk: RebuildReport,
}

/// Recreates the 0x0100..0x60FF program image from the original D88 bytes.
///
/// This consumes disk A's C0/H0/R5-R7 as the C200 loader and follows its own
/// six-entry table to fetch each block. It rejects missing/duplicate sectors,
/// non-256-byte records, inconsistent CH IDs and unsupported loader layouts.
pub fn extract_main_program(source: &[u8]) -> Result<MainProgramExtraction> {
    let image = StandardCodec
        .decode(source)
        .map_err(|error| format!("D88 解析失败：{error}"))?;
    let boot_track = find_track(&image, 0, 0)?;

    let mut loader = Vec::with_capacity(LOADER_BYTES);
    let mut loader_sectors = Vec::with_capacity(3);
    for record in 5..=7 {
        let sector = unique_record(boot_track, record)?;
        validate_sector(source, sector, 0, 0, record)?;
        loader.extend_from_slice(&source[sector.data_range.clone()]);
        loader_sectors.push(sector_source(source, sector));
    }
    if loader.len() != LOADER_BYTES {
        return Err(format!(
            "C200 loader 长度错误：期望 0x{LOADER_BYTES:X}，得到 0x{:X}",
            loader.len()
        ));
    }
    if loader.get(0..3) != Some(&[0xC3, 0xF5, 0xC2]) {
        return Err(format!(
            "C200 loader 入口不是 IDA 确认的 JP C2F5：实际字节 {:?}",
            loader.get(0..3)
        ));
    }

    let table_end = LOADER_TABLE_OFFSET
        .checked_add(MAIN_BLOCK_COUNT * LOADER_TABLE_ENTRY_BYTES)
        .ok_or_else(|| "loader 表地址溢出".to_owned())?;
    let table = loader
        .get(LOADER_TABLE_OFFSET..table_end)
        .ok_or_else(|| "C200 loader 中六项磁盘表不完整".to_owned())?;

    let mut program = Vec::with_capacity(MAIN_BLOCK_COUNT * MAIN_BLOCK_BYTES);
    let mut blocks = Vec::with_capacity(MAIN_BLOCK_COUNT);
    for index in 0..MAIN_BLOCK_COUNT {
        let entry_start = index * LOADER_TABLE_ENTRY_BYTES;
        let entry = &table[entry_start..entry_start + LOADER_TABLE_ENTRY_BYTES];
        let encoded_track = entry[0];
        let cylinder = u16::from(encoded_track & 0x7F);
        let head = encoded_track >> 7;
        let track_slot = usize::from(cylinder) * 2 + usize::from(head);

        // IDA shows C2A7 masks this parameter's high bit before passing the
        // remaining seven bits as the first sector ID. The high bit is retained
        // verbatim because its controller-side meaning is not needed to locate
        // the matching physical D88 records.
        let raw_start_record_parameter = entry[1];
        let start_record = u16::from(raw_start_record_parameter & 0x7F);
        if start_record == 0 {
            return Err(format!(
                "loader 第 {} 项的起始记录号为 0，不能映射 D88 扇区",
                index + 1
            ));
        }

        let track = find_track(&image, 0, track_slot)?;
        let load_address = usize::from(MAIN_PROGRAM_BASE)
            .checked_add(index * MAIN_BLOCK_BYTES)
            .ok_or_else(|| "主程序目标地址溢出".to_owned())?;
        let mut block_bytes = Vec::with_capacity(MAIN_BLOCK_BYTES);
        let mut sources = Vec::with_capacity(LOADER_SECTORS_PER_BLOCK);
        for offset in 0..LOADER_SECTORS_PER_BLOCK {
            let record = start_record
                .checked_add(u16::try_from(offset).map_err(|_| "扇区序号溢出".to_owned())?)
                .ok_or_else(|| "扇区记录号溢出".to_owned())?;
            let sector = unique_record(track, record)?;
            validate_sector(source, sector, cylinder, head, record)?;
            block_bytes.extend_from_slice(&source[sector.data_range.clone()]);
            sources.push(sector_source(source, sector));
        }
        if block_bytes.len() != MAIN_BLOCK_BYTES {
            return Err(format!(
                "loader 第 {} 项装入长度错误：期望 0x{MAIN_BLOCK_BYTES:X}，得到 0x{:X}",
                index + 1,
                block_bytes.len()
            ));
        }

        let end_exclusive = load_address
            .checked_add(block_bytes.len())
            .ok_or_else(|| "主程序块末地址溢出".to_owned())?;
        let block = MainProgramBlock {
            index: index + 1,
            load_address: u16::try_from(load_address)
                .map_err(|_| "主程序块基址超出 Z80 地址范围".to_owned())?,
            loaded_end_exclusive: u32::try_from(end_exclusive)
                .map_err(|_| "主程序块末地址超出报告范围".to_owned())?,
            cylinder,
            head,
            track_slot,
            start_record,
            raw_start_record_parameter,
            unused_table_byte: entry[2],
            requested_bytes: LOADER_REQUESTED_BYTES,
            loaded_bytes: block_bytes.len(),
            source_sectors: sources,
            sha256: crate::sha256_hex(&block_bytes),
        };
        program.extend_from_slice(&block_bytes);
        blocks.push(block);
    }

    let expected_program_size = MAIN_BLOCK_COUNT * MAIN_BLOCK_BYTES;
    if program.len() != expected_program_size {
        return Err(format!(
            "主程序镜像长度错误：期望 0x{expected_program_size:X}，得到 0x{:X}",
            program.len()
        ));
    }
    if u32::from(MAIN_PROGRAM_BASE) + program.len() as u32 > 0x1_0000 {
        return Err("主程序镜像超过 Z80 地址空间".into());
    }

    let report = MainProgramReport {
        schema: "tenboudai-pc88-main-program-v1",
        source_size: source.len(),
        source_sha256: crate::sha256_hex(source),
        loader_base: LOADER_BASE,
        loader_entry_address: LOADER_ENTRY_ADDRESS,
        loader_sha256: crate::sha256_hex(&loader),
        loader_table_address: LOADER_BASE + LOADER_TABLE_OFFSET as u16,
        loader_sectors,
        program_base: MAIN_PROGRAM_BASE,
        program_size: program.len(),
        program_sha256: crate::sha256_hex(&program),
        blocks,
    };
    Ok(MainProgramExtraction {
        loader,
        program,
        report,
    })
}

/// Replaces the loader-mapped 0x0100..0x60FF program while preserving the
/// original 256-byte sector contract and all unrelated D88 data.
///
/// The caller must supply the complete fixed-size program image. Changed
/// program chunks are mapped back through the verified loader table by
/// physical sector ordinal. The result is decoded again, the main program is
/// re-extracted, and its bytes must match the requested image exactly.
pub fn rebuild_main_program(source: &[u8], program: &[u8]) -> Result<MainProgramRebuild> {
    let extraction = extract_main_program(source)?;
    let replacements = main_program_sector_replacements_for(&extraction, source, program)?;

    let prepared = PreparedD88::new(source.to_vec())?;
    let (disk, disk_report) = prepared.rebuild(&replacements)?;
    let rebuilt_extraction = extract_main_program(&disk)?;
    if rebuilt_extraction.program != program {
        return Err("重建 D88 后重新提取的主程序与目标映像不一致".into());
    }

    Ok(MainProgramRebuild {
        disk,
        report: MainProgramRebuildReport {
            schema: "tenboudai-pc88-main-program-rebuild-v1",
            source_program_sha256: crate::sha256_hex(&extraction.program),
            rebuilt_program_sha256: crate::sha256_hex(&rebuilt_extraction.program),
            program_size: program.len(),
            changed_sectors: disk_report.changed_sectors,
            disk: disk_report,
        },
    })
}

/// Creates fixed-size D88 sector payloads for a rebuilt main-program image.
///
/// Text adapters can merge these with their data-sector patches and perform
/// one preserving D88 reconstruction instead of rebuilding the disk twice.
pub fn main_program_sector_replacements(
    source: &[u8],
    program: &[u8],
) -> Result<BTreeMap<SectorPatchKey, Vec<u8>>> {
    let extraction = extract_main_program(source)?;
    main_program_sector_replacements_for(&extraction, source, program)
}

fn main_program_sector_replacements_for(
    extraction: &MainProgramExtraction,
    source: &[u8],
    program: &[u8],
) -> Result<BTreeMap<SectorPatchKey, Vec<u8>>> {
    if program.len() != extraction.program.len() {
        return Err(format!(
            "主程序映像长度必须为 0x{:X} 字节，实际为 0x{:X}",
            extraction.program.len(),
            program.len()
        ));
    }

    let mut replacements = BTreeMap::new();
    let mut mapped_sectors = BTreeSet::new();
    for block in &extraction.report.blocks {
        let block_offset = usize::from(
            block
                .load_address
                .checked_sub(MAIN_PROGRAM_BASE)
                .ok_or_else(|| format!("主程序块 {} 的装入地址低于映像基址", block.index))?,
        );
        if block.source_sectors.len() != LOADER_SECTORS_PER_BLOCK {
            return Err(format!(
                "主程序块 {} 映射了 {} 个扇区，应为 {} 个",
                block.index,
                block.source_sectors.len(),
                LOADER_SECTORS_PER_BLOCK
            ));
        }

        for (sector_index, sector) in block.source_sectors.iter().enumerate() {
            let program_start = block_offset
                .checked_add(sector_index * SECTOR_BYTES)
                .ok_or_else(|| "主程序扇区偏移溢出".to_owned())?;
            let program_end = program_start
                .checked_add(SECTOR_BYTES)
                .ok_or_else(|| "主程序扇区末尾偏移溢出".to_owned())?;
            let replacement = program
                .get(program_start..program_end)
                .ok_or_else(|| format!("主程序块 {} 的扇区范围越界", block.index))?;
            let original = source
                .get(sector.source_range.clone())
                .ok_or_else(|| format!("主程序块 {} 的原始扇区范围越界", block.index))?;
            if original.len() != SECTOR_BYTES {
                return Err(format!(
                    "主程序块 {} 的物理扇区 #{} 长度不是 0x{:X}",
                    block.index, sector.physical_ordinal, SECTOR_BYTES
                ));
            }
            let key = SectorPatchKey {
                disk_index: 0,
                track_slot: block.track_slot,
                physical_ordinal: sector.physical_ordinal,
            };
            if !mapped_sectors.insert(key) {
                return Err(format!(
                    "多个主程序映像范围映射到同一物理扇区：轨槽 {} physical #{}",
                    block.track_slot, sector.physical_ordinal
                ));
            }
            if original != replacement {
                replacements.insert(key, replacement.to_vec());
            }
        }
    }
    Ok(replacements)
}

/// Ensures an extraction manifest was generated from this exact source D88.
///
/// The exported report includes the source digest plus the loader sectors,
/// loader table and every mapped main-program sector. Recomputing and
/// comparing the complete JSON value also rejects stale or hand-edited maps.
pub fn verify_main_program_map(source: &[u8], mapping_json: &[u8]) -> Result<()> {
    let supplied: serde_json::Value = serde_json::from_slice(mapping_json)
        .map_err(|error| format!("主程序提取映射 JSON 无效：{error}"))?;
    let extraction = extract_main_program(source)?;
    let expected = serde_json::to_value(&extraction.report)
        .map_err(|error| format!("无法生成当前主程序映射：{error}"))?;
    if supplied != expected {
        return Err(format!(
            "主程序提取映射与当前 D88 不匹配（当前来源 SHA-256：{}）",
            extraction.report.source_sha256
        ));
    }
    Ok(())
}

fn find_track(image: &Image, disk_index: usize, track_slot: usize) -> Result<&Track> {
    let disk = image
        .disks
        .get(disk_index)
        .ok_or_else(|| format!("D88 中没有磁盘索引 {disk_index}"))?;
    disk.tracks
        .iter()
        .find(|track| track.slot == track_slot)
        .ok_or_else(|| format!("磁盘 {disk_index} 不含轨槽 {track_slot}"))
}

fn unique_record(track: &Track, record: u16) -> Result<&Sector> {
    let mut found = track
        .sectors
        .iter()
        .filter(|sector| sector.address.id.record == record);
    let sector = found
        .next()
        .ok_or_else(|| format!("轨槽 {} 缺少 R{record}", track.slot))?;
    if found.next().is_some() {
        return Err(format!("轨槽 {} 中 R{record} 重复", track.slot));
    }
    Ok(sector)
}

fn validate_sector(
    source: &[u8],
    sector: &Sector,
    cylinder: u16,
    head: u8,
    record: u16,
) -> Result<()> {
    if sector.address.id.cylinder != cylinder
        || sector.address.id.head != head
        || sector.address.id.record != record
    {
        return Err(format!(
            "D88 CHR 头与 loader 请求不一致：轨槽 {} physical #{} 请求 C{cylinder}/H{head}/R{record}，实际 C{}/H{}/R{}",
            sector.address.track_slot,
            sector.address.physical_ordinal,
            sector.address.id.cylinder,
            sector.address.id.head,
            sector.address.id.record
        ));
    }
    if sector.data_range.end > source.len() || sector.data_range.len() != SECTOR_BYTES {
        return Err(format!(
            "D88 C{cylinder}/H{head}/R{record} 数据长度不是 {SECTOR_BYTES} 字节"
        ));
    }
    Ok(())
}

fn sector_source(source: &[u8], sector: &Sector) -> LoaderSectorSource {
    let bytes = &source[sector.data_range.clone()];
    LoaderSectorSource {
        record: sector.address.id.record,
        physical_ordinal: sector.address.physical_ordinal,
        source_range: sector.data_range.clone(),
        sha256: crate::sha256_hex(bytes),
    }
}

#[cfg(test)]
pub(crate) fn synthetic_loader_disk() -> Vec<u8> {
    let mut loader = vec![0u8; LOADER_BYTES];
    loader[..3].copy_from_slice(&[0xC3, 0xF5, 0xC2]);
    loader[LOADER_TABLE_OFFSET..LOADER_TABLE_OFFSET + 18].copy_from_slice(&[
        0x81, 1, 0, 2, 1, 0, 0x82, 1, 0, 3, 1, 0, 0x83, 1, 0, 4, 1, 0,
    ]);

    let mut image = vec![0u8; vn_d88::EXTENDED_HEADER_SIZE + 5];
    image[vn_d88::EXTENDED_HEADER_SIZE..].copy_from_slice(&[0xA1, 0xB2, 0xC3, 0xD4, 0xE5]);
    for (slot, cylinder, head) in [
        (0usize, 0u8, 0u8),
        (3, 1, 1),
        (4, 2, 0),
        (5, 2, 1),
        (6, 3, 0),
        (7, 3, 1),
        (8, 4, 0),
    ] {
        let track_offset = image.len();
        let table_offset = 0x20 + slot * 4;
        image[table_offset..table_offset + 4].copy_from_slice(&(track_offset as u32).to_le_bytes());
        for record in 1u8..=16 {
            let mut header = [0u8; vn_d88::SECTOR_HEADER_SIZE];
            header[0] = cylinder;
            header[1] = head;
            header[2] = record;
            header[3] = 1;
            header[4..6].copy_from_slice(&16u16.to_le_bytes());
            header[14..16].copy_from_slice(&(SECTOR_BYTES as u16).to_le_bytes());
            image.extend_from_slice(&header);

            if slot == 0 && (5..=7).contains(&record) {
                let start = usize::from(record - 5) * SECTOR_BYTES;
                image.extend_from_slice(&loader[start..start + SECTOR_BYTES]);
            } else {
                image.extend(std::iter::repeat_n(
                    (slot * 16 + usize::from(record)) as u8,
                    SECTOR_BYTES,
                ));
            }
        }
    }
    let size = image.len() as u32;
    image[0x1C..0x20].copy_from_slice(&size.to_le_bytes());
    image.extend_from_slice(&[0xFA, 0xCE, 0xD0, 0x88]);
    image
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_six_loader_mapped_blocks_in_address_order() {
        let source = synthetic_loader_disk();
        let extraction = extract_main_program(&source).expect("extract main program");

        assert_eq!(extraction.loader.len(), 0x300);
        assert_eq!(&extraction.loader[..3], &[0xC3, 0xF5, 0xC2]);
        assert_eq!(extraction.report.loader_sectors.len(), 3);
        assert_eq!(extraction.report.blocks.len(), MAIN_BLOCK_COUNT);
        assert_eq!(extraction.program.len(), 0x6000);
        assert_eq!(extraction.report.program_base, 0x0100);
        assert_eq!(
            extraction
                .report
                .blocks
                .iter()
                .map(|block| (block.track_slot, block.load_address))
                .collect::<Vec<_>>(),
            vec![
                (3, 0x0100),
                (4, 0x1100),
                (5, 0x2100),
                (6, 0x3100),
                (7, 0x4100),
                (8, 0x5100),
            ]
        );
        for block in &extraction.report.blocks {
            let start = usize::from(block.load_address - MAIN_PROGRAM_BASE);
            assert_eq!(
                &extraction.program[start..start + SECTOR_BYTES],
                vec![(block.track_slot * 16 + 1) as u8; SECTOR_BYTES]
            );
            assert_eq!(block.source_sectors.len(), LOADER_SECTORS_PER_BLOCK);
        }
    }

    #[test]
    fn rebuilds_only_changed_fixed_size_program_sectors_and_reextracts_exactly() {
        let source = synthetic_loader_disk();
        let original_extraction = extract_main_program(&source).expect("extract main program");

        let unchanged = rebuild_main_program(&source, &original_extraction.program)
            .expect("rebuild unchanged program");
        assert_eq!(unchanged.disk, source);
        assert_eq!(unchanged.report.changed_sectors, 0);
        assert_eq!(
            unchanged.report.disk.rebuilt_sha256,
            crate::sha256_hex(&source)
        );

        let mut modified_program = original_extraction.program.clone();
        modified_program[0x3000 + 0x10] ^= 0x5A;
        let rebuilt =
            rebuild_main_program(&source, &modified_program).expect("rebuild modified program");

        assert_eq!(rebuilt.disk.len(), source.len());
        assert_eq!(rebuilt.report.changed_sectors, 1);
        assert_eq!(rebuilt.report.disk.replacement_entries, 1);
        let reextracted = extract_main_program(&rebuilt.disk).expect("re-extract rebuilt D88");
        assert_eq!(reextracted.program, modified_program);
        assert_eq!(
            source,
            synthetic_loader_disk(),
            "source bytes remain unchanged"
        );
    }

    #[test]
    fn rejects_loader_tables_that_alias_program_sectors() {
        let mut source = synthetic_loader_disk();
        let image = StandardCodec.decode(&source).expect("decode fixture");
        let boot_track = find_track(&image, 0, 0).expect("boot track");
        let loader_sector = unique_record(boot_track, 5).expect("loader R5");
        let first_entry = loader_sector.data_range.start + LOADER_TABLE_OFFSET;
        let second_entry = first_entry + LOADER_TABLE_ENTRY_BYTES;
        let duplicate = source[second_entry..second_entry + LOADER_TABLE_ENTRY_BYTES].to_vec();
        source[first_entry..first_entry + LOADER_TABLE_ENTRY_BYTES].copy_from_slice(&duplicate);

        let extraction = extract_main_program(&source).expect("extract aliased source image");
        let error = rebuild_main_program(&source, &extraction.program)
            .expect_err("aliased program sectors are rejected");
        assert!(error.contains("映射到同一物理扇区"));
    }

    #[test]
    fn rejects_main_program_rebuilds_with_a_different_image_size() {
        let source = synthetic_loader_disk();
        let extraction = extract_main_program(&source).expect("extract main program");
        let error =
            rebuild_main_program(&source, &extraction.program[..extraction.program.len() - 1])
                .expect_err("variable-size main program is rejected");
        assert!(error.contains("主程序映像长度必须"));
    }

    #[test]
    fn rejects_loader_table_that_points_to_a_missing_track() {
        let mut source = synthetic_loader_disk();
        // D88 header + R5 header + table entry start; change the loader's
        // first physical track byte to a valid encoding of an absent slot.
        let image = StandardCodec.decode(&source).expect("decode fixture");
        let r5 = unique_record(find_track(&image, 0, 0).expect("boot track"), 5).expect("R5");
        let offset = r5.data_range.start + LOADER_TABLE_OFFSET;
        source[offset] = 0x9F;
        let error = extract_main_program(&source).expect_err("missing track is rejected");
        assert!(error.contains("轨槽"));
    }
}
