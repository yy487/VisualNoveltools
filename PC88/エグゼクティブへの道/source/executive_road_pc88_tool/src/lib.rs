#![forbid(unsafe_code)]

use n88::{N88DiskSummary, ParsedN88Disk, ResourceFile, SectorSource};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use vn_cli::{
    Field, FieldKind, Operation, OperationSpec, Parameters, PreparedOperation, Preview, Progress,
    RunReport, Value,
};
use vn_d88::{Decoder, StandardCodec};

pub mod font;
mod n88;
pub mod text;
pub use text::{ExportTextOperation, ImportTextOperation};

const EXPECTED_DISKS: usize = 3;
const DISK_LABELS: [&str; EXPECTED_DISKS] = ["a", "b", "c"];
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

pub type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiskOutput {
    pub index: usize,
    pub file_name: String,
    pub source_offset: usize,
    pub length: usize,
    pub header_name_hex: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnpackPlan {
    pub source_length: usize,
    pub source_sha256: String,
    pub disks: Vec<DiskOutput>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BundleManifest {
    format: String,
    version: u32,
    source_length: usize,
    source_sha256: String,
    disks: Vec<DiskOutput>,
    warnings: Vec<String>,
}

/// Inspect a concatenated D88 container and validate the expected A/B/C layout.
/// The returned byte ranges come directly from the D88 declared disk records.
pub fn inspect_container(source: &[u8]) -> Result<UnpackPlan> {
    let image = StandardCodec
        .decode(source)
        .map_err(|error| format!("D88 解析失败：{error}"))?;
    if image.disks.len() != EXPECTED_DISKS {
        return Err(format!(
            "预期容器恰好包含 A/B/C 三张 D88 盘，实际解析到 {} 张",
            image.disks.len()
        ));
    }
    if image.trailing_range.is_some() {
        return Err("最后一张 D88 盘之后存在未解析尾部数据；拒绝丢弃或猜测其含义".into());
    }

    let mut cursor = 0usize;
    let mut disks = Vec::with_capacity(EXPECTED_DISKS);
    for (index, disk) in image.disks.iter().enumerate() {
        if disk.source_range.start != cursor || disk.source_range.end > source.len() {
            return Err(format!(
                "D88 盘 {} 的来源范围 0x{:X}..0x{:X} 与容器不连续或越界",
                index + 1,
                disk.source_range.start,
                disk.source_range.end
            ));
        }
        let bytes = &source[disk.source_range.clone()];
        disks.push(DiskOutput {
            index: index + 1,
            file_name: format!("disk_{}.d88", DISK_LABELS[index]),
            source_offset: disk.source_range.start,
            length: bytes.len(),
            header_name_hex: hex(&disk.header.raw_name),
            sha256: sha256_hex(bytes),
        });
        cursor = disk.source_range.end;
    }
    if cursor != source.len() {
        return Err(format!(
            "三张 D88 盘覆盖到 0x{cursor:X}，但容器长度为 0x{:X}；拒绝遗漏数据",
            source.len()
        ));
    }

    let warnings = image
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            format!(
                "盘 {:?} 轨道 {:?} 范围 0x{:X}..0x{:X}: {}",
                diagnostic.disk_index,
                diagnostic.track_slot,
                diagnostic.source_range.start,
                diagnostic.source_range.end,
                diagnostic.message
            )
        })
        .collect();
    Ok(UnpackPlan {
        source_length: source.len(),
        source_sha256: sha256_hex(source),
        disks,
        warnings,
    })
}

/// Write three independently validated disk images as one directory transaction.
/// Existing output is rejected unless `overwrite` is true. The source slice is never changed.
pub fn unpack_bytes(source: &[u8], output: &Path, overwrite: bool) -> Result<UnpackPlan> {
    let mut plan = inspect_container(source)?;
    validate_output(output, overwrite)?;

    let parent = output_parent(output);
    let stage = create_unique_dir(parent, output, "stage")?;
    let write_result = populate_stage(source, &plan, &stage);
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }

    match commit_stage(&stage, output, overwrite) {
        Ok(Some(warning)) => plan.warnings.push(warning),
        Ok(None) => {}
        Err(error) => {
            let _ = fs::remove_dir_all(&stage);
            return Err(error);
        }
    }
    Ok(plan)
}

fn populate_stage(source: &[u8], plan: &UnpackPlan, stage: &Path) -> Result<()> {
    for disk in &plan.disks {
        let end = disk
            .source_offset
            .checked_add(disk.length)
            .ok_or_else(|| "D88 盘输出范围溢出".to_owned())?;
        let bytes = source
            .get(disk.source_offset..end)
            .ok_or_else(|| format!("{} 来源范围超出输入", disk.file_name))?;
        if sha256_hex(bytes) != disk.sha256 {
            return Err(format!("{} 写出前内容校验失败", disk.file_name));
        }
        write_new_file(&stage.join(&disk.file_name), bytes)?;
    }

    let manifest = BundleManifest {
        format: "executive-road-pc88-d88-split".into(),
        version: 1,
        source_length: plan.source_length,
        source_sha256: plan.source_sha256.clone(),
        disks: plan.disks.clone(),
        warnings: plan.warnings.clone(),
    };
    let json =
        serde_json::to_vec_pretty(&manifest).map_err(|error| format!("清单序列化失败：{error}"))?;
    write_new_file(&stage.join("manifest.json"), &json)?;
    Ok(())
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("创建 {} 失败：{error}", path.display()))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| format!("写入 {} 失败：{error}", path.display()))
}

fn validate_output(output: &Path, overwrite: bool) -> Result<()> {
    if output.file_name().is_none() {
        return Err("输出目录必须包含一个目录名".into());
    }
    let parent = output_parent(output);
    if !parent.is_dir() {
        return Err(format!("输出父目录不存在：{}", parent.display()));
    }
    match fs::symlink_metadata(output) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err("拒绝将输出目录设为符号链接".into());
            }
            if !metadata.is_dir() {
                return Err(format!("输出路径不是目录：{}", output.display()));
            }
            if !overwrite {
                return Err(format!("输出目录已存在；默认不覆盖：{}", output.display()));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("检查输出目录失败：{error}")),
    }
    Ok(())
}

fn output_parent(output: &Path) -> &Path {
    match output.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

fn create_unique_dir(parent: &Path, output: &Path, suffix: &str) -> Result<PathBuf> {
    let base = output
        .file_name()
        .ok_or_else(|| "输出目录必须包含一个目录名".to_owned())?
        .to_string_lossy();
    for _ in 0..100 {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(".{base}.{suffix}.{}.{}", std::process::id(), id));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "创建暂存目录 {} 失败：{error}",
                    candidate.display()
                ))
            }
        }
    }
    Err("无法分配唯一暂存目录".into())
}

fn unused_sibling(parent: &Path, output: &Path, suffix: &str) -> Result<PathBuf> {
    let base = output
        .file_name()
        .ok_or_else(|| "输出目录必须包含一个目录名".to_owned())?
        .to_string_lossy();
    for _ in 0..100 {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(".{base}.{suffix}.{}.{}", std::process::id(), id));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("无法分配唯一备份路径".into())
}

fn commit_stage(stage: &Path, output: &Path, overwrite: bool) -> Result<Option<String>> {
    let parent = output_parent(output);
    let existing = match fs::symlink_metadata(output) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err("提交前输出路径类型发生变化".into());
            }
            true
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(format!("提交前检查输出失败：{error}")),
    };
    if existing && !overwrite {
        return Err(format!(
            "执行期间输出目录出现，拒绝覆盖：{}",
            output.display()
        ));
    }
    if !existing {
        return fs::rename(stage, output)
            .map(|()| None)
            .map_err(|error| format!("提交输出目录失败：{error}"));
    }

    let backup = unused_sibling(parent, output, "backup")?;
    fs::rename(output, &backup).map_err(|error| format!("暂存旧输出目录失败：{error}"))?;
    if let Err(error) = fs::rename(stage, output) {
        let rollback = fs::rename(&backup, output);
        return Err(match rollback {
            Ok(()) => format!("提交新输出失败，已恢复旧目录：{error}"),
            Err(rollback_error) => format!(
                "提交新输出失败：{error}；旧目录仍在 {}，恢复也失败：{rollback_error}",
                backup.display()
            ),
        });
    }
    match fs::remove_dir_all(&backup) {
        Ok(()) => Ok(None),
        Err(error) => Ok(Some(format!(
            "输出已提交，但旧目录 {} 未能清理：{error}",
            backup.display()
        ))),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex(&digest)
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[usize::from(byte >> 4)] as char);
        output.push(DIGITS[usize::from(byte & 0x0f)] as char);
    }
    output
}

#[derive(Debug, Default)]
pub struct UnpackOperation;

impl Operation for UnpackOperation {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "unpack",
            "解包 D88 外层软碟",
            vec![
                Field::new("source", "原始 D88 文件", FieldKind::Path).required(),
                Field::new("output", "A/B/C 输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description = "校验并拆出三张独立 D88 盘，保留原始盘字节和来源清单。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(path) = paths.first() {
            if parameters.get("source").is_none() {
                parameters.set("source", Value::Path(path.clone()));
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let source_path = parameters.path("source")?.to_path_buf();
        let output = parameters.path("output")?.to_path_buf();
        let source = fs::read(&source_path)
            .map_err(|error| vn_cli::Error(format!("读取原始 D88 失败：{error}")))?;
        let plan = inspect_container(&source).map_err(vn_cli::Error::from)?;
        validate_output(&output, parameters.flag("overwrite")).map_err(vn_cli::Error::from)?;
        let preview = Preview {
            inputs: vec![source_path.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "重新校验来源文件未变化".into(),
                "暂存 disk_a.d88 / disk_b.d88 / disk_c.d88 与 manifest.json".into(),
                "原子提交完整输出目录".into(),
            ],
            details: vec![
                format!(
                    "源文件：{} 字节，SHA-256 {}",
                    plan.source_length, plan.source_sha256
                ),
                format!(
                    "盘大小：{} / {} / {} 字节",
                    plan.disks[0].length, plan.disks[1].length, plan.disks[2].length
                ),
                format!("轨道诊断：{} 条", plan.warnings.len()),
            ],
        };
        Ok(Box::new(PreparedUnpack {
            source_path,
            source_snapshot: source,
            output,
            overwrite: parameters.flag("overwrite"),
            preview,
        }))
    }
}

struct PreparedUnpack {
    source_path: PathBuf,
    source_snapshot: Vec<u8>,
    output: PathBuf,
    overwrite: bool,
    preview: Preview,
}

impl PreparedOperation for PreparedUnpack {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        progress.report("复核原始 D88 文件")?;
        let current = fs::read(&self.source_path)
            .map_err(|error| vn_cli::Error(format!("重新读取原始 D88 失败：{error}")))?;
        if current != self.source_snapshot {
            return Err("原始 D88 在预检后发生变化；未写出任何结果".into());
        }
        progress.report("验证 A/B/C 三张盘并写入暂存目录")?;
        let result = unpack_bytes(&self.source_snapshot, &self.output, self.overwrite)
            .map_err(vn_cli::Error::from)?;
        Ok(RunReport {
            summary: "D88 外层解包完成".into(),
            totals: vec![
                ("D88 磁盘".into(), EXPECTED_DISKS as u64),
                (
                    "原盘字节".into(),
                    self.source_snapshot.len().try_into().unwrap_or(u64::MAX),
                ),
            ],
            outputs: vec![self.output],
            warnings: result.warnings,
        })
    }
}

#[derive(Debug)]
struct ResourceInputDisk {
    index: usize,
    label: &'static str,
    file_name: String,
    source_snapshot: Vec<u8>,
    parsed: ParsedN88Disk,
}

#[derive(Debug)]
struct PreparedResources {
    workspace: PathBuf,
    manifest_snapshot: Vec<u8>,
    source_manifest: BundleManifest,
    disks: Vec<ResourceInputDisk>,
}

#[derive(Debug, Clone)]
pub struct ResourceExtractionReport {
    pub disks: usize,
    pub files: usize,
    pub bytes: u64,
    pub output: PathBuf,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
struct ResourceExtractionManifest {
    format: &'static str,
    version: u32,
    source_length: usize,
    source_sha256: String,
    input_manifest_sha256: String,
    disks: Vec<ResourceDiskManifest>,
    files: Vec<ResourceFileManifest>,
    warnings: Vec<String>,
}

#[derive(Serialize)]
struct ResourceDiskManifest {
    index: usize,
    label: String,
    source_file_name: String,
    source_sha256: String,
    summary: N88DiskSummary,
    file_count: usize,
    warnings: Vec<String>,
}

#[derive(Serialize)]
struct ResourceFileManifest {
    disk_index: usize,
    disk_label: String,
    source_file_name: String,
    output_path: String,
    directory_index: usize,
    directory_entry_source_offset: usize,
    raw_entry_hex: String,
    raw_name_hex: String,
    display_name: String,
    attribute: u8,
    start_cluster: u8,
    clusters: Vec<u8>,
    final_cluster_sectors: u8,
    length: usize,
    sha256: String,
    source_sectors: Vec<SectorSource>,
}

/// Extract all active directory files from the three images produced by `unpack`.
/// The input directory must contain its original `manifest.json` and all three
/// hash-matching D88 files. Existing output is refused unless `overwrite` is true.
pub fn extract_workspace(
    workspace: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<ResourceExtractionReport> {
    validate_output(output, overwrite)?;
    ensure_paths_disjoint(workspace, output)?;
    let prepared = prepare_resource_workspace(workspace)?;
    verify_resource_inputs(&prepared)?;
    commit_resource_outputs(&prepared, output, overwrite)
}

fn prepare_resource_workspace(workspace: &Path) -> Result<PreparedResources> {
    if !workspace.is_dir() {
        return Err(format!("D88 工作目录不存在：{}", workspace.display()));
    }
    let manifest_path = workspace.join("manifest.json");
    let manifest_snapshot =
        fs::read(&manifest_path).map_err(|error| format!("读取外层拆盘清单失败：{error}"))?;
    let source_manifest: BundleManifest = serde_json::from_slice(&manifest_snapshot)
        .map_err(|error| format!("外层拆盘清单 JSON 无效：{error}"))?;
    if source_manifest.format != "executive-road-pc88-d88-split"
        || source_manifest.version != 1
        || source_manifest.disks.len() != EXPECTED_DISKS
    {
        return Err("工作目录不是本工具生成的三盘 D88 解包结果".into());
    }

    let mut cursor = 0usize;
    let mut reconstructed_source = Vec::with_capacity(source_manifest.source_length);
    let mut disks = Vec::with_capacity(EXPECTED_DISKS);
    for (index, label) in DISK_LABELS.iter().enumerate() {
        let expected_name = format!("disk_{label}.d88");
        let record = &source_manifest.disks[index];
        if record.index != index + 1
            || record.file_name != expected_name
            || record.source_offset != cursor
        {
            return Err(format!("外层拆盘清单的第 {} 张盘映射无效", index + 1));
        }
        let path = workspace.join(&expected_name);
        let bytes =
            fs::read(&path).map_err(|error| format!("读取 {} 失败：{error}", path.display()))?;
        if bytes.len() != record.length || sha256_hex(&bytes) != record.sha256 {
            return Err(format!("{} 与外层拆盘清单的长度或哈希不符", expected_name));
        }
        let parsed = n88::parse_disk(&bytes)
            .map_err(|error| format!("解析 {} 失败：{error}", expected_name))?;
        if parsed.summary.d88_name_hex != record.header_name_hex {
            return Err(format!("{} 的 D88 盘名与拆盘清单不符", expected_name));
        }
        cursor = cursor
            .checked_add(bytes.len())
            .ok_or_else(|| "外层三盘长度溢出".to_owned())?;
        reconstructed_source.extend_from_slice(&bytes);
        disks.push(ResourceInputDisk {
            index: index + 1,
            label,
            file_name: expected_name,
            source_snapshot: bytes,
            parsed,
        });
    }
    if cursor != source_manifest.source_length
        || sha256_hex(&reconstructed_source) != source_manifest.source_sha256
    {
        return Err("三张输入盘不能按清单精确重建原始 D88 容器".into());
    }
    Ok(PreparedResources {
        workspace: workspace.to_path_buf(),
        manifest_snapshot,
        source_manifest,
        disks,
    })
}

fn verify_resource_inputs(prepared: &PreparedResources) -> Result<()> {
    let manifest_path = prepared.workspace.join("manifest.json");
    if fs::read(&manifest_path).map_err(|error| format!("重新读取拆盘清单失败：{error}"))?
        != prepared.manifest_snapshot
    {
        return Err("外层拆盘清单在预检后发生变化；未写出资源".into());
    }
    for disk in &prepared.disks {
        let path = prepared.workspace.join(&disk.file_name);
        if fs::read(&path).map_err(|error| format!("重新读取 {} 失败：{error}", disk.file_name))?
            != disk.source_snapshot
        {
            return Err(format!("{} 在预检后发生变化；未写出资源", disk.file_name));
        }
    }
    Ok(())
}

fn resource_manifest(prepared: &PreparedResources) -> ResourceExtractionManifest {
    let mut disks = Vec::with_capacity(prepared.disks.len());
    let mut files = Vec::new();
    let mut warnings = Vec::new();
    for disk in &prepared.disks {
        let record = &prepared.source_manifest.disks[disk.index - 1];
        for warning in &disk.parsed.warnings {
            warnings.push(format!("disk_{}: {warning}", disk.label));
        }
        disks.push(ResourceDiskManifest {
            index: disk.index,
            label: disk.label.into(),
            source_file_name: disk.file_name.clone(),
            source_sha256: record.sha256.clone(),
            summary: disk.parsed.summary.clone(),
            file_count: disk.parsed.files.len(),
            warnings: disk.parsed.warnings.clone(),
        });
        for file in &disk.parsed.files {
            files.push(resource_file_manifest(disk, file));
        }
    }
    ResourceExtractionManifest {
        format: "executive-road-pc88-resource-extract",
        version: 1,
        source_length: prepared.source_manifest.source_length,
        source_sha256: prepared.source_manifest.source_sha256.clone(),
        input_manifest_sha256: sha256_hex(&prepared.manifest_snapshot),
        disks,
        files,
        warnings,
    }
}

fn resource_file_manifest(disk: &ResourceInputDisk, file: &ResourceFile) -> ResourceFileManifest {
    ResourceFileManifest {
        disk_index: disk.index,
        disk_label: disk.label.into(),
        source_file_name: disk.file_name.clone(),
        output_path: format!("disk_{}/{}", disk.label, file.output_name),
        directory_index: file.directory_index,
        directory_entry_source_offset: file.directory_entry_source_offset,
        raw_entry_hex: hex(&file.raw_entry),
        raw_name_hex: hex(&file.raw_name),
        display_name: file.display_name.clone(),
        attribute: file.attribute,
        start_cluster: file.start_cluster,
        clusters: file.clusters.clone(),
        final_cluster_sectors: file.final_sectors,
        length: file.data.len(),
        sha256: file.sha256.clone(),
        source_sectors: file.source_sectors.clone(),
    }
}

fn commit_resource_outputs(
    prepared: &PreparedResources,
    output: &Path,
    overwrite: bool,
) -> Result<ResourceExtractionReport> {
    let manifest = resource_manifest(prepared);
    let stage = create_unique_dir(output_parent(output), output, "stage")?;
    let write_result = (|| {
        for disk in &prepared.disks {
            let disk_dir = stage.join(format!("disk_{}", disk.label));
            fs::create_dir(&disk_dir)
                .map_err(|error| format!("创建 {} 失败：{error}", disk_dir.display()))?;
            for file in &disk.parsed.files {
                write_new_file(&disk_dir.join(&file.output_name), &file.data)?;
            }
        }
        let json = serde_json::to_vec_pretty(&manifest)
            .map_err(|error| format!("资源清单序列化失败：{error}"))?;
        write_new_file(&stage.join("resource_manifest.json"), &json)
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }

    let mut warnings = manifest.warnings.clone();
    match commit_stage(&stage, output, overwrite) {
        Ok(Some(warning)) => warnings.push(warning),
        Ok(None) => {}
        Err(error) => {
            let _ = fs::remove_dir_all(&stage);
            return Err(error);
        }
    }
    let files = manifest.files.len();
    let bytes = manifest.files.iter().map(|file| file.length as u64).sum();
    Ok(ResourceExtractionReport {
        disks: prepared.disks.len(),
        files,
        bytes,
        output: output.to_path_buf(),
        warnings,
    })
}

fn ensure_paths_disjoint(input: &Path, output: &Path) -> Result<()> {
    let input =
        fs::canonicalize(input).map_err(|error| format!("解析输入工作目录失败：{error}"))?;
    let parent = output_parent(output);
    let parent =
        fs::canonicalize(parent).map_err(|error| format!("解析输出父目录失败：{error}"))?;
    let output = match fs::canonicalize(output) {
        Ok(path) => path,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => parent.join(
            output
                .file_name()
                .ok_or_else(|| "输出目录必须包含一个目录名".to_owned())?,
        ),
        Err(error) => return Err(format!("解析输出目录失败：{error}")),
    };
    if output == input || output.starts_with(&input) || input.starts_with(&output) {
        return Err("输入工作目录与输出目录不能重叠".into());
    }
    Ok(())
}

#[derive(Debug, Default)]
pub struct ExtractOperation;

impl Operation for ExtractOperation {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "extract",
            "提取 A/B/C 盘内资源",
            vec![
                Field::new("workspace", "已拆分的 D88 工作目录", FieldKind::Path).required(),
                Field::new("output", "资源输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description = "按目录项和分配链提取每张盘的活动文件，并记录扇区来源与哈希。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(path) = paths.first() {
            if parameters.get("workspace").is_none() {
                parameters.set("workspace", Value::Path(path.clone()));
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let workspace = parameters.path("workspace")?.to_path_buf();
        let output = parameters.path("output")?.to_path_buf();
        validate_output(&output, parameters.flag("overwrite")).map_err(vn_cli::Error::from)?;
        ensure_paths_disjoint(&workspace, &output).map_err(vn_cli::Error::from)?;
        let prepared = prepare_resource_workspace(&workspace).map_err(vn_cli::Error::from)?;
        let plan = resource_manifest(&prepared);
        let total_bytes = plan.files.iter().map(|file| file.length).sum::<usize>();
        let preview = Preview {
            inputs: vec![workspace.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "重验三张 D88、外层清单和原始容器哈希".into(),
                "校验目录终止符、160 簇分配表、所有链尾和交叉引用".into(),
                format!("暂存 {} 个目录文件与来源清单，再原子提交", plan.files.len()),
            ],
            details: vec![
                format!("活动文件：{} 个", plan.files.len()),
                format!("资源数据：{total_bytes} 字节"),
                format!(
                    "各盘文件数：{} / {} / {}",
                    prepared.disks[0].parsed.files.len(),
                    prepared.disks[1].parsed.files.len(),
                    prepared.disks[2].parsed.files.len()
                ),
                format!("目录/分配表诊断：{} 条", plan.warnings.len()),
            ],
        };
        Ok(Box::new(PreparedExtract {
            prepared,
            output,
            overwrite: parameters.flag("overwrite"),
            preview,
        }))
    }
}

struct PreparedExtract {
    prepared: PreparedResources,
    output: PathBuf,
    overwrite: bool,
    preview: Preview,
}

impl PreparedOperation for PreparedExtract {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        progress.report("复核拆盘清单与 A/B/C 原盘快照")?;
        verify_resource_inputs(&self.prepared).map_err(vn_cli::Error::from)?;
        progress.report("提取目录文件并安全提交资源目录")?;
        let report = commit_resource_outputs(&self.prepared, &self.output, self.overwrite)
            .map_err(vn_cli::Error::from)?;
        Ok(RunReport {
            summary: "三张 PC-88 软碟的目录资源提取完成".into(),
            totals: vec![
                ("D88 磁盘".into(), report.disks as u64),
                ("目录文件".into(), report.files as u64),
                ("资源字节".into(), report.bytes),
            ],
            outputs: vec![report.output],
            warnings: report.warnings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn d88_disk(name: &[u8], fill: u8) -> Vec<u8> {
        let mut disk = vec![fill; 0x2A0];
        disk[..17].fill(0);
        let name_len = name.len().min(17);
        disk[..name_len].copy_from_slice(&name[..name_len]);
        disk[0x11..0x1A].fill(0);
        disk[0x1A..0x20].fill(0);
        let disk_len = disk.len() as u32;
        disk[0x1C..0x20].copy_from_slice(&disk_len.to_le_bytes());
        disk[0x20..0x2A0].fill(0);
        disk
    }

    fn fixture() -> Vec<u8> {
        [
            d88_disk(b"DISK A", 0xA1),
            d88_disk(b"DISK B", 0xB2),
            d88_disk(b"DISK C", 0xC3),
        ]
        .concat()
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "executive-road-pc88-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn resource_disk(label: u8) -> Vec<u8> {
        let header_len = 0x2A0usize;
        let track_len = 0x1100usize;
        let mut bytes = vec![0u8; header_len + 80 * track_len];
        bytes[..6].copy_from_slice(&[b'd', b'i', b's', b'k', b' ', label]);
        for slot in 0..80usize {
            let track_start = header_len + slot * track_len;
            bytes[0x20 + slot * 4..0x24 + slot * 4]
                .copy_from_slice(&(track_start as u32).to_le_bytes());
            for record in 1..=16u8 {
                let sector_start = track_start + usize::from(record - 1) * 0x110;
                bytes[sector_start] = (slot / 2) as u8;
                bytes[sector_start + 1] = (slot % 2) as u8;
                bytes[sector_start + 2] = record;
                bytes[sector_start + 3] = 1;
                bytes[sector_start + 4..sector_start + 6].copy_from_slice(&16u16.to_le_bytes());
                bytes[sector_start + 14..sector_start + 16].copy_from_slice(&256u16.to_le_bytes());
                let data_start = sector_start + 16;
                bytes[data_start..data_start + 256].fill(0xFF);
            }
        }
        let system_start = header_len + 37 * track_len;
        let directory_start = system_start + 16;
        let mut entry = [0u8; 16];
        entry[..9].copy_from_slice(b"FILE  TXT");
        entry[9] = 0x01;
        entry[10] = 10;
        bytes[directory_start..directory_start + 16].copy_from_slice(&entry);
        bytes[directory_start + 16] = 0xFF;

        let allocation_start = system_start + 7 * 0x110 + 16;
        bytes[allocation_start..allocation_start + 160].fill(0xFF);
        bytes[allocation_start + 10] = 0xC2;

        let data_track = header_len + 5 * track_len;
        let file_data_start = data_track + 16;
        bytes[file_data_start..file_data_start + 256].fill(label);
        bytes[file_data_start + 0x110..file_data_start + 0x110 + 256].fill(label);
        let disk_len = bytes.len() as u32;
        bytes[0x1C..0x20].copy_from_slice(&disk_len.to_le_bytes());
        bytes
    }

    fn create_resource_workspace(root: &Path) -> (PathBuf, [Vec<u8>; 3]) {
        let workspace = root.join("d88_unpacked");
        fs::create_dir(&workspace).expect("create resource workspace");
        let disks = [
            resource_disk(b'a'),
            resource_disk(b'b'),
            resource_disk(b'c'),
        ];
        let source = [
            disks[0].as_slice(),
            disks[1].as_slice(),
            disks[2].as_slice(),
        ]
        .concat();
        let plan = inspect_container(&source).expect("inspect synthetic outer D88");
        for disk in &plan.disks {
            let range = disk.source_offset..disk.source_offset + disk.length;
            fs::write(workspace.join(&disk.file_name), &source[range]).expect("write D88 disk");
        }
        let manifest = BundleManifest {
            format: "executive-road-pc88-d88-split".into(),
            version: 1,
            source_length: plan.source_length,
            source_sha256: plan.source_sha256,
            disks: plan.disks,
            warnings: plan.warnings,
        };
        fs::write(
            workspace.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).expect("serialize outer manifest"),
        )
        .expect("write outer manifest");
        (workspace, disks)
    }

    #[test]
    fn splits_three_concatenated_disks_and_records_exact_ranges() {
        let source = fixture();
        let plan = inspect_container(&source).expect("valid triple D88");
        assert_eq!(plan.source_length, 3 * 0x2A0);
        assert_eq!(plan.disks.len(), 3);
        assert_eq!(
            plan.disks
                .iter()
                .map(|disk| disk.file_name.as_str())
                .collect::<Vec<_>>(),
            ["disk_a.d88", "disk_b.d88", "disk_c.d88"]
        );
        assert_eq!(plan.disks[0].source_offset, 0);
        assert_eq!(plan.disks[1].source_offset, 0x2A0);
        assert_eq!(plan.disks[2].source_offset, 0x540);
        assert!(plan.disks.iter().all(|disk| disk.length == 0x2A0));
    }

    #[test]
    fn rejects_wrong_disk_count_and_unparsed_tail() {
        let two_disks = [d88_disk(b"DISK A", 0xA1), d88_disk(b"DISK B", 0xB2)].concat();
        let four_disks = [
            d88_disk(b"DISK A", 0xA1),
            d88_disk(b"DISK B", 0xB2),
            d88_disk(b"DISK C", 0xC3),
            d88_disk(b"EXTRA", 0xD4),
        ]
        .concat();
        assert!(inspect_container(&two_disks)
            .unwrap_err()
            .contains("实际解析到 2 张"));
        assert!(inspect_container(&four_disks)
            .unwrap_err()
            .contains("实际解析到 4 张"));

        let mut trailing = fixture();
        trailing.extend_from_slice(b"TAIL");
        assert!(inspect_container(&trailing)
            .unwrap_err()
            .contains("尾部数据"));
    }

    #[test]
    fn rejects_truncated_or_non_d88_inputs() {
        assert!(inspect_container(&[]).is_err());
        let mut invalid = fixture();
        invalid[0x1C..0x20].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(inspect_container(&invalid).is_err());
    }

    #[test]
    fn writes_lossless_disk_files_and_manifest_without_changing_source() {
        let source = fixture();
        let original = source.clone();
        let root = temp_dir("unpack");
        let output = root.join("disks");
        let plan = unpack_bytes(&source, &output, false).expect("unpack");

        for disk in &plan.disks {
            let actual = fs::read(output.join(&disk.file_name)).expect("read disk output");
            let range = disk.source_offset..disk.source_offset + disk.length;
            assert_eq!(actual, source[range]);
            assert_eq!(sha256_hex(&actual), disk.sha256);
            assert_eq!(
                StandardCodec
                    .decode(&actual)
                    .expect("decode output")
                    .disks
                    .len(),
                1
            );
        }
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("manifest.json")).expect("read manifest"))
                .expect("valid manifest JSON");
        assert_eq!(manifest["disks"].as_array().unwrap().len(), 3);
        assert_eq!(source, original);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn refuses_overwrite_by_default_and_replaces_output_as_a_bundle_when_requested() {
        let root = temp_dir("overwrite");
        let output = root.join("disks");
        fs::create_dir(&output).expect("create existing output");
        fs::write(output.join("old.txt"), b"old").expect("write old file");
        let source = fixture();
        assert!(unpack_bytes(&source, &output, false)
            .unwrap_err()
            .contains("默认不覆盖"));
        assert_eq!(fs::read(output.join("old.txt")).unwrap(), b"old");

        unpack_bytes(&source, &output, true).expect("overwrite bundle");
        assert!(!output.join("old.txt").exists());
        assert!(output.join("disk_a.d88").is_file());
        fs::remove_dir_all(root).expect("cleanup");
    }

    struct NoopProgress;

    impl Progress for NoopProgress {
        fn report(&mut self, _message: &str) -> vn_cli::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn prepared_operation_rejects_a_changed_source_before_creating_outputs() {
        let root = temp_dir("changed-source");
        let source_path = root.join("source.d88");
        let output = root.join("unpacked");
        let source = fixture();
        fs::write(&source_path, &source).expect("write source");

        let mut parameters = Parameters::default();
        parameters.set("source", Value::Path(source_path.clone()));
        parameters.set("output", Value::Path(output.clone()));
        let prepared = UnpackOperation
            .prepare(&parameters)
            .expect("prepare operation");

        let mut changed = source;
        changed[0] ^= 0xFF;
        fs::write(&source_path, changed).expect("change source after preflight");
        let error = prepared.execute(&mut NoopProgress).unwrap_err().to_string();
        assert!(error.contains("预检后发生变化"));
        assert!(!output.exists());
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn extracts_all_active_files_and_records_byte_provenance() {
        let root = temp_dir("extract");
        let (workspace, disks) = create_resource_workspace(&root);
        let output = root.join("resources");
        let report = extract_workspace(&workspace, &output, false).expect("extract resources");
        assert_eq!(report.disks, 3);
        assert_eq!(report.files, 3);
        assert_eq!(report.bytes, 3 * 512);

        for (index, label) in ["a", "b", "c"].into_iter().enumerate() {
            let extracted = fs::read(output.join(format!("disk_{label}/FILE.TXT")))
                .expect("read extracted file");
            assert_eq!(extracted, vec![b'a' + index as u8; 512]);
            assert_eq!(
                fs::read(workspace.join(format!("disk_{label}.d88"))).unwrap(),
                disks[index]
            );
        }
        let manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(output.join("resource_manifest.json")).expect("read resource manifest"),
        )
        .expect("valid resource manifest");
        assert_eq!(manifest["files"].as_array().unwrap().len(), 3);
        assert_eq!(manifest["disks"][0]["summary"]["directory_entries"], 1);
        assert_eq!(
            manifest["files"][0]["directory_entry_source_offset"],
            0x277B0
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn prepared_resource_extract_rejects_changed_disk_and_overlapping_output() {
        let root = temp_dir("extract-protection");
        let (workspace, _) = create_resource_workspace(&root);
        let output = root.join("resources");
        let mut parameters = Parameters::default();
        parameters.set("workspace", Value::Path(workspace.clone()));
        parameters.set("output", Value::Path(output.clone()));
        let prepared = ExtractOperation
            .prepare(&parameters)
            .expect("prepare extract operation");

        let disk_path = workspace.join("disk_b.d88");
        let mut changed = fs::read(&disk_path).expect("read disk B");
        changed[0] ^= 0xFF;
        fs::write(&disk_path, changed).expect("change disk after preflight");
        let error = prepared.execute(&mut NoopProgress).unwrap_err().to_string();
        assert!(error.contains("在预检后发生变化"));
        assert!(!output.exists());
        assert!(
            extract_workspace(&workspace, &workspace.join("nested"), false)
                .unwrap_err()
                .contains("不能重叠")
        );
        fs::remove_dir_all(root).expect("cleanup");
    }
}
