use shangri_la1_mes::text_json::inject_input;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HELP: &str = "Shangri-La 1 MES UTF-8 JSON injector

Usage:
  inject.exe [--output PATH] SOURCE TRANSLATIONS

SOURCE is one decompressed MES/LIB file or a flat decompressed directory.
TRANSLATIONS is its JSON file or the directory produced by extract.exe.

Options:
  -o, --output PATH  Output script file or directory
  -h, --help         Show this help

Default output:
  A.MES + A.MES.json       -> A_injected.MES
  scripts/ + scripts_json/ -> scripts_injected/

Only message is writable. All source metadata, scr_msg, and optional name
fields are validated. Code entry points and jump targets are relocated for
variable-length CP932 text. Existing output is never overwritten.";

#[derive(Debug)]
struct Args {
    output: Option<PathBuf>,
    source: PathBuf,
    translations: PathBuf,
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
    let output = args.output.unwrap_or_else(|| default_output(&args.source));
    let report = inject_input(&args.source, &args.translations, &output)
        .map_err(|error| error.to_string())?;
    println!(
        "[inject] source={} translations={} output={} source_files={} json_files={} json_entries={} patched={} unchanged={} output_files={} warnings={}",
        args.source.display(),
        args.translations.display(),
        output.display(),
        report.source_files,
        report.json_files,
        report.json_entries,
        report.patched,
        report.unchanged,
        report.output_files,
        report.warnings
    );
    Ok(())
}

fn parse_args(mut args: impl Iterator<Item = OsString>) -> Result<Option<Args>, String> {
    let mut output = None;
    let mut positional = Vec::new();
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
        positional.push(PathBuf::from(arg));
    }
    if positional.len() != 2 {
        return Err(
            "inject requires exactly SOURCE and TRANSLATIONS; use --help for usage".to_string(),
        );
    }
    Ok(Some(Args {
        output,
        source: positional.remove(0),
        translations: positional.remove(0),
    }))
}

fn default_output(source: &Path) -> PathBuf {
    if source.is_dir() {
        let mut name = source
            .file_name()
            .unwrap_or_else(|| OsStr::new("scripts"))
            .to_os_string();
        name.push("_injected");
        return source.with_file_name(name);
    }
    let stem = source
        .file_stem()
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| OsStr::new("script"));
    let mut name = stem.to_os_string();
    name.push("_injected");
    if let Some(extension) = source.extension() {
        name.push(".");
        name.push(extension);
    }
    source.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_output_preserves_extension() {
        assert_eq!(
            default_output(Path::new(r"C:\game\A.MES")),
            PathBuf::from(r"C:\game\A_injected.MES")
        );
    }
}
