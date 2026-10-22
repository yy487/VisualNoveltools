use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about = "Nooch 1/2/3 FDI 资源提取、文本导入和字库生成工具")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// 递归发现三作 FDI，提取真实文件并导出翻译 JSON
    Extract {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        workspace: PathBuf,
        #[arg(long)]
        overwrite: bool,
    },
    /// 读取翻译 JSON，生成三份字库并重建完整 FDI 集
    Inject {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        workspace: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        overwrite: bool,
    },
}

fn prompt(label: &str, default: Option<&str>) -> Result<PathBuf> {
    match default {
        Some(value) => print!("{label} [{value}]: "),
        None => print!("{label}: "),
    }
    io::stdout().flush()?;
    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    let value = value.trim().trim_matches('"');
    if value.is_empty() {
        default
            .map(PathBuf::from)
            .context("没有输入路径，也没有可用默认值")
    } else {
        Ok(PathBuf::from(value))
    }
}

fn confirm(label: &str) -> Result<bool> {
    print!("{label} [y/N]: ");
    io::stdout().flush()?;
    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    Ok(matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

fn pause() {
    print!("按 Enter 键退出...");
    let _ = io::stdout().flush();
    let mut value = String::new();
    let _ = io::stdin().read_line(&mut value);
}

fn interactive() -> Result<()> {
    println!("Nooch trilogy tool {}", env!("CARGO_PKG_VERSION"));
    println!("1) 提取真实文件 + 导出 JSON");
    println!("2) 注入译文 + 生成字库 + 重建 FDI");
    print!("请选择 [1]: ");
    io::stdout().flush()?;
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let input = prompt("三部游戏所在根目录", Some(r"E:\GAL\gb\nooch!"))?;
    let workspace = prompt(
        "工作目录",
        Some(r"E:\GAL\gb\nooch!\work\nooch_tool_workspace"),
    )?;
    if choice.trim().is_empty() || choice.trim() == "1" {
        let overwrite =
            workspace.exists() && confirm("工作目录已存在，覆盖并保留其中已有译文吗？")?;
        if workspace.exists() && !overwrite {
            println!("已取消，没有修改任何文件。");
            pause();
            return Ok(());
        }
        let report = nooch_tool::extract(&input, &workspace, overwrite)?;
        println!(
            "完成：{} 张 FDI，{} 个真实文件，{} 条文本",
            report.disks, report.files, report.messages
        );
    } else if choice.trim() == "2" {
        let output = prompt(
            "重建输出目录",
            Some(r"E:\GAL\gb\nooch!\work\nooch_tool_output"),
        )?;
        let overwrite = output.exists() && confirm("重建输出目录已存在，要覆盖吗？")?;
        if output.exists() && !overwrite {
            println!("已取消，没有修改任何文件。");
            pause();
            return Ok(());
        }
        let report = nooch_tool::inject(&input, &workspace, &output, overwrite)?;
        println!(
            "完成：{} 张 FDI，{} 个资源文件发生变化，{} 条译文，{} 个字形",
            report.disks, report.changed_files, report.changed_messages, report.patched_glyphs
        );
    } else {
        anyhow::bail!("只能选择 1 或 2");
    }
    pause();
    Ok(())
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Some(Command::Extract {
            input,
            workspace,
            overwrite,
        }) => {
            let report = nooch_tool::extract(&input, &workspace, overwrite)?;
            println!(
                "完成：{} 张 FDI，{} 个真实文件，{} 条文本",
                report.disks, report.files, report.messages
            );
            Ok(())
        }
        Some(Command::Inject {
            input,
            workspace,
            output,
            overwrite,
        }) => {
            let report = nooch_tool::inject(&input, &workspace, &output, overwrite)?;
            println!(
                "完成：{} 张 FDI，{} 个资源文件发生变化，{} 条译文，{} 个字形",
                report.disks, report.changed_files, report.changed_messages, report.patched_glyphs
            );
            Ok(())
        }
        None => interactive(),
    }
}

fn main() {
    let interactive = std::env::args_os().len() == 1;
    if let Err(error) = run() {
        eprintln!("错误: {error:#}");
        if interactive {
            pause();
        }
        std::process::exit(1);
    }
}
