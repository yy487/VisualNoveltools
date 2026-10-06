#![forbid(unsafe_code)]

pub mod font_plan;
pub mod rebuild;
pub mod scr;

use fivec_new::{Archive, Inspection};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub const WORKSPACE_FORMAT: &str = "chrome-paradise-pc98-unpack-v1";

pub type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize)]
pub struct TextWorkspaceReport {
    pub output: PathBuf,
    pub files: usize,
    pub bytes: usize,
    pub diagnostics: usize,
}

/// Extract every SCR from an unpacked disk workspace into a new translation
/// workspace. All editable JSON files are kept in one flat directory; each
/// document carries its source_file for injection.
pub fn extract_scr_workspace(input: &Path, output: &Path) -> Result<TextWorkspaceReport> {
    let input = canonical_directory(input)?;
    let output = normalized_new_output(output)?;
    if source_contained_by_output(&input, &output) {
        return Err("文本输出目录不能位于输入工作区内".into());
    }
    let parent = output
        .parent()
        .ok_or_else(|| "输出目录没有父目录".to_string())?;
    let stage = allocate_stage(parent)?;
    let result = (|| {
        let mut files = 0usize;
        let mut bytes = 0usize;
        let mut diagnostics = 0usize;
        for source in recursive_files(&input)? {
            if !source
                .extension()
                .map(|value| value.eq_ignore_ascii_case("SCR"))
                .unwrap_or(false)
            {
                continue;
            }
            let raw =
                fs::read(&source).map_err(|e| format!("读取 {} 失败: {e}", source.display()))?;
            let relative = source
                .strip_prefix(&input)
                .map_err(|_| "无法计算 SCR 相对路径".to_string())?;
            let relative_text = relative.to_string_lossy().replace('\\', "/");
            let mut document = scr::extract_document(&raw, relative_text.clone())?;
            document.source_file = relative_text;
            files += 1;
            bytes += raw.len();
            diagnostics += document.diagnostics.len();
            let destination = stage
                .join("translations")
                .join(flat_translation_name(relative));
            fs::create_dir_all(stage.join("translations")).map_err(|e| e.to_string())?;
            let encoded = serde_json::to_vec_pretty(&document).map_err(|e| e.to_string())?;
            fs::write(destination, encoded).map_err(|e| e.to_string())?;
        }
        if files == 0 {
            return Err("输入工作区没有找到 .SCR 文件".into());
        }
        let manifest = serde_json::json!({
            "format": "chrome-paradise-scr-translations-v2",
            "source_workspace": input.display().to_string(),
            "files": files,
            "bytes": bytes,
            "diagnostics": diagnostics,
        });
        fs::write(
            stage.join("text_workspace.json"),
            serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if output.exists() {
            return Err("文本输出目录在执行期间出现，请重新预检".into());
        }
        fs::rename(&stage, &output).map_err(|e| format!("提交文本工作区失败: {e}"))?;
        Ok(TextWorkspaceReport {
            output: output.clone(),
            files,
            bytes,
            diagnostics,
        })
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&stage);
    }
    result
}

/// Apply SCR JSON files to a copied unpacked workspace.  The original source
/// tree is never modified and every JSON is checked against its SHA-256.
pub fn inject_scr_workspace(
    input: &Path,
    translations: &Path,
    output: &Path,
) -> Result<TextWorkspaceReport> {
    inject_scr_workspace_internal(input, translations, output, None)
}

/// Inject using a previously generated font plan. This encodes Unicode text
/// to the same carrier bytes used when creating the paired `font.tmp`.
pub fn inject_scr_workspace_with_font_plan(
    input: &Path,
    translations: &Path,
    output: &Path,
    plan: &font_plan::FontPlan,
) -> Result<TextWorkspaceReport> {
    inject_scr_workspace_internal(input, translations, output, Some(plan))
}

fn inject_scr_workspace_internal(
    input: &Path,
    translations: &Path,
    output: &Path,
    encoding_plan: Option<&font_plan::FontPlan>,
) -> Result<TextWorkspaceReport> {
    let input = canonical_directory(input)?;
    let translations = canonical_directory(translations)?;
    let output = normalized_new_output(output)?;
    if source_contained_by_output(&input, &output)
        || source_contained_by_output(&translations, &output)
    {
        return Err("注回输出目录不能位于输入或翻译工作区内".into());
    }
    let parent = output
        .parent()
        .ok_or_else(|| "输出目录没有父目录".to_string())?;
    let stage = allocate_stage(parent)?;
    let result = (|| {
        copy_directory(&input, &stage)?;
        let mut files = 0usize;
        let mut bytes = 0usize;
        let mut diagnostics = 0usize;
        let mut planned_strings = 0usize;
        let nested_translation_root = translations.join("translations");
        let translation_root = if nested_translation_root.is_dir() {
            nested_translation_root
        } else {
            translations.clone()
        };
        for json_path in recursive_files(&translation_root)? {
            if !json_path
                .extension()
                .map(|value| value.eq_ignore_ascii_case("json"))
                .unwrap_or(false)
                || json_path
                    .file_name()
                    .map(|value| value == "text_workspace.json")
                    .unwrap_or(false)
            {
                continue;
            }
            let json = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            let document: scr::ScrDocument = serde_json::from_str(&json)
                .map_err(|e| format!("解析 {} 失败: {e}", json_path.display()))?;
            let relative_source = safe_relative_source(&document.source_file)?;
            let source = input.join(&relative_source);
            let destination = stage.join(&relative_source);
            if !source.is_file() {
                return Err(format!("翻译对应的 SCR 不存在: {}", source.display()));
            }
            let raw = fs::read(&source).map_err(|e| e.to_string())?;
            let rebuilt = if let Some(plan) = encoding_plan {
                let encoded = font_plan::planned_strings_for_document(plan, &document)?;
                planned_strings += encoded.len();
                scr::apply_document_with_encoded_strings(&raw, &document, &encoded)?
            } else {
                scr::apply_document(&raw, &document)?
            };
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::write(destination, &rebuilt).map_err(|e| e.to_string())?;
            files += 1;
            bytes += rebuilt.len();
            diagnostics += document.diagnostics.len();
        }
        if files == 0 {
            return Err("翻译工作区没有找到 SCR JSON".into());
        }
        if let Some(plan) = encoding_plan {
            if files != plan.documents || planned_strings != plan.encodings.len() {
                return Err(format!(
                    "字体计划覆盖 {} 份/{} 条字符串，实际注回 {} 份/{} 条；请用当前完整翻译工作区重建 font-plan",
                    plan.documents,
                    plan.encodings.len(),
                    files,
                    planned_strings
                ));
            }
        }
        if output.exists() {
            return Err("注回输出目录在执行期间出现，请重新预检".into());
        }
        fs::rename(&stage, &output).map_err(|e| format!("提交注回工作区失败: {e}"))?;
        Ok(TextWorkspaceReport {
            output: output.clone(),
            files,
            bytes,
            diagnostics,
        })
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&stage);
    }
    result
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskSummary {
    pub disk_number: usize,
    pub source_name: String,
    pub source_size: usize,
    pub source_sha256: String,
    pub output_directory: String,
    pub files: usize,
    pub bytes: usize,
    pub inspection: Inspection,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceManifest {
    pub format: String,
    pub tool_version: String,
    pub disk_count: usize,
    pub total_files: usize,
    pub total_bytes: usize,
    pub disks: Vec<DiskSummary>,
}

#[derive(Debug, Clone)]
pub struct UnpackReport {
    pub output: PathBuf,
    pub disks: usize,
    pub files: usize,
    pub bytes: usize,
    pub warnings: Vec<String>,
}

struct PreparedDisk {
    source: PathBuf,
    archive: Archive,
}

pub struct PreparedUnpack {
    output: PathBuf,
    disks: Vec<PreparedDisk>,
}

/// Parse every source and validate the destination without writing anything.
/// The returned archives own immutable snapshots of the source bytes, so the
/// execute phase cannot accidentally use a different image after preflight.
pub fn prepare_unpack(inputs: &[PathBuf], output: &Path) -> Result<PreparedUnpack> {
    if inputs.is_empty() {
        return Err("至少需要一张 D88 输入镜像".into());
    }
    let output = normalized_new_output(output)?;
    let mut disks = Vec::with_capacity(inputs.len());
    let mut seen = Vec::new();
    for input in inputs {
        let source = canonical_file(input)?;
        let key = source.to_string_lossy().to_lowercase();
        if seen.iter().any(|value: &String| value == &key) {
            return Err(format!("重复输入镜像: {}", source.display()));
        }
        seen.push(key);
        if source_contained_by_output(&source, &output) {
            return Err(format!(
                "输出目录包含输入镜像，拒绝写入: {}",
                output.display()
            ));
        }
        let archive = Archive::open(&source)?;
        archive.selected_volumes("all")?;
        disks.push(PreparedDisk { source, archive });
    }
    Ok(PreparedUnpack { output, disks })
}

impl PreparedUnpack {
    pub fn output(&self) -> &Path {
        &self.output
    }

    pub fn disk_count(&self) -> usize {
        self.disks.len()
    }

    pub fn source_names(&self) -> Vec<String> {
        self.disks
            .iter()
            .map(|disk| {
                disk.source
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| disk.source.display().to_string())
            })
            .collect()
    }

    /// Export all validated disks into a staging directory and atomically
    /// rename it into place. A failure removes only the staging directory.
    pub fn execute(self) -> Result<UnpackReport> {
        let parent = self
            .output
            .parent()
            .ok_or_else(|| "输出目录没有父目录".to_string())?;
        let stage = allocate_stage(parent)?;
        let result = (|| {
            let mut summaries = Vec::with_capacity(self.disks.len());
            let mut files = 0usize;
            let mut bytes = 0usize;
            let mut warnings = Vec::new();

            for (index, disk) in self.disks.iter().enumerate() {
                let output_directory = format!("disk-{:02}", index + 1);
                let export_path = stage.join(&output_directory);
                let prepared = disk.archive.prepare_export("all", &export_path, false)?;
                let inspection = prepared.manifest().inspection.clone();
                let report = prepared.execute()?;
                files = files
                    .checked_add(report.files)
                    .ok_or_else(|| "文件计数溢出".to_string())?;
                bytes = bytes
                    .checked_add(report.bytes)
                    .ok_or_else(|| "字节计数溢出".to_string())?;
                warnings.extend(report.warnings);
                summaries.push(DiskSummary {
                    disk_number: index + 1,
                    source_name: disk
                        .source
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| disk.source.display().to_string()),
                    source_size: inspection.source_size,
                    source_sha256: inspection.source_sha256.clone(),
                    output_directory,
                    files: report.files,
                    bytes: report.bytes,
                    inspection,
                });
            }

            let workspace = WorkspaceManifest {
                format: WORKSPACE_FORMAT.into(),
                tool_version: env!("CARGO_PKG_VERSION").into(),
                disk_count: summaries.len(),
                total_files: files,
                total_bytes: bytes,
                disks: summaries,
            };
            let encoded = serde_json::to_vec_pretty(&workspace).map_err(|e| e.to_string())?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(stage.join("workspace.json"))
                .map_err(|e| e.to_string())?;
            file.write_all(&encoded).map_err(|e| e.to_string())?;
            file.write_all(b"\n").map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
            drop(file);

            if self.output.exists() {
                return Err("输出目录在执行期间出现，请重新预检".into());
            }
            fs::rename(&stage, &self.output).map_err(|e| format!("提交解包目录失败: {e}"))?;
            Ok(UnpackReport {
                output: self.output.clone(),
                disks: self.disks.len(),
                files,
                bytes,
                warnings,
            })
        })();

        if result.is_err() {
            let _ = fs::remove_dir_all(&stage);
        }
        result
    }
}

fn canonical_file(path: &Path) -> Result<PathBuf> {
    let resolved =
        fs::canonicalize(path).map_err(|e| format!("无法读取输入 {}: {e}", path.display()))?;
    let metadata = fs::metadata(&resolved).map_err(|e| e.to_string())?;
    if !metadata.is_file() {
        return Err(format!("输入不是普通文件: {}", resolved.display()));
    }
    Ok(resolved)
}

fn flat_translation_name(relative: &Path) -> String {
    let name = relative.to_string_lossy().replace(['\\', '/'], "__");
    let source = relative.to_string_lossy();
    let suffix = &fivec_new::sha256(source.as_bytes())[..12];
    format!("{name}__{suffix}.json")
}

fn safe_relative_source(source_file: &str) -> Result<PathBuf> {
    let path = Path::new(source_file);
    if source_file.is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "SCR JSON source_file 不是安全的相对路径: {source_file}"
        ));
    }
    Ok(path.to_path_buf())
}

fn canonical_directory(path: &Path) -> Result<PathBuf> {
    let resolved =
        fs::canonicalize(path).map_err(|e| format!("无法读取输入目录 {}: {e}", path.display()))?;
    if !resolved.is_dir() {
        return Err(format!("输入不是目录: {}", resolved.display()));
    }
    Ok(resolved)
}

fn recursive_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut output = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let mut entries = fs::read_dir(&directory)
            .map_err(|e| format!("读取目录 {} 失败: {e}", directory.display()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        entries.sort_by_key(|entry| entry.path());
        for entry in entries {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if metadata.is_dir() {
                stack.push(path);
            } else if metadata.is_file() {
                output.push(path);
            }
        }
    }
    output.sort();
    Ok(output)
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    for path in recursive_files(source)? {
        let relative = path
            .strip_prefix(source)
            .map_err(|_| "无法复制工作区相对路径".to_string())?;
        let target = destination.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::copy(&path, &target).map_err(|e| format!("复制 {} 失败: {e}", path.display()))?;
    }
    Ok(())
}

fn normalized_new_output(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    if absolute.exists() {
        return Err(format!("输出已存在: {}", absolute.display()));
    }
    let name = absolute
        .file_name()
        .ok_or_else(|| format!("无效的输出目录: {}", absolute.display()))?;
    if name.to_string_lossy().is_empty() {
        return Err("输出目录名不能为空".into());
    }
    let parent = absolute
        .parent()
        .ok_or_else(|| "输出目录没有父目录".to_string())?;
    let parent = fs::canonicalize(parent).map_err(|e| format!("输出目录的父目录无法读取: {e}"))?;
    if !parent.is_dir() {
        return Err(format!("输出目录的父路径不是目录: {}", parent.display()));
    }
    Ok(parent.join(name))
}

fn source_contained_by_output(source: &Path, output: &Path) -> bool {
    #[cfg(windows)]
    {
        let source: Vec<_> = source
            .components()
            .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
            .collect();
        let output: Vec<_> = output
            .components()
            .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
            .collect();
        source.starts_with(&output)
    }
    #[cfg(not(windows))]
    {
        source.starts_with(output)
    }
}

fn allocate_stage(parent: &Path) -> Result<PathBuf> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    for _ in 0..100 {
        let candidate = parent.join(format!(
            ".chrome-paradise-stage-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("无法创建暂存目录: {error}")),
        }
    }
    Err("无法分配暂存目录".into())
}
