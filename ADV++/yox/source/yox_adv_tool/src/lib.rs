use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use vn_cli::{Error, Result};

pub mod carrier;
pub mod font;
pub mod renderer;
pub mod script;
pub mod workflow;

const MAGIC: &[u8] = b"YOX\0";

fn u32_at(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
}

#[derive(Debug, Serialize)]
pub struct Entry {
    pub id: usize,
    pub file: String,
    pub offset: usize,
    pub size: usize,
    pub flag: u32,
    pub reserved: u32,
    pub kind: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ExtraRange {
    pub source_offset: usize,
    pub stored_offset: usize,
    pub size: usize,
}

#[derive(Serialize)]
pub struct Archive {
    pub source_file: String,
    pub source_size: usize,
    pub version: u32,
    pub table_offset: usize,
    pub entries: Vec<Entry>,
    pub extra_file: &'static str,
    pub extra_ranges: Vec<ExtraRange>,
    #[serde(skip)]
    bytes: Vec<u8>,
}

impl Archive {
    pub fn raw_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn parse(source_file: String, bytes: Vec<u8>) -> Result<Self> {
        if bytes.len() < 32 || &bytes[..4] != MAGIC || u32_at(&bytes, 4) != 0 {
            return Err("需要版本 0 的外层 YOX 档案（不能传入内层字库或脚本）".into());
        }
        let table_offset = u32_at(&bytes, 8) as usize;
        let count = u32_at(&bytes, 12) as usize;
        if table_offset < 32
            || table_offset > bytes.len()
            || count == 0
            || count > (bytes.len() - table_offset) / 16
        {
            return Err("YOX 文件表越界或条目数不受支持".into());
        }
        let mut entries = Vec::with_capacity(count);
        let mut extents = Vec::with_capacity(count);
        for id in 0..count {
            let record = table_offset + id * 16;
            let offset = u32_at(&bytes, record) as usize;
            let size = u32_at(&bytes, record + 4) as usize;
            if offset > table_offset || size > table_offset - offset || (size > 0 && offset < 32) {
                return Err(format!("条目 {id}: 数据越界或与档案头/文件表重叠").into());
            }
            let payload = &bytes[offset..offset + size];
            let kind = if payload.len() >= 8 && &payload[..4] == MAGIC {
                match u32_at(payload, 4) {
                    0x0100_0000 => "yox_font",
                    0x1000_0000 => "yox_script",
                    _ => "unknown_yox",
                }
            } else {
                "opaque"
            };
            entries.push(Entry {
                id,
                file: format!("files/{id:05}.yox"),
                offset,
                size,
                flag: u32_at(&bytes, record + 8),
                reserved: u32_at(&bytes, record + 12),
                kind,
            });
            if size > 0 {
                extents.push((offset, offset + size));
            }
        }
        extents.sort_unstable();
        let mut extra_ranges = Vec::new();
        let mut cursor = 0;
        let mut stored_offset = 0;
        for (start, end) in extents.into_iter().chain([(bytes.len(), bytes.len())]) {
            if start > cursor {
                let size = start - cursor;
                extra_ranges.push(ExtraRange {
                    source_offset: cursor,
                    stored_offset,
                    size,
                });
                stored_offset += size;
            }
            cursor = cursor.max(end);
        }
        Ok(Self {
            source_file,
            source_size: bytes.len(),
            version: 0,
            table_offset,
            entries,
            extra_file: "archive_extra.bin",
            extra_ranges,
            bytes,
        })
    }

    pub fn entry_bytes(&self, id: usize) -> Result<&[u8]> {
        let entry = self
            .entries
            .get(id)
            .ok_or_else(|| Error("档案条目编号越界".into()))?;
        Ok(&self.bytes[entry.offset..entry.offset + entry.size])
    }

    /// Rebuilds an outer DAT with validated entry replacements and 0x800 alignment.
    pub fn rebuild(&self, replacements: &[(usize, Vec<u8>)]) -> Result<Vec<u8>> {
        let mut map = std::collections::BTreeMap::new();
        for (id, payload) in replacements {
            if *id >= self.entries.len() || map.insert(*id, payload).is_some() {
                return Err("重复或越界的档案条目替换".into());
            }
        }
        let mut physical: Vec<_> = self
            .entries
            .iter()
            .filter(|entry| entry.size != 0)
            .collect();
        physical.sort_by_key(|entry| entry.offset);
        if physical.is_empty() {
            return Err("不支持没有实体数据条目的YOX档案".into());
        }
        let first = physical[0].offset;
        let mut previous_end = first;
        for entry in &physical {
            if entry.offset < previous_end {
                return Err("档案条目实体范围重叠".into());
            }
            if self.bytes[previous_end..entry.offset]
                .iter()
                .any(|byte| *byte != 0)
            {
                return Err("不支持条目之间含非零数据的档案间隙".into());
            }
            previous_end = entry.offset + entry.size;
        }
        let table_end = self.table_offset + self.entries.len() * 16;
        if previous_end > self.table_offset
            || self.bytes[previous_end..self.table_offset]
                .iter()
                .any(|byte| *byte != 0)
        {
            return Err("不支持文件表前含非零数据的档案间隙".into());
        }
        let mut out = self.bytes[..first].to_vec();
        let mut table = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            let aligned = out.len().checked_add(0x7ff).ok_or("档案长度溢出")? & !0x7ff;
            out.resize(aligned, 0);
            let offset = u32::try_from(out.len()).map_err(|_| "档案偏移超过4GiB")?;
            let payload = map
                .get(&entry.id)
                .map(|v| v.as_slice())
                .unwrap_or_else(|| &self.bytes[entry.offset..entry.offset + entry.size]);
            let size = u32::try_from(payload.len()).map_err(|_| "档案条目超过4GiB")?;
            out.extend_from_slice(payload);
            table.push((offset, size, entry.flag, entry.reserved));
        }
        let aligned = out.len().checked_add(0x7ff).ok_or("档案长度溢出")? & !0x7ff;
        out.resize(aligned, 0);
        let table_offset = u32::try_from(out.len()).map_err(|_| "档案文件表超过4GiB")?;
        for (offset, size, flag, reserved) in table {
            out.extend_from_slice(&offset.to_le_bytes());
            out.extend_from_slice(&size.to_le_bytes());
            out.extend_from_slice(&flag.to_le_bytes());
            out.extend_from_slice(&reserved.to_le_bytes());
        }
        out[8..12].copy_from_slice(&table_offset.to_le_bytes());
        out[12..16].copy_from_slice(&(self.entries.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.bytes[table_end..]);
        Ok(out)
    }

    /// Writes only to a new directory; all payloads use the validated input snapshot.
    pub fn unpack(&self, source: &Path, output: &Path) -> Result<PathBuf> {
        let source = fs::canonicalize(source)?;
        let parent = fs::canonicalize(output.parent().ok_or("输出缺少父目录")?)?;
        let name = output.file_name().ok_or("输出必须是一个新目录")?;
        let output = parent.join(name);
        if output == source || source.starts_with(&output) {
            return Err("输出不能覆盖或包含输入文件".into());
        }
        if output.try_exists()? {
            return Err("输出已存在；请使用新目录，本工具不覆盖已有输出".into());
        }
        // Each stage is exclusively created by this invocation in the output parent.
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let stage = loop {
            let path = parent.join(format!(
                ".yox-unpack-{}-{}.tmp",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => break Stage { path, parent },
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.into()),
            }
        };
        fs::create_dir(stage.path.join("files"))?;
        for entry in &self.entries {
            fs::write(
                stage.path.join(&entry.file),
                &self.bytes[entry.offset..entry.offset + entry.size],
            )?;
        }
        let mut extra = Vec::new();
        for range in &self.extra_ranges {
            extra.extend_from_slice(
                &self.bytes[range.source_offset..range.source_offset + range.size],
            );
        }
        fs::write(stage.path.join(self.extra_file), extra)?;
        let manifest = serde_json::to_vec_pretty(self).map_err(|e| Error(e.to_string()))?;
        fs::write(stage.path.join("archive.json"), manifest)?;
        if output.try_exists()? {
            return Err("输出在执行期间出现；拒绝覆盖".into());
        }
        fs::rename(&stage.path, &output)?;
        Ok(output)
    }
}

struct Stage {
    path: PathBuf,
    parent: PathBuf,
}

impl Drop for Stage {
    fn drop(&mut self) {
        if let Ok(resolved) = fs::canonicalize(&self.path) {
            if resolved.parent() == Some(self.parent.as_path()) {
                let _ = fs::remove_dir_all(resolved);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<u8> {
        let mut bytes = vec![0; 81];
        bytes[..4].copy_from_slice(MAGIC);
        bytes[8..12].copy_from_slice(&64u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&1u32.to_le_bytes());
        bytes[32..40].copy_from_slice(b"YOX\0\0\0\0\x10");
        bytes[64..68].copy_from_slice(&32u32.to_le_bytes());
        bytes[68..72].copy_from_slice(&8u32.to_le_bytes());
        bytes[72..76].copy_from_slice(&u32::MAX.to_le_bytes());
        bytes[80] = 123;
        bytes
    }

    #[test]
    fn parses_ranges_and_rejects_truncation() {
        let bytes = sample();
        let archive = Archive::parse("script.dat".into(), bytes.clone()).unwrap();
        assert_eq!(archive.entries[0].kind, "yox_script");
        assert_eq!(archive.entries[0].flag, u32::MAX);
        assert_eq!(archive.extra_ranges.len(), 2);
        assert_eq!(archive.extra_ranges[1].source_offset, 40);
        assert_eq!(archive.extra_ranges[1].size, 41);
        assert!(Archive::parse("bad".into(), bytes[..79].to_vec()).is_err());
        let mut invalid = bytes;
        invalid[68..72].copy_from_slice(&33u32.to_le_bytes());
        assert!(Archive::parse("bad".into(), invalid).is_err());
    }

    #[test]
    fn rebuild_keeps_only_original_post_table_tail() {
        let source = sample();
        let archive = Archive::parse("source.dat".into(), source).unwrap();
        let rebuilt = archive.rebuild(&[]).unwrap();
        let table = u32_at(&rebuilt, 8) as usize;
        assert_eq!(table, 0x1000);
        assert_eq!(&rebuilt[table + 16..], &[123]);
    }

    #[test]
    fn unpack_preserves_metadata_and_protects_existing_paths() {
        let root = std::env::temp_dir().join(format!("yox-adv-test-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let source = root.join("source.dat");
        fs::write(&source, sample()).unwrap();
        let archive = Archive::parse("source.dat".into(), sample()).unwrap();
        assert!(archive.unpack(&source, &source).is_err());
        assert!(archive.unpack(&source, &root).is_err());
        let output = archive.unpack(&source, &root.join("unpacked")).unwrap();
        assert_eq!(
            fs::read(output.join("files/00000.yox")).unwrap(),
            &sample()[32..40]
        );
        assert_eq!(
            fs::read(output.join("archive_extra.bin")).unwrap().len(),
            73
        );
        assert!(archive.unpack(&source, &output).is_err());
        assert_eq!(fs::read(&source).unwrap(), sample());
        assert_eq!(
            root.parent(),
            Some(fs::canonicalize(std::env::temp_dir()).unwrap().as_path())
        );
        fs::remove_dir_all(root).unwrap();
    }
}
