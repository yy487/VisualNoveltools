use canaan_tools::hdi::{prepare_hdi, HdiError};
use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(about = "Build a new PC-98 HDI by replacing files in a FAT12 directory")]
struct Args {
    /// Original Anex86 HDI image. It is read only and never overwritten.
    source: PathBuf,

    /// Directory containing replacement 8.3 files, for example packed DISK_X.CAT/LIB pairs.
    replacements: PathBuf,

    /// New output HDI path. Defaults to SOURCE_patched.hdi.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Existing FAT12 destination directory inside the image.
    #[arg(long, default_value = "CANAAN")]
    destination: String,
}

fn default_output(source: &Path) -> Result<PathBuf, HdiError> {
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| HdiError::Invalid(format!("invalid HDI filename: {}", source.display())))?;
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("hdi");
    Ok(source
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{stem}_patched.{extension}")))
}

fn run() -> Result<(), HdiError> {
    let args = Args::parse();
    let output = match args.output {
        Some(path) => path,
        None => default_output(&args.source)?,
    };
    if output.exists() {
        return Err(HdiError::OutputExists(output));
    }
    let prepared = prepare_hdi(&args.source, &args.replacements, &args.destination)?;
    prepared.write_to(&output)?;
    println!(
        "[build-hdi] source={} destination={} requested_files={} changed={} unchanged={} allocated_clusters={} freed_clusters={} free_clusters_after={} byte_exact={} output_bytes={} output={}",
        prepared.source.display(),
        prepared.destination,
        prepared.stats.requested_files,
        prepared.stats.changed_files,
        prepared.stats.unchanged_files,
        prepared.stats.allocated_clusters,
        prepared.stats.freed_clusters,
        prepared.stats.free_clusters_after,
        prepared.stats.byte_exact,
        prepared.image.bytes().len(),
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
