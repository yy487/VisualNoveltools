use canaan_tools::inject::{prepare_inject, InjectError};
use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(about = "Inject validated Canaan UTF-8 JSON into new System-98 scripts")]
struct Args {
    /// Original csNN_NN.s script or unpacked source root. It is never overwritten.
    source: PathBuf,

    /// One matching JSON file or a directory containing per-script .S.json files.
    translations: PathBuf,

    /// New script file or copied source directory. Defaults beside SOURCE.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn default_output(source: &Path) -> Result<PathBuf, InjectError> {
    let name = source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            InjectError::Invalid(format!("invalid source filename: {}", source.display()))
        })?;
    let parent = source.parent().unwrap_or_else(|| Path::new("."));
    if source.is_file() {
        let stem = source
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| {
                InjectError::Invalid(format!("invalid source filename: {}", source.display()))
            })?;
        let extension = source
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("s");
        Ok(parent.join(format!("{stem}_injected.{extension}")))
    } else {
        Ok(parent.join(format!("{name}_injected")))
    }
}

fn run() -> Result<(), InjectError> {
    let args = Args::parse();
    let output = match args.output {
        Some(path) => path,
        None => default_output(&args.source)?,
    };
    let prepared = prepare_inject(&args.source, &args.translations)?;
    prepared.write_to(&output)?;
    println!(
        "[inject] source={} translations={} json_files={} json_entries={} patched={} unchanged={} in_place={} relocated_entries={} appended_streams={} appended_bytes={} output_script_bytes={} warnings={} output={}",
        prepared.source.display(),
        prepared.translations.display(),
        prepared.stats.json_files,
        prepared.stats.json_entries,
        prepared.stats.patched,
        prepared.stats.unchanged,
        prepared.stats.in_place,
        prepared.stats.relocated_entries,
        prepared.stats.appended_streams,
        prepared.stats.appended_bytes,
        prepared.stats.output_script_bytes,
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
