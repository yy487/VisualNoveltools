use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use crate::error::{PmxError, Result};
use crate::format::{
    decode_envelope, decode_name_slot, encode_envelope, parse_payload, validate_relative_path,
    ParsedPayload, ARCHIVE_HEADER_SIZE, ARCHIVE_NAME_SIZE, ARCHIVE_RECORD_SIZE, XOR_KEY,
};
use crate::manifest::{
    decode_name_slot_hex, read_manifest, Manifest, MANIFEST_FORMAT, MANIFEST_VERSION,
};
use crate::unpack::PayloadKind;

#[derive(Debug)]
pub struct PackReport {
    pub decoded_bytes: usize,
    pub output_bytes: usize,
    pub packed_files: usize,
    pub kind: PayloadKind,
    pub reused_original: bool,
    pub output_file: PathBuf,
}

pub fn default_packed_path(input_directory: &Path) -> Result<PathBuf> {
    let directory_name = input_directory
        .file_name()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| {
            PmxError::new(format!(
                "输入目录缺少有效名称：{}",
                input_directory.display()
            ))
        })?;
    let directory_name = directory_name.to_string_lossy();
    let base_name = directory_name
        .strip_suffix("_unpacked")
        .unwrap_or(&directory_name);
    if base_name.is_empty() {
        return Err(PmxError::new("无法从输入目录生成 PMX 文件名"));
    }

    Ok(input_directory
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{base_name}_packed.Pmx")))
}

pub fn pack_directory(input_directory: &Path, output_file: &Path) -> Result<PackReport> {
    if !input_directory.is_dir() {
        return Err(PmxError::new(format!(
            "输入不是目录：{}",
            input_directory.display()
        )));
    }
    if output_file.exists() {
        return Err(PmxError::new(format!(
            "输出文件已存在，为避免覆盖而停止：{}",
            output_file.display()
        )));
    }

    let manifest = read_manifest(input_directory)?;
    validate_manifest(&manifest)?;

    let (decoded, kind, packed_files) = match manifest.payload_kind.as_str() {
        "data_archive" => (
            rebuild_archive(input_directory, &manifest)?,
            PayloadKind::DataArchive,
            manifest.entries.len(),
        ),
        "raw" => (
            rebuild_raw(input_directory, &manifest)?,
            PayloadKind::Raw,
            1,
        ),
        other => {
            return Err(PmxError::new(format!(
                "不支持的 manifest payload_kind：{other}"
            )));
        }
    };

    let original = reusable_original(input_directory, &manifest, &decoded);
    let reused_original = original.is_some();
    let encoded = match original {
        Some(encoded) => encoded,
        None => {
            let encoded = encode_envelope(&decoded)?;
            let verified = decode_envelope(&encoded)?;
            if verified != decoded {
                return Err(PmxError::new("内部错误：重新封装后的解压自检不一致"));
            }
            encoded
        }
    };

    write_atomic(output_file, &encoded)?;
    Ok(PackReport {
        decoded_bytes: decoded.len(),
        output_bytes: encoded.len(),
        packed_files,
        kind,
        reused_original,
        output_file: output_file.to_path_buf(),
    })
}

fn validate_manifest(manifest: &Manifest) -> Result<()> {
    if manifest.format != MANIFEST_FORMAT {
        return Err(PmxError::new(format!(
            "manifest format 不匹配：期望 {MANIFEST_FORMAT}，实际 {}",
            manifest.format
        )));
    }
    if manifest.manifest_version != MANIFEST_VERSION {
        return Err(PmxError::new(format!(
            "不支持的 manifest_version：{}",
            manifest.manifest_version
        )));
    }
    if manifest.xor_key != XOR_KEY {
        return Err(PmxError::new(format!(
            "manifest xor_key 不匹配：期望 {XOR_KEY}，实际 {}",
            manifest.xor_key
        )));
    }
    if manifest.compression != "zlib" {
        return Err(PmxError::new(format!(
            "不支持的压缩方式：{}",
            manifest.compression
        )));
    }
    validate_single_file_name(&manifest.source_file, "source_file")?;
    Ok(())
}

fn rebuild_archive(input_directory: &Path, manifest: &Manifest) -> Result<Vec<u8>> {
    if manifest.entries.is_empty() {
        return Err(PmxError::new("data_archive manifest 没有文件记录"));
    }
    if manifest.payload_file.is_some() {
        return Err(PmxError::new("data_archive manifest 不应包含 payload_file"));
    }

    let count = u32::try_from(manifest.entries.len())
        .map_err(|_| PmxError::new("Data PMX 文件数量超过 u32 范围"))?;
    let table_size = manifest
        .entries
        .len()
        .checked_mul(ARCHIVE_RECORD_SIZE)
        .and_then(|size| size.checked_add(ARCHIVE_HEADER_SIZE))
        .ok_or_else(|| PmxError::new("Data PMX 文件表大小溢出"))?;
    if manifest.table_size != Some(table_size) {
        return Err(PmxError::new(format!(
            "manifest table_size 不匹配：期望 {table_size}，实际 {:?}",
            manifest.table_size
        )));
    }

    let mut files = Vec::with_capacity(manifest.entries.len());
    let mut seen_paths = HashSet::with_capacity(manifest.entries.len());
    let mut original_payload_size = 0usize;
    let mut rebuilt_payload_size = 0usize;

    for (expected_index, entry) in manifest.entries.iter().enumerate() {
        if entry.index != expected_index {
            return Err(PmxError::new(format!(
                "manifest 记录顺序损坏：位置 {expected_index} 的 index 为 {}",
                entry.index
            )));
        }
        let expected_record_offset = ARCHIVE_HEADER_SIZE + expected_index * ARCHIVE_RECORD_SIZE;
        if entry.record_offset != expected_record_offset {
            return Err(PmxError::new(format!(
                "manifest 记录 {expected_index} 的 record_offset 不匹配：期望 {expected_record_offset}，实际 {}",
                entry.record_offset
            )));
        }
        let expected_data_offset = table_size
            .checked_add(original_payload_size)
            .ok_or_else(|| PmxError::new("manifest 原始载荷偏移溢出"))?;
        if entry.data_offset != expected_data_offset {
            return Err(PmxError::new(format!(
                "manifest 记录 {expected_index} 的 data_offset 不匹配：期望 {expected_data_offset}，实际 {}",
                entry.data_offset
            )));
        }
        original_payload_size = original_payload_size
            .checked_add(entry.size)
            .ok_or_else(|| PmxError::new("manifest 原始文件大小总和溢出"))?;

        let relative_path = validate_relative_path(&entry.path).map_err(|error| {
            PmxError::new(format!(
                "manifest 记录 {expected_index} 的路径无效：{error}"
            ))
        })?;
        let collision_key = entry.path.replace('\\', "/").to_lowercase();
        if !seen_paths.insert(collision_key) {
            return Err(PmxError::new(format!(
                "manifest 存在大小写不敏感的重复路径：{}",
                entry.path
            )));
        }

        let name_slot = decode_name_slot_hex(&entry.name_slot_hex).map_err(|error| {
            PmxError::new(format!(
                "manifest 记录 {expected_index} 的名称槽无效：{error}"
            ))
        })?;
        let slot_name = decode_name_slot(&name_slot).map_err(|error| {
            PmxError::new(format!(
                "manifest 记录 {expected_index} 的名称槽无法解码：{error}"
            ))
        })?;
        if slot_name.replace('\\', "/") != entry.path.replace('\\', "/") {
            return Err(PmxError::new(format!(
                "manifest 记录 {expected_index} 的 path 与原始名称槽不一致：path={} slot={slot_name}",
                entry.path
            )));
        }

        let path = input_directory.join(relative_path);
        if !path.is_file() {
            return Err(PmxError::new(format!(
                "manifest 记录 {expected_index} 对应文件不存在：{}",
                path.display()
            )));
        }
        let data = fs::read(&path)
            .map_err(|error| PmxError::context(format!("读取失败：{}", path.display()), error))?;
        u32::try_from(data.len()).map_err(|_| {
            PmxError::new(format!(
                "文件超过 u32 大小范围：{}（{} 字节）",
                path.display(),
                data.len()
            ))
        })?;
        rebuilt_payload_size = rebuilt_payload_size
            .checked_add(data.len())
            .ok_or_else(|| PmxError::new("重建文件大小总和溢出"))?;
        files.push((name_slot, data));
    }

    let expected_original_size = table_size
        .checked_add(original_payload_size)
        .ok_or_else(|| PmxError::new("manifest 原始解压大小溢出"))?;
    if manifest.decoded_size != expected_original_size {
        return Err(PmxError::new(format!(
            "manifest decoded_size 不匹配：期望 {expected_original_size}，实际 {}",
            manifest.decoded_size
        )));
    }

    let rebuilt_size = table_size
        .checked_add(rebuilt_payload_size)
        .ok_or_else(|| PmxError::new("重建 PMX 大小溢出"))?;
    let mut decoded = vec![0u8; table_size];
    decoded.reserve(rebuilt_size - table_size);
    decoded[0..4].copy_from_slice(&count.to_le_bytes());

    for (index, (name_slot, data)) in files.iter().enumerate() {
        let record_offset = ARCHIVE_HEADER_SIZE + index * ARCHIVE_RECORD_SIZE;
        decoded[record_offset..record_offset + ARCHIVE_NAME_SIZE].copy_from_slice(name_slot);
        let size = u32::try_from(data.len()).expect("validated file size");
        decoded[record_offset + ARCHIVE_NAME_SIZE..record_offset + ARCHIVE_RECORD_SIZE]
            .copy_from_slice(&size.to_le_bytes());
    }
    for (_, data) in files {
        decoded.extend_from_slice(&data);
    }

    match parse_payload(&decoded)? {
        ParsedPayload::Archive(archive) if archive.entries.len() == manifest.entries.len() => {}
        ParsedPayload::Archive(archive) => {
            return Err(PmxError::new(format!(
                "重建文件表自检数量不一致：期望 {}，实际 {}",
                manifest.entries.len(),
                archive.entries.len()
            )));
        }
        ParsedPayload::Raw => {
            return Err(PmxError::new("重建 Data PMX 被解析为 raw 载荷"));
        }
    }
    Ok(decoded)
}

fn rebuild_raw(input_directory: &Path, manifest: &Manifest) -> Result<Vec<u8>> {
    if !manifest.entries.is_empty() || manifest.table_size.is_some() {
        return Err(PmxError::new("raw manifest 不应包含文件记录或 table_size"));
    }
    let payload_file = manifest
        .payload_file
        .as_deref()
        .ok_or_else(|| PmxError::new("raw manifest 缺少 payload_file"))?;
    let relative_path = validate_relative_path(payload_file)
        .map_err(|error| PmxError::new(format!("raw payload_file 路径无效：{error}")))?;
    let path = input_directory.join(relative_path);
    if !path.is_file() {
        return Err(PmxError::new(format!(
            "raw payload 文件不存在：{}",
            path.display()
        )));
    }
    fs::read(&path)
        .map_err(|error| PmxError::context(format!("读取失败：{}", path.display()), error))
}

fn reusable_original(
    input_directory: &Path,
    manifest: &Manifest,
    decoded: &[u8],
) -> Option<Vec<u8>> {
    let candidate = input_directory.parent()?.join(&manifest.source_file);
    let metadata = fs::metadata(&candidate).ok()?;
    if metadata.len() != manifest.source_size as u64 {
        return None;
    }
    let encoded = fs::read(candidate).ok()?;
    let original_decoded = decode_envelope(&encoded).ok()?;
    (original_decoded == decoded).then_some(encoded)
}

fn validate_single_file_name(value: &str, field: &str) -> Result<()> {
    let mut components = Path::new(value).components();
    if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
        return Err(PmxError::new(format!(
            "manifest {field} 必须是单个文件名：{value}"
        )));
    }
    Ok(())
}

fn write_atomic(output_file: &Path, data: &[u8]) -> Result<()> {
    if output_file.exists() {
        return Err(PmxError::new(format!(
            "输出文件已存在，为避免覆盖而停止：{}",
            output_file.display()
        )));
    }
    let parent = output_file.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| {
        PmxError::context(format!("创建输出目录失败：{}", parent.display()), error)
    })?;

    let file_name = output_file
        .file_name()
        .ok_or_else(|| PmxError::new("输出路径缺少文件名"))?
        .to_string_lossy();
    let temporary = parent.join(format!(".{file_name}.{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| {
            PmxError::context(format!("创建临时文件失败：{}", temporary.display()), error)
        })?;

    let write_result = file.write_all(data).and_then(|_| file.sync_all());
    drop(file);
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temporary);
        return Err(PmxError::context(
            format!("写入临时文件失败：{}", temporary.display()),
            error,
        ));
    }
    if output_file.exists() {
        let _ = fs::remove_file(&temporary);
        return Err(PmxError::new(format!(
            "写入期间输出文件被创建，为避免覆盖而停止：{}",
            output_file.display()
        )));
    }
    fs::rename(&temporary, output_file).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        PmxError::context(
            format!(
                "提交输出文件失败：{} -> {}",
                temporary.display(),
                output_file.display()
            ),
            error,
        )
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;
    use crate::format::{decode_envelope, encode_envelope};
    use crate::unpack::unpack_file;

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn unchanged_archive_reuses_original_bytes() {
        let temp = TestDirectory::new();
        let source = temp.path().join("Data1.Pmx");
        let decoded = make_archive(&[("4.PMX", b"abc"), ("6.PMX", b"defgh")]);
        let encoded = encode_envelope(&decoded).unwrap();
        fs::write(&source, &encoded).unwrap();

        let unpacked = temp.path().join("Data1_unpacked");
        unpack_file(&source, &unpacked, false).unwrap();
        let output = temp.path().join("Data1_packed.Pmx");
        let report = pack_directory(&unpacked, &output).unwrap();

        assert!(report.reused_original);
        assert_eq!(fs::read(output).unwrap(), encoded);
    }

    #[test]
    fn modified_archive_updates_outer_size_and_recompresses() {
        let temp = TestDirectory::new();
        let source = temp.path().join("Data1.Pmx");
        let decoded = make_archive(&[("4.PMX", b"abc"), ("6.PMX", b"defgh")]);
        fs::write(&source, encode_envelope(&decoded).unwrap()).unwrap();

        let unpacked = temp.path().join("Data1_unpacked");
        unpack_file(&source, &unpacked, false).unwrap();
        fs::write(unpacked.join("4.PMX"), b"longer text").unwrap();
        let output = temp.path().join("Data1_packed.Pmx");
        let report = pack_directory(&unpacked, &output).unwrap();

        assert!(!report.reused_original);
        let rebuilt = decode_envelope(&fs::read(output).unwrap()).unwrap();
        let ParsedPayload::Archive(archive) = parse_payload(&rebuilt).unwrap() else {
            panic!("expected archive");
        };
        assert_eq!(archive.entries[0].size, 11);
        assert_eq!(
            &rebuilt[archive.entries[0].data_offset..archive.entries[0].data_offset + 11],
            b"longer text"
        );
        assert_eq!(archive.entries[1].size, 5);
    }

    #[test]
    fn raw_payload_supports_exact_and_modified_roundtrips() {
        let temp = TestDirectory::new();
        let source = temp.path().join("Global.Pmx");
        let original_payload = b"global payload";
        let encoded = encode_envelope(original_payload).unwrap();
        fs::write(&source, &encoded).unwrap();

        let unpacked = temp.path().join("Global_unpacked");
        unpack_file(&source, &unpacked, false).unwrap();
        let exact_output = temp.path().join("Global_exact.Pmx");
        let exact_report = pack_directory(&unpacked, &exact_output).unwrap();
        assert!(exact_report.reused_original);
        assert_eq!(fs::read(exact_output).unwrap(), encoded);

        let modified_payload = b"modified global payload";
        fs::write(unpacked.join("_payload.bin"), modified_payload).unwrap();
        let modified_output = temp.path().join("Global_modified.Pmx");
        let modified_report = pack_directory(&unpacked, &modified_output).unwrap();
        assert!(!modified_report.reused_original);
        assert_eq!(
            decode_envelope(&fs::read(modified_output).unwrap()).unwrap(),
            modified_payload
        );
    }

    fn make_archive(files: &[(&str, &[u8])]) -> Vec<u8> {
        let table_size = ARCHIVE_HEADER_SIZE + ARCHIVE_RECORD_SIZE * files.len();
        let payload_size: usize = files.iter().map(|(_, data)| data.len()).sum();
        let mut decoded = vec![0u8; table_size + payload_size];
        decoded[0..4].copy_from_slice(&(files.len() as u32).to_le_bytes());

        let mut data_offset = table_size;
        for (index, (name, data)) in files.iter().enumerate() {
            let record_offset = ARCHIVE_HEADER_SIZE + index * ARCHIVE_RECORD_SIZE;
            decoded[record_offset..record_offset + name.len()].copy_from_slice(name.as_bytes());
            decoded[record_offset + ARCHIVE_NAME_SIZE..record_offset + ARCHIVE_RECORD_SIZE]
                .copy_from_slice(&(data.len() as u32).to_le_bytes());
            decoded[data_offset..data_offset + data.len()].copy_from_slice(data);
            data_offset += data.len();
        }
        decoded
    }

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("pmx_pack_test_{}_{}", std::process::id(), id));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
