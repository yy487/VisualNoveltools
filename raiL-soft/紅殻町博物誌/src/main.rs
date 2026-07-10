use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use railsoft_xfl_tool::XflArchive;
use railsoft_xfl_tool::xfl::pack_directory;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[error] {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<OsString> = env::args_os().collect();
    let Some(command) = arguments.get(1).and_then(|value| value.to_str()) else {
        return Err(usage().into());
    };

    match command {
        "list" if arguments.len() == 3 => list_archive(Path::new(&arguments[2])),
        "unpack" if matches!(arguments.len(), 4 | 5) => {
            let force = match arguments.get(4) {
                None => false,
                Some(flag) if flag == "--force" => true,
                Some(_) => return Err(usage().into()),
            };
            unpack_archive(Path::new(&arguments[2]), Path::new(&arguments[3]), force)
        }
        "pack" if matches!(arguments.len(), 4 | 5) => {
            let force = match arguments.get(4) {
                None => false,
                Some(flag) if flag == "--force" => true,
                Some(_) => return Err(usage().into()),
            };
            pack_archive(Path::new(&arguments[2]), Path::new(&arguments[3]), force)
        }
        "--help" | "-h" | "help" => {
            println!("{}", usage());
            Ok(())
        }
        _ => Err(usage().into()),
    }
}

fn read_input(path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()).into())
}

fn list_archive(input: &Path) -> Result<(), Box<dyn Error>> {
    let data = read_input(input)?;
    let archive = XflArchive::parse(&data)?;

    print_summary(input, &archive);
    println!("index  offset      size        name");
    for (index, entry) in archive.entries.iter().enumerate() {
        println!(
            "{index:>5}  0x{:08x}  {:>10}  {}{}",
            entry.offset,
            entry.size,
            entry.name,
            if entry.name_was_escaped {
                " [byte-escaped]"
            } else {
                ""
            }
        );
    }
    Ok(())
}

fn unpack_archive(input: &Path, output: &Path, force: bool) -> Result<(), Box<dyn Error>> {
    let data = read_input(input)?;
    let archive = XflArchive::parse(&data)?;
    print_summary(input, &archive);

    let stats = archive.extract_to(output, force)?;
    println!("[unpack] output={}", absolute_or_original(output).display());
    println!("[unpack] extracted_files={}", stats.extracted_files);
    println!("[unpack] extracted_bytes={}", stats.extracted_bytes);
    println!("[unpack] escaped_names={}", stats.escaped_names);
    Ok(())
}

fn pack_archive(input: &Path, output: &Path, force: bool) -> Result<(), Box<dyn Error>> {
    if output.exists() && !force {
        return Err(format!(
            "output already exists (use --force to overwrite): {}",
            output.display()
        )
        .into());
    }
    let (data, stats) = pack_directory(input)?;
    fs::write(output, data)
        .map_err(|error| format!("failed to write {}: {error}", output.display()))?;
    println!("[pack] input={}", absolute_or_original(input).display());
    println!("[pack] output={}", absolute_or_original(output).display());
    println!("[pack] packed_files={}", stats.packed_files);
    println!("[pack] packed_bytes={}", stats.packed_bytes);
    println!("[pack] used_manifest={}", stats.used_manifest);
    Ok(())
}

fn print_summary(input: &Path, archive: &XflArchive<'_>) {
    println!("[xfl] input={}", absolute_or_original(input).display());
    println!("[xfl] magic=LB");
    println!("[xfl] version={}", archive.header.version);
    println!("[xfl] table_bytes={}", archive.header.table_size);
    println!("[xfl] entries={}", archive.header.entry_count);
    println!("[xfl] data_offset={}", archive.header.data_offset);
    println!("[xfl] payload_bytes={}", archive.payload_size());
}

fn absolute_or_original(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn usage() -> &'static str {
    "Usage:\n  railsoft-xfl list <archive.xfl>\n  railsoft-xfl unpack <archive.xfl> <output-dir> [--force]\n  railsoft-xfl pack <input-dir> <archive.xfl> [--force]"
}
