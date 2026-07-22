use shangri_la1_mes::archive::pack_directory;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HELP: &str = "Shangri-La 1 MES.ARC packer

Usage:
  pack.exe [--output ARCHIVE] TEMPLATE_ARCHIVE INPUT_DIR

TEMPLATE_ARCHIVE supplies the verified filename set and original entry order.
INPUT_DIR must be a flat directory containing exactly those compressed files.

Options:
  -o, --output ARCHIVE  Output archive path
  -h, --help            Show this help

Default output:
  mes.arc + scripts_compressed/ -> mes_packed.arc

The complete template and input directory are validated before writing.
Existing output is never overwritten.";

#[derive(Debug)]
struct Args {
    output: Option<PathBuf>,
    template: PathBuf,
    input: PathBuf,
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
    let output = args
        .output
        .unwrap_or_else(|| default_output(&args.template));
    let report =
        pack_directory(&args.template, &args.input, &output).map_err(|error| error.to_string())?;
    println!(
        "[pack] template={} input={} output={} packed_files={} payload_bytes={} output_bytes={} warnings=0",
        args.template.display(),
        args.input.display(),
        output.display(),
        report.packed_files,
        report.payload_bytes,
        report.output_bytes
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
                    .ok_or_else(|| "--output requires an archive path".to_string())?
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
            "pack requires exactly TEMPLATE_ARCHIVE and INPUT_DIR; use --help for usage"
                .to_string(),
        );
    }
    Ok(Some(Args {
        output,
        template: positional.remove(0),
        input: positional.remove(0),
    }))
}

fn default_output(template: &Path) -> PathBuf {
    let stem = template
        .file_stem()
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| OsStr::new("archive"));
    let mut name = stem.to_os_string();
    name.push("_packed");
    if let Some(extension) = template.extension() {
        name.push(".");
        name.push(extension);
    }
    template.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_output_preserves_extension() {
        assert_eq!(
            default_output(Path::new(r"C:\game\mes.arc")),
            PathBuf::from(r"C:\game\mes_packed.arc")
        );
    }
}
