use std::{
    error::Error as StdError,
    fmt,
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use liena_sdt::workflow::{
    OperationSummary, extract_directory, inject_directory, verify_directory,
};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Path used to prefill the interactive menu (Windows drag and drop).
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,
}

#[derive(Debug)]
struct EndOfInput;

impl fmt::Display for EndOfInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("end of input")
    }
}

impl StdError for EndOfInput {}

#[derive(Debug, Subcommand)]
enum Command {
    /// Extract structured SDT text to UTF-8 JSON files.
    Extract {
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        mapping: Option<PathBuf>,
        #[arg(long)]
        overwrite: bool,
    },
    /// Validate translations, rebuild SDT files, and copy the complete source tree.
    Inject {
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        translations: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        mapping: Option<PathBuf>,
        #[arg(long)]
        overwrite: bool,
    },
    /// Perform a byte-exact unchanged injection check without keeping output.
    Verify {
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        translations: PathBuf,
        #[arg(long)]
        mapping: Option<PathBuf>,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Extract {
            source,
            output,
            mapping,
            overwrite,
        }) => print_summary(
            "extract",
            &output,
            extract_directory(&source, &output, mapping.as_deref(), overwrite)?,
        ),
        Some(Command::Inject {
            source,
            translations,
            output,
            mapping,
            overwrite,
        }) => print_summary(
            "inject",
            &output,
            inject_directory(
                &source,
                &translations,
                &output,
                mapping.as_deref(),
                overwrite,
            )?,
        ),
        Some(Command::Verify {
            source,
            translations,
            mapping,
        }) => {
            let summary = verify_directory(&source, &translations, mapping.as_deref())?;
            println!(
                "verify: {} file(s), {} entry/entries, byte-exact",
                summary.files_scanned, summary.entries
            );
            Ok(())
        }
        None => interactive(cli.path),
    }
}

fn interactive(prefill: Option<PathBuf>) -> Result<()> {
    let mut prefill = prefill;
    loop {
        println!("\nLiena Crystal SDT tool");
        println!("  1. Extract");
        println!("  2. Inject");
        println!("  3. Verify unchanged round trip");
        println!("  4. Exit");
        let selection = match prompt("Select", Some("1")) {
            Ok(selection) => selection,
            Err(error) if error.downcast_ref::<EndOfInput>().is_some() => return Ok(()),
            Err(error) => return Err(error),
        };
        let operation = match selection.trim() {
            "1" => interactive_extract(&mut prefill),
            "2" => interactive_inject(&mut prefill),
            "3" => interactive_verify(&mut prefill),
            "4" => return Ok(()),
            _ => {
                eprintln!("unknown selection");
                continue;
            }
        };
        if let Err(error) = operation {
            if error.downcast_ref::<EndOfInput>().is_some() {
                return Ok(());
            }
            eprintln!("error: {error:#}");
        }
    }
}

fn interactive_extract(prefill: &mut Option<PathBuf>) -> Result<()> {
    let source = prompt_path("Source directory", prefill.take().as_deref())?;
    let output = prompt_path("JSON output directory", None)?;
    let mapping = prompt_optional_path("Character map (blank for CP932 only)")?;
    let Some(overwrite) = confirm_overwrite(&output)? else {
        println!("cancelled");
        return Ok(());
    };
    if !confirm_operation(
        "Extract",
        &[("source", &source), ("output", &output)],
        mapping.as_deref(),
        Some(overwrite),
    )? {
        println!("cancelled");
        return Ok(());
    }
    let summary = extract_directory(&source, &output, mapping.as_deref(), overwrite)?;
    print_summary("extract", &output, summary)
}

fn interactive_inject(prefill: &mut Option<PathBuf>) -> Result<()> {
    let source = prompt_path("Source directory", prefill.take().as_deref())?;
    let translations = prompt_path("Translation JSON directory", None)?;
    let output = prompt_path("Rebuilt output directory", None)?;
    let mapping = prompt_optional_path("Character map (blank for CP932 only)")?;
    let Some(overwrite) = confirm_overwrite(&output)? else {
        println!("cancelled");
        return Ok(());
    };
    if !confirm_operation(
        "Inject",
        &[
            ("source", &source),
            ("translations", &translations),
            ("output", &output),
        ],
        mapping.as_deref(),
        Some(overwrite),
    )? {
        println!("cancelled");
        return Ok(());
    }
    let summary = inject_directory(
        &source,
        &translations,
        &output,
        mapping.as_deref(),
        overwrite,
    )?;
    print_summary("inject", &output, summary)
}

fn interactive_verify(prefill: &mut Option<PathBuf>) -> Result<()> {
    let source = prompt_path("Source directory", prefill.take().as_deref())?;
    let translations = prompt_path("Translation JSON directory", None)?;
    let mapping = prompt_optional_path("Character map (blank for CP932 only)")?;
    if !confirm_operation(
        "Verify",
        &[("source", &source), ("translations", &translations)],
        mapping.as_deref(),
        None,
    )? {
        println!("cancelled");
        return Ok(());
    }
    let summary = verify_directory(&source, &translations, mapping.as_deref())?;
    println!(
        "verify: {} file(s), {} entry/entries, byte-exact",
        summary.files_scanned, summary.entries
    );
    Ok(())
}

fn prompt(label: &str, default: Option<&str>) -> Result<String> {
    match default {
        Some(value) => print!("{label} [{value}]: "),
        None => print!("{label}: "),
    }
    io::stdout().flush()?;
    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        return Err(EndOfInput.into());
    }
    let value = line.trim();
    Ok(if value.is_empty() {
        default.unwrap_or_default().to_owned()
    } else {
        value.to_owned()
    })
}

fn prompt_path(label: &str, default: Option<&Path>) -> Result<PathBuf> {
    let rendered = default.map(|path| path.to_string_lossy().into_owned());
    let value = prompt(label, rendered.as_deref())?;
    if value.is_empty() {
        bail!("{label} is required");
    }
    Ok(PathBuf::from(strip_drag_drop_quotes(&value)))
}

fn prompt_optional_path(label: &str) -> Result<Option<PathBuf>> {
    let value = prompt(label, None)?;
    Ok((!value.is_empty()).then(|| PathBuf::from(strip_drag_drop_quotes(&value))))
}

fn strip_drag_drop_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(value)
}

fn confirm_overwrite(output: &Path) -> Result<Option<bool>> {
    if !output.exists() {
        return Ok(Some(false));
    }
    Ok(matches!(
        prompt("Output exists; replace it? (y/N)", Some("N"))?
            .to_ascii_lowercase()
            .as_str(),
        "y" | "yes"
    )
    .then_some(true))
}

fn confirm_operation(
    action: &str,
    paths: &[(&str, &Path)],
    mapping: Option<&Path>,
    overwrite: Option<bool>,
) -> Result<bool> {
    println!("{action} settings:");
    for (role, path) in paths {
        println!("  {role}: {}", path.display());
    }
    match mapping {
        Some(path) => println!("  mapping: {}", path.display()),
        None => println!("  mapping: CP932 only"),
    }
    if let Some(overwrite) = overwrite {
        println!("  overwrite: {overwrite}");
    }
    Ok(matches!(
        prompt("Proceed? (y/N)", Some("N"))?
            .to_ascii_lowercase()
            .as_str(),
        "y" | "yes"
    ))
}

fn print_summary(action: &str, output: &Path, summary: OperationSummary) -> Result<()> {
    for warning in &summary.warnings {
        eprintln!("warning: {warning}");
    }
    println!(
        "{action}: {} file(s), {} JSON file(s), {} entry/entries, {} changed, {} mapped character(s); output {}",
        summary.files_scanned,
        summary.json_files,
        summary.entries,
        summary.changed,
        summary.substituted_characters,
        output.display()
    );
    io::stdout().flush().context("failed to print summary")
}
