use repi_unpack_v2::{safe_components, Archive, Entry};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
mod script;
mod text_workflow;
use vn_cli::*;

use text_workflow::{ExportText, ImportText};

struct Unpack;

impl Operation for Unpack {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "unpack",
            "解包 RepiPack v2",
            vec![
                Field::new("archive", "RepiPack v2 输入文件", FieldKind::Path).required(),
                Field::new("output", "独立输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description = "读取索引内的资源路径，解码载荷并保留原相对目录结构。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            parameters.set("archive", Value::Path(path.clone()));
            if parameters.get("output").is_none() {
                parameters.set(
                    "output",
                    Value::Path(
                        path.parent()
                            .unwrap_or_else(|| Path::new("."))
                            .join("repi_v2_unpacked"),
                    ),
                );
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let requested = parameters.path("archive")?;
        let archive_path = fs::canonicalize(requested)
            .map_err(|error| format!("无法读取输入 {}: {error}", requested.display()))?;
        if !archive_path.is_file() {
            return Err(format!("输入不是文件: {}", archive_path.display()).into());
        }
        let bytes = fs::read(&archive_path)
            .map_err(|error| format!("读取输入 {} 失败: {error}", archive_path.display()))?;
        let archive = Archive::parse(&bytes).map_err(|error| format!("解析失败: {error}"))?;
        let output = output_directory(parameters.path("output")?)?;
        let name_status = if repi_unpack_v2::source_file_name_matches(&archive, &archive_path) {
            "输入文件名与包头名称一致".to_owned()
        } else {
            format!(
                "包头名称为 {}，与输入文件名不同；仍按包内索引解码",
                archive.header_name
            )
        };
        let preview = Preview {
            inputs: vec![archive_path.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                format!("解析 RepiPack v2 表项（{} 条）", archive.entries.len()),
                "解码路径与载荷，写入独立暂存目录".into(),
                "生成 manifest.tsv 并提交输出目录".into(),
            ],
            details: vec![name_status],
        };
        Ok(Box::new(UnpackJob {
            preview,
            archive_path,
            bytes,
            archive,
            output,
            overwrite: parameters.flag("overwrite"),
        }))
    }
}

struct Repack;

impl Operation for Repack {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "repack",
            "重封 RepiPack v2",
            vec![
                Field::new("archive", "原始 RepiPack v2 文件", FieldKind::Path).required(),
                Field::new("input-dir", "已注入译文的完整脚本目录", FieldKind::Path).required(),
                Field::new(
                    "output",
                    "重封后的 DAT 文件（文件名须匹配包头）",
                    FieldKind::Path,
                )
                .required(),
            ],
        );
        spec.description =
            "未修改条目保留原 packed 字节；修改脚本尝试 LZSS 压缩，压缩无收益时使用 raw 载荷。"
                .into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            parameters.set("archive", Value::Path(path.clone()));
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let requested_archive = parameters.path("archive")?;
        let archive_path = fs::canonicalize(requested_archive).map_err(|error| {
            format!("无法读取原始 DAT {}: {error}", requested_archive.display())
        })?;
        let original = fs::read(&archive_path)
            .map_err(|error| format!("读取原始 DAT {} 失败: {error}", archive_path.display()))?;
        let archive =
            Archive::parse(&original).map_err(|error| format!("解析原始 DAT 失败: {error}"))?;
        let input_dir = fs::canonicalize(parameters.path("input-dir")?)
            .map_err(|error| format!("无法读取重建脚本目录: {error}"))?;
        if !input_dir.is_dir() {
            return Err(format!("重建脚本来源不是目录: {}", input_dir.display()).into());
        }
        let requested_output = parameters.path("output")?;
        let output = output_file_path(requested_output)?;
        let output_name = output
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        if output_name != archive.header_name {
            return Err(format!(
                "输出文件名须为包头名称 {}，当前为 {}",
                archive.header_name, output_name
            )
            .into());
        }
        if archive_path == output {
            return Err("重封输出必须与原始 DAT 分开，不能覆盖输入".into());
        }
        let (bytes, stats) = archive
            .repack_from_directory(&original, &input_dir)
            .map_err(|error| format!("重封预检失败: {error}"))?;
        let rebuilt =
            Archive::parse(&bytes).map_err(|error| format!("重封结果结构校验失败: {error}"))?;
        if rebuilt.entries.len() != stats.entries || rebuilt.header_name != archive.header_name {
            return Err("重封结果的包头或索引数量与原 DAT 不一致".into());
        }
        let preview = Preview {
            inputs: vec![archive_path.clone(), input_dir.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                format!("核对 {} 条索引路径与重建文件", stats.entries),
                "原样保留未改资源，对修改资源写入 raw 载荷并重算偏移".into(),
                "写出临时 DAT 并提交独立输出".into(),
            ],
            details: vec![format!(
                "变更条目={}（LZSS={}，raw={}），保留原 packed 条目={}，包头名称={}",
                stats.changed,
                stats.compressed_changed,
                stats.raw_changed,
                stats.preserved_packed,
                archive.header_name
            )],
        };
        Ok(Box::new(RepackJob {
            preview,
            output,
            bytes,
            stats,
            overwrite: parameters.flag("overwrite"),
        }))
    }
}

struct RepackJob {
    preview: Preview,
    output: PathBuf,
    bytes: Vec<u8>,
    stats: repi_unpack_v2::RepackStats,
    overwrite: bool,
}

impl PreparedOperation for RepackJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        let parent = self
            .output
            .parent()
            .ok_or_else(|| "重封输出文件缺少父目录".to_owned())?;
        progress.report(&format!("写入 {} 条 DAT 索引", self.stats.entries))?;
        let stage = TemporaryFile::write(parent, &self.bytes)?;
        let warnings = commit_file(stage, &self.output, self.overwrite)?
            .into_iter()
            .collect();
        Ok(RunReport {
            summary: "RepiPack v2 重封完成；输入文件和脚本来源未修改。".into(),
            totals: vec![
                ("索引条目".into(), self.stats.entries as u64),
                ("修改并重封".into(), self.stats.changed as u64),
                ("重新压缩".into(), self.stats.compressed_changed as u64),
                ("压缩无收益/raw".into(), self.stats.raw_changed as u64),
                ("原样保留 packed".into(), self.stats.preserved_packed as u64),
                ("DAT 字节".into(), self.bytes.len() as u64),
            ],
            outputs: vec![self.output.clone()],
            warnings,
        })
    }
}

struct UnpackJob {
    preview: Preview,
    archive_path: PathBuf,
    bytes: Vec<u8>,
    archive: Archive,
    output: PathBuf,
    overwrite: bool,
}

impl PreparedOperation for UnpackJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        let parent = self
            .output
            .parent()
            .ok_or_else(|| "输出目录缺少父目录".to_string())?;
        progress.report(&format!("解码 {} 条资源", self.archive.entries.len()))?;
        let stage = TemporaryDirectory::create(parent)?;
        fs::create_dir(stage.path.join("files"))?;
        fs::create_dir(stage.path.join("unresolved"))?;

        let mut manifest = String::from(
            "index\toffset\tpacked_size\tunpacked_size\tcrypt\tstatus\tname\toutput_name\n",
        );
        let mut used_paths = HashSet::new();
        let mut decoded = 0u64;
        let mut unresolved = 0u64;
        let mut decoded_bytes = 0u64;
        let mut warnings = Vec::new();

        for (index, entry) in self.archive.entries.iter().enumerate() {
            let escaped_name = escape_field(&entry.name);
            let (status, output_name) = match safe_components(&entry.name) {
                Ok(components) => {
                    let relative = unique_relative_path(components, index, &mut used_paths);
                    let target = stage.path.join("files").join(&relative);
                    match self.archive.unpack_entry(&self.bytes, entry) {
                        Ok(data) => {
                            if let Some(parent) = target.parent() {
                                fs::create_dir_all(parent)?;
                            }
                            write_new_file(&target, &data)?;
                            decoded += 1;
                            decoded_bytes += data.len() as u64;
                            let status = if data.len() != entry.unpacked_size as usize {
                                warnings.push(format!(
                                    "entry {index} ({}) LZSS 提前结束: {}/{} 字节",
                                    entry.name,
                                    data.len(),
                                    entry.unpacked_size
                                ));
                                "short-decode"
                            } else if entry.crypt > 2 {
                                warnings.push(format!(
                                    "entry {index} ({}) 使用未知数据变换类型 {}，按原字节继续",
                                    entry.name, entry.crypt
                                ));
                                "unknown-crypt"
                            } else {
                                "decoded"
                            };
                            (status.to_owned(), path_to_slashes(&relative))
                        }
                        Err(error) => {
                            write_packed_unresolved(
                                &stage.path,
                                index,
                                &self.archive,
                                &self.bytes,
                                entry,
                            )?;
                            warnings
                                .push(format!("entry {index} ({}) 解码失败: {error}", entry.name));
                            unresolved += 1;
                            ("decode-error".to_owned(), String::new())
                        }
                    }
                }
                Err(error) => {
                    write_packed_unresolved(&stage.path, index, &self.archive, &self.bytes, entry)?;
                    warnings.push(format!(
                        "entry {index} 路径不安全，保留 packed 数据: {error}"
                    ));
                    unresolved += 1;
                    ("unsafe-name".to_owned(), String::new())
                }
            };
            manifest.push_str(&format!(
                "{index}\t0x{:x}\t{}\t{}\t{}\t{status}\t{escaped_name}\t{output_name}\n",
                entry.offset, entry.packed_size, entry.unpacked_size, entry.crypt
            ));
        }

        manifest.push_str(&format!(
            "# source={}\n# header={}\n# version=2\n",
            escape_field(&self.archive_path.display().to_string()),
            escape_field(&self.archive.header_name)
        ));
        write_new_file(&stage.path.join("manifest.tsv"), manifest.as_bytes())?;
        progress.report("校验输出并提交目录")?;
        commit_directory(stage, &self.output, self.overwrite)?;

        Ok(RunReport {
            summary: "RepiPack v2 解包完成；输入文件未修改。".into(),
            totals: vec![
                ("表项".into(), self.archive.entries.len() as u64),
                ("已解码".into(), decoded),
                ("未知/失败".into(), unresolved),
                ("解码字节".into(), decoded_bytes),
            ],
            outputs: vec![self.output.clone()],
            warnings,
        })
    }
}

fn output_directory(path: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(path)
        .map_err(|error| format!("无法解析输出路径 {}: {error}", path.display()))?;
    let parent = absolute.parent().ok_or("输出目录缺少父目录")?;
    let parent = fs::canonicalize(parent)
        .map_err(|error| format!("输出父目录不存在 {}: {error}", parent.display()))?;
    let name = absolute.file_name().ok_or("输出目录缺少名称")?;
    let output = parent.join(name);
    if let Ok(metadata) = fs::symlink_metadata(&output) {
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(format!("输出路径必须是普通目录或不存在: {}", output.display()).into());
        }
    }
    Ok(output)
}

fn output_file_path(path: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(path)
        .map_err(|error| format!("无法解析输出文件路径 {}: {error}", path.display()))?;
    let parent = absolute.parent().ok_or("输出文件缺少父目录")?;
    let parent = fs::canonicalize(parent)
        .map_err(|error| format!("输出父目录不存在 {}: {error}", parent.display()))?;
    let name = absolute.file_name().ok_or("输出文件缺少名称")?;
    let output = parent.join(name);
    if let Ok(metadata) = fs::symlink_metadata(&output) {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!("输出只能覆盖普通文件: {}", output.display()).into());
        }
    }
    Ok(output)
}

fn unique_relative_path(
    components: Vec<String>,
    index: usize,
    used: &mut HashSet<String>,
) -> PathBuf {
    let relative: PathBuf = components.iter().collect();
    let key = path_to_slashes(&relative).to_lowercase();
    if used.insert(key) {
        return relative;
    }
    let file_name = relative
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "entry.bin".into());
    let candidate = relative.with_file_name(format!("{file_name}.__entry_{index:06}"));
    used.insert(path_to_slashes(&candidate).to_lowercase());
    candidate
}

fn path_to_slashes(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn escape_field(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

fn write_packed_unresolved(
    stage: &Path,
    index: usize,
    archive: &Archive,
    bytes: &[u8],
    entry: &Entry,
) -> Result<()> {
    let target = stage
        .join("unresolved")
        .join(format!("entry-{index:06}.packed"));
    write_new_file(&target, archive.packed_slice(bytes, entry)?)?;
    Ok(())
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("创建输出 {} 失败: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("写入输出 {} 失败: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("刷新输出 {} 失败: {error}", path.display()))?;
    Ok(())
}

struct TemporaryFile {
    path: PathBuf,
}

impl TemporaryFile {
    fn write(parent: &Path, bytes: &[u8]) -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let path = parent.join(format!(
                ".repi-v2-repack-{}-{}.tmp",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    file.write_all(bytes).map_err(|error| {
                        format!("写入暂存 DAT {} 失败: {error}", path.display())
                    })?;
                    file.sync_all().map_err(|error| {
                        format!("刷新暂存 DAT {} 失败: {error}", path.display())
                    })?;
                    return Ok(Self { path });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!("创建暂存 DAT {} 失败: {error}", path.display()).into())
                }
            }
        }
        Err("无法分配唯一 DAT 暂存文件".into())
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn commit_file(stage: TemporaryFile, output: &Path, overwrite: bool) -> Result<Option<String>> {
    if output.exists() {
        if !overwrite {
            return Err(format!("输出已存在，默认不覆盖: {}", output.display()).into());
        }
        let metadata = fs::symlink_metadata(output)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!("输出只能覆盖普通文件: {}", output.display()).into());
        }
        let backup = temporary_file_sibling(output)?;
        fs::rename(output, &backup)?;
        if let Err(error) = fs::rename(&stage.path, output) {
            let restore = fs::rename(&backup, output);
            return match restore {
                Ok(()) => Err(format!("提交 DAT 失败，旧输出已恢复: {error}").into()),
                Err(restore_error) => Err(format!(
                    "提交 DAT 失败 ({error})，旧输出恢复失败 ({restore_error})，备份保留在 {}",
                    backup.display()
                )
                .into()),
            };
        }
        if let Err(error) = fs::remove_file(&backup) {
            return Ok(Some(format!(
                "新 DAT 已提交，但旧文件备份无法清理: {} ({error})",
                backup.display()
            )));
        }
        return Ok(None);
    }
    fs::rename(&stage.path, output)?;
    Ok(None)
}

fn temporary_file_sibling(output: &Path) -> Result<PathBuf> {
    static NEXT: AtomicU64 = AtomicU64::new(100_000);
    let parent = output.parent().ok_or("输出文件缺少父目录")?;
    let name = output.file_name().ok_or("输出文件缺少名称")?;
    for _ in 0..100 {
        let candidate = parent.join(format!(
            ".{}.backup-{}-{}",
            name.to_string_lossy(),
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("无法分配唯一 DAT 备份路径".into())
}

fn commit_directory(stage: TemporaryDirectory, output: &Path, overwrite: bool) -> Result<()> {
    if output.exists() {
        if !overwrite {
            return Err(format!("输出已存在，默认不覆盖: {}", output.display()).into());
        }
        let metadata = fs::symlink_metadata(output)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(format!("输出只能覆盖普通目录: {}", output.display()).into());
        }
        fs::remove_dir_all(output)?;
    }
    fs::rename(&stage.path, output)?;
    Ok(())
}

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn create(parent: &Path) -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let path = parent.join(format!(
                ".repi-v2-unpack-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!("创建暂存目录失败 {}: {error}", path.display()).into())
                }
            }
        }
        Err("无法分配唯一暂存目录".into())
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

fn main() -> ExitCode {
    let result = (|| {
        let executable = std::env::current_exe()?.into_os_string();
        Panel::new(
            executable,
            "RepiPack v2 解包器",
            env!("CARGO_PKG_VERSION"),
            vec![
                Box::new(Unpack),
                Box::new(Repack),
                Box::new(ExportText),
                Box::new(ImportText),
            ],
        )?
        .run_env()
    })();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[失败] {error}");
            ExitCode::FAILURE
        }
    }
}
