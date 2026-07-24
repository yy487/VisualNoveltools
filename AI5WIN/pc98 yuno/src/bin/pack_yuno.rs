use clap::Parser;
use std::path::{Path, PathBuf};
use yuno_hdi_tools::yuno::{pack_archives, PackRequest, YunoError};

#[derive(Debug, Parser)]
#[command(about = "Rebuild YU-NO PC-98 YUNO_A ... YUNO_Q archives")]
struct Args {
    /// One or more directories created by unpack_yuno.
    #[arg(required = true)]
    unpacked: Vec<PathBuf>,

    /// Exact output file. Allowed only with one input directory.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn default_output(unpacked: &Path) -> Result<PathBuf, YunoError> {
    let name = unpacked
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            YunoError::Invalid(format!(
                "invalid unpacked directory name: {}",
                unpacked.display()
            ))
        })?;
    Ok(unpacked
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{name}_packed")))
}

fn run() -> Result<(), YunoError> {
    let args = Args::parse();
    if args.output.is_some() && args.unpacked.len() != 1 {
        return Err(YunoError::Invalid(
            "--output requires exactly one input directory".to_owned(),
        ));
    }
    let requests: Vec<_> = args
        .unpacked
        .iter()
        .map(|unpacked| {
            Ok(PackRequest {
                unpacked: unpacked.clone(),
                output: args
                    .output
                    .clone()
                    .map(Ok)
                    .unwrap_or_else(|| default_output(unpacked))?,
            })
        })
        .collect::<Result<_, YunoError>>()?;

    for stats in pack_archives(&requests)? {
        println!(
            "[pack] archive={} packed_files={} changed={} unchanged={} byte_exact={} output_bytes={} output={}",
            stats.archive,
            stats.packed_files,
            stats.changed_files,
            stats.unchanged_files,
            stats.byte_exact,
            stats.output_bytes,
            stats.output.display()
        );
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
