use shangri_la1_mes::archive::unpack_file;
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HELP: &str = "Shangri-La 1 MES.ARC unpacker

Usage:
  unpack.exe [--output DIR] ARCHIVE
  unpack.exe ARCHIVE [ARCHIVE ...]

Options:
  -o, --output DIR  Output directory (single input only)
  -h, --help        Show this help

Default output:
  mes.arc -> mes_unpacked\n
The tool validates the complete archive before writing and refuses to overwrite
an existing output directory. Multiple positional paths support Windows drag
and drop.";

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
        let report =
            unpack_file(input, &output).map_err(|error| format!("{}: {error}", input.display()))?;
        println!(
            "[unpack] input={} output={} extracted_files={} payload_bytes={} archive_bytes={} warnings=0",
            input.display(),
            output.display(),
            report.extracted_files,
            report.payload_bytes,
            report.archive_bytes
        );
    }
    Ok(())
}

fn parse_args(mut args: impl Iterator<Item = std::ffi::OsString>) -> Result<Option<Args>, String> {
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
                    .ok_or_else(|| "--output requires a directory path".to_string())?
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
        return Err("no archive input was provided; use --help for usage".to_string());
    }
    if output.is_some() && inputs.len() != 1 {
        return Err("--output can only be used with one input archive".to_string());
    }
    Ok(Some(Args { output, inputs }))
}

fn default_output(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| std::ffi::OsStr::new("archive"));
    let mut name = stem.to_os_string();
    name.push("_unpacked");
    input.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_output_uses_unpacked_suffix() {
        assert_eq!(
            default_output(Path::new(r"C:\game\mes.arc")),
            PathBuf::from(r"C:\game\mes_unpacked")
        );
    }

    #[test]
    fn output_rejects_multiple_inputs() {
        let args = ["--output", "out", "a.arc", "b.arc"]
            .into_iter()
            .map(std::ffi::OsString::from);
        assert!(parse_args(args)
            .unwrap_err()
            .contains("only be used with one input"));
    }
}
