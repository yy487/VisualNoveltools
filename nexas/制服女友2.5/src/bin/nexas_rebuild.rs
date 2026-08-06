use std::env;
use std::process::ExitCode;
use uniform_kanojo_nexas_tool::cli::{parse_output, print_rebuild_help, require_path};
use uniform_kanojo_nexas_tool::workflow::{default_rebuild_output, rebuild};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_rebuild_help();
        return ExitCode::SUCCESS;
    }
    match run(&args) {
        Ok(stats) => {
            println!(
                "[rebuild] scanned_files={} rebuilt_files={} byte_exact={} warnings={}",
                stats.scanned_files, stats.rebuilt_files, stats.byte_exact, stats.warnings
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("[rebuild][error] {error}");
            ExitCode::from(1)
        }
    }
}

fn run(
    args: &[String],
) -> uniform_kanojo_nexas_tool::ToolResult<uniform_kanojo_nexas_tool::workflow::RebuildStats> {
    let (paths, output) = parse_output(args, 1)?;
    require_path(&paths[0], "input")?;
    let output = output.unwrap_or_else(|| default_rebuild_output(&paths[0]));
    rebuild(&paths[0], &output)
}
