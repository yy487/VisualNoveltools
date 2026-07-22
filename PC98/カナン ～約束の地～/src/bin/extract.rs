use canaan_tools::extract::{prepare_extract, ExtractError};
use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(about = "Extract confirmed Canaan main-story text to per-script UTF-8 JSON")]
struct Args {
    /// One csNN_NN.s script or an unpacked root containing System-98 .S scripts.
    input: PathBuf,

    /// New JSON file or directory. Defaults beside INPUT and never overwrites.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn default_output(input: &Path) -> Result<PathBuf, ExtractError> {
    let name = input
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ExtractError::Invalid(format!("invalid input filename: {}", input.display()))
        })?;
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    if input.is_file() {
        Ok(parent.join(format!("{name}.json")))
    } else {
        Ok(parent.join(format!("{name}_json")))
    }
}

fn run() -> Result<(), ExtractError> {
    let args = Args::parse();
    let output = match args.output {
        Some(path) => path,
        None => default_output(&args.input)?,
    };
    let prepared = prepare_extract(&args.input)?;
    prepared.write_to(&output)?;
    println!(
        "[extract] input={} scanned_files={} json_files={} extracted_entries={} skipped_entries={} warnings={} output={}",
        prepared.input.display(),
        prepared.stats.scanned_files,
        prepared.stats.json_files,
        prepared.stats.extracted_entries,
        prepared.stats.skipped_entries,
        prepared.stats.warnings,
        output.display()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
