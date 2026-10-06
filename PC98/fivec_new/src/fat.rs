//! Read-only FAT12, generalized from the validated BPB/chain parsers in
//! `pc98_fdi_unpack/src/lib.rs` and `platinum_star_hdi_tool/src/lib.rs`.
//! All offsets here are volume-relative logical bytes; only ViewReader produces
//! physical source offsets. No assumptions about container sector sizes apply.

use crate::{model::*, view::ViewReader, Result};
use encoding_rs::SHIFT_JIS;
use std::collections::{HashMap, HashSet};

const ENTRY_BYTES: usize = 32;
const DOT: &[u8; 11] = b".          ";
const DOT_DOT: &[u8; 11] = b"..         ";

struct Layout {
    info: Fat12Info,
    root_offset: usize,
    fat_offset: usize,
    fat_bytes: usize,
    cluster_bytes: usize,
    volume_bytes: usize,
    media: u8,
}

/// None means that the BPB does not describe a supported FAT12 volume. Once a
/// coherent FAT12 BPB has been recognized, damaged tables/trees are errors.
pub(crate) fn probe(reader: &ViewReader<'_>) -> Result<Option<FileSystem>> {
    let Some(layout) = read_layout(reader)? else {
        return Ok(None);
    };
    let fat = reader.read(layout.fat_offset, layout.fat_bytes)?;
    if fat[0] < 0xf0
        || fat12_next(&fat, 0)? != (0xf00 | u16::from(fat[0]))
        || fat12_next(&fat, 1)? < 0xff8
    {
        return Err("FAT12 primary FAT has invalid reserved entries".into());
    }
    // Checking the final entry also proves that every data cluster has a FAT slot.
    fat12_next(&fat, (layout.info.data_clusters + 1) as u16)?;
    let mut parser = Parser {
        reader,
        fat,
        layout,
        owners: HashMap::new(),
        files: Vec::new(),
        directories: Vec::new(),
        diagnostics: Vec::new(),
    };
    if parser.layout.media != parser.fat[0] {
        parser.diagnostics.push(format!(
            "BPB media byte {:#04x} differs from primary FAT media byte {:#04x}; original bytes retained",
            parser.layout.media, parser.fat[0]
        ));
    }
    for copy in 1..parser.layout.info.fat_copies {
        let offset = parser.layout.fat_offset + copy * parser.layout.fat_bytes;
        let other = reader.read(offset, parser.layout.fat_bytes)?;
        let mismatch = parser
            .fat
            .iter()
            .zip(&other)
            .filter(|(a, b)| a != b)
            .count();
        if mismatch != 0 {
            parser.layout.info.fat_copies_identical = false;
            parser.diagnostics.push(format!(
                "FAT copy {} differs from primary FAT in {mismatch} bytes; primary FAT used without repair after validating the complete active tree",
                copy + 1
            ));
        }
    }
    if reader.length != parser.layout.volume_bytes {
        parser.diagnostics.push(format!(
            "BPB uses {} of {} available volume bytes; trailing bytes are outside the FAT12 filesystem",
            parser.layout.volume_bytes, reader.length
        ));
    }
    let root = DirectoryTask {
        display_path: String::new(),
        export_path: String::new(),
        raw_components: Vec::new(),
        cluster: 0,
        parent_cluster: 0,
        ranges: vec![(
            parser.layout.root_offset,
            parser.layout.info.root_entries * ENTRY_BYTES,
        )],
    };
    // An explicit work stack supports arbitrarily deep valid trees without a
    // Rust call-stack overflow. Cluster ownership bounds the number of directories.
    let mut pending = vec![root];
    while let Some(directory) = pending.pop() {
        parser.parse_directory(directory, &mut pending)?;
    }
    let mut unreferenced = 0;
    let mut bad_clusters = 0;
    for cluster in 2..=parser.layout.info.data_clusters + 1 {
        let next = fat12_next(&parser.fat, cluster as u16)?;
        if next == 0xff7 {
            bad_clusters += 1;
        } else if next != 0 && !parser.owners.contains_key(&(cluster as u16)) {
            unreferenced += 1;
        }
    }
    if unreferenced != 0 {
        parser.diagnostics.push(format!(
            "{unreferenced} allocated clusters are not referenced by the active directory tree; no orphan recovery was attempted"
        ));
    }
    if bad_clusters != 0 {
        parser.diagnostics.push(format!(
            "{bad_clusters} clusters are marked bad outside the validated active tree"
        ));
    }
    Ok(Some(FileSystem {
        kind: "FAT12".into(),
        fat12: parser.layout.info,
        files: parser.files,
        directories: parser.directories,
        diagnostics: parser.diagnostics,
    }))
}

fn read_layout(reader: &ViewReader<'_>) -> Result<Option<Layout>> {
    if reader.length < 36 {
        return Ok(None);
    }
    let boot = reader.read(0, 36)?;
    let bps = usize::from(word(&boot, 11));
    let spc = usize::from(boot[13]);
    let reserved = usize::from(word(&boot, 14));
    let copies = usize::from(boot[16]);
    let root_entries = usize::from(word(&boot, 17));
    let total16 = usize::from(word(&boot, 19));
    let spf = usize::from(word(&boot, 22));
    let total = if total16 == 0 {
        dword(&boot, 32) as usize
    } else {
        total16
    };
    if !(128..=4096).contains(&bps)
        || !bps.is_power_of_two()
        || !spc.is_power_of_two()
        || reserved == 0
        || copies == 0
        || root_entries == 0
        || spf == 0
        || total == 0
    {
        return Ok(None);
    }
    let root_sectors = (root_entries * ENTRY_BYTES).div_ceil(bps);
    let root_sector = copies
        .checked_mul(spf)
        .and_then(|n| n.checked_add(reserved))
        .ok_or("FAT12 metadata layout overflows")?;
    let data_sector = root_sector
        .checked_add(root_sectors)
        .ok_or("FAT12 data sector overflows")?;
    let data_sectors = total
        .checked_sub(data_sector)
        .ok_or("FAT12 metadata exceeds the BPB volume")?;
    let clusters = data_sectors / spc;
    if clusters >= 4085 {
        return Ok(None); // FAT16/32 are deliberately not advertised as supported.
    }
    if clusters == 0 {
        return Err("FAT12 BPB has no complete data cluster".into());
    }
    let volume_bytes = total
        .checked_mul(bps)
        .ok_or("FAT12 volume length overflows")?;
    if volume_bytes > reader.length {
        return Err(format!(
            "FAT12 BPB volume is truncated: needs {volume_bytes} bytes, has {}",
            reader.length
        ));
    }
    Ok(Some(Layout {
        info: Fat12Info {
            bytes_per_sector: bps,
            sectors_per_cluster: spc,
            reserved_sectors: reserved,
            fat_copies: copies,
            sectors_per_fat: spf,
            root_entries,
            total_sectors: total,
            first_data_sector: data_sector,
            data_clusters: clusters,
            fat_copies_identical: true,
            deleted_entries_skipped: 0,
            long_name_entries_skipped: 0,
        },
        root_offset: root_sector * bps,
        fat_offset: reserved * bps,
        fat_bytes: spf * bps,
        cluster_bytes: spc * bps,
        volume_bytes,
        media: boot[21],
    }))
}

struct DirectoryTask {
    display_path: String,
    export_path: String,
    raw_components: Vec<String>,
    cluster: u16,
    parent_cluster: u16,
    ranges: Vec<(usize, usize)>,
}

struct Parser<'r, 's> {
    reader: &'r ViewReader<'s>,
    fat: Vec<u8>,
    layout: Layout,
    owners: HashMap<u16, usize>,
    files: Vec<FileEntry>,
    directories: Vec<DirectoryEntry>,
    diagnostics: Vec<String>,
}

impl Parser<'_, '_> {
    fn parse_directory(
        &mut self,
        directory: DirectoryTask,
        pending: &mut Vec<DirectoryTask>,
    ) -> Result<()> {
        let mut raw_names = HashSet::new();
        let mut host_names = HashSet::new();
        let mut dot_seen = false;
        let mut dot_dot_seen = false;
        'ranges: for &(offset, length) in &directory.ranges {
            let data = self.reader.read(offset, length)?;
            for (index, entry) in data.chunks_exact(ENTRY_BYTES).enumerate() {
                let entry_offset = offset + index * ENTRY_BYTES;
                match entry[0] {
                    0 => break 'ranges,
                    0xe5 => {
                        self.layout.info.deleted_entries_skipped += 1;
                        continue;
                    }
                    _ => {}
                }
                let attributes = entry[11];
                if attributes == 0x0f {
                    self.layout.info.long_name_entries_skipped += 1;
                    continue;
                }
                if attributes & 0xc0 != 0 {
                    return Err(format!("entry at {entry_offset:#x} has unsupported reserved attribute bits {attributes:#04x}"));
                }
                let raw_name: [u8; 11] = entry[..11].try_into().expect("11-byte short name");
                if attributes & 0x08 == 0 && !raw_names.insert(raw_name) {
                    return Err(format!(
                        "directory {:?}: duplicate raw short name {} at {entry_offset:#x}",
                        directory.display_path,
                        crate::hex(&raw_name)
                    ));
                }
                let start = word(entry, 26);
                let size = dword(entry, 28) as usize;
                if &raw_name == DOT || &raw_name == DOT_DOT {
                    let expected = if &raw_name == DOT {
                        directory.cluster
                    } else {
                        directory.parent_cluster
                    };
                    if directory.cluster == 0
                        || attributes & 0x18 != 0x10
                        || size != 0
                        || start != expected
                    {
                        return Err(format!("invalid . or .. directory reference at {entry_offset:#x}: cluster {start}, expected {expected}"));
                    }
                    if &raw_name == DOT {
                        dot_seen = true;
                    } else {
                        dot_dot_seen = true;
                    }
                    continue;
                }
                if attributes & 0x08 != 0 {
                    if attributes & 0x10 != 0 {
                        return Err(format!("entry at {entry_offset:#x} has both volume-label and directory attributes"));
                    }
                    if directory.cluster != 0 {
                        return Err(format!(
                            "volume label at {entry_offset:#x} is outside the root directory"
                        ));
                    }
                    // Volume labels are metadata, never ordinary files.
                    continue;
                }
                let raw_hex = crate::hex(&raw_name);
                let decoded = decode_name(&raw_name);
                let fallback = format!("__raw_{raw_hex}");
                let display = decoded.clone().unwrap_or_else(|| fallback.clone());
                let host = match decoded.filter(|name| safe_component(name)) {
                    Some(name) if host_names.insert(name.to_uppercase()) => name,
                    _ => {
                        if !host_names.insert(fallback.to_uppercase()) {
                            return Err(format!("duplicate host mapping at {entry_offset:#x}"));
                        }
                        self.diagnostics.push(format!(
                            "directory entry {entry_offset:#x} uses reversible host name {fallback} (CP932 round-trip, Windows name, or case collision)"
                        ));
                        fallback
                    }
                };
                let display_path = join(&directory.display_path, &display);
                let export_path = join(&directory.export_path, &host);
                let mut raw_components = directory.raw_components.clone();
                raw_components.push(raw_hex);
                if attributes & 0x10 != 0 {
                    if size != 0 {
                        return Err(format!("directory {display_path:?} at {entry_offset:#x} has nonzero file size {size}"));
                    }
                    let chain = self.read_chain(start, entry_offset)?;
                    let ranges = chain
                        .iter()
                        .map(|&cluster| {
                            self.cluster_offset(cluster)
                                .map(|offset| (offset, self.layout.cluster_bytes))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    self.directories.push(DirectoryEntry {
                        display_path: display_path.clone(),
                        export_path: export_path.clone(),
                        raw_name_components_hex: raw_components.clone(),
                        directory_entry_offset: entry_offset,
                        clusters: chain.iter().copied().map(u32::from).collect(),
                    });
                    pending.push(DirectoryTask {
                        display_path,
                        export_path,
                        raw_components,
                        cluster: start,
                        parent_cluster: directory.cluster,
                        ranges,
                    });
                } else {
                    let chain = if size == 0 {
                        if start != 0 {
                            return Err(format!(
                                "empty file at {entry_offset:#x} has nonzero start cluster {start}"
                            ));
                        }
                        Vec::new()
                    } else {
                        self.read_chain(start, entry_offset)?
                    };
                    let needed = size.div_ceil(self.layout.cluster_bytes);
                    if chain.len() != needed {
                        return Err(format!("file {display_path:?} at {entry_offset:#x}: size {size} needs {needed} clusters, primary FAT chain has {}", chain.len()));
                    }
                    let mut source_ranges = Vec::new();
                    let mut cursor = 0;
                    for &cluster in &chain {
                        let length = (size - cursor).min(self.layout.cluster_bytes);
                        let offset = self.cluster_offset(cluster)?;
                        for mut span in self.reader.spans(offset, length)? {
                            span.file_offset += cursor;
                            source_ranges.push(span);
                        }
                        cursor += length;
                    }
                    let data = crate::read_spans(self.reader.source, &source_ranges, size)?;
                    self.files.push(FileEntry {
                        id: format!("entry-{entry_offset:08x}"),
                        display_path,
                        export_path,
                        raw_name_components_hex: raw_components,
                        attributes,
                        directory_entry_offset: entry_offset,
                        directory_entry_source_ranges: self
                            .reader
                            .spans(entry_offset, ENTRY_BYTES)?,
                        size,
                        sha256: crate::sha256(&data),
                        clusters: chain.into_iter().map(u32::from).collect(),
                        source_ranges,
                    });
                }
            }
        }
        if directory.cluster != 0 && (!dot_seen || !dot_dot_seen) {
            return Err(format!(
                "subdirectory {:?} is missing . or .. entries",
                directory.display_path
            ));
        }
        Ok(())
    }

    fn read_chain(&mut self, start: u16, owner: usize) -> Result<Vec<u16>> {
        let mut chain = Vec::new();
        let mut seen = HashSet::new();
        let mut cluster = start;
        loop {
            if cluster < 2
                || usize::from(cluster) > self.layout.info.data_clusters + 1
                || cluster >= 0xff0
            {
                return Err(format!("entry {owner:#x}: cluster {cluster:#05x} is outside the usable FAT12 data range"));
            }
            if !seen.insert(cluster) {
                return Err(format!(
                    "entry {owner:#x}: FAT12 chain loops at cluster {cluster}"
                ));
            }
            if let Some(previous) = self.owners.insert(cluster, owner) {
                return Err(format!("entry {owner:#x}: cluster {cluster} is cross-linked with directory entry {previous:#x}"));
            }
            chain.push(cluster);
            let next = fat12_next(&self.fat, cluster)?;
            match next {
                0xff8..=0xfff => return Ok(chain),
                0xff7 => {
                    return Err(format!(
                        "entry {owner:#x}: chain reaches a bad cluster marker at {cluster}"
                    ))
                }
                0xff0..=0xff6 => {
                    return Err(format!(
                        "entry {owner:#x}: chain reaches reserved marker {next:#05x}"
                    ))
                }
                0 | 1 => {
                    return Err(format!(
                    "entry {owner:#x}: chain reaches unallocated/reserved cluster marker {next}"
                ))
                }
                _ => cluster = next,
            }
        }
    }

    fn cluster_offset(&self, cluster: u16) -> Result<usize> {
        let offset = self.layout.info.first_data_sector * self.layout.info.bytes_per_sector
            + (usize::from(cluster) - 2) * self.layout.cluster_bytes;
        if offset
            .checked_add(self.layout.cluster_bytes)
            .is_none_or(|end| end > self.layout.volume_bytes)
        {
            return Err(format!("cluster {cluster} exceeds the FAT12 BPB volume"));
        }
        Ok(offset)
    }
}

fn fat12_next(fat: &[u8], cluster: u16) -> Result<u16> {
    let offset = usize::from(cluster) * 3 / 2;
    let pair = fat
        .get(offset..offset + 2)
        .ok_or_else(|| format!("FAT12 table has no slot for cluster {cluster}"))?;
    let packed = u16::from_le_bytes([pair[0], pair[1]]);
    Ok(if cluster & 1 == 0 {
        packed & 0x0fff
    } else {
        packed >> 4
    })
}

fn word(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn dword(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn join(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}/{name}")
    }
}

fn decode_name(raw: &[u8; 11]) -> Option<String> {
    let mut stem = raw[..8].to_vec();
    if stem[0] == 0x05 {
        stem[0] = 0xe5;
    }
    while stem.last() == Some(&b' ') {
        stem.pop();
    }
    let mut extension = &raw[8..];
    while extension.last() == Some(&b' ') {
        extension = &extension[..extension.len() - 1];
    }
    let decode = |bytes: &[u8]| -> Option<String> {
        let decoded = SHIFT_JIS.decode_without_bom_handling_and_without_replacement(bytes)?;
        let (encoded, _, errors) = SHIFT_JIS.encode(&decoded);
        if errors || encoded.as_ref() != bytes {
            return None;
        }
        Some(decoded.into_owned())
    };
    let stem = decode(&stem)?;
    let extension = decode(extension)?;
    Some(if extension.is_empty() {
        stem
    } else {
        format!("{stem}.{extension}")
    })
}

fn safe_component(name: &str) -> bool {
    if name.is_empty()
        || matches!(name, "." | "..")
        || name.ends_with([' ', '.'])
        || name.chars().any(|c| {
            c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
        })
    {
        return false;
    }
    let stem = name
        .split('.')
        .next()
        .unwrap_or(name)
        .trim_end_matches(' ')
        .to_uppercase();
    if matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$" | "CLOCK$"
    ) {
        return false;
    }
    for prefix in ["COM", "LPT"] {
        if let Some(suffix) = stem.strip_prefix(prefix) {
            if matches!(
                suffix,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            ) {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use vn_sector_map::{LinearView, SectorAddress, SectorId, ViewSegment};

    struct Fixture {
        bytes: Vec<u8>,
        bps: usize,
        spc: usize,
        fat: usize,
        fat_bytes: usize,
        root: usize,
        data: usize,
    }

    impl Fixture {
        fn new(bps: usize, spc: usize, total: usize, root_entries: usize) -> Self {
            let reserved = 3;
            let copies = 2;
            let spf = 4;
            let root_sector = reserved + copies * spf;
            let data_sector = root_sector + (root_entries * ENTRY_BYTES).div_ceil(bps);
            let mut bytes = vec![0; total * bps];
            put_word(&mut bytes, 11, bps as u16);
            bytes[13] = spc as u8;
            put_word(&mut bytes, 14, reserved as u16);
            bytes[16] = copies as u8;
            put_word(&mut bytes, 17, root_entries as u16);
            put_word(&mut bytes, 19, total as u16);
            bytes[21] = 0xf8;
            put_word(&mut bytes, 22, spf as u16);
            for copy in 0..copies {
                let offset = (reserved + copy * spf) * bps;
                bytes[offset..offset + 3].copy_from_slice(&[0xf8, 0xff, 0xff]);
            }
            Self {
                bytes,
                bps,
                spc,
                fat: reserved * bps,
                fat_bytes: spf * bps,
                root: root_sector * bps,
                data: data_sector * bps,
            }
        }

        fn standard() -> Self {
            Self::new(512, 2, 2048, 145)
        }

        fn cluster(&self, cluster: u16) -> usize {
            self.data + (usize::from(cluster) - 2) * self.bps * self.spc
        }

        fn set_chain(&mut self, chain: &[u16]) {
            for (index, &cluster) in chain.iter().enumerate() {
                self.set_fat(cluster, chain.get(index + 1).copied().unwrap_or(0xfff));
            }
        }

        fn set_fat(&mut self, cluster: u16, value: u16) {
            for copy in 0..2 {
                set_fat(
                    &mut self.bytes[self.fat + copy * self.fat_bytes..][..self.fat_bytes],
                    cluster,
                    value,
                );
            }
        }

        fn file(&mut self, offset: usize, name: &[u8; 11], chain: &[u16], data: &[u8]) {
            put_entry(
                &mut self.bytes,
                offset,
                name,
                0x20,
                chain.first().copied().unwrap_or(0),
                data.len() as u32,
            );
            self.set_chain(chain);
            for (&cluster, chunk) in chain.iter().zip(data.chunks(self.bps * self.spc)) {
                let offset = self.cluster(cluster);
                self.bytes[offset..offset + chunk.len()].copy_from_slice(chunk);
            }
        }

        fn directory(&mut self, entry: usize, name: &[u8; 11], cluster: u16, parent: u16) {
            put_entry(&mut self.bytes, entry, name, 0x10, cluster, 0);
            self.set_chain(&[cluster]);
            let offset = self.cluster(cluster);
            put_entry(&mut self.bytes, offset, DOT, 0x10, cluster, 0);
            put_entry(
                &mut self.bytes,
                offset + ENTRY_BYTES,
                DOT_DOT,
                0x10,
                parent,
                0,
            );
        }

        fn inspect(&self) -> Result<Option<FileSystem>> {
            inspect_bytes(&self.bytes)
        }
    }

    fn put_word(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn put_dword(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn put_entry(
        bytes: &mut [u8],
        offset: usize,
        name: &[u8; 11],
        attributes: u8,
        cluster: u16,
        size: u32,
    ) {
        bytes[offset..offset + 11].copy_from_slice(name);
        bytes[offset + 11] = attributes;
        put_word(bytes, offset + 26, cluster);
        put_dword(bytes, offset + 28, size);
    }
    fn set_fat(bytes: &mut [u8], cluster: u16, value: u16) {
        let offset = usize::from(cluster) * 3 / 2;
        let old = word(bytes, offset);
        let packed = if cluster & 1 == 0 {
            (old & 0xf000) | value
        } else {
            (old & 0x000f) | (value << 4)
        };
        put_word(bytes, offset, packed);
    }
    fn segment(offset: usize, length: usize, ordinal: usize) -> ViewSegment {
        ViewSegment {
            sector: SectorAddress {
                track_slot: ordinal / 16,
                physical_ordinal: ordinal % 16,
                id: SectorId {
                    cylinder: (ordinal / 16) as u16,
                    head: 0,
                    record: (ordinal % 16 + 1) as u16,
                    size_code: 1,
                },
            },
            source_range: offset..offset + length,
        }
    }
    fn inspect_bytes(bytes: &[u8]) -> Result<Option<FileSystem>> {
        let view = LinearView::new("test", bytes.len(), vec![segment(0, bytes.len(), 0)]).unwrap();
        probe(&ViewReader::new(bytes, &view, 0, bytes.len()).unwrap())
    }

    #[test]
    fn bpb_drives_nondefault_layout_cluster_size_and_complete_long_file() {
        let mut fixture = Fixture::standard();
        let payload: Vec<u8> = (0..672_000).map(|n| (n * 17 + 3) as u8).collect();
        let chain: Vec<u16> = (2..2 + payload.len().div_ceil(1024) as u16).collect();
        fixture.file(fixture.root, b"LARGE   BIN", &chain, &payload);
        let fs = fixture.inspect().unwrap().unwrap();
        assert_eq!(fs.fat12.first_data_sector, 21);
        assert_eq!(fs.fat12.sectors_per_cluster, 2);
        assert_eq!(fs.files[0].size, 672_000);
        assert_eq!(fs.files[0].clusters.len(), 657);
        assert_eq!(
            crate::read_spans(&fixture.bytes, &fs.files[0].source_ranges, payload.len()).unwrap(),
            payload
        );
        assert_eq!(fs.files[0].sha256, crate::sha256(&payload));
    }

    #[test]
    fn root_has_no_hundred_entry_limit_and_skips_deleted_lfn_and_label() {
        let mut fixture = Fixture::standard();
        for index in 0..130 {
            let name: [u8; 11] = format!("F{index:07}BIN").as_bytes().try_into().unwrap();
            fixture.file(fixture.root + index * ENTRY_BYTES, &name, &[], &[]);
        }
        fixture.bytes[fixture.root + 130 * ENTRY_BYTES] = 0xe5;
        put_entry(
            &mut fixture.bytes,
            fixture.root + 131 * ENTRY_BYTES,
            b"LONGNAME   ",
            0x0f,
            0,
            0,
        );
        put_entry(
            &mut fixture.bytes,
            fixture.root + 132 * ENTRY_BYTES,
            b"LABEL      ",
            0x08,
            0,
            0,
        );
        let fs = fixture.inspect().unwrap().unwrap();
        assert_eq!(fs.files.len(), 130);
        assert_eq!(fs.fat12.deleted_entries_skipped, 1);
        assert_eq!(fs.fat12.long_name_entries_skipped, 1);
        assert!(fs.files[0].source_ranges.is_empty());
    }

    #[test]
    fn reads_nested_directories_and_fragmented_files_with_valid_dot_references() {
        let mut fixture = Fixture::standard();
        fixture.directory(fixture.root, b"GAME       ", 2, 0);
        fixture.directory(fixture.cluster(2) + 64, b"SUBDIR     ", 3, 2);
        let payload = vec![0x5a; 1900];
        fixture.file(fixture.cluster(3) + 64, b"SCRIPT  DAT", &[7, 5], &payload);
        let fs = fixture.inspect().unwrap().unwrap();
        assert_eq!(fs.directories.len(), 2);
        assert_eq!(fs.files[0].display_path, "GAME/SUBDIR/SCRIPT.DAT");
        assert_eq!(fs.files[0].clusters, [7, 5]);
        assert_eq!(fs.files[0].raw_name_components_hex.len(), 3);
        assert_eq!(
            crate::read_spans(&fixture.bytes, &fs.files[0].source_ranges, 1900).unwrap(),
            payload
        );
    }

    #[test]
    fn fragmented_directory_continues_into_its_next_cluster() {
        let mut fixture = Fixture::standard();
        fixture.directory(fixture.root, b"DIR        ", 2, 0);
        fixture.set_chain(&[2, 6]);
        let first = fixture.cluster(2);
        for offset in (64..1024).step_by(ENTRY_BYTES) {
            fixture.bytes[first + offset] = 0xe5;
        }
        fixture.file(fixture.cluster(6), b"LATER   DAT", &[3], b"late file");
        let fs = fixture.inspect().unwrap().unwrap();
        assert_eq!(fs.directories[0].clusters, [2, 6]);
        assert_eq!(fs.files[0].display_path, "DIR/LATER.DAT");
        assert_eq!(fs.fat12.deleted_entries_skipped, 30);
    }

    #[test]
    fn deep_directory_tree_uses_an_explicit_work_stack() {
        let mut fixture = Fixture::standard();
        let mut entry = fixture.root;
        let mut parent = 0;
        for cluster in 2..82 {
            fixture.directory(entry, b"D          ", cluster, parent);
            entry = fixture.cluster(cluster) + 64;
            parent = cluster;
        }
        fixture.file(entry, b"END     DAT", &[], &[]);
        let fs = fixture.inspect().unwrap().unwrap();
        assert_eq!(fs.directories.len(), 80);
        assert_eq!(fs.files[0].raw_name_components_hex.len(), 81);
    }

    #[test]
    fn physical_sector_size_container_headers_and_volume_base_do_not_change_fat_layout() {
        let mut fixture = Fixture::new(1024, 1, 128, 64);
        let payload = vec![0x93; 1537];
        fixture.file(fixture.root, b"CONTENT BIN", &[5, 3], &payload);
        let mut logical = vec![0; 256];
        logical.extend_from_slice(&fixture.bytes);
        let mut source = vec![0; 37];
        let mut segments = Vec::new();
        for (ordinal, data) in logical.chunks(256).enumerate() {
            source.extend_from_slice(&[0xcc; 16]);
            let offset = source.len();
            source.extend_from_slice(data);
            segments.push(segment(offset, data.len(), ordinal));
        }
        let view = LinearView::new("noncontiguous", source.len(), segments).unwrap();
        let reader = ViewReader::new(&source, &view, 256, fixture.bytes.len()).unwrap();
        let fs = probe(&reader).unwrap().unwrap();
        let file = &fs.files[0];
        assert_eq!(file.source_ranges.len(), 7);
        assert_ne!(file.source_ranges[0].source_offset, fixture.cluster(5));
        assert_eq!(
            crate::read_spans(&source, &file.source_ranges, payload.len()).unwrap(),
            payload
        );
        assert_eq!(
            crate::read_spans(&source, &file.directory_entry_source_ranges, ENTRY_BYTES).unwrap(),
            fixture.bytes[fixture.root..fixture.root + ENTRY_BYTES]
        );
    }

    #[test]
    fn accepts_bpb_total32_and_reports_differing_backup_and_media() {
        let mut fixture = Fixture::standard();
        put_word(&mut fixture.bytes, 19, 0);
        put_dword(&mut fixture.bytes, 32, 2048);
        fixture.bytes[21] = 0xf0;
        fixture.bytes[fixture.fat + fixture.fat_bytes + 20] = 0x12;
        let fs = fixture.inspect().unwrap().unwrap();
        assert_eq!(fs.fat12.total_sectors, 2048);
        assert!(!fs.fat12.fat_copies_identical);
        assert!(fs.diagnostics.iter().any(|s| s.contains("media byte")));
        assert!(fs.diagnostics.iter().any(|s| s.contains("FAT copy 2")));
    }

    #[test]
    fn unknown_and_fat16_bpbs_are_not_claimed() {
        assert!(inspect_bytes(&[0; 35]).unwrap().is_none());
        assert!(inspect_bytes(&[0; 128]).unwrap().is_none());
        let mut fixture = Fixture::standard();
        fixture.bytes[13] = 3;
        assert!(fixture.inspect().unwrap().is_none());
        fixture.bytes[13] = 1;
        put_word(&mut fixture.bytes, 19, 5000);
        assert!(fixture.inspect().unwrap().is_none());
    }

    #[test]
    fn coherent_bpb_with_truncation_metadata_overflow_or_small_fat_is_rejected() {
        let mut fixture = Fixture::standard();
        fixture.bytes.pop();
        assert!(fixture.inspect().unwrap_err().contains("truncated"));
        let mut fixture = Fixture::standard();
        put_word(&mut fixture.bytes, 19, 10);
        assert!(fixture.inspect().unwrap_err().contains("metadata exceeds"));
        let mut fixture = Fixture::standard();
        put_word(&mut fixture.bytes, 22, 1);
        assert!(fixture.inspect().unwrap_err().contains("no slot"));
    }

    #[test]
    fn rejects_loop_bad_free_reserved_and_out_of_range_chain_links() {
        for (next, message) in [
            (2, "loops"),
            (0xff7, "bad cluster"),
            (0, "unallocated"),
            (1, "reserved"),
            (0xff0, "reserved marker"),
            (2000, "outside"),
        ] {
            let mut fixture = Fixture::standard();
            fixture.file(fixture.root, b"FILE    BIN", &[2], b"a");
            fixture.set_fat(2, next);
            assert!(
                fixture.inspect().unwrap_err().contains(message),
                "next={next:#x}"
            );
        }
    }

    #[test]
    fn rejects_crosslinks_size_mismatch_and_empty_allocated_file() {
        let mut fixture = Fixture::standard();
        fixture.file(fixture.root, b"FIRST   BIN", &[2], b"a");
        fixture.file(fixture.root + ENTRY_BYTES, b"SECOND  BIN", &[2], b"a");
        assert!(fixture.inspect().unwrap_err().contains("cross-linked"));
        for size in [0, 1025] {
            let mut fixture = Fixture::standard();
            fixture.file(fixture.root, b"FILE    BIN", &[2], b"a");
            put_dword(&mut fixture.bytes, fixture.root + 28, size);
            let error = fixture.inspect().unwrap_err();
            assert!(error.contains(if size == 0 {
                "empty file"
            } else {
                "needs 2 clusters"
            }));
        }
    }

    #[test]
    fn invalid_dot_reference_and_directory_alias_are_rejected() {
        let mut fixture = Fixture::standard();
        fixture.directory(fixture.root, b"DIR        ", 2, 0);
        let dotdot = fixture.cluster(2) + ENTRY_BYTES;
        put_word(&mut fixture.bytes, dotdot + 26, 2);
        assert!(fixture.inspect().unwrap_err().contains("invalid . or .."));
        let mut fixture = Fixture::standard();
        fixture.directory(fixture.root, b"DIR        ", 2, 0);
        let offset = fixture.cluster(2) + 64;
        put_entry(&mut fixture.bytes, offset, b"SELF       ", 0x10, 2, 0);
        assert!(fixture.inspect().unwrap_err().contains("cross-linked"));
    }

    #[test]
    fn names_preserve_cp932_bytes_and_map_unsafe_or_colliding_names_reversibly() {
        let mut fixture = Fixture::standard();
        let names = [
            *b"HELLO   TXT",
            *b"hello   txt",
            *b"NUL     TXT",
            *b"BAD\\NAMEBIN",
            [
                0x81, b' ', b' ', b' ', b' ', b' ', b' ', b' ', b'B', b'I', b'N',
            ],
        ];
        for (index, name) in names.iter().enumerate() {
            fixture.file(fixture.root + index * ENTRY_BYTES, name, &[], &[]);
        }
        let (encoded, _, errors) = SHIFT_JIS.encode("日本");
        assert!(!errors);
        let mut japanese = *b"        DAT";
        japanese[..encoded.len()].copy_from_slice(&encoded);
        fixture.file(fixture.root + 5 * ENTRY_BYTES, &japanese, &[], &[]);
        let fs = fixture.inspect().unwrap().unwrap();
        assert_eq!(fs.files[0].export_path, "HELLO.TXT");
        for (file, raw) in fs.files[1..5].iter().zip(&names[1..]) {
            assert_eq!(file.export_path, format!("__raw_{}", crate::hex(raw)));
            assert_eq!(file.raw_name_components_hex, [crate::hex(raw)]);
        }
        assert_eq!(fs.files[5].display_path, "日本.DAT");
        assert_eq!(fs.files[5].export_path, "日本.DAT");
    }

    #[test]
    fn duplicate_raw_names_and_corrupt_primary_are_not_repaired() {
        let mut fixture = Fixture::standard();
        fixture.file(fixture.root, b"FILE    BIN", &[], &[]);
        fixture.file(fixture.root + ENTRY_BYTES, b"FILE    BIN", &[], &[]);
        assert!(fixture.inspect().unwrap_err().contains("duplicate raw"));
        let mut fixture = Fixture::standard();
        fixture.bytes[fixture.fat + 2] = 0;
        assert!(fixture.inspect().unwrap_err().contains("primary FAT"));
    }
}
