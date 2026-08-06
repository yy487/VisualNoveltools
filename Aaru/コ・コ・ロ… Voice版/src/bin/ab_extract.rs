use kokorov_fl2::ab_workflow::extract_path;
use kokorov_fl2::{ToolError, ToolResult};
use std::env;
use std::path::PathBuf;

fn usage(program: &str) {
    println!(
        "KOKOROV AB text extractor\n\nUsage:\n  {program} [--output PATH] [--overwrite] INPUT\n\nINPUT may be one .AB file or a directory tree. A file produces INPUT.json; a\ndirectory produces a sibling <name>_json directory containing one UTF-8 JSON\nfile per .AB script. Existing output is refused unless --overwrite is used.\nWindows drag-and-drop is equivalent to passing INPUT."
    );
}

fn run() -> ToolResult<()> {
    let mut args = env::args_os();
    let program = args
        .next()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ab_extract".to_string());
    let mut output = None;
    let mut overwrite = false;
    let mut positional = Vec::new();
    while let Some(argument) = args.next() {
        if argument == "-h" || argument == "--help" {
            usage(&program);
            return Ok(());
        } else if argument == "--output" || argument == "-o" {
            output =
                Some(PathBuf::from(args.next().ok_or_else(|| {
                    ToolError("--output requires PATH".to_string())
                })?));
        } else if argument == "--overwrite" {
            overwrite = true;
        } else if argument.to_string_lossy().starts_with('-') {
            return Err(ToolError(format!(
                "unknown option: {}",
                argument.to_string_lossy()
            )));
        } else {
            positional.push(PathBuf::from(argument));
        }
    }
    if positional.len() != 1 {
        usage(&program);
        return Err(ToolError("expected exactly one INPUT".to_string()));
    }
    let report = extract_path(&positional[0], output.as_deref(), overwrite)?;
    println!(
        "[extract] scanned_files={} json_files={} extracted_entries={} warnings={} output={}",
        report.scanned_files,
        report.json_files,
        report.extracted_entries,
        report.warnings,
        report.output.display()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ab_extract: error: {error}");
        std::process::exit(1);
    }
}
