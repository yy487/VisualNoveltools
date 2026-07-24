use clap::Parser;
use std::path::{Path, PathBuf};
use yuno_hdi_tools::hdi::{unpack_hdi, HdiError};

#[derive(Debug, Parser)]
#[command(about = "Extract all files from an Anex86 PC-98 FAT16 HDI")]
struct Args {
    /// Source HDI image. It is read only and never overwritten.
    source: PathBuf,

    /// New output directory. Defaults to SOURCE_unpacked.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn default_output(source: &Path) -> Result<PathBuf, HdiError> {
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| HdiError::Invalid(format!("invalid HDI filename: {}", source.display())))?;
    Ok(source
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{stem}_unpacked")))
}

fn run() -> Result<(), HdiError> {
    let args = Args::parse();
    let output = args.output.unwrap_or(default_output(&args.source)?);
    let stats = unpack_hdi(&args.source, &output)?;
    println!(
        "[unpack] extracted_files={} extracted_dirs={} extracted_bytes={} orphan_clusters={} fat_mismatch_entries={} manifest={} output={}",
        stats.extracted_files,
        stats.extracted_dirs,
        stats.extracted_bytes,
        stats.orphan_clusters,
        stats.fat_mismatch_entries,
        stats.manifest.display(),
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
