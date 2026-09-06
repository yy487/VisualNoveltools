use crate::font::{cp932_for_carrier, EncodingPlan};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const RECORD_SIZE: usize = 256;
const POINTER_COUNT: usize = 128;
const TEXT_FORMAT: &str = "tauhido-scenario-text-v1";
const TEXT_WORKSPACE_FORMAT: &str = "tauhido-scenario-text-workspace-v1";
const REBUILD_WORKSPACE_FORMAT: &str = "tauhido-scenario-rebuild-workspace-v1";
const COMPRESSED_KANA: &str =
    "。「」、・をぁぃぅぇぉゃゅょっーあいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわん";

type Result<T> = std::result::Result<T, String>;
type CommandTableParse = (usize, Vec<usize>, Vec<(usize, usize)>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_script_index")]
    pub script_index: usize,
    #[serde(rename = "_offset")]
    pub offset: u64,
    #[serde(rename = "_script_offset")]
    pub script_offset: u32,
    #[serde(rename = "_size")]
    pub size: u32,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocument {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_source_file")]
    pub source_file: String,
    #[serde(rename = "_source_sha256")]
    pub source_sha256: String,
    #[serde(rename = "_record_size")]
    pub record_size: u16,
    pub entries: Vec<TextEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextWorkspaceManifest {
    #[serde(rename = "_format")]
    format: String,
    sources: Vec<TextWorkspaceSource>,
    summary: TextSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextWorkspaceSource {
    source_file: String,
    source_sha256: String,
    json_file: String,
    scripts: usize,
    entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextSummary {
    files: usize,
    scripts: usize,
    entries: usize,
    warnings: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RebuildManifest {
    #[serde(rename = "_format")]
    format: String,
    files: Vec<RebuildFile>,
    summary: RebuildSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RebuildFile {
    source_file: String,
    source_sha256: String,
    output_file: String,
    output_sha256: String,
    entries: usize,
    changed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RebuildSummary {
    files: usize,
    entries: usize,
    changed: usize,
}

#[derive(Debug, Clone)]
pub struct TextExtractReport {
    pub files: usize,
    pub scripts: usize,
    pub entries: usize,
    pub warnings: Vec<String>,
    pub output_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TextInjectReport {
    pub files: usize,
    pub entries: usize,
    pub changed: usize,
    pub output_root: PathBuf,
}

pub fn extract_document(source: &[u8], source_name: &str) -> Result<TextDocument> {
    let parsed = parse_scenario_file(source.to_vec(), source_name.to_string())?;
    Ok(build_text_document(&parsed)?.document)
}

pub fn messages(document: &TextDocument) -> impl Iterator<Item = &str> {
    document.entries.iter().map(|entry| entry.message.as_str())
}

pub fn rebuild_document(
    source: &[u8],
    source_name: &str,
    document: &TextDocument,
    plan: &EncodingPlan,
) -> Result<(Vec<u8>, usize)> {
    let parsed = parse_scenario_file(source.to_vec(), source_name.to_string())?;
    rebuild_scenario_file(&parsed, document, Some(plan))
}

#[derive(Debug, Clone)]
struct ScenarioFile {
    source: Vec<u8>,
    source_name: String,
    pointers: Vec<u16>,
    scripts: Vec<ScriptLayout>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct ScriptLayout {
    index: usize,
    file_start: usize,
    capacity: usize,
    text_spans: Vec<TextSpan>,
    pointer_refs: Vec<PointerRef>,
    active_end: usize,
}

#[derive(Debug, Clone)]
struct TextSpan {
    start: usize,
    end: usize,
    entry_type: &'static str,
    text: String,
}

#[derive(Debug, Clone, Copy)]
struct PointerRef {
    storage: usize,
    target: usize,
}

#[derive(Debug, Clone)]
struct PreparedText {
    document: TextDocument,
    scripts: usize,
    warnings: Vec<String>,
}

pub fn extract_text_batch(
    inputs: &[PathBuf],
    output_root: &Path,
    overwrite: bool,
) -> Result<TextExtractReport> {
    let sources = resolve_scenario_inputs(inputs)?;
    validate_output_root(
        output_root,
        overwrite,
        TEXT_WORKSPACE_FORMAT,
        "text_manifest.json",
    )?;

    let mut prepared = Vec::new();
    for path in &sources {
        let source =
            fs::read(path).map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
        let source_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or_else(|| format!("输入路径没有文件名: {}", path.display()))?;
        let parsed = parse_scenario_file(source, source_name.clone())
            .map_err(|error| format!("{}: {error}", path.display()))?;
        prepared.push(build_text_document(&parsed)?);
    }

    let staging = create_staging(output_root, "text-staging")?;
    let write_result = write_text_workspace(&prepared, &staging);
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    commit_staging(&staging, output_root, overwrite)?;

    let files = prepared.len();
    let scripts = prepared.iter().map(|item| item.scripts).sum();
    let entries = prepared
        .iter()
        .map(|item| item.document.entries.len())
        .sum();
    let warnings = prepared
        .iter()
        .flat_map(|item| item.warnings.iter().cloned())
        .collect();
    Ok(TextExtractReport {
        files,
        scripts,
        entries,
        warnings,
        output_root: output_root.to_path_buf(),
    })
}

pub fn inject_text_batch(
    sources: &[PathBuf],
    translation_root: &Path,
    output_root: &Path,
    overwrite: bool,
) -> Result<TextInjectReport> {
    let source_paths = resolve_scenario_inputs(sources)?;
    let documents = read_translation_documents(translation_root)?;
    validate_output_root(
        output_root,
        overwrite,
        REBUILD_WORKSPACE_FORMAT,
        "rebuild_manifest.json",
    )?;

    let mut parsed_sources = Vec::new();
    for path in &source_paths {
        let bytes =
            fs::read(path).map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
        let name = path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .ok_or_else(|| format!("输入路径没有文件名: {}", path.display()))?;
        let parsed = parse_scenario_file(bytes, name)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        parsed_sources.push(parsed);
    }

    let mut matched = HashSet::new();
    let mut rebuilt = Vec::new();
    for document in documents {
        let candidates: Vec<usize> = parsed_sources
            .iter()
            .enumerate()
            .filter(|(_, source)| {
                source
                    .source_name
                    .eq_ignore_ascii_case(&document.source_file)
                    && sha256_hex(&source.source) == document.source_sha256
            })
            .map(|(index, _)| index)
            .collect();
        if candidates.len() != 1 {
            return Err(format!(
                "{}: 按文件名和 SHA-256 应匹配一个源文件，实际匹配 {} 个",
                document.source_file,
                candidates.len()
            ));
        }
        let source_index = candidates[0];
        if !matched.insert(source_index) {
            return Err(format!("{}: 存在重复翻译 JSON", document.source_file));
        }
        let (bytes, changed) =
            rebuild_scenario_file(&parsed_sources[source_index], &document, None)?;
        rebuilt.push((
            parsed_sources[source_index].clone(),
            document,
            bytes,
            changed,
        ));
    }
    if rebuilt.is_empty() {
        return Err("翻译目录没有可注入的 Tauhido JSON".to_string());
    }

    let staging = create_staging(output_root, "rebuild-staging")?;
    let write_result = (|| -> Result<RebuildManifest> {
        let mut files = Vec::new();
        for (source, document, bytes, changed) in &rebuilt {
            let output_path = staging.join(&source.source_name);
            fs::write(&output_path, bytes)
                .map_err(|error| format!("写入 {} 失败: {error}", output_path.display()))?;
            files.push(RebuildFile {
                source_file: source.source_name.clone(),
                source_sha256: sha256_hex(&source.source),
                output_file: source.source_name.clone(),
                output_sha256: sha256_hex(bytes),
                entries: document.entries.len(),
                changed: *changed,
            });
        }
        let manifest = RebuildManifest {
            format: REBUILD_WORKSPACE_FORMAT.to_string(),
            summary: RebuildSummary {
                files: files.len(),
                entries: files.iter().map(|file| file.entries).sum(),
                changed: files.iter().map(|file| file.changed).sum(),
            },
            files,
        };
        write_json(&staging.join("rebuild_manifest.json"), &manifest)?;
        Ok(manifest)
    })();
    let manifest = match write_result {
        Ok(manifest) => manifest,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
    };
    commit_staging(&staging, output_root, overwrite)?;
    Ok(TextInjectReport {
        files: manifest.summary.files,
        entries: manifest.summary.entries,
        changed: manifest.summary.changed,
        output_root: output_root.to_path_buf(),
    })
}

fn build_text_document(parsed: &ScenarioFile) -> Result<PreparedText> {
    let mut entries = Vec::new();
    for script in &parsed.scripts {
        for span in &script.text_spans {
            let absolute = script
                .file_start
                .checked_add(span.start)
                .ok_or_else(|| "文本绝对偏移溢出".to_string())?;
            entries.push(TextEntry {
                file: parsed.source_name.clone(),
                index: entries.len(),
                script_index: script.index,
                offset: absolute as u64,
                script_offset: span.start as u32,
                size: (span.end - span.start) as u32,
                entry_type: span.entry_type.to_string(),
                encoding: "Tauhido hybrid CP932/kana".to_string(),
                scr_msg: span.text.clone(),
                message: span.text.clone(),
            });
        }
    }
    Ok(PreparedText {
        document: TextDocument {
            format: TEXT_FORMAT.to_string(),
            source_file: parsed.source_name.clone(),
            source_sha256: sha256_hex(&parsed.source),
            record_size: RECORD_SIZE as u16,
            entries,
        },
        scripts: parsed.scripts.len(),
        warnings: parsed.warnings.clone(),
    })
}

fn parse_scenario_file(source: Vec<u8>, source_name: String) -> Result<ScenarioFile> {
    if source.len() < RECORD_SIZE * 3 || !source.len().is_multiple_of(RECORD_SIZE) {
        return Err("文件不是至少 3 个完整 256 字节记录".to_string());
    }
    let record_count = source.len() / RECORD_SIZE;
    let mut pointers = Vec::new();
    let mut saw_zero = false;
    for index in 0..POINTER_COUNT {
        let value = read_u16(&source, index * 2)?;
        if value == 0 {
            saw_zero = true;
            continue;
        }
        if saw_zero {
            return Err(format!("记录号表在索引 {index} 的零值之后又出现非零值"));
        }
        if value < 2 || usize::from(value) > record_count + 1 {
            return Err(format!("记录号表索引 {index} 的值 {value} 越界"));
        }
        if pointers.last().is_some_and(|previous| *previous >= value) {
            return Err("记录号表的非零值没有严格递增".to_string());
        }
        pointers.push(value);
    }
    if pointers.len() < 3 {
        return Err("记录号表不足以描述页面映射和脚本".to_string());
    }
    if usize::from(*pointers.last().unwrap_or(&0)) != record_count + 1 {
        return Err(format!("记录号表终点不是文件末尾记录 {}", record_count + 1));
    }
    validate_page_map(&source, &pointers)?;

    let mut scripts = Vec::new();
    let mut warnings = Vec::new();
    for index in 1..pointers.len() - 1 {
        let start = (usize::from(pointers[index]) - 1) * RECORD_SIZE;
        let end = (usize::from(pointers[index + 1]) - 1) * RECORD_SIZE;
        let layout = parse_script(index, start, &source[start..end])?;
        if layout.active_end < layout.capacity {
            let tail = &source[start + layout.active_end..end];
            if tail.iter().any(|byte| *byte != 0) {
                warnings.push(format!(
                    "{} script {}: 活动数据后保留了 {} 字节非零不透明尾部",
                    source_name,
                    index,
                    tail.iter().filter(|byte| **byte != 0).count()
                ));
            }
        }
        scripts.push(layout);
    }
    Ok(ScenarioFile {
        source,
        source_name,
        pointers,
        scripts,
        warnings,
    })
}

fn validate_page_map(source: &[u8], pointers: &[u16]) -> Result<()> {
    let start = (usize::from(pointers[0]) - 1) * RECORD_SIZE;
    let end = (usize::from(pointers[1]) - 1) * RECORD_SIZE;
    if start != RECORD_SIZE || end <= start || !(end - start).is_multiple_of(RECORD_SIZE) {
        return Err("页面映射记录范围无效".to_string());
    }
    let mut terminated = false;
    for (index, pair) in source[start..end].chunks_exact(2).enumerate() {
        if terminated {
            if pair != [0, 0] {
                return Err(format!("页面映射终止项后索引 {index} 仍有数据"));
            }
            continue;
        }
        if pair == [0x1A, 0] {
            terminated = true;
            continue;
        }
        if !(1..=2).contains(&pair[0]) || pair[1] == 0 {
            return Err(format!(
                "页面映射索引 {index} 的磁盘/脚本号无效: {:02X} {:02X}",
                pair[0], pair[1]
            ));
        }
    }
    if !terminated {
        return Err("页面映射缺少 1A 00 终止项".to_string());
    }
    Ok(())
}

fn parse_script(index: usize, file_start: usize, bytes: &[u8]) -> Result<ScriptLayout> {
    if bytes.len() < 8 {
        return Err(format!("script {index}: 区域过短"));
    }
    let command_table = usize::from(read_u16(bytes, 0)?);
    if !(2..bytes.len()).contains(&command_table) {
        return Err(format!(
            "script {index}: 命令表偏移 0x{command_table:X} 越界"
        ));
    }
    let mut pointer_refs = vec![PointerRef {
        storage: 0,
        target: command_table,
    }];
    let (table_end, table_targets, table_ranges) =
        parse_command_table(index, bytes, command_table, &mut pointer_refs)?;
    let mut covered = vec![false; bytes.len()];
    mark_range(&mut covered, 0, 2)?;
    for (start, end) in table_ranges {
        mark_range(&mut covered, start, end)?;
    }

    let mut queue = VecDeque::new();
    queue.push_back(2usize);
    if table_targets.is_empty() {
        // 本作允许“命令表指针”直接指向一个默认处理代码入口。
        queue.push_back(command_table);
    } else {
        for target in table_targets {
            queue.push_back(target);
        }
    }
    let mut queued: HashSet<usize> = queue.iter().copied().collect();
    let mut spans = Vec::new();
    while let Some(start) = queue.pop_front() {
        parse_code_path(
            index,
            bytes,
            start,
            &mut covered,
            &mut spans,
            &mut pointer_refs,
            &mut queue,
            &mut queued,
        )?;
    }
    spans.sort_by_key(|span| span.start);
    for pair in spans.windows(2) {
        if pair[0].end > pair[1].start {
            return Err(format!("script {index}: 文本范围重叠"));
        }
    }

    let last_covered = covered
        .iter()
        .rposition(|value| *value)
        .map_or(table_end, |offset| offset + 1);
    let marker = bytes[last_covered..]
        .iter()
        .position(|byte| *byte == 0x1A)
        .map(|relative| last_covered + relative + 1);
    let active_end = marker.unwrap_or(last_covered);
    Ok(ScriptLayout {
        index,
        file_start,
        capacity: bytes.len(),
        text_spans: spans,
        pointer_refs,
        active_end,
    })
}

fn parse_command_table(
    script_index: usize,
    bytes: &[u8],
    start: usize,
    pointer_refs: &mut Vec<PointerRef>,
) -> Result<CommandTableParse> {
    let mut pos = start;
    let mut targets = Vec::new();
    let mut ranges = Vec::new();
    loop {
        let opcode = *bytes
            .get(pos)
            .ok_or_else(|| format!("script {script_index}: 命令表没有终止"))?;
        let entry_start = pos;
        match opcode {
            b'[' => pos += 1,
            b':' => {
                pos += 1;
                pos = skip_expression(script_index, bytes, pos)?;
            }
            _ => break,
        }
        let target_storage = pos
            .checked_add(2)
            .ok_or_else(|| "命令表偏移溢出".to_string())?;
        let target = usize::from(read_u16(bytes, target_storage)?);
        pos = pos
            .checked_add(4)
            .ok_or_else(|| "命令表项长度溢出".to_string())?;
        if target < pos || target >= bytes.len() {
            return Err(format!(
                "script {script_index}: 命令表目标 0x{target:X} 不在后续脚本区域"
            ));
        }
        pointer_refs.push(PointerRef {
            storage: target_storage,
            target,
        });
        targets.push(target);
        ranges.push((entry_start, pos));
    }
    if let Some(minimum) = targets.iter().min().copied() {
        if pos != minimum {
            return Err(format!(
                "script {script_index}: 命令表结束 0x{pos:X} 与首个目标 0x{minimum:X} 不一致"
            ));
        }
    }
    Ok((pos, targets, ranges))
}

#[allow(clippy::too_many_arguments)]
fn parse_code_path(
    script_index: usize,
    bytes: &[u8],
    start: usize,
    covered: &mut [bool],
    spans: &mut Vec<TextSpan>,
    pointer_refs: &mut Vec<PointerRef>,
    queue: &mut VecDeque<usize>,
    queued: &mut HashSet<usize>,
) -> Result<()> {
    if start >= bytes.len() {
        return Err(format!("script {script_index}: 代码入口 0x{start:X} 越界"));
    }
    let mut pos = start;
    while pos < bytes.len() {
        if covered[pos] {
            return Ok(());
        }
        if is_text_start(bytes[pos]) {
            let span_start = pos;
            let mut text = String::new();
            while pos < bytes.len() && is_text_start(bytes[pos]) {
                let (character, next) = decode_text_unit(script_index, bytes, pos)?;
                text.push(character);
                pos = next;
            }
            mark_range(covered, span_start, pos)?;
            spans.push(TextSpan {
                start: span_start,
                end: pos,
                entry_type: "message",
                text,
            });
            continue;
        }

        let opcode = bytes[pos];
        let instruction_start = pos;
        pos += 1;
        let mut terminal = false;
        match opcode {
            // `]` invokes NACT8S's input-code handler, then control returns to
            // BETA.OUT and parsing continues with the following instruction.
            b'A' | b'B' | b'R' | b'}' | b'M' | b']' => {}
            b'F' => terminal = true,
            b'Q' | b'G' | b'P' | b'X' | b'S' => pos = checked_advance(bytes, pos, 1)?,
            b'L' => {
                pos = checked_advance(bytes, pos, 1)?;
                terminal = true;
            }
            b'U' => pos = checked_advance(bytes, pos, 2)?,
            b'Y' | b'Z' => {
                pos = skip_expression(script_index, bytes, pos)?;
                pos = skip_expression(script_index, bytes, pos)?;
            }
            b'!' => {
                pos = skip_variable(bytes, pos)?;
                pos = skip_expression(script_index, bytes, pos)?;
            }
            b'&' => {
                pos = skip_expression(script_index, bytes, pos)?;
                terminal = true;
            }
            b'@' => {
                let storage = pos;
                let target = usize::from(read_u16(bytes, storage)?);
                pos = checked_advance(bytes, pos, 2)?;
                add_target(script_index, bytes, target, queue, queued)?;
                pointer_refs.push(PointerRef { storage, target });
                terminal = true;
            }
            b'$' => {
                let storage = pos;
                let target = usize::from(read_u16(bytes, storage)?);
                pos = checked_advance(bytes, pos, 2)?;
                add_target(script_index, bytes, target, queue, queued)?;
                pointer_refs.push(PointerRef { storage, target });
                let text_start = pos;
                let mut text = String::new();
                while *bytes
                    .get(pos)
                    .ok_or_else(|| format!("script {script_index}: 选择文本缺少 $ 终止符"))?
                    != b'$'
                {
                    if !is_text_start(bytes[pos]) {
                        return Err(format!(
                            "script {script_index}: 选择文本 0x{pos:X} 含非文本字节 {:02X}",
                            bytes[pos]
                        ));
                    }
                    let (character, next) = decode_text_unit(script_index, bytes, pos)?;
                    text.push(character);
                    pos = next;
                }
                if pos > text_start {
                    spans.push(TextSpan {
                        start: text_start,
                        end: pos,
                        entry_type: "choice",
                        text,
                    });
                }
                pos += 1;
            }
            b'{' => {
                pos = skip_expression(script_index, bytes, pos)?;
                let false_target = find_conditional_end(script_index, bytes, pos)?;
                if false_target < bytes.len() {
                    add_target(script_index, bytes, false_target, queue, queued)?;
                }
            }
            0 | 0x1A => return Ok(()),
            _ => {
                return Err(format!(
                    "script {script_index}: 代码 0x{instruction_start:X} 含未知操作码 0x{opcode:02X}"
                ));
            }
        }
        mark_range(covered, instruction_start, pos)?;
        if terminal {
            return Ok(());
        }
    }
    Ok(())
}

fn find_conditional_end(script_index: usize, bytes: &[u8], mut pos: usize) -> Result<usize> {
    let mut depth = 1usize;
    while pos < bytes.len() {
        if is_text_start(bytes[pos]) {
            while pos < bytes.len() && is_text_start(bytes[pos]) {
                let (_, next) = decode_text_unit(script_index, bytes, pos)?;
                pos = next;
            }
            continue;
        }

        let opcode = bytes[pos];
        pos += 1;
        match opcode {
            b'{' => {
                pos = skip_expression(script_index, bytes, pos)?;
                depth += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(pos);
                }
            }
            b'A' | b'B' | b'F' | b'R' | b'M' | b']' => {}
            b'Q' | b'G' | b'P' | b'X' | b'S' | b'L' => {
                pos = checked_advance(bytes, pos, 1)?;
            }
            b'U' | b'@' => pos = checked_advance(bytes, pos, 2)?,
            b'Y' | b'Z' => {
                pos = skip_expression(script_index, bytes, pos)?;
                pos = skip_expression(script_index, bytes, pos)?;
            }
            b'!' => {
                pos = skip_variable(bytes, pos)?;
                pos = skip_expression(script_index, bytes, pos)?;
            }
            b'&' => pos = skip_expression(script_index, bytes, pos)?,
            b'$' => {
                pos = checked_advance(bytes, pos, 2)?;
                loop {
                    let byte = *bytes.get(pos).ok_or_else(|| {
                        format!("script {script_index}: 条件块内选择文本缺少 $ 终止符")
                    })?;
                    if byte == b'$' {
                        pos += 1;
                        break;
                    }
                    if !is_text_start(byte) {
                        return Err(format!(
                            "script {script_index}: 条件块内选择文本 0x{pos:X} 含非文本字节 {byte:02X}"
                        ));
                    }
                    let (_, next) = decode_text_unit(script_index, bytes, pos)?;
                    pos = next;
                }
            }
            0 | 0x1A => {
                return Err(format!(
                    "script {script_index}: 条件块在终止符前缺少匹配的 }}"
                ));
            }
            _ => {
                return Err(format!(
                    "script {script_index}: 扫描条件块时在 0x{:X} 遇到未知操作码 0x{opcode:02X}",
                    pos - 1
                ));
            }
        }
    }
    Err(format!(
        "script {script_index}: 条件块越过脚本末尾，缺少匹配的 }}"
    ))
}

fn add_target(
    script_index: usize,
    bytes: &[u8],
    target: usize,
    queue: &mut VecDeque<usize>,
    queued: &mut HashSet<usize>,
) -> Result<()> {
    if !(2..bytes.len()).contains(&target) {
        return Err(format!(
            "script {script_index}: 本地跳转目标 0x{target:X} 越界"
        ));
    }
    if queued.insert(target) {
        queue.push_back(target);
    }
    Ok(())
}

fn skip_expression(script_index: usize, bytes: &[u8], mut pos: usize) -> Result<usize> {
    loop {
        let opcode = *bytes
            .get(pos)
            .ok_or_else(|| format!("script {script_index}: 表达式越过脚本末尾"))?;
        pos += 1;
        if opcode & 0x80 != 0 {
            if opcode & 0x40 != 0 {
                pos = checked_advance(bytes, pos, 1)?;
            }
            continue;
        }
        if opcode < 0x78 {
            if opcode & 0x40 == 0 {
                pos = checked_advance(bytes, pos, 1)?;
            }
            continue;
        }
        if opcode <= 0x7E {
            continue;
        }
        return Ok(pos);
    }
}

fn skip_variable(bytes: &[u8], pos: usize) -> Result<usize> {
    let value = *bytes
        .get(pos)
        .ok_or_else(|| "变量编码越过脚本末尾".to_string())?;
    checked_advance(bytes, pos, if value & 0x40 != 0 { 2 } else { 1 })
}

fn decode_text_unit(script_index: usize, bytes: &[u8], pos: usize) -> Result<(char, usize)> {
    let first = bytes[pos];
    if first == 0x20 {
        return Ok(('　', pos + 1));
    }
    if (0xA1..=0xDD).contains(&first) {
        let character = COMPRESSED_KANA
            .chars()
            .nth(usize::from(first - 0xA1))
            .ok_or_else(|| "压缩假名表索引越界".to_string())?;
        return Ok((character, pos + 1));
    }
    if (0xA0..0xE0).contains(&first) {
        return Err(format!(
            "script {script_index}: 0x{pos:X} 使用未定义的单字节文本 0x{first:02X}"
        ));
    }
    let pair = bytes
        .get(pos..pos + 2)
        .ok_or_else(|| format!("script {script_index}: 0x{pos:X} 的 CP932 字符被截断"))?;
    let decoded = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(pair)
        .ok_or_else(|| {
            format!(
                "script {script_index}: 0x{pos:X} 不是有效 CP932 双字节字符: {:02X} {:02X}",
                pair[0], pair[1]
            )
        })?;
    let mut chars = decoded.chars();
    let character = chars
        .next()
        .ok_or_else(|| "CP932 解码得到空字符串".to_string())?;
    if chars.next().is_some() {
        return Err("CP932 双字节解码得到多个字符".to_string());
    }
    Ok((character, pos + 2))
}

fn encode_message(text: &str, file: &str, entry_index: usize) -> Result<Vec<u8>> {
    let kana: HashMap<char, u8> = COMPRESSED_KANA
        .chars()
        .enumerate()
        .map(|(index, character)| (character, 0xA1 + index as u8))
        .collect();
    let mut output = Vec::new();
    let mut invalid = BTreeSet::new();
    for character in text.chars() {
        if character == '　' {
            output.push(0x20);
            continue;
        }
        if let Some(byte) = kana.get(&character) {
            output.push(*byte);
            continue;
        }
        if character == '\0' || character == '\r' || character == '\n' {
            invalid.insert(character);
            continue;
        }
        let source = character.to_string();
        let (encoded, _, had_errors) = SHIFT_JIS.encode(&source);
        if had_errors || encoded.len() != 2 || !is_two_byte_text_lead(encoded[0]) {
            invalid.insert(character);
            continue;
        }
        output.extend_from_slice(&encoded);
    }
    if !invalid.is_empty() {
        let display = invalid
            .iter()
            .map(|character| format!("{character:?} U+{:04X}", *character as u32))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "{file} entry {entry_index}: 字符不能用 Tauhido 文本编码表示: {display}；ASCII 字符请改用全角形式"
        ));
    }
    Ok(output)
}

fn encode_message_planned(
    text: &str,
    file: &str,
    entry_index: usize,
    plan: &EncodingPlan,
) -> Result<Vec<u8>> {
    let kana: HashMap<char, u8> = COMPRESSED_KANA
        .chars()
        .enumerate()
        .map(|(index, character)| (character, 0xA1 + index as u8))
        .collect();
    let mut output = Vec::new();
    for original in text.chars() {
        let normalized = crate::font::normalize_character(original)
            .map_err(|error| format!("{file} entry {entry_index}: {error}"))?;
        if normalized == '　' {
            output.push(0x20);
            continue;
        }
        let carrier = plan.carrier_for(normalized)?;
        if carrier == normalized {
            if let Some(byte) = kana.get(&normalized) {
                output.push(*byte);
                continue;
            }
        }
        output.extend_from_slice(&cp932_for_carrier(carrier)?);
    }
    Ok(output)
}

fn rebuild_scenario_file(
    source: &ScenarioFile,
    document: &TextDocument,
    plan: Option<&EncodingPlan>,
) -> Result<(Vec<u8>, usize)> {
    if document.format != TEXT_FORMAT {
        return Err(format!("{}: JSON 格式标记不受支持", document.source_file));
    }
    if document.source_file != source.source_name
        || document.source_sha256 != sha256_hex(&source.source)
    {
        return Err(format!("{}: JSON 与源文件不匹配", document.source_file));
    }
    let expected = build_text_document(source)?.document;
    if document.entries.len() != expected.entries.len() {
        return Err(format!(
            "{}: JSON 项数 {} 与源文件 {} 不一致",
            document.source_file,
            document.entries.len(),
            expected.entries.len()
        ));
    }

    let mut replacements_by_script: HashMap<usize, Vec<(usize, usize, Vec<u8>)>> = HashMap::new();
    let mut changed = 0usize;
    for (actual, original) in document.entries.iter().zip(expected.entries.iter()) {
        if actual.file != original.file
            || actual.index != original.index
            || actual.script_index != original.script_index
            || actual.offset != original.offset
            || actual.script_offset != original.script_offset
            || actual.size != original.size
            || actual.entry_type != original.entry_type
            || actual.scr_msg != original.scr_msg
        {
            return Err(format!(
                "{} entry {}: 元数据或 scr_msg 已改变",
                document.source_file, original.index
            ));
        }
        let encoded = match plan {
            Some(plan) => {
                encode_message_planned(&actual.message, &document.source_file, actual.index, plan)?
            }
            None => encode_message(&actual.message, &document.source_file, actual.index)?,
        };
        if actual.message != actual.scr_msg {
            changed += 1;
        }
        replacements_by_script
            .entry(actual.script_index)
            .or_default()
            .push((
                actual.script_offset as usize,
                actual.script_offset as usize + actual.size as usize,
                encoded,
            ));
    }

    let first_script = (usize::from(source.pointers[1]) - 1) * RECORD_SIZE;
    let mut rebuilt_scripts = Vec::with_capacity(source.scripts.len());
    for script in &source.scripts {
        let replacements = replacements_by_script
            .remove(&script.index)
            .unwrap_or_default();
        let original = &source.source[script.file_start..script.file_start + script.capacity];
        let rebuilt = rebuild_script(original, script, &replacements, plan.is_some())?;
        rebuilt_scripts.push(rebuilt);
    }
    if !replacements_by_script.is_empty() {
        return Err("JSON 包含不存在的脚本索引".to_string());
    }
    let mut output = source.source[..first_script].to_vec();
    let mut pointers = Vec::with_capacity(source.pointers.len());
    pointers.push(source.pointers[0]);
    for rebuilt in &rebuilt_scripts {
        pointers.push(
            u16::try_from(output.len() / RECORD_SIZE + 1)
                .map_err(|_| "脚本记录号超过 u16".to_string())?,
        );
        output.extend_from_slice(rebuilt);
    }
    pointers.push(
        u16::try_from(output.len() / RECORD_SIZE + 1)
            .map_err(|_| "脚本末记录号超过 u16".to_string())?,
    );
    if pointers.len() != source.pointers.len() {
        return Err("重建后的记录号表项数改变".to_string());
    }
    output[..RECORD_SIZE].fill(0);
    for (index, pointer) in pointers.iter().copied().enumerate() {
        output[index * 2..index * 2 + 2].copy_from_slice(&pointer.to_le_bytes());
    }
    let reparsed = parse_scenario_file(output.clone(), source.source_name.clone())?;
    let reextracted = build_text_document(&reparsed)?.document;
    let mut expected_entries = Vec::new();
    for actual in &document.entries {
        let text = match plan {
            Some(plan) => plan.normalize_text(&actual.message)?,
            None => actual.message.clone(),
        };
        // 空译文会把原文本跨度完全删除，重新解析时自然不再产生该条目。
        if !text.is_empty() {
            expected_entries.push((actual, text));
        }
    }
    if expected_entries.len() != reextracted.entries.len() {
        return Err(format!(
            "{}: 重建后文本条目数不一致；期望 {}（已排除空译文），实际 {}",
            document.source_file,
            expected_entries.len(),
            reextracted.entries.len()
        ));
    }
    for ((actual, expected), extracted) in expected_entries.iter().zip(&reextracted.entries) {
        let extracted_text = match plan {
            Some(plan) => plan.decode_carriers(&extracted.scr_msg),
            None => extracted.scr_msg.clone(),
        };
        if *expected != extracted_text {
            return Err(format!(
                "{} entry {} (script {}): 重建后复查文本不一致；期望 {:?}，实际 {:?}（回读 script {} offset 0x{:X}）",
                document.source_file,
                actual.index,
                actual.script_index,
                expected,
                extracted_text,
                extracted.script_index,
                extracted.script_offset
            ));
        }
    }
    Ok((output, changed))
}

fn rebuild_script(
    original: &[u8],
    layout: &ScriptLayout,
    replacements: &[(usize, usize, Vec<u8>)],
    resizable: bool,
) -> Result<Vec<u8>> {
    let mut replacements = replacements.to_vec();
    replacements.sort_by_key(|item| item.0);
    for pair in replacements.windows(2) {
        if pair[0].1 > pair[1].0 {
            return Err(format!("script {}: JSON 文本范围重叠", layout.index));
        }
    }
    let expected: Vec<(usize, usize)> = layout
        .text_spans
        .iter()
        .map(|span| (span.start, span.end))
        .collect();
    let actual: Vec<(usize, usize)> = replacements
        .iter()
        .map(|(start, end, _)| (*start, *end))
        .collect();
    if actual != expected {
        return Err(format!("script {}: JSON 文本范围集合不完整", layout.index));
    }

    let active_end = layout.active_end;
    let mut rebuilt = Vec::new();
    let mut cursor = 0usize;
    for (start, end, replacement) in &replacements {
        rebuilt.extend_from_slice(&original[cursor..*start]);
        rebuilt.extend_from_slice(replacement);
        cursor = *end;
    }
    rebuilt.extend_from_slice(&original[cursor..active_end]);
    let new_active_end = rebuilt.len();
    if !resizable && new_active_end > layout.capacity {
        return Err(format!(
            "script {}: 重建需 {} 字节，固定槽只有 {} 字节",
            layout.index, new_active_end, layout.capacity
        ));
    }

    for reference in &layout.pointer_refs {
        let new_storage = map_offset(reference.storage, &replacements)?;
        let new_target = map_offset(reference.target, &replacements)?;
        let target = u16::try_from(new_target)
            .map_err(|_| format!("script {}: 重建目标偏移超过 u16", layout.index))?;
        let slot = rebuilt
            .get_mut(new_storage..new_storage + 2)
            .ok_or_else(|| format!("script {}: 重建指针位置越界", layout.index))?;
        slot.copy_from_slice(&target.to_le_bytes());
    }
    if rebuilt[..new_active_end] == original[..layout.active_end] {
        return Ok(original.to_vec());
    }
    let opaque_tail = &original[layout.active_end..];
    let first_opaque = opaque_tail
        .iter()
        .position(|byte| *byte != 0)
        .map(|relative| layout.active_end + relative);
    if resizable {
        if let Some(opaque_start) = first_opaque {
            if new_active_end > opaque_start {
                let has_terminal_marker =
                    layout.active_end > 0 && original.get(layout.active_end - 1) == Some(&0x1A);
                let referenced = layout
                    .pointer_refs
                    .iter()
                    .any(|reference| reference.target >= layout.active_end);
                if !has_terminal_marker || referenced {
                    return Err(format!(
                        "script {}: 重建活动区结束于 0x{:X}，会覆盖从 0x{:X} 开始且不能安全重定位的未知非零数据",
                        layout.index, new_active_end, opaque_start
                    ));
                }
                // 0x1A is the script terminator. Bytes behind it are not on any
                // parsed execution path and no known pointer targets them. Keep
                // the complete post-terminator region, but move it behind the
                // expanded active region instead of overwriting it.
                rebuilt.extend_from_slice(&original[layout.active_end..]);
                let target = round_up_to_record(rebuilt.len())?;
                rebuilt.resize(target, 0);
                return Ok(rebuilt);
            }
            // 未知块不参与重定位：保持其脚本内偏移和全部原字节。
            rebuilt.resize(opaque_start, 0);
            rebuilt.extend_from_slice(&original[opaque_start..]);
            return Ok(rebuilt);
        }
    }
    let target = if resizable {
        round_up_to_record(new_active_end)?
    } else {
        layout.capacity
    };
    rebuilt.resize(target, 0);
    Ok(rebuilt)
}

fn round_up_to_record(value: usize) -> Result<usize> {
    value
        .checked_add(RECORD_SIZE - 1)
        .map(|sum| sum / RECORD_SIZE * RECORD_SIZE)
        .ok_or_else(|| "记录对齐溢出".to_string())
}

fn map_offset(offset: usize, replacements: &[(usize, usize, Vec<u8>)]) -> Result<usize> {
    let mut delta: i64 = 0;
    for (start, end, replacement) in replacements {
        // 跳转目标可以合法地落在文本首字节；它应映射到替换文本的新起点。
        if offset <= *start {
            break;
        }
        if offset < *end {
            return Err(format!("指针或结构偏移 0x{offset:X} 落在可编辑文本内部"));
        }
        delta += replacement.len() as i64 - (*end - *start) as i64;
    }
    let mapped = offset as i64 + delta;
    usize::try_from(mapped).map_err(|_| "重建偏移下溢".to_string())
}

fn write_text_workspace(prepared: &[PreparedText], staging: &Path) -> Result<()> {
    let mut names = HashSet::new();
    let mut sources = Vec::new();
    for item in prepared {
        let stem = Path::new(&item.document.source_file)
            .file_stem()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| "scenario".to_string());
        let json_name = format!("{stem}.json");
        if !names.insert(json_name.to_lowercase()) {
            return Err(format!("多个源文件会生成同名 JSON: {json_name}"));
        }
        write_json(&staging.join(&json_name), &item.document)?;
        sources.push(TextWorkspaceSource {
            source_file: item.document.source_file.clone(),
            source_sha256: item.document.source_sha256.clone(),
            json_file: json_name,
            scripts: item.scripts,
            entries: item.document.entries.len(),
        });
    }
    let manifest = TextWorkspaceManifest {
        format: TEXT_WORKSPACE_FORMAT.to_string(),
        summary: TextSummary {
            files: sources.len(),
            scripts: sources.iter().map(|source| source.scripts).sum(),
            entries: sources.iter().map(|source| source.entries).sum(),
            warnings: prepared.iter().map(|item| item.warnings.len()).sum(),
        },
        sources,
    };
    write_json(&staging.join("text_manifest.json"), &manifest)
}

fn read_translation_documents(root: &Path) -> Result<Vec<TextDocument>> {
    if !root.is_dir() {
        return Err(format!("翻译路径不是目录: {}", root.display()));
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(root)
        .map_err(|error| format!("读取翻译目录 {} 失败: {error}", root.display()))?
    {
        let entry = entry.map_err(|error| format!("读取翻译目录项失败: {error}"))?;
        if entry
            .file_type()
            .map_err(|error| format!("读取 {} 类型失败: {error}", entry.path().display()))?
            .is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        {
            paths.push(entry.path());
        }
    }
    paths.sort_by_key(|path| path.to_string_lossy().to_lowercase());
    let mut documents = Vec::new();
    for path in paths {
        let bytes =
            fs::read(&path).map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|error| format!("{}: JSON 无效: {error}", path.display()))?;
        if value.get("_format").and_then(|item| item.as_str()) != Some(TEXT_FORMAT) {
            continue;
        }
        documents.push(
            serde_json::from_value(value)
                .map_err(|error| format!("{}: 翻译 JSON 结构无效: {error}", path.display()))?,
        );
    }
    Ok(documents)
}

fn resolve_scenario_inputs(inputs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    if inputs.is_empty() {
        return Err("至少需要一个 DISK 脚本文件或目录".to_string());
    }
    let mut resolved = Vec::new();
    for input in inputs {
        let metadata = fs::metadata(input)
            .map_err(|error| format!("无法访问 {}: {error}", input.display()))?;
        if metadata.is_file() {
            resolved.push(input.clone());
            continue;
        }
        if !metadata.is_dir() {
            return Err(format!("输入不是普通文件或目录: {}", input.display()));
        }
        let mut found = Vec::new();
        for entry in fs::read_dir(input)
            .map_err(|error| format!("无法读取目录 {}: {error}", input.display()))?
        {
            let entry = entry.map_err(|error| format!("读取目录项失败: {error}"))?;
            if !entry
                .file_type()
                .map_err(|error| format!("读取 {} 类型失败: {error}", entry.path().display()))?
                .is_file()
            {
                continue;
            }
            let bytes = match fs::read(entry.path()) {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };
            let name = entry.file_name().to_string_lossy().into_owned();
            if parse_scenario_file(bytes, name).is_ok() {
                found.push(entry.path());
            }
        }
        found.sort_by_key(|path| path.to_string_lossy().to_lowercase());
        if found.is_empty() {
            return Err(format!(
                "目录中没有检测到 Tauhido 脚本文件: {}",
                input.display()
            ));
        }
        resolved.extend(found);
    }
    let mut seen = HashSet::new();
    let mut unique = Vec::new();
    for path in resolved {
        let canonical = fs::canonicalize(&path)
            .map_err(|error| format!("无法规范化 {}: {error}", path.display()))?;
        let key = canonical.to_string_lossy().to_lowercase();
        if seen.insert(key) {
            unique.push(canonical);
        }
    }
    Ok(unique)
}

fn validate_output_root(root: &Path, overwrite: bool, format: &str, marker: &str) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    if !root.is_dir() {
        return Err(format!("输出已存在且不是目录: {}", root.display()));
    }
    if !overwrite {
        return Err(format!(
            "输出目录已存在；需要显式 --overwrite: {}",
            root.display()
        ));
    }
    let mut entries = fs::read_dir(root).map_err(|error| format!("读取输出目录失败: {error}"))?;
    if entries.next().is_none() {
        return Ok(());
    }
    let bytes = fs::read(root.join(marker)).map_err(|_| {
        format!(
            "拒绝覆盖非本工具工作区（缺少有效 {marker}）: {}",
            root.display()
        )
    })?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| {
        format!(
            "拒绝覆盖非本工具工作区（{marker} 无效）: {}",
            root.display()
        )
    })?;
    if value.get("_format").and_then(|item| item.as_str()) != Some(format) {
        return Err(format!("拒绝覆盖其他格式的工作区: {}", root.display()));
    }
    Ok(())
}

fn create_staging(output_root: &Path, purpose: &str) -> Result<PathBuf> {
    let parent = output_root
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("创建输出父目录 {} 失败: {error}", parent.display()))?;
    let name = output_root
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "tauhido-output".to_string());
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("系统时间异常: {error}"))?
        .as_nanos();
    for attempt in 0..1000u32 {
        let candidate = parent.join(format!(
            ".{name}.{purpose}.{}.{nonce}.{attempt}",
            std::process::id()
        ));
        if !candidate.exists() {
            fs::create_dir(&candidate)
                .map_err(|error| format!("创建暂存目录 {} 失败: {error}", candidate.display()))?;
            return Ok(candidate);
        }
    }
    Err(format!("无法为 {} 分配暂存目录", output_root.display()))
}

fn commit_staging(staging: &Path, output_root: &Path, overwrite: bool) -> Result<()> {
    if !output_root.exists() {
        return fs::rename(staging, output_root).map_err(|error| format!("提交输出失败: {error}"));
    }
    if !overwrite {
        return Err("输出在写入期间被其他进程创建".to_string());
    }
    let parent = output_root.parent().unwrap_or_else(|| Path::new("."));
    let backup = parent.join(format!(
        ".{}.backup.{}.{}",
        output_root
            .file_name()
            .map(|value| value.to_string_lossy())
            .unwrap_or_else(|| "tauhido-output".into()),
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("系统时间异常: {error}"))?
            .as_nanos()
    ));
    fs::rename(output_root, &backup).map_err(|error| format!("备份旧输出失败: {error}"))?;
    if let Err(error) = fs::rename(staging, output_root) {
        let restore = fs::rename(&backup, output_root);
        return match restore {
            Ok(()) => Err(format!("提交新输出失败，旧输出已恢复: {error}")),
            Err(restore_error) => Err(format!(
                "提交新输出失败且旧输出恢复失败: {error}; {restore_error}"
            )),
        };
    }
    fs::remove_dir_all(&backup)
        .map_err(|error| format!("新输出已提交，但清理旧备份失败: {error}"))?;
    Ok(())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut json = serde_json::to_string_pretty(value)
        .map_err(|error| format!("序列化 {} 失败: {error}", path.display()))?;
    json.push('\n');
    fs::write(path, json.as_bytes())
        .map_err(|error| format!("写入 {} 失败: {error}", path.display()))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let raw = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| format!("在 0x{offset:X} 读取 u16 越界"))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn checked_advance(bytes: &[u8], pos: usize, count: usize) -> Result<usize> {
    let end = pos
        .checked_add(count)
        .ok_or_else(|| "脚本偏移溢出".to_string())?;
    if end > bytes.len() {
        return Err("命令参数越过脚本末尾".to_string());
    }
    Ok(end)
}

fn mark_range(covered: &mut [bool], start: usize, end: usize) -> Result<()> {
    let range = covered
        .get_mut(start..end)
        .ok_or_else(|| format!("结构范围 0x{start:X}..0x{end:X} 越界"))?;
    range.fill(true);
    Ok(())
}

fn is_text_start(byte: u8) -> bool {
    byte == 0x20 || byte > 0x7F
}

fn is_two_byte_text_lead(byte: u8) -> bool {
    (0x81..=0x9F).contains(&byte) || (0xE0..=0xFC).contains(&byte)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expression_lengths_match_mm3_evaluator() {
        assert_eq!(skip_expression(1, &[0x41, 0x7F], 0).unwrap(), 2);
        assert_eq!(skip_expression(1, &[0x00, 0x2A, 0x7F], 0).unwrap(), 3);
        assert_eq!(
            skip_expression(1, &[0x80, 0xC1, 0x02, 0x79, 0x7F], 0).unwrap(),
            5
        );
    }

    #[test]
    fn hybrid_text_round_trip() {
        let text = "　「たうひーど」漢字";
        let encoded = encode_message(text, "sample", 0).unwrap();
        let mut decoded = String::new();
        let mut pos = 0;
        while pos < encoded.len() {
            let (character, next) = decode_text_unit(1, &encoded, pos).unwrap();
            decoded.push(character);
            pos = next;
        }
        assert_eq!(decoded, text);
    }

    #[test]
    fn rejects_ascii_that_would_be_an_opcode() {
        assert!(encode_message("ABC", "sample", 0).is_err());
    }

    #[test]
    fn closing_input_code_does_not_terminate_the_script_path() {
        let bytes = [b']', b'}', 0x82, 0xA0, 0];
        let mut covered = vec![false; bytes.len()];
        let mut spans = Vec::new();
        let mut pointer_refs = Vec::new();
        let mut queue = VecDeque::new();
        let mut queued = HashSet::new();

        parse_code_path(
            1,
            &bytes,
            0,
            &mut covered,
            &mut spans,
            &mut pointer_refs,
            &mut queue,
            &mut queued,
        )
        .unwrap();

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].start, 2);
        assert_eq!(spans[0].text, "あ");
    }

    #[test]
    fn expanded_script_relocates_only_unreferenced_post_terminator_bytes() {
        let mut original = vec![0u8; RECORD_SIZE];
        original[..3].copy_from_slice(&[0x82, 0xA0, 0x1A]);
        original[5..8].copy_from_slice(&[0xDE, 0xAD, 0xBE]);
        let layout = ScriptLayout {
            index: 1,
            file_start: 0,
            capacity: original.len(),
            text_spans: vec![TextSpan {
                start: 0,
                end: 2,
                entry_type: "message",
                text: "あ".to_string(),
            }],
            pointer_refs: Vec::new(),
            active_end: 3,
        };
        let replacement = vec![0x82, 0xA0, 0x82, 0xA2, 0x82, 0xA4];

        let rebuilt = rebuild_script(&original, &layout, &[(0, 2, replacement)], true).unwrap();

        assert_eq!(&rebuilt[6..7], &[0x1A]);
        assert_eq!(&rebuilt[9..12], &[0xDE, 0xAD, 0xBE]);
        assert_eq!(rebuilt.len(), RECORD_SIZE * 2);
    }

    #[test]
    fn conditional_queues_the_path_after_its_closing_brace() {
        let bytes = [b'{', 0x41, 0x7F, b'@', 9, 0, b'}', 0x82, 0xA0, 0];
        let mut covered = vec![false; bytes.len()];
        let mut spans = Vec::new();
        let mut pointer_refs = Vec::new();
        let mut queue = VecDeque::new();
        let mut queued = HashSet::new();

        parse_code_path(
            1,
            &bytes,
            0,
            &mut covered,
            &mut spans,
            &mut pointer_refs,
            &mut queue,
            &mut queued,
        )
        .unwrap();

        assert!(queued.contains(&7));
        assert!(queued.contains(&9));
        while let Some(target) = queue.pop_front() {
            parse_code_path(
                1,
                &bytes,
                target,
                &mut covered,
                &mut spans,
                &mut pointer_refs,
                &mut queue,
                &mut queued,
            )
            .unwrap();
        }
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "あ");
    }
}
