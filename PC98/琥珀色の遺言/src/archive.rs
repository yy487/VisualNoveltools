use crate::Result;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
pub struct Reference {
    pub table: String,
    pub index: usize,
    pub executable_file_offset: usize,
    pub archive: String,
    pub offset: usize,
    pub size: usize,
    pub resource: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Resource {
    pub path: String,
    pub archive: String,
    pub offset: usize,
    pub size: usize,
    pub sha256: String,
    pub kind: String,
    pub references: Vec<usize>,
    #[serde(skip)]
    pub bytes: Vec<u8>,
}

pub struct Catalog {
    pub references: Vec<Reference>,
    pub resources: Vec<Resource>,
}

/// Each index entry is three little-endian words: offset-high, offset-low, length.
/// Offsets below are file offsets, i.e. COM runtime offsets minus 0x100.
pub fn parse(exe: &[u8], disk1: &[u8], disk2: &[u8]) -> Result<Catalog> {
    let mut references = Vec::new();
    let tables = [
        ("music", 0xaef2, 14, "DISK1.DAT"),
        ("scene", 0xaf46, 44, "DISK1.DAT"),
        ("dialogue", 0xb04e, 31, "DISK1.DAT"),
        ("disk1-opaque", 0xb108, 8, "DISK1.DAT"),
        ("disk2-main", 0xb138, 164, "DISK2.DAT"),
        ("disk2-ui", 0xb510, 9, "DISK2.DAT"),
    ];
    for (name, base, count, archive) in tables {
        for index in 0..count {
            let at = base + index * 6;
            let entry = exe.get(at..at + 6).ok_or("truncated resource index")?;
            let word = |i| usize::from(u16::from_le_bytes([entry[i], entry[i + 1]]));
            references.push(Reference {
                table: name.into(),
                index,
                executable_file_offset: at,
                archive: archive.into(),
                offset: (word(0) << 16) | word(2),
                size: word(4),
                resource: None,
            });
        }
    }
    let mut resources = Vec::new();
    for (archive, disk) in [("DISK1.DAT", disk1), ("DISK2.DAT", disk2)] {
        let mut grouped = BTreeMap::<(usize, usize), Vec<usize>>::new();
        for (i, r) in references
            .iter()
            .enumerate()
            .filter(|(_, r)| r.archive == archive)
        {
            let end = r
                .offset
                .checked_add(r.size)
                .ok_or("resource size overflow")?;
            if end > disk.len() {
                return Err(format!("{archive}: resource {i} outside archive"));
            }
            if r.size == 0 {
                if archive != "DISK2.DAT" || r.index != 0 || r.offset != 0 {
                    return Err("unexpected empty resource".into());
                }
                continue;
            }
            grouped.entry((r.offset, r.size)).or_default().push(i);
        }
        let mut next = 0;
        for ((offset, size), refs) in grouped {
            if offset != next {
                return Err(format!(
                    "{archive}: gap or overlap at {next:#x}/{offset:#x}"
                ));
            }
            next = offset + size;
            let canonical = refs
                .iter()
                .copied()
                .find(|&i| references[i].table == "dialogue")
                .unwrap_or(refs[0]);
            let r = &references[canonical];
            let kind = match r.table.as_str() {
                "scene" if r.index == 41 => "profile-text",
                "scene" if r.index == 42 => "common-text",
                "scene" => "scene-text",
                "dialogue" => "dialogue-text",
                "music" => "music",
                _ => "opaque",
            };
            let path = format!(
                "resources/{}/{}_{:03}.bin",
                archive.trim_end_matches(".DAT"),
                r.table,
                r.index
            );
            for &i in &refs {
                references[i].resource = Some(path.clone());
            }
            let bytes = disk[offset..next].to_vec();
            resources.push(Resource {
                path,
                archive: archive.into(),
                offset,
                size,
                sha256: fivec_new::sha256(&bytes),
                kind: kind.into(),
                references: refs,
                bytes,
            });
        }
        if next != disk.len() {
            return Err(format!("{archive}: unindexed trailing data at {next:#x}"));
        }
    }
    Ok(Catalog {
        references,
        resources,
    })
}
