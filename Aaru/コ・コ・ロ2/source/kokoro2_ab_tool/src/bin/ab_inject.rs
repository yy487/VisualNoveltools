use kokoro2_ab::ab_workflow::{default_inject_output, inject_path};
use kokoro2_ab::{ToolError, ToolResult};
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn usage(program: &str) {
    println!(
        "Kokoro 2 AB text injector\n\nUsage:\n  {program} [SOURCE] [JSON] [--output PATH] [--overwrite]\n  {program} --non-interactive [--output PATH] [--overwrite] SOURCE JSON\n\nSOURCE and JSON must both be files or both be directories. Directory JSON paths\nmust end in .AB.json. The interactive form treats supplied paths as prefilled\nvalues and asks for confirmation before it writes. The non-interactive form\ncreates a separate output tree and never modifies SOURCE."
    );
}

fn trim_outer_quotes(value: &str) -> &str {
    let value = value.trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn read_line(prompt: &str) -> ToolResult<String> {
    print!("{prompt}");
    io::stdout()
        .flush()
        .map_err(|error| ToolError(format!("cannot flush prompt: {error}")))?;
    let mut line = String::new();
    let count = io::stdin()
        .read_line(&mut line)
        .map_err(|error| ToolError(format!("cannot read input: {error}")))?;
    if count == 0 {
        return Err(ToolError("interactive input was closed".to_string()));
    }
    Ok(line)
}

fn prompt_existing_path(label: &str, current: Option<&Path>) -> ToolResult<PathBuf> {
    loop {
        let prompt = match current {
            Some(path) => format!("{label} [{}]: ", path.display()),
            None => format!("{label}: "),
        };
        let line = read_line(&prompt)?;
        let value = trim_outer_quotes(&line);
        let path = if value.is_empty() {
            match current {
                Some(path) => path.to_path_buf(),
                None => {
                    println!("A path is required.");
                    continue;
                }
            }
        } else {
            PathBuf::from(value)
        };
        if path.exists() {
            return Ok(path);
        }
        println!("Path does not exist: {}", path.display());
    }
}

fn prompt_output_path(current: &Path) -> ToolResult<PathBuf> {
    let line = read_line(&format!("Output script path [{}]: ", current.display()))?;
    let value = trim_outer_quotes(&line);
    Ok(if value.is_empty() {
        current.to_path_buf()
    } else {
        PathBuf::from(value)
    })
}

fn prompt_yes_no(label: &str, default: bool) -> ToolResult<bool> {
    let hint = if default { "Y/n" } else { "y/N" };
    loop {
        let line = read_line(&format!("{label} [{hint}]: "))?;
        match line.trim().to_ascii_lowercase().as_str() {
            "" => return Ok(default),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("Enter y or n."),
        }
    }
}

fn report(source: &Path, json: &Path, output: Option<&Path>, overwrite: bool) -> ToolResult<()> {
    let report = inject_path(source, json, output, overwrite)?;
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

fn run() -> ToolResult<()> {
    let mut args = env::args_os();
    let program = args
        .next()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ab_inject".to_string());
    let mut output = None;
    let mut overwrite = false;
    let mut non_interactive = false;
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
        } else if argument == "--non-interactive" {
            non_interactive = true;
        } else if argument.to_string_lossy().starts_with('-') {
            return Err(ToolError(format!(
                "unknown option: {}",
                argument.to_string_lossy()
            )));
        } else {
            positional.push(PathBuf::from(argument));
        }
    }
    if positional.len() > 2 {
        usage(&program);
        return Err(ToolError("expected at most SOURCE and JSON".to_string()));
    }
    if non_interactive {
        if positional.len() != 2 {
            return Err(ToolError(
                "--non-interactive requires SOURCE and JSON".to_string(),
            ));
        }
        return report(&positional[0], &positional[1], output.as_deref(), overwrite);
    }

    let source = prompt_existing_path(
        "Source .AB file or directory",
        positional.first().map(PathBuf::as_path),
    )?;
    let json = prompt_existing_path(
        "Translation JSON file or directory",
        positional.get(1).map(PathBuf::as_path),
    )?;
    let suggested = match output {
        Some(path) => path,
        None => default_inject_output(&source),
    };
    let output = prompt_output_path(&suggested)?;
    let overwrite = prompt_yes_no("Allow overwrite when output already exists", overwrite)?;
    println!(
        "Source: {}\nJSON: {}\nOutput: {}\nOverwrite: {}",
        source.display(),
        json.display(),
        output.display(),
        overwrite
    );
    if !prompt_yes_no("Run injection", false)? {
        println!("[inject] cancelled");
        return Ok(());
    }
    report(&source, &json, Some(&output), overwrite)
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ab_inject: error: {error}");
        std::process::exit(1);
    }
}
