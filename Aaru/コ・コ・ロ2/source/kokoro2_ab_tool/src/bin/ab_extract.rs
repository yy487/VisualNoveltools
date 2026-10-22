use kokoro2_ab::ab_workflow::{default_extract_output, extract_path};
use kokoro2_ab::{ToolError, ToolResult};
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn usage(program: &str) {
    println!(
        "Kokoro 2 AB text extractor\n\nUsage:\n  {program} [INPUT] [--output PATH] [--overwrite]\n  {program} --non-interactive [--output PATH] [--overwrite] INPUT\n\nINPUT may be one .AB file or a directory tree. The interactive form treats a\npath-only invocation as a prefilled value and asks for confirmation before it\nwrites. The non-interactive form writes one UTF-8 JSON file per .AB script;\nexisting output is refused unless --overwrite is supplied."
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
    let line = read_line(&format!("Output JSON path [{}]: ", current.display()))?;
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

fn report(input: &Path, output: Option<&Path>, overwrite: bool) -> ToolResult<()> {
    let report = extract_path(input, output, overwrite)?;
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

fn run() -> ToolResult<()> {
    let mut args = env::args_os();
    let program = args
        .next()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ab_extract".to_string());
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
    if positional.len() > 1 {
        usage(&program);
        return Err(ToolError("expected at most one INPUT".to_string()));
    }
    if non_interactive {
        let input = positional
            .into_iter()
            .next()
            .ok_or_else(|| ToolError("--non-interactive requires exactly one INPUT".to_string()))?;
        return report(&input, output.as_deref(), overwrite);
    }

    let input = prompt_existing_path(
        "Input .AB file or directory",
        positional.first().map(PathBuf::as_path),
    )?;
    let suggested = match output {
        Some(path) => path,
        None => default_extract_output(&input)?,
    };
    let output = prompt_output_path(&suggested)?;
    let overwrite = prompt_yes_no("Allow overwrite when output already exists", overwrite)?;
    println!(
        "Input: {}\nOutput: {}\nOverwrite: {}",
        input.display(),
        output.display(),
        overwrite
    );
    if !prompt_yes_no("Run extraction", false)? {
        println!("[extract] cancelled");
        return Ok(());
    }
    report(&input, Some(&output), overwrite)
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ab_extract: error: {error}");
        std::process::exit(1);
    }
}
