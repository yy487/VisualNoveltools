use crate::bundle::{Bundle, BundleError};
use crate::extract::{ExtractOptions, ExtractPlan, ExtractReport};
use crate::json_model::{
    LogicalSourceFileManifest, SnapshotFileManifest, TranslationFileManifest, WorkspaceManifest,
    WorkspaceRoles, WorkspaceStats, WORKSPACE_FORMAT,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Component;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn relative_slashes(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn sha256_bytes(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    digest.iter().map(|byte| format!("{:02x}", byte)).collect()
}

pub(crate) fn validate_manifest_relative_path(relative: &str) -> Result<PathBuf, BundleError> {
    let normalized = relative.replace('/', "\\");
    let path = PathBuf::from(&normalized);
    if normalized.is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("工作区清单含非法相对路径: {}", relative));
    }
    Ok(path)
}

pub(crate) fn resolve_workspace_role(
    workspace: &Path,
    relative: &str,
) -> Result<PathBuf, BundleError> {
    Ok(workspace.join(validate_manifest_relative_path(relative)?))
}

fn collect_snapshot_files(
    root: &Path,
    current: &Path,
    result: &mut BTreeMap<String, (u64, String)>,
) -> Result<(), BundleError> {
    let file_type = reject_symlink(current)?;
    if file_type.is_dir() {
        for entry in
            fs::read_dir(current).map_err(|e| format!("读取 {} 失败: {}", current.display(), e))?
        {
            collect_snapshot_files(
                root,
                &entry.map_err(|e| format!("读取目录项失败: {}", e))?.path(),
                result,
            )?;
        }
        return Ok(());
    }
    if !file_type.is_file() {
        return Err(format!("不支持的源快照文件类型: {}", current.display()));
    }
    let relative = current
        .strip_prefix(root)
        .map(relative_slashes)
        .map_err(|_| format!("无法计算 {} 的相对路径", current.display()))?;
    let data = fs::read(current).map_err(|e| format!("读取 {} 失败: {}", current.display(), e))?;
    if result
        .insert(relative.clone(), (data.len() as u64, sha256_bytes(&data)))
        .is_some()
    {
        return Err(format!("源快照路径重复: {}", relative));
    }
    Ok(())
}

pub(crate) fn validate_snapshot(
    source_root: &Path,
    expected_files: &[SnapshotFileManifest],
) -> Result<(), BundleError> {
    let source_root = source_root
        .canonicalize()
        .map_err(|e| format!("源快照目录不可用: {}", e))?;
    if !source_root.is_dir() {
        return Err(format!("源快照不是目录: {}", source_root.display()));
    }

    let mut expected = BTreeMap::new();
    for file in expected_files {
        let relative_path = validate_manifest_relative_path(&file.relative_path)?;
        let relative = relative_slashes(&relative_path);
        if expected
            .insert(
                relative.clone(),
                (file.size, file.sha256.to_ascii_lowercase()),
            )
            .is_some()
        {
            return Err(format!("工作区清单源快照路径重复: {}", relative));
        }
    }

    let mut actual = BTreeMap::new();
    collect_snapshot_files(&source_root, &source_root, &mut actual)?;
    for (relative, (expected_size, expected_hash)) in &expected {
        let Some((actual_size, actual_hash)) = actual.get(relative) else {
            return Err(format!("源快照缺少文件: {}", relative));
        };
        if actual_size != expected_size {
            return Err(format!(
                "源快照文件大小变化: {}，预期 {}，实际 {}",
                relative, expected_size, actual_size
            ));
        }
        if !actual_hash.eq_ignore_ascii_case(expected_hash) {
            return Err(format!("源快照文件 SHA-256 变化: {}", relative));
        }
    }
    if let Some(extra) = actual
        .keys()
        .find(|relative| !expected.contains_key(*relative))
    {
        return Err(format!("源快照含额外文件: {}", extra));
    }
    Ok(())
}

pub(crate) fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<(), BundleError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|e| format!("序列化 {} 失败: {}", path.display(), e))?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(|e| format!("写入 {} 失败: {}", path.display(), e))
}

fn reject_symlink(path: &Path) -> Result<fs::FileType, BundleError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|e| format!("读取 {} 元数据失败: {}", path.display(), e))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("拒绝符号链接输入: {}", path.display()));
    }
    Ok(metadata.file_type())
}

pub(crate) fn copy_tree(
    source: &Path,
    destination: &Path,
    files: &mut Vec<SnapshotFileManifest>,
    prefix: &Path,
) -> Result<(), BundleError> {
    let file_type = reject_symlink(source)?;
    if file_type.is_dir() {
        fs::create_dir_all(destination)
            .map_err(|e| format!("创建 {} 失败: {}", destination.display(), e))?;
        for entry in
            fs::read_dir(source).map_err(|e| format!("读取 {} 失败: {}", source.display(), e))?
        {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let child_source = entry.path();
            let child_destination = destination.join(entry.file_name());
            copy_tree(&child_source, &child_destination, files, prefix)?;
        }
        return Ok(());
    }
    if !file_type.is_file() {
        return Err(format!("不支持的输入文件类型: {}", source.display()));
    }
    let data = fs::read(source).map_err(|e| format!("读取 {} 失败: {}", source.display(), e))?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建 {} 失败: {}", parent.display(), e))?;
    }
    fs::write(destination, &data)
        .map_err(|e| format!("复制 {} 失败: {}", destination.display(), e))?;
    let relative = source
        .strip_prefix(prefix)
        .map(relative_slashes)
        .map_err(|_| format!("无法计算 {} 的相对路径", source.display()))?;
    files.push(SnapshotFileManifest {
        relative_path: relative,
        size: data.len() as u64,
        sha256: sha256_bytes(&data),
    });
    Ok(())
}

pub(crate) fn ensure_output_safe(input: &Path, output: &Path) -> Result<(), BundleError> {
    let input = input
        .canonicalize()
        .map_err(|e| format!("输入路径不可用: {}", e))?;
    let output_existing = output.exists();
    let output = if output_existing {
        output
            .canonicalize()
            .map_err(|e| format!("输出路径不可用: {}", e))?
    } else {
        let parent = output.parent().unwrap_or_else(|| Path::new("."));
        parent
            .canonicalize()
            .map_err(|e| format!("输出父目录不可用: {}", e))?
            .join(
                output
                    .file_name()
                    .ok_or_else(|| "输出路径没有目录名".to_string())?,
            )
    };
    if output == input || output.starts_with(&input) || input.starts_with(&output) {
        return Err("输出目录不能与输入目录互相包含，也不能覆盖输入目录".to_string());
    }
    Ok(())
}

pub(crate) fn stage_path(output: &Path) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let name = output
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("workspace");
    output
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(
            ".{}.sinfonia-tmp-{}-{}",
            name,
            std::process::id(),
            stamp
        ))
}

pub(crate) fn commit_stage(
    stage: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<(), BundleError> {
    if !output.exists() {
        fs::rename(stage, output).map_err(|e| format!("提交输出目录失败: {}", e))?;
        return Ok(());
    }
    if !overwrite {
        return Err(format!("输出已存在，未覆盖: {}", output.display()));
    }
    let backup = stage.with_extension("old");
    if backup.exists() {
        return Err(format!("临时备份路径已存在: {}", backup.display()));
    }
    fs::rename(output, &backup).map_err(|e| format!("暂存旧输出失败: {}", e))?;
    match fs::rename(stage, output) {
        Ok(()) => {
            fs::remove_dir_all(&backup).map_err(|e| format!("删除旧输出备份失败: {}", e))?;
            Ok(())
        }
        Err(error) => {
            let _ = fs::rename(&backup, output);
            Err(format!("提交新输出失败: {}", error))
        }
    }
}

pub fn write_extracted_workspace(
    bundle: &Bundle,
    plan: &ExtractPlan,
    options: &ExtractOptions,
) -> Result<ExtractReport, BundleError> {
    ensure_output_safe(&bundle.root, &options.output)?;
    if options.output.exists() && !options.overwrite {
        return Err(format!(
            "输出已存在，使用 --overwrite 才能覆盖: {}",
            options.output.display()
        ));
    }
    let parent = options.output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| format!("创建输出父目录失败: {}", e))?;
    let stage = stage_path(&options.output);
    if stage.exists() {
        return Err(format!("临时输出已存在: {}", stage.display()));
    }
    let result = (|| {
        fs::create_dir_all(&stage).map_err(|e| format!("创建临时目录失败: {}", e))?;
        let source_stage = stage.join("source");
        let mut snapshot_files = Vec::new();
        copy_tree(
            &bundle.root,
            &source_stage,
            &mut snapshot_files,
            &bundle.root,
        )?;
        snapshot_files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        for planned in &plan.translation_files {
            let destination = stage.join(&planned.relative_path);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("创建 {} 失败: {}", parent.display(), e))?;
            }
            write_json(&destination, &planned.translation)?;
        }
        let logical_source_files = bundle
            .source_files
            .iter()
            .enumerate()
            .map(|(file_id, file)| LogicalSourceFileManifest {
                file_id,
                file: file.clone(),
            })
            .collect();
        let roles = WorkspaceRoles {
            source_root: "source".to_string(),
            translation_root: "translation_json".to_string(),
            script_json_root: "translation_json/scripts".to_string(),
            system_json_root: "translation_json/system".to_string(),
            names_json: "translation_json/names.json".to_string(),
        };
        let translation_files = plan
            .translation_files
            .iter()
            .map(|planned| TranslationFileManifest {
                relative_path: planned.relative_path.clone(),
                group: planned.group.clone(),
                file_id: planned.file_id,
                file: planned.file.clone(),
                entries: planned.translation.entries.len(),
            })
            .collect();
        let source_manifest_format = bundle
            .manifest
            .get("format")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let workspace = WorkspaceManifest {
            format: WORKSPACE_FORMAT.to_string(),
            version: 1,
            roles,
            source_manifest_format,
            logical_source_files,
            snapshot_files,
            translation_files,
            stats: WorkspaceStats {
                source_files: bundle.source_files.len(),
                script_json_files: plan
                    .translation_files
                    .iter()
                    .filter(|file| file.group == "script")
                    .count(),
                system_json_files: plan
                    .translation_files
                    .iter()
                    .filter(|file| file.group == "system")
                    .count(),
                extracted_entries: plan.report.extracted_entries,
                dialogue_entries: plan.report.dialogue_entries,
                choice_entries: plan.report.choice_entries,
                name_entries: plan.report.name_entries,
                warnings: plan.warnings.len(),
            },
        };
        write_json(&stage.join("workspace.json"), &workspace)?;
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
    let mut report = plan.report.clone();
    report.output = options.output.clone();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::validate_manifest_relative_path;

    #[test]
    fn accepts_normal_relative_manifest_paths() {
        let path = validate_manifest_relative_path("folder/file.bin").unwrap();
        assert_eq!(path.to_string_lossy(), "folder\\file.bin");
    }

    #[test]
    fn rejects_absolute_and_traversing_manifest_paths() {
        for path in ["", "../file.bin", "folder/../file.bin", "C:/file.bin"] {
            assert!(validate_manifest_relative_path(path).is_err(), "{path}");
        }
    }
}
