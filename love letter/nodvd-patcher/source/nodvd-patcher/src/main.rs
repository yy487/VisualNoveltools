use loveletter_nodvd_patcher::{patch_description, prepare_patch, write_new_verified};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

fn default_input() -> Result<PathBuf, String> {
    let executable = env::current_exe()
        .map_err(|error| format!("cannot locate the patcher executable: {error}"))?;
    let directory = executable
        .parent()
        .ok_or_else(|| "patcher executable has no parent directory".to_owned())?;
    Ok(directory.join("loveletter.exe"))
}

fn default_output(input: &Path) -> PathBuf {
    input.with_file_name("loveletter_chs.exe")
}

fn parse_args() -> Result<Option<(PathBuf, PathBuf)>, String> {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut args = env::args_os().skip(1);

    while let Some(arg) = args.next() {
        if arg == OsStr::new("-h") || arg == OsStr::new("--help") {
            return Ok(None);
        }
        if arg == OsStr::new("-o") || arg == OsStr::new("--output") {
            let value = args
                .next()
                .ok_or_else(|| format!("{} requires a path", arg.to_string_lossy()))?;
            if output.replace(PathBuf::from(value)).is_some() {
                return Err("output was specified more than once".to_owned());
            }
            continue;
        }
        if arg.to_string_lossy().starts_with('-') {
            return Err(format!("unknown option: {}", arg.to_string_lossy()));
        }
        if input.replace(PathBuf::from(arg)).is_some() {
            return Err("only one input EXE is supported".to_owned());
        }
    }

    let input = match input {
        Some(path) => path,
        None => default_input()?,
    };
    let output = output.unwrap_or_else(|| default_output(&input));
    Ok(Some((input, output)))
}

fn print_help() {
    println!("Love Letter NoDVD patcher");
    println!();
    println!("Usage:");
    println!("  loveletter_nodvd_patcher.exe [INPUT] [-o|--output OUTPUT]");
    println!();
    println!("With no INPUT, reads loveletter.exe beside the patcher.");
    println!("The default output is loveletter_chs.exe beside the input.");
    println!("Existing outputs are never overwritten.");
    println!("Drag-and-drop of one loveletter.exe is supported.");
}

fn run() -> Result<(), String> {
    let Some((input_path, output_path)) = parse_args()? else {
        print_help();
        return Ok(());
    };
    if input_path == output_path {
        return Err("input and output paths must differ".to_owned());
    }
    if output_path.exists() {
        return Err(format!(
            "output already exists; refusing to overwrite: {}",
            output_path.display()
        ));
    }

    let source = fs::read(&input_path)
        .map_err(|error| format!("cannot read {}: {error}", input_path.display()))?;
    let (output, report) = prepare_patch(&source)?;
    write_new_verified(&output_path, &output)?;

    println!("[target] game=Love Letter (2002, Win32)");
    println!("[patch] input={}", input_path.display());
    println!("[patch] source_sha256={}", report.source_sha256);
    println!("[patch] site={}", patch_description());
    println!("[patch] changed_bytes={}", report.changed_bytes);
    println!("[patch] output_sha256={}", report.output_sha256);
    println!("[patch] output={}", output_path.display());
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
