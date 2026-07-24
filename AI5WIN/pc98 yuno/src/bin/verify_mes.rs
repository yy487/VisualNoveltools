use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use yuno_hdi_tools::mes::{compress, decompress, MesError};
use yuno_hdi_tools::mes_text::extract_document;

#[derive(Debug, Parser)]
#[command(about = "Read-only validation of YU-NO PC-98 MES streams and text records")]
struct Args {
    /// Compressed MES file or resource directory.
    input: PathBuf,
}

fn invalid(message: impl Into<String>) -> MesError {
    MesError::Invalid(message.into())
}

fn io_error(action: &str, path: &Path, source: std::io::Error) -> MesError {
    invalid(format!("{action} {}: {source}", path.display()))
}

fn is_mes(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("mes"))
}

fn collect_mes_files(directory: &Path, output: &mut Vec<PathBuf>) -> Result<(), MesError> {
    let mut entries: Vec<_> = fs::read_dir(directory)
        .map_err(|source| io_error("cannot read directory", directory, source))?
        .collect::<std::result::Result<_, _>>()
        .map_err(|source| io_error("cannot enumerate directory", directory, source))?;
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_lowercase());
    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| io_error("cannot inspect", &path, source))?;
        if file_type.is_symlink() {
            return Err(invalid(format!(
                "symbolic links are not supported: {}",
                path.display()
            )));
        }
        if file_type.is_dir() {
            collect_mes_files(&path, output)?;
        } else if file_type.is_file() && is_mes(&path) {
            output.push(path);
        }
    }
    Ok(())
}

fn run() -> Result<(), MesError> {
    let args = Args::parse();
    let files = if args.input.is_file() {
        if !is_mes(&args.input) {
            return Err(invalid(format!(
                "input file is not MES: {}",
                args.input.display()
            )));
        }
        vec![args.input]
    } else if args.input.is_dir() {
        let mut files = Vec::new();
        collect_mes_files(&args.input, &mut files)?;
        files
    } else {
        return Err(invalid(format!(
            "input does not exist: {}",
            args.input.display()
        )));
    };
    if files.is_empty() {
        return Err(invalid("no MES files found"));
    }

    let mut stored_bytes = 0usize;
    let mut decoded_bytes = 0usize;
    let mut recompressed_bytes = 0usize;
    let mut text_entries = 0usize;
    for path in &files {
        let stored = fs::read(path).map_err(|source| io_error("cannot read", path, source))?;
        let (decoded, stats) = decompress(&stored)?;
        if stats.trailing_bytes != 0 || stats.padding_value != 0 {
            return Err(invalid(format!(
                "{} has trailing bytes or nonzero padding",
                path.display()
            )));
        }
        let recompressed = compress(&decoded)?;
        let (verified, verify_stats) = decompress(&recompressed)?;
        if verified != decoded
            || verify_stats.trailing_bytes != 0
            || verify_stats.padding_value != 0
        {
            return Err(invalid(format!(
                "{} failed recompression round-trip",
                path.display()
            )));
        }
        let document = extract_document(&stored, path.to_string_lossy())?;
        stored_bytes += stored.len();
        decoded_bytes += decoded.len();
        recompressed_bytes += recompressed.len();
        text_entries += document.entries.len();
    }
    println!(
        "[verify_mes] files={} stored_bytes={} decoded_bytes={} recompressed_bytes={} \
         text_entries={} failures=0",
        files.len(),
        stored_bytes,
        decoded_bytes,
        recompressed_bytes,
        text_entries
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
