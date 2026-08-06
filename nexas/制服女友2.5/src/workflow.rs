use crate::format::{Script, StringSlot};
use crate::json::{TranslationEntry, read_entries, write_entries};
use crate::text::{
    choice_marks, has_japanese, is_name_candidate, is_pure_control, is_resource_name,
    is_system_file, is_system_text, split_boundaries, validate_body_change, validate_choice_change,
    validate_plain_text,
};
use crate::{ToolResult, error};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct ExtractStats {
    pub scanned_files: usize,
    pub json_files: usize,
    pub extracted_entries: usize,
    pub skipped: usize,
    pub warnings: usize,
}

#[derive(Debug, Default)]
pub struct InjectStats {
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub warnings: usize,
}

#[derive(Debug, Default)]
pub struct RebuildStats {
    pub scanned_files: usize,
    pub rebuilt_files: usize,
    pub byte_exact: usize,
    pub warnings: usize,
}

#[derive(Clone, Debug)]
struct SlotMeta {
    scope: String,
    function_index: Option<usize>,
    function_id: Option<i32>,
    string_index: usize,
    offset: usize,
}

#[derive(Clone, Debug)]
struct ExtractedFile {
    entries: Vec<TranslationEntry>,
    skipped: usize,
    warnings: Vec<String>,
}

#[derive(Clone, Debug)]
struct PreparedPatch {
    body_scope: String,
    body_function_index: Option<usize>,
    body_string_index: usize,
    body_value: String,
    name_scope: Option<String>,
    name_function_index: Option<usize>,
    name_string_index: Option<usize>,
    name_value: Option<String>,
}

struct EntryDetails {
    parts: crate::text::ControlParts,
    name_data: Option<(Option<String>, Option<String>, SlotMeta)>,
    entry_type: &'static str,
    rule: &'static str,
    choice_group: Option<usize>,
    choice_index: Option<usize>,
}

pub fn default_extract_output(input: &Path) -> PathBuf {
    if input.is_dir() {
        let name = input
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("mes");
        input.with_file_name(format!("{name}_json"))
    } else {
        input.with_extension("json")
    }
}

pub fn default_inject_output(input: &Path) -> PathBuf {
    if input.is_dir() {
        let name = input
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("mes");
        input.with_file_name(format!("{name}_injected"))
    } else {
        let stem = input
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("script");
        input.with_file_name(format!("{stem}_injected.bin"))
    }
}

pub fn default_rebuild_output(input: &Path) -> PathBuf {
    if input.is_dir() {
        let name = input
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("mes");
        input.with_file_name(format!("{name}_rebuilt"))
    } else {
        let stem = input
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("script");
        input.with_file_name(format!("{stem}_rebuilt.bin"))
    }
}

pub fn extract(input: &Path, output: &Path) -> ToolResult<ExtractStats> {
    ensure_output_absent(output)?;
    if input.is_dir() {
        extract_directory(input, output)
    } else {
        extract_single(input, output)
    }
}

pub fn inject(source: &Path, json: &Path, output: &Path) -> ToolResult<InjectStats> {
    ensure_output_absent(output)?;
    if source.is_dir() {
        inject_directory(source, json, output)
    } else {
        inject_single(source, json, output)
    }
}

pub fn rebuild(input: &Path, output: &Path) -> ToolResult<RebuildStats> {
    ensure_output_absent(output)?;
    if input.is_dir() {
        rebuild_directory(input, output)
    } else {
        rebuild_single(input, output)
    }
}

fn extract_single(input: &Path, output: &Path) -> ToolResult<ExtractStats> {
    let relative = file_name_for_json(input)?;
    let result = extract_file(input, &relative)?;
    let mut stats = ExtractStats {
        scanned_files: 1,
        extracted_entries: result.entries.len(),
        skipped: result.skipped,
        warnings: result.warnings.len(),
        ..ExtractStats::default()
    };
    if result.entries.is_empty() {
        return Err(error(format!(
            "{}: no translatable entries after system/resource filtering",
            input.display()
        )));
    }
    write_entries(output, &result.entries)?;
    stats.json_files = 1;
    for warning in result.warnings {
        eprintln!("[extract][warning] {warning}");
    }
    Ok(stats)
}

fn extract_directory(input: &Path, output: &Path) -> ToolResult<ExtractStats> {
    let files = collect_files(input)?;
    let mut pending = Vec::new();
    let mut stats = ExtractStats::default();
    for file in files {
        if !is_bin(&file) {
            continue;
        }
        stats.scanned_files += 1;
        let relative = relative_path(input, &file)?;
        let result = extract_file(&file, &relative)?;
        stats.extracted_entries += result.entries.len();
        stats.skipped += result.skipped;
        stats.warnings += result.warnings.len();
        if !result.entries.is_empty() {
            pending.push((relative, result.entries));
        }
    }

    fs::create_dir_all(output)?;
    for (relative, entries) in pending {
        let json_path = output.join(relative).with_extension("json");
        if let Some(parent) = json_path.parent() {
            fs::create_dir_all(parent)?;
        }
        write_entries(&json_path, &entries)?;
        stats.json_files += 1;
    }
    Ok(stats)
}

fn extract_file(path: &Path, relative: &str) -> ToolResult<ExtractedFile> {
    if is_system_file(path) {
        return Ok(ExtractedFile {
            entries: Vec::new(),
            skipped: 1,
            warnings: Vec::new(),
        });
    }
    let data =
        fs::read(path).map_err(|io_error| error(format!("{}: {io_error}", path.display())))?;
    let script = Script::parse(&data, &path.display().to_string())?;
    let mut output = ExtractedFile {
        entries: Vec::new(),
        skipped: 0,
        warnings: Vec::new(),
    };
    let mut index = 0usize;
    extract_table(
        &script.constants,
        SlotMeta {
            scope: "global".to_string(),
            function_index: None,
            function_id: None,
            string_index: 0,
            offset: 0,
        },
        relative,
        &mut index,
        &mut output,
    );
    for (function_index, function) in script.functions.iter().enumerate() {
        extract_table(
            &function.constants,
            SlotMeta {
                scope: "function".to_string(),
                function_index: Some(function_index),
                function_id: Some(function.id),
                string_index: 0,
                offset: 0,
            },
            relative,
            &mut index,
            &mut output,
        );
    }
    Ok(output)
}

fn extract_table(
    slots: &[StringSlot],
    base: SlotMeta,
    relative: &str,
    index: &mut usize,
    output: &mut ExtractedFile,
) {
    let values: Vec<String> = slots.iter().map(|slot| slot.value.clone()).collect();
    let choices = choice_marks(&values);
    let mut paired_names = HashSet::new();

    for slot_index in 0..slots.len().saturating_sub(1) {
        let name = &slots[slot_index].value;
        let body = &slots[slot_index + 1].value;
        if is_name_candidate(name)
            && !choices.contains_key(&(slot_index + 1))
            && is_extractable_body(body)
        {
            paired_names.insert(slot_index);
        }
    }

    for (slot_index, slot) in slots.iter().enumerate() {
        let value = &slot.value;
        if !slot.valid_utf8 {
            output.skipped += 1;
            output.warnings.push(format!(
                "{relative}: string table entry {slot_index} has invalid UTF-8 and was preserved"
            ));
            continue;
        }
        if let Some(choice) = choices.get(&slot_index) {
            let parts = split_boundaries(value);
            add_entry(
                output,
                index,
                relative,
                slot,
                SlotMeta {
                    scope: base.scope.clone(),
                    function_index: base.function_index,
                    function_id: base.function_id,
                    string_index: slot_index,
                    offset: slot.offset,
                },
                EntryDetails {
                    parts,
                    name_data: None,
                    entry_type: "choice",
                    rule: "choice-run",
                    choice_group: Some(choice.group),
                    choice_index: Some(choice.index),
                },
            );
            continue;
        }
        if paired_names.contains(&slot_index) {
            continue;
        }
        if is_extractable_body(value) {
            let parts = split_boundaries(value);
            let name_data = if slot_index > 0 && paired_names.contains(&(slot_index - 1)) {
                let name_slot = &slots[slot_index - 1];
                Some((name_slot.value.clone(), name_slot.offset, slot_index - 1))
            } else {
                None
            };
            let has_name = name_data.is_some();
            add_entry(
                output,
                index,
                relative,
                slot,
                SlotMeta {
                    scope: base.scope.clone(),
                    function_index: base.function_index,
                    function_id: base.function_id,
                    string_index: slot_index,
                    offset: slot.offset,
                },
                EntryDetails {
                    parts,
                    name_data: name_data.map(|(name_value, name_offset, name_index)| {
                        (
                            Some(name_value.clone()),
                            Some(name_value),
                            SlotMeta {
                                scope: base.scope.clone(),
                                function_index: base.function_index,
                                function_id: base.function_id,
                                string_index: name_index,
                                offset: name_offset,
                            },
                        )
                    }),
                    entry_type: if value.contains('「')
                        || value.contains('」')
                        || value.starts_with("@v")
                    {
                        "dialogue"
                    } else {
                        "monologue"
                    },
                    rule: if has_name { "direct-name" } else { "single" },
                    choice_group: None,
                    choice_index: None,
                },
            );
            continue;
        }
        if is_name_candidate(value) && !paired_names.contains(&slot_index) {
            let parts = split_boundaries(value);
            add_entry(
                output,
                index,
                relative,
                slot,
                SlotMeta {
                    scope: base.scope.clone(),
                    function_index: base.function_index,
                    function_id: base.function_id,
                    string_index: slot_index,
                    offset: slot.offset,
                },
                EntryDetails {
                    parts,
                    name_data: None,
                    entry_type: "name",
                    rule: "standalone-name",
                    choice_group: None,
                    choice_index: None,
                },
            );
            continue;
        }
        if !value.is_empty()
            && (is_resource_name(value) || is_pure_control(value) || is_system_text(value))
        {
            output.skipped += 1;
        }
    }
}

fn add_entry(
    output: &mut ExtractedFile,
    index: &mut usize,
    relative: &str,
    slot: &StringSlot,
    meta: SlotMeta,
    details: EntryDetails,
) {
    let name = details.name_data.as_ref().and_then(|data| data.0.clone());
    let scr_name = details.name_data.as_ref().and_then(|data| data.1.clone());
    let name_scope = details.name_data.as_ref().map(|data| data.2.scope.clone());
    let name_function_index = details
        .name_data
        .as_ref()
        .and_then(|data| data.2.function_index);
    let name_function_id = details
        .name_data
        .as_ref()
        .and_then(|data| data.2.function_id);
    let name_string_index = details.name_data.as_ref().map(|data| data.2.string_index);
    let name_offset = details.name_data.as_ref().map(|data| data.2.offset);
    let body_size = details.parts.body.len();
    output.entries.push(TranslationEntry {
        name,
        scr_msg: details.parts.body.clone(),
        message: details.parts.body,
        scr_name,
        scr_raw: slot.value.clone(),
        control_prefix: details.parts.prefix,
        control_suffix: details.parts.suffix,
        file: relative.to_string(),
        index: *index,
        offset: meta.offset,
        size: slot.raw.len(),
        body_size,
        scope: meta.scope,
        function_index: meta.function_index,
        function_id: meta.function_id,
        string_index: meta.string_index,
        name_scope,
        name_function_index,
        name_function_id,
        name_string_index,
        name_offset,
        entry_type: details.entry_type.to_string(),
        opcode: "constant_string".to_string(),
        rule: details.rule.to_string(),
        encoding: "utf-8".to_string(),
        policy: "relocate".to_string(),
        choice_group: details.choice_group,
        choice_index: details.choice_index,
    });
    *index += 1;
}

fn is_extractable_body(value: &str) -> bool {
    if value.is_empty()
        || is_resource_name(value)
        || is_pure_control(value)
        || is_system_text(value)
    {
        return false;
    }
    let body = split_boundaries(value).body;
    has_japanese(&body) && !body.is_empty()
}

fn inject_single(source: &Path, json: &Path, output: &Path) -> ToolResult<InjectStats> {
    let entries = read_entries(json)?;
    let data = fs::read(source)?;
    let (rebuilt, stats) = inject_data(&data, source, &file_name_for_json(source)?, &entries)?;
    fs::write(output, rebuilt)?;
    Ok(stats)
}

fn inject_directory(source: &Path, json_root: &Path, output: &Path) -> ToolResult<InjectStats> {
    if !json_root.is_dir() {
        return Err(error(format!(
            "{}: JSON input must be a directory",
            json_root.display()
        )));
    }
    let files = collect_files(source)?;
    let mut rebuilt = Vec::new();
    let mut seen_json = HashSet::new();
    let mut stats = InjectStats::default();
    for file in &files {
        let relative = relative_path(source, file)?;
        if !is_bin(file) {
            continue;
        }
        let json_path = json_root.join(&relative).with_extension("json");
        if !json_path.is_file() {
            continue;
        }
        seen_json.insert(json_path.clone());
        let entries = read_entries(&json_path)?;
        let data = fs::read(file)?;
        let (bytes, file_stats) = inject_data(&data, file, &relative, &entries)?;
        stats.json_entries += file_stats.json_entries;
        stats.patched += file_stats.patched;
        stats.unchanged += file_stats.unchanged;
        stats.failed += file_stats.failed;
        stats.warnings += file_stats.warnings;
        rebuilt.push((relative, bytes));
    }

    for json_path in collect_files(json_root)? {
        if json_path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        if !seen_json.contains(&json_path) {
            return Err(error(format!(
                "{}: JSON has no matching source .bin",
                json_path.display()
            )));
        }
    }

    copy_tree(source, output)?;
    for (relative, bytes) in rebuilt {
        fs::write(output.join(relative), bytes)?;
    }
    Ok(stats)
}

fn inject_data(
    data: &[u8],
    source: &Path,
    relative: &str,
    entries: &[TranslationEntry],
) -> ToolResult<(Vec<u8>, InjectStats)> {
    let mut script = Script::parse(data, &source.display().to_string())?;
    let mut stats = InjectStats {
        json_entries: entries.len(),
        ..InjectStats::default()
    };
    let mut locations = HashSet::new();
    let mut patches = Vec::with_capacity(entries.len());
    for entry in entries {
        validate_entry_metadata(entry, relative)?;
        let location = (
            entry.scope.clone(),
            entry.function_index,
            entry.string_index,
        );
        if !locations.insert(location) {
            return Err(error(format!(
                "{relative} entry {}: duplicate string location",
                entry.index
            )));
        }
        let slot = get_slot(
            &script,
            &entry.scope,
            entry.function_index,
            entry.string_index,
        )
        .ok_or_else(|| {
            error(format!(
                "{relative} entry {}: string location out of range",
                entry.index
            ))
        })?;
        if slot.value != entry.scr_raw {
            return Err(error(format!(
                "{relative} entry {}: _scr_raw does not match source string",
                entry.index
            )));
        }
        let parts = split_boundaries(&slot.value);
        if parts.body != entry.scr_msg
            || parts.prefix != entry.control_prefix
            || parts.suffix != entry.control_suffix
        {
            return Err(error(format!(
                "{relative} entry {}: source control boundary or scr_msg mismatch",
                entry.index
            )));
        }

        if entry.entry_type == "name" {
            let replacement = entry.name.as_deref().ok_or_else(|| {
                error(format!(
                    "{relative} entry {}: name entry lacks name",
                    entry.index
                ))
            })?;
            if entry.scr_name.as_deref() != Some(&slot.value) {
                return Err(error(format!(
                    "{relative} entry {}: _scr_name does not match source name",
                    entry.index
                )));
            }
            validate_plain_text(replacement, "name")
                .map_err(|message| error(format!("{relative} entry {}: {message}", entry.index)))?;
            patches.push(PreparedPatch {
                body_scope: entry.scope.clone(),
                body_function_index: entry.function_index,
                body_string_index: entry.string_index,
                body_value: replacement.to_string(),
                name_scope: None,
                name_function_index: None,
                name_string_index: None,
                name_value: None,
            });
            continue;
        }

        validate_body_change(&entry.scr_msg, &entry.message)
            .map_err(|message| error(format!("{relative} entry {}: {message}", entry.index)))?;
        if entry.entry_type == "choice" {
            validate_choice_change(&entry.message)
                .map_err(|message| error(format!("{relative} entry {}: {message}", entry.index)))?;
        }
        let name_value = if let Some(name) = entry.name.as_deref() {
            let name_scope = entry.name_scope.as_deref().ok_or_else(|| {
                error(format!(
                    "{relative} entry {}: missing _name_scope",
                    entry.index
                ))
            })?;
            let name_index = entry.name_string_index.ok_or_else(|| {
                error(format!(
                    "{relative} entry {}: missing _name_string_index",
                    entry.index
                ))
            })?;
            let name_function_index = entry.name_function_index;
            let name_slot = get_slot(&script, name_scope, name_function_index, name_index)
                .ok_or_else(|| {
                    error(format!(
                        "{relative} entry {}: name location out of range",
                        entry.index
                    ))
                })?;
            if entry.scr_name.as_deref() != Some(&name_slot.value) {
                return Err(error(format!(
                    "{relative} entry {}: _scr_name does not match source name",
                    entry.index
                )));
            }
            validate_plain_text(name, "name")
                .map_err(|message| error(format!("{relative} entry {}: {message}", entry.index)))?;
            Some(name.to_string())
        } else {
            None
        };
        patches.push(PreparedPatch {
            body_scope: entry.scope.clone(),
            body_function_index: entry.function_index,
            body_string_index: entry.string_index,
            body_value: format!(
                "{}{}{}",
                entry.control_prefix, entry.message, entry.control_suffix
            ),
            name_scope: entry.name_scope.clone(),
            name_function_index: entry.name_function_index,
            name_string_index: entry.name_string_index,
            name_value,
        });
    }

    for patch in patches {
        let body_slot = get_slot_mut(
            &mut script,
            &patch.body_scope,
            patch.body_function_index,
            patch.body_string_index,
        )
        .ok_or_else(|| {
            error(format!(
                "{relative}: body location disappeared during patch"
            ))
        })?;
        if body_slot.value != patch.body_value {
            body_slot.value = patch.body_value;
            stats.patched += 1;
        } else {
            stats.unchanged += 1;
        }
        if let (Some(scope), Some(string_index), Some(name_value)) = (
            patch.name_scope.as_deref(),
            patch.name_string_index,
            patch.name_value,
        ) {
            let name_slot =
                get_slot_mut(&mut script, scope, patch.name_function_index, string_index)
                    .ok_or_else(|| {
                        error(format!(
                            "{relative}: name location disappeared during patch"
                        ))
                    })?;
            if name_slot.value != name_value {
                name_slot.value = name_value;
                stats.patched += 1;
            }
        }
    }
    Ok((script.to_bytes(), stats))
}

fn validate_entry_metadata(entry: &TranslationEntry, relative: &str) -> ToolResult<()> {
    if entry.file.replace('\\', "/") != relative.replace('\\', "/") {
        return Err(error(format!(
            "{relative} entry {}: _file is {}",
            entry.index, entry.file
        )));
    }
    if entry.encoding != "utf-8" || entry.policy != "relocate" || entry.opcode != "constant_string"
    {
        return Err(error(format!(
            "{relative} entry {}: unsupported metadata (_encoding/_policy/_opcode)",
            entry.index
        )));
    }
    if entry.message.contains('\0') || entry.scr_msg.contains('\0') {
        return Err(error(format!(
            "{relative} entry {}: NUL is not allowed",
            entry.index
        )));
    }
    Ok(())
}

fn get_slot<'a>(
    script: &'a Script,
    scope: &str,
    function_index: Option<usize>,
    string_index: usize,
) -> Option<&'a StringSlot> {
    match scope {
        "global" if function_index.is_none() => script.constants.get(string_index),
        "function" => script
            .functions
            .get(function_index?)
            .and_then(|function| function.constants.get(string_index)),
        _ => None,
    }
}

fn get_slot_mut<'a>(
    script: &'a mut Script,
    scope: &str,
    function_index: Option<usize>,
    string_index: usize,
) -> Option<&'a mut StringSlot> {
    match scope {
        "global" if function_index.is_none() => script.constants.get_mut(string_index),
        "function" => script
            .functions
            .get_mut(function_index?)
            .and_then(|function| function.constants.get_mut(string_index)),
        _ => None,
    }
}

fn rebuild_single(input: &Path, output: &Path) -> ToolResult<RebuildStats> {
    if is_system_file(input) {
        fs::copy(input, output)?;
        return Ok(RebuildStats {
            scanned_files: 1,
            rebuilt_files: 1,
            byte_exact: 1,
            warnings: 0,
        });
    }
    let data = fs::read(input)?;
    let script = Script::parse(&data, &input.display().to_string())?;
    let rebuilt = script.to_bytes();
    let byte_exact = usize::from(rebuilt == data);
    fs::write(output, rebuilt)?;
    Ok(RebuildStats {
        scanned_files: 1,
        rebuilt_files: 1,
        byte_exact,
        warnings: usize::from(byte_exact == 0),
    })
}

fn rebuild_directory(input: &Path, output: &Path) -> ToolResult<RebuildStats> {
    let files = collect_files(input)?;
    copy_tree(input, output)?;
    let mut stats = RebuildStats::default();
    for file in files {
        if !is_bin(&file) {
            continue;
        }
        stats.scanned_files += 1;
        let relative = relative_path(input, &file)?;
        if is_system_file(&file) {
            stats.rebuilt_files += 1;
            stats.byte_exact += 1;
            continue;
        }
        let data = fs::read(&file)?;
        let script = Script::parse(&data, &file.display().to_string())?;
        let rebuilt = script.to_bytes();
        stats.rebuilt_files += 1;
        if rebuilt == data {
            stats.byte_exact += 1;
        } else {
            stats.warnings += 1;
        }
        fs::write(output.join(relative), rebuilt)?;
    }
    Ok(stats)
}

fn ensure_output_absent(output: &Path) -> ToolResult<()> {
    if output.exists() {
        return Err(error(format!(
            "output already exists; refusing to overwrite: {}",
            output.display()
        )));
    }
    Ok(())
}

fn collect_files(root: &Path) -> ToolResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files_inner(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_inner(root: &Path, files: &mut Vec<PathBuf>) -> ToolResult<()> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files_inner(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn copy_tree(source: &Path, output: &Path) -> ToolResult<()> {
    fs::create_dir_all(output)?;
    for file in collect_files(source)? {
        let relative = relative_path(source, &file)?;
        let target = output.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(file, target)?;
    }
    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> ToolResult<String> {
    let relative = path.strip_prefix(root).map_err(|_| {
        error(format!(
            "{} is outside source root {}",
            path.display(),
            root.display()
        ))
    })?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn file_name_for_json(path: &Path) -> ToolResult<String> {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.to_string())
        .ok_or_else(|| error(format!("{}: non-Unicode filename", path.display())))
}

fn is_bin(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("bin"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{Function, StringSlot};

    fn minimal_script() -> Script {
        Script {
            unknown_array1: vec![1],
            unknown_array2: vec![[2, 3]],
            opcodes: vec![[29, 0]],
            constants: vec![StringSlot::new("@v0001「本文@n行」@k")],
            variables: Vec::new(),
            parameters: Vec::new(),
            unknown_blocks: Vec::new(),
            functions: vec![Function {
                id: 7,
                unknown_array1: Vec::new(),
                opcodes: Vec::new(),
                constants: Vec::new(),
                local_variables: Vec::new(),
                parameters: Vec::new(),
                unknown_blocks: Vec::new(),
            }],
        }
    }

    #[test]
    fn structure_roundtrip_is_byte_exact() {
        let script = minimal_script();
        let bytes = script.to_bytes();
        let parsed = Script::parse(&bytes, "test").expect("parse");
        assert_eq!(parsed.to_bytes(), bytes);
    }

    #[test]
    fn body_controls_can_be_edited() {
        assert!(validate_body_change("本文@n行", "译文@n行").is_ok());
        assert!(validate_body_change("本文@n行", "译文行").is_ok());
        assert!(validate_body_change("本文", "译文@n行").is_ok());
        assert!(validate_body_change("本文", "译文\n行").is_err());
    }
}
