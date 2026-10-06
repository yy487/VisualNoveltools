//! End-to-end extraction and localization build for the five Galaxy Railway disks.

use crate::{crs_text, data_disks, fat_rebuild, font_plan, sha256, text, Result};
use fivec_new::{Archive, FileEntry};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use vn_d88::{Decoder, StandardCodec};
use vn_font::font_98;

pub const WORKSPACE_SCHEMA: &str = "galaxy-railway-pc98-workspace-v1";
pub const BUILD_SCHEMA: &str = "galaxy-railway-pc98-build-v1";
const TRANSLATIONS_DIR: &str = "translations";
#[derive(Debug, Clone)]
pub struct DiskInput {
    pub path: PathBuf,
    pub source_name: String,
    pub bytes: Vec<u8>,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceManifest {
    pub schema: String,
    pub disks: Vec<DiskRecord>,
    pub source_files: Vec<SourceFileRecord>,
    pub translation_files: Vec<TranslationFileRecord>,
    pub message_entries: usize,
    #[serde(default)]
    pub crs_entries: usize,
    pub boot_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiskRecord {
    pub disk_number: u8,
    pub source_name: String,
    pub size: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFileRecord {
    pub disk_number: u8,
    pub path: String,
    pub size: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationFileRecord {
    pub file: String,
    pub source_file: String,
    pub entries: usize,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildManifest {
    pub schema: String,
    pub source_disks: Vec<DiskRecord>,
    pub output_disks: Vec<DiskRecord>,
    pub translation_files: usize,
    pub changed_files: usize,
    pub message_entries: usize,
    pub crs_entries: usize,
    pub boot_entries: usize,
    pub font_file: String,
    pub font_sha256: String,
    pub font_face: String,
    pub patched_glyphs: usize,
    pub allocated_clusters: usize,
    pub released_clusters: usize,
}

#[derive(Debug, Clone)]
pub struct WorkflowReport {
    pub summary: String,
    pub files: usize,
    pub messages: usize,
    pub changed_files: usize,
    pub patched_glyphs: usize,
    pub outputs: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct TranslationInputs {
    pub root: PathBuf,
    pub files: BTreeMap<String, Vec<u8>>,
}

#[derive(Debug, Clone)]
struct SourceFile {
    path: String,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
struct TranslationOutput {
    file: String,
    bytes: Vec<u8>,
}

struct StagedDir {
    path: PathBuf,
    parent: PathBuf,
    committed: bool,
}

impl StagedDir {
    fn new(output: &Path) -> Result<Self> {
        if output.exists() {
            return Err(format!("输出已存在，拒绝覆盖: {}", output.display()));
        }
        let parent = output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let parent = fs::canonicalize(parent)
            .map_err(|error| format!("解析输出父目录 {} 失败: {error}", parent.display()))?;
        if !parent.is_dir() {
            return Err(format!("输出父路径不是目录: {}", parent.display()));
        }
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let name = format!(
                ".galaxy-railway-stage-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            );
            let path = parent.join(name);
            match fs::create_dir(&path) {
                Ok(()) => {
                    let path = fs::canonicalize(&path)
                        .map_err(|error| format!("解析暂存目录失败: {error}"))?;
                    return Ok(Self {
                        path,
                        parent,
                        committed: false,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(format!("无法创建暂存目录: {error}")),
            }
        }
        Err("无法分配唯一的暂存目录".into())
    }

    fn write(&self, relative: &str, bytes: &[u8]) -> Result<()> {
        let relative = safe_relative(relative)?;
        let path = self.path.join(relative);
        let parent = path.parent().ok_or("输出文件缺少父目录")?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("创建 {} 失败: {error}", parent.display()))?;
        let mut file = File::create(&path)
            .map_err(|error| format!("创建 {} 失败: {error}", path.display()))?;
        file.write_all(bytes)
            .map_err(|error| format!("写入 {} 失败: {error}", path.display()))?;
        file.sync_all()
            .map_err(|error| format!("同步 {} 失败: {error}", path.display()))
    }

    fn commit(mut self, output: &Path) -> Result<()> {
        if output.exists() {
            return Err(format!("执行期间输出路径出现: {}", output.display()));
        }
        fs::rename(&self.path, output)
            .map_err(|error| format!("提交输出目录 {} 失败: {error}", output.display()))?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for StagedDir {
    fn drop(&mut self) {
        if !self.committed {
            if let Ok(path) = fs::canonicalize(&self.path) {
                if path.parent() == Some(self.parent.as_path()) {
                    let _ = fs::remove_dir_all(path);
                }
            }
        }
    }
}

pub fn snapshot_disks(paths: [PathBuf; 5]) -> Result<[DiskInput; 5]> {
    let mut inputs = Vec::with_capacity(5);
    for (index, path) in paths.into_iter().enumerate() {
        let path = fs::canonicalize(&path).map_err(|error| {
            format!(
                "无法解析第 {} 张软碟 {}: {error}",
                index + 1,
                path.display()
            )
        })?;
        if !path.is_file() {
            return Err(format!(
                "第 {} 张软碟不是文件: {}",
                index + 1,
                path.display()
            ));
        }
        let bytes =
            fs::read(&path).map_err(|error| format!("读取第 {} 张软碟失败: {error}", index + 1))?;
        decode_single_disk(&bytes, index + 1)?;
        let source_name = path
            .file_name()
            .ok_or_else(|| format!("第 {} 张软碟没有文件名", index + 1))?
            .to_string_lossy()
            .into_owned();
        inputs.push(DiskInput {
            path,
            source_name,
            sha256: sha256(&bytes),
            bytes,
        });
    }
    inputs
        .try_into()
        .map_err(|_| "必须提供且只提供五张 D88 软碟".to_owned())
}

pub fn read_workspace(root: &Path) -> Result<WorkspaceManifest> {
    let root = fs::canonicalize(root)
        .map_err(|error| format!("无法读取文本工作区 {}: {error}", root.display()))?;
    let path = root.join("workspace.json");
    let bytes =
        fs::read(&path).map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
    let manifest: WorkspaceManifest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("workspace.json 格式错误: {error}"))?;
    if manifest.schema != WORKSPACE_SCHEMA {
        return Err(format!("不支持的工作区格式 {:?}", manifest.schema));
    }
    Ok(manifest)
}

pub fn read_translations(root: &Path) -> Result<TranslationInputs> {
    let root = fs::canonicalize(root)
        .map_err(|error| format!("无法读取翻译目录 {}: {error}", root.display()))?;
    if !root.is_dir() {
        return Err(format!("翻译路径不是目录: {}", root.display()));
    }
    let mut files = BTreeMap::new();
    for entry in fs::read_dir(&root).map_err(|error| format!("枚举翻译目录失败: {error}"))?
    {
        let entry = entry.map_err(|error| format!("读取翻译目录条目失败: {error}"))?;
        let kind = entry
            .file_type()
            .map_err(|error| format!("读取翻译条目类型失败: {error}"))?;
        if !kind.is_file() {
            continue;
        }
        let path = entry.path();
        if path
            .extension()
            .is_none_or(|extension| !extension.eq_ignore_ascii_case("json"))
        {
            continue;
        }
        let name = path
            .file_name()
            .ok_or("翻译 JSON 缺少文件名")?
            .to_string_lossy()
            .into_owned();
        if files.contains_key(&name.to_ascii_lowercase()) {
            return Err(format!("翻译目录中有不区分大小写的重名文件: {name}"));
        }
        let bytes = fs::read(&path)
            .map_err(|error| format!("读取翻译 JSON {} 失败: {error}", path.display()))?;
        files.insert(name.to_ascii_lowercase(), bytes);
    }
    Ok(TranslationInputs { root, files })
}

pub fn extract_workflow(disks: &[DiskInput; 5], output: &Path) -> Result<WorkflowReport> {
    protect_output(
        output,
        &disks
            .iter()
            .map(|disk| disk.path.as_path())
            .collect::<Vec<_>>(),
    )?;
    let staged = StagedDir::new(output)?;
    let disk1_files = extract_disk1_files(&disks[0])?;
    let mut source_files = Vec::with_capacity(disk1_files.len());
    let mut translation_files = Vec::new();
    let mut message_entries = 0usize;
    let mut crs_entries = 0usize;
    let mut boot_entries = 0usize;

    for file in &disk1_files {
        staged.write(&format!("source/disk-01/{}", file.path), &file.data)?;
        source_files.push(SourceFileRecord {
            disk_number: 1,
            path: file.path.clone(),
            size: file.data.len(),
            sha256: sha256(&file.data),
        });
        let upper = file.path.to_ascii_uppercase();
        if upper.ends_with(".CHN") {
            for document in text::extract_chn(&file.data, &disks[0].source_name)? {
                let filename = translation_filename(&document.source_file)?;
                let entries = document.entries.len();
                message_entries += entries;
                let bytes = json_bytes(&document)?;
                staged.write(&format!("{TRANSLATIONS_DIR}/{filename}"), &bytes)?;
                translation_files.push(TranslationFileRecord {
                    file: filename,
                    source_file: document.source_file,
                    entries,
                    kind: "msg".into(),
                });
            }
        } else if upper.ends_with(".CRS") {
            if let Some(document) = crs_text::extract_crs(&file.data, &file.path)? {
                if document.entries.is_empty() {
                    continue;
                }
                let filename = translation_filename(&document.source_file)?;
                let entries = document.entries.len();
                crs_entries += entries;
                staged.write(
                    &format!("{TRANSLATIONS_DIR}/{filename}"),
                    &json_bytes(&document)?,
                )?;
                translation_files.push(TranslationFileRecord {
                    file: filename,
                    source_file: document.source_file,
                    entries,
                    kind: "crs".into(),
                });
            }
        }
    }

    for (index, disk) in disks.iter().enumerate().skip(1) {
        let disk_number = index as u8 + 1;
        let source_file = format!("BOOT_DISK{disk_number:02}.D88");
        let boot = data_disks::extract_boot_document(&disk.bytes, source_file)?;
        let filename = translation_filename(&boot.source_file)?;
        boot_entries += boot.entries.len();
        staged.write(
            &format!("{TRANSLATIONS_DIR}/{filename}"),
            &json_bytes(&boot)?,
        )?;
        translation_files.push(TranslationFileRecord {
            file: filename,
            source_file: boot.source_file,
            entries: boot.entries.len(),
            kind: "boot".into(),
        });
        let resources = data_disks::extract_resource_catalogs(&disk.bytes, disk_number)?;
        staged.write(
            &format!("resources/DISK{disk_number:02}.json"),
            &json_bytes(&resources)?,
        )?;
    }

    let manifest = WorkspaceManifest {
        schema: WORKSPACE_SCHEMA.into(),
        disks: disk_records(disks),
        source_files,
        translation_files,
        message_entries,
        crs_entries,
        boot_entries,
    };
    staged.write("workspace.json", &json_bytes(&manifest)?)?;
    staged.commit(output)?;

    Ok(WorkflowReport {
        summary: format!(
            "提取完成：{} 个源文件、{} 条 MSG、{} 条 CRS、{} 条启动提示",
            manifest.source_files.len(),
            message_entries,
            crs_entries,
            boot_entries
        ),
        files: manifest.source_files.len(),
        messages: message_entries + crs_entries + boot_entries,
        changed_files: 0,
        patched_glyphs: 0,
        outputs: vec![output.to_path_buf()],
    })
}

pub fn rebuild_workflow(
    disks: &[DiskInput; 5],
    workspace: &WorkspaceManifest,
    translations: &TranslationInputs,
    output: &Path,
    font_face: &str,
) -> Result<WorkflowReport> {
    let mut protected = disks
        .iter()
        .map(|disk| disk.path.as_path())
        .collect::<Vec<_>>();
    protected.push(translations.root.as_path());
    protect_output(output, &protected)?;
    if workspace.schema != WORKSPACE_SCHEMA {
        return Err(format!("不支持的工作区格式 {:?}", workspace.schema));
    }
    validate_disk_sources(disks, &workspace.disks)?;
    let disk1_files = extract_disk1_files(&disks[0])?;
    validate_source_files(&disk1_files, &workspace.source_files)?;

    let mut expected_translation_names = BTreeSet::new();
    let mut translation_outputs = Vec::<TranslationOutput>::new();
    let mut display_texts = Vec::<font_plan::DisplayText>::new();
    let mut message_entries = 0usize;
    let mut crs_entries = 0usize;
    let mut boot_entries = 0usize;
    let mut replacements = BTreeMap::<String, Vec<u8>>::new();
    let mut chn_documents = BTreeMap::<String, Vec<text::TextDocument>>::new();
    let mut crs_documents = BTreeMap::<String, crs_text::CrsDocument>::new();
    let mut changed_files = 0usize;

    for file in &disk1_files {
        if !file.path.to_ascii_uppercase().ends_with(".CHN") {
            continue;
        }
        let documents = text::extract_chn(&file.data, &disks[0].source_name)?;
        if documents.is_empty() {
            continue;
        }
        let mut merged = Vec::with_capacity(documents.len());
        for baseline in documents {
            let filename = translation_filename(&baseline.source_file)?;
            expected_translation_names.insert(filename.to_ascii_lowercase());
            let actual = load_json_override::<text::TextDocument>(translations, &filename)?;
            let document = actual.unwrap_or(baseline);
            message_entries += document.entries.len();
            for entry in &document.entries {
                display_texts.push(font_plan::DisplayText {
                    source_file: document.source_file.clone(),
                    index: entry.index,
                    original: entry.scr_msg.clone(),
                    display: entry.message.clone(),
                });
            }
            translation_outputs.push(TranslationOutput {
                file: filename,
                bytes: json_bytes(&document)?,
            });
            merged.push(document);
        }
        chn_documents.insert(file.path.clone(), merged);
    }

    for file in &disk1_files {
        if !file.path.to_ascii_uppercase().ends_with(".CRS") {
            continue;
        }
        let Some(baseline) = crs_text::extract_crs(&file.data, &file.path)? else {
            continue;
        };
        if baseline.entries.is_empty() {
            continue;
        }
        let filename = translation_filename(&baseline.source_file)?;
        expected_translation_names.insert(filename.to_ascii_lowercase());
        let actual = load_json_override::<crs_text::CrsDocument>(translations, &filename)?;
        let document = actual.unwrap_or(baseline);
        crs_entries += document.entries.len();
        for entry in &document.entries {
            display_texts.push(font_plan::DisplayText {
                source_file: document.source_file.clone(),
                index: entry.index,
                original: entry.scr_msg.clone(),
                display: entry.message.clone(),
            });
        }
        translation_outputs.push(TranslationOutput {
            file: filename,
            bytes: json_bytes(&document)?,
        });
        crs_documents.insert(file.path.clone(), document);
    }

    let mut boot_documents = Vec::with_capacity(4);
    for (index, disk) in disks.iter().enumerate().skip(1) {
        let disk_number = index as u8 + 1;
        let source_file = format!("BOOT_DISK{disk_number:02}.D88");
        let baseline = data_disks::extract_boot_document(&disk.bytes, source_file)?;
        let filename = translation_filename(&baseline.source_file)?;
        expected_translation_names.insert(filename.to_ascii_lowercase());
        let actual = load_json_override::<data_disks::BootDocument>(translations, &filename)?;
        let document = actual.unwrap_or(baseline);
        boot_entries += document.entries.len();
        for entry in &document.entries {
            display_texts.push(font_plan::DisplayText {
                source_file: document.source_file.clone(),
                index: entry.index,
                original: entry.scr_msg.clone(),
                display: entry.message.clone(),
            });
        }
        translation_outputs.push(TranslationOutput {
            file: filename,
            bytes: json_bytes(&document)?,
        });
        boot_documents.push(document);
    }

    reject_unknown_translations(translations, &expected_translation_names)?;
    let font = font_plan::build(font_98::EMBEDDED_FONT, &display_texts, font_face)?;
    let encoding = font.encoding.clone();
    let font_bytes = font.bytes;
    let font_report = font.report;

    let mut rebuilt_disks: Vec<Vec<u8>> = disks.iter().map(|disk| disk.bytes.clone()).collect();
    for ((index, disk), document) in disks.iter().enumerate().skip(1).zip(boot_documents.iter()) {
        let rebuilt = data_disks::apply_boot_document(&disk.bytes, document, &encoding)?;
        if rebuilt != disk.bytes {
            changed_files += 1;
        }
        rebuilt_disks[index] = rebuilt;
    }

    // MSG files are reflowed with the same global EncodingPlan used to draw font.bmp.
    for file in &disk1_files {
        if !file.path.to_ascii_uppercase().ends_with(".CHN") {
            continue;
        }
        let Some(documents) = chn_documents.get(&file.path) else {
            continue;
        };
        let rebuilt = text::apply_chn(&file.data, documents, &encoding)?;
        replacements.insert(file.path.clone(), rebuilt);
    }

    for file in &disk1_files {
        let Some(document) = crs_documents.get(&file.path) else {
            continue;
        };
        let rebuilt = crs_text::apply_crs(&file.data, document, &encoding)?;
        replacements.insert(file.path.clone(), rebuilt);
    }

    let disk1_report = fat_rebuild::rebuild_disk1_d88(&disks[0].bytes, &replacements)?;
    if disk1_report.bytes != disks[0].bytes && disk1_report.changed_files > 0 {
        // Recount changed outputs from final rebuilt disk content below.
    }
    rebuilt_disks[0] = disk1_report.bytes;
    let output_disks = rebuilt_disks
        .iter()
        .enumerate()
        .map(|(index, bytes)| DiskRecord {
            disk_number: index as u8 + 1,
            source_name: disks[index].source_name.clone(),
            size: bytes.len(),
            sha256: sha256(bytes),
        })
        .collect::<Vec<_>>();

    let manifest = BuildManifest {
        schema: BUILD_SCHEMA.into(),
        source_disks: disk_records(disks),
        output_disks: output_disks.clone(),
        translation_files: translation_outputs.len(),
        changed_files: disk1_report.changed_files + changed_files,
        message_entries,
        crs_entries,
        boot_entries,
        font_file: "font.bmp".into(),
        font_sha256: sha256(&font_bytes),
        font_face: font_report.face.clone(),
        patched_glyphs: font_report.patched_glyphs,
        allocated_clusters: disk1_report.allocated_clusters,
        released_clusters: disk1_report.released_clusters,
    };

    let staged = StagedDir::new(output)?;
    for (index, bytes) in rebuilt_disks.iter().enumerate() {
        staged.write(&disks[index].source_name, bytes)?;
    }
    staged.write("font.bmp", &font_bytes)?;
    staged.write("font_mapping.json", &json_bytes(&font_report)?)?;
    staged.write("build_report.json", &json_bytes(&manifest)?)?;
    for translation in translation_outputs {
        staged.write(
            &format!("{TRANSLATIONS_DIR}/{}", translation.file),
            &translation.bytes,
        )?;
    }
    staged.commit(output)?;

    Ok(WorkflowReport {
        summary: format!(
            "重建完成：{} 张软碟，{} 条 MSG，{} 条 CRS，{} 条启动提示，改写 {} 个文件，font.bmp 绘制 {} 个字形",
            rebuilt_disks.len(), message_entries, crs_entries, boot_entries, manifest.changed_files,
            manifest.patched_glyphs
        ),
        files: disk1_files.len(),
        messages: message_entries + crs_entries + boot_entries,
        changed_files: manifest.changed_files,
        patched_glyphs: manifest.patched_glyphs,
        outputs: vec![output.to_path_buf()],
    })
}

fn extract_disk1_files(disk: &DiskInput) -> Result<Vec<SourceFile>> {
    let archive = Archive::from_bytes(disk.source_name.clone(), disk.bytes.clone())
        .map_err(|error| format!("Disk 1 FAT12 解析失败: {error}"))?;
    let volumes = archive
        .volumes()
        .iter()
        .filter(|volume| volume.disk_index == 0 && volume.filesystem.is_some())
        .collect::<Vec<_>>();
    if volumes.len() != 1 {
        return Err(format!(
            "Disk 1 需要唯一受支持的文件系统卷，实际发现 {} 个",
            volumes.len()
        ));
    }
    let volume = volumes[0];
    let filesystem = volume
        .filesystem
        .as_ref()
        .ok_or("Disk 1 FAT12 文件系统缺失")?;
    if filesystem.kind != "FAT12" {
        return Err(format!(
            "Disk 1 文件系统必须为 FAT12，实际为 {}",
            filesystem.kind
        ));
    }
    let mut files = filesystem
        .files
        .iter()
        .map(|entry| {
            let data = archive
                .read_file(&volume.id, &entry.id)
                .map_err(|error| format!("读取 {} 失败: {error}", entry.display_path))?;
            let path = normalized_file_path(entry)?;
            Ok(SourceFile { path, data })
        })
        .collect::<Result<Vec<_>>>()?;
    files.sort_by(|left, right| {
        left.path
            .to_ascii_lowercase()
            .cmp(&right.path.to_ascii_lowercase())
    });
    Ok(files)
}

fn normalized_file_path(entry: &FileEntry) -> Result<String> {
    let path = entry.export_path.replace('\\', "/");
    if path.is_empty()
        || Path::new(&path)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(format!("fivec-new 导出路径不安全: {:?}", entry.export_path));
    }
    Ok(path)
}

fn validate_disk_sources(disks: &[DiskInput; 5], expected: &[DiskRecord]) -> Result<()> {
    if expected.len() != 5 {
        return Err(format!(
            "workspace.json 记录了 {} 张源盘，而不是五张",
            expected.len()
        ));
    }
    for (index, (disk, record)) in disks.iter().zip(expected).enumerate() {
        if record.disk_number != index as u8 + 1
            || record.sha256 != disk.sha256
            || record.size != disk.bytes.len()
        {
            return Err(format!(
                "第 {} 张原盘与 workspace.json 中的来源不匹配",
                index + 1
            ));
        }
    }
    Ok(())
}

fn validate_source_files(actual: &[SourceFile], expected: &[SourceFileRecord]) -> Result<()> {
    if actual.len() != expected.len() {
        return Err(format!(
            "Disk 1 文件数与 workspace.json 不匹配 ({} != {})",
            actual.len(),
            expected.len()
        ));
    }
    let mut actual_by_path = actual
        .iter()
        .map(|file| (file.path.to_ascii_lowercase(), file))
        .collect::<BTreeMap<_, _>>();
    for record in expected {
        if record.disk_number != 1 {
            return Err(format!(
                "workspace.json 含不支持的源文件盘号 {}",
                record.disk_number
            ));
        }
        let file = actual_by_path
            .remove(&record.path.to_ascii_lowercase())
            .ok_or_else(|| format!("Disk 1 缺少源文件 {}", record.path))?;
        if file.data.len() != record.size || sha256(&file.data) != record.sha256 {
            return Err(format!(
                "Disk 1 源文件与 workspace.json 不匹配: {}",
                record.path
            ));
        }
    }
    Ok(())
}

fn load_json_override<T: for<'de> Deserialize<'de>>(
    translations: &TranslationInputs,
    filename: &str,
) -> Result<Option<T>> {
    let Some(bytes) = translations.files.get(&filename.to_ascii_lowercase()) else {
        return Ok(None);
    };
    let document = serde_json::from_slice(bytes)
        .map_err(|error| format!("翻译 JSON {filename} 格式错误: {error}"))?;
    Ok(Some(document))
}

fn reject_unknown_translations(
    translations: &TranslationInputs,
    expected: &BTreeSet<String>,
) -> Result<()> {
    if let Some(name) = translations
        .files
        .keys()
        .find(|name| !expected.contains(*name))
    {
        return Err(format!("翻译目录中有不属于当前五张原盘的 JSON: {name}"));
    }
    Ok(())
}

fn translation_filename(source_file: &str) -> Result<String> {
    if source_file.is_empty()
        || source_file.contains('/')
        || source_file.contains('\\')
        || Path::new(source_file).components().count() != 1
        || !source_file.is_ascii()
    {
        return Err(format!(
            "翻译来源文件名必须是单个 ASCII 文件名: {source_file:?}"
        ));
    }
    Ok(format!("{source_file}.json"))
}

fn disk_records(disks: &[DiskInput; 5]) -> Vec<DiskRecord> {
    disks
        .iter()
        .enumerate()
        .map(|(index, disk)| DiskRecord {
            disk_number: index as u8 + 1,
            source_name: disk.source_name.clone(),
            size: disk.bytes.len(),
            sha256: disk.sha256.clone(),
        })
        .collect()
}

fn json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|error| format!("序列化 JSON 失败: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn safe_relative(raw: &str) -> Result<PathBuf> {
    let normalized = raw.replace('\\', "/");
    let path = Path::new(&normalized);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(format!("非法输出相对路径: {raw}"));
    }
    Ok(path.to_path_buf())
}

fn protect_output(output: &Path, inputs: &[&Path]) -> Result<()> {
    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = fs::canonicalize(parent)
        .map_err(|error| format!("解析输出父目录 {} 失败: {error}", parent.display()))?;
    let target = parent.join(output.file_name().ok_or("输出路径缺少目录名")?);
    if target.exists() {
        return Err(format!("输出已存在: {}", target.display()));
    }
    for input in inputs {
        let source = fs::canonicalize(input)
            .map_err(|error| format!("解析输入 {} 失败: {error}", input.display()))?;
        if target == source || (source.is_dir() && target.starts_with(&source)) {
            return Err(format!("输出与输入范围重叠: {}", input.display()));
        }
    }
    Ok(())
}

fn decode_single_disk(raw: &[u8], disk_number: usize) -> Result<()> {
    let image = StandardCodec
        .decode(raw)
        .map_err(|error| format!("第 {disk_number} 张软碟 D88 解析失败: {error}"))?;
    if image.disks.len() != 1 {
        return Err(format!(
            "第 {disk_number} 张软碟应含一个 D88 磁盘，实际为 {} 个",
            image.disks.len()
        ));
    }
    Ok(())
}
