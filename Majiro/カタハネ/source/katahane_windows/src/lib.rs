use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

mod arc_pack;
mod mjo;
mod mjo_bytecode;
mod mjo_rebuild_clean;
mod mjo_text_clean;
mod translation;
mod workflow;
use arc_pack::pack_archive;
use mjo::{MjoDocument, MjoVersion};
pub use translation::{ExportText, ImportText};
use vn_cli::{
    Field, FieldKind, Operation, OperationSpec, Parameters, PreparedOperation, Preview, Progress,
    RunReport, Value,
};
pub use workflow::{InjectAndPack, UnpackAndExtract};

const HEADER_SIZE: usize = 0x1c;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveVersion {
    V1,
    V2,
    V3,
}

impl ArchiveVersion {
    fn index_entry_size(self) -> usize {
        match self {
            Self::V1 => 8,
            Self::V2 => 12,
            Self::V3 => 16,
        }
    }

    fn has_terminal_index_entry(self) -> bool {
        self == Self::V1
    }

    fn hash_width(self) -> usize {
        if self == Self::V3 {
            8
        } else {
            4
        }
    }

    fn signature(self) -> &'static [u8; 16] {
        match self {
            Self::V1 => b"MajiroArcV1.000\0",
            Self::V2 => b"MajiroArcV2.000\0",
            Self::V3 => b"MajiroArcV3.000\0",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveEntry {
    /// Original, unmodified string from the archive's NUL-separated name area.
    pub archive_name: String,
    /// Loadable member name resolved by matching the archive's CRC index.
    pub name: String,
    pub relative_path: PathBuf,
    pub name_hash: u64,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Archive {
    pub version: ArchiveVersion,
    pub entries: Vec<ArchiveEntry>,
    pub names_offset: usize,
    pub data_offset: usize,
    pub file_size: usize,
}

impl Archive {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < HEADER_SIZE {
            return Err("文件短于 Majiro ARC 头部".into());
        }
        let count = read_u32(bytes, 0x10)? as usize;
        let names_offset = read_u32(bytes, 0x14)? as usize;
        let mut parsed = Vec::new();
        let mut errors = Vec::new();
        for version in [ArchiveVersion::V1, ArchiveVersion::V2, ArchiveVersion::V3] {
            let index_count = count
                .checked_add(usize::from(version.has_terminal_index_entry()))
                .ok_or("索引条目数溢出")?;
            let Some(index_bytes) = index_count.checked_mul(version.index_entry_size()) else {
                continue;
            };
            if HEADER_SIZE.checked_add(index_bytes) != Some(names_offset) {
                continue;
            }
            match Self::parse_with_layout(bytes, version) {
                Ok(archive) => parsed.push(archive),
                Err(error) => errors.push(error),
            }
        }

        if let Some(hint) = archive_version_hint(bytes) {
            if let Some(index) = parsed.iter().position(|archive| archive.version == hint) {
                return Ok(parsed.swap_remove(index));
            }
        }
        match parsed.len() {
            1 => Ok(parsed.pop().expect("one candidate remains")),
            0 if errors.is_empty() => Err(format!(
                "没有可识别的 ARC 索引布局：索引边界 0x{names_offset:X} 与文件数 {count} 不匹配"
            )),
            0 => Err(errors.join("；")),
            _ => Err("ARC 结构同时符合多个索引布局，且没有可用的签名提示".into()),
        }
    }

    fn parse_with_layout(bytes: &[u8], version: ArchiveVersion) -> Result<Self, String> {
        if bytes.len() < HEADER_SIZE {
            return Err("文件短于 Majiro ARC 头部".into());
        }

        let count = read_u32(bytes, 0x10)? as usize;
        let names_offset = read_u32(bytes, 0x14)? as usize;
        let data_offset = read_u32(bytes, 0x18)? as usize;
        let index_count = count
            .checked_add(usize::from(version.has_terminal_index_entry()))
            .ok_or("索引条目数溢出")?;
        let index_bytes = index_count
            .checked_mul(version.index_entry_size())
            .ok_or("索引大小溢出")?;
        let index_end = HEADER_SIZE.checked_add(index_bytes).ok_or("索引末尾溢出")?;
        if index_end != names_offset {
            return Err(format!(
                "索引边界错误：0x{index_end:X} != 文件名区偏移 0x{names_offset:X}"
            ));
        }
        if names_offset > data_offset || data_offset > bytes.len() {
            return Err("文件名区或数据区偏移超出文件范围".into());
        }
        if bytes.len() > u32::MAX as usize {
            return Err("ARC 超过 32 位格式可寻址范围".into());
        }

        let sentinel_offset = if version.has_terminal_index_entry() {
            let sentinel_pos = HEADER_SIZE + count * version.index_entry_size();
            let offset = read_u32(bytes, sentinel_pos + 4)? as usize;
            if offset != bytes.len() {
                return Err(format!(
                    "末尾哨兵偏移 0x{offset:X} 与文件长度 0x{:X} 不一致",
                    bytes.len()
                ));
            }
            offset
        } else {
            bytes.len()
        };

        let mut index = Vec::with_capacity(count);
        let mut last_hash = None;
        for i in 0..count {
            let entry_size = version.index_entry_size();
            let pos = HEADER_SIZE + i * entry_size;
            let hash = match version.hash_width() {
                4 => u64::from(read_u32(bytes, pos)?),
                8 => read_u64(bytes, pos)?,
                _ => unreachable!("Majiro ARC hash width is described by ArchiveVersion"),
            };
            let offset_field = pos + version.hash_width();
            let offset = read_u32(bytes, offset_field)? as usize;
            let explicit_size = if version.has_terminal_index_entry() {
                None
            } else {
                Some(read_u32(bytes, offset_field + 4)? as usize)
            };
            if last_hash.is_some_and(|previous| hash < previous) {
                return Err(format!("索引哈希未排序（第 {} 项）", i + 1));
            }
            if offset < data_offset || offset > sentinel_offset {
                return Err(format!("第 {} 项的数据偏移越界", i + 1));
            }
            let end = match explicit_size {
                Some(size) => offset
                    .checked_add(size)
                    .ok_or_else(|| format!("第 {} 项的数据范围溢出", i + 1))?,
                None => 0,
            };
            if explicit_size.is_some() && end > bytes.len() {
                return Err(format!("第 {} 项的数据范围超出文件末尾", i + 1));
            }
            last_hash = Some(hash);
            index.push((hash, offset, explicit_size));
        }
        if count == 0 && data_offset != names_offset {
            return Err("空档案的名称区与数据区偏移不一致".into());
        }

        let mut cursor = names_offset;
        let mut entries = Vec::with_capacity(count);
        let mut names_seen = BTreeSet::new();
        let mut files_seen: BTreeSet<String> = BTreeSet::new();
        for i in 0..count {
            let tail = bytes
                .get(cursor..data_offset)
                .ok_or("文件名区超出文件范围")?;
            let nul = tail
                .iter()
                .position(|byte| *byte == 0)
                .ok_or_else(|| format!("第 {} 个文件名没有 NUL 终止符", i + 1))?;
            if nul == 0 {
                return Err(format!("第 {} 个文件名为空", i + 1));
            }
            let raw_name = &tail[..nul];
            let (decoded_archive_name, had_errors) =
                encoding_rs::SHIFT_JIS.decode_without_bom_handling(raw_name);
            if had_errors {
                return Err(format!("第 {} 个文件名不是有效 Shift-JIS/CP932", i + 1));
            }
            let archive_name = decoded_archive_name.into_owned();
            let raw_basename = basename_bytes(raw_name);
            let (expected_hash, start, explicit_size) = index[i];
            let actual_hash = name_hash_for_version(version, raw_basename);
            if actual_hash != expected_hash {
                return Err(format!(
                    "档案名与索引哈希不匹配: {archive_name} (文件名 0x{actual_hash:016X}, 索引 0x{expected_hash:016X})"
                ));
            }
            let name = archive_name.clone();
            let relative_path = safe_relative_path(&name)?;
            let key = relative_path
                .to_string_lossy()
                .replace('\\', "/")
                .to_lowercase();
            if !names_seen.insert(key.clone()) {
                return Err(format!("重复的 Windows 文件路径: {name}"));
            }
            for prior in &files_seen {
                if key.starts_with(&format!("{prior}/")) || prior.starts_with(&format!("{key}/")) {
                    return Err(format!("文件名产生文件/目录冲突: {name}"));
                }
            }
            files_seen.insert(key);

            let end = match explicit_size {
                Some(size) => start
                    .checked_add(size)
                    .ok_or_else(|| format!("第 {} 项的数据范围溢出", i + 1))?,
                None => index.get(i + 1).map_or(sentinel_offset, |entry| entry.1),
            };
            if end < start || end > bytes.len() {
                return Err(format!("第 {} 个条目的数据范围无效", i + 1));
            }
            entries.push(ArchiveEntry {
                archive_name,
                name,
                relative_path,
                name_hash: expected_hash,
                start,
                end,
            });
            cursor += nul + 1;
        }
        if cursor != data_offset {
            return Err(format!(
                "文件名区结束于 0x{cursor:X}，数据区从 0x{data_offset:X} 开始"
            ));
        }

        let mut physical_ranges = entries
            .iter()
            .map(|entry| (entry.start, entry.end))
            .collect::<Vec<_>>();
        physical_ranges.sort_unstable();
        for pair in physical_ranges.windows(2) {
            if pair[1].0 < pair[0].1 {
                return Err("档案条目的数据范围相互重叠".into());
            }
        }

        Ok(Self {
            version,
            entries,
            names_offset,
            data_offset,
            file_size: bytes.len(),
        })
    }
}

fn archive_version_hint(bytes: &[u8]) -> Option<ArchiveVersion> {
    [ArchiveVersion::V1, ArchiveVersion::V2, ArchiveVersion::V3]
        .into_iter()
        .find(|version| bytes.get(..16) == Some(version.signature().as_slice()))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let field = bytes
        .get(offset..offset.checked_add(4).ok_or("字段偏移溢出")?)
        .ok_or_else(|| format!("读取头部/索引字段越界: 0x{offset:X}"))?;
    Ok(u32::from_le_bytes(
        field.try_into().expect("four-byte slice"),
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, String> {
    let field = bytes
        .get(offset..offset.checked_add(8).ok_or("字段偏移溢出")?)
        .ok_or_else(|| format!("读取头部/索引字段越界: 0x{offset:X}"))?;
    Ok(u64::from_le_bytes(
        field.try_into().expect("eight-byte slice"),
    ))
}

fn basename_bytes(name: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut cursor = 0;
    while cursor < name.len() {
        let byte = name[cursor];
        if is_shift_jis_lead(byte) && cursor + 1 < name.len() {
            cursor += 2;
        } else {
            if matches!(byte, b'/' | b'\\') {
                start = cursor + 1;
            }
            cursor += 1;
        }
    }
    &name[start..]
}

fn is_shift_jis_lead(byte: u8) -> bool {
    (0x81..=0x9f).contains(&byte) || (0xe0..=0xfc).contains(&byte)
}

/// Matches Majiro's multibyte-aware lowercasing before its CRC name hash.
fn name_hash_for_version(version: ArchiveVersion, bytes: &[u8]) -> u64 {
    match version {
        ArchiveVersion::V1 | ArchiveVersion::V2 => u64::from(crc32_lower_shift_jis(bytes)),
        ArchiveVersion::V3 => crc64_lower_shift_jis(bytes),
    }
}

fn for_each_lower_shift_jis_byte(bytes: &[u8], mut visit: impl FnMut(u8)) {
    let mut cursor = 0;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if is_shift_jis_lead(byte) && cursor + 1 < bytes.len() {
            visit(byte);
            let trail = bytes[cursor + 1];
            // The engine's sub_47C650 also folds full-width Latin capitals
            // (U+FF21..U+FF3A: CP932 82 60..82 79) before hashing.
            visit(if byte == 0x82 && (0x60..=0x79).contains(&trail) {
                trail + 0x21
            } else {
                trail
            });
            cursor += 2;
        } else {
            visit(byte.to_ascii_lowercase());
            cursor += 1;
        }
    }
}

/// Matches Majiro's CRC-32 name hash after multibyte-aware lowercase folding.
fn crc32_lower_shift_jis(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for_each_lower_shift_jis_byte(bytes, |byte| crc = crc32_update_byte(crc, byte));
    !crc
}

/// Majiro ARC V3 stores the same case-folded filename hash at 64 bits.
fn crc64_lower_shift_jis(bytes: &[u8]) -> u64 {
    // sub_479150's DWORD table uses XOR-before-shift, so use its exact recurrence
    // instead of substituting a library CRC64 variant.
    const POLY_LOW: u32 = 0x53d4_6d27;
    const POLY_HIGH: u32 = 0x85e1_c3d7;
    let mut table = [0u64; 256];
    for (index, slot) in table.iter_mut().enumerate() {
        let mut low = index as u32;
        let mut high = 0u32;
        for _ in 0..8 {
            if low & 1 != 0 {
                low ^= POLY_LOW;
                high ^= POLY_HIGH;
            }
            let shifted = ((u64::from(high) << 32) | u64::from(low)) >> 1;
            low = shifted as u32;
            high = (shifted >> 32) as u32;
        }
        *slot = (u64::from(high) << 32) | u64::from(low);
    }

    let mut crc = u64::MAX;
    for_each_lower_shift_jis_byte(bytes, |byte| {
        let index = ((crc as u8) ^ byte) as usize;
        crc = (crc >> 8) ^ table[index];
    });
    !crc
}

fn crc32_update_byte(mut crc: u32, byte: u8) -> u32 {
    crc ^= u32::from(byte);
    for _ in 0..8 {
        crc = if crc & 1 != 0 {
            (crc >> 1) ^ 0xedb8_8320
        } else {
            crc >> 1
        };
    }
    crc
}

fn safe_relative_path(name: &str) -> Result<PathBuf, String> {
    if name.is_empty() || name.contains('\0') {
        return Err("档案内含空文件名".into());
    }
    let normalized = name.replace('\\', "/");
    if normalized.starts_with('/') || normalized.starts_with("//") {
        return Err(format!("拒绝绝对路径: {name}"));
    }
    let mut relative = PathBuf::new();
    for part in normalized.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            return Err(format!("拒绝不安全的相对路径: {name}"));
        }
        if part.ends_with([' ', '.']) || part.chars().any(|ch| ch < ' ' || "<>:\"|?*".contains(ch))
        {
            return Err(format!("Windows 不支持安全提取的文件名: {name}"));
        }
        let stem = part.split('.').next().unwrap_or(part).to_ascii_uppercase();
        if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        {
            return Err(format!("Windows 保留设备名不能提取: {name}"));
        }
        relative.push(part);
    }
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("拒绝非普通相对路径: {name}"));
    }
    Ok(relative)
}

pub struct Unpack;

impl Operation for Unpack {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "unpack",
            "解包 scenario.arc",
            vec![
                Field::new("source", "Majiro ARC 文件", FieldKind::Path).required(),
                Field::new("output", "解包目录", FieldKind::Path).required(),
            ],
        );
        spec.description = "按结构识别 ARC 索引布局，校验版本哈希后按原偏移提取成员。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(path) = paths.first() {
            parameters.set("source", Value::Path(path.clone()));
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let source = fs::canonicalize(parameters.path("source")?)?;
        if !source.is_file() {
            return Err("ARC 输入必须是文件".into());
        }
        let bytes = fs::read(&source)?;
        let archive = Archive::parse(&bytes).map_err(vn_cli::Error)?;
        let output = resolve_output(parameters.path("output")?)?;
        let original_output = snapshot_output(&output)?;
        let overwrite = parameters.flag("overwrite");
        if !matches!(original_output, OutputSnapshot::Missing) && !overwrite {
            return Err("输出目录已存在；请选择新目录或显式启用 --overwrite".into());
        }
        let total_bytes = archive
            .entries
            .iter()
            .map(|entry| entry.end - entry.start)
            .sum::<usize>();
        let preview = Preview {
            inputs: vec![source.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "读取 ARC 头部，按推断出的版本校验索引哈希和 CP932 文件名".into(),
                "将条目原始字节写入独立暂存目录".into(),
                "校验输出状态后提交解包目录".into(),
            ],
            details: vec![
                format!("条目数: {}", archive.entries.len()),
                format!("原始条目数据: {total_bytes} 字节"),
            ],
        };
        Ok(Box::new(UnpackJob {
            preview,
            bytes,
            archive,
            output,
            original_output,
            overwrite,
        }))
    }
}

struct UnpackJob {
    preview: Preview,
    bytes: Vec<u8>,
    archive: Archive,
    output: PathBuf,
    original_output: OutputSnapshot,
    overwrite: bool,
}

impl PreparedOperation for UnpackJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        let parent = self.output.parent().ok_or("输出目录缺少父目录")?;
        let stage =
            StageDirectory::create(parent, self.output.file_name().ok_or("输出目录名为空")?)?;
        let mut total_bytes = 0usize;
        for (index, entry) in self.archive.entries.iter().enumerate() {
            progress.report(&format!(
                "提取 {}/{}: {}",
                index + 1,
                self.archive.entries.len(),
                entry.name
            ))?;
            let destination = stage.path.join(&entry.relative_path);
            let entry_parent = destination.parent().ok_or("条目缺少父目录")?;
            fs::create_dir_all(entry_parent)?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&destination)?;
            let data = self
                .bytes
                .get(entry.start..entry.end)
                .ok_or("条目数据范围越界")?;
            file.write_all(data)?;
            file.sync_all()?;
            total_bytes += data.len();
        }
        let warning = commit_stage(&stage, &self.output, &self.original_output, self.overwrite)?;
        Ok(RunReport {
            summary: format!("解包完成，共 {} 个原始条目。", self.archive.entries.len()),
            totals: vec![
                ("文件数".into(), self.archive.entries.len() as u64),
                ("数据字节".into(), total_bytes as u64),
            ],
            outputs: vec![self.output.clone()],
            warnings: warning.into_iter().collect(),
        })
    }
}

pub struct DecodeMjo;

impl Operation for DecodeMjo {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "decode-mjo",
            "解码 MajiroObj",
            vec![
                Field::new("source", "解包后的 MJO 目录", FieldKind::Path).required(),
                Field::new("output", "V1 解码输出目录", FieldKind::Path).required(),
            ],
        );
        spec.primary = false;
        spec.description =
            "按 MJO 头部验证符号表与负载长度；将 X1 的循环 CRC32 表异或负载还原为 V1。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(source) = paths.first() {
            parameters.set("source", Value::Path(source.clone()));
            if parameters.get("output").is_none() {
                if let Some(parent) = source.parent() {
                    parameters.set("output", Value::Path(parent.join("mjo_decoded")));
                }
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let source = fs::canonicalize(parameters.path("source")?)?;
        if !source.is_dir() {
            return Err("MJO 输入必须是目录".into());
        }
        let mut decoded_files = Vec::new();
        let mut skipped_files = 0usize;
        collect_mjo_files(&source, &source, &mut decoded_files, &mut skipped_files)?;
        decoded_files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        if decoded_files.is_empty() {
            return Err("输入目录中没有 .mjo 文件".into());
        }

        let output = resolve_output(parameters.path("output")?)?;
        let original_output = snapshot_output(&output)?;
        let overwrite = parameters.flag("overwrite");
        if !matches!(original_output, OutputSnapshot::Missing) && !overwrite {
            return Err("输出目录已存在；请选择新目录或显式启用 --overwrite".into());
        }
        let x1_count = decoded_files
            .iter()
            .filter(|file| file.version == MjoVersion::X1)
            .count();
        let v1_count = decoded_files.len() - x1_count;
        let output_bytes = decoded_files
            .iter()
            .map(|file| file.bytes.len())
            .sum::<usize>();
        let mut details = vec![
            format!("MJO 文件数: {}", decoded_files.len()),
            format!("X1 异或解码: {x1_count}"),
            format!("原本为 V1: {v1_count}"),
            format!("输出字节: {output_bytes}"),
        ];
        if skipped_files > 0 {
            details.push(format!("跳过非 MJO 文件: {skipped_files}"));
        }
        let preview = Preview {
            inputs: vec![source.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "解析所有 .mjo 头部并验证已声明负载未越界".into(),
                "仅对 MajiroObjX1.000 负载应用循环 1024 字节 CRC32 表异或".into(),
                "保留头部、符号表、负载尾部并写出 MajiroObjV1.000".into(),
                "校验输出状态后提交完整目录".into(),
            ],
            details,
        };
        Ok(Box::new(DecodeMjoJob {
            preview,
            decoded_files,
            output,
            original_output,
            overwrite,
            skipped_files,
            x1_count,
            v1_count,
            output_bytes,
        }))
    }
}

struct DecodedMjoFile {
    relative_path: PathBuf,
    bytes: Vec<u8>,
    version: MjoVersion,
}

fn collect_mjo_files(
    root: &Path,
    directory: &Path,
    decoded_files: &mut Vec<DecodedMjoFile>,
    skipped_files: &mut usize,
) -> vn_cli::Result<()> {
    for item in fs::read_dir(directory)? {
        let item = item?;
        let path = item.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(format!("MJO 输入目录含符号链接，拒绝读取: {}", path.display()).into());
        }
        if metadata.is_dir() {
            collect_mjo_files(root, &path, decoded_files, skipped_files)?;
            continue;
        }
        if !metadata.is_file() {
            return Err(format!("MJO 输入含不支持的文件类型: {}", path.display()).into());
        }
        if !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("mjo"))
        {
            *skipped_files += 1;
            continue;
        }
        let relative_path = path
            .strip_prefix(root)
            .map_err(|_| "MJO 输入路径越界")?
            .to_path_buf();
        if relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err("MJO 输入包含非普通相对路径".into());
        }
        let original = fs::read(&path)?;
        let document = MjoDocument::parse(&original).map_err(|error| {
            vn_cli::Error(format!("无法解析 {}: {error}", relative_path.display()))
        })?;
        let bytes = document.as_v1_bytes(&original).map_err(|error| {
            vn_cli::Error(format!("无法解码 {}: {error}", relative_path.display()))
        })?;
        decoded_files.push(DecodedMjoFile {
            relative_path,
            bytes,
            version: document.version,
        });
    }
    Ok(())
}

struct DecodeMjoJob {
    preview: Preview,
    decoded_files: Vec<DecodedMjoFile>,
    output: PathBuf,
    original_output: OutputSnapshot,
    overwrite: bool,
    skipped_files: usize,
    x1_count: usize,
    v1_count: usize,
    output_bytes: usize,
}

impl PreparedOperation for DecodeMjoJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        let parent = self.output.parent().ok_or("输出目录缺少父目录")?;
        let stage =
            StageDirectory::create(parent, self.output.file_name().ok_or("输出目录名为空")?)?;
        for (index, decoded) in self.decoded_files.iter().enumerate() {
            progress.report(&format!(
                "解码 {}/{}: {}",
                index + 1,
                self.decoded_files.len(),
                decoded.relative_path.display()
            ))?;
            let destination = stage.path.join(&decoded.relative_path);
            let entry_parent = destination.parent().ok_or("MJO 文件缺少父目录")?;
            fs::create_dir_all(entry_parent)?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(destination)?;
            file.write_all(&decoded.bytes)?;
            file.sync_all()?;
        }
        let warning = commit_stage(&stage, &self.output, &self.original_output, self.overwrite)?;
        let mut warnings = warning.into_iter().collect::<Vec<_>>();
        if self.skipped_files > 0 {
            warnings.push(format!("跳过了 {} 个非 .mjo 文件。", self.skipped_files));
        }
        Ok(RunReport {
            summary: format!("MJO 解码完成，共 {} 个文件。", self.decoded_files.len()),
            totals: vec![
                ("X1 解码".into(), self.x1_count as u64),
                ("原本 V1".into(), self.v1_count as u64),
                ("输出字节".into(), self.output_bytes as u64),
            ],
            outputs: vec![self.output.clone()],
            warnings,
        })
    }
}

pub struct PackArchive;

impl Operation for PackArchive {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "pack-arc",
            "封包 scenario.arc",
            vec![
                Field::new("template", "原始 Majiro ARC 模板", FieldKind::Path).required(),
                Field::new("source", "完整成员目录", FieldKind::Path).required(),
                Field::new("output", "封包输出文件", FieldKind::Path).required(),
            ],
        );
        spec.primary = false;
        spec.description =
            "沿用模板签名、名称区与哈希顺序，按 ARC 版本重算成员偏移和显式长度。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(template) = paths.first() {
            parameters.set("template", Value::Path(template.clone()));
            if let Some(parent) = template.parent() {
                if parameters.get("output").is_none() {
                    parameters.set("output", Value::Path(parent.join("scenario_repacked.arc")));
                }
            }
        }
        if let Some(source) = paths.get(1) {
            parameters.set("source", Value::Path(source.clone()));
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let template_path = fs::canonicalize(parameters.path("template")?)?;
        if !template_path.is_file() {
            return Err("ARC 模板必须是文件".into());
        }
        let source = fs::canonicalize(parameters.path("source")?)?;
        if !source.is_dir() {
            return Err("封包成员输入必须是目录".into());
        }
        let template = fs::read(&template_path)?;
        let archive = Archive::parse(&template).map_err(vn_cli::Error)?;
        let mut files = BTreeMap::new();
        collect_member_files(&source, &source, &mut files)?;
        let packed = pack_archive(&template, &files).map_err(vn_cli::Error)?;
        let output = resolve_output(parameters.path("output")?)?;
        let original_output = snapshot_file_output(&output)?;
        let overwrite = parameters.flag("overwrite");
        if original_output.is_some() && !overwrite {
            return Err("输出文件已存在；请选择新路径或显式启用 --overwrite".into());
        }
        let source_bytes = files.values().map(Vec::len).sum::<usize>();
        let preview = Preview {
            inputs: vec![template_path.clone(), source.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "按原始 ARC 模板验证成员完整性和相对路径".into(),
                "按模板索引顺序重建数据区与全部条目偏移".into(),
                "更新 EOF 哨兵并提交单个 ARC 文件".into(),
            ],
            details: vec![
                format!("成员数: {}", archive.entries.len()),
                format!("成员数据: {source_bytes} 字节"),
                format!("封包大小: {} 字节", packed.len()),
            ],
        };
        Ok(Box::new(PackArchiveJob {
            preview,
            packed,
            output,
            original_output,
            overwrite,
        }))
    }
}

fn collect_member_files(
    root: &Path,
    directory: &Path,
    files: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> vn_cli::Result<()> {
    for item in fs::read_dir(directory)? {
        let item = item?;
        let path = item.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(format!("封包输入目录含符号链接，拒绝读取: {}", path.display()).into());
        }
        if metadata.is_dir() {
            collect_member_files(root, &path, files)?;
            continue;
        }
        if !metadata.is_file() {
            return Err(format!("封包输入含不支持的文件类型: {}", path.display()).into());
        }
        let relative_path = path
            .strip_prefix(root)
            .map_err(|_| "封包成员路径越界")?
            .to_path_buf();
        if relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err("封包成员包含非普通相对路径".into());
        }
        files.insert(relative_path, fs::read(path)?);
    }
    Ok(())
}

struct PackArchiveJob {
    preview: Preview,
    packed: Vec<u8>,
    output: PathBuf,
    original_output: Option<Vec<u8>>,
    overwrite: bool,
}

impl PreparedOperation for PackArchiveJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        progress.report("写入暂存 ARC 并提交输出")?;
        let parent = self.output.parent().ok_or("输出文件缺少父目录")?;
        let output_name = self.output.file_name().ok_or("输出文件名为空")?;
        let stage = StageDirectory::create(parent, output_name)?;
        let stage_file = stage.path.join(output_name);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&stage_file)?;
        file.write_all(&self.packed)?;
        file.sync_all()?;
        drop(file);
        let warning = commit_file(
            &stage_file,
            &self.output,
            &self.original_output,
            self.overwrite,
        )?;
        Ok(RunReport {
            summary: "scenario.arc 封包完成。".into(),
            totals: vec![("封包字节".into(), self.packed.len() as u64)],
            outputs: vec![self.output.clone()],
            warnings: warning.into_iter().collect(),
        })
    }
}

fn snapshot_file_output(path: &Path) -> vn_cli::Result<Option<Vec<u8>>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("封包输出已存在，但不是普通文件".into());
    }
    Ok(Some(fs::read(path)?))
}

fn commit_file(
    stage: &Path,
    output: &Path,
    expected: &Option<Vec<u8>>,
    overwrite: bool,
) -> vn_cli::Result<Option<String>> {
    let current = snapshot_file_output(output)?;
    if &current != expected {
        return Err("ARC 输出文件在预检后发生变化，请重新预检".into());
    }
    match expected {
        None => rename_no_replace(stage, output)?,
        Some(_) if overwrite => {
            let backup = unique_sibling(output, "backup")?;
            rename_no_replace(output, &backup)?;
            if let Err(error) = rename_no_replace(stage, output) {
                return match rename_no_replace(&backup, output) {
                    Ok(()) => Err(format!("提交 ARC 失败，旧输出已恢复: {error}").into()),
                    Err(restore_error) => Err(format!(
                        "提交 ARC 失败: {error}; 旧输出恢复失败: {restore_error}; 备份保留在 {}",
                        backup.display()
                    )
                    .into()),
                };
            }
            if let Err(error) = fs::remove_file(&backup) {
                return Ok(Some(format!(
                    "新 ARC 已提交；旧输出备份未能清理（{error}）: {}",
                    backup.display()
                )));
            }
        }
        Some(_) => return Err("ARC 输出文件已存在".into()),
    }
    Ok(None)
}

fn resolve_output(path: &Path) -> vn_cli::Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    let parent = absolute.parent().ok_or("输出路径缺少父目录")?;
    let name = absolute.file_name().ok_or("输出路径缺少文件名")?;
    let canonical_parent = fs::canonicalize(parent)?;
    if !canonical_parent.is_dir() {
        return Err("输出父路径不是目录".into());
    }
    Ok(canonical_parent.join(name))
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum OutputSnapshot {
    Missing,
    Directory {
        directories: BTreeSet<PathBuf>,
        files: BTreeMap<PathBuf, Vec<u8>>,
    },
}

fn snapshot_output(path: &Path) -> vn_cli::Result<OutputSnapshot> {
    if !path.try_exists()? {
        return Ok(OutputSnapshot::Missing);
    }
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("输出已存在，但不是普通目录".into());
    }
    let mut snapshot = OutputSnapshot::Directory {
        directories: BTreeSet::new(),
        files: BTreeMap::new(),
    };
    let OutputSnapshot::Directory { directories, files } = &mut snapshot else {
        unreachable!()
    };
    snapshot_dir(path, path, directories, files)?;
    Ok(snapshot)
}

fn snapshot_dir(
    root: &Path,
    directory: &Path,
    directories: &mut BTreeSet<PathBuf>,
    files: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> vn_cli::Result<()> {
    for item in fs::read_dir(directory)? {
        let item = item?;
        let path = item.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|_| "输出快照路径越界")?
            .to_path_buf();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(format!("输出目录含符号链接，拒绝覆盖: {}", relative.display()).into());
        }
        if metadata.is_dir() {
            directories.insert(relative.clone());
            snapshot_dir(root, &path, directories, files)?;
        } else if metadata.is_file() {
            files.insert(relative, fs::read(path)?);
        } else {
            return Err("输出目录含不支持的文件类型".into());
        }
    }
    Ok(())
}

static NEXT_STAGE: AtomicU64 = AtomicU64::new(0);

struct StageDirectory {
    path: PathBuf,
}

impl StageDirectory {
    fn create(parent: &Path, output_name: &std::ffi::OsStr) -> vn_cli::Result<Self> {
        for _ in 0..100 {
            let sequence = NEXT_STAGE.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!(
                ".{}.stage.{}.{}",
                output_name.to_string_lossy(),
                std::process::id(),
                sequence
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
        Err("无法分配解包暂存目录".into())
    }
}

impl Drop for StageDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn commit_stage(
    stage: &StageDirectory,
    output: &Path,
    expected: &OutputSnapshot,
    overwrite: bool,
) -> vn_cli::Result<Option<String>> {
    let current = snapshot_output(output)?;
    if &current != expected {
        return Err("输出目录在预检后发生变化，请重新预检".into());
    }
    match expected {
        OutputSnapshot::Missing => rename_no_replace(&stage.path, output)?,
        OutputSnapshot::Directory { .. } if overwrite => {
            let backup = unique_sibling(output, "backup")?;
            rename_no_replace(output, &backup)?;
            if let Err(error) = rename_no_replace(&stage.path, output) {
                return match rename_no_replace(&backup, output) {
                    Ok(()) => Err(format!("提交输出目录失败，旧输出已恢复: {error}").into()),
                    Err(restore_error) => Err(format!(
                        "提交失败: {error}; 旧输出恢复失败: {restore_error}; 备份保留在 {}",
                        backup.display()
                    )
                    .into()),
                };
            }
            if let Err(error) = fs::remove_dir_all(&backup) {
                return Ok(Some(format!(
                    "新输出已提交；旧输出备份未能清理（{error}）: {}",
                    backup.display()
                )));
            }
        }
        OutputSnapshot::Directory { .. } => return Err("输出目录已存在".into()),
    }
    Ok(None)
}

fn unique_sibling(path: &Path, kind: &str) -> vn_cli::Result<PathBuf> {
    let parent = path.parent().ok_or("输出路径缺少父目录")?;
    let name = path.file_name().ok_or("输出目录名为空")?;
    for _ in 0..100 {
        let sequence = NEXT_STAGE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".{}.{}.{}.{}",
            name.to_string_lossy(),
            kind,
            std::process::id(),
            sequence
        ));
        if !candidate.try_exists()? {
            return Ok(candidate);
        }
    }
    Err("无法分配临时备份路径".into())
}

#[cfg(windows)]
fn rename_no_replace(from: &Path, to: &Path) -> vn_cli::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    extern "system" {
        fn MoveFileExW(existing: *const u16, new: *const u16, flags: u32) -> i32;
    }
    let from: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    let success = unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), 0) };
    if success == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}

#[cfg(not(windows))]
fn rename_no_replace(from: &Path, to: &Path) -> vn_cli::Result<()> {
    if to.try_exists()? {
        return Err(format!("提交目标已存在: {}", to.display()).into());
    }
    fs::rename(from, to)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_matches_game_hash_for_known_filename() {
        assert_eq!(crc32_lower_shift_jis(b"sr006.mjo"), 0x01d7_ed10);
        assert_eq!(crc32_lower_shift_jis(b"SR006.MJO"), 0x01d7_ed10);
    }

    #[test]
    fn crc64_matches_v3_sample_and_runtime_case_folding() {
        assert_eq!(crc64_lower_shift_jis(b"sr011.mjo"), 0x8071_f5bf_2b15_4933);
        assert_eq!(
            crc64_lower_shift_jis(&[0x82, 0x60]),
            crc64_lower_shift_jis(&[0x82, 0x81])
        );
    }

    #[test]
    fn preserves_cp932_trail_bytes_during_case_fold() {
        let (name, _, had_errors) = encoding_rs::SHIFT_JIS.encode("◎スクリプト中sr047.mjo");
        assert!(!had_errors);
        assert_eq!(crc32_lower_shift_jis(&name), 0x0b12_127f);
    }

    #[test]
    fn folds_full_width_latin_capitals_like_the_runtime() {
        assert_eq!(
            crc32_lower_shift_jis(&[0x82, 0x60]),
            crc32_lower_shift_jis(&[0x82, 0x81])
        );
    }

    #[test]
    fn does_not_treat_cp932_trail_backslash_as_a_path_separator() {
        let (name, _, had_errors) = encoding_rs::SHIFT_JIS.encode("ソabc.mjo");
        assert!(!had_errors);
        assert_eq!(name.as_ref().get(0..2), Some(&[0x83, 0x5c][..]));
        assert_eq!(basename_bytes(&name), name.as_ref());
    }

    #[test]
    fn crc_matches_cp932_filename_with_japanese_prefix() {
        let (name, _, had_errors) = encoding_rs::SHIFT_JIS.encode("◎スクリプト中sr047.mjo");
        assert!(!had_errors);
        assert_eq!(crc32_lower_shift_jis(&name), 0x0b12_127f);
    }

    #[test]
    fn parses_sorted_index_and_uses_next_offset_as_member_end() {
        let version = ArchiveVersion::V1;
        let mut items = vec![
            (
                crc32_lower_shift_jis(b"alpha.mjo"),
                "alpha.mjo",
                b"raw-one".as_slice(),
            ),
            (
                crc32_lower_shift_jis(b"beta.mjo"),
                "beta.mjo",
                b"raw-two".as_slice(),
            ),
        ];
        items.sort_by_key(|item| item.0);
        let count = items.len();
        let names_offset = HEADER_SIZE + (count + 1) * version.index_entry_size();
        let names: Vec<u8> = items
            .iter()
            .flat_map(|item| item.1.as_bytes().iter().copied().chain(std::iter::once(0)))
            .collect();
        let data_offset = names_offset + names.len();
        let mut data = Vec::new();
        let mut offsets = Vec::new();
        for item in &items {
            offsets.push(data_offset + data.len());
            data.extend_from_slice(item.2);
        }
        let file_size = data_offset + data.len();
        let mut bytes = vec![0; HEADER_SIZE];
        bytes[..16].copy_from_slice(version.signature());
        bytes[0x10..0x14].copy_from_slice(&(count as u32).to_le_bytes());
        bytes[0x14..0x18].copy_from_slice(&(names_offset as u32).to_le_bytes());
        bytes[0x18..0x1c].copy_from_slice(&(data_offset as u32).to_le_bytes());
        for i in 0..count {
            bytes.extend_from_slice(&items[i].0.to_le_bytes());
            bytes.extend_from_slice(&(offsets[i] as u32).to_le_bytes());
        }
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&(file_size as u32).to_le_bytes());
        bytes.extend_from_slice(&names);
        bytes.extend_from_slice(&data);

        let archive = Archive::parse(&bytes).unwrap();
        assert_eq!(archive.entries.len(), 2);
        assert_eq!(archive.version, ArchiveVersion::V1);
        assert_eq!(
            &bytes[archive.entries[0].start..archive.entries[0].end],
            items[0].2
        );
        assert_eq!(
            &bytes[archive.entries[1].start..archive.entries[1].end],
            items[1].2
        );
    }

    #[test]
    fn rejects_bad_terminal_offset_and_unsafe_paths() {
        assert!(safe_relative_path("../escape.mjo").is_err());
        assert!(safe_relative_path("C:/escape.mjo").is_err());
        assert!(safe_relative_path("folder\\safe.mjo").is_ok());
    }
}
