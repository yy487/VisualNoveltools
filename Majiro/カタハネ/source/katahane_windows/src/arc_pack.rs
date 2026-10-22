use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use super::{Archive, ArchiveVersion};

const HEADER_SIZE: usize = 0x1c;

/// Rebuild a Majiro archive while retaining its version, hashes, names, opaque gaps,
/// and the original physical ordering of its members.
pub(super) fn pack_archive(
    template: &[u8],
    files: &BTreeMap<PathBuf, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    let archive = Archive::parse(template)?;
    let expected_paths: BTreeSet<_> = archive
        .entries
        .iter()
        .map(|entry| entry.relative_path.clone())
        .collect();
    let provided_paths: BTreeSet<_> = files.keys().cloned().collect();
    if expected_paths != provided_paths {
        let missing = expected_paths
            .difference(&provided_paths)
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>();
        let extra = provided_paths
            .difference(&expected_paths)
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>();
        let mut details = Vec::new();
        if !missing.is_empty() {
            details.push(format!("缺少条目: {}", missing.join(", ")));
        }
        if !extra.is_empty() {
            details.push(format!("多余条目: {}", extra.join(", ")));
        }
        return Err(format!(
            "封包文件集合与模板不匹配（{}）",
            details.join("；")
        ));
    }

    let payload_start = if archive.entries.is_empty() {
        archive.file_size
    } else {
        archive.data_offset
    };
    if payload_start > template.len() {
        return Err("模板的数据区起始位置超出文件范围".into());
    }

    let old_member_bytes = archive.entries.iter().try_fold(0usize, |total, entry| {
        total
            .checked_add(entry.end - entry.start)
            .ok_or_else(|| "原档案成员总长度溢出".to_owned())
    })?;
    let new_member_bytes = archive.entries.iter().try_fold(0usize, |total, entry| {
        let file = files
            .get(&entry.relative_path)
            .ok_or_else(|| format!("缺少条目: {}", entry.relative_path.display()))?;
        total
            .checked_add(file.len())
            .ok_or_else(|| "封包总长度溢出".to_owned())
    })?;
    let opaque_bytes = if archive.entries.is_empty() {
        0
    } else {
        template
            .len()
            .checked_sub(archive.data_offset)
            .and_then(|length| length.checked_sub(old_member_bytes))
            .ok_or("成员范围超出数据区")?
    };
    let final_len = payload_start
        .checked_add(opaque_bytes)
        .and_then(|length| length.checked_add(new_member_bytes))
        .ok_or("封包总长度溢出")?;
    if final_len > u32::MAX as usize {
        return Err("封包超过 Majiro ARC 的 32 位偏移范围".into());
    }

    let mut output = Vec::new();
    output
        .try_reserve_exact(final_len)
        .map_err(|error| format!("无法为封包数据分配空间: {error}"))?;
    output.extend_from_slice(&template[..payload_start]);

    let mut physical_entries = archive.entries.iter().enumerate().collect::<Vec<_>>();
    physical_entries.sort_by_key(|(index, entry)| (entry.start, *index));
    let mut source_cursor = archive.data_offset;
    for (index, entry) in physical_entries {
        if entry.start < source_cursor {
            return Err("模板成员范围重叠，不能安全重建".into());
        }
        output.extend_from_slice(
            template
                .get(source_cursor..entry.start)
                .ok_or("模板成员间的保留区超出文件范围")?,
        );
        let new_offset = u32::try_from(output.len()).map_err(|_| "成员偏移超出 u32 范围")?;
        let record_start = HEADER_SIZE + index * archive.version.index_entry_size();
        let offset_field = record_start + archive.version.hash_width();
        write_u32(&mut output, offset_field, new_offset)?;
        let member = files
            .get(&entry.relative_path)
            .ok_or_else(|| format!("缺少条目: {}", entry.relative_path.display()))?;
        if archive.version != ArchiveVersion::V1 {
            let length = u32::try_from(member.len()).map_err(|_| "成员长度超出 u32 范围")?;
            write_u32(&mut output, offset_field + 4, length)?;
        }
        output.extend_from_slice(member);
        source_cursor = entry.end;
    }
    if !archive.entries.is_empty() {
        output.extend_from_slice(
            template
                .get(source_cursor..)
                .ok_or("模板尾部保留区超出文件范围")?,
        );
    }
    if archive.version.has_terminal_index_entry() {
        let sentinel_pos = HEADER_SIZE
            .checked_add(
                archive
                    .entries
                    .len()
                    .checked_mul(archive.version.index_entry_size())
                    .ok_or("索引大小溢出")?,
            )
            .ok_or("哨兵位置溢出")?;
        let final_offset = u32::try_from(output.len()).map_err(|_| "ARC 末尾偏移超出 u32 范围")?;
        write_u32(&mut output, sentinel_pos + 4, final_offset)?;
    }
    if output.len() != final_len {
        return Err("封包输出长度与预计算结果不一致".into());
    }

    Ok(output)
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) -> Result<(), String> {
    let field = bytes
        .get_mut(offset..offset.checked_add(4).ok_or("字段偏移溢出")?)
        .ok_or_else(|| format!("写入 ARC 索引字段越界: 0x{offset:X}"))?;
    field.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{name_hash_for_version, read_u32};

    fn synthetic_archive(version: ArchiveVersion) -> (Vec<u8>, BTreeMap<PathBuf, Vec<u8>>) {
        let mut entries = vec![
            ("alpha.mjo", b"old-alpha".to_vec()),
            ("beta.mjo", b"old-beta".to_vec()),
        ];
        entries.sort_by_key(|(name, _)| name_hash_for_version(version, name.as_bytes()));

        let count = entries.len();
        let record_size = version.index_entry_size();
        let index_count = count + usize::from(version.has_terminal_index_entry());
        let names_offset = HEADER_SIZE + index_count * record_size;
        let names: Vec<u8> = entries
            .iter()
            .flat_map(|(name, _)| name.as_bytes().iter().copied().chain([0]))
            .collect();
        let data_offset = names_offset + names.len();
        let mut data = Vec::new();
        let mut offsets = Vec::new();
        for (_, content) in &entries {
            offsets.push(data_offset + data.len());
            data.extend_from_slice(content);
        }
        let file_size = data_offset + data.len();

        let mut bytes = vec![0; HEADER_SIZE];
        bytes[..16].copy_from_slice(version.signature());
        bytes[0x10..0x14].copy_from_slice(&(count as u32).to_le_bytes());
        bytes[0x14..0x18].copy_from_slice(&(names_offset as u32).to_le_bytes());
        bytes[0x18..0x1c].copy_from_slice(&(data_offset as u32).to_le_bytes());
        for (index, (name, _)) in entries.iter().enumerate() {
            let record_start = HEADER_SIZE + index * record_size;
            let hash = name_hash_for_version(version, name.as_bytes());
            bytes.extend_from_slice(&hash.to_le_bytes()[..version.hash_width()]);
            bytes.extend_from_slice(&(offsets[index] as u32).to_le_bytes());
            if version != ArchiveVersion::V1 {
                bytes.extend_from_slice(&(entries[index].1.len() as u32).to_le_bytes());
            }
            assert_eq!(bytes.len(), record_start + record_size);
        }
        if version.has_terminal_index_entry() {
            // The V1 sentinel hash is unused by the runtime; its offset is EOF.
            bytes.extend_from_slice(&0xfeed_beefu32.to_le_bytes());
            bytes.extend_from_slice(&(file_size as u32).to_le_bytes());
        }
        bytes.extend_from_slice(&names);
        bytes.extend_from_slice(&data);

        let replacements = entries
            .into_iter()
            .map(|(name, _)| {
                let replacement = match name {
                    "alpha.mjo" => b"new alpha payload".to_vec(),
                    _ => b"B".to_vec(),
                };
                (PathBuf::from(name), replacement)
            })
            .collect();
        (bytes, replacements)
    }

    fn repack_roundtrip(version: ArchiveVersion) {
        let (template, replacements) = synthetic_archive(version);
        let original = Archive::parse(&template).unwrap();
        let payload_start = original.data_offset;
        let sentinel_pos = HEADER_SIZE + original.entries.len() * version.index_entry_size();

        let packed = pack_archive(&template, &replacements).unwrap();
        let parsed = Archive::parse(&packed).unwrap();
        assert_eq!(parsed.entries.len(), original.entries.len());
        assert_eq!(
            &packed[original.names_offset..payload_start],
            &template[original.names_offset..payload_start]
        );
        if version.has_terminal_index_entry() {
            assert_eq!(read_u32(&packed, sentinel_pos).unwrap(), 0xfeed_beef);
            assert_eq!(
                read_u32(&packed, sentinel_pos + 4).unwrap() as usize,
                packed.len()
            );
        }
        for (index, (entry, original_entry)) in
            parsed.entries.iter().zip(&original.entries).enumerate()
        {
            let table_pos = HEADER_SIZE + index * version.index_entry_size();
            assert_eq!(
                &packed[table_pos..table_pos + version.hash_width()],
                &template[table_pos..table_pos + version.hash_width()]
            );
            let offset_field = table_pos + version.hash_width();
            assert_eq!(
                read_u32(&packed, offset_field).unwrap() as usize,
                entry.start
            );
            assert_eq!(entry.archive_name, original_entry.archive_name);
            assert_eq!(entry.name_hash, original_entry.name_hash);
            assert_eq!(
                &packed[entry.start..entry.end],
                replacements[&entry.relative_path].as_slice()
            );
            if version != ArchiveVersion::V1 {
                assert_eq!(
                    read_u32(&packed, offset_field + 4).unwrap() as usize,
                    entry.end - entry.start
                );
            }
        }
        assert_eq!(parsed.file_size, packed.len());
    }

    #[test]
    fn repacks_v1_using_next_offset_lengths() {
        repack_roundtrip(ArchiveVersion::V1);
    }

    #[test]
    fn repacks_v2_and_v3_using_explicit_lengths_and_versioned_hashes() {
        repack_roundtrip(ArchiveVersion::V2);
        repack_roundtrip(ArchiveVersion::V3);
    }

    #[test]
    fn infers_v3_layout_without_a_known_signature() {
        let (mut template, _) = synthetic_archive(ArchiveVersion::V3);
        template[..16].fill(0xa5);
        let parsed = Archive::parse(&template).unwrap();
        assert_eq!(parsed.version, ArchiveVersion::V3);
        assert_eq!(parsed.entries.len(), 2);
    }

    #[test]
    fn rejects_missing_extra_and_malformed_inputs() {
        let (template, replacements) = synthetic_archive(ArchiveVersion::V1);

        let mut missing = replacements.clone();
        missing.remove(&PathBuf::from("alpha.mjo"));
        assert!(pack_archive(&template, &missing)
            .unwrap_err()
            .contains("缺少条目"));

        let mut extra = replacements;
        extra.insert(PathBuf::from("extra.mjo"), b"extra".to_vec());
        assert!(pack_archive(&template, &extra)
            .unwrap_err()
            .contains("多余条目"));

        let mut malformed = template;
        let count = Archive::parse(&malformed).unwrap().entries.len();
        let sentinel_offset = HEADER_SIZE + count * ArchiveVersion::V1.index_entry_size() + 4;
        malformed[sentinel_offset] ^= 1;
        assert!(pack_archive(&malformed, &BTreeMap::new()).is_err());
    }
}
