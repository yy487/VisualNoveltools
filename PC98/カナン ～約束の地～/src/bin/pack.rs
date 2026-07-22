use canaan_tools::archive::{list_pack_directories, prepare_pack, ArchiveError, PreparedPack};
use clap::Parser;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(about = "Repack System-98 unpack directories into new DISK_X.CAT/LIB pairs")]
struct Args {
    /// Unpacked archive directory/directories, or one root containing archive subdirectories.
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    /// New output directory. Only valid with one input.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn default_root_output(input: &Path) -> Result<PathBuf, ArchiveError> {
    let name = input
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ArchiveError::Invalid(format!("invalid directory name: {}", input.display()))
        })?;
    Ok(input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{name}_packed")))
}

fn run() -> Result<(), ArchiveError> {
    let args = Args::parse();
    if args.output.is_some() && args.inputs.len() != 1 {
        return Err(ArchiveError::Invalid(
            "--output requires exactly one input".to_string(),
        ));
    }

    let mut jobs: Vec<(PreparedPack, PathBuf, PathBuf)> = Vec::new();
    let mut output_roots = Vec::new();
    for input in &args.inputs {
        let pack_dirs = list_pack_directories(input)?;
        let output_root = if let Some(output) = &args.output {
            output.clone()
        } else {
            default_root_output(input)?
        };
        if output_root.exists() {
            return Err(ArchiveError::OutputExists(output_root));
        }
        output_roots.push(output_root.clone());
        for pack_dir in pack_dirs {
            let prepared = prepare_pack(&pack_dir)?;
            let cat_output = output_root.join(&prepared.catalog_name);
            let lib_output = output_root.join(&prepared.library_name);
            jobs.push((prepared, cat_output, lib_output));
        }
    }

    let mut planned_outputs = HashSet::new();
    for (_, cat, lib) in &jobs {
        if cat.exists() {
            return Err(ArchiveError::OutputExists(cat.clone()));
        }
        if lib.exists() {
            return Err(ArchiveError::OutputExists(lib.clone()));
        }
        for output in [cat, lib] {
            let key = output.to_string_lossy().to_lowercase();
            if !planned_outputs.insert(key) {
                return Err(ArchiveError::Invalid(format!(
                    "multiple inputs resolve to the same output: {}",
                    output.display()
                )));
            }
        }
    }

    let mut written: Vec<(PathBuf, PathBuf)> = Vec::new();
    for (prepared, cat, lib) in &jobs {
        if let Err(error) = prepared.write_to(cat, lib) {
            for (written_cat, written_lib) in written.iter().rev() {
                let _ = std::fs::remove_file(written_cat);
                let _ = std::fs::remove_file(written_lib);
            }
            for root in output_roots.iter().rev() {
                let _ = std::fs::remove_dir(root);
            }
            return Err(error);
        }
        written.push((cat.clone(), lib.clone()));
        println!(
            "[pack] input={} packed_files={} changed={} reused_stored={} byte_exact_pair={} cat_bytes={} lib_bytes={} catalog={} library={}",
            prepared.input_dir.display(),
            prepared.stats.entries,
            prepared.stats.changed_entries,
            prepared.stats.reused_stored_entries,
            prepared.stats.byte_exact_pair,
            prepared.stats.output_cat_bytes,
            prepared.stats.output_lib_bytes,
            cat.display(),
            lib.display()
        );
    }
    println!("[pack] archives={} warnings=0", jobs.len());
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
