use ail_text_tool::archive::{pack_snl, unpack_snl};
use ail_text_tool::extract::extract_path;
use ail_text_tool::inject::inject_path;
use ail_text_tool::migrate::migrate_path;
use ail_text_tool::Result;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::PathBuf;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || matches!(args.first().map(String::as_str), Some("-h" | "--help")) {
        println!("{}", help_text());
        return Ok(());
    }
    let command = args.remove(0);
    let parsed = ParsedArgs::parse(args)?;
    match command.as_str() {
        "migrate" => {
            let source = parsed.required_path("--source")?;
            let legacy = parsed.required_path("--legacy")?;
            let output = parsed.required_path("--output")?;
            parsed.finish(&["--source", "--legacy", "--output", "--overwrite"])?;
            let (files, entries, refs) =
                migrate_path(&source, &legacy, &output, parsed.flag("--overwrite"))?;
            println!(
                "migrated {refs} instruction reference(s) into {entries} JSON entry/entries across {files} BIN file(s): {}",
                output.display()
            );
        }
        "extract" => {
            let input = parsed.required_path("--input")?;
            let references = parsed.required_path("--references")?;
            let output = parsed.required_path("--output")?;
            parsed.finish(&["--input", "--references", "--output", "--overwrite"])?;
            let (files, entries) =
                extract_path(&input, &references, &output, parsed.flag("--overwrite"))?;
            println!(
                "extracted {entries} entry/entries from {files} BIN file(s): {}",
                output.display()
            );
        }
        "inject" => {
            let source = parsed.required_path("--source")?;
            let json = parsed.required_path("--json")?;
            let output = parsed.required_path("--output")?;
            parsed.finish(&["--source", "--json", "--output", "--overwrite"])?;
            let (changed, unchanged, entries) =
                inject_path(&source, &json, &output, parsed.flag("--overwrite"))?;
            println!(
                "injected {entries} JSON entry/entries: {changed} rebuilt, {unchanged} byte-identical BIN file(s): {}",
                output.display()
            );
        }
        "unpack-snl" => {
            let input = parsed.required_path("--input")?;
            let output = parsed.required_path("--output")?;
            parsed.finish(&["--input", "--output", "--overwrite"])?;
            let (entries, written) = unpack_snl(&input, &output, parsed.flag("--overwrite"))?;
            println!(
                "unpacked {written} non-empty BIN(s) from {entries} archive entries: {}",
                output.display()
            );
        }
        "pack-snl" => {
            let source = parsed.required_path("--source")?;
            let bins = parsed.required_path("--bins")?;
            let output = parsed.required_path("--output")?;
            parsed.finish(&["--source", "--bins", "--output", "--overwrite"])?;
            let (entries, replaced) =
                pack_snl(&source, &bins, &output, parsed.flag("--overwrite"))?;
            println!(
                "packed {entries} archive entries, replacing {replaced}: {}",
                output.display()
            );
        }
        "help" => println!("{}", help_text()),
        other => return Err(format!("unknown command: {other}\n\n{}", help_text())),
    }
    Ok(())
}

struct ParsedArgs {
    values: HashMap<String, String>,
    flags: HashSet<String>,
}

impl ParsedArgs {
    fn parse(args: Vec<String>) -> Result<Self> {
        let mut values = HashMap::new();
        let mut flags = HashSet::new();
        let mut cursor = 0usize;
        while cursor < args.len() {
            let key = &args[cursor];
            if key == "--overwrite" {
                flags.insert(key.clone());
                cursor += 1;
                continue;
            }
            if !key.starts_with("--") {
                return Err(format!("unexpected positional argument: {key}"));
            }
            let value = args
                .get(cursor + 1)
                .ok_or_else(|| format!("{key} requires a value"))?;
            if value.starts_with("--") {
                return Err(format!("{key} requires a value"));
            }
            if values.insert(key.clone(), value.clone()).is_some() {
                return Err(format!("{key} was specified more than once"));
            }
            cursor += 2;
        }
        Ok(Self { values, flags })
    }

    fn required_path(&self, key: &str) -> Result<PathBuf> {
        self.values
            .get(key)
            .map(PathBuf::from)
            .ok_or_else(|| format!("missing required option {key}"))
    }

    fn flag(&self, key: &str) -> bool {
        self.flags.contains(key)
    }

    fn finish(&self, allowed: &[&str]) -> Result<()> {
        for key in self.values.keys().chain(self.flags.iter()) {
            if !allowed.contains(&key.as_str()) {
                return Err(format!("unknown option: {key}"));
            }
        }
        Ok(())
    }
}

fn help_text() -> &'static str {
    "AIL instruction-based text tool

Commands:
  migrate    --source ORIGINAL_BINS --legacy LEGACY_BINS --output JSON_DIR [--overwrite]
  extract    --input ORIGINAL_BINS --references JSON_DIR --output JSON_DIR [--overwrite]
  inject     --source ORIGINAL_BINS --json JSON_DIR --output BIN_DIR [--overwrite]
  unpack-snl --input SALL.SNL --output BIN_DIR [--overwrite]
  pack-snl   --source ORIGINAL.SNL --bins BIN_DIR --output NEW.SNL [--overwrite]

0047.bin is always excluded from text extraction/migration and copied unchanged during injection.
Ruby [reading] annotations and literal \\n markers are removed from editable translations.
%I/%B/%F remain hidden structural controls in message_parts."
}
