use aitsuno_area_text::{extract_path, inject_path};
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const HELP: &str = r#"area_text 0.2.0

《アソコの幸福》四碟结构化文本提取/注入工具。

用法:
  area_text.exe extract --input <UNPACKED_FDI_ROOT_OR_FILE> --output <ALL_JSON_DIR> [--overwrite]
  area_text.exe inject --source <UNPACKED_FDI_ROOT_OR_FILE> --translation <JSON_FILE_OR_DIR> --output <FILE_OR_DIR> [--overwrite]
  area_text.exe [PATH]

模式:
  带完整参数的 extract/inject 是一次性非交互操作。
  无参数或仅传入一个路径时进入交互会话；确认前不会写文件。

参数:
  -i, --input <PATH>          extract 输入单文件或完整解包 FDI 根目录
  -s, --source <PATH>         inject 原始单文件或完整解包 FDI 根目录
  -t, --translation <PATH>    inject UTF-8 JSON 文件或目录
  -o, --output <PATH>         显式输出文件或目录
      --overwrite             明确允许替换已有输出
  -h, --help                  显示帮助

规则:
  目录模式递归处理所有子目录；JSON 扁平汇总到同一输出目录，源相对路径保存在 JSON 元数据中。
  支持 AREA、GEN、MES、MAIN、OPENING、INTER、BUNSYO 及已确认的系统文本，不输出 name。
  同一逻辑条目中的原作物理换行会移除；AREA 注入不主动折行，由游戏运行时自动换行。
  修改后的 message 会先使用内置 3025 字简中到 CP932 载体映射，再编码写回。
  scr_msg 是不可修改的源校验字段；只有 message 会写回。
  已有输出可由 --overwrite 或交互确认替换；输出不得与源或翻译路径重叠。
"#;

enum Invocation {
    Help,
    Extract {
        input: PathBuf,
        output: PathBuf,
        overwrite: bool,
    },
    Inject {
        source: PathBuf,
        translation: PathBuf,
        output: PathBuf,
        overwrite: bool,
    },
    Interactive {
        prefill: Option<PathBuf>,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("错误: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match parse_invocation(env::args_os().skip(1).collect())? {
        Invocation::Help => {
            print!("{HELP}");
            Ok(())
        }
        Invocation::Extract {
            input,
            output,
            overwrite,
        } => run_extract(&input, &output, overwrite),
        Invocation::Inject {
            source,
            translation,
            output,
            overwrite,
        } => run_inject(&source, &translation, &output, overwrite),
        Invocation::Interactive { prefill } => interactive_session(prefill),
    }
}

fn parse_invocation(args: Vec<OsString>) -> Result<Invocation, String> {
    if args.is_empty() {
        return Ok(Invocation::Interactive { prefill: None });
    }
    if args.len() == 1 {
        if matches!(args[0].to_str(), Some("-h" | "--help")) {
            return Ok(Invocation::Help);
        }
        if args[0]
            .to_str()
            .map(|value| value.starts_with('-'))
            .unwrap_or(false)
        {
            return Err("未知参数；使用 --help 查看用法".to_string());
        }
        return Ok(Invocation::Interactive {
            prefill: Some(PathBuf::from(&args[0])),
        });
    }

    let command = args[0]
        .to_str()
        .ok_or_else(|| "命令名无法表示为 Unicode".to_string())?;
    match command {
        "extract" => parse_extract(&args[1..]),
        "inject" => parse_inject(&args[1..]),
        "-h" | "--help" => Ok(Invocation::Help),
        _ => Err(format!("未知命令: {command}")),
    }
}

fn parse_extract(args: &[OsString]) -> Result<Invocation, String> {
    let mut input = None;
    let mut output = None;
    let mut overwrite = false;
    let mut index = 0usize;
    while index < args.len() {
        let argument = args[index]
            .to_str()
            .ok_or_else(|| "命令选项无法表示为 Unicode".to_string())?;
        match argument {
            "-h" | "--help" => return Ok(Invocation::Help),
            "-i" | "--input" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| format!("{argument} 缺少路径"))?;
                if input.replace(PathBuf::from(value)).is_some() {
                    return Err("--input 只能指定一次".to_string());
                }
            }
            "-o" | "--output" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| format!("{argument} 缺少路径"))?;
                if output.replace(PathBuf::from(value)).is_some() {
                    return Err("--output 只能指定一次".to_string());
                }
            }
            "--overwrite" => overwrite = true,
            _ => return Err(format!("extract 未知参数: {argument}")),
        }
        index += 1;
    }
    Ok(Invocation::Extract {
        input: input.ok_or_else(|| "extract 缺少 --input".to_string())?,
        output: output.ok_or_else(|| "extract 缺少 --output".to_string())?,
        overwrite,
    })
}

fn parse_inject(args: &[OsString]) -> Result<Invocation, String> {
    let mut source = None;
    let mut translation = None;
    let mut output = None;
    let mut overwrite = false;
    let mut index = 0usize;
    while index < args.len() {
        let argument = args[index]
            .to_str()
            .ok_or_else(|| "命令选项无法表示为 Unicode".to_string())?;
        match argument {
            "-h" | "--help" => return Ok(Invocation::Help),
            "-s" | "--source" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| format!("{argument} 缺少路径"))?;
                if source.replace(PathBuf::from(value)).is_some() {
                    return Err("--source 只能指定一次".to_string());
                }
            }
            "-t" | "--translation" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| format!("{argument} 缺少路径"))?;
                if translation.replace(PathBuf::from(value)).is_some() {
                    return Err("--translation 只能指定一次".to_string());
                }
            }
            "-o" | "--output" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| format!("{argument} 缺少路径"))?;
                if output.replace(PathBuf::from(value)).is_some() {
                    return Err("--output 只能指定一次".to_string());
                }
            }
            "--overwrite" => overwrite = true,
            _ => return Err(format!("inject 未知参数: {argument}")),
        }
        index += 1;
    }
    Ok(Invocation::Inject {
        source: source.ok_or_else(|| "inject 缺少 --source".to_string())?,
        translation: translation.ok_or_else(|| "inject 缺少 --translation".to_string())?,
        output: output.ok_or_else(|| "inject 缺少 --output".to_string())?,
        overwrite,
    })
}

fn run_extract(input: &Path, output: &Path, overwrite: bool) -> Result<(), String> {
    let report = extract_path(input, output, overwrite)?;
    println!("[extract] scanned_files={}", report.scanned_files);
    println!("[extract] skipped_files={}", report.skipped_files);
    println!("[extract] json_files={}", report.json_files);
    println!("[extract] source_messages={}", report.source_messages);
    println!("[extract] source_choices={}", report.source_choices);
    println!("[extract] extracted_entries={}", report.extracted_entries);
    println!("[extract] physical_lines={}", report.physical_lines);
    println!(
        "[extract] max_visible_line_bytes={}",
        report.max_visible_line_bytes
    );
    println!("[extract] over_limit={}", report.over_limit);
    for warning in &report.warnings {
        eprintln!("[extract] warning={warning}");
    }
    println!("[extract] warnings={}", report.warnings.len());
    println!("[extract] output={}", report.output_root.display());
    Ok(())
}

fn run_inject(
    source: &Path,
    translation: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<(), String> {
    let report = inject_path(source, translation, output, overwrite)?;
    println!("[inject] json_files={}", report.json_files);
    println!("[inject] json_entries={}", report.json_entries);
    println!("[inject] patched={}", report.patched);
    println!("[inject] unchanged={}", report.unchanged);
    println!("[inject] rebuilt_files={}", report.rebuilt_files);
    println!("[inject] output={}", report.output.display());
    Ok(())
}

fn interactive_session(prefill: Option<PathBuf>) -> Result<(), String> {
    loop {
        println!();
        println!("《アソコの幸福》四碟文本工具");
        println!("1. 提取 UTF-8 JSON");
        println!("2. 注入 UTF-8 JSON");
        println!("0. 退出");
        match prompt("请选择")?.trim() {
            "0" => return Ok(()),
            "1" => {
                if let Err(error) = interactive_extract(prefill.clone()) {
                    eprintln!("操作未完成: {error}");
                }
            }
            "2" => {
                if let Err(error) = interactive_inject(prefill.clone()) {
                    eprintln!("操作未完成: {error}");
                }
            }
            _ => println!("无效选择，请重试。"),
        }
    }
}

fn interactive_extract(mut input: Option<PathBuf>) -> Result<(), String> {
    loop {
        input = Some(prompt_path(
            "完整解包 FDI 根目录或单个支持文件",
            input.as_deref(),
        )?);
        if input.as_ref().is_some_and(|path| path == Path::new("0")) {
            return Ok(());
        }
        let input_path = input.as_ref().expect("input");
        let default_output = suggested_directory(input_path, "alljson");
        let output = prompt_path("汇总 JSON 输出目录", Some(&default_output))?;
        if output == Path::new("0") {
            return Ok(());
        }
        let overwrite = prompt_yes_no("输出已存在时是否明确覆盖？[y/N]")?;
        println!("写入前确认:");
        println!("  operation=extract");
        println!("  input={}", input_path.display());
        println!("  output={}", output.display());
        println!("  overwrite={overwrite}");
        match prompt("输入 y 开始，输入 m 修改，输入 0 取消")?
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "y" | "yes" => {
                match run_extract(input_path, &output, overwrite) {
                    Ok(()) => println!("操作完成，返回主菜单。"),
                    Err(error) => eprintln!("操作失败: {error}"),
                }
                return Ok(());
            }
            "m" => continue,
            "0" | "" => return Ok(()),
            _ => println!("未确认写入，返回修改。"),
        }
    }
}

fn interactive_inject(mut source: Option<PathBuf>) -> Result<(), String> {
    loop {
        source = Some(prompt_path(
            "原始完整解包 FDI 根目录或单个支持文件",
            source.as_deref(),
        )?);
        if source.as_ref().is_some_and(|path| path == Path::new("0")) {
            return Ok(());
        }
        let translation = prompt_path("翻译 JSON 文件或目录", None)?;
        if translation == Path::new("0") {
            return Ok(());
        }
        let source_path = source.as_ref().expect("source");
        let default_output = suggested_injected_output(source_path);
        let output = prompt_path("注入输出文件或目录", Some(&default_output))?;
        if output == Path::new("0") {
            return Ok(());
        }
        let overwrite = prompt_yes_no("输出已存在时是否明确覆盖？[y/N]")?;
        println!("写入前确认:");
        println!("  operation=inject");
        println!("  source={}", source_path.display());
        println!("  translation={}", translation.display());
        println!("  output={}", output.display());
        println!("  overwrite={overwrite}");
        match prompt("输入 y 开始，输入 m 修改，输入 0 取消")?
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "y" | "yes" => {
                match run_inject(source_path, &translation, &output, overwrite) {
                    Ok(()) => println!("操作完成，返回主菜单。"),
                    Err(error) => eprintln!("操作失败: {error}"),
                }
                return Ok(());
            }
            "m" => continue,
            "0" | "" => return Ok(()),
            _ => println!("未确认写入，返回修改。"),
        }
    }
}

fn prompt_path(label: &str, default: Option<&Path>) -> Result<PathBuf, String> {
    let label = match default {
        Some(path) => format!("{label}（按 Enter 使用 {}，输入 0 返回）", path.display()),
        None => format!("{label}（输入 0 返回）"),
    };
    let value = prompt(&label)?;
    if value.trim().is_empty() {
        return default
            .map(Path::to_path_buf)
            .ok_or_else(|| format!("{label}不能为空"));
    }
    Ok(PathBuf::from(strip_outer_quotes(value.trim())))
}

fn suggested_directory(input: &Path, suffix: &str) -> PathBuf {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("area_text");
    parent.join(format!("{stem}_{suffix}"))
}

fn suggested_injected_output(source: &Path) -> PathBuf {
    if source.is_file() {
        let parent = source.parent().unwrap_or_else(|| Path::new("."));
        let stem = source
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("area");
        let extension = source.extension().and_then(|value| value.to_str());
        let name = match extension {
            Some(extension) => format!("{stem}_injected.{extension}"),
            None => format!("{stem}_injected"),
        };
        parent.join(name)
    } else {
        suggested_directory(source, "injected")
    }
}

fn prompt_yes_no(label: &str) -> Result<bool, String> {
    Ok(matches!(
        prompt(label)?.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

fn strip_outer_quotes(value: &str) -> &str {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn prompt(label: &str) -> Result<String, String> {
    print!("{label}: ");
    io::stdout()
        .flush()
        .map_err(|error| format!("无法刷新终端输出: {error}"))?;
    let mut input = String::new();
    let bytes = io::stdin()
        .read_line(&mut input)
        .map_err(|error| format!("无法读取终端输入: {error}"))?;
    if bytes == 0 {
        return Ok("0".to_string());
    }
    Ok(input.trim_end_matches(['\r', '\n']).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_only_invocation_is_interactive_prefill() {
        let invocation = parse_invocation(vec![OsString::from("任意路径")]).expect("parse");
        match invocation {
            Invocation::Interactive { prefill } => {
                assert_eq!(prefill, Some(PathBuf::from("任意路径")));
            }
            _ => panic!("expected interactive invocation"),
        }
    }

    #[test]
    fn explicit_extract_requires_output() {
        let error = match parse_invocation(vec![
            OsString::from("extract"),
            OsString::from("--input"),
            OsString::from("source"),
        ]) {
            Ok(_) => panic!("missing output should fail"),
            Err(error) => error,
        };
        assert!(error.contains("--output"));
    }
}
