use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use tongern_ks::{
    audit_source_directory, extract_directory, inject_directory, repair_json_directory,
    ExtractOptions, InjectOptions, RepairOptions,
};

type Result<T> = std::result::Result<T, String>;

fn usage(program: &str) {
    eprintln!("Tongern KAG .ks story-text extractor/injector");
    eprintln!();
    eprintln!("usage:");
    eprintln!("  {program} extract SCENARIO_DIR -o JSON_DIR [--macro MACRO.KS]");
    eprintln!("  {program} inject JSON_DIR --source SCENARIO_DIR -o OUTPUT_DIR [--macro MACRO.KS]");
    eprintln!(
        "  {program} repair-json JSON_DIR --source SCENARIO_DIR --name-dictionary NAMES.TOML -o REPAIRED_DIR [--macro MACRO.KS]"
    );
    eprintln!("  {program} DIRECTORY");
    eprintln!();
    eprintln!("drag and drop:");
    eprintln!("  SCENARIO_DIR -> sibling SCENARIO_DIR_json");
    eprintln!("  JSON_DIR with audit.json -> sibling SCENARIO_DIR_injected");
    eprintln!();
    eprintln!("rules:");
    eprintln!("  - source .ks scripts are strict CP932; translation JSON is UTF-8");
    eprintln!("  - translator text contains no physical newlines or KAG tags");
    eprintln!("  - physical-line-final [l] is protected and restored automatically");
    eprintln!("  - inline [l] and all [r] are removed from modified entries");
    eprintln!("  - ruby readings and UI text are excluded");
    eprintln!("  - writable dialogue names are synchronized through speaker macros");
    eprintln!("  - existing output directories are never overwritten");
}

fn parse_extract(program: &str, args: Vec<OsString>) -> Result<ExtractOptions> {
    let mut input = None;
    let mut output = None;
    let mut macro_path = None;
    let mut index = 0usize;

    while index < args.len() {
        let arg = &args[index];
        let text = arg.to_string_lossy();
        match text.as_ref() {
            "-h" | "--help" => {
                usage(program);
                std::process::exit(0);
            }
            "-o" | "--output" => {
                index += 1;
                output = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| format!("{text} requires a path"))?,
                ));
            }
            "--macro" => {
                index += 1;
                macro_path = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--macro requires a path".to_string())?,
                ));
            }
            _ if text.starts_with('-') => return Err(format!("unknown option: {text}")),
            _ => {
                if input.is_some() {
                    return Err(format!("unexpected extra argument: {text}"));
                }
                input = Some(PathBuf::from(arg));
            }
        }
        index += 1;
    }

    let input_dir = input.ok_or_else(|| "missing SCENARIO_DIR".to_string())?;
    Ok(ExtractOptions {
        macro_path: macro_path.unwrap_or_else(|| input_dir.join("macro.ks")),
        input_dir,
        output_dir: output.ok_or_else(|| "missing -o JSON_DIR".to_string())?,
    })
}

fn parse_inject(program: &str, args: Vec<OsString>) -> Result<InjectOptions> {
    let mut json_dir = None;
    let mut source_dir = None;
    let mut output_dir = None;
    let mut macro_path = None;
    let mut index = 0usize;

    while index < args.len() {
        let arg = &args[index];
        let text = arg.to_string_lossy();
        match text.as_ref() {
            "-h" | "--help" => {
                usage(program);
                std::process::exit(0);
            }
            "--source" => {
                index += 1;
                source_dir = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--source requires a path".to_string())?,
                ));
            }
            "-o" | "--output" => {
                index += 1;
                output_dir = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| format!("{text} requires a path"))?,
                ));
            }
            "--macro" => {
                index += 1;
                macro_path = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--macro requires a path".to_string())?,
                ));
            }
            _ if text.starts_with('-') => return Err(format!("unknown option: {text}")),
            _ => {
                if json_dir.is_some() {
                    return Err(format!("unexpected extra argument: {text}"));
                }
                json_dir = Some(PathBuf::from(arg));
            }
        }
        index += 1;
    }

    let source_dir = source_dir.ok_or_else(|| "missing --source SCENARIO_DIR".to_string())?;
    Ok(InjectOptions {
        json_dir: json_dir.ok_or_else(|| "missing JSON_DIR".to_string())?,
        macro_path: macro_path.unwrap_or_else(|| source_dir.join("macro.ks")),
        source_dir,
        output_dir: output_dir.ok_or_else(|| "missing -o OUTPUT_DIR".to_string())?,
    })
}

fn parse_repair(program: &str, args: Vec<OsString>) -> Result<RepairOptions> {
    let mut broken_json_dir = None;
    let mut source_dir = None;
    let mut output_dir = None;
    let mut macro_path = None;
    let mut name_dictionary_path = None;
    let mut index = 0usize;

    while index < args.len() {
        let arg = &args[index];
        let text = arg.to_string_lossy();
        match text.as_ref() {
            "-h" | "--help" => {
                usage(program);
                std::process::exit(0);
            }
            "--source" => {
                index += 1;
                source_dir = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--source requires a path".to_string())?,
                ));
            }
            "--name-dictionary" => {
                index += 1;
                name_dictionary_path =
                    Some(PathBuf::from(args.get(index).ok_or_else(|| {
                        "--name-dictionary requires a path".to_string()
                    })?));
            }
            "-o" | "--output" => {
                index += 1;
                output_dir = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| format!("{text} requires a path"))?,
                ));
            }
            "--macro" => {
                index += 1;
                macro_path = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--macro requires a path".to_string())?,
                ));
            }
            _ if text.starts_with('-') => return Err(format!("unknown option: {text}")),
            _ => {
                if broken_json_dir.is_some() {
                    return Err(format!("unexpected extra argument: {text}"));
                }
                broken_json_dir = Some(PathBuf::from(arg));
            }
        }
        index += 1;
    }

    let source_dir = source_dir.ok_or_else(|| "missing --source SCENARIO_DIR".to_string())?;
    Ok(RepairOptions {
        broken_json_dir: broken_json_dir.ok_or_else(|| "missing JSON_DIR".to_string())?,
        source_dir: source_dir.clone(),
        output_dir: output_dir.ok_or_else(|| "missing -o REPAIRED_DIR".to_string())?,
        macro_path: macro_path.unwrap_or_else(|| source_dir.join("macro.ks")),
        name_dictionary_path: name_dictionary_path
            .ok_or_else(|| "missing --name-dictionary NAMES.TOML".to_string())?,
    })
}

fn sibling_with_suffix(path: &Path, suffix: &str) -> Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| format!("cannot derive output name from {}", path.display()))?;
    let mut output_name = name.to_os_string();
    output_name.push(OsStr::new(suffix));
    Ok(path.with_file_name(output_name))
}

fn print_extract(options: &ExtractOptions) -> Result<()> {
    let report = extract_directory(options)?;
    println!(
        "[extract] scanned_files={} json_files={} extracted_entries={} dialogue={} monologue={} choice={} ui={} name={} ruby_removed={} controls={} warnings={} violations={} output={}",
        report.scanned_files,
        report.json_files,
        report.extracted_entries,
        report.dialogue_entries,
        report.monologue_entries,
        report.choice_entries,
        report.ui_entries,
        report.name_entries,
        report.ruby_removed,
        report.opaque_controls,
        report.warning_count,
        report.violation_count,
        options.output_dir.display()
    );
    Ok(())
}

fn print_inject(options: &InjectOptions) -> Result<()> {
    let report = inject_directory(options)?;
    println!(
        "[inject] json_files={} json_entries={} patched={} unchanged={} output_files={} output={}",
        report.json_files,
        report.json_entries,
        report.patched,
        report.unchanged,
        report.output_files,
        options.output_dir.display()
    );
    Ok(())
}

fn print_repair(options: &RepairOptions) -> Result<()> {
    let report = repair_json_directory(options)?;
    println!(
        "[repair-json] json_files={} entries={} translated_messages={} multipart_entries={} repaired_inner_quotes={} dictionary_entries={} translated_speaker_names={} unmapped_speaker_names={} output={}",
        report.json_files,
        report.entries,
        report.translated_messages,
        report.multipart_entries,
        report.repaired_inner_quotes,
        report.dictionary_entries,
        report.translated_speaker_names,
        report.unmapped_speaker_names.len(),
        options.output_dir.display()
    );
    for name in &report.unmapped_speaker_names {
        eprintln!("[repair-json] unmapped speaker name retained: {name}");
    }
    Ok(())
}

fn run_dragged_directory(path: PathBuf) -> Result<()> {
    if !path.is_dir() {
        return Err(format!(
            "drag-and-drop input must be a directory: {}",
            path.display()
        ));
    }
    if path.join("audit.json").is_file() {
        let source_dir = audit_source_directory(&path)?;
        let options = InjectOptions {
            json_dir: path,
            output_dir: sibling_with_suffix(&source_dir, "_injected")?,
            macro_path: source_dir.join("macro.ks"),
            source_dir,
        };
        print_inject(&options)
    } else {
        let options = ExtractOptions {
            output_dir: sibling_with_suffix(&path, "_json")?,
            macro_path: path.join("macro.ks"),
            input_dir: path,
        };
        print_extract(&options)
    }
}

fn run() -> Result<()> {
    let mut args = env::args_os().collect::<Vec<_>>();
    let program = args
        .first()
        .map(|arg| arg.to_string_lossy().into_owned())
        .unwrap_or_else(|| "tongern_ks".to_string());
    args.remove(0);

    if args.len() == 1 {
        let argument = args.remove(0);
        let text = argument.to_string_lossy();
        if matches!(text.as_ref(), "-h" | "--help") {
            usage(&program);
            return Ok(());
        }
        if !matches!(text.as_ref(), "extract" | "inject" | "repair-json") {
            return run_dragged_directory(PathBuf::from(argument));
        }
        args.push(argument);
    }

    let command = args
        .first()
        .map(|arg| arg.to_string_lossy().into_owned())
        .ok_or_else(|| {
            usage(&program);
            "missing command or dragged directory".to_string()
        })?;
    args.remove(0);

    match command.as_str() {
        "extract" => print_extract(&parse_extract(&program, args)?),
        "inject" => print_inject(&parse_inject(&program, args)?),
        "repair-json" => print_repair(&parse_repair(&program, args)?),
        _ => {
            usage(&program);
            Err(format!("unknown command: {command}"))
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("tongern_ks: error: {error}");
        std::process::exit(1);
    }
}
