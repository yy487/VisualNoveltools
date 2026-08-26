use nosurge_mrlk::MrlkArchive;
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
        [argument] if matches!(argument.as_str(), "-h" | "--help" | "help") => {
            print_help();
            Ok(())
        }
        [path] if !matches!(path.as_str(), "list" | "unpack" | "pack") => {
            interactive(Some(PathBuf::from(path)))
        }
        [command, rest @ ..] if command == "list" => command_list(rest),
        [command, rest @ ..] if command == "unpack" => command_unpack(rest),
        [command, rest @ ..] if command == "pack" => command_pack(rest),
        _ => {
            print_help();
            Err("invalid arguments".to_owned())
        }
    }
}

fn command_list(args: &[String]) -> Result<(), String> {
    let (archive_path, limit) = match args {
        [archive] => (archive.as_str(), usize::MAX),
        [archive, flag, count] if flag == "--limit" => (
            archive.as_str(),
            count
                .parse::<usize>()
                .map_err(|error| format!("invalid list limit {count:?}: {error}"))?,
        ),
        _ => return Err("usage: nosurge-mrlk list <archive.psarc> [--limit <count>]".to_owned()),
    };
    let archive = MrlkArchive::open(archive_path)?;
    print_summary(&archive);
    print_entries(&archive, limit);
    Ok(())
}

fn command_unpack(args: &[String]) -> Result<(), String> {
    let (archive_path, output, overwrite) = match args {
        [archive, output] => (archive, output, false),
        [archive, output, flag] if flag == "--overwrite" => (archive, output, true),
        _ => {
            return Err(
                "usage: nosurge-mrlk unpack <archive.psarc> <output-dir> [--overwrite]".to_owned(),
            )
        }
    };
    let archive = MrlkArchive::open(archive_path)?;
    print_summary(&archive);
    archive.extract_to(output, overwrite)?;
    println!(
        "unpacked {} files ({} payload bytes) to {}",
        archive.entries.len(),
        archive.payload_bytes(),
        Path::new(output).display()
    );
    Ok(())
}

fn command_pack(args: &[String]) -> Result<(), String> {
    let (template, input, output, overwrite) = match args {
        [template, input, output] => (template, input, output, false),
        [template, input, output, flag] if flag == "--overwrite" => (template, input, output, true),
        _ => return Err(
            "usage: nosurge-mrlk pack <template.psarc> <input-dir> <output.psarc> [--overwrite]"
                .to_owned(),
        ),
    };
    let archive = MrlkArchive::open(template)?;
    print_summary(&archive);
    let summary = archive.pack_from_directory(input, output, overwrite)?;
    println!(
        "packed {} files ({} payload bytes, {} total bytes) to {}",
        summary.file_count,
        summary.payload_bytes,
        summary.output_bytes,
        Path::new(output).display()
    );
    Ok(())
}

fn interactive(prefill: Option<PathBuf>) -> Result<(), String> {
    println!("Ciel nosurge DX MRLK unpacker/repacker");
    println!("No files will be written until you confirm an unpack or pack operation.\n");
    let mut default_archive = prefill
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut default_choice = if prefill.is_some() { "2" } else { "" };
    loop {
        println!("1) List archive");
        println!("2) Unpack archive");
        println!("3) Pack from directory");
        println!("0) Exit");
        let Some(choice) = prompt("Select", default_choice)? else {
            return Ok(());
        };
        default_choice = "";
        let result = match choice.trim() {
            "" | "0" => return Ok(()),
            "1" => interactive_list(&mut default_archive),
            "2" => interactive_unpack(&mut default_archive),
            "3" => interactive_pack(&mut default_archive),
            other => Err(format!("unknown menu choice {other:?}")),
        };
        if let Err(error) = result {
            eprintln!("error: {error}");
        }
        println!();
    }
}

fn interactive_list(default_archive: &mut String) -> Result<(), String> {
    let Some(archive_text) = prompt("Archive MRLK/PSARC", default_archive)? else {
        println!("cancelled");
        return Ok(());
    };
    if archive_text.trim().is_empty() {
        println!("cancelled");
        return Ok(());
    }
    let archive_path = PathBuf::from(strip_wrapping_quotes(archive_text.trim()));
    *default_archive = archive_path.to_string_lossy().into_owned();
    let archive = MrlkArchive::open(&archive_path)?;
    print_summary(&archive);
    let Some(limit_text) = prompt("Entries to show", "20")? else {
        println!("cancelled");
        return Ok(());
    };
    let limit = limit_text
        .parse::<usize>()
        .map_err(|error| format!("invalid entry limit {limit_text:?}: {error}"))?;
    print_entries(&archive, limit);
    Ok(())
}

fn interactive_unpack(default_archive: &mut String) -> Result<(), String> {
    let Some(archive_text) = prompt("Archive MRLK/PSARC", default_archive)? else {
        println!("cancelled");
        return Ok(());
    };
    if archive_text.trim().is_empty() {
        println!("cancelled");
        return Ok(());
    }
    let archive_path = PathBuf::from(strip_wrapping_quotes(archive_text.trim()));
    *default_archive = archive_path.to_string_lossy().into_owned();
    let archive = MrlkArchive::open(&archive_path)?;
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
    let Some(output_text) = prompt("Output directory", &suggested.to_string_lossy())? else {
        println!("cancelled");
        return Ok(());
    };
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
        archive.payload_bytes(),
        output_path.display()
    );
    Ok(())
}

fn interactive_pack(default_archive: &mut String) -> Result<(), String> {
    let Some(template_text) = prompt("Template MRLK/PSARC", default_archive)? else {
        println!("cancelled");
        return Ok(());
    };
    if template_text.trim().is_empty() {
        println!("cancelled");
        return Ok(());
    }
    let template_path = PathBuf::from(strip_wrapping_quotes(template_text.trim()));
    *default_archive = template_path.to_string_lossy().into_owned();
    let archive = MrlkArchive::open(&template_path)?;
    print_summary(&archive);

    let suggested_input = template_path.parent().unwrap_or_else(|| Path::new("."));
    let Some(input_text) = prompt(
        "Directory containing replacement files",
        &suggested_input.to_string_lossy(),
    )?
    else {
        println!("cancelled");
        return Ok(());
    };
    if input_text.trim().is_empty() {
        println!("cancelled");
        return Ok(());
    }
    let input_path = PathBuf::from(strip_wrapping_quotes(input_text.trim()));

    let suggested_output = template_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(
            "{}_repacked.psarc",
            template_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
        ));
    let Some(output_text) = prompt("Output archive", &suggested_output.to_string_lossy())? else {
        println!("cancelled");
        return Ok(());
    };
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

    println!("\nTemplate: {}", template_path.display());
    println!("Input:    {}", input_path.display());
    println!("Output:   {}", output_path.display());
    println!("Files:    {}", archive.entries.len());
    if !confirm("Start packing", false)? {
        println!("cancelled");
        return Ok(());
    }
    let summary = archive.pack_from_directory(&input_path, &output_path, overwrite)?;
    println!(
        "packed {} files ({} payload bytes, {} total bytes) to {}",
        summary.file_count,
        summary.payload_bytes,
        summary.output_bytes,
        output_path.display()
    );
    Ok(())
}

fn prompt(label: &str, default: &str) -> Result<Option<String>, String> {
    if default.is_empty() {
        print!("{label}: ");
    } else {
        print!("{label} [{default}]: ");
    }
    io::stdout()
        .flush()
        .map_err(|error| format!("cannot flush prompt: {error}"))?;
    let mut line = String::new();
    let bytes = io::stdin()
        .read_line(&mut line)
        .map_err(|error| format!("cannot read prompt input: {error}"))?;
    if bytes == 0 {
        return Ok(None);
    }
    let value = line.trim_end_matches(['\r', '\n']);
    if value.is_empty() {
        Ok(Some(default.to_owned()))
    } else {
        Ok(Some(value.to_owned()))
    }
}

fn confirm(label: &str, default: bool) -> Result<bool, String> {
    let marker = if default { "Y/n" } else { "y/N" };
    let Some(answer) = prompt(&format!("{label} ({marker})"), "")? else {
        return Ok(false);
    };
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

fn print_summary(archive: &MrlkArchive) {
    println!(
        "MRLK: entries={}, table_end=0x{:X}, names={} bytes, data=0x{:X}, payload={} bytes, file={} bytes",
        archive.header.file_count,
        archive.header.table_end,
        archive.header.names_size,
        archive.data_offset,
        archive.payload_bytes(),
        archive.file_size
    );
}

fn print_entries(archive: &MrlkArchive, limit: usize) {
    for entry in archive.entries.iter().take(limit) {
        println!(
            "{:5}  offset=0x{:08X}  size={:10}  {}",
            entry.index, entry.offset, entry.size, entry.name
        );
    }
    if archive.entries.len() > limit {
        println!("... {} more entries", archive.entries.len() - limit);
    }
}

fn print_help() {
    println!(
        "nosurge-mrlk 0.1.0\n\
         List, unpack, and repack Ciel nosurge DX MRLK resource archives.\n\
         Source archives are never modified. G1T and other payloads are copied as opaque files.\n\n\
         Usage:\n\
           nosurge-mrlk\n\
           nosurge-mrlk <archive.psarc>\n\
           nosurge-mrlk list <archive.psarc> [--limit <count>]\n\
           nosurge-mrlk unpack <archive.psarc> <output-dir> [--overwrite]\n\
           nosurge-mrlk pack <template.psarc> <input-dir> <output.psarc> [--overwrite]\n\n\
         Pack uses the original archive as a template for file names and order, then\n\
         rebuilds every absolute offset and size. Extra files in the input directory\n\
         are ignored; every file named by the template must exist.\n\n\
         Options:\n\
           --overwrite   Replace an existing output only after a complete staged build\n\
           -h, --help    Show this help"
    );
}
