use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::encoding::TextEncoding;
use crate::io_util::{
    copy_tree, list_files_recursive, normalize_relative, prepare_output, reject_output_overlap,
    relative_string,
};
use crate::json_model::TranslationFile;
use crate::scenario::{identity_map, Scenario};
use crate::special::{
    is_label_database, is_variable_database, verify_label_database, verify_variable_database,
};
use crate::ToolResult;

const MAX_SCENARIO_ID: u32 = 65535;

#[derive(Debug, Default, Clone)]
pub struct ScanReport {
    pub files: u64,
    pub scenarios: u64,
    pub label_databases: u64,
    pub label_blocks: u64,
    pub labels: u64,
    pub variable_databases: u64,
    pub variables: u64,
    pub unknown_files: u64,
    pub text_entries: u64,
    pub choice_entries: u64,
    pub hyperlinks: u64,
    pub continuation_hyperlinks: u64,
    pub ruby_controls: u64,
    pub font_controls: u64,
}

#[derive(Debug, Default, Clone)]
pub struct ExtractReport {
    pub scan: ScanReport,
    pub json_files: u64,
    pub json_entries: u64,
}

#[derive(Debug, Default, Clone)]
pub struct InjectReport {
    pub json_files: u64,
    pub json_entries: u64,
    pub patched_entries: u64,
    pub unchanged_entries: u64,
    pub copied_files: u64,
    pub removed_join_spaces: u64,
    pub ambiguous_join_spaces: u64,
    pub warnings: Vec<String>,
}

pub fn verify_directory(source: &Path, text_encoding: TextEncoding) -> ToolResult<ScanReport> {
    let identities = identity_map(MAX_SCENARIO_ID);
    let (root, files) = if source.is_file() {
        (
            source.parent().unwrap_or_else(|| Path::new(".")),
            vec![source.to_path_buf()],
        )
    } else {
        (source, list_files_recursive(source)?)
    };
    let mut report = ScanReport::default();
    for path in files {
        report.files += 1;
        let relative = relative_string(root, &path)?;
        let bytes = read_file(&path)?;
        match classify(&bytes, &identities) {
            FileKind::Scenario => {
                let scenario = Scenario::parse(bytes, text_encoding, &identities, &relative)?;
                add_scenario(&mut report, &scenario);
            }
            FileKind::Labels => {
                let stats = verify_label_database(&bytes, &relative)?;
                report.label_databases += 1;
                report.label_blocks += stats.blocks as u64;
                report.labels += stats.labels;
            }
            FileKind::Variables => {
                let stats = verify_variable_database(&bytes, &relative)?;
                report.variable_databases += 1;
                report.variables += stats.variables as u64;
            }
            FileKind::Unknown => report.unknown_files += 1,
        }
    }
    Ok(report)
}

pub fn extract_directory(
    source: &Path,
    output: &Path,
    source_encoding: TextEncoding,
    overwrite: bool,
) -> ToolResult<ExtractReport> {
    reject_output_overlap(source, None, output)?;
    let identities = identity_map(MAX_SCENARIO_ID);
    let files = list_files_recursive(source)?;
    let mut report = ExtractReport::default();
    let mut prepared: Vec<(PathBuf, Vec<u8>)> = Vec::new();

    for path in files {
        report.scan.files += 1;
        let relative = relative_string(source, &path)?;
        let bytes = read_file(&path)?;
        match classify(&bytes, &identities) {
            FileKind::Scenario => {
                let scenario = Scenario::parse(bytes, source_encoding, &identities, &relative)?;
                add_scenario(&mut report.scan, &scenario);
                let (json, _) = scenario.export(&relative, source_encoding)?;
                report.json_entries += json.entries.len() as u64;
                report.json_files += 1;
                let mut serialized = serde_json::to_vec_pretty(&json)
                    .map_err(|error| format!("cannot serialize {relative}: {error}"))?;
                serialized.push(b'\n');
                let json_relative = PathBuf::from(format!("{relative}.json"));
                prepared.push((json_relative, serialized));
            }
            FileKind::Labels => {
                let stats = verify_label_database(&bytes, &relative)?;
                report.scan.label_databases += 1;
                report.scan.label_blocks += stats.blocks as u64;
                report.scan.labels += stats.labels;
            }
            FileKind::Variables => {
                let stats = verify_variable_database(&bytes, &relative)?;
                report.scan.variable_databases += 1;
                report.scan.variables += stats.variables as u64;
            }
            FileKind::Unknown => report.scan.unknown_files += 1,
        }
    }

    prepare_output(output, overwrite)?;
    if let Err(error) = write_prepared(output, &prepared) {
        let _ = fs::remove_dir_all(output);
        return Err(error);
    }
    Ok(report)
}

pub fn inject_directory(
    source: &Path,
    translation: &Path,
    output: &Path,
    source_encoding: TextEncoding,
    target_encoding: TextEncoding,
    overwrite: bool,
) -> ToolResult<InjectReport> {
    reject_output_overlap(source, Some(translation), output)?;
    if !translation.is_dir() {
        return Err(format!(
            "translation directory not found: {}",
            translation.display()
        ));
    }
    let source_root = source
        .canonicalize()
        .map_err(|error| format!("cannot resolve {}: {error}", source.display()))?;
    let identities = identity_map(MAX_SCENARIO_ID);
    let mut json_paths = list_files_recursive(translation)?
        .into_iter()
        .filter(|path| {
            path.extension()
                .is_some_and(|value| value.to_string_lossy().eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<_>>();
    json_paths.sort();
    if json_paths.is_empty() {
        return Err(format!(
            "no JSON translation files found in {}",
            translation.display()
        ));
    }

    let mut seen = HashSet::new();
    let mut patched_files: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    let mut report = InjectReport::default();
    for json_path in json_paths {
        let display = json_path.display().to_string();
        let json_bytes = read_file(&json_path)?;
        let translated: TranslationFile = serde_json::from_slice(&json_bytes)
            .map_err(|error| format!("invalid UTF-8 JSON {display}: {error}"))?;
        let relative = normalize_relative(&translated.file)?;
        let key = relative.to_string_lossy().to_ascii_lowercase();
        if !seen.insert(key) {
            return Err(format!(
                "duplicate _file {:?} in translation JSON set",
                translated.file
            ));
        }
        let source_path = source.join(&relative);
        if !source_path.is_file() {
            return Err(format!(
                "{} selects missing source file {}",
                json_path.display(),
                source_path.display()
            ));
        }
        let resolved_source = source_path
            .canonicalize()
            .map_err(|error| format!("cannot resolve {}: {error}", source_path.display()))?;
        if !resolved_source.starts_with(&source_root) {
            return Err(format!(
                "{} selects a source path outside the source directory",
                translated.file
            ));
        }
        let relative_text = relative
            .components()
            .map(|value| value.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let source_bytes = read_file(&source_path)?;
        if !is_scenario_candidate(&source_bytes) {
            return Err(format!(
                "{}: selected source is not a numbered scenario",
                translated.file
            ));
        }
        let scenario = Scenario::parse(source_bytes, source_encoding, &identities, &relative_text)?;
        let entry_count = translated.entries.len() as u64;
        let (patched, stats) = scenario.apply_translation(
            &relative_text,
            source_encoding,
            target_encoding,
            &translated,
        )?;
        report.json_files += 1;
        report.json_entries += entry_count;
        report.patched_entries += stats.patched;
        report.unchanged_entries += stats.unchanged;
        report.removed_join_spaces += stats.removed_join_spaces;
        report.ambiguous_join_spaces += stats.ambiguous_join_spaces;
        report.warnings.extend(stats.warnings);
        patched_files.push((relative, patched));
    }

    prepare_output(output, overwrite)?;
    let write_result = (|| -> ToolResult<()> {
        report.copied_files = copy_tree(source, output)?;
        for (relative, bytes) in &patched_files {
            let target = output.join(relative);
            fs::write(&target, bytes)
                .map_err(|error| format!("cannot write {}: {error}", target.display()))?;
        }
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(output);
        return Err(error);
    }
    Ok(report)
}

fn write_prepared(root: &Path, files: &[(PathBuf, Vec<u8>)]) -> ToolResult<()> {
    fs::create_dir_all(root)
        .map_err(|error| format!("cannot create {}: {error}", root.display()))?;
    for (relative, bytes) in files {
        let target = root.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        }
        fs::write(&target, bytes)
            .map_err(|error| format!("cannot write {}: {error}", target.display()))?;
    }
    Ok(())
}

fn read_file(path: &Path) -> ToolResult<Vec<u8>> {
    fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

fn add_scenario(report: &mut ScanReport, scenario: &Scenario) {
    report.scenarios += 1;
    report.text_entries += scenario.text_records.len() as u64;
    report.choice_entries += scenario
        .choice_groups
        .iter()
        .map(|group| group.options.len() as u64)
        .sum::<u64>();
    report.hyperlinks += scenario
        .text_records
        .iter()
        .map(|record| record.links.len() as u64)
        .sum::<u64>();
    report.continuation_hyperlinks += scenario
        .text_records
        .iter()
        .map(|record| {
            let body_slots = record
                .parts
                .len()
                .saturating_sub(usize::from(record.instruction.flags == 0x13));
            record
                .links
                .iter()
                .filter(|link| link.line as usize > body_slots)
                .count() as u64
        })
        .sum::<u64>();
    report.ruby_controls += scenario
        .text_records
        .iter()
        .map(|record| record.rubies.len() as u64)
        .sum::<u64>();
    report.font_controls += scenario
        .text_records
        .iter()
        .map(|record| record.fonts.len() as u64)
        .sum::<u64>();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileKind {
    Scenario,
    Labels,
    Variables,
    Unknown,
}

fn classify(bytes: &[u8], identities: &HashMap<[u8; 16], u32>) -> FileKind {
    if is_variable_database(bytes) {
        FileKind::Variables
    } else if is_scenario_candidate(bytes)
        || bytes
            .get(..16)
            .and_then(|slice| <[u8; 16]>::try_from(slice).ok())
            .is_some_and(|identity| identities.contains_key(&identity))
    {
        FileKind::Scenario
    } else if is_label_database(bytes) {
        FileKind::Labels
    } else {
        FileKind::Unknown
    }
}

fn is_scenario_candidate(bytes: &[u8]) -> bool {
    if bytes.len() < 320 {
        return false;
    }
    let entry_count = u32::from_le_bytes(bytes[292..296].try_into().unwrap()) as usize;
    let code_size = u32::from_le_bytes(bytes[296..300].try_into().unwrap()) as usize;
    let data_size = u32::from_le_bytes(bytes[300..304].try_into().unwrap()) as usize;
    if !code_size.is_multiple_of(12) {
        return false;
    }
    entry_count
        .checked_mul(12)
        .and_then(|value| 304usize.checked_add(value))
        .and_then(|value| value.checked_add(code_size))
        .and_then(|value| value.checked_add(data_size))
        .and_then(|value| value.checked_add(16))
        == Some(bytes.len())
}
