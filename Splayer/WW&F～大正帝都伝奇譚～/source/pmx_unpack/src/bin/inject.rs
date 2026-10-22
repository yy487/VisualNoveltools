use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use pmx_unpack::{default_inject_output, infer_source_path, inject_input, PmxError, Result};

struct Options {
    inputs: Vec<PathBuf>,
    source: Option<PathBuf>,
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[inject][error] {error}");
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
        let source = match &options.source {
            Some(source) => source.clone(),
            None => infer_source_path(input)?,
        };
        let output = match &options.output {
            Some(output) => output.clone(),
            None => default_inject_output(&source)?,
        };
        match inject_input(input, &source, &output) {
            Ok(report) => {
                println!("[inject] json_input={}", input.display());
                println!("[inject] source={}", source.display());
                println!("[inject] json_files={}", report.json_files);
                println!("[inject] json_entries={}", report.json_entries);
                println!("[inject] patched={}", report.patched);
                println!("[inject] unchanged={}", report.unchanged);
                println!("[inject] copied_files={}", report.copied_files);
                println!("[inject] output={}", report.output.display());
            }
            Err(error) => {
                failed += 1;
                eprintln!("[inject][error] input={}: {error}", input.display());
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
            "没有输入 JSON 文件或目录；可直接拖到 inject.exe 上",
        ));
    }
    let mut inputs = Vec::new();
    let mut source = None;
    let mut output = None;
    let mut positional_only = false;
    let mut index = 0usize;
    while index < args.len() {
        let argument = &args[index];
        let text = argument.to_string_lossy();
        if !positional_only {
            match text.as_ref() {
                "-h" | "--help" => return Ok(None),
                "-s" | "--source" => {
                    index += 1;
                    let value = args
                        .get(index)
                        .ok_or_else(|| PmxError::new(format!("{text} 缺少路径参数")))?;
                    if source.replace(PathBuf::from(value)).is_some() {
                        return Err(PmxError::new("只能指定一次 --source"));
                    }
                    index += 1;
                    continue;
                }
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
        return Err(PmxError::new("没有输入 JSON 文件或目录"));
    }
    if (source.is_some() || output.is_some()) && inputs.len() != 1 {
        return Err(PmxError::new(
            "--source/--output 只能与一个输入一起使用；批量拖放使用自动推断",
        ));
    }
    Ok(Some(Options {
        inputs,
        source,
        output,
    }))
}

fn print_usage() {
    println!("inject 0.3.0 - Splayer PutText JSON 注入工具");
    println!();
    println!("用法：inject.exe [-s 源 PMX/目录] [-o 输出] <JSON 文件或目录>...");
    println!("也可以把 extract 生成的 JSON 文件或 *_json 目录拖到 inject.exe 上。");
}
