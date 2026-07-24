use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use yuno_hdi_tools::mes::{decompress, MesError};

#[derive(Debug, Parser)]
#[command(about = "Decompress a YU-NO PC-98 MES script without overwriting its source")]
struct Args {
    /// Compressed MES input.
    input: PathBuf,

    /// New decoded output. Defaults to INPUT.decoded.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn default_output(input: &Path) -> PathBuf {
    let mut value = input.as_os_str().to_os_string();
    value.push(".decoded");
    PathBuf::from(value)
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

    let stored = fs::read(&args.input).map_err(|source| {
        MesError::Invalid(format!("cannot read {}: {source}", args.input.display()))
    })?;
    let (decoded, stats) = decompress(&stored)?;
    fs::write(&output, &decoded).map_err(|source| {
        MesError::Invalid(format!("cannot write {}: {source}", output.display()))
    })?;
    println!(
        "[unpack_mes] input_bytes={} output_bytes={} entry_offset=0x{:X} \
         literal_tokens={} match_tokens={} output={}",
        stored.len(),
        stats.output_size,
        stats.entry_offset,
        stats.literal_tokens,
        stats.match_tokens,
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
