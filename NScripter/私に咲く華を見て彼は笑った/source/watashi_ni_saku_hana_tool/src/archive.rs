use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};

use encoding_rs::SHIFT_JIS;
use sha2::{Digest, Sha256};
use vn_cli::{
    Field, FieldKind, Operation, OperationSpec, Parameters, PreparedOperation, Preview, Progress,
    Result, RunReport, Value,
};

use crate::transaction;

const DEFAULT_ARCHIVE_OUTPUT: &str = r"E:\GAL\tr\work\watashi_ni_saku_hana\arc_unpacked";
const DEFAULT_ARCHIVE_PACK_OUTPUT: &str = r"E:\GAL\tr\work\watashi_ni_saku_hana\arc_repacked.nsa";
const FLAG_RAW: u8 = 0;
const FLAG_SPB: u8 = 1;
const FLAG_LZSS: u8 = 2;
const MAX_NAME_BYTES: usize = 255;

#[derive(Debug, Clone)]
struct ArchiveEntry {
    display_name: String,
    name_bytes: Vec<u8>,
    relative_path: PathBuf,
    flag: u8,
    offset: u64,
    packed_size: usize,
    plain_size: usize,
}

#[derive(Debug)]
struct ArchiveIndex {
    data_start: u64,
    file_size: u64,
    source_sha256: String,
    entries: Vec<ArchiveEntry>,
    unpacked_bytes: u64,
    flag_counts: [u64; 3],
}

pub struct UnpackArchive;
pub struct PackArchive;

impl Operation for UnpackArchive {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "unpack-archive",
            "解封包 arc.nsa",
            vec![
                Field::new("source", "原始 arc.nsa", FieldKind::Path).required(),
                Field::new("output", "资源目录输出", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "按本作 EXE 的 NSA 目录格式提取资源，并处理 SPB BMP 与 LZSS 压缩。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(source) = paths.first() {
            if parameters.get("source").is_none() {
                parameters.set("source", Value::Path(source.clone()));
            }
            if parameters.get("output").is_none() {
                parameters.set("output", Value::Path(PathBuf::from(DEFAULT_ARCHIVE_OUTPUT)));
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let source = fs::canonicalize(parameters.path("source")?)?;
        validate_archive_name(&source)?;
        let index = parse_archive_index(&source)?;
        let overwrite = parameters.flag("overwrite");
        let (output, original_snapshot) =
            transaction::prepare_directory_output(parameters.path("output")?, overwrite)?;
        transaction::ensure_inputs_disjoint_from_output(&[&source], &output)?;

        let preview = Preview {
            inputs: vec![source.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "按反汇编确认的 NSA 目录读取文件名、压缩类型、数据偏移和长度".into(),
                "原样复制普通资源，按位流还原 SPB BMP 和 LZSS 文件".into(),
                "逐项写入暂存目录并复核源文件快照".into(),
                "安全提交完整资源目录".into(),
            ],
            details: vec![
                format!("文件数：{}", index.entries.len()),
                format!(
                    "数据区偏移：{}；归档大小：{} 字节",
                    index.data_start, index.file_size
                ),
                format!(
                    "压缩标记：0={} 个、1(SPB)={} 个、2(LZSS)={} 个",
                    index.flag_counts[0], index.flag_counts[1], index.flag_counts[2]
                ),
                format!("解封后数据总量：{} 字节", index.unpacked_bytes),
                format!("SHA-256：{}", index.source_sha256),
            ],
        };
        Ok(Box::new(PreparedArchive {
            source,
            source_sha256: index.source_sha256,
            file_size: index.file_size,
            data_start: index.data_start,
            entries: index.entries,
            output,
            original_snapshot,
            overwrite,
            preview,
        }))
    }
}

impl Operation for PackArchive {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "pack-archive",
            "重封包 arc.nsa",
            vec![
                Field::new("source", "原始 arc.nsa", FieldKind::Path).required(),
                Field::new("input", "完整资源目录", FieldKind::Path).required(),
                Field::new("output", "重封包输出文件", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "按原归档条目顺序重建 NSA；未修改条目保留原压缩数据，修改项使用无压缩存储。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(source) = paths.first() {
            if parameters.get("source").is_none() {
                parameters.set("source", Value::Path(source.clone()));
            }
            if parameters.get("input").is_none() {
                parameters.set("input", Value::Path(PathBuf::from(DEFAULT_ARCHIVE_OUTPUT)));
            }
            if parameters.get("output").is_none() {
                parameters.set(
                    "output",
                    Value::Path(PathBuf::from(DEFAULT_ARCHIVE_PACK_OUTPUT)),
                );
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let source = fs::canonicalize(parameters.path("source")?)?;
        validate_archive_name(&source)?;
        let input = fs::canonicalize(parameters.path("input")?)?;
        if !input.is_dir() {
            return Err("arc.nsa 资源输入路径必须是目录".into());
        }

        let index = parse_archive_index(&source)?;
        let input_snapshot = transaction::directory_snapshot(&input)?;
        let mut files = collect_archive_files(&input)?;
        if files.len() != index.entries.len() {
            return Err(format!(
                "资源目录文件数与 arc.nsa 条目数不一致：目录 {} 个，归档 {} 个",
                files.len(),
                index.entries.len()
            )
            .into());
        }

        let mut archive = File::open(&source)?;
        let mut entries = Vec::with_capacity(index.entries.len());
        let mut changed_count = 0u64;
        for entry in index.entries {
            let key = archive_path_key(&entry.relative_path);
            let disk_path = files.remove(&key).ok_or_else(|| {
                vn_cli::Error(format!("资源目录缺少归档条目：{}", entry.display_name))
            })?;
            let replacement = fs::read(&disk_path).map_err(|error| {
                vn_cli::Error(format!("读取资源失败 {}：{error}", disk_path.display()))
            })?;
            let original_packed = read_entry_data(
                &mut archive,
                index.data_start,
                entry.offset,
                entry.packed_size,
            )?;
            let original_plain = decode_entry(&entry, &original_packed)?;
            let unchanged = replacement == original_plain;

            let (flag, packed_size, plain_size) = if unchanged {
                (entry.flag, entry.packed_size, entry.plain_size)
            } else {
                changed_count += 1;
                let size = u32::try_from(replacement.len()).map_err(|_| {
                    vn_cli::Error(format!("资源超过 NSA 单项长度上限：{}", entry.display_name))
                })? as usize;
                (FLAG_RAW, size, size)
            };
            entries.push(PackedArchiveEntry {
                source_entry: entry,
                disk_path,
                use_original_payload: unchanged,
                flag,
                packed_size,
                plain_size,
                relative_offset: 0,
            });
        }

        if let Some(extra) = files.values().next() {
            return Err(format!("资源目录包含不在 arc.nsa 中的文件：{}", extra.display()).into());
        }
        if transaction::directory_snapshot(&input)? != input_snapshot {
            return Err("资源目录在预检期间发生变化；拒绝重封包".into());
        }
        if fs::metadata(&source)?.len() != index.file_size
            || sha256_path(&source)? != index.source_sha256
        {
            return Err("arc.nsa 在预检期间发生变化；拒绝重封包".into());
        }

        let directory_size = entries.iter().try_fold(6u64, |size, entry| {
            size.checked_add(entry.source_entry.name_bytes.len() as u64 + 14)
                .ok_or_else(|| vn_cli::Error("NSA 目录大小溢出".into()))
        })?;
        let data_start = u32::try_from(directory_size)
            .map_err(|_| vn_cli::Error("NSA 目录超过 32 位偏移范围".into()))?;
        let mut payload_size = 0u64;
        for entry in &mut entries {
            entry.relative_offset = u32::try_from(payload_size)
                .map_err(|_| vn_cli::Error("NSA 数据偏移超过 32 位范围".into()))?;
            payload_size = payload_size
                .checked_add(entry.packed_size as u64)
                .ok_or_else(|| vn_cli::Error("NSA 数据区大小溢出".into()))?;
        }
        let output_size = directory_size
            .checked_add(payload_size)
            .ok_or_else(|| vn_cli::Error("NSA 输出大小溢出".into()))?;
        if output_size > u32::MAX as u64 {
            return Err("重建后的 arc.nsa 超过 4 GiB 格式上限".into());
        }

        let overwrite = parameters.flag("overwrite");
        let (output, original_output) =
            transaction::prepare_file_output(parameters.path("output")?, overwrite)?;
        transaction::ensure_inputs_disjoint_from_output(&[&source, &input], &output)?;
        let unchanged_count = entries.len() as u64 - changed_count;
        let preview = Preview {
            inputs: vec![source.clone(), input.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "校验原 arc.nsa 与完整资源目录的路径集合".into(),
                "与原条目逐项比较；未修改资源沿用原压缩字节，修改资源写为原样条目".into(),
                "重建 CP932 文件表和大端偏移，再暂存写出新归档".into(),
                "提交前复核原归档及资源目录未变化".into(),
            ],
            details: vec![
                format!("资源条目：{}", entries.len()),
                format!("未修改并保留压缩数据：{unchanged_count} 个"),
                format!("修改并改为原样存储：{changed_count} 个"),
                format!("数据区偏移：{data_start}；输出大小：{output_size} 字节"),
            ],
        };

        Ok(Box::new(PreparedPackArchive {
            source,
            source_sha256: index.source_sha256,
            source_size: index.file_size,
            input,
            input_snapshot,
            data_start: index.data_start,
            entries,
            output,
            original_output,
            overwrite,
            output_data_start: data_start,
            output_size,
            preview,
        }))
    }
}

struct PackedArchiveEntry {
    source_entry: ArchiveEntry,
    disk_path: PathBuf,
    use_original_payload: bool,
    flag: u8,
    packed_size: usize,
    plain_size: usize,
    relative_offset: u32,
}

struct PreparedPackArchive {
    source: PathBuf,
    source_sha256: String,
    source_size: u64,
    input: PathBuf,
    input_snapshot: Option<String>,
    data_start: u64,
    entries: Vec<PackedArchiveEntry>,
    output: PathBuf,
    original_output: Option<Vec<u8>>,
    overwrite: bool,
    output_data_start: u32,
    output_size: u64,
    preview: Preview,
}

impl PreparedOperation for PreparedPackArchive {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("复核原归档和资源目录")?;
        if fs::metadata(&self.source)?.len() != self.source_size
            || sha256_path(&self.source)? != self.source_sha256
        {
            return Err("arc.nsa 在预检后发生变化；未写出新归档".into());
        }
        if transaction::directory_snapshot(&self.input)? != self.input_snapshot {
            return Err("资源目录在预检后发生变化；未写出新归档".into());
        }

        progress.report("重建 NSA 文件表并流式写入数据")?;
        let source_path = self.source.clone();
        let source_sha256 = self.source_sha256.clone();
        let source_size = self.source_size;
        let input_path = self.input.clone();
        let input_snapshot = self.input_snapshot.clone();
        let data_start = self.data_start;
        let output_data_start = self.output_data_start;
        let entries = &self.entries;
        let progress_ref = progress;
        transaction::write_file_atomic_with(
            &self.output,
            &self.original_output,
            self.overwrite,
            |writer| {
                let count = u16::try_from(entries.len())
                    .map_err(|_| vn_cli::Error("NSA 条目数超过 16 位上限".into()))?;
                writer.write_all(&count.to_be_bytes())?;
                writer.write_all(&output_data_start.to_be_bytes())?;
                for entry in entries {
                    writer.write_all(&entry.source_entry.name_bytes)?;
                    writer.write_all(&[0, entry.flag])?;
                    writer.write_all(&entry.relative_offset.to_be_bytes())?;
                    writer.write_all(&(entry.packed_size as u32).to_be_bytes())?;
                    writer.write_all(&(entry.plain_size as u32).to_be_bytes())?;
                }

                let mut archive = File::open(&source_path)?;
                for (index, entry) in entries.iter().enumerate() {
                    let copied = if entry.use_original_payload {
                        let absolute_offset = data_start
                            .checked_add(entry.source_entry.offset)
                            .ok_or_else(|| vn_cli::Error("原归档数据偏移溢出".into()))?;
                        archive.seek(SeekFrom::Start(absolute_offset))?;
                        let mut limited =
                            (&mut archive).take(entry.source_entry.packed_size as u64);
                        std::io::copy(&mut limited, writer)?
                    } else {
                        let mut replacement = File::open(&entry.disk_path)?;
                        let metadata_size = replacement.metadata()?.len();
                        if metadata_size != entry.packed_size as u64 {
                            return Err(format!(
                                "资源在写入前发生长度变化：{}（预期 {}，当前 {}）",
                                entry.disk_path.display(),
                                entry.packed_size,
                                metadata_size
                            )
                            .into());
                        }
                        std::io::copy(&mut replacement, writer)?
                    };
                    if copied != entry.packed_size as u64 {
                        return Err(format!(
                            "NSA 条目写入长度不符：{}（写入 {}，预期 {}）",
                            entry.source_entry.display_name, copied, entry.packed_size
                        )
                        .into());
                    }
                    if (index + 1) % 100 == 0 || index + 1 == entries.len() {
                        progress_ref.report(&format!(
                            "已写入 {} / {} 个资源",
                            index + 1,
                            entries.len()
                        ))?;
                    }
                }

                if fs::metadata(&source_path)?.len() != source_size
                    || sha256_path(&source_path)? != source_sha256
                {
                    return Err("arc.nsa 在重封包期间发生变化；已丢弃暂存文件".into());
                }
                if transaction::directory_snapshot(&input_path)? != input_snapshot {
                    return Err("资源目录在重封包期间发生变化；已丢弃暂存文件".into());
                }
                Ok(())
            },
        )?;

        Ok(RunReport {
            summary: "arc.nsa 重封包完成。".into(),
            totals: vec![
                ("资源条目".into(), self.entries.len() as u64),
                ("输出字节".into(), self.output_size),
            ],
            outputs: vec![self.output],
            ..RunReport::default()
        })
    }
}

struct PreparedArchive {
    source: PathBuf,
    source_sha256: String,
    file_size: u64,
    data_start: u64,
    entries: Vec<ArchiveEntry>,
    output: PathBuf,
    original_snapshot: Option<String>,
    overwrite: bool,
    preview: Preview,
}

impl PreparedOperation for PreparedArchive {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("复核 arc.nsa 源文件")?;
        let mut archive = File::open(&self.source)?;
        if archive.metadata()?.len() != self.file_size
            || sha256_file(&mut archive)? != self.source_sha256
        {
            return Err("arc.nsa 在预检后发生变化；未写出资源".into());
        }

        progress.report("逐项解码并写入暂存目录")?;
        let progress_ref = progress;
        let leftover_backup = transaction::write_directory_atomic_with(
            &self.output,
            &self.original_snapshot,
            self.overwrite,
            |stage| {
                let mut source = File::open(&self.source)?;
                for (index, entry) in self.entries.iter().enumerate() {
                    let packed = read_entry_data(
                        &mut source,
                        self.data_start,
                        entry.offset,
                        entry.packed_size,
                    )?;
                    let unpacked = decode_entry(entry, &packed)?;
                    transaction::write_staged_file(stage, &entry.relative_path, &unpacked)?;
                    if (index + 1) % 100 == 0 || index + 1 == self.entries.len() {
                        progress_ref.report(&format!(
                            "已解码 {} / {} 个资源",
                            index + 1,
                            self.entries.len()
                        ))?;
                    }
                }
                if source.metadata()?.len() != self.file_size
                    || sha256_file(&mut source)? != self.source_sha256
                {
                    return Err("arc.nsa 在解码期间发生变化；已丢弃暂存目录".into());
                }
                Ok(())
            },
        )?;

        Ok(RunReport {
            summary: "arc.nsa 解封包完成。".into(),
            totals: vec![
                ("资源文件".into(), self.entries.len() as u64),
                (
                    "解封后字节".into(),
                    self.entries
                        .iter()
                        .map(|entry| entry.plain_size as u64)
                        .sum(),
                ),
            ],
            outputs: vec![self.output],
            warnings: leftover_backup.into_iter().collect(),
        })
    }
}

fn validate_archive_name(source: &Path) -> Result<()> {
    let extension = source.extension().ok_or("源文件缺少扩展名")?;
    if !extension.to_string_lossy().eq_ignore_ascii_case("nsa") {
        return Err("此操作要求输入 NSA 归档文件（.nsa）".into());
    }
    if !source.is_file() {
        return Err("arc.nsa 源路径必须是文件".into());
    }
    Ok(())
}

fn parse_archive_index(source: &Path) -> Result<ArchiveIndex> {
    let file = File::open(source)?;
    let file_size = file.metadata()?.len();
    let mut reader = BufReader::new(file);
    let count = u16::from_be_bytes(read_array::<2>(&mut reader)?) as usize;
    let data_start = u32::from_be_bytes(read_array::<4>(&mut reader)?) as u64;
    if count == 0 || data_start > file_size {
        return Err("arc.nsa 头部的文件数或数据区偏移无效".into());
    }

    let mut entries = Vec::with_capacity(count);
    let mut seen_paths = HashSet::with_capacity(count);
    let mut flag_counts = [0u64; 3];
    let mut unpacked_bytes = 0u64;
    for index in 0..count {
        let name_bytes = read_c_string(&mut reader, MAX_NAME_BYTES).map_err(|error| {
            vn_cli::Error(format!("第 {} 个 NSA 条目文件名无效：{error}", index + 1))
        })?;
        let (decoded_name, had_errors) = SHIFT_JIS.decode_without_bom_handling(&name_bytes);
        if had_errors || decoded_name.is_empty() {
            return Err(format!("第 {} 个 NSA 条目文件名不是有效 CP932", index + 1).into());
        }
        let display_name = decoded_name.into_owned();
        let relative_path = safe_archive_path(&display_name)?;
        let collision_key = relative_path.to_string_lossy().to_lowercase();
        if !seen_paths.insert(collision_key) {
            return Err(format!("NSA 中有重复的 Windows 路径：{display_name}").into());
        }

        let flag = read_array::<1>(&mut reader)?[0];
        if flag > FLAG_LZSS {
            return Err(format!("NSA 条目 {display_name} 使用未知压缩标记 {flag}").into());
        }
        let relative_offset = u32::from_be_bytes(read_array::<4>(&mut reader)?) as u64;
        let packed_size = u32::from_be_bytes(read_array::<4>(&mut reader)?) as usize;
        let plain_size = u32::from_be_bytes(read_array::<4>(&mut reader)?) as usize;
        let absolute_offset = data_start
            .checked_add(relative_offset)
            .ok_or_else(|| vn_cli::Error(format!("NSA 条目偏移溢出：{display_name}")))?;
        let end = absolute_offset
            .checked_add(packed_size as u64)
            .ok_or_else(|| vn_cli::Error(format!("NSA 条目长度溢出：{display_name}")))?;
        if end > file_size {
            return Err(format!("NSA 条目超出归档末尾：{display_name}").into());
        }
        if flag == FLAG_RAW && packed_size != plain_size {
            return Err(format!(
                "原样条目大小不一致：{display_name}（存储 {packed_size}，声明 {plain_size}）"
            )
            .into());
        }
        if flag == FLAG_SPB && !display_name.to_lowercase().ends_with(".bmp") {
            return Err(format!("SPB 压缩标记出现在非 BMP 资源：{display_name}").into());
        }
        flag_counts[flag as usize] += 1;
        unpacked_bytes = unpacked_bytes
            .checked_add(plain_size as u64)
            .ok_or("NSA 解封后总大小溢出")?;
        entries.push(ArchiveEntry {
            display_name,
            name_bytes,
            relative_path,
            flag,
            offset: relative_offset,
            packed_size,
            plain_size,
        });
    }

    let directory_end = reader.stream_position()?;
    if directory_end > data_start {
        return Err(format!(
            "NSA 目录超过数据区起点（目录结束 {directory_end}，数据区 {data_start}）"
        )
        .into());
    }
    let source_sha256 = sha256_path(source)?;
    Ok(ArchiveIndex {
        data_start,
        file_size,
        source_sha256,
        entries,
        unpacked_bytes,
        flag_counts,
    })
}

fn safe_archive_path(name: &str) -> Result<PathBuf> {
    let normalized = name.replace('\\', "/");
    if normalized.starts_with('/')
        || normalized.contains(':')
        || normalized.contains(['*', '?', '"', '<', '>', '|'])
    {
        return Err(format!("NSA 条目路径不是安全的相对路径：{name}").into());
    }
    let path = PathBuf::from(&normalized);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            !matches!(component, Component::Normal(_))
                || component
                    .as_os_str()
                    .to_string_lossy()
                    .ends_with([' ', '.'])
        })
    {
        return Err(format!("NSA 条目路径不是安全的相对路径：{name}").into());
    }
    Ok(path)
}

fn collect_archive_files(root: &Path) -> Result<HashMap<String, PathBuf>> {
    let mut files = HashMap::new();
    collect_archive_files_recursive(root, root, &mut files)?;
    Ok(files)
}

fn collect_archive_files_recursive(
    root: &Path,
    current: &Path,
    files: &mut HashMap<String, PathBuf>,
) -> Result<()> {
    let mut children = fs::read_dir(current)?.collect::<std::result::Result<Vec<_>, _>>()?;
    children.sort_by_key(|entry| entry.file_name());
    for child in children {
        let path = child.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(format!("资源目录包含符号链接：{}", path.display()).into());
        }
        if metadata.is_dir() {
            collect_archive_files_recursive(root, &path, files)?;
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|error| vn_cli::Error(format!("无法计算资源相对路径：{error}")))?;
            let key = archive_path_key(relative);
            if files.insert(key.clone(), path.clone()).is_some() {
                return Err(
                    format!("资源目录中有大小写不敏感的重复路径：{}", relative.display()).into(),
                );
            }
        } else {
            return Err(format!("资源目录包含不支持的文件类型：{}", path.display()).into());
        }
    }
    Ok(())
}

fn archive_path_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}

fn read_c_string(reader: &mut impl Read, max_bytes: usize) -> Result<Vec<u8>> {
    let mut name = Vec::new();
    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        if byte[0] == 0 {
            break;
        }
        if name.len() >= max_bytes {
            return Err(format!("文件名超过 {max_bytes} 字节").into());
        }
        name.push(byte[0]);
    }
    Ok(name)
}

fn read_array<const N: usize>(reader: &mut impl Read) -> Result<[u8; N]> {
    let mut bytes = [0u8; N];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn read_entry_data(
    source: &mut File,
    data_start: u64,
    relative_offset: u64,
    packed_size: usize,
) -> Result<Vec<u8>> {
    let offset = data_start
        .checked_add(relative_offset)
        .ok_or("NSA 条目偏移溢出")?;
    source.seek(SeekFrom::Start(offset))?;
    let mut packed = vec![0; packed_size];
    source.read_exact(&mut packed)?;
    Ok(packed)
}

fn decode_entry(entry: &ArchiveEntry, packed: &[u8]) -> Result<Vec<u8>> {
    let decoded = match entry.flag {
        FLAG_RAW => packed.to_vec(),
        FLAG_SPB => decode_spb_bmp(packed, entry.plain_size).map_err(|error| {
            vn_cli::Error(format!("SPB BMP 解码失败 {}：{error}", entry.display_name))
        })?,
        FLAG_LZSS => decode_lzss(packed, entry.plain_size).map_err(|error| {
            vn_cli::Error(format!("LZSS 解码失败 {}：{error}", entry.display_name))
        })?,
        flag => return Err(format!("未知 NSA 压缩标记 {flag}").into()),
    };
    if decoded.len() != entry.plain_size {
        return Err(format!(
            "NSA 条目解码长度不符 {}：得到 {}，声明 {}",
            entry.display_name,
            decoded.len(),
            entry.plain_size
        )
        .into());
    }
    Ok(decoded)
}

fn decode_lzss(packed: &[u8], expected_size: usize) -> std::result::Result<Vec<u8>, String> {
    let mut reader = MsbBitReader::new(packed);
    let mut dictionary = [b' '; 256];
    let mut write_position = 239usize;
    let mut output = Vec::with_capacity(expected_size);
    while output.len() < expected_size {
        if reader.read_bits(1)? != 0 {
            let byte = reader.read_bits(8)? as u8;
            output.push(byte);
            dictionary[write_position] = byte;
            write_position = (write_position + 1) & 0xff;
        } else {
            let read_position = reader.read_bits(8)? as usize;
            let length = reader.read_bits(4)? as usize + 2;
            if output.len().saturating_add(length) > expected_size {
                return Err("LZSS 回溯长度超过声明输出大小".into());
            }
            for index in 0..length {
                let byte = dictionary[(read_position + index) & 0xff];
                output.push(byte);
                dictionary[write_position] = byte;
                write_position = (write_position + 1) & 0xff;
            }
        }
    }
    Ok(output)
}

fn decode_spb_bmp(packed: &[u8], expected_size: usize) -> std::result::Result<Vec<u8>, String> {
    let mut reader = SpbBitReader::new(packed);
    let width = reader.read_bits(16)? as usize;
    let height = reader.read_bits(16)? as usize;
    if width == 0 || height == 0 || width > 16_384 || height > 16_384 {
        return Err(format!("BMP 尺寸无效：{width}x{height}"));
    }
    let row_stride = width
        .checked_mul(3)
        .and_then(|row| row.checked_add(3))
        .map(|row| row & !3)
        .ok_or_else(|| "BMP 行宽溢出".to_string())?;
    let pixel_size = row_stride
        .checked_mul(height)
        .ok_or_else(|| "BMP 像素数据长度溢出".to_string())?;
    let total_size = expected_size;
    let header_size = total_size.saturating_sub(pixel_size);
    if !matches!(header_size, 54 | 56) {
        return Err(format!(
            "BMP 尺寸 {width}x{height} 的像素数据为 {pixel_size} 字节，目录长度 {expected_size} 不匹配标准文件头"
        ));
    }

    let mut pixels = vec![0u8; pixel_size];
    for channel in 0..3 {
        let mut previous = reader.read_bits(8)? as i32;
        let mut row = 0usize;
        let mut column = 0usize;
        let mut direction = 1isize;
        put_spb_pixel(
            &mut pixels,
            row_stride,
            height,
            width,
            row,
            column,
            channel,
            previous,
        );
        column = 1;

        while row < height {
            let code = reader.read_bits(3)? as usize;
            let bit_width = match code {
                0 => 0,
                1..=6 => code + 2,
                _ => usize::from(reader.read_side_bit()?) + 1,
            };
            for _ in 0..4 {
                let value = if bit_width == 0 {
                    previous
                } else if bit_width == 8 {
                    reader.read_bits(8)? as i32
                } else {
                    let delta_code = reader.read_bits(bit_width)? as i32;
                    if delta_code & 1 == 1 {
                        previous + (delta_code + 1) / 2
                    } else {
                        previous - delta_code / 2
                    }
                };
                previous = value;
                if row < height {
                    put_spb_pixel(
                        &mut pixels,
                        row_stride,
                        height,
                        width,
                        row,
                        column,
                        channel,
                        value,
                    );
                }
                if direction > 0 {
                    column += 1;
                    if column >= width {
                        row += 1;
                        column = width.saturating_sub(1);
                        direction = -1;
                    }
                } else if column == 0 {
                    row += 1;
                    column = 0;
                    direction = 1;
                } else {
                    column -= 1;
                }
            }
        }
    }

    let mut bmp = Vec::with_capacity(total_size);
    bmp.extend_from_slice(b"BM");
    put_u32_le(&mut bmp, total_size as u32);
    bmp.extend_from_slice(&[0; 4]);
    put_u32_le(&mut bmp, header_size as u32);
    put_u32_le(&mut bmp, 40);
    put_i32_le(&mut bmp, width as i32);
    put_i32_le(&mut bmp, height as i32);
    put_u16_le(&mut bmp, 1);
    put_u16_le(&mut bmp, 24);
    put_u32_le(&mut bmp, 0);
    put_u32_le(&mut bmp, pixel_size as u32);
    put_i32_le(&mut bmp, 0);
    put_i32_le(&mut bmp, 0);
    put_u32_le(&mut bmp, 0);
    put_u32_le(&mut bmp, 0);
    if header_size == 56 {
        bmp.extend_from_slice(&[0; 2]);
    }
    bmp.extend_from_slice(&pixels);
    Ok(bmp)
}

#[allow(clippy::too_many_arguments)]
fn put_spb_pixel(
    pixels: &mut [u8],
    row_stride: usize,
    height: usize,
    width: usize,
    row: usize,
    column: usize,
    channel: usize,
    value: i32,
) {
    if row < height && column < width {
        let bottom_up_row = height - row - 1;
        let offset = bottom_up_row * row_stride + column * 3 + channel;
        pixels[offset] = value as u8;
    }
}

fn put_u16_le(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn put_u32_le(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn put_i32_le(output: &mut Vec<u8>, value: i32) {
    output.extend_from_slice(&value.to_le_bytes());
}

struct MsbBitReader<'a> {
    bytes: &'a [u8],
    bit_position: usize,
}

impl<'a> MsbBitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bit_position: 0,
        }
    }

    fn read_bits(&mut self, count: usize) -> std::result::Result<u32, String> {
        let mut value = 0u32;
        for _ in 0..count {
            let byte_index = self.bit_position / 8;
            if byte_index >= self.bytes.len() {
                return Err("LZSS 数据提前结束".into());
            }
            let bit_index = self.bit_position % 8;
            value = (value << 1) | u32::from((self.bytes[byte_index] >> (7 - bit_index)) & 1);
            self.bit_position += 1;
        }
        Ok(value)
    }
}

struct SpbBitReader<'a> {
    bytes: &'a [u8],
    position: usize,
    buffered_byte: u8,
    bit_index: u8,
}

impl<'a> SpbBitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            position: 0,
            buffered_byte: 0,
            bit_index: 0,
        }
    }

    fn read_raw_byte(&mut self) -> std::result::Result<u8, String> {
        let byte = *self
            .bytes
            .get(self.position)
            .ok_or_else(|| "SPB 数据提前结束".to_string())?;
        self.position += 1;
        Ok(byte)
    }

    fn read_bit(&mut self) -> std::result::Result<u8, String> {
        if self.bit_index == 0 {
            self.buffered_byte = self.read_raw_byte()?;
        }
        let value = (self.buffered_byte >> (7 - self.bit_index)) & 1;
        self.bit_index = (self.bit_index + 1) & 7;
        Ok(value)
    }

    fn read_bits(&mut self, count: usize) -> std::result::Result<u32, String> {
        let mut value = 0u32;
        for _ in 0..count {
            value = (value << 1) | u32::from(self.read_bit()?);
        }
        Ok(value)
    }

    fn read_side_bit(&mut self) -> std::result::Result<bool, String> {
        Ok(self.read_bit()? != 0)
    }
}

fn sha256_path(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    sha256_file(&mut file)
}

fn sha256_file(file: &mut File) -> Result<String> {
    file.seek(SeekFrom::Start(0))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1024 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect())
}
