use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use pmx_unpack::{default_output_dir, unpack_file, PmxError, Result};

struct Options {
    inputs: Vec<PathBuf>,
    output_root: Option<PathBuf>,
    force_raw: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[unpack][error] {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let Some(options) = parse_args(env::args_os().skip(1).collect())? else {
        print_usage();
        return Ok(());
    };

    if let Some(output_root) = &options.output_root {
        fs::create_dir_all(output_root).map_err(|error| {
            PmxError::context(
                format!("创建输出根目录失败：{}", output_root.display()),
                error,
            )
        })?;
    }

    let mut failed = 0usize;
    for input in &options.inputs {
        let output_dir = match &options.output_root {
            Some(root) => {
                let default = default_output_dir(input)?;
                let name = default.file_name().ok_or_else(|| {
                    PmxError::new(format!("无法生成输出目录名：{}", input.display()))
                })?;
                root.join(name)
            }
            None => default_output_dir(input)?,
        };

        match unpack_file(input, &output_dir, options.force_raw) {
            Ok(report) => {
                println!("[unpack] input={}", input.display());
                println!("[unpack] kind={}", report.kind.as_str());
                println!("[unpack] input_bytes={}", report.input_bytes);
                println!("[unpack] decoded_bytes={}", report.decoded_bytes);
                println!("[unpack] extracted_files={}", report.extracted_files);
                println!("[unpack] output={}", report.output_dir.display());
            }
            Err(error) => {
                failed += 1;
                eprintln!("[unpack][error] input={}: {error}", input.display());
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
            "没有输入文件；可把 PMX 文件拖到 pmx_unpack.exe 上",
        ));
    }

    let mut inputs = Vec::new();
    let mut output_root = None;
    let mut force_raw = false;
    let mut positional_only = false;
    let mut index = 0usize;

    while index < args.len() {
        let argument = &args[index];
        let text = argument.to_string_lossy();
        if !positional_only {
            match text.as_ref() {
                "-h" | "--help" => return Ok(None),
                "--raw" => {
                    force_raw = true;
                    index += 1;
                    continue;
                }
                "-o" | "--output" => {
                    index += 1;
                    let value = args
                        .get(index)
                        .ok_or_else(|| PmxError::new(format!("{text} 缺少目录参数")))?;
                    if output_root.replace(PathBuf::from(value)).is_some() {
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
        return Err(PmxError::new("没有输入 PMX 文件"));
    }

    Ok(Some(Options {
        inputs,
        output_root,
        force_raw,
    }))
}

fn print_usage() {
    println!("pmx_unpack 0.1.0 - Splayer PMX 解包工具");
    println!();
    println!("用法：");
    println!("  pmx_unpack.exe [选项] <PMX 文件>...");
    println!();
    println!("选项：");
    println!("  -o, --output <目录>  指定输出根目录");
    println!("      --raw            只解开 XOR/zlib 外层，不拆 Data 文件表");
    println!("  -h, --help           显示帮助");
    println!();
    println!("也可以把一个或多个 PMX 文件直接拖到 pmx_unpack.exe 上。");
}
