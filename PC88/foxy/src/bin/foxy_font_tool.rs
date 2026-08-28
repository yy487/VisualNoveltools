use encoding_rs::SHIFT_JIS;
use foxy_d88_tool::font;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

type AppResult<T> = Result<T, String>;

const OUTPUT_MARKER: &str = ".foxy_font_tool_managed";
const EXPECTED_MES_FILES: usize = 28;
const EXPLICIT_NEWLINE: [u8; 2] = [0x81, 0x93];

#[derive(Debug)]
enum Command {
    Render(RenderCli),
    Build(BuildCli),
}

#[derive(Debug, Clone)]
struct RenderCli {
    rom: PathBuf,
    output: PathBuf,
    overwrite: bool,
}

#[derive(Debug, Clone)]
struct BuildCli {
    rom: PathBuf,
    workspace: PathBuf,
    translations: PathBuf,
    output: PathBuf,
    overwrite: bool,
}

#[derive(Debug, Default, Clone)]
struct InteractivePrefill {
    rom: Option<PathBuf>,
    workspace: Option<PathBuf>,
    translations: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct TranslationDocument {
    format: String,
    version: u32,
    profile: TranslationProfile,
    files: Vec<TranslationFile>,
}

#[derive(Debug, Deserialize)]
struct TranslationProfile {
    engine: String,
    encoding: String,
    speaker_policy: String,
}

#[derive(Debug, Deserialize)]
struct TranslationFile {
    #[serde(rename = "_file")]
    file: String,
    #[serde(rename = "_sha256")]
    sha256: String,
    entries: Vec<TranslationEntry>,
}

#[derive(Debug, Deserialize)]
struct TranslationEntry {
    #[serde(rename = "_index")]
    index: usize,
    #[serde(rename = "_offset")]
    offset: usize,
    #[serde(rename = "_size")]
    size: usize,
    #[serde(rename = "_type")]
    kind: String,
    scr_msg: Option<String>,
    message: Option<String>,
    message_parts: Option<Vec<MessagePart>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum MessagePart {
    Text {
        scr_msg: String,
        message: String,
    },
    Raw {
        #[serde(rename = "_hex")]
        hex: String,
    },
}

#[derive(Debug, PartialEq, Eq)]
enum SourcePart {
    Text(String),
    Raw(String),
}

struct FontInputs {
    original_codes: Vec<u16>,
    translated_texts: Vec<String>,
    mes_files: usize,
    entries: usize,
}

#[derive(Serialize)]
struct ProbeManifest {
    format: &'static str,
    version: u32,
    tool: &'static str,
    source_file: String,
    source_size: usize,
    source_sha256: String,
    layout: &'static str,
    scale: usize,
    page_count: usize,
    pages: Vec<PageManifest>,
}

#[derive(Serialize)]
struct PageManifest {
    file: String,
    entries: usize,
    width: usize,
    height: usize,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("错误：{error}");
        std::process::exit(1);
    }
}

fn run() -> AppResult<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        return interactive(InteractivePrefill::default());
    }
    if args.len() == 1 {
        let first = args[0].to_string_lossy();
        if matches!(first.as_ref(), "-h" | "--help" | "help") {
            print_help();
            return Ok(());
        }
        if !first.starts_with('-') && first != "render" && first != "build" {
            return interactive(classify_prefill(PathBuf::from(&args[0]))?);
        }
    }
    match parse_command(&args)? {
        Command::Render(cli) => render_probe(&cli),
        Command::Build(cli) => build_font(&cli),
    }
}

fn print_help() {
    println!("FOXY KANJI1.ROM 独立字库工具");
    println!();
    println!("用法：");
    println!("  foxy_font_tool");
    println!("  foxy_font_tool <path>");
    println!("  foxy_font_tool render --rom <KANJI1.ROM> --output <DIR> [--overwrite]");
    println!(
        "  foxy_font_tool build --rom <KANJI1.ROM> --workspace <UNPACKED_DIR> --translations <MESSAGES_JSON_OR_DIR> --output <DIR> [--overwrite]"
    );
    println!();
    println!("render：把原始 ROM 渲染成清晰分页探针图。");
    println!("build ：冻结原文与译文原生槽，动态分配载体并重建 KANJI1.ROM。");
    println!("本程序不会注入 MES，也不会封装 D88。");
    println!("无参数时进入持续菜单；仅传一个路径时只预填菜单，确认前不会写文件。");
    println!("完整 render/build 命令为一次性非交互执行；已有输出必须显式使用 --overwrite。");
}

fn parse_command(args: &[OsString]) -> AppResult<Command> {
    let command = args
        .first()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "missing or non-Unicode command".to_string())?;
    match command {
        "render" => Ok(Command::Render(parse_render_args(&args[1..])?)),
        "build" => Ok(Command::Build(parse_build_args(&args[1..])?)),
        "-h" | "--help" | "help" => {
            print_help();
            Err("help requested after additional arguments".to_string())
        }
        _ => Err(format!("unknown command {command:?}; use --help")),
    }
}

fn parse_render_args(args: &[OsString]) -> AppResult<RenderCli> {
    let mut rom = None;
    let mut output = None;
    let mut overwrite = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| "render option is not valid Unicode".to_string())?;
        match option {
            "--rom" | "--input" | "-i" => {
                index += 1;
                rom = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| format!("{option} requires a path"))?,
                ));
            }
            "--output" | "-o" => {
                index += 1;
                output = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| format!("{option} requires a path"))?,
                ));
            }
            "--overwrite" => overwrite = true,
            "--non-interactive" => {}
            "-h" | "--help" => {
                print_help();
                return Err("render help requested".to_string());
            }
            _ => return Err(format!("unknown render option {option:?}")),
        }
        index += 1;
    }
    Ok(RenderCli {
        rom: rom.ok_or_else(|| "render requires --rom".to_string())?,
        output: output.ok_or_else(|| "render requires --output".to_string())?,
        overwrite,
    })
}

fn parse_build_args(args: &[OsString]) -> AppResult<BuildCli> {
    let mut rom = None;
    let mut workspace = None;
    let mut translations = None;
    let mut output = None;
    let mut overwrite = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| "build option is not valid Unicode".to_string())?;
        match option {
            "--rom" | "--input" | "-i" => {
                index += 1;
                rom = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| format!("{option} requires a path"))?,
                ));
            }
            "--workspace" | "-w" => {
                index += 1;
                workspace = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| format!("{option} requires a path"))?,
                ));
            }
            "--translations" | "-t" => {
                index += 1;
                translations = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| format!("{option} requires a path"))?,
                ));
            }
            "--output" | "-o" => {
                index += 1;
                output = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| format!("{option} requires a path"))?,
                ));
            }
            "--overwrite" => overwrite = true,
            "--non-interactive" => {}
            "-h" | "--help" => {
                print_help();
                return Err("build help requested".to_string());
            }
            _ => return Err(format!("unknown build option {option:?}")),
        }
        index += 1;
    }
    Ok(BuildCli {
        rom: rom.ok_or_else(|| "build requires --rom".to_string())?,
        workspace: workspace.ok_or_else(|| "build requires --workspace".to_string())?,
        translations: translations.ok_or_else(|| "build requires --translations".to_string())?,
        output: output.ok_or_else(|| "build requires --output".to_string())?,
        overwrite,
    })
}

fn interactive(mut prefill: InteractivePrefill) -> AppResult<()> {
    loop {
        println!();
        println!("FOXY KANJI1.ROM 独立字库工具");
        println!("1. 渲染原始 ROM 清晰探针");
        println!("2. 重建动态 KANJI1.ROM");
        println!("0. 退出");
        let Some(choice) = prompt("选择")? else {
            return Ok(());
        };
        match choice.trim() {
            "0" => return Ok(()),
            "1" => match interactive_render(&mut prefill) {
                Ok(true) => {}
                Ok(false) => return Ok(()),
                Err(error) => println!("操作失败：{error}"),
            },
            "2" => match interactive_build(&mut prefill) {
                Ok(true) => {}
                Ok(false) => return Ok(()),
                Err(error) => println!("操作失败：{error}"),
            },
            _ => println!("无效选择，请重试。"),
        }
    }
}

fn interactive_render(prefill: &mut InteractivePrefill) -> AppResult<bool> {
    let default_rom = prefill
        .rom
        .clone()
        .unwrap_or_else(|| PathBuf::from("KANJI1.ROM"));
    let Some(rom) = prompt_path("原始 KANJI1.ROM", &default_rom)? else {
        return Ok(false);
    };
    prefill.rom = Some(rom.clone());
    let default_output = rom
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .join("foxy_font_probe");
    let Some(output) = prompt_path("输出目录", &default_output)? else {
        return Ok(false);
    };
    let Some(overwrite) = prompt_yes_no("允许覆盖已识别的字库工具输出", false)?
    else {
        return Ok(false);
    };
    println!();
    println!("ROM     : {}", rom.display());
    println!("输出    : {}", output.display());
    println!("覆盖    : {}", if overwrite { "是" } else { "否" });
    let Some(confirm) = prompt_yes_no("确认渲染", false)? else {
        return Ok(false);
    };
    if !confirm {
        println!("已取消，未写入文件。");
        return Ok(true);
    }
    render_probe(&RenderCli {
        rom,
        output,
        overwrite,
    })?;
    println!("渲染完成。");
    Ok(true)
}

fn interactive_build(prefill: &mut InteractivePrefill) -> AppResult<bool> {
    let Some(rom) = prompt_path(
        "原始 KANJI1.ROM",
        prefill
            .rom
            .as_deref()
            .unwrap_or_else(|| Path::new("KANJI1.ROM")),
    )?
    else {
        return Ok(false);
    };
    prefill.rom = Some(rom.clone());
    let Some(workspace) = prompt_path(
        "D88 解包工作区",
        prefill
            .workspace
            .as_deref()
            .unwrap_or_else(|| Path::new(".")),
    )?
    else {
        return Ok(false);
    };
    prefill.workspace = Some(workspace.clone());
    let Some(translations) = prompt_path(
        "翻译 JSON 或其目录",
        prefill
            .translations
            .as_deref()
            .unwrap_or_else(|| Path::new("messages.json")),
    )?
    else {
        return Ok(false);
    };
    prefill.translations = Some(translations.clone());
    let default_output = workspace
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .join("foxy_font_build");
    let Some(output) = prompt_path("输出目录", &default_output)? else {
        return Ok(false);
    };
    let Some(overwrite) = prompt_yes_no("允许覆盖已识别的字库工具输出", false)?
    else {
        return Ok(false);
    };
    println!();
    println!("ROM       : {}", rom.display());
    println!("工作区    : {}", workspace.display());
    println!("翻译      : {}", translations.display());
    println!("输出      : {}", output.display());
    println!("覆盖      : {}", if overwrite { "是" } else { "否" });
    let Some(confirm) = prompt_yes_no("确认重建动态字库", false)? else {
        return Ok(false);
    };
    if !confirm {
        println!("已取消，未写入文件。");
        return Ok(true);
    }
    build_font(&BuildCli {
        rom,
        workspace,
        translations,
        output,
        overwrite,
    })?;
    println!("动态字库重建完成。");
    Ok(true)
}

fn classify_prefill(path: PathBuf) -> AppResult<InteractivePrefill> {
    let mut prefill = InteractivePrefill::default();
    if path.is_file() {
        let metadata = fs::metadata(&path)
            .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
        if metadata.len() == font::KANJI1_ROM_SIZE as u64 {
            prefill.rom = Some(path);
        } else if translation_document_path(&path).is_ok() {
            prefill.translations = Some(path);
        } else {
            return Err(format!(
                "path is neither a 131072-byte ROM nor FOXY translation JSON: {}",
                path.display()
            ));
        }
    } else if path.is_dir() {
        if path.join("manifest.json").is_file() && path.join("volumes").is_dir() {
            prefill.workspace = Some(path.clone());
        }
        if translation_document_path(&path).is_ok() {
            prefill.translations = Some(path.clone());
        }
        if prefill.workspace.is_none() && prefill.translations.is_none() {
            return Err(format!(
                "directory has neither an unpack manifest nor FOXY messages.json: {}",
                path.display()
            ));
        }
    } else {
        return Err(format!("path does not exist: {}", path.display()));
    }
    Ok(prefill)
}

fn render_probe(cli: &RenderCli) -> AppResult<()> {
    let rom = fs::read(&cli.rom)
        .map_err(|error| format!("failed to read {}: {error}", cli.rom.display()))?;
    font::validate_rom(&rom)?;
    ensure_disjoint(&cli.rom, &cli.output)?;
    let options = font::PreviewOptions::default();
    let pages = font::render_rom_probe_pages(&rom, font::DEFAULT_ROM_PROBE_TEXT, &options)?;
    let manifest = ProbeManifest {
        format: "FOXY KANJI1.ROM raw probe",
        version: 1,
        tool: "foxy_font_tool",
        source_file: leaf_name(&cli.rom),
        source_size: rom.len(),
        source_sha256: sha256_hex(&rom),
        layout: "JIS-derived base; 16 rows x two adjacent bytes; MSB-left",
        scale: options.scale,
        page_count: pages.len(),
        pages: pages
            .iter()
            .map(|page| PageManifest {
                file: format!("previews/{}", page.file_name),
                entries: page.entries,
                width: page.width,
                height: page.height,
            })
            .collect(),
    };
    prepare_output(&cli.output, cli.overwrite)?;
    let result = (|| -> AppResult<()> {
        let previews = cli.output.join("previews");
        fs::create_dir_all(&previews)
            .map_err(|error| format!("failed to create {}: {error}", previews.display()))?;
        for page in &pages {
            fs::write(previews.join(&page.file_name), &page.bmp)
                .map_err(|error| format!("failed to write preview {}: {error}", page.file_name))?;
        }
        write_json(&cli.output.join("manifest.json"), &manifest)
    })();
    finish_output(&cli.output, result)?;
    println!(
        "已渲染 {} 页原始 ROM 探针图：{}",
        pages.len(),
        cli.output.display()
    );
    Ok(())
}

fn build_font(cli: &BuildCli) -> AppResult<()> {
    ensure_disjoint(&cli.rom, &cli.output)?;
    ensure_disjoint(&cli.workspace, &cli.output)?;
    ensure_disjoint(&cli.translations, &cli.output)?;
    let inputs = load_font_inputs(&cli.workspace, &cli.translations)?;
    let resources = font::FontResources::load_embedded()?;
    let translated_refs = inputs
        .translated_texts
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let plan =
        resources.plan_dynamic_mapping(inputs.original_codes, translated_refs.iter().copied())?;
    for text in &translated_refs {
        resources.encode_ai1_text(text, &plan)?;
    }
    let source_rom = fs::read(&cli.rom)
        .map_err(|error| format!("failed to read {}: {error}", cli.rom.display()))?;
    let build = resources.build_rom(&source_rom, &plan)?;
    let pages =
        resources.render_preview_pages(&build.rom, &plan, &font::PreviewOptions::default())?;
    let mapping_json = font::mapping_used_json_bytes(&plan)?;
    let manifest_json = font::manifest_json_bytes(&build.manifest)?;

    prepare_output(&cli.output, cli.overwrite)?;
    let result = (|| -> AppResult<()> {
        let previews = cli.output.join("previews");
        fs::create_dir_all(&previews)
            .map_err(|error| format!("failed to create {}: {error}", previews.display()))?;
        fs::write(cli.output.join("KANJI1.ROM"), &build.rom)
            .map_err(|error| format!("failed to write KANJI1.ROM: {error}"))?;
        let mapping_path = cli.output.join("mapping_used.json");
        fs::write(&mapping_path, &mapping_json)
            .map_err(|error| format!("failed to write mapping_used.json: {error}"))?;
        let loaded = font::load_mapping_used(&mapping_path)?;
        if loaded.mapping_used != plan.mapping_used {
            return Err("mapping_used reload verification changed assignments".to_string());
        }
        fs::write(cli.output.join("manifest.json"), &manifest_json)
            .map_err(|error| format!("failed to write manifest.json: {error}"))?;
        for page in &pages {
            fs::write(previews.join(&page.file_name), &page.bmp)
                .map_err(|error| format!("failed to write preview {}: {error}", page.file_name))?;
        }
        Ok(())
    })();
    finish_output(&cli.output, result)?;
    println!(
        "动态 KANJI1.ROM 已完成：MES文件={}，文本项={}，动态映射={}，预览页={}，输出={}",
        inputs.mes_files,
        inputs.entries,
        plan.mapping_used.len(),
        pages.len(),
        cli.output.display()
    );
    Ok(())
}

fn load_font_inputs(workspace: &Path, translations: &Path) -> AppResult<FontInputs> {
    if !workspace.is_dir() {
        return Err(format!(
            "unpacked workspace is not a directory: {}",
            workspace.display()
        ));
    }
    let document_path = translation_document_path(translations)?;
    let bytes = fs::read(&document_path)
        .map_err(|error| format!("failed to read {}: {error}", document_path.display()))?;
    let document: TranslationDocument = serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse {}: {error}", document_path.display()))?;
    validate_translation_header(&document)?;
    if document.files.len() != EXPECTED_MES_FILES {
        return Err(format!(
            "FOXY font planning requires all {EXPECTED_MES_FILES} MES files, translation JSON contains {}",
            document.files.len()
        ));
    }

    let mut original_codes = BTreeSet::new();
    let mut translated_texts = Vec::new();
    let mut seen_files = BTreeSet::new();
    let mut entry_count = 0usize;
    for file in &document.files {
        let relative = safe_relative_path(&file.file)?;
        if !seen_files.insert(relative.clone()) {
            return Err(format!("translation document repeats _file {}", file.file));
        }
        let source_path = workspace.join(&relative);
        let source = fs::read(&source_path)
            .map_err(|error| format!("failed to read {}: {error}", source_path.display()))?;
        let actual_hash = sha256_hex(&source);
        if actual_hash != file.sha256 {
            return Err(format!(
                "source hash mismatch for {}: JSON={}, workspace={actual_hash}",
                file.file, file.sha256
            ));
        }
        let mut ranges = Vec::with_capacity(file.entries.len());
        for (expected_index, entry) in file.entries.iter().enumerate() {
            validate_translation_entry(entry, &file.file)?;
            if entry.index != expected_index {
                return Err(format!(
                    "entry index changed in {}: expected {expected_index}, got {}",
                    file.file, entry.index
                ));
            }
            let end = entry.offset.checked_add(entry.size).ok_or_else(|| {
                format!(
                    "entry range overflow in {} index {}",
                    file.file, entry.index
                )
            })?;
            if end > source.len() {
                return Err(format!(
                    "entry {} range {:#x}..{end:#x} exceeds {} bytes in {}",
                    entry.index,
                    entry.offset,
                    source.len(),
                    file.file
                ));
            }
            if entry.size % 2 != 0 {
                return Err(format!(
                    "entry {} in {} has odd AI1 text size {}",
                    entry.index, file.file, entry.size
                ));
            }
            validate_entry_source(entry, &source[entry.offset..end], &file.file)?;
            ranges.push((entry.offset, end, entry.index));
            for pair in source[entry.offset..end].chunks_exact(2) {
                let (_, had_errors) = SHIFT_JIS.decode_without_bom_handling(pair);
                if !had_errors {
                    original_codes.insert(u16::from_be_bytes([pair[0], pair[1]]));
                }
            }
            collect_entry_messages(entry, &mut translated_texts);
            entry_count += 1;
        }
        ranges.sort_unstable_by_key(|range| range.0);
        for pair in ranges.windows(2) {
            if pair[0].1 > pair[1].0 {
                return Err(format!(
                    "translation entry ranges overlap in {}: indices {} and {}",
                    file.file, pair[0].2, pair[1].2
                ));
            }
        }
    }
    Ok(FontInputs {
        original_codes: original_codes.into_iter().collect(),
        translated_texts,
        mes_files: document.files.len(),
        entries: entry_count,
    })
}

fn translation_document_path(path: &Path) -> AppResult<PathBuf> {
    let candidate = if path.is_dir() {
        let nested = path.join("text").join("messages.json");
        if nested.is_file() {
            nested
        } else {
            path.join("messages.json")
        }
    } else {
        path.to_path_buf()
    };
    if !candidate.is_file() {
        return Err(format!(
            "FOXY messages.json was not found at {}",
            candidate.display()
        ));
    }
    let bytes = fs::read(&candidate)
        .map_err(|error| format!("failed to read {}: {error}", candidate.display()))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse {}: {error}", candidate.display()))?;
    if value.get("format").and_then(|value| value.as_str()) != Some("FOXY AI1 translation JSON")
        || value.get("version").and_then(|value| value.as_u64()) != Some(1)
    {
        return Err(format!(
            "unsupported translation JSON: {}",
            candidate.display()
        ));
    }
    Ok(candidate)
}

fn validate_translation_header(document: &TranslationDocument) -> AppResult<()> {
    if document.format != "FOXY AI1 translation JSON" || document.version != 1 {
        return Err(format!(
            "unsupported translation document {} version {}",
            document.format, document.version
        ));
    }
    let supported_encoding = matches!(
        document.profile.encoding.as_str(),
        "CP932" | "CP932 plus FOXY carrier mapping"
    );
    if document.profile.engine != "AI1" || !supported_encoding {
        return Err(format!(
            "translation profile must be AI1/CP932, got {}/{}",
            document.profile.engine, document.profile.encoding
        ));
    }
    if document.profile.speaker_policy != "no name field; dialogue and narration use message" {
        return Err("translation profile changed the confirmed no-name speaker policy".to_string());
    }
    Ok(())
}

fn validate_translation_entry(entry: &TranslationEntry, file: &str) -> AppResult<()> {
    if entry.kind != "message" && entry.kind != "choice" {
        return Err(format!(
            "entry {} in {file} has unsupported _type {:?}",
            entry.index, entry.kind
        ));
    }
    match (&entry.scr_msg, &entry.message, &entry.message_parts) {
        (Some(_), Some(_), None) => Ok(()),
        (None, None, Some(parts)) if !parts.is_empty() => {
            for part in parts {
                match part {
                    MessagePart::Text { scr_msg, .. } if !scr_msg.is_empty() => {}
                    MessagePart::Raw { hex } if valid_hex(hex) => {}
                    MessagePart::Text { .. } => {
                        return Err(format!(
                            "entry {} in {file} has an empty message_parts source text",
                            entry.index
                        ));
                    }
                    MessagePart::Raw { .. } => {
                        return Err(format!(
                            "entry {} in {file} has malformed message_parts raw hex",
                            entry.index
                        ));
                    }
                }
            }
            Ok(())
        }
        _ => Err(format!(
            "entry {} in {file} must use scr_msg/message or message_parts, never a name field",
            entry.index
        )),
    }
}

fn collect_entry_messages(entry: &TranslationEntry, output: &mut Vec<String>) {
    if let Some(message) = &entry.message {
        output.push(message.clone());
    }
    if let Some(parts) = &entry.message_parts {
        for part in parts {
            if let MessagePart::Text { message, .. } = part {
                output.push(message.clone());
            }
        }
    }
}

fn validate_entry_source(entry: &TranslationEntry, raw: &[u8], file: &str) -> AppResult<()> {
    let actual = decode_source_parts(raw);
    match (&entry.scr_msg, &entry.message_parts) {
        (Some(scr_msg), None) => {
            if actual.iter().any(|part| matches!(part, SourcePart::Raw(_))) {
                return Err(format!(
                    "entry {} in {file} JSON lost an opaque source pair",
                    entry.index
                ));
            }
            let joined = actual
                .iter()
                .filter_map(|part| match part {
                    SourcePart::Text(text) => Some(text.as_str()),
                    SourcePart::Raw(_) => None,
                })
                .collect::<String>();
            if &joined != scr_msg {
                return Err(format!(
                    "immutable scr_msg does not match workspace bytes in {file} index {}",
                    entry.index
                ));
            }
        }
        (None, Some(parts)) => {
            if parts.len() != actual.len() {
                return Err(format!(
                    "message_parts structure does not match workspace bytes in {file} index {}",
                    entry.index
                ));
            }
            for (wanted, source) in parts.iter().zip(&actual) {
                let matches = match (wanted, source) {
                    (MessagePart::Text { scr_msg, .. }, SourcePart::Text(text)) => scr_msg == text,
                    (MessagePart::Raw { hex }, SourcePart::Raw(actual_hex)) => {
                        hex.eq_ignore_ascii_case(actual_hex)
                    }
                    _ => false,
                };
                if !matches {
                    return Err(format!(
                        "immutable message_parts source does not match workspace bytes in {file} index {}",
                        entry.index
                    ));
                }
            }
        }
        _ => {
            return Err(format!(
                "entry {} in {file} has inconsistent source fields",
                entry.index
            ));
        }
    }
    Ok(())
}

fn decode_source_parts(raw: &[u8]) -> Vec<SourcePart> {
    let mut parts = Vec::new();
    let mut text = String::new();
    for pair in raw.chunks_exact(2) {
        if pair == EXPLICIT_NEWLINE {
            text.push('\n');
            continue;
        }
        let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(pair);
        if had_errors {
            if !text.is_empty() {
                parts.push(SourcePart::Text(std::mem::take(&mut text)));
            }
            parts.push(SourcePart::Raw(format!("{:02X}{:02X}", pair[0], pair[1])));
        } else {
            text.push_str(&decoded);
        }
    }
    if !text.is_empty() {
        parts.push(SourcePart::Text(text));
    }
    parts
}

fn valid_hex(value: &str) -> bool {
    value.len().is_multiple_of(2) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn safe_relative_path(value: &str) -> AppResult<PathBuf> {
    let path = Path::new(value);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!("unsafe translation _file path: {value:?}"));
    }
    Ok(path.to_path_buf())
}

fn prepare_output(output: &Path, overwrite: bool) -> AppResult<()> {
    if output.as_os_str().is_empty() || output.file_name().is_none() {
        return Err(format!(
            "refusing unsafe output directory: {}",
            output.display()
        ));
    }
    if output.exists() {
        if !output.is_dir() {
            return Err(format!(
                "output exists and is not a directory: {}",
                output.display()
            ));
        }
        let nonempty = fs::read_dir(output)
            .map_err(|error| format!("failed to inspect {}: {error}", output.display()))?
            .next()
            .transpose()
            .map_err(|error| format!("failed to inspect {}: {error}", output.display()))?
            .is_some();
        if nonempty {
            if !overwrite {
                return Err(format!(
                    "output directory is not empty: {} (use --overwrite only when intended)",
                    output.display()
                ));
            }
            if !recognized_output(output)? {
                return Err(format!(
                    "refusing to replace unrecognized output directory: {}",
                    output.display()
                ));
            }
            fs::remove_dir_all(output)
                .map_err(|error| format!("failed to replace {}: {error}", output.display()))?;
        }
    }
    fs::create_dir_all(output)
        .map_err(|error| format!("failed to create {}: {error}", output.display()))?;
    fs::write(output.join(OUTPUT_MARKER), b"foxy_font_tool in progress\n")
        .map_err(|error| format!("failed to mark {}: {error}", output.display()))
}

fn recognized_output(output: &Path) -> AppResult<bool> {
    if output.join(OUTPUT_MARKER).is_file() {
        return Ok(true);
    }
    let manifest_path = output.join("manifest.json");
    if !manifest_path.is_file() {
        return Ok(false);
    }
    let bytes = fs::read(&manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?;
    let format = value
        .get("format")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    Ok(format == "FOXY KANJI1.ROM raw probe"
        || format == "FOXY PC-8801 KANJI1.ROM dynamic font patch")
}

fn finish_output(output: &Path, result: AppResult<()>) -> AppResult<()> {
    result?;
    fs::remove_file(output.join(OUTPUT_MARKER))
        .map_err(|error| format!("failed to clear output marker: {error}"))
}

fn ensure_disjoint(input: &Path, output: &Path) -> AppResult<()> {
    let resolved_input = fs::canonicalize(input)
        .map_err(|error| format!("failed to resolve {}: {error}", input.display()))?;
    let resolved_output = resolve_candidate(output)?;
    if resolved_input.starts_with(&resolved_output)
        || (resolved_input.is_dir() && resolved_output.starts_with(&resolved_input))
    {
        return Err(format!(
            "output {} overlaps input {}",
            output.display(),
            input.display()
        ));
    }
    Ok(())
}

fn resolve_candidate(path: &Path) -> AppResult<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path)
            .map_err(|error| format!("failed to resolve {}: {error}", path.display()));
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let resolved_parent = fs::canonicalize(parent)
        .map_err(|error| format!("failed to resolve {}: {error}", parent.display()))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| format!("output has no directory name: {}", path.display()))?;
    Ok(resolved_parent.join(file_name))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> AppResult<()> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn prompt(label: &str) -> AppResult<Option<String>> {
    print!("{label}: ");
    io::stdout()
        .flush()
        .map_err(|error| format!("failed to flush prompt: {error}"))?;
    let mut value = String::new();
    let count = io::stdin()
        .read_line(&mut value)
        .map_err(|error| format!("failed to read prompt: {error}"))?;
    if count == 0 {
        return Ok(None);
    }
    Ok(Some(value.trim_end_matches(['\r', '\n']).to_string()))
}

fn prompt_path(label: &str, default: &Path) -> AppResult<Option<PathBuf>> {
    let Some(value) = prompt(&format!("{label} [{}]", default.display()))? else {
        return Ok(None);
    };
    Ok(Some(if value.trim().is_empty() {
        default.to_path_buf()
    } else {
        PathBuf::from(strip_drag_quotes(value.trim()))
    }))
}

fn prompt_yes_no(label: &str, default: bool) -> AppResult<Option<bool>> {
    loop {
        let suffix = if default { "Y/n" } else { "y/N" };
        let Some(value) = prompt(&format!("{label} [{suffix}]"))? else {
            return Ok(None);
        };
        match value.trim().to_ascii_lowercase().as_str() {
            "" => return Ok(Some(default)),
            "y" | "yes" => return Ok(Some(true)),
            "n" | "no" => return Ok(Some(false)),
            _ => println!("请输入 y 或 n。"),
        }
    }
}

fn strip_drag_quotes(value: &str) -> &str {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn leaf_name(path: &Path) -> String {
    path.file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "KANJI1.ROM".to_string())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_command_requires_explicit_paths() {
        let args = [
            OsString::from("--rom"),
            OsString::from("input.rom"),
            OsString::from("--output"),
            OsString::from("probe"),
        ];
        let cli = parse_render_args(&args).unwrap();
        assert_eq!(cli.rom, PathBuf::from("input.rom"));
        assert_eq!(cli.output, PathBuf::from("probe"));
        assert!(!cli.overwrite);
        assert!(parse_render_args(&[]).is_err());
    }

    #[test]
    fn unsafe_translation_paths_are_rejected() {
        assert!(safe_relative_path("volumes/00/files_decoded/A.MES").is_ok());
        assert!(safe_relative_path("../A.MES").is_err());
        assert!(safe_relative_path("C:\\A.MES").is_err());
    }
}
