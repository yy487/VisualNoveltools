use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;
use yuyake_mes_tools::workflow::{default_extract_output, extract_path};

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Extract Yuyake -November- MES messages to UTF-8 JSON"
)]
struct Args {
    /// MES file(s) or directory/directories. Windows drag-and-drop is supported.
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    /// Output JSON file or directory. Only valid with one input.
    #[arg(short, long)]
    output: Option<PathBuf>,
}
fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    if args.output.is_some() && args.inputs.len() != 1 {
        bail!("--output can only be used with one input");
    }

    let mut scanned_files = 0usize;
    let mut json_files = 0usize;
    let mut extracted_entries = 0usize;
    let mut skipped_empty_pages = 0usize;
    let mut warnings = 0usize;

    for input in &args.inputs {
        let output = match &args.output {
            Some(path) => path.clone(),
            None => default_extract_output(input)?,
        };
        let report = extract_path(input, &output)?;
        scanned_files += report.scanned_files;
        json_files += report.json_files;
        extracted_entries += report.extracted_entries;
        skipped_empty_pages += report.skipped_empty_pages;
        warnings += report.warnings;
        println!("[extract] output={}", report.output.display());
    }

    println!("[extract] scanned_files={scanned_files}");
    println!("[extract] json_files={json_files}");
    println!("[extract] extracted_entries={extracted_entries}");
    println!("[extract] skipped_empty_pages={skipped_empty_pages}");
    println!("[extract] warnings={warnings}");
    Ok(())
}
