//! Translation export/import, font planning, and localized D88 build workflow.

use crate::{
    bun::{self, BunJson},
    commit_staging, discover_inputs, rebuild_d88, safe_join, sha256_hex, unique_sibling,
};
use crate::{
    mes::{self, MesJson},
    WORKSPACE_FORMAT,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Component, Path, PathBuf};
use vn_font::font_98::{self, EncodingPlan, SubstitutionMap};

pub const TRANSLATION_FORMAT: &str = "misty-blue-translation-workspace-v1";
pub const BUILD_FORMAT: &str = "misty-blue-localized-build-v1";
pub const FONT_FACE: &str = font_98::FONT_FACE;

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Deserialize)]
struct Workspace {
    #[serde(rename = "_format")]
    format: String,
    disks: Vec<WorkspaceDisk>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceDisk {
    source_file: String,
    output_directory: String,
}

#[derive(Debug, Deserialize)]
struct DiskManifest {
    #[serde(rename = "_format")]
    format: String,
    source_file: String,
    source_sha256: String,
    files: Vec<DiskFile>,
}

#[derive(Debug, Deserialize)]
struct DiskFile {
    path: String,
    output_path: String,
    size: u64,
    sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TranslationItem {
    file: String,
    json: String,
    kind: String,
    source_sha256: String,
    entries: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct TranslationManifest {
    #[serde(rename = "_format")]
    format: String,
    tool_version: String,
    source_workspace_sha256: String,
    scripts: Vec<TranslationItem>,
}

#[derive(Debug, Serialize)]
struct FontManifest {
    #[serde(rename = "_format")]
    format: String,
    file: String,
    face: String,
    source_font_sha256: String,
    output_font_sha256: String,
    patched_glyphs: usize,
    reserved_slots: usize,
    forbidden_game_slots: usize,
    mappings: Vec<font_98::EncodingPlanEntry>,
}

#[derive(Debug, Serialize)]
struct BuildManifest {
    #[serde(rename = "_format")]
    format: String,
    tool_version: String,
    source_workspace_sha256: String,
    translation_workspace_sha256: String,
    disks: Vec<String>,
    bun_files: usize,
    mes_files: usize,
    entries: usize,
    changed_entries: usize,
    changed_files: usize,
    font_file: String,
    font_mapping_file: String,
    font_patched_glyphs: usize,
}

#[derive(Debug, Clone)]
struct Artifact {
    relative_path: PathBuf,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PreparedTranslationExport {
    artifacts: Vec<Artifact>,
    output: PathBuf,
    overwrite: bool,
    bun_files: usize,
    mes_files: usize,
    entries: usize,
}

#[derive(Debug, Clone)]
pub struct PreparedLocalizedBuild {
    artifacts: Vec<Artifact>,
    output: PathBuf,
    overwrite: bool,
    disks: usize,
    bun_files: usize,
    mes_files: usize,
    entries: usize,
    changed_entries: usize,
    changed_files: usize,
    patched_glyphs: usize,
}

#[derive(Debug, Clone)]
pub struct WorkflowReport {
    pub output_root: PathBuf,
    pub disks: usize,
    pub bun_files: usize,
    pub mes_files: usize,
    pub entries: usize,
    pub changed_entries: usize,
    pub changed_files: usize,
    pub patched_glyphs: usize,
}

impl PreparedTranslationExport {
    pub fn bun_files(&self) -> usize {
        self.bun_files
    }
    pub fn mes_files(&self) -> usize {
        self.mes_files
    }
    pub fn entries(&self) -> usize {
        self.entries
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
            bun_files: self.bun_files,
            mes_files: self.mes_files,
            entries: self.entries,
            changed_entries: 0,
            changed_files: 0,
            patched_glyphs: 0,
        })
    }
}

impl PreparedLocalizedBuild {
    pub fn disks(&self) -> usize {
        self.disks
    }
    pub fn bun_files(&self) -> usize {
        self.bun_files
    }
    pub fn mes_files(&self) -> usize {
        self.mes_files
    }
    pub fn entries(&self) -> usize {
        self.entries
    }
    pub fn changed_entries(&self) -> usize {
        self.changed_entries
    }
    pub fn changed_files(&self) -> usize {
        self.changed_files
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
            bun_files: self.bun_files,
            mes_files: self.mes_files,
            entries: self.entries,
            changed_entries: self.changed_entries,
            changed_files: self.changed_files,
            patched_glyphs: self.patched_glyphs,
        })
    }
}

fn read_workspace(unpack_root: &Path) -> Result<(Vec<u8>, Workspace)> {
    let bytes = fs::read(unpack_root.join("workspace.json"))
        .map_err(|error| format!("读取解包 workspace.json 失败: {error}"))?;
    let workspace: Workspace = serde_json::from_slice(&bytes)
        .map_err(|error| format!("解包 workspace.json 无效: {error}"))?;
    if workspace.format != WORKSPACE_FORMAT {
        return Err(format!("解包工作区格式不匹配: {}", workspace.format));
    }
    Ok((bytes, workspace))
}

fn read_disk_manifest(unpack_root: &Path, disk: &WorkspaceDisk) -> Result<DiskManifest> {
    let path = unpack_root
        .join(&disk.output_directory)
        .join("manifest.json");
    let bytes =
        fs::read(&path).map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
    let manifest: DiskManifest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{} 无效: {error}", path.display()))?;
    if manifest.format != WORKSPACE_FORMAT
        || !manifest.source_file.eq_ignore_ascii_case(&disk.source_file)
    {
        return Err(format!("{} 与 workspace.json 不匹配", path.display()));
    }
    Ok(manifest)
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
    let (workspace_bytes, workspace) = read_workspace(unpack_root)?;
    let mut artifacts = Vec::new();
    let mut scripts = Vec::new();
    let mut bun_files = 0usize;
    let mut mes_files = 0usize;
    let mut entries = 0usize;
    let mut flat_json_names = BTreeSet::new();

    for disk in &workspace.disks {
        let manifest = read_disk_manifest(unpack_root, disk)?;
        for file in &manifest.files {
            let upper = file.path.to_ascii_uppercase();
            let kind = if upper.ends_with(".BUN") {
                "BUN"
            } else if upper.ends_with(".MES") {
                "MES"
            } else {
                continue;
            };
            let source_key = format!(
                "{}/{}",
                disk.output_directory,
                file.output_path.replace('\\', "/")
            );
            let source_path = safe_join(unpack_root, Path::new(&source_key))?;
            let source = fs::read(&source_path)
                .map_err(|error| format!("读取 {} 失败: {error}", source_path.display()))?;
            if source.len() as u64 != file.size
                || !sha256_hex(&source).eq_ignore_ascii_case(&file.sha256)
            {
                return Err(format!("已解包源文件被修改或损坏: {source_key}"));
            }
            let (json, count, source_sha256) = if kind == "BUN" {
                let script = bun::parse_bun(&source, source_key.clone())?;
                let count = bun::entry_count(&script);
                bun_files += 1;
                (pretty_json(&script)?, count, script.source_sha256.clone())
            } else {
                let script = mes::parse_mes(&source, source_key.clone())?;
                let count = script.entries.len();
                mes_files += 1;
                (pretty_json(&script)?, count, script.source_sha256.clone())
            };
            entries += count;
            let resource_name = file.path.replace(['/', '\\'], "__");
            let json_path = format!("{}__{resource_name}.json", disk.output_directory);
            if !flat_json_names.insert(json_path.to_ascii_uppercase()) {
                return Err(format!("扁平翻译 JSON 文件名冲突: {json_path}"));
            }
            artifacts.push(Artifact {
                relative_path: PathBuf::from(&json_path),
                bytes: json,
            });
            scripts.push(TranslationItem {
                file: source_key,
                json: json_path,
                kind: kind.to_owned(),
                source_sha256,
                entries: count,
            });
        }
    }
    scripts.sort_by(|left, right| left.file.cmp(&right.file));
    if scripts.is_empty() {
        return Err("解包工作区中没有 .BUN 或 .MES 文本资源".to_owned());
    }
    let manifest = TranslationManifest {
        format: TRANSLATION_FORMAT.to_owned(),
        tool_version: env!("CARGO_PKG_VERSION").to_owned(),
        source_workspace_sha256: sha256_hex(&workspace_bytes),
        scripts,
    };
    artifacts.push(Artifact {
        relative_path: PathBuf::from("translation-workspace.json"),
        bytes: pretty_json(&manifest)?,
    });
    artifacts.push(Artifact {
        relative_path: PathBuf::from("README.txt"),
        bytes: "Misty Blue 翻译工作区\r\n\r\n所有待翻译 JSON 均在本目录，文件名以原盘目录为前缀防止重名。\r\n只编辑各 JSON entries/records 中的 message。scr_msg 与所有下划线字段是源结构校验值。\r\nBUN 的 LF 表示原生 6C 换行；MES 不允许自行插入换行，箭头控制符保留在 token 流中。\r\n"
            .as_bytes()
            .to_vec(),
    });
    Ok(PreparedTranslationExport {
        artifacts,
        output: output.to_path_buf(),
        overwrite,
        bun_files,
        mes_files,
        entries,
    })
}

enum LoadedScript {
    Bun(BunJson, Vec<u8>),
    Mes(MesJson, Vec<u8>),
}

fn incompatible_game_slots() -> BTreeSet<u16> {
    let mut forbidden = BTreeSet::new();
    let leads = (0x81u8..=0x9f).chain(0xe0u8..=0xef);
    for lead in leads {
        for trail in (0x40u8..=0x7e).chain(0x80u8..=0xfc) {
            let pair = [lead, trail];
            let Ok(jis) = font_98::cp932_to_jis(pair) else {
                continue;
            };
            let page = jis[0] - 0x20;
            let loaded = (0x01..=0x55).contains(&page) || (0x58..=0x5f).contains(&page);
            if loaded && !bun::compatible_pair(pair) {
                forbidden.insert(u16::from_be_bytes(pair));
            }
        }
    }
    forbidden
}

fn validate_plan_texts(plan: &EncodingPlan, texts: &[String]) -> Result<()> {
    for text in texts {
        for character in text.chars() {
            let bytes = plan.encode_cp932(&character.to_string())?;
            let pair: [u8; 2] = bytes
                .as_slice()
                .try_into()
                .map_err(|_| format!("字符 {character:?} 没有单一双字节载体"))?;
            if !bun::compatible_pair(pair) {
                return Err(format!(
                    "字符 {character:?} 的原生/代换载体 {:02X}{:02X} 与游戏控制码冲突",
                    pair[0], pair[1]
                ));
            }
        }
    }
    Ok(())
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
    let (workspace_bytes, workspace) = read_workspace(unpack_root)?;
    let translation_bytes = fs::read(translation_root.join("translation-workspace.json"))
        .map_err(|error| format!("读取 translation-workspace.json 失败: {error}"))?;
    let translation: TranslationManifest = serde_json::from_slice(&translation_bytes)
        .map_err(|error| format!("translation-workspace.json 无效: {error}"))?;
    if translation.format != TRANSLATION_FORMAT {
        return Err(format!("翻译工作区格式不匹配: {}", translation.format));
    }
    if !translation
        .source_workspace_sha256
        .eq_ignore_ascii_case(&sha256_hex(&workspace_bytes))
    {
        return Err("翻译工作区不属于所选解包工作区".to_owned());
    }

    let mut loaded = Vec::new();
    let mut final_texts = Vec::new();
    let mut reserved = mes::reserved_control_cp932().collect::<BTreeSet<_>>();
    let forbidden = incompatible_game_slots();
    let mut bun_files = 0usize;
    let mut mes_files = 0usize;
    let mut entries = 0usize;
    let mut changed_entries = 0usize;
    let mut changed_files = 0usize;

    for item in &translation.scripts {
        let json_path = safe_join(translation_root, Path::new(&item.json))?;
        let json = fs::read(&json_path)
            .map_err(|error| format!("读取 {} 失败: {error}", json_path.display()))?;
        let source_path = safe_join(unpack_root, Path::new(&item.file))?;
        let source = fs::read(&source_path)
            .map_err(|error| format!("读取 {} 失败: {error}", source_path.display()))?;
        match item.kind.as_str() {
            "BUN" => {
                let script: BunJson = serde_json::from_slice(&json).map_err(|error| {
                    format!("{} 不是有效 BUN JSON: {error}", json_path.display())
                })?;
                if script.file != item.file
                    || !script
                        .source_sha256
                        .eq_ignore_ascii_case(&item.source_sha256)
                {
                    return Err(format!("翻译清单与 BUN JSON 不匹配: {}", item.file));
                }
                bun::validate_bun(&script, &source)
                    .map_err(|error| format!("{}: {error}", item.file))?;
                let changed = bun::changed_entries(&script);
                entries += bun::entry_count(&script);
                changed_entries += changed;
                changed_files += usize::from(changed != 0);
                bun_files += 1;
                final_texts.extend(bun::final_display_texts(&script)?);
                reserved.extend(bun::source_reserved_cp932(&script)?);
                loaded.push(LoadedScript::Bun(script, source));
            }
            "MES" => {
                let script: MesJson = serde_json::from_slice(&json).map_err(|error| {
                    format!("{} 不是有效 MES JSON: {error}", json_path.display())
                })?;
                if script.file != item.file
                    || !script
                        .source_sha256
                        .eq_ignore_ascii_case(&item.source_sha256)
                {
                    return Err(format!("翻译清单与 MES JSON 不匹配: {}", item.file));
                }
                mes::validate_mes(&script, &source)
                    .map_err(|error| format!("{}: {error}", item.file))?;
                let changed = mes::changed_entries(&script);
                entries += script.entries.len();
                changed_entries += changed;
                changed_files += usize::from(changed != 0);
                mes_files += 1;
                final_texts.extend(mes::final_display_texts(&script)?);
                reserved.extend(mes::source_reserved_cp932(&script)?);
                loaded.push(LoadedScript::Mes(script, source));
            }
            other => return Err(format!("{} 使用未知脚本类型 {other}", item.file)),
        }
    }
    if loaded.len() != translation.scripts.len() {
        return Err("翻译脚本清单不完整".to_owned());
    }

    let substitutions = SubstitutionMap::embedded()?;
    let plan = EncodingPlan::build_with_forbidden_cp932(
        &substitutions,
        reserved.iter().copied(),
        forbidden.iter().copied(),
        final_texts.iter().map(String::as_str),
    )?;
    validate_plan_texts(&plan, &final_texts)?;
    let built_font = font_98::prepare_font(
        font_98::EMBEDDED_FONT,
        &plan.requests(),
        &reserved,
        font_face,
    )?;

    let disk_names = workspace
        .disks
        .iter()
        .map(|disk| disk.output_directory.to_ascii_uppercase())
        .collect::<BTreeSet<_>>();
    let mut replacements = BTreeMap::<String, BTreeMap<String, Vec<u8>>>::new();
    for script in &loaded {
        let (file, rebuilt) = match script {
            LoadedScript::Bun(script, source) => (
                script.file.as_str(),
                bun::rebuild_bun(script, source, &plan)?,
            ),
            LoadedScript::Mes(script, source) => (
                script.file.as_str(),
                mes::rebuild_mes(script, source, &plan)?,
            ),
        };
        let (disk, relative) = file
            .split_once('/')
            .ok_or_else(|| format!("脚本路径缺少磁盘目录: {file}"))?;
        let relative = relative
            .strip_prefix("files/")
            .ok_or_else(|| format!("脚本路径不在 files 下: {file}"))?;
        if !disk_names.contains(&disk.to_ascii_uppercase()) {
            return Err(format!("脚本路径使用未知磁盘目录: {file}"));
        }
        replacements
            .entry(disk.to_ascii_uppercase())
            .or_default()
            .insert(relative.to_owned(), rebuilt);
    }

    let mut input_by_name = HashMap::new();
    for path in discover_inputs(original_d88)? {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("D88 文件名无法表示为 Unicode: {}", path.display()))?;
        if input_by_name
            .insert(name.to_ascii_uppercase(), path.clone())
            .is_some()
        {
            return Err(format!("重复 D88 文件名: {name}"));
        }
    }

    let mut artifacts = Vec::new();
    let mut output_disks = Vec::new();
    for disk in &workspace.disks {
        let disk_manifest = read_disk_manifest(unpack_root, disk)?;
        let source_path = input_by_name
            .get(&disk.source_file.to_ascii_uppercase())
            .ok_or_else(|| format!("缺少原始 D88: {}", disk.source_file))?;
        let source = fs::read(source_path)
            .map_err(|error| format!("读取 {} 失败: {error}", source_path.display()))?;
        if !sha256_hex(&source).eq_ignore_ascii_case(&disk_manifest.source_sha256) {
            return Err(format!("原始 D88 哈希不匹配: {}", disk.source_file));
        }
        let disk_replacements = replacements
            .remove(&disk.output_directory.to_ascii_uppercase())
            .unwrap_or_default();
        let rebuilt = rebuild_d88(source_path, &source, &disk_replacements)?;
        let relative = PathBuf::from("d88").join(&disk.source_file);
        output_disks.push(relative.to_string_lossy().replace('\\', "/"));
        artifacts.push(Artifact {
            relative_path: relative,
            bytes: rebuilt,
        });
    }
    if !replacements.is_empty() {
        return Err("存在无法映射到原始 D88 的资源替换".to_owned());
    }

    let font_manifest = FontManifest {
        format: "misty-blue-np2-font-map-v1".to_owned(),
        file: "font.bmp".to_owned(),
        face: font_face.to_owned(),
        source_font_sha256: font_98::embedded_font_sha256().to_uppercase(),
        output_font_sha256: sha256_hex(&built_font.bytes).to_uppercase(),
        patched_glyphs: built_font.patched_glyphs,
        reserved_slots: reserved.len(),
        forbidden_game_slots: forbidden.len(),
        mappings: plan.manifest_entries()?,
    };
    artifacts.push(Artifact {
        relative_path: PathBuf::from("font.bmp"),
        bytes: built_font.bytes,
    });
    artifacts.push(Artifact {
        relative_path: PathBuf::from("font_mapping.json"),
        bytes: pretty_json(&font_manifest)?,
    });
    let build_manifest = BuildManifest {
        format: BUILD_FORMAT.to_owned(),
        tool_version: env!("CARGO_PKG_VERSION").to_owned(),
        source_workspace_sha256: sha256_hex(&workspace_bytes),
        translation_workspace_sha256: sha256_hex(&translation_bytes),
        disks: output_disks,
        bun_files,
        mes_files,
        entries,
        changed_entries,
        changed_files,
        font_file: "font.bmp".to_owned(),
        font_mapping_file: "font_mapping.json".to_owned(),
        font_patched_glyphs: built_font.patched_glyphs,
    };
    artifacts.push(Artifact {
        relative_path: PathBuf::from("build-report.json"),
        bytes: pretty_json(&build_manifest)?,
    });

    Ok(PreparedLocalizedBuild {
        artifacts,
        output: output.to_path_buf(),
        overwrite,
        disks: workspace.disks.len(),
        bun_files,
        mes_files,
        entries,
        changed_entries,
        changed_files,
        patched_glyphs: built_font.patched_glyphs,
    })
}

fn pretty_json(value: &impl Serialize) -> Result<Vec<u8>> {
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
        || output
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err("输出目录为空、是文件系统根目录或包含 ..".to_owned());
    }
    if !output.exists() {
        return Ok(());
    }
    let metadata =
        fs::symlink_metadata(output).map_err(|error| format!("读取输出目录元数据失败: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("输出路径必须是普通目录".to_owned());
    }
    if !overwrite {
        return Err(format!("输出目录已存在；默认不覆盖: {}", output.display()));
    }
    if fs::read_dir(output)
        .map_err(|error| format!("读取输出目录失败: {error}"))?
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
        .map_err(|error| format!("创建输出父目录 {} 失败: {error}", parent.display()))?;
    let staging = unique_sibling(output, "staging")?;
    fs::create_dir(&staging)
        .map_err(|error| format!("创建临时输出 {} 失败: {error}", staging.display()))?;
    let result = (|| -> Result<()> {
        for artifact in artifacts {
            let target = safe_join(&staging, &artifact.relative_path)?;
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("创建 {} 失败: {error}", parent.display()))?;
            }
            fs::write(&target, &artifact.bytes)
                .map_err(|error| format!("写入 {} 失败: {error}", target.display()))?;
        }
        commit_staging(&staging, output, overwrite)
    })();
    if result.is_err() && staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_plan_rebuilds_bun_and_mes_with_compatible_chinese_carriers() {
        let bun_source = vec![4, 0, 1, 0xff, 0xab, 0x02, 0, 0];
        let mut bun_script = bun::parse_bun(&bun_source, "A/files/T.BUN".into()).unwrap();
        bun_script.records[0].segments[0].message = "存档测试".into();

        let (mes_bytes, _, errors) = encoding_rs::SHIFT_JIS.encode("日本");
        assert!(!errors);
        let mes_source = mes_bytes.into_owned();
        let mut mes_script = mes::parse_mes(&mes_source, "B/files/T.MES".into()).unwrap();
        mes_script.entries[0].message = "文本".into();

        let mut reserved = mes::reserved_control_cp932().collect::<BTreeSet<_>>();
        let forbidden = incompatible_game_slots();
        reserved.extend(bun::source_reserved_cp932(&bun_script).unwrap());
        reserved.extend(mes::source_reserved_cp932(&mes_script).unwrap());
        let mut texts = bun::final_display_texts(&bun_script).unwrap();
        texts.extend(mes::final_display_texts(&mes_script).unwrap());
        let plan = EncodingPlan::build_with_forbidden_cp932(
            &SubstitutionMap::embedded().unwrap(),
            reserved,
            forbidden,
            texts.iter().map(String::as_str),
        )
        .unwrap();
        validate_plan_texts(&plan, &texts).unwrap();

        let rebuilt_bun = bun::rebuild_bun(&bun_script, &bun_source, &plan).unwrap();
        let reparsed_bun = bun::parse_bun(&rebuilt_bun, "A/files/T.BUN".into()).unwrap();
        assert_eq!(
            plan.decode_carriers(&reparsed_bun.records[0].segments[0].scr_msg),
            "存档测试"
        );

        let rebuilt_mes = mes::rebuild_mes(&mes_script, &mes_source, &plan).unwrap();
        let reparsed_mes = mes::parse_mes(&rebuilt_mes, "B/files/T.MES".into()).unwrap();
        assert_eq!(
            plan.decode_carriers(&reparsed_mes.entries[0].scr_msg),
            "文本"
        );
    }
}
