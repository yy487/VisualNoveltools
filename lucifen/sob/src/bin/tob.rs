use oretubar_tob_sob_tool::common::{
    collect_files, confirm, ensure_output, parse_args, prompt, EncodingChoice, Result,
};
use oretubar_tob_sob_tool::tob;
use std::env;
use std::path::{Path, PathBuf};

fn json_path(root: &Path, rel: &Path) -> PathBuf {
    root.join(rel).with_extension("json")
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        return interactive(None);
    }
    if args.len() == 2 && !matches!(args[1].as_str(), "extract" | "inject" | "--help" | "-h") {
        return interactive(Some(PathBuf::from(&args[1])));
    }
    if args.iter().any(|x| x == "--help" || x == "-h") {
        println!("oretubar-tob\n  extract --input PATH --output PATH --encoding sjis|gbk [--overwrite]\n  inject  --input PATH --translation PATH --output PATH --encoding sjis|gbk [--overwrite]");
        return Ok(());
    }
    let (mode, input, output, encoding, overwrite) = parse_args(&args)?;
    let files = collect_files(&input, "tob")?;
    if mode == "extract" {
        if input.is_dir() {
            ensure_output(&output, overwrite)?;
        }
        let mut total = 0;
        let count = files.len();
        for (source, rel) in files {
            let destination = if input.is_file() {
                output.clone()
            } else {
                output.join(&rel).with_extension("json")
            };
            total += tob::extract_file(
                &source,
                &destination,
                rel.to_string_lossy().as_ref(),
                encoding,
                overwrite,
            )?;
        }
        println!("extracted {total} TOB entries from {count} files");
    } else if mode == "inject" {
        if input.is_dir() {
            ensure_output(&output, overwrite)?;
        }
        let translation = args
            .iter()
            .position(|x| x == "--translation")
            .and_then(|i| args.get(i + 1))
            .map(PathBuf::from)
            .ok_or("inject requires --translation")?;
        let mut changed = 0;
        for (source, rel) in files {
            let trans = if translation.is_file() {
                translation.clone()
            } else {
                json_path(&translation, &rel)
            };
            if !trans.exists() {
                return Err(format!("missing translation {}", trans.display()));
            }
            let destination = if input.is_file() {
                output.clone()
            } else {
                output.join(&rel)
            };
            changed += tob::inject_file(&source, &trans, &destination, encoding, overwrite)?;
        }
        println!("updated {changed} TOB records");
    } else {
        return Err(format!("unknown mode '{mode}'"));
    }
    Ok(())
}

fn interactive(mut prefill: Option<PathBuf>) -> Result<()> {
    loop {
        println!("\nORETUBAR TOB tool\n  1) Extract\n  2) Inject\n  3) Exit");
        let choice = prompt("Choice", None)?;
        match choice.trim().to_ascii_lowercase().as_str() {
            "1" | "extract" => match interactive_extract(prefill.take()) {
                Ok(()) => {}
                Err(error) if error == "end of input" => return Err(error),
                Err(error) => eprintln!("error: {error}"),
            },
            "2" | "inject" => match interactive_inject(prefill.take()) {
                Ok(()) => {}
                Err(error) if error == "end of input" => return Err(error),
                Err(error) => eprintln!("error: {error}"),
            },
            "3" | "exit" | "quit" | "q" => return Ok(()),
            "" => {}
            _ => eprintln!("Unknown choice."),
        }
    }
}

fn interactive_extract(prefill: Option<PathBuf>) -> Result<()> {
    let default_input = prefill.as_deref().and_then(Path::to_str);
    let input = PathBuf::from(prompt("Input TOB file/directory", default_input)?);
    let default_output = if input.is_file() {
        "translation.json"
    } else {
        "translation_json"
    };
    let output = PathBuf::from(prompt("JSON output file/directory", Some(default_output))?);
    let encoding = EncodingChoice::parse(&prompt("Source encoding (sjis/gbk)", None)?)?;
    let overwrite = if output.exists() {
        confirm("Output exists; overwrite it")?
    } else {
        false
    };
    if output.exists() && !overwrite {
        println!("Cancelled.");
        return Ok(());
    }
    println!(
        "Extract {} -> {} ({})",
        input.display(),
        output.display(),
        encoding.label()
    );
    if !confirm("Proceed")? {
        println!("Cancelled.");
        return Ok(());
    }
    let files = collect_files(&input, "tob")?;
    if input.is_dir() {
        ensure_output(&output, overwrite)?;
    }
    let count = files.len();
    let mut total = 0;
    for (source, rel) in files {
        let destination = if input.is_file() {
            output.clone()
        } else {
            output.join(&rel).with_extension("json")
        };
        total += oretubar_tob_sob_tool::tob::extract_file(
            &source,
            &destination,
            rel.to_string_lossy().as_ref(),
            encoding,
            overwrite,
        )?;
    }
    println!("Extracted {total} entries from {count} TOB files.");
    Ok(())
}

fn interactive_inject(prefill: Option<PathBuf>) -> Result<()> {
    let default_input = prefill.as_deref().and_then(Path::to_str);
    let input = PathBuf::from(prompt("Original TOB file/directory", default_input)?);
    let translation = PathBuf::from(prompt(
        "Translation JSON file/directory",
        Some("translation_json"),
    )?);
    let default_output = if input.is_file() {
        "rebuilt.tob"
    } else {
        "rebuilt"
    };
    let output = PathBuf::from(prompt(
        "Rebuilt output file/directory",
        Some(default_output),
    )?);
    let encoding = EncodingChoice::parse(&prompt("Source encoding (sjis/gbk)", None)?)?;
    let overwrite = if output.exists() {
        confirm("Output exists; overwrite it")?
    } else {
        false
    };
    if output.exists() && !overwrite {
        println!("Cancelled.");
        return Ok(());
    }
    println!(
        "Inject {} + {} -> {} ({})",
        input.display(),
        translation.display(),
        output.display(),
        encoding.label()
    );
    if !confirm("Proceed")? {
        println!("Cancelled.");
        return Ok(());
    }
    let files = collect_files(&input, "tob")?;
    if input.is_dir() {
        ensure_output(&output, overwrite)?;
    }
    let mut changed = 0;
    for (source, rel) in files {
        let trans = if translation.is_file() {
            translation.clone()
        } else {
            json_path(&translation, &rel)
        };
        if !trans.exists() {
            return Err(format!("missing translation {}", trans.display()));
        }
        let destination = if input.is_file() {
            output.clone()
        } else {
            output.join(&rel)
        };
        changed += oretubar_tob_sob_tool::tob::inject_file(
            &source,
            &trans,
            &destination,
            encoding,
            overwrite,
        )?;
    }
    println!("Updated {changed} TOB records.");
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
