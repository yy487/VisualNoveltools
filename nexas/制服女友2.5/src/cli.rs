use crate::{ToolResult, error};
use std::path::{Path, PathBuf};

pub fn parse_output(
    args: &[String],
    required: usize,
) -> ToolResult<(Vec<PathBuf>, Option<PathBuf>)> {
    let mut positional = Vec::new();
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--output" | "-o" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| error("--output needs a path"))?;
                output = Some(PathBuf::from(value));
            }
            value if value.starts_with('-') => {
                return Err(error(format!("unknown option: {value}")));
            }
            value => positional.push(PathBuf::from(value)),
        }
        index += 1;
    }
    if positional.len() != required {
        return Err(error(format!("expected {required} positional path(s)")));
    }
    Ok((positional, output))
}

pub fn require_path(path: &Path, label: &str) -> ToolResult<()> {
    if !path.exists() {
        return Err(error(format!("{label} does not exist: {}", path.display())));
    }
    Ok(())
}

pub fn print_extract_help() {
    println!(
        "nexas_extract\n\nUsage:\n  nexas_extract <script.bin|mes_dir> [--output <json|json_dir>]\n\nExtracts translatable NeXAS strings to per-file UTF-8 JSON.\nSystem files, resource names and pure control strings are skipped.\nExisting output is never overwritten."
    );
}

pub fn print_inject_help() {
    println!(
        "nexas_inject\n\nUsage:\n  nexas_inject <script.bin|mes_dir> <file.json|json_dir> [--output <path>]\n\nValidates UTF-8 JSON and rebuilds scripts with translated messages.\nA directory input is copied in full; unmatched files remain unchanged.\nExisting output is never overwritten."
    );
}

pub fn print_rebuild_help() {
    println!(
        "nexas_rebuild\n\nUsage:\n  nexas_rebuild <script.bin|mes_dir> [--output <path>]\n\nParses and rebuilds scripts without translation, for byte-exact round-trip checks.\nExisting output is never overwritten."
    );
}
