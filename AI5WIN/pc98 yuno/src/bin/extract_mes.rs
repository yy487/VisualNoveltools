use clap::Parser;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use yuno_hdi_tools::mes::MesError;
use yuno_hdi_tools::mes_text::{document_to_json, extract_document};

#[derive(Debug, Parser)]
#[command(about = "Extract YU-NO PC-98 MES text to per-script UTF-8 JSON files")]
struct Args {
    /// Compressed MES file or resource directory. Inputs are read only.
    input: PathBuf,

    /// New JSON file or directory. Defaults beside INPUT.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Debug)]
struct PreparedJson {
    relative: PathBuf,
    data: Vec<u8>,
    entries: usize,
    dynamic_names: usize,
    multipart: usize,
}

fn invalid(message: impl Into<String>) -> MesError {
    MesError::Invalid(message.into())
}

fn io_error(action: &str, path: &Path, source: std::io::Error) -> MesError {
    invalid(format!("{action} {}: {source}", path.display()))
}

fn is_mes(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("mes"))
}

fn collect_mes_files(
    root: &Path,
    directory: &Path,
    output: &mut Vec<PathBuf>,
) -> Result<(), MesError> {
    let mut entries: Vec<_> = fs::read_dir(directory)
        .map_err(|source| io_error("cannot read directory", directory, source))?
        .collect::<std::result::Result<_, _>>()
        .map_err(|source| io_error("cannot enumerate directory", directory, source))?;
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_lowercase());
    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| io_error("cannot inspect", &path, source))?;
        if file_type.is_symlink() {
            return Err(invalid(format!(
                "symbolic links are not supported: {}",
                path.display()
            )));
        }
        if file_type.is_dir() {
            collect_mes_files(root, &path, output)?;
        } else if file_type.is_file() && is_mes(&path) {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| invalid("MES path escaped the input root"))?;
            output.push(relative.to_path_buf());
        }
    }
    Ok(())
}

fn json_relative(relative_mes: &Path) -> PathBuf {
    let mut value: OsString = relative_mes.as_os_str().to_os_string();
    value.push(".json");
    PathBuf::from(value)
}

fn portable_path(path: &Path) -> Result<String, MesError> {
    let components: Result<Vec<_>, _> = path
        .components()
        .map(|component| {
            component
                .as_os_str()
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| invalid(format!("path is not Unicode: {}", path.display())))
        })
        .collect();
    Ok(components?.join("/"))
}

fn default_file_output(input: &Path) -> PathBuf {
    let mut value = input.as_os_str().to_os_string();
    value.push(".json");
    PathBuf::from(value)
}

fn default_directory_output(input: &Path) -> Result<PathBuf, MesError> {
    let name = input
        .file_name()
        .ok_or_else(|| invalid(format!("input directory has no name: {}", input.display())))?
        .to_string_lossy();
    Ok(input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{name}_json")))
}

fn prepare_one(source: &Path, relative: &Path) -> Result<PreparedJson, MesError> {
    let stored = fs::read(source).map_err(|error| io_error("cannot read", source, error))?;
    let document = extract_document(&stored, portable_path(relative)?)?;
    let entries = document.entries.len();
    let dynamic_names = document
        .entries
        .iter()
        .filter(|entry| entry.name_dynamic)
        .count();
    let multipart = document
        .entries
        .iter()
        .filter(|entry| entry.message_parts.is_some())
        .count();
    Ok(PreparedJson {
        relative: json_relative(relative),
        data: document_to_json(&document)?,
        entries,
        dynamic_names,
        multipart,
    })
}

fn write_prepared(output_root: &Path, prepared: &[PreparedJson]) -> Result<(), MesError> {
    fs::create_dir_all(output_root)
        .map_err(|source| io_error("cannot create output directory", output_root, source))?;
    let result = (|| {
        for item in prepared {
            let output = output_root.join(&item.relative);
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)
                    .map_err(|source| io_error("cannot create directory", parent, source))?;
            }
            fs::write(&output, &item.data)
                .map_err(|source| io_error("cannot write", &output, source))?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(output_root);
    }
    result
}

fn run() -> Result<(), MesError> {
    let args = Args::parse();
    if args.input.is_file() {
        if !is_mes(&args.input) {
            return Err(invalid(format!(
                "input file is not MES: {}",
                args.input.display()
            )));
        }
        let output = args
            .output
            .unwrap_or_else(|| default_file_output(&args.input));
        if output.exists() {
            return Err(invalid(format!(
                "refusing to overwrite existing output: {}",
                output.display()
            )));
        }
        let relative = PathBuf::from(
            args.input
                .file_name()
                .ok_or_else(|| invalid("input MES has no filename"))?,
        );
        let prepared = prepare_one(&args.input, &relative)?;
        fs::write(&output, &prepared.data)
            .map_err(|source| io_error("cannot write", &output, source))?;
        println!(
            "[extract] scanned_files=1 json_files=1 extracted_entries={} \
             dynamic_names={} multipart_entries={} warnings=0 output={}",
            prepared.entries,
            prepared.dynamic_names,
            prepared.multipart,
            output.display()
        );
        return Ok(());
    }

    if !args.input.is_dir() {
        return Err(invalid(format!(
            "input does not exist or is not a file/directory: {}",
            args.input.display()
        )));
    }
    let output = match args.output {
        Some(output) => output,
        None => default_directory_output(&args.input)?,
    };
    if output.exists() {
        return Err(invalid(format!(
            "refusing to overwrite existing output: {}",
            output.display()
        )));
    }

    let mut relative_files = Vec::new();
    collect_mes_files(&args.input, &args.input, &mut relative_files)?;
    if relative_files.is_empty() {
        return Err(invalid(format!(
            "no MES files found under {}",
            args.input.display()
        )));
    }
    let mut prepared = Vec::with_capacity(relative_files.len());
    for relative in &relative_files {
        prepared.push(prepare_one(&args.input.join(relative), relative)?);
    }
    let entries: usize = prepared.iter().map(|item| item.entries).sum();
    let dynamic_names: usize = prepared.iter().map(|item| item.dynamic_names).sum();
    let multipart: usize = prepared.iter().map(|item| item.multipart).sum();
    write_prepared(&output, &prepared)?;
    println!(
        "[extract] scanned_files={} json_files={} extracted_entries={} \
         dynamic_names={} multipart_entries={} warnings=0 output={}",
        prepared.len(),
        prepared.len(),
        entries,
        dynamic_names,
        multipart,
        output.display()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
