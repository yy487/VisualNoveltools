mod png;
mod vsp;

pub use vsp::{decode_vsp, looks_like_vsp, DecodedVsp, VspHeader};

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};

type Result<T> = std::result::Result<T, String>;

const MANIFEST_FORMAT: &str = "funnybee-vsp-png-manifest-v1";

#[derive(Clone, Copy, Debug, Default)]
pub struct ConvertOptions {
    pub overwrite: bool,
}

#[derive(Clone, Debug)]
pub struct ConvertReport {
    pub scanned_files: usize,
    pub converted_images: usize,
    pub skipped_files: usize,
    pub warnings: Vec<String>,
    pub output: PathBuf,
}

#[derive(Serialize)]
struct Manifest {
    _format: String,
    tool_version: String,
    source: String,
    scanned_files: usize,
    converted_images: usize,
    skipped_files: usize,
    warnings: Vec<String>,
    images: Vec<ImageRecord>,
}

#[derive(Serialize)]
struct ImageRecord {
    source_file: String,
    png_file: String,
    source_bytes: usize,
    source_sha256: String,
    stream_bytes: usize,
    trailing_bytes: usize,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    reserved: u8,
    palette_bank: u8,
    palette_rgb: [[u8; 3]; 16],
}

pub fn convert_recursive(
    input: &Path,
    output: &Path,
    options: ConvertOptions,
) -> Result<ConvertReport> {
    let metadata = fs::symlink_metadata(input)
        .map_err(|error| format!("无法读取输入路径 {}: {error}", input.display()))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("拒绝符号链接输入: {}", input.display()));
    }
    if metadata.is_file() {
        convert_file(input, output, options)
    } else if metadata.is_dir() {
        convert_directory(input, output, options)
    } else {
        Err(format!("输入不是普通文件或目录: {}", input.display()))
    }
}

fn convert_file(input: &Path, output: &Path, options: ConvertOptions) -> Result<ConvertReport> {
    validate_distinct_paths(input, output, false)?;
    if output.exists() && !options.overwrite {
        return Err(format!("输出已存在，需明确允许覆盖: {}", output.display()));
    }
    if output.is_dir() {
        return Err(format!("单文件输出不能是目录: {}", output.display()));
    }
    let source = fs::read(input)
        .map_err(|error| format!("无法读取输入文件 {}: {error}", input.display()))?;
    let image = decode_vsp(&source)
        .map_err(|error| format!("{} 不是有效 VSP: {error}", input.display()))?;
    let png = png::encode_png(&image)?;
    transactional_write_file(output, options.overwrite, &png)?;
    Ok(ConvertReport {
        scanned_files: 1,
        converted_images: 1,
        skipped_files: 0,
        warnings: Vec::new(),
        output: output.to_path_buf(),
    })
}

fn convert_directory(
    input: &Path,
    output: &Path,
    options: ConvertOptions,
) -> Result<ConvertReport> {
    validate_distinct_paths(input, output, true)?;
    if output.exists() && !options.overwrite {
        return Err(format!(
            "输出目录已存在，需明确允许覆盖: {}",
            output.display()
        ));
    }
    if output.exists() && !output.is_dir() {
        return Err(format!("目录输出路径是文件: {}", output.display()));
    }

    let files = collect_files(input)?;
    let scanned_files = files.len();
    let mut skipped_files = 0usize;
    let mut warnings = Vec::new();
    let mut images = Vec::new();
    let mut prepared = BTreeMap::<PathBuf, Vec<u8>>::new();
    for path in files {
        let relative = path
            .strip_prefix(input)
            .map_err(|error| format!("无法取得相对路径 {}: {error}", path.display()))?;
        validate_relative_path(relative)?;
        let source = fs::read(&path)
            .map_err(|error| format!("无法读取候选文件 {}: {error}", path.display()))?;
        if !looks_like_vsp(&source) {
            skipped_files += 1;
            continue;
        }
        let image = match decode_vsp(&source) {
            Ok(image) => image,
            Err(error) => {
                warnings.push(format!("VSP 候选解码失败 {}: {error}", path.display()));
                skipped_files += 1;
                continue;
            }
        };
        let output_relative = png_relative_path(relative)?;
        let png = png::encode_png(&image)?;
        if prepared.insert(output_relative.clone(), png).is_some() {
            return Err(format!(
                "两个输入映射到同一 PNG: {}",
                output_relative.display()
            ));
        }
        images.push(ImageRecord {
            source_file: manifest_path(relative)?,
            png_file: manifest_path(&output_relative)?,
            source_bytes: source.len(),
            source_sha256: sha256_hex(&source),
            stream_bytes: image.consumed_bytes,
            trailing_bytes: source.len().saturating_sub(image.consumed_bytes),
            x: image.header.x,
            y: image.header.y,
            width: image.header.width,
            height: image.header.height,
            reserved: image.header.reserved,
            palette_bank: image.header.palette_bank,
            palette_rgb: image.header.palette,
        });
    }
    let manifest = Manifest {
        _format: MANIFEST_FORMAT.to_string(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        source: input
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| input.display().to_string()),
        scanned_files,
        converted_images: images.len(),
        skipped_files,
        warnings: warnings.clone(),
        images,
    };
    let mut manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| format!("无法生成 VSP 清单: {error}"))?;
    manifest_bytes.push(b'\n');
    prepared.insert(PathBuf::from("_vsp_manifest.json"), manifest_bytes);
    transactional_write_directory(output, options.overwrite, &prepared)?;
    Ok(ConvertReport {
        scanned_files,
        converted_images: manifest.converted_images,
        skipped_files,
        warnings,
        output: output.to_path_buf(),
    })
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        let mut entries = fs::read_dir(&directory)
            .map_err(|error| format!("无法读取目录 {}: {error}", directory.display()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| format!("无法枚举目录 {}: {error}", directory.display()))?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries.into_iter().rev() {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("无法读取路径属性 {}: {error}", path.display()))?;
            if metadata.file_type().is_symlink() {
                return Err(format!("递归输入含符号链接，拒绝继续: {}", path.display()));
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file() {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn png_relative_path(relative: &Path) -> Result<PathBuf> {
    validate_relative_path(relative)?;
    let file_name = relative
        .file_name()
        .ok_or_else(|| format!("输入路径没有文件名: {}", relative.display()))?;
    let mut output_name = file_name.to_os_string();
    output_name.push(".png");
    Ok(relative.with_file_name(output_name))
}

fn validate_relative_path(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(format!("路径不是安全相对路径: {}", path.display()));
    }
    for component in path.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(format!("路径包含不安全分量: {}", path.display()));
        }
    }
    Ok(())
}

fn manifest_path(path: &Path) -> Result<String> {
    validate_relative_path(path)?;
    let parts = path
        .components()
        .map(|component| match component {
            Component::Normal(value) => Ok(value.to_string_lossy().into_owned()),
            _ => Err(format!("路径包含不安全分量: {}", path.display())),
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(parts.join("/"))
}

fn validate_distinct_paths(input: &Path, output: &Path, directory: bool) -> Result<()> {
    let input = fs::canonicalize(input)
        .map_err(|error| format!("无法解析输入路径 {}: {error}", input.display()))?;
    let output = resolve_output_path(output)?;
    if input == output {
        return Err("输入和输出不能是同一路径".to_string());
    }
    if directory && output.starts_with(&input) {
        return Err("输出目录不能位于输入目录内部".to_string());
    }
    Ok(())
}

fn resolve_output_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path)
            .map_err(|error| format!("无法解析输出路径 {}: {error}", path.display()));
    }
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = fs::canonicalize(parent)
        .map_err(|error| format!("无法解析输出父目录 {}: {error}", parent.display()))?;
    let name = path
        .file_name()
        .ok_or_else(|| format!("输出路径没有文件名: {}", path.display()))?;
    Ok(parent.join(name))
}

fn transactional_write_file(output: &Path, overwrite: bool, bytes: &[u8]) -> Result<()> {
    let temporary = sibling_transaction_path(output, "tmp")?;
    fs::write(&temporary, bytes)
        .map_err(|error| format!("无法写入临时 PNG {}: {error}", temporary.display()))?;
    if let Err(error) = commit_path(&temporary, output, overwrite) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}

fn transactional_write_directory(
    output: &Path,
    overwrite: bool,
    files: &BTreeMap<PathBuf, Vec<u8>>,
) -> Result<()> {
    let staging = sibling_transaction_path(output, "tmp")?;
    fs::create_dir(&staging)
        .map_err(|error| format!("无法创建事务目录 {}: {error}", staging.display()))?;
    let result = (|| {
        for (relative, bytes) in files {
            validate_relative_path(relative)?;
            let destination = staging.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("无法创建输出子目录 {}: {error}", parent.display()))?;
            }
            fs::write(&destination, bytes)
                .map_err(|error| format!("无法写入 {}: {error}", destination.display()))?;
        }
        commit_path(&staging, output, overwrite)
    })();
    if result.is_err() && staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn sibling_transaction_path(output: &Path, role: &str) -> Result<PathBuf> {
    let parent = output
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.exists() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("无法创建输出父目录 {}: {error}", parent.display()))?;
    }
    let name = output
        .file_name()
        .ok_or_else(|| format!("输出路径没有文件名: {}", output.display()))?;
    let mut temporary_name = OsString::from(".");
    temporary_name.push(name);
    temporary_name.push(format!(".{role}-{}", std::process::id()));
    let temporary = parent.join(temporary_name);
    if temporary.exists() {
        return Err(format!("事务路径已存在: {}", temporary.display()));
    }
    Ok(temporary)
}

fn commit_path(staging: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if !output.exists() {
        return fs::rename(staging, output)
            .map_err(|error| format!("无法提交输出 {}: {error}", output.display()));
    }
    if !overwrite {
        return Err(format!("输出已存在: {}", output.display()));
    }
    let backup = sibling_transaction_path(output, "backup")?;
    fs::rename(output, &backup)
        .map_err(|error| format!("无法备份已有输出 {}: {error}", output.display()))?;
    if let Err(error) = fs::rename(staging, output) {
        let _ = fs::rename(&backup, output);
        return Err(format!("提交输出失败，已尝试恢复旧输出: {error}"));
    }
    if backup.is_dir() {
        fs::remove_dir_all(&backup)
            .map_err(|error| format!("输出已提交但无法删除备份 {}: {error}", backup.display()))?;
    } else {
        fs::remove_file(&backup)
            .map_err(|error| format!("输出已提交但无法删除备份 {}: {error}", backup.display()))?;
    }
    Ok(())
}

fn sha256_hex(source: &[u8]) -> String {
    let digest = Sha256::digest(source);
    digest.iter().map(|byte| format!("{byte:02X}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_png_without_discarding_original_extension() {
        assert_eq!(
            png_relative_path(Path::new("nested/member_001.bin")).unwrap(),
            PathBuf::from("nested/member_001.bin.png")
        );
    }

    #[test]
    fn rejects_parent_components() {
        assert!(validate_relative_path(Path::new("safe/member.bin")).is_ok());
        assert!(validate_relative_path(Path::new("../escape.bin")).is_err());
    }
}
