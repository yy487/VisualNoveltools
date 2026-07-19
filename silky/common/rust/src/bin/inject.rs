use anyhow::Result;
use clap::Parser;
use silky_common::workflow::inject_path;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(about = "Inject UTF-8 JSON directly into Silky MES scripts")]
struct Args {
    /// Original .MES file or directory.
    input: PathBuf,
    /// Translation JSON file or directory.
    json: PathBuf,
    /// Output .MES file or directory. Existing output is never overwritten.
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
    let report = inject_path(
        &args.input,
        &args.json,
        args.output.as_deref(),
        &args.encoding,
        &args.pattern,
        args.jobs,
    )?;
    for warning in report.warnings.iter().take(50) {
        eprintln!("[inject][warn] {warning}");
    }
    println!("[inject] scanned_files={}", report.scanned_files);
    println!("[inject] json_entries={}", report.json_entries);
    println!("[inject] patched={}", report.patched);
    println!("[inject] unchanged={}", report.unchanged);
    println!("[inject] failed=0");
    println!("[inject] warnings={}", report.warnings.len());
    println!("[inject] output={}", report.output.display());
    Ok(())
}
