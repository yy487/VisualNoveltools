//! Data-disk boot prompts and PAL/PRS directory catalogs for Galaxy Railway.
//!
//! Boot text is stored in the disk's first sector as three rows in a screen
//! template. The boot code starts at sector offset 0xDE, advances 0x36 bytes
//! per row, and reads 16-bit words until a zero word. Prompt rows keep their
//! box border in place; an edit may vary in length inside the discovered text
//! span and the remaining span is padded with full-width spaces.

use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use vn_d88::{Decoder, Image, Sector, StandardCodec};
use vn_font::font_98::EncodingPlan;

use crate::{font_plan::encode_display, sha256, Result};

pub const BOOT_SCHEMA: &str = "galaxy-railway-pc98-boot-v1";
pub const RESOURCE_SCHEMA: &str = "galaxy-railway-pc98-resource-catalogs-v1";

const BOOT_SECTOR: Chrn = Chrn {
    cylinder: 0,
    head: 0,
    record: 1,
    size_code: 3,
};
const SCREEN_ROWS_OFFSET: usize = 0x00DE;
const SCREEN_ROW_STRIDE: usize = 0x36;
const SCREEN_ROW_COUNT: usize = 9;
const PROMPT_ROW_INDICES: [usize; 3] = [2, 4, 6];
const PROMPT_PREFIX: [u8; 4] = [0x81, 0x9E, 0x81, 0x40];
const BOX_BORDER: [u8; 2] = [0x81, 0x9E];
const FULLWIDTH_SPACE: [u8; 2] = [0x81, 0x40];

/// Extracted, hash-bound document for the three startup prompt lines.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BootDocument {
    pub schema: String,
    pub source_file: String,
    pub source_sha256: String,
    pub encoding: String,
    pub entries: Vec<BootString>,
    pub diagnostics: Vec<String>,
}

/// One prompt inside a code-referenced, NUL-terminated screen row.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BootString {
    /// Zero-based prompt order (the three rows are 2, 4, and 6).
    #[serde(rename = "_index")]
    pub index: usize,
    /// Screen-row index used by the boot code's 0x36-byte row stride.
    #[serde(rename = "_row_index")]
    pub row_index: usize,
    /// Physical sector position in the decoded disk's sector sequence.
    #[serde(rename = "_sector_index")]
    pub sector_index: usize,
    #[serde(rename = "_track_slot")]
    pub track_slot: usize,
    #[serde(rename = "_physical_ordinal")]
    pub physical_ordinal: usize,
    #[serde(rename = "_cylinder")]
    pub cylinder: u16,
    #[serde(rename = "_head")]
    pub head: u8,
    #[serde(rename = "_record")]
    pub record: u16,
    #[serde(rename = "_size_code")]
    pub size_code: u8,
    /// Offset from the start of the sector data, not from the D88 container.
    #[serde(rename = "_offset")]
    pub offset: usize,
    /// Original readable text with trailing full-width padding removed.
    pub scr_msg: String,
    /// Editable CP932 text.
    pub message: String,
    /// Bytes available before the right-hand box border.
    #[serde(rename = "_capacity")]
    pub capacity: usize,
    /// Fixed row prefix that must remain immediately before `offset`.
    #[serde(rename = "_prefix_hex")]
    pub prefix_hex: String,
    /// Right-hand box-border bytes after the editable region.
    #[serde(rename = "_border_hex")]
    pub border_hex: String,
    /// Row-local offset of the boot code's zero-word terminator.
    #[serde(rename = "_terminator_offset")]
    pub terminator_offset: usize,
    #[serde(rename = "_terminator_hex")]
    pub terminator_hex: String,
}

/// Read-only inventory of a data disk's PAL and PRS file catalogs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ResourceInventory {
    pub schema: String,
    pub disk_number: u8,
    pub source_sha256: String,
    pub total_entries: usize,
    pub catalogs: Vec<ResourceCatalog>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ResourceCatalog {
    pub extension: String,
    pub cylinder: u16,
    pub head: u8,
    pub record: u16,
    pub count: u16,
    /// File offsets are relative to the first record byte after the u16 count.
    pub offset_base: String,
    pub entries: Vec<ResourceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ResourceEntry {
    pub name: String,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone, Copy)]
struct Chrn {
    cylinder: u16,
    head: u8,
    record: u16,
    size_code: u8,
}

#[derive(Debug, Clone, Copy)]
struct CatalogLocation {
    extension: &'static str,
    chrn: Chrn,
    expected_count: u16,
}

/// Extract the boot prompt document from one PC-98 D88 data disk.
pub fn extract_boot_document(raw: &[u8], source_file: impl Into<String>) -> Result<BootDocument> {
    let source_file = source_file.into();
    let image = decode_single_disk(raw)?;
    let sectors = ordered_sectors(&image);
    let sector_index = find_sector_index(&sectors, BOOT_SECTOR)?;
    let sector = sectors[sector_index];
    let sector_bytes = get_sector_bytes(raw, sector)?;
    let mut entries = Vec::with_capacity(PROMPT_ROW_INDICES.len());

    for (index, row_index) in PROMPT_ROW_INDICES.iter().copied().enumerate() {
        let row_start = SCREEN_ROWS_OFFSET
            .checked_add(row_index * SCREEN_ROW_STRIDE)
            .ok_or_else(|| "boot screen row offset overflow".to_owned())?;
        let row_end = row_start
            .checked_add(SCREEN_ROW_STRIDE)
            .ok_or_else(|| "boot screen row end overflow".to_owned())?;
        if row_index >= SCREEN_ROW_COUNT || row_end > sector_bytes.len() {
            return Err(format!(
                "boot prompt row {row_index} exceeds the decoded boot sector"
            ));
        }

        let row = &sector_bytes[row_start..row_end];
        if row.get(..PROMPT_PREFIX.len()) != Some(PROMPT_PREFIX.as_slice()) {
            return Err(format!(
                "boot prompt row {row_index} has an unexpected left border/prefix"
            ));
        }
        let terminator_in_row = find_word_terminator(row).ok_or_else(|| {
            format!("boot prompt row {row_index} has no word terminator within its stride")
        })?;
        let text_start_in_row = PROMPT_PREFIX.len();
        let border_in_row = find_bytes(&row[text_start_in_row..terminator_in_row], &BOX_BORDER)
            .map(|offset| text_start_in_row + offset)
            .ok_or_else(|| format!("boot prompt row {row_index} has no right-hand box border"))?;
        if border_in_row + BOX_BORDER.len() != terminator_in_row {
            return Err(format!(
                "boot prompt row {row_index} has unexpected bytes between its border and terminator"
            ));
        }

        let field = &row[text_start_in_row..border_in_row];
        if field.len() % FULLWIDTH_SPACE.len() != 0 {
            return Err(format!(
                "boot prompt row {row_index} has an odd-byte CP932 text span"
            ));
        }
        let (message_bytes, padding) = split_fullwidth_padding(field);
        let padding = padding.strip_prefix(b" ").unwrap_or(padding);
        if padding.len() % FULLWIDTH_SPACE.len() != 0
            || padding
                .chunks_exact(2)
                .any(|chunk| chunk != FULLWIDTH_SPACE)
        {
            return Err(format!(
                "boot prompt row {row_index} has unsupported right-padding bytes"
            ));
        }
        let scr_msg = decode_cp932(message_bytes, row_index)?;

        entries.push(BootString {
            index,
            row_index,
            sector_index,
            track_slot: sector.address.track_slot,
            physical_ordinal: sector.address.physical_ordinal,
            cylinder: sector.address.id.cylinder,
            head: sector.address.id.head,
            record: sector.address.id.record,
            size_code: sector.address.id.size_code,
            offset: row_start + text_start_in_row,
            scr_msg: scr_msg.clone(),
            message: scr_msg,
            capacity: field.len(),
            prefix_hex: bytes_to_hex(&row[..text_start_in_row]),
            border_hex: bytes_to_hex(&row[border_in_row..border_in_row + BOX_BORDER.len()]),
            terminator_offset: row_start + terminator_in_row,
            terminator_hex: bytes_to_hex(&row[terminator_in_row..terminator_in_row + 2]),
        });
    }

    Ok(BootDocument {
        schema: BOOT_SCHEMA.to_owned(),
        source_file,
        source_sha256: sha256(raw),
        encoding: "CP932".to_owned(),
        entries,
        diagnostics: Vec::new(),
    })
}

/// Apply variable-length CP932 prompt edits without moving the boot rows.
///
/// The boot code's row stride and zero-word terminator are fixed. A translated
/// string may change length within its row's discovered text span; remaining
/// bytes are filled with CP932 full-width spaces, while the box borders,
/// terminators, and every other sector byte are retained.
pub fn apply_boot_document(
    raw: &[u8],
    document: &BootDocument,
    encoding: &EncodingPlan,
) -> Result<Vec<u8>> {
    if document.schema != BOOT_SCHEMA {
        return Err(format!(
            "unsupported boot document schema {:?}",
            document.schema
        ));
    }
    if document.source_sha256 != sha256(raw) {
        return Err("boot document source_sha256 does not match the D88 input".to_owned());
    }

    if document.encoding != "CP932" {
        return Err(format!(
            "unsupported boot document encoding {:?}; expected CP932",
            document.encoding
        ));
    }
    let baseline = extract_boot_document(raw, document.source_file.clone())?;
    if document.entries.len() != baseline.entries.len() {
        return Err(format!(
            "boot document has {} entries; source has {}",
            document.entries.len(),
            baseline.entries.len()
        ));
    }
    if document.diagnostics != baseline.diagnostics {
        return Err("boot document diagnostics are read-only".to_owned());
    }
    for (requested, source) in document.entries.iter().zip(&baseline.entries) {
        if !same_boot_location(requested, source) {
            return Err(format!(
                "boot string {} changed immutable source metadata",
                source.index
            ));
        }
    }

    let image = decode_single_disk(raw)?;
    let sectors = ordered_sectors(&image);
    let sector_index = find_sector_index(&sectors, BOOT_SECTOR)?;
    let sector = sectors[sector_index];
    let sector_bytes = get_sector_bytes(raw, sector)?;
    let mut output = raw.to_vec();

    for entry in &document.entries {
        let row_start = SCREEN_ROWS_OFFSET + entry.row_index * SCREEN_ROW_STRIDE;
        let row = sector_bytes
            .get(row_start..row_start + SCREEN_ROW_STRIDE)
            .ok_or_else(|| format!("boot prompt row {} is out of bounds", entry.row_index))?;
        let text_start_in_row = entry.offset.checked_sub(row_start).ok_or_else(|| {
            format!(
                "boot prompt row {} has an invalid text offset",
                entry.row_index
            )
        })?;
        let text_end_in_row = text_start_in_row
            .checked_add(entry.capacity)
            .ok_or_else(|| format!("boot prompt row {} capacity overflow", entry.row_index))?;
        if text_start_in_row != PROMPT_PREFIX.len()
            || row.get(..text_start_in_row) != Some(PROMPT_PREFIX.as_slice())
            || row.get(text_end_in_row..text_end_in_row + 2) != Some(BOX_BORDER.as_slice())
            || row.get(text_end_in_row + 2..text_end_in_row + 4) != Some([0, 0].as_slice())
        {
            return Err(format!(
                "boot prompt row {} border or terminator bytes changed",
                entry.row_index
            ));
        }
        if entry.prefix_hex != bytes_to_hex(PROMPT_PREFIX.as_slice())
            || entry.border_hex != bytes_to_hex(&BOX_BORDER)
            || entry.terminator_hex != "00 00"
            || entry.terminator_offset != row_start + text_end_in_row + BOX_BORDER.len()
        {
            return Err(format!(
                "boot prompt row {} has altered border metadata",
                entry.row_index
            ));
        }

        if entry.message.contains('\n') || entry.message.contains('\r') {
            return Err(format!(
                "boot prompt {} is a single-line field and cannot contain line breaks",
                entry.index
            ));
        }
        let encoded = encode_display(&entry.message, encoding)
            .map_err(|error| format!("boot prompt {} cannot be encoded: {error}", entry.index))?;
        if encoded.contains(&0) {
            return Err(format!(
                "boot prompt {} contains a zero byte reserved for row termination",
                entry.index
            ));
        }
        if encoded.len() > entry.capacity {
            return Err(format!(
                "boot prompt {} encodes to {} bytes, exceeding its {}-byte row text span",
                entry.index,
                encoded.len(),
                entry.capacity
            ));
        }
        if find_bytes(&encoded, &BOX_BORDER).is_some() {
            return Err(format!(
                "boot prompt {} contains the reserved box-border byte sequence",
                entry.index
            ));
        }
        let replacement = pad_cp932_field(encoded, entry.capacity)?;
        let start = sector
            .data_range
            .start
            .checked_add(entry.offset)
            .ok_or_else(|| "boot prompt source offset overflow".to_owned())?;
        let end = start
            .checked_add(entry.capacity)
            .ok_or_else(|| "boot prompt source range overflow".to_owned())?;
        let target = output
            .get_mut(start..end)
            .ok_or_else(|| format!("boot prompt {} exceeds the D88 source", entry.index))?;
        target.copy_from_slice(&replacement);
    }

    Ok(output)
}

/// Extract PAL/PRS directory records from a data disk without reading payloads.
pub fn extract_resource_catalogs(raw: &[u8], disk_number: u8) -> Result<ResourceInventory> {
    let locations = catalog_locations(disk_number)?;
    let image = decode_single_disk(raw)?;
    let sectors = ordered_sectors(&image);
    let mut catalogs = Vec::with_capacity(locations.len());
    let mut total_entries = 0usize;

    for location in locations {
        let catalog = parse_resource_catalog(raw, &sectors, location)?;
        total_entries = total_entries
            .checked_add(catalog.entries.len())
            .ok_or_else(|| "resource catalog entry total overflow".to_owned())?;
        catalogs.push(catalog);
    }

    Ok(ResourceInventory {
        schema: RESOURCE_SCHEMA.to_owned(),
        disk_number,
        source_sha256: sha256(raw),
        total_entries,
        catalogs,
    })
}

fn same_boot_location(left: &BootString, right: &BootString) -> bool {
    left.index == right.index
        && left.row_index == right.row_index
        && left.sector_index == right.sector_index
        && left.track_slot == right.track_slot
        && left.physical_ordinal == right.physical_ordinal
        && left.cylinder == right.cylinder
        && left.head == right.head
        && left.record == right.record
        && left.size_code == right.size_code
        && left.offset == right.offset
        && left.scr_msg == right.scr_msg
        && left.capacity == right.capacity
        && left.prefix_hex == right.prefix_hex
        && left.border_hex == right.border_hex
        && left.terminator_offset == right.terminator_offset
        && left.terminator_hex == right.terminator_hex
}

fn decode_single_disk(raw: &[u8]) -> Result<Image> {
    let image = StandardCodec
        .decode(raw)
        .map_err(|error| format!("cannot decode D88 image: {error}"))?;
    if image.disks.len() != 1 {
        return Err(format!(
            "expected one D88 disk image, found {}",
            image.disks.len()
        ));
    }
    Ok(image)
}

fn ordered_sectors(image: &Image) -> Vec<&Sector> {
    image.disks[0]
        .tracks
        .iter()
        .flat_map(|track| track.sectors.iter())
        .collect()
}

fn find_sector_index(sectors: &[&Sector], chrn: Chrn) -> Result<usize> {
    let mut found = None;
    for (index, sector) in sectors.iter().enumerate() {
        let id = sector.address.id;
        if id.cylinder == chrn.cylinder
            && id.head == chrn.head
            && id.record == chrn.record
            && id.size_code == chrn.size_code
            && found.replace(index).is_some()
        {
            return Err(format!(
                "ambiguous D88 sector C{}/H{}/R{} N{}",
                chrn.cylinder, chrn.head, chrn.record, chrn.size_code
            ));
        }
    }
    found.ok_or_else(|| {
        format!(
            "missing D88 sector C{}/H{}/R{} N{}",
            chrn.cylinder, chrn.head, chrn.record, chrn.size_code
        )
    })
}

fn get_sector_bytes<'a>(raw: &'a [u8], sector: &Sector) -> Result<&'a [u8]> {
    raw.get(sector.data_range.clone()).ok_or_else(|| {
        format!(
            "D88 sector data range 0x{:X}..0x{:X} exceeds source",
            sector.data_range.start, sector.data_range.end
        )
    })
}

fn find_word_terminator(row: &[u8]) -> Option<usize> {
    row.chunks_exact(2)
        .position(|word| word == [0, 0])
        .map(|word_index| word_index * 2)
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn split_fullwidth_padding(field: &[u8]) -> (&[u8], &[u8]) {
    let mut end = field.len();
    while end >= FULLWIDTH_SPACE.len() && field[end - FULLWIDTH_SPACE.len()..end] == FULLWIDTH_SPACE
    {
        end -= FULLWIDTH_SPACE.len();
    }
    // An odd number of ASCII/half-width bytes is aligned with one ordinary
    // space before the full-width padding so the border remains on a word
    // boundary. Treat that alignment byte as padding on the next extraction.
    if end > 0 && field[end - 1] == b' ' {
        end -= 1;
    }
    (&field[..end], &field[end..])
}

fn decode_cp932(bytes: &[u8], row_index: usize) -> Result<String> {
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(bytes);
    if had_errors {
        return Err(format!(
            "boot prompt row {row_index} contains invalid CP932 bytes"
        ));
    }
    let (roundtrip, _, had_encoding_errors) = SHIFT_JIS.encode(&decoded);
    if had_encoding_errors || roundtrip.as_ref() != bytes {
        return Err(format!("boot prompt row {row_index} is not lossless CP932"));
    }
    Ok(decoded.into_owned())
}

fn pad_cp932_field(mut encoded: Vec<u8>, capacity: usize) -> Result<Vec<u8>> {
    if encoded.len() > capacity {
        return Err("encoded prompt exceeds its discovered field span".to_owned());
    }
    let remaining = capacity - encoded.len();
    if !remaining.is_multiple_of(2) {
        encoded.push(b' ');
    }
    while encoded.len() + FULLWIDTH_SPACE.len() <= capacity {
        encoded.extend_from_slice(&FULLWIDTH_SPACE);
    }
    if encoded.len() != capacity {
        return Err("cannot pad prompt to the discovered CP932 span".to_owned());
    }
    Ok(encoded)
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn catalog_locations(disk_number: u8) -> Result<Vec<CatalogLocation>> {
    let sector = |cylinder, head, record| Chrn {
        cylinder,
        head,
        record,
        size_code: 3,
    };
    let locations = match disk_number {
        2 => vec![
            CatalogLocation {
                extension: "PAL",
                chrn: sector(0, 1, 4),
                expected_count: 17,
            },
            CatalogLocation {
                extension: "PRS",
                chrn: sector(1, 0, 1),
                expected_count: 62,
            },
        ],
        3 => vec![
            CatalogLocation {
                extension: "PRS",
                chrn: sector(0, 1, 4),
                expected_count: 61,
            },
            CatalogLocation {
                extension: "PAL",
                chrn: sector(71, 1, 5),
                expected_count: 22,
            },
        ],
        4 => vec![
            CatalogLocation {
                extension: "PRS",
                chrn: sector(0, 1, 4),
                expected_count: 50,
            },
            CatalogLocation {
                extension: "PAL",
                chrn: sector(61, 1, 6),
                expected_count: 16,
            },
        ],
        5 => vec![
            CatalogLocation {
                extension: "PRS",
                chrn: sector(0, 1, 4),
                expected_count: 52,
            },
            CatalogLocation {
                extension: "PAL",
                chrn: sector(75, 0, 6),
                expected_count: 40,
            },
        ],
        _ => {
            return Err(format!(
                "resource catalogs are defined only for data disks 2-5, got {disk_number}"
            ));
        }
    };
    Ok(locations)
}

fn parse_resource_catalog(
    raw: &[u8],
    sectors: &[&Sector],
    location: CatalogLocation,
) -> Result<ResourceCatalog> {
    let sector_index = find_sector_index(sectors, location.chrn)?;
    let prefix = read_physical_bytes(raw, sectors, sector_index, 2)?;
    let count = u16::from_le_bytes([prefix[0], prefix[1]]);
    if count != location.expected_count {
        return Err(format!(
            "{} catalog at C{}/H{}/R{} declares {count} records; expected {}",
            location.extension,
            location.chrn.cylinder,
            location.chrn.head,
            location.chrn.record,
            location.expected_count
        ));
    }
    let table_size = usize::from(count)
        .checked_mul(22)
        .and_then(|bytes| bytes.checked_add(2))
        .ok_or_else(|| "resource catalog size overflow".to_owned())?;
    let table = read_physical_bytes(raw, sectors, sector_index, table_size)?;
    let mut entries = Vec::with_capacity(usize::from(count));

    for entry_index in 0..usize::from(count) {
        let start = 2 + entry_index * 22;
        let record = &table[start..start + 22];
        let name = parse_ascii_name(&record[..14], location.extension, entry_index)?;
        let offset = u32::from_le_bytes(record[14..18].try_into().expect("four-byte offset"));
        let size = u32::from_le_bytes(record[18..22].try_into().expect("four-byte size"));
        if size == 0 {
            return Err(format!(
                "{} catalog entry {entry_index} ({name}) has zero size",
                location.extension
            ));
        }
        entries.push(ResourceEntry { name, offset, size });
    }

    let table_bytes_after_count = u32::from(count)
        .checked_mul(22)
        .ok_or_else(|| "resource catalog record-table length overflow".to_owned())?;
    if entries.first().map(|entry| entry.offset) != Some(table_bytes_after_count) {
        return Err(format!(
            "{} catalog's first payload offset does not equal count * 22",
            location.extension
        ));
    }
    for (index, pair) in entries.windows(2).enumerate() {
        let expected = pair[0]
            .offset
            .checked_add(pair[0].size)
            .ok_or_else(|| format!("{} catalog offset overflow", location.extension))?;
        if pair[1].offset != expected {
            return Err(format!(
                "{} catalog entries {} and {} are not laid out contiguously",
                location.extension,
                index,
                index + 1
            ));
        }
    }

    Ok(ResourceCatalog {
        extension: location.extension.to_owned(),
        cylinder: location.chrn.cylinder,
        head: location.chrn.head,
        record: location.chrn.record,
        count,
        offset_base: "records_start_after_u16_count".to_owned(),
        entries,
    })
}

fn parse_ascii_name(field: &[u8], extension: &str, entry_index: usize) -> Result<String> {
    let name_end = field
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(field.len());
    let name_bytes = &field[..name_end];
    if name_bytes.is_empty() || name_bytes.iter().any(|byte| !(0x20..=0x7E).contains(byte)) {
        return Err(format!(
            "{extension} catalog entry {entry_index} has an invalid ASCII filename"
        ));
    }
    if field[name_end..].iter().any(|byte| *byte != 0) {
        return Err(format!(
            "{extension} catalog entry {entry_index} has nonzero bytes after filename padding"
        ));
    }
    let name = std::str::from_utf8(name_bytes)
        .map_err(|_| format!("{extension} catalog entry {entry_index} is not ASCII"))?
        .to_owned();
    let expected_suffix = format!(".{extension}");
    if !name.ends_with(&expected_suffix) {
        return Err(format!(
            "{extension} catalog entry {entry_index} has unexpected extension in {name:?}"
        ));
    }
    Ok(name)
}

fn read_physical_bytes(
    raw: &[u8],
    sectors: &[&Sector],
    first_sector: usize,
    byte_count: usize,
) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(byte_count);
    for sector in sectors.iter().skip(first_sector) {
        let data = get_sector_bytes(raw, sector)?;
        let remaining = byte_count - bytes.len();
        let take = remaining.min(data.len());
        bytes.extend_from_slice(&data[..take]);
        if bytes.len() == byte_count {
            return Ok(bytes);
        }
    }
    Err(format!(
        "catalog table needs {byte_count} bytes but only {} remain in physical sectors",
        bytes.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use vn_font::font_98::SubstitutionMap;

    const HEADER_LEN: usize = 0x2A0;
    const SECTOR_LEN: usize = 1024;

    struct SyntheticSector {
        cylinder: u8,
        head: u8,
        record: u8,
        data: Vec<u8>,
    }

    fn synthetic_image() -> Vec<u8> {
        let mut boot = vec![0xCC; SECTOR_LEN];
        for (row_index, message) in [
            (2usize, "このディスクは偽物です"),
            (4, "新しい盤を入れてください"),
            (6, "キーを押す"),
        ] {
            let start = SCREEN_ROWS_OFFSET + row_index * SCREEN_ROW_STRIDE;
            let capacity = 46;
            let (encoded, _, had_errors) = SHIFT_JIS.encode(message);
            assert!(!had_errors);
            assert!(encoded.len() <= capacity);
            let mut field = encoded.into_owned();
            let padding = capacity - field.len();
            if padding % 2 != 0 {
                field.push(b' ');
            }
            while field.len() < capacity {
                field.extend_from_slice(&FULLWIDTH_SPACE);
            }
            boot[start..start + 4].copy_from_slice(&PROMPT_PREFIX);
            boot[start + 4..start + 4 + capacity].copy_from_slice(&field);
            boot[start + 4 + capacity..start + 4 + capacity + 2].copy_from_slice(&BOX_BORDER);
            boot[start + 4 + capacity + 2..start + 4 + capacity + 4].copy_from_slice(&[0, 0]);
        }

        let pal_table = synthetic_catalog("PAL", 17);
        let prs_table = synthetic_catalog("PRS", 62);
        let mut pal = vec![0xE5; SECTOR_LEN];
        pal[..pal_table.len()].copy_from_slice(&pal_table);
        let mut prs_first = vec![0xE5; SECTOR_LEN];
        prs_first.copy_from_slice(&prs_table[..SECTOR_LEN]);
        let mut prs_second = vec![0xE5; SECTOR_LEN];
        prs_second[..prs_table.len() - SECTOR_LEN].copy_from_slice(&prs_table[SECTOR_LEN..]);

        build_d88(vec![
            vec![SyntheticSector {
                cylinder: 0,
                head: 0,
                record: 1,
                data: boot,
            }],
            vec![SyntheticSector {
                cylinder: 0,
                head: 1,
                record: 4,
                data: pal,
            }],
            vec![
                SyntheticSector {
                    cylinder: 1,
                    head: 0,
                    record: 1,
                    data: prs_first,
                },
                SyntheticSector {
                    cylinder: 1,
                    head: 0,
                    record: 2,
                    data: prs_second,
                },
            ],
        ])
    }

    fn encoding_plan(messages: &[String]) -> EncodingPlan {
        EncodingPlan::build(
            &SubstitutionMap::embedded().expect("embedded substitutions"),
            std::iter::empty::<u16>(),
            messages.iter().map(String::as_str),
        )
        .expect("synthetic encoding plan")
    }

    fn synthetic_catalog(extension: &str, count: u16) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(2 + usize::from(count) * 22);
        bytes.extend_from_slice(&count.to_le_bytes());
        let mut offset = u32::from(count) * 22;
        for index in 0..count {
            let name = format!("F{index:03}.{extension}");
            let mut name_field = [0u8; 14];
            name_field[..name.len()].copy_from_slice(name.as_bytes());
            bytes.extend_from_slice(&name_field);
            bytes.extend_from_slice(&offset.to_le_bytes());
            bytes.extend_from_slice(&48u32.to_le_bytes());
            offset += 48;
        }
        bytes
    }

    fn build_d88(tracks: Vec<Vec<SyntheticSector>>) -> Vec<u8> {
        let mut output = vec![0u8; HEADER_LEN];
        output[..4].copy_from_slice(b"TEST");
        let mut relative = HEADER_LEN;
        for (track_slot, sectors) in tracks.iter().enumerate() {
            let table_offset = 0x20 + track_slot * 4;
            output[table_offset..table_offset + 4]
                .copy_from_slice(&(relative as u32).to_le_bytes());
            for sector in sectors {
                output.push(sector.cylinder);
                output.push(sector.head);
                output.push(sector.record);
                output.push(3);
                output.extend_from_slice(&(sectors.len() as u16).to_le_bytes());
                output.extend_from_slice(&[0; 8]);
                output.extend_from_slice(&(sector.data.len() as u16).to_le_bytes());
                output.extend_from_slice(&sector.data);
            }
            relative = output.len();
        }
        let disk_size = output.len() as u32;
        output[0x1C..0x20].copy_from_slice(&disk_size.to_le_bytes());
        output
    }

    #[test]
    fn variable_length_boot_edits_preserve_rows_borders_and_other_bytes() {
        let raw = synthetic_image();
        let mut document = extract_boot_document(&raw, "synthetic.D88").unwrap();
        assert_eq!(document.encoding, "CP932");
        assert!(document.diagnostics.is_empty());
        assert_eq!(document.entries.len(), 3);
        assert!(document.entries.iter().all(|entry| entry.capacity == 46));
        document.entries[0].message = "银河铁道提示".to_owned();
        document.entries[1].message = "Drive A".to_owned();

        let json = serde_json::to_value(&document).unwrap();
        let object = json.as_object().unwrap();
        assert!(object.contains_key("schema"));
        assert!(object.contains_key("source_file"));
        assert!(object.contains_key("source_sha256"));
        assert!(object.contains_key("encoding"));
        assert!(object.contains_key("entries"));
        assert!(object.contains_key("diagnostics"));
        assert!(!object.contains_key("strings"));
        let entry = object["entries"][0].as_object().unwrap();
        assert!(entry.contains_key("_index"));
        assert!(entry.contains_key("_row_index"));
        assert!(entry.contains_key("_offset"));
        assert!(entry.contains_key("scr_msg"));
        assert!(entry.contains_key("message"));
        assert_eq!(entry.len(), 17);
        assert!(!entry.contains_key("parts"));

        let messages = document
            .entries
            .iter()
            .map(|entry| entry.message.clone())
            .collect::<Vec<_>>();
        let plan = encoding_plan(&messages);
        let output = apply_boot_document(&raw, &document, &plan).unwrap();
        let image = decode_single_disk(&raw).unwrap();
        let sectors = ordered_sectors(&image);
        let boot_sector = sectors[find_sector_index(&sectors, BOOT_SECTOR).unwrap()];
        let allowed_ranges = document
            .entries
            .iter()
            .map(|entry| {
                let start = boot_sector.data_range.start + entry.offset;
                start..start + entry.capacity
            })
            .collect::<Vec<_>>();
        for (index, (before, after)) in raw.iter().zip(&output).enumerate() {
            if !allowed_ranges.iter().any(|range| range.contains(&index)) {
                assert_eq!(
                    before, after,
                    "unexpected change at source offset {index:#x}"
                );
            }
        }

        let extracted = extract_boot_document(&output, "synthetic.D88").unwrap();
        assert_eq!(
            plan.decode_carriers(&extracted.entries[0].message),
            document.entries[0].message
        );
        assert_eq!(extracted.entries[1].message, document.entries[1].message);
        assert_eq!(extracted.entries[2].message, document.entries[2].message);
    }

    #[test]
    fn boot_prompts_reject_line_breaks_and_overflow_without_truncating() {
        let raw = synthetic_image();
        let mut document = extract_boot_document(&raw, "synthetic.D88").unwrap();
        document.entries[0].message = "第一行\n第二行".to_owned();
        let plan = encoding_plan(&[]);
        let error = apply_boot_document(&raw, &document, &plan).unwrap_err();
        assert!(error.contains("single-line"));

        document.entries[0].message = "A".repeat(document.entries[0].capacity + 1);
        let plan = encoding_plan(
            &document
                .entries
                .iter()
                .map(|entry| entry.message.clone())
                .collect::<Vec<_>>(),
        );
        let error = apply_boot_document(&raw, &document, &plan).unwrap_err();
        assert!(error.contains("exceeding its 46-byte row text span"));
    }

    #[test]
    fn catalogs_parse_full_tables_and_validate_offset_base() {
        let raw = synthetic_image();
        let inventory = extract_resource_catalogs(&raw, 2).unwrap();
        assert_eq!(inventory.total_entries, 79);
        assert_eq!(inventory.catalogs.len(), 2);
        let pal = inventory
            .catalogs
            .iter()
            .find(|catalog| catalog.extension == "PAL")
            .unwrap();
        let prs = inventory
            .catalogs
            .iter()
            .find(|catalog| catalog.extension == "PRS")
            .unwrap();
        assert_eq!(pal.count, 17);
        assert_eq!(prs.count, 62);
        assert_eq!(pal.offset_base, "records_start_after_u16_count");
        assert_eq!(prs.entries[0].offset, 62 * 22);
        assert_eq!(
            prs.entries[1].offset,
            prs.entries[0].offset + prs.entries[0].size
        );
    }
}
