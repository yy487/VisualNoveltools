use sbridge_tools::speaker::SpeakerMap;
use sbridge_tools::workflow::{MANIFEST_NAME, extract, inject};
use std::env;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const HELP: &str = r#"sbridge-gsc - Seven Bridges GSC text extraction/injection

USAGE:
  sbridge-gsc extract <INPUT> --output <DIR> --yes [--speaker-map <JSON>]
  sbridge-gsc inject <JSON_DIR> --output <DIR> --yes [--source <SOURCE_ROOT>]
  sbridge-gsc <PATH>              Interactive prefill (drag-and-drop friendly)
  sbridge-gsc                     Interactive session

POLICY:
  Extraction writes UTF-8 JSON. Original ^n controls are removed from scr_msg
  and message because the game has runtime automatic wrapping.
  Only message is writable. name is context-only and is never imported.
  Outputs must not already exist; source files are never modified.
"#;

fn main() {
    if let Err(error) = run() {
        eprintln!("[error] {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    if args.is_empty() {
        return interactive(None);
    }
    if flag(&args[0], "-h") || flag(&args[0], "--help") {
        print!("{HELP}");
        return Ok(());
    }
    if flag(&args[0], "extract") {
        return run_extract(&args[1..]);
    }
    if flag(&args[0], "inject") {
        return run_inject(&args[1..]);
    }
    if args.len() == 1 {
        return interactive(Some(PathBuf::from(&args[0])));
    }
    Err(format!("unrecognized arguments\n\n{HELP}").into())
}

fn run_extract(args: &[OsString]) -> Result<(), Box<dyn Error>> {
    let input = positional(args)?;
    let output = option_path(args, "--output", Some("-o"))?.ok_or("extract requires --output")?;
    require_yes(args)?;
    let map_path = option_path(args, "--speaker-map", None)?;
    reject_unknown(args, &["--output", "-o", "--speaker-map", "--yes"])?;
    let map = map_path.as_deref().map(load_speaker_map).transpose()?;
    let result = extract(&input, &output, map.as_ref(), map_path.as_deref())?;
    println!(
        "Done: {} GSC file(s), {} entries; {} file(s) preserved an opaque physical tail.",
        result.files, result.entries, result.opaque_tail_files
    );
    println!("JSON: {}", output.display());
    Ok(())
}

fn run_inject(args: &[OsString]) -> Result<(), Box<dyn Error>> {
    let input = positional(args)?;
    let output = option_path(args, "--output", Some("-o"))?.ok_or("inject requires --output")?;
    require_yes(args)?;
    let source = option_path(args, "--source", None)?;
    reject_unknown(args, &["--output", "-o", "--source", "--yes"])?;
    let result = inject(&input, &output, source.as_deref())?;
    println!(
        "Done: {} GSC file(s), {} entries, {} edited message(s).",
        result.files, result.entries, result.edited_entries
    );
    println!("Rebuilt files: {}", output.display());
    Ok(())
}

fn interactive(mut prefill: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    loop {
        println!("\nSeven Bridges GSC");
        println!("1) Extract GSC -> UTF-8 JSON");
        println!("2) Inject message fields -> rebuilt GSC");
        println!("0) Exit");
        let suggested_mode = prefill
            .as_ref()
            .is_some_and(|path| path.join(MANIFEST_NAME).is_file());
        let choice = if prefill.is_some() {
            prompt(&format!(
                "Mode [{}]",
                if suggested_mode { "2" } else { "1" }
            ))?
        } else {
            prompt("Select")?
        };
        let selected = if choice.trim().is_empty() {
            if suggested_mode { "2" } else { "1" }
        } else {
            choice.trim()
        };
        match selected {
            "0" | "q" | "Q" => return Ok(()),
            "1" => {
                let input = take_or_prompt(&mut prefill, "GSC file or directory")?;
                let suggested = suggested_output(&input, "_json");
                let output = prompt_path("Output directory", &suggested)?;
                let default_map = PathBuf::from("data/speaker_map.json");
                let map_suggestion = default_map.is_file().then_some(default_map);
                let map_path = match map_suggestion {
                    Some(path) => {
                        let entered = prompt(&format!("Speaker map [{}]", path.display()))?;
                        Some(if entered.trim().is_empty() {
                            path
                        } else {
                            PathBuf::from(entered.trim())
                        })
                    }
                    None => {
                        let entered = prompt("Speaker map (blank for none)")?;
                        (!entered.trim().is_empty()).then(|| PathBuf::from(entered.trim()))
                    }
                };
                println!("Input (read-only): {}", input.display());
                println!("Output: {}", output.display());
                if yes(&prompt("Write extraction? [y/N]")?) {
                    let map = map_path.as_deref().map(load_speaker_map).transpose()?;
                    match extract(&input, &output, map.as_ref(), map_path.as_deref()) {
                        Ok(result) => println!(
                            "Done: {} file(s), {} entries, {} opaque-tail file(s).",
                            result.files, result.entries, result.opaque_tail_files
                        ),
                        Err(error) => eprintln!("[error] {error}"),
                    }
                } else {
                    println!("Cancelled; nothing was written.");
                }
            }
            "2" => {
                let input = take_or_prompt(&mut prefill, "Translation JSON directory")?;
                let suggested = suggested_output(&input, "_rebuilt");
                let output = prompt_path("Output directory", &suggested)?;
                let source_value = prompt("Source root override (blank uses manifest)")?;
                let source =
                    (!source_value.trim().is_empty()).then(|| PathBuf::from(source_value.trim()));
                println!("Translation: {}", input.display());
                println!("Output: {}", output.display());
                println!("Writable field: message only (name is ignored).");
                if yes(&prompt("Write rebuilt files? [y/N]")?) {
                    match inject(&input, &output, source.as_deref()) {
                        Ok(result) => println!(
                            "Done: {} file(s), {} entries, {} edited message(s).",
                            result.files, result.entries, result.edited_entries
                        ),
                        Err(error) => eprintln!("[error] {error}"),
                    }
                } else {
                    println!("Cancelled; nothing was written.");
                }
            }
            _ => eprintln!("Unknown selection."),
        }
    }
}

fn load_speaker_map(path: &Path) -> Result<SpeakerMap, Box<dyn Error>> {
    let map: SpeakerMap = serde_json::from_slice(&fs::read(path)?)?;
    map.validate()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(map)
}

fn positional(args: &[OsString]) -> Result<PathBuf, Box<dyn Error>> {
    args.first()
        .filter(|value| !value.to_string_lossy().starts_with('-'))
        .map(PathBuf::from)
        .ok_or_else(|| "missing input path".into())
}

fn option_path(
    args: &[OsString],
    long: &str,
    short: Option<&str>,
) -> Result<Option<PathBuf>, Box<dyn Error>> {
    let mut found = None;
    let mut index = 1usize;
    while index < args.len() {
        if flag(&args[index], long) || short.is_some_and(|name| flag(&args[index], name)) {
            index += 1;
            let value = args
                .get(index)
                .ok_or_else(|| format!("{long} requires a path"))?;
            if found.replace(PathBuf::from(value)).is_some() {
                return Err(format!("{long} was supplied more than once").into());
            }
        }
        index += 1;
    }
    Ok(found)
}

fn require_yes(args: &[OsString]) -> Result<(), Box<dyn Error>> {
    if args.iter().any(|value| flag(value, "--yes")) {
        Ok(())
    } else {
        Err("non-interactive operation requires --yes; omit the subcommand for prompts".into())
    }
}

fn reject_unknown(args: &[OsString], valid: &[&str]) -> Result<(), Box<dyn Error>> {
    let mut index = 1usize;
    while index < args.len() {
        let value = args[index].to_string_lossy();
        if !value.starts_with('-') {
            return Err(format!("unexpected positional argument: {value}").into());
        }
        if !valid.iter().any(|name| value.eq_ignore_ascii_case(name)) {
            return Err(format!("unknown option: {value}").into());
        }
        if matches!(
            value.as_ref(),
            "--output" | "-o" | "--speaker-map" | "--source"
        ) {
            index += 1;
            if index >= args.len() {
                return Err(format!("{value} requires a value").into());
            }
        }
        index += 1;
    }
    Ok(())
}

fn take_or_prompt(prefill: &mut Option<PathBuf>, label: &str) -> io::Result<PathBuf> {
    if let Some(path) = prefill.take() {
        println!("{label}: {}", path.display());
        Ok(path)
    } else {
        Ok(PathBuf::from(prompt(label)?.trim()))
    }
}

fn suggested_output(input: &Path, suffix: &str) -> PathBuf {
    let mut name = input
        .file_name()
        .unwrap_or_else(|| OsStr::new("sbridge"))
        .to_os_string();
    name.push(suffix);
    input.with_file_name(name)
}

fn prompt_path(label: &str, suggested: &Path) -> io::Result<PathBuf> {
    let entered = prompt(&format!("{label} [{}]", suggested.display()))?;
    Ok(if entered.trim().is_empty() {
        suggested.to_owned()
    } else {
        PathBuf::from(entered.trim())
    })
}

fn prompt(label: &str) -> io::Result<String> {
    print!("{label}: ");
    io::stdout().flush()?;
    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        return Ok("0".to_owned());
    }
    Ok(line.trim_end_matches(['\r', '\n']).to_owned())
}

fn yes(value: &str) -> bool {
    matches!(value.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

fn flag(value: &OsStr, expected: &str) -> bool {
    value.to_string_lossy().eq_ignore_ascii_case(expected)
}
