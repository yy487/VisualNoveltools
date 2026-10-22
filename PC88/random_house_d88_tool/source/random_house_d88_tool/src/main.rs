use anyhow::{bail, Context, Result};
use random_house_d88_tool::d88::D88Image;
use random_house_d88_tool::font::{encode_probe_text, rebuild_kanji1, scan_translation_root};
use random_house_d88_tool::patch::{
    patch_bootstrap, patch_comment_probe, patch_payload_probe, patch_reserve_only, roundtrip_disk,
};
use random_house_d88_tool::text::{extract_text, inject_text};
use std::collections::{HashMap, HashSet};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;

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
    if args.len() == 1 && Path::new(&args[0]).exists() {
        return interactive(Some(PathBuf::from(&args[0])));
    }
    let command = args[0].to_string_lossy();
    let remaining = args[1..].to_vec();
    if command == "font" {
        return run_font_command(&remaining);
    }
    if command == "text" {
        return run_text_command(&remaining);
    }
    let options = parse_options(remaining)?;
    match command.as_ref() {
        "roundtrip" => {
            let report = roundtrip_disk(&options.source, &options.output, options.overwrite)?;
            println!(
                "[roundtrip] files={} basic_files={} byte_exact={} output_bytes={} sha256={} output={}",
                report.files,
                report.basic_files,
                report.byte_exact,
                report.output_bytes,
                report.output_sha256,
                report.output.display()
            );
        }
        "bootstrap" => {
            let report = patch_bootstrap(&options.source, &options.output, options.overwrite)?;
            println!(
                "[bootstrap] files={} basic_files={} clear_patches={} changed_files={} renderer_bytes={} renderer_sha256={} payload_line_start={} title_old_bytes={} title_program_bytes={} title_new_bytes={} comment_bytes_removed={} output_bytes={} sha256={} output={}",
                report.files,
                report.basic_files,
                report.clear_patches,
                report.changed_files,
                report.renderer_bytes,
                report.renderer_sha256,
                report.payload_line_start,
                report.title_old_bytes,
                report.title_program_bytes,
                report.title_new_bytes,
                report.comment_bytes_removed,
                report.output_bytes,
                report.output_sha256,
                report.output.display()
            );
        }
        "reserve" => {
            let report = patch_reserve_only(&options.source, &options.output, options.overwrite)?;
            println!(
                "[reserve] files={} basic_files={} clear_patches={} changed_files={} output_bytes={} sha256={} output={}",
                report.files,
                report.basic_files,
                report.clear_patches,
                report.changed_files,
                report.output_bytes,
                report.output_sha256,
                report.output.display()
            );
        }
        "comment-probe" => {
            let report = patch_comment_probe(&options.source, &options.output, options.overwrite)?;
            println!(
                "[comment-probe] files={} basic_files={} clear_patches={} changed_files={} output_bytes={} sha256={} output={}",
                report.files,
                report.basic_files,
                report.clear_patches,
                report.changed_files,
                report.output_bytes,
                report.output_sha256,
                report.output.display()
            );
        }
        "payload-probe" => {
            let report = patch_payload_probe(&options.source, &options.output, options.overwrite)?;
            println!(
                "[payload-probe] files={} basic_files={} clear_patches={} changed_files={} output_bytes={} sha256={} output={}",
                report.files,
                report.basic_files,
                report.clear_patches,
                report.changed_files,
                report.output_bytes,
                report.output_sha256,
                report.output.display()
            );
        }
        _ => bail!("unknown command {command:?}; use --help"),
    }
    Ok(())
}

fn run_text_command(args: &[OsString]) -> Result<()> {
    let operation = args
        .first()
        .context("missing text operation; expected extract or inject")?
        .to_string_lossy();
    let mut options = NamedOptions::parse(&args[1..])?;
    match operation.as_ref() {
        "extract" => {
            let source = options.required_path("--source")?;
            let output = options.required_path("--output")?;
            let overwrite = options.flag("--overwrite");
            options.finish()?;
            let report = extract_text(&source, &output, overwrite)?;
            println!(
                "[text extract] basic_files={} entries={} skipped_nontext={} output={}",
                report.basic_files,
                report.entries,
                report.skipped_nontext,
                report.output.display()
            );
            Ok(())
        }
        "inject" => {
            let source = options.required_path("--source")?;
            let translation = options.required_path("--translation")?;
            let font_map = options.required_path("--font-map")?;
            let output = options.required_path("--output")?;
            let overwrite = options.flag("--overwrite");
            options.finish()?;
            let report = inject_text(&source, &translation, &font_map, &output, overwrite)?;
            println!(
                "[text inject] basic_files={} entries={} patched_entries={} unchanged_entries={} cls_patches={} transition_clear_patches={} native_ascii_echo_patches={} changed_files={} comment_bytes_removed={} ignored_space_bytes_removed={} output_bytes={} sha256={} output={}",
                report.basic_files,
                report.entries,
                report.patched_entries,
                report.unchanged_entries,
                report.cls_patches,
                report.transition_clear_patches,
                report.native_ascii_echo_patches,
                report.changed_files,
                report.comment_bytes_removed,
                report.ignored_space_bytes_removed,
                report.output_bytes,
                report.output_sha256,
                report.output.display()
            );
            Ok(())
        }
        _ => bail!("unknown text operation {operation:?}; expected extract or inject"),
    }
}

fn run_font_command(args: &[OsString]) -> Result<()> {
    let operation = args
        .first()
        .context("missing font operation; expected scan or rebuild")?
        .to_string_lossy();
    let mut options = NamedOptions::parse(&args[1..])?;
    match operation.as_ref() {
        "scan" => {
            let translations = options.required_path("--translations")?;
            let rom = options.required_path("--rom")?;
            let font = options.required_path("--font")?;
            let output = options.required_path("--output")?;
            let existing = options.optional_path("--existing");
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
        "rebuild" => {
            let rom = options.required_path("--rom")?;
            let map = options.required_path("--map")?;
            let font = options.required_path("--font")?;
            let output = options.required_path("--output")?;
            let atlas = options.optional_path("--atlas");
            let overwrite = options.flag("--overwrite");
            options.finish()?;
            run_font_rebuild(&rom, &map, &font, &output, atlas.as_deref(), overwrite)
        }
        "encode" => {
            let map = options.required_path("--map")?;
            let text = options.required_string("--text")?;
            let output = options.required_path("--output")?;
            let overwrite = options.flag("--overwrite");
            options.finish()?;
            let report = encode_probe_text(&map, &text, &output, overwrite)?;
            println!(
                "[font encode] characters={} bytes={} output={}",
                report.characters,
                report.bytes,
                report.output.display()
            );
            Ok(())
        }
        _ => bail!("unknown font operation {operation:?}; expected scan, rebuild, or encode"),
    }
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
    println!(
        "[font scan] json_files={} entries_scanned={} required_characters={} new_mappings={} remapped_mappings={} total_mappings={} output={}",
        report.json_files,
        report.entries_scanned,
        report.required_characters,
        report.new_mappings,
        report.remapped_mappings,
        report.total_mappings,
        report.output.display()
    );
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
    println!(
        "[font rebuild] mappings={} native_glyphs={} changed_slots={} changed_bytes={} output={}",
        report.mappings,
        report.native_glyphs,
        report.changed_slots,
        report.changed_bytes,
        report.output.display()
    );
    if let Some(atlas) = report.atlas {
        println!("[font rebuild] atlas={}", atlas.display());
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrefillKind {
    DiskImage,
    TranslationJson,
    TranslationRoot,
    FontMap,
    KanjiRom,
    FontFile,
    Unknown,
}

fn interactive(prefill: Option<PathBuf>) -> Result<()> {
    let kind = prefill
        .as_deref()
        .map(detect_prefill_kind)
        .transpose()?
        .unwrap_or(PrefillKind::Unknown);
    println!("random-house-d88-tool 交互模式");
    if let Some(path) = &prefill {
        println!("预填路径：{}", path.display());
        println!("识别类型：{}", prefill_label(kind));
    }

    loop {
        println!();
        println!("1. 从 D88 提取纯文本 JSON");
        println!("2. 扫描译文并更新 KANJI1 字体映射");
        println!("3. 重建 KANJI1 ROM");
        println!("4. 注入译文并生成补丁 D88");
        println!("5. D88 无修改回环验证");
        println!("0. 退出");
        let suggested = match kind {
            PrefillKind::DiskImage => Some("1"),
            PrefillKind::TranslationJson | PrefillKind::TranslationRoot => Some("2"),
            PrefillKind::FontMap | PrefillKind::KanjiRom | PrefillKind::FontFile => Some("3"),
            PrefillKind::Unknown => None,
        };
        let Some(choice) = prompt_text("请选择操作编号", suggested)? else {
            return Ok(());
        };
        if choice == "0" {
            return Ok(());
        }

        let result = match choice.as_str() {
            "1" => interactive_text_extract(prefill.as_deref(), kind),
            "2" => interactive_font_scan(prefill.as_deref(), kind),
            "3" => interactive_font_rebuild(prefill.as_deref(), kind),
            "4" => interactive_text_inject(prefill.as_deref(), kind),
            "5" => interactive_roundtrip(prefill.as_deref(), kind),
            _ => {
                println!("未知操作编号：{choice}");
                continue;
            }
        };
        if let Err(error) = result {
            println!("操作未完成：{error:#}");
        }
        println!("已返回主菜单。");
    }
}

fn interactive_text_extract(prefill: Option<&Path>, kind: PrefillKind) -> Result<()> {
    let source_default = prefill.filter(|_| kind == PrefillKind::DiskImage);
    let Some(source) = prompt_d88_file("源 D88 路径（可拖入文件）", source_default)?
    else {
        return Ok(());
    };
    let default = output_with_suffix(&source, "_translation", "json")?;
    let Some((output, overwrite)) = prompt_output_path("翻译 JSON 输出路径", &default)?
    else {
        return Ok(());
    };
    print_preflight("text extract", &[("source", &source)], &output, overwrite);
    if !confirm_write()? {
        println!("已取消，未写入文件。");
        return Ok(());
    }
    let report = extract_text(&source, &output, overwrite)?;
    println!(
        "[text extract] basic_files={} entries={} skipped_nontext={} output={}",
        report.basic_files,
        report.entries,
        report.skipped_nontext,
        report.output.display()
    );
    Ok(())
}

fn interactive_text_inject(prefill: Option<&Path>, kind: PrefillKind) -> Result<()> {
    let source_default = prefill.filter(|_| kind == PrefillKind::DiskImage);
    let translation_default = prefill.filter(|_| kind == PrefillKind::TranslationJson);
    let map_default = prefill.filter(|_| kind == PrefillKind::FontMap);
    let Some(source) = prompt_d88_file("原始 D88 路径", source_default)? else {
        return Ok(());
    };
    let Some(translation) = prompt_existing_path("翻译 JSON 路径", translation_default)? else {
        return Ok(());
    };
    let Some(font_map) = prompt_existing_path("字体映射 JSON 路径", map_default)? else {
        return Ok(());
    };
    let default = output_with_suffix(&source, "_patched", "d88")?;
    let Some((output, overwrite)) = prompt_output_path("补丁 D88 输出路径", &default)? else {
        return Ok(());
    };
    print_preflight(
        "text inject",
        &[
            ("source", &source),
            ("translation", &translation),
            ("font_map", &font_map),
        ],
        &output,
        overwrite,
    );
    if !confirm_write()? {
        println!("已取消，未写入文件。");
        return Ok(());
    }
    let report = inject_text(&source, &translation, &font_map, &output, overwrite)?;
    println!(
        "[text inject] entries={} patched={} unchanged={} transition_clear_patches={} changed_files={} comment_bytes_removed={} sha256={} output={}",
        report.entries,
        report.patched_entries,
        report.unchanged_entries,
        report.transition_clear_patches,
        report.changed_files,
        report.comment_bytes_removed,
        report.output_sha256,
        report.output.display()
    );
    Ok(())
}

fn interactive_font_scan(prefill: Option<&Path>, kind: PrefillKind) -> Result<()> {
    let translations_default = prefill.filter(|_| {
        matches!(
            kind,
            PrefillKind::TranslationJson | PrefillKind::TranslationRoot
        )
    });
    let rom_default = prefill.filter(|_| kind == PrefillKind::KanjiRom);
    let font_default = prefill.filter(|_| kind == PrefillKind::FontFile);
    let map_default = prefill.filter(|_| kind == PrefillKind::FontMap);
    let Some(translations) = prompt_existing_path("翻译 JSON 文件或目录", translations_default)?
    else {
        return Ok(());
    };
    let Some(rom) = prompt_existing_path("原始 kanji1.rom 路径", rom_default)? else {
        return Ok(());
    };
    let Some(font) = prompt_existing_path("字体文件路径", font_default)? else {
        return Ok(());
    };
    let existing = prompt_optional_existing_path("已有字体映射 JSON（留空表示新建）", map_default)?;
    let default =
        existing
            .clone()
            .unwrap_or(output_with_suffix(&translations, "_font_map", "json")?);
    let Some((output, overwrite)) = prompt_output_path("字体映射 JSON 输出路径", &default)?
    else {
        return Ok(());
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
    if !confirm_write()? {
        println!("已取消，未写入文件。");
        return Ok(());
    }
    run_font_scan(
        &translations,
        &rom,
        &font,
        existing.as_deref(),
        &output,
        overwrite,
    )
}

fn interactive_font_rebuild(prefill: Option<&Path>, kind: PrefillKind) -> Result<()> {
    let rom_default = prefill.filter(|_| kind == PrefillKind::KanjiRom);
    let map_default = prefill.filter(|_| kind == PrefillKind::FontMap);
    let font_default = prefill.filter(|_| kind == PrefillKind::FontFile);
    let Some(rom) = prompt_existing_path("原始 kanji1.rom 路径", rom_default)? else {
        return Ok(());
    };
    let Some(map) = prompt_existing_path("字体映射 JSON 路径", map_default)? else {
        return Ok(());
    };
    let Some(font) = prompt_existing_path("字体文件路径", font_default)? else {
        return Ok(());
    };
    let default = output_with_suffix(&rom, "_rebuilt", "rom")?;
    let Some((output, overwrite)) = prompt_output_path("新 kanji1.rom 输出路径", &default)?
    else {
        return Ok(());
    };
    let atlas = output.with_extension("pgm");
    print_preflight(
        "font rebuild",
        &[("rom", &rom), ("map", &map), ("font", &font)],
        &output,
        overwrite,
    );
    if !confirm_write()? {
        println!("已取消，未写入文件。");
        return Ok(());
    }
    run_font_rebuild(&rom, &map, &font, &output, Some(&atlas), overwrite)
}

fn interactive_roundtrip(prefill: Option<&Path>, kind: PrefillKind) -> Result<()> {
    let source_default = prefill.filter(|_| kind == PrefillKind::DiskImage);
    let Some(source) = prompt_d88_file("D88 路径", source_default)? else {
        return Ok(());
    };
    let default = output_with_suffix(&source, "_roundtrip", "d88")?;
    let Some((output, overwrite)) = prompt_output_path("回环 D88 输出路径", &default)? else {
        return Ok(());
    };
    print_preflight("roundtrip", &[("source", &source)], &output, overwrite);
    if !confirm_write()? {
        println!("已取消，未写入文件。");
        return Ok(());
    }
    let report = roundtrip_disk(&source, &output, overwrite)?;
    println!(
        "[roundtrip] byte_exact={} sha256={} output={}",
        report.byte_exact,
        report.output_sha256,
        report.output.display()
    );
    Ok(())
}

fn detect_prefill_kind(path: &Path) -> Result<PrefillKind> {
    if path.is_dir() {
        return Ok(if directory_contains_translation(path)? {
            PrefillKind::TranslationRoot
        } else {
            PrefillKind::Unknown
        });
    }
    if !path.is_file() {
        return Ok(PrefillKind::Unknown);
    }
    if let Some(format) = read_json_format(path)? {
        return Ok(match format.as_str() {
            "random-house-basic-json-v1" => PrefillKind::TranslationJson,
            "random-house-kanji1-map-v1" => PrefillKind::FontMap,
            _ => PrefillKind::Unknown,
        });
    }
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    if D88Image::parse(bytes.clone()).is_ok() {
        return Ok(PrefillKind::DiskImage);
    }
    if bytes.len() == 0x20_000 {
        return Ok(PrefillKind::KanjiRom);
    }
    if is_font_file(&bytes) {
        return Ok(PrefillKind::FontFile);
    }
    Ok(PrefillKind::Unknown)
}

fn directory_contains_translation(root: &Path) -> Result<bool> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        for entry in fs::read_dir(&directory)
            .with_context(|| format!("failed to read directory {}", directory.display()))?
        {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                stack.push(entry.path());
            } else if file_type.is_file()
                && read_json_format(&entry.path())?.as_deref() == Some("random-house-basic-json-v1")
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
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
        .get("_format")
        .or_else(|| value.get("format"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned))
}

fn is_font_file(bytes: &[u8]) -> bool {
    matches!(
        bytes.get(..4),
        Some([0x00, 0x01, 0x00, 0x00] | b"OTTO" | b"ttcf" | b"true")
    )
}

fn prefill_label(kind: PrefillKind) -> &'static str {
    match kind {
        PrefillKind::DiskImage => "D88 镜像",
        PrefillKind::TranslationJson => "翻译 JSON",
        PrefillKind::TranslationRoot => "包含翻译 JSON 的目录",
        PrefillKind::FontMap => "KANJI1 字体映射 JSON",
        PrefillKind::KanjiRom => "KANJI1 ROM 候选",
        PrefillKind::FontFile => "TrueType/OpenType 字体",
        PrefillKind::Unknown => "未知；请在菜单中明确指定用途",
    }
}

fn prompt_d88_file(label: &str, default: Option<&Path>) -> Result<Option<PathBuf>> {
    loop {
        let Some(path) = prompt_existing_path(label, default)? else {
            return Ok(None);
        };
        if path.is_file() {
            let bytes = fs::read(&path)
                .with_context(|| format!("failed to read D88 {}", path.display()))?;
            if D88Image::parse(bytes).is_ok() {
                return Ok(Some(path));
            }
        }
        println!("不是本工具支持的 D88 镜像：{}", path.display());
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

fn prompt_optional_existing_path(label: &str, default: Option<&Path>) -> Result<Option<PathBuf>> {
    let default_text = default.map(|path| path.to_string_lossy().into_owned());
    let Some(value) = prompt_text(label, default_text.as_deref())? else {
        return Ok(None);
    };
    let path = PathBuf::from(strip_drag_quotes(&value));
    if !path.exists() {
        bail!("路径不存在：{}", path.display());
    }
    Ok(Some(path))
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
        let Some(choice) = prompt_text("m=修改路径，o=覆盖，0=取消", None)? else {
            return Ok(None);
        };
        match choice.to_ascii_lowercase().as_str() {
            "m" | "modify" => suggestion = path,
            "o" | "overwrite" => return Ok(Some((path, true))),
            _ => println!("请输入 m、o 或 0。"),
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

fn print_preflight(operation: &str, inputs: &[(&str, &Path)], output: &Path, overwrite: bool) {
    println!("操作：{operation}");
    for (role, path) in inputs {
        println!("{role}={}", path.display());
    }
    println!("output={}", output.display());
    println!("overwrite={overwrite}");
}

fn strip_drag_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(value)
}

fn output_with_suffix(input: &Path, suffix: &str, extension: &str) -> Result<PathBuf> {
    let parent = input
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let stem = input
        .file_stem()
        .or_else(|| input.file_name())
        .context("input path has no final component")?;
    let mut name = stem.to_os_string();
    name.push(suffix);
    let mut output = parent.join(name);
    output.set_extension(extension);
    Ok(output)
}

fn is_help(value: &OsString) -> bool {
    matches!(value.to_str(), Some("-h" | "--help" | "help"))
}

struct Options {
    source: PathBuf,
    output: PathBuf,
    overwrite: bool,
}

#[derive(Debug)]
struct NamedOptions {
    values: HashMap<String, OsString>,
    flags: HashSet<String>,
}

impl NamedOptions {
    fn parse(args: &[OsString]) -> Result<Self> {
        let mut values = HashMap::new();
        let mut flags = HashSet::new();
        let mut index = 0usize;
        while index < args.len() {
            let key = args[index]
                .to_str()
                .context("option name is not valid Unicode")?
                .to_owned();
            if !key.starts_with("--") {
                bail!("unexpected positional argument {:?}", args[index]);
            }
            if key == "--overwrite" {
                if !flags.insert(key.clone()) {
                    bail!("duplicate option {key}");
                }
                index += 1;
                continue;
            }
            let value = args
                .get(index + 1)
                .with_context(|| format!("missing value for {key}"))?;
            if value.to_string_lossy().starts_with("--") {
                bail!("missing value for {key}");
            }
            if values.insert(key.clone(), value.clone()).is_some() {
                bail!("duplicate option {key}");
            }
            index += 2;
        }
        Ok(Self { values, flags })
    }

    fn required_path(&mut self, name: &str) -> Result<PathBuf> {
        self.values
            .remove(name)
            .map(PathBuf::from)
            .with_context(|| format!("missing required option {name}"))
    }

    fn required_string(&mut self, name: &str) -> Result<String> {
        self.values
            .remove(name)
            .with_context(|| format!("missing required option {name}"))?
            .into_string()
            .map_err(|_| anyhow::anyhow!("{name} is not valid Unicode"))
    }

    fn optional_path(&mut self, name: &str) -> Option<PathBuf> {
        self.values.remove(name).map(PathBuf::from)
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

fn parse_options(args: Vec<std::ffi::OsString>) -> Result<Options> {
    let mut source = None;
    let mut output = None;
    let mut overwrite = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].to_string_lossy().as_ref() {
            "--source" => {
                index += 1;
                source = Some(PathBuf::from(
                    args.get(index).context("--source needs a path")?,
                ));
            }
            "--output" => {
                index += 1;
                output = Some(PathBuf::from(
                    args.get(index).context("--output needs a path")?,
                ));
            }
            "--overwrite" => overwrite = true,
            option => bail!("unknown option {option:?}"),
        }
        index += 1;
    }
    Ok(Options {
        source: source.context("missing --source path")?,
        output: output.context("missing --output path")?,
        overwrite,
    })
}

fn print_help() {
    println!(
        "random-house-d88-tool\n\n\
         Usage:\n\
           random-house-d88-tool\n\
           random-house-d88-tool <PATH>\n\
           random-house-d88-tool roundtrip --source <disk.d88> --output <rebuilt.d88> [--overwrite]\n\
           random-house-d88-tool reserve --source <disk.d88> --output <reserved.d88> [--overwrite]\n\
           random-house-d88-tool bootstrap --source <disk.d88> --output <patched.d88> [--overwrite]\n\n\
           random-house-d88-tool text extract --source <disk.d88> --output <translation.json> [--overwrite]\n\
           random-house-d88-tool text inject --source <disk.d88> --translation <translation.json> --font-map <MAP.JSON> --output <patched.d88> [--overwrite]\n\n\
           random-house-d88-tool font scan --translations <JSON|DIR> --rom <kanji1.rom> --font <FONT> --output <MAP.JSON> [--existing <MAP.JSON>] [--overwrite]\n\
           random-house-d88-tool font rebuild --rom <kanji1.rom> --map <MAP.JSON> --font <FONT> --output <kanji1_new.rom> [--atlas <atlas.pgm>] [--overwrite]\n\n\
           random-house-d88-tool font encode --map <MAP.JSON> --text <TEXT> --output <BYTES> [--overwrite]\n\n\
         roundtrip  Structure-aware no-change D88/BASIC rebuild; output must be byte-exact.\n\
         reserve    Patch the 15 CLEAR ceilings to BBFF without inserting a loader.\n\
         bootstrap  Reserve BC00-BFFF, embed the validated renderer in TITL, and install ED42.\n\n\
         text extract/inject uses UTF-8 JSON with immutable scr_msg and pure message fields.\n\
         Injection also reserves RAM and installs the resident renderer in the rebuilt D88.\n\n\
         font scan/rebuild uses Windows GDI, fixed fullwidth slots for one-byte visible\n\
         ASCII, and printable two-byte carriers for other mapped glyphs. Legacy v1-v5\n\
         maps must be migrated with font scan, then the D88 and kanji1.rom rebuilt together.\n\n\
         No arguments opens a persistent interactive menu. A single existing path opens the\n\
         same menu with content-based prefill. Inputs and outputs remain editable, and no\n\
         interactive write occurs before an explicit confirmation.\n\n\
         The output path must differ from the source. Existing outputs are refused unless\n\
         --overwrite is explicitly supplied."
    );
}
