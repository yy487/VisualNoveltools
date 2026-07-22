use std::env;
use std::path::PathBuf;

use qc_keisei_tools::cli;

const HELP: &str = "Adviz for Windows95 (WADVIZ) ADV text extractor\n\nUsage:\n  qc_extract.exe [--output PATH] INPUT.ADV\n  qc_extract.exe [--output DIRECTORY] INPUT_DIRECTORY\n\nDefaults:\n  A01.ADV -> A01.ADV.json\n  TEXT/    -> TEXT_json/\n\nThe output must not already exist. Directory mode writes one UTF-8 JSON file per ADV file.\n";

fn parse_args() -> Result<(PathBuf, Option<PathBuf>), String> {
    let mut input = None;
    let mut output = None;
    let mut args = env::args_os().skip(1);
    while let Some(argument) = args.next() {
        if argument == "-h" || argument == "--help" {
            print!("{HELP}");
            std::process::exit(0);
        }
        if argument == "--output" || argument == "-o" {
            let value = args
                .next()
                .ok_or_else(|| "--output requires a path".to_owned())?;
            if output.replace(PathBuf::from(value)).is_some() {
                return Err("--output may only be specified once".to_owned());
            }
            continue;
        }
        let path = PathBuf::from(argument);
        if input.replace(path).is_some() {
            return Err("expected exactly one input path".to_owned());
        }
    }
    Ok((
        input.ok_or_else(|| "missing input path".to_owned())?,
        output,
    ))
}

fn main() {
    let result = parse_args().and_then(|(input, output)| cli::extract(&input, output.as_deref()));
    match result {
        Ok(report) => {
            println!("[extract] scanned_files={}", report.scanned_files);
            println!("[extract] json_files={}", report.json_files);
            println!("[extract] extracted_entries={}", report.extracted_entries);
            println!("[extract] skipped={}", report.skipped);
            println!("[extract] warnings={}", report.warnings);
            println!("[extract] output={}", report.output.display());
        }
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("\n{HELP}");
            std::process::exit(1);
        }
    }
}
