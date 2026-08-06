use std::env;
use std::process::ExitCode;
use uniform_kanojo_nexas_tool::cli::{parse_output, print_inject_help, require_path};
use uniform_kanojo_nexas_tool::workflow::{default_inject_output, inject};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_inject_help();
        return ExitCode::SUCCESS;
    }
    match run(&args) {
        Ok(stats) => {
            println!(
                "[inject] json_entries={} patched={} unchanged={} failed={} warnings={}",
                stats.json_entries, stats.patched, stats.unchanged, stats.failed, stats.warnings
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("[inject][error] {error}");
            ExitCode::from(1)
        }
    }
}

fn run(
    args: &[String],
) -> uniform_kanojo_nexas_tool::ToolResult<uniform_kanojo_nexas_tool::workflow::InjectStats> {
    let (paths, output) = parse_output(args, 2)?;
    require_path(&paths[0], "source")?;
    require_path(&paths[1], "JSON input")?;
    let output = output.unwrap_or_else(|| default_inject_output(&paths[0]));
    inject(&paths[0], &paths[1], &output)
}
