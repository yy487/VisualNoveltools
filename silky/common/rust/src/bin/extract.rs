use anyhow::Result;
use clap::Parser;
use silky_common::workflow::extract_path;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(about = "Extract Silky MES text directly to UTF-8 JSON")]
struct Args {
    /// Input .MES file or directory. Drag-and-drop is supported.
    input: PathBuf,
    /// JSON file or directory. A safe adjacent default is used when omitted.
    output: Option<PathBuf>,
    /// MES text encoding.
    #[arg(long, default_value = "cp932")]
    encoding: String,
    /// File-name wildcard used for directory input.
    #[arg(long, default_value = "*.MES")]
    pattern: String,
    /// Worker threads. 0 uses available CPUs.
    #[arg(short = 'j', long, default_value_t = 0)]
    jobs: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let report = extract_path(
        &args.input,
        args.output.as_deref(),
        &args.encoding,
        &args.pattern,
        args.jobs,
    )?;
    for warning in report.warnings.iter().take(50) {
        eprintln!("[extract][warn] {warning}");
    }
    println!("[extract] scanned_files={}", report.scanned_files);
    println!("[extract] json_files={}", report.json_files);
    println!("[extract] extracted_entries={}", report.extracted_entries);
    println!("[extract] skipped_blocks={}", report.skipped_blocks);
    println!("[extract] warnings={}", report.warnings.len());
    println!("[extract] output={}", report.output.display());
    Ok(())
}
