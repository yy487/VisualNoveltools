use kokorov_fl4::{default_output_path, parse_archive, unpack_archive, ToolError, ToolResult};
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Args {
    input: PathBuf,
    output: Option<PathBuf>,
    list: bool,
    overwrite: bool,
    non_interactive: bool,
}

fn usage(program: &str) {
    println!(
        "KOKOROV FL4.0 archive unpacker\n\nUsage:\n  {program} [ARCHIVE.FL4]\n  {program} --list ARCHIVE.FL4\n  {program} --non-interactive [--output DIR] [--overwrite] ARCHIVE.FL4\n\nModes:\n  no arguments or one path  interactive mode with editable prefilled paths\n  --list                   validate and list entries without writing\n  --non-interactive        perform extraction without prompts\n\nOptions:\n  -o, --output DIR         output directory (suggested: <archive_stem>_unpacked)\n      --overwrite          replace an existing output directory transactionally\n      --list               validate and list entries without writing files\n      --non-interactive    confirm that all write parameters are explicit\n  -h, --help               show this help\n\nThe source archive is never modified. Extraction writes fl4_manifest.json and\nrefuses existing output unless overwrite is explicitly confirmed."
    );
}

fn parse_args(raw: Vec<OsString>) -> ToolResult<Args> {
    let mut input = None;
    let mut output = None;
    let mut list = false;
    let mut overwrite = false;
    let mut non_interactive = false;
    let mut index = 0usize;
    while index < raw.len() {
        let text = raw[index].to_string_lossy();
        match text.as_ref() {
            "--list" => list = true,
            "--overwrite" => overwrite = true,
            "--non-interactive" => non_interactive = true,
            "-o" | "--output" => {
                index += 1;
                let value = raw
                    .get(index)
                    .ok_or_else(|| ToolError(format!("{text} requires a directory")))?;
                output = Some(PathBuf::from(value));
            }
            _ if text.starts_with('-') => {
                return Err(ToolError(format!("unknown option '{text}'; use --help")))
            }
            _ => {
                if input.is_some() {
                    return Err(ToolError("only one input archive is supported".to_string()));
                }
                input = Some(PathBuf::from(&raw[index]));
            }
        }
        index += 1;
    }
    let input = input.ok_or_else(|| ToolError("missing ARCHIVE.FL4 input".to_string()))?;
    if list {
        if output.is_some() || overwrite {
            return Err(ToolError(
                "--output and --overwrite are not valid with --list".to_string(),
            ));
        }
    } else if !non_interactive {
        return Err(ToolError(
            "writing without prompts requires --non-interactive".to_string(),
        ));
    }
    Ok(Args {
        input,
        output,
        list,
        overwrite,
        non_interactive,
    })
}

fn read_archive(path: &Path) -> ToolResult<(Vec<u8>, kokorov_fl4::Fl4Archive)> {
    let bytes = std::fs::read(path)
        .map_err(|error| ToolError(format!("cannot read '{}': {error}", path.display())))?;
    let archive = parse_archive(&bytes)?;
    Ok((bytes, archive))
}

fn list_archive(path: &Path) -> ToolResult<()> {
    let (_, archive) = read_archive(path)?;
    println!(
        "[list] input={} bytes={} magic=FL4.0 header_size={} entries={} index_offset={} index_size={} buckets={} flags=0x{:04x}",
        path.display(),
        archive.file_len,
        archive.header.header_size,
        archive.entries.len(),
        archive.header.index_offset,
        archive.header.index_size,
        archive.buckets.len(),
        archive.header.flags
    );
    for entry in &archive.entries {
        println!(
            "[list] index={} bucket={} bucket_index={} data_order={} offset=0x{:x} size={} name={}",
            entry.index,
            entry.bucket,
            entry.bucket_index,
            entry.data_order,
            entry.data_offset,
            entry.size,
            entry.name
        );
    }
    Ok(())
}

fn strip_outer_quotes(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.len() >= 2 {
        let bytes = trimmed.as_bytes();
        if (bytes[0] == b'"' && bytes[trimmed.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[trimmed.len() - 1] == b'\'')
        {
            return &trimmed[1..trimmed.len() - 1];
        }
    }
    trimmed
}

fn prompt(label: &str, default: Option<&Path>) -> ToolResult<String> {
    match default {
        Some(path) => print!("{label} [{}]: ", path.display()),
        None => print!("{label}: "),
    }
    io::stdout()
        .flush()
        .map_err(|error| ToolError(format!("cannot flush prompt: {error}")))?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|error| ToolError(format!("cannot read input: {error}")))?;
    Ok(strip_outer_quotes(&line).to_string())
}

fn prompt_path(label: &str, default: Option<&Path>) -> ToolResult<Option<PathBuf>> {
    let value = prompt(label, default)?;
    if value == "0" {
        return Ok(None);
    }
    if value.is_empty() {
        return Ok(default.map(Path::to_path_buf));
    }
    Ok(Some(PathBuf::from(value)))
}

fn interactive(prefill: Option<PathBuf>) -> ToolResult<()> {
    println!("KOKOROV FL4.0 interactive unpack");
    println!("Enter 0 at a path prompt to cancel.");
    let input = loop {
        let Some(candidate) = prompt_path("Source archive", prefill.as_deref())? else {
            println!("[cancel] no files were written");
            return Ok(());
        };
        match read_archive(&candidate) {
            Ok((_, archive)) => {
                println!(
                    "[preflight] recognized=FL4.0 bytes={} header_size={} files={} index_offset={} index_size={} flags=0x{:04x}",
                    archive.file_len,
                    archive.header.header_size,
                    archive.entries.len(),
                    archive.header.index_offset,
                    archive.header.index_size,
                    archive.header.flags
                );
                break candidate;
            }
            Err(error) => {
                eprintln!("Invalid source: {error}");
            }
        }
    };

    let suggested_output = default_output_path(&input);
    let Some(mut output) = prompt_path("Output directory", Some(&suggested_output))? else {
        println!("[cancel] no files were written");
        return Ok(());
    };
    let overwrite = loop {
        if !output.exists() {
            break false;
        }
        println!("Output already exists: {}", output.display());
        println!("1 = change output, 2 = replace existing output, 0 = cancel");
        let choice = prompt("Choice", None)?;
        match choice.as_str() {
            "1" => {
                let Some(candidate) = prompt_path("Output directory", Some(&output))? else {
                    println!("[cancel] no files were written");
                    return Ok(());
                };
                output = candidate;
            }
            "2" => break true,
            "0" => {
                println!("[cancel] no files were written");
                return Ok(());
            }
            _ => eprintln!("Enter 1, 2, or 0."),
        }
    };

    println!("[preflight] operation=unpack");
    println!("[preflight] source={}", input.display());
    println!("[preflight] output={}", output.display());
    println!("[preflight] overwrite={overwrite}");
    let confirmation = prompt("Execute? [y/N]", None)?;
    if !confirmation.eq_ignore_ascii_case("y") && !confirmation.eq_ignore_ascii_case("yes") {
        println!("[cancel] no files were written");
        return Ok(());
    }
    print_report(unpack_archive(&input, Some(&output), overwrite)?);
    Ok(())
}

fn print_report(report: kokorov_fl4::UnpackReport) {
    println!(
        "[unpack] input={} extracted_files={} extracted_bytes={} output={} manifest={}",
        report.input.display(),
        report.extracted_files,
        report.extracted_bytes,
        report.output.display(),
        report.manifest.display()
    );
}

fn run() -> ToolResult<()> {
    let mut raw = env::args_os().collect::<Vec<_>>();
    let program = raw
        .first()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "fl4_unpack".to_string());
    raw.remove(0);
    if raw.iter().any(|value| value == "-h" || value == "--help") {
        usage(&program);
        return Ok(());
    }
    let path_only = raw.len() == 1 && !raw[0].to_string_lossy().starts_with('-');
    if raw.is_empty() || path_only {
        return interactive(raw.first().map(PathBuf::from));
    }
    let args = parse_args(raw)?;
    if args.list {
        list_archive(&args.input)
    } else {
        debug_assert!(args.non_interactive);
        print_report(unpack_archive(
            &args.input,
            args.output.as_deref(),
            args.overwrite,
        )?);
        Ok(())
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("fl4_unpack: error: {error}");
        std::process::exit(1);
    }
}
