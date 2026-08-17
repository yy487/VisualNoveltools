use sinfonia_sb2_tool::{
    inspect_archive, pack_archive, unpack_archive, verify_archive_against_dump,
};
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
    match args.first().map(String::as_str) {
        None => interactive(None),
        Some("-h" | "--help" | "help") => {
            print_help();
            Ok(())
        }
        Some("inspect") => {
            require_len(&args, 2, "inspect <archive.sb>")?;
            cmd_inspect(Path::new(&args[1]))
        }
        Some("unpack") => cmd_unpack_args(&args),
        Some("pack") => cmd_pack_args(&args),
        Some("verify") => {
            require_len(&args, 3, "verify <archive.sb> <dump_dir>")?;
            cmd_verify(Path::new(&args[1]), Path::new(&args[2]))
        }
        Some(_) if args.len() == 1 => interactive(Some(PathBuf::from(&args[0]))),
        Some(other) => Err(format!(
            "unknown command or incomplete path invocation: {other:?}"
        )),
    }
}

fn cmd_unpack_args(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        return Err("usage: unpack <archive.sb> <output_dir> [--overwrite]".to_string());
    }
    let overwrite = parse_flag_only(&args[3..], "--overwrite")?;
    cmd_unpack(Path::new(&args[1]), Path::new(&args[2]), overwrite)
}

fn cmd_pack_args(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        return Err(
            "usage: pack <dump_dir> <output.sb> [--overwrite] [--compare-original <archive.sb>]"
                .to_string(),
        );
    }
    let mut overwrite = false;
    let mut compare_original: Option<PathBuf> = None;
    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--overwrite" => {
                overwrite = true;
                index += 1;
            }
            "--compare-original" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--compare-original requires a path".to_string())?;
                compare_original = Some(PathBuf::from(value));
                index += 2;
            }
            other => return Err(format!("unknown pack option: {other}")),
        }
    }
    cmd_pack(
        Path::new(&args[1]),
        Path::new(&args[2]),
        overwrite,
        compare_original.as_deref(),
    )
}

fn parse_flag_only(args: &[String], allowed: &str) -> Result<bool, String> {
    let mut enabled = false;
    for value in args {
        if value == allowed {
            enabled = true;
        } else {
            return Err(format!("unknown option: {value}"));
        }
    }
    Ok(enabled)
}

fn require_len(args: &[String], expected: usize, usage: &str) -> Result<(), String> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(format!("usage: {usage}"))
    }
}

fn cmd_inspect(input: &Path) -> Result<(), String> {
    let report = inspect_archive(input)?;
    println!("[inspect] file_size={}", report.file_size);
    println!("[inspect] sha256={}", report.sha256);
    println!("[inspect] header_values={:?}", report.parsed.header_values);
    for segment in &report.parsed.segments {
        println!(
            "[segment] index={} tag={} tag_offset=0x{:x} data_offset=0x{:x} size={}",
            segment.index, segment.tag, segment.tag_offset, segment.data_offset, segment.size
        );
        if segment.tag == "CSTR" {
            println!(
                "[cstr] count={} table_size={} pool_size={} entries_total_size={}",
                segment.cstr_count.unwrap_or(0),
                segment.cstr_table_size.unwrap_or(0),
                segment.cstr_pool_size.unwrap_or(0),
                segment.cstr_entries_total_size.unwrap_or(0)
            );
        }
    }
    Ok(())
}

fn cmd_unpack(input: &Path, output: &Path, overwrite: bool) -> Result<(), String> {
    let report = unpack_archive(input, output, overwrite)?;
    println!("[unpack] extracted_files={}", report.extracted_files);
    println!("[unpack] source_size={}", report.source_size);
    println!("[unpack] source_sha256={}", report.source_sha256);
    println!("[unpack] output_dir={}", report.output_dir.display());
    Ok(())
}

fn cmd_pack(
    dump_dir: &Path,
    output: &Path,
    overwrite: bool,
    compare_original: Option<&Path>,
) -> Result<(), String> {
    let report = pack_archive(dump_dir, output, overwrite, compare_original)?;
    println!("[pack] packed_files={}", report.packed_files);
    println!("[pack] output_bytes={}", report.output_bytes);
    println!("[pack] output_sha256={}", report.output_sha256);
    if let Some(equal) = report.byte_equal {
        println!("[pack] byte_equal={equal}");
    }
    println!("[pack] output={}", report.output.display());
    Ok(())
}

fn cmd_verify(archive: &Path, dump_dir: &Path) -> Result<(), String> {
    let report = verify_archive_against_dump(archive, dump_dir)?;
    println!("[verify] archive_size={}", report.archive_size);
    println!("[verify] rebuilt_size={}", report.rebuilt_size);
    println!("[verify] archive_sha256={}", report.archive_sha256);
    println!("[verify] rebuilt_sha256={}", report.rebuilt_sha256);
    println!("[verify] byte_equal={}", report.byte_equal);
    if !report.byte_equal {
        return Err("archive and rebuilt dump are not byte-equal".to_string());
    }
    Ok(())
}

fn interactive(prefill: Option<PathBuf>) -> Result<(), String> {
    let mut prefill = prefill;
    loop {
        println!();
        println!("AGSI SB2 Tool");
        println!("1. Inspect archive");
        println!("2. Unpack archive");
        println!("3. Pack dump directory");
        println!("4. Verify archive against dump");
        println!("0. Exit");
        let choice = prompt("Select", None)?;
        let result = match choice.trim() {
            "1" => interactive_inspect(prefill.as_deref()),
            "2" => interactive_unpack(prefill.as_deref()),
            "3" => interactive_pack(prefill.as_deref()),
            "4" => interactive_verify(prefill.as_deref()),
            "0" => return Ok(()),
            _ => {
                println!("Invalid selection.");
                continue;
            }
        };
        if let Err(error) = result {
            println!("Operation failed: {error}");
        }
        prefill = None;
    }
}

fn interactive_inspect(prefill: Option<&Path>) -> Result<(), String> {
    let input = prompt_path("Archive", prefill)?;
    cmd_inspect(&input)
}

fn interactive_unpack(prefill: Option<&Path>) -> Result<(), String> {
    let input = prompt_path("Archive", prefill.filter(|x| x.is_file()))?;
    let suggested = default_unpack_output(&input);
    let output = prompt_path("Output directory", Some(&suggested))?;
    let overwrite =
        output.exists() && prompt_yes_no("Output exists. Replace this managed dump", false)?;
    println!("Input:  {}", input.display());
    println!("Output: {}", output.display());
    println!("Overwrite: {overwrite}");
    if !prompt_yes_no("Proceed", false)? {
        println!("Cancelled.");
        return Ok(());
    }
    cmd_unpack(&input, &output, overwrite)
}

fn interactive_pack(prefill: Option<&Path>) -> Result<(), String> {
    let dump = prompt_path("Dump directory", prefill.filter(|x| x.is_dir()))?;
    let suggested = dump.with_extension("sb");
    let output = prompt_path("Output archive", Some(&suggested))?;
    let overwrite = output.exists() && prompt_yes_no("Output exists. Replace it", false)?;
    println!("Dump:      {}", dump.display());
    println!("Output:    {}", output.display());
    println!("Overwrite: {overwrite}");
    if !prompt_yes_no("Proceed", false)? {
        println!("Cancelled.");
        return Ok(());
    }
    cmd_pack(&dump, &output, overwrite, None)
}

fn interactive_verify(prefill: Option<&Path>) -> Result<(), String> {
    let archive_default = prefill.filter(|x| x.is_file());
    let dump_default = prefill.filter(|x| x.is_dir());
    let archive = prompt_path("Archive", archive_default)?;
    let dump = prompt_path("Dump directory", dump_default)?;
    cmd_verify(&archive, &dump)
}

fn default_unpack_output(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|x| x.to_str())
        .unwrap_or("archive");
    input.with_file_name(format!("{stem}_unpacked"))
}

fn prompt_path(label: &str, default: Option<&Path>) -> Result<PathBuf, String> {
    let default_text = default.map(|x| x.to_string_lossy().into_owned());
    let raw = prompt(label, default_text.as_deref())?;
    let trimmed = raw.trim();
    let unquoted = if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    if unquoted.is_empty() {
        return Err(format!("{label} cannot be empty"));
    }
    Ok(PathBuf::from(unquoted))
}

fn prompt_yes_no(label: &str, default: bool) -> Result<bool, String> {
    let suffix = if default { "Y/n" } else { "y/N" };
    let value = prompt(&format!("{label} [{suffix}]"), None)?;
    match value.trim().to_ascii_lowercase().as_str() {
        "" => Ok(default),
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => Err("please enter y or n".to_string()),
    }
}

fn prompt(label: &str, default: Option<&str>) -> Result<String, String> {
    match default {
        Some(value) => print!("{label} [{value}]: "),
        None => print!("{label}: "),
    }
    io::stdout()
        .flush()
        .map_err(|e| format!("stdout error: {e}"))?;
    let mut input = String::new();
    let read = io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("stdin error: {e}"))?;
    if read == 0 {
        return Err("input reached EOF".to_string());
    }
    let value = input.trim_end_matches(['\r', '\n']);
    if value.is_empty() {
        if let Some(default) = default {
            return Ok(default.to_string());
        }
    }
    Ok(value.to_string())
}

fn print_help() {
    println!("AGSI SB2 structure-aware archive tool");
    println!();
    println!("Interactive:");
    println!("  sinfonia-sb2-tool.exe");
    println!("  sinfonia-sb2-tool.exe <archive-or-dump-path>");
    println!();
    println!("Non-interactive:");
    println!("  sinfonia-sb2-tool.exe inspect <archive.sb>");
    println!("  sinfonia-sb2-tool.exe unpack <archive.sb> <output_dir> [--overwrite]");
    println!("  sinfonia-sb2-tool.exe pack <dump_dir> <output.sb> [--overwrite] [--compare-original <archive.sb>]");
    println!("  sinfonia-sb2-tool.exe verify <archive.sb> <dump_dir>");
    println!();
    println!(
        "Writes never modify the source archive. Existing outputs require explicit overwrite;"
    );
    println!("directory overwrite is limited to dumps carrying this tool's manifest format.");
}
