mod font;
mod mes;
mod resources;

use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Extract,
    Pack,
}

impl Action {
    fn label(self) -> &'static str {
        match self {
            Self::Extract => "提取 ELF-DOS 游戏资源",
            Self::Pack => "重新封装 D88 磁盘",
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        eprintln!("use --help for usage");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }
    if args.is_empty() {
        return run_menu(None);
    }
    if args.len() == 1
        && !args[0].starts_with('-')
        && !matches!(
            args[0].as_str(),
            "extract" | "pack" | "mes-extract" | "mes-inject"
        )
    {
        return run_menu(Some(PathBuf::from(&args[0])));
    }

    match args.first().map(String::as_str) {
        Some("mes-extract") => return parse_mes_extract(&args),
        Some("mes-inject") => return parse_mes_inject(&args),
        _ => {}
    }
    let action = match args.first().map(String::as_str) {
        Some("extract") => Action::Extract,
        Some("pack") => Action::Pack,
        _ => return Err("unknown command; use extract, pack, or an interactive path".to_string()),
    };
    let mut input = None;
    let mut output = None;
    let mut overwrite = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--input" | "-i" => {
                index += 1;
                input = Some(PathBuf::from(
                    args.get(index).ok_or("--input requires a path")?,
                ));
            }
            "--output" | "-o" => {
                index += 1;
                output = Some(PathBuf::from(
                    args.get(index).ok_or("--output requires a path")?,
                ));
            }
            "--overwrite" => overwrite = true,
            option if option.starts_with('-') => return Err(format!("unknown option: {option}")),
            value => return Err(format!("unexpected argument: {value}")),
        }
        index += 1;
    }
    let input = input.ok_or_else(|| format!("{} requires --input", action.label()))?;
    let output = output.ok_or_else(|| format!("{} requires --output", action.label()))?;
    run_action(action, &input, &output, overwrite)
}

fn run_action(action: Action, input: &Path, output: &Path, overwrite: bool) -> Result<()> {
    match action {
        Action::Extract => resources::extract_inputs(input, output, overwrite),
        Action::Pack => resources::pack_inputs(input, output, overwrite),
    }
}

fn parse_mes_extract(args: &[String]) -> Result<()> {
    let mut input = None;
    let mut output = None;
    let mut overwrite = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--input" | "-i" => {
                index += 1;
                input = Some(PathBuf::from(
                    args.get(index).ok_or("--input requires a path")?,
                ));
            }
            "--output" | "-o" => {
                index += 1;
                output = Some(PathBuf::from(
                    args.get(index).ok_or("--output requires a path")?,
                ));
            }
            "--overwrite" => overwrite = true,
            option if option.starts_with('-') => return Err(format!("unknown option: {option}")),
            value => return Err(format!("unexpected argument: {value}")),
        }
        index += 1;
    }
    mes::mes_extract_inputs(
        &input.ok_or("mes-extract requires --input")?,
        &output.ok_or("mes-extract requires --output")?,
        overwrite,
    )
}

fn parse_mes_inject(args: &[String]) -> Result<()> {
    let mut input = None;
    let mut translations = None;
    let mut output = None;
    let mut font_source = None;
    let mut subs = None;
    let mut font_output = None;
    let mut face = "新宋体".to_string();
    let mut overwrite = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--input" | "-i" => {
                index += 1;
                input = Some(PathBuf::from(
                    args.get(index).ok_or("--input requires a path")?,
                ));
            }
            "--translation" | "-t" => {
                index += 1;
                translations = Some(PathBuf::from(
                    args.get(index).ok_or("--translation requires a path")?,
                ));
            }
            "--output" | "-o" => {
                index += 1;
                output = Some(PathBuf::from(
                    args.get(index).ok_or("--output requires a path")?,
                ));
            }
            "--font" => {
                index += 1;
                font_source = Some(PathBuf::from(
                    args.get(index).ok_or("--font requires a path")?,
                ));
            }
            "--subs" => {
                index += 1;
                subs = Some(PathBuf::from(
                    args.get(index).ok_or("--subs requires a path")?,
                ));
            }
            "--font-output" => {
                index += 1;
                font_output = Some(PathBuf::from(
                    args.get(index).ok_or("--font-output requires a path")?,
                ));
            }
            "--font-face" => {
                index += 1;
                face = args
                    .get(index)
                    .ok_or("--font-face requires a value")?
                    .clone();
            }
            "--overwrite" => overwrite = true,
            option if option.starts_with('-') => return Err(format!("unknown option: {option}")),
            value => return Err(format!("unexpected argument: {value}")),
        }
        index += 1;
    }
    mes::mes_inject_inputs(
        &input.ok_or("mes-inject requires --input")?,
        &translations.ok_or("mes-inject requires --translation")?,
        &output.ok_or("mes-inject requires --output")?,
        &font_source.ok_or("mes-inject requires --font")?,
        &subs.ok_or("mes-inject requires --subs")?,
        &font_output.ok_or("mes-inject requires --font-output")?,
        &face,
        overwrite,
    )
}

fn print_help() {
    println!("FOXY 2 D88 resource extractor/packer");
    println!();
    println!("Interactive mode (no arguments):");
    println!("  foxy2_d88_splitter");
    println!("  foxy2_d88_splitter <D88-or-resource-path>  # editable input prefill");
    println!();
    println!("One-shot mode:");
    println!("  foxy2_d88_splitter extract --input <D88_FILE_OR_DIRECTORY> --output <RESOURCE_DIRECTORY> [--overwrite]");
    println!("  foxy2_d88_splitter pack --input <RESOURCE_DIRECTORY> --output <D88_DIRECTORY> [--overwrite]");
    println!("  foxy2_d88_splitter mes-extract --input <RESOURCE_DIRECTORY> --output <TRANSLATION_DIRECTORY> [--overwrite]");
    println!("  foxy2_d88_splitter mes-inject --input <RESOURCE_DIRECTORY> --translation <TRANSLATION_DIRECTORY> --output <RESOURCE_DIRECTORY> --font <FONT_TMP> --subs <SUBS_JSON> --font-output <FONT_TMP> [--font-face <FACE>] [--overwrite]");
    println!();
    println!("extract reads the confirmed ELF-DOS directory at logical offset 0xA0000.");
    println!("pack writes resource payloads and rebuilds affected ELF-DOS addresses/sizes; D88 and sector headers are preserved.");
}

fn run_menu(initial_input: Option<PathBuf>) -> Result<()> {
    println!("FOXY 2 D88 resource extractor/packer");
    println!("写入只会在最终确认后开始；完成、取消或可恢复错误后返回主菜单。输入 EOF 可退出。\n");
    if let Some(input) = initial_input {
        run_interactive_job(detect_action(&input), Some(input))?;
    }
    loop {
        println!("主菜单");
        println!("  1  提取 ELF-DOS 游戏资源");
        println!("  2  重新封装 D88 磁盘");
        println!("  3  提取 MES 文本为 UTF-8 JSON");
        println!("  4  注回 MES 文本并重绘字体");
        println!("  0  退出");
        let Some(choice) = prompt_line("请选择 [0]: ")? else {
            return Ok(());
        };
        match choice.trim() {
            "" | "0" => return Ok(()),
            "1" => run_interactive_job(Action::Extract, None)?,
            "2" => run_interactive_job(Action::Pack, None)?,
            "3" => run_interactive_mes_extract()?,
            "4" => run_interactive_mes_inject()?,
            _ => println!("无效选择，请输入 1、2、3、4 或 0。"),
        }
    }
}

fn run_interactive_mes_extract() -> Result<()> {
    let Some(input) = prompt_path("输入资源工作目录", None)? else {
        return Ok(());
    };
    let base = input
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("resources");
    let default_output = input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{base}_translations"));
    let Some(output) = prompt_path("输出翻译目录", Some(&default_output))? else {
        return Ok(());
    };
    let Some(overwrite) = prompt_yes_no("允许覆盖已有输出", false)? else {
        return Ok(());
    };
    println!(
        "操作: MES 提取\n输入: {}\n输出: {}\noverwrite: {}",
        input.display(),
        output.display(),
        overwrite
    );
    if prompt_yes_no("确认开始写入", false)? == Some(true) {
        match mes::mes_extract_inputs(&input, &output, overwrite) {
            Ok(()) => println!("MES 提取完成。"),
            Err(error) => println!("操作失败：{error}"),
        }
    }
    let _ = prompt_line("按 Enter 返回主菜单……")?;
    Ok(())
}

fn run_interactive_mes_inject() -> Result<()> {
    let Some(input) = prompt_path("输入资源工作目录", None)? else {
        return Ok(());
    };
    let Some(translations) = prompt_path("输入翻译 JSON 目录", None)? else {
        return Ok(());
    };
    let base = input
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("resources");
    let default_output = input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{base}_translated"));
    let Some(output) = prompt_path("输出资源工作目录", Some(&default_output))? else {
        return Ok(());
    };
    let Some(font_source) = prompt_path("输入 font.tmp", None)? else {
        return Ok(());
    };
    let Some(subs) = prompt_path("输入 subs_cn_jp.json", None)? else {
        return Ok(());
    };
    let output_base = output
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("translated_resources");
    let default_font_output = output
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{output_base}_font.tmp"));
    let Some(font_output) = prompt_path("输出重绘字体", Some(&default_font_output))? else {
        return Ok(());
    };
    let face = prompt_line("字体名称 [新宋体]: ")?
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "新宋体".to_string());
    let Some(overwrite) = prompt_yes_no("允许覆盖已有输出", false)? else {
        return Ok(());
    };
    println!("操作: MES 注回与字体重绘\n输入: {}\n翻译: {}\n资源输出: {}\n字体: {}\n字体输出: {}\nsubs: {}\noverwrite: {}", input.display(), translations.display(), output.display(), font_source.display(), font_output.display(), subs.display(), overwrite);
    if prompt_yes_no("确认开始写入", false)? == Some(true) {
        match mes::mes_inject_inputs(
            &input,
            &translations,
            &output,
            &font_source,
            &subs,
            &font_output,
            &face,
            overwrite,
        ) {
            Ok(()) => println!("MES 注回完成。"),
            Err(error) => println!("操作失败：{error}"),
        }
    }
    let _ = prompt_line("按 Enter 返回主菜单……")?;
    Ok(())
}

fn run_interactive_job(action: Action, input_prefill: Option<PathBuf>) -> Result<()> {
    let input_label = match action {
        Action::Extract => "输入 D88 文件或目录",
        Action::Pack => "输入资源工作目录",
    };
    let Some(input) = prompt_path(input_label, input_prefill.as_deref())? else {
        println!("已取消，返回主菜单。\n");
        return Ok(());
    };
    let default_output = suggested_output(action, &input);
    let Some(output) = prompt_path("输出目录", Some(&default_output))? else {
        println!("已取消，返回主菜单。\n");
        return Ok(());
    };
    let Some(overwrite) = prompt_yes_no("允许覆盖已有输出", false)? else {
        println!("已取消，返回主菜单。\n");
        return Ok(());
    };
    println!("\n请确认：");
    println!("  操作      : {}", action.label());
    println!("  输入      : {}", input.display());
    println!("  输出      : {}", output.display());
    println!("  overwrite : {overwrite}");
    if prompt_yes_no("确认开始写入", false)? != Some(true) {
        println!("已取消，未写入任何内容。返回主菜单。\n");
        return Ok(());
    }
    match run_action(action, &input, &output, overwrite) {
        Ok(()) => println!("{}完成。", action.label()),
        Err(error) => println!("操作失败：{error}"),
    }
    let _ = prompt_line("按 Enter 返回主菜单……")?;
    println!();
    Ok(())
}

fn suggested_output(action: Action, input: &Path) -> PathBuf {
    let base = if input.is_file() {
        input
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("d88")
            .to_string()
    } else {
        input
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("workspace")
            .to_string()
    };
    input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(match action {
            Action::Extract => format!("{base}_resources"),
            Action::Pack => format!("{base}_d88"),
        })
}

fn detect_action(input: &Path) -> Action {
    if input.is_file() {
        return Action::Extract;
    }
    if resources::is_resource_workspace(input) {
        Action::Pack
    } else {
        Action::Extract
    }
}

fn prompt_path(label: &str, default: Option<&Path>) -> Result<Option<PathBuf>> {
    let prompt = match default {
        Some(path) => format!("{label} [{}]: ", path.display()),
        None => format!("{label} [留空取消]: "),
    };
    let Some(line) = prompt_line(&prompt)? else {
        return Ok(None);
    };
    let value = trim_wrapping_quotes(line.trim());
    if value.is_empty() {
        return Ok(default.map(Path::to_path_buf));
    }
    Ok(Some(PathBuf::from(value)))
}

fn prompt_yes_no(label: &str, default: bool) -> Result<Option<bool>> {
    let suffix = if default { "Y/n" } else { "y/N" };
    loop {
        let Some(line) = prompt_line(&format!("{label} [{suffix}]: "))? else {
            return Ok(None);
        };
        match line.trim().to_ascii_lowercase().as_str() {
            "" => return Ok(Some(default)),
            "y" | "yes" | "是" => return Ok(Some(true)),
            "n" | "no" | "否" => return Ok(Some(false)),
            "c" | "cancel" | "取消" => return Ok(None),
            _ => println!("请输入 y 或 n；输入 c 可取消。"),
        }
    }
}

fn prompt_line(prompt: &str) -> Result<Option<String>> {
    print!("{prompt}");
    io::stdout()
        .flush()
        .map_err(|e| format!("flush prompt: {e}"))?;
    let mut line = String::new();
    let count = io::stdin()
        .read_line(&mut line)
        .map_err(|e| format!("read prompt: {e}"))?;
    if count == 0 {
        Ok(None)
    } else {
        Ok(Some(line.trim_end_matches(['\r', '\n']).to_string()))
    }
}

fn trim_wrapping_quotes(value: &str) -> &str {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            return &value[1..value.len() - 1];
        }
    }
    value
}
