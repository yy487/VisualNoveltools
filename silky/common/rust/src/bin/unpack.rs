use anyhow::{bail, Result};
use clap::{Parser, ValueEnum};
use silky_common::archive::unpack_archive;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Format {
    Auto,
    SilkyLzss,
    GarbroFixed,
}

impl Format {
    fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::SilkyLzss => "silky-lzss",
            Self::GarbroFixed => "garbro-fixed",
        }
    }
}

#[derive(Debug, Parser)]
#[command(about = "Unpack a Silky ARC archive without overwriting existing output")]
struct Args {
    /// Input .arc file. Drag-and-drop is supported.
    archive: PathBuf,
    /// Output directory. Defaults to <archive>_unpacked.
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Format::Auto)]
    format: Format,
    /// ARC file-name encoding.
    #[arg(long, default_value = "cp932")]
    encoding: String,
    /// Do not write .silky_arc_manifest.json.
    #[arg(long)]
    no_manifest: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if !args.archive.is_file() {
        bail!("archive does not exist: {}", args.archive.display());
    }
    let output = args.output.unwrap_or_else(|| {
        let stem = args
            .archive
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        args.archive.with_file_name(format!("{stem}_unpacked"))
    });
    let manifest = unpack_archive(
        &args.archive,
        &output,
        args.format.as_str(),
        &args.encoding,
        !args.no_manifest,
    )?;
    println!("[unpack] extracted_files={}", manifest.entries.len());
    println!("[unpack] format={}", manifest.format.as_str());
    println!("[unpack] warnings=0");
    println!("[unpack] output={}", output.display());
    Ok(())
}
