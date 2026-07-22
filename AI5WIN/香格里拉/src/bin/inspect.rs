use shangri_la1_mes::script::{decode_cp932_exact, ParameterValue, Script};
use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HELP: &str = "Shangri-La 1 decompressed MES inspector

Usage:
  inspect.exe [--samples COUNT] INPUT [INPUT ...]

INPUT may be one decompressed MES/LIB file or a flat directory. The inspector
parses the runtime instruction grammar, validates CP932 text, and verifies a
byte-exact parse/rebuild round trip. It does not create translation JSON.

Options:
  --samples COUNT  Maximum representative text samples per class (default: 8)
  -h, --help       Show this help";

#[derive(Debug)]
struct Args {
    samples: usize,
    inputs: Vec<PathBuf>,
}

#[derive(Debug)]
struct Report {
    files: usize,
    bytes: u64,
    instructions: u64,
    opcode_counts: [u64; 256],
    text_counts: BTreeMap<&'static str, u64>,
    names: BTreeMap<String, u64>,
    japanese_parameters: u64,
    warnings: u64,
    samples: BTreeMap<&'static str, Vec<Sample>>,
}

impl Default for Report {
    fn default() -> Self {
        Self {
            files: 0,
            bytes: 0,
            instructions: 0,
            opcode_counts: [0; 256],
            text_counts: BTreeMap::new(),
            names: BTreeMap::new(),
            japanese_parameters: 0,
            warnings: 0,
            samples: BTreeMap::new(),
        }
    }
}

#[derive(Debug)]
struct Sample {
    file: String,
    offset: usize,
    text: String,
}

#[derive(Debug, PartialEq, Eq)]
enum TextClass {
    Named(String),
    Unnamed,
    NonJapanese,
    BracketAnomaly,
}

impl TextClass {
    fn label(&self) -> &'static str {
        match self {
            Self::Named(_) => "named",
            Self::Unnamed => "unnamed",
            Self::NonJapanese => "non_japanese",
            Self::BracketAnomaly => "bracket_anomaly",
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let Some(args) = parse_args(env::args_os().skip(1))? else {
        println!("{HELP}");
        return Ok(());
    };
    let files = collect_files(&args.inputs)?;
    let mut report = Report::default();

    for path in files {
        inspect_file(&path, args.samples, &mut report)?;
    }

    println!(
        "[inspect] files={} bytes={} instructions={} opcode_kinds={} text_entries={} named={} unnamed={} non_japanese={} bracket_anomaly={} japanese_parameters={} byte_exact_roundtrips={} warnings={}",
        report.files,
        report.bytes,
        report.instructions,
        report
            .opcode_counts
            .iter()
            .filter(|count| **count != 0)
            .count(),
        report.text_counts.values().sum::<u64>(),
        count_for(&report, "named"),
        count_for(&report, "unnamed"),
        count_for(&report, "non_japanese"),
        count_for(&report, "bracket_anomaly"),
        report.japanese_parameters,
        report.files,
        report.warnings
    );
    print!("[opcode-counts]");
    for (opcode, count) in report.opcode_counts.iter().enumerate() {
        if *count != 0 {
            print!(" {opcode:02X}={count}");
        }
    }
    println!();
    for (label, samples) in &report.samples {
        for sample in samples {
            println!(
                "[text-sample] kind={label} file={} offset=0x{:X} text={:?}",
                sample.file,
                sample.offset,
                abbreviate(&sample.text, 120)
            );
        }
    }
    for (name, count) in sorted_names(&report.names) {
        println!("[name] count={count} value={name:?}");
    }
    Ok(())
}

fn inspect_file(path: &Path, sample_limit: usize, report: &mut Report) -> Result<(), String> {
    let bytes =
        fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let script = Script::parse(&bytes)
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;
    let rebuilt = script
        .rebuild()
        .map_err(|error| format!("cannot rebuild {}: {error}", path.display()))?;
    if rebuilt != bytes {
        return Err(format!(
            "byte-exact parse/rebuild mismatch: {}",
            path.display()
        ));
    }

    let mut file_text_counts: BTreeMap<&'static str, u64> = BTreeMap::new();
    for instruction in &script.instructions {
        report.opcode_counts[usize::from(instruction.opcode)] += 1;
        if instruction.opcode == 0x01 {
            let raw = instruction
                .text
                .as_deref()
                .expect("opcode 0x01 parser always records its text bytes");
            match decode_cp932_exact(raw) {
                Ok(text) => {
                    let class = classify_text(&text);
                    let label = class.label();
                    *file_text_counts.entry(label).or_default() += 1;
                    *report.text_counts.entry(label).or_default() += 1;
                    if let TextClass::Named(name) = class {
                        *report.names.entry(name).or_default() += 1;
                    }
                    let samples = report.samples.entry(label).or_default();
                    if samples.len() < sample_limit {
                        samples.push(Sample {
                            file: path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned(),
                            offset: instruction.offset,
                            text,
                        });
                    }
                }
                Err(_) => {
                    *file_text_counts.entry("decode_error").or_default() += 1;
                    *report.text_counts.entry("decode_error").or_default() += 1;
                    report.warnings += 1;
                }
            }
        }
        for parameter in &instruction.parameters {
            let ParameterValue::String(raw) = &parameter.value else {
                continue;
            };
            match decode_cp932_exact(raw) {
                Ok(text) if is_japanese(&text) => report.japanese_parameters += 1,
                Ok(_) => {}
                Err(_) => report.warnings += 1,
            }
        }
    }

    report.files += 1;
    report.bytes += bytes.len() as u64;
    report.instructions += script.instructions.len() as u64;
    println!(
        "[file] name={} size={} table_offsets={} code_offset=0x{:X} instructions={} named={} unnamed={} non_japanese={} bracket_anomaly={} decode_error={} byte_exact=true",
        path.file_name().unwrap_or_default().to_string_lossy(),
        bytes.len(),
        script.offsets.len(),
        script.code_offset,
        script.instructions.len(),
        file_text_counts.get("named").copied().unwrap_or(0),
        file_text_counts.get("unnamed").copied().unwrap_or(0),
        file_text_counts.get("non_japanese").copied().unwrap_or(0),
        file_text_counts
            .get("bracket_anomaly")
            .copied()
            .unwrap_or(0),
        file_text_counts.get("decode_error").copied().unwrap_or(0)
    );
    Ok(())
}

fn classify_text(text: &str) -> TextClass {
    const OPEN: char = '\u{ff3b}';
    const CLOSE: char = '\u{ff3d}';

    if let Some(rest) = text.strip_prefix(OPEN) {
        if let Some(close) = rest.find(CLOSE) {
            let name = &rest[..close];
            let message = &rest[close + CLOSE.len_utf8()..];
            if !name.is_empty()
                && !message.is_empty()
                && !name.contains(OPEN)
                && !name.contains(CLOSE)
                && !message.contains(OPEN)
            {
                return TextClass::Named(name.to_string());
            }
        }
        return TextClass::BracketAnomaly;
    }
    if text.contains(OPEN) || text.contains(CLOSE) {
        return TextClass::BracketAnomaly;
    }
    if is_japanese(text) {
        TextClass::Unnamed
    } else {
        TextClass::NonJapanese
    }
}

fn is_japanese(text: &str) -> bool {
    text.chars().any(|character| {
        ('\u{3040}'..='\u{30ff}').contains(&character)
            || ('\u{3400}'..='\u{9fff}').contains(&character)
            || ('\u{3000}'..='\u{303f}').contains(&character)
    })
}

fn abbreviate(text: &str, limit: usize) -> String {
    let mut characters = text.chars();
    let mut result: String = characters.by_ref().take(limit).collect();
    if characters.next().is_some() {
        result.push_str("...");
    }
    result
}

fn count_for(report: &Report, label: &'static str) -> u64 {
    report.text_counts.get(label).copied().unwrap_or(0)
}

fn sorted_names(names: &BTreeMap<String, u64>) -> Vec<(&str, u64)> {
    let mut values: Vec<_> = names
        .iter()
        .map(|(name, count)| (name.as_str(), *count))
        .collect();
    values.sort_unstable_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
    values
}

fn collect_files(inputs: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for input in inputs {
        let metadata = fs::metadata(input)
            .map_err(|error| format!("cannot inspect input {}: {error}", input.display()))?;
        if metadata.is_file() {
            files.push(input.clone());
            continue;
        }
        if !metadata.is_dir() {
            return Err(format!(
                "input is neither a regular file nor a directory: {}",
                input.display()
            ));
        }
        for item in fs::read_dir(input)
            .map_err(|error| format!("cannot read input directory {}: {error}", input.display()))?
        {
            let item = item.map_err(|error| {
                format!(
                    "cannot enumerate input directory {}: {error}",
                    input.display()
                )
            })?;
            if !item
                .file_type()
                .map_err(|error| format!("cannot inspect {}: {error}", item.path().display()))?
                .is_file()
            {
                return Err(format!(
                    "input directory must be flat and contain only files: {}",
                    item.path().display()
                ));
            }
            files.push(item.path());
        }
    }
    files.sort_unstable_by_key(|path| {
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_uppercase()
    });
    Ok(files)
}

fn parse_args(mut args: impl Iterator<Item = OsString>) -> Result<Option<Args>, String> {
    let mut samples = 8usize;
    let mut inputs = Vec::new();
    while let Some(arg) = args.next() {
        if arg == "-h" || arg == "--help" {
            return Ok(None);
        }
        if arg == "--samples" {
            let value = args
                .next()
                .ok_or_else(|| "--samples requires a nonnegative integer".to_string())?;
            samples = value
                .to_string_lossy()
                .parse()
                .map_err(|_| "--samples requires a nonnegative integer".to_string())?;
            continue;
        }
        if arg.to_string_lossy().starts_with('-') {
            return Err(format!("unknown option: {}", arg.to_string_lossy()));
        }
        inputs.push(arg.into());
    }
    if inputs.is_empty() {
        return Err("no decompressed MES input was provided; use --help for usage".to_string());
    }
    Ok(Some(Args { samples, inputs }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_fullwidth_bracket_name() {
        assert_eq!(
            classify_text("\u{ff3b}\u{4e94}\u{6708}\u{ff3d}\u{306f}\u{3058}\u{3081}\u{307e}\u{3057}\u{3066}\u{3002}"),
            TextClass::Named("\u{4e94}\u{6708}".to_string())
        );
    }

    #[test]
    fn classifies_narration_without_name() {
        assert_eq!(
            classify_text("\u{3060}\u{308c}\u{3082}\u{3044}\u{306a}\u{3044}\u{3002}"),
            TextClass::Unnamed
        );
    }
}
