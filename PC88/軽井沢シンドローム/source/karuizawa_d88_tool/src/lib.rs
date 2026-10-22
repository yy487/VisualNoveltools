#![forbid(unsafe_code)]

pub mod basic;
pub mod font;
pub mod n88;
pub mod text_compression;
pub mod workflow;

use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub type Result<T> = std::result::Result<T, String>;

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:X}", hasher.finalize())
}

pub fn read_file(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|error| format!("无法读取 {}: {error}", path.display()))
}

pub fn ensure_output_available(path: &Path, overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        return Err(format!(
            "输出已存在：{}；如需替换请显式启用 overwrite",
            path.display()
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("输出路径没有父目录：{}", path.display()))?;
    if !parent.exists() {
        return Err(format!("输出父目录不存在：{}", parent.display()));
    }
    Ok(())
}

pub fn staging_path(output: &Path, label: &str) -> Result<PathBuf> {
    let parent = output
        .parent()
        .ok_or_else(|| format!("输出路径没有父目录：{}", output.display()))?;
    let name = output
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("输出名称无法表示为 Unicode：{}", output.display()))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("系统时钟无效：{error}"))?
        .as_nanos();
    Ok(parent.join(format!(".{name}.{label}.{}.{}", std::process::id(), nonce)))
}

pub fn commit_directory(staging: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if !output.exists() {
        return fs::rename(staging, output).map_err(|error| {
            format!(
                "无法提交 {} -> {}: {error}",
                staging.display(),
                output.display()
            )
        });
    }
    if !overwrite {
        return Err(format!("提交时输出已出现：{}", output.display()));
    }
    let backup = staging_path(output, "backup")?;
    fs::rename(output, &backup)
        .map_err(|error| format!("无法暂存已有输出 {}: {error}", output.display()))?;
    match fs::rename(staging, output) {
        Ok(()) => {
            fs::remove_dir_all(&backup).map_err(|error| {
                format!(
                    "新输出已提交，但无法清理旧输出 {}: {error}",
                    backup.display()
                )
            })?;
            Ok(())
        }
        Err(error) => {
            let restore = fs::rename(&backup, output);
            Err(match restore {
                Ok(()) => format!("提交新输出失败，旧输出已恢复：{error}"),
                Err(restore_error) => format!(
                    "提交新输出失败且旧输出恢复失败：{error}; 恢复错误：{restore_error}; 旧输出位于 {}",
                    backup.display()
                ),
            })
        }
    }
}
mod large_text;
