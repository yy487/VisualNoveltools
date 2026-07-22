use shangri_la1_mes::text_json::extract_input;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HELP: &str = "Shangri-La 1 MES UTF-8 JSON extractor

Usage:
  extract.exe [--output PATH] INPUT
  extract.exe INPUT [INPUT ...]

INPUT may be one decompressed MES/LIB file or a flat decompressed directory.

Options:
  -o, --output PATH  Output JSON file or directory (single input only)
  -h, --help         Show this help

Default output:
  A.MES       -> A.MES.json
  scripts/    -> scripts_json/

Named dialogue uses the confirmed leading fullwidth ［name］ prefix. The JSON
keeps name and scr_msg immutable; translators edit message only. Existing
output is never overwritten.";

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
        let report = extract_input(input, &output)
            .map_err(|error| format!("{}: {error}", input.display()))?;
        println!(
            "[extract] input={} output={} scanned_files={} json_files={} extracted_entries={} skipped={} warnings={}",
            input.display(),
            output.display(),
            report.scanned_files,
            report.json_files,
            report.extracted_entries,
            report.skipped,
            report.warnings
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
        return Err("no decompressed script input was provided; use --help for usage".to_string());
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
        name.push("_json");
        return input.with_file_name(name);
    }
    let mut name = input
        .file_name()
        .unwrap_or_else(|| OsStr::new("script.MES"))
        .to_os_string();
    name.push(".json");
    input.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_output_appends_json_extension() {
        assert_eq!(
            default_output(Path::new(r"C:\game\A.MES")),
            PathBuf::from(r"C:\game\A.MES.json")
        );
    }
}
