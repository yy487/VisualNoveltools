use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{PmxError, Result};
use crate::script::parse_script;
use crate::text_json::{is_pmx, portable_relative_path, write_entries, TextEntry};

#[derive(Debug)]
pub struct ExtractWarning {
    pub file: String,
    pub line: usize,
    pub message: String,
}

#[derive(Debug)]
pub struct ExtractReport {
    pub scanned_files: usize,
    pub json_files: usize,
    pub extracted_entries: usize,
    pub warnings: Vec<ExtractWarning>,
    pub output: PathBuf,
}

pub fn default_extract_output(input: &Path) -> Result<PathBuf> {
    if input.is_file() {
        let file_name = input
            .file_name()
            .ok_or_else(|| PmxError::new(format!("输入路径缺少文件名：{}", input.display())))?;
        return Ok(input.with_file_name(format!("{}.json", file_name.to_string_lossy())));
    }
    if input.is_dir() {
        let name = input
            .file_name()
            .ok_or_else(|| PmxError::new(format!("输入目录缺少名称：{}", input.display())))?;
        return Ok(input
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!("{}_json", name.to_string_lossy())));
    }
    Err(PmxError::new(format!("输入不存在：{}", input.display())))
}

pub fn extract_input(input: &Path, output: &Path) -> Result<ExtractReport> {
    if input.is_file() {
        return extract_file(input, output);
    }
    if !input.is_dir() {
        return Err(PmxError::new(format!(
            "输入不是文件或目录：{}",
            input.display()
        )));
    }
    if output.exists() {
        return Err(PmxError::new(format!(
            "输出目录已存在，为避免覆盖而停止：{}",
            output.display()
        )));
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            PmxError::context(format!("创建输出父目录失败：{}", parent.display()), error)
        })?;
    }

    let mut files = collect_files(input, is_pmx)?;
    files.sort_by_key(|path| path.to_string_lossy().to_lowercase());
    fs::create_dir(output).map_err(|error| {
        PmxError::context(format!("创建输出目录失败：{}", output.display()), error)
    })?;

    let result = extract_directory_files(input, output, &files);
    if let Err(error) = result {
        let _ = fs::remove_dir_all(output);
        return Err(error);
    }
    result
}

fn extract_file(input: &Path, output: &Path) -> Result<ExtractReport> {
    if !is_pmx(input) {
        return Err(PmxError::new(format!(
            "输入文件扩展名不是 PMX：{}",
            input.display()
        )));
    }
    if output.exists() {
        return Err(PmxError::new(format!(
            "输出 JSON 已存在，为避免覆盖而停止：{}",
            output.display()
        )));
    }

    let data = fs::read(input)
        .map_err(|error| PmxError::context(format!("读取失败：{}", input.display()), error))?;
    let parsed = parse_script(&data)?;
    let file_name = input
        .file_name()
        .ok_or_else(|| PmxError::new("输入文件缺少文件名"))?
        .to_string_lossy()
        .into_owned();
    let entries: Vec<_> = parsed
        .entries
        .iter()
        .map(|entry| TextEntry::from_script(file_name.clone(), entry))
        .collect();
    write_entries(output, &entries)?;
    let warnings = parsed
        .warnings
        .into_iter()
        .map(|warning| ExtractWarning {
            file: file_name.clone(),
            line: warning.line,
            message: warning.message,
        })
        .collect();

    Ok(ExtractReport {
        scanned_files: 1,
        json_files: 1,
        extracted_entries: entries.len(),
        warnings,
        output: output.to_path_buf(),
    })
}

fn extract_directory_files(
    input: &Path,
    output: &Path,
    files: &[PathBuf],
) -> Result<ExtractReport> {
    let mut json_files = 0usize;
    let mut extracted_entries = 0usize;
    let mut warnings = Vec::new();

    for path in files {
        let relative = path
            .strip_prefix(input)
            .map_err(|_| PmxError::new(format!("文件不在输入目录内：{}", path.display())))?;
        let file_key = portable_relative_path(relative)?;
        let data = fs::read(path)
            .map_err(|error| PmxError::context(format!("读取失败：{}", path.display()), error))?;
        let parsed = parse_script(&data)
            .map_err(|error| PmxError::context(format!("解析失败：{file_key}"), error))?;
        for warning in parsed.warnings {
            warnings.push(ExtractWarning {
                file: file_key.clone(),
                line: warning.line,
                message: warning.message,
            });
        }
        if parsed.entries.is_empty() {
            continue;
        }

        let entries: Vec<_> = parsed
            .entries
            .iter()
            .map(|entry| TextEntry::from_script(file_key.clone(), entry))
            .collect();
        let json_relative = append_json_extension(relative)?;
        write_entries(&output.join(json_relative), &entries)?;
        json_files += 1;
        extracted_entries += entries.len();
    }

    Ok(ExtractReport {
        scanned_files: files.len(),
        json_files,
        extracted_entries,
        warnings,
        output: output.to_path_buf(),
    })
}

fn append_json_extension(path: &Path) -> Result<PathBuf> {
    let file_name = path
        .file_name()
        .ok_or_else(|| PmxError::new(format!("路径缺少文件名：{}", path.display())))?;
    Ok(path.with_file_name(format!("{}.json", file_name.to_string_lossy())))
}

pub(crate) fn collect_files(root: &Path, predicate: fn(&Path) -> bool) -> Result<Vec<PathBuf>> {
    let mut output = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory).map_err(|error| {
            PmxError::context(format!("读取目录失败：{}", directory.display()), error)
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                PmxError::context(format!("读取目录项失败：{}", directory.display()), error)
            })?;
            let file_type = entry.file_type().map_err(|error| {
                PmxError::context(
                    format!("读取文件类型失败：{}", entry.path().display()),
                    error,
                )
            })?;
            if file_type.is_symlink() {
                return Err(PmxError::new(format!(
                    "不支持目录中的符号链接：{}",
                    entry.path().display()
                )));
            }
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file() && predicate(&entry.path()) {
                output.push(entry.path());
            }
        }
    }
    Ok(output)
}
