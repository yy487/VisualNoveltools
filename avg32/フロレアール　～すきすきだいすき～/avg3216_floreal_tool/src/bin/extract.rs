use std::env;
use std::io;
use std::path::PathBuf;

use anyhow::{Result, ensure};
use avg3216_floreal::workflow::extract_file;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    version,
    about = "AVG3216 SEEN.TXT 结构化提取器；支持把一个或多个 SEEN.TXT 拖到 EXE 上",
    after_help = "默认输出：SEEN.TXT.json。不会覆盖已有文件。拖拽模式结束后会等待回车。"
)]
struct Args {
    #[arg(required = true, value_name = "SEEN.TXT")]
    inputs: Vec<PathBuf>,

    #[arg(short, long, value_name = "JSON")]
    output: Option<PathBuf>,

    #[arg(long, help = "结束后不等待回车，适合命令行和自动化")]
    no_pause: bool,
}

fn pause() {
    eprintln!("按回车退出……");
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}

fn run(args: &Args) -> Result<()> {
    if args.output.is_some() {
        ensure!(args.inputs.len() == 1, "--output 只允许一个输入");
    }
    for input in &args.inputs {
        let report = extract_file(input, args.output.as_deref())?;
        println!("[extract] scanned_files={}", report.scanned_files);
        println!("[extract] instructions={}", report.instruction_count);
        println!("[extract] extracted_entries={}", report.extracted_entries);
        println!("[extract] message_entries={}", report.message_entries);
        println!("[extract] choice_entries={}", report.choice_entries);
        println!("[extract] warnings=0");
        println!("[extract] output={}", report.output.display());
    }
    Ok(())
}

fn main() {
    let raw_args = env::args_os().collect::<Vec<_>>();
    let drag_like = raw_args.len() > 1
        && raw_args
            .iter()
            .skip(1)
            .all(|arg| !arg.to_string_lossy().starts_with('-'));
    let args = Args::parse();
    let should_pause = drag_like && !args.no_pause;
    if let Err(error) = run(&args) {
        eprintln!("[error] {error:#}");
        if should_pause {
            pause();
        }
        std::process::exit(1);
    }
    if should_pause {
        pause();
    }
}
