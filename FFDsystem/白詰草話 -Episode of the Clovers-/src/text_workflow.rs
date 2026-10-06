use std::collections::{BTreeMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use vn_cli::*;

use crate::script::{self, TextKind};

const MAP_NAME: &str = "source-map.tsv";
const MAP_HEADER: &str = "source\ttranslation\tfingerprint\tentries\tchoices\tskipped";

struct ExportFile {
    relative: PathBuf,
    json: Vec<u8>,
}

struct ExportTextJob {
    preview: Preview,
    output: PathBuf,
    files: Vec<ExportFile>,
    map: Vec<u8>,
    entries: u64,
    choices: u64,
    warnings: Vec<String>,
    overwrite: bool,
}

pub struct ExportText;

impl Operation for ExportText {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "export",
            "导出脚本文本与选择项",
            vec![
                Field::new("source-dir", "解包后的脚本目录", FieldKind::Path).required(),
                Field::new("output-dir", "翻译 JSON 输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description = "从 CreateBalloon、CreateBalloonEx、CreateBalloonBie、CreateText 和 AddText 提取文本，生成逐脚本翻译 JSON。"
            .into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            parameters.set("source-dir", Value::Path(path.clone()));
            if parameters.get("output-dir").is_none() {
                parameters.set(
                    "output-dir",
                    Value::Path(
                        path.parent()
                            .unwrap_or_else(|| Path::new("."))
                            .join("translations"),
                    ),
                );
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let source = canonical_directory(parameters.path("source-dir")?, "脚本来源目录")?;
        let output = output_directory(parameters.path("output-dir")?)?;
        let mut files = Vec::new();
        let mut map = String::from(MAP_HEADER);
        let mut total_entries = 0u64;
        let mut total_choices = 0u64;
        let mut warnings = Vec::new();
        let mut source_count = 0usize;

        for path in collect_tree_files(&source)? {
            if !has_txt_extension(&path) {
                continue;
            }
            let relative_source = relative_name(&source, &path)?;
            let source_bytes = fs::read(&path)
                .map_err(|error| format!("读取脚本 {} 失败: {error}", path.display()))?;
            let parsed = script::parse(&source_bytes)
                .map_err(|error| format!("解析脚本 {relative_source} 失败: {error}"))?;
            if parsed.records.is_empty() {
                if parsed.nonliteral_text_args > 0 {
                    warnings.push(format!(
                        "{relative_source}: {} 个文本构造器使用动态文本参数，未导出",
                        parsed.nonliteral_text_args
                    ));
                }
                continue;
            }

            let translation_name = format!("{relative_source}.json");
            let translation_path = safe_relative_path(&translation_name)?;
            let entries: Vec<vn_text::Entry> = parsed
                .records
                .iter()
                .map(|record| vn_text::Entry {
                    name: None,
                    message: record.message.clone(),
                })
                .collect();
            let json = vn_text::write_template(&entries)
                .map_err(|error| format!("生成 {relative_source} 的翻译模板失败: {error}"))?;
            let choices = parsed
                .records
                .iter()
                .filter(|record| record.kind == TextKind::Choice)
                .count();
            if parsed.nonliteral_text_args > 0 {
                warnings.push(format!(
                    "{relative_source}: {} 个文本构造器使用动态文本参数，未导出",
                    parsed.nonliteral_text_args
                ));
            }
            if relative_source.contains(['\t', '\r', '\n'])
                || translation_name.contains(['\t', '\r', '\n'])
            {
                return Err(format!("文件路径不能写入映射表: {relative_source}").into());
            }
            map.push_str(&format!(
                "\n{relative_source}\t{translation_name}\t{}\t{}\t{}\t{}",
                fingerprint(&source_bytes),
                entries.len(),
                choices,
                parsed.nonliteral_text_args
            ));
            total_entries += entries.len() as u64;
            total_choices += choices as u64;
            source_count += 1;
            files.push(ExportFile {
                relative: translation_path,
                json,
            });
        }

        if source_count == 0 {
            return Err("来源目录中没有可导出的对白或选择项".into());
        }
        let preview = Preview {
            inputs: vec![source.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                format!("扫描 {source_count} 个脚本并解析静态脚本文本"),
                "清理声音/字体/打字效果标签，将 <BR> 转为换行".into(),
                "写出逐脚本 JSON 和来源映射".into(),
            ],
            details: vec![format!(
                "entries={total_entries}，choices={total_choices}，输出映射={MAP_NAME}"
            )],
        };
        Ok(Box::new(ExportTextJob {
            preview,
            output,
            files,
            map: map.into_bytes(),
            entries: total_entries,
            choices: total_choices,
            warnings,
            overwrite: parameters.flag("overwrite"),
        }))
    }
}

impl PreparedOperation for ExportTextJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        let parent = self
            .output
            .parent()
            .ok_or_else(|| "翻译输出目录缺少父目录".to_owned())?;
        let stage = TemporaryDirectory::create(parent, "repi-v2-export")?;
        for file in &self.files {
            let target = stage.path.join(&file.relative);
            create_parent(&target)?;
            write_new_file(&target, &file.json)?;
        }
        write_new_file(&stage.path.join(MAP_NAME), &self.map)?;
        progress.report("校验翻译模板并提交输出目录")?;
        let mut warnings = self.warnings;
        if let Some(warning) = commit_directory(stage, &self.output, self.overwrite)? {
            warnings.push(warning);
        }
        Ok(RunReport {
            summary: "脚本文本与选择项 JSON 导出完成；脚本来源未修改。".into(),
            totals: vec![
                ("脚本".into(), self.files.len() as u64),
                ("文本项".into(), self.entries),
                ("选择项".into(), self.choices),
            ],
            outputs: vec![self.output.clone()],
            warnings,
        })
    }
}

struct ImportFile {
    relative: PathBuf,
    bytes: Vec<u8>,
}

struct ImportTextJob {
    preview: Preview,
    output: PathBuf,
    files: Vec<ImportFile>,
    entries: u64,
    choices: u64,
    warnings: Vec<String>,
    overwrite: bool,
}

pub struct ImportText;

impl Operation for ImportText {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "import",
            "注入翻译文本",
            vec![
                Field::new("source-dir", "原始脚本目录", FieldKind::Path).required(),
                Field::new("translation-dir", "翻译 JSON 目录", FieldKind::Path).required(),
                Field::new("output-dir", "重建脚本输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "校验来源映射与 JSON 形状，将译文按原始字节定位注入副本；不改动源脚本。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            parameters.set("source-dir", Value::Path(path.clone()));
            let parent = path.parent().unwrap_or_else(|| Path::new("."));
            if parameters.get("translation-dir").is_none() {
                parameters.set("translation-dir", Value::Path(parent.join("translations")));
            }
            if parameters.get("output-dir").is_none() {
                parameters.set("output-dir", Value::Path(parent.join("rebuilt")));
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let source = canonical_directory(parameters.path("source-dir")?, "脚本来源目录")?;
        let translations = canonical_directory(parameters.path("translation-dir")?, "翻译目录")?;
        let output = output_directory(parameters.path("output-dir")?)?;
        let map_path = translations.join(MAP_NAME);
        let map_metadata = fs::symlink_metadata(&map_path)
            .map_err(|error| format!("读取来源映射 {} 失败: {error}", map_path.display()))?;
        if map_metadata.file_type().is_symlink() || !map_metadata.is_file() {
            return Err(format!("来源映射必须是普通文件: {}", map_path.display()).into());
        }
        let map_text = fs::read_to_string(&map_path)
            .map_err(|error| format!("读取来源映射 {} 失败: {error}", map_path.display()))?;
        let mut rows = parse_mapping(&map_text)?;
        let mut files = Vec::new();
        let mut total_entries = 0u64;
        let mut total_choices = 0u64;
        let mut warnings = Vec::new();

        for path in collect_tree_files(&source)? {
            let relative_name = relative_name(&source, &path)?;
            let relative = safe_relative_path(&relative_name)?;
            let source_bytes = fs::read(&path)
                .map_err(|error| format!("读取来源文件 {} 失败: {error}", path.display()))?;
            let mut output_bytes = source_bytes.clone();
            if has_txt_extension(&path) {
                let parsed = script::parse(&source_bytes)
                    .map_err(|error| format!("解析脚本 {relative_name} 失败: {error}"))?;
                if !parsed.records.is_empty() {
                    let row = rows.remove(&relative_name).ok_or_else(|| {
                        format!("来源映射缺少脚本 {relative_name}，请重新导出翻译模板")
                    })?;
                    let current_fingerprint = fingerprint(&source_bytes);
                    if row.fingerprint != current_fingerprint {
                        return Err(format!(
                            "脚本 {relative_name} 在导出后发生变化，拒绝注入；请重新导出"
                        )
                        .into());
                    }
                    if row.entries != parsed.records.len() {
                        return Err(format!(
                            "脚本 {relative_name} 的文本项数已变化: map={} source={}",
                            row.entries,
                            parsed.records.len()
                        )
                        .into());
                    }
                    let choice_count = parsed
                        .records
                        .iter()
                        .filter(|record| record.kind == TextKind::Choice)
                        .count();
                    if row.choices != choice_count {
                        return Err(format!(
                            "脚本 {relative_name} 的选择项数量已变化: map={} source={choice_count}",
                            row.choices
                        )
                        .into());
                    }
                    let translation_relative = safe_relative_path(&row.translation)?;
                    let translation_path =
                        existing_regular_file(&translations, &translation_relative, "翻译 JSON")?;
                    let json = fs::read(&translation_path).map_err(|error| {
                        format!("读取翻译 JSON {} 失败: {error}", translation_path.display())
                    })?;
                    let supplied = vn_text::read_template(&json).map_err(|error| {
                        format!("读取 {} 的翻译 JSON 失败: {error}", row.translation)
                    })?;
                    let expected: Vec<vn_text::Entry> = parsed
                        .records
                        .iter()
                        .map(|record| vn_text::Entry {
                            name: None,
                            message: record.message.clone(),
                        })
                        .collect();
                    vn_text::validate_shape(&expected, &supplied).map_err(|error| {
                        format!("{relative_name} 的翻译 JSON 与来源不匹配: {error}")
                    })?;
                    let messages: Vec<String> =
                        supplied.into_iter().map(|entry| entry.message).collect();
                    output_bytes = script::inject(&source_bytes, &parsed.records, &messages)
                        .map_err(|error| format!("注入脚本 {relative_name} 失败: {error}"))?;
                    total_entries += messages.len() as u64;
                    total_choices += choice_count as u64;
                    if parsed.nonliteral_text_args > row.skipped {
                        warnings.push(format!(
                            "{relative_name}: 导出时跳过 {} 个动态文本参数；当前扫描发现 {} 个",
                            row.skipped, parsed.nonliteral_text_args
                        ));
                    }
                } else if rows.contains_key(&relative_name) {
                    return Err(
                        format!("来源映射中的脚本 {relative_name} 当前没有可编辑文本").into(),
                    );
                }
            }
            files.push(ImportFile {
                relative,
                bytes: output_bytes,
            });
        }

        if !rows.is_empty() {
            let missing = rows.keys().next().cloned().unwrap_or_default();
            return Err(format!("来源目录缺少映射中的脚本: {missing}").into());
        }
        let preview = Preview {
            inputs: vec![source.clone(), translations.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "重新解析当前来源并校验每个脚本的内容指纹".into(),
                "读取 JSON 并验证条数及来源顺序".into(),
                "按字符串字节范围注入译文，复制完整脚本目录".into(),
            ],
            details: vec![format!(
                "待注入文本项={total_entries}，脚本文件数={}，来源映射={MAP_NAME}",
                files.len()
            )],
        };
        Ok(Box::new(ImportTextJob {
            preview,
            output,
            files,
            entries: total_entries,
            choices: total_choices,
            warnings,
            overwrite: parameters.flag("overwrite"),
        }))
    }
}

impl PreparedOperation for ImportTextJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        let parent = self
            .output
            .parent()
            .ok_or_else(|| "重建输出目录缺少父目录".to_owned())?;
        let stage = TemporaryDirectory::create(parent, "repi-v2-import")?;
        for file in &self.files {
            let target = stage.path.join(&file.relative);
            create_parent(&target)?;
            write_new_file(&target, &file.bytes)?;
        }
        progress.report("校验重建目录并提交输出")?;
        let mut warnings = self.warnings;
        if let Some(warning) = commit_directory(stage, &self.output, self.overwrite)? {
            warnings.push(warning);
        }
        Ok(RunReport {
            summary: "译文注入完成；来源和翻译 JSON 未修改。".into(),
            totals: vec![
                ("脚本文件".into(), self.files.len() as u64),
                ("注入文本项".into(), self.entries),
                ("选择项".into(), self.choices),
            ],
            outputs: vec![self.output.clone()],
            warnings,
        })
    }
}

#[derive(Clone)]
struct MapRow {
    translation: String,
    fingerprint: String,
    entries: usize,
    choices: usize,
    skipped: usize,
}

fn parse_mapping(text: &str) -> Result<BTreeMap<String, MapRow>> {
    let mut lines = text.lines();
    if lines.next() != Some(MAP_HEADER) {
        return Err(format!("来源映射缺少有效表头: {MAP_NAME}").into());
    }
    let mut rows = BTreeMap::new();
    let mut translations = HashSet::new();
    for (index, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 6 {
            return Err(format!("来源映射第 {} 行字段数无效", index + 2).into());
        }
        safe_relative_path(fields[0])?;
        safe_relative_path(fields[1])?;
        if fields[2].len() != 16 || !fields[2].bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(format!("来源映射第 {} 行指纹无效", index + 2).into());
        }
        let entries = fields[3]
            .parse::<usize>()
            .map_err(|_| format!("来源映射第 {} 行文本项数无效", index + 2))?;
        let choices = fields[4]
            .parse::<usize>()
            .map_err(|_| format!("来源映射第 {} 行选择项数无效", index + 2))?;
        let skipped = fields[5]
            .parse::<usize>()
            .map_err(|_| format!("来源映射第 {} 行跳过项数无效", index + 2))?;
        if choices > entries {
            return Err(format!("来源映射第 {} 行选择项数超过总项数", index + 2).into());
        }
        if !translations.insert(fields[1].to_owned()) {
            return Err(format!("来源映射重复使用翻译文件: {}", fields[1]).into());
        }
        let row = MapRow {
            translation: fields[1].to_owned(),
            fingerprint: fields[2].to_ascii_lowercase(),
            entries,
            choices,
            skipped,
        };
        if rows.insert(fields[0].to_owned(), row).is_some() {
            return Err(format!("来源映射重复脚本路径: {}", fields[0]).into());
        }
    }
    Ok(rows)
}

fn fingerprint(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let canonical = fs::canonicalize(path)
        .map_err(|error| format!("无法读取{label} {}: {error}", path.display()))?;
    if !canonical.is_dir() {
        return Err(format!("{label}不是目录: {}", canonical.display()).into());
    }
    Ok(canonical)
}

fn output_directory(path: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(path)
        .map_err(|error| format!("无法解析输出路径 {}: {error}", path.display()))?;
    let parent = absolute.parent().ok_or("输出目录缺少父目录")?;
    let parent = fs::canonicalize(parent)
        .map_err(|error| format!("输出父目录不存在 {}: {error}", parent.display()))?;
    let name = absolute.file_name().ok_or("输出目录缺少名称")?;
    let output = parent.join(name);
    if let Ok(metadata) = fs::symlink_metadata(&output) {
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(format!("输出路径必须是普通目录或不存在: {}", output.display()).into());
        }
    }
    Ok(output)
}

fn collect_tree_files(root: &Path) -> Result<Vec<PathBuf>> {
    fn visit(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(directory)
            .map_err(|error| format!("读取目录 {} 失败: {error}", directory.display()))?
        {
            let path = entry?.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("读取文件状态 {} 失败: {error}", path.display()))?;
            if metadata.file_type().is_symlink() {
                return Err(format!("来源目录不接受符号链接: {}", path.display()).into());
            }
            if metadata.is_dir() {
                visit(&path, files)?;
            } else if metadata.is_file() {
                files.push(path);
            } else {
                return Err(format!("来源目录含有非普通文件: {}", path.display()).into());
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    visit(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn relative_name(root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| format!("文件不在来源目录内: {}", path.display()))?;
    let name = relative.to_string_lossy().replace('\\', "/");
    safe_relative_path(&name)?;
    if name.contains(['\t', '\r', '\n']) {
        return Err(format!("相对路径含有映射表保留字符: {name}").into());
    }
    Ok(name)
}

fn safe_relative_path(name: &str) -> Result<PathBuf> {
    if name.is_empty() || name.contains(['\t', '\r', '\n']) {
        return Err(format!("相对路径为空或含有保留字符: {name:?}").into());
    }
    let path = Path::new(name);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || path.to_string_lossy().contains(':')
    {
        return Err(format!("必须是安全的相对路径: {name}").into());
    }
    Ok(path.to_path_buf())
}

fn existing_regular_file(root: &Path, relative: &Path, label: &str) -> Result<PathBuf> {
    let mut path = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            return Err(format!("{label}路径不是安全的相对路径").into());
        };
        path.push(part);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("读取{label} {} 失败: {error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("{label}路径不接受符号链接: {}", path.display()).into());
        }
    }
    if !path.is_file() {
        return Err(format!("{label}不是普通文件: {}", path.display()).into());
    }
    Ok(path)
}

fn has_txt_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("txt"))
}

fn create_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("创建目录 {} 失败: {error}", parent.display()))?;
    }
    Ok(())
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("创建输出 {} 失败: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("写入输出 {} 失败: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("刷新输出 {} 失败: {error}", path.display()))?;
    Ok(())
}

fn commit_directory(
    stage: TemporaryDirectory,
    output: &Path,
    overwrite: bool,
) -> Result<Option<String>> {
    if output.exists() {
        if !overwrite {
            return Err(format!("输出已存在，默认不覆盖: {}", output.display()).into());
        }
        let metadata = fs::symlink_metadata(output)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(format!("输出只能覆盖普通目录: {}", output.display()).into());
        }
        let backup = temporary_sibling(output, "backup")?;
        fs::rename(output, &backup)?;
        if let Err(error) = fs::rename(&stage.path, output) {
            let restore = fs::rename(&backup, output);
            return match restore {
                Ok(()) => Err(format!("提交输出失败，旧目录已恢复: {error}").into()),
                Err(restore_error) => Err(format!(
                    "提交输出失败 ({error})，旧目录恢复失败 ({restore_error})，备份保留在 {}",
                    backup.display()
                )
                .into()),
            };
        }
        if let Err(error) = fs::remove_dir_all(&backup) {
            return Ok(Some(format!(
                "新输出已提交，但旧输出备份无法清理: {} ({error})",
                backup.display()
            )));
        }
        return Ok(None);
    }
    fs::rename(&stage.path, output)?;
    Ok(None)
}

fn temporary_sibling(output: &Path, suffix: &str) -> Result<PathBuf> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = output.parent().ok_or("输出目录缺少父目录")?;
    let name = output.file_name().ok_or("输出目录缺少名称")?;
    for _ in 0..100 {
        let candidate = parent.join(format!(
            ".{}.{}-{}-{}",
            name.to_string_lossy(),
            suffix,
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("无法分配唯一备份路径".into())
}

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn create(parent: &Path, label: &str) -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        for _ in 0..100 {
            let path = parent.join(format!(
                ".{label}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!("创建暂存目录失败 {}: {error}", path.display()).into())
                }
            }
        }
        Err("无法分配唯一暂存目录".into())
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
