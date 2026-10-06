use crate::{catalog, sha256_hex, text, Result};
use pc98_fdi_unpack::{extract_fdi_files, rebuild_fdi, FdiFileData};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use vn_font::font_98::{self, EncodingPlan, SubstitutionMap};

const FORMAT: &str = "38mank-void-workspace-v1";

#[derive(Debug, Clone)]
pub struct WorkflowReport {
    pub summary: String,
    pub files: usize,
    pub messages: usize,
    pub outputs: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    format: String,
    disk1_sha256: String,
    disk2_sha256: String,
    files: Vec<FileRecord>,
    members: Vec<MemberRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileRecord {
    disk: String,
    path: String,
    sha256: String,
    size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct MemberRecord {
    name: String,
    sha256: String,
    size: usize,
}

struct StagedDir {
    path: PathBuf,
    parent: PathBuf,
    committed: bool,
}

impl StagedDir {
    fn new(output: &Path) -> Result<Self> {
        if output.exists() {
            return Err(format!("输出已存在，默认拒绝覆盖: {}", output.display()));
        }
        let parent = output.parent().ok_or("输出路径缺少父目录")?;
        if !parent.is_dir() {
            return Err(format!("输出父目录不存在: {}", parent.display()));
        }
        let parent = fs::canonicalize(parent).map_err(|e| format!("解析输出父目录失败: {e}"))?;
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let name = format!(
                ".38mank-stage-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            );
            let path = parent.join(name);
            match fs::create_dir(&path) {
                Ok(()) => {
                    let path =
                        fs::canonicalize(&path).map_err(|e| format!("解析暂存目录失败: {e}"))?;
                    if path.parent() != Some(parent.as_path()) {
                        return Err("暂存目录超出预期的输出父目录".into());
                    }
                    return Ok(Self {
                        path,
                        parent,
                        committed: false,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(format!("无法建立暂存目录: {error}")),
            }
        }
        Err("无法分配暂存目录".into())
    }

    fn write(&self, relative: &str, bytes: &[u8]) -> Result<()> {
        let relative = safe_relative(relative)?;
        let path = self.path.join(relative);
        let parent = path.parent().ok_or("输出文件缺少父目录")?;
        fs::create_dir_all(parent).map_err(|e| format!("建立 {} 失败: {e}", parent.display()))?;
        let mut file =
            File::create(&path).map_err(|e| format!("建立 {} 失败: {e}", path.display()))?;
        file.write_all(bytes)
            .map_err(|e| format!("写入 {} 失败: {e}", path.display()))?;
        file.sync_all()
            .map_err(|e| format!("同步 {} 失败: {e}", path.display()))
    }

    fn commit(mut self, output: &Path) -> Result<()> {
        if output.exists() {
            return Err(format!("输出在写入期间出现: {}", output.display()));
        }
        fs::rename(&self.path, output)
            .map_err(|e| format!("提交 {} 失败: {e}", output.display()))?;
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

fn protect_output(output: &Path, inputs: &[&Path]) -> Result<()> {
    let parent = output.parent().ok_or("输出路径缺少父目录")?;
    let parent = fs::canonicalize(parent).map_err(|e| format!("解析输出父目录失败: {e}"))?;
    let target = parent.join(output.file_name().ok_or("输出路径缺少目录名")?);
    if target.exists() {
        return Err(format!("输出已存在: {}", target.display()));
    }
    for input in inputs {
        let source = fs::canonicalize(input)
            .map_err(|e| format!("解析输入 {} 失败: {e}", input.display()))?;
        if target == source || (source.is_dir() && target.starts_with(&source)) {
            return Err(format!("输出与输入范围重叠: {}", input.display()));
        }
    }
    Ok(())
}

fn safe_relative(raw: &str) -> Result<PathBuf> {
    let normalized = raw.replace('\\', "/");
    let path = Path::new(&normalized);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(format!("非法相对路径: {raw}"));
    }
    Ok(path.to_path_buf())
}

fn output_name(path: &str) -> String {
    path.replace(['/', '\\'], "__")
}

fn read_fdi(path: &Path) -> Result<(Vec<u8>, Vec<FdiFileData>)> {
    let bytes = fs::read(path).map_err(|e| format!("读取 {} 失败: {e}", path.display()))?;
    let files = extract_fdi_files(&bytes, &path.to_string_lossy())?;
    Ok((bytes, files))
}

fn json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec_pretty(value).map_err(|e| format!("序列化 JSON 失败: {e}"))
}

fn checked_read(root: &Path, relative: &str, expected: &str) -> Result<Vec<u8>> {
    let path = root.join(safe_relative(relative)?);
    let bytes = fs::read(&path).map_err(|e| format!("读取 {} 失败: {e}", path.display()))?;
    if sha256_hex(&bytes) != expected {
        return Err(format!("源工作区内容或哈希不匹配: {}", path.display()));
    }
    Ok(bytes)
}

fn read_optional_translation(root: &Path, relative: &str) -> Result<Option<text::TextDocument>> {
    let path = root.join(safe_relative(relative)?);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(|e| format!("读取 {} 失败: {e}", path.display()))?;
    let doc = serde_json::from_slice(&bytes)
        .map_err(|e| format!("翻译 JSON 格式错误 {}: {e}", path.display()))?;
    Ok(Some(doc))
}

fn font_text(value: &str) -> String {
    let mut result = String::new();
    let mut rest = value;
    while !rest.is_empty() {
        if let Some(start) = rest.find("[[PC98:") {
            result.extend(
                rest[..start]
                    .chars()
                    .filter(|c| !c.is_ascii() && !c.is_control()),
            );
            let after = &rest[start + 7..];
            if let Some(end) = after.find("]]") {
                rest = &after[end + 2..];
            } else {
                break;
            }
        } else {
            result.extend(rest.chars().filter(|c| !c.is_ascii() && !c.is_control()));
            break;
        }
    }
    result
}

fn reserve_font_text(value: &str, reserved: &mut BTreeSet<u16>) {
    for character in font_text(value).chars() {
        if let Ok(cp932) = font_98::cp932_for_carrier(character) {
            if font_98::has_loaded_np2_slot(character) {
                reserved.insert(u16::from_be_bytes(cp932));
            }
        }
    }
}

fn collect_font_document(
    document: &text::TextDocument,
    reserved: &mut BTreeSet<u16>,
    texts: &mut Vec<String>,
) {
    for entry in &document.entries {
        if entry.message == entry.scr_msg {
            reserve_font_text(&entry.scr_msg, reserved);
        }
        let display = font_text(&entry.message);
        if !display.is_empty() {
            texts.push(display);
        }
    }
}

fn build_font(
    disk1_files: &[FdiFileData],
    disk2_files: &[FdiFileData],
    translations: &Path,
) -> Result<(
    EncodingPlan,
    font_98::FontBuild,
    Vec<font_98::EncodingPlanEntry>,
)> {
    let mut reserved = BTreeSet::new();
    let mut texts = Vec::new();
    for (disk, files) in [("disk1", disk1_files), ("disk2", disk2_files)] {
        for file in files {
            let path = file.path.replace('\\', "/");
            if disk == "disk2" && path.eq_ignore_ascii_case("TXTALL.DAT") {
                let catalog = catalog::parse(&file.data)?;
                for entry in &catalog.entries {
                    let member = catalog::member_data(&file.data, entry)?;
                    if let Some(baseline) = text::extract_document(member, &entry.name)? {
                        let document = match read_optional_translation(
                            translations,
                            &format!("catalog/{}.json", entry.name),
                        )? {
                            Some(supplied) => text::merge_document(member, &entry.name, &supplied)?,
                            None => baseline,
                        };
                        collect_font_document(&document, &mut reserved, &mut texts);
                    }
                }
            } else if let Some(baseline) = text::extract_document(&file.data, &path)? {
                let document = match read_optional_translation(
                    translations,
                    &format!("{disk}/{}.json", output_name(&path)),
                )? {
                    Some(supplied) => text::merge_document(&file.data, &path, &supplied)?,
                    None => baseline,
                };
                collect_font_document(&document, &mut reserved, &mut texts);
            }
        }
    }
    let forbidden = [0x8140, 0x814F, 0x818F, 0x8197, 0x81A5];
    let plan = EncodingPlan::build_with_forbidden_cp932(
        &SubstitutionMap::embedded()?,
        reserved.iter().copied(),
        forbidden,
        texts.iter().map(String::as_str),
    )?;
    let font = font_98::prepare_font(
        font_98::EMBEDDED_FONT,
        &plan.requests(),
        &reserved,
        font_98::FONT_FACE,
    )?;
    let manifest = plan.manifest_entries()?;
    Ok((plan, font, manifest))
}

pub fn extract_workflow(disk1: &Path, disk2: &Path, output: &Path) -> Result<WorkflowReport> {
    protect_output(output, &[disk1, disk2])?;
    let (disk1_bytes, disk1_files) = read_fdi(disk1)?;
    let (disk2_bytes, disk2_files) = read_fdi(disk2)?;
    let stage = StagedDir::new(output)?;
    let mut manifest = Manifest {
        format: FORMAT.into(),
        disk1_sha256: sha256_hex(&disk1_bytes),
        disk2_sha256: sha256_hex(&disk2_bytes),
        files: Vec::new(),
        members: Vec::new(),
    };
    let mut messages = 0usize;
    let mut catalog_seen = false;
    let mut used_names = BTreeSet::new();
    let mut used_translation_names = BTreeSet::new();
    for (disk, files) in [("disk1", &disk1_files), ("disk2", &disk2_files)] {
        for file in files {
            let path = file.path.replace('\\', "/");
            safe_relative(&path)?;
            let key = format!("{disk}/{}", path.to_ascii_uppercase());
            if !used_names.insert(key) {
                return Err(format!("磁盘路径重复: {disk}/{path}"));
            }
            let raw_rel = format!("files/{disk}/{path}");
            stage.write(&raw_rel, &file.data)?;
            manifest.files.push(FileRecord {
                disk: disk.into(),
                path: path.clone(),
                sha256: sha256_hex(&file.data),
                size: file.data.len(),
            });
            if disk == "disk2" && path.eq_ignore_ascii_case("TXTALL.DAT") {
                catalog_seen = true;
                let catalog = catalog::parse(&file.data)?;
                for entry in &catalog.entries {
                    let member = catalog::member_data(&file.data, entry)?;
                    let member_rel = format!("catalog/members/{}", entry.name);
                    stage.write(&member_rel, member)?;
                    manifest.members.push(MemberRecord {
                        name: entry.name.clone(),
                        sha256: sha256_hex(member),
                        size: member.len(),
                    });
                    if let Some(doc) = text::extract_document(member, &entry.name)? {
                        messages += doc.entries.len();
                        let template_path = format!("translations/catalog/{}.json", entry.name);
                        if !used_translation_names.insert(template_path.to_ascii_uppercase()) {
                            return Err(format!("翻译路径重复: {template_path}"));
                        }
                        stage.write(&template_path, &json(&doc)?)?;
                    }
                }
                stage.write("catalog/manifest.json", &json(&catalog)?)?;
            } else if let Some(doc) = text::extract_document(&file.data, &path)? {
                messages += doc.entries.len();
                let template_path = format!("translations/{disk}/{}.json", output_name(&path));
                if !used_translation_names.insert(template_path.to_ascii_uppercase()) {
                    return Err(format!("翻译路径重复: {template_path}"));
                }
                stage.write(&template_path, &json(&doc)?)?;
            }
        }
    }
    if !catalog_seen {
        return Err("第二盘未找到 TXTALL.DAT".into());
    }
    stage.write("workspace.json", &json(&manifest)?)?;
    stage.write(
        "README.txt",
        "Copy translations/ to a separate editable directory. Edit only entries[].message there, then pass that directory to inject --translations. files/, catalog/members/, and workspace.json are immutable source baselines.\n".as_bytes(),
    )?;
    stage.commit(output)?;
    Ok(WorkflowReport {
        summary: "双盘文本工作区已导出".into(),
        files: manifest.files.len(),
        messages,
        outputs: vec![output.to_path_buf()],
    })
}

pub fn inject_workflow(
    disk1: &Path,
    disk2: &Path,
    workspace: &Path,
    translations: &Path,
    output: &Path,
) -> Result<WorkflowReport> {
    protect_output(output, &[disk1, disk2, workspace, translations])?;
    let workspace_absolute =
        fs::canonicalize(workspace).map_err(|e| format!("解析原始工作区失败: {e}"))?;
    let translations_absolute =
        fs::canonicalize(translations).map_err(|e| format!("解析独立译文目录失败: {e}"))?;
    if translations_absolute.starts_with(&workspace_absolute) {
        return Err("独立译文目录不能位于原始工作区内".into());
    }
    let manifest_path = workspace.join("workspace.json");
    let manifest: Manifest = serde_json::from_slice(
        &fs::read(&manifest_path)
            .map_err(|e| format!("读取 {} 失败: {e}", manifest_path.display()))?,
    )
    .map_err(|e| format!("工作区清单格式错误: {e}"))?;
    if manifest.format != FORMAT {
        return Err(format!("不支持的工作区格式: {}", manifest.format));
    }
    let (disk1_bytes, disk1_files) = read_fdi(disk1)?;
    let (disk2_bytes, disk2_files) = read_fdi(disk2)?;
    if sha256_hex(&disk1_bytes) != manifest.disk1_sha256
        || sha256_hex(&disk2_bytes) != manifest.disk2_sha256
    {
        return Err("原始 FDI 与导出工作区时的镜像不一致".into());
    }
    let (font_plan, font_build, font_manifest) =
        build_font(&disk1_files, &disk2_files, translations)?;
    let mut replacements = [BTreeMap::new(), BTreeMap::new()];
    let mut messages = 0usize;
    let mut catalog_seen = false;
    for (disk_index, files) in [disk1_files, disk2_files].into_iter().enumerate() {
        let disk = if disk_index == 0 { "disk1" } else { "disk2" };
        for file in files {
            let path = file.path.replace('\\', "/");
            let record = manifest
                .files
                .iter()
                .find(|item| item.disk == disk && item.path.eq_ignore_ascii_case(&path))
                .ok_or_else(|| format!("工作区清单缺少 {disk}/{path}"))?;
            if record.size != file.data.len() || sha256_hex(&file.data) != record.sha256 {
                return Err(format!("镜像文件与工作区清单不一致: {disk}/{path}"));
            }
            checked_read(workspace, &format!("files/{disk}/{path}"), &record.sha256)?;
            let mut result = file.data.clone();
            if disk_index == 1 && path.eq_ignore_ascii_case("TXTALL.DAT") {
                catalog_seen = true;
                let catalog = catalog::parse(&file.data)?;
                if catalog.entries.len() != manifest.members.len() {
                    return Err("TXTALL.DAT 成员数与工作区不一致".into());
                }
                let mut member_replacements = BTreeMap::new();
                for entry in &catalog.entries {
                    let member = catalog::member_data(&file.data, entry)?;
                    let record = manifest
                        .members
                        .iter()
                        .find(|item| item.name == entry.name)
                        .ok_or_else(|| format!("工作区清单缺少成员 {}", entry.name))?;
                    if record.size != member.len() || sha256_hex(member) != record.sha256 {
                        return Err(format!("TXTALL.DAT 成员基线不一致: {}", entry.name));
                    }
                    checked_read(
                        workspace,
                        &format!("catalog/members/{}", entry.name),
                        &record.sha256,
                    )?;
                    if let Some(doc) = read_optional_translation(
                        translations,
                        &format!("catalog/{}.json", entry.name),
                    )? {
                        let new_member =
                            text::apply_document(member, &entry.name, &doc, &font_plan)?;
                        messages += doc
                            .entries
                            .iter()
                            .filter(|item| item.message != item.scr_msg)
                            .count();
                        if new_member != member {
                            member_replacements.insert(entry.name.clone(), new_member);
                        }
                    }
                }
                result = catalog::rebuild(&file.data, &member_replacements)?;
            } else if let Some(doc) = read_optional_translation(
                translations,
                &format!("{disk}/{}.json", output_name(&path)),
            )? {
                result = text::apply_document(&file.data, &path, &doc, &font_plan)?;
                messages += doc
                    .entries
                    .iter()
                    .filter(|item| item.message != item.scr_msg)
                    .count();
            }
            if result != file.data {
                replacements[disk_index].insert(file.path, result);
            }
        }
    }
    if !catalog_seen {
        return Err("第二盘未找到 TXTALL.DAT".into());
    }
    if manifest.files.len()
        != extract_fdi_files(&disk1_bytes, &disk1.to_string_lossy())?.len()
            + extract_fdi_files(&disk2_bytes, &disk2.to_string_lossy())?.len()
    {
        return Err("工作区文件清单数量不一致".into());
    }
    let pack1 = rebuild_fdi(&disk1_bytes, &disk1.to_string_lossy(), &replacements[0])?;
    let pack2 = rebuild_fdi(&disk2_bytes, &disk2.to_string_lossy(), &replacements[1])?;
    let changed_files = pack1.changed_files + pack2.changed_files;
    let report = serde_json::json!({
        "format": "38mank-void-build-report-v1",
        "source_sha256": [manifest.disk1_sha256, manifest.disk2_sha256],
        "output_sha256": [sha256_hex(&pack1.bytes), sha256_hex(&pack2.bytes)],
        "changed_files": changed_files,
        "changed_messages": messages,
        "font": {
            "face": font_98::FONT_FACE,
            "embedded_sha256": sha256_hex(font_98::EMBEDDED_FONT),
            "output_sha256": sha256_hex(&font_build.bytes),
            "patched_glyphs": font_build.patched_glyphs,
            "mapping_entries": font_manifest.len()
        },
        "disk1": {"files": pack1.files, "changed_files": pack1.changed_files, "allocated_clusters": pack1.allocated_clusters, "released_clusters": pack1.released_clusters},
        "disk2": {"files": pack2.files, "changed_files": pack2.changed_files, "allocated_clusters": pack2.allocated_clusters, "released_clusters": pack2.released_clusters}
    });
    let stage = StagedDir::new(output)?;
    stage.write("38mank1.FDI", &pack1.bytes)?;
    stage.write("38mank2.FDI", &pack2.bytes)?;
    stage.write("font.tmp", &font_build.bytes)?;
    stage.write("font_mapping.json", &json(&font_manifest)?)?;
    stage.write("build_report.json", &json(&report)?)?;
    stage.commit(output)?;
    Ok(WorkflowReport {
        summary: "双盘 FDI 已重建".into(),
        files: changed_files,
        messages,
        outputs: vec![output.to_path_buf()],
    })
}
