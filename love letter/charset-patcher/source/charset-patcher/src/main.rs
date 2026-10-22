use loveletter_charset_patcher::{patch_description, prepare_patch, write_new_verified};
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
    Ok(directory.join("loveletter_chs.exe"))
}

fn default_output(input: &Path) -> PathBuf {
    input.with_file_name("loveletter_chs_gbk.exe")
}

fn parse_args() -> Result<Option<(PathBuf, PathBuf)>, String> {
    let mut input = None;
    let mut output = None;
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
    let input = input.unwrap_or(default_input()?);
    let output = output.unwrap_or_else(|| default_output(&input));
    Ok(Some((input, output)))
}

fn print_help() {
    println!("Love Letter CHS font charset patcher");
    println!();
    println!("Usage:");
    println!("  loveletter_charset_patcher.exe [INPUT] [-o|--output OUTPUT]");
    println!();
    println!("Changes the seven confirmed font charset values from 0x80 to 0x86.");
    println!("The input and an existing output are never overwritten.");
    println!("Default output: loveletter_chs.exe -> loveletter_chs_gbk.exe");
    println!("This patch does not change the renderer's GBK lead-byte logic.");
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
    println!("[patch] charset=0x80->0x86");
    println!("[patch] sites={}", patch_description());
    println!("[patch] patched_sites={}", report.patched_sites);
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
