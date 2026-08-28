use foxy_d88_tool::archive;
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

type CliResult<T> = Result<T, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Unpack,
    Pack,
}

impl Action {
    fn label(self) -> &'static str {
        match self {
            Self::Unpack => "解包 D88",
            Self::Pack => "封包 D88",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Job {
    action: Action,
    input: PathBuf,
    output: PathBuf,
    overwrite: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Invocation {
    Help,
    Interactive { unpack_prefill: Option<PathBuf> },
    OneShot(Job),
}

fn main() {
    let code = match run() {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("错误：{error}");
            eprintln!("使用 --help 查看命令说明。");
            1
        }
    };
    if code != 0 {
        std::process::exit(code);
    }
}

fn run() -> CliResult<()> {
    let invocation = parse_invocation(env::args_os().skip(1).collect())?;
    match invocation {
        Invocation::Help => {
            print_help();
            Ok(())
        }
        Invocation::Interactive { unpack_prefill } => run_menu(unpack_prefill),
        Invocation::OneShot(job) => execute_job(&job),
    }
}

fn print_help() {
    println!("FOXY PC-8801 D88 独立解包/封包工具");
    println!();
    println!("双击或无参数启动：");
    println!("  foxy_disk_tool");
    println!();
    println!("将 D88 路径拖到程序上（仅作交互预填，确认前不会写入）：");
    println!("  foxy_disk_tool <input.d88>");
    println!();
    println!("非交互、一次性命令：");
    println!("  foxy_disk_tool unpack --input <input.d88> --output <directory> [--overwrite]");
    println!(
        "  foxy_disk_tool pack --input <unpacked-directory> --output <output.d88> [--overwrite]"
    );
    println!();
    println!("选项：");
    println!("  -i, --input       输入 D88 或已解包目录");
    println!("  -o, --output      输出目录或 D88 文件");
    println!("      --overwrite   允许替换本工具管理的旧输出");
    println!("  -h, --help        显示本说明");
    println!();
    println!("路径可包含 Unicode 字符和空格；在命令行中请按终端规则给含空格路径加引号。");
}

fn parse_invocation(args: Vec<OsString>) -> CliResult<Invocation> {
    if args.is_empty() {
        return Ok(Invocation::Interactive {
            unpack_prefill: None,
        });
    }
    if args
        .iter()
        .any(|arg| matches!(arg.to_str(), Some("-h" | "--help")))
    {
        return Ok(Invocation::Help);
    }
    if args.len() == 1 {
        let value = &args[0];
        let text = value.to_string_lossy();
        if text != "unpack" && text != "pack" && !text.starts_with('-') {
            return Ok(Invocation::Interactive {
                unpack_prefill: Some(PathBuf::from(value)),
            });
        }
    }

    let command = args[0]
        .to_str()
        .ok_or_else(|| "command name is not valid Unicode".to_string())?;
    let action = match command {
        "unpack" => Action::Unpack,
        "pack" => Action::Pack,
        _ => return Err(format!("unknown command: {command}")),
    };
    Ok(Invocation::OneShot(parse_job(action, &args[1..])?))
}

fn parse_job(action: Action, args: &[OsString]) -> CliResult<Job> {
    let mut input = None;
    let mut output = None;
    let mut overwrite = false;
    let mut index = 0usize;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| "option name is not valid Unicode".to_string())?;
        match option {
            "--overwrite" => overwrite = true,
            "--input" | "-i" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| format!("{option} requires a path"))?;
                if input.replace(PathBuf::from(value)).is_some() {
                    return Err("input path was specified more than once".to_string());
                }
            }
            "--output" | "-o" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| format!("{option} requires a path"))?;
                if output.replace(PathBuf::from(value)).is_some() {
                    return Err("output path was specified more than once".to_string());
                }
            }
            value if value.starts_with('-') => return Err(format!("unknown option: {value}")),
            value => {
                return Err(format!(
                    "unexpected positional argument: {value}; use --input and --output"
                ))
            }
        }
        index += 1;
    }
    Ok(Job {
        action,
        input: input.ok_or_else(|| format!("{} requires --input", action.label()))?,
        output: output.ok_or_else(|| format!("{} requires --output", action.label()))?,
        overwrite,
    })
}

fn execute_job(job: &Job) -> CliResult<()> {
    match job.action {
        Action::Unpack => {
            let summary = archive::unpack_d88(&job.input, &job.output, job.overwrite)?;
            println!(
                "解包完成：{} 卷、{} 个目录项（其中 {} 个压缩项）→ {}",
                summary.volume_count,
                summary.entry_count,
                summary.compressed_entry_count,
                summary.output.display()
            );
        }
        Action::Pack => {
            let summary = archive::pack_d88(&job.input, &job.output, job.overwrite)?;
            println!(
                "封包完成：{} 卷、{} 个已变更目录项、{} bytes → {}",
                summary.volume_count,
                summary.changed_entry_count,
                summary.output_size,
                summary.output.display()
            );
        }
    }
    Ok(())
}

fn run_menu(initial_unpack_prefill: Option<PathBuf>) -> CliResult<()> {
    println!("FOXY PC-8801 D88 独立工具");
    println!("任何写入都会在最终确认后才开始。");
    if let Some(prefill) = initial_unpack_prefill {
        run_interactive_job(Action::Unpack, Some(prefill))?;
    }
    loop {
        println!();
        println!("主菜单");
        println!("  1  解包 D88");
        println!("  2  封包 D88");
        println!("  0  退出");
        let Some(choice) = prompt_line("请选择 [0]: ")? else {
            return Ok(());
        };
        match choice.trim() {
            "" | "0" => return Ok(()),
            "1" => run_interactive_job(Action::Unpack, None)?,
            "2" => run_interactive_job(Action::Pack, None)?,
            _ => println!("无效选择，请输入 1、2 或 0。"),
        }
    }
}

fn run_interactive_job(action: Action, input_prefill: Option<PathBuf>) -> CliResult<()> {
    println!();
    println!("{}设置（留空且无默认值时取消）", action.label());
    let Some(input) = prompt_path(
        match action {
            Action::Unpack => "输入 D88",
            Action::Pack => "已解包目录",
        },
        input_prefill.as_deref(),
    )?
    else {
        println!("已取消，返回主菜单。");
        return Ok(());
    };
    let default_output = match action {
        Action::Unpack => archive::suggested_unpack_output(&input),
        Action::Pack => archive::suggested_pack_output(&input),
    };
    let Some(output) = prompt_path(
        match action {
            Action::Unpack => "输出目录",
            Action::Pack => "输出 D88",
        },
        Some(&default_output),
    )?
    else {
        println!("已取消，返回主菜单。");
        return Ok(());
    };
    let Some(overwrite) = prompt_yes_no("允许覆盖旧的受管理输出", false)? else {
        println!("已取消，返回主菜单。");
        return Ok(());
    };

    println!();
    println!("请确认：");
    println!("  操作      : {}", action.label());
    println!("  输入      : {}", input.display());
    println!("  输出      : {}", output.display());
    println!("  overwrite : {overwrite}");
    if prompt_yes_no("确认开始写入", false)? != Some(true) {
        println!("已取消，未写入任何内容。返回主菜单。");
        return Ok(());
    }

    let job = Job {
        action,
        input,
        output,
        overwrite,
    };
    if let Err(error) = execute_job(&job) {
        println!("操作失败：{error}");
    }
    wait_for_enter()
}

fn prompt_path(label: &str, default: Option<&Path>) -> CliResult<Option<PathBuf>> {
    let prompt = match default {
        Some(default) => format!("{label} [{}]: ", default.display()),
        None => format!("{label} [留空取消]: "),
    };
    let Some(line) = prompt_line(&prompt)? else {
        return Ok(None);
    };
    let trimmed = trim_wrapping_quotes(line.trim());
    if trimmed.is_empty() {
        return Ok(default.map(Path::to_path_buf));
    }
    Ok(Some(PathBuf::from(trimmed)))
}

fn prompt_yes_no(label: &str, default: bool) -> CliResult<Option<bool>> {
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

fn prompt_line(prompt: &str) -> CliResult<Option<String>> {
    print!("{prompt}");
    io::stdout()
        .flush()
        .map_err(|error| format!("failed to flush prompt: {error}"))?;
    let mut line = String::new();
    let count = io::stdin()
        .read_line(&mut line)
        .map_err(|error| format!("failed to read prompt: {error}"))?;
    if count == 0 {
        Ok(None)
    } else {
        Ok(Some(line.trim_end_matches(['\r', '\n']).to_string()))
    }
}

fn wait_for_enter() -> CliResult<()> {
    let _ = prompt_line("按 Enter 返回主菜单……")?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn os(value: &str) -> OsString {
        OsString::from(value)
    }

    #[test]
    fn no_arguments_enters_menu() {
        assert_eq!(
            parse_invocation(Vec::new()).unwrap(),
            Invocation::Interactive {
                unpack_prefill: None
            }
        );
    }

    #[test]
    fn path_only_is_interactive_prefill() {
        let path = r"E:\游戏 目录\FOXY 碟.d88";
        assert_eq!(
            parse_invocation(vec![os(path)]).unwrap(),
            Invocation::Interactive {
                unpack_prefill: Some(PathBuf::from(path))
            }
        );
    }

    #[test]
    fn full_unicode_command_is_one_shot() {
        let invocation = parse_invocation(vec![
            os("unpack"),
            os("--input"),
            os(r"E:\游戏 目录\FOXY 碟.d88"),
            os("--output"),
            os(r"E:\输出 目录\FOXY 解包"),
            os("--overwrite"),
        ])
        .unwrap();
        assert_eq!(
            invocation,
            Invocation::OneShot(Job {
                action: Action::Unpack,
                input: PathBuf::from(r"E:\游戏 目录\FOXY 碟.d88"),
                output: PathBuf::from(r"E:\输出 目录\FOXY 解包"),
                overwrite: true,
            })
        );
    }

    #[test]
    fn incomplete_noninteractive_command_is_rejected() {
        let error = parse_invocation(vec![os("pack"), os("--input"), os("workspace")]).unwrap_err();
        assert!(error.contains("--output"));
    }

    #[test]
    fn pasted_quotes_are_removed_only_as_a_pair() {
        assert_eq!(
            trim_wrapping_quotes(r#""E:\a b\Foxy.d88""#),
            r"E:\a b\Foxy.d88"
        );
        assert_eq!(trim_wrapping_quotes(r"E:\a b\Foxy.d88"), r"E:\a b\Foxy.d88");
    }
}
