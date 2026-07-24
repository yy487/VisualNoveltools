use clap::Parser;
use std::fs;
use std::path::{Component, Path, PathBuf};
use yuno_hdi_tools::mes::MesError;
use yuno_hdi_tools::mes_text::{document_from_json, inject_document};

#[derive(Debug, Parser)]
#[command(about = "Inject validated UTF-8 JSON text into new YU-NO PC-98 MES files")]
struct Args {
    /// Original compressed MES file or resource directory. It is never changed.
    source: PathBuf,

    /// Translation JSON file or directory produced by extract_mes.
    json: PathBuf,

    /// New MES file or copied resource tree. Defaults beside SOURCE.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Debug)]
struct PreparedMes {
    relative: PathBuf,
    stored: Vec<u8>,
    entries: usize,
    patched: usize,
    unchanged: usize,
}

fn invalid(message: impl Into<String>) -> MesError {
    MesError::Invalid(message.into())
}

fn io_error(action: &str, path: &Path, source: std::io::Error) -> MesError {
    invalid(format!("{action} {}: {source}", path.display()))
}

fn is_mes_json(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.to_ascii_lowercase().ends_with(".mes.json"))
}

fn collect_json_files(
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
            collect_json_files(root, &path, output)?;
        } else if file_type.is_file() && is_mes_json(&path) {
            output.push(
                path.strip_prefix(root)
                    .map_err(|_| invalid("JSON path escaped the translation root"))?
                    .to_path_buf(),
            );
        }
    }
    Ok(())
}

fn mes_relative(json_relative: &Path) -> Result<PathBuf, MesError> {
    let filename = json_relative
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            invalid(format!(
                "JSON filename is not Unicode: {}",
                json_relative.display()
            ))
        })?;
    let mes_name = filename
        .strip_suffix(".json")
        .ok_or_else(|| invalid(format!("translation is not a .MES.json file: {filename}")))?;
    Ok(json_relative.with_file_name(mes_name))
}

fn validate_relative(path: &Path) -> Result<(), MesError> {
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(invalid(format!("unsafe relative path: {}", path.display())));
    }
    Ok(())
}

fn portable_path(path: &Path) -> Result<String, MesError> {
    validate_relative(path)?;
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

fn prepare_one(source: &Path, json: &Path, relative: &Path) -> Result<PreparedMes, MesError> {
    let source_stored = fs::read(source).map_err(|error| io_error("cannot read", source, error))?;
    let json_data = fs::read(json).map_err(|error| io_error("cannot read", json, error))?;
    let document = document_from_json(&json_data)?;
    let result = inject_document(&source_stored, &document, &portable_path(relative)?)?;
    Ok(PreparedMes {
        relative: relative.to_path_buf(),
        stored: result.stored,
        entries: result.json_entries,
        patched: result.patched_fields,
        unchanged: result.unchanged_fields,
    })
}

fn copy_tree(source: &Path, output: &Path) -> Result<(), MesError> {
    fs::create_dir(output)
        .map_err(|error| io_error("cannot create output directory", output, error))?;
    let result = (|| {
        let mut entries: Vec<_> = fs::read_dir(source)
            .map_err(|error| io_error("cannot read directory", source, error))?
            .collect::<std::result::Result<_, _>>()
            .map_err(|error| io_error("cannot enumerate directory", source, error))?;
        entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_lowercase());
        for entry in entries {
            let input = entry.path();
            let target = output.join(entry.file_name());
            let file_type = entry
                .file_type()
                .map_err(|error| io_error("cannot inspect", &input, error))?;
            if file_type.is_symlink() {
                return Err(invalid(format!(
                    "symbolic links are not supported: {}",
                    input.display()
                )));
            }
            if file_type.is_dir() {
                copy_tree(&input, &target)?;
            } else if file_type.is_file() {
                fs::copy(&input, &target)
                    .map_err(|error| io_error("cannot copy", &input, error))?;
            } else {
                return Err(invalid(format!(
                    "unsupported filesystem entry: {}",
                    input.display()
                )));
            }
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(output);
    }
    result
}

fn default_file_output(source: &Path) -> Result<PathBuf, MesError> {
    let stem = source
        .file_stem()
        .ok_or_else(|| invalid(format!("source file has no stem: {}", source.display())))?
        .to_string_lossy();
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("MES");
    Ok(source
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{stem}_injected.{extension}")))
}

fn default_directory_output(source: &Path) -> Result<PathBuf, MesError> {
    let name = source
        .file_name()
        .ok_or_else(|| {
            invalid(format!(
                "source directory has no name: {}",
                source.display()
            ))
        })?
        .to_string_lossy();
    Ok(source
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{name}_injected")))
}

fn run() -> Result<(), MesError> {
    let args = Args::parse();
    if args.source.is_file() {
        if !args.json.is_file() {
            return Err(invalid("file injection requires one JSON file"));
        }
        let output = match args.output {
            Some(output) => output,
            None => default_file_output(&args.source)?,
        };
        if output.exists() {
            return Err(invalid(format!(
                "refusing to overwrite existing output: {}",
                output.display()
            )));
        }
        let relative = PathBuf::from(
            args.source
                .file_name()
                .ok_or_else(|| invalid("source MES has no filename"))?,
        );
        let prepared = prepare_one(&args.source, &args.json, &relative)?;
        fs::write(&output, &prepared.stored)
            .map_err(|error| io_error("cannot write", &output, error))?;
        println!(
            "[inject] json_files=1 json_entries={} patched={} unchanged={} \
             output_files=1 output={}",
            prepared.entries,
            prepared.patched,
            prepared.unchanged,
            output.display()
        );
        return Ok(());
    }

    if !args.source.is_dir() || !args.json.is_dir() {
        return Err(invalid(
            "directory injection requires source and JSON directories",
        ));
    }
    let output = match args.output {
        Some(output) => output,
        None => default_directory_output(&args.source)?,
    };
    if output.exists() {
        return Err(invalid(format!(
            "refusing to overwrite existing output: {}",
            output.display()
        )));
    }

    let mut json_files = Vec::new();
    collect_json_files(&args.json, &args.json, &mut json_files)?;
    if json_files.is_empty() {
        return Err(invalid(format!(
            "no .MES.json files found under {}",
            args.json.display()
        )));
    }
    let mut prepared = Vec::with_capacity(json_files.len());
    for json_relative in &json_files {
        let relative = mes_relative(json_relative)?;
        validate_relative(&relative)?;
        let source = args.source.join(&relative);
        if !source.is_file() {
            return Err(invalid(format!(
                "translation source MES is missing: {}",
                source.display()
            )));
        }
        prepared.push(prepare_one(
            &source,
            &args.json.join(json_relative),
            &relative,
        )?);
    }

    copy_tree(&args.source, &output)?;
    let write_result = (|| {
        for item in &prepared {
            let target = output.join(&item.relative);
            fs::write(&target, &item.stored)
                .map_err(|error| io_error("cannot write", &target, error))?;
        }
        Ok(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_dir_all(&output);
    }
    write_result?;

    let entries: usize = prepared.iter().map(|item| item.entries).sum();
    let patched: usize = prepared.iter().map(|item| item.patched).sum();
    let unchanged: usize = prepared.iter().map(|item| item.unchanged).sum();
    println!(
        "[inject] json_files={} json_entries={} patched={} unchanged={} \
         output_files={} output={}",
        prepared.len(),
        entries,
        patched,
        unchanged,
        prepared.len(),
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
