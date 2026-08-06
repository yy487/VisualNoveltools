use merry_mpk_tool::extract::{default_extract_output, extract_path, extract_path_with_progress};
use merry_mpk_tool::font::{build_font_pair, missing_ttf_glyphs};
use merry_mpk_tool::inject::inject_path;
use merry_mpk_tool::script::{parse_msb, parse_scx, rebuild_msb, rebuild_scx};
use merry_mpk_tool::workflow::{
    build_translation_fonts_with_progress, build_translation_resources_with_progress,
    plan_translation_fonts, plan_translation_resources, required_render_targets,
    TranslationBuildProgress,
};
use merry_mpk_tool::workspace::{
    plan_translation_workspace, prepare_translation_workspace_with_progress,
    TranslationWorkspacePlan, TranslationWorkspaceProgress,
};
use merry_mpk_tool::{
    default_pack_output, default_unpack_output, pack_archive, parse_archive, unpack_archive,
    ToolError, ToolResult, MANIFEST_FILE,
};
use std::collections::{BTreeSet, HashMap};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const USAGE: &str = r#"MyMerryMay MPK/script/font tool

双击运行（无参数）会进入：
  1  自动解包 / 提取翻译 JSON
  2  构建翻译资源（MSB/SCX + BIN/PNG）
  3  仅解包/封包/验证
  4  仅重建 BIN/PNG 字库
  0  退出

Usage:
  mpk_tool.exe prepare [--output DIR] PACKAGE_DIR
  mpk_tool.exe unpack [--output DIR] ARCHIVE.mpk [ARCHIVE.mpk ...]
  mpk_tool.exe pack [--output ARCHIVE.mpk] UNPACKED_DIR
  mpk_tool.exe extract [--output DIR] SCRIPT_FILE_OR_DIR
  mpk_tool.exe inject [--output PATH] SCRIPT_FILE_OR_DIR TRANSLATION_JSON_OR_DIR
  mpk_tool.exe font-build [options] FONT_DIR_OR_BIN
  mpk_tool.exe verify FILE_OR_DIR
  mpk_tool.exe map

font-build options:
  --font TTF              TTF used to redraw mapped Chinese characters
  --donor TTF              fallback TTF for missing mapped characters (repeatable)
  --bin BIN               font_df_jpn.bin (optional when input is a directory)
  --png PNG               font_df_jpn.png (optional when input is a directory)
  --mapping JSON          extra target->carrier mappings, merged with the built-in table
  --output-dir DIR        defaults to FONT_DIR\chs
  --all                   also build font2_df_jpn.bin/.png when present

The built-in mapping is compiled from subs_cn_jp.json. Translation JSON is UTF-8
and keeps immutable source metadata internally; translators edit only message/name.
命令行输出默认不覆盖已有文件；交互模式会明确询问是否覆盖 chs/font_chs。
Dragging a three-MPK game directory prepares a workspace; MPK files unpack and manifest
directories pack.
"#;

fn main() {
    if let Err(error) = run() {
        eprintln!("[error] {error}");
        std::process::exit(1);
    }
}

fn run() -> ToolResult<()> {
    let args = env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    if args.is_empty() {
        return run_interactive();
    }
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print!("{USAGE}");
        return Ok(());
    }
    let command = args[0].to_string_lossy().to_ascii_lowercase();
    match command.as_str() {
        "prepare" | "workspace" => run_prepare(&args[1..]),
        "unpack" => run_unpack(&args[1..]),
        "pack" => run_pack(&args[1..]),
        "extract" => run_extract(&args[1..]),
        "inject" => run_inject(&args[1..]),
        "font-build" | "font" | "fnt" => run_font_build(&args[1..]),
        "verify" => run_verify(&args[1..]),
        "map" => {
            let dictionary = merry_mpk_tool::glyph::GlyphDictionary::built_in()?;
            println!("[map] embedded_entries={}", dictionary.target_count());
            Ok(())
        }
        _ => run_inferred(&args),
    }
}

type InteractiveResult = ToolResult<Option<String>>;

#[derive(Debug, Clone, Copy)]
enum PathRequirement {
    File,
    Directory,
    FileOrDirectory,
}

#[derive(Debug, Default)]
struct ResourceCounts {
    msb: usize,
    scx: usize,
    font_pairs: usize,
}

fn run_interactive() -> ToolResult<()> {
    let dictionary = merry_mpk_tool::glyph::GlyphDictionary::built_in()?;
    let mut last_notice = None;
    loop {
        clear_screen()?;
        println!("MyMerryMay 本地化工具");
        println!("======================");
        if let Some(notice) = &last_notice {
            println!("{notice}");
            println!();
        }
        println!("1  自动解包 / 提取翻译 JSON");
        println!("2  构建翻译资源（MSB/SCX + BIN/PNG）");
        println!("3  仅解包/封包/验证");
        println!("4  仅重建 BIN/PNG 字库");
        println!("0  退出");
        println!();
        println!("也可以直接把游戏数据目录、MPK、JSON 或解包目录拖入窗口。");
        let choice = prompt_line("请选择模式或粘贴路径: ")?;
        let result = match choice.trim() {
            "1" => interactive_extract(&dictionary, None),
            "2" => interactive_build(&dictionary, None),
            "3" => interactive_archive(),
            "4" => interactive_font_build(&dictionary, None),
            "0" => return Ok(()),
            _ => {
                let path = clean_input_path(&choice);
                if path.exists() {
                    interactive_smart_path(path, &dictionary)
                } else {
                    Err(ToolError(format!(
                        "请输入 1、2、3、4、0，或有效路径：'{}'",
                        path.display()
                    )))
                }
            }
        };
        match result {
            Ok(Some(summary)) => {
                last_notice = Some(format!("[完成] {summary}"));
                pause_for_menu()?;
            }
            Ok(None) => {}
            Err(error) => {
                eprintln!("[错误] {error}");
                last_notice = Some(format!("[上次错误] {error}"));
                pause_for_menu()?;
            }
        }
    }
}

fn interactive_smart_path(
    path: PathBuf,
    dictionary: &merry_mpk_tool::glyph::GlyphDictionary,
) -> InteractiveResult {
    if path.is_file() {
        if extension_is(&path, "mpk") {
            println!("已识别为 MPK，进入解包流程。");
            return interactive_unpack(Some(path));
        }
        if extension_is(&path, "json") {
            println!("已识别为翻译 JSON，进入构建流程。");
            return interactive_build(dictionary, Some(path));
        }
        if extension_is(&path, "msb") || extension_is(&path, "scx") {
            println!("已识别为脚本文件，进入提取流程。");
            return interactive_extract(dictionary, Some(path));
        }
        return Err(ToolError(format!(
            "无法根据文件类型判断操作：'{}'",
            path.display()
        )));
    }
    if path.join(MANIFEST_FILE).is_file() {
        println!("已识别为 MPK 解包目录，进入封包流程。");
        return interactive_pack(Some(path));
    }
    if merry_mpk_tool::workspace::resolve_translation_workspace(&path)?.is_some() {
        println!("已识别为翻译工作区，进入脚本及字库构建流程。");
        return interactive_build(dictionary, Some(path));
    }
    if plan_translation_workspace(&path).is_ok() {
        println!("已识别到 mes00/script/system_win 三包，进入自动提取流程。");
        return interactive_extract(dictionary, Some(path));
    }
    let counts = scan_resource_counts(&path)?;
    if counts.msb + counts.scx > 0 {
        println!("已识别到 MSB/SCX，进入提取流程。");
        return interactive_extract(dictionary, Some(path));
    }
    if contains_extension(&path, "json")? {
        println!("已识别到翻译 JSON，进入构建流程。");
        return interactive_build(dictionary, Some(path));
    }
    Err(ToolError(format!(
        "目录中没有可识别的 MPK manifest、脚本或翻译 JSON：'{}'",
        path.display()
    )))
}

fn interactive_extract(
    dictionary: &merry_mpk_tool::glyph::GlyphDictionary,
    mut initial: Option<PathBuf>,
) -> InteractiveResult {
    let input = loop {
        let path = if let Some(path) = initial.take() {
            path
        } else {
            println!();
            println!("自动解包 / 提取翻译 JSON");
            println!("推荐输入同时含 mes00.mpk、script.mpk、system_win.mpk 的游戏目录。");
            println!("也支持包含多个 DataM? 目录的上级目录，以及已解包脚本/单个脚本。");
            println!("示例：E:\\...\\MyMerryMayWithbe\\DataMB");
            println!("输入 0 返回主菜单。");
            let Some(path) =
                prompt_existing_path("脚本文件或目录: ", PathRequirement::FileOrDirectory)?
            else {
                return Ok(None);
            };
            path
        };
        if path.is_dir() {
            if let Ok(plan) = plan_translation_workspace(&path) {
                return interactive_workspace_extract(dictionary, &path, plan);
            }
        }
        if path.is_file() && !(extension_is(&path, "msb") || extension_is(&path, "scx")) {
            eprintln!("[错误] 请选择 .msb、.scx 或包含这些脚本的目录。");
            continue;
        }
        let counts = scan_resource_counts(&path)?;
        if counts.msb + counts.scx == 0 {
            eprintln!("[错误] 未在 '{}' 中检测到 .msb/.scx。", path.display());
            continue;
        }
        break path;
    };
    let counts = scan_resource_counts(&input)?;
    let output = default_extract_output(&input)?;
    if output.exists() {
        return Err(ToolError(format!(
            "输出目录已经存在，请先改名或移走：'{}'",
            output.display()
        )));
    }
    println!();
    println!("执行前检查");
    println!("  输入：{}", input.display());
    println!("  MSB：{} 个", counts.msb);
    println!("  SCX：{} 个", counts.scx);
    println!("  字库：{} 套", counts.font_pairs);
    println!("  输出：{}", output.display());
    if !prompt_confirmation("确认开始提取？(Y/n，0 返回): ", true)? {
        return Ok(None);
    }

    let mut progress = |current: usize, total: usize, path: &Path| {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        print!("\r\x1B[2K[提取] {current}/{total}  {name}");
        let _ = io::stdout().flush();
    };
    let report = extract_path_with_progress(&input, None, dictionary, &mut progress)?;
    println!("\r\x1B[2K[提取] 已处理 {} 个脚本。", report.scanned_files);
    for warning in &report.warnings {
        println!("[警告] {warning}");
    }
    println!();
    println!("提取完成");
    println!("  JSON 文件：{}", report.json_files);
    println!("  翻译条目：{}", report.extracted_entries);
    println!("  警告：{}", report.warnings.len());
    println!("  输出目录：{}", report.output.display());
    println!("下一步：翻译 JSON 中的 name/message，然后在主菜单选择 2。");
    Ok(Some(format!(
        "已提取 {} 条文本到 {}",
        report.extracted_entries,
        report.output.display()
    )))
}

fn interactive_workspace_extract(
    dictionary: &merry_mpk_tool::glyph::GlyphDictionary,
    input: &Path,
    plan: TranslationWorkspacePlan,
) -> InteractiveResult {
    if plan.output.exists() {
        return Err(ToolError(format!(
            "翻译工作区已经存在，请先改名或移走：'{}'",
            plan.output.display()
        )));
    }
    println!();
    println!("执行前检查");
    println!("  输入：{}", input.display());
    println!("  游戏数据目录：{} 个", plan.package_directories.len());
    for directory in &plan.package_directories {
        println!("    - {}", directory.display());
    }
    println!("  自动解包：{} 个 MPK", plan.archives);
    println!("  归档成员：{} 个", plan.archive_members);
    println!("  工作区：{}", plan.output.display());
    println!(
        "  翻译 JSON：{}",
        plan.output
            .join(merry_mpk_tool::workspace::TRANSLATION_DIR)
            .display()
    );
    println!("  JSON 字段：仅 name（有名字时）和 message");
    if !prompt_confirmation("确认自动解包并提取？(Y/n，0 返回): ", true)? {
        return Ok(None);
    }

    let mut progress = |event: TranslationWorkspaceProgress| match event {
        TranslationWorkspaceProgress::Unpacking {
            current,
            total,
            archive,
        } => {
            println!("[解包] {current}/{total}  {}", archive.display());
        }
        TranslationWorkspaceProgress::Extracting {
            current,
            total,
            script,
        } => {
            let name = script
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            print!("\r\x1B[2K[提取] {current}/{total}  {name}");
            let _ = io::stdout().flush();
        }
        TranslationWorkspaceProgress::WritingTranslationView => {
            println!("\r\x1B[2K[JSON] 正在生成仅含 name/message 的翻译文件...");
        }
        TranslationWorkspaceProgress::Finalizing => {
            println!("[完成] 正在写入翻译工作区...");
        }
    };
    let report = prepare_translation_workspace_with_progress(
        input,
        Some(&plan.output),
        dictionary,
        &mut progress,
    )?;
    for warning in &report.warnings {
        println!("[警告] {warning}");
    }
    println!();
    println!("自动提取完成");
    println!("  解包 MPK：{} 个", report.archives);
    println!("  解包成员：{} 个", report.extracted_members);
    println!("  脚本文件：{} 个", report.scanned_scripts);
    println!("  JSON 文件：{} 个", report.json_files);
    println!("  翻译条目：{} 条", report.translation_entries);
    println!("  翻译目录：{}", report.translation_json.display());
    println!("下一步：只编辑 name/message，然后在主菜单选择 2。");
    Ok(Some(format!(
        "已自动解包 {} 个 MPK，并提取 {} 条文本",
        report.archives, report.translation_entries
    )))
}

fn interactive_build(
    dictionary: &merry_mpk_tool::glyph::GlyphDictionary,
    mut initial: Option<PathBuf>,
) -> InteractiveResult {
    let (json_input, plan, targets) = loop {
        let path = if let Some(path) = initial.take() {
            path
        } else {
            println!();
            println!("构建翻译资源（MSB/SCX + BIN/PNG）");
            println!("请输入模式 1 生成的翻译工作区、translation_json 目录或兼容旧 JSON。");
            println!("示例：E:\\...\\DataMB_translation");
            println!("输入 0 返回主菜单。");
            let Some(path) =
                prompt_existing_path("翻译 JSON 文件或目录: ", PathRequirement::FileOrDirectory)?
            else {
                return Ok(None);
            };
            path
        };
        if path.is_file() && !extension_is(&path, "json") {
            eprintln!("[错误] 文件不是 JSON：'{}'", path.display());
            continue;
        }
        match (
            plan_translation_resources(&path, dictionary),
            required_render_targets(&path, dictionary),
        ) {
            (Ok(plan), Ok(targets)) => break (path, plan, targets),
            (Err(error), _) | (_, Err(error)) => {
                eprintln!("[错误] 无法读取这份翻译资源：{error}");
            }
        }
    };

    let Some((main_font, donors)) = prompt_redraw_fonts(&targets)? else {
        return Ok(None);
    };

    let existing_outputs = plan.output_dirs.iter().filter(|path| path.exists()).count();
    println!();
    println!("执行前检查");
    println!("  JSON：{}", plan.json_input.display());
    println!("  源目录：{}", plan.script_root.display());
    println!("  脚本文件：{} 个", plan.script_files);
    println!("  JSON 条目：{} 条", plan.json_entries);
    println!("  字体槽位冲突：0（已检查）");
    println!("  实际重绘字形：{} 个", plan.required_glyphs);
    println!("  字库：{} 套", plan.font_pairs);
    println!("  输出内容：重定位 MSB/SCX + 重建 BIN/PNG");
    println!("  主字体：{}", main_font.display());
    println!("  补字字体：{} 个", donors.len());
    for output in &plan.output_dirs {
        let state = if output.exists() {
            "（将覆盖）"
        } else {
            ""
        };
        println!("  输出：{}{}", output.display(), state);
    }
    let confirmed = if existing_outputs > 0 {
        prompt_confirmation("已有 chs 输出，确认覆盖并执行？(y/N，0 返回): ", false)?
    } else {
        prompt_confirmation("确认开始构建？(Y/n，0 返回): ", true)?
    };
    if !confirmed {
        return Ok(None);
    }

    let mut progress = |event: TranslationBuildProgress| match event {
        TranslationBuildProgress::InjectingScripts => {
            println!("[构建] 正在校验并注入 {} 个脚本文件...", plan.script_files);
        }
        TranslationBuildProgress::RenderingFont {
            current,
            total,
            input_png,
        } => {
            let name = input_png
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            println!("[字体] {current}/{total}  {name}");
        }
        TranslationBuildProgress::Finalizing => {
            println!("[构建] 正在完成 chs 输出...");
        }
    };
    let report = build_translation_resources_with_progress(
        &json_input,
        &main_font,
        &donors,
        existing_outputs > 0,
        dictionary,
        &mut progress,
    )?;
    println!();
    println!("构建完成");
    println!("  修改条目：{}", report.patched);
    println!("  未修改条目：{}", report.unchanged);
    println!("  重绘槽位：{}", report.rendered_slots);
    for output in &report.output_dirs {
        println!("  输出目录：{}", output.display());
    }
    if merry_mpk_tool::workspace::resolve_translation_workspace(&json_input)?.is_some() {
        println!(
            "下一步：在模式 3 中分别选择 chs 内含 manifest 的 mes00、script、system_win 目录封包。"
        );
    } else {
        println!("下一步：把各 chs 中的同名文件放回对应解包目录，再选择封包目录。");
    }
    Ok(Some(format!(
        "已构建 {} 条翻译，输出 {} 个 chs 目录",
        report.json_entries,
        report.output_dirs.len()
    )))
}

fn interactive_font_build(
    dictionary: &merry_mpk_tool::glyph::GlyphDictionary,
    mut initial: Option<PathBuf>,
) -> InteractiveResult {
    let (json_input, plan, targets) = loop {
        let path = if let Some(path) = initial.take() {
            path
        } else {
            println!();
            println!("仅重建 BIN/PNG 字库");
            println!("请输入模式 1 生成的翻译工作区或 translation_json 目录。");
            println!("工具会先检查槽位冲突，再按最终译文实际用字统一重绘所有匹配字库。");
            println!("输入 0 返回主菜单。");
            let Some(path) =
                prompt_existing_path("翻译 JSON 或工作区: ", PathRequirement::FileOrDirectory)?
            else {
                return Ok(None);
            };
            path
        };
        if path.is_file() && !extension_is(&path, "json") {
            eprintln!("[错误] 文件不是 JSON：'{}'", path.display());
            continue;
        }
        match (
            plan_translation_fonts(&path, dictionary),
            required_render_targets(&path, dictionary),
        ) {
            (Ok(plan), Ok(targets)) => break (path, plan, targets),
            (Err(error), _) | (_, Err(error)) => {
                eprintln!("[错误] 无法准备字库重建：{error}");
            }
        }
    };

    let Some((main_font, donors)) = prompt_redraw_fonts(&targets)? else {
        return Ok(None);
    };
    let existing_outputs = plan.output_dirs.iter().filter(|path| path.exists()).count();
    println!();
    println!("执行前检查");
    println!("  JSON：{}", plan.json_input.display());
    println!("  源目录：{}", plan.source_root.display());
    println!("  JSON 条目：{} 条", plan.json_entries);
    println!("  字体槽位冲突：0（已检查）");
    println!("  实际重绘字形：{} 个", plan.required_glyphs);
    println!("  BIN/PNG 字库：{} 套", plan.font_pairs);
    println!("  主字体：{}", main_font.display());
    println!("  补字字体：{} 个", donors.len());
    for output in &plan.output_dirs {
        let state = if output.exists() {
            "（将覆盖）"
        } else {
            ""
        };
        println!("  输出：{}{}", output.display(), state);
    }
    let confirmed = if existing_outputs > 0 {
        prompt_confirmation("已有字库 chs 输出，确认覆盖并执行？(y/N，0 返回): ", false)?
    } else {
        prompt_confirmation("确认开始重建 BIN/PNG？(Y/n，0 返回): ", true)?
    };
    if !confirmed {
        return Ok(None);
    }

    let mut progress = |event: TranslationBuildProgress| match event {
        TranslationBuildProgress::InjectingScripts => {}
        TranslationBuildProgress::RenderingFont {
            current,
            total,
            input_png,
        } => {
            let name = input_png
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            println!("[字体] {current}/{total}  {name}");
        }
        TranslationBuildProgress::Finalizing => {
            println!("[字体] 正在完成 BIN/PNG 输出...");
        }
    };
    let report = build_translation_fonts_with_progress(
        &json_input,
        &main_font,
        &donors,
        existing_outputs > 0,
        dictionary,
        &mut progress,
    )?;
    println!();
    println!("字库重建完成");
    println!("  字库：{} 套", report.font_pairs);
    println!("  实际重绘字形：{} 个", report.required_glyphs);
    println!("  重绘槽位：{} 个", report.rendered_slots);
    for output in &report.output_dirs {
        println!("  输出目录：{}", output.display());
    }
    println!("下一步：如需同时注入脚本，请返回主菜单选择 2。");
    Ok(Some(format!(
        "已重建 {} 套 BIN/PNG 字库",
        report.font_pairs
    )))
}

fn prompt_redraw_fonts(targets: &BTreeSet<char>) -> ToolResult<Option<(PathBuf, Vec<PathBuf>)>> {
    let (main_font, mut remaining) = loop {
        println!();
        println!("请输入用于统一重绘最终译文字形的主 TTF。");
        println!("输入 0 返回主菜单。");
        let Some(font) = prompt_existing_path("主重绘字体 TTF: ", PathRequirement::File)?
        else {
            return Ok(None);
        };
        match missing_ttf_glyphs(&font, targets) {
            Ok(missing) => break (font, missing.into_iter().collect::<BTreeSet<_>>()),
            Err(error) => eprintln!("[错误] 无法使用该字体：{error}"),
        }
    };

    let mut donors = Vec::new();
    while !remaining.is_empty() {
        println!();
        println!(
            "当前仍缺 {} 个实际使用字形：{}",
            remaining.len(),
            format_characters(&remaining)
        );
        println!("请输入补字 TTF；可连续添加，输入 0 取消本次构建。");
        let Some(donor) = prompt_existing_path("补字字体 TTF: ", PathRequirement::File)? else {
            return Ok(None);
        };
        match missing_ttf_glyphs(&donor, &remaining) {
            Ok(missing) => {
                let donor_missing = missing.into_iter().collect::<BTreeSet<_>>();
                if donor_missing.len() == remaining.len() {
                    println!("[提示] 该字体没有补上当前缺字，请换一个字体。");
                } else {
                    println!(
                        "[提示] 已补充 {} 个，仍缺 {} 个。",
                        remaining.len() - donor_missing.len(),
                        donor_missing.len()
                    );
                }
                remaining = donor_missing;
                donors.push(donor);
            }
            Err(error) => eprintln!("[错误] 无法使用该字体：{error}"),
        }
    }
    Ok(Some((main_font, donors)))
}

fn interactive_archive() -> InteractiveResult {
    let mut last_notice = None;
    loop {
        clear_screen()?;
        println!("仅解包 / 封包 / 验证");
        println!("====================");
        if let Some(notice) = &last_notice {
            println!("{notice}");
            println!();
        }
        println!("1  解包 MPK");
        println!("2  封包目录");
        println!("3  验证文件或目录");
        println!("0  返回主菜单");
        println!();
        println!("也可以直接拖入 MPK、解包目录或脚本目录。");
        let choice = prompt_line("请选择操作或粘贴路径: ")?;
        let result = match choice.trim() {
            "1" => interactive_unpack(None),
            "2" => interactive_pack(None),
            "3" => interactive_verify(None),
            "0" => return Ok(None),
            _ => {
                let path = clean_input_path(&choice);
                if !path.exists() {
                    Err(ToolError(format!("路径不存在：'{}'", path.display())))
                } else if path.is_file() && extension_is(&path, "mpk") {
                    println!("已识别为 MPK，进入解包流程。");
                    interactive_unpack(Some(path))
                } else if path.is_dir() && path.join(MANIFEST_FILE).is_file() {
                    println!("已识别为 MPK 解包目录，进入封包流程。");
                    interactive_pack(Some(path))
                } else {
                    println!("已识别为验证输入，进入验证流程。");
                    interactive_verify(Some(path))
                }
            }
        };
        match result {
            Ok(Some(summary)) => {
                last_notice = Some(format!("[完成] {summary}"));
                pause_for_menu()?;
            }
            Ok(None) => {}
            Err(error) => {
                eprintln!("[错误] {error}");
                last_notice = Some(format!("[上次错误] {error}"));
                pause_for_menu()?;
            }
        }
    }
}

fn interactive_unpack(mut initial: Option<PathBuf>) -> InteractiveResult {
    let input = loop {
        let path = if let Some(path) = initial.take() {
            path
        } else {
            println!();
            println!("解包 MPK");
            println!("可拖入 .mpk 文件，输入 0 返回上一级。");
            let Some(path) = prompt_existing_path("MPK 文件: ", PathRequirement::File)? else {
                return Ok(None);
            };
            path
        };
        if !extension_is(&path, "mpk") {
            eprintln!("[错误] 请选择 .mpk 文件。");
            continue;
        }
        break path;
    };
    let archive = parse_archive(&input)?;
    let default_output = default_unpack_output(&input)?;
    println!("默认输出：{}", default_output.display());
    let Some(output_choice) = prompt_output_path(
        "自定义输出目录（直接 Enter 使用默认路径，0 返回）: ",
        &default_output,
    )?
    else {
        return Ok(None);
    };
    let output = output_choice.unwrap_or(default_output);
    println!();
    println!("执行前检查");
    println!("  MPK：{}", input.display());
    println!("  文件数：{}", archive.entries.len());
    println!("  输出：{}", output.display());
    if !prompt_confirmation("确认开始解包？(Y/n，0 返回): ", true)? {
        return Ok(None);
    }
    println!("[解包] 正在写出 {} 个文件...", archive.entries.len());
    let report = unpack_archive(&input, Some(&output))?;
    for warning in &report.warning_messages {
        println!("[警告] {warning}");
    }
    println!();
    println!("解包完成");
    println!("  文件数：{}", report.extracted_files);
    println!("  输出目录：{}", report.output.display());
    println!("下一步：完成修改后，将这个目录拖入工具进行封包。");
    Ok(Some(format!(
        "已解包 {} 个文件到 {}",
        report.extracted_files,
        report.output.display()
    )))
}

fn interactive_pack(mut initial: Option<PathBuf>) -> InteractiveResult {
    let input = loop {
        let path = if let Some(path) = initial.take() {
            path
        } else {
            println!();
            println!("封包目录");
            println!("请选择含 .mpk-manifest.json 的解包目录，输入 0 返回上一级。");
            let Some(path) = prompt_existing_path("解包目录: ", PathRequirement::Directory)?
            else {
                return Ok(None);
            };
            path
        };
        if !path.join(MANIFEST_FILE).is_file() {
            eprintln!("[错误] 目录缺少 {}：'{}'", MANIFEST_FILE, path.display());
            continue;
        }
        break path;
    };
    let default_output = default_pack_output(&input)?;
    println!("默认输出：{}", default_output.display());
    let Some(output_choice) = prompt_output_path(
        "自定义输出 MPK（直接 Enter 使用默认路径，0 返回）: ",
        &default_output,
    )?
    else {
        return Ok(None);
    };
    let output = output_choice.unwrap_or(default_output);
    let files = count_regular_files(&input)?;
    println!();
    println!("执行前检查");
    println!("  解包目录：{}", input.display());
    println!("  目录文件：{} 个", files);
    println!("  输出 MPK：{}", output.display());
    if !prompt_confirmation("确认开始封包？(Y/n，0 返回): ", true)? {
        return Ok(None);
    }
    println!("[封包] 正在重建文件表和数据区...");
    let report = pack_archive(&input, Some(&output))?;
    println!();
    println!("封包完成");
    println!("  文件数：{}", report.packed_files);
    println!("  输出大小：{} 字节", report.output_bytes);
    println!("  输出文件：{}", report.output.display());
    println!("下一步：选择 3 验证新 MPK，确认文件表和边界可解析。");
    Ok(Some(format!(
        "已封包 {} 个文件到 {}",
        report.packed_files,
        report.output.display()
    )))
}

fn interactive_verify(mut initial: Option<PathBuf>) -> InteractiveResult {
    let input = if let Some(path) = initial.take() {
        path
    } else {
        println!();
        println!("验证文件或目录");
        println!("支持 MPK、MSB、SCX 或脚本目录，输入 0 返回上一级。");
        let Some(path) = prompt_existing_path("验证输入: ", PathRequirement::FileOrDirectory)?
        else {
            return Ok(None);
        };
        path
    };
    let description = if input.is_file() && extension_is(&input, "mpk") {
        let archive = parse_archive(&input)?;
        format!("MPK，{} 个成员", archive.entries.len())
    } else if input.is_file() {
        "单个脚本文件".to_string()
    } else {
        let mut files = Vec::new();
        collect_script_files(&input, &mut files)?;
        format!("脚本目录，{} 个 MSB/SCX", files.len())
    };
    println!();
    println!("执行前检查");
    println!("  输入：{}", input.display());
    println!("  类型：{description}");
    if !prompt_confirmation("确认开始验证？(Y/n，0 返回): ", true)? {
        return Ok(None);
    }
    println!("[验证] 正在执行 byte-exact 结构回环...");
    run_verify(std::slice::from_ref(&input))?;
    println!();
    println!("验证完成：未发现结构回环错误。");
    Ok(Some(format!("验证通过：{}", input.display())))
}

fn prompt_line(prompt: &str) -> ToolResult<String> {
    print!("{prompt}");
    io::stdout()
        .flush()
        .map_err(|error| ToolError(format!("cannot flush prompt: {error}")))?;
    let mut line = String::new();
    let read = io::stdin()
        .read_line(&mut line)
        .map_err(|error| ToolError(format!("cannot read input: {error}")))?;
    if read == 0 {
        return Err(ToolError("输入流已经关闭".to_string()));
    }
    Ok(line.trim().to_string())
}

fn prompt_existing_path(prompt: &str, requirement: PathRequirement) -> ToolResult<Option<PathBuf>> {
    loop {
        let value = prompt_line(prompt)?;
        if value == "0" {
            return Ok(None);
        }
        if value.is_empty() {
            eprintln!("[错误] 路径不能为空。请输入路径，或输入 0 返回。");
            continue;
        }
        let path = clean_input_path(&value);
        if !path.exists() {
            eprintln!("[错误] 路径不存在：'{}'", path.display());
            continue;
        }
        let valid_type = match requirement {
            PathRequirement::File => path.is_file(),
            PathRequirement::Directory => path.is_dir(),
            PathRequirement::FileOrDirectory => path.is_file() || path.is_dir(),
        };
        if !valid_type {
            let expected = match requirement {
                PathRequirement::File => "文件",
                PathRequirement::Directory => "目录",
                PathRequirement::FileOrDirectory => "文件或目录",
            };
            eprintln!("[错误] 这里需要{expected}：'{}'", path.display());
            continue;
        }
        return Ok(Some(path));
    }
}

fn prompt_output_path(prompt: &str, default: &Path) -> ToolResult<Option<Option<PathBuf>>> {
    loop {
        let value = prompt_line(prompt)?;
        if value == "0" {
            return Ok(None);
        }
        let path = if value.is_empty() {
            default.to_path_buf()
        } else {
            clean_input_path(&value)
        };
        if path.exists() {
            eprintln!("[错误] 输出已经存在，请换一个路径：'{}'", path.display());
            continue;
        }
        let parent = path
            .parent()
            .filter(|value| !value.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        if !parent.is_dir() {
            eprintln!("[错误] 输出的上级目录不存在：'{}'", parent.display());
            continue;
        }
        return Ok(Some(if value.is_empty() { None } else { Some(path) }));
    }
}

fn prompt_confirmation(prompt: &str, default_yes: bool) -> ToolResult<bool> {
    loop {
        let value = prompt_line(prompt)?.to_ascii_lowercase();
        match value.as_str() {
            "" => return Ok(default_yes),
            "y" | "yes" | "是" => return Ok(true),
            "n" | "no" | "否" | "0" => return Ok(false),
            _ => println!("请输入 y 或 n；直接 Enter 使用默认选项，0 取消。"),
        }
    }
}

fn pause_for_menu() -> ToolResult<()> {
    let _ = prompt_line("\n按 Enter 返回菜单...")?;
    Ok(())
}

fn clear_screen() -> ToolResult<()> {
    print!("\x1B[2J\x1B[H");
    io::stdout()
        .flush()
        .map_err(|error| ToolError(format!("cannot clear screen: {error}")))
}

fn clean_input_path(value: &str) -> PathBuf {
    let trimmed = value.trim();
    if trimmed.len() >= 2
        && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
    {
        PathBuf::from(&trimmed[1..trimmed.len() - 1])
    } else {
        PathBuf::from(trimmed)
    }
}

fn extension_is(path: &Path, extension: &str) -> bool {
    path.extension()
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

fn format_characters(characters: &BTreeSet<char>) -> String {
    characters
        .iter()
        .map(|character| format!("U+{:04X} {:?}", *character as u32, character))
        .collect::<Vec<_>>()
        .join(", ")
}

fn scan_resource_counts(path: &Path) -> ToolResult<ResourceCounts> {
    let mut counts = ResourceCounts::default();
    if path.is_file() {
        if extension_is(path, "msb") {
            counts.msb = 1;
        } else if extension_is(path, "scx") {
            counts.scx = 1;
        }
        return Ok(counts);
    }
    for entry in fs::read_dir(path)
        .map_err(|error| ToolError(format!("cannot list '{}': {error}", path.display())))?
    {
        let entry = entry.map_err(|error| {
            ToolError(format!(
                "cannot read directory entry in '{}': {error}",
                path.display()
            ))
        })?;
        let child = entry.path();
        if child.is_dir() {
            let child_counts = scan_resource_counts(&child)?;
            counts.msb += child_counts.msb;
            counts.scx += child_counts.scx;
            counts.font_pairs += child_counts.font_pairs;
        } else if extension_is(&child, "msb") {
            counts.msb += 1;
        } else if extension_is(&child, "scx") {
            counts.scx += 1;
        } else if child
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| {
                name.eq_ignore_ascii_case("font_df_jpn.bin")
                    || name.eq_ignore_ascii_case("font2_df_jpn.bin")
            })
            && child.with_extension("png").is_file()
        {
            counts.font_pairs += 1;
        }
    }
    Ok(counts)
}

fn contains_extension(path: &Path, extension: &str) -> ToolResult<bool> {
    for entry in fs::read_dir(path)
        .map_err(|error| ToolError(format!("cannot list '{}': {error}", path.display())))?
    {
        let entry = entry.map_err(|error| {
            ToolError(format!(
                "cannot read directory entry in '{}': {error}",
                path.display()
            ))
        })?;
        let child = entry.path();
        if child.is_dir() {
            if contains_extension(&child, extension)? {
                return Ok(true);
            }
        } else if extension_is(&child, extension) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn count_regular_files(path: &Path) -> ToolResult<usize> {
    let mut count = 0;
    for entry in fs::read_dir(path)
        .map_err(|error| ToolError(format!("cannot list '{}': {error}", path.display())))?
    {
        let entry = entry.map_err(|error| {
            ToolError(format!(
                "cannot read directory entry in '{}': {error}",
                path.display()
            ))
        })?;
        let child = entry.path();
        if child.is_dir() {
            count += count_regular_files(&child)?;
        } else {
            count += 1;
        }
    }
    Ok(count)
}

fn run_prepare(args: &[PathBuf]) -> ToolResult<()> {
    let (inputs, output) = parse_output(args)?;
    if inputs.len() != 1 {
        return Err(ToolError(
            "prepare accepts exactly one directory containing mes00/script/system_win MPKs"
                .to_string(),
        ));
    }
    let dictionary = merry_mpk_tool::glyph::GlyphDictionary::built_in()?;
    let mut progress = |event: TranslationWorkspaceProgress| match event {
        TranslationWorkspaceProgress::Unpacking {
            current,
            total,
            archive,
        } => println!(
            "[prepare] unpack={current}/{total} archive={}",
            archive.display()
        ),
        TranslationWorkspaceProgress::Extracting {
            current,
            total,
            script,
        } => println!(
            "[prepare] extract={current}/{total} script={}",
            script.display()
        ),
        TranslationWorkspaceProgress::WritingTranslationView => {
            println!("[prepare] writing_translation_json=1")
        }
        TranslationWorkspaceProgress::Finalizing => println!("[prepare] finalizing=1"),
    };
    let report = prepare_translation_workspace_with_progress(
        &inputs[0],
        output.as_deref(),
        &dictionary,
        &mut progress,
    )?;
    println!(
        "[prepare] package_directories={} archives={} extracted_members={} scanned_scripts={} json_files={} translation_entries={} warnings={} output={} translation_json={}",
        report.package_directories,
        report.archives,
        report.extracted_members,
        report.scanned_scripts,
        report.json_files,
        report.translation_entries,
        report.warnings.len(),
        report.output.display(),
        report.translation_json.display()
    );
    for warning in report.warnings {
        println!("[prepare] warning={warning}");
    }
    Ok(())
}

fn run_unpack(args: &[PathBuf]) -> ToolResult<()> {
    let (inputs, output) = parse_output(args)?;
    if inputs.is_empty() {
        return Err(ToolError(
            "unpack requires at least one MPK input".to_string(),
        ));
    }
    if output.is_some() && inputs.len() != 1 {
        return Err(ToolError(
            "--output can only be used with one unpack input".to_string(),
        ));
    }
    for input in inputs {
        let report = unpack_archive(&input, output.as_deref())?;
        println!(
            "[unpack] input={} extracted_files={} warnings={} output={}",
            report.input.display(),
            report.extracted_files,
            report.warnings,
            report.output.display()
        );
        for warning in report.warning_messages {
            println!("[unpack] warning={warning}");
        }
    }
    Ok(())
}

fn run_pack(args: &[PathBuf]) -> ToolResult<()> {
    let (inputs, output) = parse_output(args)?;
    if inputs.len() != 1 {
        return Err(ToolError(
            "pack accepts exactly one unpacked directory".to_string(),
        ));
    }
    let report = pack_archive(&inputs[0], output.as_deref())?;
    println!(
        "[pack] input={} packed_files={} output_bytes={} warnings={} output={}",
        report.input.display(),
        report.packed_files,
        report.output_bytes,
        report.warnings,
        report.output.display()
    );
    Ok(())
}

fn run_extract(args: &[PathBuf]) -> ToolResult<()> {
    let (inputs, output) = parse_output(args)?;
    if inputs.len() != 1 {
        return Err(ToolError(
            "extract accepts exactly one script file or directory".to_string(),
        ));
    }
    let dictionary = merry_mpk_tool::glyph::GlyphDictionary::built_in()?;
    let report = extract_path(&inputs[0], output.as_deref(), &dictionary)?;
    println!(
        "[extract] input={} scanned_files={} json_files={} extracted_entries={} warnings={} output={}",
        report.input.display(), report.scanned_files, report.json_files, report.extracted_entries,
        report.warnings.len(), report.output.display()
    );
    for warning in report.warnings {
        println!("[extract] warning={warning}");
    }
    Ok(())
}

fn run_inject(args: &[PathBuf]) -> ToolResult<()> {
    let (inputs, output) = parse_output(args)?;
    if inputs.len() != 2 {
        return Err(ToolError(
            "inject requires SCRIPT_FILE_OR_DIR and TRANSLATION_JSON_OR_DIR".to_string(),
        ));
    }
    let dictionary = merry_mpk_tool::glyph::GlyphDictionary::built_in()?;
    let report = inject_path(&inputs[0], &inputs[1], output.as_deref(), &dictionary)?;
    println!(
        "[inject] input={} json_entries={} patched={} unchanged={} warnings={} output={}",
        report.input.display(),
        report.json_entries,
        report.patched,
        report.unchanged,
        report.warnings.len(),
        report.output.display()
    );
    for warning in report.warnings {
        println!("[inject] warning={warning}");
    }
    Ok(())
}

fn run_font_build(args: &[PathBuf]) -> ToolResult<()> {
    let mut positional = Vec::new();
    let mut bin = None;
    let mut png = None;
    let mut font = None;
    let mut donors = Vec::new();
    let mut mapping = None;
    let mut output_dir = None;
    let mut all = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].to_string_lossy().as_ref() {
            "--bin" => bin = Some(next_path(args, &mut index, "--bin")?),
            "--png" => png = Some(next_path(args, &mut index, "--png")?),
            "--font" => font = Some(next_path(args, &mut index, "--font")?),
            "--donor" => donors.push(next_path(args, &mut index, "--donor")?),
            "--mapping" => mapping = Some(next_path(args, &mut index, "--mapping")?),
            "--output-dir" | "-o" => {
                output_dir = Some(next_path(args, &mut index, "--output-dir")?)
            }
            "--all" => all = true,
            value if value.starts_with('-') => {
                return Err(ToolError(format!("unknown font-build option '{value}'")))
            }
            _ => positional.push(args[index].clone()),
        }
        index += 1;
    }
    let input = positional
        .first()
        .ok_or_else(|| ToolError("font-build requires FONT_DIR_OR_BIN".to_string()))?;
    let font = font.ok_or_else(|| ToolError("font-build requires --font TTF".to_string()))?;
    let (bin, png) = match (bin, png) {
        (Some(bin), Some(png)) => (bin, png),
        (None, None) if input.is_dir() => {
            (input.join("font_df_jpn.bin"), input.join("font_df_jpn.png"))
        }
        _ => {
            return Err(ToolError(
                "font-build requires both --bin and --png when input is not a font directory"
                    .to_string(),
            ))
        }
    };
    let output_dir = output_dir.unwrap_or_else(|| {
        if input.is_dir() {
            input.join("chs")
        } else {
            input.parent().unwrap_or_else(|| Path::new(".")).join("chs")
        }
    });
    if output_dir.exists() {
        return Err(ToolError(format!(
            "output directory already exists: '{}'",
            output_dir.display()
        )));
    }
    fs::create_dir_all(&output_dir)
        .map_err(|error| ToolError(format!("cannot create '{}': {error}", output_dir.display())))?;
    let output_bin = output_dir.join(
        bin.file_name()
            .ok_or_else(|| ToolError("invalid BIN filename".to_string()))?,
    );
    let output_png = output_dir.join(
        png.file_name()
            .ok_or_else(|| ToolError("invalid PNG filename".to_string()))?,
    );
    let operation = (|| -> ToolResult<()> {
        let report = build_font_pair(
            &bin,
            &png,
            &font,
            &donors,
            &output_bin,
            &output_png,
            mapping.as_deref(),
        )?;
        println!(
            "[font-build] bin={} png={} rendered_slots={} mapping_entries={} donor_fonts={} donor_fonts_used={} output_dir={}",
            report.output_bin.display(),
            report.output_png.display(),
            report.rendered_slots,
            report.mapping_entries,
            report.donor_fonts,
            report.donor_fonts_used,
            output_dir.display()
        );
        if all && input.is_dir() {
            let bin2 = input.join("font2_df_jpn.bin");
            let png2 = input.join("font2_df_jpn.png");
            if bin2.is_file() && png2.is_file() {
                let output_bin2 = output_dir.join("font2_df_jpn.bin");
                let output_png2 = output_dir.join("font2_df_jpn.png");
                let report2 = build_font_pair(
                    &bin2,
                    &png2,
                    &font,
                    &donors,
                    &output_bin2,
                    &output_png2,
                    mapping.as_deref(),
                )?;
                println!(
                    "[font-build] bin={} png={} rendered_slots={} mapping_entries={} donor_fonts={} donor_fonts_used={}",
                    report2.output_bin.display(),
                    report2.output_png.display(),
                    report2.rendered_slots,
                    report2.mapping_entries,
                    report2.donor_fonts,
                    report2.donor_fonts_used
                );
            } else {
                println!("[font-build] warning=font2 pair not found; skipped");
            }
        }
        Ok(())
    })();
    if let Err(error) = operation {
        let _ = fs::remove_dir_all(&output_dir);
        return Err(error);
    }
    Ok(())
}

fn run_verify(args: &[PathBuf]) -> ToolResult<()> {
    if args.len() != 1 {
        return Err(ToolError(
            "verify accepts exactly one file or directory".to_string(),
        ));
    }
    let input = &args[0];
    if input.is_file() {
        verify_file(input)?;
        println!("[verify] files=1 byte_exact=1 input={}", input.display());
        return Ok(());
    }
    if !input.is_dir() {
        return Err(ToolError(format!(
            "verify input '{}' does not exist",
            input.display()
        )));
    }
    let mut files = Vec::new();
    collect_script_files(input, &mut files)?;
    files.sort();
    for file in &files {
        verify_file(file)?;
    }
    println!(
        "[verify] files={} byte_exact={} input={}",
        files.len(),
        files.len(),
        input.display()
    );
    Ok(())
}

fn verify_file(path: &Path) -> ToolResult<()> {
    let data = fs::read(path)
        .map_err(|error| ToolError(format!("cannot read '{}': {error}", path.display())))?;
    if data.starts_with(b"MES\0") {
        let parsed = parse_msb(&data)?;
        let rebuilt = rebuild_msb(&parsed)?;
        if rebuilt != data {
            return Err(ToolError(format!(
                "MSB byte-exact round trip failed for '{}'",
                path.display()
            )));
        }
    } else if data.starts_with(b"SC3\0") {
        let parsed = parse_scx(&data)?;
        let rebuilt = rebuild_scx(&parsed, &HashMap::new())?;
        if rebuilt != data {
            return Err(ToolError(format!(
                "SCX byte-exact round trip failed for '{}'",
                path.display()
            )));
        }
    } else if path
        .extension()
        .is_some_and(|value| value.eq_ignore_ascii_case("mpk"))
    {
        let archive = parse_archive(path)?;
        println!(
            "[verify] archive_entries={} archive_bytes={}",
            archive.entries.len(),
            archive.file_len
        );
    } else {
        return Err(ToolError(format!(
            "unsupported verify input '{}'",
            path.display()
        )));
    }
    Ok(())
}

fn collect_script_files(root: &Path, output: &mut Vec<PathBuf>) -> ToolResult<()> {
    for entry in fs::read_dir(root)
        .map_err(|error| ToolError(format!("cannot list '{}': {error}", root.display())))?
    {
        let entry = entry.map_err(|error| {
            ToolError(format!(
                "cannot read directory entry in '{}': {error}",
                root.display()
            ))
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_script_files(&path, output)?;
        } else if path.extension().is_some_and(|value| {
            value.eq_ignore_ascii_case("msb") || value.eq_ignore_ascii_case("scx")
        }) {
            output.push(path);
        }
    }
    Ok(())
}

fn run_inferred(args: &[PathBuf]) -> ToolResult<()> {
    if args.iter().all(|input| {
        input
            .extension()
            .is_some_and(|value| value.eq_ignore_ascii_case("mpk"))
    }) {
        return run_unpack(args);
    }
    if args.len() == 1 && args[0].is_dir() {
        if plan_translation_workspace(&args[0]).is_ok() {
            return run_prepare(args);
        }
        return run_pack(args);
    }
    Err(ToolError(
        "cannot infer operation; use prepare, unpack, pack, extract, inject, font-build, or verify"
            .to_string(),
    ))
}

fn parse_output(args: &[PathBuf]) -> ToolResult<(Vec<PathBuf>, Option<PathBuf>)> {
    let mut inputs = Vec::new();
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        if args[index].to_string_lossy() == "--output" || args[index].to_string_lossy() == "-o" {
            index += 1;
            if index >= args.len() {
                return Err(ToolError("--output requires a path".to_string()));
            }
            output = Some(args[index].clone());
        } else if args[index].to_string_lossy().starts_with('-') {
            return Err(ToolError(format!(
                "unknown option '{}'; use --help",
                args[index].display()
            )));
        } else {
            inputs.push(args[index].clone());
        }
        index += 1;
    }
    Ok((inputs, output))
}

fn next_path(args: &[PathBuf], index: &mut usize, option: &str) -> ToolResult<PathBuf> {
    *index += 1;
    if *index >= args.len() {
        return Err(ToolError(format!("{option} requires a path")));
    }
    Ok(args[*index].clone())
}
