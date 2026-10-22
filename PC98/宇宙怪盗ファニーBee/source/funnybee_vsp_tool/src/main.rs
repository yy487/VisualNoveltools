use funnybee_vsp_tool::{convert_recursive, ConvertOptions};
use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const HELP: &str = r#"funnybee_vsp_tool 0.1.0

递归识别 AliceSoft VSP 图像并批量转换为 16 色 PNG。

用法:
  funnybee_vsp_tool.exe convert --input <FILE_OR_DIR> --output <FILE_OR_DIR> [--overwrite]

选项:
  --overwrite   明确允许替换已有输出

不带参数或只给一个路径时进入交互模式。目录转换保持相对层级，PNG 文件名在原文件名后
追加 .png，并写出 _vsp_manifest.json。格式按内容识别，不依赖 .bin 或 .vsp 扩展名。
"#;

#[derive(Debug)]
struct Invocation {
    input: PathBuf,
    output: PathBuf,
    overwrite: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("错误: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        return interactive_loop(None);
    }
    if args.len() == 1 {
        if matches!(args[0].to_str(), Some("-h" | "--help")) {
            print!("{HELP}");
            return Ok(());
        }
        return interactive_loop(Some(PathBuf::from(&args[0])));
    }
    execute(&parse_args(args)?)
}

fn parse_args(args: Vec<OsString>) -> Result<Invocation, String> {
    if args.first().and_then(|value| value.to_str()) != Some("convert") {
        return Err("未知命令；使用 --help 查看用法".to_string());
    }
    let mut input = None;
    let mut output = None;
    let mut overwrite = false;
    let mut index = 1usize;
    while index < args.len() {
        match args[index].to_str() {
            Some("--input") => {
                index += 1;
                input = args.get(index).map(PathBuf::from);
                if input.is_none() {
                    return Err("--input 缺少路径".to_string());
                }
            }
            Some("--output") => {
                index += 1;
                output = args.get(index).map(PathBuf::from);
                if output.is_none() {
                    return Err("--output 缺少路径".to_string());
                }
            }
            Some("--overwrite") => overwrite = true,
            Some(option) => return Err(format!("未知选项: {option}")),
            None => return Err("命令或选项必须为 ASCII".to_string()),
        }
        index += 1;
    }
    Ok(Invocation {
        input: input.ok_or_else(|| "缺少 --input".to_string())?,
        output: output.ok_or_else(|| "缺少 --output".to_string())?,
        overwrite,
    })
}

fn execute(invocation: &Invocation) -> Result<(), String> {
    let report = convert_recursive(
        &invocation.input,
        &invocation.output,
        ConvertOptions {
            overwrite: invocation.overwrite,
        },
    )?;
    println!(
        "完成: scanned={} converted={} skipped={} warnings={} output={}",
        report.scanned_files,
        report.converted_images,
        report.skipped_files,
        report.warnings.len(),
        report.output.display()
    );
    for warning in report.warnings {
        eprintln!("警告: {warning}");
    }
    Ok(())
}

fn interactive_loop(initial: Option<PathBuf>) -> Result<(), String> {
    let mut prefill = initial;
    loop {
        println!("\nFunnyBee VSP → PNG 递归转换工具");
        println!("  1. 转换一个文件或递归转换目录");
        println!("  0. 退出");
        let choice = prompt("选择", Some("1"))?;
        if choice == "0" {
            return Ok(());
        }
        if choice != "1" {
            eprintln!("无效选择");
            continue;
        }
        let default_input = prefill
            .take()
            .map(|path| path.to_string_lossy().into_owned());
        let input = PathBuf::from(prompt("输入文件或目录", default_input.as_deref())?);
        let output_default = suggested_output(&input).to_string_lossy().into_owned();
        let output = PathBuf::from(prompt("输出文件或目录", Some(&output_default))?);
        let overwrite = output.exists() && prompt_yes_no("输出已存在，覆盖")?;
        println!("输入: {}", input.display());
        println!("输出: {}", output.display());
        if !prompt_yes_no("确认执行")? {
            println!("已取消");
            continue;
        }
        if let Err(error) = execute(&Invocation {
            input,
            output,
            overwrite,
        }) {
            eprintln!("错误: {error}");
        }
    }
}

fn suggested_output(input: &Path) -> PathBuf {
    let parent = input
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = input
        .file_name()
        .unwrap_or_else(|| OsStr::new("vsp_output"));
    let mut name = file_name.to_os_string();
    if input.is_dir() {
        name.push("_png");
    } else {
        name.push(".png");
    }
    parent.join(name)
}

fn prompt(label: &str, default: Option<&str>) -> Result<String, String> {
    match default {
        Some(value) if !value.is_empty() => print!("{label} [{value}]: "),
        _ => print!("{label}: "),
    }
    io::stdout()
        .flush()
        .map_err(|error| format!("无法刷新终端: {error}"))?;
    let mut value = String::new();
    let count = io::stdin()
        .read_line(&mut value)
        .map_err(|error| format!("无法读取终端输入: {error}"))?;
    if count == 0 {
        return Err("终端输入结束".to_string());
    }
    let value = value.trim().to_string();
    if value.is_empty() {
        if let Some(default) = default {
            return Ok(default.to_string());
        }
    }
    Ok(value)
}

fn prompt_yes_no(label: &str) -> Result<bool, String> {
    Ok(matches!(
        prompt(&format!("{label} [y/N]"), Some("N"))?
            .to_ascii_lowercase()
            .as_str(),
        "y" | "yes"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_default_is_sibling_png_directory() {
        let input = Path::new(r"C:\游戏\members");
        let mut expected_name = input.file_name().unwrap().to_os_string();
        expected_name.push("_png");
        assert_eq!(
            input.with_file_name(expected_name),
            PathBuf::from(r"C:\游戏\members_png")
        );
    }

    #[test]
    fn parses_noninteractive_conversion() {
        let invocation = parse_args(vec![
            OsString::from("convert"),
            OsString::from("--input"),
            OsString::from("输入"),
            OsString::from("--output"),
            OsString::from("输出"),
        ])
        .unwrap();
        assert_eq!(invocation.input, PathBuf::from("输入"));
        assert_eq!(invocation.output, PathBuf::from("输出"));
    }
}
