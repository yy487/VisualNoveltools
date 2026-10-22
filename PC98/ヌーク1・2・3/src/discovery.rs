use crate::model::GameId;
use anyhow::{bail, Context, Result};
use pc98_fdi_unpack::{extract_fdi_files, FdiFileData};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub struct DiskSource {
    pub path: PathBuf,
    pub relative: String,
    pub sha256: String,
    pub files: Vec<FdiFileData>,
}

pub fn discover(input: &Path, excluded: &Path) -> Result<BTreeMap<GameId, Vec<DiskSource>>> {
    let input = fs::canonicalize(input)
        .with_context(|| format!("无法访问输入根目录 {}", input.display()))?;
    let excluded = absolute_path(excluded)?;
    let excluded = exclusion_root(&input, &excluded);
    let mut parents = BTreeMap::<PathBuf, Vec<PathBuf>>::new();
    let walker = WalkDir::new(&input)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !is_excluded(entry, &excluded));
    for entry in walker {
        let entry = entry.with_context(|| format!("遍历 {} 失败", input.display()))?;
        if entry.file_type().is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|ext| ext.to_string_lossy().eq_ignore_ascii_case("fdi"))
        {
            let parent = entry
                .path()
                .parent()
                .context("FDI 没有父目录")?
                .to_path_buf();
            parents
                .entry(parent)
                .or_default()
                .push(entry.path().to_path_buf());
        }
    }
    if parents.is_empty() {
        bail!("{} 下没有发现 FDI", input.display());
    }

    let mut result = BTreeMap::<GameId, Vec<DiskSource>>::new();
    for (_parent, mut paths) in parents {
        paths.sort_by_key(|path| path.file_name().map(|n| n.to_string_lossy().to_lowercase()));
        let mut prepared = Vec::new();
        let mut members = BTreeSet::new();
        for path in paths {
            let bytes = fs::read(&path).with_context(|| format!("读取 {} 失败", path.display()))?;
            let files =
                extract_fdi_files(&bytes, &path.to_string_lossy()).map_err(anyhow::Error::msg)?;
            for file in &files {
                members.insert(basename(&file.path).to_ascii_uppercase());
            }
            let relative = slash_path(path.strip_prefix(&input).context("FDI 不在输入根目录内")?);
            prepared.push((path, relative, sha256_hex(&bytes), files));
        }
        let game = if members.contains("NOOCH.EXE") && members.contains("TEXT.MAP") {
            Some(GameId::Nooch)
        } else if members.contains("NOOCH2.EXE") && members.contains("SYSTEM.MAC") {
            Some(GameId::Nooch2)
        } else if members.contains("NK3.EXE") && members.contains("TEXTMAP.DAT") {
            Some(GameId::Nooch3)
        } else {
            None
        };
        let Some(game) = game else { continue };
        let candidate = prepared
            .into_iter()
            .map(|(path, relative, sha256, files)| DiskSource {
                path,
                relative,
                sha256,
                files,
            })
            .collect::<Vec<_>>();
        if let Some(existing) = result.get(&game) {
            if disk_hash_set(existing) != disk_hash_set(&candidate) {
                bail!(
                    "发现了两组内容不同的 {} FDI；请缩小输入根目录",
                    game.as_str()
                );
            }
            if group_depth(&candidate) < group_depth(existing) {
                result.insert(game, candidate);
            }
        } else {
            result.insert(game, candidate);
        }
    }
    for game in GameId::ALL {
        if !result.contains_key(&game) {
            bail!("没有识别到 {} 的完整 FDI 目录", game.as_str());
        }
    }
    Ok(result)
}

fn disk_hash_set(disks: &[DiskSource]) -> BTreeSet<&str> {
    disks.iter().map(|disk| disk.sha256.as_str()).collect()
}

fn group_depth(disks: &[DiskSource]) -> usize {
    disks
        .first()
        .map(|disk| Path::new(&disk.relative).components().count())
        .unwrap_or(usize::MAX)
}

pub fn safe_join(root: &Path, relative: &str) -> Result<PathBuf> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        bail!("FDI 内含不安全路径 {relative:?}");
    }
    Ok(root.join(path))
}

pub fn basename(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

pub fn sha256_hex(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

pub fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn disk_directory(index: usize, path: &Path) -> String {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let safe = stem
        .chars()
        .map(|ch| {
            if ch.is_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!("disk_{:02}_{safe}", index + 1)
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path).with_context(|| format!("无法规范化 {}", path.display()));
    }
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn exclusion_root(input: &Path, excluded: &Path) -> PathBuf {
    let Ok(relative) = excluded.strip_prefix(input) else {
        return excluded.to_path_buf();
    };
    match relative.components().next() {
        Some(Component::Normal(first)) => input.join(first),
        _ => excluded.to_path_buf(),
    }
}

fn is_excluded(entry: &DirEntry, excluded: &Path) -> bool {
    entry.path().starts_with(excluded)
}
