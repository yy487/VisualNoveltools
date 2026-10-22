use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use pmx_unpack::{default_packed_path, pack_directory, PmxError, Result};

struct Options {
    inputs: Vec<PathBuf>,
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[pack][error] {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let Some(options) = parse_args(env::args_os().skip(1).collect())? else {
        print_usage();
        return Ok(());
    };

    let mut failed = 0usize;
    for input in &options.inputs {
        let output = match &options.output {
            Some(output) => output.clone(),
            None => default_packed_path(input)?,
        };
        match pack_directory(input, &output) {
            Ok(report) => {
                println!("[pack] input={}", input.display());
                println!("[pack] kind={}", report.kind.as_str());
                println!("[pack] packed_files={}", report.packed_files);
                println!("[pack] decoded_bytes={}", report.decoded_bytes);
                println!("[pack] output_bytes={}", report.output_bytes);
                println!("[pack] reused_original={}", report.reused_original);
                println!("[pack] output={}", report.output_file.display());
            }
            Err(error) => {
                failed += 1;
                eprintln!("[pack][error] input={}: {error}", input.display());
            }
        }
    }

    if failed != 0 {
        return Err(PmxError::new(format!(
            "处理结束：成功={}，失败={failed}",
            options.inputs.len() - failed
        )));
    }
    Ok(())
}

fn parse_args(args: Vec<OsString>) -> Result<Option<Options>> {
    if args.is_empty() {
        return Err(PmxError::new(
            "没有输入目录；可把 pmx_unpack 生成的目录拖到 pmx_pack.exe 上",
        ));
    }

    let mut inputs = Vec::new();
    let mut output = None;
    let mut positional_only = false;
    let mut index = 0usize;

    while index < args.len() {
        let argument = &args[index];
        let text = argument.to_string_lossy();
        if !positional_only {
            match text.as_ref() {
                "-h" | "--help" => return Ok(None),
                "-o" | "--output" => {
                    index += 1;
                    let value = args
                        .get(index)
                        .ok_or_else(|| PmxError::new(format!("{text} 缺少文件参数")))?;
                    if output.replace(PathBuf::from(value)).is_some() {
                        return Err(PmxError::new("只能指定一次 --output"));
                    }
                    index += 1;
                    continue;
                }
                "--" => {
                    positional_only = true;
                    index += 1;
                    continue;
                }
                _ if text.starts_with('-') => {
                    return Err(PmxError::new(format!("未知参数：{text}")));
                }
                _ => {}
            }
        }

        inputs.push(PathBuf::from(argument));
        index += 1;
    }

    if inputs.is_empty() {
        return Err(PmxError::new("没有输入解包目录"));
    }
    if output.is_some() && inputs.len() != 1 {
        return Err(PmxError::new(
            "--output 只能与一个输入目录一起使用；批量拖放时使用默认输出名",
        ));
    }
    Ok(Some(Options { inputs, output }))
}

fn print_usage() {
    println!("pmx_pack 0.2.0 - Splayer PMX 封包工具");
    println!();
    println!("用法：");
    println!("  pmx_pack.exe [选项] <解包目录>...");
    println!();
    println!("选项：");
    println!("  -o, --output <文件>  单个输入时指定输出 PMX 文件");
    println!("  -h, --help           显示帮助");
    println!();
    println!("也可以把一个或多个 *_unpacked 目录直接拖到 pmx_pack.exe 上。");
}
