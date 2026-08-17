use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

pub const JSON_FORMAT: &str = "aitsuno-game-text-v3";
pub const HEADER_SIZE: usize = 44;
pub const VISIBLE_LINE_BYTE_LIMIT: usize = 62;
pub const GEN_LINE_BYTE_LIMIT: usize = 76;
const SOURCE_KIND_AREA: &str = "area";
const SOURCE_KIND_GEN: &str = "gen";
const SOURCE_KIND_MES: &str = "mes";
const SOURCE_KIND_MAIN: &str = "main";
const SOURCE_KIND_OPENING: &str = "opening";
const SOURCE_KIND_INTER1: &str = "inter1";
const SOURCE_KIND_INTER2: &str = "inter2";
const SOURCE_KIND_BUNSYO: &str = "bunsyo";
const SOURCE_KIND_ENDING: &str = "ending";
const SOURCE_KIND_HISAUCHI: &str = "hisauchi";
const INTER_LINE_BYTE_LIMIT: usize = 58;
const BUNSYO_LINE_BYTE_LIMIT: usize = 30;
const BUNSYO_RECORD_COUNT: usize = 95;
const BUNSYO_RECORD_SIZE: usize = 0x124;
const BUNSYO_DISPLAY_LINES: usize = 9;
const INTER1_PACKED_POOL_START: usize = 0x0036;
const INTER1_PACKED_POOL_END: usize = 0x035D;
const INTER1_POINTER_TABLE: usize = 0x05FE;
const INTER1_DATA_SEGMENT: u16 = 0x06D6;
const INTER2_DATA_SEGMENT: u16 = 0x05F4;
const GEN_MZ_HEADER_PARAGRAPHS: usize = 0xA0;
const GEN_DATA_SEGMENT: u16 = 0x0596;
const GEN_TEXT_START: usize = 0x0036;
const GEN_TEXT_POOL_END: usize = 0x0C0B;
const GEN_GROUP_POINTERS_OFFSET: usize = 0x0C18;
const GEN_GROUP_TABLE_OFFSET: usize = 0x0D2C;
const GEN_GROUP_COUNT: usize = 16;
const GEN_GROUP_STARTS: [usize; GEN_GROUP_COUNT] = [
    0x0C18, 0x0C28, 0x0C3C, 0x0C50, 0x0C5C, 0x0C6C, 0x0C78, 0x0CBC, 0x0CC8, 0x0CD0, 0x0CD8, 0x0CEC,
    0x0D00, 0x0D0C, 0x0D14, 0x0D24,
];
const GEN_FIXED_SPECS: [FixedStringSpec; 7] = [
    FixedStringSpec {
        data_offset: 0x0DDA,
        capacity: 20,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0DEF,
        capacity: 32,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0E10,
        capacity: 32,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0E40,
        capacity: 20,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0E55,
        capacity: 4,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0E62,
        capacity: 4,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0E68,
        capacity: 46,
        text_type: "system",
    },
];
pub const SECTION_WIDTHS: [usize; 22] = [
    2, 2, 2, 2, 4, 2, 4, 1, 2, 4, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1,
];
const AREA_CHOICE_TABLE_SECTIONS: [(usize, usize); 5] =
    [(14, 15), (10, 11), (12, 13), (16, 17), (18, 19)];

type Result<T> = std::result::Result<T, String>;
type TextSource = (PathBuf, String, Vec<u8>, ParsedSource);

#[derive(Debug, Clone)]
enum ParsedSource {
    Area(AreaFile),
    Gen(GenFile),
    Mes(MesFile),
    FixedExe(FixedExeFile),
    Inter(InterFile),
    Bunsyo(BunsyoFile),
}

#[derive(Debug, Clone)]
pub struct AreaFile {
    counts: [u16; 22],
    sections: Vec<Vec<u8>>,
    choice_tables: Vec<ChoiceTable>,
    slots: Vec<TextSlot>,
    text_pool_file_offset: usize,
}

#[derive(Debug, Clone)]
struct ChoiceTable {
    offset_section: usize,
    pool_section: usize,
    choices: Vec<ChoiceSlot>,
    pool_file_offset: usize,
}

#[derive(Debug, Clone)]
struct ChoiceSlot {
    text: String,
    pool_offset: usize,
    export: bool,
}

#[derive(Debug, Clone)]
struct TextSlot {
    text: String,
    suffix: Vec<u8>,
    pool_offset: usize,
}

#[derive(Debug, Clone)]
pub struct GenFile {
    data_file_offset: usize,
    groups: Vec<GenGroup>,
    fixed_strings: Vec<FixedString>,
}

#[derive(Debug, Clone)]
struct GenGroup {
    chunks: Vec<GenChunk>,
}

#[derive(Debug, Clone)]
struct GenChunk {
    data_offset: usize,
    capacity: usize,
    text: String,
}

#[derive(Debug, Clone)]
struct MesFile {
    texts: Vec<String>,
}

#[derive(Debug, Clone)]
struct FixedExeFile {
    source_kind: &'static str,
    data_file_offset: usize,
    strings: Vec<FixedString>,
}

#[derive(Debug, Clone)]
struct InterFile {
    source_kind: &'static str,
    data_file_offset: usize,
    groups: Vec<InterGroup>,
    fixed_strings: Vec<FixedString>,
    physical_lines: usize,
    packed_pool: Option<InterPackedPool>,
}

#[derive(Debug, Clone)]
struct InterGroup {
    slots: Vec<FixedString>,
    text_type: &'static str,
    style: InterGroupStyle,
}

#[derive(Debug, Clone)]
struct InterPackedPool {
    start_offset: usize,
    end_offset: usize,
    pointer_table_offset: usize,
    segment: u16,
    group_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InterGroupStyle {
    Instruction,
    IndentedParagraph,
    Plain,
    Title,
    Bullet,
    Arrow,
    Level(&'static str),
}

#[derive(Debug, Clone)]
struct BunsyoFile {
    records: Vec<BunsyoRecord>,
}

#[derive(Debug, Clone)]
struct BunsyoRecord {
    offset: usize,
    prompt_offset: usize,
    choice_offsets: [usize; 2],
    prompt: String,
    choices: [String; 2],
}

#[derive(Debug, Clone)]
struct FixedString {
    data_offset: usize,
    capacity: usize,
    text: String,
    text_type: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct FixedStringSpec {
    data_offset: usize,
    capacity: usize,
    text_type: &'static str,
}

const MAIN_COMMAND_OFFSETS: [usize; 23] = [
    0x00FC, 0x0107, 0x0112, 0x011D, 0x0128, 0x015C, 0x0167, 0x0172, 0x017D, 0x0188, 0x01BC, 0x01C7,
    0x01D2, 0x01DD, 0x01E8, 0x01F3, 0x022E, 0x0239, 0x0244, 0x024F, 0x025A, 0x0265, 0x0270,
];

const MAIN_OTHER_SPECS: [FixedStringSpec; 29] = [
    FixedStringSpec {
        data_offset: 0x045E,
        capacity: 32,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x047F,
        capacity: 6,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x0486,
        capacity: 6,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x048D,
        capacity: 34,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x04B0,
        capacity: 12,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x04BD,
        capacity: 12,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x04CA,
        capacity: 34,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x04ED,
        capacity: 16,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x04FE,
        capacity: 6,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x0505,
        capacity: 6,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x0569,
        capacity: 22,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x0580,
        capacity: 20,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x0595,
        capacity: 6,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x059C,
        capacity: 6,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x05D8,
        capacity: 6,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x05DF,
        capacity: 6,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x0BD6,
        capacity: 20,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0BEB,
        capacity: 32,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0C0C,
        capacity: 32,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0C3C,
        capacity: 20,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0C51,
        capacity: 4,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0CF6,
        capacity: 4,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0CFC,
        capacity: 46,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0E12,
        capacity: 50,
        text_type: "dialogue",
    },
    FixedStringSpec {
        data_offset: 0x0E45,
        capacity: 20,
        text_type: "dialogue",
    },
    FixedStringSpec {
        data_offset: 0x0E5A,
        capacity: 40,
        text_type: "dialogue",
    },
    FixedStringSpec {
        data_offset: 0x0E83,
        capacity: 34,
        text_type: "dialogue",
    },
    FixedStringSpec {
        data_offset: 0x0EA6,
        capacity: 42,
        text_type: "dialogue",
    },
    FixedStringSpec {
        data_offset: 0x0F38,
        capacity: 37,
        text_type: "dialogue",
    },
];

const OPENING_SPECS: [FixedStringSpec; 18] = [
    FixedStringSpec {
        data_offset: 0x0036,
        capacity: 24,
        text_type: "opening_bio",
    },
    FixedStringSpec {
        data_offset: 0x0050,
        capacity: 22,
        text_type: "opening_bio",
    },
    FixedStringSpec {
        data_offset: 0x0068,
        capacity: 12,
        text_type: "opening_bio",
    },
    FixedStringSpec {
        data_offset: 0x0075,
        capacity: 22,
        text_type: "opening_bio",
    },
    FixedStringSpec {
        data_offset: 0x008C,
        capacity: 18,
        text_type: "opening_bio",
    },
    FixedStringSpec {
        data_offset: 0x00A0,
        capacity: 26,
        text_type: "opening_bio",
    },
    FixedStringSpec {
        data_offset: 0x00BB,
        capacity: 18,
        text_type: "opening_bio",
    },
    FixedStringSpec {
        data_offset: 0x00CE,
        capacity: 20,
        text_type: "opening_bio",
    },
    FixedStringSpec {
        data_offset: 0x00E3,
        capacity: 28,
        text_type: "opening_bio",
    },
    FixedStringSpec {
        data_offset: 0x0100,
        capacity: 28,
        text_type: "opening_bio",
    },
    FixedStringSpec {
        data_offset: 0x011D,
        capacity: 24,
        text_type: "opening_bio",
    },
    FixedStringSpec {
        data_offset: 0x0136,
        capacity: 6,
        text_type: "opening_bio",
    },
    FixedStringSpec {
        data_offset: 0x013D,
        capacity: 18,
        text_type: "opening_bio",
    },
    FixedStringSpec {
        data_offset: 0x03DE,
        capacity: 20,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x03F3,
        capacity: 32,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0414,
        capacity: 32,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0444,
        capacity: 20,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0459,
        capacity: 4,
        text_type: "system",
    },
];

const HISAUCHI_SPECS: [FixedStringSpec; 5] = [
    FixedStringSpec {
        data_offset: 0x00D2,
        capacity: 20,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x00E7,
        capacity: 32,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0108,
        capacity: 32,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0138,
        capacity: 20,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x014D,
        capacity: 4,
        text_type: "system",
    },
];

const ENDING_SPECS: [FixedStringSpec; 5] = [
    FixedStringSpec {
        data_offset: 0x0234,
        capacity: 20,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0249,
        capacity: 32,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x026A,
        capacity: 32,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x029A,
        capacity: 20,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x02AF,
        capacity: 4,
        text_type: "system",
    },
];

const INTER1_UI_SPECS: [FixedStringSpec; 13] = [
    FixedStringSpec {
        data_offset: 0x06D6,
        capacity: 16,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x06E7,
        capacity: 10,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x06F2,
        capacity: 20,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x070A,
        capacity: 10,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x072B,
        capacity: 20,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x0740,
        capacity: 16,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x075E,
        capacity: 24,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x0777,
        capacity: 22,
        text_type: "choice",
    },
    FixedStringSpec {
        data_offset: 0x07AE,
        capacity: 20,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x07C3,
        capacity: 32,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x07E4,
        capacity: 32,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0814,
        capacity: 20,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x0829,
        capacity: 4,
        text_type: "system",
    },
];

const INTER2_UI_SPECS: [FixedStringSpec; 12] = [
    FixedStringSpec {
        data_offset: 0x2012,
        capacity: 50,
        text_type: "title",
    },
    FixedStringSpec {
        data_offset: 0x20AF,
        capacity: 20,
        text_type: "result",
    },
    FixedStringSpec {
        data_offset: 0x20C4,
        capacity: 22,
        text_type: "result",
    },
    FixedStringSpec {
        data_offset: 0x20DB,
        capacity: 34,
        text_type: "result",
    },
    FixedStringSpec {
        data_offset: 0x20FE,
        capacity: 26,
        text_type: "result",
    },
    FixedStringSpec {
        data_offset: 0x2119,
        capacity: 10,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x2124,
        capacity: 4,
        text_type: "ui",
    },
    FixedStringSpec {
        data_offset: 0x2164,
        capacity: 20,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x2179,
        capacity: 32,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x219A,
        capacity: 32,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x21CA,
        capacity: 20,
        text_type: "system",
    },
    FixedStringSpec {
        data_offset: 0x21DF,
        capacity: 4,
        text_type: "system",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PartKind {
    Dialogue,
    Monologue,
    Narration,
}

impl PartKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Dialogue => "dialogue",
            Self::Monologue => "monologue",
            Self::Narration => "narration",
        }
    }
}

#[derive(Debug, Clone)]
struct PartSpan {
    start: usize,
    end: usize,
    kind: PartKind,
}

#[derive(Debug, Clone)]
struct LogicalParts {
    spans: Vec<PartSpan>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationDocument {
    pub _format: String,
    pub _source_kind: String,
    pub _source_file: String,
    pub _source_sha256: String,
    pub _line_visible_byte_limit: usize,
    pub entries: Vec<TranslationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationEntry {
    pub _index: usize,
    pub _file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _slot_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _part_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _choice_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _gen_group_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _group_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _capacity: Option<usize>,
    pub _offset: usize,
    pub _size: usize,
    pub _type: String,
    pub _encoding: String,
    pub _policy: String,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ExtractReport {
    pub scanned_files: usize,
    pub skipped_files: usize,
    pub json_files: usize,
    pub source_messages: usize,
    pub source_choices: usize,
    pub extracted_entries: usize,
    pub physical_lines: usize,
    pub max_visible_line_bytes: usize,
    pub over_limit: usize,
    pub warnings: Vec<String>,
    pub output_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct InjectReport {
    pub json_files: usize,
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub rebuilt_files: usize,
    pub output: PathBuf,
}

#[derive(Debug)]
struct PreparedExtraction {
    relative: String,
    output_name: String,
    document: TranslationDocument,
}

#[derive(Debug)]
struct PreparedInjection {
    relative: String,
    rebuilt: Vec<u8>,
    patched: usize,
    unchanged: usize,
}

pub fn parse_area(bytes: &[u8]) -> Result<AreaFile> {
    if bytes.len() < HEADER_SIZE {
        return Err("AREA header is truncated".to_string());
    }
    let mut counts = [0u16; 22];
    for (index, count) in counts.iter_mut().enumerate() {
        *count = read_u16(bytes, index * 2)?;
    }

    let mut cursor = HEADER_SIZE;
    let mut sections = Vec::with_capacity(22);
    let mut section_offsets = [0usize; 22];
    for index in 0..22 {
        section_offsets[index] = cursor;
        let size = usize::from(counts[index])
            .checked_mul(SECTION_WIDTHS[index])
            .ok_or_else(|| format!("AREA section {index} size overflow"))?;
        let end = cursor
            .checked_add(size)
            .ok_or_else(|| format!("AREA section {index} range overflow"))?;
        let section = bytes
            .get(cursor..end)
            .ok_or_else(|| format!("AREA section {index} is truncated"))?;
        sections.push(section.to_vec());
        cursor = end;
    }
    if cursor != bytes.len() {
        return Err(format!(
            "AREA section lengths end at 0x{cursor:X}, file ends at 0x{:X}",
            bytes.len()
        ));
    }
    if counts[20] == 0 || counts[21] == 0 {
        return Err("AREA message offset table or text pool is empty".to_string());
    }
    let mut choice_tables = Vec::with_capacity(AREA_CHOICE_TABLE_SECTIONS.len());
    for &(offset_section, pool_section) in &AREA_CHOICE_TABLE_SECTIONS {
        if counts[offset_section] == 0 {
            if counts[pool_section] != 0 {
                return Err(format!(
                    "AREA choice sections {offset_section}/{pool_section} have a pool without an offset table"
                ));
            }
            continue;
        }
        if SECTION_WIDTHS[offset_section] != 2 || SECTION_WIDTHS[pool_section] != 1 {
            return Err(format!(
                "AREA choice sections {offset_section}/{pool_section} have unexpected element widths"
            ));
        }

        let choice_offset_table = &sections[offset_section];
        let choice_pool = &sections[pool_section];
        let mut choice_offsets = Vec::with_capacity(usize::from(counts[offset_section]));
        for index in 0..usize::from(counts[offset_section]) {
            choice_offsets.push(usize::from(read_u16(choice_offset_table, index * 2)?));
        }
        if choice_offsets.first() != Some(&0) {
            return Err(format!(
                "AREA choice sections {offset_section}/{pool_section} first offset is 0x{:X}, expected zero",
                choice_offsets[0]
            ));
        }
        for pair in choice_offsets.windows(2) {
            if pair[0] > pair[1] {
                return Err(format!(
                    "AREA choice sections {offset_section}/{pool_section} offsets decrease: 0x{:X}, 0x{:X}",
                    pair[0], pair[1]
                ));
            }
        }
        if choice_pool.is_empty() {
            if choice_offsets.iter().any(|&offset| offset != 0) {
                return Err(format!(
                    "AREA choice sections {offset_section}/{pool_section} have nonzero offsets into an empty pool"
                ));
            }
            choice_tables.push(ChoiceTable {
                offset_section,
                pool_section,
                choices: choice_offsets
                    .into_iter()
                    .map(|pool_offset| ChoiceSlot {
                        text: String::new(),
                        pool_offset,
                        export: false,
                    })
                    .collect(),
                pool_file_offset: section_offsets[pool_section],
            });
            continue;
        }
        if choice_offsets.last().copied().unwrap_or(0) >= choice_pool.len() {
            return Err(format!(
                "AREA choice sections {offset_section}/{pool_section} offset points outside the string pool"
            ));
        }

        let unique_offsets: Vec<usize> = choice_offsets
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(index, offset)| {
                (index == 0 || choice_offsets[index - 1] != offset).then_some(offset)
            })
            .collect();
        let mut decoded_by_offset = HashMap::new();
        for (index, &start) in unique_offsets.iter().enumerate() {
            let end = unique_offsets
                .get(index + 1)
                .copied()
                .unwrap_or(choice_pool.len());
            let bytes = choice_pool
                .get(start..end)
                .ok_or_else(|| {
                    format!(
                        "AREA choice sections {offset_section}/{pool_section} string is outside the pool"
                    )
                })?;
            if bytes.last() != Some(&0) || bytes[..bytes.len() - 1].contains(&0) {
                return Err(format!(
                    "AREA choice sections {offset_section}/{pool_section} string at pool offset 0x{start:X} is not exactly one NUL-terminated string"
                ));
            }
            let text = decode_cp932(&bytes[..bytes.len() - 1]).map_err(|error| {
                format!(
                    "AREA choice sections {offset_section}/{pool_section} at pool offset 0x{start:X}: {error}"
                )
            })?;
            decoded_by_offset.insert(start, text);
        }
        let mut choices = Vec::with_capacity(choice_offsets.len());
        for offset in choice_offsets {
            choices.push(ChoiceSlot {
                text: decoded_by_offset
                    .get(&offset)
                    .expect("decoded choice offset")
                    .clone(),
                pool_offset: offset,
                export: pool_section != 19
                    || !decoded_by_offset
                        .get(&offset)
                        .expect("decoded choice offset")
                        .is_empty(),
            });
        }
        choice_tables.push(ChoiceTable {
            offset_section,
            pool_section,
            choices,
            pool_file_offset: section_offsets[pool_section],
        });
    }

    let offset_table = &sections[20];
    let pool = &sections[21];
    let mut offsets = Vec::with_capacity(usize::from(counts[20]));
    for index in 0..usize::from(counts[20]) {
        offsets.push(usize::from(read_u16(offset_table, index * 2)?));
    }
    if offsets.first() != Some(&0) {
        return Err(format!(
            "AREA first message offset is 0x{:X}, expected zero",
            offsets[0]
        ));
    }
    for pair in offsets.windows(2) {
        if pair[0] >= pair[1] {
            return Err(format!(
                "AREA message offsets are not strictly increasing: 0x{:X}, 0x{:X}",
                pair[0], pair[1]
            ));
        }
    }
    if offsets.last().copied().unwrap_or(0) >= pool.len() {
        return Err("AREA message offset points outside the text pool".to_string());
    }

    let mut slots = Vec::with_capacity(offsets.len());
    for (index, &start) in offsets.iter().enumerate() {
        let end = offsets.get(index + 1).copied().unwrap_or(pool.len());
        let slot = pool
            .get(start..end)
            .ok_or_else(|| format!("AREA message slot {index} is outside the text pool"))?;
        let nul = slot
            .iter()
            .position(|&byte| byte == 0)
            .ok_or_else(|| format!("AREA message slot {index} has no NUL terminator"))?;
        let text = decode_cp932(&slot[..nul])
            .map_err(|error| format!("AREA message slot {index}: {error}"))?;
        slots.push(TextSlot {
            text,
            suffix: slot[nul + 1..].to_vec(),
            pool_offset: start,
        });
    }

    Ok(AreaFile {
        counts,
        sections,
        choice_tables,
        slots,
        text_pool_file_offset: section_offsets[21],
    })
}

pub fn parse_gen(bytes: &[u8]) -> Result<GenFile> {
    if bytes.get(..2) != Some(b"MZ") {
        return Err("GEN executable has no MZ header".to_string());
    }
    let header_paragraphs = usize::from(read_u16(bytes, 0x08)?);
    if header_paragraphs != GEN_MZ_HEADER_PARAGRAPHS {
        return Err(format!(
            "GEN MZ header has {header_paragraphs} paragraphs, expected {GEN_MZ_HEADER_PARAGRAPHS}"
        ));
    }
    let data_file_offset = header_paragraphs
        .checked_mul(16)
        .and_then(|offset| offset.checked_add(usize::from(GEN_DATA_SEGMENT) * 16))
        .ok_or_else(|| "GEN data segment file offset overflow".to_string())?;
    let required = data_file_offset
        .checked_add(GEN_GROUP_TABLE_OFFSET + GEN_GROUP_COUNT * 4)
        .ok_or_else(|| "GEN data range overflow".to_string())?;
    if bytes.len() < required {
        return Err("GEN executable is truncated before its text tables".to_string());
    }

    for (index, &expected_offset) in GEN_GROUP_STARTS.iter().enumerate() {
        let pointer = data_file_offset + GEN_GROUP_TABLE_OFFSET + index * 4;
        let offset = usize::from(read_u16(bytes, pointer)?);
        let segment = read_u16(bytes, pointer + 2)?;
        if offset != expected_offset || segment != GEN_DATA_SEGMENT {
            return Err(format!(
                "GEN group table index {index} is {segment:04X}:{offset:04X}, expected {:04X}:{expected_offset:04X}",
                GEN_DATA_SEGMENT
            ));
        }
    }

    let mut group_text_offsets = Vec::with_capacity(GEN_GROUP_COUNT);
    let mut seen_text_offsets = HashSet::new();
    for (group_index, &group_start) in GEN_GROUP_STARTS.iter().enumerate() {
        let group_end = GEN_GROUP_STARTS
            .get(group_index + 1)
            .copied()
            .unwrap_or(GEN_GROUP_TABLE_OFFSET);
        let mut cursor = group_start;
        let mut text_offsets = Vec::new();
        loop {
            if cursor + 4 > group_end {
                return Err(format!(
                    "GEN text group {group_index} has no pointer-list terminator"
                ));
            }
            let pointer = data_file_offset + cursor;
            let text_offset = usize::from(read_u16(bytes, pointer)?);
            let segment = read_u16(bytes, pointer + 2)?;
            cursor += 4;
            if text_offset == 0 && segment == 0 {
                break;
            }
            if segment != GEN_DATA_SEGMENT {
                return Err(format!(
                    "GEN text group {group_index} contains pointer {segment:04X}:{text_offset:04X}"
                ));
            }
            if !(GEN_TEXT_START..GEN_GROUP_POINTERS_OFFSET).contains(&text_offset) {
                return Err(format!(
                    "GEN text group {group_index} points outside the text pool: 0x{text_offset:04X}"
                ));
            }
            if !seen_text_offsets.insert(text_offset) {
                return Err(format!(
                    "GEN text offset 0x{text_offset:04X} is referenced more than once"
                ));
            }
            text_offsets.push(text_offset);
        }
        if cursor != group_end {
            return Err(format!(
                "GEN text group {group_index} ends at 0x{cursor:04X}, expected 0x{group_end:04X}"
            ));
        }
        if text_offsets.is_empty() {
            return Err(format!("GEN text group {group_index} is empty"));
        }
        group_text_offsets.push(text_offsets);
    }

    let all_text_offsets: Vec<usize> = group_text_offsets.iter().flatten().copied().collect();
    if all_text_offsets.first() != Some(&GEN_TEXT_START) {
        return Err("GEN text pool does not start at data offset 0x0036".to_string());
    }
    if all_text_offsets.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err("GEN text pointers are not strictly increasing".to_string());
    }

    let mut decoded_chunks = HashMap::new();
    for (index, &text_offset) in all_text_offsets.iter().enumerate() {
        let next_offset = all_text_offsets
            .get(index + 1)
            .copied()
            .unwrap_or(GEN_TEXT_POOL_END);
        if next_offset <= text_offset + 1 {
            return Err(format!(
                "GEN text slot at data offset 0x{text_offset:04X} has no capacity"
            ));
        }
        let capacity = next_offset - text_offset - 1;
        let start = data_file_offset + text_offset;
        let slot = &bytes[start..start + capacity + 1];
        let nul = slot.iter().position(|&byte| byte == 0).ok_or_else(|| {
            format!("GEN text at data offset 0x{text_offset:04X} has no NUL terminator")
        })?;
        if slot[nul + 1..].iter().any(|&byte| byte != 0) {
            return Err(format!(
                "GEN text at data offset 0x{text_offset:04X} has nonzero padding"
            ));
        }
        let text = decode_cp932(&slot[..nul])
            .map_err(|error| format!("GEN text at data offset 0x{text_offset:04X}: {error}"))?;
        decoded_chunks.insert(
            text_offset,
            GenChunk {
                data_offset: text_offset,
                capacity,
                text,
            },
        );
    }

    let groups = group_text_offsets
        .into_iter()
        .map(|offsets| GenGroup {
            chunks: offsets
                .into_iter()
                .map(|offset| decoded_chunks.remove(&offset).expect("decoded GEN chunk"))
                .collect(),
        })
        .collect();

    let fixed_strings = parse_fixed_strings(bytes, data_file_offset, &GEN_FIXED_SPECS)?;
    Ok(GenFile {
        data_file_offset,
        groups,
        fixed_strings,
    })
}

impl GenFile {
    fn physical_line_stats(&self) -> Result<(usize, usize, usize)> {
        let mut lines = 0usize;
        let mut maximum = 0usize;
        let mut over_limit = 0usize;
        for chunk in self.groups.iter().flat_map(|group| &group.chunks) {
            let bytes = visible_cp932_len(&chunk.text)?;
            lines += 1;
            maximum = maximum.max(bytes);
            if bytes > GEN_LINE_BYTE_LIMIT {
                over_limit += 1;
            }
        }
        for string in &self.fixed_strings {
            let bytes = visible_cp932_len(&string.text)?;
            lines += 1;
            maximum = maximum.max(bytes);
            if bytes > GEN_LINE_BYTE_LIMIT {
                over_limit += 1;
            }
        }
        Ok((lines, maximum, over_limit))
    }

    fn translation_entries(&self, source_file: &str) -> Result<Vec<TranslationEntry>> {
        let mut entries = Vec::with_capacity(self.groups.len());
        for (group_index, group) in self.groups.iter().enumerate() {
            let scr_msg = join_gen_group(group);
            let first = group.chunks.first().expect("validated GEN group");
            let source_size = group.chunks.iter().map(|chunk| chunk.capacity).sum();
            entries.push(TranslationEntry {
                _index: group_index,
                _file: source_file.to_string(),
                _slot_index: None,
                _part_index: None,
                _choice_index: None,
                _gen_group_index: Some(group_index),
                _group_index: None,
                _capacity: Some(source_size),
                _offset: self.data_file_offset + first.data_offset,
                _size: source_size,
                _type: "gen_intro".to_string(),
                _encoding: "cp932".to_string(),
                _policy: "in_place_group".to_string(),
                scr_msg: scr_msg.clone(),
                message: scr_msg,
            });
        }
        for (fixed_index, string) in self.fixed_strings.iter().enumerate() {
            entries.push(TranslationEntry {
                _index: entries.len(),
                _file: source_file.to_string(),
                _slot_index: Some(fixed_index),
                _part_index: None,
                _choice_index: None,
                _gen_group_index: None,
                _group_index: None,
                _capacity: Some(string.capacity),
                _offset: self.data_file_offset + string.data_offset,
                _size: encode_cp932(&string.text)?.len(),
                _type: string.text_type.to_string(),
                _encoding: "cp932".to_string(),
                _policy: "in_place".to_string(),
                scr_msg: string.text.clone(),
                message: string.text.clone(),
            });
        }
        Ok(entries)
    }

    fn rebuild_with_document(
        &self,
        source: &[u8],
        document: &TranslationDocument,
    ) -> Result<(Vec<u8>, usize, usize)> {
        validate_document_header(document)?;
        let expected_entries = self.translation_entries(&document._source_file)?;
        if document.entries.len() != expected_entries.len() {
            return Err(format!(
                "JSON entry count {} does not match GEN group count {}",
                document.entries.len(),
                expected_entries.len()
            ));
        }

        let mut rebuilt = source.to_vec();
        let mut patched = 0usize;
        let mut unchanged = 0usize;
        let mut seen_indices = HashSet::new();
        for (expected, entry) in expected_entries.iter().zip(&document.entries) {
            if !seen_indices.insert(entry._index) {
                return Err(format!("duplicate JSON _index {}", entry._index));
            }
            if entry._index != expected._index
                || entry._file != expected._file
                || entry._slot_index != expected._slot_index
                || entry._part_index != expected._part_index
                || entry._choice_index != expected._choice_index
                || entry._gen_group_index != expected._gen_group_index
                || entry._group_index != expected._group_index
                || entry._capacity != expected._capacity
                || entry._offset != expected._offset
                || entry._size != expected._size
                || entry._type != expected._type
                || entry._encoding != "cp932"
                || entry._policy != expected._policy
            {
                return Err(format!(
                    "JSON metadata mismatch at GEN _index {}",
                    expected._index
                ));
            }
            if entry.scr_msg != expected.scr_msg {
                return Err(format!(
                    "scr_msg mismatch at GEN _index {}",
                    expected._index
                ));
            }
            if entry.message == entry.scr_msg {
                unchanged += 1;
                continue;
            }
            validate_message(&entry.message)
                .map_err(|error| format!("GEN _index {}: {error}", entry._index))?;
            if entry.message.contains('\n') {
                return Err(format!(
                    "GEN _index {} contains LF; GEN line allocation is automatic",
                    entry._index
                ));
            }
            let projected = prepare_message(&entry.message)?;
            if let Some(group_index) = entry._gen_group_index {
                let group = &self.groups[group_index];
                let chunks = split_gen_message(&projected, group)?;
                for (chunk, encoded) in group.chunks.iter().zip(chunks) {
                    write_fixed_string(
                        &mut rebuilt,
                        self.data_file_offset + chunk.data_offset,
                        chunk.capacity,
                        &encoded,
                    )?;
                }
            } else {
                let fixed_index = entry._slot_index.expect("validated GEN fixed index");
                let string = &self.fixed_strings[fixed_index];
                let encoded = encode_cp932(&projected)?;
                write_fixed_string(
                    &mut rebuilt,
                    self.data_file_offset + string.data_offset,
                    string.capacity,
                    &encoded,
                )?;
            }
            patched += 1;
        }
        Ok((rebuilt, patched, unchanged))
    }
}

fn join_gen_group(group: &GenGroup) -> String {
    let mut output = String::new();
    for (index, chunk) in group.chunks.iter().enumerate() {
        if index == 0 {
            output.push_str(&chunk.text);
        } else {
            output.push_str(chunk.text.trim_start_matches('　'));
        }
    }
    remove_layout_line_breaks(&output)
}

fn split_gen_message(text: &str, group: &GenGroup) -> Result<Vec<Vec<u8>>> {
    let total_capacity: usize = group.chunks.iter().map(|chunk| chunk.capacity).sum();
    let encoded = encode_cp932(text)?;
    if encoded.len() > total_capacity {
        return Err(format!(
            "GEN message uses {} CP932 bytes, exceeds group capacity {}",
            encoded.len(),
            total_capacity
        ));
    }

    let mut output = vec![Vec::new(); group.chunks.len()];
    let mut chunk_index = 0usize;
    for character in text.chars() {
        let character_bytes = encode_cp932(&character.to_string())?;
        while chunk_index < group.chunks.len()
            && output[chunk_index].len() + character_bytes.len()
                > group.chunks[chunk_index].capacity
        {
            chunk_index += 1;
        }
        if chunk_index == group.chunks.len() {
            return Err(format!(
                "GEN message cannot be divided across {} fixed string slots",
                group.chunks.len()
            ));
        }
        output[chunk_index].extend_from_slice(&character_bytes);
    }
    Ok(output)
}

fn parse_mes(bytes: &[u8]) -> Result<MesFile> {
    if bytes.is_empty() || bytes.last() != Some(&0) {
        return Err("MES file is not NUL-terminated".to_string());
    }
    let mut texts = Vec::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let relative_nul = bytes[cursor..]
            .iter()
            .position(|&byte| byte == 0)
            .ok_or_else(|| "MES string has no NUL terminator".to_string())?;
        if relative_nul == 0 {
            return Err("MES file contains an empty string".to_string());
        }
        let text = decode_cp932(&bytes[cursor..cursor + relative_nul])?;
        if !text.starts_with('『') || !text.ends_with('』') {
            return Err("MES string is not enclosed by 『...』".to_string());
        }
        texts.push(text);
        cursor += relative_nul + 1;
    }
    if texts.len() != 5 {
        return Err(format!("MES file has {} strings, expected 5", texts.len()));
    }
    Ok(MesFile { texts })
}

impl MesFile {
    fn physical_line_stats(&self) -> Result<(usize, usize, usize)> {
        let mut maximum = 0usize;
        let mut over_limit = 0usize;
        for text in &self.texts {
            let bytes = visible_cp932_len(text)?;
            maximum = maximum.max(bytes);
            if bytes > VISIBLE_LINE_BYTE_LIMIT {
                over_limit += 1;
            }
        }
        Ok((self.texts.len(), maximum, over_limit))
    }

    fn translation_entries(&self, source_file: &str) -> Result<Vec<TranslationEntry>> {
        let mut entries = Vec::with_capacity(self.texts.len());
        let mut offset = 0usize;
        for (slot_index, text) in self.texts.iter().enumerate() {
            let size = encode_cp932(text)?.len();
            entries.push(TranslationEntry {
                _index: slot_index,
                _file: source_file.to_string(),
                _slot_index: Some(slot_index),
                _part_index: None,
                _choice_index: None,
                _gen_group_index: None,
                _group_index: None,
                _capacity: None,
                _offset: offset,
                _size: size,
                _type: "hint".to_string(),
                _encoding: "cp932".to_string(),
                _policy: "rebuild".to_string(),
                scr_msg: text.clone(),
                message: text.clone(),
            });
            offset += size + 1;
        }
        Ok(entries)
    }

    fn rebuild_with_document(
        &self,
        document: &TranslationDocument,
    ) -> Result<(Vec<u8>, usize, usize)> {
        validate_document_header(document)?;
        let expected_entries = self.translation_entries(&document._source_file)?;
        if document.entries.len() != expected_entries.len() {
            return Err(format!(
                "JSON entry count {} does not match MES string count {}",
                document.entries.len(),
                expected_entries.len()
            ));
        }
        let mut rebuilt = Vec::new();
        let mut patched = 0usize;
        let mut unchanged = 0usize;
        for (expected, entry) in expected_entries.iter().zip(&document.entries) {
            if entry._index != expected._index
                || entry._file != expected._file
                || entry._slot_index != expected._slot_index
                || entry._part_index.is_some()
                || entry._choice_index.is_some()
                || entry._gen_group_index.is_some()
                || entry._group_index.is_some()
                || entry._capacity.is_some()
                || entry._offset != expected._offset
                || entry._size != expected._size
                || entry._type != "hint"
                || entry._encoding != "cp932"
                || entry._policy != "rebuild"
            {
                return Err(format!(
                    "JSON metadata mismatch at MES _index {}",
                    expected._index
                ));
            }
            if entry.scr_msg != expected.scr_msg {
                return Err(format!(
                    "scr_msg mismatch at MES _index {}",
                    expected._index
                ));
            }
            if entry.message.contains(['\r', '\n']) {
                return Err(format!(
                    "MES _index {} contains a physical line break",
                    entry._index
                ));
            }
            let output_text = if entry.message == entry.scr_msg {
                unchanged += 1;
                entry.scr_msg.clone()
            } else {
                patched += 1;
                prepare_message(&entry.message)?
            };
            rebuilt.extend_from_slice(&encode_cp932(&output_text)?);
            rebuilt.push(0);
        }
        Ok((rebuilt, patched, unchanged))
    }
}

fn parse_main(bytes: &[u8]) -> Result<FixedExeFile> {
    let mut specs = Vec::with_capacity(MAIN_COMMAND_OFFSETS.len() + MAIN_OTHER_SPECS.len());
    specs.extend(
        MAIN_COMMAND_OFFSETS
            .iter()
            .map(|&data_offset| FixedStringSpec {
                data_offset,
                capacity: 10,
                text_type: "command",
            }),
    );
    specs.extend_from_slice(&MAIN_OTHER_SPECS);
    parse_fixed_exe(
        bytes,
        SOURCE_KIND_MAIN,
        0x01C0,
        0x0E8A,
        73_508,
        0x00C2,
        b"area0.dat\0",
        &specs,
    )
}

fn parse_opening(bytes: &[u8]) -> Result<FixedExeFile> {
    parse_fixed_exe(
        bytes,
        SOURCE_KIND_OPENING,
        0x00C0,
        0x0839,
        39_324,
        0x0190,
        b"circle1.dat\0",
        &OPENING_SPECS,
    )
}

fn parse_hisauchi(bytes: &[u8]) -> Result<FixedExeFile> {
    parse_fixed_exe(
        bytes,
        SOURCE_KIND_HISAUCHI,
        0x0080,
        0x05F7,
        28_712,
        0x0048,
        b"opening.exe\0",
        &HISAUCHI_SPECS,
    )
}

fn parse_ending(bytes: &[u8]) -> Result<FixedExeFile> {
    parse_fixed_exe(
        bytes,
        SOURCE_KIND_ENDING,
        0x0080,
        0x05ED,
        29_116,
        0x0036,
        b"data\\end1.dat\0",
        &ENDING_SPECS,
    )
}

fn parse_inter1(bytes: &[u8]) -> Result<InterFile> {
    let data_file_offset = validate_fixed_exe_layout(
        bytes,
        SOURCE_KIND_INTER1,
        0x00C0,
        usize::from(INTER1_DATA_SEGMENT),
        35_000,
        0x0662,
        b"im1.dat\0",
    )?;
    let instruction_slots = parse_packed_strings(
        bytes,
        data_file_offset,
        INTER1_PACKED_POOL_START,
        15,
        "instruction",
    )?;
    let instruction_end = instruction_slots
        .last()
        .map(|slot| slot.data_offset + slot.capacity + 1)
        .unwrap_or(0);
    if instruction_end > INTER1_PACKED_POOL_END
        || bytes[data_file_offset + instruction_end..data_file_offset + INTER1_PACKED_POOL_END]
            .iter()
            .any(|&byte| byte != 0)
    {
        return Err(format!(
            "inter1 instruction pool ending at 0x{instruction_end:04X} exceeds or corrupts its reserved range"
        ));
    }
    if bytes.get(data_file_offset + INTER1_PACKED_POOL_END) != Some(&0) {
        return Err("inter1 instruction pool has no trailing separator NUL".to_string());
    }
    for (index, slot) in instruction_slots.iter().enumerate() {
        let pointer = data_file_offset + INTER1_POINTER_TABLE + index * 4;
        if usize::from(read_u16(bytes, pointer)?) != slot.data_offset
            || read_u16(bytes, pointer + 2)? != INTER1_DATA_SEGMENT
        {
            return Err(format!(
                "inter1 instruction pointer {index} does not match its string"
            ));
        }
    }

    let essay_slots =
        parse_fixed_slot_series(bytes, data_file_offset, 0x0374, 11, 58, "commentary")?;
    let mut groups = Vec::new();
    for &(start, end) in &[(0, 2), (2, 8), (8, 11), (11, 15)] {
        groups.push(inter_group(
            &instruction_slots[start..end],
            "instruction",
            InterGroupStyle::Instruction,
        ));
    }
    for &(start, end) in &[(0, 2), (2, 6), (6, 9), (9, 11)] {
        groups.push(inter_group(
            &essay_slots[start..end],
            "commentary",
            InterGroupStyle::IndentedParagraph,
        ));
    }

    Ok(InterFile {
        source_kind: SOURCE_KIND_INTER1,
        data_file_offset,
        groups,
        fixed_strings: parse_fixed_strings(bytes, data_file_offset, &INTER1_UI_SPECS)?,
        physical_lines: 15 + 11 + INTER1_UI_SPECS.len(),
        packed_pool: Some(InterPackedPool {
            start_offset: INTER1_PACKED_POOL_START,
            end_offset: INTER1_PACKED_POOL_END,
            pointer_table_offset: INTER1_POINTER_TABLE,
            segment: INTER1_DATA_SEGMENT,
            group_count: 4,
        }),
    })
}

fn parse_inter2(bytes: &[u8]) -> Result<InterFile> {
    let data_file_offset = validate_fixed_exe_layout(
        bytes,
        SOURCE_KIND_INTER2,
        0x00A0,
        usize::from(INTER2_DATA_SEGMENT),
        37_438,
        0x204B,
        b"window.dat\0",
    )?;
    let intro_slots =
        parse_fixed_slot_series(bytes, data_file_offset, 0x004E, 37, 58, "commentary")?;
    if bytes
        .get(data_file_offset + 0x08D5..data_file_offset + 0x0910)
        .is_none_or(|gap| gap.iter().any(|&byte| byte != 0))
    {
        return Err("inter2 reserved zero gap does not match".to_string());
    }
    let layout_slots =
        parse_fixed_slot_series(bytes, data_file_offset, 0x0910, 150, 38, "analysis")?;
    const BLANK_LAYOUT_SLOTS: [usize; 43] = [
        1, 30, 31, 33, 42, 43, 44, 45, 59, 60, 73, 74, 75, 76, 77, 86, 87, 88, 89, 90, 91, 93, 103,
        104, 105, 106, 107, 108, 116, 117, 118, 119, 120, 122, 133, 134, 135, 136, 137, 146, 147,
        148, 149,
    ];
    for &index in &BLANK_LAYOUT_SLOTS {
        if layout_slots[index].text != "　".repeat(19) {
            return Err(format!("inter2 layout blank slot {index} does not match"));
        }
    }

    let mut groups = Vec::new();
    for &(start, end) in &[
        (0, 6),
        (6, 11),
        (11, 19),
        (19, 22),
        (22, 30),
        (30, 34),
        (34, 37),
    ] {
        groups.push(inter_group(
            &intro_slots[start..end],
            "commentary",
            InterGroupStyle::IndentedParagraph,
        ));
    }
    for &(start, end, text_type, style) in &[
        (0, 1, "title", InterGroupStyle::Title),
        (2, 19, "analysis", InterGroupStyle::Plain),
        (19, 30, "analysis", InterGroupStyle::Plain),
        (32, 33, "title", InterGroupStyle::Title),
        (34, 42, "analysis", InterGroupStyle::Plain),
        (46, 48, "analysis", InterGroupStyle::Plain),
        (48, 49, "bullet_label", InterGroupStyle::Bullet),
        (49, 50, "bullet_text", InterGroupStyle::Arrow),
        (50, 51, "bullet_label", InterGroupStyle::Bullet),
        (51, 52, "bullet_text", InterGroupStyle::Arrow),
        (52, 53, "analysis", InterGroupStyle::Plain),
        (53, 54, "bullet_label", InterGroupStyle::Bullet),
        (54, 55, "bullet_text", InterGroupStyle::Arrow),
        (55, 56, "bullet_label", InterGroupStyle::Bullet),
        (56, 57, "bullet_text", InterGroupStyle::Arrow),
        (57, 58, "bullet_label", InterGroupStyle::Bullet),
        (58, 59, "bullet_text", InterGroupStyle::Arrow),
        (61, 63, "analysis", InterGroupStyle::Plain),
        (63, 64, "bullet_label", InterGroupStyle::Bullet),
        (64, 65, "bullet_text", InterGroupStyle::Arrow),
        (65, 66, "bullet_label", InterGroupStyle::Bullet),
        (66, 67, "bullet_text", InterGroupStyle::Arrow),
        (67, 68, "bullet_label", InterGroupStyle::Bullet),
        (68, 69, "bullet_text", InterGroupStyle::Arrow),
        (69, 73, "analysis", InterGroupStyle::Plain),
        (78, 79, "bullet_label", InterGroupStyle::Bullet),
        (79, 81, "bullet_text", InterGroupStyle::Arrow),
        (81, 82, "bullet_label", InterGroupStyle::Bullet),
        (82, 83, "bullet_text", InterGroupStyle::Arrow),
        (83, 84, "bullet_label", InterGroupStyle::Bullet),
        (84, 86, "bullet_text", InterGroupStyle::Arrow),
        (92, 93, "title", InterGroupStyle::Title),
        (94, 103, "analysis", InterGroupStyle::Plain),
        (109, 110, "level", InterGroupStyle::Level("Ｄ→")),
        (110, 111, "level", InterGroupStyle::Level("Ｃ→")),
        (111, 113, "level", InterGroupStyle::Level("Ｂ→")),
        (113, 116, "level", InterGroupStyle::Level("Ａ→")),
        (121, 122, "title", InterGroupStyle::Title),
        (123, 133, "analysis", InterGroupStyle::Plain),
        (138, 139, "level", InterGroupStyle::Level("Ｄ→")),
        (139, 141, "level", InterGroupStyle::Level("Ｃ→")),
        (141, 143, "level", InterGroupStyle::Level("Ｂ→")),
        (143, 146, "level", InterGroupStyle::Level("Ａ→")),
    ] {
        groups.push(inter_group(&layout_slots[start..end], text_type, style));
    }

    Ok(InterFile {
        source_kind: SOURCE_KIND_INTER2,
        data_file_offset,
        groups,
        fixed_strings: parse_fixed_strings(bytes, data_file_offset, &INTER2_UI_SPECS)?,
        physical_lines: 37 + 150 + INTER2_UI_SPECS.len(),
        packed_pool: None,
    })
}

fn parse_bunsyo(bytes: &[u8]) -> Result<BunsyoFile> {
    let payload_size = BUNSYO_RECORD_COUNT * BUNSYO_RECORD_SIZE;
    if bytes.len() != payload_size + 1 || bytes[payload_size] != 0x1A {
        return Err(format!(
            "BUNSYO is {} bytes or has no DOS EOF, expected {} bytes ending in 0x1A",
            bytes.len(),
            payload_size + 1
        ));
    }
    let choice_prefixes = [encode_cp932("Ⅰ）")?, encode_cp932("Ⅱ）")?];
    let continuation = encode_cp932("　　")?;
    let mut records = Vec::with_capacity(BUNSYO_RECORD_COUNT);
    for record_index in 0..BUNSYO_RECORD_COUNT {
        let record_offset = record_index * BUNSYO_RECORD_SIZE;
        let record = &bytes[record_offset..record_offset + BUNSYO_RECORD_SIZE];
        let mut rows = Vec::with_capacity(11);
        let mut cursor = 0usize;
        while cursor < record.len() {
            let relative_end = record[cursor..]
                .windows(2)
                .position(|pair| pair == b"\r\n")
                .ok_or_else(|| format!("BUNSYO record {record_index} has a truncated line"))?;
            rows.push((cursor, &record[cursor..cursor + relative_end]));
            cursor += relative_end + 2;
        }
        if rows.len() != 11 || !rows[10].1.is_empty() {
            return Err(format!(
                "BUNSYO record {record_index} does not contain 11 physical lines"
            ));
        }
        let separators: Vec<_> = rows
            .iter()
            .enumerate()
            .filter_map(|(index, (_, row))| row.is_empty().then_some(index))
            .collect();
        if separators.len() != 2 || separators[1] != 10 {
            return Err(format!(
                "BUNSYO record {record_index} has unexpected blank-line separators"
            ));
        }
        for (index, (_, row)) in rows.iter().enumerate() {
            if !row.is_empty() && row.len() != BUNSYO_LINE_BYTE_LIMIT {
                return Err(format!(
                    "BUNSYO record {record_index} row {index} is {} bytes, expected {}",
                    row.len(),
                    BUNSYO_LINE_BYTE_LIMIT
                ));
            }
        }

        let separator = separators[0];
        let prompt_start = rows[..separator]
            .iter()
            .position(|(_, row)| !is_fullwidth_padding(row))
            .ok_or_else(|| format!("BUNSYO record {record_index} has no prompt"))?;
        if rows[prompt_start..separator]
            .iter()
            .any(|(_, row)| is_fullwidth_padding(row))
        {
            return Err(format!(
                "BUNSYO record {record_index} has padding inside its prompt"
            ));
        }
        let first_choice = rows[separator + 1..10]
            .iter()
            .position(|(_, row)| row.starts_with(&choice_prefixes[0]))
            .map(|index| separator + 1 + index)
            .ok_or_else(|| format!("BUNSYO record {record_index} has no choice I"))?;
        let second_choice = rows[first_choice + 1..10]
            .iter()
            .position(|(_, row)| row.starts_with(&choice_prefixes[1]))
            .map(|index| first_choice + 1 + index)
            .ok_or_else(|| format!("BUNSYO record {record_index} has no choice II"))?;
        if first_choice != separator + 1 {
            return Err(format!(
                "BUNSYO record {record_index} has padding before choice I"
            ));
        }
        let tail_start = rows[second_choice + 1..10]
            .iter()
            .position(|(_, row)| is_fullwidth_padding(row))
            .map(|index| second_choice + 1 + index)
            .unwrap_or(10);
        if rows[tail_start..10]
            .iter()
            .any(|(_, row)| !is_fullwidth_padding(row))
        {
            return Err(format!(
                "BUNSYO record {record_index} has text after trailing padding"
            ));
        }

        let prompt = decode_joined_rows(&rows[prompt_start..separator], None, None)
            .map_err(|error| format!("BUNSYO record {record_index} prompt: {error}"))?;
        let first = decode_joined_rows(
            &rows[first_choice..second_choice],
            Some(&choice_prefixes[0]),
            Some(&continuation),
        )
        .map_err(|error| format!("BUNSYO record {record_index} choice I: {error}"))?;
        let second = decode_joined_rows(
            &rows[second_choice..tail_start],
            Some(&choice_prefixes[1]),
            Some(&continuation),
        )
        .map_err(|error| format!("BUNSYO record {record_index} choice II: {error}"))?;
        let used =
            separator - prompt_start + second_choice - first_choice + tail_start - second_choice;
        let leading = prompt_start;
        let trailing = 10 - tail_start;
        if leading != (BUNSYO_DISPLAY_LINES - used) / 2
            || trailing != BUNSYO_DISPLAY_LINES - used - leading
        {
            return Err(format!(
                "BUNSYO record {record_index} is not vertically centered"
            ));
        }
        records.push(BunsyoRecord {
            offset: record_offset,
            prompt_offset: record_offset + rows[prompt_start].0,
            choice_offsets: [
                record_offset + rows[first_choice].0 + choice_prefixes[0].len(),
                record_offset + rows[second_choice].0 + choice_prefixes[1].len(),
            ],
            prompt,
            choices: [first, second],
        });
    }
    Ok(BunsyoFile { records })
}

fn decode_joined_rows(
    rows: &[(usize, &[u8])],
    first_prefix: Option<&[u8]>,
    continuation_prefix: Option<&[u8]>,
) -> Result<String> {
    let mut encoded = Vec::new();
    for (index, (_, row)) in rows.iter().enumerate() {
        let trimmed = trim_fullwidth_padding(row);
        let body = if index == 0 {
            if let Some(prefix) = first_prefix {
                trimmed
                    .strip_prefix(prefix)
                    .ok_or_else(|| "first row has an unexpected prefix".to_string())?
            } else {
                trimmed
            }
        } else {
            match continuation_prefix {
                Some(prefix) => trimmed
                    .strip_prefix(prefix)
                    .ok_or_else(|| "continuation row has an unexpected prefix".to_string())?,
                None => trimmed,
            }
        };
        encoded.extend_from_slice(body);
    }
    if encoded.is_empty() {
        return Err("logical text is empty".to_string());
    }
    decode_cp932(&encoded)
}

fn trim_fullwidth_padding(mut bytes: &[u8]) -> &[u8] {
    while bytes.ends_with(&[0x81, 0x40]) {
        bytes = &bytes[..bytes.len() - 2];
    }
    bytes
}

fn is_fullwidth_padding(bytes: &[u8]) -> bool {
    !bytes.is_empty()
        && bytes.len().is_multiple_of(2)
        && bytes.chunks_exact(2).all(|pair| pair == [0x81, 0x40])
}

fn parse_packed_strings(
    bytes: &[u8],
    data_file_offset: usize,
    start_offset: usize,
    count: usize,
    text_type: &'static str,
) -> Result<Vec<FixedString>> {
    let mut strings = Vec::with_capacity(count);
    let mut cursor = start_offset;
    for index in 0..count {
        let start = data_file_offset
            .checked_add(cursor)
            .ok_or_else(|| "packed string offset overflow".to_string())?;
        let relative_nul = bytes
            .get(start..)
            .and_then(|tail| tail.iter().position(|&byte| byte == 0))
            .ok_or_else(|| format!("packed string {index} has no NUL terminator"))?;
        if relative_nul == 0 {
            return Err(format!("packed string {index} is empty"));
        }
        let text = decode_cp932(&bytes[start..start + relative_nul])
            .map_err(|error| format!("packed string {index}: {error}"))?;
        strings.push(FixedString {
            data_offset: cursor,
            capacity: relative_nul,
            text,
            text_type,
        });
        cursor += relative_nul + 1;
    }
    Ok(strings)
}

fn parse_fixed_slot_series(
    bytes: &[u8],
    data_file_offset: usize,
    start_offset: usize,
    count: usize,
    capacity: usize,
    text_type: &'static str,
) -> Result<Vec<FixedString>> {
    let specs: Vec<_> = (0..count)
        .map(|index| FixedStringSpec {
            data_offset: start_offset + index * (capacity + 1),
            capacity,
            text_type,
        })
        .collect();
    parse_fixed_strings(bytes, data_file_offset, &specs)
}

fn inter_group(
    slots: &[FixedString],
    text_type: &'static str,
    style: InterGroupStyle,
) -> InterGroup {
    InterGroup {
        slots: slots.to_vec(),
        text_type,
        style,
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_fixed_exe(
    bytes: &[u8],
    source_kind: &'static str,
    header_paragraphs: usize,
    data_segment: usize,
    expected_file_size: usize,
    anchor_offset: usize,
    anchor: &[u8],
    specs: &[FixedStringSpec],
) -> Result<FixedExeFile> {
    let data_file_offset = validate_fixed_exe_layout(
        bytes,
        source_kind,
        header_paragraphs,
        data_segment,
        expected_file_size,
        anchor_offset,
        anchor,
    )?;
    let strings = parse_fixed_strings(bytes, data_file_offset, specs)?;
    Ok(FixedExeFile {
        source_kind,
        data_file_offset,
        strings,
    })
}

#[allow(clippy::too_many_arguments)]
fn validate_fixed_exe_layout(
    bytes: &[u8],
    source_kind: &'static str,
    header_paragraphs: usize,
    data_segment: usize,
    expected_file_size: usize,
    anchor_offset: usize,
    anchor: &[u8],
) -> Result<usize> {
    if bytes.get(..2) != Some(b"MZ") {
        return Err(format!("{source_kind} executable has no MZ header"));
    }
    if usize::from(read_u16(bytes, 0x08)?) != header_paragraphs {
        return Err(format!(
            "{source_kind} executable has an unexpected MZ header size"
        ));
    }
    if bytes.len() != expected_file_size {
        return Err(format!(
            "{source_kind} executable is {} bytes, expected {expected_file_size}",
            bytes.len()
        ));
    }
    let data_file_offset = header_paragraphs * 16 + data_segment * 16;
    let anchor_start = data_file_offset + anchor_offset;
    if bytes.get(anchor_start..anchor_start + anchor.len()) != Some(anchor) {
        return Err(format!(
            "{source_kind} executable data anchor does not match"
        ));
    }
    Ok(data_file_offset)
}

fn parse_fixed_strings(
    bytes: &[u8],
    data_file_offset: usize,
    specs: &[FixedStringSpec],
) -> Result<Vec<FixedString>> {
    let mut strings = Vec::with_capacity(specs.len());
    for spec in specs {
        let start = data_file_offset
            .checked_add(spec.data_offset)
            .ok_or_else(|| "fixed string file offset overflow".to_string())?;
        let end = start
            .checked_add(spec.capacity + 1)
            .ok_or_else(|| "fixed string range overflow".to_string())?;
        let slot = bytes.get(start..end).ok_or_else(|| {
            format!(
                "fixed string at data offset 0x{:04X} is outside the file",
                spec.data_offset
            )
        })?;
        let nul = slot.iter().position(|&byte| byte == 0).ok_or_else(|| {
            format!(
                "fixed string at data offset 0x{:04X} has no NUL within {} bytes",
                spec.data_offset, spec.capacity
            )
        })?;
        if slot[nul + 1..].iter().any(|&byte| byte != 0) {
            return Err(format!(
                "fixed string at data offset 0x{:04X} has nonzero bytes after its terminator",
                spec.data_offset
            ));
        }
        let text = decode_cp932(&slot[..nul]).map_err(|error| {
            format!(
                "fixed string at data offset 0x{:04X}: {error}",
                spec.data_offset
            )
        })?;
        if text.is_empty() {
            return Err(format!(
                "fixed string at data offset 0x{:04X} is empty",
                spec.data_offset
            ));
        }
        strings.push(FixedString {
            data_offset: spec.data_offset,
            capacity: spec.capacity,
            text,
            text_type: spec.text_type,
        });
    }
    Ok(strings)
}

impl FixedExeFile {
    fn physical_line_stats(&self) -> Result<(usize, usize, usize)> {
        let mut maximum = 0usize;
        let mut over_limit = 0usize;
        for string in &self.strings {
            let bytes = visible_cp932_len(&string.text)?;
            maximum = maximum.max(bytes);
            if bytes > VISIBLE_LINE_BYTE_LIMIT {
                over_limit += 1;
            }
        }
        Ok((self.strings.len(), maximum, over_limit))
    }

    fn translation_entries(&self, source_file: &str) -> Result<Vec<TranslationEntry>> {
        self.strings
            .iter()
            .enumerate()
            .map(|(slot_index, string)| {
                Ok(TranslationEntry {
                    _index: slot_index,
                    _file: source_file.to_string(),
                    _slot_index: Some(slot_index),
                    _part_index: None,
                    _choice_index: None,
                    _gen_group_index: None,
                    _group_index: None,
                    _capacity: Some(string.capacity),
                    _offset: self.data_file_offset + string.data_offset,
                    _size: encode_cp932(&string.text)?.len(),
                    _type: string.text_type.to_string(),
                    _encoding: "cp932".to_string(),
                    _policy: "in_place".to_string(),
                    scr_msg: string.text.clone(),
                    message: string.text.clone(),
                })
            })
            .collect()
    }

    fn rebuild_with_document(
        &self,
        source: &[u8],
        document: &TranslationDocument,
    ) -> Result<(Vec<u8>, usize, usize)> {
        validate_document_header(document)?;
        let expected_entries = self.translation_entries(&document._source_file)?;
        if document.entries.len() != expected_entries.len() {
            return Err(format!(
                "JSON entry count {} does not match {} fixed string count {}",
                document.entries.len(),
                self.source_kind,
                expected_entries.len()
            ));
        }
        let mut rebuilt = source.to_vec();
        let mut patched = 0usize;
        let mut unchanged = 0usize;
        for (expected, entry) in expected_entries.iter().zip(&document.entries) {
            if entry._index != expected._index
                || entry._file != expected._file
                || entry._slot_index != expected._slot_index
                || entry._part_index.is_some()
                || entry._choice_index.is_some()
                || entry._gen_group_index.is_some()
                || entry._group_index.is_some()
                || entry._capacity != expected._capacity
                || entry._offset != expected._offset
                || entry._size != expected._size
                || entry._type != expected._type
                || entry._encoding != "cp932"
                || entry._policy != "in_place"
            {
                return Err(format!(
                    "JSON metadata mismatch at {} _index {}",
                    self.source_kind, expected._index
                ));
            }
            if entry.scr_msg != expected.scr_msg {
                return Err(format!(
                    "scr_msg mismatch at {} _index {}",
                    self.source_kind, expected._index
                ));
            }
            if entry.message == entry.scr_msg {
                unchanged += 1;
                continue;
            }
            if entry.message.contains(['\r', '\n']) {
                return Err(format!(
                    "{} _index {} contains a physical line break",
                    self.source_kind, entry._index
                ));
            }
            let projected = prepare_message(&entry.message)?;
            let encoded = encode_cp932(&projected)?;
            let string = &self.strings[entry._slot_index.expect("validated fixed index")];
            write_fixed_string(
                &mut rebuilt,
                self.data_file_offset + string.data_offset,
                string.capacity,
                &encoded,
            )?;
            patched += 1;
        }
        Ok((rebuilt, patched, unchanged))
    }
}

impl BunsyoFile {
    fn source_messages(&self) -> usize {
        self.records.len()
    }

    fn source_choices(&self) -> usize {
        self.records.len() * 2
    }

    fn physical_line_stats(&self) -> Result<(usize, usize, usize)> {
        let mut maximum = 0usize;
        for record in &self.records {
            maximum = maximum.max(visible_cp932_len(&record.prompt)?.min(BUNSYO_LINE_BYTE_LIMIT));
            for choice in &record.choices {
                maximum = maximum.max(visible_cp932_len(choice)?.min(BUNSYO_LINE_BYTE_LIMIT));
            }
        }
        Ok((self.records.len() * BUNSYO_DISPLAY_LINES, maximum, 0))
    }

    fn translation_entries(&self, source_file: &str) -> Result<Vec<TranslationEntry>> {
        let mut entries = Vec::with_capacity(self.records.len() * 3);
        for (record_index, record) in self.records.iter().enumerate() {
            let prompt_size = encode_cp932(&record.prompt)?.len();
            entries.push(TranslationEntry {
                _index: entries.len(),
                _file: source_file.to_string(),
                _slot_index: Some(record_index),
                _part_index: None,
                _choice_index: None,
                _gen_group_index: None,
                _group_index: None,
                _capacity: None,
                _offset: record.prompt_offset,
                _size: prompt_size,
                _type: "prompt".to_string(),
                _encoding: "cp932".to_string(),
                _policy: "record_reflow".to_string(),
                scr_msg: record.prompt.clone(),
                message: record.prompt.clone(),
            });
            for choice_index in 0..2 {
                let choice = &record.choices[choice_index];
                entries.push(TranslationEntry {
                    _index: entries.len(),
                    _file: source_file.to_string(),
                    _slot_index: Some(record_index),
                    _part_index: None,
                    _choice_index: Some(choice_index),
                    _gen_group_index: None,
                    _group_index: None,
                    _capacity: None,
                    _offset: record.choice_offsets[choice_index],
                    _size: encode_cp932(choice)?.len(),
                    _type: "choice".to_string(),
                    _encoding: "cp932".to_string(),
                    _policy: "record_reflow".to_string(),
                    scr_msg: choice.clone(),
                    message: choice.clone(),
                });
            }
        }
        Ok(entries)
    }

    fn rebuild_with_document(
        &self,
        source: &[u8],
        document: &TranslationDocument,
    ) -> Result<(Vec<u8>, usize, usize)> {
        validate_document_header(document)?;
        let expected = self.translation_entries(&document._source_file)?;
        if document.entries.len() != expected.len() {
            return Err(format!(
                "JSON entry count {} does not match BUNSYO text count {}",
                document.entries.len(),
                expected.len()
            ));
        }
        for (expected, entry) in expected.iter().zip(&document.entries) {
            if entry._index != expected._index
                || entry._file != expected._file
                || entry._slot_index != expected._slot_index
                || entry._part_index.is_some()
                || entry._choice_index != expected._choice_index
                || entry._gen_group_index.is_some()
                || entry._group_index.is_some()
                || entry._capacity.is_some()
                || entry._offset != expected._offset
                || entry._size != expected._size
                || entry._type != expected._type
                || entry._encoding != "cp932"
                || entry._policy != "record_reflow"
            {
                return Err(format!(
                    "JSON metadata mismatch at BUNSYO _index {}",
                    expected._index
                ));
            }
            if entry.scr_msg != expected.scr_msg {
                return Err(format!(
                    "scr_msg mismatch at BUNSYO _index {}",
                    expected._index
                ));
            }
            if entry.message.contains(['\r', '\n']) {
                return Err(format!(
                    "BUNSYO _index {} contains a physical line break",
                    entry._index
                ));
            }
        }

        let mut rebuilt = source.to_vec();
        let mut patched = 0usize;
        let mut unchanged = 0usize;
        for (record_index, record) in self.records.iter().enumerate() {
            let entries = &document.entries[record_index * 3..record_index * 3 + 3];
            let changed = entries.iter().any(|entry| entry.message != entry.scr_msg);
            for entry in entries {
                if entry.message == entry.scr_msg {
                    unchanged += 1;
                } else {
                    patched += 1;
                }
            }
            if !changed {
                continue;
            }
            let prompt = prepare_message(&entries[0].message)?;
            let first = prepare_message(&entries[1].message)?;
            let second = prepare_message(&entries[2].message)?;
            let record_bytes = build_bunsyo_record(&prompt, &first, &second)?;
            rebuilt[record.offset..record.offset + BUNSYO_RECORD_SIZE]
                .copy_from_slice(&record_bytes);
        }
        Ok((rebuilt, patched, unchanged))
    }
}

fn build_bunsyo_record(prompt: &str, first: &str, second: &str) -> Result<Vec<u8>> {
    let prompt_rows = split_cp932_chunks(prompt, BUNSYO_LINE_BYTE_LIMIT)?;
    let choice_rows = [
        split_cp932_chunks(first, 26)?,
        split_cp932_chunks(second, 26)?,
    ];
    if prompt_rows.is_empty() || choice_rows.iter().any(Vec::is_empty) {
        return Err("BUNSYO prompt and choices must not be empty".to_string());
    }
    let used = prompt_rows.len() + choice_rows[0].len() + choice_rows[1].len();
    if used > BUNSYO_DISPLAY_LINES {
        return Err(format!(
            "BUNSYO record needs {used} display lines, exceeds {}",
            BUNSYO_DISPLAY_LINES
        ));
    }
    let leading = (BUNSYO_DISPLAY_LINES - used) / 2;
    let trailing = BUNSYO_DISPLAY_LINES - used - leading;
    let blank = vec![0x81, 0x40]
        .into_iter()
        .cycle()
        .take(BUNSYO_LINE_BYTE_LIMIT)
        .collect::<Vec<_>>();
    let mut rows = vec![blank.clone(); leading];
    for row in prompt_rows {
        rows.push(pad_fullwidth(row, BUNSYO_LINE_BYTE_LIMIT)?);
    }
    rows.push(Vec::new());
    for (choice_index, chunks) in choice_rows.into_iter().enumerate() {
        let first_prefix = encode_cp932(if choice_index == 0 {
            "Ⅰ）"
        } else {
            "Ⅱ）"
        })?;
        let continuation = encode_cp932("　　")?;
        for (line_index, chunk) in chunks.into_iter().enumerate() {
            let mut row = if line_index == 0 {
                first_prefix.clone()
            } else {
                continuation.clone()
            };
            row.extend_from_slice(&chunk);
            rows.push(pad_fullwidth(row, BUNSYO_LINE_BYTE_LIMIT)?);
        }
    }
    rows.extend((0..trailing).map(|_| blank.clone()));
    rows.push(Vec::new());
    if rows.len() != 11 {
        return Err("internal BUNSYO line count mismatch".to_string());
    }
    let mut output = Vec::with_capacity(BUNSYO_RECORD_SIZE);
    for row in rows {
        output.extend_from_slice(&row);
        output.extend_from_slice(b"\r\n");
    }
    if output.len() != BUNSYO_RECORD_SIZE {
        return Err("internal BUNSYO record size mismatch".to_string());
    }
    Ok(output)
}

fn split_cp932_chunks(text: &str, capacity: usize) -> Result<Vec<Vec<u8>>> {
    let mut chunks = vec![Vec::new()];
    for character in text.chars() {
        let encoded = encode_cp932(&character.to_string())?;
        if encoded.len() > capacity {
            return Err(format!(
                "character {character} exceeds the {capacity}-byte line capacity"
            ));
        }
        if chunks.last().expect("one chunk").len() + encoded.len() > capacity {
            chunks.push(Vec::new());
        }
        chunks
            .last_mut()
            .expect("one chunk")
            .extend_from_slice(&encoded);
    }
    if text.is_empty() {
        chunks.clear();
    }
    Ok(chunks)
}

fn pad_fullwidth(mut encoded: Vec<u8>, capacity: usize) -> Result<Vec<u8>> {
    if encoded.len() > capacity {
        return Err(format!(
            "encoded text uses {} bytes, exceeds fixed capacity {capacity}",
            encoded.len()
        ));
    }
    let remaining = capacity - encoded.len();
    if !remaining.is_multiple_of(2) {
        return Err(format!(
            "encoded text uses {} bytes and cannot be padded to {capacity} with fullwidth spaces",
            encoded.len()
        ));
    }
    for _ in 0..remaining / 2 {
        encoded.extend_from_slice(&[0x81, 0x40]);
    }
    Ok(encoded)
}

impl InterFile {
    fn physical_line_stats(&self) -> Result<(usize, usize, usize)> {
        let mut maximum = 0usize;
        let mut over_limit = 0usize;
        for string in self
            .groups
            .iter()
            .flat_map(|group| &group.slots)
            .chain(&self.fixed_strings)
        {
            let bytes = visible_cp932_len(&string.text)?;
            maximum = maximum.max(bytes);
            if bytes > INTER_LINE_BYTE_LIMIT {
                over_limit += 1;
            }
        }
        Ok((self.physical_lines, maximum, over_limit))
    }

    fn source_messages(&self) -> usize {
        self.groups.len() + self.fixed_strings.len()
    }

    fn translation_entries(&self, source_file: &str) -> Result<Vec<TranslationEntry>> {
        let mut entries = Vec::with_capacity(self.source_messages());
        for (group_index, group) in self.groups.iter().enumerate() {
            let scr_msg = join_inter_group(group)?;
            let first = group.slots.first().expect("validated inter group");
            entries.push(TranslationEntry {
                _index: entries.len(),
                _file: source_file.to_string(),
                _slot_index: None,
                _part_index: None,
                _choice_index: None,
                _gen_group_index: None,
                _group_index: Some(group_index),
                _capacity: Some(inter_group_capacity(group)?),
                _offset: self.data_file_offset + first.data_offset,
                _size: encode_cp932(&scr_msg)?.len(),
                _type: group.text_type.to_string(),
                _encoding: "cp932".to_string(),
                _policy: if group.style == InterGroupStyle::Instruction {
                    "packed_pool"
                } else {
                    "in_place_group"
                }
                .to_string(),
                scr_msg: scr_msg.clone(),
                message: scr_msg,
            });
        }
        for (slot_index, string) in self.fixed_strings.iter().enumerate() {
            entries.push(TranslationEntry {
                _index: entries.len(),
                _file: source_file.to_string(),
                _slot_index: Some(slot_index),
                _part_index: None,
                _choice_index: None,
                _gen_group_index: None,
                _group_index: None,
                _capacity: Some(string.capacity),
                _offset: self.data_file_offset + string.data_offset,
                _size: encode_cp932(&string.text)?.len(),
                _type: string.text_type.to_string(),
                _encoding: "cp932".to_string(),
                _policy: "in_place".to_string(),
                scr_msg: string.text.clone(),
                message: string.text.clone(),
            });
        }
        Ok(entries)
    }

    fn rebuild_with_document(
        &self,
        source: &[u8],
        document: &TranslationDocument,
    ) -> Result<(Vec<u8>, usize, usize)> {
        validate_document_header(document)?;
        let expected_entries = self.translation_entries(&document._source_file)?;
        if document.entries.len() != expected_entries.len() {
            return Err(format!(
                "JSON entry count {} does not match {} text count {}",
                document.entries.len(),
                self.source_kind,
                expected_entries.len()
            ));
        }

        let mut rebuilt = source.to_vec();
        let mut patched = 0usize;
        let mut unchanged = 0usize;
        let mut changed_groups: HashMap<usize, Vec<Vec<u8>>> = HashMap::new();
        for (expected, entry) in expected_entries.iter().zip(&document.entries) {
            if entry._index != expected._index
                || entry._file != expected._file
                || entry._slot_index != expected._slot_index
                || entry._part_index.is_some()
                || entry._choice_index.is_some()
                || entry._gen_group_index.is_some()
                || entry._group_index != expected._group_index
                || entry._capacity != expected._capacity
                || entry._offset != expected._offset
                || entry._size != expected._size
                || entry._type != expected._type
                || entry._encoding != "cp932"
                || entry._policy != expected._policy
            {
                return Err(format!(
                    "JSON metadata mismatch at {} _index {}",
                    self.source_kind, expected._index
                ));
            }
            if entry.scr_msg != expected.scr_msg {
                return Err(format!(
                    "scr_msg mismatch at {} _index {}",
                    self.source_kind, expected._index
                ));
            }
            if entry.message == entry.scr_msg {
                unchanged += 1;
                continue;
            }
            if entry.message.contains(['\r', '\n']) {
                return Err(format!(
                    "{} _index {} contains a physical line break",
                    self.source_kind, entry._index
                ));
            }
            let projected = prepare_message(&entry.message)?;
            if let Some(group_index) = entry._group_index {
                let group = &self.groups[group_index];
                let chunks = split_inter_message(&projected, group)?;
                if group.style == InterGroupStyle::Instruction {
                    changed_groups.insert(group_index, chunks);
                } else {
                    for (slot, encoded) in group.slots.iter().zip(&chunks) {
                        write_padded_fixed_string(
                            &mut rebuilt,
                            self.data_file_offset + slot.data_offset,
                            slot.capacity,
                            encoded,
                        )?;
                    }
                }
            } else {
                let slot_index = entry._slot_index.expect("validated fixed slot");
                let string = &self.fixed_strings[slot_index];
                let encoded = encode_cp932(&projected)?;
                write_fixed_string(
                    &mut rebuilt,
                    self.data_file_offset + string.data_offset,
                    string.capacity,
                    &encoded,
                )?;
            }
            patched += 1;
        }

        if let Some(pool) = &self.packed_pool {
            if changed_groups
                .keys()
                .any(|&group_index| group_index < pool.group_count)
            {
                let mut strings = Vec::new();
                for group_index in 0..pool.group_count {
                    let group = &self.groups[group_index];
                    if let Some(chunks) = changed_groups.get(&group_index) {
                        strings.extend(chunks.iter().cloned());
                    } else {
                        for slot in &group.slots {
                            strings.push(encode_cp932(&slot.text)?);
                        }
                    }
                }
                let mut packed = Vec::new();
                let mut offsets = Vec::with_capacity(strings.len());
                for string in strings {
                    offsets.push(pool.start_offset + packed.len());
                    packed.extend_from_slice(&string);
                    packed.push(0);
                }
                let capacity = pool.end_offset - pool.start_offset;
                if packed.len() > capacity {
                    return Err(format!(
                        "inter1 instruction strings use {} bytes, exceed packed pool capacity {capacity}",
                        packed.len()
                    ));
                }
                let pool_range = self.data_file_offset + pool.start_offset
                    ..self.data_file_offset + pool.end_offset;
                rebuilt[pool_range.clone()].fill(0);
                rebuilt[pool_range.start..pool_range.start + packed.len()].copy_from_slice(&packed);
                for (index, offset) in offsets.into_iter().enumerate() {
                    let pointer = self.data_file_offset + pool.pointer_table_offset + index * 4;
                    let offset = u16::try_from(offset)
                        .map_err(|_| "inter1 packed pointer exceeds u16".to_string())?;
                    rebuilt[pointer..pointer + 2].copy_from_slice(&offset.to_le_bytes());
                    rebuilt[pointer + 2..pointer + 4].copy_from_slice(&pool.segment.to_le_bytes());
                }
            }
        }
        Ok((rebuilt, patched, unchanged))
    }
}

fn join_inter_group(group: &InterGroup) -> Result<String> {
    let mut output = String::new();
    for (index, slot) in group.slots.iter().enumerate() {
        let body = match group.style {
            InterGroupStyle::Instruction => slot
                .text
                .strip_prefix(if index == 0 { "※　" } else { "　　" })
                .ok_or_else(|| format!("inter1 instruction slot {index} has no layout prefix"))?
                .trim_end_matches('　'),
            InterGroupStyle::IndentedParagraph => {
                slot.text.trim_end_matches('　').trim_start_matches('　')
            }
            InterGroupStyle::Plain => slot.text.trim_end_matches('　'),
            InterGroupStyle::Title => slot.text.trim_matches('　'),
            InterGroupStyle::Bullet => slot
                .text
                .strip_prefix('・')
                .ok_or_else(|| "inter2 bullet has no bullet prefix".to_string())?
                .trim_end_matches('　'),
            InterGroupStyle::Arrow => slot
                .text
                .strip_prefix(if index == 0 { "　→" } else { "　　" })
                .ok_or_else(|| format!("inter2 arrow slot {index} has no layout prefix"))?
                .trim_end_matches('　'),
            InterGroupStyle::Level(prefix) => slot
                .text
                .strip_prefix(if index == 0 { prefix } else { "　　" })
                .ok_or_else(|| format!("inter2 level slot {index} has no layout prefix"))?
                .trim_end_matches('　'),
        };
        output.push_str(body);
    }
    if output.is_empty() {
        return Err("intermission logical group is empty".to_string());
    }
    Ok(output)
}

fn split_inter_message(text: &str, group: &InterGroup) -> Result<Vec<Vec<u8>>> {
    if text.is_empty() {
        return Err("intermission message must not be empty".to_string());
    }
    if group.style == InterGroupStyle::Title {
        let encoded = encode_cp932(text)?;
        if group.slots.len() != 1 || encoded.len() > group.slots[0].capacity {
            return Err("inter2 title exceeds its fixed slot".to_string());
        }
        if encoded.len() % 2 != 0 {
            return Err("inter2 title must occupy a whole number of fullwidth cells".to_string());
        }
        let remaining_cells = (group.slots[0].capacity - encoded.len()) / 2;
        let left = remaining_cells / 2;
        let right = remaining_cells - left;
        let mut centered = Vec::with_capacity(group.slots[0].capacity);
        for _ in 0..left {
            centered.extend_from_slice(&[0x81, 0x40]);
        }
        centered.extend_from_slice(&encoded);
        for _ in 0..right {
            centered.extend_from_slice(&[0x81, 0x40]);
        }
        return Ok(vec![centered]);
    }

    let prefixes: Vec<Vec<u8>> = (0..group.slots.len())
        .map(|index| encode_cp932(inter_group_prefix(group.style, index)))
        .collect::<Result<_>>()?;
    let capacities: Vec<_> = group
        .slots
        .iter()
        .enumerate()
        .map(|(index, slot)| {
            let physical_capacity = if group.style == InterGroupStyle::Instruction {
                INTER_LINE_BYTE_LIMIT
            } else {
                slot.capacity
            };
            physical_capacity
                .checked_sub(prefixes[index].len())
                .ok_or_else(|| "intermission prefix exceeds slot capacity".to_string())
        })
        .collect::<Result<_>>()?;
    let mut bodies = vec![Vec::new(); group.slots.len()];
    let mut slot_index = 0usize;
    for character in text.chars() {
        let character_bytes = encode_cp932(&character.to_string())?;
        while slot_index < bodies.len()
            && bodies[slot_index].len() + character_bytes.len() > capacities[slot_index]
        {
            slot_index += 1;
        }
        if slot_index == bodies.len() {
            return Err(format!(
                "intermission message cannot be divided across {} slots with capacity {}",
                group.slots.len(),
                inter_group_capacity(group)?
            ));
        }
        bodies[slot_index].extend_from_slice(&character_bytes);
    }
    let mut output = Vec::with_capacity(bodies.len());
    for (index, body) in bodies.into_iter().enumerate() {
        let mut encoded = prefixes[index].clone();
        encoded.extend_from_slice(&body);
        if group.style != InterGroupStyle::Instruction {
            encoded = pad_fullwidth(encoded, group.slots[index].capacity)?;
        }
        output.push(encoded);
    }
    Ok(output)
}

fn inter_group_prefix(style: InterGroupStyle, slot_index: usize) -> &'static str {
    match style {
        InterGroupStyle::Instruction => {
            if slot_index == 0 {
                "※　"
            } else {
                "　　"
            }
        }
        InterGroupStyle::IndentedParagraph => {
            if slot_index == 0 {
                "　"
            } else {
                ""
            }
        }
        InterGroupStyle::Plain | InterGroupStyle::Title => "",
        InterGroupStyle::Bullet => "・",
        InterGroupStyle::Arrow => {
            if slot_index == 0 {
                "　→"
            } else {
                "　　"
            }
        }
        InterGroupStyle::Level(prefix) => {
            if slot_index == 0 {
                prefix
            } else {
                "　　"
            }
        }
    }
}

fn inter_group_capacity(group: &InterGroup) -> Result<usize> {
    group
        .slots
        .iter()
        .enumerate()
        .try_fold(0usize, |total, (index, slot)| {
            let physical_capacity = if group.style == InterGroupStyle::Instruction {
                INTER_LINE_BYTE_LIMIT
            } else {
                slot.capacity
            };
            let prefix = encode_cp932(inter_group_prefix(group.style, index))?.len();
            let body = physical_capacity
                .checked_sub(prefix)
                .ok_or_else(|| "intermission prefix exceeds slot capacity".to_string())?;
            Ok(total + body)
        })
}

fn write_padded_fixed_string(
    output: &mut [u8],
    start: usize,
    capacity: usize,
    encoded: &[u8],
) -> Result<()> {
    if encoded.len() != capacity {
        return Err(format!(
            "padded string uses {} bytes, expected exactly {capacity}",
            encoded.len()
        ));
    }
    let target = output
        .get_mut(start..start + capacity + 1)
        .ok_or_else(|| "padded fixed string write is outside the file".to_string())?;
    target[..capacity].copy_from_slice(encoded);
    target[capacity] = 0;
    Ok(())
}

fn write_fixed_string(
    output: &mut [u8],
    start: usize,
    capacity: usize,
    encoded: &[u8],
) -> Result<()> {
    if encoded.len() > capacity {
        return Err(format!(
            "encoded text uses {} bytes, exceeds fixed capacity {}",
            encoded.len(),
            capacity
        ));
    }
    let end = start
        .checked_add(capacity + 1)
        .ok_or_else(|| "fixed string write range overflow".to_string())?;
    let slot = output
        .get_mut(start..end)
        .ok_or_else(|| "fixed string write is outside the file".to_string())?;
    slot.fill(0);
    slot[..encoded.len()].copy_from_slice(encoded);
    Ok(())
}

impl AreaFile {
    pub fn source_messages(&self) -> usize {
        self.slots.len()
    }

    pub fn source_choices(&self) -> usize {
        self.choice_tables
            .iter()
            .map(|table| table.choices.iter().filter(|choice| choice.export).count())
            .sum()
    }

    pub fn physical_line_stats(&self) -> Result<(usize, usize, usize)> {
        let mut lines = 0usize;
        let mut maximum = 0usize;
        let mut over_limit = 0usize;
        for slot in &self.slots {
            for line in slot.text.split('\n') {
                let bytes = visible_cp932_len(line)?;
                lines += 1;
                maximum = maximum.max(bytes);
                if bytes > VISIBLE_LINE_BYTE_LIMIT {
                    over_limit += 1;
                }
            }
        }
        Ok((lines, maximum, over_limit))
    }

    #[cfg(test)]
    fn translation_entries(&self) -> Result<Vec<TranslationEntry>> {
        self.translation_entries_for_file_with_warnings("sample.bin")
            .map(|(entries, _)| entries)
    }

    #[cfg(test)]
    fn translation_entries_with_warnings(&self) -> Result<(Vec<TranslationEntry>, Vec<String>)> {
        self.translation_entries_for_file_with_warnings("sample.bin")
    }

    fn translation_entries_for_file_with_warnings(
        &self,
        source_file: &str,
    ) -> Result<(Vec<TranslationEntry>, Vec<String>)> {
        let mut entries = Vec::new();
        let mut warnings = Vec::new();
        for (slot_index, slot) in self.slots.iter().enumerate() {
            let logical = split_logical_parts(&slot.text)
                .map_err(|error| format!("message slot {slot_index}: {error}"))?;
            warnings.extend(
                logical
                    .warnings
                    .iter()
                    .map(|warning| format!("message slot {slot_index}: {warning}")),
            );
            for (part_index, span) in logical.spans.iter().enumerate() {
                let raw = &slot.text[span.start..span.end];
                let normalized = remove_layout_line_breaks(raw);
                if normalized.trim().is_empty() {
                    continue;
                }
                let prefix_len = encode_cp932(&slot.text[..span.start])?.len();
                let raw_size = encode_cp932(raw)?.len();
                entries.push(TranslationEntry {
                    _index: entries.len(),
                    _file: source_file.to_string(),
                    _slot_index: Some(slot_index),
                    _part_index: Some(part_index),
                    _choice_index: None,
                    _gen_group_index: None,
                    _group_index: None,
                    _capacity: None,
                    _offset: self.text_pool_file_offset + slot.pool_offset + prefix_len,
                    _size: raw_size,
                    _type: span.kind.as_str().to_string(),
                    _encoding: "cp932".to_string(),
                    _policy: "relocate".to_string(),
                    scr_msg: normalized.clone(),
                    message: normalized,
                });
            }
        }
        let mut choice_index = 0usize;
        for table in &self.choice_tables {
            for choice in &table.choices {
                if !choice.export {
                    continue;
                }
                let size = encode_cp932(&choice.text)?.len();
                entries.push(TranslationEntry {
                    _index: entries.len(),
                    _file: source_file.to_string(),
                    _slot_index: None,
                    _part_index: None,
                    _choice_index: Some(choice_index),
                    _gen_group_index: None,
                    _group_index: None,
                    _capacity: None,
                    _offset: table.pool_file_offset + choice.pool_offset,
                    _size: size,
                    _type: "choice".to_string(),
                    _encoding: "cp932".to_string(),
                    _policy: "relocate".to_string(),
                    scr_msg: choice.text.clone(),
                    message: choice.text.clone(),
                });
                choice_index += 1;
            }
        }
        Ok((entries, warnings))
    }

    fn rebuild_with_document(
        &self,
        document: &TranslationDocument,
    ) -> Result<(Vec<u8>, usize, usize)> {
        validate_document_header(document)?;
        let source_entries = self
            .translation_entries_for_file_with_warnings(&document._source_file)?
            .0;
        if document.entries.len() != source_entries.len() {
            return Err(format!(
                "JSON entry count {} does not match source entry count {}",
                document.entries.len(),
                source_entries.len()
            ));
        }

        let mut replacements: HashMap<usize, BTreeMap<usize, &TranslationEntry>> = HashMap::new();
        let mut choice_replacements: HashMap<usize, &TranslationEntry> = HashMap::new();
        let mut seen_indices = HashSet::new();
        for (expected, entry) in source_entries.iter().zip(&document.entries) {
            if !seen_indices.insert(entry._index) {
                return Err(format!("duplicate JSON _index {}", entry._index));
            }
            if entry._index != expected._index
                || entry._file != expected._file
                || entry._slot_index != expected._slot_index
                || entry._part_index != expected._part_index
                || entry._choice_index != expected._choice_index
                || entry._gen_group_index.is_some()
                || entry._group_index.is_some()
                || entry._capacity.is_some()
                || entry._offset != expected._offset
                || entry._size != expected._size
                || entry._type != expected._type
                || entry._encoding != "cp932"
                || entry._policy != "relocate"
            {
                return Err(format!(
                    "JSON metadata mismatch at _index {}",
                    expected._index
                ));
            }
            if entry.scr_msg != expected.scr_msg {
                return Err(format!("scr_msg mismatch at _index {}", expected._index));
            }
            if let Some(choice_index) = entry._choice_index {
                validate_choice(&entry.message)
                    .map_err(|error| format!("_index {}: {error}", entry._index))?;
                choice_replacements.insert(choice_index, entry);
            } else {
                validate_message(&entry.message)
                    .map_err(|error| format!("_index {}: {error}", entry._index))?;
                replacements
                    .entry(entry._slot_index.expect("validated message slot index"))
                    .or_default()
                    .insert(
                        entry._part_index.expect("validated message part index"),
                        entry,
                    );
            }
        }

        let mut patched = 0usize;
        let mut unchanged = 0usize;
        let mut rebuilt_texts = Vec::with_capacity(self.slots.len());
        for (slot_index, slot) in self.slots.iter().enumerate() {
            let logical = split_logical_parts(&slot.text)?;
            let slot_replacements = replacements.get(&slot_index);
            let mut output = String::new();
            let mut cursor = 0usize;
            for (part_index, span) in logical.spans.iter().enumerate() {
                output.push_str(&slot.text[cursor..span.start]);
                if let Some(entry) = slot_replacements.and_then(|items| items.get(&part_index)) {
                    if entry.message == entry.scr_msg {
                        output.push_str(&slot.text[span.start..span.end]);
                        unchanged += 1;
                    } else {
                        output.push_str(&prepare_message(&entry.message)?);
                        patched += 1;
                    }
                } else {
                    output.push_str(&slot.text[span.start..span.end]);
                }
                cursor = span.end;
            }
            output.push_str(&slot.text[cursor..]);
            rebuilt_texts.push(output);
        }

        let mut rebuilt_choices = Vec::with_capacity(self.source_choices());
        let mut choice_index = 0usize;
        for table in &self.choice_tables {
            for choice in &table.choices {
                if !choice.export {
                    continue;
                }
                let entry = choice_replacements
                    .get(&choice_index)
                    .ok_or_else(|| format!("missing JSON choice index {choice_index}"))?;
                if entry.message == entry.scr_msg {
                    rebuilt_choices.push(choice.text.clone());
                    unchanged += 1;
                } else {
                    rebuilt_choices.push(prepare_message(&entry.message)?);
                    patched += 1;
                }
                choice_index += 1;
            }
        }

        let rebuilt = self.rebuild(&rebuilt_texts, &rebuilt_choices)?;
        Ok((rebuilt, patched, unchanged))
    }

    fn rebuild(&self, texts: &[String], choice_texts: &[String]) -> Result<Vec<u8>> {
        if texts.len() != self.slots.len() {
            return Err("internal message count mismatch".to_string());
        }
        if choice_texts.len() != self.source_choices() {
            return Err("internal choice count mismatch".to_string());
        }

        let mut counts = self.counts;
        let mut sections = self.sections.clone();
        let mut choice_cursor = 0usize;
        for table in &self.choice_tables {
            let table_choice_count = table.choices.iter().filter(|choice| choice.export).count();
            let table_end = choice_cursor + table_choice_count;
            let table_texts = &choice_texts[choice_cursor..table_end];
            if table_choice_count == 0 {
                continue;
            }
            let mut choice_pool = Vec::new();
            let mut choice_offsets = Vec::with_capacity(table.choices.len());
            let mut rebuilt_aliases: HashMap<(usize, String), u16> = HashMap::new();
            let mut local_index = 0usize;
            for choice in &table.choices {
                let text = if !choice.export {
                    &choice.text
                } else {
                    let text = &table_texts[local_index];
                    local_index += 1;
                    text
                };
                let alias_key = (choice.pool_offset, text.to_string());
                if let Some(&offset) = rebuilt_aliases.get(&alias_key) {
                    choice_offsets.push(offset);
                    continue;
                }
                if choice_pool.len() > usize::from(u16::MAX) {
                    return Err(format!(
                        "choice pool offset for index {} exceeds u16",
                        choice_cursor + local_index.saturating_sub(1)
                    ));
                }
                let offset = choice_pool.len() as u16;
                let encoded = encode_cp932(text).map_err(|error| {
                    format!(
                        "choice index {}: {error}",
                        choice_cursor + local_index.saturating_sub(1)
                    )
                })?;
                choice_pool.extend_from_slice(&encoded);
                choice_pool.push(0);
                choice_offsets.push(offset);
                rebuilt_aliases.insert(alias_key, offset);
            }
            if choice_pool.len() > usize::from(u16::MAX) {
                return Err(format!(
                    "rebuilt choice pool for sections {}/{} is {} bytes, exceeds u16 limit 65535",
                    table.offset_section,
                    table.pool_section,
                    choice_pool.len()
                ));
            }

            let mut offset_bytes = Vec::with_capacity(choice_offsets.len() * 2);
            for offset in choice_offsets {
                offset_bytes.extend_from_slice(&offset.to_le_bytes());
            }
            sections[table.offset_section] = offset_bytes;
            sections[table.pool_section] = choice_pool;
            counts[table.pool_section] = sections[table.pool_section].len() as u16;
            choice_cursor = table_end;
        }

        let mut pool = Vec::new();
        let mut offsets = Vec::with_capacity(texts.len());
        for (index, (text, slot)) in texts.iter().zip(&self.slots).enumerate() {
            if pool.len() > usize::from(u16::MAX) {
                return Err(format!("text pool offset for slot {index} exceeds u16"));
            }
            offsets.push(pool.len() as u16);
            let encoded =
                encode_cp932(text).map_err(|error| format!("message slot {index}: {error}"))?;
            pool.extend_from_slice(&encoded);
            pool.push(0);
            pool.extend_from_slice(&slot.suffix);
        }
        if pool.len() > usize::from(u16::MAX) {
            return Err(format!(
                "rebuilt text pool is {} bytes, exceeds u16 limit 65535",
                pool.len()
            ));
        }

        counts[21] = pool.len() as u16;
        let mut offset_bytes = Vec::with_capacity(offsets.len() * 2);
        for offset in offsets {
            offset_bytes.extend_from_slice(&offset.to_le_bytes());
        }
        sections[20] = offset_bytes;
        sections[21] = pool;
        let mut output =
            Vec::with_capacity(HEADER_SIZE + sections.iter().map(Vec::len).sum::<usize>());
        for count in counts {
            output.extend_from_slice(&count.to_le_bytes());
        }
        for section in &sections {
            output.extend_from_slice(section);
        }
        Ok(output)
    }
}

pub fn extract_path(input: &Path, output_root: &Path, overwrite: bool) -> Result<ExtractReport> {
    let sources = collect_text_sources(input)?;
    if sources.is_empty() {
        return Err(format!(
            "no structurally valid AREA or GEN text source found in {}",
            input.display()
        ));
    }
    let scanned_files = count_files(input)?;
    let skipped_files = if input.is_file() {
        0
    } else {
        scanned_files.saturating_sub(sources.len())
    };
    let mut basename_counts = HashMap::new();
    for (_, relative, _, _) in &sources {
        let basename = Path::new(relative)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("invalid source basename: {relative}"))?
            .to_lowercase();
        *basename_counts.entry(basename).or_insert(0usize) += 1;
    }
    validate_disjoint_output(output_root, &[("extract input", input)])?;
    let mut prepared = Vec::with_capacity(sources.len());
    let mut source_messages = 0usize;
    let mut source_choices = 0usize;
    let mut extracted_entries = 0usize;
    let mut physical_lines = 0usize;
    let mut max_visible_line_bytes = 0usize;
    let mut over_limit = 0usize;
    let mut warnings = Vec::new();
    for (_path, relative, bytes, source) in sources {
        let basename = Path::new(&relative)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("invalid source basename: {relative}"))?;
        let duplicate_basename = basename_counts
            .get(&basename.to_lowercase())
            .copied()
            .unwrap_or(0)
            > 1;
        let output_name = flat_json_name(&relative, duplicate_basename)?;
        let (entries, source_kind, line_limit, lines, maximum, over) = match source {
            ParsedSource::Area(area) => {
                let (entries, area_warnings) =
                    area.translation_entries_for_file_with_warnings(&relative)?;
                warnings.extend(
                    area_warnings
                        .into_iter()
                        .map(|warning| format!("{relative}: {warning}")),
                );
                let (lines, maximum, over) = area.physical_line_stats()?;
                source_messages += area.source_messages();
                source_choices += area.source_choices();
                (
                    entries,
                    SOURCE_KIND_AREA,
                    VISIBLE_LINE_BYTE_LIMIT,
                    lines,
                    maximum,
                    over,
                )
            }
            ParsedSource::Gen(gen) => {
                let entries = gen.translation_entries(&relative)?;
                let (lines, maximum, over) = gen.physical_line_stats()?;
                source_messages += gen.groups.len() + gen.fixed_strings.len();
                (
                    entries,
                    SOURCE_KIND_GEN,
                    GEN_LINE_BYTE_LIMIT,
                    lines,
                    maximum,
                    over,
                )
            }
            ParsedSource::Mes(mes) => {
                let entries = mes.translation_entries(&relative)?;
                let (lines, maximum, over) = mes.physical_line_stats()?;
                source_messages += mes.texts.len();
                (
                    entries,
                    SOURCE_KIND_MES,
                    VISIBLE_LINE_BYTE_LIMIT,
                    lines,
                    maximum,
                    over,
                )
            }
            ParsedSource::FixedExe(exe) => {
                let entries = exe.translation_entries(&relative)?;
                let (lines, maximum, over) = exe.physical_line_stats()?;
                source_messages += exe.strings.len();
                (
                    entries,
                    exe.source_kind,
                    VISIBLE_LINE_BYTE_LIMIT,
                    lines,
                    maximum,
                    over,
                )
            }
            ParsedSource::Inter(inter) => {
                let entries = inter.translation_entries(&relative)?;
                let (lines, maximum, over) = inter.physical_line_stats()?;
                source_messages += inter.source_messages();
                (
                    entries,
                    inter.source_kind,
                    INTER_LINE_BYTE_LIMIT,
                    lines,
                    maximum,
                    over,
                )
            }
            ParsedSource::Bunsyo(bunsyo) => {
                let entries = bunsyo.translation_entries(&relative)?;
                let (lines, maximum, over) = bunsyo.physical_line_stats()?;
                source_messages += bunsyo.source_messages();
                source_choices += bunsyo.source_choices();
                (
                    entries,
                    SOURCE_KIND_BUNSYO,
                    BUNSYO_LINE_BYTE_LIMIT,
                    lines,
                    maximum,
                    over,
                )
            }
        };
        extracted_entries += entries.len();
        physical_lines += lines;
        max_visible_line_bytes = max_visible_line_bytes.max(maximum);
        over_limit += over;
        prepared.push(PreparedExtraction {
            relative: relative.clone(),
            output_name,
            document: TranslationDocument {
                _format: JSON_FORMAT.to_string(),
                _source_kind: source_kind.to_string(),
                _source_file: relative,
                _source_sha256: sha256_hex(&bytes),
                _line_visible_byte_limit: line_limit,
                entries,
            },
        });
    }

    validate_output_dir(output_root, overwrite)?;
    let staging = unique_staging(output_root, "area-extract")?;
    let result = (|| {
        fs::create_dir(&staging)
            .map_err(|error| format!("cannot create {}: {error}", staging.display()))?;
        for item in &prepared {
            let output = safe_join(&staging, &item.output_name)?;
            let mut json = serde_json::to_vec_pretty(&item.document)
                .map_err(|error| format!("cannot serialize {}: {error}", item.relative))?;
            json.push(b'\n');
            fs::write(&output, json)
                .map_err(|error| format!("cannot write {}: {error}", output.display()))?;
        }
        commit_directory(&staging, output_root, overwrite)
    })();
    if result.is_err() && staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    result?;

    Ok(ExtractReport {
        scanned_files,
        skipped_files,
        json_files: prepared.len(),
        source_messages,
        source_choices,
        extracted_entries,
        physical_lines,
        max_visible_line_bytes,
        over_limit,
        warnings,
        output_root: output_root.to_path_buf(),
    })
}

pub fn inject_path(
    source: &Path,
    translation: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<InjectReport> {
    let documents = load_translation_documents(translation)?;
    if documents.is_empty() {
        return Err(format!(
            "no {} translation JSON found in {}",
            JSON_FORMAT,
            translation.display()
        ));
    }

    let source_is_file = source
        .metadata()
        .map_err(|error| format!("cannot inspect {}: {error}", source.display()))?
        .is_file();
    validate_disjoint_output(output, &[("source", source), ("translation", translation)])?;
    if source_is_file && documents.len() != 1 {
        return Err("single-file source requires exactly one translation JSON".to_string());
    }

    let mut prepared = Vec::with_capacity(documents.len());
    let mut total_entries = 0usize;
    let mut total_patched = 0usize;
    let mut total_unchanged = 0usize;
    for document in documents {
        validate_document_header(&document)?;
        let relative = normalize_manifest_path(&document._source_file)?;
        let source_path = if source_is_file {
            source.to_path_buf()
        } else {
            safe_join(source, &relative)?
        };
        let bytes = fs::read(&source_path)
            .map_err(|error| format!("cannot read {}: {error}", source_path.display()))?;
        if sha256_hex(&bytes) != document._source_sha256 {
            return Err(format!(
                "source SHA-256 mismatch for {}",
                source_path.display()
            ));
        }
        let (rebuilt, patched, unchanged) = match document._source_kind.as_str() {
            SOURCE_KIND_AREA => {
                let area = parse_area(&bytes)
                    .map_err(|error| format!("{}: {error}", source_path.display()))?;
                area.rebuild_with_document(&document)?
            }
            SOURCE_KIND_GEN => {
                let gen = parse_gen(&bytes)
                    .map_err(|error| format!("{}: {error}", source_path.display()))?;
                gen.rebuild_with_document(&bytes, &document)?
            }
            SOURCE_KIND_MES => {
                let mes = parse_mes(&bytes)
                    .map_err(|error| format!("{}: {error}", source_path.display()))?;
                mes.rebuild_with_document(&document)?
            }
            SOURCE_KIND_MAIN => {
                let main = parse_main(&bytes)
                    .map_err(|error| format!("{}: {error}", source_path.display()))?;
                main.rebuild_with_document(&bytes, &document)?
            }
            SOURCE_KIND_OPENING => {
                let opening = parse_opening(&bytes)
                    .map_err(|error| format!("{}: {error}", source_path.display()))?;
                opening.rebuild_with_document(&bytes, &document)?
            }
            SOURCE_KIND_HISAUCHI => {
                let hisauchi = parse_hisauchi(&bytes)
                    .map_err(|error| format!("{}: {error}", source_path.display()))?;
                hisauchi.rebuild_with_document(&bytes, &document)?
            }
            SOURCE_KIND_ENDING => {
                let ending = parse_ending(&bytes)
                    .map_err(|error| format!("{}: {error}", source_path.display()))?;
                ending.rebuild_with_document(&bytes, &document)?
            }
            SOURCE_KIND_INTER1 => {
                let inter = parse_inter1(&bytes)
                    .map_err(|error| format!("{}: {error}", source_path.display()))?;
                inter.rebuild_with_document(&bytes, &document)?
            }
            SOURCE_KIND_INTER2 => {
                let inter = parse_inter2(&bytes)
                    .map_err(|error| format!("{}: {error}", source_path.display()))?;
                inter.rebuild_with_document(&bytes, &document)?
            }
            SOURCE_KIND_BUNSYO => {
                let bunsyo = parse_bunsyo(&bytes)
                    .map_err(|error| format!("{}: {error}", source_path.display()))?;
                bunsyo.rebuild_with_document(&bytes, &document)?
            }
            _ => unreachable!("validated source kind"),
        };
        total_entries += document.entries.len();
        total_patched += patched;
        total_unchanged += unchanged;
        prepared.push(PreparedInjection {
            relative,
            rebuilt,
            patched,
            unchanged,
        });
    }

    if source_is_file {
        validate_output_file(output, overwrite)?;
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        }
        let staging = unique_staging(output, "area-file")?;
        let result = (|| {
            fs::write(&staging, &prepared[0].rebuilt)
                .map_err(|error| format!("cannot write {}: {error}", staging.display()))?;
            commit_file(&staging, output, overwrite)
        })();
        if result.is_err() && staging.exists() {
            let _ = fs::remove_file(&staging);
        }
        result?;
    } else {
        validate_output_dir(output, overwrite)?;
        let staging = unique_staging(output, "area-inject")?;
        let result = (|| {
            copy_tree(source, &staging)?;
            for item in &prepared {
                let target = safe_join(&staging, &item.relative)?;
                fs::write(&target, &item.rebuilt)
                    .map_err(|error| format!("cannot write {}: {error}", target.display()))?;
            }
            commit_directory(&staging, output, overwrite)
        })();
        if result.is_err() && staging.exists() {
            let _ = fs::remove_dir_all(&staging);
        }
        result?;
    }

    debug_assert_eq!(
        total_patched,
        prepared.iter().map(|item| item.patched).sum::<usize>()
    );
    debug_assert_eq!(
        total_unchanged,
        prepared.iter().map(|item| item.unchanged).sum::<usize>()
    );
    Ok(InjectReport {
        json_files: prepared.len(),
        json_entries: total_entries,
        patched: total_patched,
        unchanged: total_unchanged,
        rebuilt_files: prepared.len(),
        output: output.to_path_buf(),
    })
}

fn split_logical_parts(text: &str) -> Result<LogicalParts> {
    let mut quoted = Vec::new();
    let mut stack: Vec<char> = Vec::new();
    let mut quote_start = None;
    let mut opener_offset = None;
    let mut top_kind = None;
    for (offset, ch) in text.char_indices() {
        match ch {
            '「' | '『' => {
                if stack.is_empty() {
                    let mut start = offset;
                    if text[..offset].ends_with('○') {
                        start -= '○'.len_utf8();
                    }
                    quote_start = Some(start);
                    opener_offset = Some(offset);
                    top_kind = Some(if ch == '「' {
                        PartKind::Dialogue
                    } else {
                        PartKind::Monologue
                    });
                }
                stack.push(ch);
            }
            '」' | '』' => {
                let expected = if ch == '」' { '「' } else { '『' };
                let opener = stack.pop().ok_or_else(|| {
                    format!("unmatched closing quote {ch} at UTF-8 offset {offset}")
                })?;
                if opener != expected {
                    return Err(format!(
                        "mismatched quote {opener}...{ch} at UTF-8 offset {offset}"
                    ));
                }
                if stack.is_empty() {
                    let mut end = offset + ch.len_utf8();
                    if text[end..].starts_with('△') {
                        end += '△'.len_utf8();
                    }
                    quoted.push(PartSpan {
                        start: quote_start.take().expect("top-level quote start"),
                        end,
                        kind: top_kind.take().expect("top-level quote kind"),
                    });
                    opener_offset = None;
                }
            }
            _ => {}
        }
    }
    if let Some(opener) = stack.first().copied() {
        let start = quote_start.expect("unmatched top-level quote start");
        let opener_at = opener_offset.expect("unmatched top-level opener offset");
        let kind = top_kind.expect("unmatched top-level quote kind");
        let line_end = text[opener_at..]
            .find('\n')
            .map(|relative| opener_at + relative)
            .unwrap_or(text.len());

        let mut recovered = split_logical_parts(&text[..start])?;
        let raw = &text[start..line_end];
        let end = raw
            .rfind(|ch: char| !ch.is_whitespace())
            .map(|last| {
                start
                    + last
                    + raw[last..]
                        .chars()
                        .next()
                        .expect("last recovered quote char")
                        .len_utf8()
            })
            .unwrap_or(start);
        if end > start {
            recovered.spans.push(PartSpan { start, end, kind });
        }

        let tail_start = if line_end < text.len() {
            line_end + '\n'.len_utf8()
        } else {
            line_end
        };
        if tail_start < text.len() {
            let mut tail = split_logical_parts(&text[tail_start..])?;
            for span in &mut tail.spans {
                span.start += tail_start;
                span.end += tail_start;
            }
            recovered.spans.extend(tail.spans);
            recovered.warnings.extend(tail.warnings);
        }
        recovered.warnings.push(format!(
            "unmatched opening quote {opener} at UTF-8 offset {opener_at}; recovered through its first physical line"
        ));
        recovered.spans.sort_by_key(|span| span.start);
        return Ok(recovered);
    }

    let mut spans = Vec::new();
    let mut cursor = 0usize;
    for quote in quoted {
        push_trimmed_narration(text, cursor, quote.start, &mut spans);
        spans.push(quote.clone());
        cursor = quote.end;
    }
    push_trimmed_narration(text, cursor, text.len(), &mut spans);
    Ok(LogicalParts {
        spans,
        warnings: Vec::new(),
    })
}

fn push_trimmed_narration(text: &str, start: usize, end: usize, spans: &mut Vec<PartSpan>) {
    let raw = &text[start..end];
    let Some(first) = raw.find(|ch: char| !ch.is_whitespace()) else {
        return;
    };
    let last = raw
        .rfind(|ch: char| !ch.is_whitespace())
        .expect("non-whitespace narration");
    let last_end = last + raw[last..].chars().next().expect("last char").len_utf8();
    spans.push(PartSpan {
        start: start + first,
        end: start + last_end,
        kind: PartKind::Narration,
    });
}

fn remove_layout_line_breaks(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\n' {
            while characters.next_if_eq(&'　').is_some() {}
        } else {
            output.push(character);
        }
    }
    output
}

pub fn prepare_message(message: &str) -> Result<String> {
    validate_message_structure(message)?;
    let projected = project_translation(message)?;
    encode_cp932(&projected)?;
    Ok(projected)
}

fn validate_message(message: &str) -> Result<()> {
    prepare_message(message).map(|_| ())
}

fn validate_message_structure(message: &str) -> Result<()> {
    if message.contains('\0') {
        return Err("message contains NUL".to_string());
    }
    if message.contains('\r') {
        return Err("message contains CR; use LF only".to_string());
    }
    let mut control_open = false;
    for ch in message.chars() {
        match ch {
            '○' if control_open => return Err("nested ○ control".to_string()),
            '○' => control_open = true,
            '△' if !control_open => return Err("△ control without preceding ○".to_string()),
            '△' => control_open = false,
            _ => {}
        }
    }
    if control_open {
        return Err("unclosed ○ control".to_string());
    }
    Ok(())
}

fn project_translation(text: &str) -> Result<String> {
    let mapping = built_in_glyph_mapping()?;
    Ok(text
        .chars()
        .map(|character| mapping.get(&character).copied().unwrap_or(character))
        .collect())
}

fn built_in_glyph_mapping() -> Result<&'static HashMap<char, char>> {
    static MAPPING: OnceLock<std::result::Result<HashMap<char, char>, String>> = OnceLock::new();
    MAPPING
        .get_or_init(|| {
            let raw: HashMap<String, String> =
                serde_json::from_str(include_str!("../assets/subs_cn_jp.json"))
                    .map_err(|error| format!("embedded CN/JP mapping is invalid: {error}"))?;
            let mut mapping = HashMap::with_capacity(raw.len());
            let mut carriers = HashSet::with_capacity(raw.len());
            for (target, carrier) in raw {
                let mut target_chars = target.chars();
                let target = target_chars
                    .next()
                    .ok_or_else(|| "embedded mapping contains an empty target".to_string())?;
                if target_chars.next().is_some() {
                    return Err("embedded mapping target is not one Unicode scalar".to_string());
                }
                let mut carrier_chars = carrier.chars();
                let carrier = carrier_chars
                    .next()
                    .ok_or_else(|| "embedded mapping contains an empty carrier".to_string())?;
                if carrier_chars.next().is_some() {
                    return Err(format!(
                        "embedded mapping carrier for {target:?} is not one Unicode scalar"
                    ));
                }
                let encoded = encode_cp932(&carrier.to_string()).map_err(|error| {
                    format!("embedded mapping carrier {carrier:?} is not CP932: {error}")
                })?;
                if encoded.len() != 2 {
                    return Err(format!(
                        "embedded mapping carrier {carrier:?} is not a double-byte CP932 glyph"
                    ));
                }
                if !carriers.insert(carrier) {
                    return Err(format!(
                        "embedded mapping reuses carrier character {carrier:?}"
                    ));
                }
                mapping.insert(target, carrier);
            }
            Ok(mapping)
        })
        .as_ref()
        .map_err(Clone::clone)
}

fn validate_choice(message: &str) -> Result<()> {
    validate_message(message)?;
    if message.contains('\n') {
        return Err("choice contains LF".to_string());
    }
    Ok(())
}

fn visible_cp932_len(text: &str) -> Result<usize> {
    let visible = text.replace(['○', '△'], "");
    Ok(encode_cp932(&visible)?.len())
}

fn encode_cp932(text: &str) -> Result<Vec<u8>> {
    let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        let unsupported: Vec<String> = text
            .chars()
            .filter(|ch| {
                let (_, _, error) = SHIFT_JIS.encode(&ch.to_string());
                error
            })
            .map(|ch| format!("{ch} (U+{:04X})", u32::from(ch)))
            .collect();
        return Err(format!(
            "text is not encodable as CP932: {}",
            unsupported.join(", ")
        ));
    }
    Ok(encoded.into_owned())
}

fn decode_cp932(bytes: &[u8]) -> Result<String> {
    SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(bytes)
        .map(|text| text.into_owned())
        .ok_or_else(|| "invalid CP932 byte sequence".to_string())
}

fn validate_document_header(document: &TranslationDocument) -> Result<()> {
    if document._format != JSON_FORMAT {
        return Err(format!("unsupported JSON _format: {}", document._format));
    }
    let expected_line_limit = match document._source_kind.as_str() {
        SOURCE_KIND_AREA => VISIBLE_LINE_BYTE_LIMIT,
        SOURCE_KIND_GEN => GEN_LINE_BYTE_LIMIT,
        SOURCE_KIND_MES | SOURCE_KIND_MAIN | SOURCE_KIND_OPENING | SOURCE_KIND_ENDING
        | SOURCE_KIND_HISAUCHI => VISIBLE_LINE_BYTE_LIMIT,
        SOURCE_KIND_INTER1 | SOURCE_KIND_INTER2 => INTER_LINE_BYTE_LIMIT,
        SOURCE_KIND_BUNSYO => BUNSYO_LINE_BYTE_LIMIT,
        kind => return Err(format!("unsupported JSON _source_kind: {kind}")),
    };
    if document._line_visible_byte_limit != expected_line_limit {
        return Err(format!(
            "JSON line limit {} does not match tool limit {}",
            document._line_visible_byte_limit, expected_line_limit
        ));
    }
    normalize_manifest_path(&document._source_file)?;
    Ok(())
}

fn collect_text_sources(input: &Path) -> Result<Vec<TextSource>> {
    let metadata = input
        .metadata()
        .map_err(|error| format!("cannot inspect {}: {error}", input.display()))?;
    if metadata.is_file() {
        let bytes =
            fs::read(input).map_err(|error| format!("cannot read {}: {error}", input.display()))?;
        let source = parse_supported_source(&bytes)
            .ok_or_else(|| format!("{} is not a supported game text source", input.display()))?;
        let relative = input
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("non-Unicode file name: {}", input.display()))?
            .to_string();
        return Ok(vec![(input.to_path_buf(), relative, bytes, source)]);
    }
    if !metadata.is_dir() {
        return Err(format!(
            "input is not a file or directory: {}",
            input.display()
        ));
    }

    let mut files = Vec::new();
    walk_files(input, input, &mut files)?;
    let mut sources = Vec::new();
    for (path, relative) in files {
        let bytes =
            fs::read(&path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        if let Some(source) = parse_supported_source(&bytes) {
            sources.push((path, relative, bytes, source));
        }
    }
    sources.sort_by(|left, right| left.1.cmp(&right.1));
    Ok(sources)
}

fn flat_json_name(relative: &str, duplicate_basename: bool) -> Result<String> {
    let normalized = normalize_manifest_path(relative)?;
    let basename = Path::new(&normalized)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("invalid source basename: {relative}"))?;
    if !duplicate_basename {
        return Ok(format!("{basename}.json"));
    }
    let parent = normalized
        .rsplit_once('/')
        .map(|(parent, _)| parent)
        .filter(|parent| !parent.is_empty())
        .unwrap_or("root");
    let flattened: String = parent
        .chars()
        .flat_map(|character| {
            if character == '/' || character == '\\' {
                "__".chars().collect::<Vec<_>>()
            } else if character.is_control()
                || matches!(character, '<' | '>' | ':' | '"' | '|' | '?' | '*')
            {
                "_".chars().collect()
            } else {
                vec![character]
            }
        })
        .take(80)
        .collect();
    let hash = sha256_hex(normalized.as_bytes());
    Ok(format!("{flattened}__{}__{basename}.json", &hash[..12]))
}

fn parse_supported_source(bytes: &[u8]) -> Option<ParsedSource> {
    if let Ok(area) = parse_area(bytes) {
        Some(ParsedSource::Area(area))
    } else if let Ok(gen) = parse_gen(bytes) {
        Some(ParsedSource::Gen(gen))
    } else if let Ok(mes) = parse_mes(bytes) {
        Some(ParsedSource::Mes(mes))
    } else if let Ok(main) = parse_main(bytes) {
        Some(ParsedSource::FixedExe(main))
    } else if let Ok(opening) = parse_opening(bytes) {
        Some(ParsedSource::FixedExe(opening))
    } else if let Ok(hisauchi) = parse_hisauchi(bytes) {
        Some(ParsedSource::FixedExe(hisauchi))
    } else if let Ok(ending) = parse_ending(bytes) {
        Some(ParsedSource::FixedExe(ending))
    } else if let Ok(inter1) = parse_inter1(bytes) {
        Some(ParsedSource::Inter(inter1))
    } else if let Ok(inter2) = parse_inter2(bytes) {
        Some(ParsedSource::Inter(inter2))
    } else if let Ok(bunsyo) = parse_bunsyo(bytes) {
        Some(ParsedSource::Bunsyo(bunsyo))
    } else {
        None
    }
}

fn load_translation_documents(path: &Path) -> Result<Vec<TranslationDocument>> {
    let metadata = path
        .metadata()
        .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
    let mut paths = Vec::new();
    if metadata.is_file() {
        paths.push(path.to_path_buf());
    } else if metadata.is_dir() {
        let mut files = Vec::new();
        walk_files(path, path, &mut files)?;
        paths.extend(files.into_iter().map(|(file, _)| file).filter(|file| {
            file.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("json"))
                .unwrap_or(false)
        }));
    } else {
        return Err(format!(
            "translation path is not a file or directory: {}",
            path.display()
        ));
    }
    paths.sort();

    let mut documents = Vec::new();
    let mut seen_sources = HashSet::new();
    for file in paths {
        let bytes =
            fs::read(&file).map_err(|error| format!("cannot read {}: {error}", file.display()))?;
        let document = serde_json::from_slice::<TranslationDocument>(&bytes)
            .map_err(|error| format!("invalid translation JSON {}: {error}", file.display()))?;
        if document._format != JSON_FORMAT {
            return Err(format!(
                "unsupported translation format {:?} in {}; expected {JSON_FORMAT}",
                document._format,
                file.display()
            ));
        }
        let relative = normalize_manifest_path(&document._source_file)?;
        if !seen_sources.insert(relative.clone()) {
            return Err(format!("duplicate translation source: {relative}"));
        }
        documents.push(document);
    }
    Ok(documents)
}

fn walk_files(root: &Path, current: &Path, output: &mut Vec<(PathBuf, String)>) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(current)
        .map_err(|error| format!("cannot read {}: {error}", current.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot enumerate {}: {error}", current.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_type = entry
            .file_type()
            .map_err(|error| format!("cannot inspect {}: {error}", entry.path().display()))?;
        if file_type.is_symlink() {
            return Err(format!(
                "symbolic links are not supported: {}",
                entry.path().display()
            ));
        }
        if file_type.is_dir() {
            walk_files(root, &entry.path(), output)?;
        } else if file_type.is_file() {
            let relative = entry
                .path()
                .strip_prefix(root)
                .map_err(|_| "internal relative path error".to_string())?
                .components()
                .map(|component| match component {
                    Component::Normal(value) => value
                        .to_str()
                        .map(str::to_string)
                        .ok_or_else(|| "non-Unicode path component".to_string()),
                    _ => Err("unsafe relative path component".to_string()),
                })
                .collect::<Result<Vec<_>>>()?
                .join("/");
            output.push((entry.path(), relative));
        }
    }
    Ok(())
}

fn count_files(input: &Path) -> Result<usize> {
    if input.is_file() {
        return Ok(1);
    }
    let mut files = Vec::new();
    walk_files(input, input, &mut files)?;
    Ok(files.len())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    if destination.exists() {
        return Err(format!(
            "staging path already exists: {}",
            destination.display()
        ));
    }
    fs::create_dir(destination)
        .map_err(|error| format!("cannot create {}: {error}", destination.display()))?;
    let mut files = Vec::new();
    walk_files(source, source, &mut files)?;
    for (file, relative) in files {
        let target = safe_join(destination, &relative)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        }
        fs::copy(&file, &target).map_err(|error| {
            format!(
                "cannot copy {} to {}: {error}",
                file.display(),
                target.display()
            )
        })?;
    }
    Ok(())
}

fn validate_output_dir(output: &Path, overwrite: bool) -> Result<()> {
    validate_output_path(output)?;
    if output.exists() && !overwrite {
        return Err(format!(
            "output already exists; choose another path or use --overwrite: {}",
            output.display()
        ));
    }
    if output.exists() && !output.is_dir() {
        return Err(format!("output is not a directory: {}", output.display()));
    }
    Ok(())
}

fn validate_output_file(output: &Path, overwrite: bool) -> Result<()> {
    validate_output_path(output)?;
    if output.exists() && !overwrite {
        return Err(format!(
            "output already exists; choose another path or use --overwrite: {}",
            output.display()
        ));
    }
    if output.exists() && !output.is_file() {
        return Err(format!("output is not a file: {}", output.display()));
    }
    Ok(())
}

fn validate_output_path(output: &Path) -> Result<()> {
    if output.as_os_str().is_empty() {
        return Err("output path is empty".to_string());
    }
    if output
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err("output path must not contain ..".to_string());
    }
    if output.parent().is_none() {
        return Err(format!(
            "refusing filesystem root output: {}",
            output.display()
        ));
    }
    Ok(())
}

fn validate_disjoint_output(output: &Path, inputs: &[(&str, &Path)]) -> Result<()> {
    let output_components = comparable_path_components(output)?;
    for (role, input) in inputs {
        let input_components = comparable_path_components(input)?;
        if component_prefix(&output_components, &input_components)
            || component_prefix(&input_components, &output_components)
        {
            return Err(format!(
                "output must not overlap {role}: output={} {role}={}",
                output.display(),
                input.display()
            ));
        }
    }
    Ok(())
}

fn comparable_path_components(path: &Path) -> Result<Vec<String>> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| format!("cannot resolve current directory: {error}"))?
            .join(path)
    };

    let mut existing = absolute.as_path();
    let mut missing = Vec::new();
    while !existing.exists() {
        let name = existing.file_name().ok_or_else(|| {
            format!(
                "cannot resolve path for overlap validation: {}",
                path.display()
            )
        })?;
        missing.push(name.to_os_string());
        existing = existing.parent().ok_or_else(|| {
            format!(
                "cannot resolve path for overlap validation: {}",
                path.display()
            )
        })?;
    }

    let mut resolved = fs::canonicalize(existing)
        .map_err(|error| format!("cannot resolve {}: {error}", existing.display()))?;
    for component in missing.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
        .collect())
}

fn component_prefix(prefix: &[String], path: &[String]) -> bool {
    prefix.len() <= path.len() && prefix.iter().zip(path).all(|(left, right)| left == right)
}

fn unique_staging(output: &Path, label: &str) -> Result<PathBuf> {
    let parent = output
        .parent()
        .ok_or_else(|| format!("output has no parent: {}", output.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    for suffix in 0..1000usize {
        let candidate = parent.join(format!(".{label}-staging-{}-{suffix}", std::process::id()));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "cannot allocate staging path under {}",
        parent.display()
    ))
}

fn commit_file(staging: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if !output.exists() {
        return fs::rename(staging, output).map_err(|error| {
            format!(
                "cannot commit {} to {}: {error}",
                staging.display(),
                output.display()
            )
        });
    }
    if !overwrite {
        return Err(format!("output already exists: {}", output.display()));
    }
    let parent = output
        .parent()
        .ok_or_else(|| format!("output has no parent: {}", output.display()))?;
    let backup = parent.join(format!(
        ".area-file-backup-{}-{}",
        std::process::id(),
        output
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("output")
    ));
    if backup.exists() {
        return Err(format!("backup path already exists: {}", backup.display()));
    }
    fs::rename(output, &backup).map_err(|error| format!("cannot create output backup: {error}"))?;
    if let Err(error) = fs::rename(staging, output) {
        let rollback = fs::rename(&backup, output);
        return match rollback {
            Ok(()) => Err(format!("commit failed; previous output restored: {error}")),
            Err(rollback_error) => Err(format!(
                "commit failed and rollback failed; backup is {}: {error}; {rollback_error}",
                backup.display()
            )),
        };
    }
    fs::remove_file(&backup)
        .map_err(|error| format!("new output committed but backup cleanup failed: {error}"))?;
    Ok(())
}

fn commit_directory(staging: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if !output.exists() {
        return fs::rename(staging, output).map_err(|error| {
            format!(
                "cannot commit {} to {}: {error}",
                staging.display(),
                output.display()
            )
        });
    }
    if !overwrite {
        return Err(format!("output already exists: {}", output.display()));
    }
    let parent = output
        .parent()
        .ok_or_else(|| format!("output has no parent: {}", output.display()))?;
    let backup = parent.join(format!(
        ".area-backup-{}-{}",
        std::process::id(),
        output
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("output")
    ));
    if backup.exists() {
        return Err(format!("backup path already exists: {}", backup.display()));
    }
    fs::rename(output, &backup).map_err(|error| format!("cannot create output backup: {error}"))?;
    if let Err(error) = fs::rename(staging, output) {
        let rollback = fs::rename(&backup, output);
        return match rollback {
            Ok(()) => Err(format!("commit failed; previous output restored: {error}")),
            Err(rollback_error) => Err(format!(
                "commit failed and rollback failed; backup is {}: {error}; {rollback_error}",
                backup.display()
            )),
        };
    }
    fs::remove_dir_all(&backup)
        .map_err(|error| format!("new output committed but backup cleanup failed: {error}"))?;
    Ok(())
}

fn safe_join(root: &Path, relative: &str) -> Result<PathBuf> {
    let normalized = normalize_manifest_path(relative)?;
    let mut output = root.to_path_buf();
    for component in normalized.split('/') {
        output.push(component);
    }
    Ok(output)
}

fn normalize_manifest_path(relative: &str) -> Result<String> {
    if relative.is_empty() || relative.starts_with('/') || relative.starts_with('\\') {
        return Err(format!("unsafe relative path: {relative:?}"));
    }
    let mut normalized = Vec::new();
    for component in relative.replace('\\', "/").split('/') {
        if component.is_empty() || component == "." || component == ".." {
            return Err(format!("unsafe relative path component: {relative:?}"));
        }
        if component.contains(':') || component.contains('\0') {
            return Err(format!("unsafe relative path component: {relative:?}"));
        }
        normalized.push(component.to_string());
    }
    Ok(normalized.join("/"))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let slice = bytes
        .get(offset..offset.saturating_add(2))
        .ok_or_else(|| format!("u16 read at 0x{offset:X} is out of bounds"))?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02X}").expect("write to string");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    type TestChoice<'a> = (&'a str, Option<usize>);
    type TestChoiceTable<'a> = (usize, &'a [TestChoice<'a>]);

    fn unique_test_dir(label: &str) -> PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        std::env::temp_dir().join(format!(
            "aitsuno-area-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn make_area(texts: &[(&str, &[u8])]) -> Vec<u8> {
        make_area_with_choices(texts, &[])
    }

    fn make_area_with_choices(
        texts: &[(&str, &[u8])],
        choices: &[(&str, Option<usize>)],
    ) -> Vec<u8> {
        make_area_with_choice_tables(texts, &[(14, choices)])
    }

    fn make_area_with_choice_tables(
        texts: &[(&str, &[u8])],
        choice_tables: &[TestChoiceTable<'_>],
    ) -> Vec<u8> {
        let mut counts = [0u16; 22];
        counts[0] = 2;
        let mut sections = vec![Vec::new(); 22];
        sections[0] = vec![1, 0, 2, 0];
        for &(offset_section, choices) in choice_tables {
            let pool_section = offset_section + 1;
            counts[offset_section] = choices.len() as u16;
            let mut choice_pool = Vec::new();
            let mut choice_offsets = Vec::new();
            for (index, (text, alias_of)) in choices.iter().enumerate() {
                if let Some(alias_of) = alias_of {
                    assert!(*alias_of < index);
                    choice_offsets.push(choice_offsets[*alias_of]);
                } else {
                    choice_offsets.push(choice_pool.len() as u16);
                    choice_pool.extend_from_slice(&encode_cp932(text).expect("encode choice"));
                    choice_pool.push(0);
                }
            }
            counts[pool_section] = choice_pool.len() as u16;
            for offset in choice_offsets {
                sections[offset_section].extend_from_slice(&offset.to_le_bytes());
            }
            sections[pool_section] = choice_pool;
        }
        counts[20] = texts.len() as u16;
        let mut pool = Vec::new();
        let mut offsets = Vec::new();
        for (text, suffix) in texts {
            offsets.push(pool.len() as u16);
            pool.extend_from_slice(&encode_cp932(text).expect("encode"));
            pool.push(0);
            pool.extend_from_slice(suffix);
        }
        counts[21] = pool.len() as u16;
        for offset in offsets {
            sections[20].extend_from_slice(&offset.to_le_bytes());
        }
        sections[21] = pool;
        let mut bytes = Vec::new();
        for count in counts {
            bytes.extend_from_slice(&count.to_le_bytes());
        }
        for section in sections {
            bytes.extend_from_slice(&section);
        }
        bytes
    }

    #[test]
    fn parses_sections_and_preserves_unchanged_bytes() {
        let source = make_area(&[("「短い文」", &[]), ("○『独白』△", &[0xAA, 0x55])]);
        let area = parse_area(&source).expect("parse");
        let document = TranslationDocument {
            _format: JSON_FORMAT.to_string(),
            _source_kind: SOURCE_KIND_AREA.to_string(),
            _source_file: "sample.bin".to_string(),
            _source_sha256: sha256_hex(&source),
            _line_visible_byte_limit: VISIBLE_LINE_BYTE_LIMIT,
            entries: area.translation_entries().expect("entries"),
        };
        let (rebuilt, patched, unchanged) = area
            .rebuild_with_document(&document)
            .expect("unchanged rebuild");
        assert_eq!(rebuilt, source);
        assert_eq!(patched, 0);
        assert_eq!(unchanged, 2);
    }

    #[test]
    fn combines_physical_lines_inside_matching_quotes() {
        let source = make_area(&[(
            "「そうですか？あなたのやっていることは健康法にはほど遠いよう\n　ですね」",
            &[],
        )]);
        let area = parse_area(&source).expect("parse");
        let entries = area.translation_entries().expect("entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].scr_msg,
            "「そうですか？あなたのやっていることは健康法にはほど遠いようですね」"
        );
    }

    #[test]
    fn removes_short_line_break_inside_matching_quotes() {
        let source = make_area(&[("『短い段落。\n　次の短い段落。』", &[])]);
        let area = parse_area(&source).expect("parse");
        let entries = area.translation_entries().expect("entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].scr_msg, "『短い段落。次の短い段落。』");
    }

    #[test]
    fn removes_all_layout_indent_after_soft_wrap() {
        let first_line = format!("『　“{}", "文".repeat(27));
        assert_eq!(visible_cp932_len(&first_line).expect("line width"), 60);
        let source_text = format!("{first_line}\n　　続き”』");
        let source = make_area(&[(&source_text, &[])]);
        let area = parse_area(&source).expect("parse");
        let entries = area.translation_entries().expect("entries");
        assert_eq!(entries[0].scr_msg, format!("{first_line}続き”』"));
    }

    #[test]
    fn recovers_unmatched_opening_quote_without_swallowing_following_parts() {
        let source = make_area(&[(
            "「前置き。\n『短い物語。』\n\nどうしますか？\n１，続ける",
            &[],
        )]);
        let area = parse_area(&source).expect("parse");
        let (entries, warnings) = area.translation_entries_with_warnings().expect("entries");
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].scr_msg, "「前置き。");
        assert_eq!(entries[1].scr_msg, "『短い物語。』");
        assert_eq!(entries[2].scr_msg, "どうしますか？１，続ける");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("unmatched opening quote 「"));
    }

    #[test]
    fn separates_multiple_quote_pairs_without_names() {
        let source = make_area(&[("○「質問ですか？」△\n「回答です」", &[])]);
        let area = parse_area(&source).expect("parse");
        let entries = area.translation_entries().expect("entries");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].scr_msg, "○「質問ですか？」△");
        assert_eq!(entries[1].scr_msg, "「回答です」");
        assert!(entries.iter().all(|entry| !entry._type.is_empty()));
    }

    #[test]
    fn leaves_runtime_soft_wrapping_to_the_game() {
        let message = format!("「{}」", "テ".repeat(40));
        assert!(visible_cp932_len(&message).expect("message width") > 62);
        assert_eq!(prepare_message(&message).expect("prepare"), message);
    }

    #[test]
    fn preserves_controls_and_explicit_hard_line_breaks() {
        let message = "○「みじかい。\n　つぎ。」△";
        assert_eq!(prepare_message(message).expect("prepare"), message);
    }

    #[test]
    fn projects_simplified_chinese_to_embedded_cp932_carriers() {
        let prepared = prepare_message("黑赶").expect("prepare mapped Chinese");
        assert_eq!(prepared, "黒骭");
        assert_eq!(
            encode_cp932(&prepared).expect("encode"),
            [0x8D, 0x95, 0xE9, 0x8C]
        );
        assert_eq!(built_in_glyph_mapping().expect("mapping").len(), 3025);
    }

    #[test]
    fn projects_chinese_during_message_and_choice_rebuild() {
        let source = make_area_with_choices(&[("原文", &[])], &[("選択", None)]);
        let area = parse_area(&source).expect("parse");
        let mut document = TranslationDocument {
            _format: JSON_FORMAT.to_string(),
            _source_kind: SOURCE_KIND_AREA.to_string(),
            _source_file: "sample.bin".to_string(),
            _source_sha256: sha256_hex(&source),
            _line_visible_byte_limit: VISIBLE_LINE_BYTE_LIMIT,
            entries: area.translation_entries().expect("entries"),
        };
        document
            .entries
            .iter_mut()
            .find(|entry| entry._choice_index.is_none())
            .expect("message")
            .message = "黑赶".to_string();
        document
            .entries
            .iter_mut()
            .find(|entry| entry._choice_index == Some(0))
            .expect("choice")
            .message = "黑赶".to_string();

        let (rebuilt, patched, unchanged) = area
            .rebuild_with_document(&document)
            .expect("rebuild with carrier projection");
        assert_eq!(patched, 2);
        assert_eq!(unchanged, 0);
        let reparsed = parse_area(&rebuilt).expect("reparse");
        assert_eq!(reparsed.slots[0].text, "黒骭");
        assert_eq!(reparsed.choice_tables[0].choices[0].text, "黒骭");
    }

    #[test]
    fn rejects_output_paths_that_overlap_inputs() {
        let root = unique_test_dir("overlap");
        let source = root.join("source");
        let translation = root.join("translation");
        fs::create_dir_all(&source).expect("create source");
        fs::create_dir_all(&translation).expect("create translation");

        assert!(validate_disjoint_output(&source, &[("source", &source)]).is_err());
        assert!(validate_disjoint_output(&source.join("nested"), &[("source", &source)]).is_err());
        assert!(validate_disjoint_output(&root, &[("source", &source)]).is_err());
        assert!(validate_disjoint_output(
            &translation,
            &[("source", &source), ("translation", &translation)]
        )
        .is_err());
        assert!(validate_disjoint_output(
            &root.join("output"),
            &[("source", &source), ("translation", &translation)]
        )
        .is_ok());

        fs::remove_dir_all(&root).expect("remove test directory");
    }

    #[test]
    fn rejects_damaged_json_in_translation_directory() {
        let root = unique_test_dir("damaged-json");
        fs::create_dir_all(&root).expect("create translation directory");
        let broken = root.join("broken.JSON");
        fs::write(&broken, b"{not-json").expect("write broken JSON");

        let error = load_translation_documents(&root).expect_err("damaged JSON must fail");
        assert!(error.contains("invalid translation JSON"));
        assert!(error.contains("broken.JSON"));

        fs::remove_dir_all(&root).expect("remove test directory");
    }

    #[test]
    fn changed_message_rebuilds_offsets_and_preserves_suffix() {
        let source = make_area(&[("「一」", &[0x11, 0x22]), ("「二」", &[0x33])]);
        let area = parse_area(&source).expect("parse");
        let mut document = TranslationDocument {
            _format: JSON_FORMAT.to_string(),
            _source_kind: SOURCE_KIND_AREA.to_string(),
            _source_file: "sample.bin".to_string(),
            _source_sha256: sha256_hex(&source),
            _line_visible_byte_limit: VISIBLE_LINE_BYTE_LIMIT,
            entries: area.translation_entries().expect("entries"),
        };
        document.entries[0].message = "「テストテストテスト」".to_string();
        let (rebuilt, patched, unchanged) = area
            .rebuild_with_document(&document)
            .expect("changed rebuild");
        assert_eq!(patched, 1);
        assert_eq!(unchanged, 1);
        let reparsed = parse_area(&rebuilt).expect("reparse");
        assert_eq!(reparsed.slots[0].suffix, vec![0x11, 0x22]);
        assert_eq!(reparsed.slots[1].suffix, vec![0x33]);
        assert_eq!(
            remove_layout_line_breaks(&reparsed.slots[0].text),
            document.entries[0].message
        );
    }

    #[test]
    fn choice_aliases_round_trip_and_split_only_when_translations_diverge() {
        let source = make_area_with_choices(
            &[("「本文」", &[])],
            &[("共有", None), ("共有", Some(0)), ("別", None)],
        );
        let area = parse_area(&source).expect("parse");
        assert_eq!(
            area.choice_tables[0].choices[0].pool_offset,
            area.choice_tables[0].choices[1].pool_offset
        );
        let mut document = TranslationDocument {
            _format: JSON_FORMAT.to_string(),
            _source_kind: SOURCE_KIND_AREA.to_string(),
            _source_file: "sample.bin".to_string(),
            _source_sha256: sha256_hex(&source),
            _line_visible_byte_limit: VISIBLE_LINE_BYTE_LIMIT,
            entries: area.translation_entries().expect("entries"),
        };
        assert!(document
            .entries
            .iter()
            .all(|entry| entry._file == "sample.bin"));

        let (unchanged_bytes, patched, unchanged) = area
            .rebuild_with_document(&document)
            .expect("unchanged rebuild");
        assert_eq!(unchanged_bytes, source);
        assert_eq!(patched, 0);
        assert_eq!(unchanged, 4);

        let alias_entry = document
            .entries
            .iter_mut()
            .find(|entry| entry._choice_index == Some(1))
            .expect("aliased choice entry");
        alias_entry.message = "テスト".to_string();
        let (rebuilt, patched, unchanged) = area
            .rebuild_with_document(&document)
            .expect("changed alias rebuild");
        assert_eq!(patched, 1);
        assert_eq!(unchanged, 3);
        let reparsed = parse_area(&rebuilt).expect("reparse");
        assert_eq!(reparsed.choice_tables[0].choices[0].text, "共有");
        assert_eq!(reparsed.choice_tables[0].choices[1].text, "テスト");
        assert_ne!(
            reparsed.choice_tables[0].choices[0].pool_offset,
            reparsed.choice_tables[0].choices[1].pool_offset
        );
    }

    #[test]
    fn extracts_and_rebuilds_all_area_choice_tables() {
        let table_10 = [("十", None)];
        let table_12 = [("十二", None)];
        let table_14 = [("十四", None)];
        let table_16 = [("十六", None)];
        let table_18 = [("十八", None)];
        let source = make_area_with_choice_tables(
            &[("「本文」", &[])],
            &[
                (10, &table_10),
                (12, &table_12),
                (14, &table_14),
                (16, &table_16),
                (18, &table_18),
            ],
        );
        let area = parse_area(&source).expect("parse");
        assert_eq!(area.source_choices(), 5);
        let mut document = TranslationDocument {
            _format: JSON_FORMAT.to_string(),
            _source_kind: SOURCE_KIND_AREA.to_string(),
            _source_file: "sample.bin".to_string(),
            _source_sha256: sha256_hex(&source),
            _line_visible_byte_limit: VISIBLE_LINE_BYTE_LIMIT,
            entries: area.translation_entries().expect("entries"),
        };
        let extracted_choices: Vec<_> = document
            .entries
            .iter()
            .filter(|entry| entry._choice_index.is_some())
            .map(|entry| entry.scr_msg.as_str())
            .collect();
        assert_eq!(extracted_choices, ["十四", "十", "十二", "十六", "十八"]);

        let (unchanged_bytes, patched, unchanged) = area
            .rebuild_with_document(&document)
            .expect("unchanged rebuild");
        assert_eq!(unchanged_bytes, source);
        assert_eq!(patched, 0);
        assert_eq!(unchanged, 6);

        for (choice_index, translated) in ["主表", "目的", "場所", "対象", "物品"]
            .into_iter()
            .enumerate()
        {
            document
                .entries
                .iter_mut()
                .find(|entry| entry._choice_index == Some(choice_index))
                .expect("choice entry")
                .message = translated.to_string();
        }
        let (rebuilt, patched, unchanged) = area
            .rebuild_with_document(&document)
            .expect("changed rebuild");
        assert_eq!(patched, 5);
        assert_eq!(unchanged, 1);
        let reparsed = parse_area(&rebuilt).expect("reparse");
        assert_eq!(reparsed.choice_tables[0].choices[0].text, "主表");
        assert_eq!(reparsed.choice_tables[1].choices[0].text, "目的");
        assert_eq!(reparsed.choice_tables[2].choices[0].text, "場所");
        assert_eq!(reparsed.choice_tables[3].choices[0].text, "対象");
        assert_eq!(reparsed.choice_tables[4].choices[0].text, "物品");
    }

    #[test]
    fn preserves_empty_area_choice_table_sentinels_without_exporting_them() {
        let mut source = make_area(&[("「本文」", &[])]);
        source[18 * 2..18 * 2 + 2].copy_from_slice(&2u16.to_le_bytes());
        source.splice(48..48, [0, 0, 0, 0]);
        let area = parse_area(&source).expect("parse empty sentinel table");
        assert_eq!(area.source_choices(), 0);
        let document = TranslationDocument {
            _format: JSON_FORMAT.to_string(),
            _source_kind: SOURCE_KIND_AREA.to_string(),
            _source_file: "sample.bin".to_string(),
            _source_sha256: sha256_hex(&source),
            _line_visible_byte_limit: VISIBLE_LINE_BYTE_LIMIT,
            entries: area.translation_entries().expect("entries"),
        };
        let (rebuilt, patched, unchanged) = area
            .rebuild_with_document(&document)
            .expect("unchanged rebuild");
        assert_eq!(rebuilt, source);
        assert_eq!(patched, 0);
        assert_eq!(unchanged, 1);
    }

    #[test]
    fn rejects_line_break_in_choice() {
        let source = make_area_with_choices(&[("「本文」", &[])], &[("選択", None)]);
        let area = parse_area(&source).expect("parse");
        let mut document = TranslationDocument {
            _format: JSON_FORMAT.to_string(),
            _source_kind: SOURCE_KIND_AREA.to_string(),
            _source_file: "sample.bin".to_string(),
            _source_sha256: sha256_hex(&source),
            _line_visible_byte_limit: VISIBLE_LINE_BYTE_LIMIT,
            entries: area.translation_entries().expect("entries"),
        };
        document
            .entries
            .iter_mut()
            .find(|entry| entry._choice_index == Some(0))
            .expect("choice")
            .message = "改\n行".to_string();
        let error = area
            .rebuild_with_document(&document)
            .expect_err("choice LF should fail");
        assert!(error.contains("choice contains LF"));
    }

    #[test]
    fn rejects_unencodable_and_malformed_control_text() {
        assert!(validate_message("emoji 😀").is_err());
        assert!(validate_message("△text").is_err());
        assert!(validate_message("○text").is_err());
        assert!(validate_message("○text△").is_ok());
    }

    fn make_bunsyo() -> Vec<u8> {
        let record = build_bunsyo_record(
            "あなたの不幸はどちらか？",
            "絶対静止的不幸",
            "等速直線運動的不幸",
        )
        .expect("build BUNSYO record");
        let mut bytes = Vec::with_capacity(BUNSYO_RECORD_COUNT * BUNSYO_RECORD_SIZE + 1);
        for _ in 0..BUNSYO_RECORD_COUNT {
            bytes.extend_from_slice(&record);
        }
        bytes.push(0x1A);
        bytes
    }

    fn put_padded_slot(bytes: &mut [u8], start: usize, capacity: usize, text: &str) {
        let encoded = pad_fullwidth(encode_cp932(text).expect("encode test slot"), capacity)
            .expect("pad test slot");
        bytes[start..start + capacity].copy_from_slice(&encoded);
        bytes[start + capacity] = 0;
    }

    fn put_fixed_specs(bytes: &mut [u8], data_file_offset: usize, specs: &[FixedStringSpec]) {
        for spec in specs {
            let start = data_file_offset + spec.data_offset;
            bytes[start..start + spec.capacity + 1].fill(0);
            bytes[start..start + 2].copy_from_slice(&encode_cp932("文").expect("encode"));
        }
    }

    fn make_inter1() -> Vec<u8> {
        let mut bytes = vec![0; 35_000];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[0x08..0x0A].copy_from_slice(&0x00C0u16.to_le_bytes());
        let base = 0x7960;
        bytes[base + 0x0662..base + 0x0662 + 8].copy_from_slice(b"im1.dat\0");
        let starts = [0usize, 2, 8, 11];
        let mut cursor = INTER1_PACKED_POOL_START;
        for index in 0..15 {
            let first = starts.contains(&index);
            let text = if first { "※　説明" } else { "　　続" };
            let encoded = encode_cp932(text).expect("encode instruction");
            let pointer = base + INTER1_POINTER_TABLE + index * 4;
            bytes[pointer..pointer + 2]
                .copy_from_slice(&u16::try_from(cursor).expect("pointer").to_le_bytes());
            bytes[pointer + 2..pointer + 4].copy_from_slice(&INTER1_DATA_SEGMENT.to_le_bytes());
            let target = base + cursor;
            bytes[target..target + encoded.len()].copy_from_slice(&encoded);
            bytes[target + encoded.len()] = 0;
            cursor += encoded.len() + 1;
        }
        let paragraph_starts = [0usize, 2, 6, 9];
        for index in 0..11 {
            put_padded_slot(
                &mut bytes,
                base + 0x0374 + index * 59,
                58,
                if paragraph_starts.contains(&index) {
                    "　段落"
                } else {
                    "続き"
                },
            );
        }
        put_fixed_specs(&mut bytes, base, &INTER1_UI_SPECS);
        bytes
    }

    fn make_inter2() -> Vec<u8> {
        let mut bytes = vec![0; 37_438];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[0x08..0x0A].copy_from_slice(&0x00A0u16.to_le_bytes());
        let base = 0x6940;
        let intro_starts = [0usize, 6, 11, 19, 22, 30, 34];
        for index in 0..37 {
            put_padded_slot(
                &mut bytes,
                base + 0x004E + index * 59,
                58,
                if intro_starts.contains(&index) {
                    "　段落"
                } else {
                    "続き"
                },
            );
        }
        for index in 0..150 {
            put_padded_slot(&mut bytes, base + 0x0910 + index * 39, 38, "");
        }
        let mut groups = Vec::new();
        for &(start, end, style) in &[
            (0, 1, InterGroupStyle::Title),
            (2, 19, InterGroupStyle::Plain),
            (19, 30, InterGroupStyle::Plain),
            (32, 33, InterGroupStyle::Title),
            (34, 42, InterGroupStyle::Plain),
            (46, 48, InterGroupStyle::Plain),
            (48, 49, InterGroupStyle::Bullet),
            (49, 50, InterGroupStyle::Arrow),
            (50, 51, InterGroupStyle::Bullet),
            (51, 52, InterGroupStyle::Arrow),
            (52, 53, InterGroupStyle::Plain),
            (53, 54, InterGroupStyle::Bullet),
            (54, 55, InterGroupStyle::Arrow),
            (55, 56, InterGroupStyle::Bullet),
            (56, 57, InterGroupStyle::Arrow),
            (57, 58, InterGroupStyle::Bullet),
            (58, 59, InterGroupStyle::Arrow),
            (61, 63, InterGroupStyle::Plain),
            (63, 64, InterGroupStyle::Bullet),
            (64, 65, InterGroupStyle::Arrow),
            (65, 66, InterGroupStyle::Bullet),
            (66, 67, InterGroupStyle::Arrow),
            (67, 68, InterGroupStyle::Bullet),
            (68, 69, InterGroupStyle::Arrow),
            (69, 73, InterGroupStyle::Plain),
            (78, 79, InterGroupStyle::Bullet),
            (79, 81, InterGroupStyle::Arrow),
            (81, 82, InterGroupStyle::Bullet),
            (82, 83, InterGroupStyle::Arrow),
            (83, 84, InterGroupStyle::Bullet),
            (84, 86, InterGroupStyle::Arrow),
            (92, 93, InterGroupStyle::Title),
            (94, 103, InterGroupStyle::Plain),
            (109, 110, InterGroupStyle::Level("Ｄ→")),
            (110, 111, InterGroupStyle::Level("Ｃ→")),
            (111, 113, InterGroupStyle::Level("Ｂ→")),
            (113, 116, InterGroupStyle::Level("Ａ→")),
            (121, 122, InterGroupStyle::Title),
            (123, 133, InterGroupStyle::Plain),
            (138, 139, InterGroupStyle::Level("Ｄ→")),
            (139, 141, InterGroupStyle::Level("Ｃ→")),
            (141, 143, InterGroupStyle::Level("Ｂ→")),
            (143, 146, InterGroupStyle::Level("Ａ→")),
        ] {
            groups.push((start, end, style));
        }
        for (start, end, style) in groups {
            for index in start..end {
                let prefix = inter_group_prefix(style, index - start);
                put_padded_slot(
                    &mut bytes,
                    base + 0x0910 + index * 39,
                    38,
                    &format!("{prefix}文"),
                );
            }
        }
        bytes[base + 0x204B..base + 0x204B + 11].copy_from_slice(b"window.dat\0");
        put_fixed_specs(&mut bytes, base, &INTER2_UI_SPECS);
        bytes
    }

    #[test]
    fn bunsyo_extracts_and_reflows_fixed_records() {
        let source = make_bunsyo();
        let bunsyo = parse_bunsyo(&source).expect("parse BUNSYO");
        let mut document = TranslationDocument {
            _format: JSON_FORMAT.to_string(),
            _source_kind: SOURCE_KIND_BUNSYO.to_string(),
            _source_file: "GRAPH/BUNSYO.DAT".to_string(),
            _source_sha256: sha256_hex(&source),
            _line_visible_byte_limit: BUNSYO_LINE_BYTE_LIMIT,
            entries: bunsyo
                .translation_entries("GRAPH/BUNSYO.DAT")
                .expect("entries"),
        };
        assert_eq!(document.entries.len(), 285);
        assert_eq!(document.entries[0].scr_msg, "あなたの不幸はどちらか？");
        let (unchanged, patched, same) = bunsyo
            .rebuild_with_document(&source, &document)
            .expect("unchanged rebuild");
        assert_eq!(unchanged, source);
        assert_eq!((patched, same), (0, 285));

        document.entries[0].message = "テ".repeat(20);
        let (rebuilt, patched, same) = bunsyo
            .rebuild_with_document(&source, &document)
            .expect("changed rebuild");
        assert_eq!((patched, same), (1, 284));
        let reparsed = parse_bunsyo(&rebuilt).expect("reparse changed BUNSYO");
        assert_eq!(reparsed.records[0].prompt, "テ".repeat(20));

        document.entries[0].message = "テ".repeat(121);
        assert!(bunsyo.rebuild_with_document(&source, &document).is_err());
    }

    #[test]
    fn inter1_rebuilds_packed_pool_and_far_pointers() {
        let source = make_inter1();
        let inter = parse_inter1(&source).expect("parse INTER1");
        assert_eq!(inter.groups.len(), 8);
        assert_eq!(inter.fixed_strings.len(), 13);
        let mut document = TranslationDocument {
            _format: JSON_FORMAT.to_string(),
            _source_kind: SOURCE_KIND_INTER1.to_string(),
            _source_file: "INTER1.EXE".to_string(),
            _source_sha256: sha256_hex(&source),
            _line_visible_byte_limit: INTER_LINE_BYTE_LIMIT,
            entries: inter.translation_entries("INTER1.EXE").expect("entries"),
        };
        document.entries[0].message = "変更した説明".to_string();
        let (rebuilt, patched, unchanged) = inter
            .rebuild_with_document(&source, &document)
            .expect("rebuild INTER1");
        assert_eq!((patched, unchanged), (1, 20));
        let reparsed = parse_inter1(&rebuilt).expect("reparse INTER1");
        assert_eq!(
            join_inter_group(&reparsed.groups[0]).expect("join"),
            "変更した説明"
        );
        for index in 0..15 {
            let pointer = 0x7960 + INTER1_POINTER_TABLE + index * 4;
            assert_eq!(
                read_u16(&rebuilt, pointer + 2).expect("segment"),
                INTER1_DATA_SEGMENT
            );
        }
    }

    #[test]
    fn inter2_uses_confirmed_groups_without_editable_page_labels() {
        let source = make_inter2();
        let inter = parse_inter2(&source).expect("parse INTER2");
        assert_eq!(inter.groups.len(), 50);
        assert_eq!(inter.fixed_strings.len(), 12);
        let mut document = TranslationDocument {
            _format: JSON_FORMAT.to_string(),
            _source_kind: SOURCE_KIND_INTER2.to_string(),
            _source_file: "INTER2.EXE".to_string(),
            _source_sha256: sha256_hex(&source),
            _line_visible_byte_limit: INTER_LINE_BYTE_LIMIT,
            entries: inter.translation_entries("INTER2.EXE").expect("entries"),
        };
        assert_eq!(document.entries.len(), 62);
        assert_eq!(document.entries[7]._type, "title");
        assert!(document
            .entries
            .iter()
            .all(|entry| entry.scr_msg != "Ⅰ" && entry.scr_msg != "Ⅱ"));
        document.entries[7].message = "試験型".to_string();
        document.entries[14].message = "回答文".to_string();
        let (rebuilt, patched, unchanged) = inter
            .rebuild_with_document(&source, &document)
            .expect("rebuild INTER2");
        assert_eq!((patched, unchanged), (2, 60));
        let reparsed = parse_inter2(&rebuilt).expect("reparse INTER2");
        let entries = reparsed.translation_entries("INTER2.EXE").expect("entries");
        assert_eq!(entries[7].scr_msg, "試験型");
        assert_eq!(entries[14].scr_msg, "回答文");
    }

    fn make_mes() -> Vec<u8> {
        let mut bytes = Vec::new();
        for index in 0..5 {
            bytes
                .extend_from_slice(&encode_cp932(&format!("『文章{index}』")).expect("encode MES"));
            bytes.push(0);
        }
        bytes
    }

    #[test]
    fn recursive_extract_flattens_json_and_disambiguates_duplicate_basenames() {
        let root = unique_test_dir("flat-json");
        let source = root.join("source");
        fs::create_dir_all(source.join("disk1")).expect("create disk1");
        fs::create_dir_all(source.join("disk2")).expect("create disk2");
        fs::write(source.join("disk1").join("SAME.MES"), make_mes()).expect("write first MES");
        fs::write(source.join("disk2").join("SAME.MES"), make_mes()).expect("write second MES");
        fs::write(source.join("UNIQUE.MES"), make_mes()).expect("write unique MES");
        fs::write(source.join("ignored.bin"), b"not text").expect("write ignored file");
        let output = root.join("alljson");
        let report = extract_path(&source, &output, false).expect("extract recursively");
        assert_eq!(
            (
                report.scanned_files,
                report.skipped_files,
                report.json_files
            ),
            (4, 1, 3)
        );
        let files: Vec<_> = fs::read_dir(&output)
            .expect("read alljson")
            .map(|entry| entry.expect("entry").path())
            .collect();
        assert_eq!(files.len(), 3);
        assert!(files.iter().all(|path| path.is_file()));
        assert!(output.join("UNIQUE.MES.json").is_file());
        let duplicate_names: HashSet<_> = files
            .iter()
            .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
            .filter(|name| name.ends_with("SAME.MES.json"))
            .collect();
        assert_eq!(duplicate_names.len(), 2);
        fs::remove_dir_all(&root).expect("remove flat-json test directory");
    }
}
