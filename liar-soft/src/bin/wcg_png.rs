use sbridge_tools::png::encode_bgra_png;
use sbridge_tools::wcg::{decode_wcg, looks_like_wcg};
use std::env;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

const HELP: &str = r#"wcg-png - convert Liar-soft WCG images to PNG

USAGE:
  wcg-png convert <INPUT> --output <OUTPUT> --yes [--recursive] [--overwrite]
  wcg-png <INPUT>                 Interactive prefill (drag-and-drop friendly)
  wcg-png                         Interactive session

RULES:
  INPUT may be one WCG file or a directory. Files are detected by content.
  Directory input writes a matching PNG tree below OUTPUT.
  Non-interactive conversion requires both --output and --yes.
  Existing files are never replaced unless --overwrite is supplied.
"#;

#[derive(Debug)]
struct ConvertOptions {
    input: PathBuf,
    output: PathBuf,
    recursive: bool,
    overwrite: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("[error] {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    if args.is_empty() {
        return interactive_session(None);
    }
    if is_flag(&args[0], "-h") || is_flag(&args[0], "--help") {
        print!("{HELP}");
        return Ok(());
    }
    if is_flag(&args[0], "convert") {
        let options = parse_convert(&args[1..])?;
        return convert(&options);
    }
    if args.len() == 1 {
        return interactive_session(Some(PathBuf::from(&args[0])));
    }
    Err(format!("unrecognized arguments\n\n{HELP}").into())
}

fn parse_convert(args: &[OsString]) -> Result<ConvertOptions, Box<dyn Error>> {
    let input = args
        .first()
        .filter(|arg| !arg.to_string_lossy().starts_with('-'))
        .map(PathBuf::from)
        .ok_or("convert requires an INPUT path")?;
    let mut output = None;
    let mut recursive = false;
    let mut overwrite = false;
    let mut yes = false;
    let mut index = 1usize;
    while index < args.len() {
        if is_flag(&args[index], "--output") || is_flag(&args[index], "-o") {
            index += 1;
            output = Some(PathBuf::from(
                args.get(index).ok_or("--output requires a path")?,
            ));
        } else if is_flag(&args[index], "--recursive") {
            recursive = true;
        } else if is_flag(&args[index], "--overwrite") {
            overwrite = true;
        } else if is_flag(&args[index], "--yes") {
            yes = true;
        } else {
            return Err(format!("unknown option: {}", args[index].to_string_lossy()).into());
        }
        index += 1;
    }
    if !yes {
        return Err("non-interactive conversion requires --yes; omit 'convert' for prompts".into());
    }
    Ok(ConvertOptions {
        input,
        output: output.ok_or("non-interactive conversion requires --output")?,
        recursive,
        overwrite,
    })
}

fn interactive_session(mut prefill: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    loop {
        println!("\nWCG → PNG");
        println!("1) Convert image(s)");
        println!("0) Exit");
        let choice = if prefill.is_some() {
            "1".to_owned()
        } else {
            prompt("Select")?
        };
        match choice.trim() {
            "0" | "q" | "Q" => return Ok(()),
            "1" => {
                let input = match prefill.take() {
                    Some(path) => {
                        println!("Input: {}", path.display());
                        path
                    }
                    None => PathBuf::from(prompt("Input WCG file or directory")?.trim()),
                };
                let suggested = suggested_output(&input);
                let entered = prompt(&format!("Output [{}]", suggested.display()))?;
                let output = if entered.trim().is_empty() {
                    suggested
                } else {
                    PathBuf::from(entered.trim())
                };
                let recursive =
                    input.is_dir() && yes(&prompt("Recurse into subdirectories? [y/N]")?);
                let overwrite = yes(&prompt("Replace existing PNG files? [y/N]")?);
                let files = discover(&input, recursive)?;
                println!("Ready: {} WCG file(s) -> {}", files.len(), output.display());
                if yes(&prompt("Write these files? [y/N]")?) {
                    let options = ConvertOptions {
                        input,
                        output,
                        recursive,
                        overwrite,
                    };
                    if let Err(error) = convert(&options) {
                        eprintln!("[error] {error}");
                    }
                } else {
                    println!("Cancelled; nothing was written.");
                }
            }
            _ => eprintln!("Unknown selection."),
        }
    }
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

fn suggested_output(input: &Path) -> PathBuf {
    if input.is_dir() {
        let mut name = input
            .file_name()
            .unwrap_or_else(|| OsStr::new("wcg"))
            .to_os_string();
        name.push("_png");
        input.with_file_name(name)
    } else {
        input.with_extension("png")
    }
}

fn convert(options: &ConvertOptions) -> Result<(), Box<dyn Error>> {
    let files = discover(&options.input, options.recursive)?;
    if files.is_empty() {
        return Err(format!(
            "no supported WCG streams found at {}",
            options.input.display()
        )
        .into());
    }

    let input_is_directory = options.input.is_dir();
    let mut written = 0usize;
    let mut skipped = 0usize;
    for source in files {
        let target = output_path(options, &source, input_is_directory)?;
        if target.exists() && !options.overwrite {
            eprintln!("[skip] exists: {}", target.display());
            skipped += 1;
            continue;
        }
        let data = fs::read(&source)?;
        let image = decode_wcg(&data).map_err(|error| format!("{}: {error}", source.display()))?;
        let png = encode_bgra_png(image.width, image.height, &image.bgra)
            .map_err(|error| format!("{}: {error}", source.display()))?;
        write_output(&target, &png, options.overwrite)?;
        println!(
            "[ok] {} -> {} ({}x{})",
            source.display(),
            target.display(),
            image.width,
            image.height
        );
        written += 1;
    }
    println!("Done: {written} written, {skipped} skipped.");
    Ok(())
}

fn discover(input: &Path, recursive: bool) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    if input.is_file() {
        let mut header = [0u8; 16];
        let mut file = fs::File::open(input)?;
        let read = file.read(&mut header)?;
        if !looks_like_wcg(&header[..read]) {
            return Err(format!("{} is not a supported WCG stream", input.display()).into());
        }
        return Ok(vec![input.to_owned()]);
    }
    if !input.is_dir() {
        return Err(format!("input does not exist: {}", input.display()).into());
    }
    let mut files = Vec::new();
    discover_directory(input, recursive, &mut files)?;
    files.sort();
    Ok(files)
}

fn discover_directory(
    directory: &Path,
    recursive: bool,
    files: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let mut entries: Vec<_> = fs::read_dir(directory)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            if recursive {
                discover_directory(&path, true, files)?;
            }
        } else if kind.is_file() {
            let mut header = [0u8; 16];
            let mut file = fs::File::open(&path)?;
            let read = file.read(&mut header)?;
            if looks_like_wcg(&header[..read]) {
                files.push(path);
            }
        }
    }
    Ok(())
}

fn output_path(
    options: &ConvertOptions,
    source: &Path,
    input_is_directory: bool,
) -> Result<PathBuf, Box<dyn Error>> {
    if input_is_directory {
        let relative = source.strip_prefix(&options.input)?;
        Ok(options.output.join(relative).with_extension("png"))
    } else if options.output.is_dir() {
        let name = source.file_name().ok_or("input file has no name")?;
        Ok(options.output.join(name).with_extension("png"))
    } else {
        Ok(options.output.clone())
    }
}

fn write_output(path: &Path, bytes: &[u8], overwrite: bool) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if path.exists() && !overwrite {
        return Err(format!("output exists: {}", path.display()).into());
    }

    let mut attempt = 0u32;
    let temp = loop {
        let mut name = path
            .file_name()
            .ok_or("output path has no file name")?
            .to_os_string();
        name.push(format!(".tmp-{}-{attempt}", std::process::id()));
        let candidate = path.with_file_name(name);
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(mut file) => {
                file.write_all(bytes)?;
                file.sync_all()?;
                break candidate;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => attempt += 1,
            Err(error) => return Err(error.into()),
        }
    };

    if path.exists()
        && let Err(error) = fs::remove_file(path)
    {
        let _ = fs::remove_file(&temp);
        return Err(error.into());
    }
    if let Err(error) = fs::rename(&temp, path) {
        let _ = fs::remove_file(&temp);
        return Err(error.into());
    }
    Ok(())
}

fn is_flag(value: &OsStr, expected: &str) -> bool {
    value.to_string_lossy().eq_ignore_ascii_case(expected)
}
