#![forbid(unsafe_code)]

pub mod boot_memory;
pub mod codec;
pub mod dictionary;
pub mod hdi;
pub mod install;
mod output;
pub mod workflow;
pub use output::Job;

use codec::Record;
use fivec_new::{Archive, FileSpan};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub type Result<T> = std::result::Result<T, String>;
pub const SCHEMA: &str = "uma-pc98-primary-v3";
pub const DISKS: [&str; 4] = ["01_DISK.D88", "02_DISK.D88", "03_DISK.D88", "04_DISK.D88"];
const MES: [(&str, usize); 8] = [
    ("CHSALES.MES", 40),
    ("HEVENT.MES", 121),
    ("HJOIN.MES", 16),
    ("HMAIN.MES", 88),
    ("HTRAIN.MES", 160),
    ("JMAIN.MES", 11),
    ("PADDOCK.MES", 96),
    ("RRESULT.MES", 32),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub path: String,
    pub mes: bool,
    #[serde(default)]
    pub xor_width: Option<usize>,
    pub records: Vec<Record>,
}
impl Unit {
    pub fn translation_path(&self) -> PathBuf {
        PathBuf::from(format!("{}.json", self.path))
    }
    pub fn entries(&self) -> Vec<vn_text::Entry> {
        self.records
            .iter()
            .map(|r| vn_text::Entry {
                message: r.message.clone(),
                name: None,
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileIdentity {
    pub path: String,
    pub size: usize,
    pub ranges: Vec<FileSpan>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiskIdentity {
    pub name: String,
    pub size: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Primary {
    pub schema: String,
    pub disks: Vec<DiskIdentity>,
    pub files: Vec<FileIdentity>,
    pub units: Vec<Unit>,
}
pub struct Disk {
    pub name: String,
    pub bytes: Vec<u8>,
    pub files: Vec<FileIdentity>,
}
pub struct Game {
    pub disks: Vec<Disk>,
    pub files: BTreeMap<String, Vec<u8>>,
    pub primary: Primary,
}

#[derive(Deserialize)]
struct ExeCatalog {
    disk: String,
    file: String,
    size: usize,
    slots: Vec<ExeSlot>,
}
#[derive(Deserialize)]
struct ExeSlot {
    offset: usize,
    hex: String,
}

pub fn unhex(s: &str) -> Result<Vec<u8>> {
    if !s.len().is_multiple_of(2) || !s.is_ascii() {
        return Err("十六进制字节格式错误".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|p| u8::from_str_radix(&s[p..p + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

pub fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}
pub fn json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec_pretty(value)
        .map(|mut b| {
            b.push(b'\n');
            b
        })
        .map_err(|e| e.to_string())
}

/// The source is an immutable snapshot; use the shared validated FAT source map,
/// with bounds/coverage checks and no hash comparison.
fn mapped_file(source: &[u8], file: &FileIdentity) -> Result<Vec<u8>> {
    let mut data = Vec::with_capacity(file.size);
    for span in &file.ranges {
        if span.file_offset != data.len() {
            return Err("文件来源范围不连续".into());
        }
        let end = span
            .source_offset
            .checked_add(span.length)
            .ok_or("来源范围溢出")?;
        data.extend_from_slice(
            source
                .get(span.source_offset..end)
                .ok_or("来源范围超出磁盘镜像")?,
        );
    }
    if data.len() != file.size {
        return Err("文件来源范围与目录项大小不一致".into());
    }
    Ok(data)
}

pub fn patch_file(image: &mut [u8], file: &FileIdentity, data: &[u8]) -> Result<()> {
    if data.len() != file.size {
        return Err("封盘只接受保持原文件大小的内容".into());
    }
    let mut covered = 0;
    let mut destinations = Vec::new();
    // Validate the whole write before changing any bytes, including for library callers.
    for span in &file.ranges {
        if span.file_offset != covered {
            return Err("回写来源范围重叠或有缺口".into());
        }
        let end = span
            .source_offset
            .checked_add(span.length)
            .ok_or("回写范围溢出")?;
        covered = covered.checked_add(span.length).ok_or("文件范围溢出")?;
        if end > image.len() || covered > data.len() {
            return Err("回写范围越界".into());
        }
        destinations.push((span.source_offset, end));
    }
    if covered != data.len() {
        return Err("回写范围没有覆盖完整文件".into());
    }
    destinations.sort_unstable();
    if destinations.windows(2).any(|p| p[0].1 > p[1].0) {
        return Err("回写的物理范围发生重叠".into());
    }
    for span in &file.ranges {
        image[span.source_offset..span.source_offset + span.length]
            .copy_from_slice(&data[span.file_offset..span.file_offset + span.length]);
    }
    Ok(())
}

pub fn load_game(source: &Path) -> Result<Game> {
    let mut disks = Vec::new();
    let mut files = BTreeMap::new();
    let mut identities = Vec::new();
    for name in DISKS {
        let bytes = read(&source.join(name))?;
        let archive = Archive::from_bytes(name, bytes.clone())?;
        if archive.inspection().format != "d88" {
            return Err(format!("{name}: 不是已支持的 D88"));
        }
        let volumes = archive.selected_volumes("disk-000-whole")?;
        let filesystem = volumes[0].filesystem.as_ref().ok_or("不支持的文件系统")?;
        let stem = name.strip_suffix(".D88").ok_or("磁盘名称错误")?;
        let mut disk_files = Vec::new();
        for entry in &filesystem.files {
            let identity = FileIdentity {
                path: format!("{stem}/{}", entry.export_path),
                size: entry.size,
                ranges: entry.source_ranges.clone(),
            };
            if files
                .insert(identity.path.clone(), mapped_file(&bytes, &identity)?)
                .is_some()
            {
                return Err("重复文件路径".into());
            }
            identities.push(identity.clone());
            disk_files.push(identity);
        }
        disks.push(Disk {
            name: name.into(),
            bytes,
            files: disk_files,
        });
    }
    identities.sort_by(|a, b| a.path.cmp(&b.path));
    let mut units = Vec::new();
    for (name, count) in MES {
        let path = format!("03_DISK/{name}");
        let bytes = files.get(&path).ok_or_else(|| format!("缺少 {path}"))?;
        units.push(Unit {
            path,
            mes: true,
            xor_width: None,
            records: codec::mes_records(bytes, count)?,
        });
    }
    for (name, count) in [("EHPARTS.NPA", 1119), ("EHPARTS.NPB", 1552)] {
        let path = format!("03_DISK/{name}");
        let bytes = files.get(&path).ok_or_else(|| format!("缺少 {path}"))?;
        units.push(Unit {
            path,
            mes: false,
            xor_width: Some(14),
            records: dictionary::records(bytes, count, 14)?,
        });
    }
    let path = "03_DISK/JNAME.DAT".to_owned();
    let bytes = files.get(&path).ok_or_else(|| format!("缺少 {path}"))?;
    units.push(Unit {
        path,
        mes: false,
        xor_width: Some(8),
        records: dictionary::records(bytes, 210, 8)?,
    });
    let path = "03_DISK/RNAME.DAT".to_owned();
    let bytes = files.get(&path).ok_or_else(|| format!("缺少 {path}"))?;
    units.push(Unit {
        path,
        mes: false,
        xor_width: Some(12),
        records: dictionary::records(bytes, 211, 12)?,
    });
    let catalog: Vec<ExeCatalog> = serde_json::from_str(include_str!("../assets/exe_slots.json"))
        .map_err(|e| e.to_string())?;
    for exe in catalog {
        let stem = exe.disk.strip_suffix(".D88").ok_or("EXE 目录错误")?;
        let path = format!("{stem}/{}", exe.file);
        let data = files.get(&path).ok_or_else(|| format!("缺少 {path}"))?;
        if data.len() != exe.size || data.get(..2) != Some(b"MZ") {
            return Err(format!("{path}: EXE 大小/签名与已分析版本不同"));
        }
        let mut records = Vec::new();
        let mut previous_end = 0;
        for slot in exe.slots {
            let raw = unhex(&slot.hex)?;
            let end = slot.offset.checked_add(raw.len()).ok_or("EXE 槽位溢出")?;
            if slot.offset < previous_end
                || raw.last() != Some(&0)
                || data.get(slot.offset..end) != Some(raw.as_slice())
            {
                return Err(format!(
                    "{path}: 原文槽位 0x{:X} 与反汇编基线不同或重叠",
                    slot.offset
                ));
            }
            let (message, expression_lines) = codec::decode(&raw[..raw.len() - 1], false)?;
            records.push(Record {
                offset: slot.offset,
                raw,
                message,
                expression_lines,
            });
            previous_end = end;
        }
        units.push(Unit {
            path,
            mes: false,
            xor_width: None,
            records,
        });
    }
    units.sort_by(|a, b| a.path.cmp(&b.path));
    for unit in &units {
        for (index, record) in unit.records.iter().enumerate() {
            if unit.xor_width.is_none() {
                codec::format_signature(&record.message, unit.mes)
                    .map_err(|e| format!("{} 第 {index} 条: {e}", unit.path))?;
            }
        }
    }
    let primary = Primary {
        schema: SCHEMA.into(),
        disks: disks
            .iter()
            .map(|d| DiskIdentity {
                name: d.name.clone(),
                size: d.bytes.len(),
            })
            .collect(),
        files: identities,
        units,
    };
    Ok(Game {
        disks,
        files,
        primary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fragmented_patch_preserves_headers_and_rejects_partial_writes() {
        let mut image = vec![0x55; 32];
        let mut file = FileIdentity {
            path: "test".into(),
            size: 4,
            ranges: vec![
                FileSpan {
                    file_offset: 0,
                    source_offset: 4,
                    length: 2,
                },
                FileSpan {
                    file_offset: 2,
                    source_offset: 20,
                    length: 2,
                },
            ],
        };
        patch_file(&mut image, &file, b"abcd").unwrap();
        assert_eq!(&image[4..6], b"ab");
        assert_eq!(&image[20..22], b"cd");
        assert_eq!(image.iter().filter(|b| **b == 0x55).count(), 28);
        let before = image.clone();
        file.ranges[1].source_offset = 31;
        assert!(patch_file(&mut image, &file, b"1234").is_err());
        assert_eq!(image, before);
        file.ranges[1].source_offset = 5;
        assert!(patch_file(&mut image, &file, b"1234").is_err());
        assert_eq!(image, before);
    }

    #[test]
    fn embedded_catalog_has_exact_nonoverlapping_source_slots() {
        let catalog: Vec<ExeCatalog> =
            serde_json::from_str(include_str!("../assets/exe_slots.json")).unwrap();
        assert_eq!(catalog.iter().map(|e| e.slots.len()).sum::<usize>(), 536);
        for exe in catalog {
            let mut previous = 0;
            for slot in exe.slots {
                let raw = unhex(&slot.hex).unwrap();
                assert_eq!(raw.last(), Some(&0));
                assert!(!raw[..raw.len() - 1].contains(&0));
                assert!(slot.offset >= previous && slot.offset + raw.len() <= exe.size);
                let (text, _) = codec::decode(&raw[..raw.len() - 1], false).unwrap();
                codec::format_signature(&text, false).unwrap();
                previous = slot.offset + raw.len();
            }
        }
    }
}
