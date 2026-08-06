use std::env;
use std::process::ExitCode;
use uniform_kanojo_nexas_tool::cli::{parse_output, print_extract_help, require_path};
use uniform_kanojo_nexas_tool::workflow::{default_extract_output, extract};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_extract_help();
        return ExitCode::SUCCESS;
    }
    match run(&args) {
        Ok(stats) => {
            println!(
                "[extract] scanned_files={} json_files={} extracted_entries={} skipped={} warnings={}",
                stats.scanned_files,
                stats.json_files,
                stats.extracted_entries,
                stats.skipped,
                stats.warnings
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("[extract][error] {error}");
            ExitCode::from(1)
        }
    }
}

fn run(
    args: &[String],
) -> uniform_kanojo_nexas_tool::ToolResult<uniform_kanojo_nexas_tool::workflow::ExtractStats> {
    let (paths, output) = parse_output(args, 1)?;
    require_path(&paths[0], "input")?;
    let output = output.unwrap_or_else(|| default_extract_output(&paths[0]));
    extract(&paths[0], &output)
}
