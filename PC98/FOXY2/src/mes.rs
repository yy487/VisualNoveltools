use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

const TOOL_NAME: &str = "foxy2_d88_splitter 0.3.0";
const NEWLINE_SJIS: [u8; 2] = [0x81, 0x93];

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct MesTranslationFile {
    format: String,
    tool: String,
    #[serde(rename = "_resource_directory")]
    resource_directory: String,
    #[serde(rename = "_file")]
    file: String,
    #[serde(rename = "_encoding")]
    encoding: String,
    #[serde(rename = "_dict_base")]
    dict_base: u8,
    entries: Vec<MesEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MesEntry {
    #[serde(rename = "_index")]
    index: usize,
    #[serde(rename = "_offset")]
    offset: usize,
    #[serde(rename = "_byte_length")]
    byte_length: usize,
    #[serde(rename = "_type")]
    entry_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    scr_name: Option<String>,
    scr_msg: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MesBatchManifest {
    format: String,
    tool: String,
    files: Vec<MesBatchItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MesBatchItem {
    resource_directory: String,
    file: String,
    translation_file: String,
    entry_count: usize,
}

#[derive(Debug, Deserialize)]
struct ResourceRoot {
    disks: Vec<ResourceSummary>,
}

#[derive(Debug, Deserialize)]
struct ResourceSummary {
    output_directory: String,
}

#[derive(Debug, Deserialize)]
struct ResourceDisk {
    entries: Vec<ResourceFile>,
}

#[derive(Debug, Deserialize)]
struct ResourceFile {
    name: String,
    output_path: String,
    extension: String,
}

#[derive(Debug)]
struct ResourceDiskInput {
    directory_name: String,
    path: PathBuf,
    manifest: ResourceDisk,
}

#[derive(Debug)]
struct Ai5File {
    dict_end: usize,
    dict_base: u8,
    dictionary: Vec<[u8; 2]>,
    runs: Vec<TextRun>,
}

#[derive(Debug)]
struct TextRun {
    offset: usize,
    byte_length: usize,
    text: String,
}

#[derive(Debug, Clone)]
pub(crate) struct FontPatchRequest {
    pub(crate) carrier: char,
    pub(crate) replacement: char,
}

#[derive(Debug)]
struct PreparedMes {
    disk_name: String,
    relative_file: String,
    bytes: Vec<u8>,
}

pub fn mes_extract_inputs(input: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if !input.is_dir() {
        return Err(format!(
            "MES extraction input must be a resource directory: {}",
            input.display()
        ));
    }
    if output.exists() && !overwrite {
        return Err(format!(
            "output already exists; pass --overwrite: {}",
            output.display()
        ));
    }
    ensure_output_outside_input(input, output)?;
    let disks = discover_resource_disks(input)?;
    if disks.is_empty() {
        return Err(format!(
            "no resource disk manifests found in {}",
            input.display()
        ));
    }
    let mut prepared = Vec::new();
    for disk in &disks {
        for file in &disk.manifest.entries {
            if !file.extension.eq_ignore_ascii_case("mes") {
                continue;
            }
            let source_path = safe_join(&disk.path, &file.output_path)?;
            let bytes = fs::read(&source_path)
                .map_err(|e| format!("read {}: {e}", source_path.display()))?;
            let parsed =
                parse_ai5(&bytes).map_err(|e| format!("{}: {e}", source_path.display()))?;
            let relative_file = file.output_path.clone();
            let output_disk = output.join(&disk.directory_name);
            let output_path = output_disk.join(mes_json_name(&file.name));
            let entries = parsed
                .runs
                .iter()
                .enumerate()
                .map(|(index, run)| {
                    let (name, message) = split_name_prefix(&run.text);
                    MesEntry {
                        index,
                        offset: run.offset,
                        byte_length: run.byte_length,
                        entry_type: "dialogue".to_string(),
                        name: name.clone(),
                        scr_name: name,
                        scr_msg: message.clone(),
                        message,
                    }
                })
                .collect::<Vec<_>>();
            prepared.push((
                disk.directory_name.clone(),
                relative_file,
                file.name.clone(),
                parsed.dict_base,
                entries,
                output_path,
            ));
        }
    }
    if prepared.is_empty() {
        return Err("no .MES files found in resource manifests".to_string());
    }
    if output.exists() {
        remove_output(output)?;
    }
    fs::create_dir_all(output).map_err(|e| format!("create output: {e}"))?;
    let mut batch = Vec::with_capacity(prepared.len());
    for (disk_name, relative_file, source_name, dict_base, entries, output_path) in prepared {
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        if output_path.exists() && !overwrite {
            return Err(format!(
                "translation output already exists; pass --overwrite: {}",
                output_path.display()
            ));
        }
        write_json(
            &output_path,
            &MesTranslationFile {
                format: "Foxy 2 AI5 MES translation entries".to_string(),
                tool: TOOL_NAME.to_string(),
                resource_directory: disk_name.clone(),
                file: relative_file.clone(),
                encoding: "cp932".to_string(),
                dict_base,
                entries: entries.clone(),
            },
        )?;
        batch.push(MesBatchItem {
            resource_directory: disk_name,
            file: relative_file,
            translation_file: output_path
                .strip_prefix(output)
                .unwrap_or(&output_path)
                .to_string_lossy()
                .replace('\\', "/"),
            entry_count: entries.len(),
        });
        let _ = source_name;
    }
    write_json(
        &output.join("manifest.json"),
        &MesBatchManifest {
            format: "Foxy 2 AI5 MES translation workspace".to_string(),
            tool: TOOL_NAME.to_string(),
            files: batch,
        },
    )?;
    println!("extracted MES translations into {}", output.display());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn mes_inject_inputs(
    input: &Path,
    translations: &Path,
    output: &Path,
    font_source: &Path,
    subs_path: &Path,
    font_output: &Path,
    face: &str,
    overwrite: bool,
) -> Result<()> {
    if !input.is_dir() || !translations.is_dir() {
        return Err("MES injection input and translation paths must be directories".to_string());
    }
    if output.exists() && !overwrite {
        return Err(format!(
            "output already exists; pass --overwrite: {}",
            output.display()
        ));
    }
    ensure_output_outside_input(input, output)?;
    ensure_output_outside_input(translations, output)?;
    ensure_output_outside_input(input, font_output)?;
    ensure_output_outside_input(font_source, font_output)?;
    if font_output.exists() && !overwrite {
        return Err(format!(
            "font output already exists; pass --overwrite: {}",
            font_output.display()
        ));
    }
    let disks = discover_resource_disks(input)?;
    let translation_files = discover_translation_files(translations)?;
    if translation_files.is_empty() {
        return Err(format!(
            "no MES translation JSON files found in {}",
            translations.display()
        ));
    }
    let mappings = load_substitution_map(subs_path)?;
    let mut prepared = Vec::new();
    let mut font_requests = Vec::new();
    let mut literal_characters = HashSet::new();
    let mut translated_targets = HashSet::new();
    for (translation_path, translation) in translation_files {
        if !translation.encoding.eq_ignore_ascii_case("cp932") {
            return Err(format!(
                "{} declares unsupported encoding {}",
                translation_path.display(),
                translation.encoding
            ));
        }
        if !translated_targets.insert((
            translation.resource_directory.clone(),
            translation.file.clone(),
        )) {
            return Err(format!(
                "more than one translation JSON targets {}/{}",
                translation.resource_directory, translation.file
            ));
        }
        let disk = disks
            .iter()
            .find(|disk| disk.directory_name == translation.resource_directory)
            .ok_or_else(|| {
                format!(
                    "{} references unknown resource directory {}",
                    translation_path.display(),
                    translation.resource_directory
                )
            })?;
        let resource_path = safe_join(&disk.path, &translation.file)?;
        let source = fs::read(&resource_path)
            .map_err(|e| format!("read {}: {e}", resource_path.display()))?;
        let parsed = parse_ai5(&source).map_err(|e| format!("{}: {e}", resource_path.display()))?;
        if parsed.dict_base != translation.dict_base {
            return Err(format!(
                "{} dictionary base does not match translation JSON",
                translation_path.display()
            ));
        }
        let mut dictionary = parsed.dictionary.clone();
        let mut required_dictionary_pairs = HashSet::new();
        let mut pending_texts = Vec::new();
        let mut used_indices = HashSet::new();
        for entry in &translation.entries {
            if !used_indices.insert(entry.index) {
                return Err(format!(
                    "{} contains duplicate entry index {}",
                    translation_path.display(),
                    entry.index
                ));
            }
            let run = parsed.runs.get(entry.index).ok_or_else(|| {
                format!(
                    "{} entry {} is out of range",
                    translation_path.display(),
                    entry.index
                )
            })?;
            let (source_name, source_message) = split_name_prefix(&run.text);
            if source_name != entry.scr_name
                || source_message != entry.scr_msg
                || run.offset != entry.offset
                || run.byte_length != entry.byte_length
            {
                return Err(format!(
                    "{} entry {} source metadata/text does not match MES",
                    translation_path.display(),
                    entry.index
                ));
            }
            let replacement_name = match (&entry.scr_name, &entry.name) {
                (Some(_), Some(name)) => Some(name.as_str()),
                (Some(_), None) => {
                    return Err(format!(
                        "{} entry {} must keep a name field",
                        translation_path.display(),
                        entry.index
                    ))
                }
                (None, Some(_)) => {
                    return Err(format!(
                        "{} entry {} adds a name to unnamed source text",
                        translation_path.display(),
                        entry.index
                    ))
                }
                (None, None) => None,
            };
            let full_text = if let Some(name) = replacement_name {
                format!("［{name}］{}", entry.message)
            } else {
                entry.message.clone()
            };
            required_dictionary_pairs.extend(
                required_pairs_for_text(&full_text, &mappings).map_err(|e| {
                    format!("{} entry {}: {e}", translation_path.display(), entry.index)
                })?,
            );
            pending_texts.push((entry.index, entry.offset, entry.byte_length, full_text));
        }
        let mut reusable_dictionary_indices = reusable_dictionary_indices(
            &source,
            &parsed.runs,
            parsed.dict_base,
            &parsed.dictionary,
            &required_dictionary_pairs,
        );
        assign_dictionary_slots(
            &mut dictionary,
            parsed.dict_base,
            &required_dictionary_pairs,
            &mut reusable_dictionary_indices,
        )
        .map_err(|e| format!("{}: {e}", translation_path.display()))?;
        let mut patches = Vec::with_capacity(pending_texts.len());
        for (entry_index, offset, byte_length, full_text) in pending_texts {
            let encoded = prepare_text(
                &full_text,
                &mappings,
                parsed.dict_base,
                &mut dictionary,
                &mut Vec::new(),
                &mut font_requests,
                &mut literal_characters,
            )
            .map_err(|e| format!("{} entry {}: {e}", translation_path.display(), entry_index))?;
            patches.push((offset, byte_length, encoded));
        }
        patches.sort_by_key(|(offset, _, _)| *offset);
        for pair in patches.windows(2) {
            if pair[0].0 + pair[0].1 > pair[1].0 {
                return Err(format!(
                    "{} contains overlapping MES entries",
                    translation_path.display()
                ));
            }
        }
        let dictionary_delta = dictionary
            .len()
            .checked_sub(parsed.dictionary.len())
            .ok_or("MES dictionary unexpectedly shrank")?;
        let mut rebuilt = if dictionary_delta == 0 {
            let mut rebuilt = source;
            rewrite_dictionary_in_place(&mut rebuilt, &dictionary)?;
            rebuilt
        } else {
            let new_dict_end = parsed
                .dict_end
                .checked_add(dictionary_delta * 2)
                .ok_or("MES dictionary offset overflow")?;
            if new_dict_end > u16::MAX as usize {
                return Err(format!(
                    "{} dictionary is too large for AI5",
                    translation_path.display()
                ));
            }
            let mut expanded = Vec::with_capacity(source.len() + dictionary_delta * 2);
            expanded.extend_from_slice(&(new_dict_end as u16).to_le_bytes());
            for pair in &dictionary {
                expanded.extend_from_slice(pair);
            }
            expanded.extend_from_slice(&source[parsed.dict_end..]);
            expanded
        };
        let offset_delta = dictionary_delta * 2;
        for (offset, byte_length, encoded) in patches.into_iter().rev() {
            let shifted_offset = offset
                .checked_add(offset_delta)
                .ok_or("MES patch offset overflow")?;
            let end = shifted_offset
                .checked_add(byte_length)
                .ok_or("MES patch range overflow")?;
            if end > rebuilt.len() {
                return Err(format!(
                    "{} entry range exceeds MES",
                    translation_path.display()
                ));
            }
            rebuilt.splice(shifted_offset..end, encoded);
        }
        prepared.push(PreparedMes {
            disk_name: translation.resource_directory,
            relative_file: translation.file,
            bytes: rebuilt,
        });
    }

    let font_requests = coalesce_font_requests(font_requests)?;
    for request in &font_requests {
        if literal_characters.contains(&request.carrier) {
            return Err(format!(
                "carrier {} is also used literally in translated text and cannot be redrawn as {}",
                request.carrier, request.replacement
            ));
        }
    }
    let patched_font = crate::font::redraw_font(font_source, &font_requests, face)?;
    if output.exists() {
        remove_output(output)?;
    }
    copy_tree(input, output, overwrite)?;
    for mes in &prepared {
        let target = safe_join(&output.join(&mes.disk_name), &mes.relative_file)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        fs::write(&target, &mes.bytes).map_err(|e| format!("write {}: {e}", target.display()))?;
    }
    if let Some(parent) = font_output.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    fs::write(font_output, patched_font)
        .map_err(|e| format!("write {}: {e}", font_output.display()))?;
    println!(
        "injected {} MES file(s); redrew {} font slot(s)",
        prepared.len(),
        font_requests.len()
    );
    Ok(())
}

fn discover_resource_disks(input: &Path) -> Result<Vec<ResourceDiskInput>> {
    let mut result = Vec::new();
    let root_path = input.join("manifest.json");
    if root_path.is_file() {
        if let Ok(root) = read_json::<ResourceRoot>(&root_path) {
            for summary in root.disks {
                let directory = safe_join(input, &summary.output_directory)?;
                let manifest = read_json::<ResourceDisk>(&directory.join("manifest.json"))?;
                result.push(ResourceDiskInput {
                    directory_name: summary.output_directory,
                    path: directory,
                    manifest,
                });
            }
            if !result.is_empty() {
                return Ok(result);
            }
        }
        if let Ok(manifest) = read_json::<ResourceDisk>(&root_path) {
            result.push(ResourceDiskInput {
                directory_name: ".".to_string(),
                path: input.to_path_buf(),
                manifest,
            });
            return Ok(result);
        }
    }
    Err(format!(
        "{} is not a resource workspace manifest",
        input.display()
    ))
}

fn discover_translation_files(input: &Path) -> Result<Vec<(PathBuf, MesTranslationFile)>> {
    let manifest_path = input.join("manifest.json");
    let manifest: MesBatchManifest = read_json(&manifest_path)?;
    if manifest.format != "Foxy 2 AI5 MES translation workspace" {
        return Err(format!(
            "{} is not a Foxy 2 MES translation manifest",
            manifest_path.display()
        ));
    }
    let mut result = Vec::with_capacity(manifest.files.len());
    let mut paths = HashSet::with_capacity(manifest.files.len());
    for item in manifest.files {
        let path = safe_join(input, &item.translation_file)?;
        if !paths.insert(path.clone()) {
            return Err(format!(
                "{} lists a translation file more than once",
                manifest_path.display()
            ));
        }
        let translation: MesTranslationFile = read_json(&path)?;
        if translation.resource_directory != item.resource_directory
            || translation.file != item.file
            || translation.entries.len() != item.entry_count
        {
            return Err(format!(
                "{} does not match its translation manifest entry",
                path.display()
            ));
        }
        result.push((path, translation));
    }
    result.sort_by_key(|(path, _)| path.to_string_lossy().to_string());
    Ok(result)
}

fn parse_ai5(bytes: &[u8]) -> Result<Ai5File> {
    if bytes.len() < 2 {
        return Err("MES file is shorter than its AI5 header".to_string());
    }
    let dict_end = u16::from_le_bytes([bytes[0], bytes[1]]) as usize;
    if dict_end < 2 || dict_end > bytes.len() || !(dict_end - 2).is_multiple_of(2) {
        return Err(format!("invalid AI5 dictionary offset {dict_end:#x}"));
    }
    let mut dictionary = Vec::new();
    for offset in (2..dict_end).step_by(2) {
        dictionary.push([bytes[offset], bytes[offset + 1]]);
    }
    let dict_base = detect_dict_base(&bytes[dict_end..], dictionary.len());
    let runs = scan_text_runs(bytes, dict_end, dict_base, &dictionary)?;
    Ok(Ai5File {
        dict_end,
        dict_base,
        dictionary,
        runs,
    })
}

fn detect_dict_base(code: &[u8], dictionary_len: usize) -> u8 {
    if dictionary_len == 0 {
        return 0x80;
    }
    let score = |base: u8| {
        code.iter()
            .filter(|byte| **byte >= base && usize::from(**byte - base) < dictionary_len)
            .count()
    };
    if score(0x80) >= score(0xD0) {
        0x80
    } else {
        0xD0
    }
}

fn scan_text_runs(
    bytes: &[u8],
    code_start: usize,
    dict_base: u8,
    dictionary: &[[u8; 2]],
) -> Result<Vec<TextRun>> {
    let mut parser = Ai5Parser {
        bytes,
        cursor: code_start,
        dict_base,
        dictionary,
        runs: Vec::new(),
    };
    parser.parse_statements(None);
    Ok(parser.runs)
}

struct Ai5Parser<'a> {
    bytes: &'a [u8],
    cursor: usize,
    dict_base: u8,
    dictionary: &'a [[u8; 2]],
    runs: Vec<TextRun>,
}

impl<'a> Ai5Parser<'a> {
    fn parse_statements(&mut self, end: Option<u8>) {
        while self.cursor < self.bytes.len() {
            if end == Some(self.bytes[self.cursor]) {
                self.cursor += 1;
                return;
            }
            let saved = self.cursor;
            if !self.parse_statement() {
                self.cursor = saved + 1;
            }
        }
    }

    fn parse_statement(&mut self) -> bool {
        let byte = self.bytes[self.cursor];
        match byte {
            0x00 | 0x02 => {
                self.cursor += 1;
                true
            }
            0x01 => {
                self.cursor += 1;
                self.parse_statements(Some(0x00));
                true
            }
            0x04 => self.parse_sys(),
            0x06 => self.parse_string(),
            0x0A..=0x0E => self.parse_set(),
            0x0F => self.parse_condition(),
            0x10..=0x1F => self.parse_command(),
            0x60..=0xFF => self.parse_text(),
            _ => false,
        }
    }

    fn parse_text(&mut self) -> bool {
        let start = self.cursor;
        let mut raw = Vec::new();
        while let Some((pair, next)) =
            char_token(self.bytes, self.cursor, self.dict_base, self.dictionary)
        {
            raw.extend_from_slice(&pair);
            self.cursor = next;
        }
        if raw.is_empty() {
            return false;
        }
        match decode_text(&raw) {
            Ok(text) => self.runs.push(TextRun {
                offset: start,
                byte_length: self.cursor - start,
                text,
            }),
            Err(_) => {
                self.cursor = start + 1;
                return false;
            }
        }
        true
    }

    fn parse_string(&mut self) -> bool {
        self.cursor += 1;
        while self.cursor < self.bytes.len() {
            let byte = self.bytes[self.cursor];
            self.cursor += 1;
            if byte == 0x06 {
                return true;
            }
            if !(byte == 0x09 || (0x20..=0x7E).contains(&byte) || (0xA1..=0xDF).contains(&byte)) {
                return false;
            }
        }
        false
    }

    fn parse_sys(&mut self) -> bool {
        if self.cursor + 2 > self.bytes.len() {
            return false;
        }
        self.cursor += 2;
        self.parse_params();
        true
    }

    fn parse_command(&mut self) -> bool {
        self.cursor += 1;
        self.parse_params();
        true
    }

    fn parse_set(&mut self) -> bool {
        let opcode = self.bytes[self.cursor];
        self.cursor += 1;
        match opcode {
            0x0A => self.parse_num() && self.parse_exprs(),
            0x0B => self.parse_expr() && self.parse_exprs(),
            0x0C => self.parse_var() && self.parse_expr(),
            0x0D | 0x0E => self.parse_var() && self.parse_expr() && self.parse_exprs(),
            _ => false,
        }
    }

    fn parse_condition(&mut self) -> bool {
        self.cursor += 1;
        if !self.parse_expr() || self.cursor >= self.bytes.len() || self.bytes[self.cursor] != 0x01
        {
            return false;
        }
        self.cursor += 1;
        self.parse_statements(Some(0x00));
        while self.cursor < self.bytes.len() && self.bytes[self.cursor] == 0x02 {
            let saved = self.cursor;
            self.cursor += 1;
            if self.cursor >= self.bytes.len() || self.bytes[self.cursor] != 0x01 {
                self.cursor = saved;
                break;
            }
            self.cursor += 1;
            self.parse_statements(Some(0x00));
        }
        true
    }

    fn parse_params(&mut self) {
        if self.param_start() {
            self.parse_param();
            while self.cursor < self.bytes.len() && self.bytes[self.cursor] == 0x02 {
                let saved = self.cursor;
                self.cursor += 1;
                if !self.param_start() {
                    self.cursor = saved;
                    break;
                }
                if !self.parse_param() {
                    self.cursor = saved;
                    break;
                }
            }
        }
    }

    fn parse_param(&mut self) -> bool {
        if self.cursor >= self.bytes.len() {
            return false;
        }
        match self.bytes[self.cursor] {
            0x01 => {
                self.cursor += 1;
                self.parse_statements(Some(0x00));
                true
            }
            0x06 => self.parse_string(),
            _ => self.parse_expr(),
        }
    }

    fn param_start(&self) -> bool {
        self.cursor < self.bytes.len()
            && matches!(self.bytes[self.cursor], 0x01 | 0x06 | 0x07..=0x5A)
    }

    fn parse_exprs(&mut self) -> bool {
        if !self.parse_expr() {
            return false;
        }
        while self.cursor < self.bytes.len() && self.bytes[self.cursor] == 0x02 {
            let saved = self.cursor;
            self.cursor += 1;
            if !self.parse_expr() {
                self.cursor = saved;
                break;
            }
        }
        true
    }

    fn parse_expr(&mut self) -> bool {
        let start = self.cursor;
        while self.cursor < self.bytes.len() && self.bytes[self.cursor] != 0x03 {
            if !self.parse_expr_term() {
                self.cursor = start;
                return false;
            }
        }
        if self.cursor >= self.bytes.len() {
            self.cursor = start;
            return false;
        }
        self.cursor += 1;
        true
    }

    fn parse_expr_term(&mut self) -> bool {
        if self.cursor >= self.bytes.len() {
            return false;
        }
        let byte = self.bytes[self.cursor];
        match byte {
            0x07 => self.advance(2),
            0x08 => self.advance(3),
            0x09 => self.advance(4),
            0x20..=0x5A => self.advance(1),
            _ => false,
        }
    }

    fn parse_num(&mut self) -> bool {
        if self.cursor >= self.bytes.len() {
            return false;
        }
        match self.bytes[self.cursor] {
            0x30..=0x3F => self.advance(1),
            0x07 => self.advance(2),
            0x08 => self.advance(3),
            0x09 => self.advance(4),
            _ => false,
        }
    }

    fn parse_var(&mut self) -> bool {
        if self.cursor < self.bytes.len() && (0x40..=0x5A).contains(&self.bytes[self.cursor]) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn advance(&mut self, count: usize) -> bool {
        if self.cursor + count <= self.bytes.len() {
            self.cursor += count;
            true
        } else {
            false
        }
    }
}

fn char_token(
    bytes: &[u8],
    offset: usize,
    dict_base: u8,
    dictionary: &[[u8; 2]],
) -> Option<([u8; 2], usize)> {
    let first = *bytes.get(offset)?;
    if first >= dict_base {
        let index = usize::from(first - dict_base);
        return dictionary
            .get(index)
            .copied()
            .map(|pair| (pair, offset + 1));
    }
    if first < 0x60 || first >= dict_base || offset + 1 >= bytes.len() {
        return None;
    }
    let lead = first.wrapping_add(0x20);
    let trail = bytes[offset + 1];
    if !valid_sjis_pair(lead, trail) {
        return None;
    }
    Some(([lead, trail], offset + 2))
}

fn valid_sjis_pair(lead: u8, trail: u8) -> bool {
    ((0x81..=0x9F).contains(&lead) || (0xE0..=0xFC).contains(&lead))
        && ((0x40..=0x7E).contains(&trail) || (0x80..=0xFC).contains(&trail))
        && trail != 0x7F
}

fn decode_text(raw: &[u8]) -> Result<String> {
    if !raw.len().is_multiple_of(2) {
        return Err("text token stream has an odd CP932 length".to_string());
    }
    let mut text = String::new();
    for pair in raw.chunks_exact(2) {
        if pair == NEWLINE_SJIS {
            text.push('\n');
        } else {
            let (decoded, _, had_errors) = SHIFT_JIS.decode(pair);
            if had_errors {
                return Err(format!(
                    "invalid CP932 pair {:02X} {:02X}",
                    pair[0], pair[1]
                ));
            }
            text.push_str(&decoded);
        }
    }
    Ok(text)
}

fn split_name_prefix(text: &str) -> (Option<String>, String) {
    let close = if text.starts_with('［') {
        '］'
    } else if text.starts_with('[') {
        ']'
    } else {
        return (None, text.to_string());
    };
    let open_len = text.chars().next().map(char::len_utf8).unwrap_or(0);
    if let Some((end, close_char)) = text
        .char_indices()
        .skip(1)
        .find(|(_, character)| *character == close)
    {
        let message_start = end + close_char.len_utf8();
        return (
            Some(text[open_len..end].to_string()),
            text[message_start..].to_string(),
        );
    }
    (None, text.to_string())
}

fn prepare_text(
    text: &str,
    mappings: &HashMap<char, char>,
    dict_base: u8,
    dictionary: &mut Vec<[u8; 2]>,
    reusable_dictionary_indices: &mut Vec<usize>,
    font_requests: &mut Vec<FontPatchRequest>,
    literal_characters: &mut HashSet<char>,
) -> Result<Vec<u8>> {
    let mut units = Vec::new();
    for original in text.chars() {
        if original == '\r' || (original.is_control() && original != '\n') {
            return Err(format!(
                "unsupported control character U+{:04X}; only LF is an allowed display newline",
                original as u32
            ));
        }
        let normalized = fullwidth_ascii(original);
        if normalized == '\n' {
            units.push(NEWLINE_SJIS.to_vec());
            continue;
        }
        let (encoded, request) = encode_display_character(original, normalized, mappings)?;
        if let Some(request) = request {
            font_requests.push(request);
        } else {
            literal_characters.insert(normalized);
        }
        units.push(encoded);
    }
    let mut result = Vec::new();
    let mut dict_lookup = HashMap::new();
    for (index, pair) in dictionary.iter().enumerate() {
        dict_lookup.insert(*pair, index);
    }
    for unit in units {
        if unit.len() == 1 {
            return Err(format!(
                "single-byte character 0x{:02X} cannot appear in an AI5 text run; use a fullwidth character",
                unit[0]
            ));
        } else if unit.len() == 2 {
            let pair = [unit[0], unit[1]];
            if let Some(index) = dict_lookup.get(&pair) {
                let token = usize::from(dict_base) + *index;
                if token > 0xFF {
                    return Err("dictionary index cannot be represented by AI5".to_string());
                }
                result.push(token as u8);
            } else if pair[0] >= 0xA0 {
                let index = if let Some(index) = reusable_dictionary_indices.pop() {
                    let old_pair = dictionary[index];
                    if dict_lookup.get(&old_pair) == Some(&index) {
                        dict_lookup.remove(&old_pair);
                    }
                    dictionary[index] = pair;
                    index
                } else {
                    if dictionary.len() >= 256usize - usize::from(dict_base) {
                        return Err(format!("CP932 pair {:02X} {:02X} needs a new AI5 dictionary slot, but the dictionary is full", pair[0], pair[1]));
                    }
                    let index = dictionary.len();
                    dictionary.push(pair);
                    index
                };
                dict_lookup.insert(pair, index);
                result.push(dict_base + index as u8);
            } else {
                let token = pair[0].checked_sub(0x20).ok_or_else(|| {
                    format!(
                        "CP932 pair {:02X} {:02X} cannot be represented by Foxy AI5",
                        pair[0], pair[1]
                    )
                })?;
                if token >= dict_base {
                    return Err(format!(
                        "CP932 pair {:02X} {:02X} collides with AI5 dictionary range",
                        pair[0], pair[1]
                    ));
                }
                result.extend_from_slice(&[token, pair[1]]);
            }
        } else {
            return Err("CP932 encoder returned an unsupported unit".to_string());
        }
    }
    Ok(result)
}

fn reusable_dictionary_indices(
    bytes: &[u8],
    runs: &[TextRun],
    dict_base: u8,
    dictionary: &[[u8; 2]],
    required_pairs: &HashSet<[u8; 2]>,
) -> Vec<usize> {
    let mut used = HashSet::new();
    for run in runs {
        let end = run.offset.saturating_add(run.byte_length).min(bytes.len());
        let mut cursor = run.offset.min(bytes.len());
        while cursor < end {
            let token = bytes[cursor];
            if token >= dict_base {
                let index = usize::from(token - dict_base);
                if index < dictionary.len() {
                    used.insert(index);
                }
            }
            cursor += if token < dict_base && (0x60..0x80).contains(&token) {
                2
            } else {
                1
            };
        }
    }
    (0..dictionary.len())
        .filter(|index| used.contains(index) && !required_pairs.contains(&dictionary[*index]))
        .rev()
        .collect()
}

fn assign_dictionary_slots(
    dictionary: &mut Vec<[u8; 2]>,
    dict_base: u8,
    required_pairs: &HashSet<[u8; 2]>,
    reusable_indices: &mut Vec<usize>,
) -> Result<()> {
    let mut dictionary_lookup = HashMap::new();
    for (index, pair) in dictionary.iter().enumerate() {
        dictionary_lookup.insert(*pair, index);
    }
    let mut new_pairs = required_pairs
        .iter()
        .copied()
        .filter(|pair| pair[0] >= 0xA0 && !dictionary_lookup.contains_key(pair))
        .collect::<Vec<_>>();
    new_pairs.sort_unstable();
    for pair in new_pairs {
        let index = if let Some(index) = reusable_indices.pop() {
            let old_pair = dictionary[index];
            if dictionary_lookup.get(&old_pair) == Some(&index) {
                dictionary_lookup.remove(&old_pair);
            }
            dictionary[index] = pair;
            index
        } else {
            if dictionary.len() >= 256usize - usize::from(dict_base) {
                return Err(format!(
                    "CP932 pair {:02X} {:02X} needs a new AI5 dictionary slot, but the dictionary is full",
                    pair[0], pair[1]
                ));
            }
            let index = dictionary.len();
            dictionary.push(pair);
            index
        };
        dictionary_lookup.insert(pair, index);
    }
    Ok(())
}

fn rewrite_dictionary_in_place(bytes: &mut [u8], dictionary: &[[u8; 2]]) -> Result<()> {
    let required = dictionary
        .len()
        .checked_mul(2)
        .and_then(|size| 2usize.checked_add(size))
        .ok_or("MES dictionary size overflow")?;
    if required > bytes.len() {
        return Err("MES dictionary exceeds the file while rewriting reused slots".to_string());
    }
    for (index, pair) in dictionary.iter().enumerate() {
        let offset = 2 + index * 2;
        bytes[offset..offset + 2].copy_from_slice(pair);
    }
    Ok(())
}

fn required_pairs_for_text(text: &str, mappings: &HashMap<char, char>) -> Result<HashSet<[u8; 2]>> {
    let mut required = HashSet::new();
    for original in text.chars() {
        if original == '\r' || (original.is_control() && original != '\n') {
            return Err(format!(
                "unsupported control character U+{:04X}; only LF is an allowed display newline",
                original as u32
            ));
        }
        let normalized = fullwidth_ascii(original);
        if normalized == '\n' {
            continue;
        }
        let (encoded, _) = encode_display_character(original, normalized, mappings)?;
        if encoded.len() == 2 {
            required.insert([encoded[0], encoded[1]]);
        }
    }
    Ok(required)
}

fn encode_display_character(
    original: char,
    normalized: char,
    mappings: &HashMap<char, char>,
) -> Result<(Vec<u8>, Option<FontPatchRequest>)> {
    let (encoded, had_errors) = encode_cp932(normalized);
    let outside_loaded_font = encoded.len() == 2 && !crate::font::has_loaded_np2_slot(normalized);
    if !had_errors && !encoded.is_empty() && !outside_loaded_font {
        return Ok((encoded, None));
    }

    let reason = if outside_loaded_font {
        format!(
            "encodes to CP932 pair {:02X} {:02X} outside the loaded NP2 font pages",
            encoded[0], encoded[1]
        )
    } else {
        "is not CP932-encodable".to_string()
    };
    let carrier = mappings
        .get(&normalized)
        .copied()
        .or_else(|| mappings.get(&original).copied())
        .ok_or_else(|| format!("character {original} {reason} and has no subs_cn_jp mapping"))?;
    let (mapped, mapped_errors) = encode_cp932(carrier);
    if mapped_errors || mapped.len() != 2 {
        return Err(format!(
            "mapping carrier {carrier} is not a two-byte CP932 character"
        ));
    }
    if !crate::font::has_loaded_np2_slot(carrier) {
        return Err(format!(
            "mapping carrier {carrier} does not have a loaded NP2 font slot"
        ));
    }
    Ok((
        mapped,
        Some(FontPatchRequest {
            carrier,
            replacement: original,
        }),
    ))
}

fn encode_cp932(character: char) -> (Vec<u8>, bool) {
    let value = character.to_string();
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&value);
    (encoded.into_owned(), had_errors)
}

fn fullwidth_ascii(character: char) -> char {
    if character.is_ascii_digit() {
        return char::from_u32(character as u32 + 0xFEE0).unwrap();
    }
    match character {
        '!' => '！',
        '"' => '＂',
        '#' => '＃',
        '$' => '＄',
        '%' => '％',
        '&' => '＆',
        '\'' => '＇',
        '(' => '（',
        ')' => '）',
        '*' => '＊',
        '+' => '＋',
        ',' => '，',
        '-' => '－',
        '.' => '．',
        '/' => '／',
        ':' => '：',
        ';' => '；',
        '<' => '＜',
        '=' => '＝',
        '>' => '＞',
        '?' => '？',
        '@' => '＠',
        '[' => '［',
        '\\' => '＼',
        ']' => '］',
        '^' => '＾',
        '_' => '＿',
        '`' => '｀',
        '{' => '｛',
        '|' => '｜',
        '}' => '｝',
        '~' => '～',
        _ => character,
    }
}

fn load_substitution_map(path: &Path) -> Result<HashMap<char, char>> {
    let raw: HashMap<String, String> = read_json(path)?;
    let mut map = HashMap::new();
    for (source, target) in raw {
        let source_char = source.chars().collect::<Vec<_>>();
        let target_char = target.chars().collect::<Vec<_>>();
        if source_char.len() != 1 || target_char.len() != 1 {
            return Err(format!(
                "{} contains a mapping that is not one character per side",
                path.display()
            ));
        }
        map.insert(source_char[0], target_char[0]);
    }
    Ok(map)
}

fn coalesce_font_requests(requests: Vec<FontPatchRequest>) -> Result<Vec<FontPatchRequest>> {
    let mut result = Vec::new();
    let mut seen = HashMap::new();
    for request in requests {
        if let Some(previous) = seen.insert(request.carrier, request.replacement) {
            if previous != request.replacement {
                return Err(format!(
                    "carrier {} is requested for both {} and {}",
                    request.carrier, previous, request.replacement
                ));
            }
        } else {
            result.push(request);
        }
    }
    Ok(result)
}

fn copy_tree(input: &Path, output: &Path, overwrite: bool) -> Result<()> {
    fs::create_dir_all(output).map_err(|e| format!("create output: {e}"))?;
    for item in fs::read_dir(input).map_err(|e| format!("read {}: {e}", input.display()))? {
        let source = item.map_err(|e| format!("read input entry: {e}"))?.path();
        let target = output.join(source.file_name().ok_or("input entry has no filename")?);
        if source.is_dir() {
            copy_tree(&source, &target, overwrite)?;
        } else if !target.exists() || overwrite {
            fs::copy(&source, &target).map_err(|e| format!("copy {}: {e}", source.display()))?;
        } else {
            return Err(format!(
                "output file already exists; pass --overwrite: {}",
                target.display()
            ));
        }
    }
    Ok(())
}

fn ensure_output_outside_input(input: &Path, output: &Path) -> Result<()> {
    let input =
        fs::canonicalize(input).map_err(|e| format!("resolve input {}: {e}", input.display()))?;
    let output_resolved = if output.exists() {
        fs::canonicalize(output).map_err(|e| format!("resolve output {}: {e}", output.display()))?
    } else if let (Some(parent), Some(name)) = (output.parent(), output.file_name()) {
        fs::canonicalize(parent)
            .map(|parent| parent.join(name))
            .unwrap_or_else(|_| output.to_path_buf())
    } else {
        output.to_path_buf()
    };
    if output_resolved.starts_with(&input) {
        return Err(
            "output directory must not be the input directory or one of its descendants"
                .to_string(),
        );
    }
    Ok(())
}

fn remove_output(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|e| format!("remove old output {}: {e}", path.display()))
    } else {
        fs::remove_file(path).map_err(|e| format!("remove old output {}: {e}", path.display()))
    }
}

fn safe_join(base: &Path, relative: &str) -> Result<PathBuf> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe relative path in MES manifest: {relative}"));
    }
    Ok(base.join(path))
}

fn mes_json_name(name: &str) -> String {
    let stem = Path::new(name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("mes");
    format!("{}.json", stem)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("parse {}: {e}", path.display()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| format!("serialize JSON: {e}"))?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(|e| format!("write {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_fullwidth_named_dialogue_on_utf8_boundaries() {
        let (name, message) = split_name_prefix("［リサ］本文");
        assert_eq!(name.as_deref(), Some("リサ"));
        assert_eq!(message, "本文");
    }

    #[test]
    fn maps_chinese_and_encodes_newline_digits_and_punctuation() {
        let mappings = HashMap::from([('你', '凜')]);
        let mut dictionary = Vec::new();
        let mut requests = Vec::new();
        let encoded = prepare_text(
            "你\n1!",
            &mappings,
            0x80,
            &mut dictionary,
            &mut Vec::new(),
            &mut requests,
            &mut HashSet::new(),
        )
        .expect("translation should encode");
        assert_eq!(dictionary, vec![[0xEA, 0xA3]]);
        assert_eq!(encoded, vec![0x80, 0x61, 0x93, 0x62, 0x50, 0x61, 0x49]);
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].carrier, '凜');
        assert_eq!(requests[0].replacement, '你');
    }

    #[test]
    fn rejects_ascii_letters_inside_ai5_text() {
        let error = prepare_text(
            "A",
            &HashMap::new(),
            0x80,
            &mut Vec::new(),
            &mut Vec::new(),
            &mut Vec::new(),
            &mut HashSet::new(),
        )
        .expect_err("ASCII letter must not become an opcode");
        assert!(error.contains("single-byte"));
    }

    #[test]
    fn maps_cp932_extension_characters_outside_loaded_np2_pages() {
        let mappings = HashMap::from([('羡', '羨'), ('赶', '骭')]);
        let mut dictionary = Vec::new();
        let mut requests = Vec::new();
        let encoded = prepare_text(
            "羡赶",
            &mappings,
            0x80,
            &mut dictionary,
            &mut Vec::new(),
            &mut requests,
            &mut HashSet::new(),
        )
        .expect("CP932 extension characters should use loaded carriers");
        assert_eq!(dictionary, vec![[0xE9, 0x8C]]);
        assert_eq!(encoded, vec![0x71, 0x41, 0x80]);
        assert_eq!(requests.len(), 2);
        assert_eq!((requests[0].carrier, requests[0].replacement), ('羨', '羡'));
        assert_eq!((requests[1].carrier, requests[1].replacement), ('骭', '赶'));
        assert_eq!(
            required_pairs_for_text("羡赶", &mappings).expect("required carrier pairs"),
            HashSet::from([[0x91, 0x41], [0xE9, 0x8C]])
        );
    }

    #[test]
    fn rejects_unloaded_cp932_extension_without_mapping() {
        let error = required_pairs_for_text("羡", &HashMap::new())
            .expect_err("an unloaded CP932 extension needs a carrier mapping");
        assert!(error.contains("outside the loaded NP2 font pages"));
    }

    #[test]
    fn reuses_an_unreferenced_dictionary_slot_when_full() {
        let mappings = HashMap::from([('你', '凜')]);
        let mut dictionary = vec![[0x82, 0xA0], [0x82, 0xA1]];
        let mut reusable = vec![1];
        let mut requests = Vec::new();
        let encoded = prepare_text(
            "你",
            &mappings,
            0x80,
            &mut dictionary,
            &mut reusable,
            &mut requests,
            &mut HashSet::new(),
        )
        .expect("an unreferenced slot should be reusable");
        assert_eq!(dictionary, vec![[0x82, 0xA0], [0xEA, 0xA3]]);
        assert_eq!(encoded, vec![0x81]);
        assert!(reusable.is_empty());
    }

    #[test]
    fn preallocates_all_reused_pairs_before_encoding_runs() {
        let mappings = HashMap::from([('你', '凜')]);
        let mut dictionary = vec![[0x82, 0xA0], [0x82, 0xA1]];
        let mut reusable = vec![1];
        assign_dictionary_slots(
            &mut dictionary,
            0x80,
            &HashSet::from([[0xEA, 0xA3]]),
            &mut reusable,
        )
        .expect("pair should receive a slot before run encoding");
        let mut requests = Vec::new();
        let encoded = prepare_text(
            "你你",
            &mappings,
            0x80,
            &mut dictionary,
            &mut Vec::new(),
            &mut requests,
            &mut HashSet::new(),
        )
        .expect("preallocated pair should remain stable");
        assert_eq!(encoded, vec![0x81, 0x81]);
    }

    #[test]
    fn writes_reused_dictionary_pairs_without_growing_the_table() {
        let mut source = vec![0x06, 0x00, 0x82, 0xA0, 0x82, 0xA1, 0x80];
        let dictionary = [[0x82, 0xA0], [0xEA, 0xA3]];
        rewrite_dictionary_in_place(&mut source, &dictionary)
            .expect("reused dictionary bytes should be written");
        assert_eq!(source, vec![0x06, 0x00, 0x82, 0xA0, 0xEA, 0xA3, 0x80]);
    }
}
