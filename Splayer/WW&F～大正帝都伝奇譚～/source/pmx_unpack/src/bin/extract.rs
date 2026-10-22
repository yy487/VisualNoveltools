use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use pmx_unpack::{default_extract_output, extract_input, PmxError, Result};

struct Options {
    inputs: Vec<PathBuf>,
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[extract][error] {error}");
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
            None => default_extract_output(input)?,
        };
        match extract_input(input, &output) {
            Ok(report) => {
                for warning in &report.warnings {
                    eprintln!(
                        "[extract][warn] file={} line={}: {}",
                        warning.file, warning.line, warning.message
                    );
                }
                println!("[extract] input={}", input.display());
                println!("[extract] scanned_files={}", report.scanned_files);
                println!("[extract] json_files={}", report.json_files);
                println!("[extract] extracted_entries={}", report.extracted_entries);
                println!("[extract] warnings={}", report.warnings.len());
                println!("[extract] output={}", report.output.display());
            }
            Err(error) => {
                failed += 1;
                eprintln!("[extract][error] input={}: {error}", input.display());
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
            "没有输入 PMX 文件或目录；可直接拖到 extract.exe 上",
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
                        .ok_or_else(|| PmxError::new(format!("{text} 缺少路径参数")))?;
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
        return Err(PmxError::new("没有输入 PMX 文件或目录"));
    }
    if output.is_some() && inputs.len() != 1 {
        return Err(PmxError::new(
            "--output 只能与一个输入一起使用；批量拖放使用默认输出名",
        ));
    }
    Ok(Some(Options { inputs, output }))
}

fn print_usage() {
    println!("extract 0.3.0 - Splayer PutText JSON 提取工具");
    println!();
    println!("用法：extract.exe [-o 输出] <PMX 文件或目录>...");
    println!("也可以直接拖放文件或目录到 extract.exe 上。");
}
