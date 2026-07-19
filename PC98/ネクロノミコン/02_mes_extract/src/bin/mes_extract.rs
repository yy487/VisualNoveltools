use necronomicon_mes::{ScriptJson, parse_script};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

struct PreparedFile {
    destination: PathBuf,
    json: Vec<u8>,
    entries: usize,
    warnings: usize,
}

fn collect_inputs(path: &Path) -> Result<Vec<PathBuf>, String> {
    if path.is_file() {
        if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("mes"))
        {
            return Ok(vec![path.to_owned()]);
        }
        return Err(format!("input is not a MES file: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("input does not exist: {}", path.display()));
    }

    let mut files = Vec::new();
    let disk_roots = fs::read_dir(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|child| {
            child.is_dir()
                && child
                    .file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|name| {
                        name.len() == 6
                            && name.starts_with("DISK_")
                            && matches!(name.as_bytes()[5], b'A'..=b'K')
                    })
        })
        .collect::<Vec<_>>();
    let mut pending = if disk_roots.is_empty() {
        vec![path.to_owned()]
    } else {
        disk_roots
    };
    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|error| format!("cannot read {}: {error}", directory.display()))?;
        for entry in entries {
            let entry = entry
                .map_err(|error| format!("cannot enumerate {}: {error}", directory.display()))?;
            let child = entry.path();
            if child.is_dir() {
                pending.push(child);
            } else if child
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("mes"))
            {
                files.push(child);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn add_json_extension(path: &Path) -> PathBuf {
    let mut name: OsString = path
        .file_name()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| OsString::from("script.MES"));
    name.push(".json");
    path.with_file_name(name)
}

fn default_output(input: &Path) -> PathBuf {
    if input.is_file() {
        return add_json_extension(input);
    }
    let base = input
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("MES");
    input.with_file_name(format!("{base}_json"))
}

fn prepare(
    input_root: &Path,
    output: &Path,
    input: &Path,
    multiple: bool,
) -> Result<PreparedFile, String> {
    let relative = if multiple {
        input.strip_prefix(input_root).map_err(|_| {
            format!(
                "{} is outside input root {}",
                input.display(),
                input_root.display()
            )
        })?
    } else {
        input.file_name().map(Path::new).unwrap_or(input)
    };
    let destination = if multiple || output.is_dir() {
        add_json_extension(&output.join(relative))
    } else {
        output.to_owned()
    };
    let source =
        fs::read(input).map_err(|error| format!("cannot read {}: {error}", input.display()))?;
    let file_label = slash_path(relative);
    let parsed: ScriptJson = parse_script(&source, file_label)
        .map_err(|error| format!("cannot parse {}: {error}", input.display()))?;
    let entries = parsed.entries.len();
    let warnings = parsed.warnings.len();
    let mut json = serde_json::to_vec_pretty(&parsed)
        .map_err(|error| format!("cannot serialize {}: {error}", input.display()))?;
    json.push(b'\n');
    Ok(PreparedFile {
        destination,
        json,
        entries,
        warnings,
    })
}

fn temp_path(final_path: &Path) -> PathBuf {
    let name = final_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("mes_json");
    final_path.with_file_name(format!(".{name}.partial-{}", std::process::id()))
}

fn write_single(prepared: PreparedFile) -> Result<(), String> {
    if prepared.destination.exists() {
        return Err(format!(
            "refusing to overwrite {}",
            prepared.destination.display()
        ));
    }
    if let Some(parent) = prepared.destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    let temporary = temp_path(&prepared.destination);
    if temporary.exists() {
        return Err(format!(
            "temporary path already exists: {}",
            temporary.display()
        ));
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| format!("cannot create {}: {error}", temporary.display()))?;
    if let Err(error) = file.write_all(&prepared.json) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("cannot write {}: {error}", temporary.display()));
    }
    drop(file);
    fs::rename(&temporary, &prepared.destination).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!(
            "cannot finalize {}: {error}",
            prepared.destination.display()
        )
    })
}

fn write_directory(prepared: Vec<PreparedFile>, output: &Path) -> Result<(), String> {
    if output.exists() {
        return Err(format!("refusing to overwrite {}", output.display()));
    }
    let temporary = temp_path(output);
    if temporary.exists() {
        return Err(format!(
            "temporary path already exists: {}",
            temporary.display()
        ));
    }
    fs::create_dir_all(&temporary)
        .map_err(|error| format!("cannot create {}: {error}", temporary.display()))?;

    let result = (|| {
        for item in prepared {
            let relative = item.destination.strip_prefix(output).map_err(|_| {
                format!(
                    "output {} is outside {}",
                    item.destination.display(),
                    output.display()
                )
            })?;
            let destination = temporary.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
            }
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&destination)
                .map_err(|error| format!("cannot create {}: {error}", destination.display()))?;
            file.write_all(&item.json)
                .map_err(|error| format!("cannot write {}: {error}", destination.display()))?;
        }
        fs::rename(&temporary, output)
            .map_err(|error| format!("cannot finalize {}: {error}", output.display()))
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&temporary);
    }
    result
}

fn print_help() {
    println!("mes_extract - extract NECRONOMICON MES text to UTF-8 JSON");
    println!("Usage: mes_extract <file-or-directory> [--output <file-or-directory>]");
    println!("A directory is searched recursively for .MES files.");
    println!(
        "A dump root containing DISK_A..DISK_K scans those disk trees and ignores work copies."
    );
    println!("Each physical text segment becomes one scr_msg/message entry.");
    println!("A leading 「name」 is split into immutable _scr_name and writable name.");
    println!(
        "The full sub_D77C token IR preserves every source byte; outputs are never overwritten."
    );
}

fn run() -> Result<(), String> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return Ok(());
    }
    let input_root = PathBuf::from(&args[0]);
    let mut output = None;
    let mut index = 1usize;
    while index < args.len() {
        if args[index] == "--output" {
            let value = args
                .get(index + 1)
                .ok_or_else(|| "--output requires a path".to_owned())?;
            output = Some(PathBuf::from(value));
            index += 2;
        } else {
            return Err(format!(
                "unknown argument: {}",
                args[index].to_string_lossy()
            ));
        }
    }

    let inputs = collect_inputs(&input_root)?;
    if inputs.is_empty() {
        return Err(format!("no MES files found under {}", input_root.display()));
    }
    let multiple = input_root.is_dir() || inputs.len() > 1;
    let output = output.unwrap_or_else(|| default_output(&input_root));
    if multiple && output.exists() {
        return Err(format!("refusing to overwrite {}", output.display()));
    }

    let mut prepared = Vec::with_capacity(inputs.len());
    let mut total_entries = 0usize;
    let mut total_warnings = 0usize;
    for input in &inputs {
        let item = prepare(&input_root, &output, input, multiple)?;
        total_entries += item.entries;
        total_warnings += item.warnings;
        prepared.push(item);
    }

    if multiple {
        write_directory(prepared, &output)?;
    } else {
        write_single(prepared.pop().expect("one prepared file"))?;
    }
    println!("[extract] scanned_files={}", inputs.len());
    println!("[extract] json_files={}", inputs.len());
    println!("[extract] extracted_entries={total_entries}");
    println!("[extract] warnings={total_warnings}");
    println!("[extract] output={}", output.display());
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("[extract] error: {error}");
        std::process::exit(1);
    }
}
