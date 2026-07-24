use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use yuno_hdi_tools::yuno::{unpack_archives, UnpackRequest, YunoError};

#[derive(Debug, Parser)]
#[command(about = "Extract YU-NO PC-98 AI5 resources from YUNO_A through YUNO_Q")]
struct Args {
    /// One or more YUNO_A ... YUNO_Q archives. Inputs are read only.
    #[arg(required = true)]
    sources: Vec<PathBuf>,

    /// Output root. Each archive is extracted into OUTPUT/YUNO_X.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn default_output(source: &Path) -> Result<PathBuf, YunoError> {
    let name = source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            YunoError::Invalid(format!("invalid archive filename: {}", source.display()))
        })?;
    Ok(source
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{name}_unpacked")))
}

fn run() -> Result<(), YunoError> {
    let args = Args::parse();
    let requests: Vec<_> = args
        .sources
        .iter()
        .map(|source| {
            let output = if let Some(root) = &args.output {
                let name = source.file_name().ok_or_else(|| {
                    YunoError::Invalid(format!(
                        "archive path has no filename: {}",
                        source.display()
                    ))
                })?;
                root.join(name)
            } else {
                default_output(source)?
            };
            Ok(UnpackRequest {
                source: source.clone(),
                output,
            })
        })
        .collect::<Result<_, YunoError>>()?;

    let mut created_root = false;
    if let Some(root) = &args.output {
        if root.exists() {
            if !root.is_dir() {
                return Err(YunoError::Invalid(format!(
                    "output root is not a directory: {}",
                    root.display()
                )));
            }
        } else {
            fs::create_dir_all(root).map_err(|source| YunoError::Io {
                path: root.clone(),
                source,
            })?;
            created_root = true;
        }
    }

    let result = unpack_archives(&requests);
    if result.is_err() && created_root {
        if let Some(root) = &args.output {
            let _ = fs::remove_dir(root);
        }
    }
    for stats in result? {
        println!(
            "[unpack] archive={} extracted_files={} extracted_bytes={} manifest={} output={}",
            stats.archive,
            stats.extracted_files,
            stats.extracted_bytes,
            stats.manifest.display(),
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
