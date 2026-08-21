use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

use rendezvous_scr::{Error, Result, TextPolicy, extract, inject_directory, write_json_directory};

#[derive(Debug)]
struct ExtractOptions {
    input: PathBuf,
    output: PathBuf,
    policy: TextPolicy,
    overwrite: bool,
}

#[derive(Debug)]
struct InjectOptions {
    source: PathBuf,
    translation: PathBuf,
    output: PathBuf,
    overwrite: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match arguments.as_slice() {
        [] => interactive(None),
        [argument] if argument == "-h" || argument == "--help" => {
            print_help();
            Ok(())
        }
        [argument] if argument == "-V" || argument == "--version" => {
            println!("rendezvous-scr {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        [command, rest @ ..] if command == "extract" => {
            if rest
                .iter()
                .any(|argument| argument == "-h" || argument == "--help")
            {
                print_extract_help();
                return Ok(());
            }
            run_extract(&parse_extract_options(rest)?)
        }
        [command, rest @ ..] if command == "inject" => {
            if rest
                .iter()
                .any(|argument| argument == "-h" || argument == "--help")
            {
                print_inject_help();
                return Ok(());
            }
            run_inject(&parse_inject_options(rest)?)
        }
        [path] if !path.starts_with('-') => interactive(Some(PathBuf::from(strip_quotes(path)))),
        _ => Err(Error::new("invalid arguments; run with --help")),
    }
}

fn parse_inject_options(arguments: &[String]) -> Result<InjectOptions> {
    let mut source = None;
    let mut translation = None;
    let mut output = None;
    let mut overwrite = false;
    let mut index = 0usize;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "-s" | "--source" => {
                index += 1;
                source = Some(PathBuf::from(required_value(arguments, index, "--source")?));
            }
            "-t" | "--translation" => {
                index += 1;
                translation = Some(PathBuf::from(required_value(
                    arguments,
                    index,
                    "--translation",
                )?));
            }
            "-o" | "--output" => {
                index += 1;
                output = Some(PathBuf::from(required_value(arguments, index, "--output")?));
            }
            "--overwrite" => overwrite = true,
            unknown => return Err(Error::new(format!("unknown inject option: {unknown}"))),
        }
        index += 1;
    }
    Ok(InjectOptions {
        source: source.ok_or_else(|| Error::new("inject requires --source"))?,
        translation: translation.ok_or_else(|| Error::new("inject requires --translation"))?,
        output: output.ok_or_else(|| Error::new("inject requires --output"))?,
        overwrite,
    })
}

fn parse_extract_options(arguments: &[String]) -> Result<ExtractOptions> {
    let mut input = None;
    let mut output = None;
    let mut policy = TextPolicy::FixOrig;
    let mut overwrite = false;
    let mut index = 0usize;

    while index < arguments.len() {
        match arguments[index].as_str() {
            "-i" | "--input" => {
                index += 1;
                input = Some(PathBuf::from(required_value(arguments, index, "--input")?));
            }
            "-o" | "--output" => {
                index += 1;
                output = Some(PathBuf::from(required_value(arguments, index, "--output")?));
            }
            "--raw-text" => policy = TextPolicy::Raw,
            "--overwrite" => overwrite = true,
            unknown => return Err(Error::new(format!("unknown extract option: {unknown}"))),
        }
        index += 1;
    }

    Ok(ExtractOptions {
        input: input.ok_or_else(|| Error::new("extract requires --input"))?,
        output: output.ok_or_else(|| Error::new("extract requires --output"))?,
        policy,
        overwrite,
    })
}

fn required_value<'a>(arguments: &'a [String], index: usize, option: &str) -> Result<&'a str> {
    arguments
        .get(index)
        .map(String::as_str)
        .ok_or_else(|| Error::new(format!("{option} requires a value")))
}

fn run_extract(options: &ExtractOptions) -> Result<()> {
    let extraction = extract(&options.input, options.policy)?;
    write_json_directory(
        &options.input,
        &options.output,
        &extraction.files,
        options.overwrite,
    )?;
    for warning in &extraction.warnings {
        eprintln!("warning: {warning}");
    }
    println!(
        "scanned {} files; parsed {} scripts, {} commands and {} strings",
        extraction.summary.files_scanned,
        extraction.summary.scripts_parsed,
        extraction.summary.commands,
        extraction.summary.strings
    );
    println!(
        "wrote {} JSON files with {} entries: {} messages ({} named), {} choices; {} warnings",
        extraction.summary.json_files,
        extraction.summary.entries,
        extraction.summary.messages,
        extraction.summary.named_messages,
        extraction.summary.choices,
        extraction.summary.warnings
    );
    println!("output directory: {}", options.output.display());
    Ok(())
}

fn run_inject(options: &InjectOptions) -> Result<()> {
    let summary = inject_directory(
        &options.source,
        &options.translation,
        &options.output,
        options.overwrite,
    )?;
    println!(
        "validated {} translation files and {} entries against {} SCR:2005 scripts",
        summary.translation_files, summary.entries, summary.scripts
    );
    println!(
        "changed {} entries: {} messages and {} names; rebuilt {} scripts, {} remained byte-exact",
        summary.changed_entries,
        summary.changed_messages,
        summary.changed_names,
        summary.rebuilt_scripts,
        summary.byte_exact_scripts
    );
    println!(
        "copied {} source files to output directory: {}",
        summary.source_files,
        options.output.display()
    );
    Ok(())
}

fn interactive(prefill: Option<PathBuf>) -> Result<()> {
    println!("Rendezvous SCR:2005 version 5 extractor");
    let mut prefill = prefill;
    loop {
        if prefill.is_none() {
            println!();
            println!("1. Extract text");
            println!("2. Inject translations");
            println!("3. Exit");
            let Some(choice) = prompt("Select", Some("1"))? else {
                return Ok(());
            };
            match choice.trim() {
                "" | "1" => {}
                "2" => {
                    match interactive_inject() {
                        Ok(InteractiveOutcome::Completed) => {}
                        Ok(InteractiveOutcome::Cancelled) => println!("Cancelled."),
                        Err(error) => eprintln!("error: {error}"),
                    }
                    continue;
                }
                "3" => return Ok(()),
                _ => {
                    println!("Unknown selection.");
                    continue;
                }
            }
        }

        match interactive_extract(prefill.take()) {
            Ok(InteractiveOutcome::Completed) => {}
            Ok(InteractiveOutcome::Cancelled) => println!("Cancelled."),
            Err(error) => eprintln!("error: {error}"),
        }
    }
}

fn interactive_inject() -> Result<InteractiveOutcome> {
    let Some(source) = prompt("Source script directory", None)? else {
        return Ok(InteractiveOutcome::Cancelled);
    };
    if source.trim().is_empty() {
        return Ok(InteractiveOutcome::Cancelled);
    }
    let Some(translation) = prompt("Translation JSON directory", None)? else {
        return Ok(InteractiveOutcome::Cancelled);
    };
    if translation.trim().is_empty() {
        return Ok(InteractiveOutcome::Cancelled);
    }
    let default_output = env::current_dir()
        .map_err(Error::from)?
        .join("injected-scripts")
        .to_string_lossy()
        .into_owned();
    let Some(output) = prompt("Output script directory", Some(&default_output))? else {
        return Ok(InteractiveOutcome::Cancelled);
    };
    let output = PathBuf::from(strip_quotes(&output));
    let overwrite = if output.exists() {
        let Some(answer) = prompt(
            "Output directory exists. Replace the whole directory? [y/N]",
            Some("N"),
        )?
        else {
            return Ok(InteractiveOutcome::Cancelled);
        };
        if !is_yes(&answer) {
            return Ok(InteractiveOutcome::Cancelled);
        }
        true
    } else {
        false
    };

    println!();
    println!("Source:      {}", strip_quotes(&source));
    println!("Translation: {}", strip_quotes(&translation));
    println!("Output:      {}", output.display());
    println!("Overwrite:   {}", if overwrite { "yes" } else { "no" });
    let Some(answer) = prompt("Proceed? [y/N]", Some("N"))? else {
        return Ok(InteractiveOutcome::Cancelled);
    };
    if !is_yes(&answer) {
        return Ok(InteractiveOutcome::Cancelled);
    }
    run_inject(&InjectOptions {
        source: PathBuf::from(strip_quotes(&source)),
        translation: PathBuf::from(strip_quotes(&translation)),
        output,
        overwrite,
    })?;
    Ok(InteractiveOutcome::Completed)
}

enum InteractiveOutcome {
    Completed,
    Cancelled,
}

fn interactive_extract(prefill: Option<PathBuf>) -> Result<InteractiveOutcome> {
    let input_default = prefill.as_ref().map(|path| path.to_string_lossy());
    let Some(input) = prompt("Input SCR file or directory", input_default.as_deref())? else {
        return Ok(InteractiveOutcome::Cancelled);
    };
    if input.trim().is_empty() {
        return Ok(InteractiveOutcome::Cancelled);
    }

    let default_output = env::current_dir()
        .map_err(Error::from)?
        .join("extracted-json")
        .to_string_lossy()
        .into_owned();
    let Some(output) = prompt("Output directory", Some(&default_output))? else {
        return Ok(InteractiveOutcome::Cancelled);
    };
    if output.trim().is_empty() {
        return Ok(InteractiveOutcome::Cancelled);
    }

    let Some(normalize) = prompt("Apply fixOrig normalization? [Y/n]", Some("Y"))? else {
        return Ok(InteractiveOutcome::Cancelled);
    };
    let policy = if is_no(&normalize) {
        TextPolicy::Raw
    } else {
        TextPolicy::FixOrig
    };
    let output = PathBuf::from(strip_quotes(&output));
    let overwrite = if output.exists() {
        let Some(answer) = prompt(
            "Output directory exists. Replace the whole directory? [y/N]",
            Some("N"),
        )?
        else {
            return Ok(InteractiveOutcome::Cancelled);
        };
        if !is_yes(&answer) {
            return Ok(InteractiveOutcome::Cancelled);
        }
        true
    } else {
        false
    };

    println!();
    println!("Input:     {}", strip_quotes(&input));
    println!("Output:    {}", output.display());
    println!("Text mode: {}", policy.label());
    println!("Overwrite: {}", if overwrite { "yes" } else { "no" });
    let Some(answer) = prompt("Proceed? [y/N]", Some("N"))? else {
        return Ok(InteractiveOutcome::Cancelled);
    };
    if !is_yes(&answer) {
        return Ok(InteractiveOutcome::Cancelled);
    }

    run_extract(&ExtractOptions {
        input: PathBuf::from(strip_quotes(&input)),
        output,
        policy,
        overwrite,
    })?;
    Ok(InteractiveOutcome::Completed)
}

fn prompt(label: &str, default: Option<&str>) -> Result<Option<String>> {
    match default {
        Some(default) => print!("{label} [{default}]: "),
        None => print!("{label}: "),
    }
    io::stdout().flush().map_err(Error::from)?;
    let mut input = String::new();
    let read = io::stdin().read_line(&mut input).map_err(Error::from)?;
    if read == 0 {
        return Ok(None);
    }
    let input = input.trim_end_matches(['\r', '\n']);
    if input.is_empty() {
        Ok(default.map(str::to_owned).or_else(|| Some(String::new())))
    } else {
        Ok(Some(input.to_owned()))
    }
}

fn strip_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
}

fn is_yes(value: &str) -> bool {
    matches!(value.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

fn is_no(value: &str) -> bool {
    matches!(value.trim().to_ascii_lowercase().as_str(), "n" | "no")
}

fn print_help() {
    println!("rendezvous-scr - SCR:2005 version 5 text extractor and injector");
    println!();
    println!("USAGE:");
    println!("  rendezvous-scr");
    println!("  rendezvous-scr <PATH>");
    println!("  rendezvous-scr extract --input <PATH> --output <DIR> [OPTIONS]");
    println!("  rendezvous-scr inject --source <DIR> --translation <DIR> --output <DIR> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("  extract    Extract each script into a matching UTF-8 JSON file");
    println!("  inject     Rebuild scripts from matching translation JSON files");
    println!();
    println!("Run 'rendezvous-scr <COMMAND> --help' for command options.");
}

fn print_extract_help() {
    println!("USAGE:");
    println!("  rendezvous-scr extract --input <PATH> --output <DIR> [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  -i, --input <PATH>    SCR:2005 file or directory to scan by content");
    println!("  -o, --output <DIR>    Directory for one JSON per source script");
    println!("      --raw-text        Disable legacy fixOrig character normalization");
    println!("      --overwrite       Replace the complete existing output directory");
    println!("  -h, --help            Show this help");
}

fn print_inject_help() {
    println!("USAGE:");
    println!("  rendezvous-scr inject --source <DIR> --translation <DIR> --output <DIR> [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  -s, --source <DIR>       Original source script tree");
    println!("  -t, --translation <DIR>  One matching JSON per translated script");
    println!("  -o, --output <DIR>       Rebuilt copy of the complete source tree");
    println!("      --overwrite          Replace the complete existing output directory");
    println!("  -h, --help               Show this help");
}
