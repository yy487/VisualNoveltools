use shangri_la1_mes::payload::{transform_input, Transform};
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HELP: &str = "Shangri-La 1 MES LZSS compressor

Usage:
  compress.exe [--output PATH] INPUT
  compress.exe INPUT [INPUT ...]

INPUT may be one decompressed script or a flat directory produced by
inject.exe/decompress.exe.

Options:
  -o, --output PATH  Output file or directory (single input only)
  -h, --help         Show this help

Default output:
  A.MES                 -> A_compressed.MES
  scripts_injected/     -> scripts_injected_compressed/

The output follows this game's sub_438670-compatible 4 KiB LZSS stream.
Existing output is never overwritten.";

#[derive(Debug)]
struct Args {
    output: Option<PathBuf>,
    inputs: Vec<PathBuf>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let Some(args) = parse_args(env::args_os().skip(1))? else {
        println!("{HELP}");
        return Ok(());
    };
    for input in &args.inputs {
        let output = args.output.clone().unwrap_or_else(|| default_output(input));
        let report = transform_input(input, &output, Transform::Compress)
            .map_err(|error| error.to_string())?;
        println!(
            "[compress] input={} output={} processed_files={} input_bytes={} compressed_bytes={} warnings=0",
            input.display(),
            output.display(),
            report.processed_files,
            report.input_bytes,
            report.output_bytes
        );
    }
    Ok(())
}

fn parse_args(mut args: impl Iterator<Item = OsString>) -> Result<Option<Args>, String> {
    let mut output = None;
    let mut inputs = Vec::new();
    while let Some(arg) = args.next() {
        if arg == "-h" || arg == "--help" {
            return Ok(None);
        }
        if arg == "-o" || arg == "--output" {
            if output.is_some() {
                return Err("--output may only be specified once".to_string());
            }
            output = Some(
                args.next()
                    .ok_or_else(|| "--output requires a path".to_string())?
                    .into(),
            );
            continue;
        }
        if arg.to_string_lossy().starts_with('-') {
            return Err(format!("unknown option: {}", arg.to_string_lossy()));
        }
        inputs.push(arg.into());
    }
    if inputs.is_empty() {
        return Err("no decompressed input was provided; use --help for usage".to_string());
    }
    if output.is_some() && inputs.len() != 1 {
        return Err("--output can only be used with one input".to_string());
    }
    Ok(Some(Args { output, inputs }))
}

fn default_output(input: &Path) -> PathBuf {
    if input.is_dir() {
        let mut name = input
            .file_name()
            .unwrap_or_else(|| OsStr::new("scripts"))
            .to_os_string();
        name.push("_compressed");
        return input.with_file_name(name);
    }
    let stem = input
        .file_stem()
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| OsStr::new("script"));
    let mut name = stem.to_os_string();
    name.push("_compressed");
    if let Some(extension) = input.extension() {
        name.push(".");
        name.push(extension);
    }
    input.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_output_preserves_extension() {
        assert_eq!(
            default_output(Path::new(r"C:\game\A.MES")),
            PathBuf::from(r"C:\game\A_compressed.MES")
        );
    }
}
