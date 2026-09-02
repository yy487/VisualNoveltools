use oretubar_tob_tool::workflow::{extract_path, inject_path, verify_path};
use oretubar_tob_tool::Result;
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const NAMES_WRITABLE: bool = false;

const HELP: &str = "ORETUBAR TOB0 extraction/injection tool

Usage:
  oretubar-tob extract --input <TOB_FILE_OR_DIR> --output <JSON_FILE_OR_DIR> [--overwrite]
  oretubar-tob inject --input <TOB_FILE_OR_DIR> --translation <JSON_FILE_OR_DIR> --output <NEW_FILE_OR_DIR> [--overwrite]
  oretubar-tob verify --input <TOB_FILE_OR_DIR>

With no arguments, an interactive menu is opened. A single path pre-fills the
input field; no file is written until all paths are shown and confirmed.

The proven source and injection encoding is CP932. Source TOB files are never
modified in place. Existing output requires --overwrite.";

#[derive(Debug)]
struct Arguments {
    mode: String,
    input: PathBuf,
    translation: Option<PathBuf>,
    output: Option<PathBuf>,
    overwrite: bool,
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        return interactive(None);
    }
    if args.len() == 2
        && !matches!(
            args[1].as_str(),
            "extract" | "inject" | "verify" | "-h" | "--help"
        )
    {
        return interactive(Some(PathBuf::from(&args[1])));
    }
    if args
        .iter()
        .any(|argument| matches!(argument.as_str(), "-h" | "--help"))
    {
        println!("{HELP}");
        return Ok(());
    }
    run_arguments(parse_arguments(&args)?)
}

fn parse_arguments(args: &[String]) -> Result<Arguments> {
    let mode = args.get(1).ok_or("missing command")?.clone();
    if !matches!(mode.as_str(), "extract" | "inject" | "verify") {
        return Err(format!("unknown command '{mode}'\n\n{HELP}"));
    }
    let mut input = None;
    let mut translation = None;
    let mut output = None;
    let mut overwrite = false;
    let mut index = 2usize;
    while index < args.len() {
        match args[index].as_str() {
            "--input" => {
                index += 1;
                input = Some(PathBuf::from(
                    args.get(index).ok_or("missing --input value")?,
                ));
            }
            "--translation" => {
                index += 1;
                translation = Some(PathBuf::from(
                    args.get(index).ok_or("missing --translation value")?,
                ));
            }
            "--output" => {
                index += 1;
                output = Some(PathBuf::from(
                    args.get(index).ok_or("missing --output value")?,
                ));
            }
            "--overwrite" => overwrite = true,
            other => return Err(format!("unknown argument '{other}'")),
        }
        index += 1;
    }
    let input = input.ok_or("missing --input")?;
    match mode.as_str() {
        "extract" if output.is_none() => return Err("extract requires --output".to_string()),
        "inject" if output.is_none() || translation.is_none() => {
            return Err("inject requires --translation and --output".to_string())
        }
        "verify" if output.is_some() || translation.is_some() || overwrite => {
            return Err("verify accepts only --input".to_string())
        }
        _ => {}
    }
    Ok(Arguments {
        mode,
        input,
        translation,
        output,
        overwrite,
    })
}

fn run_arguments(arguments: Arguments) -> Result<()> {
    match arguments.mode.as_str() {
        "extract" => {
            let output = arguments.output.as_deref().ok_or("missing output")?;
            let totals = extract_path(&arguments.input, output, arguments.overwrite)?;
            println!(
                "extracted {} entries from {} TOB files into {}",
                totals.entries,
                totals.files,
                output.display()
            );
        }
        "inject" => {
            let translation = arguments
                .translation
                .as_deref()
                .ok_or("missing translation")?;
            let output = arguments.output.as_deref().ok_or("missing output")?;
            let totals = inject_path(
                &arguments.input,
                translation,
                output,
                arguments.overwrite,
                NAMES_WRITABLE,
            )?;
            println!(
                "rebuilt {} TOB files into {}; {} files / {} physical text slots changed",
                totals.files,
                output.display(),
                totals.changed_files,
                totals.changed_entries
            );
        }
        "verify" => {
            let totals = verify_path(&arguments.input)?;
            println!(
                "verified {} TOB files and {} entries; every unchanged round trip is byte-exact; {} real modified cases re-extracted correctly",
                totals.files, totals.entries, totals.verified_modifications
            );
        }
        _ => unreachable!("mode was validated"),
    }
    Ok(())
}

fn interactive(mut prefill: Option<PathBuf>) -> Result<()> {
    loop {
        println!("\nORETUBAR TOB0 tool\n  1) Extract\n  2) Inject\n  3) Verify\n  4) Exit");
        let choice = prompt("Choice", None)?;
        let result = match choice.to_ascii_lowercase().as_str() {
            "1" | "extract" => interactive_extract(prefill.take()),
            "2" | "inject" => interactive_inject(prefill.take()),
            "3" | "verify" => interactive_verify(prefill.take()),
            "4" | "exit" | "quit" | "q" => return Ok(()),
            _ => {
                eprintln!("Unknown choice.");
                continue;
            }
        };
        if let Err(error) = result {
            if error == "end of input" {
                return Err(error);
            }
            eprintln!("error: {error}");
        }
    }
}

fn interactive_extract(prefill: Option<PathBuf>) -> Result<()> {
    let input = prompt_path("Input TOB file/directory", prefill.as_deref())?;
    let default_output = if input.is_file() {
        Path::new("translation.json")
    } else {
        Path::new("translation_json")
    };
    let output = prompt_path("JSON output file/directory", Some(default_output))?;
    let overwrite = approve_output(&output)?;
    println!(
        "Extract\n  input: {}\n  output: {}\n  encoding: CP932",
        input.display(),
        output.display()
    );
    if !confirm("Proceed")? {
        println!("Cancelled.");
        return Ok(());
    }
    run_arguments(Arguments {
        mode: "extract".to_string(),
        input,
        translation: None,
        output: Some(output),
        overwrite,
    })
}

fn interactive_inject(prefill: Option<PathBuf>) -> Result<()> {
    let input = prompt_path("Original TOB file/directory", prefill.as_deref())?;
    let translation = prompt_path(
        "Translation JSON file/directory",
        Some(Path::new("translation_json")),
    )?;
    let default_output = if input.is_file() {
        Path::new("rebuilt.tob")
    } else {
        Path::new("rebuilt")
    };
    let output = prompt_path("Rebuilt output file/directory", Some(default_output))?;
    let overwrite = approve_output(&output)?;
    println!(
        "Inject\n  input: {}\n  translation: {}\n  output: {}\n  encoding: CP932\n  names: read-only",
        input.display(),
        translation.display(),
        output.display()
    );
    if !confirm("Proceed")? {
        println!("Cancelled.");
        return Ok(());
    }
    run_arguments(Arguments {
        mode: "inject".to_string(),
        input,
        translation: Some(translation),
        output: Some(output),
        overwrite,
    })
}

fn interactive_verify(prefill: Option<PathBuf>) -> Result<()> {
    let input = prompt_path("TOB file/directory", prefill.as_deref())?;
    println!("Verify (read only)\n  input: {}", input.display());
    if !confirm("Proceed")? {
        println!("Cancelled.");
        return Ok(());
    }
    run_arguments(Arguments {
        mode: "verify".to_string(),
        input,
        translation: None,
        output: None,
        overwrite: false,
    })
}

fn prompt_path(label: &str, default: Option<&Path>) -> Result<PathBuf> {
    let default = default.map(|path| path.to_string_lossy().into_owned());
    Ok(PathBuf::from(prompt(label, default.as_deref())?))
}

fn prompt(label: &str, default: Option<&str>) -> Result<String> {
    match default {
        Some(value) => print!("{label} [{value}]: "),
        None => print!("{label}: "),
    }
    io::stdout().flush().map_err(|error| error.to_string())?;
    let mut value = String::new();
    if io::stdin()
        .read_line(&mut value)
        .map_err(|error| error.to_string())?
        == 0
    {
        return Err("end of input".to_string());
    }
    let value = value.trim();
    if value.is_empty() {
        default
            .map(str::to_string)
            .ok_or_else(|| format!("{label} is required"))
    } else {
        Ok(value.to_string())
    }
}

fn confirm(label: &str) -> Result<bool> {
    let answer = prompt(&format!("{label}? (y/N)"), Some("N"))?;
    Ok(matches!(answer.to_ascii_lowercase().as_str(), "y" | "yes"))
}

fn approve_output(output: &Path) -> Result<bool> {
    if !output.exists() {
        return Ok(false);
    }
    if confirm("Output exists; overwrite it")? {
        Ok(true)
    } else {
        Err("cancelled".to_string())
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
