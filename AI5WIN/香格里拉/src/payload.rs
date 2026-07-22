use crate::compression::{compress_lzss, decompress_lzss};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transform {
    Compress,
    Decompress { max_output: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformReport {
    pub processed_files: usize,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformError(String);

impl TransformError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for TransformError {}

pub fn transform_input(
    input: &Path,
    output: &Path,
    transform: Transform,
) -> Result<TransformReport, TransformError> {
    let metadata = fs::metadata(input).map_err(|error| io_error("inspect input", input, error))?;
    if metadata.is_file() {
        transform_file(input, output, transform)
    } else if metadata.is_dir() {
        transform_directory(input, output, transform)
    } else {
        Err(TransformError::new(format!(
            "input is neither a regular file nor a directory: {}",
            input.display()
        )))
    }
}

fn transform_file(
    input: &Path,
    output: &Path,
    transform: Transform,
) -> Result<TransformReport, TransformError> {
    reject_existing_output(output)?;
    let source = fs::read(input).map_err(|error| io_error("read input file", input, error))?;
    let transformed = apply_transform(&source, transform)
        .map_err(|error| TransformError::new(format!("{}: {error}", input.display())))?;
    let staging = staging_path(output)?;
    reject_existing_output(&staging)?;
    fs::write(&staging, &transformed)
        .map_err(|error| io_error("write staging file", &staging, error))?;
    if let Err(error) = fs::rename(&staging, output) {
        let _ = fs::remove_file(&staging);
        return Err(io_error("commit output file", output, error));
    }
    Ok(TransformReport {
        processed_files: 1,
        input_bytes: source.len() as u64,
        output_bytes: transformed.len() as u64,
    })
}

fn transform_directory(
    input: &Path,
    output: &Path,
    transform: Transform,
) -> Result<TransformReport, TransformError> {
    reject_existing_output(output)?;
    let mut inputs = Vec::new();
    for item in
        fs::read_dir(input).map_err(|error| io_error("read input directory", input, error))?
    {
        let item = item.map_err(|error| io_error("enumerate input directory", input, error))?;
        let path = item.path();
        if !item
            .file_type()
            .map_err(|error| io_error("inspect input item", &path, error))?
            .is_file()
        {
            return Err(TransformError::new(format!(
                "input directory must be flat and contain only files: {}",
                path.display()
            )));
        }
        inputs.push((item.file_name(), path));
    }
    inputs.sort_unstable_by_key(|(name, _)| name.to_string_lossy().to_ascii_uppercase());

    let mut prepared = Vec::with_capacity(inputs.len());
    let mut report = TransformReport {
        processed_files: 0,
        input_bytes: 0,
        output_bytes: 0,
    };
    for (name, path) in inputs {
        let source = fs::read(&path).map_err(|error| io_error("read input file", &path, error))?;
        let transformed = apply_transform(&source, transform)
            .map_err(|error| TransformError::new(format!("{}: {error}", path.display())))?;
        report.processed_files += 1;
        report.input_bytes += source.len() as u64;
        report.output_bytes += transformed.len() as u64;
        prepared.push((name, transformed));
    }

    let staging = staging_path(output)?;
    reject_existing_output(&staging)?;
    fs::create_dir(&staging)
        .map_err(|error| io_error("create staging directory", &staging, error))?;
    let result = (|| {
        for (name, bytes) in &prepared {
            let path = staging.join(name);
            fs::write(&path, bytes).map_err(|error| io_error("write output file", &path, error))?;
        }
        fs::rename(&staging, output)
            .map_err(|error| io_error("commit output directory", output, error))?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    Ok(report)
}

fn apply_transform(input: &[u8], transform: Transform) -> Result<Vec<u8>, String> {
    match transform {
        Transform::Compress => compress_lzss(input).map_err(|error| error.to_string()),
        Transform::Decompress { max_output } => {
            decompress_lzss(input, max_output).map_err(|error| error.to_string())
        }
    }
}

fn reject_existing_output(path: &Path) -> Result<(), TransformError> {
    if path.exists() {
        Err(TransformError::new(format!(
            "output already exists: {}",
            path.display()
        )))
    } else {
        Ok(())
    }
}

fn staging_path(output: &Path) -> Result<PathBuf, TransformError> {
    let file_name = output.file_name().ok_or_else(|| {
        TransformError::new(format!(
            "output must name an item below a parent: {}",
            output.display()
        ))
    })?;
    let mut name = file_name.to_os_string();
    name.push(".partial");
    Ok(output.with_file_name(name))
}

fn io_error(action: &str, path: &Path, error: io::Error) -> TransformError {
    TransformError::new(format!("failed to {action} {}: {error}", path.display()))
}
