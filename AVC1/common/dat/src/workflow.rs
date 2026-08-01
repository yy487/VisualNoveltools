use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use encoding_rs::SHIFT_JIS;

use crate::archive::{ArchiveBranch, decode_all, parse_archive, rebuild_archive, title_key};
use crate::manifest::Manifest;

#[derive(Clone, Debug)]
pub struct UnpackReport {
    pub branch: ArchiveBranch,
    pub entries: usize,
    pub unpacked_bytes: usize,
    pub crc64: u64,
    pub key_low32: u32,
    pub output: PathBuf,
}

#[derive(Clone, Debug)]
pub struct PackReport {
    pub branch: ArchiveBranch,
    pub entries: usize,
    pub input_bytes: usize,
    pub output_bytes: usize,
    pub key_low32: u32,
    pub output: PathBuf,
}

#[derive(Clone, Debug)]
pub struct VerifyReport {
    pub branch: ArchiveBranch,
    pub entries: usize,
    pub unpacked_bytes: usize,
    pub index_end: usize,
    pub data_base: usize,
    pub opaque_bytes: usize,
    pub crc64: u64,
    pub key_low32: u32,
}

#[derive(Clone, Debug)]
pub struct RoundtripReport {
    pub branch: ArchiveBranch,
    pub entries: usize,
    pub internal_exact: bool,
    pub archive_byte_exact: bool,
    pub original_bytes: usize,
    pub rebuilt_bytes: usize,
    pub key_low32: u32,
}

pub fn default_unpack_output(input: &Path) -> Result<PathBuf> {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let stem = input
        .file_stem()
        .and_then(OsStr::to_str)
        .context("输入文件名不是有效 Unicode")?;
    Ok(parent.join(format!("{stem}_unpacked")))
}

pub fn unpack(input: &Path, output: Option<&Path>, game_title: &str) -> Result<UnpackReport> {
    let output = match output {
        Some(path) => path.to_path_buf(),
        None => default_unpack_output(input)?,
    };
    if output.exists() {
        bail!("输出目录已存在，拒绝覆盖: {}", output.display());
    }

    let data = fs::read(input).with_context(|| format!("读取失败: {}", input.display()))?;
    let archive = parse_archive(&data).with_context(|| format!("解析失败: {}", input.display()))?;
    let (crc64, key_low32) = title_key(game_title)?;
    let plain_entries = decode_all(&data, &archive, key_low32)
        .context("完整解包失败；请确认游戏名与 EXE 内使用的标题字符串完全一致")?;
    let file_names = make_file_names(&archive.entries, &plain_entries);
    let source_file = input
        .file_name()
        .and_then(OsStr::to_str)
        .context("输入文件名不是有效 Unicode")?
        .to_owned();
    let manifest = Manifest::new(
        source_file,
        &archive,
        game_title.to_owned(),
        crc64,
        key_low32,
        &file_names,
        &plain_entries,
    )?;

    write_unpack_transactional(&output, &manifest, &file_names, &plain_entries)?;
    Ok(UnpackReport {
        branch: archive.branch,
        entries: archive.entries.len(),
        unpacked_bytes: plain_entries.iter().map(Vec::len).sum(),
        crc64,
        key_low32,
        output,
    })
}

pub fn pack(
    input_dir: &Path,
    output: Option<&Path>,
    preserve_capacity: bool,
) -> Result<PackReport> {
    let manifest_path = input_dir.join("manifest.json");
    let manifest_text = fs::read_to_string(&manifest_path)
        .with_context(|| format!("读取失败: {}", manifest_path.display()))?;
    let manifest: Manifest = serde_json::from_str(&manifest_text)
        .with_context(|| format!("JSON 解析失败: {}", manifest_path.display()))?;
    let (template, key_low32) = manifest.validate_and_template()?;

    let files_dir = input_dir.join("files");
    let mut plain_entries = Vec::with_capacity(manifest.entries.len());
    for entry in &manifest.entries {
        let path = files_dir.join(&entry.file);
        plain_entries.push(
            fs::read(&path).with_context(|| format!("读取明文条目失败: {}", path.display()))?,
        );
    }

    let (rebuilt, _) = rebuild_archive(&template, &plain_entries, key_low32, preserve_capacity)?;
    let output = match output {
        Some(path) => path.to_path_buf(),
        None => default_pack_output(input_dir, &manifest.source_file),
    };
    write_file_transactional(&output, &rebuilt)?;

    Ok(PackReport {
        branch: template.branch,
        entries: template.entries.len(),
        input_bytes: plain_entries.iter().map(Vec::len).sum(),
        output_bytes: rebuilt.len(),
        key_low32,
        output,
    })
}

pub fn verify(input: &Path, game_title: &str) -> Result<VerifyReport> {
    let data = fs::read(input).with_context(|| format!("读取失败: {}", input.display()))?;
    let archive = parse_archive(&data).with_context(|| format!("解析失败: {}", input.display()))?;
    let (crc64, key_low32) = title_key(game_title)?;
    let plains = decode_all(&data, &archive, key_low32)
        .context("完整验证失败；请确认游戏名与 EXE 内使用的标题字符串完全一致")?;
    let data_base = archive
        .entries
        .iter()
        .map(|entry| entry.offset as usize)
        .min()
        .unwrap_or(archive.index_end);
    let opaque_bytes = archive
        .layout
        .iter()
        .map(|item| item.gap_before.len())
        .sum::<usize>()
        + archive.trailing_data.len();
    Ok(VerifyReport {
        branch: archive.branch,
        entries: archive.entries.len(),
        unpacked_bytes: plains.iter().map(Vec::len).sum(),
        index_end: archive.index_end,
        data_base,
        opaque_bytes,
        crc64,
        key_low32,
    })
}

pub fn roundtrip(input: &Path, game_title: &str) -> Result<RoundtripReport> {
    let data = fs::read(input).with_context(|| format!("读取失败: {}", input.display()))?;
    let archive = parse_archive(&data).with_context(|| format!("解析失败: {}", input.display()))?;
    let (_, key_low32) = title_key(game_title)?;
    let plain_entries =
        decode_all(&data, &archive, key_low32).context("原始封包解码失败；请确认游戏名")?;
    let (rebuilt, _) = rebuild_archive(&archive, &plain_entries, key_low32, true)?;
    let rebuilt_archive = parse_archive(&rebuilt).context("重建封包解析失败")?;
    let rebuilt_plain =
        decode_all(&rebuilt, &rebuilt_archive, key_low32).context("重建封包解码失败")?;
    Ok(RoundtripReport {
        branch: archive.branch,
        entries: archive.entries.len(),
        internal_exact: plain_entries == rebuilt_plain,
        archive_byte_exact: data == rebuilt,
        original_bytes: data.len(),
        rebuilt_bytes: rebuilt.len(),
        key_low32,
    })
}

fn default_pack_output(input_dir: &Path, source_file: &str) -> PathBuf {
    let parent = input_dir.parent().unwrap_or_else(|| Path::new("."));
    let source = Path::new(source_file);
    let stem = source
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("script");
    let extension = source
        .extension()
        .and_then(OsStr::to_str)
        .map(|ext| format!(".{ext}"))
        .unwrap_or_else(|| ".dat".to_owned());
    parent.join(format!("{stem}_packed{extension}"))
}

fn make_file_names(entries: &[crate::archive::ArchiveEntry], plains: &[Vec<u8>]) -> Vec<String> {
    let mut used = HashSet::new();
    entries
        .iter()
        .map(|entry| {
            let label = first_label(&plains[entry.index]);
            let base = match label {
                Some(label) => format!("{:04}_{}.txt", entry.index, sanitize_name(&label)),
                None => format!(
                    "{:04}_{:08X}_{:08X}.txt",
                    entry.index, entry.key_hi, entry.key_lo
                ),
            };
            deduplicate_name(base, &mut used)
        })
        .collect()
}

fn first_label(raw: &[u8]) -> Option<String> {
    let (decoded, _, had_errors) = SHIFT_JIS.decode(raw);
    if had_errors {
        return None;
    }
    decoded.lines().find_map(|line| {
        let trimmed = line.trim();
        let rest = trimmed.strip_prefix('*')?;
        let label: String = rest
            .chars()
            .take_while(|ch| !matches!(ch, ':' | ' ' | '/' | '\t' | '\r' | '\n'))
            .collect();
        (!label.is_empty()).then_some(label)
    })
}

fn sanitize_name(text: &str) -> String {
    let mut output = String::new();
    let mut previous_underscore = false;
    for ch in text.trim().chars().take(80) {
        let invalid = ch.is_control()
            || ch.is_whitespace()
            || matches!(ch, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|');
        let mapped = if invalid { '_' } else { ch };
        if mapped == '_' && previous_underscore {
            continue;
        }
        previous_underscore = mapped == '_';
        output.push(mapped);
    }
    let cleaned = output.trim_matches(['.', '_', ' ']);
    if cleaned.is_empty() {
        "entry".to_owned()
    } else {
        cleaned.to_owned()
    }
}

fn deduplicate_name(base: String, used: &mut HashSet<String>) -> String {
    if used.insert(base.to_lowercase()) {
        return base;
    }
    let path = Path::new(&base);
    let stem = path.file_stem().and_then(OsStr::to_str).unwrap_or("entry");
    let suffix = path.extension().and_then(OsStr::to_str).unwrap_or("txt");
    for number in 1_u32.. {
        let candidate = format!("{stem}_{number}.{suffix}");
        if used.insert(candidate.to_lowercase()) {
            return candidate;
        }
    }
    unreachable!()
}

fn write_unpack_transactional(
    output: &Path,
    manifest: &Manifest,
    file_names: &[String],
    plain_entries: &[Vec<u8>],
) -> Result<()> {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .with_context(|| format!("创建输出父目录失败: {}", parent.display()))?;
    let staging = unique_staging_path(output, true)?;
    fs::create_dir(&staging).with_context(|| format!("创建临时目录失败: {}", staging.display()))?;

    let result = (|| -> Result<()> {
        let files_dir = staging.join("files");
        fs::create_dir(&files_dir)
            .with_context(|| format!("创建 files 目录失败: {}", files_dir.display()))?;
        for (name, plain) in file_names.iter().zip(plain_entries) {
            let path = files_dir.join(name);
            fs::write(&path, plain).with_context(|| format!("写入失败: {}", path.display()))?;
        }
        let manifest_path = staging.join("manifest.json");
        fs::write(&manifest_path, manifest.to_pretty_json()?)
            .with_context(|| format!("写入失败: {}", manifest_path.display()))?;
        fs::rename(&staging, output).with_context(|| {
            format!(
                "提交输出目录失败: {} -> {}",
                staging.display(),
                output.display()
            )
        })?;
        Ok(())
    })();

    if result.is_err() && staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn write_file_transactional(output: &Path, data: &[u8]) -> Result<()> {
    if output.exists() {
        bail!("输出文件已存在，拒绝覆盖: {}", output.display());
    }
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .with_context(|| format!("创建输出父目录失败: {}", parent.display()))?;
    let staging = unique_staging_path(output, false)?;
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staging)
            .with_context(|| format!("创建临时文件失败: {}", staging.display()))?;
        file.write_all(data)
            .with_context(|| format!("写入临时文件失败: {}", staging.display()))?;
        file.sync_all()
            .with_context(|| format!("同步临时文件失败: {}", staging.display()))?;
        drop(file);
        fs::rename(&staging, output).with_context(|| {
            format!(
                "提交输出文件失败: {} -> {}",
                staging.display(),
                output.display()
            )
        })?;
        Ok(())
    })();
    if result.is_err() && staging.exists() {
        let _ = fs::remove_file(&staging);
    }
    result
}

fn unique_staging_path(output: &Path, directory: bool) -> Result<PathBuf> {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let name = output
        .file_name()
        .and_then(OsStr::to_str)
        .context("输出名称不是有效 Unicode")?;
    let kind = if directory { "dir" } else { "file" };
    for counter in 0..100_u32 {
        let candidate = parent.join(format!(
            ".{name}.acv1_dat_tool.{}.{}.{}.tmp",
            std::process::id(),
            kind,
            counter
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    bail!("无法为输出创建唯一临时路径: {}", output.display())
}

#[allow(dead_code)]
fn _ensure_file_is_sync(file: &File) -> Result<()> {
    file.sync_all().context("同步文件失败")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_name_removes_windows_metacharacters() {
        assert_eq!(sanitize_name(" a:*?b "), "a_b");
    }

    #[test]
    fn duplicate_names_are_stable() {
        let mut used = HashSet::new();
        assert_eq!(deduplicate_name("x.txt".to_owned(), &mut used), "x.txt");
        assert_eq!(deduplicate_name("x.txt".to_owned(), &mut used), "x_1.txt");
    }

    #[test]
    fn existing_output_is_not_overwritten() {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("out.dat");
        fs::write(&output, b"original").unwrap();
        assert!(write_file_transactional(&output, b"new").is_err());
        assert_eq!(fs::read(output).unwrap(), b"original");
    }
}
