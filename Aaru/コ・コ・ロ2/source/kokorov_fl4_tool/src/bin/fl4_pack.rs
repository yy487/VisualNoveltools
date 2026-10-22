use kokorov_fl4::{
    default_packed_output_path, inspect_pack_input, pack_archive, PackInputInfo, PackReport,
    ToolError, ToolResult,
};
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Args {
    input: PathBuf,
    output: Option<PathBuf>,
    overwrite: bool,
    non_interactive: bool,
}

fn usage(program: &str) {
    println!(
        "KOKOROV FL4.0 archive packer\n\nUsage:\n  {program} [UNPACKED_DIR_OR_MANIFEST]\n  {program} --non-interactive [--output ARCHIVE.FL4] [--overwrite] UNPACKED_DIR_OR_MANIFEST\n\nModes:\n  no arguments or one path  interactive mode with editable prefilled paths\n  --non-interactive        perform packing without prompts\n\nOptions:\n  -o, --output FILE        packed archive (suggested: <input_dir>_packed.FL4)\n      --overwrite          replace an existing packed output transactionally\n      --non-interactive    confirm that all write parameters are explicit\n  -h, --help               show this help\n\nInput may be any directory containing fl4_manifest.json or an explicitly selected\nmanifest file with any name. The packer reads only manifest-listed payloads,\nkeeps names and bucket order immutable, and never writes inside the input tree."
    );
}

fn parse_args(raw: Vec<OsString>) -> ToolResult<Args> {
    let mut input = None;
    let mut output = None;
    let mut overwrite = false;
    let mut non_interactive = false;
    let mut index = 0usize;
    while index < raw.len() {
        let text = raw[index].to_string_lossy();
        match text.as_ref() {
            "--overwrite" => overwrite = true,
            "--non-interactive" => non_interactive = true,
            "-o" | "--output" => {
                index += 1;
                let value = raw
                    .get(index)
                    .ok_or_else(|| ToolError(format!("{text} requires an archive path")))?;
                output = Some(PathBuf::from(value));
            }
            _ if text.starts_with('-') => {
                return Err(ToolError(format!("unknown option '{text}'; use --help")))
            }
            _ => {
                if input.is_some() {
                    return Err(ToolError("only one pack input is supported".to_string()));
                }
                input = Some(PathBuf::from(&raw[index]));
            }
        }
        index += 1;
    }
    if !non_interactive {
        return Err(ToolError(
            "writing without prompts requires --non-interactive".to_string(),
        ));
    }
    Ok(Args {
        input: input
            .ok_or_else(|| ToolError("missing UNPACKED_DIR_OR_MANIFEST input".to_string()))?,
        output,
        overwrite,
        non_interactive,
    })
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

fn print_input_info(info: &PackInputInfo) {
    println!(
        "[preflight] recognized=KOKOROV_FL4_manifest root={} manifest={} files={} original_bytes={}",
        info.root.display(),
        info.manifest.display(),
        info.entries,
        info.original_file_size
    );
}

fn interactive(prefill: Option<PathBuf>) -> ToolResult<()> {
    println!("KOKOROV FL4.0 interactive pack");
    println!("Enter 0 at a path prompt to cancel.");
    let (input, info) = loop {
        let Some(candidate) = prompt_path("Unpacked directory or manifest", prefill.as_deref())?
        else {
            println!("[cancel] no archive was written");
            return Ok(());
        };
        match inspect_pack_input(&candidate) {
            Ok(info) => {
                print_input_info(&info);
                break (candidate, info);
            }
            Err(error) => eprintln!("Invalid pack input: {error}"),
        }
    };

    let suggested_output = default_packed_output_path(&info.root);
    let Some(mut output) = prompt_path("Packed output", Some(&suggested_output))? else {
        println!("[cancel] no archive was written");
        return Ok(());
    };
    let overwrite = loop {
        if !output.exists() {
            break false;
        }
        let metadata = std::fs::symlink_metadata(&output).map_err(|error| {
            ToolError(format!(
                "cannot inspect output '{}': {error}",
                output.display()
            ))
        })?;
        println!("Output already exists: {}", output.display());
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            println!("This output cannot be replaced. 1 = change output, 0 = cancel");
            match prompt("Choice", None)?.as_str() {
                "1" => {
                    let Some(candidate) = prompt_path("Packed output", Some(&output))? else {
                        println!("[cancel] no archive was written");
                        return Ok(());
                    };
                    output = candidate;
                }
                "0" => {
                    println!("[cancel] no archive was written");
                    return Ok(());
                }
                _ => eprintln!("Enter 1 or 0."),
            }
        } else {
            println!("1 = change output, 2 = replace existing output, 0 = cancel");
            match prompt("Choice", None)?.as_str() {
                "1" => {
                    let Some(candidate) = prompt_path("Packed output", Some(&output))? else {
                        println!("[cancel] no archive was written");
                        return Ok(());
                    };
                    output = candidate;
                }
                "2" => break true,
                "0" => {
                    println!("[cancel] no archive was written");
                    return Ok(());
                }
                _ => eprintln!("Enter 1, 2, or 0."),
            }
        }
    };

    println!("[preflight] operation=pack");
    println!("[preflight] input={}", input.display());
    println!("[preflight] output={}", output.display());
    println!("[preflight] overwrite={overwrite}");
    let confirmation = prompt("Execute? [y/N]", None)?;
    if !confirmation.eq_ignore_ascii_case("y") && !confirmation.eq_ignore_ascii_case("yes") {
        println!("[cancel] no archive was written");
        return Ok(());
    }
    print_report(pack_archive(&input, Some(&output), overwrite)?);
    Ok(())
}

fn print_report(report: PackReport) {
    println!(
        "[pack] input={} manifest={} packed_files={} output_bytes={} output={}",
        report.input.display(),
        report.manifest.display(),
        report.packed_files,
        report.output_bytes,
        report.output.display()
    );
}

fn run() -> ToolResult<()> {
    let mut raw = env::args_os().collect::<Vec<_>>();
    let program = raw
        .first()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "fl4_pack".to_string());
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
    debug_assert!(args.non_interactive);
    print_report(pack_archive(
        &args.input,
        args.output.as_deref(),
        args.overwrite,
    )?);
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("fl4_pack: error: {error}");
        std::process::exit(1);
    }
}
