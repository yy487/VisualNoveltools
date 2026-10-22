use anyhow::{bail, Context, Result};
use kuki_disk_msg::d88::{pack_disk, unpack_disk, D88ProjectImage};
use kuki_disk_msg::font::{rebuild_kanji1, scan_translation_root, FontMap};
use kuki_disk_msg::msg::MsgFile;
use kuki_disk_msg::msg_json::{extract_msg, inject_msg, inject_msg_with_font_map};
use std::collections::{HashMap, HashSet};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        return interactive(None);
    }
    if is_help(&args[0]) {
        print_help();
        return Ok(());
    }
    if args.len() == 1 && !args[0].to_string_lossy().starts_with('-') {
        return interactive(Some(PathBuf::from(&args[0])));
    }
    if args.len() < 2 {
        bail!("missing operation; run with --help for usage");
    }
    let group = text_arg(&args[0], "command group")?;
    let operation = text_arg(&args[1], "operation")?;
    let options = Options::parse(&args[2..])?;
    match (group.as_str(), operation.as_str()) {
        ("disk", "unpack") => disk_unpack(options),
        ("disk", "pack") => disk_pack(options),
        ("msg", "inspect") => msg_inspect(options),
        ("msg", "extract") => msg_extract(options),
        ("msg", "inject") => msg_inject(options),
        ("font", "scan") => font_scan(options),
        ("font", "rebuild") => font_rebuild(options),
        _ => bail!("unknown operation {group} {operation}; run with --help for usage"),
    }
}

fn disk_unpack(mut options: Options) -> Result<()> {
    let input = options.required_path("--input")?;
    let output = options.required_path("--output")?;
    let overwrite = options.flag("--overwrite");
    options.finish()?;
    run_disk_unpack(&input, &output, overwrite)
}

fn disk_pack(mut options: Options) -> Result<()> {
    let workspace = options.required_path("--workspace")?;
    let output = options.required_path("--output")?;
    let overwrite = options.flag("--overwrite");
    options.finish()?;
    run_disk_pack(&workspace, &output, overwrite)
}

fn msg_inspect(mut options: Options) -> Result<()> {
    let input = options.required_path("--input")?;
    let samples = options
        .optional_string("--samples")?
        .map(|value| {
            value
                .parse::<usize>()
                .with_context(|| format!("invalid --samples value {value:?}"))
        })
        .transpose()?
        .unwrap_or(8);
    options.finish()?;
    inspect_msg_path(&input, samples)
}

fn msg_extract(mut options: Options) -> Result<()> {
    let input = options.required_path("--input")?;
    let output = options.required_path("--output")?;
    let overwrite = options.flag("--overwrite");
    options.finish()?;
    run_msg_extract(&input, &output, overwrite)
}

fn msg_inject(mut options: Options) -> Result<()> {
    let source = options.required_path("--source")?;
    let translation = options.required_path("--translation")?;
    let output = options.required_path("--output")?;
    let font_map_path = options.optional_path("--font-map")?;
    let overwrite = options.flag("--overwrite");
    options.finish()?;
    let font_map = font_map_path.as_deref().map(FontMap::load).transpose()?;
    run_msg_inject(&source, &translation, &output, font_map.as_ref(), overwrite)
}

fn font_scan(mut options: Options) -> Result<()> {
    let translations = options.required_path("--translations")?;
    let rom = options.required_path("--rom")?;
    let font = options.required_path("--font")?;
    let output = options.required_path("--output")?;
    let existing = options.optional_path("--existing")?;
    let overwrite = options.flag("--overwrite");
    options.finish()?;
    run_font_scan(
        &translations,
        &rom,
        &font,
        existing.as_deref(),
        &output,
        overwrite,
    )
}

fn font_rebuild(mut options: Options) -> Result<()> {
    let rom = options.required_path("--rom")?;
    let map = options.required_path("--map")?;
    let font = options.required_path("--font")?;
    let output = options.required_path("--output")?;
    let atlas = options.optional_path("--atlas")?;
    let overwrite = options.flag("--overwrite");
    options.finish()?;
    run_font_rebuild(&rom, &map, &font, &output, atlas.as_deref(), overwrite)
}

fn run_disk_unpack(input: &Path, output: &Path, overwrite: bool) -> Result<()> {
    let report = unpack_disk(input, output, overwrite)?;
    println!("[unpack] extracted_files={}", report.extracted_files);
    println!("[unpack] extracted_bytes={}", report.output_bytes);
    println!("[unpack] output={}", report.output.display());
    Ok(())
}

fn run_disk_pack(workspace: &Path, output: &Path, overwrite: bool) -> Result<()> {
    let report = pack_disk(workspace, output, overwrite)?;
    println!("[pack] packed_files={}", report.packed_files);
    println!("[pack] changed_files={}", report.changed_files);
    println!("[pack] used_blocks={}", report.used_blocks);
    println!("[pack] output_bytes={}", report.output_bytes);
    println!("[pack] byte_exact_template={}", report.byte_exact_template);
    println!("[pack] output={}", report.output.display());
    Ok(())
}

fn inspect_msg_path(input: &Path, samples: usize) -> Result<()> {
    let bytes =
        fs::read(input).with_context(|| format!("failed to read MSG file {}", input.display()))?;
    let msg = MsgFile::parse(&bytes)
        .with_context(|| format!("failed to parse MSG file {}", input.display()))?;
    let inspection = msg.inspect()?;
    println!("[inspect] file={}", input.display());
    println!("[inspect] file_bytes={}", bytes.len());
    println!("[inspect] blocks={}", inspection.block_count);
    println!("[inspect] pointer_entries={}", inspection.pointer_count);
    println!("[inspect] text_fragments={}", inspection.text_fragments);
    let controls = inspection
        .control_counts
        .iter()
        .map(|(value, count)| format!("{value:02X}:{count}"))
        .collect::<Vec<_>>()
        .join(",");
    println!("[inspect] controls={controls}");

    let mut shown = 0usize;
    for (logical_index, physical_index) in msg.logical_block_order().into_iter().enumerate() {
        if shown == samples {
            break;
        }
        let rendered = msg.render_block_for_inspection(physical_index)?;
        if rendered.trim().is_empty() {
            continue;
        }
        let block = &msg.blocks[physical_index];
        println!(
            "[sample] logical={} physical={} offset=0x{:04X} bytes={} text={}",
            logical_index,
            physical_index,
            block.original_offset,
            block.bytes.len(),
            rendered.replace('\r', "[CR]").replace('\n', "[LF]")
        );
        shown += 1;
    }
    Ok(())
}

fn run_msg_extract(input: &Path, output: &Path, overwrite: bool) -> Result<()> {
    let report = extract_msg(input, output, overwrite)?;
    println!("[extract] scanned_files={}", report.scanned_files);
    println!("[extract] json_files={}", report.json_files);
    println!("[extract] extracted_entries={}", report.extracted_entries);
    println!("[extract] skipped_files={}", report.skipped_files);
    println!("[extract] warnings={}", report.warnings);
    println!("[extract] output={}", report.output.display());
    Ok(())
}

fn run_msg_inject(
    source: &Path,
    translation: &Path,
    output: &Path,
    font_map: Option<&FontMap>,
    overwrite: bool,
) -> Result<()> {
    let report = if let Some(font_map) = font_map {
        inject_msg_with_font_map(source, translation, output, Some(font_map), overwrite)?
    } else {
        inject_msg(source, translation, output, overwrite)?
    };
    println!("[inject] json_files={}", report.json_files);
    println!("[inject] json_entries={}", report.json_entries);
    println!("[inject] patched={}", report.patched);
    println!("[inject] unchanged={}", report.unchanged);
    println!("[inject] output_files={}", report.output_files);
    println!("[inject] warnings={}", report.warnings);
    println!("[inject] output={}", report.output.display());
    Ok(())
}

fn run_font_scan(
    translations: &Path,
    rom: &Path,
    font: &Path,
    existing: Option<&Path>,
    output: &Path,
    overwrite: bool,
) -> Result<()> {
    let report = scan_translation_root(translations, rom, font, existing, output, overwrite)?;
    println!("[font scan] json_files={}", report.json_files);
    println!("[font scan] entries_scanned={}", report.entries_scanned);
    println!(
        "[font scan] required_characters={}",
        report.required_characters
    );
    println!("[font scan] new_mappings={}", report.new_mappings);
    println!("[font scan] repaired_mappings={}", report.repaired_mappings);
    println!("[font scan] total_mappings={}", report.total_mappings);
    println!("[font scan] output={}", report.output.display());
    Ok(())
}

fn run_font_rebuild(
    rom: &Path,
    map: &Path,
    font: &Path,
    output: &Path,
    atlas: Option<&Path>,
    overwrite: bool,
) -> Result<()> {
    let report = rebuild_kanji1(rom, map, font, output, atlas, overwrite)?;
    println!("[font rebuild] mappings={}", report.mappings);
    println!("[font rebuild] changed_slots={}", report.changed_slots);
    println!("[font rebuild] changed_bytes={}", report.changed_bytes);
    println!("[font rebuild] output={}", report.output.display());
    if let Some(atlas) = report.atlas {
        println!("[font rebuild] atlas={}", atlas.display());
    }
    Ok(())
}

fn print_help() {
    println!(
        "kuki-disk-msg - PC-88 D88 / MSG.ZIL tool\n\
\n\
Usage:\n\
  kuki-disk-msg\n\
  kuki-disk-msg <PATH>\n\
  kuki-disk-msg disk unpack --input <IMAGE.D88> --output <WORKSPACE> [--overwrite]\n\
  kuki-disk-msg disk pack --workspace <WORKSPACE|disk_manifest.json> --output <IMAGE.D88> [--overwrite]\n\
  kuki-disk-msg msg inspect --input <MSG.ZIL> [--samples <COUNT>]\n\
  kuki-disk-msg msg extract --input <MSG.ZIL|DIR> --output <JSON|DIR> [--overwrite]\n\
  kuki-disk-msg msg inject --source <MSG.ZIL|DIR> --translation <JSON|DIR> --output <MSG.ZIL|DIR> [--font-map <MAP.JSON>] [--overwrite]\n\
  kuki-disk-msg font scan --translations <JSON|DIR> --rom <kanji1.rom> --font <simsun.ttf> --output <MAP.JSON> [--existing <MAP.JSON>] [--overwrite]\n\
  kuki-disk-msg font rebuild --rom <kanji1.rom> --map <MAP.JSON> --font <simsun.ttf> --output <kanji1_new.rom> [--atlas <atlas.pgm>] [--overwrite]\n\
\n\
No arguments opens the interactive menu. A single path opens the same menu with\n\
content-based prefill; no write occurs before confirmation. Explicit commands are\n\
non-interactive. Existing outputs are refused unless --overwrite is supplied.\n\
The original D88 and workspace template are never valid pack destinations. MSG\n\
JSON uses CP932, immutable scr_msg/_scr_name validation, and editable [HH]\n\
control tokens; [0E] delimiters are rebuilt automatically."
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrefillKind {
    DiskImage,
    DiskWorkspace,
    MsgSource,
    MsgTranslation,
    Unknown,
}

fn interactive(prefill: Option<PathBuf>) -> Result<()> {
    let kind = prefill
        .as_deref()
        .map(detect_prefill_kind)
        .transpose()?
        .unwrap_or(PrefillKind::Unknown);
    println!("kuki-disk-msg interactive");
    if let Some(path) = &prefill {
        println!("prefill={}", path.display());
        println!("detected={}", prefill_label(kind));
    }
    println!("1. DISK 解包");
    println!("2. DISK 封包");
    println!("3. MSG 查看");
    println!("4. MSG 提取 JSON");
    println!("5. MSG 注入");
    println!("6. KANJI1 映射扫描");
    println!("7. KANJI1 ROM 重建");
    println!("0. 退出");
    let suggested = match kind {
        PrefillKind::DiskImage => Some("1"),
        PrefillKind::DiskWorkspace => Some("2"),
        PrefillKind::MsgSource => Some("4"),
        PrefillKind::MsgTranslation => Some("5"),
        PrefillKind::Unknown => None,
    };
    let Some(choice) = prompt_text("请选择操作编号", suggested)? else {
        return Ok(());
    };
    match choice.as_str() {
        "1" => interactive_disk_unpack(prefill.filter(|_| kind == PrefillKind::DiskImage)),
        "2" => interactive_disk_pack(prefill.filter(|_| kind == PrefillKind::DiskWorkspace)),
        "3" => interactive_msg_inspect(prefill.filter(|_| kind == PrefillKind::MsgSource)),
        "4" => interactive_msg_extract(prefill.filter(|_| kind == PrefillKind::MsgSource)),
        "5" => interactive_msg_inject(prefill, kind),
        "6" => interactive_font_scan(),
        "7" => interactive_font_rebuild(),
        _ => bail!("unknown interactive operation {choice:?}"),
    }
}

fn interactive_disk_unpack(prefill: Option<PathBuf>) -> Result<()> {
    let Some(input) = prompt_d88_file("D88 文件路径（可直接拖入文件）", prefill.as_deref())?
    else {
        return Ok(());
    };
    let default = default_directory_output(&input, "_unpacked")?;
    let Some((output, overwrite)) = prompt_output_path("解包工作区输出路径", &default)?
    else {
        return Ok(());
    };
    print_preflight("disk unpack", &[("input", &input)], &output, overwrite);
    if confirm_write()? {
        run_disk_unpack(&input, &output, overwrite)?;
    }
    Ok(())
}

fn interactive_disk_pack(prefill: Option<PathBuf>) -> Result<()> {
    let Some(workspace) = prompt_disk_workspace(
        "DISK 工作区目录或 disk_manifest.json 路径",
        prefill.as_deref(),
    )?
    else {
        return Ok(());
    };
    let root = if workspace.is_file() {
        workspace.parent().unwrap_or_else(|| Path::new("."))
    } else {
        &workspace
    };
    let default = default_file_output(root, "_packed", "d88")?;
    let Some((output, overwrite)) = prompt_output_path("D88 输出路径", &default)? else {
        return Ok(());
    };
    print_preflight(
        "disk pack",
        &[("workspace", &workspace)],
        &output,
        overwrite,
    );
    if confirm_write()? {
        run_disk_pack(&workspace, &output, overwrite)?;
    }
    Ok(())
}

fn interactive_msg_inspect(prefill: Option<PathBuf>) -> Result<()> {
    let Some(input) = prompt_msg_file("MSG 文件路径", prefill.as_deref())? else {
        return Ok(());
    };
    let samples = prompt_text("显示样例数量", Some("8"))?
        .unwrap_or_else(|| "0".to_owned())
        .parse::<usize>()
        .context("sample count must be a non-negative integer")?;
    inspect_msg_path(&input, samples)
}

fn interactive_msg_extract(prefill: Option<PathBuf>) -> Result<()> {
    let Some(input) = prompt_existing_path("MSG 文件或目录路径", prefill.as_deref())? else {
        return Ok(());
    };
    let default = if input.is_file() {
        append_suffix_to_filename(&input, ".json")?
    } else {
        default_directory_output(&input, "_json")?
    };
    let Some((output, overwrite)) = prompt_output_path("JSON 文件或目录输出路径", &default)?
    else {
        return Ok(());
    };
    print_preflight("msg extract", &[("input", &input)], &output, overwrite);
    if confirm_write()? {
        run_msg_extract(&input, &output, overwrite)?;
    }
    Ok(())
}

fn interactive_msg_inject(prefill: Option<PathBuf>, kind: PrefillKind) -> Result<()> {
    let source_prefill = prefill
        .as_deref()
        .filter(|_| kind == PrefillKind::MsgSource);
    let translation_prefill = prefill
        .as_deref()
        .filter(|_| kind == PrefillKind::MsgTranslation);
    let Some(source) = prompt_existing_path("源 MSG 文件或目录路径", source_prefill)?
    else {
        return Ok(());
    };
    let Some(translation) = prompt_existing_path("翻译 JSON 文件或目录路径", translation_prefill)?
    else {
        return Ok(());
    };
    let font_map_path = prompt_existing_path("字体映射 JSON（可留空）", None)?;
    let font_map = font_map_path.as_deref().map(FontMap::load).transpose()?;
    let default = if source.is_file() {
        injected_file_output(&source)?
    } else {
        default_directory_output(&source, "_injected")?
    };
    let Some((output, overwrite)) = prompt_output_path("注入后文件或目录输出路径", &default)?
    else {
        return Ok(());
    };
    print_preflight(
        "msg inject",
        &[("source", &source), ("translation", &translation)],
        &output,
        overwrite,
    );
    if confirm_write()? {
        run_msg_inject(&source, &translation, &output, font_map.as_ref(), overwrite)?;
    }
    Ok(())
}

fn interactive_font_scan() -> Result<()> {
    let Some(translations) = prompt_existing_path("翻译 JSON 文件或目录路径", None)?
    else {
        return Ok(());
    };
    let Some(rom) = prompt_existing_path("kanji1.rom 路径", None)? else {
        return Ok(());
    };
    let Some(font) = prompt_existing_path("simsun_cnjp.ttf 路径", None)? else {
        return Ok(());
    };
    let default = default_file_output(&translations, "_kanji1_map", "json")?;
    let Some((output, overwrite)) = prompt_output_path("映射 JSON 输出路径", &default)?
    else {
        return Ok(());
    };
    let existing = if output.is_file() {
        Some(output.clone())
    } else {
        None
    };
    print_preflight(
        "font scan",
        &[
            ("translations", &translations),
            ("rom", &rom),
            ("font", &font),
        ],
        &output,
        overwrite,
    );
    if confirm_write()? {
        run_font_scan(
            &translations,
            &rom,
            &font,
            existing.as_deref(),
            &output,
            overwrite,
        )?;
    }
    Ok(())
}

fn interactive_font_rebuild() -> Result<()> {
    let Some(rom) = prompt_existing_path("kanji1.rom 路径", None)? else {
        return Ok(());
    };
    let Some(map) = prompt_existing_path("字体映射 JSON 路径", None)? else {
        return Ok(());
    };
    let Some(font) = prompt_existing_path("simsun_cnjp.ttf 路径", None)? else {
        return Ok(());
    };
    let default = default_file_output(&rom, "_rebuilt", "rom")?;
    let Some((output, overwrite)) = prompt_output_path("kanji1.rom 输出路径", &default)? else {
        return Ok(());
    };
    let atlas = output.with_extension("pgm");
    print_preflight(
        "font rebuild",
        &[("rom", &rom), ("map", &map), ("font", &font)],
        &output,
        overwrite,
    );
    if confirm_write()? {
        run_font_rebuild(&rom, &map, &font, &output, Some(&atlas), overwrite)?;
    }
    Ok(())
}

fn print_preflight(operation: &str, inputs: &[(&str, &Path)], output: &Path, overwrite: bool) {
    println!("operation={operation}");
    for (role, path) in inputs {
        println!("{role}={}", path.display());
    }
    println!("output={}", output.display());
    println!("overwrite={overwrite}");
}

fn detect_prefill_kind(path: &Path) -> Result<PrefillKind> {
    if path.is_file() {
        return detect_file_kind(path);
    }
    if path.is_dir() {
        return detect_directory_kind(path);
    }
    Ok(PrefillKind::Unknown)
}

fn detect_file_kind(path: &Path) -> Result<PrefillKind> {
    if let Some(format) = read_json_format(path)? {
        return Ok(match format.as_str() {
            "kuki-ayaka-d88-workspace-v1" => PrefillKind::DiskWorkspace,
            "kuki-ayaka-msg-json-v1" | "kuki-ayaka-msg-workspace-v1" => PrefillKind::MsgTranslation,
            _ => PrefillKind::Unknown,
        });
    }
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(PrefillKind::Unknown),
    };
    if D88ProjectImage::parse(bytes.clone()).is_ok() {
        return Ok(PrefillKind::DiskImage);
    }
    if is_valid_msg(&bytes) {
        return Ok(PrefillKind::MsgSource);
    }
    Ok(PrefillKind::Unknown)
}

fn detect_directory_kind(root: &Path) -> Result<PrefillKind> {
    let mut stack = vec![root.to_path_buf()];
    let mut found_msg = false;
    while let Some(directory) = stack.pop() {
        for entry in match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(_) => continue,
        } {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => continue,
            };
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                stack.push(entry.path());
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            if let Some(format) = read_json_format(&entry.path())? {
                match format.as_str() {
                    "kuki-ayaka-d88-workspace-v1" => return Ok(PrefillKind::DiskWorkspace),
                    "kuki-ayaka-msg-json-v1" | "kuki-ayaka-msg-workspace-v1" => {
                        return Ok(PrefillKind::MsgTranslation)
                    }
                    _ => {}
                }
            }
            if !found_msg {
                if let Ok(bytes) = fs::read(entry.path()) {
                    found_msg = is_valid_msg(&bytes);
                }
            }
        }
    }
    Ok(if found_msg {
        PrefillKind::MsgSource
    } else {
        PrefillKind::Unknown
    })
}

fn is_valid_msg(bytes: &[u8]) -> bool {
    MsgFile::parse(bytes)
        .and_then(|msg| {
            for index in 0..msg.blocks.len() {
                msg.display_segments(index)?;
            }
            Ok(())
        })
        .is_ok()
}

fn read_json_format(path: &Path) -> Result<Option<String>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(None),
    };
    let value: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    Ok(value
        .get("format")
        .or_else(|| value.get("_format"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned))
}

fn prefill_label(kind: PrefillKind) -> &'static str {
    match kind {
        PrefillKind::DiskImage => "D88 image",
        PrefillKind::DiskWorkspace => "D88 workspace/manifest",
        PrefillKind::MsgSource => "MSG source",
        PrefillKind::MsgTranslation => "MSG translation JSON/workspace",
        PrefillKind::Unknown => "unknown; choose roles manually",
    }
}

fn prompt_existing_path(label: &str, default: Option<&Path>) -> Result<Option<PathBuf>> {
    loop {
        let default_text = default.map(|path| path.to_string_lossy().into_owned());
        let Some(value) = prompt_text(label, default_text.as_deref())? else {
            return Ok(None);
        };
        let path = PathBuf::from(strip_drag_quotes(&value));
        if path.exists() {
            return Ok(Some(path));
        }
        println!("路径不存在，请重新输入：{}", path.display());
    }
}

fn prompt_d88_file(label: &str, default: Option<&Path>) -> Result<Option<PathBuf>> {
    loop {
        let Some(path) = prompt_existing_path(label, default)? else {
            return Ok(None);
        };
        if !path.is_file() {
            println!(
                "这里需要 D88 文件，不是目录：{}\n请拖入 .d88 文件，或输入完整文件路径。",
                path.display()
            );
            continue;
        }
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                println!("无法读取 D88 文件 {}：{}", path.display(), error);
                continue;
            }
        };
        match D88ProjectImage::parse(bytes) {
            Ok(_) => return Ok(Some(path)),
            Err(error) => {
                println!("不是本工具支持的 D88 镜像 {}：{error:#}", path.display());
            }
        }
    }
}

fn prompt_disk_workspace(label: &str, default: Option<&Path>) -> Result<Option<PathBuf>> {
    loop {
        let Some(path) = prompt_existing_path(label, default)? else {
            return Ok(None);
        };
        if detect_prefill_kind(&path)? == PrefillKind::DiskWorkspace {
            return Ok(Some(path));
        }
        println!(
            "未识别为 DISK 工作区：{}\n请输入包含有效 manifest 的工作区目录或 manifest 文件。",
            path.display()
        );
    }
}

fn prompt_msg_file(label: &str, default: Option<&Path>) -> Result<Option<PathBuf>> {
    loop {
        let Some(path) = prompt_existing_path(label, default)? else {
            return Ok(None);
        };
        if !path.is_file() {
            println!("这里需要单个 MSG 文件，不是目录：{}", path.display());
            continue;
        }
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                println!("无法读取 MSG 文件 {}：{}", path.display(), error);
                continue;
            }
        };
        if is_valid_msg(&bytes) {
            return Ok(Some(path));
        }
        println!("不是本工具支持的 MSG 文件：{}", path.display());
    }
}

fn prompt_output_path(label: &str, default: &Path) -> Result<Option<(PathBuf, bool)>> {
    let mut suggestion = default.to_path_buf();
    loop {
        let default_text = suggestion.to_string_lossy().into_owned();
        let Some(value) = prompt_text(label, Some(&default_text))? else {
            return Ok(None);
        };
        let path = PathBuf::from(strip_drag_quotes(&value));
        if !path.exists() {
            return Ok(Some((path, false)));
        }
        println!("输出已存在：{}", path.display());
        let Some(choice) = prompt_text("选择：m=修改路径，o=覆盖，0=取消", None)?
        else {
            return Ok(None);
        };
        match choice.to_ascii_lowercase().as_str() {
            "o" | "overwrite" => return Ok(Some((path, true))),
            "m" | "modify" => suggestion = path,
            _ => println!("请输入 m、o 或 0"),
        }
    }
}

fn prompt_text(label: &str, default: Option<&str>) -> Result<Option<String>> {
    match default {
        Some(value) => print!("{label} [{value}]: "),
        None => print!("{label}: "),
    }
    io::stdout().flush().context("failed to flush prompt")?;
    let mut line = String::new();
    if io::stdin()
        .read_line(&mut line)
        .context("failed to read input")?
        == 0
    {
        return Ok(None);
    }
    let value = line.trim().to_owned();
    if value == "0" {
        return Ok(None);
    }
    if value.is_empty() {
        return Ok(default.map(str::to_owned));
    }
    Ok(Some(value))
}

fn confirm_write() -> Result<bool> {
    let Some(value) = prompt_text("确认执行？y/N", Some("N"))? else {
        return Ok(false);
    };
    Ok(matches!(value.to_ascii_lowercase().as_str(), "y" | "yes"))
}

fn strip_drag_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(value)
}

fn default_directory_output(input: &Path, suffix: &str) -> Result<PathBuf> {
    let parent = input
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = input
        .file_stem()
        .or_else(|| input.file_name())
        .context("input path has no final component")?;
    let mut output_name = name.to_os_string();
    output_name.push(suffix);
    Ok(parent.join(output_name))
}

fn default_file_output(input: &Path, suffix: &str, extension: &str) -> Result<PathBuf> {
    let mut output = default_directory_output(input, suffix)?;
    output.set_extension(extension);
    Ok(output)
}

fn append_suffix_to_filename(input: &Path, suffix: &str) -> Result<PathBuf> {
    let name = input.file_name().context("input path has no filename")?;
    let mut output_name = name.to_os_string();
    output_name.push(suffix);
    Ok(input.with_file_name(output_name))
}

fn injected_file_output(source: &Path) -> Result<PathBuf> {
    let parent = source
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let stem = source.file_stem().context("source file has no stem")?;
    let mut name = stem.to_os_string();
    name.push("_injected");
    let mut output = parent.join(name);
    if let Some(extension) = source.extension() {
        output.set_extension(extension);
    }
    Ok(output)
}

fn is_help(value: &OsString) -> bool {
    matches!(value.to_str(), Some("-h" | "--help" | "help"))
}

fn text_arg(value: &OsString, role: &str) -> Result<String> {
    value
        .to_str()
        .map(str::to_owned)
        .with_context(|| format!("{role} is not valid Unicode"))
}

#[derive(Debug)]
struct Options {
    values: HashMap<String, OsString>,
    flags: HashSet<String>,
}

impl Options {
    fn parse(args: &[OsString]) -> Result<Self> {
        let mut values = HashMap::new();
        let mut flags = HashSet::new();
        let mut cursor = 0usize;
        while cursor < args.len() {
            let key = text_arg(&args[cursor], "option")?;
            if !key.starts_with("--") {
                bail!("unexpected positional argument {:?}", args[cursor]);
            }
            if key == "--overwrite" {
                if !flags.insert(key.clone()) {
                    bail!("duplicate option {key}");
                }
                cursor += 1;
                continue;
            }
            let value = args
                .get(cursor + 1)
                .with_context(|| format!("missing value for {key}"))?;
            if value.to_string_lossy().starts_with("--") {
                bail!("missing value for {key}");
            }
            if values.insert(key.clone(), value.clone()).is_some() {
                bail!("duplicate option {key}");
            }
            cursor += 2;
        }
        Ok(Self { values, flags })
    }

    fn required_path(&mut self, name: &str) -> Result<PathBuf> {
        self.values
            .remove(name)
            .map(PathBuf::from)
            .with_context(|| format!("missing required option {name}"))
    }

    fn optional_path(&mut self, name: &str) -> Result<Option<PathBuf>> {
        Ok(self.values.remove(name).map(PathBuf::from))
    }

    fn optional_string(&mut self, name: &str) -> Result<Option<String>> {
        self.values
            .remove(name)
            .map(|value| text_arg(&value, name))
            .transpose()
    }

    fn flag(&mut self, name: &str) -> bool {
        self.flags.remove(name)
    }

    fn finish(self) -> Result<()> {
        if let Some(name) = self.values.keys().next() {
            bail!("unknown option {name}");
        }
        if let Some(name) = self.flags.iter().next() {
            bail!("unknown flag {name}");
        }
        Ok(())
    }
}
