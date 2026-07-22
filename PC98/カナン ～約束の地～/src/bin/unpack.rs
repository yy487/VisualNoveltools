use canaan_tools::archive::{list_catalogs, prepare_unpack, ArchiveError, PreparedUnpack};
use clap::Parser;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(about = "Unpack System-98 DISK_X.CAT/LIB pairs without overwriting inputs")]
struct Args {
    /// CAT file(s), or one directory containing CAT/LIB pairs.
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    /// Output directory. Only valid with one input; a directory input creates one subdirectory per pair.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn default_file_output(cat: &Path) -> Result<PathBuf, ArchiveError> {
    let stem = cat
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ArchiveError::Invalid(format!("invalid CAT filename: {}", cat.display())))?;
    Ok(cat
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{stem}_unpacked")))
}

fn default_directory_output(input: &Path) -> Result<PathBuf, ArchiveError> {
    let name = input
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ArchiveError::Invalid(format!("invalid directory name: {}", input.display()))
        })?;
    Ok(input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{name}_unpacked")))
}

fn run() -> Result<(), ArchiveError> {
    let args = Args::parse();
    if args.output.is_some() && args.inputs.len() != 1 {
        return Err(ArchiveError::Invalid(
            "--output requires exactly one input".to_string(),
        ));
    }

    let mut jobs: Vec<(PreparedUnpack, PathBuf)> = Vec::new();
    let mut output_roots = Vec::new();
    for input in &args.inputs {
        if input.is_dir() {
            let root = if let Some(output) = &args.output {
                output.clone()
            } else {
                default_directory_output(input)?
            };
            if root.exists() {
                return Err(ArchiveError::OutputExists(root));
            }
            output_roots.push(root.clone());
            for cat in list_catalogs(input)? {
                let stem = cat
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .ok_or_else(|| {
                        ArchiveError::Invalid(format!("invalid CAT filename: {}", cat.display()))
                    })?;
                jobs.push((prepare_unpack(&cat)?, root.join(stem)));
            }
        } else {
            let output = if let Some(output) = &args.output {
                output.clone()
            } else {
                default_file_output(input)?
            };
            jobs.push((prepare_unpack(input)?, output));
        }
    }

    let mut planned_outputs = HashSet::new();
    for (_, output) in &jobs {
        if output.exists() {
            return Err(ArchiveError::OutputExists(output.clone()));
        }
        let key = output.to_string_lossy().to_lowercase();
        if !planned_outputs.insert(key) {
            return Err(ArchiveError::Invalid(format!(
                "multiple inputs resolve to the same output: {}",
                output.display()
            )));
        }
    }

    let mut written = Vec::new();
    for (prepared, output) in &jobs {
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent).map_err(|source| ArchiveError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        if let Err(error) = prepared.write_to(output) {
            for path in written.iter().rev() {
                let _ = std::fs::remove_dir_all(path);
            }
            for root in output_roots.iter().rev() {
                let _ = std::fs::remove_dir(root);
            }
            return Err(error);
        }
        written.push(output.clone());
        println!(
            "[unpack] archive={} extracted_files={} raw={} compressed={} stored_bytes={} decoded_bytes={} output={}",
            prepared.cat_path.display(),
            prepared.stats.entries,
            prepared.stats.raw,
            prepared.stats.compressed,
            prepared.stats.stored_bytes,
            prepared.stats.decoded_bytes,
            output.display()
        );
    }
    println!("[unpack] archives={} warnings=0", jobs.len());
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
