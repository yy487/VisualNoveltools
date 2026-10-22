use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{PmxError, Result};
use crate::extract::collect_files;
use crate::script::{encode_cp932, parse_script, ScriptText};
use crate::text_json::{is_json, portable_relative_path, read_entries, TextEntry};

#[derive(Debug)]
pub struct InjectReport {
    pub json_files: usize,
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub copied_files: usize,
    pub output: PathBuf,
}

struct PreparedPatch {
    relative_path: PathBuf,
    data: Vec<u8>,
    json_entries: usize,
    patched: usize,
    unchanged: usize,
}

pub fn infer_source_path(json_input: &Path) -> Result<PathBuf> {
    if json_input.is_file() {
        let name = json_input
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| PmxError::new("JSON 文件名不是有效 Unicode"))?;
        let source_name = name
            .strip_suffix(".json")
            .ok_or_else(|| PmxError::new("JSON 文件名必须以 .json 结尾"))?;
        return Ok(json_input.with_file_name(source_name));
    }
    if json_input.is_dir() {
        let name = json_input
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| PmxError::new("JSON 目录名不是有效 Unicode"))?;
        let source_name = name.strip_suffix("_json").ok_or_else(|| {
            PmxError::new("无法推断源目录：JSON 目录名必须以 _json 结尾，或使用 --source")
        })?;
        return Ok(json_input.with_file_name(source_name));
    }
    Err(PmxError::new(format!(
        "JSON 输入不存在：{}",
        json_input.display()
    )))
}

pub fn default_inject_output(source: &Path) -> Result<PathBuf> {
    if source.is_file() {
        let stem = source
            .file_stem()
            .ok_or_else(|| PmxError::new("源文件缺少文件名"))?
            .to_string_lossy();
        let extension = source
            .extension()
            .map(|value| value.to_string_lossy())
            .unwrap_or_default();
        let name = if extension.is_empty() {
            format!("{stem}_injected")
        } else {
            format!("{stem}_injected.{extension}")
        };
        return Ok(source.with_file_name(name));
    }
    if source.is_dir() {
        let name = source
            .file_name()
            .ok_or_else(|| PmxError::new("源目录缺少名称"))?
            .to_string_lossy();
        return Ok(source
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!("{name}_injected")));
    }
    Err(PmxError::new(format!("源路径不存在：{}", source.display())))
}

pub fn inject_input(json_input: &Path, source: &Path, output: &Path) -> Result<InjectReport> {
    if json_input.is_file() {
        return inject_single_file(json_input, source, output);
    }
    if !json_input.is_dir() || !source.is_dir() {
        return Err(PmxError::new("目录注入要求 JSON 输入和源路径都为目录"));
    }
    if output.exists() {
        return Err(PmxError::new(format!(
            "输出目录已存在，为避免覆盖而停止：{}",
            output.display()
        )));
    }
    ensure_output_outside_source(source, output)?;

    let mut json_files = collect_files(json_input, is_json)?;
    json_files.sort_by_key(|path| path.to_string_lossy().to_lowercase());
    if json_files.is_empty() {
        return Err(PmxError::new("JSON 目录中没有 .json 文件"));
    }

    let mut prepared = Vec::with_capacity(json_files.len());
    for json_path in &json_files {
        let json_relative = json_path
            .strip_prefix(json_input)
            .map_err(|_| PmxError::new(format!("JSON 不在输入目录内：{}", json_path.display())))?;
        let source_relative = remove_json_extension(json_relative)?;
        let source_path = source.join(&source_relative);
        if !source_path.is_file() {
            return Err(PmxError::new(format!(
                "JSON 对应的源 PMX 不存在：{}",
                source_path.display()
            )));
        }
        let expected_file = portable_relative_path(&source_relative)?;
        prepared.push(prepare_patch(json_path, &source_path, expected_file)?);
    }

    let copied_files = match copy_tree(source, output) {
        Ok(count) => count,
        Err(error) => {
            let _ = fs::remove_dir_all(output);
            return Err(error);
        }
    };
    let write_result = write_prepared(output, &prepared);
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(output);
        return Err(error);
    }

    Ok(InjectReport {
        json_files: prepared.len(),
        json_entries: prepared.iter().map(|patch| patch.json_entries).sum(),
        patched: prepared.iter().map(|patch| patch.patched).sum(),
        unchanged: prepared.iter().map(|patch| patch.unchanged).sum(),
        copied_files,
        output: output.to_path_buf(),
    })
}

fn inject_single_file(json: &Path, source: &Path, output: &Path) -> Result<InjectReport> {
    if !source.is_file() {
        return Err(PmxError::new(format!(
            "源 PMX 不存在：{}",
            source.display()
        )));
    }
    if output.exists() {
        return Err(PmxError::new(format!(
            "输出文件已存在，为避免覆盖而停止：{}",
            output.display()
        )));
    }
    let file_key = source
        .file_name()
        .ok_or_else(|| PmxError::new("源 PMX 缺少文件名"))?
        .to_string_lossy()
        .into_owned();
    let patch = prepare_patch(json, source, file_key)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            PmxError::context(format!("创建输出目录失败：{}", parent.display()), error)
        })?;
    }
    fs::write(output, &patch.data)
        .map_err(|error| PmxError::context(format!("写入失败：{}", output.display()), error))?;
    Ok(InjectReport {
        json_files: 1,
        json_entries: patch.json_entries,
        patched: patch.patched,
        unchanged: patch.unchanged,
        copied_files: 0,
        output: output.to_path_buf(),
    })
}

fn prepare_patch(json: &Path, source: &Path, expected_file: String) -> Result<PreparedPatch> {
    let entries = read_entries(json)?;
    let source_data = fs::read(source)
        .map_err(|error| PmxError::context(format!("读取失败：{}", source.display()), error))?;
    let parsed = parse_script(&source_data)
        .map_err(|error| PmxError::context(format!("解析失败：{expected_file}"), error))?;
    if entries.len() != parsed.entries.len() {
        return Err(PmxError::new(format!(
            "{expected_file} JSON 条目数与源 PutText 数量不一致：json={} source={}",
            entries.len(),
            parsed.entries.len()
        )));
    }

    let mut by_index = HashMap::with_capacity(entries.len());
    for entry in entries {
        let index = entry.index;
        if by_index.insert(index, entry).is_some() {
            return Err(PmxError::new(format!(
                "{expected_file} JSON 存在重复 _index={index}"
            )));
        }
    }

    let mut replacements = Vec::new();
    let mut patched = 0usize;
    let mut unchanged = 0usize;
    for source_entry in &parsed.entries {
        let json_entry = by_index.remove(&source_entry.index).ok_or_else(|| {
            PmxError::new(format!(
                "{expected_file} JSON 缺少 _index={}",
                source_entry.index
            ))
        })?;
        validate_entry(&expected_file, source_entry, &json_entry)?;
        if json_entry.message.contains(['\r', '\n']) {
            return Err(PmxError::new(format!(
                "{expected_file} index={} message 含真实换行；脚本控制换行必须写成字面 \\n",
                source_entry.index
            )));
        }
        if json_entry.message.contains('\0') {
            return Err(PmxError::new(format!(
                "{expected_file} index={} message 含 NUL",
                source_entry.index
            )));
        }
        let encoded_message = encode_cp932(&json_entry.message).map_err(|error| {
            PmxError::new(format!(
                "{expected_file} index={} message 编码失败：{error}",
                source_entry.index
            ))
        })?;
        let mut name_changed = false;
        match (&source_entry.name, &json_entry.name) {
            (Some(source_name), Some(translated_name)) => {
                if translated_name.is_empty()
                    || translated_name.contains(['[', ']', '\r', '\n', '\0'])
                    || translated_name.contains("\\n")
                {
                    return Err(PmxError::new(format!(
                        "{expected_file} index={} name 为空或含括号、换行、NUL 等非法字符",
                        source_entry.index
                    )));
                }
                let encoded_name = encode_cp932(translated_name).map_err(|error| {
                    PmxError::new(format!(
                        "{expected_file} index={} name 编码失败：{error}",
                        source_entry.index
                    ))
                })?;
                let name_start = source_entry.name_offset.ok_or_else(|| {
                    PmxError::new(format!(
                        "{expected_file} index={} 缺少源角色名偏移",
                        source_entry.index
                    ))
                })?;
                let name_end = source_entry.name_end.ok_or_else(|| {
                    PmxError::new(format!(
                        "{expected_file} index={} 缺少源角色名结尾",
                        source_entry.index
                    ))
                })?;
                replacements.push((name_start, name_end, encoded_name));
                name_changed = translated_name != source_name;
            }
            (None, None) => {}
            (Some(_), None) => {
                return Err(PmxError::new(format!(
                    "{expected_file} index={} JSON 缺少 name",
                    source_entry.index
                )));
            }
            (None, Some(_)) => {
                return Err(PmxError::new(format!(
                    "{expected_file} index={} 无名文本不能新增 name",
                    source_entry.index
                )));
            }
        }
        if json_entry.message == source_entry.message && !name_changed {
            unchanged += 1;
        } else {
            patched += 1;
        }
        replacements.push((
            source_entry.message_offset,
            source_entry.message_end,
            encoded_message,
        ));
    }
    if !by_index.is_empty() {
        return Err(PmxError::new(format!(
            "{expected_file} JSON 含源文件不存在的 _index"
        )));
    }

    replacements.sort_by_key(|replacement| std::cmp::Reverse(replacement.0));
    let mut output = source_data;
    for (start, end, replacement) in replacements {
        output.splice(start..end, replacement);
    }

    Ok(PreparedPatch {
        relative_path: PathBuf::from(expected_file),
        data: output,
        json_entries: parsed.entries.len(),
        patched,
        unchanged,
    })
}

fn validate_entry(file: &str, source: &ScriptText, json: &TextEntry) -> Result<()> {
    if json.file.replace('\\', "/") != file.replace('\\', "/") {
        return Err(PmxError::new(format!(
            "{file} index={} _file 不匹配：{}",
            source.index, json.file
        )));
    }
    if json.scr_msg != source.message {
        return Err(PmxError::new(format!(
            "{file} index={} scr_msg 校验失败\n  json: {}\n  file: {}",
            source.index, json.scr_msg, source.message
        )));
    }
    if json.offset != source.message_offset
        || json.inst_offset != source.inst_offset
        || json.size != source.message_end - source.message_offset
        || json.line != source.line
    {
        return Err(PmxError::new(format!(
            "{file} index={} 定位元数据校验失败",
            source.index
        )));
    }
    let source_name_size = source
        .name_offset
        .zip(source.name_end)
        .map(|(start, end)| end - start);
    if json.name_offset != source.name_offset || json.name_size != source_name_size {
        return Err(PmxError::new(format!(
            "{file} index={} 角色名定位元数据校验失败",
            source.index
        )));
    }
    let expected_kind = if source.name.is_some() {
        "dialogue"
    } else {
        "monologue"
    };
    if json.kind != expected_kind
        || json.opcode != "PutText"
        || !json.encoding.eq_ignore_ascii_case("cp932")
        || json.policy != "relocate"
    {
        return Err(PmxError::new(format!(
            "{file} index={} 类型或编码元数据校验失败",
            source.index
        )));
    }
    Ok(())
}

fn remove_json_extension(path: &Path) -> Result<PathBuf> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| PmxError::new(format!("JSON 文件名无效：{}", path.display())))?;
    let source_name = name
        .strip_suffix(".json")
        .ok_or_else(|| PmxError::new(format!("JSON 文件缺少 .json 后缀：{}", path.display())))?;
    Ok(path.with_file_name(source_name))
}

fn copy_tree(source: &Path, output: &Path) -> Result<usize> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            PmxError::context(format!("创建输出父目录失败：{}", parent.display()), error)
        })?;
    }
    fs::create_dir(output).map_err(|error| {
        PmxError::context(format!("创建输出目录失败：{}", output.display()), error)
    })?;
    let mut copied = 0usize;
    let mut pending = vec![(source.to_path_buf(), output.to_path_buf())];
    while let Some((source_directory, output_directory)) = pending.pop() {
        for entry in fs::read_dir(&source_directory).map_err(|error| {
            PmxError::context(
                format!("读取源目录失败：{}", source_directory.display()),
                error,
            )
        })? {
            let entry = entry.map_err(|error| {
                PmxError::context(
                    format!("读取目录项失败：{}", source_directory.display()),
                    error,
                )
            })?;
            let file_type = entry.file_type().map_err(|error| {
                PmxError::context(
                    format!("读取文件类型失败：{}", entry.path().display()),
                    error,
                )
            })?;
            let target = output_directory.join(entry.file_name());
            if file_type.is_symlink() {
                return Err(PmxError::new(format!(
                    "不支持源目录中的符号链接：{}",
                    entry.path().display()
                )));
            }
            if file_type.is_dir() {
                fs::create_dir(&target).map_err(|error| {
                    PmxError::context(format!("创建目录失败：{}", target.display()), error)
                })?;
                pending.push((entry.path(), target));
            } else if file_type.is_file() {
                fs::copy(entry.path(), &target).map_err(|error| {
                    PmxError::context(format!("复制文件失败：{}", target.display()), error)
                })?;
                copied += 1;
            }
        }
    }
    Ok(copied)
}

fn ensure_output_outside_source(source: &Path, output: &Path) -> Result<()> {
    let source = fs::canonicalize(source).map_err(|error| {
        PmxError::context(format!("解析源目录失败：{}", source.display()), error)
    })?;
    let output_parent = output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(output_parent).map_err(|error| {
        PmxError::context(
            format!("创建输出父目录失败：{}", output_parent.display()),
            error,
        )
    })?;
    let output_parent = fs::canonicalize(output_parent).map_err(|error| {
        PmxError::context(
            format!("解析输出父目录失败：{}", output_parent.display()),
            error,
        )
    })?;
    let output_name = output
        .file_name()
        .ok_or_else(|| PmxError::new("输出目录缺少名称"))?;
    let output = output_parent.join(output_name);
    if output.starts_with(&source) {
        return Err(PmxError::new(format!(
            "输出目录不能位于源目录内部：{}",
            output.display()
        )));
    }
    Ok(())
}

fn write_prepared(output: &Path, patches: &[PreparedPatch]) -> Result<()> {
    for patch in patches {
        let target = output.join(&patch.relative_path);
        fs::write(&target, &patch.data)
            .map_err(|error| PmxError::context(format!("写入失败：{}", target.display()), error))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::text_json::write_entries;

    static NEXT_TEST_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new(label: &str) -> Self {
            let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "pmx_unpack_{label}_{}_{}",
                std::process::id(),
                sequence
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn translates_name_and_preserves_put_text_structure() {
        let directory = TestDirectory::new("translated_name");
        let source_path = directory.0.join("sample.PMX");
        let json_path = directory.0.join("sample.PMX.json");
        let source =
            encode_cp932("PutText 0,,\\i(235)[梨　花]\\n\\i(238)「きゃあ！」\\i(237)\r\n").unwrap();
        fs::write(&source_path, &source).unwrap();

        let parsed = parse_script(&source).unwrap();
        let mut entry = TextEntry::from_script("sample.PMX".to_owned(), &parsed.entries[0]);
        entry.name = Some("梨花確認".to_owned());
        entry.message = "「確認メッセージ」".to_owned();
        write_entries(&json_path, &[entry]).unwrap();

        let patch = prepare_patch(&json_path, &source_path, "sample.PMX".to_owned()).unwrap();
        let expected =
            encode_cp932("PutText 0,,\\i(235)[梨花確認]\\n\\i(238)「確認メッセージ」\\i(237)\r\n")
                .unwrap();
        assert_eq!(patch.data, expected);
        assert_eq!(patch.patched, 1);
        assert_eq!(patch.unchanged, 0);
    }

    #[test]
    fn counts_name_only_change_as_patched() {
        let directory = TestDirectory::new("name_only");
        let source_path = directory.0.join("sample.PMX");
        let json_path = directory.0.join("sample.PMX.json");
        let source = encode_cp932("PutText 0,,[静　麻]\\n「いい天気だ」\r\n").unwrap();
        fs::write(&source_path, &source).unwrap();

        let parsed = parse_script(&source).unwrap();
        let mut entry = TextEntry::from_script("sample.PMX".to_owned(), &parsed.entries[0]);
        entry.name = Some("静麻テスト".to_owned());
        write_entries(&json_path, &[entry]).unwrap();

        let patch = prepare_patch(&json_path, &source_path, "sample.PMX".to_owned()).unwrap();
        let expected = encode_cp932("PutText 0,,[静麻テスト]\\n「いい天気だ」\r\n").unwrap();
        assert_eq!(patch.data, expected);
        assert_eq!(patch.patched, 1);
        assert_eq!(patch.unchanged, 0);
    }
}
