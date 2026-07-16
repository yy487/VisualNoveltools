use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use sha2::{Digest, Sha256};

use crate::archive::{PaclArchive, pack_compress_literals, pack_decompress};
use crate::script::{ScriptAnalysis, analyze_tpc32, encode_cp932_double_byte, patch_tpc32};
use crate::text_json::{FORMAT_ID, ProjectProfile, TextEntry, TranslationDocument, read_document};

#[derive(Debug, Clone)]
pub struct ExtractReport {
    pub scanned_files: usize,
    pub instruction_count: usize,
    pub extracted_entries: usize,
    pub message_entries: usize,
    pub choice_entries: usize,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
pub struct InjectReport {
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub modified_files: usize,
    pub output_bytes: usize,
    pub byte_exact_no_change: bool,
    pub output: PathBuf,
}

struct ScannedItem {
    plain: Vec<u8>,
    analysis: ScriptAnalysis,
    global_text_start: usize,
}

struct Binding {
    item_index: usize,
    text_index: usize,
}

struct Scan {
    archive: PaclArchive,
    items: Vec<ScannedItem>,
    entries: Vec<TextEntry>,
    bindings: Vec<Binding>,
    instruction_count: usize,
}

fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        write!(&mut out, "{byte:02x}").expect("writing to String cannot fail");
    }
    out
}

fn scan_archive(data: &[u8]) -> Result<Scan> {
    let archive = PaclArchive::parse(data)?;
    let mut scanned_items = Vec::with_capacity(archive.items.len());
    let mut entries = Vec::new();
    let mut bindings = Vec::new();
    let mut instruction_count = 0usize;

    for item in &archive.items {
        let plain = pack_decompress(&item.block)
            .with_context(|| format!("{}: PACK 解压失败", item.name))?;
        ensure!(
            plain.len() == item.unpacked_size,
            "{}: 解压尺寸不一致",
            item.name
        );
        let analysis = analyze_tpc32(&item.name, &plain)?;
        instruction_count += analysis.instruction_count;
        let global_text_start = entries.len();
        for (text_index, text) in analysis.texts.iter().enumerate() {
            let global_index = entries.len();
            entries.push(TextEntry {
                file: item.name.clone(),
                file_index: item.index,
                subscript: text.subscript_name.clone(),
                index: global_index,
                offset: text.instruction_offset,
                subscript_offset: text.subscript_offset,
                size: text.byte_length,
                entry_type: text.kind.as_str().to_owned(),
                opcode: format!("{:02X}", text.opcode),
                choice_index: text.choice_index,
                scr_msg: text.text.clone(),
                message: text.text.clone(),
            });
            bindings.push(Binding {
                item_index: item.index,
                text_index,
            });
        }
        scanned_items.push(ScannedItem {
            plain,
            analysis,
            global_text_start,
        });
    }

    Ok(Scan {
        archive,
        items: scanned_items,
        entries,
        bindings,
        instruction_count,
    })
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

pub fn default_extract_output(input: &Path) -> PathBuf {
    append_suffix(input, ".json")
}

fn default_inject_output(source: &Path) -> Result<PathBuf> {
    let stem = source
        .file_stem()
        .context("源文件没有可用文件名")?
        .to_os_string();
    let mut name = stem;
    name.push("_injected");
    if let Some(extension) = source.extension() {
        name.push(".");
        name.push(extension);
    }
    Ok(source.with_file_name(name))
}

fn temp_output_path(output: &Path) -> Result<PathBuf> {
    let file_name = output.file_name().context("输出路径缺少文件名")?;
    let mut temp_name = OsString::from(".");
    temp_name.push(file_name);
    temp_name.push(format!(".tmp-{}", std::process::id()));
    Ok(output.with_file_name(temp_name))
}

fn write_new_file(output: &Path, data: &[u8]) -> Result<()> {
    ensure!(
        !output.exists(),
        "输出已存在，拒绝覆盖：{}",
        output.display()
    );
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    ensure!(parent.is_dir(), "输出目录不存在：{}", parent.display());
    let temp = temp_output_path(output)?;
    ensure!(!temp.exists(), "临时输出已存在：{}", temp.display());
    if let Err(error) = fs::write(&temp, data) {
        let _ = fs::remove_file(&temp);
        return Err(error).with_context(|| format!("写入临时文件失败：{}", temp.display()));
    }
    if let Err(error) = fs::rename(&temp, output) {
        let _ = fs::remove_file(&temp);
        return Err(error)
            .with_context(|| format!("提交输出失败：{} -> {}", temp.display(), output.display()));
    }
    Ok(())
}

pub fn extract_file(input: &Path, output: Option<&Path>) -> Result<ExtractReport> {
    ensure!(input.is_file(), "输入不是文件：{}", input.display());
    let output = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_extract_output(input));
    ensure!(
        !output.exists(),
        "输出已存在，拒绝覆盖：{}",
        output.display()
    );

    let source = fs::read(input).with_context(|| format!("读取失败：{}", input.display()))?;
    let scan = scan_archive(&source)?;
    let source_file = input
        .file_name()
        .context("输入路径缺少文件名")?
        .to_string_lossy()
        .into_owned();
    let document = TranslationDocument {
        format: FORMAT_ID.to_owned(),
        source_file,
        source_size: source.len() as u64,
        source_sha256: sha256_hex(&source),
        profile: ProjectProfile::default(),
        entries: scan.entries,
    };
    let mut json = serde_json::to_string_pretty(&document).context("序列化翻译 JSON 失败")?;
    json.push('\n');
    write_new_file(&output, json.as_bytes())?;

    let message_entries = document
        .entries
        .iter()
        .filter(|entry| entry.entry_type == "message")
        .count();
    let choice_entries = document.entries.len() - message_entries;
    Ok(ExtractReport {
        scanned_files: scan.archive.items.len(),
        instruction_count: scan.instruction_count,
        extracted_entries: document.entries.len(),
        message_entries,
        choice_entries,
        output,
    })
}

fn validate_immutable(json: &TextEntry, actual: &TextEntry, position: usize) -> Result<()> {
    ensure!(
        json.index == position,
        "JSON 条目 {position}: _index 应为 {position}"
    );
    ensure!(
        json.file == actual.file,
        "JSON 条目 {position}: _file 不匹配：JSON={}，源={}",
        json.file,
        actual.file
    );
    ensure!(
        json.file_index == actual.file_index,
        "JSON 条目 {position}: _file_index 不匹配"
    );
    ensure!(
        json.subscript == actual.subscript,
        "JSON 条目 {position}: _subscript 不匹配"
    );
    ensure!(
        json.offset == actual.offset,
        "JSON 条目 {position}: _offset 不匹配"
    );
    ensure!(
        json.subscript_offset == actual.subscript_offset,
        "JSON 条目 {position}: _subscript_offset 不匹配"
    );
    ensure!(
        json.size == actual.size,
        "JSON 条目 {position}: _size 不匹配"
    );
    ensure!(
        json.entry_type == actual.entry_type,
        "JSON 条目 {position}: _type 不匹配"
    );
    ensure!(
        json.opcode == actual.opcode,
        "JSON 条目 {position}: _opcode 不匹配"
    );
    ensure!(
        json.choice_index == actual.choice_index,
        "JSON 条目 {position}: _choice_index 不匹配"
    );
    ensure!(
        json.scr_msg == actual.scr_msg,
        "JSON 条目 {position}: scr_msg 已被修改或源文件不匹配"
    );
    Ok(())
}

fn resolve_source(
    json_path: &Path,
    document: &TranslationDocument,
    source: Option<&Path>,
) -> PathBuf {
    if let Some(source) = source {
        return source.to_path_buf();
    }
    json_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&document.source_file)
}

pub fn inject_file(
    json_path: &Path,
    source_override: Option<&Path>,
    output_override: Option<&Path>,
) -> Result<InjectReport> {
    ensure!(
        json_path.is_file(),
        "JSON 输入不是文件：{}",
        json_path.display()
    );
    let document = read_document(json_path)?;
    ensure!(
        document.format == FORMAT_ID,
        "不支持的 JSON 格式：{}",
        document.format
    );
    ensure!(!document.profile.name_field, "本项目配置禁止 name 字段");
    ensure!(
        document
            .profile
            .source_encoding
            .eq_ignore_ascii_case("CP932")
            && document
                .profile
                .target_encoding
                .eq_ignore_ascii_case("CP932"),
        "当前版本只支持 CP932 -> CP932"
    );

    let source_path = resolve_source(json_path, &document, source_override);
    ensure!(
        source_path.is_file(),
        "找不到源 SEEN.TXT：{}；可使用 --source 指定",
        source_path.display()
    );
    let output = match output_override {
        Some(path) => path.to_path_buf(),
        None => default_inject_output(&source_path)?,
    };
    ensure!(
        !output.exists(),
        "输出已存在，拒绝覆盖：{}",
        output.display()
    );

    let source =
        fs::read(&source_path).with_context(|| format!("读取失败：{}", source_path.display()))?;
    ensure!(
        source.len() as u64 == document.source_size,
        "源文件尺寸不匹配：JSON={}，实际={}",
        document.source_size,
        source.len()
    );
    let actual_hash = sha256_hex(&source);
    ensure!(
        actual_hash.eq_ignore_ascii_case(&document.source_sha256),
        "源文件 SHA-256 不匹配：JSON={}，实际={actual_hash}",
        document.source_sha256
    );

    let scan = scan_archive(&source)?;
    ensure!(
        document.entries.len() == scan.entries.len(),
        "JSON/源文本条目数不一致：JSON={}，源={}",
        document.entries.len(),
        scan.entries.len()
    );

    let mut replacements_by_item: HashMap<usize, HashMap<usize, Vec<u8>>> = HashMap::new();
    let mut patched = 0usize;
    for (position, (json_entry, actual_entry)) in
        document.entries.iter().zip(&scan.entries).enumerate()
    {
        validate_immutable(json_entry, actual_entry, position)?;
        if json_entry.message == json_entry.scr_msg {
            continue;
        }
        let label = format!(
            "JSON 条目 {} ({}:{} 0x{:X})",
            position, json_entry.file, json_entry.subscript, json_entry.offset
        );
        let encoded = encode_cp932_double_byte(&json_entry.message, &label)?;
        let binding = &scan.bindings[position];
        replacements_by_item
            .entry(binding.item_index)
            .or_default()
            .insert(binding.text_index, encoded);
        patched += 1;
    }

    let output_bytes = if patched == 0 {
        source.clone()
    } else {
        let mut pack_replacements = HashMap::new();
        for (&item_index, replacements) in &replacements_by_item {
            let scanned = scan
                .items
                .get(item_index)
                .with_context(|| format!("PACL 条目索引 {item_index} 越界"))?;
            let patched_plain = patch_tpc32(&scanned.plain, &scanned.analysis, replacements)
                .with_context(|| {
                    format!("{}: TPC32 注入失败", scan.archive.items[item_index].name)
                })?;
            let verified = analyze_tpc32(&scan.archive.items[item_index].name, &patched_plain)
                .with_context(|| {
                    format!(
                        "{}: 注入后结构验证失败",
                        scan.archive.items[item_index].name
                    )
                })?;
            ensure!(
                verified.texts.len() == scanned.analysis.texts.len(),
                "{}: 注入后文本条目数变化",
                scan.archive.items[item_index].name
            );
            for &text_index in replacements.keys() {
                let global_position = scanned
                    .global_text_start
                    .checked_add(text_index)
                    .context("文本全局索引溢出")?;
                ensure!(
                    verified.texts[text_index].text == document.entries[global_position].message,
                    "{}: 注入后文本回读不一致",
                    scan.archive.items[item_index].name
                );
            }
            pack_replacements.insert(item_index, pack_compress_literals(&patched_plain)?);
        }
        let rebuilt = scan.archive.repack(&pack_replacements)?;
        let verification = scan_archive(&rebuilt).context("重建 SEEN.TXT 全量验证失败")?;
        ensure!(
            verification.entries.len() == document.entries.len(),
            "重建后文本条目数变化"
        );
        rebuilt
    };

    let byte_exact_no_change = patched == 0 && output_bytes == source;
    if patched == 0 && !byte_exact_no_change {
        bail!("无修改注入没有保持字节完全一致");
    }
    write_new_file(&output, &output_bytes)?;

    Ok(InjectReport {
        json_entries: document.entries.len(),
        patched,
        unchanged: document.entries.len() - patched,
        modified_files: replacements_by_item.len(),
        output_bytes: output_bytes.len(),
        byte_exact_no_change,
        output,
    })
}
