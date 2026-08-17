use crate::bundle::{Bundle, BundleError};
use crate::encoding::encode_cp932;
use crate::extract::build_plan;
use crate::json_model::{
    editable_name, restore_name_wrappers, TranslationEntry, TranslationFile, WorkspaceManifest,
    WORKSPACE_FORMAT,
};
use crate::workspace::{
    commit_stage, ensure_output_safe, resolve_workspace_role, sha256_bytes, validate_snapshot,
    write_json,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone)]
pub struct InjectOptions {
    pub workspace: PathBuf,
    pub source: Option<PathBuf>,
    pub translations: Option<PathBuf>,
    pub output: PathBuf,
    pub overwrite: bool,
}

#[derive(Debug, Clone, Default)]
pub struct InjectReport {
    pub json_files: usize,
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub warnings: usize,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
struct Proposal {
    value: String,
    locations: Vec<String>,
}

#[derive(Debug, Clone)]
struct PatchPlan {
    patches: BTreeMap<usize, String>,
    report: InjectReport,
}

pub(crate) fn read_workspace(root: &Path) -> Result<WorkspaceManifest, BundleError> {
    let text = fs::read_to_string(root.join("workspace.json"))
        .map_err(|e| format!("读取 workspace.json 失败: {}", e))?;
    let manifest: WorkspaceManifest = serde_json::from_str(&text)
        .map_err(|e| format!("workspace.json 不是有效工作区清单: {}", e))?;
    if manifest.format != WORKSPACE_FORMAT {
        return Err(format!("不支持的工作区格式: {}", manifest.format));
    }
    if manifest.version != 1 {
        return Err(format!("不支持的工作区版本: {}", manifest.version));
    }
    Ok(manifest)
}

fn collect_json_files(root: &Path, result: &mut Vec<PathBuf>) -> Result<(), BundleError> {
    let metadata =
        fs::symlink_metadata(root).map_err(|e| format!("读取 {} 失败: {}", root.display(), e))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("拒绝翻译目录中的符号链接: {}", root.display()));
    }
    if metadata.is_dir() {
        for entry in
            fs::read_dir(root).map_err(|e| format!("读取 {} 失败: {}", root.display(), e))?
        {
            collect_json_files(
                &entry.map_err(|e| format!("读取目录项失败: {}", e))?.path(),
                result,
            )?;
        }
    } else if metadata.is_file()
        && root
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("json"))
    {
        result.push(root.to_path_buf());
    }
    Ok(())
}

fn file_key(group: &str, file_id: Option<usize>, file: Option<&str>) -> String {
    format!(
        "{}|{}|{}",
        group,
        file_id.map(|value| value.to_string()).unwrap_or_default(),
        file.unwrap_or_default()
    )
}

fn validate_entry(
    bundle: &Bundle,
    group: &str,
    entry: &TranslationEntry,
    location: &str,
) -> Result<(), BundleError> {
    let source = bundle
        .cstr
        .get(entry.cstr_id)
        .ok_or_else(|| format!("{} 的 CSTR[{}] 越界", location, entry.cstr_id))?;
    if source.text != entry.scr_msg {
        return Err(format!(
            "{} scr_msg 校验失败: CSTR[{}] 源文本已变化",
            location, entry.cstr_id
        ));
    }
    if source.pool_offset != entry.offset
        || source.size.saturating_sub(1) != entry.size
        || source.size != entry.cstr_size
    {
        return Err(format!(
            "{} CSTR 定位元数据校验失败: CSTR[{}]",
            location, entry.cstr_id
        ));
    }
    if entry.file_id >= bundle.source_files.len()
        || bundle.source_files[entry.file_id] != entry.file
    {
        return Err(format!("{} 来源文件校验失败", location));
    }
    let valid_type = matches!(
        (group, entry.type_name.as_str()),
        ("script", "dialogue" | "narration" | "choice")
            | ("system", "system_choice")
            | ("names", "name")
    );
    if !valid_type {
        return Err(format!(
            "{} 的分组/类型组合无效: {}/{}",
            location, group, entry.type_name
        ));
    }
    if entry.type_name == "name" {
        let name = entry
            .name
            .as_ref()
            .ok_or_else(|| format!("{} name 条目缺少 name", location))?;
        let scr_name = entry
            .scr_name
            .as_ref()
            .ok_or_else(|| format!("{} name 条目缺少 scr_name", location))?;
        if entry.message != entry.scr_msg || scr_name != &editable_name(&entry.scr_msg) {
            return Err(format!(
                "{} name 条目必须只编辑 name，并保留 scr_name/message/scr_msg 源字段",
                location
            ));
        }
        if entry.scr_msg.is_empty() {
            if !name.is_empty() {
                return Err(format!(
                    "{} 空 TalkInfo 是旁白，name 必须保持为空",
                    location
                ));
            }
        } else if name.contains('【') || name.contains('】') {
            return Err(format!(
                "{} name 只填写括号内的角色名，不要输入【】",
                location
            ));
        }
    } else if matches!(entry.type_name.as_str(), "dialogue" | "narration") {
        let scr_name = entry
            .scr_name
            .as_ref()
            .ok_or_else(|| format!("{} 缺少 scr_name", location))?;
        let name = entry
            .name
            .as_ref()
            .ok_or_else(|| format!("{} 缺少 name", location))?;
        if let Some(name_cstr_id) = entry.name_cstr_id {
            let name_source = bundle
                .cstr
                .get(name_cstr_id)
                .ok_or_else(|| format!("{} 的姓名 CSTR[{}] 越界", location, name_cstr_id))?;
            if scr_name != &editable_name(&name_source.text) || name != scr_name {
                return Err(format!(
                    "{} 姓名校验失败: 角色名只能在 names.json 编辑，CSTR[{}]",
                    location, name_cstr_id
                ));
            }
        } else if entry.message_id.is_some() {
            if name != scr_name || entry.talk_info_id.is_some() {
                return Err(format!(
                    "{} 第二部姓名标签来自样式映射，只能编辑 message",
                    location
                ));
            }
        } else {
            return Err(format!("{} 对话条目缺少姓名来源元数据", location));
        }
    } else if entry.name.is_some()
        || entry.scr_name.is_some()
        || entry.name_cstr_id.is_some()
        || entry.talk_info_id.is_some()
        || entry.talk_style.is_some()
        || entry.message_id.is_some()
    {
        return Err(format!("{} 选项条目含不应存在的姓名元数据", location));
    }
    Ok(())
}

fn validate_canonical_entry(
    entry: &TranslationEntry,
    canonical: &TranslationEntry,
    location: &str,
) -> Result<(), BundleError> {
    let mut normalized = entry.clone();
    if canonical.type_name == "name" {
        normalized.name = canonical.name.clone();
    } else {
        normalized.message = canonical.message.clone();
    }
    if normalized != *canonical {
        return Err(format!("{} 的不可变身份或定位元数据已变化", location));
    }
    Ok(())
}

fn add_proposal(
    map: &mut HashMap<usize, Proposal>,
    id: usize,
    value: String,
    location: String,
) -> Result<(), BundleError> {
    if let Some(previous) = map.get_mut(&id) {
        if previous.value != value {
            return Err(format!(
                "CSTR[{}] 存在冲突译文: {} 与 {}",
                id, previous.value, value
            ));
        }
        previous.locations.push(location);
    } else {
        map.insert(
            id,
            Proposal {
                value,
                locations: vec![location],
            },
        );
    }
    Ok(())
}

fn build_patch_plan(
    bundle: &Bundle,
    workspace: &WorkspaceManifest,
    translation_root: &Path,
) -> Result<PatchPlan, BundleError> {
    let canonical_plan = build_plan(bundle)?;
    let mut canonical_files = BTreeMap::new();
    for planned in canonical_plan.translation_files {
        let key = file_key(
            &planned.translation.group,
            planned.translation.file_id,
            planned.translation.file.as_deref(),
        );
        if canonical_files
            .insert(key.clone(), (planned.relative_path, planned.translation))
            .is_some()
        {
            return Err(format!("规范提取计划角色重复: {}", key));
        }
    }
    let source_manifest_format = bundle
        .manifest
        .get("format")
        .and_then(Value::as_str)
        .unwrap_or("<missing>");
    if workspace.source_manifest_format != source_manifest_format {
        return Err("workspace.json 的源 manifest 格式与源快照不一致".to_string());
    }
    if workspace.logical_source_files.len() != bundle.source_files.len()
        || workspace
            .logical_source_files
            .iter()
            .enumerate()
            .any(|(file_id, item)| {
                item.file_id != file_id || item.file != bundle.source_files[file_id]
            })
    {
        return Err("workspace.json 的逻辑源文件清单与源快照不一致".to_string());
    }

    let mut files = Vec::new();
    collect_json_files(translation_root, &mut files)?;
    files.sort();
    let mut expected = BTreeMap::new();
    for item in &workspace.translation_files {
        let key = file_key(&item.group, item.file_id, item.file.as_deref());
        let Some((canonical_path, canonical_translation)) = canonical_files.get(&key) else {
            return Err(format!("workspace.json 含未知翻译文件角色: {}", key));
        };
        if item.relative_path != *canonical_path
            || item.entries != canonical_translation.entries.len()
        {
            return Err(format!("workspace.json 翻译文件元数据变化: {}", key));
        }
        if expected.insert(key.clone(), item.entries).is_some() {
            return Err(format!("workspace.json 翻译文件角色重复: {}", key));
        }
    }
    if expected.len() != canonical_files.len() {
        return Err("workspace.json 翻译文件角色清单不完整".to_string());
    }
    let mut actual = BTreeSet::new();
    let mut body_proposals: HashMap<usize, Proposal> = HashMap::new();
    let mut name_proposals: HashMap<usize, Proposal> = HashMap::new();
    let mut json_entries = 0usize;
    let mut modified_entries = 0usize;
    for path in &files {
        let text =
            fs::read_to_string(path).map_err(|e| format!("读取 {} 失败: {}", path.display(), e))?;
        let translation: TranslationFile = serde_json::from_str(&text)
            .map_err(|e| format!("{} 不是有效翻译 JSON: {}", path.display(), e))?;
        if translation.format != crate::json_model::TRANSLATION_FORMAT || translation.version != 1 {
            return Err(format!("{} 不是受支持的翻译文件格式", path.display()));
        }
        let key = file_key(
            &translation.group,
            translation.file_id,
            translation.file.as_deref(),
        );
        let Some(expected_entries) = expected.get(&key).copied() else {
            return Err(format!(
                "{} 不在 workspace.json 的翻译清单中",
                path.display()
            ));
        };
        let canonical_translation = &canonical_files[&key].1;
        if !actual.insert(key.clone()) {
            return Err(format!("翻译文件角色重复: {}", key));
        }
        if translation.entries.len() != expected_entries {
            return Err(format!(
                "{} 条目数量变化: 预期 {}，实际 {}",
                path.display(),
                expected_entries,
                translation.entries.len()
            ));
        }
        let mut indices = BTreeSet::new();
        for entry in &translation.entries {
            json_entries += 1;
            let location = format!("{}::_index={}", path.display(), entry.index);
            if entry.index >= expected_entries || !indices.insert(entry.index) {
                return Err(format!("{} 的 _index 越界或重复", location));
            }
            validate_canonical_entry(
                entry,
                &canonical_translation.entries[entry.index],
                &location,
            )?;
            validate_entry(bundle, &translation.group, entry, &location)?;
            if entry.type_name == "name" {
                let value = entry.name.as_ref().unwrap();
                if value != &editable_name(&entry.scr_msg) {
                    modified_entries += 1;
                    add_proposal(
                        &mut name_proposals,
                        entry.cstr_id,
                        restore_name_wrappers(value, &entry.scr_msg),
                        location,
                    )?;
                }
                continue;
            }
            if entry.message != entry.scr_msg {
                modified_entries += 1;
                add_proposal(
                    &mut body_proposals,
                    entry.cstr_id,
                    entry.message.clone(),
                    location.clone(),
                )?;
            }
        }
    }
    let expected_keys = expected.keys().cloned().collect::<BTreeSet<_>>();
    if actual != expected_keys {
        let missing = expected_keys
            .difference(&actual)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let extra = actual
            .difference(&expected_keys)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "翻译文件清单不完整。缺少: [{}]，额外: [{}]",
            missing, extra
        ));
    }
    let mut patches = BTreeMap::new();
    for (id, proposal) in body_proposals.into_iter().chain(name_proposals.into_iter()) {
        if bundle.cstr[id].text != proposal.value {
            if let Some(previous) = patches.insert(id, proposal.value.clone()) {
                if previous != proposal.value {
                    return Err(format!("CSTR[{}] 同时被正文和姓名提出不同译文", id));
                }
            }
        }
    }
    Ok(PatchPlan {
        report: InjectReport {
            json_files: files.len(),
            json_entries,
            patched: patches.len(),
            unchanged: json_entries.saturating_sub(modified_entries),
            warnings: 0,
            output: PathBuf::new(),
        },
        patches,
    })
}

pub fn preview_inject(options: &InjectOptions) -> Result<InjectReport, BundleError> {
    let workspace = options
        .workspace
        .canonicalize()
        .map_err(|e| format!("工作区不可用: {}", e))?;
    let workspace_manifest = read_workspace(&workspace)?;
    let source_root = options.source.clone().map(Ok).unwrap_or_else(|| {
        resolve_workspace_role(&workspace, &workspace_manifest.roles.source_root)
    })?;
    let translation_root = options.translations.clone().map(Ok).unwrap_or_else(|| {
        resolve_workspace_role(&workspace, &workspace_manifest.roles.translation_root)
    })?;
    validate_snapshot(&source_root, &workspace_manifest.snapshot_files)?;
    let bundle = Bundle::load(&source_root)?;
    if !translation_root.is_dir() {
        return Err(format!("翻译目录不存在: {}", translation_root.display()));
    }
    let mut plan = build_patch_plan(&bundle, &workspace_manifest, &translation_root)?;
    plan.report.output = options.output.clone();
    Ok(plan.report)
}

fn swap_nibbles(data: &[u8]) -> Vec<u8> {
    data.iter()
        .map(|value| (value >> 4) | ((value & 0x0f) << 4))
        .collect()
}

fn rebuild_cstr(
    bundle: &Bundle,
    patches: &BTreeMap<usize, String>,
) -> Result<(Vec<u8>, Vec<u8>), BundleError> {
    let mut table = Vec::with_capacity(bundle.cstr.len() * 8);
    let mut pool = Vec::new();
    for entry in &bundle.cstr {
        let text = patches
            .get(&entry.id)
            .map(String::as_str)
            .unwrap_or(&entry.text);
        let mut bytes = encode_cp932(text, &format!("CSTR[{}]", entry.id))?;
        bytes.push(0);
        let pool_offset = u32::try_from(pool.len())
            .map_err(|_| format!("CSTR[{}] 池 offset 超过 u32", entry.id))?;
        let entry_size = u32::try_from(bytes.len())
            .map_err(|_| format!("CSTR[{}] 编码后大小超过 u32", entry.id))?;
        table.extend_from_slice(&pool_offset.to_le_bytes());
        table.extend_from_slice(&entry_size.to_le_bytes());
        pool.extend_from_slice(&bytes);
    }
    let mut decoded = table.clone();
    decoded.extend_from_slice(&pool);
    let mut obfuscated = table;
    obfuscated.extend_from_slice(&swap_nibbles(&pool));
    Ok((obfuscated, decoded))
}

fn manifest_payload_path(root: &Path, file: &str) -> Result<PathBuf, BundleError> {
    let path = Path::new(file);
    let mut components = path.components();
    if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
        return Err(format!("manifest 段文件名不安全: {}", file));
    }
    Ok(root.join(path))
}

fn update_manifest(root: &Path, cstr_bytes: &[u8]) -> Result<(), BundleError> {
    let path = root.join("manifest.json");
    let text =
        fs::read_to_string(&path).map_err(|e| format!("读取 {} 失败: {}", path.display(), e))?;
    let mut manifest: Value =
        serde_json::from_str(&text).map_err(|e| format!("manifest.json 解析失败: {}", e))?;
    let table_size = bundle_table_size(
        fs::metadata(root.join("CSTR.bin"))
            .map_err(|e| format!("读取 CSTR.bin 大小失败: {}", e))?
            .len(),
        manifest.get("header_values"),
    )?;
    let Some(segments) = manifest.get_mut("segments").and_then(Value::as_array_mut) else {
        return Err("manifest.segments 缺失".to_string());
    };
    let mut cursor = 44u64;
    for segment in segments.iter_mut() {
        let tag = segment
            .get("tag")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let file = segment
            .get("file")
            .and_then(Value::as_str)
            .ok_or_else(|| "manifest 段缺少 file".to_string())?
            .to_string();
        let no_tag = segment
            .get("no_tag")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if let Some(object) = segment.as_object_mut() {
            object.insert("tag_offset".to_string(), Value::from(cursor));
        }
        if !no_tag {
            cursor += 4;
        }
        let payload_path = manifest_payload_path(root, &file)?;
        let payload_len = fs::metadata(&payload_path)
            .map_err(|e| format!("读取 {} 大小失败: {}", file, e))?
            .len();
        if let Some(object) = segment.as_object_mut() {
            object.insert("data_offset".to_string(), Value::from(cursor));
            object.insert("size".to_string(), Value::from(payload_len));
            if tag == "CSTR" {
                if payload_len < table_size {
                    return Err("重建后的 CSTR 小于表大小".to_string());
                }
                object.insert("cstr_table_size".to_string(), Value::from(table_size));
                object.insert(
                    "cstr_pool_offset_in_file".to_string(),
                    Value::from(cursor + table_size),
                );
                object.insert(
                    "cstr_pool_size".to_string(),
                    Value::from(payload_len - table_size),
                );
                if object.contains_key("cstr_entries_total_size") {
                    object.insert(
                        "cstr_entries_total_size".to_string(),
                        Value::from(payload_len - table_size),
                    );
                }
            }
        }
        cursor += payload_len;
    }
    if let Some(object) = manifest.as_object_mut() {
        object.insert("modified".to_string(), Value::from(true));
        object.insert(
            "modified_by".to_string(),
            Value::from("sinfonia-script-tool"),
        );
        object.insert("packed_size_estimate".to_string(), Value::from(cursor));
        object.insert(
            "cstr_sha256".to_string(),
            Value::from(sha256_bytes(cstr_bytes)),
        );
    }
    write_json(&path, &manifest)
}

fn bundle_table_size(payload_len: u64, header_values: Option<&Value>) -> Result<u64, BundleError> {
    let count = header_values
        .and_then(Value::as_array)
        .and_then(|values| values.get(9))
        .and_then(Value::as_u64)
        .ok_or_else(|| "manifest.header_values[9] 缺失".to_string())?;
    let table_size = count
        .checked_mul(8)
        .ok_or_else(|| "CSTR 表大小溢出".to_string())?;
    if payload_len < table_size {
        return Err("CSTR 载荷小于表大小".to_string());
    }
    Ok(table_size)
}

pub fn inject_workspace(options: &InjectOptions) -> Result<InjectReport, BundleError> {
    let workspace = options
        .workspace
        .canonicalize()
        .map_err(|e| format!("工作区不可用: {}", e))?;
    let workspace_manifest = read_workspace(&workspace)?;
    let source_root = options.source.clone().map(Ok).unwrap_or_else(|| {
        resolve_workspace_role(&workspace, &workspace_manifest.roles.source_root)
    })?;
    let translation_root = options.translations.clone().map(Ok).unwrap_or_else(|| {
        resolve_workspace_role(&workspace, &workspace_manifest.roles.translation_root)
    })?;
    validate_snapshot(&source_root, &workspace_manifest.snapshot_files)?;
    let bundle = Bundle::load(&source_root)?;
    if !translation_root.is_dir() {
        return Err(format!("翻译目录不存在: {}", translation_root.display()));
    }
    ensure_output_safe(&workspace, &options.output)?;
    ensure_output_safe(&source_root, &options.output)?;
    if options.output.exists() && !options.overwrite {
        return Err(format!(
            "输出已存在，使用 --overwrite 才能覆盖: {}",
            options.output.display()
        ));
    }
    let mut plan = build_patch_plan(&bundle, &workspace_manifest, &translation_root)?;
    plan.report.output = options.output.clone();
    let parent = options.output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| format!("创建输出父目录失败: {}", e))?;
    let stage = crate::workspace::stage_path(&options.output);
    if stage.exists() {
        return Err(format!("临时输出已存在: {}", stage.display()));
    }
    let result = (|| {
        let mut snapshot_files = Vec::new();
        crate::workspace::copy_tree(&source_root, &stage, &mut snapshot_files, &source_root)?;
        if !plan.patches.is_empty() {
            let (cstr, decoded) = rebuild_cstr(&bundle, &plan.patches)?;
            fs::write(stage.join("CSTR.bin"), &cstr)
                .map_err(|e| format!("写入 CSTR.bin 失败: {}", e))?;
            fs::write(stage.join("CSTR_decode.bin"), &decoded)
                .map_err(|e| format!("写入 CSTR_decode.bin 失败: {}", e))?;
            update_manifest(&stage, &cstr)?;
        }
        Ok::<(), BundleError>(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    if let Err(error) = commit_stage(&stage, &options.output, options.overwrite) {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    Ok(plan.report)
}
