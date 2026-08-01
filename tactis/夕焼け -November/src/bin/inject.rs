use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use yuyake_mes_tools::workflow::{default_inject_output, inject_path};

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Inject UTF-8 JSON messages into Yuyake -November- MES scripts"
)]
struct Args {
    /// Original MES file or MES directory.
    source: PathBuf,

    /// Extracted JSON file or JSON directory.
    translations: PathBuf,

    /// New MES file or directory. Existing paths are never overwritten.
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
    let output = match args.output {
        Some(path) => path,
        None => default_inject_output(&args.source)?,
    };
    let report = inject_path(&args.source, &args.translations, &output)?;
    println!("[inject] json_files={}", report.json_files);
    println!("[inject] json_entries={}", report.json_entries);
    println!("[inject] patched={}", report.patched);
    println!("[inject] unchanged={}", report.unchanged);
    println!("[inject] failed={}", report.failed);
    println!("[inject] warnings={}", report.warnings);
    println!("[inject] output={}", report.output.display());
    Ok(())
}
