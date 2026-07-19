use anyhow::Result;
use clap::{Parser, ValueEnum};
use silky_common::archive::{repack_archive, RepackOptions};
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
#[command(about = "Repack a directory as a Silky ARC archive")]
struct Args {
    /// Directory produced by unpack. Drag-and-drop is supported.
    input: PathBuf,
    /// Output .arc. Defaults to <directory>_repacked.arc.
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Format::Auto)]
    format: Format,
    /// ARC file-name encoding. A manifest encoding takes precedence.
    #[arg(long, default_value = "cp932")]
    encoding: String,
    /// Explicit source manifest. Defaults to input/.silky_arc_manifest.json.
    #[arg(long)]
    manifest: Option<PathBuf>,
    /// Store newly added files raw; existing compressed entries stay compressed.
    #[arg(long)]
    no_compress: bool,
    /// Store every entry raw, including entries marked compressed by the manifest.
    #[arg(long)]
    store_all: bool,
    /// LZSS worker threads. 0 uses available CPUs.
    #[arg(short = 'j', long, default_value_t = 0)]
    jobs: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let output = args.output.unwrap_or_else(|| {
        let name = args.input.file_name().unwrap_or_default().to_string_lossy();
        args.input.with_file_name(format!("{name}_repacked.arc"))
    });
    let manifest = repack_archive(
        &args.input,
        &output,
        args.manifest.as_deref(),
        RepackOptions {
            format: args.format.as_str(),
            encoding: &args.encoding,
            compress_new: !args.no_compress && !args.store_all,
            preserve_packed: !args.store_all,
            jobs: args.jobs,
        },
    )?;
    let output_bytes = std::fs::metadata(&output)?.len();
    println!("[repack] packed_files={}", manifest.entries.len());
    println!("[repack] format={}", manifest.format.as_str());
    println!("[repack] output_bytes={output_bytes}");
    println!("[repack] output={}", output.display());
    Ok(())
}
