//! Composed translation export/import workflows.
use crate::mes::{self, ScriptJson};
use crate::{
    commit_staging, discover_d88_inputs, join_manifest_path, rebuild_d88, sha256_hex,
    unique_sibling, WorkspaceManifest, WORKSPACE_FORMAT,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Component, Path, PathBuf};
use vn_font::font_98::{self, EncodingPlan, SubstitutionMap};

pub const TRANSLATION_FORMAT: &str = "shinjuku-monogatari-translation-workspace-v1";
pub const BUILD_FORMAT: &str = "shinjuku-monogatari-localized-build-v1";
pub const FONT_FACE: &str = "新宋体";

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TranslationScript {
    file: String,
    json: String,
    source_sha256: String,
    entries: usize,
    warnings: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TranslationManifest {
    #[serde(rename = "_format")]
    format: String,
    tool_version: String,
    source_workspace_sha256: String,
    scripts: Vec<TranslationScript>,
}

#[derive(Debug, Clone, Serialize)]
struct FontManifest {
    #[serde(rename = "_format")]
    format: String,
    file: String,
    face: String,
    source_font_sha256: String,
    output_font_sha256: String,
    patched_glyphs: usize,
    reserved_source_slots: usize,
    mappings: Vec<font_98::EncodingPlanEntry>,
}

#[derive(Debug, Clone, Serialize)]
struct BuildManifest {
    #[serde(rename = "_format")]
    format: String,
    tool_version: String,
    source_workspace_sha256: String,
    translation_workspace_sha256: String,
    disks: Vec<String>,
    scripts: usize,
    entries: usize,
    changed_entries: usize,
    changed_scripts: usize,
    font_file: String,
    font_mapping_file: String,
    font_patched_glyphs: usize,
}

#[derive(Debug, Clone)]
struct Artifact {
    relative_path: String,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PreparedTranslationExport {
    artifacts: Vec<Artifact>,
    output: PathBuf,
    overwrite: bool,
    scripts: usize,
    entries: usize,
    warnings: usize,
}

#[derive(Debug, Clone)]
pub struct PreparedLocalizedBuild {
    artifacts: Vec<Artifact>,
    output: PathBuf,
    overwrite: bool,
    disks: usize,
    scripts: usize,
    entries: usize,
    changed_entries: usize,
    changed_scripts: usize,
    patched_glyphs: usize,
}

#[derive(Debug, Clone)]
pub struct WorkflowReport {
    pub output_root: PathBuf,
    pub disks: usize,
    pub scripts: usize,
    pub entries: usize,
    pub changed_entries: usize,
    pub changed_scripts: usize,
    pub patched_glyphs: usize,
    pub warnings: Vec<String>,
}

impl PreparedTranslationExport {
    pub fn output(&self) -> &Path {
        &self.output
    }

    pub fn scripts(&self) -> usize {
        self.scripts
    }

    pub fn entries(&self) -> usize {
        self.entries
    }

    pub fn warnings(&self) -> usize {
        self.warnings
    }

    pub fn write(self) -> Result<WorkflowReport> {
        write_artifacts(
            &self.output,
            self.overwrite,
            "translation-workspace.json",
            TRANSLATION_FORMAT,
            &self.artifacts,
        )?;
        Ok(WorkflowReport {
            output_root: self.output,
            disks: 0,
            scripts: self.scripts,
            entries: self.entries,
            changed_entries: 0,
            changed_scripts: 0,
            patched_glyphs: 0,
            warnings: (self.warnings != 0)
                .then(|| format!("{} 个脚本词法警告已记录在对应 JSON", self.warnings))
                .into_iter()
                .collect(),
        })
    }
}

impl PreparedLocalizedBuild {
    pub fn output(&self) -> &Path {
        &self.output
    }

    pub fn disks(&self) -> usize {
        self.disks
    }

    pub fn scripts(&self) -> usize {
        self.scripts
    }

    pub fn entries(&self) -> usize {
        self.entries
    }

    pub fn changed_entries(&self) -> usize {
        self.changed_entries
    }

    pub fn patched_glyphs(&self) -> usize {
        self.patched_glyphs
    }

    pub fn write(self) -> Result<WorkflowReport> {
        write_artifacts(
            &self.output,
            self.overwrite,
            "build-report.json",
            BUILD_FORMAT,
            &self.artifacts,
        )?;
        Ok(WorkflowReport {
            output_root: self.output,
            disks: self.disks,
            scripts: self.scripts,
            entries: self.entries,
            changed_entries: self.changed_entries,
            changed_scripts: self.changed_scripts,
            patched_glyphs: self.patched_glyphs,
            warnings: Vec::new(),
        })
    }
}

pub fn prepare_translation_export(
    unpack_root: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<PreparedTranslationExport> {
    validate_separate_output(output, &[unpack_root])?;
    validate_managed_output(
        output,
        overwrite,
        "translation-workspace.json",
        TRANSLATION_FORMAT,
    )?;
    let workspace_bytes = fs::read(unpack_root.join("workspace.json"))
        .map_err(|error| format!("无法读取解包 workspace.json: {error}"))?;
    let workspace: WorkspaceManifest = serde_json::from_slice(&workspace_bytes)
        .map_err(|error| format!("解包 workspace.json 无效: {error}"))?;
    if workspace._format != WORKSPACE_FORMAT {
        return Err(format!("解包工作区格式不匹配: {}", workspace._format));
    }

    let mut artifacts = Vec::new();
    let mut scripts = Vec::new();
    let mut entries = 0;
    let mut warnings = 0;
    for disk in &workspace.disks {
        for file in &disk.files {
            if !file.path.to_ascii_uppercase().ends_with(".MES") {
                continue;
            }
            let source_key = format!("{}/{}", disk.output_dir, file.path);
            let source_path = join_manifest_path(unpack_root, &source_key)?;
            let bytes = fs::read(&source_path)
                .map_err(|error| format!("无法读取 {}: {error}", source_path.display()))?;
            if sha256_hex(&bytes) != file.sha256 {
                return Err(format!("已解包源文件被修改或损坏: {source_key}"));
            }
            let script = mes::parse_script(&bytes, source_key.clone())?;
            entries += script.entries.len();
            warnings += script.warnings.len();
            let json_path = format!("{source_key}.json");
            artifacts.push(Artifact {
                relative_path: json_path.clone(),
                bytes: pretty_json(&script)?,
            });
            scripts.push(TranslationScript {
                file: source_key,
                json: json_path,
                source_sha256: file.sha256.clone(),
                entries: script.entries.len(),
                warnings: script.warnings.len(),
            });
        }
    }
    scripts.sort_by(|left, right| left.file.cmp(&right.file));
    if scripts.is_empty() {
        return Err("解包工作区中没有 .MES 脚本".to_owned());
    }
    let manifest = TranslationManifest {
        format: TRANSLATION_FORMAT.to_owned(),
        tool_version: env!("CARGO_PKG_VERSION").to_owned(),
        source_workspace_sha256: sha256_hex(&workspace_bytes),
        scripts,
    };
    artifacts.push(Artifact {
        relative_path: "translation-workspace.json".to_owned(),
        bytes: pretty_json(&manifest)?,
    });
    Ok(PreparedTranslationExport {
        artifacts,
        output: output.to_path_buf(),
        overwrite,
        scripts: manifest.scripts.len(),
        entries,
        warnings,
    })
}

pub fn prepare_localized_build(
    original_d88: &[PathBuf],
    unpack_root: &Path,
    translation_root: &Path,
    output: &Path,
    overwrite: bool,
    font_face: &str,
) -> Result<PreparedLocalizedBuild> {
    validate_separate_output(output, &[unpack_root, translation_root])?;
    validate_managed_output(output, overwrite, "build-report.json", BUILD_FORMAT)?;
    if font_face.trim().is_empty() || font_face.contains('\0') {
        return Err("字库字体名称不能为空或包含 NUL".to_owned());
    }

    let workspace_bytes = fs::read(unpack_root.join("workspace.json"))
        .map_err(|error| format!("无法读取解包 workspace.json: {error}"))?;
    let workspace: WorkspaceManifest = serde_json::from_slice(&workspace_bytes)
        .map_err(|error| format!("解包 workspace.json 无效: {error}"))?;
    if workspace._format != WORKSPACE_FORMAT {
        return Err(format!("解包工作区格式不匹配: {}", workspace._format));
    }
    let translation_manifest_bytes = fs::read(translation_root.join("translation-workspace.json"))
        .map_err(|error| format!("无法读取 translation-workspace.json: {error}"))?;
    let translation_manifest: TranslationManifest =
        serde_json::from_slice(&translation_manifest_bytes)
            .map_err(|error| format!("translation-workspace.json 无效: {error}"))?;
    if translation_manifest.format != TRANSLATION_FORMAT {
        return Err(format!(
            "翻译工作区格式不匹配: {}",
            translation_manifest.format
        ));
    }
    if translation_manifest.source_workspace_sha256 != sha256_hex(&workspace_bytes) {
        return Err("翻译工作区不属于所选解包工作区".to_owned());
    }

    let mut loaded = Vec::new();
    let mut final_texts = Vec::new();
    let mut reserved = mes::reserved_control_cp932().collect::<BTreeSet<_>>();
    let mut entries = 0;
    let mut changed_entries = 0;
    let mut changed_scripts = 0;
    for item in &translation_manifest.scripts {
        let json_path = join_manifest_path(translation_root, &item.json)?;
        let script_bytes = fs::read(&json_path)
            .map_err(|error| format!("无法读取 {}: {error}", json_path.display()))?;
        let script: ScriptJson = serde_json::from_slice(&script_bytes)
            .map_err(|error| format!("{} 不是有效 UTF-8 MES JSON: {error}", json_path.display()))?;
        if script.file != item.file || script.source_sha256 != item.source_sha256 {
            return Err(format!("翻译清单与脚本 JSON 不匹配: {}", item.file));
        }
        let source_path = join_manifest_path(unpack_root, &item.file)?;
        let source = fs::read(&source_path)
            .map_err(|error| format!("无法读取 {}: {error}", source_path.display()))?;
        mes::validate_script(&script, &source)
            .map_err(|error| format!("{}: {error}", item.file))?;
        entries += script.entries.len();
        let changed = script
            .entries
            .iter()
            .filter(|entry| entry.name != entry.scr_name || entry.message != entry.scr_msg)
            .count();
        changed_entries += changed;
        changed_scripts += usize::from(changed != 0);
        final_texts.extend(mes::final_display_texts(&script)?);
        reserved.extend(mes::source_reserved_cp932(&script)?);
        loaded.push((script, source));
    }
    if loaded.len() != translation_manifest.scripts.len() {
        return Err("翻译脚本清单不完整".to_owned());
    }

    let substitutions = SubstitutionMap::embedded()?;
    let plan = EncodingPlan::build(
        &substitutions,
        reserved.iter().copied(),
        final_texts.iter().map(String::as_str),
    )?;
    let requests = plan.requests();
    let built_font =
        font_98::prepare_font(font_98::EMBEDDED_FONT, &requests, &reserved, font_face)?;

    let disk_names = workspace
        .disks
        .iter()
        .map(|disk| disk.output_dir.to_uppercase())
        .collect::<BTreeSet<_>>();
    let mut replacements = BTreeMap::<String, BTreeMap<String, Vec<u8>>>::new();
    for (script, source) in &loaded {
        let (disk, relative) = script
            .file
            .split_once('/')
            .ok_or_else(|| format!("脚本路径缺少磁盘目录: {}", script.file))?;
        if !disk_names.contains(&disk.to_uppercase()) {
            return Err(format!("脚本路径使用未知磁盘目录: {}", script.file));
        }
        let rebuilt = mes::rebuild_script(script, source, &plan)
            .map_err(|error| format!("{}: {error}", script.file))?;
        replacements
            .entry(disk.to_uppercase())
            .or_default()
            .insert(relative.to_owned(), rebuilt);
    }

    let input_paths = discover_d88_inputs(original_d88)?;
    let mut input_by_name = HashMap::new();
    for path in input_paths {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("D88 文件名无法表示为 Unicode: {}", path.display()))?;
        if input_by_name
            .insert(name.to_uppercase(), path.clone())
            .is_some()
        {
            return Err(format!("重复 D88 文件名: {name}"));
        }
    }

    let mut artifacts = Vec::new();
    let mut output_disks = Vec::new();
    for disk in &workspace.disks {
        let source_path = input_by_name
            .get(&disk.source_file.to_uppercase())
            .ok_or_else(|| format!("缺少原始 D88: {}", disk.source_file))?;
        let source = fs::read(source_path)
            .map_err(|error| format!("无法读取 {}: {error}", source_path.display()))?;
        if sha256_hex(&source) != disk.source_sha256 {
            return Err(format!("原始 D88 哈希不匹配: {}", disk.source_file));
        }
        let disk_replacements = replacements
            .remove(&disk.output_dir.to_uppercase())
            .unwrap_or_default();
        let rebuilt = rebuild_d88(source_path, &source, &disk_replacements)?;
        let relative = format!("d88/{}", disk.source_file);
        output_disks.push(relative.clone());
        artifacts.push(Artifact {
            relative_path: relative,
            bytes: rebuilt,
        });
    }
    if !replacements.is_empty() {
        return Err("存在无法映射到原始 D88 的脚本替换".to_owned());
    }

    let font_manifest = FontManifest {
        format: "shinjuku-monogatari-np2-font-map-v1".to_owned(),
        file: "font.bmp".to_owned(),
        face: font_face.to_owned(),
        source_font_sha256: font_98::embedded_font_sha256().to_uppercase(),
        output_font_sha256: sha256_hex(&built_font.bytes),
        patched_glyphs: built_font.patched_glyphs,
        reserved_source_slots: reserved.len(),
        mappings: plan.manifest_entries()?,
    };
    artifacts.push(Artifact {
        relative_path: "font.bmp".to_owned(),
        bytes: built_font.bytes,
    });
    artifacts.push(Artifact {
        relative_path: "font_mapping.json".to_owned(),
        bytes: pretty_json(&font_manifest)?,
    });
    let build_manifest = BuildManifest {
        format: BUILD_FORMAT.to_owned(),
        tool_version: env!("CARGO_PKG_VERSION").to_owned(),
        source_workspace_sha256: sha256_hex(&workspace_bytes),
        translation_workspace_sha256: sha256_hex(&translation_manifest_bytes),
        disks: output_disks,
        scripts: loaded.len(),
        entries,
        changed_entries,
        changed_scripts,
        font_file: "font.bmp".to_owned(),
        font_mapping_file: "font_mapping.json".to_owned(),
        font_patched_glyphs: built_font.patched_glyphs,
    };
    artifacts.push(Artifact {
        relative_path: "build-report.json".to_owned(),
        bytes: pretty_json(&build_manifest)?,
    });

    Ok(PreparedLocalizedBuild {
        artifacts,
        output: output.to_path_buf(),
        overwrite,
        disks: workspace.disks.len(),
        scripts: loaded.len(),
        entries,
        changed_entries,
        changed_scripts,
        patched_glyphs: built_font.patched_glyphs,
    })
}

fn pretty_json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|error| format!("JSON 序列化失败: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn validate_separate_output(output: &Path, inputs: &[&Path]) -> Result<()> {
    let output = std::path::absolute(output)
        .map_err(|error| format!("无法解析输出目录 {}: {error}", output.display()))?;
    for input in inputs {
        let input = fs::canonicalize(input)
            .map_err(|error| format!("无法解析输入目录 {}: {error}", input.display()))?;
        if output == input || output.starts_with(&input) || input.starts_with(&output) {
            return Err(format!(
                "输入与输出目录必须互相独立: {} / {}",
                input.display(),
                output.display()
            ));
        }
    }
    Ok(())
}

fn validate_managed_output(
    output: &Path,
    overwrite: bool,
    marker: &str,
    expected_format: &str,
) -> Result<()> {
    if output.as_os_str().is_empty()
        || output
            .components()
            .all(|part| !matches!(part, Component::Normal(_)))
    {
        return Err("拒绝把文件系统根目录作为输出".to_owned());
    }
    if output
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return Err("输出目录不能包含 ..".to_owned());
    }
    if !output.exists() {
        return Ok(());
    }
    let metadata =
        fs::symlink_metadata(output).map_err(|error| format!("无法读取输出目录元数据: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("输出路径必须是普通目录".to_owned());
    }
    if !overwrite {
        return Err(format!("输出目录已存在；默认不覆盖: {}", output.display()));
    }
    if fs::read_dir(output)
        .map_err(|error| format!("无法读取输出目录: {error}"))?
        .next()
        .is_none()
    {
        return Ok(());
    }
    let bytes =
        fs::read(output.join(marker)).map_err(|_| format!("拒绝覆盖不含 {marker} 的非空目录"))?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| format!("现有 {marker} 无效，拒绝覆盖"))?;
    if value.get("_format").and_then(|item| item.as_str()) != Some(expected_format) {
        return Err(format!(
            "现有输出不是本操作生成的 {expected_format}，拒绝覆盖"
        ));
    }
    Ok(())
}

fn write_artifacts(
    output: &Path,
    overwrite: bool,
    marker: &str,
    format: &str,
    artifacts: &[Artifact],
) -> Result<()> {
    validate_managed_output(output, overwrite, marker, format)?;
    let parent = output
        .parent()
        .ok_or_else(|| "输出目录缺少父目录".to_owned())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("无法创建输出父目录 {}: {error}", parent.display()))?;
    let name = output
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "输出目录名无法表示为 Unicode".to_owned())?;
    let staging = unique_sibling(parent, &format!(".{name}.tmp-{}", std::process::id()))?;
    fs::create_dir(&staging).map_err(|error| format!("无法创建临时输出: {error}"))?;
    let result = (|| -> Result<()> {
        for artifact in artifacts {
            let target = join_manifest_path(&staging, &artifact.relative_path)?;
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("无法创建 {}: {error}", parent.display()))?;
            }
            fs::write(&target, &artifact.bytes)
                .map_err(|error| format!("无法写入 {}: {error}", target.display()))?;
        }
        commit_staging(&staging, output, overwrite)
    })();
    if result.is_err() && staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}
