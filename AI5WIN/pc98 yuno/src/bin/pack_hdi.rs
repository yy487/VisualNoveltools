use clap::Parser;
use std::path::{Path, PathBuf};
use yuno_hdi_tools::hdi::{prepare_pack, HdiError};

#[derive(Debug, Parser)]
#[command(about = "Build a new Anex86 PC-98 FAT16 HDI from an unpacked directory")]
struct Args {
    /// Original HDI used to create the unpacked directory. It is never overwritten.
    source: PathBuf,

    /// Directory created by unpack_hdi, including .hdi_manifest.json.
    unpacked: PathBuf,

    /// New output HDI. Defaults to SOURCE_packed.hdi.
    #[arg(short, long)]
    output: Option<PathBuf>,
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
        .join(format!("{stem}_packed.{extension}")))
}

fn run() -> Result<(), HdiError> {
    let args = Args::parse();
    let output = args.output.unwrap_or(default_output(&args.source)?);
    let prepared = prepare_pack(&args.source, &args.unpacked)?;
    prepared.write_to(&output)?;
    println!(
        "[pack] source_files={} changed={} unchanged={} allocated_clusters={} freed_clusters={} free_clusters_after={} source_fat_mismatch_entries={} byte_exact={} output_bytes={} output={}",
        prepared.stats.source_files,
        prepared.stats.changed_files,
        prepared.stats.unchanged_files,
        prepared.stats.allocated_clusters,
        prepared.stats.freed_clusters,
        prepared.stats.free_clusters_after,
        prepared.stats.source_fat_mismatch_entries,
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
