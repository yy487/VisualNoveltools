use clap::{Parser, Subcommand};
use loveletter_obj_text_tool::{
    extract_entries, inject_entries, parse_obj, ExtractReport, InjectOptions, Result, TextEntry,
};
use serde_json::from_slice;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "loveletter-obj-text-tool")]
#[command(about = "Extract/inject CP932 text from Love Letter .o VM scripts")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Extract one .o file or a directory tree to UTF-8 JSON.
    Extract {
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Inject one JSON file or a directory tree into a copied .o tree.
    Inject {
        source: PathBuf,
        json: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Allow changing split speaker names after _scr_name validation.
        #[arg(long)]
        write_names: bool,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Extract { input, output } => run_extract(&input, output.as_deref()),
        Command::Inject {
            source,
            json,
            output,
            write_names,
        } => run_inject(&source, &json, output.as_deref(), write_names),
    }
}

fn run_extract(input: &Path, output: Option<&Path>) -> Result<()> {
    if input.is_file() {
        let output = output
            .map(PathBuf::from)
            .unwrap_or_else(|| default_file_output(input, "json"));
        ensure_new_path(&output)?;
        let bytes = fs::read(input)?;
        let parsed = parse_obj(&bytes)?;
        let file_name = file_name(input)?;
        let report = extract_entries(&parsed, &file_name)?;
        write_json(&output, &report.entries)?;
        print_extract_report(&file_name, &output, &report, 1);
        return Ok(());
    }
    if !input.is_dir() {
        return Err(crate_error(format!(
            "input does not exist: {}",
            input.display()
        )));
    }

    let output = output
        .map(PathBuf::from)
        .unwrap_or_else(|| default_dir_output(input, "json"));
    ensure_new_path(&output)?;
    let files = collect_files(input)?;
    let mut scanned = 0usize;
    let mut json_files = 0usize;
    let mut total_entries = 0usize;
    let mut total_skipped = 0usize;
    let mut warnings = 0usize;
    fs::create_dir_all(&output)?;
    for source in files.into_iter().filter(|path| is_o_file(path)) {
        scanned += 1;
        let relative = source
            .strip_prefix(input)
            .map_err(|_| crate_error(format!("cannot relativize {}", source.display())))?;
        let mut destination = output.join(relative);
        destination.set_extension("json");
        let bytes = fs::read(&source)?;
        let parsed = parse_obj(&bytes)?;
        let file_name = file_name(&source)?;
        let report = extract_entries(&parsed, &file_name)?;
        write_json(&destination, &report.entries)?;
        json_files += 1;
        total_entries += report.entries.len();
        total_skipped += report.skipped;
        warnings += report.warnings.len();
        print_extract_report(&file_name, &destination, &report, 1);
    }
    println!(
        "[extract] scanned_files={} json_files={} extracted_entries={} skipped={} warnings={} output={}",
        scanned,
        json_files,
        total_entries,
        total_skipped,
        warnings,
        output.display()
    );
    Ok(())
}

fn run_inject(source: &Path, json: &Path, output: Option<&Path>, write_names: bool) -> Result<()> {
    if source.is_file() {
        if !json.is_file() {
            return Err(crate_error(format!(
                "JSON file does not exist: {}",
                json.display()
            )));
        }
        let output = output
            .map(PathBuf::from)
            .unwrap_or_else(|| default_file_output(source, "injected.o"));
        ensure_new_path(&output)?;
        let bytes = fs::read(source)?;
        let parsed = parse_obj(&bytes)?;
        let entries = read_json(json)?;
        let file_name = file_name(source)?;
        let (rebuilt, report) =
            inject_entries(&parsed, &file_name, &entries, InjectOptions { write_names })?;
        write_new(&output, &rebuilt)?;
        print_inject_report(&file_name, &output, &report, 1);
        return Ok(());
    }
    if !source.is_dir() {
        return Err(crate_error(format!(
            "source does not exist: {}",
            source.display()
        )));
    }
    if !json.is_dir() {
        return Err(crate_error(format!(
            "JSON directory does not exist: {}",
            json.display()
        )));
    }
    let output = output
        .map(PathBuf::from)
        .unwrap_or_else(|| default_dir_output(source, "injected"));
    ensure_new_path(&output)?;

    let files = collect_files(source)?;
    let mut pending = Vec::new();
    let mut json_entries = 0usize;
    let mut patched = 0usize;
    let mut unchanged = 0usize;
    let mut skipped = 0usize;
    for source_file in files {
        let relative = source_file
            .strip_prefix(source)
            .map_err(|_| crate_error(format!("cannot relativize {}", source_file.display())))?;
        let destination = output.join(relative);
        if !is_o_file(&source_file) {
            pending.push((destination, fs::read(&source_file)?));
            continue;
        }

        let mut json_path = json.join(relative);
        json_path.set_extension("json");
        let source_bytes = fs::read(&source_file)?;
        let file_name = file_name(&source_file)?;
        if !json_path.is_file() {
            skipped += 1;
            pending.push((destination, source_bytes));
            println!(
                "[inject] file={} json=missing action=unchanged",
                source_file.display()
            );
            continue;
        }
        let parsed = parse_obj(&source_bytes)?;
        let entries = read_json(&json_path)?;
        let (rebuilt, report) =
            inject_entries(&parsed, &file_name, &entries, InjectOptions { write_names })?;
        json_entries += report.json_entries;
        patched += report.patched;
        unchanged += report.unchanged;
        pending.push((destination, rebuilt));
        print_inject_report(&file_name, &json_path, &report, 1);
    }
    for (path, bytes) in &pending {
        write_new(path, bytes)?;
    }
    println!(
        "[inject] json_entries={} patched={} unchanged={} skipped_files={} output={}",
        json_entries,
        patched,
        unchanged,
        skipped,
        output.display()
    );
    Ok(())
}

fn read_json(path: &Path) -> Result<Vec<TextEntry>> {
    let bytes = fs::read(path)?;
    Ok(from_slice(&bytes)?)
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    let mut with_newline = bytes;
    with_newline.push(b'\n');
    write_new(path, &with_newline)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    if path.exists() {
        return Err(crate_error(format!(
            "refusing to overwrite existing output: {}",
            path.display()
        )));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(())
}

fn ensure_new_path(path: &Path) -> Result<()> {
    if path.exists() {
        return Err(crate_error(format!(
            "refusing to overwrite existing output: {}",
            path.display()
        )));
    }
    Ok(())
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files_inner(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_inner(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for item in fs::read_dir(root)? {
        let path = item?.path();
        if path.is_dir() {
            collect_files_inner(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn is_o_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("o"))
}

fn file_name(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| crate_error(format!("path has no UTF-8 file name: {}", path.display())))
}

fn default_file_output(input: &Path, suffix: &str) -> PathBuf {
    let name = input
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output");
    if suffix == "json" {
        input.with_file_name(format!("{name}.json"))
    } else {
        let stem = input
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or(name);
        input.with_file_name(format!("{stem}_{suffix}"))
    }
}

fn default_dir_output(input: &Path, suffix: &str) -> PathBuf {
    let name = input
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output");
    input.with_file_name(format!("{name}_{suffix}"))
}

fn print_extract_report(file_name: &str, output: &Path, report: &ExtractReport, files: usize) {
    println!(
        "[extract] file={} output={} entries={} skipped={} warnings={} scanned_files={}",
        file_name,
        output.display(),
        report.entries.len(),
        report.skipped,
        report.warnings.len(),
        files
    );
    for warning in &report.warnings {
        eprintln!("[extract][warning] {warning}");
    }
}

fn print_inject_report(
    file_name: &str,
    output: &Path,
    report: &loveletter_obj_text_tool::InjectReport,
    files: usize,
) {
    println!(
        "[inject] file={} output={} json_entries={} patched={} unchanged={} warnings={} scanned_files={}",
        file_name,
        output.display(),
        report.json_entries,
        report.patched,
        report.unchanged,
        report.warnings.len(),
        files
    );
    for warning in &report.warnings {
        eprintln!("[inject][warning] {warning}");
    }
}

fn crate_error(message: String) -> loveletter_obj_text_tool::ToolError {
    loveletter_obj_text_tool::ToolError::Text(message)
}
