use shangri_la1_mes::payload::{transform_input, Transform};
use shangri_la1_mes::text_json::MAX_SCRIPT_BYTES;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HELP: &str = "Shangri-La 1 MES LZSS decompressor

Usage:
  decompress.exe [--output PATH] INPUT
  decompress.exe INPUT [INPUT ...]

INPUT may be one compressed file or a flat directory produced by unpack.exe.

Options:
  -o, --output PATH  Output file or directory (single input only)
  -h, --help         Show this help

Default output:
  A.MES          -> A_decompressed.MES
  mes_unpacked/  -> mes_unpacked_decompressed/

The implementation follows this game's sub_438670 routine: a 4096-byte zeroed
window, initial write position 0xFEE, LSB-first flags, and 3..18 byte matches.
Output is limited to the runtime's 64,000-byte script buffer. Existing output
is never overwritten.";

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
        let report = transform_input(
            input,
            &output,
            Transform::Decompress {
                max_output: MAX_SCRIPT_BYTES,
            },
        )
        .map_err(|error| error.to_string())?;
        println!(
            "[decompress] input={} output={} processed_files={} compressed_bytes={} output_bytes={} warnings=0",
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
        return Err("no compressed input was provided; use --help for usage".to_string());
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
        name.push("_decompressed");
        return input.with_file_name(name);
    }

    let stem = input
        .file_stem()
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| OsStr::new("script"));
    let mut name = stem.to_os_string();
    name.push("_decompressed");
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
            PathBuf::from(r"C:\game\A_decompressed.MES")
        );
    }

    #[test]
    fn output_rejects_multiple_inputs() {
        let args = ["--output", "out", "a.mes", "b.mes"]
            .into_iter()
            .map(OsString::from);
        assert!(parse_args(args)
            .unwrap_err()
            .contains("only be used with one input"));
    }
}
