use ai5win_mes::encoding::EncodingRoute;
use ai5win_mes::workflow::{extract_path, inject_path, verify_path, WorkflowStats};
use ai5win_mes::{fail, Result};
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const HELP: &str = r#"ai5win-mes-tool - AI5WIN MES structured text extractor/injector

USAGE:
  ai5win-mes-tool extract --input <FILE_OR_DIR> --output <DIR> [--encoding cp932] [--overwrite]
  ai5win-mes-tool inject --source <FILE_OR_DIR> --translation <DIR> --output <FILE_OR_DIR> --encoding <cp932|gbk> [--overwrite]
  ai5win-mes-tool verify --input <FILE_OR_DIR> [--encoding cp932|gbk]
  ai5win-mes-tool [PATH]

NOTES:
  JSON is always UTF-8. Edit only message and, where present, name.
  GBK injection re-encodes every extracted text slot, including unchanged text.
  GBK output requires a separately patched runtime; the stock executable is Shift-JIS-specific.
"#;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments
        .first()
        .is_some_and(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        print!("{HELP}");
        return Ok(());
    }
    if arguments.is_empty() {
        return interactive(None);
    }
    if arguments.len() == 1 && !matches!(arguments[0].as_str(), "extract" | "inject" | "verify") {
        return interactive(Some(PathBuf::from(&arguments[0])));
    }
    let command = arguments[0].as_str();
    let options = parse_options(&arguments[1..])?;
    match command {
        "extract" => {
            let input = required_path(&options, "input")?;
            let output = required_path(&options, "output")?;
            let encoding = option_encoding(&options, "encoding", EncodingRoute::Cp932)?;
            print_stats(
                "extracted",
                extract_path(&input, &output, encoding, options.flag("overwrite"))?,
            );
        }
        "inject" => {
            let source = required_path(&options, "source")?;
            let translation = required_path(&options, "translation")?;
            let output = required_path(&options, "output")?;
            let encoding = required_encoding(&options, "encoding")?;
            print_stats(
                "injected",
                inject_path(
                    &source,
                    &translation,
                    &output,
                    encoding,
                    options.flag("overwrite"),
                )?,
            );
        }
        "verify" => {
            let input = required_path(&options, "input")?;
            let encoding = option_encoding(&options, "encoding", EncodingRoute::Cp932)?;
            print_stats("verified", verify_path(&input, encoding)?);
        }
        _ => return fail(format!("unknown command {command:?}\n\n{HELP}")),
    }
    Ok(())
}

#[derive(Default)]
struct Options {
    values: HashMap<String, String>,
    flags: HashMap<String, bool>,
}

impl Options {
    fn flag(&self, name: &str) -> bool {
        self.flags.get(name).copied().unwrap_or(false)
    }
}

fn parse_options(arguments: &[String]) -> Result<Options> {
    let mut options = Options::default();
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        if argument == "--overwrite" {
            options.flags.insert("overwrite".to_owned(), true);
            index += 1;
            continue;
        }
        let Some(name) = argument.strip_prefix("--") else {
            return fail(format!("unexpected positional argument {argument:?}"));
        };
        if !matches!(
            name,
            "input" | "output" | "source" | "translation" | "encoding"
        ) {
            return fail(format!("unknown option --{name}"));
        }
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| format!("--{name} requires a value"))?;
        if value.starts_with("--") {
            return fail(format!("--{name} requires a value"));
        }
        if options
            .values
            .insert(name.to_owned(), value.clone())
            .is_some()
        {
            return fail(format!("--{name} was supplied more than once"));
        }
        index += 2;
    }
    Ok(options)
}

fn required_path(options: &Options, name: &str) -> Result<PathBuf> {
    options
        .values
        .get(name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing required option --{name}").into())
}

fn required_encoding(options: &Options, name: &str) -> Result<EncodingRoute> {
    let value = options
        .values
        .get(name)
        .ok_or_else(|| format!("missing required option --{name}"))?;
    value.parse().map_err(Into::into)
}

fn option_encoding(options: &Options, name: &str, default: EncodingRoute) -> Result<EncodingRoute> {
    match options.values.get(name) {
        Some(value) => value.parse().map_err(Into::into),
        None => Ok(default),
    }
}

fn print_stats(action: &str, stats: WorkflowStats) {
    println!(
        "{action}: {} scripts, {} text entries ({} files scanned)",
        stats.script_files, stats.text_entries, stats.scanned_files
    );
}

fn interactive(mut prefill: Option<PathBuf>) -> Result<()> {
    loop {
        println!("\nAI5WIN MES tool\n  1) Extract\n  2) Inject\n  3) Verify\n  4) Exit");
        let choice = prompt("Choice", None)?;
        match choice.trim().to_ascii_lowercase().as_str() {
            "1" | "extract" => interactive_extract(prefill.take())?,
            "2" | "inject" => interactive_inject(prefill.take())?,
            "3" | "verify" => interactive_verify(prefill.take())?,
            "4" | "exit" | "quit" | "q" => return Ok(()),
            "" => continue,
            _ => eprintln!("Unknown choice."),
        }
    }
}

fn interactive_extract(prefill: Option<PathBuf>) -> Result<()> {
    let default_input = prefill.as_deref().and_then(Path::to_str);
    let input = PathBuf::from(prompt("Input file/directory", default_input)?);
    let output = PathBuf::from(prompt(
        "Translation output directory",
        Some("translation_json"),
    )?);
    let encoding: EncodingRoute = prompt("Source encoding", Some("cp932"))?.parse()?;
    let overwrite = output.exists() && confirm("Output exists. Replace it")?;
    if output.exists() && !overwrite {
        println!("Cancelled.");
        return Ok(());
    }
    println!(
        "Extract {} -> {} ({encoding})",
        input.display(),
        output.display()
    );
    if !confirm("Proceed")? {
        println!("Cancelled.");
        return Ok(());
    }
    match extract_path(&input, &output, encoding, overwrite) {
        Ok(stats) => print_stats("extracted", stats),
        Err(error) => eprintln!("error: {error}"),
    }
    Ok(())
}

fn interactive_inject(prefill: Option<PathBuf>) -> Result<()> {
    let default_source = prefill.as_deref().and_then(Path::to_str);
    let source = PathBuf::from(prompt("Original script file/directory", default_source)?);
    let translation = PathBuf::from(prompt("Translation directory", Some("translation_json"))?);
    let output = PathBuf::from(prompt("Rebuilt output file/directory", Some("rebuilt"))?);
    let encoding: EncodingRoute = prompt("Output encoding (cp932/gbk)", Some("cp932"))?.parse()?;
    let overwrite = output.exists() && confirm("Output exists. Replace it")?;
    if output.exists() && !overwrite {
        println!("Cancelled.");
        return Ok(());
    }
    println!(
        "Inject {} + {} -> {} ({encoding})",
        source.display(),
        translation.display(),
        output.display()
    );
    if !confirm("Proceed")? {
        println!("Cancelled.");
        return Ok(());
    }
    match inject_path(&source, &translation, &output, encoding, overwrite) {
        Ok(stats) => print_stats("injected", stats),
        Err(error) => eprintln!("error: {error}"),
    }
    Ok(())
}

fn interactive_verify(prefill: Option<PathBuf>) -> Result<()> {
    let default_input = prefill.as_deref().and_then(Path::to_str);
    let input = PathBuf::from(prompt("Input file/directory", default_input)?);
    let encoding: EncodingRoute = prompt("Text encoding", Some("cp932"))?.parse()?;
    match verify_path(&input, encoding) {
        Ok(stats) => print_stats("verified", stats),
        Err(error) => eprintln!("error: {error}"),
    }
    Ok(())
}

fn prompt(label: &str, default: Option<&str>) -> Result<String> {
    match default {
        Some(value) => print!("{label} [{value}]: "),
        None => print!("{label}: "),
    }
    io::stdout().flush()?;
    let mut value = String::new();
    if io::stdin().read_line(&mut value)? == 0 {
        return fail("end of input");
    }
    let value = value.trim();
    if value.is_empty() {
        default
            .map(str::to_owned)
            .ok_or_else(|| format!("{label} is required").into())
    } else {
        Ok(value.to_owned())
    }
}

fn confirm(label: &str) -> Result<bool> {
    let answer = prompt(&format!("{label}? (y/N)"), Some("N"))?;
    Ok(matches!(answer.to_ascii_lowercase().as_str(), "y" | "yes"))
}
