use crate::font::FontMap;
use crate::msg::{decode_editable_text_with, encode_plain_cp932_text, MsgDisplaySegment, MsgFile};
use crate::util::{
    commit_output_directory, prepare_output_directory, sha256_hex, validate_relative_path,
    write_output_file,
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const JSON_FORMAT: &str = "kuki-ayaka-msg-json-v1";
const WORKSPACE_FORMAT: &str = "kuki-ayaka-msg-workspace-v1";
const NAME_POLICY: &str = "0E-segments-leading-name-before-quote-v1";
pub const MSG_MANIFEST_NAME: &str = "msg_manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgTranslationFile {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_file")]
    pub file: PathBuf,
    #[serde(rename = "_source_sha256")]
    pub source_sha256: String,
    #[serde(rename = "_source_size")]
    pub source_size: usize,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_name_policy")]
    pub name_policy: String,
    pub entries: Vec<MsgTranslationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgTranslationEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    pub scr_name: Option<String>,
    pub scr_msg: String,
    pub message: String,
    #[serde(rename = "_file")]
    pub file: PathBuf,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_block_index")]
    pub block_index: usize,
    #[serde(rename = "_segment_index")]
    pub segment_index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MsgWorkspaceManifest {
    format: String,
    roles: MsgWorkspaceRoles,
    source: MsgWorkspaceSource,
    files: Vec<MsgWorkspaceFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MsgWorkspaceRoles {
    translation_root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MsgWorkspaceSource {
    original_path: PathBuf,
    kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MsgWorkspaceFile {
    source_file: PathBuf,
    translation_json: PathBuf,
    source_sha256: String,
    entries: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsgExtractReport {
    pub scanned_files: usize,
    pub json_files: usize,
    pub extracted_entries: usize,
    pub skipped_files: usize,
    pub warnings: usize,
    pub output: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsgInjectReport {
    pub json_files: usize,
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub output_files: usize,
    pub warnings: usize,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
struct PreparedTranslation {
    source_relative: PathBuf,
    json_relative: PathBuf,
    translation: MsgTranslationFile,
    json_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
struct InjectedFile {
    source_relative: PathBuf,
    bytes: Vec<u8>,
    entries: usize,
    patched: usize,
}

pub fn extract_msg(input: &Path, output: &Path, overwrite: bool) -> Result<MsgExtractReport> {
    if input.is_file() {
        extract_single(input, output, overwrite)
    } else if input.is_dir() {
        extract_directory(input, output, overwrite)
    } else {
        bail!(
            "MSG input does not exist or is unsupported: {}",
            input.display()
        );
    }
}

pub fn inject_msg(
    source: &Path,
    translation: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<MsgInjectReport> {
    inject_msg_with_font_map(source, translation, output, None, overwrite)
}

pub fn inject_msg_with_font_map(
    source: &Path,
    translation: &Path,
    output: &Path,
    font_map: Option<&FontMap>,
    overwrite: bool,
) -> Result<MsgInjectReport> {
    if source.is_file() && translation.is_file() {
        inject_single(source, translation, output, font_map, overwrite)
    } else if source.is_dir() && translation.is_dir() {
        inject_directory(source, translation, output, font_map, overwrite)
    } else {
        bail!(
            "source and translation must both be files or both be directories: source={}, translation={}",
            source.display(),
            translation.display()
        );
    }
}

pub fn build_translation(
    source_bytes: &[u8],
    source_relative: &Path,
) -> Result<MsgTranslationFile> {
    validate_relative_path(source_relative)?;
    let msg = MsgFile::parse(source_bytes)?;
    let mut entries = Vec::new();
    for block_index in 0..msg.blocks.len() {
        let block = &msg.blocks[block_index];
        for segment in msg.display_segments(block_index)? {
            if !segment.has_editable_text {
                continue;
            }
            let entry_type = if segment.name.is_some() {
                "dialogue"
            } else {
                "text"
            };
            entries.push(MsgTranslationEntry {
                name: segment.name.clone(),
                scr_name: segment.name.clone(),
                scr_msg: segment.scr_msg.clone(),
                message: segment.scr_msg,
                file: source_relative.to_path_buf(),
                index: entries.len(),
                block_index,
                segment_index: segment.segment_index,
                offset: usize::from(block.original_offset) + segment.block_relative_offset,
                size: segment.raw_size,
                entry_type: entry_type.to_owned(),
                encoding: "CP932".to_owned(),
            });
        }
    }
    if entries.is_empty() {
        bail!("MSG contains no editable text entries");
    }
    Ok(MsgTranslationFile {
        format: JSON_FORMAT.to_owned(),
        file: source_relative.to_path_buf(),
        source_sha256: sha256_hex(source_bytes),
        source_size: source_bytes.len(),
        encoding: "CP932".to_owned(),
        name_policy: NAME_POLICY.to_owned(),
        entries,
    })
}

pub fn inject_translation(
    source_bytes: &[u8],
    source_relative: &Path,
    translation: &MsgTranslationFile,
) -> Result<(Vec<u8>, usize, usize)> {
    inject_translation_with_font_map(source_bytes, source_relative, translation, None)
}

pub fn inject_translation_with_font_map(
    source_bytes: &[u8],
    source_relative: &Path,
    translation: &MsgTranslationFile,
    font_map: Option<&FontMap>,
) -> Result<(Vec<u8>, usize, usize)> {
    validate_translation_header(source_bytes, source_relative, translation)?;
    let mut msg = MsgFile::parse(source_bytes)?;
    let source_segments = collect_editable_segments(&msg)?;
    if source_segments.len() != translation.entries.len() {
        bail!(
            "JSON entry count {} does not match source entry count {} for {}",
            translation.entries.len(),
            source_segments.len(),
            source_relative.display()
        );
    }

    let mut edits = HashMap::new();
    let mut patched = 0usize;
    for (expected_index, (entry, source_segment)) in
        translation.entries.iter().zip(&source_segments).enumerate()
    {
        validate_entry(
            entry,
            expected_index,
            source_relative,
            source_segment,
            &msg,
            font_map,
        )?;
        if entry.message != entry.scr_msg || entry.name != entry.scr_name {
            patched += 1;
        }
        if edits
            .insert(
                (entry.block_index, entry.segment_index),
                (entry.name.as_deref(), entry.message.as_str()),
            )
            .is_some()
        {
            bail!(
                "duplicate JSON target block {} segment {}",
                entry.block_index,
                entry.segment_index
            );
        }
    }

    for block_index in 0..msg.blocks.len() {
        let source_block = msg.blocks[block_index].bytes.clone();
        let segments = msg.display_segments(block_index)?;
        let mut rebuilt_block = Vec::new();
        for (position, segment) in segments.iter().enumerate() {
            if position > 0 {
                rebuilt_block.push(0x0E);
            }
            if let Some((name, message)) = edits.get(&(block_index, segment.segment_index)) {
                let mut encoder = |plain: &str, role: &str| {
                    if let Some(font_map) = font_map {
                        font_map.encode_plain(plain, role)
                    } else {
                        encode_plain_cp932_text(plain, role)
                    }
                };
                rebuilt_block.extend_from_slice(&segment.rebuild_with_encoder(
                    *name,
                    message,
                    &mut encoder,
                )?);
            } else {
                let start = segment.block_relative_offset;
                rebuilt_block.extend_from_slice(&source_block[start..start + segment.raw_size]);
            }
        }
        msg.blocks[block_index].bytes = rebuilt_block;
    }

    let rebuilt = msg.rebuild()?;
    if patched == 0 && rebuilt != source_bytes {
        bail!("unchanged MSG injection was not byte-exact");
    }
    verify_injected_translation(
        &rebuilt,
        source_relative,
        translation,
        &source_segments,
        font_map,
    )?;
    Ok((rebuilt, patched, translation.entries.len() - patched))
}

fn extract_single(input: &Path, output: &Path, overwrite: bool) -> Result<MsgExtractReport> {
    reject_same_path(
        input,
        output,
        "MSG extract output must not overwrite its source",
    )?;
    let source_bytes =
        fs::read(input).with_context(|| format!("failed to read MSG {}", input.display()))?;
    let source_relative = PathBuf::from(
        input
            .file_name()
            .context("MSG input path has no filename")?,
    );
    let translation = build_translation(&source_bytes, &source_relative)
        .with_context(|| format!("failed to extract MSG {}", input.display()))?;
    let json = serialize_json(&translation)?;
    write_output_file(output, &json, overwrite)?;
    Ok(MsgExtractReport {
        scanned_files: 1,
        json_files: 1,
        extracted_entries: translation.entries.len(),
        skipped_files: 0,
        warnings: 0,
        output: output.to_path_buf(),
    })
}

fn extract_directory(input: &Path, output: &Path, overwrite: bool) -> Result<MsgExtractReport> {
    reject_output_inside_source(input, output)?;
    let source_files = collect_regular_files(input)?;
    let scanned_files = source_files.len();
    let mut prepared = Vec::new();
    let mut warnings = 0usize;
    for path in source_files {
        let relative = path
            .strip_prefix(input)
            .expect("collected path remains under source root")
            .to_path_buf();
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                eprintln!(
                    "[warning] unable to read candidate {}; skipped: {}",
                    path.display(),
                    error
                );
                warnings += 1;
                continue;
            }
        };
        let translation = match build_translation(&bytes, &relative) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let json_relative = append_json_extension(&relative)?;
        let json_bytes = serialize_json(&translation)?;
        prepared.push(PreparedTranslation {
            source_relative: relative,
            json_relative,
            translation,
            json_bytes,
        });
    }
    if prepared.is_empty() {
        bail!(
            "directory contains no structurally valid MSG files: {}",
            input.display()
        );
    }

    let manifest = MsgWorkspaceManifest {
        format: WORKSPACE_FORMAT.to_owned(),
        roles: MsgWorkspaceRoles {
            translation_root: PathBuf::from("."),
        },
        source: MsgWorkspaceSource {
            original_path: input.to_path_buf(),
            kind: "directory".to_owned(),
        },
        files: prepared
            .iter()
            .map(|item| MsgWorkspaceFile {
                source_file: item.source_relative.clone(),
                translation_json: item.json_relative.clone(),
                source_sha256: item.translation.source_sha256.clone(),
                entries: item.translation.entries.len(),
            })
            .collect(),
    };
    let manifest_json = serialize_json(&manifest)?;
    let temporary = prepare_output_directory(output, overwrite)?;
    let result = (|| -> Result<()> {
        for item in &prepared {
            let destination = temporary.join(&item.json_relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            fs::write(&destination, &item.json_bytes)
                .with_context(|| format!("failed to write {}", destination.display()))?;
        }
        let manifest_path = temporary.join(MSG_MANIFEST_NAME);
        fs::write(&manifest_path, manifest_json)
            .with_context(|| format!("failed to write {}", manifest_path.display()))?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&temporary);
        return Err(error);
    }
    commit_output_directory(&temporary, output, overwrite)?;
    Ok(MsgExtractReport {
        scanned_files,
        json_files: prepared.len(),
        extracted_entries: prepared
            .iter()
            .map(|item| item.translation.entries.len())
            .sum(),
        skipped_files: scanned_files - prepared.len(),
        warnings,
        output: output.to_path_buf(),
    })
}

fn inject_single(
    source: &Path,
    translation_path: &Path,
    output: &Path,
    font_map: Option<&FontMap>,
    overwrite: bool,
) -> Result<MsgInjectReport> {
    reject_same_path(
        source,
        output,
        "MSG inject output must not overwrite its source",
    )?;
    let source_bytes =
        fs::read(source).with_context(|| format!("failed to read MSG {}", source.display()))?;
    let translation = read_translation(translation_path)?;
    let source_relative = PathBuf::from(
        source
            .file_name()
            .context("MSG source path has no filename")?,
    );
    let (rebuilt, patched, unchanged) =
        inject_translation_with_font_map(&source_bytes, &source_relative, &translation, font_map)?;
    write_output_file(output, &rebuilt, overwrite)?;
    Ok(MsgInjectReport {
        json_files: 1,
        json_entries: translation.entries.len(),
        patched,
        unchanged,
        output_files: 1,
        warnings: 0,
        output: output.to_path_buf(),
    })
}

fn inject_directory(
    source: &Path,
    translation_root: &Path,
    output: &Path,
    font_map: Option<&FontMap>,
    overwrite: bool,
) -> Result<MsgInjectReport> {
    reject_output_inside_source(source, output)?;
    let translations = collect_translations(translation_root)?;
    let mut seen_sources = HashSet::new();
    let mut prepared = Vec::with_capacity(translations.len());
    for translation in translations {
        validate_relative_path(&translation.file)?;
        if !seen_sources.insert(translation.file.clone()) {
            bail!("multiple JSON files target {}", translation.file.display());
        }
        let source_path = source.join(&translation.file);
        if !source_path.is_file() {
            bail!("JSON source file is missing: {}", source_path.display());
        }
        let source_bytes = fs::read(&source_path)
            .with_context(|| format!("failed to read MSG source {}", source_path.display()))?;
        let (bytes, patched, _) = inject_translation_with_font_map(
            &source_bytes,
            &translation.file,
            &translation,
            font_map,
        )?;
        prepared.push(InjectedFile {
            source_relative: translation.file.clone(),
            bytes,
            entries: translation.entries.len(),
            patched,
        });
    }
    if prepared.is_empty() {
        bail!("translation directory contains no MSG translation JSON files");
    }

    let source_file_count = collect_regular_files(source)?.len();
    let temporary = prepare_output_directory(output, overwrite)?;
    let result = (|| -> Result<()> {
        copy_directory_contents(source, &temporary)?;
        for item in &prepared {
            let destination = temporary.join(&item.source_relative);
            fs::write(&destination, &item.bytes)
                .with_context(|| format!("failed to write {}", destination.display()))?;
        }
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&temporary);
        return Err(error);
    }
    commit_output_directory(&temporary, output, overwrite)?;
    let json_entries = prepared.iter().map(|item| item.entries).sum::<usize>();
    let patched = prepared.iter().map(|item| item.patched).sum::<usize>();
    Ok(MsgInjectReport {
        json_files: prepared.len(),
        json_entries,
        patched,
        unchanged: json_entries - patched,
        output_files: source_file_count,
        warnings: 0,
        output: output.to_path_buf(),
    })
}

fn validate_translation_header(
    source_bytes: &[u8],
    source_relative: &Path,
    translation: &MsgTranslationFile,
) -> Result<()> {
    if translation.format != JSON_FORMAT {
        bail!(
            "unsupported MSG JSON format {:?}; expected {:?}",
            translation.format,
            JSON_FORMAT
        );
    }
    if translation.name_policy != NAME_POLICY {
        bail!("MSG JSON name policy does not match this tool");
    }
    if translation.encoding != "CP932" {
        bail!("MSG JSON encoding must be CP932");
    }
    validate_relative_path(&translation.file)?;
    if translation.file != source_relative {
        bail!(
            "MSG JSON targets {}, but source role is {}",
            translation.file.display(),
            source_relative.display()
        );
    }
    if translation.source_size != source_bytes.len() {
        bail!(
            "MSG source size mismatch for {}: JSON {}, actual {}",
            source_relative.display(),
            translation.source_size,
            source_bytes.len()
        );
    }
    let hash = sha256_hex(source_bytes);
    if translation.source_sha256 != hash {
        bail!(
            "MSG source SHA-256 mismatch for {}: JSON {}, actual {}",
            source_relative.display(),
            translation.source_sha256,
            hash
        );
    }
    Ok(())
}

fn validate_entry(
    entry: &MsgTranslationEntry,
    expected_index: usize,
    source_relative: &Path,
    source_segment: &MsgDisplaySegment,
    msg: &MsgFile,
    font_map: Option<&FontMap>,
) -> Result<()> {
    let block = &msg.blocks[source_segment.block_index];
    let expected_offset = usize::from(block.original_offset) + source_segment.block_relative_offset;
    let expected_type = if source_segment.name.is_some() {
        "dialogue"
    } else {
        "text"
    };
    if entry.file != source_relative
        || entry.index != expected_index
        || entry.block_index != source_segment.block_index
        || entry.segment_index != source_segment.segment_index
        || entry.offset != expected_offset
        || entry.size != source_segment.raw_size
        || entry.entry_type != expected_type
        || entry.encoding != "CP932"
    {
        bail!(
            "MSG JSON metadata mismatch at {} entry {}",
            source_relative.display(),
            expected_index
        );
    }
    if entry.scr_msg != source_segment.scr_msg {
        bail!(
            "scr_msg was modified or no longer matches source at {} entry {}",
            source_relative.display(),
            expected_index
        );
    }
    match &source_segment.name {
        Some(source_name) => {
            if entry.scr_name.as_ref() != Some(source_name) {
                bail!(
                    "_scr_name was modified or no longer matches source at {} entry {}",
                    source_relative.display(),
                    expected_index
                );
            }
            if entry.name.is_none() {
                bail!(
                    "name is missing at {} entry {}",
                    source_relative.display(),
                    expected_index
                );
            }
        }
        None => {
            if entry.scr_name.is_some() || entry.name.is_some() {
                bail!(
                    "name fields are not writable for unnamed {} entry {}",
                    source_relative.display(),
                    expected_index
                );
            }
        }
    }
    let mut encoder = |plain: &str, role: &str| {
        if let Some(font_map) = font_map {
            font_map.encode_plain(plain, role)
        } else {
            encode_plain_cp932_text(plain, role)
        }
    };
    decode_editable_text_with(&entry.message, &mut encoder).with_context(|| {
        format!(
            "invalid message at {} entry {}",
            source_relative.display(),
            expected_index
        )
    })?;
    Ok(())
}

fn verify_injected_translation(
    rebuilt: &[u8],
    source_relative: &Path,
    translation: &MsgTranslationFile,
    source_segments: &[MsgDisplaySegment],
    font_map: Option<&FontMap>,
) -> Result<()> {
    let reparsed = MsgFile::parse(rebuilt).context("rebuilt MSG failed structural verification")?;
    if source_segments.len() != translation.entries.len() {
        bail!("source MSG editable entry count changed during verification");
    }
    for (index, (source_segment, entry)) in
        source_segments.iter().zip(&translation.entries).enumerate()
    {
        let mut encoder = |plain: &str, role: &str| {
            if let Some(font_map) = font_map {
                font_map.encode_plain(plain, role)
            } else {
                encode_plain_cp932_text(plain, role)
            }
        };
        let expected = source_segment.rebuild_with_encoder(
            entry.name.as_deref(),
            &entry.message,
            &mut encoder,
        )?;
        let actual_segments = reparsed.display_segments(source_segment.block_index)?;
        let actual_segment = actual_segments
            .get(source_segment.segment_index)
            .with_context(|| {
                format!(
                    "rebuilt MSG is missing block {} segment {}",
                    source_segment.block_index, source_segment.segment_index
                )
            })?;
        let start = actual_segment.block_relative_offset;
        let end = start + actual_segment.raw_size;
        let actual = &reparsed.blocks[source_segment.block_index].bytes[start..end];
        if actual != expected {
            bail!(
                "rebuilt MSG segment mismatch at {} entry {}",
                source_relative.display(),
                index
            );
        }
    }
    Ok(())
}

fn collect_editable_segments(msg: &MsgFile) -> Result<Vec<MsgDisplaySegment>> {
    let mut output = Vec::new();
    for block_index in 0..msg.blocks.len() {
        output.extend(
            msg.display_segments(block_index)?
                .into_iter()
                .filter(|segment| segment.has_editable_text),
        );
    }
    Ok(output)
}

fn collect_translations(root: &Path) -> Result<Vec<MsgTranslationFile>> {
    let manifest_path = root.join(MSG_MANIFEST_NAME);
    let paths = if manifest_path.is_file() {
        let bytes = fs::read(&manifest_path)
            .with_context(|| format!("failed to read {}", manifest_path.display()))?;
        let manifest: MsgWorkspaceManifest = serde_json::from_slice(&bytes)
            .with_context(|| format!("invalid workspace manifest {}", manifest_path.display()))?;
        if manifest.format != WORKSPACE_FORMAT {
            bail!("unsupported MSG workspace manifest format");
        }
        if manifest.roles.translation_root != Path::new(".") {
            validate_relative_path(&manifest.roles.translation_root)?;
        }
        let translation_root = root.join(&manifest.roles.translation_root);
        manifest
            .files
            .into_iter()
            .map(|file| {
                validate_relative_path(&file.translation_json)?;
                Ok(translation_root.join(file.translation_json))
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        collect_regular_files(root)?
    };

    let mut translations = Vec::new();
    for path in paths {
        if path == manifest_path {
            continue;
        }
        let bytes = fs::read(&path)
            .with_context(|| format!("failed to read translation candidate {}", path.display()))?;
        let value: serde_json::Value = match serde_json::from_slice(&bytes) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if value.get("_format").and_then(serde_json::Value::as_str) != Some(JSON_FORMAT) {
            continue;
        }
        translations.push(
            serde_json::from_value(value)
                .with_context(|| format!("invalid MSG translation JSON {}", path.display()))?,
        );
    }
    translations.sort_by(|left: &MsgTranslationFile, right: &MsgTranslationFile| {
        left.file
            .to_string_lossy()
            .cmp(&right.file.to_string_lossy())
    });
    Ok(translations)
}

fn read_translation(path: &Path) -> Result<MsgTranslationFile> {
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read translation JSON {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid MSG translation JSON {}", path.display()))
}

fn serialize_json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value).context("failed to serialize JSON")?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn append_json_extension(path: &Path) -> Result<PathBuf> {
    let name = path
        .file_name()
        .context("source relative path has no filename")?;
    let mut output_name = name.to_os_string();
    output_name.push(".json");
    Ok(path.with_file_name(output_name))
}

fn collect_regular_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_regular_files_inner(root, &mut files)?;
    files.sort_by(|left, right| left.to_string_lossy().cmp(&right.to_string_lossy()));
    Ok(files)
}

fn collect_regular_files_inner(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)
        .with_context(|| format!("failed to read directory {}", root.display()))?
    {
        let entry = entry.with_context(|| format!("failed to enumerate {}", root.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", entry.path().display()))?;
        if file_type.is_symlink() {
            bail!(
                "symbolic links are not supported: {}",
                entry.path().display()
            );
        }
        if file_type.is_dir() {
            collect_regular_files_inner(&entry.path(), files)?;
        } else if file_type.is_file() {
            files.push(entry.path());
        }
    }
    Ok(())
}

fn copy_directory_contents(source: &Path, destination: &Path) -> Result<()> {
    for entry in fs::read_dir(source)
        .with_context(|| format!("failed to read directory {}", source.display()))?
    {
        let entry = entry.with_context(|| format!("failed to enumerate {}", source.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", entry.path().display()))?;
        if file_type.is_symlink() {
            bail!(
                "symbolic links are not supported: {}",
                entry.path().display()
            );
        }
        let target = destination.join(entry.file_name());
        if file_type.is_dir() {
            fs::create_dir(&target)
                .with_context(|| format!("failed to create {}", target.display()))?;
            copy_directory_contents(&entry.path(), &target)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &target).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    entry.path().display(),
                    target.display()
                )
            })?;
        }
    }
    Ok(())
}

fn reject_same_path(source: &Path, output: &Path, message: &str) -> Result<()> {
    if normalize_for_compare(source)? == normalize_for_compare(output)? {
        bail!("{message}");
    }
    Ok(())
}

fn reject_output_inside_source(source: &Path, output: &Path) -> Result<()> {
    let source = normalize_for_compare(source)?;
    let output = normalize_for_compare(output)?;
    if output == source || output.starts_with(&source) {
        bail!("output directory must not be the source directory or be inside it");
    }
    Ok(())
}

fn normalize_for_compare(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return path
            .canonicalize()
            .with_context(|| format!("failed to resolve {}", path.display()));
    }
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = parent
        .canonicalize()
        .with_context(|| format!("failed to resolve parent {}", parent.display()))?;
    let name = path.file_name().context("path has no final component")?;
    Ok(parent.join(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{FontMapEntry, FONT_MAP_FORMAT, KANJI1_GLYPH_BYTES, KANJI1_SIZE};

    fn fixture() -> Vec<u8> {
        let (dialogue, _, errors) = encoding_rs::SHIFT_JIS.encode("妖華  「~久しぶりね。」");
        assert!(!errors);
        let (continuation, _, errors) = encoding_rs::SHIFT_JIS.encode("        「~続き。」");
        assert!(!errors);
        let mut block = vec![0xFE, b' ', b' '];
        block.extend_from_slice(&dialogue);
        block.push(0x0E);
        block.extend_from_slice(&continuation);
        let mut bytes = vec![0x02, 0x00];
        bytes.extend_from_slice(&block);
        bytes.push(0);
        bytes
    }

    #[test]
    fn extracts_dialogue_and_unnamed_continuation() {
        let bytes = fixture();
        let translation = build_translation(&bytes, Path::new("arbitrary.zil")).unwrap();
        assert_eq!(translation.entries.len(), 2);
        assert_eq!(translation.entries[0].name.as_deref(), Some("妖華"));
        assert_eq!(translation.entries[0].scr_msg, "「~久しぶりね。」");
        assert_eq!(translation.entries[1].name, None);
        assert_eq!(translation.entries[1].scr_msg, "「~続き。」");
    }

    #[test]
    fn unchanged_injection_is_byte_exact() {
        let bytes = fixture();
        let translation = build_translation(&bytes, Path::new("arbitrary.zil")).unwrap();
        let (rebuilt, patched, unchanged) =
            inject_translation(&bytes, Path::new("arbitrary.zil"), &translation).unwrap();
        assert_eq!(rebuilt, bytes);
        assert_eq!(patched, 0);
        assert_eq!(unchanged, 2);
    }

    #[test]
    fn injects_changed_name_message_and_control() {
        let bytes = fixture();
        let mut translation = build_translation(&bytes, Path::new("arbitrary.zil")).unwrap();
        translation.entries[0].name = Some("清美".to_owned());
        translation.entries[0].message = "「変更[13]。」".to_owned();
        let (rebuilt, patched, _) =
            inject_translation(&bytes, Path::new("arbitrary.zil"), &translation).unwrap();
        assert_eq!(patched, 1);
        let reparsed = MsgFile::parse(&rebuilt).unwrap();
        let segments = reparsed.display_segments(0).unwrap();
        assert_eq!(segments[0].name.as_deref(), Some("清美"));
        assert_eq!(segments[0].scr_msg, "「変更[13]。」");
    }

    #[test]
    fn verifies_mapped_name_by_encoded_carrier_bytes() {
        let bytes = fixture();
        let mut translation = build_translation(&bytes, Path::new("arbitrary.zil")).unwrap();
        translation.entries[0].name = Some("妖华".to_owned());
        let font_map = FontMap {
            format: FONT_MAP_FORMAT.to_owned(),
            version: 2,
            mapping_id: "test".to_owned(),
            source_rom_sha256: String::new(),
            ttf_sha256: String::new(),
            rom_size: KANJI1_SIZE,
            carrier_policy: String::new(),
            entries: vec![FontMapEntry {
                character: "华".to_owned(),
                carrier: "亜".to_owned(),
                carrier_cp932: "889F".to_owned(),
                jis_ku: 16,
                jis_ten: 1,
                slot: 0x601,
                rom_offset: 0x601 * KANJI1_GLYPH_BYTES,
                kind: "substitute".to_owned(),
                usage_count: 1,
            }],
        };

        let (rebuilt, patched, unchanged) = inject_translation_with_font_map(
            &bytes,
            Path::new("arbitrary.zil"),
            &translation,
            Some(&font_map),
        )
        .unwrap();
        assert_eq!(patched, 1);
        assert_eq!(unchanged, 1);

        let reparsed = MsgFile::parse(&rebuilt).unwrap();
        let segments = reparsed.display_segments(0).unwrap();
        assert_eq!(segments[0].name.as_deref(), Some("妖亜"));
    }

    #[test]
    fn verifies_changed_unspaced_name_from_source_structure() {
        let (dialogue, _, errors) = encoding_rs::SHIFT_JIS.encode("いつみ「~久しぶりね。」");
        assert!(!errors);
        let mut bytes = vec![0x02, 0x00];
        bytes.extend_from_slice(&dialogue);
        bytes.push(0);
        let mut translation = build_translation(&bytes, Path::new("arbitrary.zil")).unwrap();
        translation.entries[0].name = Some("逸美".to_owned());

        let (rebuilt, patched, unchanged) =
            inject_translation(&bytes, Path::new("arbitrary.zil"), &translation).unwrap();
        assert_eq!(patched, 1);
        assert_eq!(unchanged, 0);

        let reparsed = MsgFile::parse(&rebuilt).unwrap();
        let segments = reparsed.display_segments(0).unwrap();
        assert_eq!(segments[0].name, None);
        assert_eq!(segments[0].scr_msg, "逸美「~久しぶりね。」");
    }

    #[test]
    fn rejects_modified_source_fields_and_structural_controls() {
        let bytes = fixture();
        let mut translation = build_translation(&bytes, Path::new("arbitrary.zil")).unwrap();
        translation.entries[0].scr_msg.push('x');
        assert!(inject_translation(&bytes, Path::new("arbitrary.zil"), &translation).is_err());

        let mut translation = build_translation(&bytes, Path::new("arbitrary.zil")).unwrap();
        translation.entries[0].message = "bad[0E]split".to_owned();
        assert!(inject_translation(&bytes, Path::new("arbitrary.zil"), &translation).is_err());

        let mut translation = build_translation(&bytes, Path::new("arbitrary.zil")).unwrap();
        translation.entries[0].message = "bad[0G]token".to_owned();
        assert!(inject_translation(&bytes, Path::new("arbitrary.zil"), &translation).is_err());

        let mut translation = build_translation(&bytes, Path::new("arbitrary.zil")).unwrap();
        translation.entries[0].scr_name = Some("清美".to_owned());
        assert!(inject_translation(&bytes, Path::new("arbitrary.zil"), &translation).is_err());
    }

    #[test]
    fn rejects_unencodable_characters() {
        let bytes = fixture();
        let mut translation = build_translation(&bytes, Path::new("arbitrary.zil")).unwrap();
        translation.entries[0].message = "emoji 😀".to_owned();
        let error = format!(
            "{:#}",
            inject_translation(&bytes, Path::new("arbitrary.zil"), &translation).unwrap_err()
        );
        assert!(error.contains("not encodable as CP932"));
    }
}
