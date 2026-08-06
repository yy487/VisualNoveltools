use kokorov_fl2::ab_workflow::inject_path;
use kokorov_fl2::{ToolError, ToolResult};
use std::env;
use std::path::PathBuf;

fn usage(program: &str) {
    println!(
        "KOKOROV AB text injector\n\nUsage:\n  {program} [--output PATH] [--overwrite] SOURCE JSON\n\nSOURCE and JSON must both be files or both be directories. For directories,\nJSON paths must end in .AB.json. The complete source tree is copied to a sibling\n<name>_injected directory, then translated scripts are replaced. Existing\noutput is refused unless --overwrite is used. Source files are never modified."
    );
}

fn run() -> ToolResult<()> {
    let mut args = env::args_os();
    let program = args
        .next()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ab_inject".to_string());
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
    if positional.len() != 2 {
        usage(&program);
        return Err(ToolError("expected SOURCE and JSON".to_string()));
    }
    let report = inject_path(&positional[0], &positional[1], output.as_deref(), overwrite)?;
    println!(
        "[inject] json_files={} json_entries={} patched={} unchanged={} failed=0 warnings={} output={}",
        report.json_files,
        report.json_entries,
        report.patched,
        report.unchanged,
        report.warnings,
        report.output.display()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ab_inject: error: {error}");
        std::process::exit(1);
    }
}
