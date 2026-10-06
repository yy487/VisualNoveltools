use repi_unpack::{
    built_in_name_candidates, content_fingerprint, extract_entry_bytes, faeries_script, hash_hex,
    load_name_candidates, md5_name, pack_archive_with_profile, repipack_profile, Archive, Entry,
    NameCandidate, PackItem, RepiProfile,
};
use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use vn_cli::*;

const TEXT_MAP_NAME: &str = "repi-map.tsv";
const TEXT_MAP_HEADER: &str = "source\ttranslation\tfingerprint\tmessages\tchoices";

struct Unpack;

impl Operation for Unpack {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "unpack",
            "解包 RepiPack v5",
            vec![
                Field::new("archive", "RepiPack 输入文件", FieldKind::Path).required(),
                Field::new("output", "独立输出目录", FieldKind::Path).required(),
                Field::new(
                    "names-file",
                    "补充名称表（UTF-8/Shift-JIS，每行一个 basename）",
                    FieldKind::Path,
                ),
                Field::new("name", "直接指定逻辑文件名（可重复）", FieldKind::Paths),
            ],
        );
        spec.description =
            "内置 Fairies Script.dat 文件名表；解析表项、恢复数据密钥并执行 LZSS。其他候选名可用名称表补充。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            parameters.set("archive", Value::Path(path.clone()));
            if parameters.get("output").is_none() {
                let output = path
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .join("repi_unpacked");
                parameters.set("output", Value::Path(output));
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let requested_archive = parameters.path("archive")?;
        let archive_path = fs::canonicalize(requested_archive)
            .map_err(|error| format!("无法读取输入 {}: {error}", requested_archive.display()))?;
        if !archive_path.is_file() {
            return Err(format!("输入不是文件: {}", archive_path.display()).into());
        }
        let bytes = fs::read(&archive_path)
            .map_err(|error| format!("读取输入失败 {}: {error}", archive_path.display()))?;
        let archive = Archive::parse(&bytes).map_err(|error| format!("解析失败: {error}"))?;

        let mut names = built_in_name_candidates()?;
        let mut inputs = vec![archive_path.clone()];
        if let Some(Value::Path(path)) = parameters.get("names-file") {
            names.extend(load_name_candidates(path)?);
            inputs.push(
                fs::canonicalize(path)
                    .map_err(|error| format!("无法读取名称表 {}: {error}", path.display()))?,
            );
        }
        if let Some(Value::Paths(paths)) = parameters.get("name") {
            for path in paths {
                let name = path.to_string_lossy().into_owned();
                names
                    .entry(md5_name(&name))
                    .or_insert_with(|| NameCandidate {
                        raw: name.as_bytes().to_vec(),
                        display: name,
                    });
            }
        }

        let output = std::path::absolute(parameters.path("output")?)
            .map_err(|error| format!("无法解析输出路径: {error}"))?;
        let resolved = names
            .keys()
            .filter(|hash| archive.entries.iter().any(|entry| &entry.hash == *hash))
            .count();
        let preview = Preview {
            inputs,
            outputs: vec![output.clone()],
            steps: vec![
                "读取并校验 RepiPack v5 头部和表项".into(),
                "按名称恢复条目密钥，解压到独立暂存目录".into(),
                "写入 manifest.tsv，并一次性提交输出目录".into(),
            ],
            details: vec![format!(
                "header={}，entries={}，匹配到名称={}，未知条目保留到 unresolved/",
                archive.header_name,
                archive.entries.len(),
                resolved
            )],
        };
        Ok(Box::new(UnpackJob {
            preview,
            archive_path,
            bytes,
            archive,
            names,
            output,
            overwrite: parameters.flag("overwrite"),
        }))
    }
}

struct UnpackJob {
    preview: Preview,
    archive_path: PathBuf,
    bytes: Vec<u8>,
    archive: Archive,
    names: HashMap<[u8; 16], NameCandidate>,
    output: PathBuf,
    overwrite: bool,
}

impl PreparedOperation for UnpackJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        let parent = self
            .output
            .parent()
            .ok_or_else(|| "输出目录缺少父目录".to_string())?;
        if !parent.is_dir() {
            return Err(format!("输出父目录不存在: {}", parent.display()).into());
        }

        progress.report(&format!("读取 {} 条表项", self.archive.entries.len()))?;
        let stage = TemporaryDirectory::create(parent)?;
        fs::create_dir(stage.path.join("files"))?;
        fs::create_dir(stage.path.join("unresolved"))?;

        let mut manifest =
            String::from("index\thash\toffset\tpacked_size\tunpacked_size\tflags\tstatus\tname\n");
        let mut used_names = HashSet::new();
        let mut decoded_count = 0u64;
        let mut unresolved_count = 0u64;
        let mut decoded_bytes = 0u64;
        let mut warnings = Vec::new();

        for (index, entry) in self.archive.entries.iter().enumerate() {
            let hash = hash_hex(&entry.hash);
            let name = self.names.get(&entry.hash);
            let (status, output_name) = if let Some(name) = name {
                let output_name = unique_output_name(&name.display, &hash, &mut used_names);
                let target = stage.path.join("files").join(&output_name);
                match extract_entry_bytes(&self.bytes, entry, &name.raw) {
                    Ok(data) => {
                        write_new_file(&target, &data)?;
                        decoded_count += 1;
                        decoded_bytes += data.len() as u64;
                        if data.len() != entry.unpacked_size as usize {
                            warnings.push(format!(
                                "entry {index} ({}) LZSS 提前结束: {}/{} 字节",
                                name.display,
                                data.len(),
                                entry.unpacked_size
                            ));
                            ("short-decode", output_name)
                        } else {
                            ("decoded", output_name)
                        }
                    }
                    Err(error) => {
                        let raw_target =
                            stage.path.join("unresolved").join(format!("{hash}.packed"));
                        write_new_file(&raw_target, packed_slice(&self.bytes, entry)?)?;
                        warnings.push(format!(
                            "entry {index} ({}) 解码失败: {error}",
                            name.display
                        ));
                        unresolved_count += 1;
                        ("decode-error", output_name)
                    }
                }
            } else {
                let raw_target = stage.path.join("unresolved").join(format!("{hash}.packed"));
                write_new_file(&raw_target, packed_slice(&self.bytes, entry)?)?;
                unresolved_count += 1;
                ("unresolved", String::new())
            };
            manifest.push_str(&format!(
                "{index}\t{hash}\t0x{:x}\t{}\t{}\t0x{:08x}\t{status}\t{}\n",
                entry.offset, entry.packed_size, entry.unpacked_size, entry.flags, output_name
            ));
        }

        manifest.push_str(&format!(
            "# source={}\n# header={}\n# version={}\n",
            self.archive_path.display(),
            self.archive.header_name,
            self.archive.version
        ));
        write_new_file(&stage.path.join("manifest.tsv"), manifest.as_bytes())?;
        progress.report("校验输出并提交目录")?;

        if self.output.exists() {
            if !self.overwrite {
                return Err(format!("输出已存在，默认不覆盖: {}", self.output.display()).into());
            }
            fs::remove_dir_all(&self.output)?;
        }
        fs::rename(&stage.path, &self.output)?;

        Ok(RunReport {
            summary: "RepiPack 解包完成；输入文件未修改。".into(),
            totals: vec![
                ("表项".into(), self.archive.entries.len() as u64),
                ("已解码".into(), decoded_count),
                ("未知/失败".into(), unresolved_count),
                ("解码字节".into(), decoded_bytes),
            ],
            outputs: vec![self.output.clone()],
            warnings,
        })
    }
}

struct ExportText;

impl Operation for ExportText {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "export",
            "导出对白与选项",
            vec![
                Field::new("source-dir", "已解包的脚本目录", FieldKind::Path).required(),
                Field::new("output-dir", "翻译 JSON 输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description = "按文件导出 -message 正文和 -case 选项；name 仅作只读上下文。".into();
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
        let mut map = String::from(TEXT_MAP_HEADER);
        let mut message_count = 0usize;
        let mut choice_count = 0usize;

        for path in collect_tree_files(&source)? {
            if !has_txt_extension(&path) {
                continue;
            }
            let relative = relative_name(&source, &path)?;
            let bytes = fs::read(&path)
                .map_err(|error| format!("读取脚本 {} 失败: {error}", path.display()))?;
            let script = faeries_script::parse(&bytes, &relative)?;
            if script.entries.is_empty() {
                continue;
            }
            let entries: Vec<vn_text::Entry> = script
                .entries
                .iter()
                .map(|message| vn_text::Entry {
                    name: message.name.clone(),
                    message: message.text.clone(),
                })
                .collect();
            let translation = format!("{relative}.json");
            validate_relative_name(&translation)?;
            if relative.contains(['\t', '\r', '\n']) {
                return Err(format!("脚本相对路径不能写入映射表: {relative}").into());
            }
            let json = vn_text::write_template(&entries)
                .map_err(|error| format!("生成 {relative} 的翻译 JSON 失败: {error}"))?;
            map.push_str(&format!(
                "\n{relative}\t{translation}\t{}\t{}\t{}",
                content_fingerprint(&bytes),
                script.message_count,
                script.choice_count
            ));
            message_count += script.message_count;
            choice_count += script.choice_count;
            files.push(ExportTextFile {
                translation,
                json,
                count: entries.len(),
                choice_count: script.choice_count,
            });
        }

        if files.is_empty() {
            return Err("来源目录中没有包含 -message 或 @select 的 .txt 脚本".into());
        }
        map.push('\n');
        let preview = Preview {
            inputs: vec![source.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "按 CP932 解析 -message 正文及 @select 选项".into(),
                "逐源脚本写出翻译 JSON 和来源映射".into(),
                "写入暂存目录后提交输出".into(),
            ],
            details: vec![format!(
                "脚本文件={}，message={}，choice={}；name 在导入时不可修改",
                files.len(),
                message_count,
                choice_count
            )],
        };
        Ok(Box::new(ExportTextJob {
            preview,
            output,
            files,
            map,
            overwrite: parameters.flag("overwrite"),
        }))
    }
}

struct ExportTextFile {
    translation: String,
    json: Vec<u8>,
    count: usize,
    choice_count: usize,
}

struct ExportTextJob {
    preview: Preview,
    output: PathBuf,
    files: Vec<ExportTextFile>,
    map: String,
    overwrite: bool,
}

impl PreparedOperation for ExportTextJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        let parent = self.output.parent().ok_or("输出目录缺少父目录")?;
        let stage = TemporaryDirectory::create(parent)?;
        let mut total_entries = 0u64;
        for file in &self.files {
            progress.report(&format!("写出 {} 条文本记录", file.count))?;
            let relative = safe_relative_path(&file.translation)?;
            let target = stage.path.join(relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            write_new_file(&target, &file.json)?;
            total_entries += file.count as u64;
        }
        write_new_file(&stage.path.join(TEXT_MAP_NAME), self.map.as_bytes())?;
        let warnings = commit_directory(&stage.path, &self.output, self.overwrite)?;
        Ok(RunReport {
            summary: "对白与选项文本导出完成。name 是只读上下文。".into(),
            totals: vec![
                ("脚本文件".into(), self.files.len() as u64),
                ("文本记录".into(), total_entries),
                (
                    "choice".into(),
                    self.files.iter().map(|file| file.choice_count as u64).sum(),
                ),
            ],
            outputs: vec![self.output.clone()],
            warnings,
        })
    }
}

struct ImportText;

impl Operation for ImportText {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "import",
            "回注脚本对白",
            vec![
                Field::new("source-dir", "原始脚本目录", FieldKind::Path).required(),
                Field::new("translation-dir", "导出的翻译目录", FieldKind::Path).required(),
                Field::new("output-dir", "回注后脚本输出目录", FieldKind::Path).required(),
            ],
        );
        spec.primary = false;
        spec.description = "校验来源和条目结构；name 仅作上下文并忽略其值，只回写 message。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            parameters.set("source-dir", Value::Path(path.clone()));
            if paths.len() > 1 {
                parameters.set("translation-dir", Value::Path(paths[1].clone()));
            }
            if parameters.get("output-dir").is_none() {
                parameters.set(
                    "output-dir",
                    Value::Path(
                        path.parent()
                            .unwrap_or_else(|| Path::new("."))
                            .join("translated-scripts"),
                    ),
                );
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let source = canonical_directory(parameters.path("source-dir")?, "原始脚本目录")?;
        let translations = canonical_directory(parameters.path("translation-dir")?, "翻译目录")?;
        let output = output_directory(parameters.path("output-dir")?)?;
        let map_path = translations.join(TEXT_MAP_NAME);
        let map_bytes = fs::read(&map_path)
            .map_err(|error| format!("读取来源映射 {} 失败: {error}", map_path.display()))?;
        let map_text = std::str::from_utf8(&map_bytes)
            .map_err(|error| format!("来源映射不是 UTF-8: {error}"))?;
        let rows = parse_text_map(map_text)?;

        let source_paths = collect_tree_files(&source)?;
        let mut snapshots = Vec::with_capacity(source_paths.len());
        let mut source_indexes = HashMap::new();
        for path in source_paths {
            let relative = relative_name(&source, &path)?;
            let bytes = fs::read(&path)
                .map_err(|error| format!("读取来源文件 {} 失败: {error}", path.display()))?;
            source_indexes.insert(relative.clone(), snapshots.len());
            snapshots.push(SourceSnapshot { relative, bytes });
        }

        let mut used_sources = HashSet::new();
        let mut used_translations = HashSet::new();
        let mut message_count = 0usize;
        let mut choice_count = 0usize;
        for row in &rows {
            validate_relative_name(&row.source)?;
            let translation_rel = validate_relative_name(&row.translation)?;
            if !used_sources.insert(row.source.clone()) {
                return Err(format!("映射表重复来源脚本: {}", row.source).into());
            }
            if !used_translations.insert(row.translation.clone()) {
                return Err(format!("映射表重复翻译文件: {}", row.translation).into());
            }
            let Some(&source_index) = source_indexes.get(&row.source) else {
                return Err(format!("当前来源缺少映射中的脚本: {}", row.source).into());
            };
            let snapshot = &snapshots[source_index];
            if content_fingerprint(&snapshot.bytes) != row.fingerprint {
                return Err(format!(
                    "来源脚本已变化，拒绝回注: {}（请从当前来源重新导出）",
                    row.source
                )
                .into());
            }

            let script = faeries_script::parse(&snapshot.bytes, &row.source)?;
            if script.message_count != row.message_count || script.choice_count != row.choice_count
            {
                return Err(format!(
                    "{} 的 message/choice 数量与映射不符: {}/{} != {}/{}",
                    row.source,
                    script.message_count,
                    script.choice_count,
                    row.message_count,
                    row.choice_count
                )
                .into());
            }
            let expected: Vec<vn_text::Entry> = script
                .entries
                .iter()
                .map(|message| vn_text::Entry {
                    name: message.name.clone(),
                    message: message.text.clone(),
                })
                .collect();
            let translation_path = resolve_file(&translations, &translation_rel)?;
            let translation_bytes = fs::read(&translation_path).map_err(|error| {
                format!("读取翻译文件 {} 失败: {error}", translation_path.display())
            })?;
            let supplied = vn_text::read_template(&translation_bytes).map_err(|error| {
                format!("翻译 JSON {} 无效: {error}", translation_path.display())
            })?;
            vn_text::validate_shape(&expected, &supplied).map_err(|error| {
                format!(
                    "翻译 JSON {} 条目结构不匹配: {error}",
                    translation_path.display()
                )
            })?;
            // `name` is contextual metadata only. Shape validation above keeps
            // the JSON layout bound to the source, while its value is never
            // consulted for injection; only `message` is written back.
            let translated: Vec<String> =
                supplied.iter().map(|entry| entry.message.clone()).collect();
            let rewritten = faeries_script::rewrite(&snapshot.bytes, &script, &translated)
                .map_err(|error| format!("回注 {} 失败: {error}", row.source))?;
            snapshots[source_index].bytes = rewritten;
            message_count += script.message_count;
            choice_count += script.choice_count;
        }

        if rows.is_empty() {
            return Err("来源映射没有可回注的脚本记录".into());
        }
        let mut actual_sources = HashSet::new();
        for snapshot in &snapshots {
            if has_txt_extension(Path::new(&snapshot.relative)) {
                let script = faeries_script::parse(&snapshot.bytes, &snapshot.relative)?;
                if !script.entries.is_empty() {
                    actual_sources.insert(snapshot.relative.clone());
                }
            }
        }
        if actual_sources != used_sources {
            let missing = actual_sources
                .difference(&used_sources)
                .next()
                .cloned()
                .unwrap_or_else(|| "未知脚本".into());
            return Err(format!(
                "来源映射未覆盖全部 message/choice 脚本（至少缺少 {missing}）；请重新导出"
            )
            .into());
        }
        let preview = Preview {
            inputs: vec![source.clone(), translations.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "校验来源映射、脚本指纹和翻译 JSON 形状".into(),
                "忽略 name 内容，只回写对白正文和选项 text".into(),
                "保留来源目录其他文件并提交完整输出目录".into(),
            ],
            details: vec![format!(
                "脚本文件={}，message={}，choice={}；分支目标和其他指令保持原样",
                rows.len(),
                message_count,
                choice_count
            )],
        };
        Ok(Box::new(ImportTextJob {
            preview,
            output,
            files: snapshots,
            overwrite: parameters.flag("overwrite"),
        }))
    }
}

struct SourceSnapshot {
    relative: String,
    bytes: Vec<u8>,
}

struct TextMapRow {
    source: String,
    translation: String,
    fingerprint: String,
    message_count: usize,
    choice_count: usize,
}

struct ImportTextJob {
    preview: Preview,
    output: PathBuf,
    files: Vec<SourceSnapshot>,
    overwrite: bool,
}

impl PreparedOperation for ImportTextJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        let parent = self.output.parent().ok_or("输出目录缺少父目录")?;
        let stage = TemporaryDirectory::create(parent)?;
        for file in &self.files {
            progress.report(&format!("复制并写入 {}", file.relative))?;
            let relative = safe_relative_path(&file.relative)?;
            let target = stage.path.join(relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            write_new_file(&target, &file.bytes)?;
        }
        let warnings = commit_directory(&stage.path, &self.output, self.overwrite)?;
        Ok(RunReport {
            summary: "脚本文本回注完成；只修改了对白正文和选项显示文字。".into(),
            totals: vec![("复制文件".into(), self.files.len() as u64)],
            outputs: vec![self.output.clone()],
            warnings,
        })
    }
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
    if !parent.is_dir() {
        return Err(format!("输出父路径不是目录: {}", parent.display()).into());
    }
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
        let entries = fs::read_dir(directory)
            .map_err(|error| format!("读取目录 {} 失败: {error}", directory.display()))?;
        for entry in entries {
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

fn has_txt_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("txt"))
}

fn relative_name(root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| format!("文件不在来源目录内: {}", path.display()))?;
    let name = relative.to_string_lossy().replace('\\', "/");
    validate_relative_name(&name)?;
    if name.chars().any(|ch| matches!(ch, '\t' | '\r' | '\n')) {
        return Err(format!("相对路径含有映射表保留字符: {name}").into());
    }
    Ok(name)
}

fn validate_relative_name(name: &str) -> Result<PathBuf> {
    if name.is_empty() || name.chars().any(|ch| matches!(ch, '\t' | '\r' | '\n')) {
        return Err(format!("映射路径为空或含有非法字符: {name:?}").into());
    }
    let path = Path::new(name);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("映射路径必须是安全的相对路径: {name}").into());
    }
    Ok(path.to_path_buf())
}

fn safe_relative_path(name: &str) -> Result<PathBuf> {
    validate_relative_name(name)
}

fn parse_text_map(text: &str) -> Result<Vec<TextMapRow>> {
    let mut lines = text.lines();
    if lines.next() != Some(TEXT_MAP_HEADER) {
        return Err(format!("来源映射缺少有效表头: {TEXT_MAP_NAME}").into());
    }
    let mut rows = Vec::new();
    for (index, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 5 {
            return Err(format!("来源映射第 {} 行字段数无效", index + 2).into());
        }
        validate_relative_name(fields[0])?;
        validate_relative_name(fields[1])?;
        if fields[2].len() != 32 || !fields[2].bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(format!("来源映射第 {} 行指纹无效", index + 2).into());
        }
        let message_count = fields[3]
            .parse::<usize>()
            .map_err(|_| format!("来源映射第 {} 行 message 数无效", index + 2))?;
        let choice_count = fields[4]
            .parse::<usize>()
            .map_err(|_| format!("来源映射第 {} 行 choice 数无效", index + 2))?;
        if message_count == 0 && choice_count == 0 {
            return Err(format!("来源映射第 {} 行文本记录数不能为零", index + 2).into());
        }
        rows.push(TextMapRow {
            source: fields[0].to_owned(),
            translation: fields[1].to_owned(),
            fingerprint: fields[2].to_ascii_lowercase(),
            message_count,
            choice_count,
        });
    }
    Ok(rows)
}

fn resolve_file(root: &Path, relative: &Path) -> Result<PathBuf> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            return Err(format!("不是安全的相对文件路径: {}", relative.display()).into());
        };
        current.push(part);
        let metadata = fs::symlink_metadata(&current)
            .map_err(|error| format!("映射文件不存在 {}: {error}", current.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("映射路径不允许符号链接: {}", current.display()).into());
        }
    }
    if !current.is_file() {
        return Err(format!("映射目标不是文件: {}", current.display()).into());
    }
    Ok(current)
}

fn commit_directory(stage: &Path, output: &Path, overwrite: bool) -> Result<Vec<String>> {
    if !output.exists() {
        fs::rename(stage, output)
            .map_err(|error| format!("提交输出目录 {} 失败: {error}", output.display()))?;
        return Ok(Vec::new());
    }
    if !overwrite {
        return Err(format!("输出目录已存在，默认不覆盖: {}", output.display()).into());
    }

    static NEXT_BACKUP: AtomicU64 = AtomicU64::new(0);
    let parent = output.parent().ok_or("输出目录缺少父目录")?;
    let mut backup = None;
    for _ in 0..100 {
        let candidate = parent.join(format!(
            ".repi-text-backup-{}-{}",
            std::process::id(),
            NEXT_BACKUP.fetch_add(1, Ordering::Relaxed)
        ));
        if !candidate.exists() {
            backup = Some(candidate);
            break;
        }
    }
    let backup = backup.ok_or("无法分配输出目录备份名")?;
    fs::rename(output, &backup)
        .map_err(|error| format!("无法暂存现有输出目录 {}: {error}", output.display()))?;
    if let Err(error) = fs::rename(stage, output) {
        return match fs::rename(&backup, output) {
            Ok(()) => Err(format!("提交输出失败，已恢复旧目录: {error}").into()),
            Err(restore_error) => Err(format!(
                "提交输出失败: {error}; 恢复旧目录也失败: {restore_error}; 旧目录保留在 {}",
                backup.display()
            )
            .into()),
        };
    }
    match fs::remove_dir_all(&backup) {
        Ok(()) => Ok(Vec::new()),
        Err(error) => Ok(vec![format!(
            "新输出已提交，但旧输出备份未能清理: {} ({error})",
            backup.display()
        )]),
    }
}

struct Repack;

impl Operation for Repack {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "repack",
            "回封 RepiPack v5",
            vec![
                Field::new("input-dir", "待封包文件目录", FieldKind::Path).required(),
                Field::new("archive", "RepiPack 输出文件", FieldKind::Path).required(),
                Field::new(
                    "names-file",
                    "名称映射（逻辑名<TAB>源文件；可选）",
                    FieldKind::Path,
                ),
                Field::new("pack-version", "封包版本", FieldKind::Text)
                    .default(Value::Text("5".into())),
                Field::new("key-index", "key_index", FieldKind::Text)
                    .default(Value::Text("12".into())),
            ],
        );
        spec.primary = false;
        spec.description =
            "按逻辑文件名重新计算 MD5 表项并写出可读取的 RepiPack；当前使用原始条目，不做 LZSS 压缩。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            parameters.set("input-dir", Value::Path(path.clone()));
            if parameters.get("archive").is_none() {
                let archive = path
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .join("repacked.dat");
                parameters.set("archive", Value::Path(archive));
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let input_dir = fs::canonicalize(parameters.path("input-dir")?)
            .map_err(|error| format!("无法读取输入目录: {error}"))?;
        if !input_dir.is_dir() {
            return Err(format!("不是目录: {}", input_dir.display()).into());
        }

        let mut inputs = vec![input_dir.clone()];
        let items =
            if let Some(Value::Path(names_file)) = parameters.get("names-file") {
                inputs.push(fs::canonicalize(names_file).map_err(|error| {
                    format!("无法读取名称映射 {}: {error}", names_file.display())
                })?);
                load_repack_items(&input_dir, names_file)?
            } else {
                discover_repack_items(&input_dir)?
            };
        if items.is_empty() {
            return Err("输入目录没有可封包文件".into());
        }
        let total_bytes: u64 = items.iter().map(|item| item.data.len() as u64).sum();
        let archive = std::path::absolute(parameters.path("archive")?)
            .map_err(|error| format!("无法解析输出路径: {error}"))?;
        let version = parameters.text("pack-version")?;
        let key_index = parameters.text("key-index")?;
        let profile = repipack_profile(
            version,
            key_index
                .parse::<u32>()
                .map_err(|_| format!("key_index 不是数字: {key_index}"))?,
        )?;
        let preview = Preview {
            inputs,
            outputs: vec![archive.clone()],
            steps: vec![
                "读取逻辑文件名与源文件快照".into(),
                "生成 RepiPack v5 头部、MD5 表和加密数据区".into(),
                "写入临时文件并提交输出".into(),
            ],
            details: vec![format!(
                "entries={}，原始数据={} 字节",
                items.len(),
                total_bytes
            )],
        };
        Ok(Box::new(RepackJob {
            preview,
            archive,
            items,
            profile,
            overwrite: parameters.flag("overwrite"),
        }))
    }
}

struct RepackJob {
    preview: Preview,
    archive: PathBuf,
    items: Vec<PackItem>,
    profile: RepiProfile,
    overwrite: bool,
}

impl PreparedOperation for RepackJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        let parent = self
            .archive
            .parent()
            .ok_or_else(|| "输出文件缺少父目录".to_string())?;
        if !parent.is_dir() {
            return Err(format!("输出父目录不存在: {}", parent.display()).into());
        }
        let header_name = self
            .archive
            .file_name()
            .ok_or_else(|| "输出文件缺少文件名".to_string())?
            .to_string_lossy()
            .into_owned();
        progress.report(&format!("封包 {} 个条目", self.items.len()))?;
        let bytes = pack_archive_with_profile(&header_name, &self.items, self.profile)?;
        let temporary = TemporaryFile::create(parent)?;
        fs::write(&temporary.path, &bytes)?;
        let file = fs::OpenOptions::new().write(true).open(&temporary.path)?;
        file.sync_all()?;
        drop(file);
        if self.archive.exists() {
            if !self.overwrite {
                return Err(format!("输出已存在，默认不覆盖: {}", self.archive.display()).into());
            }
            fs::remove_file(&self.archive)?;
        }
        fs::rename(&temporary.path, &self.archive)?;
        Ok(RunReport {
            summary: "RepiPack 回封完成；当前条目使用原始存储。".into(),
            totals: vec![
                ("条目".into(), self.items.len() as u64),
                ("输出字节".into(), bytes.len() as u64),
                ("版本".into(), self.profile.version as u64),
                ("key_index".into(), self.profile.key_index as u64),
            ],
            outputs: vec![self.archive.clone()],
            ..RunReport::default()
        })
    }
}

fn discover_repack_items(input_dir: &Path) -> Result<Vec<PackItem>> {
    let mut paths = fs::read_dir(input_dir)
        .map_err(|error| format!("读取输入目录失败: {error}"))?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| format!("读取输入目录失败: {error}"))?;
    paths.sort();
    let mut items = Vec::new();
    for path in paths {
        if path.is_file() {
            let name = path
                .file_name()
                .ok_or_else(|| format!("文件缺少名称: {}", path.display()))?
                .to_string_lossy()
                .into_owned();
            items.push(PackItem {
                name,
                data: fs::read(&path)
                    .map_err(|error| format!("读取 {} 失败: {error}", path.display()))?,
            });
        }
    }
    Ok(items)
}

fn load_repack_items(input_dir: &Path, names_file: &Path) -> Result<Vec<PackItem>> {
    let text = fs::read_to_string(names_file)
        .map_err(|error| format!("读取名称映射失败 {}: {error}", names_file.display()))?;
    let mut items = Vec::new();
    for (line_number, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (logical_name, source_name) = if let Some((logical, source)) = line.split_once('\t') {
            (logical.trim(), source.trim())
        } else {
            (line, line)
        };
        if logical_name.is_empty() || source_name.is_empty() {
            return Err(format!("名称映射第 {} 行为空", line_number + 1).into());
        }
        let source = PathBuf::from(source_name);
        let source = if source.is_absolute() {
            source
        } else {
            input_dir.join(source)
        };
        if !source.is_file() {
            return Err(format!(
                "名称映射第 {} 行源文件不存在: {}",
                line_number + 1,
                source.display()
            )
            .into());
        }
        items.push(PackItem {
            name: logical_name.to_owned(),
            data: fs::read(&source)
                .map_err(|error| format!("读取 {} 失败: {error}", source.display()))?,
        });
    }
    Ok(items)
}

fn packed_slice<'a>(bytes: &'a [u8], entry: &Entry) -> Result<&'a [u8]> {
    let start = entry.offset as usize;
    let end = start
        .checked_add(entry.packed_size as usize)
        .ok_or("packed 数据范围溢出")?;
    bytes
        .get(start..end)
        .ok_or("packed 数据超出输入文件".into())
}

fn unique_output_name(name: &str, hash: &str, used: &mut HashSet<String>) -> String {
    let base = name
        .rsplit(['/', '\\'])
        .next()
        .filter(|value| !value.is_empty() && *value != "." && *value != "..")
        .unwrap_or(hash);
    let sanitized: String = base
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            ch if ch.is_control() => '_',
            ch => ch,
        })
        .collect();
    let sanitized = if sanitized.is_empty() {
        hash.to_owned()
    } else {
        sanitized
    };
    if used.insert(sanitized.clone()) {
        return sanitized;
    }
    let with_hash = format!("{}-{}", sanitized, &hash[..8]);
    used.insert(with_hash.clone());
    with_hash
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

struct TemporaryDirectory {
    path: PathBuf,
}

struct TemporaryFile {
    path: PathBuf,
}

impl TemporaryFile {
    fn create(parent: &Path) -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(100_000);
        for _ in 0..100 {
            let path = parent.join(format!(
                ".repi-repack-{}-{}.tmp",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!("创建暂存文件失败 {}: {error}", path.display()).into())
                }
            }
        }
        Err("无法分配唯一暂存文件".into())
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = fs::remove_file(&self.path);
        }
    }
}

impl TemporaryDirectory {
    fn create(parent: &Path) -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let path = parent.join(format!(
                ".repi-unpack-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
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

fn main() -> ExitCode {
    let result = (|| {
        let executable = std::env::current_exe()?.into_os_string();
        Panel::new(
            executable,
            "RepiPack 解包器",
            env!("CARGO_PKG_VERSION"),
            vec![
                Box::new(Unpack),
                Box::new(ExportText),
                Box::new(ImportText),
                Box::new(Repack),
            ],
        )?
        .run_env()
    })();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[失败] {error}");
            ExitCode::FAILURE
        }
    }
}
