use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use yuno_hdi_tools::mes::{compress, decompress, MesError};

#[derive(Debug, Parser)]
#[command(about = "Compress a decoded YU-NO PC-98 MES script to a new file")]
struct Args {
    /// Decoded MES input.
    input: PathBuf,

    /// New compressed output. Defaults to INPUT.packed.MES.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn default_output(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .unwrap_or(input.as_os_str())
        .to_string_lossy();
    input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{stem}.packed.MES"))
}

fn run() -> Result<(), MesError> {
    let args = Args::parse();
    let output = args.output.unwrap_or_else(|| default_output(&args.input));
    if output.exists() {
        return Err(MesError::Invalid(format!(
            "refusing to overwrite existing output: {}",
            output.display()
        )));
    }

    let decoded = fs::read(&args.input).map_err(|source| {
        MesError::Invalid(format!("cannot read {}: {source}", args.input.display()))
    })?;
    let stored = compress(&decoded)?;
    let (verified, _) = decompress(&stored)?;
    if verified != decoded {
        return Err(MesError::Invalid(
            "compressed MES verification did not reproduce the input".to_owned(),
        ));
    }
    fs::write(&output, &stored).map_err(|source| {
        MesError::Invalid(format!("cannot write {}: {source}", output.display()))
    })?;
    println!(
        "[pack_mes] input_bytes={} output_bytes={} output={}",
        decoded.len(),
        stored.len(),
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
