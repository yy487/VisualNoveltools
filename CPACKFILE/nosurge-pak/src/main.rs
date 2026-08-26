use nosurge_pak::PakArchive;
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [] => interactive(None),
        [arg] if arg == "-h" || arg == "--help" || arg == "help" => {
            print_help();
            Ok(())
        }
        [path] if path != "list" && path != "unpack" => interactive(Some(PathBuf::from(path))),
        [command, rest @ ..] if command == "list" => command_list(rest),
        [command, rest @ ..] if command == "unpack" => command_unpack(rest),
        _ => {
            print_help();
            Err("invalid arguments".to_owned())
        }
    }
}

fn command_list(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.len() > 3 {
        return Err("usage: nosurge-pak list <archive.pak> [--limit <count>]".to_owned());
    }
    let mut limit = usize::MAX;
    if args.len() == 3 {
        if args[1] != "--limit" {
            return Err("expected --limit <count>".to_owned());
        }
        limit = args[2]
            .parse::<usize>()
            .map_err(|e| format!("invalid list limit {:?}: {e}", args[2]))?;
    } else if args.len() != 1 {
        return Err("usage: nosurge-pak list <archive.pak> [--limit <count>]".to_owned());
    }
    let archive = PakArchive::open(&args[0])?;
    print_summary(&archive);
    for entry in archive.entries.iter().take(limit) {
        println!(
            "{:5}  offset=0x{:08X}  size={:10}  {}",
            entry.index, entry.offset, entry.size, entry.name
        );
    }
    if archive.entries.len() > limit {
        println!("... {} more entries", archive.entries.len() - limit);
    }
    Ok(())
}

fn command_unpack(args: &[String]) -> Result<(), String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(
            "usage: nosurge-pak unpack <archive.pak> <output-dir> [--overwrite]".to_owned(),
        );
    }
    let overwrite = match args.get(2) {
        None => false,
        Some(flag) if flag == "--overwrite" => true,
        Some(_) => return Err("the only supported third argument is --overwrite".to_owned()),
    };
    let archive = PakArchive::open(&args[0])?;
    print_summary(&archive);
    archive.extract_to(&args[1], overwrite)?;
    println!(
        "unpacked {} files ({} payload bytes) to {}",
        archive.entries.len(),
        archive.file_size - archive.data_base,
        Path::new(&args[1]).display()
    );
    Ok(())
}

fn interactive(prefill: Option<PathBuf>) -> Result<(), String> {
    println!("Ciel nosurge DX PAK unpacker");
    println!("No files will be written until you confirm an unpack operation.\n");
    let mut default_archive = prefill
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut default_choice = if prefill.is_some() { "2" } else { "" };
    loop {
        println!("1) List archive");
        println!("2) Unpack archive");
        println!("0) Exit");
        let choice = prompt("Select", default_choice)?;
        default_choice = "";
        let result = match choice.trim() {
            "" | "0" => return Ok(()),
            "1" => interactive_list(&mut default_archive),
            "2" => interactive_unpack(&mut default_archive),
            other => Err(format!("unknown menu choice {other:?}")),
        };
        if let Err(error) = result {
            eprintln!("error: {error}");
        }
        println!();
    }
}

fn interactive_list(default_archive: &mut String) -> Result<(), String> {
    let archive_text = prompt("Archive PAK", default_archive)?;
    if archive_text.trim().is_empty() {
        println!("cancelled");
        return Ok(());
    }
    let archive_path = PathBuf::from(strip_wrapping_quotes(archive_text.trim()));
    *default_archive = archive_path.to_string_lossy().into_owned();
    let archive = PakArchive::open(&archive_path)?;
    print_summary(&archive);
    let limit_text = prompt("Entries to show", "20")?;
    let limit = limit_text
        .parse::<usize>()
        .map_err(|e| format!("invalid entry limit {limit_text:?}: {e}"))?;
    for entry in archive.entries.iter().take(limit) {
        println!(
            "{:5}  offset=0x{:08X}  size={:10}  {}",
            entry.index, entry.offset, entry.size, entry.name
        );
    }
    if archive.entries.len() > limit {
        println!("... {} more entries", archive.entries.len() - limit);
    }
    Ok(())
}

fn interactive_unpack(default_archive: &mut String) -> Result<(), String> {
    let archive_text = prompt("Archive PAK", default_archive)?;
    if archive_text.trim().is_empty() {
        println!("cancelled");
        return Ok(());
    }
    let archive_path = PathBuf::from(strip_wrapping_quotes(archive_text.trim()));
    *default_archive = archive_path.to_string_lossy().into_owned();
    let archive = PakArchive::open(&archive_path)?;
    print_summary(&archive);

    let suggested = archive_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(
            "{}_unpacked",
            archive_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
        ));
    let output_text = prompt("Output directory", &suggested.to_string_lossy())?;
    if output_text.trim().is_empty() {
        println!("cancelled");
        return Ok(());
    }
    let output_path = PathBuf::from(strip_wrapping_quotes(output_text.trim()));
    let overwrite = if output_path.exists() {
        if !confirm("Output exists. Replace it", false)? {
            println!("cancelled");
            return Ok(());
        }
        true
    } else {
        false
    };
    println!("\nArchive: {}", archive_path.display());
    println!("Output:  {}", output_path.display());
    println!("Files:   {}", archive.entries.len());
    if !confirm("Start unpacking", false)? {
        println!("cancelled");
        return Ok(());
    }
    archive.extract_to(&output_path, overwrite)?;
    println!(
        "unpacked {} files ({} payload bytes) to {}",
        archive.entries.len(),
        archive.file_size - archive.data_base,
        output_path.display()
    );
    Ok(())
}

fn prompt(label: &str, default: &str) -> Result<String, String> {
    if default.is_empty() {
        print!("{label}: ");
    } else {
        print!("{label} [{default}]: ");
    }
    io::stdout()
        .flush()
        .map_err(|e| format!("cannot flush prompt: {e}"))?;
    let mut line = String::new();
    let bytes = io::stdin()
        .read_line(&mut line)
        .map_err(|e| format!("cannot read prompt input: {e}"))?;
    if bytes == 0 {
        return Ok(String::new());
    }
    let value = line.trim_end_matches(['\r', '\n']);
    if value.is_empty() {
        Ok(default.to_owned())
    } else {
        Ok(value.to_owned())
    }
}

fn confirm(label: &str, default: bool) -> Result<bool, String> {
    let marker = if default { "Y/n" } else { "y/N" };
    let answer = prompt(&format!("{label} ({marker})"), "")?;
    if answer.trim().is_empty() {
        return Ok(default);
    }
    match answer.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => Err(format!("expected yes or no, got {answer:?}")),
    }
}

fn strip_wrapping_quotes(value: &str) -> &str {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn print_summary(archive: &PakArchive) {
    println!(
        "PAK: field_00=0x{:08X}, flags=0x{:08X}, entries={}, table_end=0x{:X}, payload_bytes={}, file_bytes={}",
        archive.header.field_00,
        archive.header.flags,
        archive.header.file_count,
        archive.data_base,
        archive.file_size - archive.data_base,
        archive.file_size
    );
}

fn print_help() {
    println!(
        "nosurge-pak 0.1.0\n\
         Read and unpack Ciel nosurge DX PAK archives. Source archives are never modified.\n\n\
         Usage:\n\
           nosurge-pak                         Interactive mode\n\
           nosurge-pak <archive.pak>           Interactive mode with a prefilled path\n\
           nosurge-pak list <archive.pak> [--limit <count>]\n\
           nosurge-pak unpack <archive.pak> <output-dir> [--overwrite]\n\n\
         Options:\n\
           --overwrite   Replace an existing output only after a complete temporary extraction\n\
           -h, --help    Show this help"
    );
}
