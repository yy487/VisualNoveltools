use necronomicon_mes::{ScriptJson, rebuild_script};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

struct PreparedFile {
    destination: PathBuf,
    bytes: Vec<u8>,
    changed: bool,
    entries: usize,
    patched: usize,
}

fn collect_inputs(path: &Path) -> Result<Vec<PathBuf>, String> {
    if path.is_file() {
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            return Ok(vec![path.to_owned()]);
        }
        return Err(format!("input is not a JSON file: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("input does not exist: {}", path.display()));
    }
    let mut files = Vec::new();
    let mut pending = vec![path.to_owned()];
    while let Some(directory) = pending.pop() {
        for item in fs::read_dir(&directory)
            .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
        {
            let item =
                item.map_err(|error| format!("cannot enumerate {}: {error}", directory.display()))?;
            let child = item.path();
            if child.is_dir() {
                pending.push(child);
            } else if child.extension().and_then(|value| value.to_str()) == Some("json") {
                files.push(child);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn relative_source_path(file: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(file.replace('/', "\\"));
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!("unsafe _file path: {file:?}"));
    }
    Ok(path)
}

fn temp_path(final_path: &Path) -> PathBuf {
    let name = final_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("mes");
    final_path.with_file_name(format!(".{name}.partial-{}", std::process::id()))
}

fn prepare(
    source_root: &Path,
    output: &Path,
    input: &Path,
    single_output: bool,
) -> Result<PreparedFile, String> {
    let json_bytes =
        fs::read(input).map_err(|error| format!("cannot read {}: {error}", input.display()))?;
    let script: ScriptJson = serde_json::from_slice(&json_bytes)
        .map_err(|error| format!("cannot parse {}: {error}", input.display()))?;
    let entries = script.entries.len();
    let patched = script
        .entries
        .iter()
        .filter(|entry| entry.name != entry.scr_name || entry.message != entry.scr_msg)
        .count();
    let relative = relative_source_path(&script.file)?;
    let source_path = source_root.join(&relative);
    let source = fs::read(&source_path)
        .map_err(|error| format!("cannot read source {}: {error}", source_path.display()))?;
    let rebuilt = rebuild_script(&script, &source)
        .map_err(|error| format!("cannot rebuild {}: {error}", input.display()))?;
    let destination = if single_output {
        output.to_owned()
    } else {
        output.join(&relative)
    };
    Ok(PreparedFile {
        destination,
        changed: rebuilt != source,
        bytes: rebuilt,
        entries,
        patched,
    })
}

fn write_one(item: PreparedFile) -> Result<(), String> {
    if item.destination.exists() {
        return Err(format!(
            "refusing to overwrite {}",
            item.destination.display()
        ));
    }
    if let Some(parent) = item.destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    let temporary = temp_path(&item.destination);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| format!("cannot create {}: {error}", temporary.display()))?;
    if let Err(error) = file.write_all(&item.bytes) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("cannot write {}: {error}", temporary.display()));
    }
    drop(file);
    fs::rename(&temporary, &item.destination).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("cannot finalize {}: {error}", item.destination.display())
    })
}

fn write_many(items: Vec<PreparedFile>, output: &Path) -> Result<(), String> {
    if output.exists() {
        return Err(format!("refusing to overwrite {}", output.display()));
    }
    let temporary = temp_path(output);
    fs::create_dir_all(&temporary)
        .map_err(|error| format!("cannot create {}: {error}", temporary.display()))?;
    let result = (|| {
        for item in items {
            let relative = item
                .destination
                .strip_prefix(output)
                .map_err(|_| "output path escaped root".to_owned())?;
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
            file.write_all(&item.bytes)
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
    println!("mes_inject - rebuild NECRONOMICON MES from UTF-8 JSON");
    println!(
        "Usage: mes_inject <json-file-or-directory> --source-root <dump-root> [--output <file-or-directory>]"
    );
    println!("Text is encoded strictly as CP932 plus the game's custom hiragana bytes.");
    println!(
        "Immutable sub_D77C tokens are source-validated; only name/message may grow or shrink."
    );
}

fn run() -> Result<(), String> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return Ok(());
    }
    let input = PathBuf::from(&args[0]);
    let mut source_root = None;
    let mut output = None;
    let mut index = 1usize;
    while index < args.len() {
        match args[index].to_string_lossy().as_ref() {
            "--source-root" => {
                source_root = Some(PathBuf::from(
                    args.get(index + 1).ok_or("--source-root requires a path")?,
                ));
                index += 2;
            }
            "--output" => {
                output = Some(PathBuf::from(
                    args.get(index + 1).ok_or("--output requires a path")?,
                ));
                index += 2;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    let source_root = source_root.ok_or("--source-root is required")?;
    let inputs = collect_inputs(&input)?;
    if inputs.is_empty() {
        return Err(format!("no JSON files found under {}", input.display()));
    }
    let multiple = input.is_dir() || inputs.len() > 1;
    let output = output.unwrap_or_else(|| {
        if multiple {
            input.with_file_name(format!(
                "{}_mes",
                input
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("json")
            ))
        } else {
            input.with_extension("")
        }
    });
    let mut prepared = Vec::with_capacity(inputs.len());
    let mut changed = 0usize;
    let mut entries = 0usize;
    let mut patched = 0usize;
    for file in &inputs {
        let item = prepare(&source_root, &output, file, !multiple)?;
        changed += usize::from(item.changed);
        entries += item.entries;
        patched += item.patched;
        prepared.push(item);
    }
    if multiple {
        write_many(prepared, &output)?;
    } else {
        write_one(prepared.pop().expect("one input"))?;
    }
    println!("[inject] json_files={}", inputs.len());
    println!("[inject] json_entries={entries}");
    println!("[inject] patched={patched}");
    println!("[inject] unchanged={}", entries - patched);
    println!("[inject] changed_files={changed}");
    println!("[inject] failed=0");
    println!("[inject] output={}", output.display());
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("[inject] error: {error}");
        std::process::exit(1);
    }
}
