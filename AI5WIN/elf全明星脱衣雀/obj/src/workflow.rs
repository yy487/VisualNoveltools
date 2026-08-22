use crate::encoding::{decode_text, encode_text, EncodingRoute};
use crate::obj::{ObjContainer, ObjReplacement};
use crate::{fail, sha256_hex, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const SCHEMA_VERSION: u32 = 1;
const FORMAT: &str = "ai5win-mahjong-obj";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationDocument {
    pub schema_version: u32,
    pub format: String,
    pub source_file: String,
    pub source_sha256: String,
    pub source_encoding: EncodingRoute,
    pub source_storage: String,
    pub entries: Vec<TranslationEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationEntry {
    pub scr_msg: String,
    pub message: String,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: u64,
    #[serde(rename = "_inst_offset")]
    pub inst_offset: u32,
    #[serde(rename = "_size")]
    pub size: u32,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_speaker_id", skip_serializing_if = "Option::is_none")]
    pub speaker_id: Option<i32>,
    #[serde(rename = "_voice", skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub format: String,
    pub source_encoding: EncodingRoute,
    pub files: Vec<ManifestFile>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestFile {
    pub source_file: String,
    pub source_sha256: String,
    pub source_storage: String,
    pub translation_file: String,
    pub entry_count: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WorkflowStats {
    pub scanned_files: usize,
    pub script_files: usize,
    pub text_entries: usize,
}

pub fn extract_path(
    input: &Path,
    output: &Path,
    source_encoding: EncodingRoute,
    overwrite: bool,
) -> Result<WorkflowStats> {
    let input = absolute_existing(input)?;
    reject_nested_output(&input, output)?;
    check_destination(output, overwrite)?;
    let stage = staging_path(output)?;
    if stage.exists() {
        remove_path(&stage)?;
    }
    fs::create_dir_all(&stage)?;

    let result = (|| {
        let (root, files) = input_files(&input)?;
        let mut manifest = Manifest {
            schema_version: SCHEMA_VERSION,
            format: FORMAT.to_owned(),
            source_encoding,
            files: Vec::new(),
        };
        let mut stats = WorkflowStats {
            scanned_files: files.len(),
            ..WorkflowStats::default()
        };
        for source_path in files {
            let stored = fs::read(&source_path)?;
            let Ok(container) = ObjContainer::parse(&stored) else {
                continue;
            };
            let relative = relative_name(&root, &source_path)?;
            let document = analyze_container(&container, &stored, &relative, source_encoding)?;
            let translation_file = format!("{relative}.json");
            write_json(
                &stage.join(path_from_manifest(&translation_file)?),
                &document,
            )?;
            manifest.files.push(ManifestFile {
                source_file: relative,
                source_sha256: document.source_sha256.clone(),
                source_storage: document.source_storage.clone(),
                translation_file,
                entry_count: document.entries.len(),
            });
            stats.script_files += 1;
            stats.text_entries += document.entries.len();
        }
        manifest
            .files
            .sort_by(|left, right| left.source_file.cmp(&right.source_file));
        if manifest.files.is_empty() {
            return fail("no structurally valid AI5WIN Mahjong OBJ was found");
        }
        write_json(&stage.join("ai5win_obj_manifest.json"), &manifest)?;
        commit_stage(&stage, output, overwrite)?;
        Ok(stats)
    })();
    if result.is_err() && stage.exists() {
        let _ = remove_path(&stage);
    }
    result
}

pub fn inject_path(
    source: &Path,
    translation: &Path,
    output: &Path,
    output_encoding: EncodingRoute,
    overwrite: bool,
) -> Result<WorkflowStats> {
    let source = absolute_existing(source)?;
    let translation = absolute_existing(translation)?;
    reject_nested_output(&source, output)?;
    reject_nested_output(&translation, output)?;
    check_destination(output, overwrite)?;
    let manifest: Manifest = read_json(&translation.join("ai5win_obj_manifest.json"))?;
    validate_manifest(&manifest)?;
    let stage = staging_path(output)?;
    if stage.exists() {
        remove_path(&stage)?;
    }

    let result = (|| {
        if source.is_dir() {
            copy_tree(&source, &stage)?;
        } else {
            if manifest.files.len() != 1 {
                return fail("file source requires a manifest containing exactly one OBJ");
            }
            if let Some(parent) = stage.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source, &stage)?;
        }
        let mut stats = WorkflowStats {
            scanned_files: manifest.files.len(),
            ..WorkflowStats::default()
        };
        for file in &manifest.files {
            let source_path = source_member(&source, &file.source_file)?;
            let destination_path = source_member(&stage, &file.source_file)?;
            let stored = fs::read(&source_path)?;
            if sha256_hex(&stored) != file.source_sha256 {
                return fail(format!(
                    "source hash mismatch for {}; use the exact files used for extraction",
                    file.source_file
                ));
            }
            let container = ObjContainer::parse(&stored)
                .map_err(|error| format!("{}: {error}", file.source_file))?;
            if container.storage.label() != file.source_storage {
                return fail(format!("{}: source storage mode changed", file.source_file));
            }
            let fresh = analyze_container(
                &container,
                &stored,
                &file.source_file,
                manifest.source_encoding,
            )?;
            let edited: TranslationDocument =
                read_json(&translation.join(path_from_manifest(&file.translation_file)?))?;
            let rebuilt_decoded = inject_document(&container, &fresh, &edited, output_encoding)?;
            let rebuilt_stored = container.store_rebuilt(rebuilt_decoded)?;
            verify_injected(&rebuilt_stored, &file.source_file, &edited, output_encoding)?;
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(destination_path, rebuilt_stored)?;
            stats.script_files += 1;
            stats.text_entries += edited.entries.len();
        }
        commit_stage(&stage, output, overwrite)?;
        Ok(stats)
    })();
    if result.is_err() && stage.exists() {
        let _ = remove_path(&stage);
    }
    result
}

pub fn verify_path(input: &Path, encoding: EncodingRoute) -> Result<WorkflowStats> {
    let input = absolute_existing(input)?;
    let (root, files) = input_files(&input)?;
    let mut stats = WorkflowStats {
        scanned_files: files.len(),
        ..WorkflowStats::default()
    };
    for path in files {
        let stored = fs::read(&path)?;
        let Ok(container) = ObjContainer::parse(&stored) else {
            continue;
        };
        let relative = relative_name(&root, &path)?;
        let document = analyze_container(&container, &stored, &relative, encoding)?;
        stats.script_files += 1;
        stats.text_entries += document.entries.len();
    }
    if stats.script_files == 0 {
        return fail("no structurally valid AI5WIN Mahjong OBJ was found");
    }
    Ok(stats)
}

fn analyze_container(
    container: &ObjContainer,
    stored: &[u8],
    relative: &str,
    encoding: EncodingRoute,
) -> Result<TranslationDocument> {
    let mut entries = Vec::new();
    for (index, slot) in container.script.message_slots()?.into_iter().enumerate() {
        let message =
            decode_text(container.script.text_bytes(&slot.text), encoding).map_err(|error| {
                format!(
                    "{relative}: message at decoded OBJ offset 0x{:X}: {error}",
                    slot.text.data_start
                )
            })?;
        let voice = slot
            .voice
            .as_deref()
            .map(|bytes| {
                decode_text(bytes, encoding).map_err(|error| {
                    format!(
                        "{relative}: voice before decoded OBJ offset 0x{:X}: {error}",
                        slot.instruction_offset
                    )
                })
            })
            .transpose()?;
        let entry_type = if voice.is_some()
            || (!message.is_empty() && message.chars().all(|character| character == '…'))
        {
            "dialogue"
        } else {
            "debug_label"
        };
        entries.push(TranslationEntry {
            scr_msg: message.clone(),
            message,
            file: relative.to_owned(),
            index,
            offset: u64::from(slot.text.data_start),
            inst_offset: slot.instruction_offset,
            size: slot.text.data_end - slot.text.data_start,
            entry_type: entry_type.to_owned(),
            speaker_id: slot.speaker_id,
            voice,
        });
    }
    Ok(TranslationDocument {
        schema_version: SCHEMA_VERSION,
        format: FORMAT.to_owned(),
        source_file: relative.to_owned(),
        source_sha256: sha256_hex(stored),
        source_encoding: encoding,
        source_storage: container.storage.label().to_owned(),
        entries,
    })
}

fn inject_document(
    container: &ObjContainer,
    fresh: &TranslationDocument,
    edited: &TranslationDocument,
    output_encoding: EncodingRoute,
) -> Result<Vec<u8>> {
    validate_document_header(fresh, edited)?;
    if fresh.entries.len() != edited.entries.len() {
        return fail(format!(
            "{}: entry count changed from {} to {}",
            fresh.source_file,
            fresh.entries.len(),
            edited.entries.len()
        ));
    }
    let force_reencode = output_encoding == EncodingRoute::Gbk;
    let mut replacements = Vec::new();
    let mut owned_offsets = HashSet::new();
    for (fresh_entry, edited_entry) in fresh.entries.iter().zip(&edited.entries) {
        validate_entry_metadata(fresh_entry, edited_entry)?;
        let start = u32::try_from(fresh_entry.offset)?;
        if !owned_offsets.insert(start) {
            return fail(format!(
                "{}: duplicate message ownership at decoded offset 0x{start:X}",
                fresh.source_file
            ));
        }
        if force_reencode || edited_entry.message != fresh_entry.scr_msg {
            let encoded = encode_text(&edited_entry.message, output_encoding).map_err(|error| {
                format!(
                    "{} entry #{} message: {error}",
                    fresh.source_file, fresh_entry.index
                )
            })?;
            replacements.push(ObjReplacement {
                start,
                end: start + fresh_entry.size,
                data: encoded,
            });
        }
    }
    container.script.rebuild(replacements)
}

fn validate_document_header(
    fresh: &TranslationDocument,
    edited: &TranslationDocument,
) -> Result<()> {
    if edited.schema_version != SCHEMA_VERSION
        || edited.format != FORMAT
        || edited.source_file != fresh.source_file
        || edited.source_sha256 != fresh.source_sha256
        || edited.source_encoding != fresh.source_encoding
        || edited.source_storage != fresh.source_storage
    {
        return fail(format!(
            "{}: translation document metadata does not match the extracted source",
            fresh.source_file
        ));
    }
    Ok(())
}

fn validate_entry_metadata(fresh: &TranslationEntry, edited: &TranslationEntry) -> Result<()> {
    let valid = edited.scr_msg == fresh.scr_msg
        && edited.file == fresh.file
        && edited.index == fresh.index
        && edited.offset == fresh.offset
        && edited.inst_offset == fresh.inst_offset
        && edited.size == fresh.size
        && edited.entry_type == fresh.entry_type
        && edited.speaker_id == fresh.speaker_id
        && edited.voice == fresh.voice;
    if !valid {
        return fail(format!(
            "{} entry #{}: immutable metadata or scr_msg was changed",
            fresh.file, fresh.index
        ));
    }
    Ok(())
}

fn verify_injected(
    rebuilt: &[u8],
    relative: &str,
    edited: &TranslationDocument,
    encoding: EncodingRoute,
) -> Result<()> {
    let container = ObjContainer::parse(rebuilt)?;
    let checked = analyze_container(&container, rebuilt, relative, encoding)?;
    if checked.entries.len() != edited.entries.len() {
        return fail(format!(
            "{relative}: rebuilt message count changed from {} to {}",
            edited.entries.len(),
            checked.entries.len()
        ));
    }
    for (actual, expected) in checked.entries.iter().zip(&edited.entries) {
        if actual.message != expected.message
            || actual.speaker_id != expected.speaker_id
            || actual.voice != expected.voice
        {
            return fail(format!(
                "{relative} entry #{} failed Unicode/context verification after injection",
                expected.index
            ));
        }
    }
    Ok(())
}

fn validate_manifest(manifest: &Manifest) -> Result<()> {
    if manifest.schema_version != SCHEMA_VERSION || manifest.format != FORMAT {
        return fail("unsupported or malformed AI5WIN OBJ manifest");
    }
    let mut sources = HashSet::new();
    let mut translations = HashSet::new();
    for file in &manifest.files {
        path_from_manifest(&file.source_file)?;
        path_from_manifest(&file.translation_file)?;
        if !matches!(file.source_storage.as_str(), "plain" | "lzss") {
            return fail(format!(
                "{}: unsupported source storage {:?}",
                file.source_file, file.source_storage
            ));
        }
        if !sources.insert(&file.source_file) || !translations.insert(&file.translation_file) {
            return fail("manifest contains duplicate source or translation paths");
        }
    }
    Ok(())
}

fn input_files(input: &Path) -> Result<(PathBuf, Vec<PathBuf>)> {
    if input.is_file() {
        let root = input
            .parent()
            .ok_or("input file has no parent directory")?
            .to_path_buf();
        return Ok((root, vec![input.to_path_buf()]));
    }
    let mut files = Vec::new();
    collect_files(input, &mut files)?;
    files.sort();
    Ok((input.to_path_buf(), files))
}

fn collect_files(root: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, output)?;
        } else if path.is_file() {
            output.push(path);
        }
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn relative_name(root: &Path, path: &Path) -> Result<String> {
    let relative = path.strip_prefix(root)?;
    let value = relative
        .to_str()
        .ok_or("non-Unicode paths are not supported in translation metadata")?;
    Ok(value.replace('\\', "/"))
}

fn path_from_manifest(value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        })
    {
        return fail(format!("unsafe relative path in manifest: {value:?}"));
    }
    Ok(path.to_path_buf())
}

fn source_member(root: &Path, relative: &str) -> Result<PathBuf> {
    if root.is_file() || (!root.exists() && root.extension().is_some()) {
        return Ok(root.to_path_buf());
    }
    Ok(root.join(path_from_manifest(relative)?))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut data = serde_json::to_vec_pretty(value)?;
    data.push(b'\n');
    fs::write(path, data)?;
    Ok(())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()).into())
}

fn absolute_existing(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).map_err(|error| format!("{}: {error}", path.display()).into())
}

fn check_destination(path: &Path, overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        return fail(format!(
            "output already exists: {}; pass --overwrite to replace it",
            path.display()
        ));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn commit_stage(stage: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if output.exists() {
        if !overwrite {
            return fail(format!(
                "output appeared while working: {}",
                output.display()
            ));
        }
        remove_path(output)?;
    }
    fs::rename(stage, output)?;
    Ok(())
}

fn remove_path(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn staging_path(output: &Path) -> Result<PathBuf> {
    let name = output
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("output path must have a Unicode file name")?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(output.with_file_name(format!(".{name}.ai5win-obj-stage-{stamp}")))
}

fn reject_nested_output(source: &Path, output: &Path) -> Result<()> {
    let absolute_output = if output.is_absolute() {
        output.to_path_buf()
    } else {
        std::env::current_dir()?.join(output)
    };
    if absolute_output == source {
        return fail("output must be different from every input path");
    }
    if source.is_dir() && absolute_output.starts_with(source) {
        return fail("output must not be inside the source directory");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obj::{lzss_compress_greedy, ObjContainer};

    fn expression_integer(value: u32) -> Vec<u8> {
        let mut result = vec![0x01];
        result.extend_from_slice(&value.to_be_bytes());
        result.push(0xFF);
        result
    }

    fn sample(message: &[u8]) -> Vec<u8> {
        let mut data = vec![0x04];
        data.extend_from_slice(&expression_integer(0));
        data.extend_from_slice(&11u32.to_be_bytes());
        data.push(0x03);
        data.extend_from_slice(&expression_integer(2));
        data.push(0x02);
        data.extend_from_slice(message);
        data.extend_from_slice(&[0, 0xFF]);
        data.extend_from_slice(&expression_integer(1));
        data.extend_from_slice(&[0, 0]);
        data
    }

    #[test]
    fn gbk_reencodes_unchanged_message() {
        let cp932_message = encode_text("高揚感", EncodingRoute::Cp932).unwrap();
        let decoded = sample(&cp932_message);
        let stored = lzss_compress_greedy(&decoded);
        let container = ObjContainer::parse(&stored).unwrap();
        let document =
            analyze_container(&container, &stored, "AE_MAHJONG.OBJ", EncodingRoute::Cp932).unwrap();
        let rebuilt =
            inject_document(&container, &document, &document, EncodingRoute::Gbk).unwrap();
        assert_ne!(rebuilt, decoded);
        let rebuilt_stored = container.store_rebuilt(rebuilt).unwrap();
        verify_injected(
            &rebuilt_stored,
            "AE_MAHJONG.OBJ",
            &document,
            EncodingRoute::Gbk,
        )
        .unwrap();
    }

    #[test]
    fn cp932_unchanged_preserves_stored_bytes() {
        let decoded = sample(&encode_text("テスト", EncodingRoute::Cp932).unwrap());
        let stored = lzss_compress_greedy(&decoded);
        let container = ObjContainer::parse(&stored).unwrap();
        let document =
            analyze_container(&container, &stored, "sample.OBJ", EncodingRoute::Cp932).unwrap();
        let rebuilt =
            inject_document(&container, &document, &document, EncodingRoute::Cp932).unwrap();
        assert_eq!(container.store_rebuilt(rebuilt).unwrap(), stored);
    }
}
