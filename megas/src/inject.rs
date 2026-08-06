use crate::glyph::GlyphDictionary;
use crate::script::{
    parse_msb, parse_scx, rebuild_msb, rebuild_scx, rebuild_text_body, split_text,
};
use crate::text_json::{read_entries, TextEntry};
use crate::{hex_encode, ToolError, ToolResult};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone)]
pub struct InjectReport {
    pub input: PathBuf,
    pub json: PathBuf,
    pub output: PathBuf,
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub warnings: Vec<String>,
}

pub fn default_inject_output(input: &Path) -> ToolResult<PathBuf> {
    if input.is_dir() {
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        let name = input
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| {
                ToolError(format!(
                    "cannot derive output name from '{}'",
                    input.display()
                ))
            })?;
        return Ok(parent.join(format!("{name}_injected")));
    }
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let file_name = input
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ToolError(format!(
                "cannot derive output name from '{}'",
                input.display()
            ))
        })?;
    Ok(parent.join(format!("{file_name}.injected")))
}

pub fn inject_path(
    input: &Path,
    json_path: &Path,
    output: Option<&Path>,
    dictionary: &GlyphDictionary,
) -> ToolResult<InjectReport> {
    if input.is_file() {
        let entries = read_entries(json_path)?;
        let output_path = output
            .map(Path::to_path_buf)
            .unwrap_or(default_inject_output(input)?);
        refuse_existing(&output_path)?;
        let data = fs::read(input)
            .map_err(|error| ToolError(format!("cannot read '{}': {error}", input.display())))?;
        let (patched_data, patched, unchanged) = patch_file(input, &data, &entries, dictionary)?;
        fs::write(&output_path, patched_data).map_err(|error| {
            ToolError(format!(
                "cannot write injected file '{}': {error}",
                output_path.display()
            ))
        })?;
        return Ok(InjectReport {
            input: input.to_path_buf(),
            json: json_path.to_path_buf(),
            output: output_path,
            json_entries: entries.len(),
            patched,
            unchanged,
            warnings: Vec::new(),
        });
    }
    if !input.is_dir() {
        return Err(ToolError(format!(
            "inject input '{}' is not a file or directory",
            input.display()
        )));
    }
    if !json_path.is_dir() {
        return Err(ToolError(format!(
            "directory injection JSON input '{}' is not a directory",
            json_path.display()
        )));
    }
    let output_path = output
        .map(Path::to_path_buf)
        .unwrap_or(default_inject_output(input)?);
    refuse_existing(&output_path)?;
    let parent = output_path.parent().unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(ToolError(format!(
            "output parent '{}' does not exist",
            parent.display()
        )));
    }
    let mut grouped: HashMap<PathBuf, Vec<TextEntry>> = HashMap::new();
    let mut json_files = Vec::new();
    collect_json_files(json_path, &mut json_files)?;
    json_files.sort();
    let mut warnings = Vec::new();
    for json_file in json_files {
        let bytes = fs::read(&json_file).map_err(|error| {
            ToolError(format!(
                "cannot read JSON '{}': {error}",
                json_file.display()
            ))
        })?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|error| {
            ToolError(format!(
                "cannot parse JSON '{}': {error}",
                json_file.display()
            ))
        })?;
        if !value.is_array() {
            warnings.push(format!(
                "skipped non-translation JSON '{}'",
                json_file.display()
            ));
            continue;
        }
        let entries: Vec<TextEntry> = serde_json::from_value(value).map_err(|error| {
            ToolError(format!(
                "translation JSON '{}' has an invalid entry: {error}",
                json_file.display()
            ))
        })?;
        for entry in entries {
            let relative = safe_relative_path(&entry.file)?;
            grouped.entry(relative).or_default().push(entry);
        }
    }
    let temp_path = temp_sibling(&output_path)?;
    refuse_existing(&temp_path)?;
    fs::create_dir(&temp_path).map_err(|error| {
        ToolError(format!(
            "cannot create temporary injection directory '{}': {error}",
            temp_path.display()
        ))
    })?;
    if let Err(error) = copy_directory(input, &temp_path) {
        let _ = fs::remove_dir_all(&temp_path);
        return Err(error);
    }
    let mut patched = 0;
    let mut unchanged = 0;
    for (relative, entries) in &grouped {
        let source = input.join(relative);
        if !source.is_file() {
            let _ = fs::remove_dir_all(&temp_path);
            return Err(ToolError(format!(
                "translation entry source '{}' does not exist under '{}'",
                relative.display(),
                input.display()
            )));
        }
        let bytes = fs::read(&source)
            .map_err(|error| ToolError(format!("cannot read '{}': {error}", source.display())))?;
        let (patched_data, file_patched, file_unchanged) =
            patch_file(&source, &bytes, entries, dictionary)?;
        fs::write(temp_path.join(relative), patched_data).map_err(|error| {
            ToolError(format!(
                "cannot write injected file '{}': {error}",
                temp_path.join(relative).display()
            ))
        })?;
        patched += file_patched;
        unchanged += file_unchanged;
    }
    fs::rename(&temp_path, &output_path).map_err(|error| {
        let _ = fs::remove_dir_all(&temp_path);
        ToolError(format!(
            "cannot finalize injected output '{}': {error}",
            output_path.display()
        ))
    })?;
    Ok(InjectReport {
        input: input.to_path_buf(),
        json: json_path.to_path_buf(),
        output: output_path,
        json_entries: grouped.values().map(Vec::len).sum(),
        patched,
        unchanged,
        warnings,
    })
}

fn patch_file(
    source: &Path,
    data: &[u8],
    entries: &[TextEntry],
    dictionary: &GlyphDictionary,
) -> ToolResult<(Vec<u8>, usize, usize)> {
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if extension.eq_ignore_ascii_case("msb") {
        patch_msb(data, entries, dictionary)
    } else if extension.eq_ignore_ascii_case("scx") {
        patch_scx(data, entries, dictionary)
    } else {
        Err(ToolError(format!(
            "translation file '{}' has unsupported script extension .{extension}",
            source.display()
        )))
    }
}

fn patch_msb(
    data: &[u8],
    entries: &[TextEntry],
    dictionary: &GlyphDictionary,
) -> ToolResult<(Vec<u8>, usize, usize)> {
    let mut file = parse_msb(data)?;
    let body_base = 16 + file.records.len() * 8;
    let mut seen = vec![false; file.records.len()];
    let mut patched = 0;
    let mut unchanged = 0;
    for entry in entries {
        if entry.index >= file.records.len() {
            return Err(ToolError(format!(
                "MSB translation index {} is outside {} records",
                entry.index,
                file.records.len()
            )));
        }
        if seen[entry.index] {
            return Err(ToolError(format!(
                "duplicate MSB translation index {}",
                entry.index
            )));
        }
        seen[entry.index] = true;
        let record = &mut file.records[entry.index];
        validate_entry_location(
            entry,
            record.id,
            body_base + record.relative_offset as usize,
            record.body.len(),
        )?;
        let split = split_text(&record.body, dictionary);
        if entry.scr_msg != split.message {
            return Err(ToolError(format!(
                "MSB record {} scr_msg mismatch: JSON preserves {:?}, source is {:?}",
                entry.index, entry.scr_msg, split.message
            )));
        }
        if entry
            .raw_body
            .as_deref()
            .is_some_and(|raw| raw.to_ascii_lowercase() != hex_encode(&record.body))
        {
            return Err(ToolError(format!(
                "MSB record {} _raw_body mismatch",
                entry.index
            )));
        }
        if split.has_name && entry.scr_name.as_deref() != split.name.as_deref() {
            return Err(ToolError(format!(
                "MSB record {} _scr_name mismatch",
                entry.index
            )));
        }
        if !split.has_name && entry.name.as_ref().is_some_and(|value| !value.is_empty()) {
            return Err(ToolError(format!(
                "MSB record {} unexpectedly contains a name",
                entry.index
            )));
        }
        if entry
            .message_parts
            .as_ref()
            .is_some_and(|parts| !parts.is_empty())
        {
            return Err(ToolError(format!(
                "MSB record {} uses unsupported message_parts",
                entry.index
            )));
        }
        let replacement = rebuild_text_body(
            &record.body,
            entry.name.as_deref(),
            &entry.message,
            dictionary,
        )?;
        if replacement == record.body {
            unchanged += 1;
        } else {
            record.body = replacement;
            patched += 1;
        }
    }
    let output = rebuild_msb(&file)?;
    Ok((output, patched, unchanged))
}

fn patch_scx(
    data: &[u8],
    entries: &[TextEntry],
    dictionary: &GlyphDictionary,
) -> ToolResult<(Vec<u8>, usize, usize)> {
    let file = parse_scx(data)?;
    let mut replacements = HashMap::new();
    let mut patched = 0;
    let mut unchanged = 0;
    for entry in entries {
        if entry.index >= file.blocks.len() {
            return Err(ToolError(format!(
                "SCX translation index {} is outside {} blocks",
                entry.index,
                file.blocks.len()
            )));
        }
        let block = &file.blocks[entry.index];
        validate_entry_location(entry, 0, file.pointers[entry.index] as usize, block.len())?;
        let (source_message, _suffix_len) = crate::script::scx_content(block, dictionary);
        if entry.scr_msg != source_message {
            return Err(ToolError(format!(
                "SCX block {} scr_msg mismatch: JSON preserves {:?}, source is {:?}",
                entry.index, entry.scr_msg, source_message
            )));
        }
        if entry
            .raw_body
            .as_deref()
            .is_some_and(|raw| raw.to_ascii_lowercase() != hex_encode(block))
        {
            return Err(ToolError(format!(
                "SCX block {} _raw_body mismatch",
                entry.index
            )));
        }
        if entry.name.as_ref().is_some_and(|value| !value.is_empty()) {
            return Err(ToolError(format!(
                "SCX block {} contains an unsupported name",
                entry.index
            )));
        }
        let split = crate::script::encode_scx_content(block, &entry.message, dictionary)?;
        let source_content_len = block.len() - crate::script::scx_content(block, dictionary).1;
        if split == block[..source_content_len] {
            unchanged += 1;
        } else {
            replacements.insert(entry.index, split);
            patched += 1;
        }
    }
    let output = rebuild_scx(&file, &replacements)?;
    Ok((output, patched, unchanged))
}

fn validate_entry_location(
    entry: &TextEntry,
    id: u32,
    offset: usize,
    size: usize,
) -> ToolResult<()> {
    if entry.id.is_some_and(|value| value != id) {
        return Err(ToolError(format!(
            "translation index {} ID mismatch",
            entry.index
        )));
    }
    if entry.offset.is_some_and(|value| value != offset) {
        return Err(ToolError(format!(
            "translation index {} offset mismatch",
            entry.index
        )));
    }
    if entry.size.is_some_and(|value| value != size) {
        return Err(ToolError(format!(
            "translation index {} size mismatch",
            entry.index
        )));
    }
    Ok(())
}

fn collect_json_files(root: &Path, output: &mut Vec<PathBuf>) -> ToolResult<()> {
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
            collect_json_files(&path, output)?;
        } else if path
            .extension()
            .is_some_and(|value| value.eq_ignore_ascii_case("json"))
        {
            output.push(path);
        }
    }
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> ToolResult<()> {
    for entry in fs::read_dir(source)
        .map_err(|error| ToolError(format!("cannot list '{}': {error}", source.display())))?
    {
        let entry = entry.map_err(|error| {
            ToolError(format!(
                "cannot read directory entry in '{}': {error}",
                source.display()
            ))
        })?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            fs::create_dir_all(&destination_path).map_err(|error| {
                ToolError(format!(
                    "cannot create '{}': {error}",
                    destination_path.display()
                ))
            })?;
            copy_directory(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path).map_err(|error| {
                ToolError(format!(
                    "cannot copy '{}' to '{}': {error}",
                    source_path.display(),
                    destination_path.display()
                ))
            })?;
        }
    }
    Ok(())
}

fn safe_relative_path(value: &str) -> ToolResult<PathBuf> {
    let normalized = value.replace('/', "\\");
    let path = PathBuf::from(&normalized);
    if path.is_absolute() {
        return Err(ToolError(format!(
            "translation _file '{}' is absolute",
            value
        )));
    }
    for component in path.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err(ToolError(format!(
                "translation _file '{}' escapes the source directory",
                value
            )));
        }
    }
    Ok(path)
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
