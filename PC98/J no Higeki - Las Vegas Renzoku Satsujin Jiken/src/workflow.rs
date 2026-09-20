//! Translation workspace, font planning, and complete localized D88 builds.

use crate::jack::{self, AdwJson, AtxJson, ExeJson};
use encoding_rs::SHIFT_JIS;
use misty_blue_d88_tool::{rebuild_d88, WORKSPACE_FORMAT};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Component, Path, PathBuf};
use vn_font::font_98::{self, EncodingPlan, SubstitutionMap};

pub const TRANSLATION_FORMAT: &str = "j-no-higeki-translation-workspace-v1";
pub const BUILD_FORMAT: &str = "j-no-higeki-localized-build-v1";
pub const FONT_FACE: &str = font_98::FONT_FACE;

type Result<T> = std::result::Result<T, String>;

const REQUIRED: [&str; 6] = [
    "JACK.EXE",
    "JACK_C.ATX",
    "JACK_C.ADW",
    "JACK_C.FLG",
    "JACK_C.FRM",
    "JACK_C.MSG",
];

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

#[derive(Debug, Clone, Deserialize)]
struct DiskFile {
    path: String,
    output_path: String,
    size: u64,
    sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TranslationItem {
    kind: String,
    file: String,
    json: String,
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
    mappings: Vec<font_98::EncodingPlanEntry>,
}

#[derive(Debug, Serialize)]
struct BuildInput {
    name: String,
    sha256: String,
}

#[derive(Debug, Serialize)]
struct BuildManifest {
    #[serde(rename = "_format")]
    format: String,
    tool_version: String,
    source_workspace_sha256: String,
    translation_inputs: Vec<BuildInput>,
    disks: Vec<String>,
    atx_entries: usize,
    adw_entries: usize,
    exe_entries: usize,
    changed_entries: usize,
    changed_resources: usize,
    font_file: String,
    font_mapping_file: String,
    font_patched_glyphs: usize,
}

#[derive(Debug, Clone)]
struct Artifact {
    relative_path: PathBuf,
    bytes: Vec<u8>,
}

#[derive(Debug)]
struct Resource {
    name: String,
    key: String,
    bytes: Vec<u8>,
}

#[derive(Debug)]
struct Sources {
    disk_directory: String,
    disk_source_file: String,
    exe: Resource,
    atx: Resource,
    adw: Resource,
    flg: Resource,
    frm: Resource,
    msg: Resource,
}

#[derive(Debug, Clone)]
pub struct PreparedTranslationExport {
    artifacts: Vec<Artifact>,
    output: PathBuf,
    overwrite: bool,
    atx_entries: usize,
    adw_entries: usize,
    exe_entries: usize,
}

#[derive(Debug, Clone)]
pub struct PreparedLocalizedBuild {
    artifacts: Vec<Artifact>,
    output: PathBuf,
    overwrite: bool,
    disks: usize,
    entries: usize,
    changed_entries: usize,
    changed_resources: usize,
    patched_glyphs: usize,
}

#[derive(Debug, Clone)]
pub struct WorkflowReport {
    pub output_root: PathBuf,
    pub disks: usize,
    pub entries: usize,
    pub changed_entries: usize,
    pub changed_resources: usize,
    pub patched_glyphs: usize,
}

impl PreparedTranslationExport {
    pub fn atx_entries(&self) -> usize {
        self.atx_entries
    }
    pub fn adw_entries(&self) -> usize {
        self.adw_entries
    }
    pub fn exe_entries(&self) -> usize {
        self.exe_entries
    }
    pub fn entries(&self) -> usize {
        self.atx_entries + self.adw_entries + self.exe_entries
    }
    pub fn write(self) -> Result<WorkflowReport> {
        let entries = self.entries();
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
            entries,
            changed_entries: 0,
            changed_resources: 0,
            patched_glyphs: 0,
        })
    }
}

impl PreparedLocalizedBuild {
    pub fn disks(&self) -> usize {
        self.disks
    }
    pub fn entries(&self) -> usize {
        self.entries
    }
    pub fn changed_entries(&self) -> usize {
        self.changed_entries
    }
    pub fn changed_resources(&self) -> usize {
        self.changed_resources
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
            entries: self.entries,
            changed_entries: self.changed_entries,
            changed_resources: self.changed_resources,
            patched_glyphs: self.patched_glyphs,
        })
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn pretty_json(value: &impl Serialize) -> Result<Vec<u8>> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|error| format!("JSON 序列化失败: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn safe_join(root: &Path, relative: &Path) -> Result<PathBuf> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(format!("不安全的相对路径: {}", relative.display()));
    }
    Ok(root.join(relative))
}

fn read_workspace(unpack_root: &Path) -> Result<(Vec<u8>, Workspace)> {
    let path = unpack_root.join("workspace.json");
    let bytes =
        fs::read(&path).map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
    let workspace: Workspace = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{} 无效: {error}", path.display()))?;
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

fn read_resource(unpack_root: &Path, disk: &WorkspaceDisk, file: &DiskFile) -> Result<Resource> {
    let key = format!(
        "{}/{}",
        disk.output_directory,
        file.output_path.replace('\\', "/")
    );
    let path = safe_join(unpack_root, Path::new(&key))?;
    let bytes =
        fs::read(&path).map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
    if bytes.len() as u64 != file.size || !sha256(&bytes).eq_ignore_ascii_case(&file.sha256) {
        return Err(format!("已解包源文件被修改或损坏: {key}"));
    }
    Ok(Resource {
        name: file.path.clone(),
        key,
        bytes,
    })
}

fn load_sources(unpack_root: &Path, workspace: &Workspace) -> Result<Sources> {
    let mut found = Vec::new();
    for (disk_index, disk) in workspace.disks.iter().enumerate() {
        let manifest = read_disk_manifest(unpack_root, disk)?;
        let files = manifest
            .files
            .iter()
            .map(|file| (file.path.to_ascii_uppercase(), file))
            .collect::<HashMap<_, _>>();
        if REQUIRED.iter().all(|name| files.contains_key(*name)) {
            found.push((disk_index, manifest));
        }
    }
    if found.len() != 1 {
        return Err(format!(
            "含完整 JACK.EXE/JACK_C.* 资源的系统盘应恰好一张，实际 {}",
            found.len()
        ));
    }
    let (disk_index, manifest) = found.pop().expect("length checked");
    let disk = &workspace.disks[disk_index];
    let files = manifest
        .files
        .iter()
        .map(|file| (file.path.to_ascii_uppercase(), file))
        .collect::<HashMap<_, _>>();
    let take = |name: &str| {
        read_resource(
            unpack_root,
            disk,
            files
                .get(name)
                .copied()
                .ok_or_else(|| format!("系统盘缺少 {name}"))?,
        )
    };
    Ok(Sources {
        disk_directory: disk.output_directory.clone(),
        disk_source_file: disk.source_file.clone(),
        exe: take("JACK.EXE")?,
        atx: take("JACK_C.ATX")?,
        adw: take("JACK_C.ADW")?,
        flg: take("JACK_C.FLG")?,
        frm: take("JACK_C.FRM")?,
        msg: take("JACK_C.MSG")?,
    })
}

fn parse_and_validate_sources(sources: &Sources) -> Result<(jack::ParsedAtx, AdwJson, ExeJson)> {
    let atx = jack::parse_atx(&sources.atx.bytes, sources.atx.key.clone())?;
    let adw = jack::parse_adw(&sources.adw.bytes, sources.adw.key.clone())?;
    let exe = jack::parse_exe(&sources.exe.bytes, sources.exe.key.clone())?;
    jack::validate_flg(&sources.flg.bytes)?;
    jack::validate_msg(&sources.msg.bytes, &atx)?;
    jack::validate_frm(&sources.frm.bytes, &atx)?;
    if atx.json.entries.len() != 1779
        || atx.messages.len() != 415
        || atx.scenes.len() != 109
        || adw.entries.len() != 306
        || exe.entries.len() != 1
    {
        return Err(format!(
            "JACK 资源统计与已确认版本不匹配：ATX {}/{}/{}，ADW {}，EXE {}",
            atx.json.entries.len(),
            atx.messages.len(),
            atx.scenes.len(),
            adw.entries.len(),
            exe.entries.len()
        ));
    }
    Ok((atx, adw, exe))
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
    let sources = load_sources(unpack_root, &workspace)?;
    let (atx, adw, exe) = parse_and_validate_sources(&sources)?;
    let scripts = vec![
        TranslationItem {
            kind: "ATX".into(),
            file: sources.atx.key.clone(),
            json: "JACK_C.ATX.json".into(),
            source_sha256: sha256(&sources.atx.bytes),
            entries: atx.json.entries.len(),
        },
        TranslationItem {
            kind: "ADW".into(),
            file: sources.adw.key.clone(),
            json: "JACK_C.ADW.json".into(),
            source_sha256: sha256(&sources.adw.bytes),
            entries: adw.entries.len(),
        },
        TranslationItem {
            kind: "EXE".into(),
            file: sources.exe.key.clone(),
            json: "JACK.EXE.ui.json".into(),
            source_sha256: sha256(&sources.exe.bytes),
            entries: exe.entries.len(),
        },
    ];
    let manifest = TranslationManifest {
        format: TRANSLATION_FORMAT.into(),
        tool_version: env!("CARGO_PKG_VERSION").into(),
        source_workspace_sha256: sha256(&workspace_bytes),
        scripts,
    };
    let artifacts = vec![
        Artifact {
            relative_path: "JACK_C.ATX.json".into(),
            bytes: pretty_json(&atx.json)?,
        },
        Artifact {
            relative_path: "JACK_C.ADW.json".into(),
            bytes: pretty_json(&adw)?,
        },
        Artifact {
            relative_path: "JACK.EXE.ui.json".into(),
            bytes: pretty_json(&exe)?,
        },
        Artifact {
            relative_path: "translation-workspace.json".into(),
            bytes: pretty_json(&manifest)?,
        },
        Artifact {
            relative_path: "README.txt".into(),
            bytes: "《J 的悲剧》翻译工作区\r\n\r\n只编辑三个 JSON 条目中的 message；scr_msg 与所有下划线字段是只读校验值。\r\nATX 的 logical_entries 是按游戏输出顺序合并的整句，优先编辑其顶层 message；parts[].text 仅是原始分段，不是第二份翻译字段。修改整句后，构建会合并原来的 ?/+ 链。\r\nJACK_C.ATX.json：剧情、MSG 与物品名，共 1779 项。\r\nJACK_C.ADW.json：右侧命令菜单显示词，共 306 项；别名只读。\r\nJACK.EXE.ui.json：底部固定 Select Menu:，编码后最多 14 字节。\r\n书页、标题及场景图是 TITLE*.GAS/JACK_GRA 图像，不在文本提取注入范围。\r\n"
                .as_bytes()
                .to_vec(),
        },
    ];
    Ok(PreparedTranslationExport {
        artifacts,
        output: output.to_path_buf(),
        overwrite,
        atx_entries: atx.json.entries.len(),
        adw_entries: adw.entries.len(),
        exe_entries: exe.entries.len(),
    })
}

#[derive(Default)]
struct TranslationSelection {
    atx: Option<AtxJson>,
    adw: Option<AdwJson>,
    exe: Option<ExeJson>,
    inputs: Vec<BuildInput>,
}

fn read_json_kind(bytes: &[u8], path: &Path, selection: &mut TranslationSelection) -> Result<()> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| format!("{} 不是有效 JSON: {error}", path.display()))?;
    let format = value
        .get("_format")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("{} 缺少 _format", path.display()))?;
    match format {
        jack::ATX_FORMAT => {
            if selection.atx.is_some() {
                return Err("重复选择 ATX 翻译 JSON".into());
            }
            selection.atx = Some(
                serde_json::from_slice(bytes)
                    .map_err(|error| format!("{} 不是有效 ATX JSON: {error}", path.display()))?,
            );
        }
        jack::ADW_FORMAT => {
            if selection.adw.is_some() {
                return Err("重复选择 ADW 翻译 JSON".into());
            }
            selection.adw = Some(
                serde_json::from_slice(bytes)
                    .map_err(|error| format!("{} 不是有效 ADW JSON: {error}", path.display()))?,
            );
        }
        jack::EXE_FORMAT => {
            if selection.exe.is_some() {
                return Err("重复选择 EXE UI 翻译 JSON".into());
            }
            selection.exe =
                Some(serde_json::from_slice(bytes).map_err(|error| {
                    format!("{} 不是有效 EXE UI JSON: {error}", path.display())
                })?);
        }
        other => return Err(format!("{} 使用不支持的 JSON 格式 {other}", path.display())),
    }
    selection.inputs.push(BuildInput {
        name: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("translation.json")
            .to_owned(),
        sha256: sha256(bytes),
    });
    Ok(())
}

fn load_translation_selection(
    selected: &[PathBuf],
    workspace_hash: &str,
) -> Result<TranslationSelection> {
    if selected.is_empty() {
        return Err("至少选择一个翻译 JSON 或翻译工作区目录".into());
    }
    let mut selection = TranslationSelection::default();
    for selected_path in selected {
        let path = fs::canonicalize(selected_path)
            .map_err(|error| format!("无法访问 {}: {error}", selected_path.display()))?;
        if path.is_file() {
            let bytes = fs::read(&path)
                .map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
            read_json_kind(&bytes, &path, &mut selection)?;
            continue;
        }
        if !path.is_dir() {
            return Err(format!("翻译输入不是文件或目录: {}", path.display()));
        }
        let manifest_path = path.join("translation-workspace.json");
        let manifest_bytes = fs::read(&manifest_path)
            .map_err(|error| format!("读取 {} 失败: {error}", manifest_path.display()))?;
        let manifest: TranslationManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|error| format!("{} 无效: {error}", manifest_path.display()))?;
        if manifest.format != TRANSLATION_FORMAT
            || !manifest
                .source_workspace_sha256
                .eq_ignore_ascii_case(workspace_hash)
        {
            return Err(format!("{} 不属于所选解包工作区", manifest_path.display()));
        }
        selection.inputs.push(BuildInput {
            name: "translation-workspace.json".into(),
            sha256: sha256(&manifest_bytes),
        });
        for item in &manifest.scripts {
            let json_path = safe_join(&path, Path::new(&item.json))?;
            let bytes = fs::read(&json_path)
                .map_err(|error| format!("读取 {} 失败: {error}", json_path.display()))?;
            read_json_kind(&bytes, &json_path, &mut selection)?;
        }
    }
    Ok(selection)
}

fn verify_rebuilt_atx(script: &AtxJson, rebuilt: &[u8], plan: &EncodingPlan) -> Result<()> {
    let parsed = jack::parse_atx(rebuilt, script.file.clone())?;
    if script
        .logical_entries
        .iter()
        .any(|logical| logical.message != logical.scr_msg)
    {
        // A changed logical entry is deliberately collapsed to one literal
        // output chain. Syntax, MSG and FRM are validated separately; the
        // collapsed chain no longer has the original atom count to compare.
        return Ok(());
    }
    if parsed.json.entries.len() != script.entries.len() {
        return Err("重建 ATX 的文本条目数发生变化".to_owned());
    }
    for (expected, actual) in script.entries.iter().zip(&parsed.json.entries) {
        let expected_text = if expected.message.is_empty() || expected.message == expected.scr_msg {
            expected.scr_msg.clone()
        } else {
            jack::normalize_translation(&expected.message)?
        };
        if plan.decode_carriers(&actual.scr_msg) != expected_text {
            return Err(format!("重建 ATX entry {} 读回文本不一致", expected.index));
        }
    }
    Ok(())
}

fn verify_rebuilt_adw(script: &AdwJson, rebuilt: &[u8], plan: &EncodingPlan) -> Result<()> {
    let parsed = jack::parse_adw(rebuilt, script.file.clone())?;
    if parsed.entries.len() != script.entries.len() {
        return Err("重建 ADW 的文本条目数发生变化".to_owned());
    }
    for (expected, actual) in script.entries.iter().zip(&parsed.entries) {
        let expected_text = if expected.message == expected.scr_msg {
            expected.scr_msg.clone()
        } else {
            jack::normalize_translation(&expected.message)?
        };
        if plan.decode_carriers(&actual.scr_msg) != expected_text
            || actual.aliases != expected.aliases
        {
            return Err(format!(
                "重建 ADW entry {} 读回文本或别名不一致",
                expected.index
            ));
        }
    }
    Ok(())
}

fn verify_rebuilt_exe(
    script: &ExeJson,
    source: &[u8],
    rebuilt: &[u8],
    plan: &EncodingPlan,
) -> Result<()> {
    if script.entries[0].message == script.entries[0].scr_msg {
        if rebuilt != source {
            return Err("未修改 EXE UI 时 JACK.EXE 未保持逐字节一致".into());
        }
        return Ok(());
    }
    let entry = &script.entries[0];
    let expected = jack::normalize_translation(&entry.message)?;
    let encoded_len = expected.chars().count() * 2;
    let field = &rebuilt[entry.offset..entry.offset + entry.capacity];
    let decoded = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(&field[..encoded_len])
        .ok_or_else(|| "重建 EXE UI 字段不是有效 CP932".to_owned())?;
    if plan.decode_carriers(&decoded) != expected
        || field[encoded_len..].iter().any(|byte| *byte != b' ')
    {
        return Err("重建 EXE UI 字段读回或填充不一致".into());
    }
    Ok(())
}

pub fn prepare_localized_build(
    original_d88: &[PathBuf],
    unpack_root: &Path,
    translations: &[PathBuf],
    output: &Path,
    overwrite: bool,
    font_face: &str,
) -> Result<PreparedLocalizedBuild> {
    validate_separate_output(output, &[unpack_root])?;
    validate_managed_output(output, overwrite, "build-report.json", BUILD_FORMAT)?;
    if font_face.trim().is_empty() || font_face.contains('\0') {
        return Err("字库字体名称不能为空或包含 NUL".to_owned());
    }
    let (workspace_bytes, workspace) = read_workspace(unpack_root)?;
    let workspace_hash = sha256(&workspace_bytes);
    let sources = load_sources(unpack_root, &workspace)?;
    let (baseline_atx, baseline_adw, baseline_exe) = parse_and_validate_sources(&sources)?;
    let selection = load_translation_selection(translations, &workspace_hash)?;
    let TranslationSelection {
        atx: selected_atx,
        adw: selected_adw,
        exe: selected_exe,
        inputs: translation_inputs,
    } = selection;
    let atx = selected_atx.unwrap_or_else(|| baseline_atx.json.clone());
    let adw = selected_adw.unwrap_or_else(|| baseline_adw.clone());
    let exe = selected_exe.unwrap_or_else(|| baseline_exe.clone());
    if atx.file != sources.atx.key || adw.file != sources.adw.key || exe.file != sources.exe.key {
        return Err("翻译 JSON 的 _file 不属于所选 JACK 工作区".to_owned());
    }
    jack::validate_atx(&atx, &sources.atx.bytes)?;
    jack::validate_adw(&adw, &sources.adw.bytes)?;
    jack::validate_exe(&exe, &sources.exe.bytes)?;

    let changed_entries =
        jack::changed_atx(&atx) + jack::changed_adw(&adw) + jack::changed_exe(&exe);
    let final_texts = jack::final_changed_texts(&atx, &adw, &exe)?;
    let reserved = jack::source_reserved_cp932(&atx, &adw, &exe)?;
    let plan = EncodingPlan::build(
        &SubstitutionMap::embedded()?,
        reserved.iter().copied(),
        final_texts.iter().map(String::as_str),
    )?;
    let built_font = font_98::prepare_font(
        font_98::EMBEDDED_FONT,
        &plan.requests(),
        &reserved,
        font_face,
    )?;

    let rebuilt_atx = jack::rebuild_atx(&atx, &sources.atx.bytes, &plan)?;
    let reparsed_atx = jack::parse_atx(&rebuilt_atx, sources.atx.key.clone())?;
    let rebuilt_adw = jack::rebuild_adw(&adw, &sources.adw.bytes, &plan)?;
    let rebuilt_exe = jack::rebuild_exe(&exe, &sources.exe.bytes, &plan)?;
    let rebuilt_msg = jack::rebuild_msg(&sources.msg.bytes, &reparsed_atx)?;
    let rebuilt_frm = jack::rebuild_frm(&sources.frm.bytes, &reparsed_atx)?;
    jack::validate_msg(&rebuilt_msg, &reparsed_atx)?;
    jack::validate_frm(&rebuilt_frm, &reparsed_atx)?;
    verify_rebuilt_atx(&atx, &rebuilt_atx, &plan)?;
    verify_rebuilt_adw(&adw, &rebuilt_adw, &plan)?;
    verify_rebuilt_exe(&exe, &sources.exe.bytes, &rebuilt_exe, &plan)?;
    if changed_entries == 0 {
        let mut changed = Vec::new();
        if rebuilt_atx != sources.atx.bytes {
            changed.push("JACK_C.ATX");
        }
        if rebuilt_adw != sources.adw.bytes {
            changed.push("JACK_C.ADW");
        }
        if rebuilt_exe != sources.exe.bytes {
            changed.push("JACK.EXE");
        }
        if rebuilt_msg != sources.msg.bytes {
            changed.push("JACK_C.MSG");
        }
        if rebuilt_frm != sources.frm.bytes {
            changed.push("JACK_C.FRM");
        }
        if built_font.bytes != font_98::EMBEDDED_FONT {
            changed.push("font.bmp");
        }
        if !changed.is_empty() {
            return Err(format!(
                "零修改构建未保持逐字节一致: {}",
                changed.join(", ")
            ));
        }
    }

    let mut system_replacements = BTreeMap::new();
    system_replacements.insert(sources.atx.name.clone(), rebuilt_atx);
    system_replacements.insert(sources.adw.name.clone(), rebuilt_adw);
    system_replacements.insert(sources.exe.name.clone(), rebuilt_exe);
    system_replacements.insert(sources.msg.name.clone(), rebuilt_msg);
    system_replacements.insert(sources.frm.name.clone(), rebuilt_frm);
    let changed_resources = system_replacements
        .iter()
        .filter(|(name, bytes)| {
            let source = match name.to_ascii_uppercase().as_str() {
                "JACK_C.ATX" => &sources.atx.bytes,
                "JACK_C.ADW" => &sources.adw.bytes,
                "JACK.EXE" => &sources.exe.bytes,
                "JACK_C.MSG" => &sources.msg.bytes,
                "JACK_C.FRM" => &sources.frm.bytes,
                _ => return true,
            };
            bytes.as_slice() != source.as_slice()
        })
        .count();

    let mut input_by_name = HashMap::new();
    for path in discover_d88(original_d88)? {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("D88 文件名无法表示为 Unicode: {}", path.display()))?
            .to_owned();
        if input_by_name
            .insert(name.to_ascii_uppercase(), path)
            .is_some()
        {
            return Err(format!("重复 D88 文件名: {name}"));
        }
    }
    let mut artifacts = Vec::new();
    let mut output_disks = Vec::new();
    for disk in &workspace.disks {
        let manifest = read_disk_manifest(unpack_root, disk)?;
        let source_path = input_by_name
            .get(&disk.source_file.to_ascii_uppercase())
            .ok_or_else(|| format!("缺少原始 D88: {}", disk.source_file))?;
        let source = fs::read(source_path)
            .map_err(|error| format!("读取 {} 失败: {error}", source_path.display()))?;
        if !sha256(&source).eq_ignore_ascii_case(&manifest.source_sha256) {
            return Err(format!("原始 D88 哈希不匹配: {}", disk.source_file));
        }
        let replacements = if disk
            .output_directory
            .eq_ignore_ascii_case(&sources.disk_directory)
        {
            system_replacements.clone()
        } else {
            BTreeMap::new()
        };
        let rebuilt = rebuild_d88(source_path, &source, &replacements)?;
        if changed_entries == 0 && rebuilt != source {
            return Err(format!("零修改构建改变了 D88: {}", disk.source_file));
        }
        let relative = PathBuf::from("d88").join(&disk.source_file);
        output_disks.push(relative.to_string_lossy().replace('\\', "/"));
        artifacts.push(Artifact {
            relative_path: relative,
            bytes: rebuilt,
        });
    }
    if !workspace.disks.iter().any(|disk| {
        disk.source_file
            .eq_ignore_ascii_case(&sources.disk_source_file)
    }) {
        return Err("系统盘没有映射到 workspace.json".into());
    }

    let font_manifest = FontManifest {
        format: "j-no-higeki-np2-font-map-v1".into(),
        file: "font.bmp".into(),
        face: font_face.to_owned(),
        source_font_sha256: font_98::embedded_font_sha256().to_uppercase(),
        output_font_sha256: sha256(&built_font.bytes).to_uppercase(),
        patched_glyphs: built_font.patched_glyphs,
        reserved_slots: reserved.len(),
        mappings: plan.manifest_entries()?,
    };
    artifacts.push(Artifact {
        relative_path: "font.bmp".into(),
        bytes: built_font.bytes,
    });
    artifacts.push(Artifact {
        relative_path: "font_mapping.json".into(),
        bytes: pretty_json(&font_manifest)?,
    });
    let build_manifest = BuildManifest {
        format: BUILD_FORMAT.into(),
        tool_version: env!("CARGO_PKG_VERSION").into(),
        source_workspace_sha256: workspace_hash,
        translation_inputs,
        disks: output_disks,
        atx_entries: atx.entries.len(),
        adw_entries: adw.entries.len(),
        exe_entries: exe.entries.len(),
        changed_entries,
        changed_resources,
        font_file: "font.bmp".into(),
        font_mapping_file: "font_mapping.json".into(),
        font_patched_glyphs: built_font.patched_glyphs,
    };
    artifacts.push(Artifact {
        relative_path: "build-report.json".into(),
        bytes: pretty_json(&build_manifest)?,
    });
    artifacts.push(Artifact {
        relative_path: "README.txt".into(),
        bytes: "《J 的悲剧》汉化成品\r\n\r\nd88 子目录是完整回封镜像；font.bmp 请作为 NP2/兼容模拟器的 PC-98 字库使用。\r\nfont_mapping.json 记录本次中文字槽映射，build-report.json 记录来源和校验摘要。\r\n原始 D88、解包工作区和翻译 JSON 均未被修改。\r\n"
            .as_bytes()
            .to_vec(),
    });

    Ok(PreparedLocalizedBuild {
        artifacts,
        output: output.to_path_buf(),
        overwrite,
        disks: workspace.disks.len(),
        entries: atx.entries.len() + adw.entries.len() + exe.entries.len(),
        changed_entries,
        changed_resources,
        patched_glyphs: built_font.patched_glyphs,
    })
}

fn discover_d88(selected: &[PathBuf]) -> Result<Vec<PathBuf>> {
    if selected.is_empty() {
        return Err("至少需要一个 D88 文件或目录".into());
    }
    let mut inputs = Vec::new();
    for selected_path in selected {
        let path = fs::canonicalize(selected_path)
            .map_err(|error| format!("无法访问 {}: {error}", selected_path.display()))?;
        if path.is_file() {
            if !path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("d88"))
            {
                return Err(format!("输入文件不是 D88: {}", path.display()));
            }
            inputs.push(path);
        } else if path.is_dir() {
            let mut children = fs::read_dir(&path)
                .map_err(|error| format!("无法读取 {}: {error}", path.display()))?
                .map(|entry| entry.map(|value| value.path()))
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|error| format!("枚举 {} 失败: {error}", path.display()))?;
            children.sort();
            inputs.extend(children.into_iter().filter(|child| {
                child.is_file()
                    && child
                        .extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|value| value.eq_ignore_ascii_case("d88"))
            }));
        }
    }
    inputs.sort();
    inputs.dedup_by(|left, right| {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    });
    if inputs.is_empty() {
        return Err("所选路径中没有 D88 文件".into());
    }
    Ok(inputs)
}

fn validate_separate_output(output: &Path, inputs: &[&Path]) -> Result<()> {
    let output = std::path::absolute(output)
        .map_err(|error| format!("无法解析输出目录 {}: {error}", output.display()))?;
    for input in inputs {
        let input = fs::canonicalize(input)
            .map_err(|error| format!("无法解析输入路径 {}: {error}", input.display()))?;
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
    if value.get("_format").and_then(serde_json::Value::as_str) != Some(expected_format) {
        return Err(format!(
            "现有输出不是本操作生成的 {expected_format}，拒绝覆盖"
        ));
    }
    Ok(())
}

fn unique_sibling(path: &Path, label: &str) -> Result<PathBuf> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("路径没有父目录: {}", path.display()))?;
    let leaf = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("路径名无法表示为 Unicode: {}", path.display()))?;
    for index in 0..1000usize {
        let candidate = parent.join(format!(".{leaf}.{label}-{}-{index}", std::process::id()));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(format!("无法为 {} 分配临时目录", path.display()))
}

fn commit_staging(staging: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if !output.exists() {
        return fs::rename(staging, output)
            .map_err(|error| format!("提交输出 {} 失败: {error}", output.display()));
    }
    if !overwrite {
        return Err(format!("输出已存在，默认不覆盖: {}", output.display()));
    }
    let backup = unique_sibling(output, "backup")?;
    fs::rename(output, &backup)
        .map_err(|error| format!("暂存旧输出 {} 失败: {error}", output.display()))?;
    if let Err(error) = fs::rename(staging, output) {
        let rollback = fs::rename(&backup, output);
        return match rollback {
            Ok(()) => Err(format!("提交失败，已恢复旧输出: {error}")),
            Err(rollback_error) => Err(format!(
                "提交失败且旧输出恢复失败；备份位于 {}: {error}; {rollback_error}",
                backup.display()
            )),
        };
    }
    fs::remove_dir_all(&backup).map_err(|error| format!("新输出已提交，但清理旧备份失败: {error}"))
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
