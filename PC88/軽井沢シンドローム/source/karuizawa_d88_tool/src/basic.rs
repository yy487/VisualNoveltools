use crate::text_compression::TextDictionary;
use crate::{sha256_hex, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

pub const TRANSLATION_FORMAT: &str = "karuizawa-n88-basic-translation-v2";
pub const SCRIPT_NAMES: [&str; 8] = [
    "KAL1 61", "KAL2 S1", "KAL3 S1", "KAL4 S1", "KAL5 S1", "KAL6 S1", "D", "S",
];

// D is the copy-protection carrier: its FAT chain crosses the nonstandard
// 00/52/52 protection sectors.  It can be inspected/exported, but must never
// be rebuilt as an ordinary BASIC program.
pub const PATCHABLE_SCRIPT_NAMES: [&str; 7] = [
    "KAL1 61", "KAL2 S1", "KAL3 S1", "KAL4 S1", "KAL5 S1", "KAL6 S1", "S",
];

const GRAPHIC_CHARACTERS: &str = concat!(
    "▁▂▃▄▅▆▇█▏▎▍▌▋▊▉┼",
    "┴┬┤├▔─│▕┌┐└┘╭╮╰╯",
    " ｡｢｣､･ｦｧｨｩｪｫｬｭｮｯ",
    "ｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿ",
    "ﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏ",
    "ﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝﾞﾟ",
    "═╞╪╡◢◣◥◤♠♥♦♣●○╱╲",
    "╳円年月日時分秒        "
);

#[derive(Debug, Clone)]
pub struct BasicLine {
    pub number: u16,
    pub link: u16,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct BasicProgram {
    pub lines: Vec<BasicLine>,
}

#[derive(Debug, Clone)]
struct StringSpan {
    index: usize,
    start: usize,
    end: usize,
    statement_start: usize,
    statement_end: usize,
}

#[derive(Debug, Clone, Copy)]
struct TranslationRegion {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslationEntry {
    pub id: String,
    pub line: u16,
    pub string_index: usize,
    pub kind: String,
    pub variable: Option<String>,
    pub source_raw_hex: String,
    pub source: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationFile {
    pub format: String,
    pub script: String,
    pub source_sha256: String,
    pub entries: Vec<TranslationEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct InjectionStats {
    pub entries: usize,
    pub changed_entries: usize,
    pub dimension_patches: usize,
    pub name_input_patches: usize,
    pub program_bytes: usize,
}

pub fn parse_program(data: &[u8]) -> Result<BasicProgram> {
    let mut lines: Vec<BasicLine> = Vec::new();
    let mut offset = 0usize;
    let mut saw_linked_line = false;
    while offset + 4 <= data.len() {
        let link = read_u16(data, offset)?;
        let number = read_u16(data, offset + 2)?;
        // Lines 10 and 50 in the KAL/S overlays deliberately carry a zero
        // link. Once the real linked program has begun, a zero word is the
        // two-byte BASIC end marker. Bytes after it are unlinked sector residue
        // and must not be moved into the rebuilt program.
        if link == 0 && (number == 0 || saw_linked_line) {
            break;
        }
        if let Some(previous) = lines.last() {
            if number <= previous.number {
                break;
            }
        }
        let end = body_end(data, offset + 4)?;
        lines.push(BasicLine {
            number,
            link,
            body: data[offset + 4..end].to_vec(),
        });
        saw_linked_line |= link != 0;
        offset = end + 1;
    }
    if lines.is_empty() {
        return Err("N88-BASIC 程序没有可解析行".into());
    }
    Ok(BasicProgram { lines })
}

pub fn extract_translation(script: &str, data: &[u8]) -> Result<TranslationFile> {
    let program = parse_program(data)?;
    let mut entries = Vec::new();
    for line in &program.lines {
        for span in string_spans(&line.body)? {
            let raw = &line.body[span.start..span.end];
            if raw.is_empty() {
                continue;
            }
            let statement = &line.body[span.statement_start..span.statement_end];
            if statement.first() == Some(&0x8F) {
                continue;
            }
            let (kind, variable) =
                match classify_string(statement, span.start - span.statement_start, raw) {
                    Some(value) => value,
                    None => continue,
                };
            let regions = translation_regions(raw, &kind);
            if regions.is_empty() {
                continue;
            }
            let source = decode_translation_text(raw, &regions);
            let source_raw = regions
                .iter()
                .flat_map(|region| raw[region.start..region.end].iter().copied())
                .collect::<Vec<_>>();
            entries.push(TranslationEntry {
                id: format!("{}:{:05}:{}", id_script(script), line.number, span.index),
                line: line.number,
                string_index: span.index,
                kind,
                variable,
                source_raw_hex: hex(&source_raw),
                message: source.clone(),
                source,
            });
        }
    }
    Ok(TranslationFile {
        format: TRANSLATION_FORMAT.into(),
        script: script.into(),
        source_sha256: sha256_hex(data),
        entries,
    })
}

pub fn inject_translation(
    script: &str,
    source: &[u8],
    translation: &TranslationFile,
    dictionary: &TextDictionary,
    large_font: bool,
) -> Result<(Vec<u8>, InjectionStats)> {
    if translation.format != TRANSLATION_FORMAT {
        return Err(format!("{script}: 不支持的翻译格式 {}", translation.format));
    }
    if translation.script != script {
        return Err(format!(
            "翻译脚本标识 {:?} 与目标 {:?} 不一致",
            translation.script, script
        ));
    }
    let source_sha = sha256_hex(source);
    if translation.source_sha256 != source_sha {
        return Err(format!(
            "{script}: 源脚本哈希不一致，JSON={}，原盘={source_sha}",
            translation.source_sha256
        ));
    }
    let baseline = extract_translation(script, source)?;
    if translation.entries.len() != baseline.entries.len() {
        return Err(format!(
            "{script}: JSON 条目数 {} 与源脚本 {} 不一致",
            translation.entries.len(),
            baseline.entries.len()
        ));
    }
    let mut replacements: HashMap<(u16, usize), String> = HashMap::new();
    let mut changed_entries = 0usize;
    for (edited, original) in translation.entries.iter().zip(&baseline.entries) {
        if edited.id != original.id
            || edited.line != original.line
            || edited.string_index != original.string_index
            || edited.kind != original.kind
            || edited.variable != original.variable
            || edited.source_raw_hex != original.source_raw_hex
            || edited.source != original.source
        {
            return Err(format!(
                "{script}: 条目 {} 的定位、原文或受保护元数据被修改",
                original.id
            ));
        }
        if edited.message == original.source {
            continue;
        }
        replacements.insert(
            (original.line, original.string_index),
            edited.message.clone(),
        );
        changed_entries += 1;
    }

    let mut program = parse_program(source)?;
    if !dictionary.is_empty() {
        if script == "S" {
            patch_dictionary_loader(&mut program)?;
        } else {
            // RUN loads a new BASIC overlay through the disk loader.  On M88
            // that path may use the same high-RAM scratch area as the
            // persistent dictionary, so reload the standalone image from the
            // new program before any chapter renderer can call BD93h.
            patch_dictionary_reload(&mut program)?;
        }
    }
    let mut dimension_patches = 0usize;
    let mut oversized_lines = Vec::new();
    for line in &mut program.lines {
        let spans = string_spans(&line.body)?;
        let mut edits = Vec::new();
        let mut literal_columns = HashMap::new();
        let mut speaker_breaks = HashSet::new();
        for span in spans {
            let raw = &line.body[span.start..span.end];
            let statement = &line.body[span.statement_start..span.statement_end];
            let Some((kind, _)) =
                classify_string(statement, span.start - span.statement_start, raw)
            else {
                continue;
            };
            let regions = translation_regions(raw, &kind);
            let message = replacements.get(&(line.number, span.index));
            let value = if kind == "print" {
                let Some(message) = message else { continue };
                let Some(region) = regions.first() else {
                    continue;
                };
                let mut value = raw[..region.start].to_vec();
                value.extend(encode_print(message)?);
                value.extend_from_slice(&raw[region.end..]);
                value
            } else if large_font {
                let rows = raw
                    .windows(2)
                    .filter(|pair| pair[0] == b'F' && pair[1].is_ascii_digit())
                    .count();
                let menu = (rows > 1 && (9110..=9130).contains(&line.number))
                    .then(|| {
                        message
                            .map(|text| crate::large_text::compact_menu(text, rows))
                            .transpose()
                    })
                    .transpose()?
                    .flatten();
                let uncompressed = rebuild_message_literal(
                    raw,
                    &regions,
                    menu.as_deref().or(message.map(String::as_str)),
                    &TextDictionary::default(),
                )?;
                let initial_column = literal_initial_column(
                    &line.body,
                    &span,
                    &literal_columns,
                    &mut speaker_breaks,
                );
                let (value, column) =
                    crate::large_text::wrap_literal(&uncompressed, initial_column, dictionary)?;
                literal_columns.insert(span.end, column);
                value
            } else {
                rebuild_message_literal(raw, &regions, message.map(String::as_str), dictionary)?
            };
            if value.contains(&b'"') || value.contains(&0) {
                return Err(format!(
                    "{script} 第 {} 行第 {} 个字符串的编码结果含终止字节",
                    line.number, span.index
                ));
            }
            if value != raw {
                edits.push((span.start, span.end, value));
            }
        }
        edits.extend(
            speaker_breaks
                .into_iter()
                .map(|offset| (offset, offset, vec![b'"', b'+', b'"', 0xF3])),
        );
        edits.sort_by_key(|(start, _, _)| std::cmp::Reverse(*start));
        for (start, end, value) in edits {
            line.body.splice(start..end, value);
        }
        dimension_patches += patch_dimensions(&mut line.body, large_font);
        if line.body.len() + 5 > 255 {
            oversized_lines.push((line.number, line.body.len() + 5));
        }
    }
    if !oversized_lines.is_empty() {
        let details = oversized_lines
            .iter()
            .map(|(line, bytes)| format!("{line}行={bytes}字节"))
            .collect::<Vec<_>>()
            .join("、");
        return Err(format!(
            "{script} 有 {} 条回注行超过 N88-BASIC 单行 255 字节限制：{details}",
            oversized_lines.len()
        ));
    }
    let name_input_patches = if script == "KAL1 61" {
        patch_english_name_input(&mut program)?
    } else {
        0
    };
    if large_font && script.starts_with("KAL") {
        patch_large_pagination(&mut program)?;
    }
    let rebuilt = encode_program(&program)?;
    validate_line_links(script, &rebuilt)?;
    Ok((
        rebuilt.clone(),
        InjectionStats {
            entries: baseline.entries.len(),
            changed_entries,
            dimension_patches,
            name_input_patches,
            program_bytes: rebuilt.len(),
        },
    ))
}

pub fn untranslated_count(file: &TranslationFile) -> usize {
    file.entries
        .iter()
        .filter(|entry| entry.message == entry.source && contains_japanese_source(entry))
        .count()
}

pub fn renderer_control_bytes(source: &[u8]) -> Result<HashSet<u8>> {
    let program = parse_program(source)?;
    let mut output = HashSet::new();
    for line in &program.lines {
        for span in string_spans(&line.body)? {
            let raw = &line.body[span.start..span.end];
            let statement = &line.body[span.statement_start..span.statement_end];
            let Some((kind, _)) =
                classify_string(statement, span.start - span.statement_start, raw)
            else {
                continue;
            };
            if kind == "print" {
                continue;
            }
            let regions = translation_regions(raw, &kind);
            for (offset, byte) in raw.iter().copied().enumerate() {
                if !regions
                    .iter()
                    .any(|region| (region.start..region.end).contains(&offset))
                {
                    output.insert(byte);
                }
            }
        }
    }
    Ok(output)
}

fn contains_japanese_source(entry: &TranslationEntry) -> bool {
    decode_hex(&entry.source_raw_hex).is_ok_and(|raw| raw.iter().any(|byte| *byte >= 0x80))
}

fn classify_string(
    statement: &[u8],
    quote_start_in_statement: usize,
    raw: &[u8],
) -> Option<(String, Option<String>)> {
    if contains_token_outside_strings(statement, 0x91)
        || contains_token_outside_strings(statement, 0x9B)
    {
        return Some(("print".into(), None));
    }
    if [0x8A, 0xB7, 0xC1, 0xC2, 0xD4, 0xD5]
        .iter()
        .any(|token| contains_token_outside_strings(statement, *token))
    {
        return None;
    }
    let prefix = &statement[..quote_start_in_statement];
    let active_start =
        last_token_position(prefix, &[0xDD, 0x9F]).map_or(0, |position| position + 1);
    if active_start == 0 && contains_token_outside_strings(prefix, 0x8B) {
        return None;
    }
    let variable = assignment_variable(&prefix[active_start..])?;
    let visible = raw.iter().any(|byte| *byte >= 0x80)
        || raw
            .iter()
            .any(|byte| matches!(*byte, b'+' | b'=' | b'\\' | b'/' | b'W'));
    match variable.as_str() {
        "A$" => Some(("message".into(), Some(variable))),
        "K$" | "RUM$" | "ATA$" if visible => Some(("speaker-or-fragment".into(), Some(variable))),
        _ => None,
    }
}

fn translation_regions(raw: &[u8], kind: &str) -> Vec<TranslationRegion> {
    if kind == "print" {
        let Some(start) = raw.iter().position(|byte| !byte.is_ascii_whitespace()) else {
            return Vec::new();
        };
        let end = raw
            .iter()
            .rposition(|byte| !byte.is_ascii_whitespace())
            .map_or(start, |position| position + 1);
        if !raw[start..end]
            .iter()
            .any(|byte| byte.is_ascii_alphanumeric() || *byte >= 0xA6)
        {
            return Vec::new();
        }
        return vec![TranslationRegion { start, end }];
    }

    // Low ASCII in the game's message language is structural.  Split at every
    // such control and expose only kana-bearing runs, keeping colors, waits,
    // line breaks, speaker selectors, slash state and padding in the source
    // BASIC bytes.  Punctuation at a run boundary remains structural too.
    let mut output = Vec::new();
    let mut run_start = 0usize;
    let mut offset = 0usize;
    while offset < raw.len() {
        let control_len = message_control_len(raw, offset);
        if control_len == 0 {
            offset += 1;
            continue;
        }
        if let Some(region) = trimmed_message_region(raw, run_start, offset) {
            output.push(region);
        }
        offset += control_len;
        run_start = offset;
    }
    if let Some(region) = trimmed_message_region(raw, run_start, raw.len()) {
        output.push(region);
    }
    output
}

fn decode_translation_text(raw: &[u8], regions: &[TranslationRegion]) -> String {
    let mut output = String::new();
    let mut previous_end = None;
    for region in regions {
        if previous_end.is_some_and(|end| gap_needs_space(&raw[end..region.start]))
            && !output.ends_with(' ')
        {
            output.push(' ');
        }
        output.push_str(&decode_pc88(&raw[region.start..region.end]));
        previous_end = Some(region.end);
    }
    output
}

fn gap_needs_space(gap: &[u8]) -> bool {
    gap.iter().any(|byte| {
        matches!(
            *byte,
            b' ' | b'+' | b'=' | b'W' | b'*' | b'\\' | b'>' | b'<'
        )
    })
}

fn distribute_translation(
    raw: &[u8],
    regions: &[TranslationRegion],
    message: &str,
) -> Result<Vec<String>> {
    if !message.is_ascii() {
        return Err("消息正文只支持半角 ASCII；无法按原控制区间排版".into());
    }
    if regions.is_empty() {
        return Ok(Vec::new());
    }
    let mut rows = vec![vec![0usize]];
    for index in 1..regions.len() {
        if raw[regions[index - 1].end..regions[index].start].contains(&b'+') {
            rows.push(Vec::new());
        }
        rows.last_mut().unwrap().push(index);
    }
    let weights = regions
        .iter()
        .map(|region| region.end - region.start)
        .collect::<Vec<_>>();
    let row_weights = rows
        .iter()
        .map(|row| row.iter().map(|index| weights[*index]).sum())
        .collect::<Vec<usize>>();
    // Existing numbered-menu translations separate their labels with two spaces.
    // Use that boundary only when it matches the original number of menu rows.
    let labels = message
        .split("  ")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let numbered_menu = rows.iter().all(|row| {
        let index = row[0];
        let start = if index == 0 {
            0
        } else {
            regions[index - 1].end
        };
        raw[start..regions[index].start]
            .iter()
            .any(u8::is_ascii_digit)
    });
    let row_texts = if rows.len() > 1 && numbered_menu && labels.len() == rows.len() {
        labels.into_iter().map(str::to_owned).collect()
    } else {
        split_at_word_boundaries(message, &row_weights)
    };
    let mut output = vec![String::new(); regions.len()];
    for (row, text) in rows.iter().zip(row_texts) {
        let region_weights = row.iter().map(|index| weights[*index]).collect::<Vec<_>>();
        let parts = split_at_word_boundaries(&text, &region_weights);
        for (position, (index, part)) in row.iter().zip(parts).enumerate() {
            output[*index] = part;
            if position + 1 < row.len() && !output[*index].is_empty() {
                let next = row[position + 1];
                let gap = &raw[regions[*index].end..regions[next].start];
                if !gap.contains(&b' ') {
                    output[*index].push(' ');
                }
            }
        }
    }
    Ok(output)
}

fn rebuild_message_literal(
    raw: &[u8],
    regions: &[TranslationRegion],
    message: Option<&str>,
    dictionary: &TextDictionary,
) -> Result<Vec<u8>> {
    let parts = message
        .map(|text| distribute_translation(raw, regions, text))
        .transpose()?;
    let mut output = Vec::new();
    let mut cursor = 0;
    let mut previous = "";
    for (index, region) in regions.iter().enumerate() {
        output.extend(encode_preserved_gap(
            &raw[cursor..region.start],
            previous,
            dictionary,
        )?);
        if let Some(parts) = &parts {
            output.extend(dictionary.encode_text(&parts[index])?);
            previous = &parts[index];
        } else {
            output.extend_from_slice(&raw[region.start..region.end]);
            previous = "";
        }
        cursor = region.end;
    }
    output.extend(encode_preserved_gap(&raw[cursor..], previous, dictionary)?);
    Ok(output)
}

fn encode_preserved_gap(
    raw: &[u8],
    previous: &str,
    dictionary: &TextDictionary,
) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut translated_punctuation = previous.trim_end().ends_with(['.', ',', '!', '?']);
    for byte in raw {
        let punctuation = match byte {
            0xA1 | 0xA5 => Some("."),
            0xA2 | 0xA3 => Some("\""),
            0xA4 => Some(","),
            b'!' => Some("!"),
            b'?' => Some("?"),
            _ => None,
        };
        if let Some(text) = punctuation {
            // The translation owns its sentence ending; source quote marks stay.
            if !translated_punctuation || matches!(*byte, 0xA2 | 0xA3) {
                output.extend(dictionary.encode_text(text)?);
            }
        } else {
            output.push(*byte);
            if matches!(*byte, b'+' | b'=' | b'\\' | b'0'..=b'9') {
                translated_punctuation = false;
            }
        }
    }
    Ok(output)
}

fn split_at_word_boundaries(message: &str, weights: &[usize]) -> Vec<String> {
    let total: usize = weights.iter().sum();
    let mut cumulative = 0;
    let mut start = 0;
    let mut output = Vec::with_capacity(weights.len());
    for (index, weight) in weights.iter().enumerate() {
        cumulative += weight;
        let end = if index + 1 == weights.len() {
            message.len()
        } else {
            let target = message.len() * cumulative / total.max(1);
            (start..=message.len())
                .filter(|offset| *offset == message.len() || message.as_bytes()[*offset] == b' ')
                .min_by_key(|offset| offset.abs_diff(target))
                .unwrap_or(message.len())
        };
        output.push(message[start..end].trim().to_owned());
        start = end;
    }
    output
}

fn message_control_len(raw: &[u8], offset: usize) -> usize {
    match raw[offset] {
        0x00..=0x1F | b'B'..=b'G' | b'+' | b'=' | b'W' | b'*' | b'/' | b'>' | b'<' => 1,
        b'\\' if raw.get(offset + 1).is_some_and(u8::is_ascii_digit) => 2,
        b'\\' => 1,
        _ => 0,
    }
}

fn trimmed_message_region(raw: &[u8], start: usize, end: usize) -> Option<TranslationRegion> {
    let relative_start = raw[start..end]
        .iter()
        .position(|byte| is_japanese_letter(*byte))?;
    let absolute_start = start + relative_start;
    let relative_end = raw[absolute_start..end]
        .iter()
        .rposition(|byte| is_japanese_letter(*byte))?;
    let mut absolute_end = absolute_start + relative_end + 1;
    while absolute_end < end && matches!(raw[absolute_end], 0xDE | 0xDF) {
        absolute_end += 1;
    }
    Some(TranslationRegion {
        start: absolute_start,
        end: absolute_end,
    })
}

fn is_japanese_letter(byte: u8) -> bool {
    matches!(byte, 0xA6..=0xDD | 0xF1..=0xF7)
}

fn assignment_variable(prefix: &[u8]) -> Option<String> {
    let mut found = None;
    let mut offset = 0usize;
    while offset < prefix.len() {
        let payload = token_payload_len(prefix[offset]);
        if payload != 0 {
            offset = (offset + 1 + payload).min(prefix.len());
            continue;
        }
        if prefix[offset].is_ascii_alphabetic() {
            let start = offset;
            offset += 1;
            while offset < prefix.len() && prefix[offset].is_ascii_alphanumeric() {
                offset += 1;
            }
            if prefix.get(offset) == Some(&b'$') {
                offset += 1;
                let name = String::from_utf8_lossy(&prefix[start..offset]).to_ascii_uppercase();
                let mut cursor = offset;
                while prefix.get(cursor) == Some(&b' ') {
                    cursor += 1;
                }
                if prefix
                    .get(cursor)
                    .is_some_and(|byte| matches!(*byte, 0xF1 | b'='))
                {
                    found = Some(name);
                }
            }
        } else {
            offset += 1;
        }
    }
    found
}

fn contains_token_outside_strings(data: &[u8], wanted: u8) -> bool {
    let mut quoted = false;
    let mut offset = 0usize;
    while offset < data.len() {
        let byte = data[offset];
        if byte == b'"' {
            quoted = !quoted;
            offset += 1;
        } else if !quoted && byte == wanted {
            return true;
        } else if !quoted {
            offset = (offset + 1 + token_payload_len(byte)).min(data.len());
        } else {
            offset += 1;
        }
    }
    false
}

fn last_token_position(data: &[u8], wanted: &[u8]) -> Option<usize> {
    let mut quoted = false;
    let mut result = None;
    let mut offset = 0usize;
    while offset < data.len() {
        let byte = data[offset];
        if byte == b'"' {
            quoted = !quoted;
            offset += 1;
        } else if !quoted {
            if wanted.contains(&byte) {
                result = Some(offset);
            }
            offset = (offset + 1 + token_payload_len(byte)).min(data.len());
        } else {
            offset += 1;
        }
    }
    result
}

fn string_spans(body: &[u8]) -> Result<Vec<StringSpan>> {
    let mut spans = Vec::new();
    let mut statement_start = 0usize;
    let mut index = 0usize;
    let mut offset = 0usize;
    while offset < body.len() {
        let byte = body[offset];
        if byte == 0x8F {
            break;
        }
        if byte == b':' {
            statement_start = offset + 1;
            offset += 1;
            continue;
        }
        if byte != b'"' {
            offset = (offset + 1 + token_payload_len(byte)).min(body.len());
            continue;
        }
        let start = offset + 1;
        let end = body[start..]
            .iter()
            .position(|value| *value == b'"')
            .map(|relative| start + relative)
            .unwrap_or(body.len());
        spans.push(StringSpan {
            index,
            start,
            end,
            statement_start,
            statement_end: statement_end(body, end.saturating_add(1)),
        });
        index += 1;
        if end == body.len() {
            break;
        }
        offset = end + 1;
    }
    Ok(spans)
}

fn statement_end(body: &[u8], start: usize) -> usize {
    let mut quoted = false;
    let mut offset = start;
    while offset < body.len() {
        let byte = body[offset];
        if byte == b'"' {
            quoted = !quoted;
            offset += 1;
        } else if byte == b':' && !quoted {
            return offset;
        } else if byte == 0x8F && !quoted {
            return body.len();
        } else if !quoted {
            offset = (offset + 1 + token_payload_len(byte)).min(body.len());
        } else {
            offset += 1;
        }
    }
    body.len()
}

fn body_end(data: &[u8], start: usize) -> Result<usize> {
    let mut quoted = false;
    let mut offset = start;
    while offset < data.len() {
        let value = data[offset];
        if quoted {
            if value == b'"' {
                quoted = false;
            } else if value == 0 {
                return Ok(offset);
            }
            offset += 1;
            continue;
        }
        if value == 0 {
            return Ok(offset);
        }
        if value == b'"' {
            quoted = true;
            offset += 1;
            continue;
        }
        if value == 0x8F {
            return data[offset + 1..]
                .iter()
                .position(|byte| *byte == 0)
                .map(|relative| offset + 1 + relative)
                .ok_or_else(|| format!("REM at 0x{offset:X} 没有行结束符"));
        }
        let payload = token_payload_len(value);
        if offset + 1 + payload > data.len() {
            return Err(format!("0x{offset:X} 的 token 载荷被截断"));
        }
        offset += 1 + payload;
    }
    Err(format!("0x{start:X} 的 BASIC 行没有结束符"))
}

fn token_payload_len(value: u8) -> usize {
    match value {
        0x0B | 0x0C | 0x0D | 0x0E | 0x1C => 2,
        0x0F | 0xFF => 1,
        0x1D => 4,
        0x1F => 8,
        _ => 0,
    }
}

fn literal_initial_column(
    body: &[u8],
    span: &StringSpan,
    columns: &HashMap<usize, usize>,
    speaker_breaks: &mut HashSet<usize>,
) -> Option<usize> {
    let prefix = &body[span.statement_start..span.start - 1];
    let assignment = last_token_position(prefix, &[0xF1])?;
    let mut offset = span.statement_start + assignment + 1;
    let mut column = Some(0);
    while offset < span.start - 1 {
        match body[offset] {
            b' ' | 0xF3 => offset += 1,
            b'"' => {
                let end = offset + 1 + body[offset + 1..].iter().position(|byte| *byte == b'"')?;
                column = columns.get(&end).copied();
                offset = end + 1;
            }
            byte if byte.is_ascii_alphabetic() => {
                let start = offset;
                while offset < span.start - 1
                    && (body[offset].is_ascii_alphanumeric() || body[offset] == b'$')
                {
                    offset += 1;
                }
                let width = match &body[start..offset] {
                    b"NA$" | b"AT$" | b"A1$" => Some(8),
                    b"ATA$" | b"RUM$" => Some(9),
                    _ => None,
                };
                if column.is_none() && width.is_some() {
                    // Keep a speaker with the following text when appending to A$.
                    speaker_breaks.insert(start);
                    column = Some(0);
                }
                column = column.zip(width).map(|(column, width)| column + width);
            }
            _ => return None,
        }
    }
    column
}

fn patch_dimensions(body: &mut Vec<u8>, large_font: bool) -> usize {
    if large_font {
        if !body
            .windows(6)
            .any(|bytes| bytes == [b'A', b'D', 0xF1, 0x0C, 0x86, 0xBE])
        {
            return 0;
        }
        let pattern = [0x98, b' ', b'A', b'D', b',', 0x1A];
        let Some(offset) = body
            .windows(pattern.len())
            .position(|bytes| bytes == pattern)
        else {
            return 0;
        };
        body.splice(offset + 5..offset + 6, [0x0F, 12]);
        return 1;
    }
    const PATTERN: [u8; 6] = [b'A', b'D', 0xF3, 0x12, b',', 0x13];
    let mut count = 0usize;
    let mut offset = 0usize;
    while offset + PATTERN.len() <= body.len() {
        if body[offset..offset + PATTERN.len()] == PATTERN {
            body[offset + PATTERN.len() - 1] = 0x12;
            count += 1;
            offset += PATTERN.len();
        } else {
            offset += 1;
        }
    }
    count
}

fn patch_large_pagination(program: &mut BasicProgram) -> Result<()> {
    for number in [177, 185, 29940, 29945, 29950] {
        if program.lines.iter().any(|line| line.number == number) {
            return Err(format!("大字分页预留行 {number} 已被占用"));
        }
    }
    let line = program
        .lines
        .iter_mut()
        .find(|line| line.number == 175)
        .ok_or("缺少正文输出行 175")?;
    let shortcut = [0x8B, b' ', b'L', b'S', b'P', b' ', 0x89, b' ', 0x0E, 120, 0];
    if !line.body.ends_with(&shortcut) {
        return Err("正文行 175 的快速输出分支与预期不符".into());
    }
    line.body.truncate(line.body.len() - shortcut.len());
    while line.body.last() == Some(&b' ') {
        line.body.pop();
    }
    if line.body.pop() != Some(b':') {
        return Err("正文行 175 快速分支前缺少语句分隔符".into());
    }

    // A new A$ starts at a row boundary even when GOSUB 170 continues an old page.
    // Low VRAM address at x=0 is (D0h + row*10h) AND FFh.
    let mut align = vec![0x8B, b' '];
    align.extend(b"LA");
    align.extend([
        0xF0, 0x11, b' ', 0xF8, b' ', 0xFF, 0x81, b'(', b'A', b'$', b',', 0x12, b')', 0xF2, 0xF0,
        b'"', b'+', b'"', b' ', 0xF8, b' ',
    ]);
    append_peek(&mut align, 0xBE89);
    align.push(0xF1);
    append_number(&mut align, 0xEA);
    align.extend([b' ', 0xF8, b' ']);
    append_peek(&mut align, 0xBE80);
    align.extend([0xF2, 0xF0, b'(', b'(']);
    append_number(&mut align, 0xD0);
    align.push(0xF3);
    append_peek(&mut align, 0xBE8C);
    align.push(0xF5);
    append_number(&mut align, 16);
    align.extend([b')', b' ', 0xF8, b' ']);
    append_number(&mut align, 255);
    align.extend([b')', b' ', 0xDD, b' ']);
    align.extend(b"C%");
    align.extend([0xF1, 0xE0, b'(']);
    append_number(&mut align, 43);
    align.push(b')');

    let mut check = vec![0x8B, b' '];
    append_peek(&mut check, 0xBE89);
    check.push(0xF1);
    append_number(&mut check, 0xEA);
    check.extend([b' ', 0xF8, b' ']);
    append_peek(&mut check, 0xBE8C);
    check.extend([0xF0, 0x14, b' ', 0xDD, b' ', 0x8D, b' ', 0x0E, 0xF4, 0x74]);

    // Trailing nonprinting controls must not clear the last visible page.
    let mut controls = vec![0x8B, b' '];
    for (index, byte) in b"+*=W/<>\\".iter().enumerate() {
        if index != 0 {
            controls.extend([b' ', 0xF9, b' ']);
        }
        controls.extend(b"C%");
        controls.push(0xF1);
        append_number(&mut controls, *byte);
    }
    controls.extend([b' ', 0xDD, b' ', 0x8E]);
    let colors = vec![
        0x8B, b' ', b'C', b'%', 0xF0, 0xF1, 0x0F, b'B', b' ', 0xF8, b' ', b'C', b'%', 0xF2, 0xF1,
        0x0F, b'G', b' ', 0xDD, b' ', 0x8E,
    ];

    // CLS is CEh. USR2 clears graphics only; preserve I, C%, K$ and A$.
    let mut page = vec![0xD6, b' ', 0x1A, b',', 0x0F, 19, b':', 0x91, b' '];
    page.extend(b"\"HIT ANY KEY\";:PG$");
    page.extend([
        0xF1, 0x85, b'$', b'(', 0x12, b')', b':', 0xCE, b':', b'A', 0xF1, 0xE0, 0x13, b'(', 0x11,
        b')',
    ]);
    for (address, value) in [(0xBE80u16, 0xD0u8), (0xBE81, 0xEA), (0xBE8C, 0)] {
        page.extend([b':', 0x98, b' ', 0x0C]);
        page.extend(address.to_le_bytes());
        page.push(b',');
        append_number(&mut page, value);
    }
    page.extend([b':', 0x8E]);
    for (number, body) in [
        (177, align),
        (185, check),
        (29940, controls),
        (29945, colors),
        (29950, page),
    ] {
        program.lines.push(BasicLine {
            number,
            link: 1,
            body,
        });
    }
    program.lines.sort_by_key(|line| line.number);
    Ok(())
}

fn append_peek(body: &mut Vec<u8>, address: u16) {
    body.extend([0xFF, 0x97, b'(', 0x0C]);
    body.extend(address.to_le_bytes());
    body.push(b')');
}

fn append_number(body: &mut Vec<u8>, value: u8) {
    if value <= 9 {
        body.push(0x11 + value);
    } else {
        body.extend([0x0F, value]);
    }
}

fn patch_english_name_input(program: &mut BasicProgram) -> Result<usize> {
    let line_8845 = program
        .lines
        .iter_mut()
        .find(|line| line.number == 8845)
        .ok_or("KAL1 61 缺少姓名处理行 8845")?;
    let marker = b"N1$";
    if line_8845
        .body
        .windows(marker.len())
        .any(|window| window == marker)
    {
        return Err("KAL1 61 姓名处理行 8845 已含正式补丁标记".into());
    }
    line_8845.body.extend_from_slice(&name_encoder_suffix());
    if line_8845.body.len() + 5 > 255 {
        return Err(format!(
            "KAL1 61 第 8845 行加入英文姓名编码后为 {} 字节，超过 255 字节",
            line_8845.body.len() + 5
        ));
    }

    let line_8890 = program
        .lines
        .iter_mut()
        .find(|line| line.number == 8890)
        .ok_or("KAL1 61 缺少姓名校验行 8890")?;
    let spans = string_spans(&line_8890.body)?;
    let target = spans
        .iter()
        .find(|span| line_8890.body[span.start..span.end] == [0xA6])
        .ok_or("KAL1 61 第 8890 行没有预期的片假名下界字符串 A6h")?;
    line_8890.body[target.start] = b'!';
    Ok(2)
}

fn patch_dictionary_loader(program: &mut BasicProgram) -> Result<()> {
    let line = program
        .lines
        .iter_mut()
        .find(|line| line.number == 120)
        .ok_or("S 缺少启动 BLOAD 行 120")?;
    if line.body.windows(6).any(|window| window == b"M3DICT") {
        return Err("S 启动行 120 已含 M3DICT 字典加载补丁".into());
    }
    line.body.extend_from_slice(&[
        b':', 0xD4, b' ', b'"', b'M', b'3', b'D', b'I', b'C', b'T', b'"', b',', 0x0C, 0x00, 0xF2,
    ]);
    if line.body.len() + 5 > 255 {
        return Err("S 启动 BLOAD 行 120 加入字典加载后超过 255 字节".into());
    }
    Ok(())
}

fn patch_dictionary_reload(program: &mut BasicProgram) -> Result<()> {
    if program
        .lines
        .iter()
        .any(|line| line.body.windows(6).any(|window| window == b"M3DICT"))
    {
        return Err("章节 BASIC 已含 M3DICT 字典加载补丁".into());
    }
    // Keep this as the first executable line.  It runs after RUN has loaded
    // the overlay and before the original line 10/50 startup code.
    program.lines.push(BasicLine {
        number: 1,
        // KAL overlays begin with two deliberately unlinked records (lines
        // 1/10 or 10/50); keep the loader line in that prefix so the parser
        // and the original runtime both reach the linked body.
        link: 0,
        body: vec![
            0xD4, b' ', b'"', b'M', b'3', b'D', b'I', b'C', b'T', b'"', b',', 0x0C, 0x00, 0xF2,
        ],
    });
    program.lines.sort_by_key(|line| line.number);
    Ok(())
}

fn name_encoder_suffix() -> Vec<u8> {
    let mut output = Vec::new();
    append_name_encoder(&mut output, b"NA$", b"N1$");
    append_name_encoder(&mut output, b"A1$", b"A2$");
    output.extend_from_slice(b":NA$");
    output.push(0xF1);
    output.extend_from_slice(b"N1$:A1$");
    output.push(0xF1);
    output.extend_from_slice(b"A2$");
    output
}

fn append_name_encoder(output: &mut Vec<u8>, source: &[u8], target: &[u8]) {
    output.push(b':');
    output.extend_from_slice(target);
    output.extend_from_slice(&[0xF1, b'"', b'/', b'"', b':', 0x82, b' ', b'I', 0xF1, 0x12]);
    output.extend_from_slice(&[b' ', 0xDC, b' ', 0xFF, 0x92, b'(']);
    output.extend_from_slice(source);
    output.extend_from_slice(b"):");
    output.extend_from_slice(target);
    output.push(0xF1);
    output.extend_from_slice(target);
    output.extend_from_slice(&[0xF3, 0xFF, 0x96, b'(', 0xFF, 0x95, b'(', 0xFF, 0x83, b'(']);
    output.extend_from_slice(source);
    output.extend_from_slice(&[b',', b'I', b',', 0x12, b')', b')', 0xF3, 0x0F, 101, b')']);
    output.extend_from_slice(&[b':', 0x83, b':']);
    output.extend_from_slice(target);
    output.push(0xF1);
    output.extend_from_slice(target);
    output.extend_from_slice(&[0xF3, b'"', b'/', b'"']);
}

fn encode_program(program: &BasicProgram) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    for line in &program.lines {
        let next_offset = output
            .len()
            .checked_add(4 + line.body.len() + 1)
            .ok_or("BASIC 行地址计算溢出")?;
        let link = if line.link == 0 {
            0
        } else {
            // These overlays store a one-based file offset to the following
            // line/end marker, not an absolute RAM address.
            u16::try_from(next_offset + 1).map_err(|_| "BASIC 行地址超过 16 位范围")?
        };
        output.extend_from_slice(&link.to_le_bytes());
        output.extend_from_slice(&line.number.to_le_bytes());
        output.extend_from_slice(&line.body);
        output.push(0);
    }
    output.extend_from_slice(&[0, 0]);
    Ok(output)
}

fn validate_line_links(script: &str, data: &[u8]) -> Result<()> {
    let program = parse_program(data)?;
    let mut offset = 0usize;
    for line in &program.lines {
        let next_offset = offset + 4 + line.body.len() + 1;
        if line.link != 0 && usize::from(line.link) != next_offset + 1 {
            return Err(format!(
                "{script} 第 {} 行的下一行地址为 {:04X}h，预期 {:04X}h",
                line.number,
                line.link,
                next_offset + 1
            ));
        }
        offset = next_offset;
    }
    Ok(())
}

pub fn decode_message_markup(raw: &[u8]) -> String {
    let mut output = String::new();
    let mut offset = 0usize;
    let mut emphasis = false;
    while offset < raw.len() {
        let byte = raw[offset];
        if byte >= 0x80 {
            let end = raw[offset..]
                .iter()
                .position(|value| *value < 0x80)
                .map_or(raw.len(), |relative| offset + relative);
            output.push_str(&decode_pc88(&raw[offset..end]));
            offset = end;
            continue;
        }
        match byte {
            b'B' => output.push_str("[color=red]"),
            b'C' => output.push_str("[color=magenta]"),
            b'D' => output.push_str("[color=green]"),
            b'E' => output.push_str("[color=cyan]"),
            b'F' => output.push_str("[color=yellow]"),
            b'G' => output.push_str("[color=white]"),
            b'+' => output.push_str("[nl]"),
            b'=' => output.push_str("[indent]"),
            b'W' => output.push_str("[wait]"),
            b'*' => output.push_str("[beep]"),
            b'/' => {
                if emphasis {
                    output.push_str("[/em]");
                } else {
                    output.push_str("[em]");
                }
                emphasis = !emphasis;
            }
            b'\\' if raw.get(offset + 1).is_some_and(u8::is_ascii_digit) => {
                output.push_str(&format!("[speaker={}]", char::from(raw[offset + 1])));
                offset += 1;
            }
            b'>' => output.push_str("[speaker=>]"),
            b'<' => output.push_str("[speaker=<]"),
            0x20..=0x7E => push_escaped_ascii(&mut output, char::from(byte)),
            _ => output.push_str(&format!("[raw={byte:02X}]")),
        }
        offset += 1;
    }
    output
}

pub fn encode_message_markup(message: &str) -> Result<Vec<u8>> {
    encode_message_markup_with_context(message, false)
}

fn encode_message_markup_with_context(message: &str, initial_direct_mode: bool) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut text = String::new();
    let chars = message.chars().collect::<Vec<_>>();
    let mut offset = 0usize;
    let mut emphasis_depth = 0usize;
    let mut direct_mode = initial_direct_mode;
    while offset < chars.len() {
        if chars[offset] == '[' {
            if chars.get(offset + 1) == Some(&'[') {
                text.push('[');
                offset += 2;
                continue;
            }
            flush_encoded_text(&mut output, &mut text, direct_mode)?;
            let end = chars[offset + 1..]
                .iter()
                .position(|character| *character == ']')
                .map(|relative| offset + 1 + relative)
                .ok_or_else(|| "消息中存在未闭合的 '['；字面 '[' 请写成 '[['".to_string())?;
            let tag = chars[offset + 1..end].iter().collect::<String>();
            match tag.as_str() {
                "nl" => output.push(b'+'),
                "indent" => output.push(b'='),
                "wait" => output.push(b'W'),
                "beep" => output.push(b'*'),
                "em" => {
                    if emphasis_depth != 0 {
                        return Err("消息暂不支持嵌套 [em]".into());
                    }
                    output.push(b'/');
                    direct_mode = !direct_mode;
                    emphasis_depth = 1;
                }
                "/em" => {
                    if emphasis_depth == 0 {
                        return Err("消息含有多余的 [/em]".into());
                    }
                    output.push(b'/');
                    direct_mode = !direct_mode;
                    emphasis_depth = 0;
                }
                "color=red" => output.push(b'B'),
                "color=magenta" => output.push(b'C'),
                "color=green" => output.push(b'D'),
                "color=cyan" => output.push(b'E'),
                "color=yellow" => output.push(b'F'),
                "color=white" => output.push(b'G'),
                "speaker=>" => output.push(b'>'),
                "speaker=<" => output.push(b'<'),
                _ if tag.len() == 9 && tag.starts_with("speaker=") => {
                    let value = tag.as_bytes()[8];
                    if !value.is_ascii_digit() {
                        return Err(format!("无效说话人标签 [{tag}]"));
                    }
                    output.extend_from_slice(&[b'\\', value]);
                }
                _ if tag.len() == 6 && tag.starts_with("raw=") => {
                    let value = u8::from_str_radix(&tag[4..], 16)
                        .map_err(|_| format!("无效原始字节标签 [{tag}]"))?;
                    if value == 0 || value == b'"' {
                        return Err(format!("[{tag}] 不能安全放入 BASIC 字符串"));
                    }
                    output.push(value);
                }
                _ => return Err(format!("未知消息标签 [{tag}]")),
            }
            offset = end + 1;
        } else if chars[offset] == ']' && chars.get(offset + 1) == Some(&']') {
            text.push(']');
            offset += 2;
        } else {
            text.push(chars[offset]);
            offset += 1;
        }
    }
    flush_encoded_text(&mut output, &mut text, direct_mode)?;
    if emphasis_depth != 0 {
        return Err("消息含有未闭合的 [em]".into());
    }
    if direct_mode != initial_direct_mode {
        return Err("消息改变了正文区域外的直接字形状态".into());
    }
    Ok(output)
}

fn flush_encoded_text(output: &mut Vec<u8>, text: &mut String, direct_mode: bool) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }
    if !direct_mode {
        output.push(b'/');
    }
    for character in text.chars() {
        if character == ' ' {
            output.push(b' ');
        } else if ('!'..='~').contains(&character) {
            output.push(0x86 + (character as u8 - b'!'));
        } else {
            return Err(format!(
                "英文消息只支持 ASCII !–~ 与空格，发现字符 {character:?}"
            ));
        }
    }
    if !direct_mode {
        output.push(b'/');
    }
    text.clear();
    Ok(())
}

fn encode_print(message: &str) -> Result<Vec<u8>> {
    if !message
        .chars()
        .all(|character| character == ' ' || ('!'..='~').contains(&character))
    {
        return Err("PRINT 文本只支持半角 ASCII 20h–7Eh".into());
    }
    if message.contains('"') {
        return Err("PRINT 文本暂不支持双引号".into());
    }
    Ok(message.as_bytes().to_vec())
}

pub fn decode_pc88(raw: &[u8]) -> String {
    let mut output = String::new();
    let mut offset = 0usize;
    while offset < raw.len() {
        if let Some(mark) = raw.get(offset + 1).copied() {
            if let Some(composed) = fullwidth_kana_pair(raw[offset], mark) {
                output.push_str(composed);
                offset += 2;
                continue;
            }
        }
        if let Some(fullwidth) = fullwidth_kana(raw[offset]) {
            output.push_str(fullwidth);
        } else {
            output.push(pc88_char(raw[offset]));
        }
        offset += 1;
    }
    output
}

fn fullwidth_kana(value: u8) -> Option<&'static str> {
    const TABLE: [&str; 63] = [
        "。", "「", "」", "、", "・", "ヲ", "ァ", "ィ", "ゥ", "ェ", "ォ", "ャ", "ュ", "ョ", "ッ",
        "ー", "ア", "イ", "ウ", "エ", "オ", "カ", "キ", "ク", "ケ", "コ", "サ", "シ", "ス", "セ",
        "ソ", "タ", "チ", "ツ", "テ", "ト", "ナ", "ニ", "ヌ", "ネ", "ノ", "ハ", "ヒ", "フ", "ヘ",
        "ホ", "マ", "ミ", "ム", "メ", "モ", "ヤ", "ユ", "ヨ", "ラ", "リ", "ル", "レ", "ロ", "ワ",
        "ン", "゛", "゜",
    ];
    (0xA1..=0xDF)
        .contains(&value)
        .then(|| TABLE[usize::from(value - 0xA1)])
}

fn fullwidth_kana_pair(base: u8, mark: u8) -> Option<&'static str> {
    match (base, mark) {
        (0xA6, 0xDE) => Some("ヺ"),
        (0xB2, 0xDE) => Some("ヸ"),
        (0xB3, 0xDE) => Some("ヴ"),
        (0xB4, 0xDE) => Some("ヹ"),
        (0xB6, 0xDE) => Some("ガ"),
        (0xB7, 0xDE) => Some("ギ"),
        (0xB8, 0xDE) => Some("グ"),
        (0xB9, 0xDE) => Some("ゲ"),
        (0xBA, 0xDE) => Some("ゴ"),
        (0xBB, 0xDE) => Some("ザ"),
        (0xBC, 0xDE) => Some("ジ"),
        (0xBD, 0xDE) => Some("ズ"),
        (0xBE, 0xDE) => Some("ゼ"),
        (0xBF, 0xDE) => Some("ゾ"),
        (0xC0, 0xDE) => Some("ダ"),
        (0xC1, 0xDE) => Some("ヂ"),
        (0xC2, 0xDE) => Some("ヅ"),
        (0xC3, 0xDE) => Some("デ"),
        (0xC4, 0xDE) => Some("ド"),
        (0xCA, 0xDE) => Some("バ"),
        (0xCB, 0xDE) => Some("ビ"),
        (0xCC, 0xDE) => Some("ブ"),
        (0xCD, 0xDE) => Some("ベ"),
        (0xCE, 0xDE) => Some("ボ"),
        (0xCA, 0xDF) => Some("パ"),
        (0xCB, 0xDF) => Some("ピ"),
        (0xCC, 0xDF) => Some("プ"),
        (0xCD, 0xDF) => Some("ペ"),
        (0xCE, 0xDF) => Some("ポ"),
        (0xDC, 0xDE) => Some("ヷ"),
        _ => None,
    }
}

fn pc88_char(value: u8) -> char {
    if (0x20..=0x7E).contains(&value) {
        char::from(value)
    } else if value >= 0x80 {
        GRAPHIC_CHARACTERS
            .chars()
            .nth(usize::from(value - 0x80))
            .unwrap_or('�')
    } else {
        '�'
    }
}

fn push_escaped_ascii(output: &mut String, character: char) {
    match character {
        '[' => output.push_str("[["),
        ']' => output.push_str("]]"),
        _ => output.push(character),
    }
}

fn id_script(script: &str) -> String {
    script
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}

fn decode_hex(value: &str) -> Result<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return Err("十六进制字符串长度不是偶数".into());
    }
    (0..value.len())
        .step_by(2)
        .map(|offset| {
            u8::from_str_radix(&value[offset..offset + 2], 16)
                .map_err(|_| format!("无效十六进制字节：{}", &value[offset..offset + 2]))
        })
        .collect()
}

fn read_u16(data: &[u8], offset: usize) -> Result<u16> {
    let raw = data
        .get(offset..offset + 2)
        .ok_or_else(|| format!("0x{offset:X} 的 u16 读取越界"))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

pub fn translation_index(files: Vec<TranslationFile>) -> Result<BTreeMap<String, TranslationFile>> {
    let mut output = BTreeMap::new();
    for file in files {
        let key = file.script.to_ascii_uppercase();
        if output.insert(key.clone(), file).is_some() {
            return Err(format!("重复的翻译脚本：{key}"));
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appended_speaker_stays_with_the_following_text() {
        let source = encode_program(&BasicProgram {
            lines: vec![
                BasicLine {
                    number: 175,
                    link: 1,
                    body: vec![
                        b'L', b'A', 0xF1, 0x11, b':', 0x8B, b' ', b'L', b'S', b'P', b' ', 0x89,
                        b' ', 0x0E, 120, 0,
                    ],
                },
                BasicLine {
                    number: 4230,
                    link: 1,
                    body: [
                        b"A$".as_slice(),
                        &[0xF1],
                        b"A$",
                        &[0xF3],
                        b"RUM$",
                        &[0xF3, b'"', 0xA6, b'+', b'W', b'*', b'"'],
                    ]
                    .concat(),
                },
            ],
        })
        .unwrap();
        let mut translation = extract_translation("KAL2 S1", &source).unwrap();
        translation.entries[0].message = "Dad bought it before he died.".into();
        let (rebuilt, _) = inject_translation(
            "KAL2 S1",
            &source,
            &translation,
            &TextDictionary::default(),
            true,
        )
        .unwrap();
        let program = parse_program(&rebuilt).unwrap();
        let body = &program
            .lines
            .iter()
            .find(|line| line.number == 4230)
            .unwrap()
            .body;
        let prefix = [
            b"A$".as_slice(),
            &[0xF1],
            b"A$",
            &[0xF3],
            b"\"+\"",
            &[0xF3],
            b"RUM$",
            &[0xF3, b'"'],
        ]
        .concat();
        assert!(body.starts_with(&prefix));
        assert_ne!(body[prefix.len()], b'+');
        assert!(body.ends_with(b"+W*\""));
    }

    #[test]
    fn distributes_dialogue_across_existing_line_breaks() {
        let raw = b"E\xA6\xA7\xA8+    \xA9\xAA\xAB+    \xAC\xAD\xAE";
        let regions = translation_regions(raw, "message");
        let text = "You finally made it! I am so happy. Nagisa is out shopping now";
        let parts = distribute_translation(raw, &regions, text).unwrap();
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().all(|part| !part.is_empty()));
        assert_eq!(parts.join(" "), text);
        assert!(parts.iter().all(|part| part.len() < 30));
    }

    #[test]
    fn keeps_numbered_menu_labels_on_their_original_rows() {
        let raw = b"+F0 E\xA6\xA7+F1 E\xA8\xA9+F2 E/\xAA/ \xAB";
        let regions = translation_regions(raw, "message");
        let parts = distribute_translation(raw, &regions, "LOOK  TALK  KISS HIM").unwrap();
        assert_eq!(parts, vec!["LOOK", "TALK", "KISS", "HIM"]);
    }

    #[test]
    fn preserves_word_spacing_across_color_changes() {
        let raw = b"E\xA6\xA7F\xA8\xA9E\xAA\xAB";
        let regions = translation_regions(raw, "message");
        let parts = distribute_translation(raw, &regions, "one two three").unwrap();
        assert_eq!(parts.concat(), "one two three");
    }

    #[test]
    fn converts_native_punctuation_to_ascii_glyphs() {
        let dictionary = TextDictionary::default();
        assert_eq!(
            encode_preserved_gap(&[0xA1, 0xA2, 0xA3, 0xA4, 0xA5], "", &dictionary).unwrap(),
            dictionary.encode_text(".\"\",.").unwrap()
        );
    }

    #[test]
    fn keeps_sentence_ending_once_and_preserves_waits() {
        let dictionary = TextDictionary::default();
        let raw = b"E\xA6\xA7\xA8\xA5\xA5\xA5?WW+";
        let regions = translation_regions(raw, "message");
        let output =
            rebuild_message_literal(raw, &regions, Some("Is that the villa?"), &dictionary)
                .unwrap();
        let mut expected = vec![b'E'];
        expected.extend(dictionary.encode_text("Is that the villa?").unwrap());
        expected.extend_from_slice(b"WW+");
        assert_eq!(output, expected);
    }

    #[test]
    fn converts_speaker_quote_without_an_editable_text_region() {
        let dictionary = TextDictionary::default();
        let raw = b"G\xA2";
        let output = rebuild_message_literal(raw, &[], None, &dictionary).unwrap();
        assert_eq!(output, vec![b'G', 0x87]);
    }

    #[test]
    fn graphic_table_has_128_entries() {
        assert_eq!(GRAPHIC_CHARACTERS.chars().count(), 128);
    }

    #[test]
    fn markup_encodes_controls_outside_direct_runs() {
        let encoded =
            encode_message_markup("[color=white]ENTER NAME[nl][speaker=8]OK![wait]").unwrap();
        assert_eq!(encoded[0], b'G');
        assert!(encoded.windows(2).any(|pair| pair == [b'\\', b'8']));
        assert_eq!(encoded.last(), Some(&b'W'));
        assert!(!encoded.contains(&b'!'));
    }

    #[test]
    fn halfwidth_kana_uses_fullwidth_dictionary() {
        assert_eq!(decode_pc88(&[0xB6, 0xDE, 0xAF, 0xBA, 0xB3]), "ガッコウ");
        assert_eq!(decode_pc88(&[0xA2, 0xB8, 0xD0, 0xBA, 0xA3]), "「クミコ」");
    }

    #[test]
    fn translation_region_omits_boundary_controls_and_empty_strings() {
        assert!(translation_regions(b" W+  ", "message").is_empty());
        let raw = [b'E', b' ', 0xA2, 0xB6, 0xDE, 0xAF, 0xBA, 0xB3, 0xA3, b'+'];
        let regions = translation_regions(&raw, "message");
        assert_eq!(regions.len(), 1);
        let region = regions[0];
        assert_eq!(
            &raw[region.start..region.end],
            &[0xB6, 0xDE, 0xAF, 0xBA, 0xB3]
        );
        assert_eq!(
            decode_message_markup(&raw[region.start..region.end]),
            "ガッコウ"
        );
    }

    #[test]
    fn controls_split_message_into_plain_segments() {
        let raw = [
            0xCA, b' ', 0xBA, b'/', 0xB8, 0xD0, 0xBA, b'/', b'+', 0xB6, 0xDE,
        ];
        let regions = translation_regions(&raw, "message");
        let decoded = regions
            .iter()
            .map(|region| decode_pc88(&raw[region.start..region.end]))
            .collect::<Vec<_>>();
        assert_eq!(decoded, ["ハ コ", "クミコ", "ガ"]);
        assert_eq!(decode_translation_text(&raw, &regions), "ハ コクミコ ガ");
        assert_eq!(
            TextDictionary::default()
                .encode_text("KUMIKO")
                .unwrap()
                .len(),
            6
        );
    }

    #[test]
    fn string_scanner_skips_quote_byte_inside_numeric_payload() {
        let body = [
            0x8D, b' ', 0x0E, 0x60, 0x22, b':', b'A', b'T', b'A', b'$', 0xF1, b'"', b'F', b'"',
            0xF3, b'A', b'1', b'$', 0xF3, b'"', b'G', 0xA2, b'"',
        ];
        let spans = string_spans(&body).unwrap();
        assert_eq!(spans.len(), 2);
        assert_eq!(&body[spans[0].start..spans[0].end], b"F");
        assert_eq!(&body[spans[1].start..spans[1].end], &[b'G', 0xA2]);
    }

    #[test]
    fn rebuilds_one_based_line_links_and_drops_unlinked_residue() {
        let source = [
            0x00, 0x00, 10, 0, b'A', 0, // deliberately unlinked header line
            0x0E, 0x00, 20, 0, b'B', b'C', 0, // link -> one-based end marker
            0x00, 0x00, 0xDE, 0xAD, 0xFF, 0xFF, // end marker + live trailer
        ];
        let mut program = parse_program(&source).unwrap();
        assert_eq!(program.lines.len(), 2);
        program.lines[1].body.extend_from_slice(b"DE");

        let rebuilt = encode_program(&program).unwrap();
        assert_eq!(&rebuilt[6..8], &[0x10, 0x00]);
        assert!(rebuilt.ends_with(&[0x00, 0x00]));
        assert!(!rebuilt.windows(2).any(|window| window == [0xDE, 0xAD]));
        validate_line_links("TEST", &rebuilt).unwrap();
    }
}
