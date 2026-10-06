//! Text resources proven by the game's Z80 text readers.
//!
//! This adapter intentionally models only the mode-2 resources proven to feed
//! the C000 indexed-string reader or the D300/E900 tagged-record reader. It does
//! not infer text from arbitrary bytes or try to parse unrelated opcodes.

use std::collections::{BTreeMap, BTreeSet};

use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use vn_d88::{Decoder, Image, Sector, StandardCodec};
use vn_font::font_88::{self, FontResources};
use vn_text::Entry;

use crate::disk_rebuild::{PreparedD88, RebuildReport, SectorPatchKey};
use crate::font::Pc88FontReport;
use crate::{sha256_hex, Result};

const MODE2_TABLE_TRACK_SLOT: usize = 74; // C37/H0
const MODE2_TABLE_RECORD: u16 = 33;
const MODE2_TABLE_LOADED_BYTES: usize = 0x4E;
const MAIN_PROGRAM_BASE: u16 = 0x0100;
const MAIN_PROGRAM_TABLE_ADDRESS: u16 = 0x3A1D;
const MODE2_TEXT_SELECTORS: [u8; 3] = [0, 1, 5];
// IDs absent from every confirmed active glyph stream: mode-2 selectors
// 0/1/5/11/12 and mode-4 selectors 5/6. The OP08 ROM prompts also do not use
// these IDs (including the bounded 0xBC..0xC0 runtime placeholder range).
// The readers treat IDs below 0xE0 other than 0 and D300/E900 newline 0x0D
// as glyph-table lookups.
const SINGLE_BYTE_DONOR_TOKENS: &[u8] = &[
    0x34, 0x4C, 0x60, 0x62, 0x63, 0x6A, 0x77, 0x7B, 0x82, 0x83, 0x87, 0x8A, 0xA9, 0xB3, 0xB5, 0xB6,
    0xB7, 0xB9, 0xBA, 0xBB, 0xD6, 0xDF, 0x02, 0x0B, 0x0F, 0x10, 0x11, 0x12, 0x17,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextReader {
    C000,
    D300,
    E900,
}

impl TextReader {
    fn for_selector(selector: u8) -> Result<Self> {
        match selector {
            0 => Ok(Self::C000),
            1 => Ok(Self::D300),
            5 => Ok(Self::E900),
            _ => Err(format!("未确认 selector {selector} 的文本 reader")),
        }
    }

    fn has_record_newlines(self) -> bool {
        matches!(self, Self::D300 | Self::E900)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DescriptorBoundary {
    pub track_code: u8,
    pub record: u8,
    pub offset: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextSourceSpan {
    pub disk_index: usize,
    pub track_slot: usize,
    pub physical_ordinal: usize,
    pub record: u16,
    pub source_data_offset: usize,
    pub sector_data_offset: usize,
    pub resource_offset: usize,
    pub length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextPrimaryRecord {
    pub index: usize,
    pub stream_start: usize,
    pub stream_end: usize,
    pub tag: u8,
    pub raw_record_hex: String,
    pub raw_body_hex: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextPrimaryResource {
    pub selector: u8,
    pub reader: TextReader,
    pub start: DescriptorBoundary,
    pub end: DescriptorBoundary,
    pub capacity: usize,
    pub source_sha256: String,
    pub records: Vec<TextPrimaryRecord>,
    pub trailing_zero_padding_hex: String,
    pub source_spans: Vec<TextSourceSpan>,
}

/// Adapter-owned identity and decoding record, separate from translator JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextPrimaryManifest {
    pub schema: String,
    pub source_d88_sha256: String,
    pub main_program_sha256: String,
    pub single_byte_table_sha256: String,
    pub loaded_mode2_table_sha256: String,
    pub loaded_mode2_table_bytes: usize,
    pub resources: Vec<TextPrimaryResource>,
}

#[derive(Debug, Clone)]
struct ResourceData {
    primary: TextPrimaryResource,
    bytes: Vec<u8>,
    raw_bodies: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
struct ParsedRecords {
    records: Vec<TextPrimaryRecord>,
    raw_bodies: Vec<Vec<u8>>,
    trailing_padding: Vec<u8>,
}

/// A complete parse of the currently supported, IDA-confirmed text sources.
#[derive(Debug, Clone)]
pub struct TextExtraction {
    pub primary: TextPrimaryManifest,
    pub entries: Vec<Entry>,
    pub original_double_byte_codes: Vec<u16>,
    resources: Vec<ResourceData>,
    single_byte_table: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextCompressionMapping {
    pub character: char,
    pub token: u8,
    pub jis: u16,
    pub occurrences_by_selector: Vec<(u8, usize)>,
    pub saved_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextBuildReport {
    pub source_d88_sha256: String,
    pub rebuilt_d88_sha256: String,
    pub changed_records: usize,
    pub total_records: usize,
    pub resource_capacities: Vec<(u8, usize)>,
    pub compression_mappings: Vec<TextCompressionMapping>,
    pub compression_saved_bytes: usize,
    pub disk_rebuild: RebuildReport,
}

#[derive(Debug, Clone)]
pub struct TextBuild {
    pub disk: Vec<u8>,
    pub kanji1_rom: Vec<u8>,
    pub generated_glyph_table: Option<Vec<u8>>,
    pub font: Pc88FontReport,
    pub report: TextBuildReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BoundaryPosition {
    track_slot: usize,
    record: u16,
    offset: usize,
}

impl TextExtraction {
    pub fn translation_entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Re-encodes edited messages, patches only the source selector spans,
    /// rebuilds the D88, and reparses the output before returning it.
    pub fn rebuild_translations(
        &self,
        source: &[u8],
        translations: &[Entry],
        kanji1_rom: &[u8],
    ) -> Result<TextBuild> {
        self.rebuild_translations_with_font_face(
            source,
            translations,
            kanji1_rom,
            font_88::DEFAULT_GLYPH_FONT_FACE,
        )
    }

    pub fn rebuild_translations_with_font_face(
        &self,
        source: &[u8],
        translations: &[Entry],
        kanji1_rom: &[u8],
        font_face: &str,
    ) -> Result<TextBuild> {
        self.rebuild_translations_with_font_face_and_glyph_table(
            source,
            translations,
            kanji1_rom,
            font_face,
            None,
        )
    }

    pub fn rebuild_translations_with_font_face_and_glyph_table(
        &self,
        source: &[u8],
        translations: &[Entry],
        kanji1_rom: &[u8],
        font_face: &str,
        additional_glyph_table: Option<&[u8]>,
    ) -> Result<TextBuild> {
        if sha256_hex(source) != self.primary.source_d88_sha256 {
            return Err("文本来源 D88 已变化；请从当前原盘重新导出 primary.json".into());
        }
        vn_text::validate_shape(&self.entries, translations)
            .map_err(|error| format!("翻译 JSON 结构与当前文本不匹配：{error}"))?;
        if translations.iter().any(|entry| entry.name.is_some()) {
            return Err("展望台当前导出的文本没有可回写 name 字段".into());
        }

        let messages = translations
            .iter()
            .map(|entry| entry.message.clone())
            .collect::<Vec<_>>();
        let font_messages = messages
            .iter()
            .map(|message| strip_opaque_token_markers(message))
            .collect::<Vec<_>>();
        let font_output =
            crate::font::build_pc88_kanji1_with_font_face_and_glyph_table_and_reserved_jis_codes(
                kanji1_rom,
                &self.original_double_byte_codes,
                &font_messages,
                font_face,
                additional_glyph_table,
                &self.single_byte_table,
            )?;
        let font_resources = &font_output.resources;
        let original_single_reverse = build_single_reverse_map(&self.single_byte_table)?;
        let mut uncompressed_sizes = Vec::with_capacity(self.resources.len());
        let mut changed_records = 0usize;
        let mut entry_index = 0usize;
        for resource in &self.resources {
            let mut byte_len = 0usize;
            for (record_index, record) in resource.primary.records.iter().enumerate() {
                let translation = translations
                    .get(entry_index)
                    .ok_or_else(|| "翻译记录数量在资源重建时发生变化".to_owned())?;
                let original_body = resource
                    .raw_bodies
                    .get(record_index)
                    .ok_or_else(|| "primary 记录缺少原始 token 字节".to_owned())?;
                let body = if translation.message == record.message {
                    original_body.clone()
                } else {
                    changed_records += 1;
                    encode_message(
                        resource.primary.reader,
                        &translation.message,
                        font_resources,
                        &font_output.report.plan,
                        &original_single_reverse,
                        &BTreeMap::new(),
                    )?
                };
                byte_len = byte_len
                    .checked_add(body.len() + 2)
                    .ok_or_else(|| "文本资源编码长度溢出".to_owned())?;
                entry_index += 1;
            }
            uncompressed_sizes.push(byte_len);
        }
        if entry_index != translations.len() {
            return Err("翻译 JSON 含有未映射到文本资源的记录".into());
        }

        let compression_mappings = plan_single_byte_compression(
            &self.resources,
            translations,
            &uncompressed_sizes,
            &original_single_reverse,
            font_resources,
            &font_output.report.plan,
        )?;
        let compressed_by_character = compression_mappings
            .iter()
            .map(|mapping| (mapping.character, mapping.token))
            .collect::<BTreeMap<_, _>>();
        let compressed_by_token = compression_mappings
            .iter()
            .map(|mapping| (mapping.token, mapping.character))
            .collect::<BTreeMap<_, _>>();
        let compressed_tokens = compression_mappings
            .iter()
            .map(|mapping| mapping.token)
            .collect::<BTreeSet<_>>();
        let single_reverse =
            build_single_reverse_map_excluding(&self.single_byte_table, &compressed_tokens)?;

        let mut output_resources = Vec::with_capacity(self.resources.len());
        let mut entry_index = 0usize;
        for resource in &self.resources {
            let mut bytes = Vec::with_capacity(resource.bytes.len());
            for (record_index, record) in resource.primary.records.iter().enumerate() {
                let translation = translations
                    .get(entry_index)
                    .ok_or_else(|| "翻译记录数量在资源重建时发生变化".to_owned())?;
                let original_body = resource
                    .raw_bodies
                    .get(record_index)
                    .ok_or_else(|| "primary 记录缺少原始 token 字节".to_owned())?;
                let body = if translation.message == record.message {
                    original_body.clone()
                } else {
                    encode_message(
                        resource.primary.reader,
                        &translation.message,
                        font_resources,
                        &font_output.report.plan,
                        &single_reverse,
                        &compressed_by_character,
                    )?
                };
                bytes.push(record.tag);
                bytes.extend_from_slice(&body);
                bytes.push(0);
                entry_index += 1;
            }
            if bytes.len() > resource.bytes.len() {
                return Err(format!(
                    "selector {} 注入需要 {} 字节，原 descriptor 只有 {} 字节；拒绝覆盖相邻资源",
                    resource.primary.selector,
                    bytes.len(),
                    resource.bytes.len()
                ));
            }
            bytes.resize(resource.bytes.len(), 0);
            output_resources.push(bytes);
        }

        let prepared = PreparedD88::new(source.to_vec())?;
        let mut patched_program = crate::program::extract_main_program(source)?.program;
        let program_table_offset = usize::from(MAIN_PROGRAM_TABLE_ADDRESS - MAIN_PROGRAM_BASE) + 2;
        for mapping in &compression_mappings {
            let offset = program_table_offset + usize::from(mapping.token) * 2;
            let table_entry = patched_program
                .get_mut(offset..offset + 2)
                .ok_or_else(|| format!("token 0x{:02X} 的主程序单字节表项越界", mapping.token))?;
            table_entry.copy_from_slice(&mapping.jis.to_le_bytes());
        }
        let mut replacement_payloads =
            crate::program::main_program_sector_replacements(source, &patched_program)?;
        for (resource, replacement) in self.resources.iter().zip(&output_resources) {
            for span in &resource.primary.source_spans {
                let key = SectorPatchKey {
                    disk_index: span.disk_index,
                    track_slot: span.track_slot,
                    physical_ordinal: span.physical_ordinal,
                };
                let sector = find_sector_by_ordinal(
                    prepared.image(),
                    key.disk_index,
                    key.track_slot,
                    key.physical_ordinal,
                )?;
                let payload = replacement_payloads
                    .entry(key)
                    .or_insert_with(|| source[sector.data_range.clone()].to_vec());
                let destination_end = span
                    .sector_data_offset
                    .checked_add(span.length)
                    .ok_or_else(|| "文本扇区写入范围溢出".to_owned())?;
                let source_end = span
                    .resource_offset
                    .checked_add(span.length)
                    .ok_or_else(|| "文本资源写入范围溢出".to_owned())?;
                let destination = payload
                    .get_mut(span.sector_data_offset..destination_end)
                    .ok_or_else(|| "文本扇区写入范围越界".to_owned())?;
                let input = replacement
                    .get(span.resource_offset..source_end)
                    .ok_or_else(|| "文本资源写入范围越界".to_owned())?;
                destination.copy_from_slice(input);
            }
        }
        let (disk, disk_rebuild) = prepared.rebuild(&replacement_payloads)?;
        if crate::program::extract_main_program(&disk)?.program != patched_program {
            return Err("重建 D88 后重新提取的主程序与单字节压缩映射不一致".into());
        }

        let rebuilt = parse_text_source(&disk)?;
        let carrier_targets = font_output
            .report
            .plan
            .mapping_used
            .iter()
            .map(|mapping| (mapping.carrier_cp932, mapping.target))
            .collect::<BTreeMap<_, _>>();
        let actual = rebuilt
            .resources
            .iter()
            .flat_map(|resource| {
                resource.raw_bodies.iter().map(|body| {
                    decode_body_with_carriers(
                        resource.primary.reader,
                        body,
                        &rebuilt.single_byte_table,
                        &carrier_targets,
                        &compressed_by_token,
                    )
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let expected = translations.iter().map(|entry| &entry.message);
        if actual.iter().ne(expected) {
            return Err("重建 D88 后重新解析出的文本与翻译不一致".into());
        }
        for (before, after) in self
            .primary
            .resources
            .iter()
            .zip(&rebuilt.primary.resources)
        {
            if before.selector != after.selector
                || before.records.len() != after.records.len()
                || before
                    .records
                    .iter()
                    .zip(&after.records)
                    .any(|(a, b)| a.tag != b.tag)
                || before.capacity != after.capacity
            {
                return Err(format!(
                    "重建后 selector {} 的索引、tag 或 descriptor 容量发生变化",
                    before.selector
                ));
            }
        }

        Ok(TextBuild {
            disk,
            kanji1_rom: font_output.rom,
            generated_glyph_table: font_output.generated_glyph_table,
            font: font_output.report,
            report: TextBuildReport {
                source_d88_sha256: self.primary.source_d88_sha256.clone(),
                rebuilt_d88_sha256: disk_rebuild.rebuilt_sha256.clone(),
                changed_records,
                total_records: translations.len(),
                resource_capacities: self
                    .resources
                    .iter()
                    .map(|resource| (resource.primary.selector, resource.primary.capacity))
                    .collect(),
                compression_saved_bytes: compression_mappings
                    .iter()
                    .map(|mapping| mapping.saved_bytes)
                    .sum(),
                compression_mappings,
                disk_rebuild,
            },
        })
    }
}

/// Reconstructs and decodes only mode-2 selector 0/1/5 resources proven to
/// feed sub_5652 or the sub_3090/sub_317F/sub_30A0 reader chain.
pub fn parse_text_source(source: &[u8]) -> Result<TextExtraction> {
    let main = crate::program::extract_main_program(source)?;
    let image = StandardCodec
        .decode(source)
        .map_err(|error| format!("D88 解析失败：{error}"))?;
    let single_byte_table = read_single_byte_table(&main.program)?;
    let table_sector =
        find_sector_by_record(&image, 0, MODE2_TABLE_TRACK_SLOT, MODE2_TABLE_RECORD)?;
    let mode2_table = source
        .get(table_sector.data_range.clone())
        .and_then(|bytes| bytes.get(..MODE2_TABLE_LOADED_BYTES))
        .ok_or_else(|| "C37/H0/R33 中缺少 IDA 确認的 0x4E 字节 mode-2 表".to_owned())?;
    let table_sha256 = sha256_hex(mode2_table);

    let mut resources = Vec::with_capacity(MODE2_TEXT_SELECTORS.len());
    let mut entries = Vec::new();
    let mut original_double_byte_codes = BTreeSet::new();
    for selector in MODE2_TEXT_SELECTORS {
        let reader = TextReader::for_selector(selector)?;
        let start_index = usize::from(selector) * 3;
        let end_index = start_index + 3;
        let start_raw = mode2_table
            .get(start_index..start_index + 3)
            .ok_or_else(|| format!("selector {selector} 的起始 descriptor 未加载"))?;
        let end_raw = mode2_table
            .get(end_index..end_index + 3)
            .ok_or_else(|| format!("selector {selector} 的结束 descriptor 未加载"))?;
        let start = parse_boundary(start_raw)?;
        let end = parse_boundary(end_raw)?;
        validate_known_text_boundary(selector, start, end)?;
        let (bytes, source_spans) = read_descriptor_range(&image, source, start, end)?;
        let parsed = parse_records(&bytes, reader, &single_byte_table)?;
        if parsed.records.is_empty() {
            return Err(format!(
                "文本 selector {selector} 没有可由 reader 索引的记录"
            ));
        }
        let expected = match selector {
            0 => Some((0x277, 40)),
            1 => Some((0x1024, 142)),
            5 => Some((0x893, 60)),
            _ => None,
        };
        if let Some((capacity, record_count)) = expected {
            if bytes.len() != capacity || parsed.records.len() != record_count {
                return Err(format!(
                    "文本 selector {selector} 与 IDA/原盘审计锚不符：容量 {} (期望 {capacity:#X})，记录 {} (期望 {record_count})",
                    bytes.len(),
                    parsed.records.len()
                ));
            }
        }
        for body in &parsed.raw_bodies {
            collect_original_double_byte_codes(
                body,
                reader,
                &single_byte_table,
                &mut original_double_byte_codes,
            )?;
        }
        let resource = TextPrimaryResource {
            selector,
            reader,
            start,
            end,
            capacity: bytes.len(),
            source_sha256: sha256_hex(&bytes),
            records: parsed.records,
            trailing_zero_padding_hex: bytes_to_hex(&parsed.trailing_padding),
            source_spans,
        };
        entries.extend(resource.records.iter().map(|record| Entry {
            name: None,
            message: record.message.clone(),
        }));
        resources.push(ResourceData {
            primary: resource,
            bytes,
            raw_bodies: parsed.raw_bodies,
        });
    }

    let primary = TextPrimaryManifest {
        schema: "tenboudai-pc88-text-primary-v2".into(),
        source_d88_sha256: sha256_hex(source),
        main_program_sha256: main.report.program_sha256,
        single_byte_table_sha256: sha256_hex(&single_byte_table_bytes(&single_byte_table)),
        loaded_mode2_table_sha256: table_sha256,
        loaded_mode2_table_bytes: MODE2_TABLE_LOADED_BYTES,
        resources: resources
            .iter()
            .map(|resource| resource.primary.clone())
            .collect(),
    };
    Ok(TextExtraction {
        primary,
        entries,
        original_double_byte_codes: original_double_byte_codes.into_iter().collect(),
        resources,
        single_byte_table,
    })
}

fn validate_known_text_boundary(
    selector: u8,
    start: DescriptorBoundary,
    end: DescriptorBoundary,
) -> Result<()> {
    let expected = match selector {
        0 => (
            DescriptorBoundary {
                track_code: 0x25,
                record: 0x22,
                offset: 0xA4,
            },
            DescriptorBoundary {
                track_code: 0x25,
                record: 0x25,
                offset: 0x1B,
            },
        ),
        1 => (
            DescriptorBoundary {
                track_code: 0x25,
                record: 0x25,
                offset: 0x1B,
            },
            DescriptorBoundary {
                track_code: 0xA5,
                record: 0x22,
                offset: 0x3F,
            },
        ),
        5 => (
            DescriptorBoundary {
                track_code: 0xA5,
                record: 0x27,
                offset: 0x51,
            },
            DescriptorBoundary {
                track_code: 0xA5,
                record: 0x2F,
                offset: 0xE4,
            },
        ),
        _ => return Err(format!("selector {selector} 没有已审计的文本边界")),
    };
    if (start, end) != expected {
        return Err(format!(
            "selector {selector} 的 mode-2 descriptor 与已审计原盘不符：{start:?} -> {end:?}"
        ));
    }
    Ok(())
}

fn parse_boundary(bytes: &[u8]) -> Result<DescriptorBoundary> {
    if bytes.len() != 3 || bytes[0] == 0 || bytes[1] == 0 {
        return Err(format!(
            "无效的 mode-2 descriptor 边界：{}",
            bytes_to_hex(bytes)
        ));
    }
    Ok(DescriptorBoundary {
        track_code: bytes[0],
        record: bytes[1],
        offset: bytes[2],
    })
}

fn boundary_position(boundary: DescriptorBoundary) -> BoundaryPosition {
    let cylinder = usize::from(boundary.track_code & 0x7F);
    let head = usize::from(boundary.track_code >> 7);
    BoundaryPosition {
        track_slot: cylinder * 2 + head,
        record: u16::from(boundary.record),
        offset: usize::from(boundary.offset),
    }
}

fn read_descriptor_range(
    image: &Image,
    source: &[u8],
    start: DescriptorBoundary,
    end: DescriptorBoundary,
) -> Result<(Vec<u8>, Vec<TextSourceSpan>)> {
    let start = boundary_position(start);
    let end = boundary_position(end);
    if (start.track_slot, start.record) > (end.track_slot, end.record) {
        return Err("文本 descriptor 的结束位置早于起始位置".into());
    }
    let disk = image
        .disks
        .first()
        .ok_or_else(|| "D88 中不存在 disk 0".to_owned())?;
    let mut sectors = disk
        .tracks
        .iter()
        .flat_map(|track| track.sectors.iter())
        .collect::<Vec<_>>();
    sectors.sort_by_key(|sector| {
        (
            sector.address.track_slot,
            sector.address.id.record,
            sector.address.physical_ordinal,
        )
    });
    for pair in sectors.windows(2) {
        if pair[0].address.track_slot == pair[1].address.track_slot
            && pair[0].address.id.record == pair[1].address.id.record
        {
            return Err(format!(
                "文本来源存在重复 CHR：C{}/H{} R{}",
                pair[0].address.id.cylinder, pair[0].address.id.head, pair[0].address.id.record
            ));
        }
    }
    let first = sectors
        .iter()
        .position(|sector| {
            sector.address.track_slot == start.track_slot
                && sector.address.id.record == start.record
        })
        .ok_or_else(|| {
            format!(
                "文本起始扇区 C{}/H{} R{} 不存在",
                start.track_slot / 2,
                start.track_slot % 2,
                start.record
            )
        })?;
    let last = sectors
        .iter()
        .position(|sector| {
            sector.address.track_slot == end.track_slot && sector.address.id.record == end.record
        })
        .ok_or_else(|| {
            format!(
                "文本结束扇区 C{}/H{} R{} 不存在",
                end.track_slot / 2,
                end.track_slot % 2,
                end.record
            )
        })?;
    if first > last {
        return Err("D88 CHR 顺序与文本 descriptor 顺序相反".into());
    }

    let mut bytes = Vec::new();
    let mut spans = Vec::new();
    for index in first..=last {
        let sector = sectors[index];
        if index > first {
            let previous = sectors[index - 1];
            if previous.address.track_slot == sector.address.track_slot {
                if sector.address.id.record != previous.address.id.record + 1 {
                    return Err(format!(
                        "文本来源扇区编号不连续：slot {} R{} 后不是下一扇区",
                        sector.address.track_slot, previous.address.id.record
                    ));
                }
            } else {
                let previous_track_end = sectors
                    .iter()
                    .filter(|candidate| candidate.address.track_slot == previous.address.track_slot)
                    .map(|candidate| candidate.address.id.record)
                    .max();
                let next_track_start = sectors
                    .iter()
                    .filter(|candidate| candidate.address.track_slot == sector.address.track_slot)
                    .map(|candidate| candidate.address.id.record)
                    .min();
                if sector.address.track_slot != previous.address.track_slot + 1
                    || previous_track_end != Some(previous.address.id.record)
                    || next_track_start != Some(sector.address.id.record)
                {
                    return Err(format!(
                        "文本来源跨轨道时 CHR 边界不连续：slot {} R{} -> slot {} R{}",
                        previous.address.track_slot,
                        previous.address.id.record,
                        sector.address.track_slot,
                        sector.address.id.record
                    ));
                }
            }
        }
        let payload = source
            .get(sector.data_range.clone())
            .ok_or_else(|| "文本来源扇区数据范围越界".to_owned())?;
        let begin = if index == first { start.offset } else { 0 };
        let finish = if index == last {
            end.offset
        } else {
            payload.len()
        };
        if begin > finish || finish > payload.len() {
            return Err(format!(
                "文本来源偏移越界：slot {} R{} payload={} range={begin}..{finish}",
                sector.address.track_slot,
                sector.address.id.record,
                payload.len()
            ));
        }
        if begin == finish {
            continue;
        }
        let resource_offset = bytes.len();
        bytes.extend_from_slice(&payload[begin..finish]);
        spans.push(TextSourceSpan {
            disk_index: 0,
            track_slot: sector.address.track_slot,
            physical_ordinal: sector.address.physical_ordinal,
            record: sector.address.id.record,
            source_data_offset: sector.data_range.start + begin,
            sector_data_offset: begin,
            resource_offset,
            length: finish - begin,
        });
    }
    Ok((bytes, spans))
}

fn read_single_byte_table(program: &[u8]) -> Result<Vec<u16>> {
    let address = usize::from(MAIN_PROGRAM_TABLE_ADDRESS - MAIN_PROGRAM_BASE);
    let start = address
        .checked_add(2)
        .ok_or_else(|| "单字节映射表地址溢出".to_owned())?;
    let end = start
        .checked_add(0xE0 * 2)
        .ok_or_else(|| "单字节映射表长度溢出".to_owned())?;
    let table = program
        .get(start..end)
        .ok_or_else(|| "主程序映像中的单字节映射表不完整".to_owned())?;
    Ok(table
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect())
}

fn single_byte_table_bytes(table: &[u16]) -> Vec<u8> {
    table.iter().flat_map(|value| value.to_le_bytes()).collect()
}

fn parse_records(bytes: &[u8], reader: TextReader, single_table: &[u16]) -> Result<ParsedRecords> {
    let mut records = Vec::new();
    let mut raw_bodies = Vec::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() && bytes[cursor] != 0 {
        let start = cursor;
        let tag = bytes[cursor];
        cursor += 1;
        let body_start = cursor;
        let terminator = loop {
            let token = *bytes
                .get(cursor)
                .ok_or_else(|| format!("文本 selector 中存在未终止记录 {}", records.len()))?;
            if token == 0 {
                break cursor;
            }
            if token >= 0xE0 {
                cursor = cursor
                    .checked_add(2)
                    .ok_or_else(|| "双字节 token 偏移溢出".to_owned())?;
                if cursor > bytes.len() {
                    return Err(format!(
                        "文本 selector 中记录 {} 的双字节 token 被截断",
                        records.len()
                    ));
                }
            } else {
                cursor += 1;
            }
        };
        let body = &bytes[body_start..terminator];
        let decoded = decode_body(reader, body, single_table).map_err(|error| {
            format!(
                "文本 selector 记录 {} (stream 0x{start:X}) 解码失败：{error}",
                records.len()
            )
        })?;
        let end = terminator + 1;
        records.push(TextPrimaryRecord {
            index: records.len(),
            stream_start: start,
            stream_end: end,
            tag,
            raw_record_hex: bytes_to_hex(&bytes[start..end]),
            raw_body_hex: bytes_to_hex(body),
            message: decoded,
        });
        raw_bodies.push(body.to_vec());
        cursor = end;
    }
    let padding = bytes[cursor..].to_vec();
    if padding.iter().any(|byte| *byte != 0) {
        return Err(format!(
            "文本 selector 字符串之后含非零数据，无法证明属于 padding：{}",
            bytes_to_hex(&padding)
        ));
    }
    Ok(ParsedRecords {
        records,
        raw_bodies,
        trailing_padding: padding,
    })
}

fn decode_body(reader: TextReader, body: &[u8], single_table: &[u16]) -> Result<String> {
    decode_body_with_carriers(
        reader,
        body,
        single_table,
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
}

fn decode_body_with_carriers(
    reader: TextReader,
    body: &[u8],
    single_table: &[u16],
    carrier_targets: &BTreeMap<u16, char>,
    compressed_targets: &BTreeMap<u8, char>,
) -> Result<String> {
    let mut output = String::new();
    let mut cursor = 0usize;
    while cursor < body.len() {
        let token = body[cursor];
        if token >= 0xE0 {
            let trail = *body
                .get(cursor + 1)
                .ok_or_else(|| format!("双字节 token 0x{token:02X} 缺少 trail"))?;
            let raw = u16::from_be_bytes([token, trail]);
            let decoded = pc88_double_to_jis(raw).and_then(|jis| {
                let cp932 = jis_to_sjis(jis)?;
                match carrier_targets.get(&cp932) {
                    Some(character) => Ok(*character),
                    None => jis_to_char(jis),
                }
            });
            match decoded {
                Ok(character) => output.push(character),
                Err(_) => output.push_str(&opaque_token_marker(raw)),
            }
            cursor += 2;
        } else {
            if reader.has_record_newlines() && token == 0x0D {
                output.push('\n');
            } else if let Some(target) = compressed_targets.get(&token) {
                output.push(*target);
            } else {
                let jis = *single_table
                    .get(usize::from(token))
                    .ok_or_else(|| format!("单字节 token 0x{token:02X} 不在主程序表中"))?;
                output.push(jis_to_char(normalize_jis(jis)?)?);
            }
            cursor += 1;
        }
    }
    Ok(output)
}

fn plan_single_byte_compression(
    resources: &[ResourceData],
    translations: &[Entry],
    uncompressed_sizes: &[usize],
    single_reverse: &BTreeMap<char, u8>,
    font_resources: &FontResources,
    font_plan: &vn_font::font_88::DynamicFontPlan,
) -> Result<Vec<TextCompressionMapping>> {
    if resources.len() != uncompressed_sizes.len() {
        return Err("压缩规划缺少文本资源长度".into());
    }

    let mut needed = resources
        .iter()
        .zip(uncompressed_sizes)
        .map(|(resource, size)| size.saturating_sub(resource.bytes.len()))
        .collect::<Vec<_>>();
    if needed.iter().all(|count| *count == 0) {
        return Ok(Vec::new());
    }
    let initial_needed = needed.clone();

    let mut counts_by_character = BTreeMap::<char, Vec<usize>>::new();
    let mut entry_index = 0usize;
    for (resource_index, resource) in resources.iter().enumerate() {
        for record in &resource.primary.records {
            let translation = translations
                .get(entry_index)
                .ok_or_else(|| "翻译记录数量在压缩规划时发生变化".to_owned())?;
            if translation.message != record.message {
                for character in translation.message.chars() {
                    if !('\u{3400}'..='\u{9FFF}').contains(&character)
                        || single_reverse.contains_key(&character)
                    {
                        continue;
                    }
                    let counts = counts_by_character
                        .entry(character)
                        .or_insert_with(|| vec![0; resources.len()]);
                    counts[resource_index] += 1;
                }
            }
            entry_index += 1;
        }
    }
    if entry_index != translations.len() {
        return Err("翻译 JSON 含有未映射到文本资源的记录".into());
    }

    let mut jis_by_character = BTreeMap::<char, u16>::new();
    for character in counts_by_character.keys() {
        let encoded = font_resources
            .encode_text(&character.to_string(), font_plan)
            .map_err(|error| {
                format!(
                    "U+{:04X} {character:?} 无法为单字节压缩取得字形载体：{error}",
                    *character as u32
                )
            })?;
        if encoded.bytes.len() != 2 {
            continue;
        }
        let cp932 = u16::from_be_bytes([encoded.bytes[0], encoded.bytes[1]]);
        let jis = font_88::sjis_to_jis(cp932)
            .ok_or_else(|| format!("CP932 {cp932:04X} 无法还原为单字节表的 JIS 值"))?;
        jis_by_character.insert(*character, jis);
    }

    let mut selected = Vec::<char>::new();
    while needed.iter().any(|count| *count > 0) {
        let mut best = None;
        let mut best_score = 0u64;
        for (character, counts) in &counts_by_character {
            if selected.contains(character) || !jis_by_character.contains_key(character) {
                continue;
            }
            let score = counts
                .iter()
                .enumerate()
                .map(|(index, count)| {
                    if initial_needed[index] == 0 {
                        0
                    } else {
                        ((*count).min(needed[index]) as u64 * 1_000_000)
                            / initial_needed[index] as u64
                    }
                })
                .sum::<u64>();
            if score > best_score {
                best = Some(*character);
                best_score = score;
            }
        }
        let character = best.ok_or_else(|| {
            let remaining = resources
                .iter()
                .zip(&needed)
                .filter(|(_, size)| **size > 0)
                .map(|(resource, size)| {
                    format!("selector {} 还差 {size} 字节", resource.primary.selector)
                })
                .collect::<Vec<_>>()
                .join("，");
            format!("单字节汉字压缩无法覆盖容量缺口：{remaining}")
        })?;
        selected.push(character);
        for (index, count) in counts_by_character[&character].iter().enumerate() {
            needed[index] = needed[index].saturating_sub(*count);
        }
        if selected.len() > SINGLE_BYTE_DONOR_TOKENS.len() {
            return Err("当前文本所需单字节汉字映射超过已审计的 donor 数量".into());
        }
    }

    Ok(selected
        .into_iter()
        .zip(SINGLE_BYTE_DONOR_TOKENS.iter().copied())
        .map(|(character, token)| {
            let occurrences_by_selector = resources
                .iter()
                .enumerate()
                .filter_map(|(index, resource)| {
                    let occurrences = counts_by_character[&character][index];
                    (occurrences > 0).then_some((resource.primary.selector, occurrences))
                })
                .collect::<Vec<_>>();
            let saved_bytes = occurrences_by_selector
                .iter()
                .map(|(_, occurrences)| occurrences)
                .sum();
            TextCompressionMapping {
                character,
                token,
                jis: jis_by_character[&character],
                occurrences_by_selector,
                saved_bytes,
            }
        })
        .collect())
}

fn collect_original_double_byte_codes(
    body: &[u8],
    reader: TextReader,
    single_table: &[u16],
    output: &mut BTreeSet<u16>,
) -> Result<()> {
    let mut cursor = 0usize;
    while cursor < body.len() {
        let token = body[cursor];
        if token >= 0xE0 {
            let trail = *body
                .get(cursor + 1)
                .ok_or_else(|| format!("原文双字节 token 0x{token:02X} 缺少 trail"))?;
            let raw = u16::from_be_bytes([token, trail]);
            if let Ok(jis) = pc88_double_to_jis(raw) {
                if let Ok(cp932) = jis_to_sjis(jis) {
                    output.insert(cp932);
                }
            }
            cursor += 2;
        } else {
            let _ = if reader.has_record_newlines() && token == 0x0D {
                None
            } else {
                let value = *single_table
                    .get(usize::from(token))
                    .ok_or_else(|| format!("单字节 token 0x{token:02X} 不在主程序表中"))?;
                Some(normalize_jis(value)?)
            };
            cursor += 1;
        }
    }
    Ok(())
}

fn build_single_reverse_map(single_table: &[u16]) -> Result<BTreeMap<char, u8>> {
    build_single_reverse_map_excluding(single_table, &BTreeSet::new())
}

fn build_single_reverse_map_excluding(
    single_table: &[u16],
    excluded_tokens: &BTreeSet<u8>,
) -> Result<BTreeMap<char, u8>> {
    let mut reverse = BTreeMap::new();
    for token in 1u16..=0xDF {
        if excluded_tokens.contains(&(token as u8)) {
            continue;
        }
        let jis = *single_table
            .get(usize::from(token))
            .ok_or_else(|| "主程序单字节映射表不完整".to_owned())?;
        let character = jis_to_char(normalize_jis(jis)?)?;
        reverse.entry(character).or_insert(token as u8);
    }
    Ok(reverse)
}

fn encode_message(
    reader: TextReader,
    message: &str,
    font_resources: &FontResources,
    font_plan: &vn_font::font_88::DynamicFontPlan,
    single_reverse: &BTreeMap<char, u8>,
    compressed_by_character: &BTreeMap<char, u8>,
) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(message.len());
    let mut cursor = 0usize;
    while cursor < message.len() {
        let rest = &message[cursor..];
        if let Some((raw, marker_len)) = parse_opaque_token_marker(rest) {
            output.extend_from_slice(&raw.to_be_bytes());
            cursor += marker_len;
            continue;
        }
        let character = rest
            .chars()
            .next()
            .ok_or_else(|| "译文 UTF-8 游标越界".to_owned())?;
        if character == '\0' {
            return Err("译文含 NUL 字符，不能写入 NUL 结尾记录".into());
        }
        if character == '\n' {
            if !reader.has_record_newlines() {
                return Err("C000 reader 将 0x0D 当作字形，不能把换行写入 selector 0".into());
            }
            output.push(0x0D);
            cursor += character.len_utf8();
            continue;
        }
        if character == '\r' {
            return Err("译文含 CR；请使用 LF 表示 D300/E900 文本换行".into());
        }
        if let Some(token) = compressed_by_character.get(&character) {
            output.push(*token);
            cursor += character.len_utf8();
            continue;
        }
        if let Some(token) = single_reverse.get(&character) {
            output.push(*token);
            cursor += character.len_utf8();
            continue;
        }
        let encoded = font_resources
            .encode_text(&character.to_string(), font_plan)
            .map_err(|error| {
                format!(
                    "U+{:04X} {character:?} 无法通过 PC-88 字库计划编码：{error}",
                    character as u32
                )
            })?;
        if encoded.bytes.len() != 2 {
            return Err(format!(
                "U+{:04X} {character:?} 没有游戏单字节映射，PC-88 字库也未生成双字节载体",
                character as u32
            ));
        }
        let cp932 = u16::from_be_bytes([encoded.bytes[0], encoded.bytes[1]]);
        let jis = font_88::sjis_to_jis(cp932)
            .ok_or_else(|| format!("CP932 {cp932:04X} 不能还原为标准 JIS row/cell"))?;
        let raw = jis_to_pc88_double(jis)?;
        output.extend_from_slice(&raw.to_be_bytes());
        cursor += character.len_utf8();
    }
    Ok(output)
}

fn opaque_token_marker(raw: u16) -> String {
    format!("⟦PC88:{raw:04X}⟧")
}

fn parse_opaque_token_marker(text: &str) -> Option<(u16, usize)> {
    const PREFIX: &str = "⟦PC88:";
    const CLOSE: &str = "⟧";
    let rest = text.strip_prefix(PREFIX)?;
    let close = rest.find(CLOSE)?;
    let digits = &rest[..close];
    if digits.len() != 4 {
        return None;
    }
    let raw = u16::from_str_radix(digits, 16).ok()?;
    if raw < 0xE000 || pc88_double_to_jis(raw).and_then(jis_to_char).is_ok() {
        return None;
    }
    Some((raw, PREFIX.len() + close + CLOSE.len()))
}

fn strip_opaque_token_markers(message: &str) -> String {
    let mut output = String::with_capacity(message.len());
    let mut cursor = 0usize;
    while cursor < message.len() {
        let rest = &message[cursor..];
        if let Some((_, marker_len)) = parse_opaque_token_marker(rest) {
            cursor += marker_len;
            continue;
        }
        let Some(character) = rest.chars().next() else {
            break;
        };
        output.push(character);
        cursor += character.len_utf8();
    }
    output
}

fn pc88_double_to_jis(raw: u16) -> Result<u16> {
    let jis = if raw >= 0xFF60 {
        raw.checked_sub(0xDE40)
    } else {
        raw.checked_sub(0xB000)
    }
    .ok_or_else(|| format!("双字节 token {raw:04X} 在 reader 偏置下溢"))?;
    normalize_jis(jis)
}

fn jis_to_pc88_double(jis: u16) -> Result<u16> {
    let row = jis >> 8;
    let cell = jis & 0xFF;
    if row == 0x21 && (0x21..=0x7E).contains(&cell) {
        return Ok(jis + 0xDE40);
    }
    if (0x30..=0x4F).contains(&row)
        && (0x21..=0x7E).contains(&cell)
        && (row != 0x4F || cell <= 0x53)
    {
        let raw = jis + 0xB000;
        if !(0xE000..0xFF60).contains(&raw) || pc88_double_to_jis(raw)? != jis {
            return Err(format!("JIS {jis:04X} 超出游戏双字节 token 映射"));
        }
        return Ok(raw);
    }
    Err(format!("JIS {jis:04X} 不在游戏可用双字节字形范围"))
}

fn normalize_jis(value: u16) -> Result<u16> {
    let row = (value >> 8) as u8;
    let cell = value as u8;
    let (row, cell) = if (0xA1..=0xFE).contains(&row) && (0xA1..=0xFE).contains(&cell) {
        (row & 0x7F, cell & 0x7F)
    } else {
        (row, cell)
    };
    if !(0x21..=0x7E).contains(&row) || !(0x21..=0x7E).contains(&cell) {
        return Err(format!("映射表值 {value:04X} 不是有效 JIS row/cell"));
    }
    Ok((u16::from(row) << 8) | u16::from(cell))
}

fn jis_to_sjis(jis: u16) -> Result<u16> {
    let row = u8::try_from(jis >> 8).map_err(|_| format!("JIS {jis:04X} row 无效"))?;
    let cell = u8::try_from(jis & 0xFF).map_err(|_| format!("JIS {jis:04X} cell 无效"))?;
    if !(0x21..=0x7E).contains(&row) || !(0x21..=0x7E).contains(&cell) {
        return Err(format!("JIS {jis:04X} row/cell 越界"));
    }
    let row_index = u16::from(row - 0x21);
    let mut lead = (row_index >> 1) + 0x81;
    if lead > 0x9F {
        lead += 0x40;
    }
    let trail = if row_index & 1 == 0 {
        let trail = u16::from(cell) + 0x1F;
        if trail >= 0x7F {
            trail + 1
        } else {
            trail
        }
    } else {
        u16::from(cell) + 0x7E
    };
    let sjis = (lead << 8) | trail;
    if font_88::sjis_to_jis(sjis) != Some(jis) {
        return Err(format!("JIS {jis:04X} 无法往返转换为 CP932"));
    }
    Ok(sjis)
}

fn jis_to_char(jis: u16) -> Result<char> {
    let sjis = jis_to_sjis(jis)?;
    let bytes = sjis.to_be_bytes();
    let (decoded, _, had_errors) = SHIFT_JIS.decode(&bytes);
    if had_errors || decoded.chars().count() != 1 {
        return Err(format!(
            "JIS {jis:04X}/CP932 {sjis:04X} 无法映射为单个 Unicode 字符"
        ));
    }
    decoded
        .chars()
        .next()
        .ok_or_else(|| format!("JIS {jis:04X} 解码结果为空"))
}

fn find_sector_by_record(
    image: &Image,
    disk_index: usize,
    track_slot: usize,
    record: u16,
) -> Result<&Sector> {
    let disk = image
        .disks
        .get(disk_index)
        .ok_or_else(|| format!("D88 disk {disk_index} 不存在"))?;
    let track = disk
        .tracks
        .iter()
        .find(|track| track.slot == track_slot)
        .ok_or_else(|| format!("D88 disk {disk_index} track-slot {track_slot} 不存在"))?;
    let mut matches = track
        .sectors
        .iter()
        .filter(|sector| sector.address.id.record == record);
    let sector = matches
        .next()
        .ok_or_else(|| format!("D88 disk {disk_index} slot {track_slot} R{record} 不存在"))?;
    if matches.next().is_some() {
        return Err(format!(
            "D88 disk {disk_index} slot {track_slot} R{record} 不唯一"
        ));
    }
    Ok(sector)
}

fn find_sector_by_ordinal(
    image: &Image,
    disk_index: usize,
    track_slot: usize,
    physical_ordinal: usize,
) -> Result<&Sector> {
    let disk = image
        .disks
        .get(disk_index)
        .ok_or_else(|| format!("D88 disk {disk_index} 不存在"))?;
    let track = disk
        .tracks
        .iter()
        .find(|track| track.slot == track_slot)
        .ok_or_else(|| format!("D88 disk {disk_index} track-slot {track_slot} 不存在"))?;
    track
        .sectors
        .iter()
        .find(|sector| sector.address.physical_ordinal == physical_ordinal)
        .ok_or_else(|| {
            format!(
                "D88 disk {disk_index} slot {track_slot} physical ordinal {physical_ordinal} 不存在"
            )
        })
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02X}");
    }
    output
}
