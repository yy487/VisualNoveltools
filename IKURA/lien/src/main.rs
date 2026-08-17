use lien_isf::archive::default_unpack_output;
use lien_isf::text::EncodingRoute;
use lien_isf::workflow::{
    TRANSLATION_MANIFEST, extract_archive, inject_archive, pack_archive, unpack_archive,
    verify_archive,
};
use lien_isf::{Result, fail};
use std::collections::{HashMap, HashSet};
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const HELP: &str = r#"Lien DRS/ISF localization tool

Usage:
  lien-isf-tool unpack  --input <DRS> --output <DIR> [--overwrite]
  lien-isf-tool pack    --input <DIR> --manifest <JSON> --output <DRS> [--overwrite]
  lien-isf-tool extract --input <DRS> --output <DIR> [--overwrite]
  lien-isf-tool inject  --source <DRS> --translation <DIR> --output <DRS> --encoding <cp932|gbk> [--overwrite]
  lien-isf-tool verify  --input <DRS> [--structure-only]
  lien-isf-tool [PATH]

No arguments, or one path without a subcommand, starts the interactive session.
All write commands require an explicit output in non-interactive mode. Existing
outputs are rejected unless --overwrite is supplied. Source files are never
overwritten implicitly.

Encoding routes:
  cp932  Strict native Lien text tokens; no glyph mapping or substitution table.
  gbk    Strict GBK double-byte text for a separately patched runtime.
"#;

fn main() {
    if let Err(error) = run() {
        eprintln!("[error] {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    if args.is_empty() {
        return interactive(None);
    }
    if args.len() == 1 {
        let value = args[0].to_string_lossy();
        if value == "-h" || value == "--help" || value == "help" {
            print!("{HELP}");
            return Ok(());
        }
        if !value.starts_with('-')
            && !matches!(
                value.as_ref(),
                "unpack" | "pack" | "extract" | "inject" | "verify"
            )
        {
            return interactive(Some(PathBuf::from(&args[0])));
        }
    }

    let command = args[0].to_string_lossy().into_owned();
    let options = Options::parse(&args[1..])?;
    match command.as_str() {
        "unpack" => {
            options.reject_unknown(&["input", "output"])?;
            options.reject_flags(true, false)?;
            let input = options.required_path("input")?;
            let output = options.required_path("output")?;
            let files = unpack_archive(&input, &output, options.overwrite)?;
            println!("[unpack] extracted_files={files}");
            println!("[unpack] output={}", output.display());
        }
        "pack" => {
            options.reject_unknown(&["input", "manifest", "output"])?;
            options.reject_flags(true, false)?;
            let input = options.required_path("input")?;
            let manifest = options.required_path("manifest")?;
            let output = options.required_path("output")?;
            let (files, bytes) = pack_archive(&input, &manifest, &output, options.overwrite)?;
            println!("[pack] packed_files={files}");
            println!("[pack] output_bytes={bytes}");
            println!("[pack] output={}", output.display());
        }
        "extract" => {
            options.reject_unknown(&["input", "output"])?;
            options.reject_flags(true, false)?;
            let input = options.required_path("input")?;
            let output = options.required_path("output")?;
            let stats = extract_archive(&input, &output, options.overwrite)?;
            print_extract_stats(&stats, &output);
        }
        "inject" => {
            options.reject_unknown(&["source", "translation", "output", "encoding"])?;
            options.reject_flags(true, false)?;
            let source = options.required_path("source")?;
            let translation = options.required_path("translation")?;
            let output = options.required_path("output")?;
            let route: EncodingRoute = options.required_string("encoding")?.parse()?;
            let stats = inject_archive(&source, &translation, &output, route, options.overwrite)?;
            print_inject_stats(&stats, &output, route);
        }
        "verify" => {
            options.reject_unknown(&["input"])?;
            options.reject_flags(false, true)?;
            let input = options.required_path("input")?;
            run_verify(&input, options.structure_only)?;
        }
        _ => return fail(format!("unknown command {command:?}; use --help")),
    }
    Ok(())
}

struct Options {
    values: HashMap<String, OsString>,
    overwrite: bool,
    structure_only: bool,
}

impl Options {
    fn parse(args: &[OsString]) -> Result<Self> {
        let mut values = HashMap::new();
        let mut overwrite = false;
        let mut structure_only = false;
        let mut position = 0;
        while position < args.len() {
            let flag = args[position].to_string_lossy();
            if flag == "--overwrite" {
                if overwrite {
                    return fail("--overwrite was supplied more than once");
                }
                overwrite = true;
                position += 1;
                continue;
            }
            if flag == "--structure-only" {
                if structure_only {
                    return fail("--structure-only was supplied more than once");
                }
                structure_only = true;
                position += 1;
                continue;
            }
            let Some(name) = flag.strip_prefix("--") else {
                return fail(format!("unexpected positional argument: {flag}"));
            };
            if name.is_empty() {
                return fail("invalid empty option name");
            }
            position += 1;
            let value = args
                .get(position)
                .ok_or_else(|| format!("missing value for --{name}"))?;
            if value.to_string_lossy().starts_with("--") {
                return fail(format!("missing value for --{name}"));
            }
            if values.insert(name.to_owned(), value.clone()).is_some() {
                return fail(format!("--{name} was supplied more than once"));
            }
            position += 1;
        }
        Ok(Self {
            values,
            overwrite,
            structure_only,
        })
    }

    fn required_path(&self, name: &str) -> Result<PathBuf> {
        self.values
            .get(name)
            .map(PathBuf::from)
            .ok_or_else(|| format!("missing required --{name}").into())
    }

    fn required_string(&self, name: &str) -> Result<String> {
        self.values
            .get(name)
            .map(|value| value.to_string_lossy().into_owned())
            .ok_or_else(|| format!("missing required --{name}").into())
    }

    fn reject_unknown(&self, allowed: &[&str]) -> Result<()> {
        let allowed: HashSet<&str> = allowed.iter().copied().collect();
        for name in self.values.keys() {
            if !allowed.contains(name.as_str()) {
                return fail(format!("unknown option --{name}"));
            }
        }
        Ok(())
    }

    fn reject_flags(&self, allow_overwrite: bool, allow_structure_only: bool) -> Result<()> {
        if self.overwrite && !allow_overwrite {
            return fail("--overwrite is not valid for this command");
        }
        if self.structure_only && !allow_structure_only {
            return fail("--structure-only is only valid for verify");
        }
        Ok(())
    }
}

fn interactive(prefill: Option<PathBuf>) -> Result<()> {
    let mut prefill = prefill;
    loop {
        println!();
        println!("Lien DRS/ISF tool");
        println!("1. Unpack DRS");
        println!("2. Extract translation JSON");
        println!("3. Inject translation JSON");
        println!("4. Pack DRS");
        println!("5. Verify archive");
        println!("0. Exit");
        let choice = prompt("Select operation", None)?;
        if choice == "0" || choice.is_empty() {
            return Ok(());
        }
        let result = match choice.as_str() {
            "1" => interactive_unpack(prefill.take()),
            "2" => interactive_extract(prefill.take()),
            "3" => interactive_inject(prefill.take()),
            "4" => interactive_pack(prefill.take()),
            "5" => interactive_verify(prefill.take()),
            _ => {
                println!("Unknown selection.");
                continue;
            }
        };
        if let Err(error) = result {
            eprintln!("[error] {error}");
        }
    }
}

fn interactive_unpack(prefill: Option<PathBuf>) -> Result<()> {
    let input = prompt_path("Input DRS", prefill.as_deref())?;
    let default = default_unpack_output(&input);
    let (output, overwrite) = prompt_output_path("Output directory", Some(&default))?;
    preview("unpack", &[(&"input", &input), (&"output", &output)])?;
    let files = unpack_archive(&input, &output, overwrite)?;
    println!("[unpack] extracted_files={files}");
    println!("[unpack] output={}", output.display());
    Ok(())
}

fn interactive_extract(prefill: Option<PathBuf>) -> Result<()> {
    let input = prompt_path("Input DRS", prefill.as_deref())?;
    let default = sibling_with_suffix(&input, "_json");
    let (output, overwrite) = prompt_output_path("Output translation directory", Some(&default))?;
    preview("extract", &[(&"input", &input), (&"output", &output)])?;
    let stats = extract_archive(&input, &output, overwrite)?;
    print_extract_stats(&stats, &output);
    Ok(())
}

fn interactive_inject(prefill: Option<PathBuf>) -> Result<()> {
    let source = prompt_path("Source DRS", prefill.as_deref())?;
    let translation = prompt_path("Translation directory", None)?;
    let route_text = prompt("Encoding route (cp932/gbk)", Some("cp932"))?;
    let route: EncodingRoute = route_text.parse()?;
    let default = sibling_with_suffix(&source, "_injected");
    let (output, overwrite) = prompt_output_path("Output DRS", Some(&default))?;
    println!("encoding={route}");
    preview(
        "inject",
        &[
            (&"source", &source),
            (&"translation", &translation),
            (&"output", &output),
        ],
    )?;
    let stats = inject_archive(&source, &translation, &output, route, overwrite)?;
    print_inject_stats(&stats, &output, route);
    Ok(())
}

fn interactive_pack(prefill: Option<PathBuf>) -> Result<()> {
    let input = prompt_path("Input member directory", prefill.as_deref())?;
    let default_manifest = input.join("lien-drs-manifest.json");
    let manifest = prompt_path("DRS manifest", Some(&default_manifest))?;
    let default_output = input.with_extension("packed.drs");
    let (output, overwrite) = prompt_output_path("Output DRS", Some(&default_output))?;
    preview(
        "pack",
        &[
            (&"input", &input),
            (&"manifest", &manifest),
            (&"output", &output),
        ],
    )?;
    let (files, bytes) = pack_archive(&input, &manifest, &output, overwrite)?;
    println!("[pack] packed_files={files}");
    println!("[pack] output_bytes={bytes}");
    println!("[pack] output={}", output.display());
    Ok(())
}

fn interactive_verify(prefill: Option<PathBuf>) -> Result<()> {
    let input = prompt_path("Input DRS", prefill.as_deref())?;
    run_verify(&input, false)
}

fn prompt(label: &str, default: Option<&str>) -> Result<String> {
    match default {
        Some(value) => print!("{label} [{value}]: "),
        None => print!("{label}: "),
    }
    io::stdout().flush()?;
    let mut input = String::new();
    if io::stdin().read_line(&mut input)? == 0 {
        return fail("input reached EOF");
    }
    let value = input.trim().to_owned();
    if value.is_empty() {
        Ok(default.unwrap_or_default().to_owned())
    } else {
        Ok(value)
    }
}

fn prompt_path(label: &str, default: Option<&Path>) -> Result<PathBuf> {
    let default_text = default.map(|path| path.to_string_lossy().into_owned());
    loop {
        let value = prompt(label, default_text.as_deref())?;
        let unquoted = strip_outer_quotes(value.trim());
        if !unquoted.is_empty() {
            return Ok(PathBuf::from(unquoted));
        }
        println!("A path is required.");
    }
}

fn strip_outer_quotes(value: &str) -> &str {
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn prompt_output_path(label: &str, default: Option<&Path>) -> Result<(PathBuf, bool)> {
    let mut suggestion = default.map(Path::to_path_buf);
    loop {
        let output = prompt_path(label, suggestion.as_deref())?;
        if !output.exists() {
            return Ok((output, false));
        }
        println!("Output already exists: {}", output.display());
        let answer = prompt("Modify path, overwrite, or cancel? (m/o/c)", Some("m"))?;
        if answer.eq_ignore_ascii_case("o") || answer.eq_ignore_ascii_case("overwrite") {
            return Ok((output, true));
        }
        if answer.eq_ignore_ascii_case("c") || answer.eq_ignore_ascii_case("cancel") {
            return fail("operation cancelled");
        }
        suggestion = Some(output);
    }
}

fn preview(operation: &str, paths: &[(&&str, &PathBuf)]) -> Result<()> {
    println!("operation={operation}");
    for (role, path) in paths {
        println!("{role}={}", path.display());
    }
    if confirm("Execute")? {
        Ok(())
    } else {
        fail("operation cancelled")
    }
}

fn confirm(label: &str) -> Result<bool> {
    let answer = prompt(&format!("{label}? (y/N)"), Some("N"))?;
    Ok(answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes"))
}

fn sibling_with_suffix(input: &Path, suffix: &str) -> PathBuf {
    let name = input
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_owned());
    input.with_file_name(format!("{name}{suffix}"))
}

fn print_extract_stats(stats: &lien_isf::workflow::ExtractStats, output: &Path) {
    println!("[extract] scanned_files={}", stats.scanned_files);
    println!("[extract] json_files={}", stats.json_files);
    println!("[extract] extracted_entries={}", stats.extracted_entries);
    println!("[extract] excluded_opcodes={}", stats.excluded_opcodes);
    println!("[extract] multipart_opcodes={}", stats.multipart_opcodes);
    println!("[extract] warnings={}", stats.warnings);
    println!("[extract] output={}", output.display());
}

fn print_inject_stats(
    stats: &lien_isf::workflow::InjectStats,
    output: &Path,
    route: EncodingRoute,
) {
    println!("[inject] encoding={route}");
    println!("[inject] json_files={}", stats.json_files);
    println!("[inject] json_entries={}", stats.json_entries);
    println!("[inject] patched_messages={}", stats.patched_messages);
    println!("[inject] patched_names={}", stats.patched_names);
    println!("[inject] unchanged={}", stats.unchanged);
    println!("[inject] rebuilt_scripts={}", stats.rebuilt_scripts);
    println!("[inject] output_bytes={}", stats.output_bytes);
    println!("[inject] output={}", output.display());
}

fn run_verify(input: &Path, structure_only: bool) -> Result<()> {
    let stats = verify_archive(input, !structure_only)?;
    println!(
        "[verify] mode={}",
        if structure_only { "structure" } else { "full" }
    );
    println!("[verify] archive_files={}", stats.archive_files);
    println!("[verify] parsed_scripts={}", stats.parsed_scripts);
    println!("[verify] opcode_count={}", stats.opcode_count);
    println!("[verify] text_entries={}", stats.text_entries);
    println!("[verify] excluded_opcodes={}", stats.excluded_opcodes);
    println!(
        "[verify] script_roundtrip_exact={}",
        stats.script_roundtrip_exact
    );
    println!(
        "[verify] archive_roundtrip_exact={}",
        stats.archive_roundtrip_exact
    );
    Ok(())
}

#[allow(dead_code)]
fn expected_translation_manifest(directory: &Path) -> PathBuf {
    directory.join(TRANSLATION_MANIFEST)
}
