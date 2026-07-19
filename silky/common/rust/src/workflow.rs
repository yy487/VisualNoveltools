use crate::mes::MesScript;
use crate::text::{extract_entries, inject_entries, InjectionStats};
use crate::util::{
    file_name_string, matching_files, pretty_json_bytes, read_utf8, strip_known_suffix,
    write_new_file, write_new_tree,
};
use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ExtractReport {
    pub scanned_files: usize,
    pub json_files: usize,
    pub extracted_entries: usize,
    pub skipped_blocks: usize,
    pub warnings: Vec<String>,
    pub output: PathBuf,
}

#[derive(Debug)]
pub struct InjectReport {
    pub scanned_files: usize,
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub warnings: Vec<String>,
    pub output: PathBuf,
}

struct PreparedExtract {
    relative_output: PathBuf,
    bytes: Vec<u8>,
    entries: usize,
    skipped: usize,
    warnings: Vec<String>,
}

struct PreparedInject {
    relative_output: PathBuf,
    bytes: Vec<u8>,
    stats: InjectionStats,
    warnings: Vec<String>,
}

pub fn extract_path(
    input: &Path,
    output: Option<&Path>,
    encoding: &str,
    pattern: &str,
    jobs: usize,
) -> Result<ExtractReport> {
    if input.is_file() {
        let output = output.map(Path::to_path_buf).unwrap_or_else(|| {
            let name = file_name_string(input).unwrap_or_else(|_| "script.MES".to_owned());
            input.with_file_name(format!("{name}.json"))
        });
        let prepared = prepare_extract(input, &output, encoding)?;
        write_new_file(&output, &prepared.bytes)?;
        return Ok(ExtractReport {
            scanned_files: 1,
            json_files: 1,
            extracted_entries: prepared.entries,
            skipped_blocks: prepared.skipped,
            warnings: prepared.warnings,
            output,
        });
    }
    if !input.is_dir() {
        bail!("input does not exist: {}", input.display());
    }
    let output = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_sibling_directory(input, "_json"));
    if output.exists() {
        bail!("output directory already exists: {}", output.display());
    }
    let files = matching_files(input, pattern)?;
    if files.is_empty() {
        bail!("no files matched {pattern:?} in {}", input.display());
    }

    let pool = worker_pool(jobs, files.len())?;
    let prepared: Vec<PreparedExtract> = pool.install(|| {
        files
            .par_iter()
            .map(|path| {
                let file_name = file_name_string(path)?;
                let base = strip_known_suffix(&file_name, &[".MES", ".mes"]);
                prepare_extract(path, Path::new(&format!("{base}.json")), encoding)
            })
            .collect::<Result<Vec<_>>>()
    })?;

    let mut tree = Vec::with_capacity(prepared.len());
    let mut extracted_entries = 0usize;
    let mut skipped_blocks = 0usize;
    let mut warnings = Vec::new();
    for item in prepared {
        extracted_entries += item.entries;
        skipped_blocks += item.skipped;
        warnings.extend(item.warnings);
        tree.push((item.relative_output, item.bytes));
    }
    write_new_tree(&output, &tree)?;
    Ok(ExtractReport {
        scanned_files: files.len(),
        json_files: tree.len(),
        extracted_entries,
        skipped_blocks,
        warnings,
        output,
    })
}

pub fn inject_path(
    input: &Path,
    json: &Path,
    output: Option<&Path>,
    encoding: &str,
    pattern: &str,
    jobs: usize,
) -> Result<InjectReport> {
    if input.is_file() {
        if !json.is_file() {
            bail!("JSON input is not a file: {}", json.display());
        }
        let output = output.map(Path::to_path_buf).unwrap_or_else(|| {
            let name = file_name_string(input).unwrap_or_else(|_| "script.MES".to_owned());
            let base = strip_known_suffix(&name, &[".MES", ".mes"]);
            input.with_file_name(format!("{base}_injected.MES"))
        });
        let prepared = prepare_inject(
            input,
            json,
            output.file_name().map(Path::new).unwrap_or(&output),
            encoding,
        )?;
        write_new_file(&output, &prepared.bytes)?;
        return Ok(InjectReport {
            scanned_files: 1,
            json_entries: prepared.stats.json_entries,
            patched: prepared.stats.patched,
            unchanged: prepared.stats.unchanged,
            warnings: prepared.warnings,
            output,
        });
    }
    if !input.is_dir() {
        bail!("MES input does not exist: {}", input.display());
    }
    if !json.is_dir() {
        bail!("batch JSON input is not a directory: {}", json.display());
    }
    let output = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_sibling_directory(input, "_injected"));
    if output.exists() {
        bail!("output directory already exists: {}", output.display());
    }
    let files = matching_files(input, pattern)?;
    if files.is_empty() {
        bail!("no files matched {pattern:?} in {}", input.display());
    }

    let mut tasks = Vec::with_capacity(files.len());
    for mes_path in &files {
        let file_name = file_name_string(mes_path)?;
        let base = strip_known_suffix(&file_name, &[".MES", ".mes"]);
        let json_path = json.join(format!("{base}.json"));
        if !json_path.is_file() {
            bail!("missing JSON for {file_name}: {}", json_path.display());
        }
        tasks.push((
            mes_path.clone(),
            json_path,
            PathBuf::from(format!("{base}.MES")),
        ));
    }

    let pool = worker_pool(jobs, tasks.len())?;
    let prepared: Vec<PreparedInject> = pool.install(|| {
        tasks
            .par_iter()
            .map(|(mes, json, relative)| prepare_inject(mes, json, relative, encoding))
            .collect::<Result<Vec<_>>>()
    })?;

    let mut tree = Vec::with_capacity(prepared.len());
    let mut json_entries = 0usize;
    let mut patched = 0usize;
    let mut unchanged = 0usize;
    let mut warnings = Vec::new();
    for item in prepared {
        json_entries += item.stats.json_entries;
        patched += item.stats.patched;
        unchanged += item.stats.unchanged;
        warnings.extend(item.warnings);
        tree.push((item.relative_output, item.bytes));
    }
    write_new_tree(&output, &tree)?;
    Ok(InjectReport {
        scanned_files: files.len(),
        json_entries,
        patched,
        unchanged,
        warnings,
        output,
    })
}

fn prepare_extract(
    input: &Path,
    relative_output: &Path,
    encoding: &str,
) -> Result<PreparedExtract> {
    let source = fs::read(input).with_context(|| format!("failed to read {}", input.display()))?;
    let source_file = file_name_string(input)?;
    let script = MesScript::parse(&source, encoding)
        .with_context(|| format!("failed to parse MES {}", input.display()))?;
    let result = extract_entries(&script, &source_file);
    let warnings = result
        .warnings
        .into_iter()
        .map(|warning| format!("{source_file}: {warning}"))
        .collect();
    Ok(PreparedExtract {
        relative_output: relative_output.to_path_buf(),
        bytes: pretty_json_bytes(&result.entries)?,
        entries: result.entries.len(),
        skipped: result.skipped_blocks,
        warnings,
    })
}

fn prepare_inject(
    input: &Path,
    json: &Path,
    relative_output: &Path,
    encoding: &str,
) -> Result<PreparedInject> {
    let source = fs::read(input).with_context(|| format!("failed to read {}", input.display()))?;
    let source_file = file_name_string(input)?;
    let json_text = read_utf8(json)?;
    let mut script = MesScript::parse(&source, encoding)
        .with_context(|| format!("failed to parse MES {}", input.display()))?;
    let stats = inject_entries(&mut script, &json_text, &source_file)
        .with_context(|| format!("failed to inject {}", input.display()))?;
    let rebuilt = script
        .to_bytes()
        .with_context(|| format!("failed to rebuild MES {}", input.display()))?;
    MesScript::parse(&rebuilt, encoding).with_context(|| {
        format!(
            "rebuilt MES failed structural verification: {}",
            input.display()
        )
    })?;
    if stats.patched == 0 && rebuilt != source {
        bail!(
            "zero-change MES roundtrip is not byte-exact: {}",
            input.display()
        );
    }
    let warnings = stats
        .warnings
        .iter()
        .map(|warning| format!("{source_file}: {warning}"))
        .collect();
    Ok(PreparedInject {
        relative_output: relative_output.to_path_buf(),
        bytes: rebuilt,
        stats,
        warnings,
    })
}

fn worker_pool(jobs: usize, task_count: usize) -> Result<rayon::ThreadPool> {
    let available = std::thread::available_parallelism().map_or(1, usize::from);
    let workers = (if jobs == 0 { available } else { jobs }).clamp(1, task_count.max(1));
    rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .build()
        .context("failed to create worker pool")
}

fn default_sibling_directory(input: &Path, suffix: &str) -> PathBuf {
    let name = file_name_string(input).unwrap_or_else(|_| "silky".to_owned());
    input.with_file_name(format!("{name}{suffix}"))
}
