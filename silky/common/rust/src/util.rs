use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn read_utf8(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes);
    String::from_utf8(bytes.to_vec())
        .with_context(|| format!("file is not valid UTF-8: {}", path.display()))
}

pub fn pretty_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut text = serde_json::to_string_pretty(value)?;
    text.push('\n');
    Ok(text.into_bytes())
}

pub fn write_new_file(path: &Path, data: &[u8]) -> Result<()> {
    if path.exists() {
        bail!("output already exists: {}", path.display());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    file.write_all(data)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn write_new_tree(root: &Path, files: &[(PathBuf, Vec<u8>)]) -> Result<()> {
    if root.exists() {
        bail!("output directory already exists: {}", root.display());
    }
    let parent = root
        .parent()
        .ok_or_else(|| anyhow!("output directory has no parent: {}", root.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;

    let name = root
        .file_name()
        .ok_or_else(|| anyhow!("output directory has no name: {}", root.display()))?
        .to_string_lossy();
    let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temp = parent.join(format!(".{name}.silky-tmp-{}-{nonce}", std::process::id()));
    if temp.exists() {
        bail!("temporary output already exists: {}", temp.display());
    }

    let result = (|| -> Result<()> {
        fs::create_dir(&temp).with_context(|| format!("failed to create {}", temp.display()))?;
        for (relative, data) in files {
            validate_relative_path(relative)?;
            let output = temp.join(relative);
            if let Some(output_parent) = output.parent() {
                fs::create_dir_all(output_parent)
                    .with_context(|| format!("failed to create {}", output_parent.display()))?;
            }
            write_new_file(&output, data)?;
        }
        fs::rename(&temp, root).with_context(|| {
            format!(
                "failed to publish output {} -> {}",
                temp.display(),
                root.display()
            )
        })?;
        Ok(())
    })();

    if result.is_err() && temp.exists() {
        let _ = fs::remove_dir_all(&temp);
    }
    result
}

pub fn validate_relative_path(path: &Path) -> Result<()> {
    use std::path::Component;
    if path.as_os_str().is_empty() || path.is_absolute() {
        bail!("unsafe relative path: {}", path.display());
    }
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => bail!("unsafe relative path: {}", path.display()),
        }
    }
    Ok(())
}

pub fn matching_files(directory: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    if !directory.is_dir() {
        bail!("input is not a directory: {}", directory.display());
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(directory)
        .with_context(|| format!("failed to list {}", directory.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && wildcard_match(pattern, &entry.file_name().to_string_lossy())
        {
            files.push(entry.path());
        }
    }
    files.sort_by(|a, b| {
        a.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .cmp(&b.file_name().unwrap_or_default().to_string_lossy())
    });
    Ok(files)
}

pub fn wildcard_match(pattern: &str, candidate: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let candidate: Vec<char> = candidate.chars().collect();
    let mut dp = vec![vec![false; candidate.len() + 1]; pattern.len() + 1];
    dp[0][0] = true;
    for i in 1..=pattern.len() {
        if pattern[i - 1] == '*' {
            dp[i][0] = dp[i - 1][0];
        }
        for j in 1..=candidate.len() {
            dp[i][j] = match pattern[i - 1] {
                '*' => dp[i - 1][j] || dp[i][j - 1],
                '?' => dp[i - 1][j - 1],
                p => dp[i - 1][j - 1] && p.eq_ignore_ascii_case(&candidate[j - 1]),
            };
        }
    }
    dp[pattern.len()][candidate.len()]
}

pub fn file_name_string(path: &Path) -> Result<String> {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| anyhow!("path has no file name: {}", path.display()))
}

pub fn strip_known_suffix<'a>(name: &'a str, suffixes: &[&str]) -> &'a str {
    for suffix in suffixes {
        if name.len() >= suffix.len()
            && name[name.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
        {
            return &name[..name.len() - suffix.len()];
        }
    }
    name.rsplit_once('.').map_or(name, |(stem, _)| stem)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_is_case_insensitive() {
        assert!(wildcard_match("*.MES", "TEST.mes"));
        assert!(wildcard_match("A?C.*", "abc.MES"));
        assert!(!wildcard_match("*.MES", "test.json"));
    }
}
