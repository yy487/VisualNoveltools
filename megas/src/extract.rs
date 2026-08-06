use crate::glyph::{project_units, scan_units, GlyphDictionary, UnitKind};
use crate::script::{parse_msb, parse_scx, split_text};
use crate::text_json::{write_entries, TextEntry};
use crate::{hex_encode, ToolError, ToolResult};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

type ExtractProgress<'a> = &'a mut dyn FnMut(usize, usize, &Path);

#[derive(Debug, Clone)]
pub struct ExtractReport {
    pub input: PathBuf,
    pub output: PathBuf,
    pub scanned_files: usize,
    pub json_files: usize,
    pub extracted_entries: usize,
    pub warnings: Vec<String>,
}

pub fn default_extract_output(input: &Path) -> ToolResult<PathBuf> {
    if input.is_dir() {
        let name = input
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| {
                ToolError(format!(
                    "cannot derive output name from '{}', invalid directory name",
                    input.display()
                ))
            })?;
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        return Ok(parent.join(format!("{name}_json")));
    }
    let file_name = input
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ToolError(format!(
                "cannot derive output name from '{}', invalid filename",
                input.display()
            ))
        })?;
    Ok(input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{file_name}.json")))
}

pub fn extract_path(
    input: &Path,
    output: Option<&Path>,
    dictionary: &GlyphDictionary,
) -> ToolResult<ExtractReport> {
    extract_path_impl(input, output, dictionary, None)
}

pub fn extract_path_with_progress(
    input: &Path,
    output: Option<&Path>,
    dictionary: &GlyphDictionary,
    progress: &mut dyn FnMut(usize, usize, &Path),
) -> ToolResult<ExtractReport> {
    extract_path_impl(input, output, dictionary, Some(progress))
}

fn extract_path_impl(
    input: &Path,
    output: Option<&Path>,
    dictionary: &GlyphDictionary,
    mut progress: Option<ExtractProgress<'_>>,
) -> ToolResult<ExtractReport> {
    if input.is_file() {
        let output_path = output
            .map(Path::to_path_buf)
            .unwrap_or(default_extract_output(input)?);
        refuse_existing(&output_path)?;
        if let Some(callback) = progress.as_deref_mut() {
            callback(1, 1, input);
        }
        let data = fs::read(input)
            .map_err(|error| ToolError(format!("cannot read '{}': {error}", input.display())))?;
        let entries = extract_file_bytes(input, &data, &path_display_name(input), dictionary)?;
        write_entries(&output_path, &entries)?;
        return Ok(ExtractReport {
            input: input.to_path_buf(),
            output: output_path,
            scanned_files: 1,
            json_files: 1,
            extracted_entries: entries.len(),
            warnings: Vec::new(),
        });
    }
    if !input.is_dir() {
        return Err(ToolError(format!(
            "extract input '{}' is not a file or directory",
            input.display()
        )));
    }
    let output_path = output
        .map(Path::to_path_buf)
        .unwrap_or(default_extract_output(input)?);
    refuse_existing(&output_path)?;
    let parent = output_path.parent().unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(ToolError(format!(
            "output parent '{}' does not exist",
            parent.display()
        )));
    }
    let temp_path = temp_sibling(&output_path)?;
    refuse_existing(&temp_path)?;
    fs::create_dir(&temp_path).map_err(|error| {
        ToolError(format!(
            "cannot create temporary extract directory '{}': {error}",
            temp_path.display()
        ))
    })?;
    let operation = (|| -> ToolResult<(usize, usize, usize, Vec<String>)> {
        let mut files = Vec::new();
        collect_script_files(input, &mut files)?;
        files.sort();
        let mut json_files = 0;
        let mut entries_count = 0;
        let mut warnings = Vec::new();
        let scanned_files = files.len();
        for (file_index, source) in files.iter().enumerate() {
            let relative = source.strip_prefix(input).map_err(|error| {
                ToolError(format!(
                    "cannot make '{}' relative to '{}': {error}",
                    source.display(),
                    input.display()
                ))
            })?;
            if let Some(callback) = progress.as_deref_mut() {
                callback(file_index + 1, scanned_files, relative);
            }
            let data = fs::read(source).map_err(|error| {
                ToolError(format!("cannot read '{}': {error}", source.display()))
            })?;
            let source_name = path_to_json(relative);
            let entries = match extract_file_bytes(source, &data, &source_name, dictionary) {
                Ok(value) => value,
                Err(error) => {
                    warnings.push(format!("{}: {error}", source.display()));
                    continue;
                }
            };
            let destination = temp_path.join(format!("{}.json", source_name));
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    ToolError(format!(
                        "cannot create JSON directory '{}': {error}",
                        parent.display()
                    ))
                })?;
            }
            write_entries(&destination, &entries)?;
            json_files += 1;
            entries_count += entries.len();
        }
        Ok((scanned_files, json_files, entries_count, warnings))
    })();
    let (scanned_files, json_files, extracted_entries, warnings) = match operation {
        Ok(value) => value,
        Err(error) => {
            let _ = fs::remove_dir_all(&temp_path);
            return Err(error);
        }
    };
    fs::rename(&temp_path, &output_path).map_err(|error| {
        let _ = fs::remove_dir_all(&temp_path);
        ToolError(format!(
            "cannot finalize extract output '{}': {error}",
            output_path.display()
        ))
    })?;
    Ok(ExtractReport {
        input: input.to_path_buf(),
        output: output_path,
        scanned_files,
        json_files,
        extracted_entries,
        warnings,
    })
}

pub fn extract_file_bytes(
    source: &Path,
    data: &[u8],
    source_name: &str,
    dictionary: &GlyphDictionary,
) -> ToolResult<Vec<TextEntry>> {
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if extension.eq_ignore_ascii_case("msb") {
        extract_msb(data, source_name, dictionary)
    } else if extension.eq_ignore_ascii_case("scx") {
        extract_scx(data, source_name, dictionary)
    } else {
        Err(ToolError(format!(
            "unsupported script extension for '{}': .{extension}",
            source.display()
        )))
    }
}

pub fn extract_msb(
    data: &[u8],
    source_name: &str,
    dictionary: &GlyphDictionary,
) -> ToolResult<Vec<TextEntry>> {
    let file = parse_msb(data)?;
    let body_base = 16 + file.records.len() * 8;
    let mut output = Vec::with_capacity(file.records.len());
    for (index, record) in file.records.iter().enumerate() {
        let split = split_text(&record.body, dictionary);
        let body_units = scan_units(&record.body);
        let name_tokens = split
            .name_range
            .as_ref()
            .map(|range| glyphs_in_range(&body_units, range))
            .unwrap_or_default();
        let message_tokens = glyphs_in_range(&body_units, &split.message_range);
        let projection_units = body_units
            .iter()
            .filter(|unit| unit.offset < split.message_range.end)
            .copied()
            .collect::<Vec<_>>();
        let projection = project_units(&projection_units, dictionary);
        let unresolved = projection.unresolved;
        let mut extra = BTreeMap::new();
        extra.insert("_name_tokens".to_string(), json!(name_tokens));
        extra.insert("_message_tokens".to_string(), json!(message_tokens));
        extra.insert(
            "_unresolved_glyphs".to_string(),
            json!(unresolved
                .iter()
                .map(|value| format!("{value:04X}"))
                .collect::<Vec<_>>()),
        );
        extra.insert(
            "_controls".to_string(),
            json!(body_units
                .iter()
                .filter_map(|unit| match unit.kind {
                    UnitKind::Byte(value) => Some(format!("{value:02X}")),
                    UnitKind::Glyph(_) => None,
                })
                .collect::<Vec<_>>()),
        );
        extra.insert(
            "_layout".to_string(),
            json!(if split.has_name {
                "01 name / 02 message / terminator"
            } else {
                "message / terminator (no name separator)"
            }),
        );
        extra.insert(
            "_terminator".to_string(),
            json!(split.terminator.map(|value| vec![value[0], value[1]])),
        );
        output.push(TextEntry {
            file: source_name.to_string(),
            index,
            id: Some(record.id),
            offset: Some(body_base + record.relative_offset as usize),
            size: Some(record.body.len()),
            kind: Some(
                if split.has_name {
                    "dialogue"
                } else {
                    "monologue"
                }
                .to_string(),
            ),
            encoding: Some("glyph-index".to_string()),
            policy: Some("relocate".to_string()),
            name: split.name.clone(),
            scr_name: split.name,
            scr_msg: split.message.clone(),
            message: split.message,
            message_parts: None,
            raw_body: Some(hex_encode(&record.body)),
            extra,
        });
    }
    Ok(output)
}

pub fn extract_scx(
    data: &[u8],
    source_name: &str,
    dictionary: &GlyphDictionary,
) -> ToolResult<Vec<TextEntry>> {
    let file = parse_scx(data)?;
    let mut output = Vec::with_capacity(file.blocks.len());
    for (index, block) in file.blocks.iter().enumerate() {
        let suffix_len = if block.len() >= 2
            && matches!(block[block.len() - 2], 0x03 | 0x08)
            && block[block.len() - 1] == 0xFF
        {
            2
        } else {
            0
        };
        let content_end = block.len() - suffix_len;
        let units = scan_units(&block[..content_end]);
        let projection = project_units(&units, dictionary);
        let mut extra = BTreeMap::new();
        extra.insert(
            "_message_tokens".to_string(),
            json!(projection.glyph_indices),
        );
        extra.insert(
            "_unresolved_glyphs".to_string(),
            json!(projection
                .unresolved
                .iter()
                .map(|value| format!("{value:04X}"))
                .collect::<Vec<_>>()),
        );
        extra.insert(
            "_controls".to_string(),
            json!(projection
                .controls
                .iter()
                .map(|value| format!("{value:02X}"))
                .collect::<Vec<_>>()),
        );
        extra.insert(
            "_terminator".to_string(),
            json!(if suffix_len == 2 {
                vec![block[block.len() - 2], block[block.len() - 1]]
            } else {
                Vec::new()
            }),
        );
        extra.insert("_scx_fc".to_string(), json!(file.fc));
        extra.insert("_scx_f4".to_string(), json!(file.f4));
        extra.insert("_scx_f8".to_string(), json!(file.f8));
        output.push(TextEntry {
            file: source_name.to_string(),
            index,
            id: None,
            offset: file
                .pointers
                .get(index)
                .copied()
                .map(|value| value as usize),
            size: Some(block.len()),
            kind: Some("scx-block".to_string()),
            encoding: Some("glyph-index".to_string()),
            policy: Some("relocate".to_string()),
            name: None,
            scr_name: None,
            scr_msg: projection.text.clone(),
            message: projection.text,
            message_parts: None,
            raw_body: Some(hex_encode(block)),
            extra,
        });
    }
    Ok(output)
}

fn glyphs_in_range(units: &[crate::glyph::Unit], range: &std::ops::Range<usize>) -> Vec<u16> {
    units
        .iter()
        .filter(|unit| unit.offset >= range.start && unit.offset < range.end)
        .filter_map(|unit| match unit.kind {
            UnitKind::Glyph(index) => Some(index),
            UnitKind::Byte(_) => None,
        })
        .collect()
}

fn collect_script_files(root: &Path, output: &mut Vec<PathBuf>) -> ToolResult<()> {
    for entry in fs::read_dir(root)
        .map_err(|error| ToolError(format!("cannot list '{}': {error}", root.display())))?
    {
        let entry = entry.map_err(|error| {
            ToolError(format!(
                "cannot read directory entry in '{}': {error}",
                root.display()
            ))
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_script_files(&path, output)?;
        } else if path.extension().is_some_and(|value| {
            value.eq_ignore_ascii_case("msb") || value.eq_ignore_ascii_case("scx")
        }) {
            output.push(path);
        }
    }
    Ok(())
}

fn path_display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string()
}

fn path_to_json(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn refuse_existing(path: &Path) -> ToolResult<()> {
    if path.exists() {
        return Err(ToolError(format!(
            "output already exists: '{}'",
            path.display()
        )));
    }
    Ok(())
}

fn temp_sibling(path: &Path) -> ToolResult<PathBuf> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ToolError(format!(
                "cannot derive temporary name from '{}'",
                path.display()
            ))
        })?;
    Ok(parent.join(format!(".{name}.tmp-{}", std::process::id())))
}
