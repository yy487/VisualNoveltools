//! Directory and member rebuilding for the game's `TXTALL.DAT` archive.
//!
//! The archive begins with a little-endian `u16` entry count. Each 22-byte
//! directory record has a 14-byte NUL-padded ASCII name, a little-endian
//! `u32` absolute offset, and a little-endian `u32` byte length. Bytes in
//! gaps between members and after the final member are opaque and preserved.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

const HEADER_LEN: usize = 2;
const RECORD_LEN: usize = 22;
const NAME_LEN: usize = 14;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub index: usize,
    pub name: String,
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Catalog {
    pub table_end: usize,
    pub entries: Vec<CatalogEntry>,
}

pub fn parse(bytes: &[u8]) -> Result<Catalog, String> {
    if bytes.len() < HEADER_LEN {
        return Err("TXTALL.DAT is shorter than its entry count".into());
    }
    let count = usize::from(u16::from_le_bytes([bytes[0], bytes[1]]));
    let table_end = count
        .checked_mul(RECORD_LEN)
        .and_then(|size| HEADER_LEN.checked_add(size))
        .ok_or_else(|| "TXTALL.DAT directory size overflow".to_string())?;
    if table_end > bytes.len() {
        return Err(format!(
            "TXTALL.DAT directory ends at 0x{table_end:X}, past file length 0x{:X}",
            bytes.len()
        ));
    }

    let mut entries = Vec::with_capacity(count);
    let mut names = BTreeSet::new();
    for index in 0..count {
        let record_start = HEADER_LEN + index * RECORD_LEN;
        let name_bytes = &bytes[record_start..record_start + NAME_LEN];
        let name_end = name_bytes
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(NAME_LEN);
        if name_end == 0 {
            return Err(format!("TXTALL.DAT entry {index} has an empty name"));
        }
        if name_bytes[name_end..].iter().any(|&byte| byte != 0) {
            return Err(format!(
                "TXTALL.DAT entry {index} has nonzero bytes after its name terminator"
            ));
        }
        let raw_name = &name_bytes[..name_end];
        if raw_name
            .iter()
            .any(|&byte| !byte.is_ascii_graphic() || matches!(byte, b'/' | b'\\' | b':'))
        {
            return Err(format!(
                "TXTALL.DAT entry {index} has a non-ASCII or unsafe name"
            ));
        }
        let name = String::from_utf8(raw_name.to_vec())
            .map_err(|_| format!("TXTALL.DAT entry {index} has a non-ASCII name"))?;
        if name == "." || name == ".." {
            return Err(format!("TXTALL.DAT entry {index} has an unsafe name"));
        }
        if !names.insert(name.to_ascii_uppercase()) {
            return Err(format!("TXTALL.DAT contains duplicate member {name}"));
        }

        let offset_pos = record_start + NAME_LEN;
        let offset = u32::from_le_bytes(
            bytes[offset_pos..offset_pos + 4]
                .try_into()
                .expect("record was bounded by table_end"),
        ) as usize;
        let length = u32::from_le_bytes(
            bytes[offset_pos + 4..offset_pos + 8]
                .try_into()
                .expect("record was bounded by table_end"),
        ) as usize;
        let end = offset
            .checked_add(length)
            .ok_or_else(|| format!("TXTALL.DAT member {name} range overflows"))?;
        if offset < table_end || end > bytes.len() {
            return Err(format!(
                "TXTALL.DAT member {name} range 0x{offset:X}..0x{end:X} is outside payload 0x{table_end:X}..0x{:X}",
                bytes.len()
            ));
        }
        entries.push(CatalogEntry {
            index,
            name,
            offset,
            length,
        });
    }

    let mut by_position: Vec<&CatalogEntry> = entries.iter().collect();
    by_position.sort_by_key(|entry| (entry.offset, entry.length, entry.index));
    let mut previous_end = table_end;
    for entry in by_position {
        if entry.offset < previous_end {
            return Err(format!(
                "TXTALL.DAT member {} overlaps an earlier member",
                entry.name
            ));
        }
        if entry.length != 0 {
            previous_end = entry.offset + entry.length;
        }
    }

    Ok(Catalog { table_end, entries })
}

pub fn member_data<'a>(bytes: &'a [u8], entry: &CatalogEntry) -> Result<&'a [u8], String> {
    let end = entry
        .offset
        .checked_add(entry.length)
        .ok_or_else(|| format!("member {} range overflows", entry.name))?;
    bytes
        .get(entry.offset..end)
        .ok_or_else(|| format!("member {} is outside the source archive", entry.name))
}

/// Rebuild changed members while retaining the original record names, gaps,
/// and trailer. An empty or byte-identical replacement set returns the source
/// bytes unchanged.
pub fn rebuild(
    original: &[u8],
    replacements: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    let catalog = parse(original)?;
    let known: BTreeSet<&str> = catalog
        .entries
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    for name in replacements.keys() {
        if !known.contains(name.as_str()) {
            return Err(format!("TXTALL.DAT has no member named {name}"));
        }
    }
    let any_change = catalog.entries.iter().any(|entry| {
        replacements.get(&entry.name).is_some_and(|replacement| {
            replacement.as_slice() != &original[entry.offset..entry.offset + entry.length]
        })
    });
    if !any_change {
        return Ok(original.to_vec());
    }

    let mut rebuilt = original[..catalog.table_end].to_vec();
    let mut by_position: Vec<&CatalogEntry> = catalog.entries.iter().collect();
    by_position.sort_by_key(|entry| (entry.offset, entry.length, entry.index));
    let mut source_cursor = catalog.table_end;
    for entry in by_position {
        rebuilt.extend_from_slice(&original[source_cursor..entry.offset]);
        let new_offset = u32::try_from(rebuilt.len())
            .map_err(|_| format!("TXTALL.DAT member {} offset exceeds u32", entry.name))?;
        let data = replacements
            .get(&entry.name)
            .map(Vec::as_slice)
            .unwrap_or(&original[entry.offset..entry.offset + entry.length]);
        let new_length = u32::try_from(data.len())
            .map_err(|_| format!("TXTALL.DAT member {} length exceeds u32", entry.name))?;
        rebuilt.extend_from_slice(data);
        source_cursor = entry.offset + entry.length;
        let offset_pos = HEADER_LEN + entry.index * RECORD_LEN + NAME_LEN;
        rebuilt[offset_pos..offset_pos + 4].copy_from_slice(&new_offset.to_le_bytes());
        rebuilt[offset_pos + 4..offset_pos + 8].copy_from_slice(&new_length.to_le_bytes());
    }
    rebuilt.extend_from_slice(&original[source_cursor..]);
    if rebuilt.len() > u32::MAX as usize {
        return Err("rebuilt TXTALL.DAT exceeds 4 GiB".into());
    }
    Ok(rebuilt)
}
