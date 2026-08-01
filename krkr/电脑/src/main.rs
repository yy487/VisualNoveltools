use diannao_ks::{
    audit_source_directory, extract_directory, inject_directory, ExtractOptions, InjectOptions,
};
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, String>;

fn usage(program: &str) {
    eprintln!("Diannao KAG .ks state-machine text extractor/injector");
    eprintln!();
    eprintln!("usage:");
    eprintln!("  {program} extract SCENARIO_DIR -o JSON_DIR");
    eprintln!("  {program} inject JSON_DIR --source SCENARIO_DIR -o OUTPUT_DIR");
    eprintln!("  {program} DIRECTORY");
    eprintln!();
    eprintln!("drag and drop:");
    eprintln!("  SCENARIO_DIR -> sibling SCENARIO_DIR_json");
    eprintln!("  JSON_DIR with audit.json -> sibling SCENARIO_DIR_injected");
    eprintln!();
    eprintln!("rules:");
    eprintln!("  - source .ks scripts must be BOM-less, byte-exact CP932");
    eprintln!("  - translation interchange is one UTF-8 JSON file per .ks file");
    eprintln!("  - scr_msg and underscore-prefixed fields are immutable");
    eprintln!("  - each body text span is translated directly through message");
    eprintln!("  - source files and existing output directories are never overwritten");
}

fn sibling_with_suffix(path: &Path, suffix: &str) -> Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| format!("cannot derive output name from {}", path.display()))?;
    let mut output_name = name.to_os_string();
    output_name.push(OsStr::new(suffix));
    Ok(path.with_file_name(output_name))
}

fn parse_extract(program: &str, args: Vec<OsString>) -> Result<ExtractOptions> {
    let mut input = None;
    let mut output = None;
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
    Ok(ExtractOptions {
        input_dir: input.ok_or_else(|| "missing SCENARIO_DIR".to_string())?,
        output_dir: output.ok_or_else(|| "missing -o JSON_DIR".to_string())?,
    })
}

fn parse_inject(program: &str, args: Vec<OsString>) -> Result<InjectOptions> {
    let mut json_dir = None;
    let mut source_dir = None;
    let mut output_dir = None;
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
    Ok(InjectOptions {
        json_dir: json_dir.ok_or_else(|| "missing JSON_DIR".to_string())?,
        source_dir: source_dir.ok_or_else(|| "missing --source SCENARIO_DIR".to_string())?,
        output_dir: output_dir.ok_or_else(|| "missing -o OUTPUT_DIR".to_string())?,
    })
}

fn print_extract(options: &ExtractOptions) -> Result<()> {
    let report = extract_directory(options)?;
    println!(
        "[extract] scanned_files={} json_files={} extracted_entries={} body={} choice={} controls={} dash={} wait={} ruby={} font={} emb={} warnings={} output={}",
        report.scanned_files,
        report.json_files,
        report.extracted_entries,
        report.body_entries,
        report.choice_entries,
        report.tracked_controls,
        report.dash_controls,
        report.wait_controls,
        report.ruby_controls,
        report.font_controls,
        report.emb_controls,
        report.warning_count,
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
            source_dir,
        };
        print_inject(&options)
    } else {
        let options = ExtractOptions {
            output_dir: sibling_with_suffix(&path, "_json")?,
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
        .unwrap_or_else(|| "diannao_ks".to_string());
    args.remove(0);

    if args.len() == 1 {
        let argument = args.remove(0);
        let text = argument.to_string_lossy();
        if matches!(text.as_ref(), "-h" | "--help") {
            usage(&program);
            return Ok(());
        }
        if !matches!(text.as_ref(), "extract" | "inject") {
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
        _ => {
            usage(&program);
            Err(format!("unknown command: {command}"))
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("diannao_ks: error: {error}");
        std::process::exit(1);
    }
}
