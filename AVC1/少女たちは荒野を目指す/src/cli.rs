use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use crate::workflow::{default_extract_output, default_inject_output, extract, inject};
use crate::{ToolResult, error};

const EXTRACT_HELP: &str = r#"mes_extract - extract CP932 MES text to per-file UTF-8 JSON

Usage:
  mes_extract <TXT-or-directory> [-o <output>]

Defaults:
  script.txt -> script.txt.json
  mes/       -> mes_json/

The output must not already exist. A directory input scans .txt files recursively.
Windows drag-and-drop is supported by dropping one TXT file or one directory on the EXE.
"#;

const INJECT_HELP: &str = r#"mes_inject - inject UTF-8 JSON translations into CP932 MES text

Usage:
  mes_inject <source-TXT> <translation-JSON> [-o <output-TXT>]
  mes_inject <source-directory> <JSON-directory> [-o <output-directory>]

Defaults:
  script.txt -> script_injected.txt
  mes/       -> mes_injected/

The output must not already exist. Directory injection copies the complete source tree,
then patches files with matching *.txt.json translations. Drop both required paths on
the EXE in the order shown above.
"#;

pub fn run_extract(args: Vec<OsString>) -> ToolResult<()> {
    if wants_help(&args) {
        print!("{EXTRACT_HELP}");
        return Ok(());
    }
    let parsed = parse_args(args, 1, EXTRACT_HELP)?;
    let source = PathBuf::from(&parsed.positionals[0]);
    let output = match parsed.output {
        Some(path) => PathBuf::from(path),
        None => default_extract_output(&source)?,
    };
    let stats = extract(&source, &output)?;
    println!("[extract] scanned_files={}", stats.scanned_files);
    println!("[extract] json_files={}", stats.json_files);
    println!("[extract] extracted_entries={}", stats.extracted_entries);
    println!("[extract] warnings={}", stats.warnings);
    println!("[extract] output={}", output.display());
    Ok(())
}

pub fn run_inject(args: Vec<OsString>) -> ToolResult<()> {
    if wants_help(&args) {
        print!("{INJECT_HELP}");
        return Ok(());
    }
    let parsed = parse_args(args, 2, INJECT_HELP)?;
    let source = PathBuf::from(&parsed.positionals[0]);
    let translations = PathBuf::from(&parsed.positionals[1]);
    let output = match parsed.output {
        Some(path) => PathBuf::from(path),
        None => default_inject_output(&source)?,
    };
    let stats = inject(&source, &translations, &output)?;
    println!("[inject] json_entries={}", stats.json_entries);
    println!("[inject] patched={}", stats.patched);
    println!("[inject] unchanged={}", stats.unchanged);
    println!("[inject] failed={}", stats.failed);
    println!("[inject] warnings={}", stats.warnings);
    println!("[inject] output={}", output.display());
    Ok(())
}

struct ParsedArgs {
    positionals: Vec<OsString>,
    output: Option<OsString>,
}

fn parse_args(args: Vec<OsString>, expected: usize, help: &str) -> ToolResult<ParsedArgs> {
    let mut positionals = Vec::new();
    let mut output = None;
    let mut iterator = args.into_iter();
    while let Some(argument) = iterator.next() {
        if argument == OsStr::new("-o") || argument == OsStr::new("--output") {
            if output.is_some() {
                return Err(error("--output may only be specified once"));
            }
            output = Some(
                iterator
                    .next()
                    .ok_or_else(|| error("--output requires a path"))?,
            );
        } else if argument.to_string_lossy().starts_with('-') {
            return Err(error(format!(
                "unknown option: {}\n\n{help}",
                argument.to_string_lossy()
            )));
        } else {
            positionals.push(argument);
        }
    }
    if positionals.len() != expected {
        return Err(error(format!(
            "expected {expected} positional path(s), got {}\n\n{help}",
            positionals.len()
        )));
    }
    Ok(ParsedArgs {
        positionals,
        output,
    })
}

fn wants_help(args: &[OsString]) -> bool {
    args.iter()
        .any(|argument| argument == OsStr::new("-h") || argument == OsStr::new("--help"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_output_in_any_position() {
        let args = vec![
            OsString::from("-o"),
            OsString::from("out"),
            OsString::from("source"),
        ];
        let parsed = parse_args(args, 1, EXTRACT_HELP).unwrap();
        assert_eq!(parsed.positionals, vec![OsString::from("source")]);
        assert_eq!(parsed.output, Some(OsString::from("out")));
    }
}
