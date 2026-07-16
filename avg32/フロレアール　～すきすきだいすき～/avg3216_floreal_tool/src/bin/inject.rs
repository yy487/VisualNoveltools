use std::env;
use std::io;
use std::path::PathBuf;

use anyhow::{Result, ensure};
use avg3216_floreal::workflow::inject_file;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    version,
    about = "AVG3216 SEEN.TXT 结构化注入器；支持把一个或多个 SEEN.TXT.json 拖到 EXE 上",
    after_help = "默认从 JSON 同目录寻找 source_file，并输出 SEEN_injected.TXT。不会覆盖已有文件。拖拽模式结束后会等待回车。"
)]
struct Args {
    #[arg(required = true, value_name = "SEEN.TXT.json")]
    inputs: Vec<PathBuf>,

    #[arg(long, value_name = "SEEN.TXT")]
    source: Option<PathBuf>,

    #[arg(short, long, value_name = "SEEN_injected.TXT")]
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
    if args.source.is_some() || args.output.is_some() {
        ensure!(
            args.inputs.len() == 1,
            "--source/--output 只允许一个 JSON 输入"
        );
    }
    for input in &args.inputs {
        let report = inject_file(input, args.source.as_deref(), args.output.as_deref())?;
        println!("[inject] json_entries={}", report.json_entries);
        println!("[inject] patched={}", report.patched);
        println!("[inject] unchanged={}", report.unchanged);
        println!("[inject] modified_files={}", report.modified_files);
        println!(
            "[inject] byte_exact_no_change={}",
            report.byte_exact_no_change
        );
        println!("[inject] output_bytes={}", report.output_bytes);
        println!("[inject] warnings=0");
        println!("[inject] output={}", report.output.display());
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
