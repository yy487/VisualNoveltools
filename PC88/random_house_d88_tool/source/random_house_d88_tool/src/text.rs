use crate::basic::{BasicProgram, BasicQuotedString};
use crate::d88::D88Image;
use crate::font::{FontMap, TRANSLATION_FORMAT};
use crate::pc88::native_glyph_for_byte;
use crate::renderer::resident_renderer;
use anyhow::{bail, ensure, Context, Result};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const TRANSLATION_VERSION: u32 = 1;
const BASIC_ATTRIBUTE: u8 = 0x80;
const PRINT_TOKEN: u8 = 0x91;
const DATA_TOKEN: u8 = 0x84;
const SYMBOL_TOKEN: [u8; 2] = [0xFF, 0xDF];
const Q_ASSIGN: [u8; 3] = [b'Q', b'$', 0xF1];
const SC_ASSIGN: [u8; 4] = [b'S', b'C', b'$', 0xF1];
const MESSAGE_ASSIGN: [u8; 5] = [b'M', b'E', b'S', b'$', 0xF1];
const MAX_RENDER_CELLS: usize = 80;
const EXPECTED_CLS_PATCHES: usize = 146;
const EXPECTED_NATIVE_ASCII_ECHO_PATCHES: usize = 70;
const EXPECTED_TRANSITION_CLEAR_PATCHES: usize = 1;
const ME1_PATIENT_LINE: u16 = 140;
const ME1_PATIENT_LINE_PREFIX: [u8; 7] = [0xCB, b' ', 0x17, b':', b'Q', b'$', 0xF1];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextTranslation {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_version")]
    pub version: u32,
    #[serde(rename = "_source_sha256")]
    pub source_sha256: String,
    #[serde(rename = "_source_size")]
    pub source_size: usize,
    pub entries: Vec<TextEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEntry {
    pub scr_msg: String,
    pub message: String,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_file_index")]
    pub file_index: usize,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_line")]
    pub line: u16,
    #[serde(rename = "_line_index")]
    pub line_index: usize,
    #[serde(rename = "_statement_offset")]
    pub statement_offset: usize,
    #[serde(rename = "_body_offset")]
    pub body_offset: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub kind: String,
    #[serde(rename = "_terminated")]
    pub terminated: bool,
    #[serde(rename = "_raw_sha256")]
    pub raw_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextExtractReport {
    pub basic_files: usize,
    pub entries: usize,
    pub skipped_nontext: usize,
    pub output: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInjectReport {
    pub basic_files: usize,
    pub entries: usize,
    pub patched_entries: usize,
    pub unchanged_entries: usize,
    pub cls_patches: usize,
    pub transition_clear_patches: usize,
    pub native_ascii_echo_patches: usize,
    pub changed_files: usize,
    pub comment_bytes_removed: usize,
    pub ignored_space_bytes_removed: usize,
    pub output_bytes: usize,
    pub output_sha256: String,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
struct Candidate {
    file: String,
    file_index: usize,
    line: u16,
    line_index: usize,
    statement_offset: usize,
    body_offset: usize,
    offset: usize,
    terminated: bool,
    kind: &'static str,
    raw: Vec<u8>,
    text: String,
}

pub fn extract_text(source: &Path, output: &Path, overwrite: bool) -> Result<TextExtractReport> {
    ensure_distinct_paths(source, output)?;
    let source_bytes =
        fs::read(source).with_context(|| format!("failed to read D88 {}", source.display()))?;
    let image = D88Image::parse(source_bytes.clone()).context("failed to parse source D88")?;
    let (candidates, skipped_nontext) = collect_candidates(&image)?;
    let entries = candidates
        .into_iter()
        .enumerate()
        .map(|(index, candidate)| TextEntry {
            scr_msg: candidate.text.clone(),
            message: candidate.text,
            file: candidate.file,
            file_index: candidate.file_index,
            index,
            line: candidate.line,
            line_index: candidate.line_index,
            statement_offset: candidate.statement_offset,
            body_offset: candidate.body_offset,
            offset: candidate.offset,
            size: candidate.raw.len(),
            kind: candidate.kind.to_owned(),
            terminated: candidate.terminated,
            raw_sha256: sha256_hex(&candidate.raw),
        })
        .collect::<Vec<_>>();
    let translation = TextTranslation {
        format: TRANSLATION_FORMAT.to_owned(),
        version: TRANSLATION_VERSION,
        source_sha256: sha256_hex(&source_bytes),
        source_size: source_bytes.len(),
        entries,
    };
    let mut bytes =
        serde_json::to_vec_pretty(&translation).context("failed to serialize translation JSON")?;
    bytes.push(b'\n');
    write_output_file(output, &bytes, overwrite)?;
    Ok(TextExtractReport {
        basic_files: image
            .files
            .iter()
            .filter(|file| file.attribute == BASIC_ATTRIBUTE)
            .count(),
        entries: translation.entries.len(),
        skipped_nontext,
        output: output.to_path_buf(),
    })
}

pub fn inject_text(
    source: &Path,
    translation_path: &Path,
    font_map_path: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<TextInjectReport> {
    ensure_distinct_paths(source, output)?;
    let source_bytes =
        fs::read(source).with_context(|| format!("failed to read D88 {}", source.display()))?;
    let translation_bytes = fs::read(translation_path).with_context(|| {
        format!(
            "failed to read translation JSON {}",
            translation_path.display()
        )
    })?;
    let translation: TextTranslation = serde_json::from_slice(&translation_bytes)
        .with_context(|| format!("invalid translation JSON {}", translation_path.display()))?;
    ensure!(
        translation.format == TRANSLATION_FORMAT && translation.version == TRANSLATION_VERSION,
        "unsupported translation format {:?} version {}",
        translation.format,
        translation.version
    );
    ensure!(
        translation.source_size == source_bytes.len(),
        "translation source size does not match input D88"
    );
    ensure!(
        translation.source_sha256 == sha256_hex(&source_bytes),
        "translation source SHA-256 does not match input D88"
    );
    let image = D88Image::parse(source_bytes).context("failed to parse source D88")?;
    let (candidates, _) = collect_candidates(&image)?;
    ensure!(
        translation.entries.len() == candidates.len(),
        "translation entry count {} does not match source candidate count {}",
        translation.entries.len(),
        candidates.len()
    );
    let font_map = FontMap::load(font_map_path)?;
    let mut seen_indices = HashSet::new();
    let mut encoded = Vec::with_capacity(candidates.len());
    let mut patched_entries = 0usize;
    let mut unchanged_entries = 0usize;
    for (expected_index, (entry, candidate)) in translation
        .entries
        .iter()
        .zip(candidates.iter())
        .enumerate()
    {
        ensure!(
            seen_indices.insert(entry.index),
            "duplicate translation _index {}",
            entry.index
        );
        validate_entry(entry, candidate, expected_index)?;
        if entry.message == entry.scr_msg {
            unchanged_entries += 1;
            encoded.push(candidate.raw.clone());
            continue;
        }
        ensure!(
            !entry.message.contains('"'),
            "entry {} message contains a double quote that cannot be embedded in N88 BASIC",
            entry.index
        );
        let cells = entry.message.chars().count();
        ensure!(
            cells <= MAX_RENDER_CELLS,
            "entry {} message uses {cells} screen cells; current renderer limit is {MAX_RENDER_CELLS}",
            entry.index
        );
        encoded.push(
            font_map.encode_plain(&entry.message, &format!("entry {} message", entry.index))?,
        );
        patched_entries += 1;
    }

    let renderer = resident_renderer()?;
    let mut replacements = HashMap::new();
    let mut allocation_details = Vec::new();
    let mut comment_bytes_removed = 0usize;
    let mut ignored_space_bytes_removed = 0usize;
    let disk_has_no_free_clusters = image.free_cluster_count() == 0;
    let mut basic_files = 0usize;
    let mut cls_patches = 0usize;
    let mut transition_clear_patches = 0usize;
    let mut native_ascii_echo_patches = 0usize;
    for file in &image.files {
        if file.attribute != BASIC_ATTRIBUTE {
            continue;
        }
        basic_files += 1;
        let mut program = BasicProgram::parse(&file.data)
            .with_context(|| format!("failed to parse BASIC file {}", file.name))?;
        let mut file_entries = candidates
            .iter()
            .enumerate()
            .filter(|(_, candidate)| candidate.file_index == file.index)
            .collect::<Vec<_>>();
        file_entries.sort_by(|left, right| {
            right
                .1
                .line_index
                .cmp(&left.1.line_index)
                .then_with(|| right.1.body_offset.cmp(&left.1.body_offset))
        });
        for (entry_index, candidate) in file_entries {
            if encoded[entry_index] == candidate.raw {
                continue;
            }
            program.replace_quoted_string(
                candidate.line_index,
                candidate.body_offset,
                &candidate.raw,
                &encoded[entry_index],
                candidate.terminated,
            )?;
        }
        if file.name.eq_ignore_ascii_case("ME 1") {
            program
                .instrument_page_clear_before_line(ME1_PATIENT_LINE, &ME1_PATIENT_LINE_PREFIX)
                .context("failed to insert the ME 1 patient-screen transition clear")?;
            transition_clear_patches += 1;
        }
        native_ascii_echo_patches += program.instrument_native_ascii_echo().with_context(|| {
            format!(
                "failed to instrument native ASCII input echoes in {}",
                file.name
            )
        })?;
        cls_patches += program
            .instrument_cls()
            .with_context(|| format!("failed to instrument CLS statements in {}", file.name))?;
        program
            .patch_memory_ceiling()
            .with_context(|| format!("failed to reserve renderer RAM in {}", file.name))?;
        if disk_has_no_free_clusters && !file.name.eq_ignore_ascii_case("TITL") {
            let capacity = file.clusters.len() * 8 * 256;
            let (comments, spaces) = compact_program_to_capacity(&mut program, capacity)?;
            comment_bytes_removed += comments;
            ignored_space_bytes_removed += spaces;
        }
        let (bytes, program_bytes) = if file.name.eq_ignore_ascii_case("TITL") {
            program
                .install_bootstrap(&renderer, None)
                .context("failed to install resident renderer bootstrap")?;
            let capacity = file.clusters.len() * 8 * 256;
            if disk_has_no_free_clusters {
                let (comments, spaces) = compact_program_to_capacity(&mut program, capacity)?;
                comment_bytes_removed += comments;
                ignored_space_bytes_removed += spaces;
            }
            let unpadded = program.compact_unpadded_rebuild()?;
            let program_bytes = unpadded.len();
            let mut bytes = unpadded;
            bytes.resize(bytes.len().div_ceil(256) * 256, 0);
            (bytes, program_bytes)
        } else {
            let unpadded = program.compact_unpadded_rebuild()?;
            let program_bytes = unpadded.len();
            let mut bytes = unpadded;
            bytes.resize(bytes.len().div_ceil(256) * 256, 0);
            (bytes, program_bytes)
        };
        allocation_details.push(format!(
            "{}:program={} allocated={} original={}",
            file.name,
            program_bytes,
            bytes.len(),
            file.data.len()
        ));
        replacements.insert(file.name.clone(), bytes);
    }
    ensure!(
        cls_patches == EXPECTED_CLS_PATCHES,
        "expected {EXPECTED_CLS_PATCHES} executable CLS statements, found {cls_patches}"
    );
    ensure!(
        native_ascii_echo_patches == EXPECTED_NATIVE_ASCII_ECHO_PATCHES,
        "expected {EXPECTED_NATIVE_ASCII_ECHO_PATCHES} native ASCII echo statements, found {native_ascii_echo_patches}"
    );
    ensure!(
        transition_clear_patches == EXPECTED_TRANSITION_CLEAR_PATCHES,
        "expected {EXPECTED_TRANSITION_CLEAR_PATCHES} profiled transition clear, found {transition_clear_patches}"
    );
    let rebuilt = image.rebuild(&replacements).with_context(|| {
        format!(
            "translated BASIC allocation plan: {}",
            allocation_details.join("; ")
        )
    })?;
    let output_sha256 = sha256_hex(&rebuilt.bytes);
    write_output_file(output, &rebuilt.bytes, overwrite)?;
    Ok(TextInjectReport {
        basic_files,
        entries: candidates.len(),
        patched_entries,
        unchanged_entries,
        cls_patches,
        transition_clear_patches,
        native_ascii_echo_patches,
        changed_files: rebuilt.changed_files,
        comment_bytes_removed,
        ignored_space_bytes_removed,
        output_bytes: rebuilt.bytes.len(),
        output_sha256,
        output: output.to_path_buf(),
    })
}

fn compact_program_to_capacity(
    program: &mut BasicProgram,
    capacity: usize,
) -> Result<(usize, usize)> {
    let mut size = program.compact_unpadded_rebuild()?.len();
    if size <= capacity {
        return Ok((0, 0));
    }
    let comments = program.trim_rem_comment_bytes(size - capacity);
    size = program.compact_unpadded_rebuild()?.len();
    let spaces = if size > capacity {
        program.trim_ignored_space_bytes(size - capacity)
    } else {
        0
    };
    Ok((comments, spaces))
}

fn collect_candidates(image: &D88Image) -> Result<(Vec<Candidate>, usize)> {
    let mut candidates = Vec::new();
    let mut skipped_nontext = 0usize;
    for file in &image.files {
        if file.attribute != BASIC_ATTRIBUTE {
            continue;
        }
        let program = BasicProgram::parse(&file.data)
            .with_context(|| format!("failed to parse BASIC file {}", file.name))?;
        for quoted in program.quoted_strings()? {
            let line = &program.lines[quoted.line_index];
            let outside = outside_literal_prefix(line.body.as_slice(), &quoted);
            let Some(kind) = classify_candidate(&outside) else {
                continue;
            };
            let text = match decode_pc88_text(&quoted.bytes) {
                Ok(text) => text,
                Err(_) => {
                    skipped_nontext += 1;
                    continue;
                }
            };
            candidates.push(Candidate {
                file: file.name.clone(),
                file_index: file.index,
                line: quoted.line_number,
                line_index: quoted.line_index,
                statement_offset: quoted.statement_offset,
                body_offset: quoted.body_offset,
                offset: quoted.file_offset,
                terminated: quoted.terminated,
                kind,
                raw: quoted.bytes,
                text,
            });
        }
    }
    Ok((candidates, skipped_nontext))
}

fn outside_literal_prefix(line: &[u8], quoted: &BasicQuotedString) -> Vec<u8> {
    let end = quoted.body_offset.saturating_sub(1).min(line.len());
    let mut outside = Vec::new();
    let mut in_literal = false;
    for byte in line[quoted.statement_offset..end].iter().copied() {
        if byte == b'"' {
            in_literal = !in_literal;
        } else if !in_literal {
            outside.push(byte);
        }
    }
    outside
}

fn classify_candidate(outside: &[u8]) -> Option<&'static str> {
    if outside.contains(&PRINT_TOKEN) {
        Some("direct_print")
    } else if outside
        .windows(Q_ASSIGN.len())
        .any(|window| window == Q_ASSIGN)
    {
        Some("message_q")
    } else if outside
        .windows(SC_ASSIGN.len())
        .any(|window| window == SC_ASSIGN)
    {
        Some("message_sc")
    } else if outside
        .windows(MESSAGE_ASSIGN.len())
        .any(|window| window == MESSAGE_ASSIGN)
    {
        Some("menu_prompt")
    } else if outside
        .windows(SYMBOL_TOKEN.len())
        .any(|window| window == SYMBOL_TOKEN)
    {
        Some("symbol_text")
    } else if outside.iter().copied().find(|byte| *byte != b' ') == Some(DATA_TOKEN) {
        Some("data_text")
    } else {
        None
    }
}

fn decode_pc88_text(raw: &[u8]) -> Result<String> {
    let mut output = String::new();
    for byte in raw.iter().copied() {
        if let Some(glyph) = native_glyph_for_byte(byte) {
            output.push(glyph);
            continue;
        }
        match byte {
            0x20..=0x7E => output.push(char::from(byte)),
            0xA1..=0xDF => {
                let bytes = [byte];
                let (text, had_errors) = SHIFT_JIS.decode_without_bom_handling(&bytes);
                ensure!(!had_errors, "invalid CP932 single byte 0x{byte:02X}");
                output.push_str(&text);
            }
            _ => bail!("unsupported PC-88 text byte 0x{byte:02X}"),
        }
    }
    Ok(output)
}

fn validate_entry(entry: &TextEntry, candidate: &Candidate, expected_index: usize) -> Result<()> {
    ensure!(
        entry.index == expected_index,
        "translation _index sequence changed"
    );
    ensure!(
        entry.file == candidate.file
            && entry.file_index == candidate.file_index
            && entry.line == candidate.line
            && entry.line_index == candidate.line_index
            && entry.statement_offset == candidate.statement_offset
            && entry.body_offset == candidate.body_offset
            && entry.offset == candidate.offset
            && entry.size == candidate.raw.len()
            && entry.kind == candidate.kind
            && entry.terminated == candidate.terminated,
        "translation metadata changed for entry {}",
        entry.index
    );
    ensure!(
        entry.scr_msg == candidate.text,
        "immutable scr_msg changed for entry {}",
        entry.index
    );
    ensure!(
        entry.raw_sha256 == sha256_hex(&candidate.raw),
        "source byte hash changed for entry {}",
        entry.index
    );
    Ok(())
}

fn ensure_distinct_paths(source: &Path, output: &Path) -> Result<()> {
    let source = source
        .canonicalize()
        .with_context(|| format!("failed to resolve source {}", source.display()))?;
    let output = if output.exists() {
        output
            .canonicalize()
            .with_context(|| format!("failed to resolve output {}", output.display()))?
    } else {
        let parent = output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        parent
            .canonicalize()
            .with_context(|| format!("failed to resolve output parent {}", parent.display()))?
            .join(output.file_name().context("output has no filename")?)
    };
    ensure!(source != output, "output path must differ from source path");
    Ok(())
}

fn write_output_file(path: &Path, bytes: &[u8], overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        bail!(
            "output already exists: {}; pass --overwrite",
            path.display()
        );
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    ensure!(
        parent.is_dir(),
        "output parent is not a directory: {}",
        parent.display()
    );
    fs::write(path, bytes).with_context(|| format!("failed to write {}", path.display()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:X}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pc88_visible_glyphs_decode_semantically() {
        assert_eq!(
            decode_pc88_text(&[0xB1, 0xBD, 0xB6, b' ', b':', b' ', 0xE9]).unwrap(),
            "ｱｽｶ : ♥"
        );
        assert_eq!(decode_pc88_text(&[0x83, 0xE5]).unwrap(), "▄◣");
        assert_eq!(decode_pc88_text(b"\\1").unwrap(), "¥1");
    }

    #[test]
    fn role_classifier_ignores_tokens_inside_previous_literals() {
        let line = [
            b'Q', b'$', 0xF1, b'"', 0x91, b'"', 0xF3, b'"', b'X', b'"', 0,
        ];
        let quoted = BasicQuotedString {
            index: 1,
            line_index: 0,
            line_number: 10,
            body_offset: 8,
            file_offset: 12,
            statement_offset: 0,
            bytes: vec![b'X'],
            terminated: true,
        };
        assert_eq!(
            classify_candidate(&outside_literal_prefix(&line, &quoted)),
            Some("message_q")
        );
    }
}
