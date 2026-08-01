use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

use acv1_dat_tool::archive::parse_archive;
use acv1_dat_tool::workflow::{self, PackReport, UnpackReport};

#[derive(Debug, Parser)]
#[command(
    name = "acv1_dat_tool",
    version,
    about = "ACV1/legacy script.dat 通用解包与回封工具",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// 自动判断分支并解包；未给游戏名时交互输入
    Unpack {
        input: PathBuf,
        #[arg(long)]
        game_title: Option<String>,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// 根据 manifest.json 保留原分支并回封
    Pack {
        input_dir: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long, help = "将 out_capacity 改为实际明文长度")]
        no_preserve_capacity: bool,
    },
    /// 自动判断分支并完整验证所有条目
    Verify {
        input: PathBuf,
        #[arg(long)]
        game_title: Option<String>,
    },
    /// 在内存中执行解包、回封、再解包并比较明文
    Roundtrip {
        input: PathBuf,
        #[arg(long)]
        game_title: Option<String>,
    },
}

fn main() -> ExitCode {
    init_console_utf8();
    let args: Vec<OsString> = std::env::args_os().collect();
    if let Some(path) = drag_drop_path(&args) {
        let success = run_drag_drop(&path);
        pause_before_exit();
        return if success {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }

    match run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[error] {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Unpack {
            input,
            game_title,
            output,
        } => {
            let title = require_or_prompt_title(game_title)?;
            print_unpack_report(&workflow::unpack(&input, output.as_deref(), &title)?);
        }
        Command::Pack {
            input_dir,
            output,
            no_preserve_capacity,
        } => {
            print_pack_report(&workflow::pack(
                &input_dir,
                output.as_deref(),
                !no_preserve_capacity,
            )?);
        }
        Command::Verify { input, game_title } => {
            let title = require_or_prompt_title(game_title)?;
            let report = workflow::verify(&input, &title)?;
            println!("[verify] branch={}", report.branch);
            println!("[verify] entries={}", report.entries);
            println!("[verify] unpacked_bytes={}", report.unpacked_bytes);
            println!("[verify] index_end=0x{:X}", report.index_end);
            println!("[verify] data_base=0x{:X}", report.data_base);
            println!("[verify] opaque_bytes={}", report.opaque_bytes);
            println!("[verify] crc64=0x{:016X}", report.crc64);
            println!("[verify] key_low32=0x{:08X}", report.key_low32);
        }
        Command::Roundtrip { input, game_title } => {
            let title = require_or_prompt_title(game_title)?;
            let report = workflow::roundtrip(&input, &title)?;
            println!("[roundtrip] branch={}", report.branch);
            println!("[roundtrip] entries={}", report.entries);
            println!("[roundtrip] internal_exact={}", report.internal_exact);
            println!(
                "[roundtrip] archive_byte_exact={}",
                report.archive_byte_exact
            );
            println!("[roundtrip] original_bytes={}", report.original_bytes);
            println!("[roundtrip] rebuilt_bytes={}", report.rebuilt_bytes);
            println!("[roundtrip] key_low32=0x{:08X}", report.key_low32);
            if !report.internal_exact {
                bail!("回环明文不一致");
            }
        }
    }
    Ok(())
}

fn run_drag_drop(path: &Path) -> bool {
    let result: Result<()> = (|| {
        if path.is_file() {
            drag_unpack(path)
        } else if path.is_dir() {
            drag_pack(path)
        } else {
            bail!("拖入路径既不是文件也不是目录: {}", path.display())
        }
    })();

    match result {
        Ok(()) => true,
        Err(error) => {
            eprintln!();
            eprintln!("========== 操作失败 ==========");
            eprintln!("{error:#}");
            eprintln!("==============================");
            false
        }
    }
}

fn drag_unpack(path: &Path) -> Result<()> {
    let data = std::fs::read(path).with_context(|| format!("读取失败: {}", path.display()))?;
    let archive =
        parse_archive(&data).with_context(|| format!("无法识别封包: {}", path.display()))?;
    println!("检测到封包分支: {}", archive.branch);
    println!("索引条目数: {}", archive.entries.len());
    let title = prompt_title()?;
    let report = workflow::unpack(path, None, &title)?;
    println!();
    println!("========== 解包成功 ==========");
    println!("分支: {}", report.branch);
    println!("条目数: {}", report.entries);
    println!("CRC64: 0x{:016X}", report.crc64);
    println!("密钥低 32 位: 0x{:08X}", report.key_low32);
    println!("输出目录: {}", report.output.display());
    println!("==============================");
    Ok(())
}

fn drag_pack(path: &Path) -> Result<()> {
    println!("检测到目录，按 manifest.json 回封: {}", path.display());
    let report = workflow::pack(path, None, true)?;
    println!();
    println!("========== 回封成功 ==========");
    println!("分支: {}", report.branch);
    println!("条目数: {}", report.entries);
    println!("密钥低 32 位: 0x{:08X}", report.key_low32);
    println!("输出文件: {}", report.output.display());
    println!("==============================");
    Ok(())
}

fn drag_drop_path(args: &[OsString]) -> Option<PathBuf> {
    if args.len() != 2 {
        return None;
    }
    let path = PathBuf::from(&args[1]);
    path.exists().then_some(path)
}

fn require_or_prompt_title(game_title: Option<String>) -> Result<String> {
    match game_title {
        Some(title) if !title.is_empty() => Ok(title),
        Some(_) => bail!("游戏名不能为空"),
        None => prompt_title(),
    }
}

fn prompt_title() -> Result<String> {
    print!("请输入游戏名（必须与游戏 EXE 使用的标题字符串完全一致）: ");
    io::stdout().flush().context("刷新终端输出失败")?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("读取游戏名失败")?;
    let title = input.trim_end_matches(['\r', '\n']).to_owned();
    if title.is_empty() {
        bail!("游戏名不能为空");
    }
    Ok(title)
}

fn print_unpack_report(report: &UnpackReport) {
    println!("[unpack] branch={}", report.branch);
    println!("[unpack] extracted_files={}", report.entries);
    println!("[unpack] unpacked_bytes={}", report.unpacked_bytes);
    println!("[unpack] crc64=0x{:016X}", report.crc64);
    println!("[unpack] key_low32=0x{:08X}", report.key_low32);
    println!("[unpack] output={}", report.output.display());
}

fn print_pack_report(report: &PackReport) {
    println!("[pack] branch={}", report.branch);
    println!("[pack] packed_files={}", report.entries);
    println!("[pack] input_bytes={}", report.input_bytes);
    println!("[pack] output_bytes={}", report.output_bytes);
    println!("[pack] key_low32=0x{:08X}", report.key_low32);
    println!("[pack] output={}", report.output.display());
}

fn pause_before_exit() {
    print!("按 Enter 键退出...");
    let _ = io::stdout().flush();
    let mut ignored = String::new();
    let _ = io::stdin().read_line(&mut ignored);
}

#[cfg(windows)]
fn init_console_utf8() {
    const CP_UTF8: u32 = 65001;
    unsafe extern "system" {
        fn SetConsoleCP(code_page: u32) -> i32;
        fn SetConsoleOutputCP(code_page: u32) -> i32;
    }
    unsafe {
        SetConsoleCP(CP_UTF8);
        SetConsoleOutputCP(CP_UTF8);
    }
}

#[cfg(not(windows))]
fn init_console_utf8() {}
